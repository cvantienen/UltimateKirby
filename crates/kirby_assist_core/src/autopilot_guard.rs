//! Small geometric warning envelope, not native hitbox/hurtbox collision data.
//! Only use this for a fighter with an active or verified imminent attack.
pub mod duck;
pub mod duck_geometry;

pub const LOOKAHEAD_FRAMES: f32 = 5.0;
pub const HOLD_FRAMES: f32 = 4.0;
#[derive(Clone, Copy)]
pub struct Envelope {
    pub scan_x: f32,
    pub scan_y: f32,
    pub warning_x: f32,
    pub warning_y: f32,
    pub half_width: f32,
    pub height: f32,
    pub half_depth: f32,
    pub forecast: f32,
}
impl Default for Envelope {
    fn default() -> Self {
        Self {
            scan_x: 96.,
            scan_y: 96.,
            warning_x: 24.,
            warning_y: 28.,
            half_width: 6.,
            height: 18.,
            half_depth: 6.,
            forecast: LOOKAHEAD_FRAMES,
        }
    }
}

/// Root positions/velocities are in world units and game frames. Five-frame
/// linear projection helps with approaching aerials, but cannot predict startup,
/// acceleration, teleports, projectiles or the actual shape of an attack.
pub fn approaching_active_attack(
    own: [f32; 2],
    own_velocity: [f32; 2],
    enemy: [f32; 2],
    enemy_velocity: [f32; 2],
) -> bool {
    approaching_active_attack_with(
        own,
        own_velocity,
        enemy,
        enemy_velocity,
        Envelope::default(),
    )
}
pub fn approaching_active_attack_with(
    own: [f32; 2],
    own_velocity: [f32; 2],
    enemy: [f32; 2],
    enemy_velocity: [f32; 2],
    e: Envelope,
) -> bool {
    if !own
        .iter()
        .chain(own_velocity.iter())
        .chain(enemy.iter())
        .chain(enemy_velocity.iter())
        .all(|v| v.is_finite())
    {
        return false;
    }
    let relative = [enemy[0] - own[0], enemy[1] - own[1]];
    let velocity = [
        enemy_velocity[0] - own_velocity[0],
        enemy_velocity[1] - own_velocity[1],
    ];
    // Swept segment against a rectangular warning envelope: no missed crossing
    // just because both sampled endpoints fall outside it.
    let mut enter = 0.0_f32;
    let mut leave = e.forecast;
    for (axis, radius) in [(0, e.warning_x), (1, e.warning_y)] {
        if velocity[axis].abs() < 0.0001 {
            if relative[axis].abs() > radius {
                return false;
            }
        } else {
            let a = (-radius - relative[axis]) / velocity[axis];
            let b = (radius - relative[axis]) / velocity[axis];
            enter = enter.max(a.min(b));
            leave = leave.min(a.max(b));
            if enter > leave {
                return false;
            }
        }
    }
    true
}

/// A nearby Up-B can reach the stage before its first active-hitbox observation.
/// Reserve shield conservatively during its native startup/first continuation;
/// active geometry handles later phases. This does not confirm a hit or punish.
pub fn recovery_warning(
    kind: i32,
    status: i32,
    previous: i32,
    own: [f32; 2],
    velocity: [f32; 2],
    enemy: [f32; 2],
    enemy_velocity: [f32; 2],
    envelope: Envelope,
) -> bool {
    // Recovery startup can have zero speed before a sudden upward burst. Use
    // the existing scan depth below the stage, rather than extrapolating zero
    // velocity and waiting until the rising hitbox is already beside us.
    let mut warning = envelope;
    if enemy[1] < own[1] - warning.warning_y && enemy_velocity[1] >= velocity[1] {
        warning.warning_y = warning.warning_y.max(warning.scan_y);
    }
    crate::reaction_rules::automatic_up_b_status(kind, status, previous)
        && approaching_active_attack_with(own, velocity, enemy, enemy_velocity, warning)
}

