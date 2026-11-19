//! Live stick-mash and HID tech-button policy shared by Switch and host tests.
//! HID masks here are hardware button names, separate from logical actions.
//! This module only PRODUCES requests. It does not send hardware packets or
//! directly write game memory; the live adapter and MapperButtonOverlay do that.
//!
//! Flow: parse text settings -> advance scenario/cooldown history once per
//! observed mapper update -> evaluate presets -> choose a mash stick or ZR press.
//! Extend detection in src/live.rs and raw translation in native_input.rs too.
use crate::{
    manual_controls::TurboSettings, settings::AssistSettings, AssistConfig, AssistEngine, AutoMash,
    AutoTech, ButtonSet, ControllerFrame, FighterKind, FighterObservation, InputPermissions,
    RunContext, Scenario, ScenarioSample,
};
use std::str::FromStr;


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

// ---------------------------------------------------------------------------
// Startup configuration: types, defaults, text parsing and validation.
//
// Parsing has no file I/O: src/live.rs reads config.txt once and passes its
// contents here. Start with defaults, apply keys in file order, then validate.
// A bad value returns Err; no partially parsed config reaches the live hooks.
//
// Fighter ledge actions, options and timing are loaded only from sequences.txt.
// This section supplies shared settings and numeric adjustments, not sequences.
// ---------------------------------------------------------------------------


/// Sets the default value for tech cooldown frames.
/// Holds the DEFAULT TECH COOLDOWN FRAMES value.
const DEFAULT_TECH_COOLDOWN_FRAMES: u16 = 40;
/// Sets the default value for collision lookahead frames.
/// Holds the DEFAULT COLLISION LOOKAHEAD FRAMES value.
const DEFAULT_COLLISION_LOOKAHEAD_FRAMES: f32 = 3.0;

#[derive(Clone, Copy, Debug)]
// These are Copy values, so each callback can take a stable settings snapshot.
// Fields named "frames" in the text config count observed mapper updates; their
// relationship to the simulation clock still depends on the live input cadence.
/// Runtime Smash data carried by `LiveSettings`.
pub struct LiveSettings {
    /// Stores whether `LiveSettings` is enabled.
    pub enabled: bool,
    /// Runtime auto mash value used by LiveSettings..
    pub auto_mash: bool,
    /// Runtime auto tech value used by LiveSettings..
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
    /// Returns the default value for this type.
    fn default() -> Self {
        Self {
            enabled: true,
            auto_mash: false,
            auto_tech: true,
            tech_debug: false,
            takeover: false,
            tech_cooldown: DEFAULT_TECH_COOLDOWN_FRAMES,
        }
    }
}

/// Startup configuration shared with the Switch plugin. Unknown keys and
/// invalid values reject the configuration instead of silently enabling tools.
#[derive(Clone, Copy, Debug)]
pub struct LiveConfig {
    /// L-held approximate defense against nearby active fighter attacks.
    pub autopilot_shield: bool,
    /// Runtime down tilt trip smash value used by LiveConfig..
    pub down_tilt_trip_smash: bool,
    /// Runtime kirby fair1 smash value used by LiveConfig..
    pub kirby_fair1_smash: bool,
    /// Runtime auto sdi value used by LiveConfig..
    pub auto_sdi: bool,
    /// Runtime auto di value used by LiveConfig..
    pub auto_di: crate::auto_di::Settings,
    /// Runtime auto spot dodge value used by LiveConfig..
    pub auto_spot_dodge: bool,
    /// Runtime controls value used by LiveConfig..
    pub controls: LiveSettings,
    /// Runtime control layout value used by LiveConfig..
    pub control_layout: crate::native_input::ControlLayout,
    /// Facing-direction creep strength for the selected Turbo t-bag binding.
    pub turbo: TurboSettings,
    /// Runtime assist value used by LiveConfig..
    pub assist: AssistSettings,
    /// Local physical controller number: 0 = Player 1. Its game mapper slot
    /// is discovered dynamically; battle participant position is irrelevant.
    pub player_index: u32,
    /// Linear velocity horizon for collision rays; not a complete physics model.
    pub lookahead: f32,
}

