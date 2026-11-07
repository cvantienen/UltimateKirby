//! Small copied policy values; no native queries or persistent caches.
use super::*;
pub fn default_auto_grab_ranges(kind: i32, standing: f32) -> (f32, f32) {
    if !standing.is_finite() || standing <= 0. {
        return (0., 0.);
    }
    let Some((base, dash)) = crate::fighter_data::ordinary_grab(kind, 0x37)
        .zip(crate::fighter_data::ordinary_grab(kind, 0x39))
        .and_then(|(base, dash)| Some((base.reach?, dash.reach?)))
        .filter(|(base, dash)| *base > 0. && *dash > 0.)
    else {
        return (0., 0.);
    };
    (standing, standing * dash / base)
}
impl Tuning {
    pub fn auto_grab_range(&self, kind: i32, standing: f32) -> f32 {
        self.value(AUTO_GRAB_CLOSE, default_auto_grab_ranges(kind, standing).0)
    }
    pub fn auto_grab_ranges(&self, kind: i32, standing: f32) -> (f32, f32) {
        if let (Some(close), Some(far)) = (self.0[AUTO_GRAB_CLOSE], self.0[AUTO_GRAB_FAR]) {
            return (close, far.max(close));
        }
        let defaults = default_auto_grab_ranges(kind, standing);
        let close = self.value(AUTO_GRAB_CLOSE, defaults.0);
        let far = self.value(AUTO_GRAB_FAR, defaults.1);
        (close, far.max(close))
    }
    pub fn shield(&self) -> crate::autopilot_guard::Envelope {
        crate::autopilot_guard::Envelope {
            scan_x: self.value(SHIELD_SCAN_X, 96.),
            scan_y: self.value(SHIELD_SCAN_Y, 96.),
            warning_x: self.value(SHIELD_WARNING_X, 24.),
            warning_y: self.value(SHIELD_WARNING_Y, 28.),
            half_width: self.value(SHIELD_BODY_HALF_WIDTH, 6.),
            height: self.value(SHIELD_BODY_HEIGHT, 18.),
            half_depth: self.value(SHIELD_BODY_HALF_DEPTH, 6.),
            forecast: self.value(SHIELD_FORECAST, 5.),
        }
    }
    pub fn tech_body(&self) -> crate::tech_prediction::Body {
        crate::tech_prediction::Body {
            half_width: self.value(TECH_HALF_WIDTH, 2.5),
            floor_origin: self.value(TECH_FLOOR_ORIGIN, 1.),
            wall: [
                self.value(TECH_WALL_LOW, 1.),
                self.value(TECH_WALL_MID, 3.5),
                self.value(TECH_WALL_HIGH, 6.),
            ],
            ceiling_origin: self.value(TECH_CEILING_ORIGIN, 5.),
            floor_padding: self.value(TECH_FLOOR_PADDING, 1.5),
            ceiling_padding: self.value(TECH_CEILING_PADDING, 2.),
        }
    }
    pub fn legacy_punish(&self, mut c: crate::option_read::Config) -> crate::option_read::Config {
        c.down_reach = self.value(PUNISH_DOWN_REACH, c.down_reach);
        c.forward_reach = self.value(PUNISH_FORWARD_REACH, c.forward_reach);
        c
    }
    pub fn legacy_roll(
        &self,
        mut c: crate::roll_read::FighterConfig,
        choice: crate::roll_read::Choice,
    ) -> crate::roll_read::FighterConfig {
        if choice == crate::roll_read::Choice::Grab {
            c.reach = self.value(1, c.reach);
            c.height = self.value(ROLL_GRAB_HEIGHT, c.height);
            c.watch_distance = self.value(ROLL_GRAB_WATCH, c.watch_distance);
        } else {
            c.reach = self.value(PUNISH_DOWN_REACH, c.reach);
            c.height = self.value(ROLL_SMASH_HEIGHT, c.height);
            c.watch_distance = self.value(ROLL_SMASH_WATCH, c.watch_distance);
        }
        c
    }
    pub fn jab_lock(&self, kind: i32) -> Option<crate::jab_lock::Config> {
        let mut c = crate::jab_lock::config(kind)?;
        c.jab_reach = self.value(JAB_LOCK_REACH, c.jab_reach);
        c.smash_reach = self.value(JAB_LOCK_SMASH_REACH, c.smash_reach);
        c.grab_reach = self.0[1].or(c.grab_reach);
        c.chase_distance = self.value(JAB_LOCK_CHASE_DISTANCE, c.chase_distance);
        Some(c)
    }
}
