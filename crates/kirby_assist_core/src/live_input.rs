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
mod config {
    //! Startup config types, defaults, text parsing and validation.
    //!
    //! Parsing has no file I/O: src/live.rs reads config.txt once and passes its
    //! contents here. Start with defaults, apply keys in file order, then validate.
    //! A bad value returns Err; no partially parsed config reaches the live hooks.
    //!
    //! Fighter ledge actions, options and timing are loaded only from sequences.txt.
    //! This parser supplies shared settings and numeric adjustments, not sequences.
    use crate::manual_controls::TurboSettings;
    use crate::settings::AssistSettings;

    pub const DEFAULT_TECH_COOLDOWN_FRAMES: u16 = 40;
    pub const DEFAULT_COLLISION_LOOKAHEAD_FRAMES: f32 = 3.0;

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
        pub down_tilt_trip_smash: bool,
        pub kirby_fair1_smash: bool,
        pub auto_sdi: bool,
        pub auto_di: crate::auto_di::Settings,
        pub auto_spot_dodge: bool,
        pub controls: LiveSettings,
        pub control_layout: crate::native_input::ControlLayout,
        /// Facing-direction creep strength for the selected Turbo t-bag binding.
        pub turbo: TurboSettings,
        pub assist: AssistSettings,
        /// Local physical controller number: 0 = Player 1. Its game mapper slot
        /// is discovered dynamically; battle participant position is irrelevant.
        pub player_index: u32,
        /// Linear velocity horizon for collision rays; not a complete physics model.
        pub lookahead: f32,
    }
    impl Default for LiveConfig {
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
    impl LiveConfig {
        // To expose a new setting, add its field/default, a match arm here, range
        // validation below, a deployment config comment, and parser/policy tests.
        pub fn parse(contents: &str) -> Result<Self, String> {
            // Start from Rust defaults, then replace only keys present in the file.
            // `.parse()` infers a type from the destination field (bool/u16/f32).
            // `?` propagates Err immediately: no partially parsed config is installed.
            let mut config = Self::default();
            // These retired controls are still parsed and validated so older SD
            // config files remain loadable, but they no longer affect live policy.
            let mut legacy_roll_range = None;
            let mut legacy_roll_distance = None;
            let mut legacy_tilt_range = None;
            let mut legacy_grab_range = None;
            let mut legacy_anti_air_range = None;
            let mut legacy_anti_air_height = None;
            let mut legacy_turbo_phase_frames = None;
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
                    "autopilot_shield" => {
                        config.autopilot_shield = value.parse().map_err(|_| invalid())?
                    }
                    // Accept and validate retired settings in older SD files.
                    "kirby_auto_duck" => {
                        let _: bool = value.parse().map_err(|_| invalid())?;
                    }
                    "down_tilt_trip_smash" => {
                        config.down_tilt_trip_smash = value.parse().map_err(|_| invalid())?
                    }
                    "kirby_fair1_smash" => {
                        config.kirby_fair1_smash = value.parse().map_err(|_| invalid())?
                    }
                    "auto_sdi" => config.auto_sdi = value.parse().map_err(|_| invalid())?,
                    "auto_spot_dodge" => {
                        config.auto_spot_dodge = value.parse().map_err(|_| invalid())?
                    }
                    "auto_di" => config.auto_di.enabled = value.parse().map_err(|_| invalid())?,
                    "auto_di_mode" => {
                        config.auto_di.mode = match value {
                            "adaptive" => crate::auto_di::Mode::Adaptive,
                            "survival" => crate::auto_di::Mode::Survival,
                            "escape" => crate::auto_di::Mode::Escape,
                            _ => return Err(invalid()),
                        }
                    }
                    "auto_di_survival_speed" => {
                        config.auto_di.survival_speed = value.parse().map_err(|_| invalid())?
                    }
                    // Old SD files remain usable; removed combo keys do nothing.
                    "combo_enabled"
                    | "combo_starter"
                    | "combo_max_start_percent"
                    | "combo_grab_range"
                    | "combo_fair_range" => {}
                    "enabled" => config.controls.enabled = value.parse().map_err(|_| invalid())?,
                    "control_layout" => {
                        config.control_layout = match value.to_ascii_lowercase().as_str() {
                            "default" => crate::native_input::ControlLayout::Default,
                            "brother" => crate::native_input::ControlLayout::Brother,
                            _ => return Err(invalid()),
                        }
                    }
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
                    // Legacy hard-coded D-pad shortcuts are accepted but never enabled.
                    "dpad_b_reverse" | "dpad_slide" => {
                        value.parse::<bool>().map_err(|_| invalid())?;
                    }
                    // Validate legacy switches, but only the selected button built-in activates Turbo.
                    "r3_turbo" | "l3_turbo" | "dpad_turbo" => {
                        value.parse::<bool>().map_err(|_| invalid())?;
                    }
                    // Compatibility only: all ledge move selection/timing lives in sequences.txt.
                    "ledge_nair_hit_lead_frames"
                    | "ledge_stone_hit_lead_frames"
                    | "ledge_stone_startup_frames"
                    | "ledge_drop_request_timeout_frames"
                    | "ledge_roll_hit_window_frames"
                    | "ledge_grab_buffer_frames"
                    | "ledge_getup_attack" => {}
                    "ledge_trap" => config.assist.enabled = value.parse().map_err(|_| invalid())?,

                    "ledge_drop_defense" => {
                        config.assist.drop_defense = value.parse().map_err(|_| invalid())?
                    }
                    "ledge_trap_range" => {
                        config.assist.horizontal_range = value.parse().map_err(|_| invalid())?
                    }
                    "ledge_trap_vertical_range" => {
                        config.assist.vertical_range = value.parse().map_err(|_| invalid())?
                    }

                    "ledge_tilt_range" => {
                        legacy_tilt_range = Some(value.parse::<f32>().map_err(|_| invalid())?)
                    }
                    "ledge_grab_range" => {
                        legacy_grab_range = Some(value.parse::<f32>().map_err(|_| invalid())?)
                    }
                    "ledge_follow_range" => {
                        config.assist.follow_range = value.parse().map_err(|_| invalid())?
                    }
                    "ledge_follow_vertical_range" => {
                        config.assist.follow_vertical_range = value.parse().map_err(|_| invalid())?
                    }
                    "ledge_distance_agnostic" => {
                        config.assist.distance_agnostic = value.parse().map_err(|_| invalid())?
                    }
                    "ledge_manual_spacing" => {
                        config.assist.manual_spacing = value.parse().map_err(|_| invalid())?
                    }
                    "ledge_walk_strength" => {
                        config.assist.walk_strength = value.parse().map_err(|_| invalid())?
                    }
                    "ledge_roll_dash_range" | "ledge_roll_smash_range" => {
                        legacy_roll_range = Some(value.parse::<f32>().map_err(|_| invalid())?)
                    }

                    "ledge_auto_position" => {
                        config.assist.auto_position = value.parse().map_err(|_| invalid())?
                    }
                    "ledge_stand_distance" => {
                        config.assist.stand_distance = value.parse().map_err(|_| invalid())?
                    }
                    "ledge_roll_distance" => {
                        legacy_roll_distance = Some(if value.eq_ignore_ascii_case("auto") {
                            None
                        } else {
                            Some(value.parse::<f32>().map_err(|_| invalid())?)
                        });
                    }
                    "ledge_move_strength" => {
                        config.assist.move_strength = value.parse().map_err(|_| invalid())?
                    }

                    "ledge_throw_outward" => {
                        config.assist.throw_outward = value.parse().map_err(|_| invalid())?
                    }
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
                    "ledge_anti_air_range" => {
                        legacy_anti_air_range = Some(value.parse::<f32>().map_err(|_| invalid())?)
                    }
                    "ledge_anti_air_height" => {
                        legacy_anti_air_height = Some(value.parse::<f32>().map_err(|_| invalid())?)
                    }
                    "ledge_grab_button" => {
                        config.assist.grab_button = match value.to_ascii_uppercase().as_str() {
                            "R" => crate::native_input::MAPPER_R,
                            "L" => crate::native_input::MAPPER_L,
                            "ZL" => crate::native_input::MAPPER_ZL,
                            _ => return Err(invalid()),
                        }
                    }
                    "turbo_phase_frames" => {
                        legacy_turbo_phase_frames = Some(value.parse::<u16>().map_err(|_| invalid())?)
                    }
                    "turbo_walk_strength" => {
                        config.turbo.walk_strength = value.parse().map_err(|_| invalid())?
                    }
                    "player_index" | "npad_id" => {
                        config.player_index = value.parse().map_err(|_| invalid())?
                    }
                    "tech_lookahead_frames" => {
                        config.lookahead = value.parse().map_err(|_| invalid())?
                    }
                    _ => return Err(invalid()),
                }
            }

            if !config.assist.walk_strength.is_finite()
                || !(0.1..=0.45).contains(&config.assist.walk_strength)
                || legacy_roll_range
                    .is_some_and(|range| !range.is_finite() || !(1.0..=20.0).contains(&range))
            {
                return Err(
                    "ledge walk strength must be finite 0.1..0.45; roll smash range 1..20".into(),
                );
            }
            // Validate after parsing so all paths share the same bounds. Floating
            // point parsing accepts NaN/infinity, hence is_finite in addition to range
            // checks. To expose a wider range, verify the consuming policy first.
            if !config.assist.horizontal_range.is_finite()
                || !(1.0..=20.0).contains(&config.assist.horizontal_range)
                || !config.assist.vertical_range.is_finite()
                || !(1.0..=20.0).contains(&config.assist.vertical_range)
            {
                return Err("ledge trap ranges must be finite and 1..20".into());
            }
            if [legacy_tilt_range, legacy_grab_range]
                .iter()
                .flatten()
                .any(|value| !value.is_finite() || !(1.0..=20.0).contains(value))
            {
                return Err("ledge attack ranges must be finite 1..20".into());
            }
            if [
                config.assist.follow_range,
                config.assist.follow_vertical_range,
            ]
            .iter()
            .any(|v| !v.is_finite() || !(1.0..=100.0).contains(v))
                || !config.assist.stand_distance.is_finite()
                || !(2.0..=15.0).contains(&config.assist.stand_distance)
                || !config.assist.move_strength.is_finite()
                || !(0.1..=1.0).contains(&config.assist.move_strength)
            {
                return Err("invalid ledge follow, position, or grab buffer settings".into());
            }
            if legacy_roll_distance.is_some_and(|distance| {
                distance
                    .is_some_and(|distance| !distance.is_finite() || !(2.0..=100.0).contains(&distance))
            }) {
                return Err("ledge_roll_distance must be auto or finite 2..100".into());
            }
            if legacy_anti_air_range
                .is_some_and(|range| !range.is_finite() || !(1.0..=20.0).contains(&range))
                || legacy_anti_air_height
                    .is_some_and(|height| !height.is_finite() || !(1.0..=40.0).contains(&height))
            {
                return Err("anti-air range must be 1..20 and height 1..40".into());
            }
            if legacy_turbo_phase_frames.is_some_and(|frames| !(1..=10).contains(&frames)) {
                return Err("turbo_phase_frames must be 1..10".into());
            }
            if !config.turbo.walk_strength.is_finite()
                || !(0.05..=0.6).contains(&config.turbo.walk_strength)
            {
                return Err("turbo_walk_strength must be 0.05..0.6".into());
            }
            if config.player_index > 7
                || !config.lookahead.is_finite()
                || !(1.0..=5.0).contains(&config.lookahead)
            {
                return Err("player_index must be 0..7; tech_lookahead_frames must be 1..5".into());
            }
            if !config.auto_di.survival_speed.is_finite()
                || !(0.1..=30.0).contains(&config.auto_di.survival_speed)
            {
                return Err("auto_di_survival_speed must be finite and 0.1..30".into());
            }
            Ok(config)
        }
    }
}
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

