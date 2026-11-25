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
    /// Fraction of the native fastest safe non-dash walk; zero stops ground movement.
    pub ground_speed: f32,
    pub maximum_drift: f32,
    pub minimum_drift: f32,
    pub ground_braking: bool,
    pub air_braking: bool,
    /// Root-distance zone above minimum spacing in which approach tapers.
    pub braking_distance: f32,
    pub ground: AttackConfig,
    pub aerial: AttackConfig,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            minimum_distance: 15.,
            engagement_range: 20.,
            ground_speed: 1.,
            maximum_drift: 1.,
            minimum_drift: 0.1,
            ground_braking: true,
            air_braking: true,
            braking_distance: 5.,
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
    /// Horizontal magnitude caps for leftward and rightward manual movement.
    pub movement_caps: Option<[f32; 2]>,
}

#[derive(Default)]
pub struct Spacing {
    tick: Option<i64>,
    output: Output,
    pending: bool,
    pending_target: Option<u32>,
    pending_frames: u8,
    delivered_tick: Option<i64>,
    pending_airborne: bool,
    pending_output: Output,
}

pub fn manual_walk(stick: [f32; 2], cap: f32) -> [f32; 2] {
    let cap = if cap.is_finite() { cap.clamp(0., 1.) } else { 0. };
    [if stick[0].is_finite() { stick[0].clamp(-cap, cap) } else { 0. }, stick[1]]
}

pub fn manual_movement(stick: [f32; 2], caps: [f32; 2]) -> [f32; 2] {
    manual_walk(stick, caps[usize::from(stick[0] >= 0.)])
}

pub fn mapped_walk_axis(axis: i8, cap: f32) -> i8 {
    let limit = crate::native_input::mapped_stick_axis(cap).max(0);
    axis.clamp(-limit, limit)
}

fn safe_walk(c: GameplayContext) -> f32 {
    c.walk_stick.filter(|v| v.is_finite() && (0. ..=1.).contains(v)).unwrap_or(0.)
}

