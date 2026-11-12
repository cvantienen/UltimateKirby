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

mod config;
pub use config::{LiveConfig, LiveSettings};

// These bit positions name Nintendo HID buttons, not Smash's mapper bitfield.
// Example: A is bit 0 here but bit 5 in native_input.rs. Keep the conversion
// explicit rather than copying a u64 mask directly into native Controller memory.
/// `HID_A` is a configuration value used by the Smash runtime.
pub const HID_A: u64 = 1;
/// `HID_B` is a configuration value used by the Smash runtime.
pub const HID_B: u64 = 1 << 1;
/// `HID_X` is a configuration value used by the Smash runtime.
pub const HID_X: u64 = 1 << 2;
/// `HID_Y` is a configuration value used by the Smash runtime.
pub const HID_Y: u64 = 1 << 3;
/// `HID_L` is a configuration value used by the Smash runtime.
pub const HID_L: u64 = 1 << 6;
/// `HID_R` is a configuration value used by the Smash runtime.
pub const HID_R: u64 = 1 << 7;
/// `HID_ZL` is a configuration value used by the Smash runtime.
pub const HID_ZL: u64 = 1 << 8;
/// `HID_ZR` is a configuration value used by the Smash runtime.
pub const HID_ZR: u64 = 1 << 9;
/// `HID_PLUS` is a configuration value used by the Smash runtime.
pub const HID_PLUS: u64 = 1 << 10;
/// `HID_MINUS` is a configuration value used by the Smash runtime.
pub const HID_MINUS: u64 = 1 << 11;
// Limit which bits the request utilities may affect; Plus/Minus are excluded.
/// `ASSIST_BUTTONS` is a configuration value used by the Smash runtime.
pub const ASSIST_BUTTONS: u64 = HID_A | HID_B | HID_X | HID_Y | HID_L | HID_R | HID_ZL | HID_ZR;

// One full-strength cardinal direction per distinct mashable input update.
// A 90-degree turn gives a fresh direction without diagonal overlap or a
// button-mash request. These are normalized left-stick values, not C-stick.
/// `MASH_DIRECTIONS` is a configuration value used by the Smash runtime.
const MASH_DIRECTIONS: [[f32; 2]; 4] = [
    [1.0, 0.0],  // Right
    [0.0, 1.0],  // Up
    [-1.0, 0.0], // Left
    [0.0, -1.0], // Down
];

// Scenario age resets on exit/re-entry. It is separate from the tech cooldown:
// a new collision may begin while the previous request is still cooling down.
/// Runtime Smash data carried by `ScenarioCounter`.
#[derive(Default)]
struct ScenarioCounter {
/// Runtime previous active value used by ScenarioCounter..
    previous_active: bool,
/// current native animation frame.
    frame: u32,
}

impl ScenarioCounter {
    /// Applies the incoming state changes to the current value.
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
///
pub struct RawAutoControls {
    // `Option<T>` explicitly represents absence (None) or a value (Some(T)).
    // A changed identity or backwards clock invalidates this entire session.
/// Runtime identity value used by RawAutoControls..
    identity: Option<u32>,
/// Runtime last tick value used by RawAutoControls..
    last_tick: Option<i64>,
/// Runtime clatter value used by RawAutoControls..
    clatter: ScenarioCounter,
/// Runtime tech value used by RawAutoControls..
    tech: ScenarioCounter,
    // cooldown counts eligible scheduler updates, not wall-clock milliseconds
    // or the game's internal tech lockout. It prevents our repeated ZR attempts.
/// Runtime cooldown value used by RawAutoControls..
    cooldown: u16,
/// Runtime proposed value used by RawAutoControls..
    proposed: u64,
/// Runtime mash stick value used by RawAutoControls..
    mash_stick: Option<[f32; 2]>,
/// Runtime tech needs release value used by RawAutoControls..
    tech_needs_release: bool,
}

impl RawAutoControls {
    /// Cached left-stick proposal from the same update as the button result.
    /// None returns ownership to the fresh physical stick immediately.
    pub fn mash_stick(&self) -> Option<[f32; 2]> {
        self.mash_stick
    }

    /// Returns the remaining tech cooldown in game frames.
    pub fn tech_cooldown_remaining(&self) -> u16 {
        self.cooldown
    }

    /// Restores the state to its initial configuration.
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
    /// Applies the incoming state changes to the current value.
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
            self.cooldown = settings
                .tech_cooldown
                .max(crate::tech_input::MIN_TECH_COOLDOWN_FRAMES);
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
/// Runtime last sample value used by HidButtonOverlay..
    last_sample: Option<i64>,
/// Runtime latched value used by HidButtonOverlay..
    latched: u64,
/// Runtime owned value used by HidButtonOverlay..
    owned: u64,
}

impl HidButtonOverlay {
    /// Applies the supplied configuration or action.
    pub fn apply(&mut self, sample: i64, human: u64, requested: u64, connected: bool) -> u64 {
        self.apply_owned(sample, human, requested, 0, connected)
    }

