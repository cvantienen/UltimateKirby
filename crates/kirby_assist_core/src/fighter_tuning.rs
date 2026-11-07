//! Input-policy overrides only. Never writes native fighter parameters.
use crate::ledge_trap::LedgeSettings;
mod analysis;
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
    "auto_grab_close_range", "auto_grab_dash_range",
    "reflect_default", "reflect_startup", "reflect_margin", "reflect_max_hold",
    "reflect_half_width", "reflect_height", "reflect_half_depth",
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
    "Automatic grab: shielded range", "Automatic grab: approach range",
    "Auto-reflect at startup", "Reflect: startup frames", "Reflect: early margin",
    "Reflect: maximum hold frames", "Reflect: body half-width", "Reflect: body height",
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
    "No-dash grab limit while you shield. A vulnerable grabbable opponent in front qualifies; enemy shield is not required.", "Approach-grab limit when not shielding: one full-stick pulse toward a shielding opponent, then R. Never used for a dash out of shield.",
    "Explicit fighter-specific projectile defense. Unsupported fighters never inherit a reflector.",
    "Frames before reflection/absorption begins; cannot reduce Game & Watch below his native six-frame startup.",
    "Start this many frames before predicted contact plus native startup; no waiting action is inserted.",
    "Bound on holding one confirmed reflector; vanished threats, capture and other interruptions release sooner.",
    "Estimated body half-width for incoming projectile prediction; does not change native geometry.",
    "Estimated body height above the root for incoming projectile prediction.",
    "Estimated depth allowance for incoming projectile prediction; off-plane attacks are ignored.",
];
pub const REFLECT_DEFAULT:usize=48;
pub const REFLECT_STARTUP:usize=49;
pub const REFLECT_MARGIN:usize=50;
pub const REFLECT_MAX_HOLD:usize=51;
pub const REFLECT_HALF_WIDTH:usize=52;
pub const REFLECT_HEIGHT:usize=53;
pub const REFLECT_HALF_DEPTH:usize=54;
pub const SHIELD_SCAN_X: usize = 44;
pub const SHIELD_SCAN_Y: usize = 45;
pub const AUTO_GRAB_CLOSE:usize=46;
pub const AUTO_GRAB_FAR:usize=47;
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tuning(pub [Option<f32>; COUNT]);
#[derive(Clone,Copy)]
pub struct Values(pub [f32;COUNT]);
impl Default for Values {fn default()->Self{Self([0.;COUNT])}}
impl std::ops::Deref for Values {type Target=[f32;COUNT];fn deref(&self)->&Self::Target{&self.0}}
impl std::ops::DerefMut for Values {fn deref_mut(&mut self)->&mut Self::Target{&mut self.0}}
impl Default for Tuning { fn default() -> Self { Self([None; COUNT]) } }
impl Tuning {
    pub fn value(&self, i: usize, fallback: f32) -> f32 { self.0[i].unwrap_or(fallback) }
    pub fn frames(i: usize) -> bool { matches!(i, 13 | SHIELD_FORECAST | SHIELD_HOLD | REFLECT_STARTUP | REFLECT_MARGIN | REFLECT_MAX_HOLD) }
    pub fn visible_index(kind:i32,row:usize)->usize {
        (row..COUNT).find(|i|Self::visible(kind,*i))
            .or_else(||(0..COUNT).rev().find(|i|Self::visible(kind,*i))).unwrap_or(0)
    }
    pub fn visible(kind: i32, i: usize) -> bool {
        if i>=COUNT {return false;}
        if i>=REFLECT_DEFAULT {return crate::projectile_reflect::supports(kind);}
        // Retired fixed punish helpers: accept old file keys, hide inert UI.
        if (PUNISH_DOWN_REACH..=ROLL_GRAB_WATCH).contains(&i) { return false; }
        if (JAB_LOCK_REACH..=JAB_LOCK_HEIGHT).contains(&i) { return kind==5; }
        if (CROUCH_RANGE..=CROUCH_HEIGHT).contains(&i) { return kind==6; }
        true
    }
    pub fn is_bool(i: usize) -> bool { (8..=12).contains(&i) || i==REFLECT_DEFAULT }
    pub fn grab_dodge_default(self, base: bool) -> bool { self.0[11].map_or(base, |x| x == 1.0) }
    pub fn oos_default(self) -> bool { self.0[12].unwrap_or(1.0) == 1.0 }
    pub fn dodge_lookahead(self) -> f32 { self.0[13].unwrap_or(crate::spot_dodge::LOOKAHEAD) }
    pub fn bounds(i: usize) -> (f32, f32, f32) {
        match i {
            REFLECT_DEFAULT => (0.,1.,1.),
            REFLECT_STARTUP => (6.,30.,1.),
            REFLECT_MARGIN => (0.,8.,1.),
            REFLECT_MAX_HOLD => (6.,120.,1.),
            REFLECT_HALF_WIDTH | REFLECT_HALF_DEPTH => (1.,30.,0.5),
            REFLECT_HEIGHT => (1.,60.,0.5),
            8..=12 => (0.0, 1.0, 1.0),
            13 => (1.0, 8.0, 0.5),
            0 => (0.1, 0.5, 0.01),
            7 => (0.1, 1.0, 0.01),
            1 => (1.0, 60.0, 0.5),
            SHIELD_WARNING_X => (0.0, 100.0, 0.5),
            SHIELD_WARNING_Y => (0.0, 100.0, 0.5),
            SHIELD_BODY_HALF_WIDTH => (0.0, 40.0, 0.5),
            SHIELD_BODY_HEIGHT => (0.0, 60.0, 0.5),
            SHIELD_BODY_HALF_DEPTH => (0.0, 40.0, 0.5),
            SHIELD_FORECAST => (0.0, 10.0, 0.5),
            SHIELD_HOLD => (0.0, 20.0, 0.5),
            GRAB_DODGE_PADDING => (0.0, 30.0, 0.5),
            GRAB_DODGE_HEIGHT => (0.0, 60.0, 0.5),
            TECH_HALF_WIDTH => (0.0, 30.0, 0.5),
            TECH_FLOOR_ORIGIN => (0.0, 40.0, 0.5),
            TECH_WALL_LOW => (0.0, 40.0, 0.5),
            TECH_WALL_MID => (0.0, 40.0, 0.5),
            TECH_WALL_HIGH => (0.0, 60.0, 0.5),
            TECH_CEILING_ORIGIN => (0.0, 60.0, 0.5),
            TECH_FLOOR_PADDING => (0.0, 15.0, 0.5),
            TECH_CEILING_PADDING => (0.0, 15.0, 0.5),
            PUNISH_DOWN_REACH => (0.0, 100.0, 0.5),
            PUNISH_FORWARD_REACH => (0.0, 100.0, 0.5),
            PUNISH_HEIGHT => (0.0, 60.0, 0.5),
            ROLL_SMASH_HEIGHT => (0.0, 60.0, 0.5),
            ROLL_SMASH_WATCH => (0.0, 150.0, 0.5),
            ROLL_GRAB_HEIGHT => (0.0, 60.0, 0.5),
            ROLL_GRAB_WATCH => (0.0, 150.0, 0.5),
            JAB_LOCK_REACH => (0.0, 60.0, 0.5),
            JAB_LOCK_SMASH_REACH => (0.0, 100.0, 0.5),
            JAB_LOCK_CHASE_DISTANCE => (0.0, 150.0, 0.5),
            JAB_LOCK_HEIGHT => (0.0, 100.0, 0.5),
            CROUCH_RANGE => (0.0, 150.0, 0.5),
            CROUCH_HEIGHT => (0.0, 100.0, 0.5),
            SHIELD_SCAN_X | SHIELD_SCAN_Y => (1.0, 300.0, 1.0),
            _ => (1.0, 150.0, 0.5),
        }
    }
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
        self.0[i] = Some(value);
        Ok(())
    }
    pub fn values(s: LedgeSettings) -> Values {
        Values([
            s.walk_strength,
            s.fighter.grab.standing_range,
            s.horizontal_range,
            s.vertical_range,
            s.follow_range,
            s.follow_vertical_range,
            s.stand_distance,
            s.move_strength,
            if s.manual_spacing { 1.0 } else { 0.0 },
            if s.auto_position { 1.0 } else { 0.0 },
            if s.distance_agnostic { 0.0 } else { 1.0 },
            1.0, 1.0, crate::spot_dodge::LOOKAHEAD,
            24.0,
            28.0,
            6.0,
            18.0,
            6.0,
            5.0,
            4.0,
            2.0,
            8.0,
            2.5,
            1.0,
            1.0,
            3.5,
            6.0,
            5.0,
            1.5,
            2.0,
            if s.fighter_kind==6 {14.0} else if s.fighter_kind==5 {13.0} else {12.0},
            if s.fighter_kind==6 {22.0} else {21.0},
            8.0,
            6.0,
            64.0,
            4.0,
            if s.fighter_kind==6 {32.0} else {36.0},
            7.0,
            12.0,
            60.0,
            24.0,
            45.0,
            20.0,
            96.0,96.0,
            analysis::default_auto_grab_ranges(s.fighter_kind,s.fighter.grab.standing_range).0,
            analysis::default_auto_grab_ranges(s.fighter_kind,s.fighter.grab.standing_range).1,
            0.,6.,2.,45.,5.,16.,5.,
        ])
    }
    pub fn apply(self, mut s: LedgeSettings) -> LedgeSettings {
        let v = self.0;
        if let Some(x) = v[0] {
            s.walk_strength = x;
        }
        if let Some(x) = v[1] {
            s.grab_range = x;
            s.fighter.grab.standing_range = x;
        }
        if let Some(x) = v[2] {
            s.horizontal_range = x;
        }
        if let Some(x) = v[3] {
            s.vertical_range = x;
        }
        if let Some(x) = v[4] {
            s.follow_range = x;
        }
        if let Some(x) = v[5] {
            s.follow_vertical_range = x;
        }
        if let Some(x) = v[6] {
            s.stand_distance = x;
        }
        if let Some(x) = v[7] {
            s.move_strength = x;
        }
        if let Some(x) = v[8] {
            s.manual_spacing = x == 1.0;
        }
        if let Some(x) = v[9] {
            s.auto_position = x == 1.0;
        }
        if let Some(x) = v[10] {
            s.distance_agnostic = x != 1.0;
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sequence_config::{Catalog, persist};
    const SOURCE: &str = "# keep recipes\n[kirby.neutral.nair]\nstart_delay=0\nactions=nair\n[yoshi.neutral.grab]\nstart_delay=0\nactions=grab:fighter_range\n";
    #[test]
    fn saved_overrides_are_per_fighter_and_preserve_recipes() {
        let mut tuning=Tuning::default();
        tuning.set("walk_strength","0.28").unwrap();
        tuning.set("grab_range","14.5").unwrap();
        tuning.set("use_range_limits","true").unwrap();
        let output=persist::update_tuning(SOURCE,5,tuning).unwrap();
        assert!(output.starts_with(SOURCE));
        let catalog=Catalog::parse(&output).unwrap();
        assert_eq!(catalog.len(),2);
        assert_eq!(catalog.tuning.get(&5),Some(&tuning));
        assert!(catalog.tuning.get(&6).is_none());
        let recipe=catalog.find(5,crate::fighter_profiles::presets::PresetSlot::Neutral,"grab").unwrap();
        let renamed=persist::update_named(&output,5,crate::fighter_profiles::presets::PresetSlot::Neutral,"grab",recipe,false,Some("Named grab")).unwrap();
        assert_eq!(Catalog::parse(&renamed).unwrap().tuning[&5],tuning);

        let cleared=persist::update_tuning(&output,5,Tuning::default()).unwrap();
        assert!(cleared.starts_with(SOURCE));
        assert_eq!(cleared.matches("[yoshi.settings]").count(),1);
        assert_eq!(Catalog::parse(&cleared).unwrap().tuning[&5],Tuning::default());
    }
    #[test]
    fn invalid_settings_reject_before_loading() {
        for text in ["walk_strength=NaN", "walk_strength=0.9", "grab_range=-1", "unknown=4", "manual_spacing=1", "walk_strength=0.2\nwalk_strength=0.3"] {
            assert!(Catalog::parse(&format!("[yoshi.settings]\n{text}")).is_err(),"{text}");
        }
        assert!(Catalog::parse("[yoshi.settings]\n[yoshi.settings]\n").is_err());
    }
    #[test]
    fn applied_values_change_decisions_and_inputs_only() {
        let base=LedgeSettings::default();
        assert_eq!(Tuning::default().apply(base).walk_strength,base.walk_strength);
        let mut tuning=Tuning::default();
        tuning.set("walk_strength","0.28").unwrap();
        tuning.set("grab_range","14.5").unwrap();
        tuning.set("use_range_limits","true").unwrap();
        let s=tuning.apply(base);
        assert_eq!(s.fighter.grab.standing_range,14.5);
        assert_eq!(s.grab_range,14.5);
        assert!(!s.distance_agnostic);
        assert_eq!(s.fighter.grab.standing,base.fighter.grab.standing);
        assert_eq!(s.sequence_overrides,base.sequence_overrides);
        assert_eq!(crate::manual_controls::ledge_walk_stick([1.0,-0.6],s.walk_strength),[0.26,-0.6]);
        assert_eq!(crate::manual_controls::ledge_walk_stick([-1.0,0.0],s.walk_strength),[-0.5,0.0]);
        assert_eq!(crate::manual_controls::ledge_walk_stick([0.15,0.0],s.walk_strength),[0.15,0.0]);
    }
}