impl Default for LiveConfig {
    /// Returns the default value for this type.
    fn default() -> Self {
        Self {
            autopilot_shield: true,
            down_tilt_trip_smash: true,
            kirby_fair1_smash: true,
            auto_sdi: true,
            auto_di: crate::auto_di::Settings::default(),
            auto_spot_dodge: true,
            controls: LiveSettings::default(),
            control_layout: crate::native_input::ControlLayout::Default,
            turbo: TurboSettings::default(),
            assist: AssistSettings::default(),
            player_index: 0,
            lookahead: DEFAULT_COLLISION_LOOKAHEAD_FRAMES,
        }
    }
}

/// Returns the `invalid_line` result.
/// Gets the invalid line result.
fn invalid_line(line_number: usize, line: &str) -> String {
    format!("invalid config line {line_number}: {line}")
}

/// One `key = value` line. Every malformed value reports the same line-numbered
/// message, so each key only names its destination field.
struct Setting<'a> {
    /// Runtime line number value used by Setting..
    line_number: usize,
    /// Runtime line value used by Setting..
    line: &'a str,
    /// Runtime value value used by Setting..
    value: &'a str,
}

impl Setting<'_> {
    /// Returns the `invalid` result.
    fn invalid(&self) -> String {
        invalid_line(self.line_number, self.line)
    }

    /// The destination type decides whether this is a bool, integer or float.
    fn parse<T: FromStr>(&self) -> Result<T, String> {
        self.value.parse().map_err(|_| self.invalid())
    }

    /// Saves one mapped live-input configuration value.
    fn set<T: FromStr>(&self, field: &mut T) -> Result<(), String> {
        *field = self.parse()?;
        Ok(())
    }

    /// Older SD files carry values that no longer do anything; keep rejecting
    /// malformed ones so a typo is still reported.
    fn accept<T: FromStr>(&self) -> Result<(), String> {
        self.parse::<T>().map(drop)
    }

    /// Returns the `choice` result.
    fn choice<T: Copy>(&self, options: &[(&str, T)], ignore_case: bool) -> Result<T, String> {
        options
            .iter()
            .find(|(name, _)| {
                if ignore_case {
                    name.eq_ignore_ascii_case(self.value)
                } else {
                    *name == self.value
                }
            })
            .map(|&(_, choice)| choice)
            .ok_or_else(|| self.invalid())
    }
}

/// Retired controls are still parsed and validated so older SD config files
/// remain loadable, but they no longer affect live policy.
#[derive(Default)]
struct RetiredSettings {
    /// Runtime roll range value used by RetiredSettings..
    roll_range: Option<f32>,
    /// Outer `None`: key absent. Inner `None`: the literal `auto`.
    roll_distance: Option<Option<f32>>,
    /// Runtime tilt range value used by RetiredSettings..
    tilt_range: Option<f32>,
    /// Runtime grab range value used by RetiredSettings..
    grab_range: Option<f32>,
    /// Runtime anti air range value used by RetiredSettings..
    anti_air_range: Option<f32>,
    /// Runtime anti air height value used by RetiredSettings..
    anti_air_height: Option<f32>,
    /// Runtime turbo phase frames value used by RetiredSettings..
    turbo_phase_frames: Option<u16>,
}

// Validate after parsing so all paths share the same bounds. Floating point
// parsing accepts NaN/infinity, hence is_finite in addition to range checks.
// To expose a wider range, verify the consuming policy first.
/// Determines whether the `finite_in` condition holds.
/// Checks whether finite in.
fn finite_in(value: f32, min: f32, max: f32) -> bool {
    value.is_finite() && (min..=max).contains(&value)
}

/// Determines whether the `retired_in` condition holds.
/// Checks whether retired in.
fn retired_in(value: Option<f32>, min: f32, max: f32) -> bool {
    value.is_none_or(|value| finite_in(value, min, max))
}

