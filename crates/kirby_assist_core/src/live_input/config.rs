//! Startup config types, defaults, text parsing and validation.
//!
//! Parsing has no file I/O: src/live.rs reads config.txt once and passes its
//! contents here. Start with defaults, apply keys in file order, then validate.
//! A bad value returns Err; no partially parsed config reaches the live hooks.
//!
//! Fighter actions/menus are compiled in fighter_profiles and ledge_trap.
//! This parser supplies shared settings and numeric adjustments, not sequences.
use crate::ledge_trap::LedgeSettings;
use crate::manual_controls::TurboSettings;

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
    /// L-held approximate defense against nearby active fighter attacks.
    pub autopilot_shield: bool,
    /// Legacy setting; Auto-duck is disabled in the live adapter for now.
    pub kirby_auto_duck: bool,
    pub down_tilt_trip_smash: bool,
    pub kirby_fair1_smash: bool,
    pub auto_sdi: bool,
    pub auto_di: crate::auto_di::Settings,
    pub auto_spot_dodge: bool,
    pub auto_ledge_trump: bool,
    pub dpad_slide: bool,
    pub controls: LiveSettings,
    pub turbo: TurboSettings,
    pub ledge: LedgeSettings,
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
            kirby_auto_duck: false,
            down_tilt_trip_smash: true,
            kirby_fair1_smash: true,
            auto_sdi: true,
            auto_di: crate::auto_di::Settings::default(),
            auto_spot_dodge: true,
            auto_ledge_trump: true,
            dpad_slide: false,
            controls: LiveSettings::default(),
            turbo: TurboSettings::default(),
            ledge: LedgeSettings::default(),
            player_index: 0,
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
                "kirby_auto_duck" => {
                    config.kirby_auto_duck = value.parse().map_err(|_| invalid())?
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
                "auto_ledge_trump" => {
                    config.auto_ledge_trump = value.parse().map_err(|_| invalid())?
                }
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
                "dpad_slide" => config.dpad_slide = value.parse().map_err(|_| invalid())?,
                "r3_turbo" | "l3_turbo" => {
                    config.turbo.enabled = value.parse().map_err(|_| invalid())?
                }
                "ledge_trap" => config.ledge.enabled = value.parse().map_err(|_| invalid())?,
                "ledge_nair_hit_lead_frames" => {
                    config.ledge.response_adjustments.nair_hit_lead =
                        Some(value.parse().map_err(|_| invalid())?);
                }
                "ledge_stone_hit_lead_frames" => {
                    config.ledge.response_adjustments.stone_hit_lead =
                        Some(value.parse().map_err(|_| invalid())?);
                }
                "ledge_stone_startup_frames" => {
                    config.ledge.response_adjustments.stone_startup =
                        Some(value.parse().map_err(|_| invalid())?);
                }
                "ledge_drop_request_timeout_frames" => {
                    config.ledge.drop_request_timeout_frames =
                        value.parse().map_err(|_| invalid())?;
                }
                "ledge_drop_defense" => {
                    config.ledge.drop_defense = value.parse().map_err(|_| invalid())?
                }
                "ledge_trap_range" => {
                    config.ledge.horizontal_range = value.parse().map_err(|_| invalid())?
                }
                "ledge_trap_vertical_range" => {
                    config.ledge.vertical_range = value.parse().map_err(|_| invalid())?
                }
                "ledge_getup_attack" => {
                    config.ledge.getup_attack = match value.to_ascii_lowercase().as_str() {
                        "neutral_air" | "nair" => crate::ledge_trap::GetupAttack::NeutralAir,
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
                "ledge_follow_range" => {
                    config.ledge.follow_range = value.parse().map_err(|_| invalid())?
                }
                "ledge_follow_vertical_range" => {
                    config.ledge.follow_vertical_range = value.parse().map_err(|_| invalid())?
                }
                "ledge_distance_agnostic" => {
                    config.ledge.distance_agnostic = value.parse().map_err(|_| invalid())?
                }
                "ledge_manual_spacing" => {
                    config.ledge.manual_spacing = value.parse().map_err(|_| invalid())?
                }
                "ledge_walk_strength" => {
                    config.ledge.walk_strength = value.parse().map_err(|_| invalid())?
                }
                "ledge_roll_dash_range" | "ledge_roll_smash_range" => {
                    config.ledge.roll_dash_range = value.parse().map_err(|_| invalid())?
                }
                "ledge_roll_hit_window_frames" => {
                    config.ledge.roll_hit_window_frames = value.parse().map_err(|_| invalid())?
                }
                "ledge_auto_position" => {
                    config.ledge.auto_position = value.parse().map_err(|_| invalid())?
                }
                "ledge_stand_distance" => {
                    config.ledge.stand_distance = value.parse().map_err(|_| invalid())?
                }
                "ledge_roll_distance" => {
                    config.ledge.roll_distance = if value.eq_ignore_ascii_case("auto") {
                        None
                    } else {
                        Some(value.parse().map_err(|_| invalid())?)
                    };
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
                // Ledge jumps use the paired neutral jump+A response.
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
                "player_index" | "npad_id" => {
                    config.player_index = value.parse().map_err(|_| invalid())?
                }
                "tech_lookahead_frames" => {
                    config.lookahead = value.parse().map_err(|_| invalid())?
                }
                _ => return Err(invalid()),
            }
        }
        let adjustments = config.ledge.response_adjustments;
        if [adjustments.nair_hit_lead, adjustments.stone_hit_lead]
            .iter()
            .flatten()
            .any(|lead| *lead > 20)
            || adjustments
                .stone_startup
                .is_some_and(|startup| !(1..=120).contains(&startup))
            || !(1..=120).contains(&config.ledge.drop_request_timeout_frames)
        {
            return Err("response lead must be 0..20; Stone startup and drop request timeout must be 1..120".into());
        }
        if !(1..=13).contains(&config.ledge.roll_hit_window_frames) {
            return Err("ledge_roll_hit_window_frames must be 1..13".into());
        }
        if !config.ledge.walk_strength.is_finite()
            || !(0.1..=0.45).contains(&config.ledge.walk_strength)
            || !config.ledge.roll_dash_range.is_finite()
            || !(1.0..=20.0).contains(&config.ledge.roll_dash_range)
        {
            return Err(
                "ledge walk strength must be finite 0.1..0.45; roll smash range 1..20".into(),
            );
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
        if config
            .ledge
            .roll_distance
            .is_some_and(|distance| !distance.is_finite() || !(2.0..=100.0).contains(&distance))
        {
            return Err("ledge_roll_distance must be auto or finite 2..100".into());
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
