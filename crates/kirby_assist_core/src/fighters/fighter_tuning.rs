//! Input-policy overrides only. Never writes native fighter parameters.
//! `schema` owns the stable menu/file fields; `Tuning` converts them into
//! policy values with the shared `DEFAULTS` fallbacks.
use crate::settings::AssistSettings;

/// Default close/approach automatic-grab ranges: the standing range, and that
/// range scaled by the fighter's dash-grab/standing-grab reach ratio.
///
/// The approach range is never shorter than the close range. A dash grab whose
/// reach rounds to the standing reach would otherwise produce an approach range
/// a float ulp below it, which disagrees with [`Tuning::auto_grab_ranges`].
fn default_auto_grab_ranges(kind: i32, standing: f32) -> (f32, f32) {
    if !standing.is_finite() || standing <= 0. {
        return (0., 0.);
    }
    let Some((base, dash)) = crate::fighter_data::ordinary_grab(kind, crate::native_states::CATCH)
        .zip(crate::fighter_data::ordinary_grab(
            kind,
            crate::native_states::CATCH_DASH,
        ))
        .and_then(|(base, dash)| Some((base.reach?, dash.reach?)))
        .filter(|(base, dash)| *base > 0. && *dash > 0.)
    else {
        return (0., 0.);
    };
    (standing, (standing * dash / base).max(standing))
}

/// Whether this fighter should turn down tilt toward the opponent by default.
fn tilt_turn_down_default(kind: i32) -> bool {
    matches!(
        kind,
        crate::fighter_kinds::KIRBY
            | crate::fighter_kinds::YOSHI
            | crate::fighter_kinds::LUCINA
    )
}

mod schema {
    //! Stable menu and file schema for per-fighter input-policy overrides.
    //!
    //! The index order is shared by saved configuration, menu rendering, and
    //! runtime policy values. Keep each key, label, and hint at the same index.
    /// Number of stable per-fighter tuning fields.
    pub const COUNT: usize = 83;
    /// Persistent configuration keys in stable tuning-field order.
    pub const KEYS: [&str; COUNT] = [
        "walk_strength",
        "grab_range",
        "horizontal_range",
        "vertical_range",
        "follow_range",
        "follow_vertical_range",
        "stand_distance",
        "move_strength",
        "manual_spacing",
        "auto_position",
        "use_range_limits",
        "grab_dodge_default",
        "out_of_shield_default",
        "grab_dodge_lookahead",
        "shield_warning_x",
        "shield_warning_y",
        "shield_body_half_width",
        "shield_body_height",
        "shield_body_half_depth",
        "shield_forecast",
        "shield_hold",
        "grab_dodge_padding",
        "grab_dodge_height",
        "tech_half_width",
        "tech_floor_origin",
        "tech_wall_low",
        "tech_wall_mid",
        "tech_wall_high",
        "tech_ceiling_origin",
        "tech_floor_padding",
        "tech_ceiling_padding",
        "punish_down_reach",
        "punish_forward_reach",
        "punish_height",
        "roll_smash_height",
        "roll_smash_watch",
        "roll_grab_height",
        "roll_grab_watch",
        "crouch_range",
        "crouch_height",
        "shield_scan_x",
        "shield_scan_y",
        "auto_grab_close_range",
        "auto_grab_dash_range",
        "reflect_default",
        "reflect_startup",
        "reflect_margin",
        "reflect_max_hold",
        "reflect_half_width",
        "reflect_height",
        "reflect_half_depth",
        "tilt_turn_down",
        "tilt_turn_forward",
        "tilt_turn_up",
        "counter_default",
        "counter_startup",
        "counter_margin",
        "counter_half_width",
        "counter_height",
        "counter_half_depth",
        "auto_spacing_minimum_distance",
        "auto_spacing_ground_move",
        "auto_spacing_ground_startup",
        "auto_spacing_ground_active_frames",
        "auto_spacing_ground_reach",
        "auto_spacing_ground_height",
        "auto_spacing_ground_commitment",
        "auto_spacing_aerial_move",
        "auto_spacing_aerial_startup",
        "auto_spacing_aerial_active_frames",
        "auto_spacing_aerial_reach",
        "auto_spacing_aerial_height",
        "auto_spacing_aerial_commitment",
        "auto_di_default",
        "auto_sdi_default",
        "auto_spacing_engagement_range",
        "auto_spacing_ground_speed",
        "auto_spacing_maximum_drift",
        "auto_spacing_minimum_drift",
        "auto_spacing_ground_braking",
        "auto_spacing_air_braking",
        "auto_spacing_braking_distance",
        "auto_defense_lead_frames",
    ];

    /// Menu labels aligned with the stable tuning-field order.
    pub const LABELS: [&str; COUNT] = [
        "Walking stick %",
        "Grab reach estimate",
        "Ledge acquire X",
        "Ledge acquire Y",
        "Ledge follow X",
        "Ledge follow Y",
        "Ledge stand distance",
        "Auto-position stick %",
        "Manual walking cap",
        "Auto ledge stance",
        "Use acquire/follow limits",
        "Grab avoidance at startup",
        "Automatic grabs at startup",
        "Grab dodge forecast frames",
        "Shield: warning X",
        "Shield: warning Y",
        "Shield: body half-width",
        "Shield: body height",
        "Shield: body half-depth",
        "Shield: forecast frames",
        "Shield: release-tail frames",
        "Grab dodge: body padding",
        "Grab dodge: height tolerance",
        "Tech probes: half-width",
        "Tech probes: floor origin Y",
        "Tech probes: low wall Y",
        "Tech probes: middle wall Y",
        "Tech probes: high wall Y",
        "Tech probes: ceiling origin Y",
        "Tech probes: floor extra reach",
        "Tech probes: ceiling extra reach",
        "Legacy punish: down-smash reach",
        "Legacy punish: forward-smash reach",
        "Legacy punish: height tolerance",
        "Legacy roll: smash height",
        "Legacy roll: smash watch distance",
        "Legacy roll: grab height",
        "Legacy roll: grab watch distance",
        "Kirby crouch: proximity X",
        "Kirby crouch: proximity Y",
        "Shield: scan X cutoff",
        "Shield: scan Y cutoff",
        "Automatic grab: shielded range",
        "Automatic grab: approach range",
        "Auto-reflect at startup",
        "Reflect: startup frames",
        "Reflect: early margin",
        "Reflect: maximum hold frames",
        "Reflect: body half-width",
        "Reflect: body height",
        "Reflect: body half-depth",
        "Tilt face: down",
        "Tilt face: forward",
        "Tilt face: up",
        "Auto-counter at startup",
        "Counter: startup frames",
        "Counter: early margin",
        "Counter: body half-width",
        "Counter: body height",
        "Counter: body half-depth",
        "Auto-spacing: minimum distance",
        "Spacing ground move (0-5)",
        "Spacing ground: startup frames",
        "Spacing ground: active frames",
        "Spacing ground: reach estimate",
        "Spacing ground: height tolerance",
        "Spacing ground: commitment frames",
        "Spacing aerial move (0-5)",
        "Spacing aerial: startup frames",
        "Spacing aerial: active frames",
        "Spacing aerial: reach estimate",
        "Spacing aerial: height tolerance",
        "Spacing aerial: commitment frames",
        "Auto DI (experimental)",
        "Auto SDI (experimental)",
        "Spacing: attack engagement range",
        "Spacing: ground walk fraction (0-1)",
        "Spacing: maximum approach drift",
        "Spacing: minimum approach strength",
        "Spacing: Ground Braking",
        "Spacing: Air Braking",
        "Spacing: braking activation distance",
        "Auto Defense lead (frames)",
    ];

