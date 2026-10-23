//! Live stick-mash and HID tech-button policy shared by Switch and host tests.
//! HID masks here are hardware button names, separate from logical actions.
//! This module only PRODUCES requests. It does not send hardware packets or
//! directly write game memory; the live adapter and MapperButtonOverlay do that.
//!
//! Flow: parse text settings -> advance scenario/cooldown history once per
//! observed mapper update -> evaluate presets -> choose a mash stick or ZR press.
//! Extend detection in src/live.rs and raw translation in native_input.rs too.
use crate::{
    AssistConfig, AssistEngine, AutoMash, AutoTech, ButtonSet, ControllerFrame, FighterKind,
    FighterObservation, InputPermissions, RunContext, Scenario, ScenarioSample,
};

// Keep settings independent of the scheduler while preserving its public API.
mod config;
pub use config::{LiveConfig, LiveSettings};

// These bit positions name Nintendo HID buttons, not Smash's mapper bitfield.
// Example: A is bit 0 here but bit 5 in native_input.rs. Keep the conversion
// explicit rather than copying a u64 mask directly into native Controller memory.
pub const HID_A: u64 = 1;
pub const HID_B: u64 = 1 << 1;
pub const HID_X: u64 = 1 << 2;
pub const HID_Y: u64 = 1 << 3;
pub const HID_L: u64 = 1 << 6;
pub const HID_R: u64 = 1 << 7;
pub const HID_ZL: u64 = 1 << 8;
// Limit which bits the request utilities may affect; Plus/Minus are excluded.
pub const ASSIST_BUTTONS: u64 = HID_A | HID_B | HID_X | HID_Y | HID_L | HID_R | HID_ZL | HID_ZR;

// One full-strength cardinal direction per distinct mashable input update.
// A 90-degree turn gives a fresh direction without diagonal overlap or a
// button-mash request. These are normalized left-stick values, not C-stick.
const MASH_DIRECTIONS: [[f32; 2]; 4] = [
    [1.0, 0.0],  // Right
    [0.0, 1.0],  // Up
    [-1.0, 0.0], // Left
    [0.0, -1.0], // Down
];
pub const HID_ZR: u64 = 1 << 9;
pub const HID_PLUS: u64 = 1 << 10;
pub const HID_MINUS: u64 = 1 << 11;

// Scenario age resets on exit/re-entry. It is separate from the tech cooldown:
// a new collision may begin while the previous request is still cooling down.
#[derive(Default)]
struct ScenarioCounter {
    previous_active: bool,
    frame: u32,
}

impl ScenarioCounter {
    fn update(&mut self, active: bool) -> u32 {
        self.frame = if active && self.previous_active {
            // Saturation stops at u32::MAX instead of wrapping back to zero
            // and accidentally re-triggering a once-only rule after a long hold.
            self.frame.saturating_add(1)
        } else {
            0
        };
        self.previous_active = active;
        self.frame
    }
}

/// One session per controller. `input_tick` must identify the latest native
/// sample consumed by the mapping callback; duplicate callbacks reuse output.
#[derive(Default)]
// This is the stateful part of live policy. The engine itself is stateless.
// Keep one instance per selected controller and reset when object identity or
// input-clock ordering changes; otherwise a respawn could inherit old timing.
pub struct RawAutoControls {
    // `Option<T>` explicitly represents absence (None) or a value (Some(T)).
    // A changed identity or backwards clock invalidates this entire session.
    identity: Option<u32>,
    last_tick: Option<i64>,
    clatter: ScenarioCounter,
    tech: ScenarioCounter,
    // cooldown counts eligible scheduler updates, not wall-clock milliseconds
    // or the game's internal tech lockout. It prevents our repeated ZR attempts.
    cooldown: u16,
    proposed: u64,
    mash_stick: Option<[f32; 2]>,
    tech_needs_release: bool,
}

