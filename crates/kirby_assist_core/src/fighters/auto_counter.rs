//! Input-only counter policy for explicitly configured fighters.
use crate::{
    autopilot_guard::Hitbox,
    fighter_tuning::{self as tuning, Tuning},
    gameplay::ControllerOutput,
    ledge_trap::SEQUENCE_BUTTONS,
    native_input::MAPPER_B,
    timing::{AnimationClock, AnimationSample},
};

/// Register verified down-special counter adapters here. Settings still opt in per fighter.
pub fn supports(kind: i32) -> bool {
    kind == crate::fighter_kinds::LUCINA
}

#[derive(Clone, Copy)]
pub struct Config {
    pub startup: f32,
    pub margin: f32,
    pub half_width: f32,
    pub height: f32,
    pub half_depth: f32,
}
impl Config {
    pub fn for_fighter(kind: i32, values: Tuning) -> Option<Self> {
        if !supports(kind) || values.0[tuning::COUNTER_DEFAULT].is_none() {
            return None;
        }
        Some(Self {
            startup: values.get(tuning::COUNTER_STARTUP),
            margin: values.get(tuning::COUNTER_MARGIN),
            half_width: values.get(tuning::COUNTER_HALF_WIDTH),
            height: values.get(tuning::COUNTER_HEIGHT),
            half_depth: values.get(tuning::COUNTER_HALF_DEPTH),
        })
    }
    pub fn horizon(self) -> f32 {
        self.startup + self.margin
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Threat {
    pub id: u32,
    pub motion: u64,
    pub status: i32,
    pub frame: f32,
    pub in_frames: f32,
}

/// Only active, direct melee hitboxes are eligible. There is no assumed startup
/// attack for a fighter without observable geometry.
pub fn predict(
    config: Config,
    origin: [f32; 3],
    velocity: [f32; 2],
    id: u32,
    motion: u64,
    status: i32,
    frame: f32,
    hit: Hitbox,
) -> Option<Threat> {
    let values = origin
        .iter()
        .chain(velocity.iter())
        .chain(hit.start.iter())
        .chain(hit.end.iter())
        .chain(hit.velocity.iter())
        .chain([
            &hit.radius,
            &config.startup,
            &config.margin,
            &config.half_width,
            &config.height,
            &config.half_depth,
        ]);
    if !values.into_iter().all(|v| v.is_finite())
        || !frame.is_finite()
        || !(1.0..=30.0).contains(&config.startup)
        || !(0.0..=10.0).contains(&config.margin)
        || !(0.0..=40.0).contains(&config.horizon())
        || config.half_width <= 0.0
        || config.height <= 0.0
        || config.half_depth <= 0.0
        || hit.radius <= 0.0
        || hit.radius > 100.0
    {
        return None;
    }
    let enter = crate::projectile_geometry::swept_capsule_aabb(
        hit,
        [
            origin[0] - config.half_width,
            origin[1],
            origin[2] - config.half_depth,
        ],
        [
            origin[0] + config.half_width,
            origin[1] + config.height,
            origin[2] + config.half_depth,
        ],
        velocity,
        config.horizon(),
    )?;
    Some(Threat {
        id,
        motion,
        status,
        frame,
        in_frames: enter,
    })
}

/// Forecast a catalogued attack before its hitbox appears. Only a nearby
/// opponent with a verified damaging motion and active window is eligible.
pub fn predict_startup(
    config: Config,
    origin: [f32; 3],
    velocity: [f32; 2],
    enemy_position: [f32; 3],
    enemy_velocity: [f32; 2],
    id: u32,
    kind: i32,
    motion: u64,
    status: i32,
    frame: f32,
    rate: f32,
) -> Option<Threat> {
    if !origin
        .iter()
        .chain(velocity.iter())
        .chain(enemy_position.iter())
        .chain(enemy_velocity.iter())
        .all(|v| v.is_finite())
        || !frame.is_finite()
        || frame < 0.0
        || !rate.is_finite()
        || rate <= 0.0
        || !(1.0..=30.0).contains(&config.startup)
        || !(0.0..=10.0).contains(&config.margin)
        || !(0.0..=40.0).contains(&config.horizon())
        || config.half_width <= 0.0
        || config.height <= 0.0
        || config.half_depth <= 0.0
    {
        return None;
    }
    let move_data = crate::fighter_data::fighter(kind)?.move_for_motion(motion)?;
    if !move_data.has_damage_data() {
        return None;
    }
    let in_frames = move_data
        .active
        .iter()
        .filter(|window| frame <= window.first as f32)
        .map(|window| (window.first as f32 - frame) / rate)
        .filter(|t| (config.startup..=config.horizon()).contains(t))
        .min_by(|a, b| a.total_cmp(b))?;
    let dx = enemy_position[0] - origin[0] + (enemy_velocity[0] - velocity[0]) * in_frames;
    let dy = enemy_position[1] - origin[1] + (enemy_velocity[1] - velocity[1]) * in_frames;
    // The catalog has move timing but no hitbox reach. Limit startup
    // predictions to nearby opponents; active geometry remains the fallback.
    if dx.abs() > config.half_width + 18.0
        || dy < -8.0
        || dy > config.height + 8.0
        || (enemy_position[2] - origin[2]).abs() > config.half_depth + 4.0
    {
        return None;
    }
    Some(Threat {
        id,
        motion,
        status,
        frame,
        in_frames,
    })
}

/// Keep the earliest counterable contact, rather than letting an already
/// unavoidable hitbox hide a second attack with enough startup remaining.
pub fn prefer_actionable(config: Config, best: Option<Threat>, candidate: Threat) -> Option<Threat> {
    if !candidate.in_frames.is_finite()
        || !(config.startup..=config.horizon()).contains(&candidate.in_frames)
    {
        return best;
    }
    if best.is_none_or(|old| candidate.in_frames < old.in_frames) {
        Some(candidate)
    } else {
        best
    }
}

pub fn ready(status: i32, situation: i32) -> bool {
    match situation {
        crate::native_states::situation::GROUND => matches!(status,
            crate::native_states::status::WAIT..=crate::native_states::status::TURN_RUN_BRAKE
            | crate::native_states::status::SQUAT..=crate::native_states::status::LANDING_LIGHT
            | crate::native_states::status::OTTOTTO | crate::native_states::status::OTTOTTO_WAIT),
        crate::native_states::situation::AIR => crate::native_states::jump_or_fall(status),
        _ => false,
    }
}

#[derive(Clone, Copy)]
pub struct Context {
    pub kind: i32,
    pub situation: i32,
    pub can_special: bool,
    pub stopped: bool,
    pub animation: AnimationSample,
}

/// At most two mapped pulses per attacker motion when the first fails to
/// start counter; unacknowledged mapper deliveries may retry while threatened.
pub struct Counter {
    clock: AnimationClock,
    target: Option<(u32, u64)>,
    began: i64,
    last: Option<(u32, u64)>,
    /// Final mapped counter delivery waits for native startup or a short deadline.
    receipt: bool,
    last_frame: f32,
    last_status: i32,
    last_motion: u64,
    last_object: u32,
    receipt_polls: u8,
    last_poll: Option<i64>,
    attempts: u8,
}
impl Counter {
    pub const fn new() -> Self {
        Self {
            clock: AnimationClock::new(),
            target: None,
            began: 0,
            last: None,
            receipt: false,
            last_frame: 0.0,
            last_status: -1,
            last_motion: 0,
            last_object: 0,
            receipt_polls: 0,
            last_poll: None,
            attempts: 0,
        }
    }
    pub fn active(&self) -> bool {
        self.target.is_some()
    }
    pub fn awaiting_receipt(&self) -> bool {
        self.receipt
    }
    /// Discard a previously consumed attack after it ceases to be observable.
    pub fn clear_stale_attack(&mut self) {
        self.last = None;
        self.last_object = 0;
        self.attempts = 0;
    }
    pub fn cancel(&mut self) {
        self.target = None;
        self.receipt = false;
        self.receipt_polls = 0;
        self.last_poll = None;
    }
    /// Mapper-poll fallback for a frozen native animation during online delivery.
    pub fn poll_receipt(&mut self, tick: i64) {
        if self.receipt && self.last_poll != Some(tick) {
            self.last_poll = Some(tick);
            self.receipt_polls = self.receipt_polls.saturating_add(1);
            if self.receipt_polls >= 24 {
                self.cancel();
            }
        }
    }
    /// A completed special relinquishes ownership before the next mapper poll.
    pub fn finished(&mut self, status: i32, frame: f32) -> bool {
        if self.receipt
            && status == crate::native_states::status::SPECIAL_LW
            && frame.is_finite()
            && frame >= 1.0
        {
            self.cancel();
            return true;
        }
        false
    }
    /// Reset the attack key when the observed attacker starts a new animation.
    pub fn observe_attacker(&mut self, threat: Threat) {
        if self.last_object != threat.id
            || self.last_motion != threat.motion
            || self.last_status != threat.status
            || threat.frame < self.last_frame
        {
            self.last = None;
            self.attempts = 0;
        }
        self.last_object = threat.id;
        self.last_motion = threat.motion;
        self.last_status = threat.status;
        self.last_frame = threat.frame;
    }
    pub fn mapped(&mut self) {
        if let Some(target) = self.target {
            self.last = Some(target);
            self.receipt = true;
            self.attempts = self.attempts.saturating_add(1);
        }
    }
    pub fn sample(
        &mut self,
        allowed: bool,
        config: Option<Config>,
        c: Context,
        threat: Option<Threat>,
    ) -> ControllerOutput {
        let Some(config) = config.filter(|_| allowed && supports(c.kind)) else {
            self.cancel();
            return ControllerOutput::default();
        };
        if c.stopped {
            return ControllerOutput::default();
        }
        let Some(now) = self.clock.observe(c.animation, false) else {
            self.cancel();
            return ControllerOutput::default();
        };
        if self.receipt {
            if self.finished(c.animation.status, c.animation.frame) {
                return ControllerOutput::default();
            }
            if c.animation.status == crate::native_states::status::SPECIAL_LW {
                return ControllerOutput::default();
            }
            if self.receipt_polls >= 2 && now.saturating_sub(self.began) >= 2 && self.attempts < 2 {
                // The mapped pulse did not start the special. Release B before
                // another attempt against the still-observable attack.
                self.target = None;
                self.receipt = false;
                self.receipt_polls = 0;
                return ControllerOutput::default();
            } else {
                if now.saturating_sub(self.began) > 12 {
                    self.cancel();
                }
                return ControllerOutput::default();
            }
        }
        let Some(threat) = threat.filter(|t| t.in_frames.is_finite() && t.in_frames >= 0.0) else {
            self.cancel();
            self.clear_stale_attack();
            return ControllerOutput::default();
        };
        self.observe_attacker(threat);
        let key = (threat.id, threat.motion);
        if self.target.is_some() && self.target != Some(key) {
            self.cancel();
        }
        if self.target.is_none() {
            if !ready(c.animation.status, c.situation)
                || !c.can_special
                || (self.last == Some(key) && self.attempts >= 2)
                || !(config.startup..=config.horizon()).contains(&threat.in_frames)
            {
                return ControllerOutput::default();
            }
            self.target = Some(key);
            self.began = now;
        }
        if now.saturating_sub(self.began) > 12
            || !ready(c.animation.status, c.situation)
            || !c.can_special
        {
            self.cancel();
            return ControllerOutput::default();
        }
        ControllerOutput {
            requested: MAPPER_B,
            owned: SEQUENCE_BUTTONS,
            stick: Some([0.0, -1.0]),
            ..Default::default()
        }
    }
}
impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        Config {
            startup: 5.0,
            margin: 2.0,
            half_width: 5.0,
            height: 16.0,
            half_depth: 5.0,
        }
    }

