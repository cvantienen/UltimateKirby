//! Input-policy overrides only. Never writes native fighter parameters.
//! `schema.rs` owns the stable menu/file fields; `analysis.rs` groups the
//! fighter-derived defaults and policy conversions.
use crate::settings::AssistSettings;
mod analysis;
mod schema;

pub const MIN_GAMEWATCH_REFLECT_STARTUP_FRAMES: f32 = 6.0;
pub const MIN_GRAB_DODGE_LOOKAHEAD_FRAMES: f32 = 1.0;
pub const MAX_GRAB_DODGE_LOOKAHEAD_FRAMES: f32 = 8.0;
pub const DEFAULT_GRAB_DODGE_PADDING: f32 = 2.0;
pub const MAX_GRAB_DODGE_PADDING: f32 = 30.0;
pub const DEFAULT_GRAB_DODGE_HEIGHT: f32 = 8.0;
pub const MAX_GRAB_DODGE_HEIGHT: f32 = 60.0;
pub const DEFAULT_REFLECT_MARGIN_FRAMES: f32 = 2.0;
pub const DEFAULT_REFLECT_MAX_HOLD_FRAMES: f32 = 45.0;
pub const DEFAULT_REFLECT_HALF_WIDTH: f32 = 5.0;
pub const DEFAULT_REFLECT_HEIGHT: f32 = 16.0;
pub const DEFAULT_REFLECT_HALF_DEPTH: f32 = 5.0;

