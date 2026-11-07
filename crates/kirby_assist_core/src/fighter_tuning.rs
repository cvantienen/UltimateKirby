//! Input-policy overrides only. Never writes native fighter parameters.
//! `schema.rs` owns the stable menu/file fields; `analysis.rs` groups the
//! fighter-derived defaults and policy conversions.
use crate::ledge_trap::LedgeSettings;
mod analysis;
mod schema;

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
            13 | SHIELD_FORECAST
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
            .unwrap_or(0)
    }
    pub fn visible(kind: i32, i: usize) -> bool {
        if i >= COUNT {
            return false;
        }
        if i >= REFLECT_DEFAULT {
            return crate::projectile_reflect::supports(kind);
        }
        // Retired fixed punish helpers: accept old file keys, hide inert UI.
        if (PUNISH_DOWN_REACH..=ROLL_GRAB_WATCH).contains(&i) {
            return false;
        }
        if (JAB_LOCK_REACH..=JAB_LOCK_HEIGHT).contains(&i) {
            return kind == crate::fighter_kinds::YOSHI;
        }
        if (CROUCH_RANGE..=CROUCH_HEIGHT).contains(&i) {
            return kind == crate::fighter_kinds::KIRBY;
        }
        true
    }
    pub fn is_bool(i: usize) -> bool {
        (8..=12).contains(&i) || i == REFLECT_DEFAULT
    }
    pub fn grab_dodge_default(self, base: bool) -> bool {
        self.0[11].map_or(base, |x| x == 1.0)
    }
    pub fn oos_default(self) -> bool {
        self.0[12].unwrap_or(1.0) == 1.0
    }
    pub fn dodge_lookahead(self) -> f32 {
        self.0[13].unwrap_or(crate::spot_dodge::LOOKAHEAD)
    }
    pub fn bounds(i: usize) -> (f32, f32, f32) {
        match i {
            REFLECT_DEFAULT => (0., 1., 1.),
            REFLECT_STARTUP => (6., 30., 1.),
            REFLECT_MARGIN => (0., 8., 1.),
            REFLECT_MAX_HOLD => (6., 120., 1.),
            REFLECT_HALF_WIDTH | REFLECT_HALF_DEPTH => (1., 30., 0.5),
            REFLECT_HEIGHT => (1., 60., 0.5),
            8..=12 => (0.0, 1.0, 1.0),
            13 => (1.0, 8.0, 0.5),
            0 => (0.1, 0.5, 0.01),
            7 => (0.1, 1.0, 0.01),
            1 => (1.0, 60.0, 0.5),
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
            1.0,
            1.0,
            crate::spot_dodge::LOOKAHEAD,
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
            if s.fighter_kind == crate::fighter_kinds::KIRBY {
                14.0
            } else if s.fighter_kind == crate::fighter_kinds::YOSHI {
                13.0
            } else {
                12.0
            },
            if s.fighter_kind == crate::fighter_kinds::KIRBY {
                22.0
            } else {
                21.0
            },
            8.0,
            6.0,
            64.0,
            4.0,
            if s.fighter_kind == crate::fighter_kinds::KIRBY {
                32.0
            } else {
                36.0
            },
            7.0,
            12.0,
            60.0,
            24.0,
            45.0,
            20.0,
            96.0,
            96.0,
            analysis::default_auto_grab_ranges(s.fighter_kind, s.fighter.grab.standing_range).0,
            analysis::default_auto_grab_ranges(s.fighter_kind, s.fighter.grab.standing_range).1,
            0.,
            6.,
            2.,
            45.,
            5.,
            16.,
            5.,
        ])
    }
    pub fn apply(self, mut s: LedgeSettings) -> LedgeSettings {
        let v = self.0;
        if let Some(x) = v[0] {
            s.walk_strength = x;
        }
        if let Some(x) = v[1] {
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
        let base = LedgeSettings::default();
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
