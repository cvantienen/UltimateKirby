//! Small geometric warning envelope, not native hitbox/hurtbox collision data.
//! Only use this for a fighter with an active or verified imminent attack.
use crate::fighter_profiles::LedgeFighterData;

pub mod duck;
pub mod duck_geometry;

pub const LOOKAHEAD_FRAMES: f32 = 3.0;
pub const HOLD_FRAMES: f32 = 4.0;

/// Root positions/velocities are in world units and game frames. Three-frame
/// linear projection helps with approaching aerials, but cannot predict startup,
/// acceleration, teleports, projectiles or the actual shape of an attack.
pub fn approaching_active_attack(
    own: [f32; 2],
    own_velocity: [f32; 2],
    enemy: [f32; 2],
    enemy_velocity: [f32; 2],
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
    let mut leave = LOOKAHEAD_FRAMES;
    for (axis, radius) in [(0, 24.0_f32), (1, 28.0_f32)] {
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

/// Predict only move families described by the actual opponent's compiled
/// profile. Unknown attacks retain active-hitbox fallback, never Kirby data.
pub fn imminent_attack(
    data: LedgeFighterData,
    status: i32,
    motion: u64,
    frame: f32,
    rate: f32,
) -> bool {
    if !frame.is_finite() || frame < 0.0 || !rate.is_finite() || rate <= 0.0 {
        return false;
    }
    let next = |first: u8| {
        let left = (f32::from(first) - frame) / rate;
        (0.0..=LOOKAHEAD_FRAMES).contains(&left)
    };
    match status {
        0x2a => next(data.forward_tilt.first),
        0x2c => next(data.down_tilt.first),
        0x32 => {
            next(data.down_smash.first)
                || data
                    .down_smash_back
                    .is_some_and(|window| next(window.first))
        }
        0x36 if motion == 0xc3a4e2597 => next(data.neutral_air.first),
        _ => false,
    }
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
    remaining: f32,
}
impl GuardHold {
    /// Four advancing native frames of tail coverage, not mapper polls or ms.
    /// Release/invalid readiness clears immediately; hitstop does not spend it.
    pub fn sample(&mut self, eligible: bool, threat: bool, c: GuardClock) -> bool {
        if !eligible {
            *self = Self::default();
            return false;
        }
        let valid = c.motion != 0
            && c.frame.is_finite()
            && c.frame >= 0.0
            && c.rate.is_finite()
            && c.rate > 0.0;
        if !valid {
            *self = Self::default();
            return threat;
        }
        let elapsed = if c.stopped {
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
                .unwrap_or(HOLD_FRAMES)
            })
        };
        self.previous = Some(c);
        self.remaining = if threat {
            HOLD_FRAMES
        } else {
            (self.remaining - elapsed).max(0.0)
        };
        self.remaining > 0.0
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
