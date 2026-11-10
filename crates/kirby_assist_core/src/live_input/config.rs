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
use std::str::FromStr;

/// Sets the default value for tech cooldown frames.
///
/// Holds the DEFAULT TECH COOLDOWN FRAMES value.
const DEFAULT_TECH_COOLDOWN_FRAMES: u16 = 40;
/// Sets the default value for collision lookahead frames.
///
/// Holds the DEFAULT COLLISION LOOKAHEAD FRAMES value.
const DEFAULT_COLLISION_LOOKAHEAD_FRAMES: f32 = 3.0;

#[derive(Clone, Copy, Debug)]
// These are Copy values, so each callback can take a stable settings snapshot.
// Fields named "frames" in the text config count observed mapper updates; their
// relationship to the simulation clock still depends on the live input cadence.
/// Defines the `LiveSettings` data structure.
///
/// Holds the data for LiveSettings.
pub struct LiveSettings {
    /// Stores whether `LiveSettings` is enabled.
    ///
    ///
    ///  Turns assistance on or off.
    pub enabled: bool,
    /// Stores the `auto_mash` value in `LiveSettings` as `bool`.
    ///
    ///
    ///  Holds the auto mash value used by LiveSettings.
    pub auto_mash: bool,
    /// Stores the `auto_tech` value in `LiveSettings` as `bool`.
    ///
    ///
    ///  Holds the auto tech value used by LiveSettings.
    pub auto_tech: bool,
    /// Enables extra diagnostic messages, not all logging in the plugin.
    ///
    ///  Enables extra diagnostic messages, not all logging in the plugin.
    pub tech_debug: bool,
    /// Permit temporary releases of the active tool's owned physical buttons.
    ///
    ///  Permit temporary releases of the active tool's owned physical buttons.
    pub takeover: bool,
    /// Limits repeated ZR requests; does not change the game's own lockout.
    ///
    ///  Limits repeated ZR requests; does not change the game's own lockout.
    pub tech_cooldown: u16,
}

// Defaults are used when the SD config is absent. The supplied deployment file
// intentionally enables takeover and extra diagnostic details.
impl Default for LiveSettings {
    /// Returns the default value for this type.
    ///
    ///
    ///  Returns the default value for this type.
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
///
///  Startup configuration shared with the Switch plugin. Unknown keys and invalid values reject the configuration instead of silently enabling tools.
#[derive(Clone, Copy, Debug)]
pub struct LiveConfig {
    /// L-held approximate defense against nearby active fighter attacks.
    ///
    ///  L-held approximate defense against nearby active fighter attacks.
    pub autopilot_shield: bool,
    /// Stores the `down_tilt_trip_smash` value in `LiveConfig` as `bool`.
    ///
    ///
    ///  Holds the down tilt trip smash value used by LiveConfig.
    pub down_tilt_trip_smash: bool,
    /// Stores the `kirby_fair1_smash` value in `LiveConfig` as `bool`.
    ///
    ///
    ///  Holds the kirby fair1 smash value used by LiveConfig.
    pub kirby_fair1_smash: bool,
    /// Stores the `auto_sdi` value in `LiveConfig` as `bool`.
    ///
    ///
    ///  Holds the auto sdi value used by LiveConfig.
    pub auto_sdi: bool,
    /// Stores the `auto_di` value in `LiveConfig` as `crate::auto_di::Settings`.
    ///
    ///
    ///  Holds the auto di value used by LiveConfig.
    pub auto_di: crate::auto_di::Settings,
    /// Stores the `auto_spot_dodge` value in `LiveConfig` as `bool`.
    ///
    ///
    ///  Holds the auto spot dodge value used by LiveConfig.
    pub auto_spot_dodge: bool,
    /// Stores the `controls` value in `LiveConfig` as `LiveSettings`.
    ///
    ///
    ///  Holds the controls value used by LiveConfig.
    pub controls: LiveSettings,
    /// Stores the `control_layout` value in `LiveConfig` as `crate::native_input::ControlLayout`.
    ///
    ///
    ///  Holds the control layout value used by LiveConfig.
    pub control_layout: crate::native_input::ControlLayout,
    /// Facing-direction creep strength for the selected Turbo t-bag binding.
    ///
    ///  Facing-direction creep strength for the selected Turbo t-bag binding.
    pub turbo: TurboSettings,
    /// Stores the `assist` value in `LiveConfig` as `AssistSettings`.
    ///
    ///
    ///  Holds the assist value used by LiveConfig.
    pub assist: AssistSettings,
    /// Local physical controller number: 0 = Player 1. Its game mapper slot
    /// is discovered dynamically; battle participant position is irrelevant.
    ///
    ///  Local physical controller number: 0 = Player 1. Its game mapper slot is discovered dynamically; battle participant position is irrelevant.
    pub player_index: u32,
    /// Linear velocity horizon for collision rays; not a complete physics model.
    ///
    ///  Linear velocity horizon for collision rays; not a complete physics model.
    pub lookahead: f32,
}

impl Default for LiveConfig {
    /// Returns the default value for this type.
    ///
    ///
    ///  Returns the default value for this type.
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
///
/// Gets the invalid line result.
fn invalid_line(line_number: usize, line: &str) -> String {
    format!("invalid config line {line_number}: {line}")
}

/// One `key = value` line. Every malformed value reports the same line-numbered
/// message, so each key only names its destination field.
///
///  One key = value line. Every malformed value reports the same line-numbered message, so each key only names its destination field.
struct Setting<'a> {
    /// Stores the `line_number` value in `Setting` as `usize`.
    ///
    ///
    ///  Holds the line number value used by Setting.
    line_number: usize,
    /// Stores the `line` value in `Setting` as `&'a str`.
    ///
    ///
    ///  Holds the line value used by Setting.
    line: &'a str,
    /// Stores the `value` value in `Setting` as `&'a str`.
    ///
    ///
    ///  Holds the value value used by Setting.
    value: &'a str,
}

impl Setting<'_> {
    /// Returns the `invalid` result.
    ///
    ///
    ///  Gets the invalid result.
    fn invalid(&self) -> String {
        invalid_line(self.line_number, self.line)
    }