    #[test]
    fn known_nearby_startup_requests_counter_before_hitbox_exists() {
        let kind = 5;
        let motion = crate::timing::motion_hash("attack_air_f");
        let first = crate::fighter_data::fighter(kind)
            .unwrap()
            .move_for_motion(motion)
            .unwrap()
            .active[0]
            .first as f32;
        let threat = predict_startup(
            config(),
            [0.0; 3],
            [0.0; 2],
            [12.0, 0.0, 0.0],
            [0.0; 2],
            7,
            kind,
            motion,
            crate::native_states::status::ATTACK_AIR,
            first - 6.0,
            1.0,
        )
        .unwrap();
        let mut counter = Counter::new();
        let output = counter.sample(
            true,
            Some(config()),
            Context {
                kind: crate::fighter_kinds::LUCINA,
                situation: crate::native_states::situation::GROUND,
                can_special: true,
                stopped: false,
                animation: AnimationSample {
                    object: 1,
                    status: crate::native_states::status::WAIT,
                    kind: 1,
                    frame: 1.0,
                    auxiliary: 0.0,
                },
            },
            Some(threat),
        );
        assert_eq!(output.requested & MAPPER_B, MAPPER_B);
        assert_eq!(output.stick, Some([0.0, -1.0]));
    }

    #[test]
    fn moving_with_direction_requests_down_special_and_releases_on_receipt() {
        let threat = Threat {
            id: 7,
            motion: 9,
            status: crate::native_states::status::ATTACK,
            frame: 3.0,
            in_frames: 6.0,
        };
        for status in [
            crate::native_states::status::WALK,
            crate::native_states::status::RUN,
            crate::native_states::status::TURN_RUN_BRAKE,
            crate::native_states::status::SQUAT_WAIT,
        ] {
            let mut counter = Counter::new();
            let output = counter.sample(
                true,
                Some(config()),
                Context {
                    kind: crate::fighter_kinds::LUCINA,
                    situation: crate::native_states::situation::GROUND,
                    can_special: crate::native_states::moving_down_special_input(
                        status,
                        crate::native_states::situation::GROUND,
                    ),
                    stopped: false,
                    animation: AnimationSample {
                        object: 1,
                        status,
                        kind: 1,
                        frame: 3.0,
                        auxiliary: 0.0,
                    },
                },
                Some(threat),
            );
            assert_eq!(output.requested & MAPPER_B, MAPPER_B, "status {status}");
            assert_eq!(output.stick, Some([0.0, -1.0]));
            counter.mapped();
            assert!(counter.finished(crate::native_states::status::SPECIAL_LW, 1.0));
        }
    }