    /// Applies the owned.
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
///
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

pub mod mapper_hook {
    //! Startup-only recognition of the FIM mapper, including Skyline entry patches.
    //! No fixed game address, pointer dereference, or instruction modification here.

/// `MAPPING_NEEDLE` is a configuration value used by the Smash runtime.
    pub const MAPPING_NEEDLE: &[u8] = &[
        0xff, 0x03, 0x02, 0xd1, 0xf7, 0x23, 0x00, 0xf9, 0xf6, 0x57, 0x05, 0xa9, 0xf4, 0x4f, 0x06,
        0xa9, 0xfd, 0x7b, 0x07, 0xa9, 0xfd, 0xc3, 0x01, 0x91, 0x3f, 0x04, 0x00, 0x31, 0xe0, 0x77,
        0x00, 0x54,
    ];

/// `AARCH64_INSTRUCTION_BYTES` is a configuration value used by the Smash runtime.
    const AARCH64_INSTRUCTION_BYTES: usize = 4;
/// `AARCH64_INSTRUCTION_WORD_BITS` is a configuration value used by the Smash runtime.
    const AARCH64_INSTRUCTION_WORD_BITS: u32 = 32;
/// `PATCHED_ENTRY_PREFIX_BYTES` is a configuration value used by the Smash runtime.
    const PATCHED_ENTRY_PREFIX_BYTES: usize = 5 * AARCH64_INSTRUCTION_BYTES;
/// `ENTRY_INSTRUCTION_0_OFFSET_BYTES` is a configuration value used by the Smash runtime.
    const ENTRY_INSTRUCTION_0_OFFSET_BYTES: usize = 0;
/// `ENTRY_INSTRUCTION_1_OFFSET_BYTES` is a configuration value used by the Smash runtime.
    const ENTRY_INSTRUCTION_1_OFFSET_BYTES: usize = AARCH64_INSTRUCTION_BYTES;
/// `ENTRY_INSTRUCTION_2_OFFSET_BYTES` is a configuration value used by the Smash runtime.
    const ENTRY_INSTRUCTION_2_OFFSET_BYTES: usize = 2 * AARCH64_INSTRUCTION_BYTES;
/// `ENTRY_INSTRUCTION_3_OFFSET_BYTES` is a configuration value used by the Smash runtime.
    const ENTRY_INSTRUCTION_3_OFFSET_BYTES: usize = 3 * AARCH64_INSTRUCTION_BYTES;
/// `ENTRY_INSTRUCTION_4_OFFSET_BYTES` is a configuration value used by the Smash runtime.
    const ENTRY_INSTRUCTION_4_OFFSET_BYTES: usize = 4 * AARCH64_INSTRUCTION_BYTES;
/// `BRANCH_OPCODE_MASK` is a configuration value used by the Smash runtime.
    const BRANCH_OPCODE_MASK: u32 = 0xfc00_0000;
/// `BRANCH_OPCODE` is a configuration value used by the Smash runtime.
    const BRANCH_OPCODE: u32 = 0x1400_0000;
/// `LOAD_X17_LITERAL_OPCODE` is a configuration value used by the Smash runtime.
    const LOAD_X17_LITERAL_OPCODE: u32 = 0x5800_0051;
/// `BRANCH_X17_OPCODE` is a configuration value used by the Smash runtime.
    const BRANCH_X17_OPCODE: u32 = 0xd61f_0220;
/// `NO_OPERATION_OPCODE` is a configuration value used by the Smash runtime.
    const NO_OPERATION_OPCODE: u32 = 0xd503_201f;
/// `TEST_BRANCH_PATCH_WORD` is a configuration value used by the Smash runtime.
    #[cfg(test)]
    const TEST_BRANCH_PATCH_WORD: u32 = 0x1400_0100;
/// `TEST_TARGET_LOW_WORD` is a configuration value used by the Smash runtime.
    #[cfg(test)]
    const TEST_TARGET_LOW_WORD: u32 = 0x1234_5000;
/// `TEST_TARGET_HIGH_WORD` is a configuration value used by the Smash runtime.
    #[cfg(test)]
    const TEST_TARGET_HIGH_WORD: u32 = 0x71;

    /// Reads one little-endian AArch64 instruction at the supplied byte offset.
    fn instruction(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(
            bytes[offset..offset + AARCH64_INSTRUCTION_BYTES]
                .try_into()
                .unwrap(),
        )
    }

