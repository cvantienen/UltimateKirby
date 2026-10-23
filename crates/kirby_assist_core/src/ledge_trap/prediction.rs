//! Pure timing, braking and approximate reach calculations.
//!
//! These helpers only calculate: they neither remember a ledge visit nor press
//! buttons. World distance, animation frames and transport delay are different
//! quantities. Keep their conversions here when adding another move/profile.
use super::{LedgeContext, LedgeSettings, Opponent, CLIFF_CLIMB, CLIFF_ESCAPE, JUMP_SQUAT_FRAMES};
use crate::timing::{ActiveWindow, PredictedContact, RecoveryWindow};

/// Conservative neutral-stop estimate: queued coasting, then discrete native
/// ground deceleration. Slopes, surfaces and status multipliers remain limits.
pub(super) fn roll_stopping_distance(ctx: LedgeContext, direction: f32) -> Option<f32> {
    let brake = ctx
        .ground_brake
        .filter(|b| b.is_finite() && (0.001..=3.0).contains(b))?;
    if !ctx.input_delay.is_finite() || !ctx.velocity[0].is_finite() {
        return None;
    }
    let mut speed = (ctx.velocity[0] * direction).max(0.0);
    let mut distance = speed * (ctx.input_delay.clamp(0.0, 12.0) + 1.0);
    for _ in 0..120 {
        if speed <= 0.0 {
            return Some(distance);
        }
        distance += speed;
        speed = (speed - brake).max(0.0);
    }
    None
}

pub(super) fn ground_attack_ready(ctx: LedgeContext, settings: LedgeSettings) -> bool {
    if settings.manual_spacing {
        matches!(ctx.status, 0..=2 | 0x6 | 0x11..=0x15)
    } else {
        ctx.status == 0
    }
}

/// Approximate root-position reach during an attack's active frames. Values
/// are from Kirby's measured frame data: https://ultimateframedata.com/kirby .
/// Unknown invincibility expiry requires actual vulnerability; never guess it.
#[derive(Clone, Copy)]
pub(super) struct AttackWindow {
    pub(super) first: u8,
    pub(super) last: u8,
    pub(super) range: f32,
    pub(super) height: [f32; 2],
    pub(super) front_only: bool,
}
impl AttackWindow {
    pub(super) fn timing_contact(
        self,
        ctx: LedgeContext,
        enemy: Opponent,
        actionable_in: Option<f32>,
    ) -> Option<PredictedContact> {
        ActiveWindow {
            first: u16::from(self.first),
            last: u16::from(self.last),
        }
        .first_contact(
            ctx.input_delay,
            RecoveryWindow {
                vulnerable_in: if enemy.vulnerable {
                    Some(0.0)
                } else {
                    enemy.vulnerable_in
                },
                actionable_in: actionable_in.or(enemy.actionable_in),
            },
            |_| true,
        )
    }

    pub(super) fn scheduled_contact(
        self,
        ctx: LedgeContext,
        enemy: Opponent,
        actionable_in: Option<f32>,
        ignore_distance: bool,
    ) -> Option<PredictedContact> {
        if ignore_distance {
            self.timing_contact(ctx, enemy, actionable_in)
        } else {
            self.contact_before(ctx, enemy, actionable_in)
        }
    }

    pub(super) fn contact_before(
        self,
        ctx: LedgeContext,
        enemy: Opponent,
        actionable_in: Option<f32>,
    ) -> Option<PredictedContact> {
        ActiveWindow {
            first: u16::from(self.first),
            last: u16::from(self.last),
        }
        .first_contact(
            ctx.input_delay,
            RecoveryWindow {
                vulnerable_in: if enemy.vulnerable {
                    Some(0.0)
                } else {
                    enemy.vulnerable_in
                },
                actionable_in: actionable_in.or(enemy.actionable_in),
            },
            |lead| {
                let [x, y] = predicted_offset(ctx, enemy, lead);
                x.abs() <= self.range
                    && (self.height[0]..=self.height[1]).contains(&y)
                    && (!self.front_only || x * ctx.facing >= -0.5)
            },
        )
    }
}

