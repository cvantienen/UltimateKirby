//! Stable menu and file schema for per-fighter input-policy overrides.
//!
//! The index order is shared by saved configuration, menu rendering, and
//! runtime policy values. Keep each key, label, and hint at the same index.
pub const COUNT: usize = 55;
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
    "jab_lock_reach",
    "jab_lock_smash_reach",
    "jab_lock_chase_distance",
    "jab_lock_height",
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
];

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
    "Yoshi jab lock: jab reach",
    "Yoshi jab lock: smash reach",
    "Yoshi jab lock: chase distance",
    "Yoshi jab lock: chase height",
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
];

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
    "Root-based jab-lock reach estimate; affects only the existing Yoshi helper.",
    "Root-based smash punish reach estimate for the Yoshi tech chase.",
    "Maximum root separation to start/continue the Yoshi tech chase.",
    "Maximum root-height difference for Yoshi tech-chase candidates.",
    "Maximum horizontal root gap for Kirby automatic crouch.",
    "Maximum vertical root gap for Kirby automatic crouch.",
    "Coarse root-distance cutoff before bounded native hitbox reads; enlarged by relative speed.",
    "Coarse root-height cutoff before bounded native hitbox reads; enlarged by relative speed.",
    "No-dash grab limit while you shield. A vulnerable grabbable opponent in front qualifies; enemy shield is not required.",
    "Approach-grab limit when not shielding: one full-stick pulse toward a shielding opponent, then R. Never used for a dash out of shield.",
    "Explicit fighter-specific projectile defense. Unsupported fighters never inherit a reflector.",
    "Frames before reflection/absorption begins; cannot reduce Game & Watch below his native six-frame startup.",
    "Start this many frames before predicted contact plus native startup; no waiting action is inserted.",
    "Bound on holding one confirmed reflector; vanished threats, capture and other interruptions release sooner.",
    "Estimated body half-width for incoming projectile prediction; does not change native geometry.",
    "Estimated body height above the root for incoming projectile prediction.",
    "Estimated depth allowance for incoming projectile prediction; off-plane attacks are ignored.",
];

pub const REFLECT_DEFAULT: usize = 48;
pub const REFLECT_STARTUP: usize = 49;
pub const REFLECT_MARGIN: usize = 50;
pub const REFLECT_MAX_HOLD: usize = 51;
pub const REFLECT_HALF_WIDTH: usize = 52;
pub const REFLECT_HEIGHT: usize = 53;
pub const REFLECT_HALF_DEPTH: usize = 54;
pub const SHIELD_SCAN_X: usize = 44;
pub const SHIELD_SCAN_Y: usize = 45;
pub const AUTO_GRAB_CLOSE: usize = 46;
pub const AUTO_GRAB_FAR: usize = 47;
pub const SHIELD_WARNING_X: usize = 14;
pub const SHIELD_WARNING_Y: usize = 15;
pub const SHIELD_BODY_HALF_WIDTH: usize = 16;
pub const SHIELD_BODY_HEIGHT: usize = 17;
pub const SHIELD_BODY_HALF_DEPTH: usize = 18;
pub const SHIELD_FORECAST: usize = 19;
pub const SHIELD_HOLD: usize = 20;
pub const GRAB_DODGE_PADDING: usize = 21;
pub const GRAB_DODGE_HEIGHT: usize = 22;
pub const TECH_HALF_WIDTH: usize = 23;
pub const TECH_FLOOR_ORIGIN: usize = 24;
pub const TECH_WALL_LOW: usize = 25;
pub const TECH_WALL_MID: usize = 26;
pub const TECH_WALL_HIGH: usize = 27;
pub const TECH_CEILING_ORIGIN: usize = 28;
pub const TECH_FLOOR_PADDING: usize = 29;
pub const TECH_CEILING_PADDING: usize = 30;
pub const PUNISH_DOWN_REACH: usize = 31;
pub const PUNISH_FORWARD_REACH: usize = 32;
pub const PUNISH_HEIGHT: usize = 33;
pub const ROLL_SMASH_HEIGHT: usize = 34;
pub const ROLL_SMASH_WATCH: usize = 35;
pub const ROLL_GRAB_HEIGHT: usize = 36;
pub const ROLL_GRAB_WATCH: usize = 37;
pub const JAB_LOCK_REACH: usize = 38;
pub const JAB_LOCK_SMASH_REACH: usize = 39;
pub const JAB_LOCK_CHASE_DISTANCE: usize = 40;
pub const JAB_LOCK_HEIGHT: usize = 41;
pub const CROUCH_RANGE: usize = 42;
pub const CROUCH_HEIGHT: usize = 43;
