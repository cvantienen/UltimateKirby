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
    /// Retired saved estimate; native readiness controls repetition.
    pub commitment: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    pub minimum_distance: f32,
    /// Retained configuration identity; move selection is fighter-independent.
    pub fighter_kind: i32,
    pub engagement_range: f32,
    /// Direct horizontal stick cap, also bounded by the native non-dash threshold.
    pub ground_speed: f32,
    /// Horizontal automatic-retreat stick caps; attack selection remains uncapped.
    pub ground_retreat_stick: f32,
    pub air_retreat_stick: f32,
    /// Retired automatic-approach controls retained for saved configuration compatibility.
    pub maximum_drift: f32,
    pub minimum_drift: f32,
    pub ground_braking: bool,
    pub air_braking: bool,
    pub braking_distance: f32,
    pub ground: AttackConfig,
    pub aerial: AttackConfig,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            minimum_distance: 15.,
            fighter_kind: -1,
            engagement_range: 20.,
            ground_speed: 0.49,
            ground_retreat_stick: 1.,
            air_retreat_stick: 1.,
            maximum_drift: 1.,
            minimum_drift: 0.1,
            ground_braking: true,
            air_braking: true,
            braking_distance: 10.,
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
    pending_side: f32,
    pending_attack: AttackConfig,
    pending_output: Output,
    retreat_side: Option<f32>,
    threat_retreat: Option<(u32, f32)>,
    ground_retreat: Option<GroundRetreat>,
    pending_config: Option<Config>,
    aerial_selection: Option<(i64, [f32; 2])>,
    recovery_input: bool,
}

struct GroundRetreat {
    target: u32,
    side: f32,
    start_x: f32,
}

const RETREAT_STEP: f32 = 3.;

pub fn manual_walk(stick: [f32; 2], cap: f32) -> [f32; 2] {
    let cap = if cap.is_finite() { cap.clamp(0., 1.) } else { 0. };
    [if stick[0].is_finite() { stick[0].clamp(-cap, cap) } else { 0. }, stick[1]]
}

pub fn manual_movement(stick: [f32; 2], caps: [f32; 2]) -> [f32; 2] {
    manual_walk(stick, caps[usize::from(stick[0] >= 0.)])
}

pub fn brake_ground_movement(status: i32, situation: i32) -> bool {
    situation == situation::GROUND && matches!(status,
        status::DASH | status::TURN_DASH | status::RUN | status::TURN_RUN
            | status::RUN_BRAKE | status::TURN_RUN_BRAKE)
}

pub fn mapped_walk_axis(axis: i8, cap: f32) -> i8 {
    let limit = crate::native_input::mapped_stick_axis(cap).max(0);
    axis.clamp(-limit, limit)
}

fn safe_walk(c: GameplayContext) -> f32 {
    c.walk_stick.filter(|v| v.is_finite() && (0. ..=1.).contains(v)).unwrap_or(0.)
}

fn ground_cap(config: Config, c: GameplayContext) -> f32 {
    config.ground_speed.min(c.walk_stick
        .filter(|v| v.is_finite() && (0. ..=1.).contains(v))
        .unwrap_or(0.49))
}

fn retreat_stick(config: Config, airborne: bool, side: f32) -> [f32; 2] {
    let cap = if airborne {
        config.air_retreat_stick
    } else {
        config.ground_retreat_stick
    };
    [-side * cap, 0.]
}