    /// Player-facing behavior descriptions aligned with each tuning field.
    pub const HINTS: [&str; COUNT] = [
        "Walking baseline uses the original curve: slightly lower at partial stick, 50% at full horizontal stick. Recipes keep their own strengths.",
        "Your grab reach estimate for recipes, legacy grab and Yoshi chase helpers. Saved attacker overrides also feed ordinary-grab threat estimates. Never extends hitboxes.",
        "Horizontal acquisition limit; requires acquire/follow limits ON.",
        "Vertical acquisition limit; requires acquire/follow limits ON.",
        "Horizontal limit after acquiring a target; requires acquire/follow limits ON.",
        "Vertical limit after acquiring a target; requires acquire/follow limits ON.",
        "Hold-L manual walk stop gap from the tracked ledge; also the automatic stance offset. Toggling autopilot ON alone does not limit walking.",
        "Input strength for eligible automatic stance positioning, not a change to physics.",
        "Enable manual walking limits while L is held. Autopilot ON without L does not cap manual walking.",
        "Existing automatic stance path; requires manual spacing OFF and range limits ON. Recipes own their movements.",
        "OFF preserves distance-agnostic tracking. ON enables the four acquisition/follow limits.",
        "Saved startup ON/OFF for grab avoidance and its Auto react follow-up sequence.",
        "Saved startup ON/OFF for automatic grabs and shield backup; configure their ranges/actions under Auto react.",
        "Frames ahead to forecast ordinary grab contact (1-8). Default 6; larger forecasts can dodge earlier, not guarantee success.",
        "Root-to-root horizontal warning distance for approaching verified attacks.",
        "Root-to-root vertical warning distance; not attack reach.",
        "Estimated body radius around your root used against native hitbox geometry.",
        "Estimated standing body height above your root, not a native hurtbox edit.",
        "Estimated body depth used by the shield threat test.",
        "Shared approach, known-startup and active-hitbox prediction horizon.",
        "Extra defensive shield hold after a warning ends. Regrabs still clear the old tail.",
        "Your estimated horizontal body allowance added to the actual attacker grab reach.",
        "Maximum root-height difference for predicted ordinary grab contact.",
        "Horizontal body offsets for floor, wall and ceiling probes; does not alter native tech eligibility.",
        "Floor ray origin above your root.",
        "Lowest wall probe height above your root.",
        "Middle wall probe height above your root.",
        "Highest wall probe height above your root.",
        "Upward ray origin above your root.",
        "Extra downward distance on floor sweeps, including curved/refined sweeps.",
        "Extra upward distance on ceiling sweeps.",
        "Root-based reach for legacy spot-punish and legacy roll down-smash; recipes retain their own ranges.",
        "Root-based forward-smash reach for legacy automatic punish selection.",
        "Vertical root gap used by the legacy spot-dodge punish.",
        "Height tolerance for the Kirby/Yoshi legacy roll smash reader.",
        "Maximum root distance for watching legacy roll-smash opportunities.",
        "Height tolerance for the Kirby/Yoshi legacy roll grab reader.",
        "Maximum root distance for watching legacy roll-grab opportunities.",
        "Maximum horizontal root gap for Kirby automatic crouch.",
        "Maximum vertical root gap for Kirby automatic crouch.",
        "Root-distance limit for estimated startup/recovery warnings; active geometry is checked independently.",
        "Root-height limit for estimated startup/recovery warnings; active geometry is checked independently.",
        "No-dash grab limit while you shield. A vulnerable grabbable opponent in front qualifies; enemy shield is not required.",
        "Approach-grab limit when not shielding: one full-stick pulse toward a shielding opponent, then R. Never used for a dash out of shield.",
        "Explicit fighter-specific projectile defense. Unsupported fighters never inherit a reflector.",
        "Frames before reflection/absorption begins; cannot reduce Game & Watch below his native six-frame startup.",
        "Start this many frames before predicted contact plus native startup; no waiting action is inserted.",
        "Bound on holding one confirmed reflector; vanished threats, capture and other interruptions release sooner.",
        "Estimated body half-width for incoming projectile prediction; does not change native geometry.",
        "Estimated body height above the root for incoming projectile prediction.",
        "Estimated depth allowance for incoming projectile prediction; off-plane attacks are ignored.",
        "Prepare down tilt toward the opponent. Kirby and Yoshi default ON.",
        "Aim physical forward C-stick tilts toward the opponent. Requires Tilt Attack C-stick mapping.",
        "Aim physical up C-stick tilts toward the opponent. Requires Tilt Attack C-stick mapping.",
        "Explicit fighter counter configuration; only verified down-special counters are supported.",
        "Frames before the counter becomes active; tune to the fighter's native move startup.",
        "Additional frames ahead of predicted melee contact to begin the counter.",
        "Estimated horizontal body half-width for active melee hitbox prediction.",
        "Estimated body height above root for active melee hitbox prediction.",
        "Estimated body half-depth for active melee hitbox prediction.",
        "Minimum root-to-root spacing while the assigned Auto-Spacing & Attack button is held. Universal input assistance, never a toggle.",
        "0 disabled (default for every fighter); 1 jab; 2 forward tilt; 3 up tilt; 4 down tilt; 5 back (invalid, disabled). Ground tilts use directional tilt inputs, never move or status edits.",
        "Estimated frames before the configured ground attack becomes active; does not change native startup.",
        "Estimated active duration of the configured ground attack; never changes native hitboxes.",
        "Tunable ground reach estimate, not an exact hitbox or hit guarantee.",
        "Tunable ground root-height tolerance, not an exact hitbox or hit guarantee.",
        "Estimated total ground attack commitment in frames; independent of startup and active duration.",
        "0 disabled (default for every fighter); 1 neutral air; 2 forward air; 3 up air; 4 down air; 5 back air. Already airborne only; never jumps or consumes manual jump.",
        "Estimated frames before the configured aerial becomes active; does not change native startup.",
        "Estimated active duration of the configured aerial; never changes native hitboxes.",
        "Tunable aerial reach estimate, not an exact hitbox or hit guarantee.",
        "Tunable aerial root-height tolerance, not an exact hitbox or hit guarantee.",
        "Estimated total aerial commitment in frames; independent of startup and active duration.",
        "Experimental input-only DI with observed blast bounds; not terrain-aware or guaranteed survival. Default OFF.",
        "Experimental input-only SDI with observed blast bounds and attacker direction. Default OFF.",
        "Attack only inside this horizontal distance. Ground approach walks toward the target even outside attack range; aerial direction stays manual outside range. Independent of minimum retreat spacing; holding blocks other automatic modes.",
        "Fraction of the native fastest safe non-dash walk: 0 stops ground movement including retreat, 1 uses maximum safe walk. Positive settings retreat at maximum safe walk; neither direction dashes.",
        "Maximum approaching aerial stick strength while held. Lucina defaults to 0.6; others to 1. Moving away permits full drift. Attack-selection packets retain their required direction.",
        "Minimum approaching horizontal strength at the spacing boundary (0-1, default 0.1), capped by the selected maximum. Enabled braking smoothly tapers within its activation distance; never changes native physics.",
        "Smooth approaching ground input near minimum spacing. OFF keeps the configured walk fraction. Never slows retreat.",
        "Smooth approaching aerial input near minimum spacing. OFF keeps the configured maximum drift. Never slows retreat.",
        "Root-distance above minimum spacing where enabled braking begins (default 5). Outside this zone movement keeps its normal configured strength. Zero disables tapering.",
        "Frames ahead of predicted contact to begin Auto Defense (0-30, default 4). Input-only timing, not native move startup.",
    ];