/// Predict only move families described by the actual opponent's compiled
/// profile. Unknown attacks retain active-hitbox fallback, never Kirby data.
/// Verified compiled active windows, including aerials, jabs and other mapped
/// attacks. Unknown/ambiguous motion mappings do not invent startup timing.
pub fn referenced_imminent_attack(kind: i32, motion: u64, frame: f32, rate: f32) -> bool {
    referenced_imminent_attack_with(kind, motion, frame, rate, LOOKAHEAD_FRAMES)
}
pub fn referenced_imminent_attack_with(
    kind: i32,
    motion: u64,
    frame: f32,
    rate: f32,
    forecast: f32,
) -> bool {
    if !frame.is_finite() || frame < 0.0 || !rate.is_finite() || rate <= 0.0 {
        return false;
    }
    crate::fighter_data::fighter(kind)
        .and_then(|p| p.move_for_motion(motion))
        .is_some_and(|m| {
            m.has_damage_data() && {
                m.active.iter().any(|w| {
                    frame <= w.last as f32 && ((w.first as f32 - frame) / rate).max(0.0) <= forecast
                })
            }
        })
}
/// Conservative swept hitbox AABB versus a standing defensive envelope.
/// Bounds are estimates, not live hurtboxes; reads never alter attack geometry.
pub fn active_hitbox_threat(
    position: [f32; 3],
    velocity: [f32; 2],
    h: duck_geometry::Hitbox,
) -> bool {
    active_hitbox_threat_with(position, velocity, h, Envelope::default())
}
pub fn active_hitbox_threat_with(
    position: [f32; 3],
    velocity: [f32; 2],
    h: duck_geometry::Hitbox,
    e: Envelope,
) -> bool {
    if !position
        .iter()
        .chain(velocity.iter())
        .chain(h.start.iter())
        .chain(h.end.iter())
        .chain(h.velocity.iter())
        .chain([&h.radius])
        .all(|x| x.is_finite())
        || !(0.0..=100.0).contains(&h.radius)
        || h.radius == 0.0
    {
        return false;
    }
    let mut enter = 0.0_f32;
    let mut leave = e.forecast;
    for axis in 0..3 {
        let (low, high) = match axis {
            0 => (position[0] - e.half_width, position[0] + e.half_width),
            1 => (position[1], position[1] + e.height),
            _ => (position[2] - e.half_depth, position[2] + e.half_depth),
        };
        let lo = h.start[axis].min(h.end[axis]) - h.radius;
        let hi = h.start[axis].max(h.end[axis]) + h.radius;
        let v = if axis < 2 {
            h.velocity[axis] - velocity[axis]
        } else {
            0.0
        };
        if v.abs() < 0.0001 {
            if hi < low || lo > high {
                return false;
            }
        } else {
            let a = (low - hi) / v;
            let b = (high - lo) / v;
            enter = enter.max(a.min(b));
            leave = leave.min(a.max(b));
            if enter > leave {
                return false;
            }
        }
    }
    true
}

