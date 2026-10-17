//! Live stick-mash and HID tech-button policy shared by Switch and host tests.
//! HID masks here are hardware button names, separate from logical actions.
//! This module only PRODUCES requests. It does not send hardware packets or
//! directly write game memory; the live adapter and MapperButtonOverlay do that.
//!
//! Flow: parse text settings -> advance scenario/cooldown history once per
//! observed mapper update -> evaluate presets -> choose a mash stick or ZR press.
//! Extend detection in src/live.rs and raw translation in native_input.rs too.
use crate::ledge_trap::LedgeSettings;
use crate::manual_controls::TurboSettings;
use crate::{
    AssistConfig, AssistEngine, AutoMash, AutoTech, ButtonSet, ControllerFrame, FighterKind,
    FighterObservation, InputPermissions, RunContext, Scenario, ScenarioSample,
};

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

#[derive(Clone, Copy, Debug)]
// These are Copy values, so each callback can take a stable settings snapshot.
// Fields named "frames" in the text config count observed mapper updates; their
// relationship to the simulation clock still depends on the live input cadence.
pub struct LiveSettings {
    pub enabled: bool,
    pub auto_mash: bool,
    pub auto_tech: bool,
    /// Enables extra diagnostic messages, not all logging in the plugin.
    pub tech_debug: bool,
    /// Permit temporary releases of the active tool's owned physical buttons.
    pub takeover: bool,
    /// Limits repeated ZR requests; does not change the game's own lockout.
    pub tech_cooldown: u16,
}

// Defaults are used when the SD config is absent. The supplied deployment file
// intentionally enables takeover and extra diagnostic details.
impl Default for LiveSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_mash: true,
            auto_tech: true,
            tech_debug: false,
            takeover: false,
            tech_cooldown: 40,
        }
    }
}