    pub const AUTO_SPACING_ENGAGEMENT_RANGE: usize = 75;
    pub const AUTO_SPACING_GROUND_SPEED: usize = 76;
    pub const AUTO_SPACING_MAXIMUM_DRIFT: usize = 77;
    pub const AUTO_SPACING_MINIMUM_DRIFT: usize = 78;
    pub const AUTO_SPACING_GROUND_BRAKING: usize = 79;
    pub const AUTO_SPACING_AIR_BRAKING: usize = 80;
    pub const AUTO_SPACING_BRAKING_DISTANCE: usize = 81;
    pub const AUTO_DEFENSE_LEAD_FRAMES: usize = 82;

    /// Experimental damage-stick preferences, appended for file compatibility.
    pub const AUTO_DI_DEFAULT: usize = 73;
    /// Independent experimental SDI preference.
    pub const AUTO_SDI_DEFAULT: usize = 74;

    /// Minimum hold-mode spacing distance.
    pub const AUTO_SPACING_MINIMUM_DISTANCE: usize = 60;
    /// Ground move selector: disabled, jab, forward/up/down tilt, invalid back.
    pub const AUTO_SPACING_GROUND_MOVE: usize = 61;
    /// Ground startup estimate.
    pub const AUTO_SPACING_GROUND_STARTUP: usize = 62;
    /// Ground active duration estimate.
    pub const AUTO_SPACING_GROUND_ACTIVE_FRAMES: usize = 63;
    /// Ground reach estimate.
    pub const AUTO_SPACING_GROUND_REACH: usize = 64;
    /// Ground height tolerance.
    pub const AUTO_SPACING_GROUND_HEIGHT: usize = 65;
    /// Ground total commitment estimate.
    pub const AUTO_SPACING_GROUND_COMMITMENT: usize = 66;
    /// Aerial move selector: disabled, neutral/forward/up/down/back air.
    pub const AUTO_SPACING_AERIAL_MOVE: usize = 67;
    /// Aerial startup estimate.
    pub const AUTO_SPACING_AERIAL_STARTUP: usize = 68;
    /// Aerial active duration estimate.
    pub const AUTO_SPACING_AERIAL_ACTIVE_FRAMES: usize = 69;
    /// Aerial reach estimate.
    pub const AUTO_SPACING_AERIAL_REACH: usize = 70;
    /// Aerial height tolerance.
    pub const AUTO_SPACING_AERIAL_HEIGHT: usize = 71;
    /// Aerial total commitment estimate.
    pub const AUTO_SPACING_AERIAL_COMMITMENT: usize = 72;
    /// Stable tuning-array index for tilt turn down.
    pub const TILT_TURN_DOWN: usize = 51;
    /// Stable tuning-array index for tilt turn forward.
    pub const TILT_TURN_FORWARD: usize = 52;
    /// Stable tuning-array index for tilt turn up.
    pub const TILT_TURN_UP: usize = 53;
    /// Explicit counter setup flag.
    pub const COUNTER_DEFAULT: usize = 54;
    /// Counter native startup estimate.
    pub const COUNTER_STARTUP: usize = 55;
    /// Early activation margin.
    pub const COUNTER_MARGIN: usize = 56;
    /// Body half-width for melee prediction.
    pub const COUNTER_HALF_WIDTH: usize = 57;
    /// Body height for melee prediction.
    pub const COUNTER_HEIGHT: usize = 58;
    /// Body half-depth for melee prediction.
    pub const COUNTER_HALF_DEPTH: usize = 59;
    /// Stable tuning-array index for reflect default.
    pub const REFLECT_DEFAULT: usize = 44;
    /// Stable tuning-array index for reflect startup.
    pub const REFLECT_STARTUP: usize = 45;
    /// Stable tuning-array index for reflect margin.
    pub const REFLECT_MARGIN: usize = 46;
    /// Stable tuning-array index for reflect max hold.
    pub const REFLECT_MAX_HOLD: usize = 47;
    /// Stable tuning-array index for reflect half width.
    pub const REFLECT_HALF_WIDTH: usize = 48;
    /// Stable tuning-array index for reflect height.
    pub const REFLECT_HEIGHT: usize = 49;
    /// Stable tuning-array index for reflect half depth.
    pub const REFLECT_HALF_DEPTH: usize = 50;
    /// Stable tuning-array index for shield scan x.
    pub const SHIELD_SCAN_X: usize = 40;
    /// Stable tuning-array index for shield scan y.
    pub const SHIELD_SCAN_Y: usize = 41;
    /// Stable tuning-array index for auto grab close.
    pub const AUTO_GRAB_CLOSE: usize = 42;
    /// Stable tuning-array index for auto grab far.
    pub const AUTO_GRAB_FAR: usize = 43;
    /// Stable tuning-array index for shield warning x.
    pub const SHIELD_WARNING_X: usize = 14;
    /// Stable tuning-array index for shield warning y.
    pub const SHIELD_WARNING_Y: usize = 15;
    /// Stable tuning-array index for shield body half width.
    pub const SHIELD_BODY_HALF_WIDTH: usize = 16;
    /// Stable tuning-array index for shield body height.
    pub const SHIELD_BODY_HEIGHT: usize = 17;
    /// Stable tuning-array index for shield body half depth.
    pub const SHIELD_BODY_HALF_DEPTH: usize = 18;
    /// Stable tuning-array index for shield forecast.
    pub const SHIELD_FORECAST: usize = 19;
    /// Stable tuning-array index for shield hold.
    pub const SHIELD_HOLD: usize = 20;
    /// Stable tuning-array index for grab dodge padding.
    pub const GRAB_DODGE_PADDING: usize = 21;
    /// Stable tuning-array index for grab dodge height.
    pub const GRAB_DODGE_HEIGHT: usize = 22;
    /// Stable tuning-array index for tech half width.
    pub const TECH_HALF_WIDTH: usize = 23;
    /// Stable tuning-array index for tech floor origin.
    pub const TECH_FLOOR_ORIGIN: usize = 24;
    /// Stable tuning-array index for tech wall low.
    pub const TECH_WALL_LOW: usize = 25;
    /// Stable tuning-array index for tech wall mid.
    pub const TECH_WALL_MID: usize = 26;
    /// Stable tuning-array index for tech wall high.
    pub const TECH_WALL_HIGH: usize = 27;
    /// Stable tuning-array index for tech ceiling origin.
    pub const TECH_CEILING_ORIGIN: usize = 28;
    /// Stable tuning-array index for tech floor padding.
    pub const TECH_FLOOR_PADDING: usize = 29;
    /// Stable tuning-array index for tech ceiling padding.
    pub const TECH_CEILING_PADDING: usize = 30;
    /// Stable tuning-array index for punish down reach.
    pub const PUNISH_DOWN_REACH: usize = 31;
    /// Stable tuning-array index for punish forward reach.
    pub const PUNISH_FORWARD_REACH: usize = 32;
    /// Stable tuning-array index for punish height.
    pub const PUNISH_HEIGHT: usize = 33;
    /// Stable tuning-array index for roll smash height.
    pub const ROLL_SMASH_HEIGHT: usize = 34;
    /// Stable tuning-array index for roll smash watch.
    pub const ROLL_SMASH_WATCH: usize = 35;
    /// Stable tuning-array index for roll grab height.
    pub const ROLL_GRAB_HEIGHT: usize = 36;
    /// Stable tuning-array index for roll grab watch.
    pub const ROLL_GRAB_WATCH: usize = 37;
    /// Stable tuning-array index for crouch range.
    pub const CROUCH_RANGE: usize = 38;
    /// Stable tuning-array index for crouch height.
    pub const CROUCH_HEIGHT: usize = 39;
    /// Stable tuning-array index for walk strength.
    pub const WALK_STRENGTH: usize = 0;
    /// Stable tuning-array index for grab range.
    pub const GRAB_RANGE: usize = 1;
    /// Stable tuning-array index for horizontal range.
    pub const HORIZONTAL_RANGE: usize = 2;
    /// Stable tuning-array index for vertical range.
    pub const VERTICAL_RANGE: usize = 3;
    /// Stable tuning-array index for follow range.
    pub const FOLLOW_RANGE: usize = 4;
    /// Stable tuning-array index for follow vertical range.
    pub const FOLLOW_VERTICAL_RANGE: usize = 5;
    /// Stable tuning-array index for stand distance.
    pub const STAND_DISTANCE: usize = 6;
    /// Stable tuning-array index for move strength.
    pub const MOVE_STRENGTH: usize = 7;
    /// Stable tuning-array index for manual spacing.
    pub const MANUAL_SPACING: usize = 8;
    /// Stable tuning-array index for auto position.
    pub const AUTO_POSITION: usize = 9;
    /// Stable tuning-array index for use range limits.
    pub const USE_RANGE_LIMITS: usize = 10;
    /// Stable tuning-array index for grab dodge default.
    pub const GRAB_DODGE_DEFAULT: usize = 11;
    /// Stable tuning-array index for out of shield default.
    pub const OUT_OF_SHIELD_DEFAULT: usize = 12;
    /// Stable tuning-array index for grab dodge lookahead.
    pub const GRAB_DODGE_LOOKAHEAD: usize = 13;
}

