//! Held, input-only spacing. Distances are root-based estimates, not hitbox edits.
use crate::{
    gameplay::{ControllerOutput, GameplayContext, Opponent},
    native_input::{GAMEPLAY_BUTTONS, MAPPER_A, SEQUENCE_JUMP_BUTTONS},
    native_states::{self, situation, status},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Attack {
    #[default]
    Disabled,
    Neutral,
    Forward,
    Up,
    Down,
    Back,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AttackConfig {
    pub attack: Attack,
    pub startup: f32,
    pub active_frames: f32,
    pub reach: f32,
    pub height: f32,
    /// Earliest estimated return to movement, including recovery.
    pub commitment: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    pub minimum_distance: f32,
    pub ground: AttackConfig,
    pub aerial: AttackConfig,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            minimum_distance: 12.,
            ground: AttackConfig::default(),
            aerial: AttackConfig::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Output {
    pub controller: ControllerOutput,
    pub right_stick: [f32; 2],
    pub attack: bool,
}

#[derive(Default)]
pub struct Spacing {
    tick: Option<i64>,
    output: Output,
    pending: bool,
    pending_pose: Option<(i32, u64, f32)>,
    pending_frames: u8,
    delivered_tick: Option<i64>,
}

impl Spacing {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Only a winning mapper packet starts the acceptance wait.
    pub fn mapped(&mut self, tick: i64) {
        if self.tick == Some(tick) && self.output.attack && self.delivered_tick != Some(tick) {
            self.pending = true;
            self.pending_frames = 0;
            self.delivered_tick = Some(tick);
        }
    }

    pub fn sample(
        &mut self,
        tick: i64,
        held: bool,
        config: Config,
        c: GameplayContext,
        target: Option<Opponent>,
    ) -> Output {
        if !held || !valid_context(config, c) {
            self.reset();
            return Output::default();
        }
        if self.tick == Some(tick) {
            return self.output;
        }
        self.tick = Some(tick);
        // Jump remains human-owned. Every other gameplay source, including both
        // sticks, is replaced even on a neutral wait or a missing-target frame.
        let mut out = Output {
            controller: ControllerOutput {
                owned: (GAMEPLAY_BUTTONS & !SEQUENCE_JUMP_BUTTONS)
                    | crate::native_input::MAPPER_LEFT_STICK_DIRECTIONS
                    | crate::native_input::MAPPER_RIGHT_STICK_DIRECTIONS,
                stick: Some([0.; 2]),
                ..ControllerOutput::NONE
            },
            ..Output::default()
        };
        let pose = (c.status, c.motion_kind, c.motion_frame);
        let airborne = c.situation == situation::AIR;
        let ready = if airborne {
            c.can_air_attack
        } else {
            c.can_act
        };
        if self.pending {
            if !ready || is_attack(c.status) {
                self.pending = false;
            } else if !c.stopped && self.pending_pose != Some(pose) {
                self.pending_frames = self.pending_frames.saturating_add(1);
                if self.pending_frames >= 24 {
                    self.pending = false;
                }
            }
        }
        self.pending_pose = Some(pose);
        if let Some(enemy) = target.filter(valid_target) {
            let dx = enemy.position[0] - c.position[0];
            let side = if dx.abs() > 0.001 {
                dx.signum()
            } else {
                c.facing.signum()
            };
            let distance = dx.abs();
            let toward = c.velocity[0] * side;
            let closing = (c.velocity[0] - enemy.velocity[0]) * side;
            let braking = c.ground_brake.filter(|b| b.is_finite() && *b > 0.);
            let stop_distance = if toward > 0. {
                braking.map_or(f32::INFINITY, |b| toward * toward / (2. * b))
            } else {
                0.
            };
            let retreat = toward > 0.
                || distance <= config.minimum_distance + 2. + closing.max(0.) * 3. + stop_distance;
            let floor_safe = airborne || c.safe_ground[usize::from(side < 0.)];
            if retreat && floor_safe {
                out.controller.stick = Some([-side, 0.]);
            }
            let attack = if airborne {
                config.aerial
            } else {
                config.ground
            };
            if !self.pending
                && !c.stopped
                && ready
                && attack_valid(attack, airborne)
                && predicts_hit(config.minimum_distance, attack, c, enemy, side)
            {
                // Fair/Bair are relative to native facing. Never substitute the
                // opposite aerial while trying to turn an airborne fighter.
                let facing_ok = match attack.attack {
                    Attack::Forward if airborne => c.facing * side > 0.,
                    Attack::Back => c.facing * side < 0.,
                    _ => true,
                };
                // Neutral attacks require a neutral left stick on the press;
                // do not trade an urgent retreat for a jab/neutral-air input.
                let neutral_ok = attack.attack != Attack::Neutral || !retreat;
                if facing_ok && neutral_ok && floor_safe {
                    out.attack = true;
                    match attack.attack {
                        Attack::Neutral => out.controller.requested = MAPPER_A,
                        Attack::Forward | Attack::Back => out.right_stick = [side, 0.],
                        Attack::Up => out.right_stick = [0., 1.],
                        Attack::Down => out.right_stick = [0., -1.],
                        Attack::Disabled => unreachable!(),
                    }
                }
            }
        }
        self.output = out;
        out
    }
}

fn valid_context(config: Config, c: GameplayContext) -> bool {
    c.gameplay
        && c.can_control
        && !native_states::sequence_interrupted(c.status)
        && !crate::action_owner::native_grab_active(c.status)
        && matches!(c.situation, situation::GROUND | situation::AIR)
        && config.minimum_distance.is_finite()
        && config.minimum_distance >= 0.
        && c.position
            .iter()
            .chain(c.velocity.iter())
            .all(|v| v.is_finite())
        && c.facing.is_finite()
        && c.facing.abs() >= 0.5
}

fn valid_target(e: &Opponent) -> bool {
    e.position
        .iter()
        .chain(e.velocity.iter())
        .all(|v| v.is_finite())
}

fn is_attack(s: i32) -> bool {
    (status::ATTACK..=status::ATTACK_AIR).contains(&s)
}

fn attack_valid(a: AttackConfig, airborne: bool) -> bool {
    a.attack != Attack::Disabled
        && (airborne || a.attack != Attack::Back)
        && [a.startup, a.active_frames, a.reach, a.height, a.commitment]
            .iter()
            .all(|v| v.is_finite() && *v > 0.)
        && a.startup <= 120.
        && a.active_frames <= 120.
        && a.commitment <= 240.
        && a.commitment >= a.startup + a.active_frames - 1.
}

fn predicts_hit(minimum: f32, a: AttackConfig, c: GameplayContext, e: Opponent, side: f32) -> bool {
    if !e.vulnerable || !c.input_delay.is_finite() || !(0. ..=30.).contains(&c.input_delay) {
        return false;
    }
    let gap = (e.position[0] - c.position[0]) * side;
    let airborne = c.situation == situation::AIR;
    // A grounded attack can stop retreat drift. Do not budget that drift as
    // guaranteed spacing during recovery; inward momentum remains a liability.
    let own_velocity = if airborne {
        c.velocity[0] * side
    } else {
        (c.velocity[0] * side).max(0.)
    };
    let relative = e.velocity[0] * side - own_velocity;
    let end = c.input_delay + a.commitment;
    if gap.min(gap + relative * end) < minimum {
        return false;
    }
    let first = c.input_delay + a.startup;
    let last = first + a.active_frames - 1.;
    let vx = (e.velocity[0] - c.velocity[0]) * side;
    // Both axes must connect on the same active frame. Native gravity and
    // terminal speed refine aerial height rather than extrapolating a rising
    // jump as a straight line through the entire startup.
    let Some((lo, hi)) = interval(gap, vx, minimum, a.reach, first, last) else {
        return false;
    };
    let height_at = |time| {
        e.position[1] + vertical_travel(e.velocity[1], e.air_physics, time)
            - c.position[1]
            - vertical_travel(c.velocity[1], c.air_physics, time)
    };
    let mut time = lo;
    while time <= hi {
        if height_at(time).abs() <= a.height {
            return true;
        }
        time += 1.;
    }
    height_at(hi).abs() <= a.height
}

fn vertical_travel(speed: f32, physics: Option<crate::gameplay::AirPhysics>, time: f32) -> f32 {
    let Some(physics) =
        physics.and_then(|p| crate::gameplay::AirPhysics::new(p.gravity, p.terminal_speed))
    else {
        return speed * time;
    };
    let speed = speed.max(-physics.terminal_speed);
    let accelerating = ((speed + physics.terminal_speed) / physics.gravity).min(time);
    speed * accelerating
        - 0.5 * physics.gravity * accelerating * accelerating
        - physics.terminal_speed * (time - accelerating)
}

fn interval(
    position: f32,
    velocity: f32,
    min: f32,
    max: f32,
    lo: f32,
    hi: f32,
) -> Option<(f32, f32)> {
    if min > max {
        return None;
    }
    if velocity.abs() < 0.0001 {
        return (position >= min && position <= max).then_some((lo, hi));
    }
    let a = (min - position) / velocity;
    let b = (max - position) / velocity;
    let lo = lo.max(a.min(b));
    let hi = hi.min(a.max(b));
    (lo <= hi).then_some((lo, hi))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn context() -> GameplayContext {
        GameplayContext {
            gameplay: true,
            can_control: true,
            can_act: true,
            can_air_attack: true,
            situation: situation::GROUND,
            status: status::WAIT,
            facing: 1.,
            ground_brake: Some(0.1),
            safe_ground: [true; 2],
            ..Default::default()
        }
    }
    fn config() -> Config {
        let a = AttackConfig {
            attack: Attack::Forward,
            startup: 8.,
            active_frames: 5.,
            reach: 20.,
            height: 9.,
            commitment: 33.,
        };
        Config {
            minimum_distance: 12.,
            ground: a,
            aerial: AttackConfig {
                startup: 6.,
                active_frames: 3.,
                commitment: 30.,
                ..a
            },
        }
    }
    fn enemy(x: f32, vx: f32) -> Option<Opponent> {
        Some(Opponent {
            id: 1,
            position: [x, 0.],
            velocity: [vx, 0.],
            vulnerable: true,
            ..Default::default()
        })
    }
    #[test]
    fn auto_spacing_brakes_before_boundary_and_retreats_after_breach() {
        let mut s = Spacing::default();
        let c = GameplayContext {
            velocity: [2., 0.],
            ..context()
        };
        let out = s.sample(1, true, config(), c, enemy(22., 0.));
        assert_eq!(out.controller.stick, Some([-1., 0.]));
        assert!(!out.attack);
        let out = s.sample(2, true, config(), context(), enemy(-8., 0.));
        assert_eq!(out.controller.stick, Some([1., 0.]));
        assert!(!out.attack);
    }
    #[test]
    fn auto_spacing_predicts_active_window_without_sacrificing_recovery_gap() {
        let mut cfg = config();
        cfg.ground.commitment = 12.;
        let mut s = Spacing::default();
        assert!(s.sample(1, true, cfg, context(), enemy(24., -0.4)).attack);
        assert!(
            !s.sample(2, true, config(), context(), enemy(20., -1.))
                .attack
        );
        assert!(
            !s.sample(3, true, config(), context(), enemy(18., 1.))
                .attack
        );
        cfg.minimum_distance = 21.;
        assert!(!s.sample(4, true, cfg, context(), enemy(20., 0.)).attack);
    }
    #[test]
    fn auto_spacing_release_is_immediate_even_on_same_tick_and_keeps_jump_manual() {
        let mut s = Spacing::default();
        let out = s.sample(1, true, config(), context(), enemy(18., 0.));
        assert!(out.attack);
        assert_eq!(out.controller.owned & SEQUENCE_JUMP_BUTTONS, 0);
        s.mapped(1);
        assert_eq!(
            s.sample(1, false, config(), context(), enemy(18., 0.)),
            Output::default()
        );
        assert!(
            s.sample(2, true, config(), context(), enemy(18., 0.))
                .attack
        );
    }
    #[test]
    fn auto_spacing_repeats_only_after_delivery_and_native_recovery() {
        let mut s = Spacing::default();
        let mut c = context();
        assert!(s.sample(1, true, config(), c, enemy(18., 0.)).attack);
        s.mapped(1);
        assert!(!s.sample(2, true, config(), c, enemy(18., 0.)).attack);
        c.status = status::ATTACK_S3;
        c.can_act = false;
        assert!(!s.sample(3, true, config(), c, enemy(18., 0.)).attack);
        assert!(
            s.sample(4, true, config(), context(), enemy(18., 0.))
                .attack
        );
    }
    #[test]
    fn auto_spacing_aerial_gravity_and_vertical_range_are_predicted() {
        let mut s = Spacing::default();
        let c = GameplayContext {
            situation: situation::AIR,
            status: status::FALL,
            velocity: [0., 3.],
            air_physics: crate::gameplay::AirPhysics::new(0.5, 3.),
            ..context()
        };
        let mut cfg = config();
        cfg.aerial.height = 3.;
        let target = Opponent {
            position: [18., 9.],
            vulnerable: true,
            ..Default::default()
        };
        assert!(s.sample(1, true, cfg, c, Some(target)).attack);
        assert!(
            !s.sample(
                2,
                true,
                cfg,
                GameplayContext {
                    air_physics: None,
                    ..c
                },
                Some(target)
            )
            .attack
        );
        assert!(
            !s.sample(
                3,
                true,
                cfg,
                c,
                Some(Opponent {
                    vulnerable: false,
                    ..target
                })
            )
            .attack
        );
    }

    #[test]
    fn auto_spacing_ownership_and_mapped_attack_preserve_jump() {
        use crate::action_owner::{ActionOwnership, Intent, Owner};
        use crate::native_input::*;
        let mut ownership = ActionOwnership::new();
        let intent = Intent {
            owner: Owner::Spacing,
            present: true,
            committed: true,
        };
        ownership.set_automatic_suspended(true);
        assert_eq!(ownership.select(true, &[intent]), Some(Owner::Spacing));
        assert!(!ownership.allows(Owner::Ledge));
        assert!(!ownership.allows(Owner::RollRead));
        let safety = Intent {
            owner: Owner::Safety,
            present: true,
            committed: false,
        };
        assert_eq!(
            ownership.select(true, &[intent, safety]),
            Some(Owner::Safety)
        );
        ownership.release(Owner::Spacing);
        assert_eq!(ownership.select(true, &[]), None);
        let physical = MAPPED_JUMP | MAPPED_SPECIAL | MAPPED_GUARD | MAPPED_CATCH;
        let attack = mapped_spacing_attack(physical, true, true);
        assert_ne!(attack & MAPPED_JUMP, 0);
        assert_ne!(attack & MAPPED_ATTACK, 0);
        assert_eq!(attack & (MAPPED_SPECIAL | MAPPED_GUARD | MAPPED_CATCH), 0);
        assert_eq!(mapped_spacing_attack(attack, false, false), MAPPED_JUMP);
    }

    #[test]
    fn auto_spacing_air_drift_facing_edges_and_invalid_observations() {
        let mut s = Spacing::default();
        let c = GameplayContext {
            situation: situation::AIR,
            status: status::FALL,
            ..context()
        };
        let out = s.sample(1, true, config(), c, enemy(13., 0.));
        assert_eq!(out.controller.stick, Some([-1., 0.]));
        assert_eq!(out.right_stick, [1., 0.]);
        assert!(
            !s.sample(
                2,
                true,
                config(),
                GameplayContext { facing: -1., ..c },
                enemy(13., 0.)
            )
            .attack
        );
        let out = s.sample(
            3,
            true,
            config(),
            GameplayContext {
                safe_ground: [false; 2],
                ..context()
            },
            enemy(8., 0.),
        );
        assert_eq!(out.controller.stick, Some([0.; 2]));
        assert!(!out.attack);
        assert_eq!(
            s.sample(
                4,
                true,
                config(),
                GameplayContext {
                    position: [f32::NAN, 0.],
                    ..c
                },
                enemy(13., 0.)
            ),
            Output::default()
        );
        let out = s.sample(5, true, config(), c, None);
        assert_eq!(out.right_stick, [0.; 2]);
        assert!(!out.attack);
    }
}