    /// The destination type decides whether this is a bool, integer or float.
    ///
    ///  The destination type decides whether this is a bool, integer or float.
    fn parse<T: FromStr>(&self) -> Result<T, String> {
        self.value.parse().map_err(|_| self.invalid())
    }

    /// Stores the supplied value.
    ///
    ///
    ///  Sets the value.
    fn set<T: FromStr>(&self, field: &mut T) -> Result<(), String> {
        *field = self.parse()?;
        Ok(())
    }

    /// Older SD files carry values that no longer do anything; keep rejecting
    /// malformed ones so a typo is still reported.
    ///
    ///  Older SD files carry values that no longer do anything; keep rejecting malformed ones so a typo is still reported.
    fn accept<T: FromStr>(&self) -> Result<(), String> {
        self.parse::<T>().map(drop)
    }

    /// Returns the `choice` result.
    ///
    ///
    ///  Gets the choice result.
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
///
///  Retired controls are still parsed and validated so older SD config files remain loadable, but they no longer affect live policy.
#[derive(Default)]
struct RetiredSettings {
    /// Stores the `roll_range` value in `RetiredSettings` as `Option<f32>`.
    ///
    ///
    ///  Optionally stores the roll range for RetiredSettings; it may be absent.
    roll_range: Option<f32>,
    /// Outer `None`: key absent. Inner `None`: the literal `auto`.
    ///
    ///  Outer None: key absent. Inner None: the literal auto.
    roll_distance: Option<Option<f32>>,
    /// Stores the `tilt_range` value in `RetiredSettings` as `Option<f32>`.
    ///
    ///
    ///  Optionally stores the tilt range for RetiredSettings; it may be absent.
    tilt_range: Option<f32>,
    /// Stores the `grab_range` value in `RetiredSettings` as `Option<f32>`.
    ///
    ///
    ///  Optionally stores the grab range for RetiredSettings; it may be absent.
    grab_range: Option<f32>,
    /// Stores the `anti_air_range` value in `RetiredSettings` as `Option<f32>`.
    ///
    ///
    ///  Optionally stores the anti air range for RetiredSettings; it may be absent.
    anti_air_range: Option<f32>,
    /// Stores the `anti_air_height` value in `RetiredSettings` as `Option<f32>`.
    ///
    ///
    ///  Optionally stores the anti air height for RetiredSettings; it may be absent.
    anti_air_height: Option<f32>,
    /// Stores the `turbo_phase_frames` value in `RetiredSettings` as `Option<u16>`.
    ///
    ///
    ///  Sets the turbo phase frames timing for RetiredSettings.
    turbo_phase_frames: Option<u16>,
}

// Validate after parsing so all paths share the same bounds. Floating point
// parsing accepts NaN/infinity, hence is_finite in addition to range checks.
// To expose a wider range, verify the consuming policy first.
/// Determines whether the `finite_in` condition holds.
///
/// Checks whether finite in.
fn finite_in(value: f32, min: f32, max: f32) -> bool {
    value.is_finite() && (min..=max).contains(&value)
}

/// Determines whether the `retired_in` condition holds.
///
/// Checks whether retired in.
fn retired_in(value: Option<f32>, min: f32, max: f32) -> bool {
    value.is_none_or(|value| finite_in(value, min, max))
}

impl LiveConfig {
    /// Parses the supplied source into the module’s validated representation.
    ///
    ///
    ///  Reads the source and checks that it is valid.
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
    ///
    ///
    ///  Makes the supplied change.
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
    ///
    ///
    ///  Gets the validate result.
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