/// Minimum gamewatch reflect startup frames accepted by the input policy.
pub const MIN_GAMEWATCH_REFLECT_STARTUP_FRAMES: f32 = 6.0;
/// Minimum grab dodge lookahead frames accepted by the input policy.
pub const MIN_GRAB_DODGE_LOOKAHEAD_FRAMES: f32 = 1.0;
/// Maximum grab dodge lookahead frames accepted by the input policy.
pub const MAX_GRAB_DODGE_LOOKAHEAD_FRAMES: f32 = 8.0;
/// Default grab dodge padding accepted by the input policy.
pub const DEFAULT_GRAB_DODGE_PADDING: f32 = 2.0;
/// Maximum grab dodge padding accepted by the input policy.
pub const MAX_GRAB_DODGE_PADDING: f32 = 30.0;
/// Default grab dodge height accepted by the input policy.
pub const DEFAULT_GRAB_DODGE_HEIGHT: f32 = 8.0;
/// Maximum grab dodge height accepted by the input policy.
pub const MAX_GRAB_DODGE_HEIGHT: f32 = 60.0;
/// Default reflect margin frames accepted by the input policy.
const DEFAULT_REFLECT_MARGIN_FRAMES: f32 = 2.0;
/// Default reflect max hold frames accepted by the input policy.
const DEFAULT_REFLECT_MAX_HOLD_FRAMES: f32 = 45.0;
/// Default reflect half width accepted by the input policy.
const DEFAULT_REFLECT_HALF_WIDTH: f32 = 5.0;
/// Default reflect height accepted by the input policy.
const DEFAULT_REFLECT_HEIGHT: f32 = 16.0;
/// Default reflect half depth accepted by the input policy.
const DEFAULT_REFLECT_HALF_DEPTH: f32 = 5.0;

pub use schema::*;

/// Fixed fallback for every override that has one, shared by the runtime reads
/// (`Tuning::get`) and the menu display (`Tuning::values`) so they cannot drift.
/// Rows derived from settings or the fighter kind are filled in by `values`.
const DEFAULTS: [f32; COUNT] = {
    let mut d = [0.; COUNT];
    d[GRAB_DODGE_DEFAULT] = 1.;
    d[OUT_OF_SHIELD_DEFAULT] = 1.;
    d[GRAB_DODGE_LOOKAHEAD] = crate::spot_dodge::LOOKAHEAD;
    d[SHIELD_WARNING_X] = 24.;
    d[SHIELD_WARNING_Y] = 28.;
    d[SHIELD_BODY_HALF_WIDTH] = 6.;
    d[SHIELD_BODY_HEIGHT] = 18.;
    d[SHIELD_BODY_HALF_DEPTH] = 6.;
    d[SHIELD_FORECAST] = 5.;
    d[SHIELD_HOLD] = 4.;
    d[GRAB_DODGE_PADDING] = DEFAULT_GRAB_DODGE_PADDING;
    d[GRAB_DODGE_HEIGHT] = DEFAULT_GRAB_DODGE_HEIGHT;
    d[TECH_HALF_WIDTH] = 2.5;
    d[TECH_FLOOR_ORIGIN] = 1.;
    d[TECH_WALL_LOW] = 1.;
    d[TECH_WALL_MID] = 3.5;
    d[TECH_WALL_HIGH] = 6.;
    d[TECH_CEILING_ORIGIN] = 5.;
    d[TECH_FLOOR_PADDING] = 1.5;
    d[TECH_CEILING_PADDING] = 2.;
    d[PUNISH_HEIGHT] = 8.;
    d[ROLL_SMASH_HEIGHT] = 6.;
    d[ROLL_SMASH_WATCH] = 64.;
    d[ROLL_GRAB_HEIGHT] = 4.;
    d[CROUCH_RANGE] = 45.;
    d[CROUCH_HEIGHT] = 20.;
    d[SHIELD_SCAN_X] = 96.;
    d[SHIELD_SCAN_Y] = 96.;
    d[REFLECT_STARTUP] = MIN_GAMEWATCH_REFLECT_STARTUP_FRAMES;
    d[REFLECT_MARGIN] = DEFAULT_REFLECT_MARGIN_FRAMES;
    d[REFLECT_MAX_HOLD] = DEFAULT_REFLECT_MAX_HOLD_FRAMES;
    d[REFLECT_HALF_WIDTH] = DEFAULT_REFLECT_HALF_WIDTH;
    d[REFLECT_HEIGHT] = DEFAULT_REFLECT_HEIGHT;
    d[REFLECT_HALF_DEPTH] = DEFAULT_REFLECT_HALF_DEPTH;
    d[COUNTER_STARTUP] = 6.;
    d[COUNTER_MARGIN] = 2.;
    d[COUNTER_HALF_WIDTH] = 5.;
    d[COUNTER_HEIGHT] = 16.;
    d[COUNTER_HALF_DEPTH] = 5.;
    d[AUTO_SPACING_MINIMUM_DISTANCE] = 15.;
    d[AUTO_SPACING_GROUND_SPEED] = 1.;
    d[AUTO_SPACING_GROUND_BRAKING] = 1.;
    d[AUTO_SPACING_AIR_BRAKING] = 1.;
    d[AUTO_SPACING_BRAKING_DISTANCE] = 10.;
    d[AUTO_DEFENSE_LEAD_FRAMES] = 4.;
    d[AUTO_SPACING_ENGAGEMENT_RANGE] = 20.;
    d[AUTO_SPACING_MAXIMUM_DRIFT] = 1.;
    d[AUTO_SPACING_MINIMUM_DRIFT] = 0.1;
    d[AUTO_SPACING_GROUND_STARTUP] = 8.;
    d[AUTO_SPACING_GROUND_ACTIVE_FRAMES] = 5.;
    d[AUTO_SPACING_GROUND_REACH] = 20.;
    d[AUTO_SPACING_GROUND_HEIGHT] = 9.;
    d[AUTO_SPACING_GROUND_COMMITMENT] = 33.;
    d[AUTO_SPACING_AERIAL_STARTUP] = 6.;
    d[AUTO_SPACING_AERIAL_ACTIVE_FRAMES] = 3.;
    d[AUTO_SPACING_AERIAL_REACH] = 20.;
    d[AUTO_SPACING_AERIAL_HEIGHT] = 12.;
    d[AUTO_SPACING_AERIAL_COMMITMENT] = 30.;
    d
};