pub use schema::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tuning(pub [Option<f32>; COUNT]);
#[derive(Clone, Copy)]
pub struct Values(pub [f32; COUNT]);
impl Default for Values {
    fn default() -> Self {
        Self([0.; COUNT])
    }
}
impl std::ops::Deref for Values {
    type Target = [f32; COUNT];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Values {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Default for Tuning {
    fn default() -> Self {
        Self([None; COUNT])
    }
}
impl Tuning {
    pub fn value(&self, i: usize, fallback: f32) -> f32 {
        self.0[i].unwrap_or(fallback)
    }
    pub fn frames(i: usize) -> bool {
        matches!(
            i,
            GRAB_DODGE_LOOKAHEAD
                | SHIELD_FORECAST
                | SHIELD_HOLD
                | REFLECT_STARTUP
                | REFLECT_MARGIN
                | REFLECT_MAX_HOLD
        )
    }
    pub fn visible_index(kind: i32, row: usize) -> usize {
        (row..COUNT)
            .find(|i| Self::visible(kind, *i))
            .or_else(|| (0..COUNT).rev().find(|i| Self::visible(kind, *i)))
            .unwrap_or(WALK_STRENGTH)
    }
    pub fn visible(kind: i32, i: usize) -> bool {
        crate::fighter_tuning_defaults::visible(kind, i)
    }
    pub fn is_bool(i: usize) -> bool {
        (MANUAL_SPACING..=OUT_OF_SHIELD_DEFAULT).contains(&i)
            || i == REFLECT_DEFAULT
            || (TILT_TURN_DOWN..=TILT_TURN_UP).contains(&i)
    }
    pub fn tilt_turn(self, kind: i32) -> crate::posture::TiltSettings {
        crate::posture::TiltSettings {
            down: self.value(
                TILT_TURN_DOWN,
                if crate::fighter_tuning_defaults::tilt_turn_down(kind) {
                    1.0
                } else {
                    0.0
                },
            ) == 1.0,
            forward: self.value(TILT_TURN_FORWARD, 0.0) == 1.0,
            up: self.value(TILT_TURN_UP, 0.0) == 1.0,
        }
    }
    pub fn grab_dodge_default(self, base: bool) -> bool {
        self.0[GRAB_DODGE_DEFAULT].map_or(base, |x| x == 1.0)
    }
    pub fn oos_default(self) -> bool {
        self.0[OUT_OF_SHIELD_DEFAULT].unwrap_or(1.0) == 1.0
    }
    pub fn dodge_lookahead(self) -> f32 {
        self.0[GRAB_DODGE_LOOKAHEAD].unwrap_or(crate::spot_dodge::LOOKAHEAD)
    }
    pub fn bounds(i: usize) -> (f32, f32, f32) {
        match i {
            REFLECT_DEFAULT | TILT_TURN_DOWN | TILT_TURN_FORWARD | TILT_TURN_UP => (0., 1., 1.),
            REFLECT_STARTUP => (6., 30., 1.),
            REFLECT_MARGIN => (0., 8., 1.),
            REFLECT_MAX_HOLD => (6., 120., 1.),
            REFLECT_HALF_WIDTH | REFLECT_HALF_DEPTH => (1., 30., 0.5),
            REFLECT_HEIGHT => (1., 60., 0.5),
            MANUAL_SPACING..=OUT_OF_SHIELD_DEFAULT => (0.0, 1.0, 1.0),
            GRAB_DODGE_LOOKAHEAD => (
                MIN_GRAB_DODGE_LOOKAHEAD_FRAMES,
                MAX_GRAB_DODGE_LOOKAHEAD_FRAMES,
                0.5,
            ),
            WALK_STRENGTH => (0.1, 0.5, 0.01),
            MOVE_STRENGTH => (0.1, 1.0, 0.01),
            GRAB_RANGE => (1.0, 60.0, 0.5),
            AUTO_GRAB_CLOSE | AUTO_GRAB_FAR => (0.0, 150.0, 0.5),
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
    pub fn values(s: AssistSettings) -> Values {
        let mut values = [0.0; COUNT];
        values[WALK_STRENGTH] = s.walk_strength;
        values[GRAB_RANGE] = s.fighter.grab.standing_range;
        values[HORIZONTAL_RANGE] = s.horizontal_range;
        values[VERTICAL_RANGE] = s.vertical_range;
        values[FOLLOW_RANGE] = s.follow_range;
        values[FOLLOW_VERTICAL_RANGE] = s.follow_vertical_range;
        values[STAND_DISTANCE] = s.stand_distance;
        values[MOVE_STRENGTH] = s.move_strength;
        values[MANUAL_SPACING] = if s.manual_spacing { 1.0 } else { 0.0 };
        values[AUTO_POSITION] = if s.auto_position { 1.0 } else { 0.0 };
        values[USE_RANGE_LIMITS] = if s.distance_agnostic { 0.0 } else { 1.0 };
        values[GRAB_DODGE_DEFAULT] = 1.0;
        values[OUT_OF_SHIELD_DEFAULT] = 1.0;
        values[GRAB_DODGE_LOOKAHEAD] = crate::spot_dodge::LOOKAHEAD;
        values[SHIELD_WARNING_X] = 24.0;
        values[SHIELD_WARNING_Y] = 28.0;
        values[SHIELD_BODY_HALF_WIDTH] = 6.0;
        values[SHIELD_BODY_HEIGHT] = 18.0;
        values[SHIELD_BODY_HALF_DEPTH] = 6.0;
        values[SHIELD_FORECAST] = 5.0;
        values[SHIELD_HOLD] = 4.0;
        values[GRAB_DODGE_PADDING] = DEFAULT_GRAB_DODGE_PADDING;
        values[GRAB_DODGE_HEIGHT] = DEFAULT_GRAB_DODGE_HEIGHT;
        values[TECH_HALF_WIDTH] = 2.5;
        values[TECH_FLOOR_ORIGIN] = 1.0;
        values[TECH_WALL_LOW] = 1.0;
        values[TECH_WALL_MID] = 3.5;
        values[TECH_WALL_HIGH] = 6.0;
        values[TECH_CEILING_ORIGIN] = 5.0;
        values[TECH_FLOOR_PADDING] = 1.5;
        values[TECH_CEILING_PADDING] = 2.0;
        crate::fighter_tuning_defaults::legacy_ranges(s.fighter_kind, &mut values);
        values[PUNISH_HEIGHT] = 8.0;
        values[ROLL_SMASH_HEIGHT] = 6.0;
        values[ROLL_SMASH_WATCH] = 64.0;
        values[ROLL_GRAB_HEIGHT] = 4.0;
        values[JAB_LOCK_REACH] = 7.0;
        values[JAB_LOCK_SMASH_REACH] = 12.0;
        values[JAB_LOCK_CHASE_DISTANCE] = 60.0;
        values[JAB_LOCK_HEIGHT] = 24.0;
        values[CROUCH_RANGE] = 45.0;
        values[CROUCH_HEIGHT] = 20.0;
        values[SHIELD_SCAN_X] = 96.0;
        values[SHIELD_SCAN_Y] = 96.0;
        let (auto_grab_close, auto_grab_far) =
            analysis::default_auto_grab_ranges(s.fighter_kind, s.fighter.grab.standing_range);
        values[AUTO_GRAB_CLOSE] = auto_grab_close;
        values[AUTO_GRAB_FAR] = auto_grab_far;
        values[REFLECT_DEFAULT] = 0.0;
        values[REFLECT_STARTUP] = MIN_GAMEWATCH_REFLECT_STARTUP_FRAMES;
        values[REFLECT_MARGIN] = DEFAULT_REFLECT_MARGIN_FRAMES;
        values[REFLECT_MAX_HOLD] = DEFAULT_REFLECT_MAX_HOLD_FRAMES;
        values[REFLECT_HALF_WIDTH] = DEFAULT_REFLECT_HALF_WIDTH;
        values[REFLECT_HEIGHT] = DEFAULT_REFLECT_HEIGHT;
        values[REFLECT_HALF_DEPTH] = DEFAULT_REFLECT_HALF_DEPTH;
        values[TILT_TURN_DOWN] = if crate::fighter_tuning_defaults::tilt_turn_down(s.fighter_kind) {
            1.0
        } else {
            0.0
        };
        Values(values)
    }
    pub fn apply(self, mut s: AssistSettings) -> AssistSettings {
        let v = self.0;
        if let Some(x) = v[WALK_STRENGTH] {
            s.walk_strength = x;
        }
        if let Some(x) = v[GRAB_RANGE] {
            s.fighter.grab.standing_range = x;
        }
        if let Some(x) = v[HORIZONTAL_RANGE] {
            s.horizontal_range = x;
        }
        if let Some(x) = v[VERTICAL_RANGE] {
            s.vertical_range = x;
        }
        if let Some(x) = v[FOLLOW_RANGE] {
            s.follow_range = x;
        }
        if let Some(x) = v[FOLLOW_VERTICAL_RANGE] {
            s.follow_vertical_range = x;
        }
        if let Some(x) = v[STAND_DISTANCE] {
            s.stand_distance = x;
        }
        if let Some(x) = v[MOVE_STRENGTH] {
            s.move_strength = x;
        }
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
    const SOURCE: &str = "# keep recipes\n[kirby.neutral.nair]\nstart_delay=0\nactions=nair\n[yoshi.neutral.grab]\nstart_delay=0\nactions=grab:fighter_range\n";
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