fn movement_caps(config: Config, c: GameplayContext, target: Option<Opponent>) -> [f32; 2] {
    let airborne = c.situation == situation::AIR;
    let retreat = if airborne { 1. } else if config.ground_speed == 0. { 0. } else { safe_walk(c) };
    let maximum = if airborne { config.maximum_drift } else { retreat * config.ground_speed };
    let braking = if airborne { config.air_braking } else { config.ground_braking };
    let mut caps = [maximum; 2];
    if let Some(enemy) = target.filter(valid_target) {
        let dx = enemy.position[0] - c.position[0];
        let relative_velocity = enemy.velocity[0] - c.velocity[0];
        let closing_velocity = relative_velocity * dx.signum();
        let predicted_distance = (dx.abs() + closing_velocity * 3.).max(0.);
        let side = if dx.abs() > 0.001 { dx.signum() } else { c.facing.signum() };
        let approach = if braking && config.braking_distance > 0. {
            let effective_distance = dx.abs().min(predicted_distance);
            let fraction = ((effective_distance - config.minimum_distance) / config.braking_distance).clamp(0., 1.);
            // Smoothstep joins the normal cap and boundary cap without a slope jump.
            let shaped = fraction * fraction * (3. - 2. * fraction);
            let minimum = config.minimum_drift.min(maximum);
            minimum + (maximum - minimum) * shaped
        } else {
            maximum
        };
        caps[usize::from(side > 0.)] = approach;
        caps[usize::from(side < 0.)] = retreat;
    }
    caps
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
        mut config: Config,
        c: GameplayContext,
        target: Option<Opponent>,
    ) -> Output {
        let selected_attack = if c.situation == situation::AIR { config.aerial } else { config.ground };
        config.minimum_distance = if selected_attack.reach.is_finite() {
            if selected_attack.reach > 0. { selected_attack.reach * 0.75 } else { 15. }
        } else { 0. };
        if !held || !valid_context(config, c) {
            self.reset();
            return Output::default();
        }
        let new_tick = self.tick != Some(tick);
        self.tick = Some(tick);
        let airborne = c.situation == situation::AIR;
        let caps = movement_caps(config, c, target);
        let walk_cap = (!airborne).then_some(safe_walk(c));
        let Some(enemy) = target.filter(valid_target) else {
            self.pending = false;
            self.pending_frames = 0;
            self.output = Output { walk_cap, movement_caps: Some(caps),
                controller: if !airborne && matches!(c.status,
                    status::DASH | status::TURN_DASH | status::RUN | status::TURN_RUN
                        | status::RUN_BRAKE | status::TURN_RUN_BRAKE) {
                    ControllerOutput { stick: Some([0.; 2]),
                        owned: crate::native_input::MAPPER_LEFT_STICK_DIRECTIONS,
                        ..ControllerOutput::NONE }
                } else { ControllerOutput::NONE },
                ..Output::default() };
            return self.output;
        };
        let in_range = true; // Admission is determined by the move's projected active window.
        // Ground approach is independent of attack engagement; jumps stay human-owned.
        let mut out = Output {
            controller: ControllerOutput {
                owned: (GAMEPLAY_BUTTONS & !SEQUENCE_JUMP_BUTTONS)
                    | crate::native_input::MAPPER_LEFT_STICK_DIRECTIONS
                    | crate::native_input::MAPPER_RIGHT_STICK_DIRECTIONS,
                stick: None,
                ..ControllerOutput::NONE
            },
            walk_cap,
            movement_caps: Some(caps),
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
            if self.pending_target != Some(enemy.id)
                || airborne != self.pending_airborne
                || is_attack(c.status)
                || !predicts_hit(selected_attack, c, enemy, side)
            {
                self.pending = false;
            } else if new_tick && !c.stopped {
                self.pending_frames = self.pending_frames.saturating_add(1);
                if self.pending_frames >= 24 {
                    self.pending = false;
                }
            }
        }
        {
            let distance = dx.abs();
            let projected = distance + (enemy.velocity[0] - c.velocity[0]) * side * 3.;
            let retreat = is_attack(c.status) || projected.min(distance) <= config.minimum_distance;
            let floor_safe = airborne || c.safe_ground[usize::from(side < 0.)];
            if retreat && floor_safe {
                out.controller.stick =
                    Some([-side * caps[usize::from(side < 0.)], 0.]);
            } else if !airborne {
                let approach = !retreat && !c.stopped && ready
                    && c.safe_ground[usize::from(side > 0.)];
                out.controller.stick = Some([
                    if approach { side * caps[usize::from(side > 0.)] } else { 0. }, 0.,
                ]);
            }
            let attack = if airborne {
                config.aerial
            } else {
                config.ground
            };
            if in_range
                && !self.pending
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
                        out.controller.stick = Some([side * caps[usize::from(side > 0.)], 0.]);
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
                    self.pending_target = Some(enemy.id);
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
        if !airborne && matches!(c.status,
            status::DASH | status::TURN_DASH | status::RUN | status::TURN_RUN
                | status::RUN_BRAKE | status::TURN_RUN_BRAKE)
        {
            // Neutral input brakes existing dash/run momentum before walking resumes.
            out.controller.stick = Some([0.; 2]);
            out.controller.requested = 0;
            out.right_stick = [0.; 2];
            out.attack = false;
            self.pending = false;
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
        && config.braking_distance.is_finite()
        && config.braking_distance >= 0.
        && [config.ground_speed, config.maximum_drift, config.minimum_drift]
            .iter().all(|v| v.is_finite() && (0. ..=1.).contains(v))
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
            ground_speed: 1.,
            maximum_drift: 1.,
            minimum_drift: 0.1,
            ground_braking: true,
            air_braking: true,
            braking_distance: 10.,
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
    fn auto_spacing_brakes_existing_dash_with_or_without_target() {
        for state in [status::DASH, status::TURN_DASH, status::RUN, status::TURN_RUN,
            status::RUN_BRAKE, status::TURN_RUN_BRAKE] {
            for target in [None, enemy(40., -8.)] {
                let out = Spacing::default().sample(1, true, config(),
                    GameplayContext { status: state, ..context() }, target);
                assert_eq!(out.controller.stick, Some([0.; 2]));
                assert!(!out.attack);
            }
        }
    }

    #[test]
    fn auto_spacing_uses_move_reach_and_retreats_before_incoming_crosses_it() {
        let cfg = Config { minimum_distance: 999., engagement_range: 0., ..config() };
        let mut spacing = Spacing::default();
        let out = spacing.sample(1, true, cfg, context(), enemy(24., -4.));
        assert_eq!(out.controller.stick, Some([-GROUND_WALK_STICK, 0.]));
        assert!(Spacing::default().sample(2, true, cfg, context(), enemy(20., 0.)).attack);
    }

    #[test]
    fn auto_spacing_adaptive_caps_taper_only_in_final_ten_units_on_both_sides() {
        for maximum in [0.6, 1.] {
            for side in [-1., 1.] {
                let cfg = Config { minimum_distance: 15., engagement_range: 20.,
                    maximum_drift: maximum, minimum_drift: 0.1, ..config() };
                let c = GameplayContext { situation: situation::AIR, status: status::FALL, ..context() };
                for (distance, fraction) in [(40., 1.), (25., 1.), (24.99, 0.999),
                    (20., 0.5), (15., 0.), (10., 0.)] {
                    let mut s = Spacing::default();
                    let out = s.sample(1, true, cfg, c, enemy(side * distance, 0.));
                    let caps = out.movement_caps.unwrap();
                    let shaped = fraction * fraction * (3. - 2. * fraction);
                    let approach = 0.1 + (maximum - 0.1) * shaped;
                    assert!((caps[usize::from(side > 0.)] - approach).abs() < 0.0001);
                    assert_eq!(manual_movement([-side, 0.3], caps), [-side, 0.3]);
                    assert!((manual_movement([side, 0.3], caps)[0] - side * approach).abs() < 0.0001);
                    assert_eq!(manual_movement([side * 0.05, 0.3], caps), [side * 0.05, 0.3]);
                }
            }
        }
    }

    #[test]
    fn auto_spacing_projects_closing_and_receding_opponent_velocity_for_braking() {
        let cfg = Config { minimum_distance: 15., braking_distance: 10., ..Config::default() };
        let context = GameplayContext { situation: situation::AIR, ..context() };
        let closing = movement_caps(cfg, context, enemy(30., -5.));
        let receding = movement_caps(cfg, context, enemy(30., 5.));
        assert!(closing[1] < receding[1]);
        let own_closing = GameplayContext { velocity: [3., 0.], ..context };
        let cap = movement_caps(cfg, own_closing, enemy(30., 0.));
        assert!(cap[1] < receding[1]);
    }

    #[test]
    fn auto_spacing_ground_fraction_never_dashes_or_limits_retreat() {
        for ground_speed in [0., 0.4, 1.] {
            for side in [-1., 1.] {
                let cfg = Config { ground_speed, minimum_drift: 0.2, ..config() };
                let maximum = GROUND_WALK_STICK * ground_speed;
                let far = movement_caps(cfg, context(), enemy(side * 40., 0.));
                assert_eq!(far[usize::from(side > 0.)], maximum);
                let retreat = if ground_speed == 0. { 0. } else { GROUND_WALK_STICK };
                assert_eq!(far[usize::from(side < 0.)], retreat);
                let close = movement_caps(cfg, context(), enemy(side * 17., 0.));
                let minimum = 0.2_f32.min(maximum);
                assert_eq!(close[usize::from(side > 0.)], minimum + (maximum - minimum) * 0.5);
                for (distance, status) in [(10., status::WAIT), (18., status::ATTACK_S3)] {
                    let out = Spacing::default().sample(1, true,
                        Config { ground_speed, ..Config::default() },
                        GameplayContext { status, ..context() }, enemy(side * distance, 0.));
                    assert_eq!(out.controller.stick, Some([-side * retreat, 0.]));
                }
                for cap in close {
                    assert!(crate::native_input::mapped_stick_axis(cap) <= 88);
                }
            }
        }
        let cfg = Config { maximum_drift: 0.05, minimum_drift: 0.2, ..config() };
        let c = GameplayContext { situation: situation::AIR, ..context() };
        assert_eq!(movement_caps(cfg, c, enemy(12., 0.)), [1., 0.05]);
        assert_eq!(movement_caps(cfg, c, None), [0.05; 2]);
        let mut s = Spacing::default();
        assert_eq!(s.sample(1, false, cfg, c, enemy(12., 0.)), Output::default());
    }

    #[test]
    fn auto_spacing_auto_approaches_without_attacks_and_zero_stops_even_turning() {
        for side in [-1., 1.] {
            for speed in [0., 0.4, 1.] {
                let cfg = Config { ground_speed: speed, minimum_drift: 0., ..Config::default() };
                let c = GameplayContext { facing: -side, ..context() };
                for (distance, fraction) in [(40., 1.), (20., 1.), (17.5, 0.5), (15.01, 0.000011984)] {
                    let out = Spacing::default().sample(1, true, cfg, c, enemy(side * distance, 0.));
                    let x = out.controller.stick.unwrap()[0];
                    assert!((x - side * GROUND_WALK_STICK * speed * fraction).abs() < 0.00001);
                    assert!(!out.attack);
                    assert_eq!(out.controller.requested, 0);
                    assert_eq!(out.controller.owned & SEQUENCE_JUMP_BUTTONS, 0);
                    assert_ne!(out.controller.owned & crate::native_input::MAPPER_LEFT_STICK_DIRECTIONS, 0);
                }
                let blocked = Spacing::default().sample(1, true, cfg,
                    GameplayContext { safe_ground: [false; 2], ..c }, enemy(side * 40., 0.));
                assert_eq!(blocked.controller.stick, Some([0.; 2]));
                let retreat = Spacing::default().sample(1, true, cfg, c, enemy(side * 14., 0.));
                assert_eq!(retreat.controller.stick,
                    Some([-side * if speed == 0. { 0. } else { GROUND_WALK_STICK }, 0.]));
                let mut attack_cfg = config();
                attack_cfg.ground_speed = 0.;
                let turn = Spacing::default().sample(1, true, attack_cfg, c, enemy(side * 18., 0.));
                assert_eq!(turn.controller.stick, Some([0.; 2]));
                assert!(!turn.attack);
            }
        }
    }

    #[test]
    fn auto_spacing_braking_is_independent_and_local_to_configured_zone() {
        for airborne in [false, true] {
            for side in [-1., 1.] {
                for (ground_braking, air_braking) in [(false, true), (true, false)] {
                    let c = GameplayContext { situation: if airborne { situation::AIR } else { situation::GROUND }, ..context() };
                    let cfg = Config { ground_speed: 0.5, maximum_drift: 0.6,
                        ground_braking, air_braking, braking_distance: 4., ..Config::default() };
                    let maximum = if airborne { 0.6 } else { GROUND_WALK_STICK * 0.5 };
                    let retreat = if airborne { 1. } else { GROUND_WALK_STICK };
                    let enabled = if airborne { air_braking } else { ground_braking };
                    for (distance, fraction) in [(14., 0.), (15., 0.), (16., 0.15625), (17., 0.5), (19., 1.), (40., 1.)] {
                        let caps = movement_caps(cfg, c, enemy(side * distance, 0.));
                        let expected = if enabled { 0.1 + (maximum - 0.1) * fraction } else { maximum };
                        assert!((caps[usize::from(side > 0.)] - expected).abs() < 0.0001);
                        assert_eq!(caps[usize::from(side < 0.)], retreat);
                    }
                    let caps = movement_caps(Config { braking_distance: 0., ..cfg }, c, enemy(side * 15., 0.));
                    assert_eq!(caps[usize::from(side > 0.)], maximum);
                }
            }
        }
    }

    #[test]
    fn auto_spacing_engagement_boundary_cancels_retries_but_keeps_ground_approach() {
        let mut s = Spacing::default();
        let cfg = Config { minimum_distance: 15., engagement_range: 20., ..config() };
        let attack = s.sample(1, true, cfg, context(), enemy(20., 0.));
        assert!(attack.attack);
        s.mapped(1);
        let outside = s.sample(2, true, cfg, context(), enemy(20.01, -10.));
        assert_eq!(outside.controller.requested, 0);
        assert_eq!(outside.controller.stick, Some([-GROUND_WALK_STICK, 0.]));
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
                assert_eq!(turn.controller.stick,
                    Some([side * turn.movement_caps.unwrap()[usize::from(side > 0.)], 0.]));
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
    fn auto_spacing_tapers_before_boundary_and_retreats_after_breach() {
        let mut s = Spacing::default();
        let c = context();
        let out = s.sample(1, true, Config::default(), c, enemy(17.5, 0.));
        let approach = 0.1 + (GROUND_WALK_STICK - 0.1) * 0.5;
        assert_eq!(out.controller.stick, Some([approach, 0.]));
        assert_eq!(out.movement_caps, Some([GROUND_WALK_STICK, approach]));
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
        assert!(s.sample(1, true, cfg, context(), enemy(24., -0.4)).attack);
        s.reset();
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
    fn auto_spacing_outside_range_approaches_and_retreat_never_dashes() {
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
            assert_eq!(out.controller.stick, Some([side * GROUND_WALK_STICK, 0.]));
            assert_eq!(out.controller.requested, 0);
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
        assert_eq!(s.output.controller.stick, Some([-GROUND_WALK_STICK, 0.]));
        assert_eq!(
            s.sample(25, false, config(), c, enemy(13., 0.)),
            Output::default()
        );
    }

    #[test]
    fn auto_spacing_same_tick_native_changes_override_cached_attack() {
        for (c, target) in [
            (context(), None),
            (context(), enemy(40., 0.)),
            (GameplayContext { stopped: true, ..context() }, enemy(18., 0.)),
            (GameplayContext { status: status::ATTACK_S3, ..context() }, enemy(18., 0.)),
        ] {
            let mut s = Spacing::default();
            assert!(s.sample(1, true, config(), context(), enemy(18., 0.)).attack);
            s.mapped(1);
            let out = s.sample(1, true, config(), c, target);
            assert!(!out.attack);
            assert_eq!(out.controller.requested, 0);
        }
    }

    #[test]
    fn auto_spacing_changed_target_cannot_inherit_a_pending_attack() {
        let mut s = Spacing::default();
        assert!(s.sample(1, true, config(), context(), enemy(18., 0.)).attack);
        s.mapped(1);
        assert!(!s.sample(2, true, config(), context(), enemy(18., 0.)).attack);
        let replacement = Opponent {
            id: 2,
            vulnerable: false,
            ..enemy(18., 0.).unwrap()
        };
        let out = s.sample(3, true, config(), context(), Some(replacement));
        assert!(!out.attack);
        assert_eq!(out.controller.requested, 0);
    }

    #[test]
    fn auto_spacing_same_tick_rechecks_do_not_spend_retry_budget() {
        let mut s = Spacing::default();
        s.sample(1, true, config(), context(), enemy(18., 0.));
        s.mapped(1);
        let released = s.sample(2, true, config(), context(), enemy(18., 0.));
        for _ in 0..40 {
            assert_eq!(s.sample(2, true, config(), context(), enemy(18., 0.)), released);
        }
        assert!(s.sample(3, true, config(), context(), enemy(18., 0.)).attack);
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