impl RawAutoControls {
    /// Cached left-stick proposal from the same update as the button result.
    /// None returns ownership to the fresh physical stick immediately.
    pub fn mash_stick(&self) -> Option<[f32; 2]> {
        self.mash_stick
    }

    pub fn tech_cooldown_remaining(&self) -> u16 {
        self.cooldown
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Stop/unknown native timing releases cached pulses without forgetting
    /// cooldowns. Resuming the same frozen animation must not replay a press.
    pub fn pause_inputs(&mut self) {
        self.proposed = 0;
        self.mash_stick = None;
    }

    // Convenience entry for host examples that assume the previous ZR output
    // was released. The live adapter calls update_with_buttons with real history.
    pub fn update(
        &mut self,
        settings: LiveSettings,
        input_tick: i64,
        object_id: u32,
        clatter_active: bool,
        tech_opportunity: bool,
    ) -> u64 {
        self.update_with_buttons(
            settings,
            input_tick,
            object_id,
            clatter_active,
            tech_opportunity,
            0,
        )
    }

    /// `previous_buttons` is the last game-visible output, not a continuously
    /// held physical ZR. Only delay tech when a release is actually needed.
    #[allow(clippy::too_many_arguments)]
    pub fn update_with_buttons(
        &mut self,
        settings: LiveSettings,
        input_tick: i64,
        object_id: u32,
        clatter_active: bool,
        tech_opportunity: bool,
        previous_buttons: u64,
    ) -> u64 {
        if !settings.enabled {
            self.reset();
            return 0;
        }
        // Reset before the duplicate-tick check: the same tick number can belong
        // to a newly spawned fighter and must not reuse the old fighter's input.
        if self.identity != Some(object_id) || self.last_tick.is_some_and(|tick| input_tick < tick)
        {
            self.reset();
            self.identity = Some(object_id);
        }
        // Several native queries can happen inside one input update. Reusing
        // the proposal prevents those queries from advancing the mash pattern
        // or using up cooldown multiple times before input is actually mapped.
        if self.last_tick == Some(input_tick) {
            // Cancellation/readiness changes outrank cached timed output.
            if !clatter_active || !settings.auto_mash {
                self.mash_stick = None;
            }
            if !tech_opportunity || !settings.auto_tech {
                self.proposed = 0;
            }
            return self.proposed;
        }
        self.last_tick = Some(input_tick);
        self.cooldown = self.cooldown.saturating_sub(1);
        let mash_frame = self.clatter.update(clatter_active);
        // Arm from the first READY opportunity, not the first collision seen
        // during cooldown. Otherwise an opportunity that starts while blocked
        // can never fire, because its once-only frame has already passed.
        let tech_frame = self
            .tech
            .update(tech_opportunity && settings.auto_tech && self.cooldown == 0);
        // ZR already visible? With takeover, first release it for an update,
        // then request a fresh press. Without takeover a human-held shield is
        // preserved, so the policy cannot force that release edge.
        if tech_frame == 0 {
            self.tech_needs_release = settings.takeover && previous_buttons & HID_ZR != 0;
        }
        // Reuse the engine's scenario priority and existing tech timing. The
        // generic mash preset selects Clatter; live translation below replaces
        // its teaching-example button recipe with a rotating left-stick input.
        let mut config = AssistConfig {
            enabled: true,
            ..AssistConfig::default()
        };
        let mut mash = AutoMash::default().rule();
        mash.enabled = settings.auto_mash;
        // Clatter must be selected on EVERY update for the analog cycle. This
        // intentionally differs from AutoMash's teaching button pulse interval.
        mash.timing = crate::RuleTiming::pulse_every(1);
        let mut tech = AutoTech::default().rule();
        if self.tech_needs_release {
            tech.timing = crate::RuleTiming::after(1);
        }
        tech.enabled = settings.auto_tech && self.cooldown == 0;
        config.set_rule(mash);
        config.set_rule(tech);
        let observation = FighterObservation {
            active: [
                if tech_opportunity {
                    ScenarioSample::new(Scenario::Tech, tech_frame)
                } else {
                    ScenarioSample::EMPTY
                },
                if clatter_active {
                    ScenarioSample::new(Scenario::Clatter, mash_frame)
                } else {
                    ScenarioSample::EMPTY
                },
                ScenarioSample::EMPTY,
            ],
        };
        // The live adapter has established local human input ownership. Raw
        // controls apply to every fighter; no character identity is required.
        let decision = AssistEngine.evaluate(
            config,
            RunContext {
                is_locally_human_controlled: true,
                fighter: FighterKind::Unknown,
            },
            observation,
            ControllerFrame::default(),
            InputPermissions::default(),
        );
        // Only tech emits a button. Mash emits a cached full-strength stick
        // direction; neither the face buttons nor C-stick automate mashing.
        self.proposed = 0;
        self.mash_stick = if matches!(
            decision.status,
            crate::DecisionStatus::ProposedController(Scenario::Clatter)
        ) {
            // % 4 cycles indices 0,1,2,3. `as usize` supplies the integer type
            // Rust uses for indexing arrays. One direction is presented per
            // distinct input update, not several sequential inputs in one frame.
            Some(MASH_DIRECTIONS[(mash_frame % 4) as usize])
        } else {
            None
        };
        if decision.recipe.buttons_to_add.contains(ButtonSet::SHIELD) {
            self.proposed |= HID_ZR;
            self.cooldown = settings.tech_cooldown.max(2);
        }
        self.proposed
    }
}

/// Legacy HID-sample utility retained for host tests/alternate adapters.
/// The CURRENT Switch plugin uses MapperButtonOverlay from native_input.rs;
/// changing this helper alone will not change the installed plugin's output.
/// Keeps repeated reads of one native sample consistent. Only changes button
/// bits; the live hook's separate overlay handles sticks and leaves connection
/// flags and the sample ID intact.
#[derive(Default)]
pub struct HidButtonOverlay {
    last_sample: Option<i64>,
    latched: u64,
    owned: u64,
}

impl HidButtonOverlay {
    pub fn apply(&mut self, sample: i64, human: u64, requested: u64, connected: bool) -> u64 {
        self.apply_owned(sample, human, requested, 0, connected)
    }