impl LiveConfig {
    /// Parses the supplied source into the module’s validated representation.
    pub fn parse(contents: &str) -> Result<Self, String> {
        // Start from Rust defaults, then replace only keys present in the file.
        // `?` propagates Err immediately: no partially parsed config is installed.
        let mut config = Self::default();
        let mut retired = RetiredSettings::default();
        for (index, line) in contents.lines().enumerate() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let line_number = index + 1;
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| invalid_line(line_number, line))?;
            let setting = Setting {
                line_number,
                line,
                value: value.trim(),
            };
            config.apply(&mut retired, key.trim(), &setting)?;
        }
        config.validate(&retired)?;
        Ok(config)
    }

    // To expose a new setting, add its field/default, a match arm here, range
    // validation below, a deployment config comment, and parser/policy tests.
    /// Applies the supplied configuration or action.
    fn apply(
        &mut self,
        retired: &mut RetiredSettings,
        key: &str,
        setting: &Setting,
    ) -> Result<(), String> {
        match key {
            "autopilot_shield" => setting.set(&mut self.autopilot_shield)?,
            "kirby_auto_duck" => setting.accept::<bool>()?,
            "down_tilt_trip_smash" => setting.set(&mut self.down_tilt_trip_smash)?,
            "kirby_fair1_smash" => setting.set(&mut self.kirby_fair1_smash)?,
            "auto_sdi" => setting.set(&mut self.auto_sdi)?,
            "auto_spot_dodge" => setting.set(&mut self.auto_spot_dodge)?,
            "auto_di" => setting.set(&mut self.auto_di.enabled)?,
            "auto_di_mode" => {
                self.auto_di.mode = setting.choice(
                    &[
                        ("adaptive", crate::auto_di::Mode::Adaptive),
                        ("survival", crate::auto_di::Mode::Survival),
                        ("escape", crate::auto_di::Mode::Escape),
                    ],
                    false,
                )?
            }
            "auto_di_survival_speed" => setting.set(&mut self.auto_di.survival_speed)?,
            // Old SD files remain usable; removed combo keys do nothing.
            "combo_enabled"
            | "combo_starter"
            | "combo_max_start_percent"
            | "combo_grab_range"
            | "combo_fair_range" => {}
            "enabled" => setting.set(&mut self.controls.enabled)?,
            "control_layout" => {
                self.control_layout = setting.choice(
                    &[
                        ("default", crate::native_input::ControlLayout::Default),
                        ("brother", crate::native_input::ControlLayout::Brother),
                    ],
                    true,
                )?
            }
            "auto_mash" => setting.set(&mut self.controls.auto_mash)?,
            "auto_tech" => setting.set(&mut self.controls.auto_tech)?,
            "tech_debug" => setting.set(&mut self.controls.tech_debug)?,
            "tech_cooldown_frames" => setting.set(&mut self.controls.tech_cooldown)?,
            // Removed button-mash settings are ignored so an older SD
            // config still loads. Analog mashing advances every update.
            "mash_interval_frames" | "alternate_mash" | "mash_buttons_per_frame" => {}
            "takeover" => setting.set(&mut self.controls.takeover)?,
            // Legacy hard-coded D-pad shortcuts are accepted but never enabled.
            "dpad_b_reverse" | "dpad_slide" => setting.accept::<bool>()?,
            // Validate legacy switches, but only the selected button built-in activates Turbo.
            "r3_turbo" | "l3_turbo" | "dpad_turbo" => setting.accept::<bool>()?,
            // Compatibility only: all ledge move selection/timing lives in sequences.txt.
            "ledge_nair_hit_lead_frames"
            | "ledge_stone_hit_lead_frames"
            | "ledge_stone_startup_frames"
            | "ledge_drop_request_timeout_frames"
            | "ledge_roll_hit_window_frames"
            | "ledge_grab_buffer_frames"
            | "ledge_getup_attack" => {}
            "ledge_trap" => setting.set(&mut self.assist.enabled)?,
            "ledge_drop_defense" => setting.set(&mut self.assist.drop_defense)?,
            "ledge_trap_range" => setting.set(&mut self.assist.horizontal_range)?,
            "ledge_trap_vertical_range" => setting.set(&mut self.assist.vertical_range)?,
            "ledge_tilt_range" => retired.tilt_range = Some(setting.parse()?),
            "ledge_grab_range" => retired.grab_range = Some(setting.parse()?),
            "ledge_follow_range" => setting.set(&mut self.assist.follow_range)?,
            "ledge_follow_vertical_range" => setting.set(&mut self.assist.follow_vertical_range)?,
            "ledge_distance_agnostic" => setting.set(&mut self.assist.distance_agnostic)?,
            "ledge_manual_spacing" => setting.set(&mut self.assist.manual_spacing)?,
            "ledge_walk_strength" => setting.set(&mut self.assist.walk_strength)?,
            "ledge_roll_dash_range" | "ledge_roll_smash_range" => {
                retired.roll_range = Some(setting.parse()?)
            }
            "ledge_auto_position" => setting.set(&mut self.assist.auto_position)?,
            "ledge_stand_distance" => setting.set(&mut self.assist.stand_distance)?,
            "ledge_roll_distance" => {
                retired.roll_distance = Some(if setting.value.eq_ignore_ascii_case("auto") {
                    None
                } else {
                    Some(setting.parse()?)
                })
            }
            "ledge_move_strength" => setting.set(&mut self.assist.move_strength)?,
            "ledge_throw_outward" => setting.set(&mut self.assist.throw_outward)?,
            // Compatibility only: ignore removed aerial settings in old SD
            // configs so their presence cannot disable mash/tech/ledge tools.
            // Ledge jumps use the paired neutral jump+A response.
            "ledge_jump_response"
            | "ledge_fair_range"
            | "ledge_fair_vertical_range"
            | "ledge_fair_delay_frames"
            | "ledge_fair_lead_frames"
            | "ledge_fair_jump_squat_buffer"
            | "ledge_fair_buffer_frame" => {}
            "ledge_anti_air_range" => retired.anti_air_range = Some(setting.parse()?),
            "ledge_anti_air_height" => retired.anti_air_height = Some(setting.parse()?),
            "ledge_grab_button" => {
                self.assist.grab_button = setting.choice(
                    &[
                        ("R", crate::native_input::MAPPER_R),
                        ("L", crate::native_input::MAPPER_L),
                        ("ZL", crate::native_input::MAPPER_ZL),
                    ],
                    true,
                )?
            }
            "turbo_phase_frames" => retired.turbo_phase_frames = Some(setting.parse()?),
            "turbo_walk_strength" => setting.set(&mut self.turbo.walk_strength)?,
            "player_index" | "npad_id" => setting.set(&mut self.player_index)?,
            "tech_lookahead_frames" => setting.set(&mut self.lookahead)?,
            _ => return Err(setting.invalid()),
        }
        Ok(())
    }

    /// Returns the `validate` result.
    fn validate(&self, retired: &RetiredSettings) -> Result<(), String> {
        let assist = &self.assist;
        if !finite_in(assist.walk_strength, 0.1, 0.45) || !retired_in(retired.roll_range, 1.0, 20.0)
        {
            return Err(
                "ledge walk strength must be finite 0.1..0.45; roll smash range 1..20".into(),
            );
        }
        if !finite_in(assist.horizontal_range, 1.0, 20.0)
            || !finite_in(assist.vertical_range, 1.0, 20.0)
        {
            return Err("ledge trap ranges must be finite and 1..20".into());
        }
        if !retired_in(retired.tilt_range, 1.0, 20.0) || !retired_in(retired.grab_range, 1.0, 20.0)
        {
            return Err("ledge attack ranges must be finite 1..20".into());
        }
        if !finite_in(assist.follow_range, 1.0, 100.0)
            || !finite_in(assist.follow_vertical_range, 1.0, 100.0)
            || !finite_in(assist.stand_distance, 2.0, 15.0)
            || !finite_in(assist.move_strength, 0.1, 1.0)
        {
            return Err("invalid ledge follow, position, or grab buffer settings".into());
        }
        if !retired_in(retired.roll_distance.flatten(), 2.0, 100.0) {
            return Err("ledge_roll_distance must be auto or finite 2..100".into());
        }
        if !retired_in(retired.anti_air_range, 1.0, 20.0)
            || !retired_in(retired.anti_air_height, 1.0, 40.0)
        {
            return Err("anti-air range must be 1..20 and height 1..40".into());
        }
        if retired
            .turbo_phase_frames
            .is_some_and(|frames| !(1..=10).contains(&frames))
        {
            return Err("turbo_phase_frames must be 1..10".into());
        }
        if !finite_in(self.turbo.walk_strength, 0.05, 0.6) {
            return Err("turbo_walk_strength must be 0.05..0.6".into());
        }
        if self.player_index > 7 || !finite_in(self.lookahead, 1.0, 5.0) {
            return Err("player_index must be 0..7; tech_lookahead_frames must be 1..5".into());
        }
        if !finite_in(self.auto_di.survival_speed, 0.1, 30.0) {
            return Err("auto_di_survival_speed must be finite and 0.1..30".into());
        }
        Ok(())
    }
}

// Compiled only by cargo test on the host; these assertions are not live
// Switch logging or gameplay callbacks. Add regressions here for new behavior.
#[cfg(test)]
#[path = "../tests/unit/live_input.rs"]
mod tests;