#[derive(Clone, Copy)]
pub struct GuardClock {
    pub status: i32,
    pub motion: u64,
    pub frame: f32,
    pub rate: f32,
    pub stopped: bool,
}
#[derive(Default)]
pub struct GuardHold {
    previous: Option<GuardClock>,
    previous_enemy: Option<(u32, GuardClock)>,
    remaining: f32,
}
impl GuardHold {
    /// Four advancing native frames of tail coverage, not mapper polls or ms.
    /// Release/invalid readiness clears immediately; hitstop does not spend it.
    pub fn sample(&mut self, eligible: bool, threat: bool, c: GuardClock) -> bool {
        self.sample_with_duration(eligible, threat, c, HOLD_FRAMES)
    }
    pub fn sample_with_duration(
        &mut self,
        eligible: bool,
        threat: bool,
        c: GuardClock,
        hold: f32,
    ) -> bool {
        self.sample_inner(eligible, threat, c, hold, None, false)
    }
    /// Live ledge defense can hold a static guard pose. Use only a copied
    /// progressing opponent clock; never keep a stale tail without a clock.
    /// This is passive defense only, not an authored shield sequence.
    pub fn sample_with_progress(
        &mut self,
        eligible: bool,
        threat: bool,
        c: GuardClock,
        hold: f32,
        enemy: Option<(u32, GuardClock)>,
    ) -> bool {
        self.sample_inner(eligible, threat, c, hold, enemy, true)
    }
    fn sample_inner(
        &mut self,
        eligible: bool,
        threat: bool,
        c: GuardClock,
        hold: f32,
        enemy: Option<(u32, GuardClock)>,
        source_aware: bool,
    ) -> bool {
        if !eligible || !hold.is_finite() || !(0.0..=20.0).contains(&hold) {
            *self = Self::default();
            return false;
        }
        // A gap between multihit hitboxes must not release an existing shield
        // during native shieldstun. Never generate an attack on shield release.
        let threat = threat || c.status == 0x1e;
        let valid = c.motion != 0
            && c.frame.is_finite()
            && c.frame >= 0.0
            && c.rate.is_finite()
            && c.rate > 0.0;
        if !valid {
            // Hitstop may freeze/reset the guard pose. Preserve its tail until
            // an advancing clock returns; unknown non-stopped clocks fail open.
            if c.stopped {
                if threat {
                    self.remaining = hold;
                }
                self.previous = None;
                self.previous_enemy = None;
                return threat || self.remaining > 0.;
            }
            *self = Self::default();
            return threat;
        }
        let mut elapsed = if c.stopped {
            0.0
        } else {
            self.previous.map_or(0.0, |p| {
                crate::timing::motion_progress(
                    Some(crate::timing::MotionStamp::new(
                        p.status, p.motion, p.frame, p.rate,
                    )),
                    crate::timing::MotionStamp::new(c.status, c.motion, c.frame, c.rate),
                )
                .elapsed(0.0, 1.0)
                .unwrap_or(hold)
            })
        };
        let enemy = enemy.filter(|(_, e)| {
            e.motion != 0
                && e.frame.is_finite()
                && e.frame >= 0.
                && e.rate.is_finite()
                && e.rate > 0.
                && !e.stopped
                && !matches!(e.status, 0x1b | 0x1c | 0x77 | 0x7e | 0x7f)
        });
        let enemy_elapsed = enemy.and_then(|(id, e)| {
            let previous = self
                .previous_enemy
                .filter(|(old, _)| *old == id)
                .map(|(_, p)| crate::timing::MotionStamp::new(p.status, p.motion, p.frame, p.rate));
            crate::timing::motion_progress(
                previous,
                crate::timing::MotionStamp::new(e.status, e.motion, e.frame, e.rate),
            )
            .elapsed(0., 1.)
            .filter(|n| n.is_finite() && (0.0..=8.0).contains(n))
        });
        self.previous_enemy = if c.stopped { None } else { enemy };
        if source_aware && !c.stopped && elapsed == 0. && matches!(c.status, 0x1b | 0x1c) {
            if let Some(delta) = enemy_elapsed {
                elapsed = delta;
            } else if !threat {
                self.remaining = 0.;
            }
        }
        self.previous = Some(c);
        self.remaining = if threat {
            hold
        } else {
            (self.remaining - elapsed).max(0.0)
        };
        threat || self.remaining > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checks_incoming_and_crossing_attacks_without_reacting_to_distant_retreats() {
        assert!(approaching_active_attack(
            [0.; 2],
            [0.; 2],
            [20., 0.],
            [-2., 0.]
        ));
        assert!(approaching_active_attack(
            [0.; 2],
            [0.; 2],
            [50., 0.],
            [-50., 0.]
        ));
        assert!(!approaching_active_attack(
            [0.; 2],
            [0.; 2],
            [30., 0.],
            [2., 0.]
        ));
        assert!(!approaching_active_attack(
            [0.; 2],
            [0.; 2],
            [0., 50.],
            [0., 0.]
        ));
        assert!(!approaching_active_attack(
            [f32::NAN, 0.],
            [0.; 2],
            [0.; 2],
            [0.; 2]
        ));
    }
}

#[cfg(test)]
mod approach_regressions {
    use super::*;
    #[test]
    fn verified_fair_startup_is_warned_and_unknown_motion_is_not_invented() {
        let m = crate::timing::motion_hash("attack_air_f");
        assert!(referenced_imminent_attack(5, m, 12.0, 1.0));
        assert!(!referenced_imminent_attack(5, m, 1.0, 1.0));
        assert!(!referenced_imminent_attack(5, 0, 12.0, 1.0));
        assert!(!referenced_imminent_attack(5, m, f32::NAN, 1.0));
    }
    #[test]
    fn extended_and_descending_hitboxes_are_seen_without_close_roots() {
        let long = duck_geometry::Hitbox {
            start: [8.0, 10.0, 0.0],
            end: [35.0, 10.0, 0.0],
            radius: 3.0,
            velocity: [-1.0, 0.0],
        };
        assert!(active_hitbox_threat([0.0; 3], [0.0; 2], long));
        let falling = duck_geometry::Hitbox {
            start: [0.0, 40.0, 0.0],
            end: [0.0, 40.0, 0.0],
            radius: 3.0,
            velocity: [0.0, -5.0],
        };
        assert!(active_hitbox_threat([0.0; 3], [0.0; 2], falling));
        assert!(!active_hitbox_threat(
            [0.0; 3],
            [0.0; 2],
            duck_geometry::Hitbox {
                velocity: [0.0, 5.0],
                ..falling
            }
        ));
        assert!(!active_hitbox_threat(
            [0.0; 3],
            [0.0; 2],
            duck_geometry::Hitbox {
                start: [0.0, 10.0, 40.0],
                end: [0.0, 10.0, 40.0],
                ..long
            }
        ));
    }
}