    #[test]
    fn selects_actionable_hit_over_earlier_unavoidable_contact() {
        let threat = |id, in_frames| Threat {
            id,
            motion: 9,
            status: crate::native_states::status::ATTACK,
            frame: 3.0,
            in_frames,
        };
        let best = prefer_actionable(config(), None, threat(1, 0.0));
        let best = prefer_actionable(config(), best, threat(2, 6.0));
        assert_eq!(best, Some(threat(2, 6.0)));
        assert_eq!(prefer_actionable(config(), best, threat(3, 8.0)), best);
        assert_eq!(
            prefer_actionable(config(), best, threat(4, 5.0)),
            Some(threat(4, 5.0))
        );
    }

    #[test]
    fn retries_once_after_mapped_input_fails_to_start_counter() {
        let mut counter = Counter::new();
        let threat = Threat {
            id: 7,
            motion: 9,
            status: crate::native_states::status::ATTACK,
            frame: 3.0,
            in_frames: 7.0,
        };
        let mut context = Context {
            kind: crate::fighter_kinds::LUCINA,
            situation: crate::native_states::situation::GROUND,
            can_special: true,
            stopped: false,
            animation: AnimationSample {
                object: 1,
                status: crate::native_states::status::RUN,
                kind: 1,
                frame: 1.0,
                auxiliary: 0.0,
            },
        };
        assert_ne!(
            counter.sample(true, Some(config()), context, Some(threat)).requested & MAPPER_B,
            0
        );
        counter.mapped();
        counter.poll_receipt(1);
        context.animation.frame = 2.0;
        assert_eq!(counter.sample(true, Some(config()), context, Some(threat)).requested, 0);
        counter.poll_receipt(2);
        context.animation.frame = 3.0;
        assert_eq!(counter.sample(true, Some(config()), context, Some(threat)).requested, 0);
        context.animation.frame = 4.0;
        assert_ne!(
            counter.sample(true, Some(config()), context, Some(threat)).requested & MAPPER_B,
            0
        );
        counter.mapped();
        context.animation.frame = 4.0;
        assert_eq!(counter.sample(true, Some(config()), context, Some(threat)).requested, 0);
        context.animation.frame = 20.0;
        assert_eq!(counter.sample(true, Some(config()), context, Some(threat)).requested, 0);
        assert_eq!(counter.sample(true, Some(config()), context, Some(threat)).requested, 0);
    }