pub(super) fn jump_contact(
    window: AttackWindow,
    ctx: LedgeContext,
    enemy: Opponent,
    grounded: bool,
    floor_y: Option<f32>,
) -> bool {
    ActiveWindow {
        first: u16::from(window.first),
        last: u16::from(window.last),
    }
    .first_contact(
        ctx.input_delay,
        RecoveryWindow {
            vulnerable_in: if enemy.vulnerable {
                Some(0.0)
            } else {
                enemy.vulnerable_in
            },
            actionable_in: enemy.actionable_in,
        },
        |lead| {
            let [x, linear_y] = predicted_offset(ctx, enemy, lead);
            let enemy_y = enemy
                .air_physics
                .filter(|_| enemy.situation == 2)
                .map_or(enemy.velocity[1] * lead, |physics| {
                    physics.displacement(enemy.velocity[1], lead)
                });
            let own_air_time = if grounded {
                (lead - f32::from(JUMP_SQUAT_FRAMES)).max(0.0)
            } else {
                lead
            };
            let own_speed = if grounded {
                ctx.short_hop_speed
            } else {
                Some(ctx.velocity[1])
            };
            let own_y = ctx
                .air_physics
                .zip(own_speed)
                .map(|(physics, speed)| physics.displacement(speed, own_air_time));
            let y = own_y.map_or(linear_y, |own_y| {
                enemy.position[1] - ctx.position[1] + enemy_y - own_y
            });
            // A forecast below the takeoff floor cannot preserve aerial hitboxes.
            let still_airborne = own_y.is_none_or(|height| {
                if grounded {
                    height > 0.0
                } else {
                    floor_y.is_none_or(|floor| ctx.position[1] + height > floor)
                }
            });
            still_airborne
                && x.abs() <= window.range
                && (window.height[0]..=window.height[1]).contains(&y)
        },
    )
    .is_some()
}

pub(super) fn predicted_offset(ctx: LedgeContext, enemy: Opponent, lead: f32) -> [f32; 2] {
    // Grounded ledge animation travel ends with that animation. Do not extend a
    // roll's last measured velocity indefinitely beyond its native end frame.
    let travel = if matches!(enemy.status, CLIFF_ESCAPE | CLIFF_CLIMB) {
        enemy
            .motion
            .and_then(|motion| motion.remaining)
            .filter(|left| left.is_finite() && *left >= 0.0)
            .map_or(lead, |left| left.min(lead))
    } else {
        lead
    };
    // Relative position + estimated enemy travel - short own-momentum estimate.
    // Values here are world distances, not controller stick output. Limiting
    // ledge travel above avoids projecting a completed roll forever onward.
    let own_travel = if ctx.situation == 2 {
        lead
    } else {
        lead.min(2.0)
    };
    [
        enemy.position[0] - ctx.position[0] + finite(enemy.velocity[0]) * travel
            - finite(ctx.velocity[0]) * own_travel,
        enemy.position[1] - ctx.position[1] + finite(enemy.velocity[1]) * travel
            - finite(ctx.velocity[1]) * own_travel,
    ]
}

