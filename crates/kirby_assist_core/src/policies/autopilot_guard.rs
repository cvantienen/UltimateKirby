//! Small geometric warning envelope, not native hitbox/hurtbox collision data.
//! Only use this for a fighter with an active or verified imminent attack.

/// Copied attack geometry shared by guard and projectile threat checks.
///
///  Copied attack geometry shared by guard and projectile threat checks.
#[derive(Clone, Copy, Debug)]
pub struct Hitbox {
    /// Stores the `start` value in `Hitbox` as `[f32; 3]`.
    ///
    ///
///  Stores the start collection for Hitbox.
    pub start: [f32; 3],
    /// Stores the `end` value in `Hitbox` as `[f32; 3]`.
    ///
    ///
///  Stores the end collection for Hitbox.
    pub end: [f32; 3],
    /// Stores the `radius` value in `Hitbox` as `f32`.
    ///
    ///
///  Holds the radius value used by Hitbox.
    pub radius: f32,
    /// Stores the `velocity` value in `Hitbox` as `[f32; 2]`.
    ///
    ///
///  Stores the velocity collection for Hitbox.
    pub velocity: [f32; 2],
}

/// Defines the `LOOKAHEAD_FRAMES` constant used by this module.
///
/// Provides the LOOKAHEAD FRAMES constant value.
pub const LOOKAHEAD_FRAMES: f32 = 10.0;
/// Defines the `HOLD_FRAMES` constant used by this module.
///
/// Provides the HOLD FRAMES constant value.
pub const HOLD_FRAMES: f32 = 4.0;
/// Defines the `Envelope` data structure.
///
/// Holds the data for Envelope.
#[derive(Clone, Copy)]
pub struct Envelope {
    /// Stores the `scan_x` value in `Envelope` as `f32`.
    ///
    ///
///  Holds the scan x value used by Envelope.
    pub scan_x: f32,
    /// Stores the `scan_y` value in `Envelope` as `f32`.
    ///
    ///
///  Holds the scan y value used by Envelope.
    pub scan_y: f32,
    /// Stores the `warning_x` value in `Envelope` as `f32`.
    ///
    ///
///  Holds the warning x value used by Envelope.
    pub warning_x: f32,
    /// Stores the `warning_y` value in `Envelope` as `f32`.
    ///
    ///
///  Holds the warning y value used by Envelope.
    pub warning_y: f32,
    /// Stores the `half_width` value in `Envelope` as `f32`.
    ///
    ///
///  Holds the half width value used by Envelope.
    pub half_width: f32,
    /// Stores the `height` value in `Envelope` as `f32`.
    ///
    ///
///  Holds the height value used by Envelope.
    pub height: f32,
    /// Stores the `half_depth` value in `Envelope` as `f32`.
    ///
    ///
///  Holds the half depth value used by Envelope.
    pub half_depth: f32,
    /// Stores the `forecast` value in `Envelope` as `f32`.
    ///
    ///
///  Holds the forecast value used by Envelope.
    pub forecast: f32,
}
impl Default for Envelope {
    /// Returns the default value for this type.
    ///
    ///
///  Returns the default value for this type.
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
///
///  Root positions/velocities are in world units and game frames. Five-frame linear projection helps with approaching aerials, but cannot predict startup, acceleration, teleports, projectiles or the actual shape of an attack.
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
/// Determines whether the `approaching_active_attack_with` condition holds.
///
/// Checks whether approaching active attack with.
pub fn approaching_active_attack_with(
    own: [f32; 2],
    own_velocity: [f32; 2],
    enemy: [f32; 2],
    enemy_velocity: [f32; 2],
    e: Envelope,
) -> bool {
    if !finite(&[&own, &own_velocity, &enemy, &enemy_velocity]) {
        return false;
    }
    let relative = [enemy[0] - own[0], enemy[1] - own[1]];
    let velocity = [
        enemy_velocity[0] - own_velocity[0],
        enemy_velocity[1] - own_velocity[1],
    ];
    // Swept segment against a rectangular warning envelope: no missed crossing
    // just because both sampled endpoints fall outside it.
    let mut window = [0.0_f32, e.forecast];
    [(0, e.warning_x), (1, e.warning_y)]
        .into_iter()
        .all(|(axis, radius)| {
            let p = relative[axis];
            sweep_axis(&mut window, p, p, -radius, radius, velocity[axis])
        })
}

/// Determines whether the `finite` condition holds.
///
/// Checks whether finite.
fn finite(groups: &[&[f32]]) -> bool {
    groups.iter().all(|g| g.iter().all(|v| v.is_finite()))
}
/// One axis of a swept overlap test. A span `[lo, hi]` moving at `velocity`
/// overlaps `[low, high]` during some time interval; narrow `window`
/// (`[enter, leave]`) to it and report whether any time is left.
///
///  One axis of a swept overlap test. A span [lo, hi] moving at velocity overlaps [low, high] during some time interval; narrow window ([enter, leave]) to it and report whether any time is left.
fn sweep_axis(window: &mut [f32; 2], lo: f32, hi: f32, low: f32, high: f32, velocity: f32) -> bool {
    if velocity.abs() < 0.0001 {
        return hi >= low && lo <= high;
    }
    let a = (low - hi) / velocity;
    let b = (high - lo) / velocity;
    window[0] = window[0].max(a.min(b));
    window[1] = window[1].min(a.max(b));
    window[0] <= window[1]
}

/// A nearby Up-B can reach the stage before its first active-hitbox observation.
/// Reserve shield conservatively during its native startup/first continuation;
/// active geometry handles later phases. This does not confirm a hit or punish.
///
///  A nearby Up-B can reach the stage before its first active-hitbox observation. Reserve shield conservatively during its native startup/first continuation; active geometry handles later phases. This does not confirm a hit or punish.
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
///
///  Predict only move families described by the actual opponent's compiled profile. Unknown attacks retain active-hitbox fallback, never Kirby data. Verified compiled active windows, including aerials, jabs and other mapped.
pub fn referenced_imminent_attack(kind: i32, motion: u64, frame: f32, rate: f32) -> bool {
    referenced_imminent_attack_with(kind, motion, frame, rate, LOOKAHEAD_FRAMES)
}

/// Predict whether an opponent's attack will become active within the forecast window.
///
///  Predict whether an opponent's attack will become active within the forecast window.
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
///
///  Conservative swept hitbox AABB versus a standing defensive envelope. Bounds are estimates, not live hurtboxes; reads never alter attack geometry.
pub fn active_hitbox_threat(position: [f32; 3], velocity: [f32; 2], h: Hitbox) -> bool {
    active_hitbox_threat_with(position, velocity, h, Envelope::default())
}
/// Conservative swept hitbox AABB versus a standing defensive envelope, with custom envelope.
///
///  Conservative swept hitbox AABB versus a standing defensive envelope, with custom envelope.
pub fn active_hitbox_threat_with(
    position: [f32; 3],
    velocity: [f32; 2],
    h: Hitbox,
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

/// Defines the `GuardClock` data structure.
///
/// Holds the data for GuardClock.
#[derive(Clone, Copy)]
pub struct GuardClock {
    /// Stores the `status` value in `GuardClock` as `i32`.
    ///
    ///
///  Holds the status value used by GuardClock.
    pub status: i32,
    /// Stores the `motion` value in `GuardClock` as `u64`.
    ///
    ///
///  Holds the motion value used by GuardClock.
    pub motion: u64,
    /// Stores the `frame` value in `GuardClock` as `f32`.
    ///
    ///
///  Holds the frame value used by GuardClock.
    pub frame: f32,
    /// Stores the `rate` value in `GuardClock` as `f32`.
    ///
    ///
///  Holds the rate value used by GuardClock.
    pub rate: f32,
    /// Stores the `stopped` value in `GuardClock` as `bool`.
    ///
    ///
///  Holds the stopped value used by GuardClock.
    pub stopped: bool,
}
/// Defines the `GuardHold` data structure.
///
/// Holds the data for GuardHold.
#[derive(Default)]
pub struct GuardHold {
    /// Stores the `previous` value in `GuardHold` as `Option<GuardClock>`.
    ///
    ///
///  Optionally stores the previous for GuardHold; it may be absent.
    previous: Option<GuardClock>,
    /// Stores the `previous_enemy` value in `GuardHold` as `Option<(u32`.
    ///
    ///
///  Optionally stores the previous enemy for GuardHold; it may be absent.
    previous_enemy: Option<(u32, GuardClock)>,
    /// Stores the `remaining` value in `GuardHold` as `f32`.
    ///
    ///
///  Holds the remaining value used by GuardHold.
    remaining: f32,
}
impl GuardHold {
    /// Four advancing native frames of tail coverage, not mapper polls or ms.
    /// Release/invalid readiness clears immediately; hitstop does not spend it.
    ///
///  Four advancing native frames of tail coverage, not mapper polls or ms. Release/invalid readiness clears immediately; hitstop does not spend it.
    pub fn sample(&mut self, eligible: bool, threat: bool, c: GuardClock) -> bool {
        self.sample_with_duration(eligible, threat, c, HOLD_FRAMES)
    }
    /// Samples the with duration.
    ///
    ///
///  Samples the with duration.
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
    ///
///  Live ledge defense can hold a static guard pose. Use only a copied progressing opponent clock; never keep a stale tail without a clock. This is passive defense only, not an authored shield sequence.
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
    /// Samples the inner.
    ///
    ///
///  Samples the inner.
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
        let threat = threat || c.status == crate::native_states::GUARD_DAMAGE;
        let valid = c.motion != crate::timing::NO_MOTION_KIND
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
            e.motion != crate::timing::NO_MOTION_KIND
                && e.frame.is_finite()
                && e.frame >= 0.
                && e.rate.is_finite()
                && e.rate > 0.
                && !e.stopped
                && !matches!(
                    e.status,
                    crate::native_states::GUARD_ON
                        | crate::native_states::GUARD
                        | crate::native_states::status::CLIFF_WAIT
                        | crate::native_states::status::OTTOTTO
                        | crate::native_states::status::OTTOTTO_WAIT
                )
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
        if source_aware
            && !c.stopped
            && elapsed == 0.
            && matches!(
                c.status,
                crate::native_states::GUARD_ON | crate::native_states::GUARD
            )
        {
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
    /// Regression test for `checks_incoming_and_crossing_attacks_without_reacting_to_distant_retreats`.
    ///
    ///
///  Checks that checks incoming and crossing attacks without reacting to distant retreats.
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
    /// Regression test for `verified_fair_startup_is_warned_and_unknown_motion_is_not_invented`.
    ///
    ///
///  Checks that verified fair startup is warned and unknown motion is not invented.
    #[test]
    fn verified_fair_startup_is_warned_and_unknown_motion_is_not_invented() {
        let m = crate::timing::motion_hash("attack_air_f");
        assert!(referenced_imminent_attack(5, m, 12.0, 1.0));
        assert!(!referenced_imminent_attack(5, m, 1.0, 1.0));
        assert!(!referenced_imminent_attack(5, 0, 12.0, 1.0));
        assert!(!referenced_imminent_attack(5, m, f32::NAN, 1.0));
    }
    /// Regression test for `extended_and_descending_hitboxes_are_seen_without_close_roots`.
    ///
    ///
///  Checks that extended and descending hitboxes are seen without close roots.
    #[test]
    fn extended_and_descending_hitboxes_are_seen_without_close_roots() {
        let long = Hitbox {
            start: [8.0, 10.0, 0.0],
            end: [35.0, 10.0, 0.0],
            radius: 3.0,
            velocity: [-1.0, 0.0],
        };
        assert!(active_hitbox_threat([0.0; 3], [0.0; 2], long));
        let falling = Hitbox {
            start: [0.0, 40.0, 0.0],
            end: [0.0, 40.0, 0.0],
            radius: 3.0,
            velocity: [0.0, -5.0],
        };
        assert!(active_hitbox_threat([0.0; 3], [0.0; 2], falling));
        assert!(!active_hitbox_threat(
            [0.0; 3],
            [0.0; 2],
            Hitbox {
                velocity: [0.0, 5.0],
                ..falling
            }
        ));
        assert!(!active_hitbox_threat(
            [0.0; 3],
            [0.0; 2],
            Hitbox {
                start: [0.0, 10.0, 40.0],
                end: [0.0, 10.0, 40.0],
                ..long
            }
        ));
    }
}