    #[test]
    fn never_retries_when_native_counter_has_started() {
        let mut counter = Counter::new();
        let threat = Threat {
            id: 7,
            motion: 9,
            status: crate::native_states::status::ATTACK,
            frame: 3.0,
            in_frames: 7.0,
        };
        let mut context = Context {
            kind: crate::fighter_kinds::LUCINA,
            situation: crate::native_states::situation::GROUND,
            can_special: true,
            stopped: false,
            animation: AnimationSample {
                object: 1,
                status: crate::native_states::status::RUN,
                kind: 1,
                frame: 1.0,
                auxiliary: 0.0,
            },
        };
        counter.sample(true, Some(config()), context, Some(threat));
        counter.mapped();
        context.animation.status = crate::native_states::status::SPECIAL_LW;
        for frame in 1..=4 {
            context.animation.frame = frame as f32;
            assert_eq!(counter.sample(true, Some(config()), context, Some(threat)).requested, 0);
        }
        assert!(!counter.active());
    }

    #[test]
    fn startup_rejects_unknown_far_or_too_late_attacks() {
        let kind = 5;
        let motion = crate::timing::motion_hash("attack_air_f");
        let first = crate::fighter_data::fighter(kind)
            .unwrap()
            .move_for_motion(motion)
            .unwrap()
            .active[0]
            .first as f32;
        let predict = |position, motion, frame| {
            predict_startup(
                config(),
                [0.0; 3],
                [0.0; 2],
                position,
                [0.0; 2],
                7,
                kind,
                motion,
                crate::native_states::status::ATTACK_AIR,
                frame,
                1.0,
            )
        };
        assert!(predict([40.0, 0.0, 0.0], motion, first - 6.0).is_none());
        assert!(predict([12.0, 0.0, 0.0], 0, first - 6.0).is_none());
        assert!(predict([12.0, 0.0, 0.0], motion, first - 2.0).is_none());
    }
}