// Native kinetic readings may be unavailable in transitions; never turn NaN
// into a controller input or a successful geometry comparison.
pub(super) fn finite(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

/// Choose the input sequence from this fighter instance's real jump speeds.
/// `None` means neither modeled hop currently covers the target. Missing native
/// parameters retain the explicit short-hop fallback, not an invented speed.
pub(super) fn choose_hop(
    policy: crate::fighter_profiles::execution::Hop,
    frames: crate::fighter_profiles::MoveFrames,
    settings: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
) -> Option<bool> {
    use crate::fighter_profiles::execution::Hop;
    match policy {
        Hop::Short => return Some(false),
        Hop::Full => return Some(true),
        Hop::Adaptive => {}
    }
    let (Some(physics), Some(short), Some(full)) =
        (ctx.air_physics, ctx.short_hop_speed, ctx.full_hop_speed)
    else {
        return Some(false);
    };
    if !short.is_finite() || !full.is_finite() || short <= 0.0 || full <= 0.0 {
        return None;
    }
    // Initial ledge-jump poses can still report zero launch velocity. Wait for
    // actual launch evidence instead of launching every fighter on that cue.
    if matches!(enemy.status, super::CLIFF_JUMP1 | 0x7c) && enemy.velocity[1].abs() < 0.01 {
        return None;
    }
    for (is_full, speed) in [(false, short), (true, full)] {
        // Full hop needs airborne A after held X: include a second delivery leg.
        let setup = if is_full {
            4 + ctx.input_delay.ceil().clamp(0.0, 12.0) as u16
        } else {
            3
        };
        let contact = ActiveWindow {
            first: u16::from(frames.first) + setup,
            last: u16::from(frames.last) + setup,
        }
        .first_contact(
            ctx.input_delay,
            RecoveryWindow {
                vulnerable_in: if enemy.vulnerable {
                    Some(0.0)
                } else {
                    enemy.vulnerable_in
                },
                actionable_in: enemy.actionable_in,
            },
            |lead| {
                // Input transport precedes our jump squat; it is not airborne time.
                let own_time = (lead - ctx.input_delay - 3.0).max(0.0);
                let own_y = jump_displacement(physics, speed, own_time);
                let enemy_y = enemy
                    .air_physics
                    .map_or(enemy.velocity[1] * lead, |physics| {
                        jump_displacement(physics, enemy.velocity[1], lead)
                    });
                let relative_y = enemy.position[1] - ctx.position[1] + enemy_y - own_y;
                let x = predicted_offset(ctx, enemy, lead)[0];
                own_y > 0.0
                    && (-6.0..=settings.anti_air_height).contains(&relative_y)
                    && (settings.distance_agnostic || x.abs() <= settings.anti_air_range)
            },
        );
        if contact.is_some() {
            return Some(is_full);
        }
    }
    None
}

/// Constant-cost equivalent of the discrete gravity forecast, including a
/// fractional final step and the native terminal-speed clamp. Forecast only.
fn jump_displacement(physics: super::AirPhysics, speed: f32, frames: f32) -> f32 {
    let time = frames.clamp(0.0, 120.0);
    if time <= 1.0 {
        return speed * time;
    }
    let v = (speed - physics.gravity).max(-physics.terminal_speed);
    let remaining = time - 1.0;
    let steps = remaining.floor();
    let free = ((v + physics.terminal_speed) / physics.gravity)
        .ceil()
        .clamp(0.0, steps);
    speed + free * v
        - physics.gravity * free * (free - 1.0) * 0.5
        - (steps - free) * physics.terminal_speed
        + remaining.fract() * (v - steps * physics.gravity).max(-physics.terminal_speed)
}

#[cfg(test)]
mod execution_tests {
    use super::*;
    use crate::fighter_profiles::{execution::Hop, MoveFrames};
    use crate::ledge_trap::{
        tests::{context, enemy},
        AirPhysics, CLIFF_JUMP3,
    };
    #[test]
    fn adaptive_uses_native_height_and_can_choose_full_over_short() {
        let ctx = LedgeContext {
            air_physics: AirPhysics::new(0.1, 2.0),
            short_hop_speed: Some(2.0),
            full_hop_speed: Some(4.0),
            ..context()
        };
        let s = LedgeSettings {
            distance_agnostic: true,
            ..LedgeSettings::default()
        };
        let target = Opponent {
            position: [5.0, 35.0],
            situation: 2,
            actionable_in: Some(12.0),
            ..enemy(CLIFF_JUMP3, true)
        };
        let frames = MoveFrames { first: 7, last: 30 };
        assert_eq!(
            choose_hop(Hop::Adaptive, frames, s, ctx, target),
            Some(true)
        );
        assert_eq!(
            choose_hop(
                Hop::Adaptive,
                frames,
                s,
                ctx,
                Opponent {
                    position: [5.0, 10.0],
                    ..target
                }
            ),
            Some(false)
        );
        assert_eq!(choose_hop(Hop::Short, frames, s, ctx, target), Some(false));
    }
    #[test]
    fn unknown_native_physics_preserves_explicit_short_fallback_and_unseen_launch_waits() {
        let s = LedgeSettings::default();
        let frames = MoveFrames { first: 7, last: 30 };
        let target = enemy(super::super::CLIFF_JUMP1, true);
        assert_eq!(
            choose_hop(Hop::Adaptive, frames, s, context(), target),
            Some(false)
        );
        let ctx = LedgeContext {
            air_physics: AirPhysics::new(0.1, 2.0),
            short_hop_speed: Some(2.0),
            full_hop_speed: Some(4.0),
            ..context()
        };
        assert_eq!(choose_hop(Hop::Adaptive, frames, s, ctx, target), None);
    }
    #[test]
    fn constant_cost_displacement_matches_discrete_model() {
        let physics = AirPhysics::new(0.1, 2.0).unwrap();
        for speed in [-3.0, -2.0, 0.0, 2.0, 4.0] {
            for time in [0.0, 0.5, 1.0, 2.0, 7.25, 12.5, 40.0, 90.0] {
                assert!(
                    (jump_displacement(physics, speed, time) - physics.displacement(speed, time))
                        .abs()
                        < 0.002
                );
            }
        }
    }
}
