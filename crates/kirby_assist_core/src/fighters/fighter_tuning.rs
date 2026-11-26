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
        "counter_aim_after_activation",
    ];

    /// Menu labels aligned with the stable tuning-field order.
    pub const LABELS: [&str; COUNT] = [
        "Walking stick %",
        "Grab reach estimate",
        "Ledge target: horizontal search limit (units)",
        "Ledge target: vertical search limit (units)",
        "Ledge target: horizontal tracking limit (units)",
        "Ledge target: vertical tracking limit (units)",
        "Ledge stand distance",
        "Auto-position stick %",
        "Manual walking cap",
        "Auto ledge stance",
        "Use acquire/follow limits",
        "Grab avoidance at startup",
        "Automatic grabs at startup",
        "Grab dodge forecast frames",
        "Shield: horizontal warning gap (units)",
        "Shield: vertical warning gap (units)",
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
        "Kirby crouch: horizontal opponent gap (units)",
        "Kirby crouch: vertical opponent gap (units)",
        "Shield: horizontal scan limit (units)",
        "Shield: vertical scan limit (units)",
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
        "Spacing: retreat below this gap (units)",
        "Spacing: ground attack selection (0-4)",
        "Ground attack: first hit frame",
        "Ground attack: active hit duration (frames)",
        "Ground attack: predicted horizontal reach (units)",
        "Ground attack: vertical tolerance (units)",
        "Spacing ground: commitment frames",
        "Spacing: aerial attack selection (0-5)",
        "Aerial attack: first hit frame",
        "Aerial attack: active hit duration (frames)",
        "Aerial attack: predicted horizontal reach (units)",
        "Aerial attack: vertical tolerance (units)",
        "Spacing aerial: commitment frames",
        "Auto DI (experimental)",
        "Auto SDI (experimental)",
        "Spacing: attack only inside this gap (units)",
        "Spacing: ground walk stick strength (0-1)",
        "Spacing: maximum aerial approach drift (0-1)",
        "Spacing: minimum aerial approach drift (0-1)",
        "Spacing: ground braking",
        "Spacing: air braking",
        "Spacing: braking distance (game units)",
        "Counter: aim after activation",
    ];

    /// Player-facing behavior descriptions aligned with each tuning field.
    pub const HINTS: [&str; COUNT] = [
        "Manual ledge-walking stick scale, not a physics speed change. Larger values give stronger movement; smaller values give gentler movement. The existing curve is slightly lower at partial stick and 50% at full stick. Recipes keep their own strengths; auto spacing uses its separate ground cap.",
        "Estimated grab reach in game units for recipes and chase helpers. Increase to attempt grabs farther away; decrease to require closer contact. Saved attacker estimates also affect grab warnings. Never extends native hitboxes.",
        "Horizontal opponent-search limit in game units with range limits ON. Increase to find farther opponents; decrease to require closer opponents.",
        "Vertical opponent-search limit in game units with range limits ON. Increase to find opponents on higher/lower platforms; decrease to require similar heights.",
        "Horizontal tracking limit after selecting an opponent, in game units with range limits ON. Increase to keep a farther target; decrease to release it sooner.",
        "Vertical tracking limit after selecting an opponent, in game units with range limits ON. Increase to tolerate height separation; decrease to release the target sooner.",
        "Ledge-mode stop distance from the tracked ledge in game units; also the automatic stance offset. Increase to stand farther inward; decrease to stand nearer the edge. Does not change auto-spacing's opponent gap.",
        "Controller stick strength for automatic ledge positioning. Increase to move faster; decrease for gentler positioning. Does not alter native speed or recipe movement.",
        "ON limits manual walking while L is held in ledge mode. OFF leaves that movement uncapped. This is separate from auto-spacing's ground stick cap.",
        "ON positions you at the configured ledge stance when manual walking cap is OFF and acquire/follow limits are ON. OFF leaves positioning manual. Recipes keep their own movement.",
        "ON uses the four horizontal/vertical search and tracking limits above. OFF tracks without those distance cutoffs; it does not extend attack reach.",
        "ON starts grab avoidance and its Auto react follow-up enabled when settings load; OFF starts them disabled. Configure the response under Auto react. Suppressed while auto spacing runs; does not change native dodge timing.",
        "ON starts automatic grabs and shield backup enabled when settings load; OFF starts them disabled. Configure their distances/actions under Auto react. Suppressed while auto spacing runs.",
        "Frames ahead to forecast ordinary grab contact (1-8). Default 6; larger forecasts can dodge earlier, not guarantee success.",
        "Horizontal opponent-gap warning limit in game units for verified incoming attacks. Increase to consider farther threats; decrease for close threats only. Does not define your attack reach.",
        "Vertical opponent-gap warning limit in game units. Increase to consider attacks across larger height differences; decrease for similar heights only.",
        "Estimated body half-width in game units for incoming hitbox tests. Increase for more cautious shielding; decrease for tighter collision estimates. Does not change your hurtbox.",
        "Estimated standing body height above your root in game units. Increase to consider higher incoming hits; decrease for a shorter estimate. Does not change your hurtbox.",
        "Estimated body half-depth in game units for shield prediction. Increase to include more off-plane threats; decrease to reject them sooner. Does not change native geometry.",
        "Game frames ahead to predict approach, attack startup and active hits. Increase to shield earlier; decrease to react closer to contact. Longer prediction can be less accurate.",
        "Extra game frames to keep defensive shield held after a threat warning ends. Increase for a longer safety tail; decrease to release sooner. New grab threats clear the old tail.",
        "Extra horizontal body allowance in game units added to the opponent's grab reach. Increase to dodge more cautiously; decrease for closer predictions. Does not extend grabs.",
        "Maximum vertical root gap in game units for predicted grabs. Increase to consider wider height differences; decrease to ignore grabs from different heights.",
        "Horizontal collision-probe offset from your root in game units. Increase to check a wider body; decrease for narrower checks. Never changes native tech eligibility.",
        "Floor-probe starting height above your root in game units. Increase to start farther above the floor; decrease to start closer. Affects predicted collision timing, not native tech windows.",
        "Lowest wall-probe height above your root in game units. Increase to test higher wall contact; decrease to test lower contact.",
        "Middle wall-probe height above your root in game units. Increase/decrease to shift the middle wall-contact sample up/down.",
        "Highest wall-probe height above your root in game units. Increase to detect higher wall contact; decrease for lower contact.",
        "Ceiling-probe starting height above your root in game units. Increase to start nearer overhead surfaces; decrease to start lower.",
        "Extra downward floor-probe reach in game units. Increase to detect floor contacts farther below; decrease for tighter contact prediction. Does not allow techs outside native eligibility.",
        "Extra upward ceiling-probe reach in game units. Increase to detect ceilings farther above; decrease for tighter contact prediction. Does not extend native tech windows.",
        "Root-based reach for legacy spot-punish and legacy roll down-smash; recipes retain their own ranges.",
        "Root-based forward-smash reach for legacy automatic punish selection.",
        "Vertical root gap used by the legacy spot-dodge punish.",
        "Height tolerance for the Kirby/Yoshi legacy roll smash reader.",
        "Maximum root distance for watching legacy roll-smash opportunities.",
        "Height tolerance for the Kirby/Yoshi legacy roll grab reader.",
        "Maximum root distance for watching legacy roll-grab opportunities.",
        "Horizontal opponent-gap limit in game units for Kirby automatic crouch. Increase to crouch around farther opponents; decrease for closer opponents only.",
        "Vertical opponent-gap limit in game units for Kirby automatic crouch. Increase to include larger height differences; decrease for similar heights.",
        "Horizontal gap limit in game units for estimated startup/recovery warnings. Increase to watch farther opponents; decrease for close threats. Active hitboxes are checked separately.",
        "Vertical gap limit in game units for estimated startup/recovery warnings. Increase to watch opponents at different heights; decrease for similar heights. Active hitboxes are checked separately.",
        "Standing-grab distance limit in game units while you shield. Increase for farther grab attempts; decrease to wait until closer. Requires a vulnerable grabbable opponent in front; does not dash out of shield.",
        "Approach-grab distance limit in game units when not shielding. Increase for farther attempts; decrease for closer ones. Uses one full-stick approach pulse then grab against a shielding opponent, never a dash out of shield.",
        "ON enables this fighter's supported projectile reflection/absorption helper; OFF disables it. Unsupported fighters do not receive a reflector. Suppressed while auto spacing runs.",
        "Game frames until the reflector/absorber becomes active. Increase to request earlier before projectile contact; decrease only if real startup is faster. Game & Watch remains at least six native frames.",
        "Extra game frames of early warning beyond predicted contact and native startup. Increase to begin reflection earlier; decrease to react later. No extra waiting action is inserted.",
        "Maximum game frames to hold one confirmed reflector. Increase for a longer hold; decrease to release sooner. Missing threats, capture and interruptions can release earlier.",
        "Estimated body half-width in game units for projectile prediction. Increase for more cautious defense; decrease for tighter collision estimates. Does not change your hurtbox.",
        "Estimated body height above your root in game units for projectile prediction. Increase to include higher projectiles; decrease for a shorter estimate.",
        "Estimated body half-depth in game units for projectile prediction. Increase to include more off-plane projectiles; decrease to reject them sooner. Does not change native geometry.",
        "ON prepares manual down tilt facing toward the tracked opponent; OFF keeps your own facing. Kirby and Yoshi default ON. This does not change move startup or auto-spacing's selected move.",
        "ON aims manual forward C-stick tilts toward the tracked opponent; OFF keeps your direction. Requires Tilt Attack C-stick mapping. Does not change attack reach or startup.",
        "ON faces manual up C-stick tilts toward the tracked opponent; OFF keeps your own facing. Requires Tilt Attack C-stick mapping. Does not change attack reach or startup.",
        "Enable the configured counter adapter for supported fighters only (currently Lucina). Default activation is hold Y; normal Y jump is suppressed during that hold. A detected incoming threat is still required.",
        "Estimated frames before the native counter becomes active. Increase to request earlier; decrease only if native startup is actually faster. This never changes the move's real startup.",
        "Extra forecast frames before predicted incoming attack/projectile contact. Increase for earlier reactions; decrease to react later. Does not enable counters without a detected threat.",
        "Estimated body half-width in game units for counter threat tests. Increase for more cautious counter requests; decrease for tighter collision estimates. Does not change your hurtbox.",
        "Estimated body height above your root in game units for counter threat tests. Increase to include higher hits; decrease for a shorter estimate.",
        "Estimated body half-depth in game units for counter threat tests. Increase to include more off-plane hits; decrease to reject them sooner. Does not change native geometry.",
        "Retreat boundary: horizontal fighter-root gap in game units. Increase to back away sooner; decrease to tolerate closer opponents. A predicted in-range attack takes priority over retreat. Select spacing with L mode swap, then hold L; approach stays manual.",
        "0 OFF; 1 jab; 2 forward tilt; 3 up tilt; 4 down tilt; 5 invalid/OFF. Select the move whose timing and distance estimates you enter below. Repeats as soon as native recovery allows. All fighters honor this selection.",
        "Game frames from the ground attack input to its first active hit frame, plus observed input delay. A larger estimate forecasts contact farther ahead; a smaller one forecasts sooner. Enter the move's real startup, not a desired speed: this never speeds up the move.",
        "Number of active ground-hit frames starting at the first hit frame. Increase to allow more predicted contact times; decrease for a narrower timing window. Overestimating may cause misses. Does not extend native hitboxes.",
        "Maximum predicted horizontal root-to-root gap for a ground hit, in game units. Increase to attempt farther hits; decrease to wait for closer contact. Also limited by attack engagement range. Calibrate for the selected move; not exact hitbox size.",
        "Maximum predicted vertical root gap during a ground hit, in game units. Increase to attempt larger height differences; decrease for tighter matching. Up tilt requires the opponent at/above you; down tilt at/below you.",
        "Retired estimate retained only for saved-file compatibility; ignored by spacing and hidden from the menu. Native recovery/cancel readiness decides when another attack is allowed.",
        "0 OFF; 1 neutral air; 2 forward air; 3 up air; 4 down air; 5 back air. Uses the estimates below after you jump, fall or leave a platform; never jumps for you. Forward/back air require the correct native facing.",
        "Game frames from aerial input to its first active hit frame, plus observed input delay. A larger estimate checks contact farther into your fall/rise; a smaller one checks sooner. Gravity is included when observed. Enter real startup; this does not alter native timing.",
        "Number of active aerial-hit frames starting at the first hit frame. Increase to allow a longer predicted contact window; decrease to require tighter timing. Overestimating may cause early attacks that miss.",
        "Maximum predicted horizontal root gap for an aerial hit, in game units. Increase to attempt farther hits; decrease to wait until closer. Also limited by current attack engagement range. Does not extend attack reach.",
        "Maximum predicted vertical root gap for an aerial hit, in game units. Increase to attack earlier across larger height differences; decrease to wait for closer heights. Up air requires the opponent at/above you; down air at/below you.",
        "Retired estimate retained only for saved-file compatibility; ignored by spacing and hidden from the menu. Native aerial recovery/cancel readiness controls repetition; no extra cooldown is added.",
        "Experimental input-only DI with observed blast bounds; not terrain-aware or guaranteed survival. Default OFF.",
        "Experimental input-only SDI with observed blast bounds and attacker direction. Default OFF.",
        "Current horizontal root-distance limit in game units: no attack starts or retries outside it, even if the opponent is approaching. Increase to allow farther attempts; decrease to wait until closer. Each move must also predict a hit within its own reach/height. Zero disables attacks; approach remains manual.",
        "Ground movement cap from 0 to 1 while spacing is active, also bounded by the native non-dash threshold. Increase to walk faster; decrease to walk slower. Manual controls outside spacing are unchanged.",
        "Retired automatic-approach drift setting: retained for file compatibility, ignored and hidden. Approach is manual; automatic retreat uses full directional drift.",
        "Retired approach-taper setting: retained for file compatibility, ignored and hidden. Manual aerial drift is not tapered by spacing.",
        "Retired approach-braking switch: retained for file compatibility, ignored and hidden. Ground movement uses its separate configured stick cap.",
        "Retired aerial-braking switch: retained for file compatibility, ignored and hidden. Manual approach drift is not braked by spacing.",
        "Retired approach-braking distance: retained for file compatibility, ignored and hidden. The retreat boundary remains configurable above.",
        "Lucina only: aim toward the tracked attacker after native counter stance starts, stopping before retaliation. Controller input only; native turnaround is not guaranteed.",
    ];

    pub const AUTO_SPACING_ENGAGEMENT_RANGE: usize = 75;
    pub const AUTO_SPACING_GROUND_SPEED: usize = 76;
    pub const AUTO_SPACING_MAXIMUM_DRIFT: usize = 77;
    pub const AUTO_SPACING_MINIMUM_DRIFT: usize = 78;
    pub const AUTO_SPACING_GROUND_BRAKING: usize = 79;
    pub const AUTO_SPACING_AIR_BRAKING: usize = 80;
    pub const AUTO_SPACING_BRAKING_DISTANCE: usize = 81;
    pub const COUNTER_AIM_AFTER_ACTIVATION: usize = 82;

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
    d[AUTO_SPACING_GROUND_SPEED] = 0.49;
    d[AUTO_SPACING_GROUND_BRAKING] = 1.;
    d[AUTO_SPACING_AIR_BRAKING] = 1.;
    d[AUTO_SPACING_BRAKING_DISTANCE] = 10.;
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
            COUNTER_AIM_AFTER_ACTIVATION => kind == crate::fighter_kinds::LUCINA,
            PUNISH_DOWN_REACH..=ROLL_GRAB_WATCH => false,
            AUTO_SPACING_MAXIMUM_DRIFT..=AUTO_SPACING_BRAKING_DISTANCE
                | AUTO_SPACING_GROUND_COMMITMENT | AUTO_SPACING_AERIAL_COMMITMENT => false,
            AUTO_SPACING_MINIMUM_DISTANCE | AUTO_SPACING_ENGAGEMENT_RANGE => true,
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
            || i == COUNTER_AIM_AFTER_ACTIVATION
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
            maximum_drift: self.get(AUTO_SPACING_MAXIMUM_DRIFT),
            minimum_drift: self.get(AUTO_SPACING_MINIMUM_DRIFT),
            ground_braking: self.get(AUTO_SPACING_GROUND_BRAKING) == 1.,
            air_braking: self.get(AUTO_SPACING_AIR_BRAKING) == 1.,
            braking_distance: self.get(AUTO_SPACING_BRAKING_DISTANCE),
            fighter_kind: kind,
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
        if key == "auto_defense_lead_frames" {
            let value = text.parse::<f32>().map_err(|_| format!("{key}: expected a number"))?;
            return if value.is_finite() && (0. ..=30.).contains(&value) {
                Ok(())
            } else {
                Err(format!("{key}: must be 0..30"))
            };
        }
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
        values[AUTO_SPACING_MAXIMUM_DRIFT] = 1.;
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
#[path = "../../tests/unit/fighter_tuning_overrides.rs"]
mod tests;