pub mod mapper_hook {
    //! Startup-only recognition of the FIM mapper, including Skyline entry patches.
    //! No fixed game address, pointer dereference, or instruction modification here.

    pub const MAPPING_NEEDLE: &[u8] = &[
        0xff, 0x03, 0x02, 0xd1, 0xf7, 0x23, 0x00, 0xf9, 0xf6, 0x57, 0x05, 0xa9, 0xf4, 0x4f, 0x06, 0xa9,
        0xfd, 0x7b, 0x07, 0xa9, 0xfd, 0xc3, 0x01, 0x91, 0x3f, 0x04, 0x00, 0x31, 0xe0, 0x77, 0x00, 0x54,
    ];

    const AARCH64_INSTRUCTION_BYTES: usize = 4;
    const AARCH64_INSTRUCTION_WORD_BITS: u32 = 32;
    const PATCHED_ENTRY_PREFIX_BYTES: usize = 5 * AARCH64_INSTRUCTION_BYTES;
    const ENTRY_INSTRUCTION_0_OFFSET_BYTES: usize = 0;
    const ENTRY_INSTRUCTION_1_OFFSET_BYTES: usize = AARCH64_INSTRUCTION_BYTES;
    const ENTRY_INSTRUCTION_2_OFFSET_BYTES: usize = 2 * AARCH64_INSTRUCTION_BYTES;
    const ENTRY_INSTRUCTION_3_OFFSET_BYTES: usize = 3 * AARCH64_INSTRUCTION_BYTES;
    const ENTRY_INSTRUCTION_4_OFFSET_BYTES: usize = 4 * AARCH64_INSTRUCTION_BYTES;
    const BRANCH_OPCODE_MASK: u32 = 0xfc00_0000;
    const BRANCH_OPCODE: u32 = 0x1400_0000;
    const LOAD_X17_LITERAL_OPCODE: u32 = 0x5800_0051;
    const BRANCH_X17_OPCODE: u32 = 0xd61f_0220;
    const NO_OPERATION_OPCODE: u32 = 0xd503_201f;
    #[cfg(test)]
    const TEST_BRANCH_PATCH_WORD: u32 = 0x1400_0100;
    #[cfg(test)]
    const TEST_TARGET_LOW_WORD: u32 = 0x1234_5000;
    #[cfg(test)]
    const TEST_TARGET_HIGH_WORD: u32 = 0x71;

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

        fn patched(words: &[u32]) -> Vec<u8> {
            let mut bytes = MAPPING_NEEDLE.to_vec();
            for (i, word) in words.iter().enumerate() {
                bytes[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
            }
            bytes
        }

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
