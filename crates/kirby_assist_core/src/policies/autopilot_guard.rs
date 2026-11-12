//! Small geometric warning envelope, not native hitbox/hurtbox collision data.
//! Only use this for a fighter with an active or verified imminent attack.

/// World-space hitbox geometry shared by attack and projectile threat checks.
#[derive(Clone, Copy, Debug)]
pub struct Hitbox {
    /// First endpoint of the world-space hitbox segment.
    pub start: [f32; 3],
    /// Second endpoint of the world-space hitbox segment.
    pub end: [f32; 3],
    /// Hitbox radius in world units.
    pub radius: f32,
    /// Relative velocity used for swept threat prediction.
    pub velocity: [f32; 2],
}
/// How many frames to look ahead to shield against incoming attacks.
pub const LOOKAHEAD_FRAMES: f32 = 10.0;
/// How many frames to hold before letting go of the shield.
pub const HOLD_FRAMES: f32 = 4.0;
/// Represents the warning envelope for detecting approaching attacks.
/// This envelope defines the scanning and warning regions along the x and y axes, as well as the dimensions and forecast time for detecting incoming attacks.
/// containss:
/// - `scan_x` and `scan_y`: The scanning region along the x and y axes.
/// - `warning_x` and `warning_y`: The warning region along the x and y axes.
/// - `half_width`, `height`, and `half_depth`: The dimensions of the envelope.
/// - `forecast`: The forecast time for detecting incoming attacks.
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
/// Determines whether the `approaching_active_attack_with` condition holds.
/// Checks whether approaching active attack with the given envelope.
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
    let mut window = [0.0_f32, e.forecast];
    [(0, e.warning_x), (1, e.warning_y)]
        .into_iter()
        .all(|(axis, radius)| {
            let p = relative[axis];
            sweep_axis(&mut window, p, p, -radius, radius, velocity[axis])
        })
}

/// Determines whether the `finite` condition holds.
/// Checks whether finite.
fn finite(groups: &[&[f32]]) -> bool {
    groups.iter().all(|g| g.iter().all(|v| v.is_finite()))
}
/// One axis of a swept overlap test. A span `[lo, hi]` moving at `velocity`
/// overlaps `[low, high]` during some time interval; narrow `window`
/// (`[enter, leave]`) to it and report whether any time is left.
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

/// Predict whether an opponent's attack will become active within the forecast window.
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
pub fn active_hitbox_threat(position: [f32; 3], velocity: [f32; 2], h: Hitbox) -> bool {
    active_hitbox_threat_with(position, velocity, h, Envelope::default())
}
/// Conservative swept hitbox AABB versus a standing defensive envelope, with custom envelope.
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
    for (axis, (&start, &end)) in h.start.iter().zip(&h.end).enumerate() {
        let (low, high) = match axis {
            0 => (position[0] - e.half_width, position[0] + e.half_width),
            1 => (position[1], position[1] + e.height),
            _ => (position[2] - e.half_depth, position[2] + e.half_depth),
        };
        let lo = start.min(end) - h.radius;
        let hi = start.max(end) + h.radius;
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

/// Runtime Smash data carried by `GuardClock`.
#[derive(Clone, Copy)]
pub struct GuardClock {
/// Native fighter status observed for the guard decision.
    pub status: i32,
/// Native motion hash observed for the guard decision.
    pub motion: u64,
/// Current native animation frame.
    pub frame: f32,
/// Native animation playback rate.
    pub rate: f32,
/// Whether native animation progress is stopped, such as hitstop.
    pub stopped: bool,
}
/// Runtime Smash data carried by `GuardHold`.
#[derive(Default)]
pub struct GuardHold {
/// Previous guard-clock sample used to measure native progress.
    previous: Option<GuardClock>,
/// Previous opponent clock used for live ledge-defense timing.
    previous_enemy: Option<(u32, GuardClock)>,
/// Remaining native-frame shield tail after a threat ends.
    remaining: f32,
}
impl GuardHold {
    /// Samples the default four-frame shield tail after an approaching threat.
    pub fn sample(&mut self, eligible: bool, threat: bool, c: GuardClock) -> bool {
        self.sample_with_duration(eligible, threat, c, HOLD_FRAMES)
    }
    /// Samples a caller-supplied shield-tail duration in native frames.
    pub fn sample_with_duration(
        &mut self,
        eligible: bool,
        threat: bool,
        c: GuardClock,
        hold: f32,
    ) -> bool {
        self.sample_inner(eligible, threat, c, hold, None, false)
    }
    /// Samples a shield tail using a progressing copied opponent clock.
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
    /// Performs the shared guard-tail state update.
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
    /// Approaching and crossing attacks warn; distant retreats do not.
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
    /// Verified fair startup warns, while unknown motions remain unpredicted.
    #[test]
    fn verified_fair_startup_is_warned_and_unknown_motion_is_not_invented() {
        let m = crate::timing::motion_hash("attack_air_f");
        assert!(referenced_imminent_attack(5, m, 12.0, 1.0));
        assert!(!referenced_imminent_attack(5, m, 1.0, 1.0));
        assert!(!referenced_imminent_attack(5, 0, 12.0, 1.0));
        assert!(!referenced_imminent_attack(5, m, f32::NAN, 1.0));
    }
    /// Extended and descending hitboxes are detected even with distant roots.
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