fn movement_caps(config: Config, c: GameplayContext, _target: Option<Opponent>) -> [f32; 2] {
    [if c.situation == situation::AIR { 1. } else { ground_cap(config, c) }; 2]
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
            if self.pending_airborne && self.aerial_selection.is_none() {
                self.aerial_selection = self.output.controller.stick.map(|stick| (tick, stick));
            }
        }
    }

    fn sample_retreat(
        &mut self,
        config: Config,
        c: GameplayContext,
        enemy: Opponent,
        side: f32,
        out: &mut Output,
    ) -> bool {
        let distance = (enemy.position[0] - c.position[0]).abs();
        let projected = distance + (enemy.velocity[0] - c.velocity[0]) * side * 5.;
        if c.situation == situation::AIR {
            self.ground_retreat = None;
            let boundary = config.minimum_distance;
            let retreat = projected.min(distance) < (boundary - RETREAT_STEP).max(0.)
                || self.retreat_side == Some(side) && distance < boundary + RETREAT_STEP;
            self.retreat_side = retreat.then_some(side);
            if retreat && !c.stopped {
                out.controller.stick = Some(retreat_stick(config, true, side));
            }
            retreat
        } else {
            self.retreat_side = None;
            if self.ground_retreat.as_ref().is_some_and(|retreat| {
                retreat.target != enemy.id
                    || retreat.side != side
                    || (retreat.start_x - c.position[0]) * retreat.side >= RETREAT_STEP
                        && distance >= config.minimum_distance + RETREAT_STEP
                    || distance >= (config.minimum_distance + RETREAT_STEP).max(32.)
            }) {
                self.ground_retreat = None;
            }
            // Finish a full outward step before releasing the retreat input.
            if self.ground_retreat.is_none()
                && projected.min(distance) < (config.minimum_distance - RETREAT_STEP).max(0.)
            {
                self.ground_retreat = Some(GroundRetreat {
                    target: enemy.id,
                    side,
                    start_x: c.position[0],
                });
            }
            let retreat = self
                .ground_retreat
                .as_ref()
                .is_some_and(|retreat| retreat.side == side);
            if retreat {
                self.pending = false;
                if !c.stopped {
                    out.controller.stick = Some(if c.safe_ground[usize::from(side < 0.)] {
                        retreat_stick(config, false, side)
                    } else {
                        [0.; 2]
                    });
                }
            }
            retreat
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
        self.sample_threat(tick, held, config, c, target, false)
    }

    pub fn sample_threat(
        &mut self,
        tick: i64,
        held: bool,
        config: Config,
        c: GameplayContext,
        target: Option<Opponent>,
        damaging_startup: bool,
    ) -> Output {
        if !held || !valid_context(config, c) {
            self.reset();
            return Output::default();
        }
        let recovery_input = c.can_air_attack
            && native_states::airborne_damage_input(c.status, c.situation, Some(0.0));
        if recovery_input && !self.recovery_input {
            self.reset();
        }
        self.recovery_input = recovery_input;
        let new_tick = self.tick != Some(tick);
        self.tick = Some(tick);
        let airborne = c.situation == situation::AIR;
        if self.aerial_selection.is_some_and(|(start, _)|
            !airborne || aerial_started(c)
                || !c.stopped && !c.can_air_attack && !is_attack(c.status)
                    && !attack_ready(c.status, c.situation, false)
                || tick < start || tick.saturating_sub(start) >= 24)
        {
            self.aerial_selection = None;
        }
        let caps = movement_caps(config, c, target);
        let walk_cap = (!airborne).then_some(ground_cap(config, c));
        let Some(enemy) = target.filter(valid_target) else {
            self.pending = false;
            self.pending_frames = 0;
            self.retreat_side = None;
            self.threat_retreat = None;
            self.ground_retreat = None;
            self.output = Output { walk_cap, movement_caps: Some(caps),
                ..Output::default() };
            if let Some((_, stick)) = self.aerial_selection {
                self.output.controller.stick = Some(stick);
                self.output.controller.owned = crate::native_input::MAPPER_LEFT_STICK_DIRECTIONS;
            }
            return self.output;
        };
        let mut out = Output {
            controller: ControllerOutput {
                owned: crate::native_input::MAPPER_LEFT_STICK_DIRECTIONS,
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
            c.can_act && !brake_ground_movement(c.status, c.situation)
        };
        let dx = enemy.position[0] - c.position[0];
        let side = if dx.abs() > 0.001 { dx.signum() } else { c.facing.signum() };
        let mut attack_context = c;
        if !airborne && (c.velocity[0] * side < 0.
            || brake_ground_movement(c.status, c.situation))
        {
            // A tilt waits for locomotion to end; do not extrapolate a dash
            // through its startup and veto the neutral input needed to brake.
            attack_context.velocity[0] = 0.;
        }
        let selected_attack = if airborne { config.aerial } else { config.ground };
        let engaged = config.engagement_range > 0. && dx.abs() <= config.engagement_range;
        if self.pending {
            if !engaged
                || !ready && !attack_ready(c.status, c.situation, false)
                || self.pending_config != Some(config)
                || self.pending_target != Some(enemy.id)
                || airborne != self.pending_airborne
                || self.pending_side != side
                || self.pending_attack != selected_attack
                || !attack_valid(selected_attack, airborne)
                || !facing_matches(selected_attack.attack, airborne, c.facing, side)
                || is_attack(c.status)
                || brake_ground_movement(c.status, c.situation)
                || !predicts_hit(selected_attack, attack_context, enemy, side)
            {
                self.pending = false;
            } else if new_tick && !c.stopped {
                self.pending_frames = self.pending_frames.saturating_add(1);
                if self.pending_frames >= 24 {
                    self.pending = false;
                }
            }
        }
        let attack_in_range = engaged && attack_valid(selected_attack, airborne)
            && predicts_hit(selected_attack, attack_context, enemy, side);
        let attack_priority = attack_in_range && !is_attack(c.status)
            && (!airborne || facing_matches(selected_attack.attack, true, c.facing, side));
        if attack_priority {
            self.threat_retreat = None;
            self.ground_retreat = None;
            self.retreat_side = None;
            out.controller.stick = Some([0.; 2]);
        }
        let incoming = !attack_priority && incoming_retreat(c, enemy, damaging_startup);
        if incoming {
            self.threat_retreat = Some((enemy.id, side));
        } else if self.threat_retreat.is_some_and(|(id, direction)| {
            id != enemy.id || direction != side
                || dx.abs() >= config.minimum_distance.max(30.) + 2.
        }) {
            self.threat_retreat = None;
        }
        if self.threat_retreat.is_some() {
            self.pending = false;
            self.ground_retreat = None;
            self.retreat_side = None;
            // Preserve an aerial's selecting axis until native motion confirms it.
            // Once confirmed, drift away even while that aerial is still running.
            out.controller.stick = self.aerial_selection.map(|(_, stick)| stick)
                .or_else(|| (!c.stopped).then_some(
                    if airborne || c.safe_ground[usize::from(side < 0.)] {
                        retreat_stick(config, airborne, side)
                    } else {
                        [0.; 2]
                    }));
            self.output = out;
            return out;
        }
        {
            if !attack_priority {
                self.sample_retreat(config, c, enemy, side, &mut out);
            }
            let attack = selected_attack;
            if !self.pending
                && self.aerial_selection.is_none()
                && !c.stopped
                && ready
                && !is_attack(c.status)
                && attack_valid(attack, airborne)
                && (airborne || attack.attack != Attack::Forward || safe_walk(c) > 0.)
                && attack_in_range
            {
                if !airborne && c.facing * side < 0. {
                    // Turn with a sub-dash input and wait for native facing before A.
                    if config.ground_speed > 0. && safe_walk(c) > 0.
                        && c.safe_ground[usize::from(side > 0.)]
                    {
                        out.controller.stick = Some([side * safe_walk(c).min(0.6), 0.]);
                    }
                } else if facing_matches(attack.attack, airborne, c.facing, side) {
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
                        // Select tilts directly with A and a sub-dash main stick.
                        // A raw full-strength C-stick can select a smash or flick
                        // movement before the final mapped spacing correction.
                        out.controller.stick = Some([
                            direction[0] * safe_walk(c).min(0.6),
                            direction[1] * 0.6,
                        ]);
                        out.controller.requested = MAPPER_A;
                    }
                    self.pending_target = Some(enemy.id);
                    self.pending_airborne = airborne;
                    self.pending_side = side;
                    self.pending_attack = attack;
                    self.pending_config = Some(config);
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
            out.controller.stick = if airborne || out.attack {
                self.pending_output.controller.stick
            } else {
                Some([0.; 2])
            };
        }
        // Native AttackAir entry can precede motion selection. A buffered A must
        // retain its selecting direction even if the target leaves the hit forecast.
        if let Some((_, stick)) = self.aerial_selection {
            out.controller.stick = Some(stick);
        }
        if out.attack || self.pending {
            out.controller.owned |= (GAMEPLAY_BUTTONS & !SEQUENCE_JUMP_BUTTONS)
                | crate::native_input::MAPPER_RIGHT_STICK_DIRECTIONS;
        }
        self.output = out;
        out
    }
}

/// Manual action selection must not inherit spacing's movement-only axis caps.
pub fn manual_action(buttons: u32, right_stick: [f32; 2]) -> bool {
    buttons & (MAPPER_A | crate::native_input::MAPPER_B | crate::native_input::MAPPER_R
        | crate::native_input::MAPPER_ZR | crate::native_input::MAPPER_RIGHT_STICK_DIRECTIONS) != 0
        || right_stick.iter().any(|v| v.is_finite() && v.abs() > 0.1)
}

pub fn manual_action_pending(
    buttons: u32,
    right_stick: [f32; 2],
    pressed: u32,
    status: i32,
    motion: u64,
    frame: f32,
) -> bool {
    let started = is_attack(status)
        && frame.is_finite()
        && frame > 0.
        && (status != status::ATTACK_AIR
            || aerial_started(GameplayContext {
                status,
                motion_kind: motion,
                motion_frame: frame,
                ..Default::default()
            }));
    manual_action(buttons, right_stick)
        && (!started
            || manual_action(pressed, [0.; 2])
            || buttons
                & (crate::native_input::MAPPER_B
                    | crate::native_input::MAPPER_R
                    | crate::native_input::MAPPER_ZR)
                != 0)
}

fn valid_context(config: Config, c: GameplayContext) -> bool {
    c.gameplay
        && c.can_control
        && (!native_states::sequence_interrupted(c.status)
            || c.can_air_attack
                && native_states::airborne_damage_input(c.status, c.situation, Some(0.0)))
        && !crate::action_owner::native_grab_active(c.status)
        && (!(status::CLIFF_CATCH_MOVE..=status::CLIFF_JUMP3).contains(&c.status)
            || c.can_act || c.can_air_attack)
        && matches!(c.situation, situation::GROUND | situation::AIR)
        && config.minimum_distance.is_finite()
        && config.minimum_distance >= 0.
        && config.engagement_range.is_finite()
        && config.engagement_range >= 0.
        && config.ground_speed.is_finite()
        && (0. ..=1.).contains(&config.ground_speed)
        && [config.ground_retreat_stick, config.air_retreat_stick]
            .into_iter()
            .all(|cap| cap.is_finite() && (0. ..=1.).contains(&cap))
        && c.position
            .iter()
            .chain(c.velocity.iter())
            .all(|v| v.is_finite())
        && c.facing.is_finite()
        && c.facing.abs() >= 0.5
}

pub fn damaging_startup(kind: i32, status: i32, motion: u64, frame: f32) -> bool {
    if matches!(status, native_states::status::ESCAPE..=native_states::status::ESCAPE_AIR_SLIDE
        | native_states::status::GUARD_ON..=native_states::status::GUARD_OFF)
        || !frame.is_finite()
        || frame < 0.
    {
        return false;
    }
    crate::fighter_data::fighter(kind)
        .and_then(|fighter| fighter.move_for_motion(motion))
        .is_some_and(|data| {
            data.has_damage_data()
                && data
                    .active
                    .iter()
                    .any(|window| frame < f32::from(window.first))
        })
}

fn incoming_retreat(c: GameplayContext, enemy: Opponent, damaging_startup: bool) -> bool {
    let dx = enemy.position[0] - c.position[0];
    let approach = enemy.velocity[0] * dx.signum() < -0.01
        && !matches!(enemy.status, status::ESCAPE..=status::ESCAPE_AIR_SLIDE
            | status::GUARD_ON..=status::GUARD_OFF);
    dx.abs() <= 30. && (damaging_startup || approach)
}

pub fn shielding(s: i32) -> bool {
    matches!(s, status::GUARD_ON | status::GUARD | status::GUARD_DAMAGE)
}

fn valid_target(e: &Opponent) -> bool {
    e.position
        .iter()
        .chain(e.velocity.iter())
        .all(|v| v.is_finite())
}

/// Locomotion states with ordinary attack transitions, independent of cancel windows.
pub fn attack_ready(s: i32, situation: i32, cancel_enabled: bool) -> bool {
    match situation {
        situation::GROUND => cancel_enabled || matches!(s,
            status::WAIT | status::WALK | status::WALK_BRAKE | status::TURN
                | status::SQUAT..=status::SQUAT_RV),
        situation::AIR => cancel_enabled || matches!(s,
            status::JUMP | status::JUMP_AERIAL | status::FALL | status::FALL_AERIAL),
        _ => false,
    }
}

fn facing_matches(attack: Attack, airborne: bool, facing: f32, side: f32) -> bool {
    if !airborne || attack == Attack::Forward {
        facing * side > 0.
    } else if attack == Attack::Back {
        facing * side < 0.
    } else {
        true
    }
}

fn aerial_started(c: GameplayContext) -> bool {
    c.status == status::ATTACK_AIR && c.motion_frame > 0.
        && ["attack_air_n", "attack_air_f", "attack_air_b", "attack_air_hi", "attack_air_lw"]
            .into_iter().any(|motion| c.motion_kind == crate::timing::motion_hash(motion))
}

fn is_attack(s: i32) -> bool {
    (status::ATTACK..=status::ATTACK_AIR).contains(&s)
}

fn attack_valid(a: AttackConfig, airborne: bool) -> bool {
    a.attack != Attack::Disabled
        && (airborne || a.attack != Attack::Back)
        && [a.startup, a.active_frames, a.reach, a.height]
            .iter()
            .all(|v| v.is_finite() && *v > 0.)
        && a.startup <= 300.
        && a.active_frames <= 300.
}

fn predicts_hit(a: AttackConfig, c: GameplayContext, e: Opponent, side: f32) -> bool {
    if !e.vulnerable || shielding(e.status)
        || !c.input_delay.is_finite() || !(0. ..=30.).contains(&c.input_delay) {
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
        if height_connects(a, height_at(time)) {
            return true;
        }
        time += 1.;
    }
    height_connects(a, height_at(hi))
}

fn height_connects(attack: AttackConfig, height: f32) -> bool {
    height.abs() <= attack.height && match attack.attack {
        Attack::Up => height >= 0.,
        Attack::Down => height <= 0.,
        _ => true,
    }
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
#[path = "../../tests/unit/policy_auto_spacing.rs"]
mod tests;