/// Optional per-fighter overrides for input-assistance policy values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tuning(
    /// Per-field optional tuning overrides in schema order.
    pub [Option<f32>; COUNT],
);
/// Resolved numeric tuning values after defaults and fighter overrides are applied.
#[derive(Clone, Copy)]
pub struct Values(
    /// Fully resolved tuning values in schema order.
    pub [f32; COUNT],
);
impl Default for Values {
    /// Initializes the resolved tuning array before defaults are populated.
    fn default() -> Self {
        Self([0.; COUNT])
    }
}
impl std::ops::Deref for Values {
    /// Resolved tuning array exposed through `Deref`.
    type Target = [f32; COUNT];
    /// Exposes the wrapped value through the `Deref` implementation.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Values {
    /// Returns mutable access to the deref value.
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Default for Tuning {
    /// Starts with no per-fighter overrides; shared policy defaults remain active.
    fn default() -> Self {
        Self([None; COUNT])
    }
}
impl Tuning {
    /// Returns one override, falling back when that fighter has no saved value.
    pub fn value(&self, i: usize, fallback: f32) -> f32 {
        self.0[i].unwrap_or(fallback)
    }
    /// The override, or this row's shared fixed default.
    pub fn get(&self, i: usize) -> f32 {
        self.value(i, DEFAULTS[i])
    }
    /// Whether this tuning row is displayed and parsed as a frame count.
    pub fn frames(i: usize) -> bool {
        matches!(
            i,
            GRAB_DODGE_LOOKAHEAD
                | SHIELD_FORECAST
                | SHIELD_HOLD
                | REFLECT_STARTUP
                | REFLECT_MARGIN
                | REFLECT_MAX_HOLD
                | COUNTER_STARTUP
                | COUNTER_MARGIN
                | AUTO_DEFENSE_LEAD_FRAMES
                | AUTO_SPACING_GROUND_STARTUP
                | AUTO_SPACING_GROUND_ACTIVE_FRAMES
                | AUTO_SPACING_GROUND_COMMITMENT
                | AUTO_SPACING_AERIAL_STARTUP
                | AUTO_SPACING_AERIAL_ACTIVE_FRAMES
                | AUTO_SPACING_AERIAL_COMMITMENT
        )
    }
    /// Menu rows shown for this fighter: reflect rows need a supported reflector,
    /// the legacy punish/roll rows are hidden, and crouch rows are Kirby-only.
    pub fn visible(kind: i32, i: usize) -> bool {
        match i {
            REFLECT_DEFAULT..=REFLECT_HALF_DEPTH => crate::projectile_reflect::supports(kind),
            COUNTER_DEFAULT..=COUNTER_HALF_DEPTH => crate::auto_counter::supports(kind),
            PUNISH_DOWN_REACH..=ROLL_GRAB_WATCH => false,
            AUTO_SPACING_MINIMUM_DISTANCE | AUTO_SPACING_ENGAGEMENT_RANGE => false,
            CROUCH_RANGE..=CROUCH_HEIGHT => kind == crate::fighter_kinds::KIRBY,
            TILT_TURN_DOWN..=TILT_TURN_UP => matches!(
                kind,
                crate::fighter_kinds::KIRBY
                    | crate::fighter_kinds::YOSHI
                    | crate::fighter_kinds::LUCINA
            ),
            _ => i < COUNT,
        }
    }
    /// Whether the menu row stores an on/off value instead of a scalar.
    pub fn is_bool(i: usize) -> bool {
        (MANUAL_SPACING..=OUT_OF_SHIELD_DEFAULT).contains(&i)
            || i == REFLECT_DEFAULT
            || i == COUNTER_DEFAULT
            || i == AUTO_DI_DEFAULT
            || i == AUTO_SDI_DEFAULT
            || i == AUTO_SPACING_GROUND_BRAKING
            || i == AUTO_SPACING_AIR_BRAKING
            || (TILT_TURN_DOWN..=TILT_TURN_UP).contains(&i)
    }
    /// Resolves which physical C-stick tilts should face the opponent.
    pub fn tilt_turn(self, kind: i32) -> crate::posture::TiltSettings {
        crate::posture::TiltSettings {
            down: self.value(TILT_TURN_DOWN, f32::from(tilt_turn_down_default(kind))) == 1.0,
            forward: self.get(TILT_TURN_FORWARD) == 1.0,
            up: self.get(TILT_TURN_UP) == 1.0,
        }
    }
    /// Resolves universal hold-mode spacing and independent attack estimates.
    pub fn auto_spacing(self, kind: i32) -> crate::auto_spacing::Config {
        use crate::auto_spacing::{Attack, AttackConfig, Config};
        let attack = |i, aerial| match self.get(i) {
            1. => Attack::Neutral,
            2. => Attack::Forward,
            3. => Attack::Up,
            4. => Attack::Down,
            5. if aerial => Attack::Back,
            _ => Attack::Disabled,
        };
        Config {
            minimum_distance: self.get(AUTO_SPACING_MINIMUM_DISTANCE),
            engagement_range: self.get(AUTO_SPACING_ENGAGEMENT_RANGE),
            ground_speed: self.get(AUTO_SPACING_GROUND_SPEED),
            maximum_drift: self.value(AUTO_SPACING_MAXIMUM_DRIFT,
                if kind == crate::fighter_kinds::LUCINA { 0.6 } else { 1. }),
            minimum_drift: self.get(AUTO_SPACING_MINIMUM_DRIFT),
            ground_braking: self.get(AUTO_SPACING_GROUND_BRAKING) == 1.,
            air_braking: self.get(AUTO_SPACING_AIR_BRAKING) == 1.,
            braking_distance: self.get(AUTO_SPACING_BRAKING_DISTANCE),
            ground: AttackConfig {
                attack: attack(AUTO_SPACING_GROUND_MOVE, false),
                startup: self.get(AUTO_SPACING_GROUND_STARTUP),
                active_frames: self.get(AUTO_SPACING_GROUND_ACTIVE_FRAMES),
                reach: self.get(AUTO_SPACING_GROUND_REACH),
                height: self.get(AUTO_SPACING_GROUND_HEIGHT),
                commitment: self.get(AUTO_SPACING_GROUND_COMMITMENT),
            },
            aerial: AttackConfig {
                attack: attack(AUTO_SPACING_AERIAL_MOVE, true),
                startup: self.get(AUTO_SPACING_AERIAL_STARTUP),
                active_frames: self.get(AUTO_SPACING_AERIAL_ACTIVE_FRAMES),
                reach: self.get(AUTO_SPACING_AERIAL_REACH),
                height: self.get(AUTO_SPACING_AERIAL_HEIGHT),
                commitment: self.get(AUTO_SPACING_AERIAL_COMMITMENT),
            },
        }
    }
    /// Returns the default grab-dodge setting.
    pub fn grab_dodge_default(self, base: bool) -> bool {
        self.0[GRAB_DODGE_DEFAULT].map_or(base, |x| x == 1.0)
    }
    /// Returns the default out-of-shield setting.
    pub fn oos_default(self) -> bool {
        self.get(OUT_OF_SHIELD_DEFAULT) == 1.0
    }
    /// Frames ahead used to predict an ordinary grab before spot-dodging.
    pub fn dodge_lookahead(self) -> f32 {
        self.get(GRAB_DODGE_LOOKAHEAD)
    }
    /// Close-range limit for an automatic standing grab.
    pub fn auto_grab_range(&self, kind: i32, standing: f32) -> f32 {
        self.auto_grab_ranges(kind, standing).0
    }
    /// Returns the per-fighter automatic-grab ranges.
    pub fn auto_grab_ranges(&self, kind: i32, standing: f32) -> (f32, f32) {
        let (close, far) = match (self.0[AUTO_GRAB_CLOSE], self.0[AUTO_GRAB_FAR]) {
            (Some(close), Some(far)) => (close, far),
            (close, far) => {
                let defaults = default_auto_grab_ranges(kind, standing);
                (close.unwrap_or(defaults.0), far.unwrap_or(defaults.1))
            }
        };
        (close, far.max(close))
    }
    /// Returns the shield observation used by the response.
    pub fn shield(&self) -> crate::autopilot_guard::Envelope {
        crate::autopilot_guard::Envelope {
            scan_x: self.get(SHIELD_SCAN_X),
            scan_y: self.get(SHIELD_SCAN_Y),
            warning_x: self.get(SHIELD_WARNING_X),
            warning_y: self.get(SHIELD_WARNING_Y),
            half_width: self.get(SHIELD_BODY_HALF_WIDTH),
            height: self.get(SHIELD_BODY_HEIGHT),
            half_depth: self.get(SHIELD_BODY_HALF_DEPTH),
            forecast: self.get(SHIELD_FORECAST),
        }
    }
    /// Returns the body area used for tech prediction.
    pub fn tech_body(&self) -> crate::tech_prediction::Body {
        crate::tech_prediction::Body {
            half_width: self.get(TECH_HALF_WIDTH),
            floor_origin: self.get(TECH_FLOOR_ORIGIN),
            wall: [
                self.get(TECH_WALL_LOW),
                self.get(TECH_WALL_MID),
                self.get(TECH_WALL_HIGH),
            ],
            ceiling_origin: self.get(TECH_CEILING_ORIGIN),
            floor_padding: self.get(TECH_FLOOR_PADDING),
            ceiling_padding: self.get(TECH_CEILING_PADDING),
        }
    }
    /// Returns the geometric bounds for the observed object.
    pub fn bounds(i: usize) -> (f32, f32, f32) {
        if Self::is_bool(i) {
            return (0., 1., 1.);
        }
        match i {
            AUTO_SPACING_GROUND_MOVE | AUTO_SPACING_AERIAL_MOVE => (0., 5., 1.),
            AUTO_DEFENSE_LEAD_FRAMES => (0., 30., 1.),
            AUTO_SPACING_GROUND_SPEED | AUTO_SPACING_MAXIMUM_DRIFT
            | AUTO_SPACING_MINIMUM_DRIFT => (0., 1., 0.01),
            AUTO_SPACING_MINIMUM_DISTANCE
            | AUTO_SPACING_BRAKING_DISTANCE
            | AUTO_SPACING_ENGAGEMENT_RANGE
            | AUTO_SPACING_GROUND_REACH
            | AUTO_SPACING_AERIAL_REACH => (0., 150., 0.5),
            AUTO_SPACING_GROUND_HEIGHT | AUTO_SPACING_AERIAL_HEIGHT => (0., 100., 0.5),
            AUTO_SPACING_GROUND_STARTUP
            | AUTO_SPACING_GROUND_ACTIVE_FRAMES
            | AUTO_SPACING_GROUND_COMMITMENT
            | AUTO_SPACING_AERIAL_STARTUP
            | AUTO_SPACING_AERIAL_ACTIVE_FRAMES
            | AUTO_SPACING_AERIAL_COMMITMENT => (1., 300., 1.),
            REFLECT_STARTUP => (6., 30., 1.),
            REFLECT_MARGIN => (0., 8., 1.),
            REFLECT_MAX_HOLD => (6., 120., 1.),
            COUNTER_STARTUP => (1., 30., 1.),
            COUNTER_MARGIN => (0., 10., 1.),
            COUNTER_HALF_WIDTH | COUNTER_HALF_DEPTH => (1., 30., 0.5),
            COUNTER_HEIGHT => (1., 60., 0.5),
            GRAB_DODGE_LOOKAHEAD => (
                MIN_GRAB_DODGE_LOOKAHEAD_FRAMES,
                MAX_GRAB_DODGE_LOOKAHEAD_FRAMES,
                0.5,
            ),
            WALK_STRENGTH => (0.1, 0.5, 0.01),
            MOVE_STRENGTH => (0.1, 1.0, 0.01),
            SHIELD_SCAN_X | SHIELD_SCAN_Y => (1.0, 300.0, 1.0),
            GRAB_RANGE => (1.0, 60.0, 0.5),
            REFLECT_HALF_WIDTH | REFLECT_HALF_DEPTH => (1., 30., 0.5),
            REFLECT_HEIGHT => (1., 60., 0.5),
            SHIELD_FORECAST => (0.0, 10.0, 0.5),
            TECH_FLOOR_PADDING | TECH_CEILING_PADDING => (0.0, 15.0, 0.5),
            SHIELD_HOLD => (0.0, 20.0, 0.5),
            GRAB_DODGE_PADDING | TECH_HALF_WIDTH => (0.0, 30.0, 0.5),
            SHIELD_BODY_HALF_WIDTH
            | SHIELD_BODY_HALF_DEPTH
            | TECH_FLOOR_ORIGIN
            | TECH_WALL_LOW
            | TECH_WALL_MID => (0.0, 40.0, 0.5),
            SHIELD_BODY_HEIGHT | GRAB_DODGE_HEIGHT | TECH_WALL_HIGH | TECH_CEILING_ORIGIN
            | PUNISH_HEIGHT | ROLL_SMASH_HEIGHT | ROLL_GRAB_HEIGHT => (0.0, 60.0, 0.5),
            SHIELD_WARNING_X | SHIELD_WARNING_Y | PUNISH_DOWN_REACH | PUNISH_FORWARD_REACH
            | CROUCH_HEIGHT => (0.0, 100.0, 0.5),
            AUTO_GRAB_CLOSE | AUTO_GRAB_FAR | ROLL_SMASH_WATCH | ROLL_GRAB_WATCH | CROUCH_RANGE => {
                (0.0, 150.0, 0.5)
            }
            _ => (1.0, 150.0, 0.5),
        }
    }
    /// Parses and saves one named per-fighter tuning override.
    pub fn set(&mut self, key: &str, text: &str) -> Result<(), String> {
        let i = KEYS
            .iter()
            .position(|k| *k == key)
            .ok_or_else(|| format!("unknown fighter setting {key}"))?;
        if self.0[i].is_some() {
            return Err(format!("duplicate fighter setting {key}"));
        }
        let value = if Self::is_bool(i) {
            match text {
                "true" => 1.0,
                "false" => 0.0,
                _ => return Err(format!("{key}: expected true or false")),
            }
        } else {
            text.parse::<f32>()
                .map_err(|_| format!("{key}: expected a number"))?
        };
        let (lo, hi, _) = Self::bounds(i);
        if !value.is_finite() || !(lo..=hi).contains(&value) {
            return Err(format!("{key}: must be {lo}..{hi}"));
        }
        if matches!(i, AUTO_SPACING_GROUND_MOVE | AUTO_SPACING_AERIAL_MOVE) && value.fract() != 0. {
            return Err(format!("{key}: expected an integer move selector 0..5"));
        }
        self.0[i] = Some(value);
        Ok(())
    }
    /// Returns the resolved tuning values.
    pub fn values(s: AssistSettings) -> Values {
        let mut values = DEFAULTS;
        values[WALK_STRENGTH] = s.walk_strength;
        values[GRAB_RANGE] = s.fighter.grab.standing_range;
        values[HORIZONTAL_RANGE] = s.horizontal_range;
        values[VERTICAL_RANGE] = s.vertical_range;
        values[FOLLOW_RANGE] = s.follow_range;
        values[FOLLOW_VERTICAL_RANGE] = s.follow_vertical_range;
        values[STAND_DISTANCE] = s.stand_distance;
        values[MOVE_STRENGTH] = s.move_strength;
        values[MANUAL_SPACING] = f32::from(s.manual_spacing);
        values[AUTO_POSITION] = f32::from(s.auto_position);
        values[USE_RANGE_LIMITS] = f32::from(!s.distance_agnostic);
        let kirby = s.fighter_kind == crate::fighter_kinds::KIRBY;
        let yoshi = s.fighter_kind == crate::fighter_kinds::YOSHI;
        values[PUNISH_DOWN_REACH] = if kirby {
            14.0
        } else if yoshi {
            13.0
        } else {
            12.0
        };
        values[PUNISH_FORWARD_REACH] = if kirby { 22.0 } else { 21.0 };
        values[ROLL_GRAB_WATCH] = if kirby { 32.0 } else { 36.0 };
        (values[AUTO_GRAB_CLOSE], values[AUTO_GRAB_FAR]) =
            default_auto_grab_ranges(s.fighter_kind, s.fighter.grab.standing_range);
        values[TILT_TURN_DOWN] = f32::from(tilt_turn_down_default(s.fighter_kind));
        values[AUTO_SPACING_MAXIMUM_DRIFT] = if s.fighter_kind == crate::fighter_kinds::LUCINA { 0.6 } else { 1. };
        Values(values)
    }
    /// Applies saved per-fighter overrides to the runtime assistance settings.
    pub fn apply(self, mut s: AssistSettings) -> AssistSettings {
        let v = self.0;
        let set = |slot: usize, target: &mut f32| {
            if let Some(x) = v[slot] {
                *target = x;
            }
        };
        set(WALK_STRENGTH, &mut s.walk_strength);
        set(GRAB_RANGE, &mut s.fighter.grab.standing_range);
        set(HORIZONTAL_RANGE, &mut s.horizontal_range);
        set(VERTICAL_RANGE, &mut s.vertical_range);
        set(FOLLOW_RANGE, &mut s.follow_range);
        set(FOLLOW_VERTICAL_RANGE, &mut s.follow_vertical_range);
        set(STAND_DISTANCE, &mut s.stand_distance);
        set(MOVE_STRENGTH, &mut s.move_strength);
        if let Some(x) = v[MANUAL_SPACING] {
            s.manual_spacing = x == 1.0;
        }
        if let Some(x) = v[AUTO_POSITION] {
            s.auto_position = x == 1.0;
        }
        if let Some(x) = v[USE_RANGE_LIMITS] {
            s.distance_agnostic = x != 1.0;
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sequence_config::{persist, Catalog};
    /// Startup-loaded per-fighter tuning overrides.
    const SOURCE: &str = "# keep recipes\n[kirby.neutral.nair]\nstart_delay=0\nactions=nair\n[yoshi.neutral.grab]\nstart_delay=0\nactions=grab:fighter_range\n";
    /// Regression test for `saved_overrides_are_per_fighter_and_preserve_recipes`.
    #[test]
    fn saved_overrides_are_per_fighter_and_preserve_recipes() {
        let mut tuning = Tuning::default();
        tuning.set("walk_strength", "0.28").unwrap();
        tuning.set("grab_range", "14.5").unwrap();
        tuning.set("use_range_limits", "true").unwrap();
        let output = persist::update_tuning(SOURCE, 5, tuning).unwrap();
        assert!(output.starts_with(SOURCE));
        let catalog = Catalog::parse(&output).unwrap();
        assert_eq!(catalog.len(), 2);
        assert_eq!(catalog.tuning.get(&5), Some(&tuning));
        assert!(catalog.tuning.get(&6).is_none());
        let recipe = catalog
            .find(
                5,
                crate::fighter_profiles::presets::PresetSlot::Neutral,
                "grab",
            )
            .unwrap();
        let renamed = persist::update_named(
            &output,
            5,
            crate::fighter_profiles::presets::PresetSlot::Neutral,
            "grab",
            recipe,
            false,
            Some("Named grab"),
        )
        .unwrap();
        assert_eq!(Catalog::parse(&renamed).unwrap().tuning[&5], tuning);

        let cleared = persist::update_tuning(&output, 5, Tuning::default()).unwrap();
        assert!(cleared.starts_with(SOURCE));
        assert_eq!(cleared.matches("[yoshi.settings]").count(), 1);
        assert_eq!(
            Catalog::parse(&cleared).unwrap().tuning[&5],
            Tuning::default()
        );
    }
    /// Regression test for `invalid_settings_reject_before_loading`.
    #[test]
    fn invalid_settings_reject_before_loading() {
        for text in [
            "walk_strength=NaN",
            "walk_strength=0.9",
            "grab_range=-1",
            "unknown=4",
            "manual_spacing=1",
            "walk_strength=0.2\nwalk_strength=0.3",
        ] {
            assert!(
                Catalog::parse(&format!("[yoshi.settings]\n{text}")).is_err(),
                "{text}"
            );
        }
        assert!(Catalog::parse("[yoshi.settings]\n[yoshi.settings]\n").is_err());
    }
    #[test]
    fn auto_spacing_overrides_roundtrip_independently() {
        use crate::auto_spacing::Attack;
        let mut tuning = Tuning::default();
        for (key, value) in [
            ("auto_spacing_minimum_distance", "14.5"),
            ("auto_spacing_engagement_range", "23.5"),
            ("auto_spacing_ground_speed", "1"),
            ("auto_spacing_maximum_drift", "0.6"),
            ("auto_spacing_minimum_drift", "0.15"),
            ("auto_spacing_ground_braking", "false"),
            ("auto_spacing_air_braking", "true"),
            ("auto_spacing_braking_distance", "3.5"),
            ("auto_defense_lead_frames", "12"),
            ("auto_spacing_ground_move", "3"),
            ("auto_spacing_ground_startup", "9"),
            ("auto_spacing_ground_active_frames", "2"),
            ("auto_spacing_ground_reach", "18.5"),
            ("auto_spacing_ground_height", "21"),
            ("auto_spacing_ground_commitment", "40"),
            ("auto_spacing_aerial_move", "5"),
            ("auto_spacing_aerial_startup", "7"),
            ("auto_spacing_aerial_active_frames", "4"),
            ("auto_spacing_aerial_reach", "22"),
            ("auto_spacing_aerial_height", "13.5"),
            ("auto_spacing_aerial_commitment", "31"),
        ] {
            tuning.set(key, value).unwrap();
        }
        let output = persist::update_tuning(SOURCE, crate::fighter_kinds::YOSHI, tuning).unwrap();
        let loaded = Catalog::parse(&output).unwrap().tuning[&crate::fighter_kinds::YOSHI];
        assert_eq!(loaded, tuning);
        let config = loaded.auto_spacing(crate::fighter_kinds::YOSHI);
        assert_eq!(config.minimum_distance, 14.5);
        assert_eq!(config.engagement_range, 23.5);
        assert_eq!(config.ground_speed, 1.);
        assert_eq!(config.maximum_drift, 0.6);
        assert_eq!(config.minimum_drift, 0.15);
        assert!(!config.ground_braking);
        assert!(config.air_braking);
        assert_eq!(config.braking_distance, 3.5);
        assert_eq!(loaded.get(AUTO_DEFENSE_LEAD_FRAMES), 12.);
        assert!(matches!(config.ground.attack, Attack::Up));
        assert!(matches!(config.aerial.attack, Attack::Back));
        assert_eq!(
            [
                config.ground.startup,
                config.ground.active_frames,
                config.ground.reach,
                config.ground.height,
                config.ground.commitment
            ],
            [9., 2., 18.5, 21., 40.]
        );
        assert_eq!(
            [
                config.aerial.startup,
                config.aerial.active_frames,
                config.aerial.reach,
                config.aerial.height,
                config.aerial.commitment
            ],
            [7., 4., 22., 13.5, 31.]
        );
        let mut back = Tuning::default();
        back.set("auto_spacing_ground_move", "5").unwrap();
        assert!(matches!(
            back.auto_spacing(crate::fighter_kinds::LUCINA)
                .ground
                .attack,
            Attack::Disabled
        ));
    }
    #[test]
    fn auto_spacing_rejects_invalid_fields() {
        for text in [
            "auto_spacing_ground_move=1.5",
            "auto_spacing_aerial_move=6",
            "auto_spacing_ground_move=-1",
            "auto_spacing_minimum_distance=NaN",
            "auto_spacing_ground_startup=0",
            "auto_spacing_ground_active_frames=301",
            "auto_spacing_aerial_reach=-1",
            "auto_spacing_aerial_height=inf",
            "auto_spacing_aerial_commitment=0",
            "auto_spacing_ground_speed=1.01",
            "auto_spacing_maximum_drift=-0.1",
            "auto_spacing_minimum_drift=NaN",
            "auto_spacing_ground_braking=1",
            "auto_spacing_air_braking=maybe",
            "auto_spacing_braking_distance=-1",
            "auto_spacing_braking_distance=NaN",
            "auto_defense_lead_frames=-1",
            "auto_defense_lead_frames=31",
            "auto_spacing_minimum_distance=12\nauto_spacing_minimum_distance=14",
        ] {
            assert!(
                Catalog::parse(&format!("[yoshi.settings]\n{text}")).is_err(),
                "{text}"
            );
        }
    }
    #[test]
    fn auto_spacing_saved_lucina_values_survive_new_defaults() {
        let catalog = Catalog::parse("[lucina.settings]\nauto_spacing_ground_move=2\nauto_spacing_aerial_move=2\nauto_spacing_minimum_distance=12\nauto_spacing_ground_speed=0\nauto_spacing_maximum_drift=0.45\n").unwrap();
        let tuning = catalog.tuning[&crate::fighter_kinds::LUCINA];
        let config = tuning.auto_spacing(crate::fighter_kinds::LUCINA);
        assert_eq!(config.ground.attack, crate::auto_spacing::Attack::Forward);
        assert_eq!(config.aerial.attack, crate::auto_spacing::Attack::Forward);
        assert_eq!(config.minimum_distance, 12.);
        assert_eq!(config.ground_speed, 0.);
        assert_eq!(config.maximum_drift, 0.45);
        assert_eq!(tuning.get(AUTO_DEFENSE_LEAD_FRAMES), 4.);
        for value in ["0", "30"] {
            let mut changed = tuning;
            changed.set("auto_defense_lead_frames", value).unwrap();
            let source = persist::update_tuning("", crate::fighter_kinds::LUCINA, changed).unwrap();
            assert_eq!(Catalog::parse(&source).unwrap().tuning[&crate::fighter_kinds::LUCINA], changed);
        }
    }

    #[test]
    fn auto_spacing_defaults_resolve_by_fighter_and_match_menu() {
        use crate::auto_spacing::Attack;
        for kind in [
            crate::fighter_kinds::LUCINA,
            crate::fighter_kinds::YOSHI,
            crate::fighter_kinds::KIRBY,
        ] {
            let config = Tuning::default().auto_spacing(kind);
            let menu = Tuning::values(AssistSettings {
                fighter_kind: kind,
                ..AssistSettings::default()
            });
            assert_eq!(config.minimum_distance, menu[AUTO_SPACING_MINIMUM_DISTANCE]);
            assert_eq!(config.maximum_drift, menu[AUTO_SPACING_MAXIMUM_DRIFT]);
            assert_eq!(config.ground_speed, 1.);
            assert_eq!(config.minimum_distance, 15.);
            assert_eq!(config.engagement_range, 20.);
            assert!(config.ground_braking && config.air_braking);
            assert_eq!(config.braking_distance, 10.);
            assert_eq!(config.minimum_drift, 0.1);
            assert_eq!(config.maximum_drift, if kind == crate::fighter_kinds::LUCINA { 0.6 } else { 1. });
            assert!(matches!(config.ground.attack, Attack::Disabled));
            assert!(matches!(config.aerial.attack, Attack::Disabled));
            assert_eq!(menu[AUTO_SPACING_GROUND_MOVE], 0.);
            assert_eq!(menu[AUTO_SPACING_AERIAL_MOVE], 0.);
        }
    }
    /// Regression test for `applied_values_change_decisions_and_inputs_only`.
    #[test]
    fn applied_values_change_decisions_and_inputs_only() {
        let base = AssistSettings::default();
        assert_eq!(
            Tuning::default().apply(base).walk_strength,
            base.walk_strength
        );
        let mut tuning = Tuning::default();
        tuning.set("walk_strength", "0.28").unwrap();
        tuning.set("grab_range", "14.5").unwrap();
        tuning.set("use_range_limits", "true").unwrap();
        let s = tuning.apply(base);
        assert_eq!(s.fighter.grab.standing_range, 14.5);
        assert!(!s.distance_agnostic);
        assert_eq!(
            s.fighter.grab.standing_startup,
            base.fighter.grab.standing_startup
        );
        assert_eq!(s.sequence_overrides, base.sequence_overrides);
        assert_eq!(
            crate::manual_controls::ledge_walk_stick([1.0, -0.6], s.walk_strength),
            [0.26, -0.6]
        );
        assert_eq!(
            crate::manual_controls::ledge_walk_stick([-1.0, 0.0], s.walk_strength),
            [-0.5, 0.0]
        );
        assert_eq!(
            crate::manual_controls::ledge_walk_stick([0.15, 0.0], s.walk_strength),
            [0.15, 0.0]
        );
    }
}