    /// Accept only the original entry or Skyline's B / LDR+BR entry replacements.
    /// The original bytes after the replaced region must still match. Reject two
    /// candidates even if one is unpatched: silently choosing one is unsafe.
    pub fn mapper_offset(text: &[u8]) -> Option<usize> {
        if text.len() < MAPPING_NEEDLE.len() {
            return None;
        }
        let mut found = None;
        for offset in (0..=text.len() - MAPPING_NEEDLE.len()).step_by(AARCH64_INSTRUCTION_BYTES) {
            let entry = &text[offset..offset + MAPPING_NEEDLE.len()];
            // Skyline overwrites at most five instructions. Filter by the intact
            // suffix first, then validate the exact layout and remaining prologue.
            if entry[PATCHED_ENTRY_PREFIX_BYTES..] != MAPPING_NEEDLE[PATCHED_ENTRY_PREFIX_BYTES..] {
                continue;
            }
            let original = entry == MAPPING_NEEDLE;
            let branch = instruction(entry, ENTRY_INSTRUCTION_0_OFFSET_BYTES) & BRANCH_OPCODE_MASK
                == BRANCH_OPCODE
                && entry[ENTRY_INSTRUCTION_1_OFFSET_BYTES..]
                    == MAPPING_NEEDLE[ENTRY_INSTRUCTION_1_OFFSET_BYTES..];
            let absolute = instruction(entry, ENTRY_INSTRUCTION_0_OFFSET_BYTES)
                == LOAD_X17_LITERAL_OPCODE
                && instruction(entry, ENTRY_INSTRUCTION_1_OFFSET_BYTES) == BRANCH_X17_OPCODE
                && instruction(entry, ENTRY_INSTRUCTION_2_OFFSET_BYTES) as u64
                    | ((instruction(entry, ENTRY_INSTRUCTION_3_OFFSET_BYTES) as u64)
                        << AARCH64_INSTRUCTION_WORD_BITS)
                    != 0
                && entry[ENTRY_INSTRUCTION_4_OFFSET_BYTES..]
                    == MAPPING_NEEDLE[ENTRY_INSTRUCTION_4_OFFSET_BYTES..];
            let aligned_absolute = instruction(entry, ENTRY_INSTRUCTION_0_OFFSET_BYTES)
                == NO_OPERATION_OPCODE
                && instruction(entry, ENTRY_INSTRUCTION_1_OFFSET_BYTES) == LOAD_X17_LITERAL_OPCODE
                && instruction(entry, ENTRY_INSTRUCTION_2_OFFSET_BYTES) == BRANCH_X17_OPCODE
                && instruction(entry, ENTRY_INSTRUCTION_3_OFFSET_BYTES) as u64
                    | ((instruction(entry, ENTRY_INSTRUCTION_4_OFFSET_BYTES) as u64)
                        << AARCH64_INSTRUCTION_WORD_BITS)
                    != 0;
            if original || branch || absolute || aligned_absolute {
                if found.is_some() {
                    return None;
                }
                found = Some(offset);
            }
        }
        found
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// Builds the patched instruction fixture used by tests.
        fn patched(words: &[u32]) -> Vec<u8> {
            let mut bytes = MAPPING_NEEDLE.to_vec();
            for (i, word) in words.iter().enumerate() {
                bytes[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
            }
            bytes
        }

        /// Regression test for `recognizes_original_and_all_skyline_entry_layouts`.
        #[test]
        fn recognizes_original_and_all_skyline_entry_layouts() {
            for entry in [
                MAPPING_NEEDLE.to_vec(),
                patched(&[TEST_BRANCH_PATCH_WORD]),
                patched(&[
                    LOAD_X17_LITERAL_OPCODE,
                    BRANCH_X17_OPCODE,
                    TEST_TARGET_LOW_WORD,
                    TEST_TARGET_HIGH_WORD,
                ]),
                patched(&[
                    NO_OPERATION_OPCODE,
                    LOAD_X17_LITERAL_OPCODE,
                    BRANCH_X17_OPCODE,
                    TEST_TARGET_LOW_WORD,
                    TEST_TARGET_HIGH_WORD,
                ]),
            ] {
                let mut text = vec![0; 12];
                text.extend(entry);
                assert_eq!(mapper_offset(&text), Some(12));
            }
        }

        /// Regression test for `rejects_ambiguity_damage_null_target_and_unaligned_matches`.
        #[test]
        fn rejects_ambiguity_damage_null_target_and_unaligned_matches() {
            let mut duplicate = MAPPING_NEEDLE.to_vec();
            duplicate.extend(patched(&[TEST_BRANCH_PATCH_WORD]));
            assert_eq!(mapper_offset(&duplicate), None);
            let mut damaged = patched(&[TEST_BRANCH_PATCH_WORD]);
            damaged[24] ^= 1;
            assert_eq!(mapper_offset(&damaged), None);
            assert_eq!(
                mapper_offset(&patched(&[
                    LOAD_X17_LITERAL_OPCODE,
                    BRANCH_X17_OPCODE,
                    0,
                    0,
                ])),
                None
            );
            let mut unaligned = vec![0];
            unaligned.extend(MAPPING_NEEDLE);
            assert_eq!(mapper_offset(&unaligned), None);
            assert_eq!(mapper_offset(&[]), None);
        }
    }
}

// Compiled only by cargo test on the host; these assertions are not live
// Switch logging or gameplay callbacks. Add regressions here for new behavior.
#[cfg(test)]
#[path = "../tests/unit/live_input.rs"]
mod tests;