    pub fn apply_owned(
        &mut self,
        sample: i64,
        human: u64,
        requested: u64,
        owned: u64,
        connected: bool,
    ) -> u64 {
        if !connected || human & (HID_PLUS | HID_MINUS) != 0 {
            self.last_sample = None;
            self.latched = 0;
            self.owned = 0;
            return human;
        }
        if self.last_sample != Some(sample) {
            self.last_sample = Some(sample);
            self.latched = requested & ASSIST_BUTTONS;
            self.owned = owned & ASSIST_BUTTONS;
        }
        (human & !self.owned) | self.latched
    }
}

/// Returns the unique aligned instruction-pattern match, or no supported hook.
/// This deliberately rejects ambiguous matches instead of choosing an address.
// Scan in 4-byte steps because AArch64 instructions are 4-byte aligned. Returning
// None on two matches is deliberate: choosing an arbitrary match could hook a
// different function and corrupt game memory. Never add a guessed fallback.
pub fn unique_instruction_offset(text: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > text.len() {
        return None;
    }
    let mut found = None;
    for offset in (0..=text.len() - needle.len()).step_by(4) {
        if &text[offset..offset + needle.len()] == needle {
            if found.is_some() {
                return None;
            }
            found = Some(offset);
        }
    }
    found
}

// Compiled only by cargo test on the host; these assertions are not live
// Switch logging or gameplay callbacks. Add regressions here for new behavior.
#[cfg(test)]
mod tests;

#[cfg(test)]
mod combat_config_tests;

#[cfg(test)]
mod roll_hit_window_tests;