/// Startup configuration shared with the Switch plugin. Unknown keys and
/// invalid values reject the configuration instead of silently enabling tools.
#[derive(Clone, Copy, Debug)]
pub struct LiveConfig {
    pub controls: LiveSettings,
    pub turbo: TurboSettings,
    pub ledge: LedgeSettings,
    /// Switch controller number, not fighter entry ID or mapper input index.
    pub npad: u32,
    /// Linear velocity horizon for collision rays; not a complete physics model.
    pub lookahead: f32,
}
impl Default for LiveConfig {
    fn default() -> Self {
        Self {
            controls: LiveSettings::default(),
            turbo: TurboSettings::default(),
            ledge: LedgeSettings::default(),
            npad: 0,
            lookahead: 3.0,
        }
    }
}
impl LiveConfig {
    // To expose a new setting, add its field/default, a match arm here, range
    // validation below, a deployment config comment, and parser/policy tests.
    pub fn parse(contents: &str) -> Result<Self, String> {
        // Start from Rust defaults, then replace only keys present in the file.
        // `.parse()` infers a type from the destination field (bool/u16/f32).
        // `?` propagates Err immediately: no partially parsed config is installed.
        let mut config = Self::default();
        let mut explicit_tilt_approach = false;
        let mut explicit_grab_approach = false;
        for (number, line) in contents.lines().enumerate() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            // This closure creates a useful line-numbered error. map_err converts
            // standard parse errors into that same project-specific message.
            let invalid = || format!("invalid config line {}: {}", number + 1, line);
            let (key, value) = line.split_once('=').ok_or_else(invalid)?;
            let value = value.trim();
            match key.trim() {
                "enabled" => config.controls.enabled = value.parse().map_err(|_| invalid())?,
                "auto_mash" => config.controls.auto_mash = value.parse().map_err(|_| invalid())?,
                "auto_tech" => config.controls.auto_tech = value.parse().map_err(|_| invalid())?,
                "tech_debug" => {
                    config.controls.tech_debug = value.parse().map_err(|_| invalid())?
                }
                "tech_cooldown_frames" => {
                    config.controls.tech_cooldown = value.parse().map_err(|_| invalid())?
                }
                // Removed button-mash settings are ignored so an older SD
                // config still loads. Analog mashing advances every update.
                "mash_interval_frames" | "alternate_mash" | "mash_buttons_per_frame" => {}
                "takeover" => config.controls.takeover = value.parse().map_err(|_| invalid())?,
                "ledge_trap" => config.ledge.enabled = value.parse().map_err(|_| invalid())?,
                "ledge_trap_range" => {
                    config.ledge.horizontal_range = value.parse().map_err(|_| invalid())?
                }
                "ledge_trap_vertical_range" => {
                    config.ledge.vertical_range = value.parse().map_err(|_| invalid())?
                }
                "ledge_getup_attack" => {
                    config.ledge.getup_attack = match value.to_ascii_lowercase().as_str() {
                        "down_tilt" => crate::ledge_trap::GetupAttack::DownTilt,
                        "forward_tilt" => crate::ledge_trap::GetupAttack::ForwardTilt,
                        _ => return Err(invalid()),
                    };
                }
                "ledge_tilt_range" => {
                    config.ledge.tilt_range = value.parse().map_err(|_| invalid())?
                }
                "ledge_grab_range" => {
                    config.ledge.grab_range = value.parse().map_err(|_| invalid())?
                }
                "ledge_tilt_approach_distance" => {
                    config.ledge.tilt_approach_distance = value.parse().map_err(|_| invalid())?;
                    explicit_tilt_approach = true;
                }
                "ledge_grab_approach_distance" => {
                    config.ledge.grab_approach_distance = value.parse().map_err(|_| invalid())?;
                    explicit_grab_approach = true;
                }
                "ledge_follow_range" => {
                    config.ledge.follow_range = value.parse().map_err(|_| invalid())?
                }
                "ledge_follow_vertical_range" => {
                    config.ledge.follow_vertical_range = value.parse().map_err(|_| invalid())?
                }
                "ledge_auto_position" => {
                    config.ledge.auto_position = value.parse().map_err(|_| invalid())?
                }
                "ledge_stand_distance" => {
                    config.ledge.stand_distance = value.parse().map_err(|_| invalid())?
                }
                "ledge_move_strength" => {
                    config.ledge.move_strength = value.parse().map_err(|_| invalid())?
                }
                "ledge_grab_buffer_frames" => {
                    config.ledge.grab_buffer_frames = value.parse().map_err(|_| invalid())?
                }
                "ledge_throw_outward" => {
                    config.ledge.throw_outward = value.parse().map_err(|_| invalid())?
                }
                // Compatibility only: ignore removed aerial settings in old SD
                // configs so their presence cannot disable mash/tech/ledge tools.
                // Ledge jumps now always use grounded up tilt.
                "ledge_jump_response"
                | "ledge_fair_range"
                | "ledge_fair_vertical_range"
                | "ledge_fair_delay_frames"
                | "ledge_fair_lead_frames"
                | "ledge_fair_jump_squat_buffer"
                | "ledge_fair_buffer_frame" => {}
                "ledge_anti_air_range" => {
                    config.ledge.anti_air_range = value.parse().map_err(|_| invalid())?
                }
                "ledge_anti_air_height" => {
                    config.ledge.anti_air_height = value.parse().map_err(|_| invalid())?
                }
                "ledge_grab_button" => {
                    config.ledge.grab_button = match value.to_ascii_uppercase().as_str() {
                        "R" => crate::native_input::MAPPER_R,
                        "L" => crate::native_input::MAPPER_L,
                        "ZL" => crate::native_input::MAPPER_ZL,
                        _ => return Err(invalid()),
                    }
                }
                "dpad_turbo" => config.turbo.enabled = value.parse().map_err(|_| invalid())?,
                "turbo_phase_frames" => {
                    config.turbo.phase_frames = value.parse().map_err(|_| invalid())?
                }
                "turbo_walk_strength" => {
                    config.turbo.walk_strength = value.parse().map_err(|_| invalid())?
                }
                "npad_id" => config.npad = value.parse().map_err(|_| invalid())?,
                "tech_lookahead_frames" => {
                    config.lookahead = value.parse().map_err(|_| invalid())?
                }
                _ => return Err(invalid()),
            }
        }
        // Validate after parsing so all paths share the same bounds. Floating
        // point parsing accepts NaN/infinity, hence is_finite in addition to range
        // checks. To expose a wider range, verify the consuming policy first.
        if !config.ledge.horizontal_range.is_finite()
            || !(1.0..=20.0).contains(&config.ledge.horizontal_range)
            || !config.ledge.vertical_range.is_finite()
            || !(1.0..=20.0).contains(&config.ledge.vertical_range)
        {
            return Err("ledge trap ranges must be finite and 1..20".into());
        }
        if [config.ledge.tilt_range, config.ledge.grab_range]
            .iter()
            .any(|value| !value.is_finite() || !(1.0..=20.0).contains(value))
        {
            return Err("ledge attack ranges must be finite 1..20".into());
        }
        // Older SD configs may specify a narrow attack range without the new
        // approach keys. Fit omitted defaults inside that range for compatibility.
        if !explicit_tilt_approach {
            config.ledge.tilt_approach_distance = config
                .ledge
                .tilt_approach_distance
                .min((config.ledge.tilt_range - 0.5).max(0.5));
        }
        if !explicit_grab_approach {
            config.ledge.grab_approach_distance = config
                .ledge
                .grab_approach_distance
                .min((config.ledge.grab_range - 0.5).max(0.5));
        }
        // Distances are independent tuning fields, but pursuit must reach a
        // point inside its matching attack gate rather than stop outside it.
        for (name, distance, range) in [
            (
                "ledge_tilt_approach_distance",
                config.ledge.tilt_approach_distance,
                config.ledge.tilt_range,
            ),
            (
                "ledge_grab_approach_distance",
                config.ledge.grab_approach_distance,
                config.ledge.grab_range,
            ),
        ] {
            if !distance.is_finite() || !(0.5..=20.0).contains(&distance) || distance >= range {
                return Err(format!(
                    "{name} must be finite, 0.5..20, and smaller than its attack range"
                ));
            }
        }
        if [
            config.ledge.follow_range,
            config.ledge.follow_vertical_range,
        ]
        .iter()
        .any(|v| !v.is_finite() || !(1.0..=100.0).contains(v))
            || !config.ledge.stand_distance.is_finite()
            || !(2.0..=15.0).contains(&config.ledge.stand_distance)
            || !config.ledge.move_strength.is_finite()
            || !(0.1..=1.0).contains(&config.ledge.move_strength)
            || config.ledge.grab_buffer_frames > 6
        {
            return Err("invalid ledge follow, position, or grab buffer settings".into());
        }
        if !config.ledge.anti_air_range.is_finite()
            || !(1.0..=20.0).contains(&config.ledge.anti_air_range)
            || !config.ledge.anti_air_height.is_finite()
            || !(1.0..=40.0).contains(&config.ledge.anti_air_height)
        {
            return Err("anti-air range must be 1..20 and height 1..40".into());
        }
        if !(1..=10).contains(&config.turbo.phase_frames)
            || !config.turbo.walk_strength.is_finite()
            || !(0.05..=0.6).contains(&config.turbo.walk_strength)
        {
            return Err(
                "turbo_phase_frames must be 1..10; turbo_walk_strength must be 0.05..0.6".into(),
            );
        }
        if config.npad > 7
            || !config.lookahead.is_finite()
            || !(1.0..=5.0).contains(&config.lookahead)
        {
            return Err("npad_id must be 0..7; tech_lookahead_frames must be 1..5".into());
        }
        Ok(config)
    }
}

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
        // The live adapter has already established human control and Kirby
        // identity before calling this policy; these context values reflect that
        // contract rather than discovering those facts from a controller mask.
        let decision = AssistEngine.evaluate(
            config,
            RunContext {
                is_locally_human_controlled: true,
                fighter: FighterKind::Kirby,
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
mod tests {
    use super::*;

    #[test]
    fn direct_config_selects_one_controller_and_takeover_tools() {
        let config = LiveConfig::parse(include_str!(
            "../../../deployment/ultimate/kirby_assist/config.txt"
        ))
        .unwrap();
        assert_eq!(config.npad, 0);
        assert!(config.controls.takeover);
        assert!(config.controls.auto_mash && config.controls.auto_tech);
    }

    #[test]
    fn analog_mash_composes_with_button_takeover_and_restores_fresh_physical_input() {
        use crate::manual_controls::{
            ManualOverlay, TurboOutput, LEFT_STICK_DIRECTIONS, RIGHT_STICK_DIRECTIONS, STICK_CLICKS,
        };
        use crate::native_input::{mapper_buttons, ButtonTrace, MapperButtonOverlay};
        let settings = LiveSettings {
            takeover: true,
            ..LiveSettings::default()
        };
        let mut controls = RawAutoControls::default();
        let mut buttons = MapperButtonOverlay::default();
        let mut manual = ManualOverlay::default();
        let blocked = RIGHT_STICK_DIRECTIONS | STICK_CLICKS;
        let held = mapper_buttons(ASSIST_BUTTONS) | blocked | LEFT_STICK_DIRECTIONS | (1 << 18);
        let human = ButtonTrace {
            held,
            previous: held,
            ..ButtonTrace::default()
        };
        let physical_stick = [0.2, -0.3];
        for tick in 0..12 {
            let requested = controls.update(settings, tick, 7, true, false);
            assert_eq!(
                requested, 0,
                "analog mash must never press an action button"
            );
            assert_eq!(controls.update(settings, tick, 7, true, false), requested);
            let stick = controls.mash_stick().unwrap();
            let assisted = buttons.apply(human, requested, ASSIST_BUTTONS);
            let frame = manual.apply_controls(
                assisted,
                physical_stick,
                TurboOutput {
                    consume_dpad: false,
                    stick: Some(stick),
                },
                0,
                0,
                blocked,
            );
            assert_eq!(frame.stick, stick);
            assert_eq!(
                frame.buttons.held & (mapper_buttons(ASSIST_BUTTONS) | blocked),
                0
            );
            assert_eq!((frame.buttons.held & LEFT_STICK_DIRECTIONS).count_ones(), 1);
            assert_ne!(frame.buttons.held & (1 << 18), 0, "menu is never owned");
        }
        assert_eq!(controls.update(settings, 12, 7, false, false), 0);
        assert_eq!(controls.mash_stick(), None);
        let assisted = buttons.apply(human, 0, 0);
        let frame =
            manual.apply_controls(assisted, physical_stick, TurboOutput::default(), 0, 0, 0);
        assert_eq!(frame.stick, physical_stick);
        assert_eq!(frame.buttons.held, human.held);
        assert_eq!(
            manual
                .apply_controls(human, physical_stick, TurboOutput::default(), 0, 0, 0)
                .buttons,
            human
        );
    }

    #[test]
    fn bundled_config_keeps_analog_mash_and_extended_prediction() {
        let config = LiveConfig::parse(include_str!(
            "../../../deployment/ultimate/kirby_assist/config.txt"
        ))
        .unwrap();
        assert!(config.ledge.enabled);
        assert_eq!(config.ledge.horizontal_range, 12.0);
        assert_eq!(config.ledge.vertical_range, 10.0);
        assert_eq!(config.ledge.tilt_range, 7.0);
        assert_eq!(config.ledge.grab_range, 6.0);
        assert_eq!(config.ledge.follow_range, 40.0);
        assert_eq!(config.ledge.follow_vertical_range, 48.0);
        assert!(config.ledge.auto_position);
        assert_eq!(config.ledge.stand_distance, 9.0);
        assert_eq!(config.ledge.move_strength, 0.65);
        assert_eq!(config.ledge.grab_buffer_frames, 3);
        assert_eq!(config.ledge.grab_button, crate::native_input::MAPPER_R);
        assert_eq!(config.ledge.anti_air_range, 8.0);
        assert_eq!(config.ledge.anti_air_height, 18.0);
        assert!(config.turbo.enabled);
        assert_eq!(config.turbo.phase_frames, 1);
        assert_eq!(config.turbo.walk_strength, 0.25);
        assert_eq!(config.lookahead, 5.0);
    }

    #[test]
    fn analog_mash_to_tech_keeps_the_existing_zr_release_and_cooldown() {
        let settings = LiveSettings {
            takeover: true,
            ..LiveSettings::default()
        };
        let mut controls = RawAutoControls::default();
        let mut overlay = HidButtonOverlay::default();
        for tick in 0..8 {
            assert_eq!(controls.update(settings, tick, 1, true, false), 0);
            assert!(controls.mash_stick().is_some());
        }
        assert_eq!(
            controls.update_with_buttons(settings, 8, 1, false, true, HID_ZR),
            0
        );
        assert_eq!(controls.mash_stick(), None);
        assert_eq!(overlay.apply_owned(9, HID_ZR, 0, HID_ZR, true), 0);
        assert_eq!(controls.update(settings, 9, 1, false, true), HID_ZR);
        assert_eq!(
            overlay.apply_owned(10, HID_ZR, HID_ZR, HID_ZR, true),
            HID_ZR
        );
        assert_eq!(controls.update(settings, 10, 1, false, true), 0);
        assert_eq!(overlay.apply_owned(11, HID_ZR, 0, HID_ZR, true), 0);
    }

    #[test]
    fn one_update_tech_opportunity_is_not_delayed_when_zr_is_released() {
        let settings = LiveSettings {
            takeover: true,
            ..LiveSettings::default()
        };
        let mut controls = RawAutoControls::default();
        assert_eq!(
            controls.update_with_buttons(settings, 0, 1, false, true, 0),
            HID_ZR
        );
        assert_eq!(
            controls.update_with_buttons(settings, 1, 1, false, false, HID_ZR),
            0
        );
        assert_eq!(
            controls.update_with_buttons(settings, 2, 1, false, true, 0),
            0
        );
    }

    #[test]
    fn grounded_getup_move_is_selectable_without_changing_other_responses() {
        let default = LiveConfig::parse("ledge_trap = true").unwrap();
        assert_eq!(
            default.ledge.getup_attack,
            crate::ledge_trap::GetupAttack::DownTilt
        );
        let forward =
            LiveConfig::parse("ledge_trap = true\nledge_getup_attack = FORWARD_TILT").unwrap();
        assert_eq!(
            forward.ledge.getup_attack,
            crate::ledge_trap::GetupAttack::ForwardTilt
        );
        assert_eq!(forward.ledge.anti_air_range, default.ledge.anti_air_range);
        assert_eq!(forward.ledge.grab_button, default.ledge.grab_button);
    }

    #[test]
    fn outward_throw_setting_is_optional_and_bundled_forward_tilt_is_selected() {
        assert!(LiveConfig::default().ledge.throw_outward);
        assert!(
            !LiveConfig::parse("ledge_throw_outward = false")
                .unwrap()
                .ledge
                .throw_outward
        );
        assert!(LiveConfig::parse("ledge_throw_outward = maybe").is_err());
        let supplied = LiveConfig::parse(include_str!(
            "../../../deployment/ultimate/kirby_assist/config.txt"
        ))
        .unwrap();
        assert!(supplied.ledge.throw_outward);
        assert_eq!(
            supplied.ledge.getup_attack,
            crate::ledge_trap::GetupAttack::ForwardTilt
        );
    }

    #[test]
    fn approach_distances_are_validated_without_breaking_older_narrow_ranges() {
        let old = LiveConfig::parse("ledge_tilt_range = 2\nledge_grab_range = 1").unwrap();
        assert_eq!(old.ledge.tilt_approach_distance, 1.5);
        assert_eq!(old.ledge.grab_approach_distance, 0.5);
        let independent =
            LiveConfig::parse("ledge_grab_range = 10\nledge_grab_approach_distance = 4").unwrap();
        assert_eq!(independent.ledge.grab_approach_distance, 4.0);
        for bad in [
            "ledge_tilt_approach_distance = NaN",
            "ledge_grab_approach_distance = 0.4",
            "ledge_grab_range = 5\nledge_grab_approach_distance = 5",
            "ledge_tilt_range = 3\nledge_tilt_approach_distance = 4",
        ] {
            assert!(LiveConfig::parse(bad).is_err(), "{bad}");
        }
        let bundled = LiveConfig::parse(include_str!(
            "../../../deployment/ultimate/kirby_assist/config.txt"
        ))
        .unwrap();
        assert_eq!(bundled.ledge.stand_distance, 9.0);
        assert_eq!(bundled.ledge.tilt_approach_distance, 6.0);
        assert_eq!(bundled.ledge.grab_approach_distance, 5.0);
    }

    #[test]
    fn removed_aerial_settings_do_not_disable_existing_sd_configs() {
        let config = LiveConfig::parse("ledge_trap = true\nledge_grab_button = l\nledge_jump_response = fair\nledge_fair_range = 11.0\nledge_fair_vertical_range = 10.0\nledge_fair_delay_frames = 0.0\nledge_fair_lead_frames = 4.0\nledge_fair_jump_squat_buffer = true\nledge_fair_buffer_frame = 1.0").unwrap();
        assert!(config.ledge.enabled);
        assert_eq!(config.ledge.grab_button, crate::native_input::MAPPER_L);
        assert_eq!(config.ledge.anti_air_range, 8.0);
        assert!(config.controls.auto_mash);
        assert!(config.controls.auto_tech);
    }
    #[test]
    fn old_button_mash_settings_are_ignored_without_disabling_other_tools() {
        let config = LiveConfig::parse("auto_mash = true\nalternate_mash = true\nmash_buttons_per_frame = 4\nmash_interval_frames = 3\ntakeover = true").unwrap();
        let mut controls = RawAutoControls::default();
        for tick in 0..4 {
            assert_eq!(controls.update(config.controls, tick, 1, true, false), 0);
            assert!(controls.mash_stick().is_some());
        }
        assert!(config.controls.auto_tech);
    }

    #[test]
    fn startup_config_handles_comments_toggles_and_rejects_mistakes() {
        let config = LiveConfig::parse(
            "# Pro Controller\nenabled = false\nauto_tech = false\nnpad_id = 2\ntech_lookahead_frames = 4 # prediction\n",
        ).unwrap();
        assert!(!config.controls.enabled);
        assert!(!config.controls.auto_tech);
        assert!(config.controls.auto_mash);
        assert_eq!(config.npad, 2);
        assert_eq!(config.lookahead, 4.0);
        for invalid in [
            "enabled = yes",
            "ledge_trap = yes",
            "ledge_getup_attack = smash",
            "ledge_follow_range = NaN",
            "ledge_follow_vertical_range = 101",
            "ledge_auto_position = yes",
            "ledge_stand_distance = 1",
            "ledge_move_strength = 1.1",
            "ledge_grab_buffer_frames = 7",
            "ledge_grab_button = ZR",
            "ledge_anti_air_range = NaN",
            "ledge_anti_air_height = 41",
            "ledge_grab_button = A",
            "ledge_trap_range = NaN",
            "ledge_tilt_range = NaN",
            "ledge_grab_range = 21",
            "ledge_trap_range = 21",
            "ledge_trap_vertical_range = 0",
            "turbo_phase_frames = 0",
            "turbo_phase_frames = 11",
            "turbo_walk_strength = NaN",
            "turbo_walk_strength = 0.9",
            "npad_id = 8",
            "npad_id = -1",
            "tech_lookahead_frames = NaN",
            "tech_lookahead_frames = 6",
            "auto_tehc = false",
            "enabled",
        ] {
            assert!(LiveConfig::parse(invalid).is_err(), "accepted {invalid}");
        }
    }

    #[test]
    fn analog_cycle_advances_once_per_update_and_restarts_on_exit_or_reassignment() {
        let mut controls = RawAutoControls::default();
        let settings = LiveSettings::default();
        let expected = [[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0], [0.0, -1.0]];
        for tick in 0..12 {
            assert_eq!(controls.update(settings, tick, 10, true, false), 0);
            assert_eq!(controls.mash_stick(), Some(expected[(tick % 4) as usize]));
            controls.update(settings, tick, 10, true, false);
            assert_eq!(controls.mash_stick(), Some(expected[(tick % 4) as usize]));
        }
        controls.update(settings, 12, 10, false, false);
        assert_eq!(controls.mash_stick(), None);
        controls.update(settings, 13, 10, true, false);
        assert_eq!(controls.mash_stick(), Some(expected[0]));
        controls.update(settings, 14, 10, true, false);
        assert_eq!(controls.mash_stick(), Some(expected[1]));
        controls.update(settings, 14, 11, true, false);
        assert_eq!(controls.mash_stick(), Some(expected[0]));
    }

    #[test]
    fn tech_is_one_press_with_cooldown_and_no_timer_bypass() {
        let mut controls = RawAutoControls::default();
        let settings = LiveSettings {
            tech_cooldown: 4,
            ..LiveSettings::default()
        };
        assert_eq!(controls.update(settings, 0, 1, false, true), HID_ZR);
        assert_eq!(controls.update(settings, 1, 1, false, false), 0);
        assert_eq!(controls.update(settings, 2, 1, false, true), 0);
        assert_eq!(controls.update(settings, 3, 1, false, false), 0);
        assert_eq!(controls.update(settings, 4, 1, false, true), HID_ZR);
        assert_eq!(controls.update(settings, 5, 1, false, true), 0);
    }

    #[test]
    fn tech_opportunity_that_starts_during_cooldown_fires_when_ready() {
        let settings = LiveSettings {
            tech_cooldown: 4,
            ..LiveSettings::default()
        };
        let mut controls = RawAutoControls::default();
        assert_eq!(controls.update(settings, 0, 1, false, true), HID_ZR);
        assert_eq!(controls.update(settings, 1, 1, false, false), 0);
        assert_eq!(controls.update(settings, 2, 1, false, true), 0);
        assert_eq!(controls.update(settings, 3, 1, false, true), 0);
        assert_eq!(controls.update(settings, 4, 1, false, true), HID_ZR);
        assert_eq!(controls.update(settings, 4, 1, false, true), HID_ZR);
        for tick in 5..8 {
            assert_eq!(controls.update(settings, tick, 1, false, true), 0);
        }
        assert_eq!(controls.update(settings, 8, 1, false, true), HID_ZR);
    }

    #[test]
    fn disabling_and_input_clock_reset_cancel_old_stick_output() {
        let mut controls = RawAutoControls::default();
        let settings = LiveSettings::default();
        controls.update(settings, 20, 1, true, false);
        assert_eq!(controls.mash_stick(), Some([1.0, 0.0]));
        controls.update(
            LiveSettings {
                enabled: false,
                ..settings
            },
            20,
            1,
            true,
            false,
        );
        assert_eq!(controls.mash_stick(), None);
        controls.update(settings, 20, 1, true, false);
        controls.update(settings, 21, 1, true, false);
        assert_eq!(controls.mash_stick(), Some([0.0, 1.0]));
        controls.update(settings, 0, 1, true, false);
        assert_eq!(controls.mash_stick(), Some([1.0, 0.0]));
    }

    #[test]
    fn per_tool_settings_and_tech_priority_cancel_mash_stick() {
        let mut controls = RawAutoControls::default();
        let settings = LiveSettings {
            auto_tech: false,
            ..LiveSettings::default()
        };
        assert_eq!(controls.update(settings, 0, 1, false, true), 0);
        assert_eq!(controls.mash_stick(), None);
        controls.update(settings, 1, 1, true, false);
        assert_eq!(controls.mash_stick(), Some([1.0, 0.0]));
        controls.update(
            LiveSettings {
                auto_mash: false,
                ..settings
            },
            2,
            1,
            true,
            false,
        );
        assert_eq!(controls.mash_stick(), None);
        let mut controls = RawAutoControls::default();
        assert_eq!(
            controls.update(LiveSettings::default(), 0, 1, true, true),
            HID_ZR
        );
        assert_eq!(controls.mash_stick(), None);
    }

    #[test]
    fn raw_overlay_matches_button_trace_and_never_removes_human_input() {
        let mut overlay = HidButtonOverlay::default();
        let human = (1 << 1) | (1 << 7) | (1 << 63);
        // Equivalent button values to physically holding B+R while pressing A.
        assert_eq!(overlay.apply(1, human, HID_A, true), human | HID_A);
        // A policy change waits for a new hardware sample.
        assert_eq!(overlay.apply(1, human, HID_ZR, true), human | HID_A);
        assert_eq!(overlay.apply(2, human, HID_ZR, true), human | HID_ZR);
        // A release request also waits for the next native sample. Two reads
        // of the same sampling number must not fabricate a button transition.
        assert_eq!(overlay.apply(2, human, 0, true), human | HID_ZR);
        assert_eq!(overlay.apply(3, human, 0, true), human);
        // User-held A remains held during an assist-release gap.
        assert_eq!(overlay.apply(4, human | HID_A, 0, true), human | HID_A);
        assert_eq!(
            overlay.apply(5, human, u64::MAX, true),
            human | ASSIST_BUTTONS
        );
        assert_eq!(
            overlay.apply(6, human | HID_PLUS, HID_A, true),
            human | HID_PLUS
        );
        assert_eq!(overlay.apply(7, human, HID_A, false), human);
    }

    #[test]
    fn instruction_scan_requires_a_single_aligned_match() {
        let pattern = [1, 2, 3, 4];
        assert_eq!(
            unique_instruction_offset(&[0, 0, 0, 0, 1, 2, 3, 4], &pattern),
            Some(4)
        );
        assert_eq!(
            unique_instruction_offset(&[1, 2, 3, 4, 1, 2, 3, 4], &pattern),
            None
        );
        assert_eq!(unique_instruction_offset(&[0, 1, 2, 3, 4], &pattern), None);
        assert_eq!(unique_instruction_offset(&pattern, &[]), None);
        assert_eq!(unique_instruction_offset(&[], &pattern), None);
    }
}
