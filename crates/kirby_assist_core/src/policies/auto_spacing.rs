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
    pub engagement_range: f32,
    pub ground: AttackConfig,
    pub aerial: AttackConfig,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            minimum_distance: 12.,
            engagement_range: 20.,
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
    pub walk_cap: Option<f32>,
}

#[derive(Default)]
pub struct Spacing {
    tick: Option<i64>,
    output: Output,
    pending: bool,
    pending_frames: u8,
    delivered_tick: Option<i64>,
    pending_airborne: bool,
    pending_output: Output,
}

pub fn manual_walk(stick: [f32; 2], cap: f32) -> [f32; 2] {
    let cap = if cap.is_finite() { cap.clamp(0., 1.) } else { 0. };
    [if stick[0].is_finite() { stick[0].clamp(-cap, cap) } else { 0. }, stick[1]]
}

impl Spacing {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Only a winning mapper packet starts the acceptance wait.
    pub fn mapped(&mut self, tick: i64) {
        if self.tick == Some(tick) && self.output.attack && self.delivered_tick != Some(tick) {
            if !self.pending {
                self.pending_frames = 0;
            }
            self.pending = true;
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
        let airborne = c.situation == situation::AIR;
        let walk_cap = (!airborne).then_some(c.walk_stick.unwrap_or(0.));
        let Some(enemy) = target.filter(valid_target).filter(|e| {
            (e.position[0] - c.position[0]).abs() <= config.engagement_range
        }) else {
            self.pending = false;
            self.pending_frames = 0;
            self.output = Output { walk_cap, ..Output::default() };
            return self.output;
        };
        // Jump stays human-owned; outside engagement only the walk cap applies.
        let mut out = Output {
            controller: ControllerOutput {
                owned: (GAMEPLAY_BUTTONS & !SEQUENCE_JUMP_BUTTONS)
                    | crate::native_input::MAPPER_LEFT_STICK_DIRECTIONS
                    | crate::native_input::MAPPER_RIGHT_STICK_DIRECTIONS,
                stick: Some([0.; 2]),
                ..ControllerOutput::NONE
            },
            walk_cap,
            ..Output::default()
        };
        let ready = if airborne {
            c.can_air_attack
        } else {
            c.can_act
        };
        let dx = enemy.position[0] - c.position[0];
        let side = if dx.abs() > 0.001 { dx.signum() } else { c.facing.signum() };
        let aim = if self.pending_airborne {
            self.pending_output.controller.stick.unwrap_or([0.; 2])[0]
        } else {
            self.pending_output.right_stick[0]
        };
        if self.pending && aim * side < 0. {
            self.pending = false;
        }
        if self.pending {
            if airborne != self.pending_airborne
                || is_attack(c.status)
            {
                self.pending = false;
            } else if !c.stopped {
                self.pending_frames = self.pending_frames.saturating_add(1);
                if self.pending_frames >= 24 {
                    self.pending = false;
                }
            }
        }
        {
            let distance = dx.abs();
            let toward = c.velocity[0] * side;
            let closing = (c.velocity[0] - enemy.velocity[0]) * side;
            let braking = c.ground_brake.filter(|b| b.is_finite() && *b > 0.);
            let stop_distance = if toward > 0. {
                braking.map_or(toward * 3., |b| toward * toward / (2. * b))
            } else {
                0.
            };
            let retreat = is_attack(c.status)
                || distance <= config.minimum_distance + closing.max(0.) * 3. + stop_distance;
            let floor_safe = airborne || c.safe_ground[usize::from(side < 0.)];
            if retreat && floor_safe {
                out.controller.stick =
                    Some([-side * if airborne { 1. } else { walk_cap.unwrap_or(0.) }, 0.]);
            }
            let attack = if airborne {
                config.aerial
            } else {
                config.ground
            };
            if !self.pending
                && !c.stopped
                && ready
                && !is_attack(c.status)
                && attack_valid(attack, airborne)
                && predicts_hit(attack, c, enemy, side)
            {
                // Fair/Bair are relative to native facing. Never substitute the
                // opposite aerial while trying to turn an airborne fighter.
                let facing_ok = match attack.attack {
                    Attack::Forward if airborne => c.facing * side > 0.,
                    Attack::Back => c.facing * side < 0.,
                    _ => true,
                };
                if !airborne && c.facing * side < 0. {
                    if c.safe_ground[usize::from(side > 0.)] {
                        out.controller.stick = Some([side * walk_cap.unwrap_or(0.), 0.]);
                    }
                } else if facing_ok {
                    out.attack = true;
                    let direction = match attack.attack {
                        Attack::Neutral => [0.; 2],
                        Attack::Forward | Attack::Back => [side, 0.],
                        Attack::Up => [0., 1.],
                        Attack::Down => [0., -1.],
                        Attack::Disabled => unreachable!(),
                    };
                    if airborne {
                        // Aerial selection uses A plus the matching main-stick
                        // direction. Retreat starts once native startup is seen,
                        // never competing with the attack-selection packet.
                        out.controller.stick = Some(direction);
                        out.controller.requested = MAPPER_A;
                    } else {
                        out.controller.stick = Some([0.; 2]);
                        out.right_stick = direction;
                        out.controller.requested = match direction {
                            [x, _] if x > 0. => crate::native_input::MAPPER_RIGHT_STICK_RIGHT,
                            [x, _] if x < 0. => crate::native_input::MAPPER_RIGHT_STICK_LEFT,
                            [_, y] if y > 0. => crate::native_input::MAPPER_RIGHT_STICK_UP,
                            [_, y] if y < 0. => crate::native_input::MAPPER_RIGHT_STICK_DOWN,
                            _ => MAPPER_A,
                        };
                    }
                    self.pending_airborne = airborne;
                    self.pending_output = out;
                }
            }
        }
        if self.pending && !c.stopped {
            // Release between retries to create fresh A/C-stick edges. A mapped
            // Attack bit is delivery, not proof the configured move started.
            if self.pending_frames % 2 == 0 {
                out.controller.requested = self.pending_output.controller.requested;
                out.right_stick = self.pending_output.right_stick;
                out.attack = true;
            }
            out.controller.stick = self.pending_output.controller.stick;
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
        && config.engagement_range.is_finite()
        && config.engagement_range >= 0.
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

fn predicts_hit(a: AttackConfig, c: GameplayContext, e: Opponent, side: f32) -> bool {
    if !e.vulnerable || !c.input_delay.is_finite() || !(0. ..=30.).contains(&c.input_delay) {
        return false;
    }
    let gap = (e.position[0] - c.position[0]) * side;
    let first = c.input_delay + a.startup;
    let last = first + a.active_frames - 1.;
    let vx = (e.velocity[0] - c.velocity[0]) * side;
    // Both axes must connect on the same active frame. Native gravity and
    // terminal speed refine aerial height rather than extrapolating a rising
    // jump as a straight line through the entire startup.
    let Some((lo, hi)) = interval(gap, vx, 0., a.reach, first, last) else {
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
    const GROUND_WALK_STICK: f32 = 88. / 127.;
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
            walk_stick: Some(GROUND_WALK_STICK),
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
            engagement_range: 20.,
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
    fn auto_spacing_engagement_boundary_cancels_retries_without_stealing_movement() {
        let mut s = Spacing::default();
        let cfg = Config { minimum_distance: 15., engagement_range: 20., ..config() };
        let attack = s.sample(1, true, cfg, context(), enemy(20., 0.));
        assert!(attack.attack);
        s.mapped(1);
        let outside = s.sample(2, true, cfg, context(), enemy(20.01, -10.));
        assert_eq!(outside.controller, ControllerOutput::NONE);
        assert!(!outside.attack);
        assert!(s.sample(3, true, cfg, context(), enemy(20., 0.)).attack);
        let missing = s.sample(4, true, cfg, context(), None);
        assert_eq!(missing.controller, ControllerOutput::NONE);
        assert_eq!(missing.walk_cap, context().walk_stick);
    }

    #[test]
    fn auto_spacing_turns_ground_attacks_toward_enemy_before_pressing() {
        for attack in [Attack::Neutral, Attack::Forward, Attack::Up, Attack::Down] {
            for side in [-1., 1.] {
                let mut s = Spacing::default();
                let mut cfg = config();
                cfg.ground.attack = attack;
                let c = GameplayContext { facing: -side, ..context() };
                let turn = s.sample(1, true, cfg, c, enemy(side * 18., 0.));
                assert!(!turn.attack);
                assert_eq!(turn.controller.requested, 0);
                assert_eq!(turn.controller.stick, Some([side * GROUND_WALK_STICK, 0.]));
                let out = s.sample(2, true, cfg, GameplayContext { facing: side, ..c }, enemy(side * 18., 0.));
                assert!(out.attack);
                assert_eq!(out.controller.stick, Some([0.; 2]));
                s.mapped(2);
                let out = s.sample(3, true, cfg, GameplayContext {
                    facing: side, status: status::ATTACK_S3, can_act: false, ..c
                }, enemy(side * 18., 0.));
                assert!(!out.attack);
                assert_eq!(out.controller.stick, Some([-side * GROUND_WALK_STICK, 0.]));
            }
        }
    }

    #[test]
    fn auto_spacing_fair_uses_direction_on_every_attempt_then_immediately_drifts_back() {
        for side in [-1., 1.] {
            let mut s = Spacing::default();
            let c = GameplayContext {
                facing: side, situation: situation::AIR, status: status::FALL, ..context()
            };
            for tick in 1..=3 {
                let out = s.sample(tick, true, config(), c, enemy(side * 10., 0.));
                assert_eq!(out.controller.stick, Some([side, 0.]));
                assert_eq!(out.right_stick, [0.; 2]);
                assert_eq!(out.controller.requested, if tick == 2 { 0 } else { MAPPER_A });
                if out.attack { s.mapped(tick); }
            }
            let out = s.sample(4, true, config(), GameplayContext {
                status: status::ATTACK_AIR, can_air_attack: false,
                motion_kind: crate::timing::MOTION_ATTACK_AIR_F, ..c
            }, enemy(side * 10., 0.));
            assert!(!out.attack);
            assert_eq!(out.controller.stick, Some([-side, 0.]));
        }
    }

    #[test]
    fn auto_spacing_crossing_enemy_cancels_the_old_attack_direction() {
        let mut s = Spacing::default();
        let c = GameplayContext { situation: situation::AIR, status: status::FALL, ..context() };
        assert!(s.sample(1, true, config(), c, enemy(10., 0.)).attack);
        s.mapped(1);
        for tick in 2..=3 {
            let out = s.sample(tick, true, config(), c, enemy(-10., 0.));
            assert!(!out.attack);
            assert_eq!(out.controller.requested, 0);
        }
    }

    #[test]
    fn auto_spacing_native_walk_cap_is_maximal_and_preserves_manual_direction() {
        use crate::native_input::{fastest_walk_stick, mapped_stick_axis};
        for dash in [0.5, 0.66, 0.8, 1.] {
            let cap = fastest_walk_stick(0.2, dash).unwrap();
            let mapped = mapped_stick_axis(cap);
            assert!(f32::from(mapped) / 127. < dash);
            assert!(f32::from(mapped + 1) / 127. >= dash);
            for x in [-1., -0.1, 0., 0.1, 1.] {
                let result = manual_walk([x, 0.4], cap);
                assert_eq!(result, [x.clamp(-cap, cap), 0.4]);
            }
        }
        for (walk, dash) in [(0.2, f32::NAN), (0.2, 0.), (0.8, 0.5), (0.2, 1.1)] {
            assert_eq!(fastest_walk_stick(walk, dash), None);
        }
        let mut s = Spacing::default();
        let unknown = s.sample(1, true, config(), GameplayContext { walk_stick: None, ..context() }, enemy(40., 0.));
        assert_eq!(unknown.walk_cap, Some(0.));
        assert_eq!(s.sample(2, false, config(), context(), enemy(40., 0.)), Output::default());
    }

    #[test]
    fn auto_spacing_brakes_before_boundary_and_retreats_after_breach() {
        let mut s = Spacing::default();
        let c = GameplayContext {
            velocity: [2., 0.],
            ..context()
        };
        let out = s.sample(1, true, Config::default(), c, enemy(20., 0.));
        assert_eq!(out.controller.stick, Some([-GROUND_WALK_STICK, 0.]));
        assert!(!out.attack);
        let out = s.sample(2, true, Config::default(), context(), enemy(-8., 0.));
        assert_eq!(out.controller.stick, Some([GROUND_WALK_STICK, 0.]));
        assert!(!out.attack);
    }
    #[test]
    fn auto_spacing_attacks_approaches_and_breaches_without_recovery_gap_veto() {
        let mut cfg = config();
        cfg.ground.commitment = 12.;
        let mut s = Spacing::default();
        assert!(!s.sample(1, true, cfg, context(), enemy(24., -0.4)).attack);
        assert!(
            s.sample(2, true, config(), context(), enemy(20., -1.))
                .attack
        );
        assert!(
            !s.sample(3, true, config(), context(), enemy(18., 1.))
                .attack
        );
        cfg.minimum_distance = 21.;
        let out = s.sample(4, true, cfg, context(), enemy(20., 0.));
        assert!(out.attack);
        assert_eq!(out.controller.stick, Some([0.; 2]));
        assert_eq!(out.controller.requested, crate::native_input::MAPPER_RIGHT_STICK_RIGHT);
    }
    #[test]
    fn auto_spacing_outside_range_releases_ownership_and_retreat_never_dashes() {
        let mut s = Spacing::default();
        for side in [-1., 1.] {
            let c = GameplayContext {
                velocity: [side, 0.],
                ..context()
            };
            let out = s.sample(
                if side < 0. { 1 } else { 2 },
                true,
                config(),
                c,
                enemy(side * 80., 0.),
            );
            assert!(!out.attack);
            assert_eq!(out.controller, ControllerOutput::NONE);
            assert_eq!(out.walk_cap, Some(GROUND_WALK_STICK));
        }
        let ground = s.sample(3, true, Config::default(), context(), enemy(10., 0.));
        assert_eq!(ground.controller.stick, Some([-GROUND_WALK_STICK, 0.]));
        let air = s.sample(
            4,
            true,
            Config::default(),
            GameplayContext {
                situation: situation::AIR,
                status: status::FALL,
                ..context()
            },
            enemy(10., 0.),
        );
        assert_eq!(air.controller.stick, Some([-1., 0.]));
    }

    #[test]
    fn auto_spacing_retries_until_native_startup_not_readiness_loss() {
        let mut s = Spacing::default();
        s.sample(1, true, config(), context(), enemy(13., 0.));
        s.mapped(1);
        let c = GameplayContext {
            status: status::TURN,
            can_act: false,
            ..context()
        };
        let release = s.sample(2, true, config(), c, enemy(13., 0.));
        assert!(!release.attack);
        assert_eq!(release.controller.stick, Some([0.; 2]));
        let retry = s.sample(3, true, config(), c, enemy(13., 0.));
        assert!(retry.attack);
        assert_eq!(retry.controller.requested, crate::native_input::MAPPER_RIGHT_STICK_RIGHT);
        assert_eq!(retry.controller.stick, Some([0.; 2]));
        for tick in 4..=25 {
            let out = s.sample(tick, true, config(), c, enemy(13., 0.));
            if out.attack {
                s.mapped(tick);
            }
        }
        assert_eq!(s.output.controller.stick, Some([0.; 2]));
        assert_eq!(
            s.sample(25, false, config(), c, enemy(13., 0.)),
            Output::default()
        );
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
        let retry = s.sample(3, true, config(), c, enemy(18., 0.));
        assert!(retry.attack);
        assert_eq!(retry.controller.stick, Some([0.; 2]));
        s.mapped(3);
        c.status = status::ATTACK_S3;
        c.can_act = false;
        assert!(!s.sample(4, true, config(), c, enemy(13., 0.)).attack);
        assert_eq!(s.output.controller.stick, Some([-GROUND_WALK_STICK, 0.]));
        assert!(
            s.sample(5, true, config(), context(), enemy(18., 0.))
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
        assert_eq!(out.controller.stick, Some([1., 0.]));
        assert_eq!(out.right_stick, [0.; 2]);
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
        assert!(out.attack);
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
