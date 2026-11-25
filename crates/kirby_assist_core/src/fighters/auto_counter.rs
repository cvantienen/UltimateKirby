//! Input-only counter policy for explicitly configured fighters.
use crate::{
    fighter_tuning::{self as tuning, Tuning},
    gameplay::ControllerOutput,
    ledge_trap::SEQUENCE_BUTTONS,
    native_input::MAPPER_B,
    timing::{AnimationClock, AnimationSample},
};

/// Register verified down-special counter adapters here; the physical hold admits requests.
pub fn supports(kind: i32) -> bool {
    kind == crate::fighter_kinds::LUCINA
}

#[derive(Clone, Copy)]
pub struct Config {
    pub startup: f32,
    pub input_delay: f32,
    pub margin: f32,
    pub half_width: f32,
    pub height: f32,
    pub half_depth: f32,
}
impl Config {
    pub fn for_fighter(kind: i32, values: Tuning) -> Option<Self> {
        if !supports(kind) {
            return None;
        }
        Some(Self {
            startup: values.get(tuning::COUNTER_STARTUP),
            input_delay: 0.0,
            margin: values.get(tuning::COUNTER_MARGIN),
            half_width: values.get(tuning::COUNTER_HALF_WIDTH),
            height: values.get(tuning::COUNTER_HEIGHT),
            half_depth: values.get(tuning::COUNTER_HALF_DEPTH),
        })
    }
    pub fn activation(self) -> f32 {
        self.startup + self.input_delay
    }
    pub fn horizon(self) -> f32 {
        self.activation() + self.margin
    }
    fn valid(self) -> bool {
        (1.0..=30.0).contains(&self.startup)
            && (0.0..=120.0).contains(&self.input_delay)
            && (0.0..=10.0).contains(&self.margin)
            && self.horizon() <= 40.0
            && [self.half_width, self.height, self.half_depth]
                .iter()
                .all(|n| n.is_finite() && *n > 0.0)
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

/// Best-effort catalog startup estimate when no live hitbox exists yet.
/// Root positions cannot establish the exact reach of a future disjoint.
pub fn predicted_attack(
    config: Config,
    origin: [f32; 3],
    own_velocity: [f32; 2],
    enemy_position: [f32; 3],
    enemy_velocity: [f32; 2],
    kind: i32,
    id: u32,
    motion: u64,
    status: i32,
    frame: f32,
    rate: f32,
) -> Option<Threat> {
    if !origin
        .iter()
        .chain(enemy_position.iter())
        .chain(own_velocity.iter())
        .chain(enemy_velocity.iter())
        .all(|v| v.is_finite())
        || !frame.is_finite()
        || frame < 0.0
        || !rate.is_finite()
        || rate <= 0.0
        || !melee_status(status)
        || !config.valid()
    {
        return None;
    }
    let move_data = crate::fighter_data::fighter(kind)?.move_for_motion(motion)?;
    if !move_data.has_damage_data() {
        return None;
    }
    // Screen during each active window, not at counter activation: an early
    // crossing is still best effort, and later contact may occur in the same hit.
    let in_frames = move_data.active.iter().filter_map(|window| {
        let first = ((f32::from(window.first) - frame) / rate).max(0.0);
        let last = ((f32::from(window.last) - frame) / rate).min(config.horizon());
        root_contact(config, origin, own_velocity, enemy_position, enemy_velocity,
            first, last, 12.0, 6.0)
    }).min_by(|a, b| a.total_cmp(b))?;
    Some(Threat {
        id,
        motion,
        status,
        frame,
        in_frames,
    })
}

/// Root-space startup screening only. Live hitboxes must instead be checked
/// against their actual world-space geometry, including long disjoints.
pub fn nearby_attacker(
    config: Config,
    origin: [f32; 3],
    own_velocity: [f32; 2],
    enemy_position: [f32; 3],
    enemy_velocity: [f32; 2],
) -> bool {
    if !(1.0..=30.0).contains(&config.startup)
        || !config.half_width.is_finite()
        || config.half_width <= 0.0
        || !config.height.is_finite()
        || config.height <= 0.0
        || !config.half_depth.is_finite()
        || config.half_depth <= 0.0
        || !origin
            .iter()
            .chain(enemy_position.iter())
            .chain(own_velocity.iter())
            .chain(enemy_velocity.iter())
            .all(|v| v.is_finite())
    {
        return false;
    }
    config.valid() && root_contact(config, origin, own_velocity, enemy_position,
        enemy_velocity, 0.0, config.horizon(), 18.0, 12.0).is_some()
}

// Root-space screening is only a broad phase, never verified hitbox reach.
fn root_contact(
    config: Config,
    origin: [f32; 3],
    own_velocity: [f32; 2],
    enemy_position: [f32; 3],
    enemy_velocity: [f32; 2],
    first: f32,
    last: f32,
    horizontal_margin: f32,
    vertical_margin: f32,
) -> Option<f32> {
    if first > last || (enemy_position[2] - origin[2]).abs() > config.half_depth + 4.0 {
        return None;
    }
    let mut enter = f64::from(first);
    let mut leave = f64::from(last);
    for (axis, low, high) in [
        (0, -config.half_width - horizontal_margin, config.half_width + horizontal_margin),
        (1, -vertical_margin, config.height + vertical_margin),
    ] {
        let position = f64::from(enemy_position[axis]) - f64::from(origin[axis]);
        let speed = f64::from(enemy_velocity[axis]) - f64::from(own_velocity[axis]);
        if speed == 0.0 {
            if position < f64::from(low) || position > f64::from(high) {
                return None;
            }
        } else {
            let first = (f64::from(low) - position) / speed;
            let last = (f64::from(high) - position) / speed;
            enter = enter.max(first.min(last));
            leave = leave.min(first.max(last));
            if enter > leave {
                return None;
            }
        }
    }
    Some(enter as f32)
}

/// A disjoint hitbox can reach the defender even if the attacking fighter's
/// root is too far away for startup-only prediction.
pub fn attacker_relevant(
    config: Config,
    origin: [f32; 3],
    own_velocity: [f32; 2],
    enemy_position: [f32; 3],
    enemy_velocity: [f32; 2],
    has_active_hitbox: bool,
) -> bool {
    origin
        .iter()
        .chain(own_velocity.iter())
        .chain(enemy_position.iter())
        .chain(enemy_velocity.iter())
        .all(|v| v.is_finite())
        && (has_active_hitbox
            || nearby_attacker(config, origin, own_velocity, enemy_position, enemy_velocity))
}

/// Estimated lifetime within the current catalog window, never the next hit.
/// This bounds a live-geometry forecast; it is not native per-slot expiry.
pub fn active_remaining(kind: i32, motion: u64, frame: f32, rate: f32) -> Option<f32> {
    if !crate::timing::clock_known(motion, frame, rate) {
        return None;
    }
    let data = crate::fighter_data::fighter(kind)?.move_for_motion(motion)?;
    if !data.has_damage_data() {
        return None;
    }
    data.active
        .iter()
        .find(|window| frame >= f32::from(window.first) && frame <= f32::from(window.last))
        .map(|window| (f32::from(window.last) - frame) / rate)
}

/// Forecast observed geometry even when the catalog lacks an active lifetime.
pub fn forecast_horizon(config: Config, remaining: Option<f32>) -> f32 {
    remaining.filter(|n| n.is_finite() && *n >= 0.)
        .unwrap_or(config.horizon()).min(config.horizon())
}

/// Fighter-owned normal and recovery attacks establish a melee source without
/// unverified capability bytes. Specials/articles need separate ownership evidence.
pub fn melee_status(status: i32) -> bool {
    use crate::native_states::status;
    (status::ATTACK..=status::ATTACK_AIR).contains(&status)
        || matches!(status, status::DOWN_STAND_ATTACK | status::CLIFF_ATTACK)
}

/// Unknown active lifetimes prove current contact only, not future contact.
pub fn contact_horizon(remaining: Option<f32>, stopped: bool) -> f32 {
    if stopped {
        0.0
    } else {
        remaining.filter(|n| n.is_finite() && *n >= 0.0).unwrap_or(0.0)
    }
}

/// Check live melee geometry when the counter becomes active, not just while it is starting.
pub fn active_melee_threat(
    config: Config,
    origin: [f32; 3],
    own_velocity: [f32; 2],
    hitbox: crate::autopilot_guard::Hitbox,
) -> bool {
    active_melee_contact(config, origin, own_velocity, hitbox, config.horizon())
        .is_some_and(|contact| contact >= config.activation())
}

/// Earliest estimated contact with the configured body envelope. The current
/// active window bounds translation; future pose changes remain unknown.
pub fn active_melee_contact(
    config: Config,
    origin: [f32; 3],
    own_velocity: [f32; 2],
    hitbox: crate::autopilot_guard::Hitbox,
    remaining: f32,
) -> Option<f32> {
    if !config.valid() || !remaining.is_finite() || remaining < 0.0 {
        return None;
    }
    crate::projectile_geometry::swept_capsule_aabb(
        hitbox,
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
        own_velocity,
        remaining.min(config.horizon()),
    )
}

/// Sweeps each half-frame of the defender's falling path against the translated
/// live capsule. This estimates current motion, not future attack poses or landings.
/// Observed fast-fall speed is never slowed to the ordinary terminal speed.
pub fn incoming_contact(
    config: Config,
    mut origin: [f32; 3],
    mut velocity: [f32; 2],
    mut hitbox: crate::autopilot_guard::Hitbox,
    remaining: f32,
    air: Option<crate::gameplay::AirPhysics>,
) -> Option<f32> {
    let Some(air) = air else {
        return active_melee_contact(config, origin, velocity, hitbox, remaining);
    };
    let air = crate::gameplay::AirPhysics::new(air.gravity, air.terminal_speed)?;
    if !config.valid()
        || !remaining.is_finite()
        || remaining < 0.0
        || !origin.iter().chain(velocity.iter()).all(|v| v.is_finite())
    {
        return None;
    }
    let horizon = remaining.min(config.horizon());
    let terminal = air.terminal_speed.max(-velocity[1]);
    let mut elapsed = 0.0;
    // Config::valid caps the horizon at 40 frames: at most 80 swept segments.
    loop {
        let dt = (horizon - elapsed).min(0.5);
        let accelerating = dt.min(((velocity[1] + terminal) / air.gravity).max(0.0));
        let next_y = (velocity[1] - air.gravity * dt).max(-terminal);
        let displacement_y = velocity[1] * accelerating
            - 0.5 * air.gravity * accelerating * accelerating
            - terminal * (dt - accelerating);
        let segment_velocity = [
            velocity[0],
            if dt > 0.0 { displacement_y / dt } else { velocity[1] },
        ];
        if let Some(contact) = active_melee_contact(config, origin, segment_velocity, hitbox, dt) {
            return Some(elapsed + contact);
        }
        if elapsed + dt >= horizon {
            return None;
        }
        for axis in 0..2 {
            origin[axis] += segment_velocity[axis] * dt;
            hitbox.start[axis] += hitbox.velocity[axis] * dt;
            hitbox.end[axis] += hitbox.velocity[axis] * dt;
        }
        velocity[1] = next_y;
        elapsed += dt;
    }
}

/// Projectiles must have a positively identified nearby owner, independently of
/// their own contact geometry. Melee disjoints do not use this distance gate.
pub fn projectile_owner_near(origin: [f32; 3], owner: [f32; 3], half_depth: f32) -> bool {
    origin.iter().chain(owner.iter()).all(|v| v.is_finite())
        && half_depth.is_finite()
        && half_depth > 0.0
        && (owner[0] - origin[0]).abs() <= 40.0
        && (owner[1] - origin[1]).abs() <= 40.0
        && (owner[2] - origin[2]).abs() <= half_depth + 4.0
}

/// Movement may submit an input before its transition term is enabled;
/// native input handling still decides whether the special starts.
pub fn can_request(status: i32, situation: i32, transition_enabled: bool) -> bool {
    ready(status, situation)
        && (transition_enabled || crate::native_states::moving_down_special_input(status, situation))
}

pub fn ready(status: i32, situation: i32) -> bool {
    matches!(situation, crate::native_states::situation::GROUND | crate::native_states::situation::AIR)
        && !crate::native_states::sequence_interrupted(status)
        && !crate::action_owner::native_grab_active(status)
        && status != crate::native_states::status::SPECIAL_LW
}

#[derive(Clone, Copy)]
pub struct Context {
    pub kind: i32,
    pub situation: i32,
    pub can_special: bool,
    pub stopped: bool,
    pub animation: AnimationSample,
}

/// Retry with a clean release while contact remains projected and native
/// down-special readiness permits it; acknowledge native startup before retrying.
pub struct Counter {
    clock: AnimationClock,
    target: Option<(u32, u64)>,
    began: i64,
    last: Option<(u32, u64)>,
    /// Final mapped counter delivery waits for native startup or a short deadline.
    receipt: bool,
    release_pending: bool,
    last_frame: f32,
    last_status: i32,
    last_motion: u64,
    last_object: u32,
    receipt_polls: u8,
    poll_limit: u8,
    last_poll: Option<i64>,
    attempts: u8,
    history: [Option<(u32, u64, i32, f32, u8)>; 16],
    attacker_poses: [Option<(u32, i32, u64, f32)>; 16],
    pose_cursor: usize,
}
impl Counter {
    pub const fn new() -> Self {
        Self {
            clock: AnimationClock::new(),
            target: None,
            began: 0,
            last: None,
            receipt: false,
            release_pending: false,
            last_frame: 0.0,
            last_status: -1,
            last_motion: 0,
            last_object: 0,
            receipt_polls: 0,
            poll_limit: 24,
            last_poll: None,
            attempts: 0,
            history: [None; 16],
            attacker_poses: [None; 16],
            pose_cursor: 0,
        }
    }
    pub fn active(&self) -> bool {
        self.target.is_some()
    }
    pub fn awaiting_receipt(&self) -> bool {
        self.receipt
    }
    pub fn cancel(&mut self) {
        self.target = None;
        self.receipt = false;
        self.release_pending = false;
        self.receipt_polls = 0;
        self.last_poll = None;
    }
    /// Raw controller polls are a bounded delivery watchdog, not simulation time.
    pub fn poll_receipt(&mut self, tick: i64) {
        if self.active() && self.last_poll != Some(tick) {
            self.last_poll = Some(tick);
            self.receipt_polls = self.receipt_polls.saturating_add(1);
            if self.receipt_polls >= self.poll_limit {
                self.last = self.target;
                self.attempts = 2;
                self.cancel();
            }
        }
    }
    pub fn mapped_release(&mut self) {
        if self.release_pending {
            self.cancel();
        }
    }
    fn release_output() -> ControllerOutput {
        ControllerOutput {
            owned: SEQUENCE_BUTTONS,
            stick: Some([0.0; 2]),
            ..Default::default()
        }
    }
    /// A completed special relinquishes ownership before the next mapper poll.
    pub fn finished(&mut self, status: i32, frame: f32) -> bool {
        if (self.receipt || self.release_pending)
            && status == crate::native_states::status::SPECIAL_LW
            && frame.is_finite()
            && frame >= 1.0
        {
            // A confirmed counter consumes this attack; failed deliveries do not.
            self.attempts = u8::MAX;
            self.cancel();
            return true;
        }
        false
    }
    /// Observe even non-threatening poses so repeated identical attacks rearm.
    /// Missing geometry must not itself erase the budget for a lingering attack.
    pub fn observe_attacker_pose(&mut self, pose: (u32, i32, u64, f32)) {
        if pose.2 == 0 || !pose.3.is_finite() || pose.3 < 0. {
            return;
        }
        let index = self.attacker_poses.iter().position(|old| old.is_some_and(|old| old.0 == pose.0));
        if let Some(old) = index.and_then(|i| self.attacker_poses[i]) {
            if !crate::timing::same_motion_event(old, pose) {
                if self.last_object == pose.0 {
                    self.last = None;
                    self.attempts = 0;
                    self.last_status = pose.1;
                    self.last_motion = pose.2;
                    self.last_frame = pose.3;
                }
                for entry in &mut self.history {
                    if entry.is_some_and(|old| old.0 == pose.0) {
                        *entry = None;
                    }
                }
            }
        }
        let index = index.or_else(|| self.attacker_poses.iter().position(Option::is_none))
            .unwrap_or_else(|| {
                let index = self.pose_cursor;
                self.pose_cursor = (index + 1) % self.attacker_poses.len();
                index
            });
        self.attacker_poses[index] = Some(pose);
    }

    /// Reset the attack key when the observed attacker starts a new animation.
    pub fn observe_attacker(&mut self, threat: Threat) {
        self.observe_attacker_pose((threat.id, threat.status, threat.motion, threat.frame));
        if self.last_object != threat.id {
            let previous = (
                self.last_object,
                self.last_motion,
                self.last_status,
                self.last_frame,
                self.attempts,
            );
            if let Some(slot) = self
                .history
                .iter_mut()
                .find(|entry| entry.is_some_and(|old| old.0 == self.last_object))
            {
                *slot = Some(previous);
            } else if self.last_motion != 0 {
                if let Some(slot) = self.history.iter_mut().find(|entry| entry.is_none()) {
                    *slot = Some(previous);
                }
            }
            let old = self.history.iter().flatten().find(|old| old.0 == threat.id);
            self.attempts = old.map_or_else(
                || {
                    if self.history.iter().all(Option::is_some) {
                        2
                    } else {
                        0
                    }
                },
                |old| {
                    if old.1 == threat.motion && old.2 == threat.status && threat.frame >= old.3 {
                        old.4
                    } else {
                        0
                    }
                },
            );
            self.last = (self.attempts > 0).then_some((threat.id, threat.motion));
        } else if self.last_motion != threat.motion
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
        if !self.receipt && !self.release_pending {
            if let Some(target) = self.target {
                self.last = Some(target);
                self.receipt = true;
                self.receipt_polls = 0;
                self.attempts = self.attempts.saturating_add(1).min(u8::MAX - 1);
            }
        }
    }
    pub fn sample(
        &mut self,
        allowed: bool,
        config: Option<Config>,
        c: Context,
        threat: Option<Threat>,
    ) -> ControllerOutput {
        let Some(config) = config.filter(|config| allowed && supports(c.kind) && config.valid())
        else {
            self.cancel();
            return ControllerOutput::default();
        };
        if c.stopped {
            self.cancel();
            return ControllerOutput::default();
        }
        let Some(now) = self.clock.observe(c.animation, false) else {
            self.cancel();
            return ControllerOutput::default();
        };
        if self.finished(c.animation.status, c.animation.frame) {
            return ControllerOutput::default();
        }
        if self.release_pending {
            return Self::release_output();
        }
        if self.receipt {
            if c.animation.status == crate::native_states::status::SPECIAL_LW {
                return ControllerOutput::default();
            }
            if self.receipt_polls >= 2
                && now.saturating_sub(self.began) >= config.input_delay.ceil() as i64 + 2
            {
                // The mapped pulse did not start the special. Release B before
                // another attempt against the still-observable attack.
                self.receipt = false;
                self.release_pending = true;
                return Self::release_output();
            } else {
                if now.saturating_sub(self.began) > 12 + config.input_delay.ceil() as i64 {
                    self.cancel();
                }
                return ControllerOutput::default();
            }
        }
        let Some(threat) = threat.filter(|t| {
            t.in_frames.is_finite()
                && t.in_frames >= 0.0
                && t.in_frames <= config.horizon()
                && t.frame.is_finite()
                && t.frame >= 0.0
                && t.motion != 0
        }) else {
            self.cancel();
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
                || (self.last == Some(key) && self.attempts == u8::MAX)
                || threat.in_frames > config.horizon()
            {
                return ControllerOutput::default();
            }
            self.target = Some(key);
            self.began = now;
            self.poll_limit = 24 + config.input_delay.ceil() as u8;
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
            input_delay: 0.0,
            margin: 2.0,
            half_width: 5.0,
            height: 16.0,
            half_depth: 5.0,
        }
    }

    #[test]
    fn failed_counter_deliveries_keep_retrying_same_projected_threat() {
        let mut counter = Counter::new();
        let mut c = context();
        for attempt in 0..4 {
            let hit = threat(7);
            assert_eq!(counter.sample(true, Some(config()), c, Some(hit)).requested, MAPPER_B);
            counter.mapped();
            for tick in 0..2 {
                counter.poll_receipt(attempt * 3 + tick);
            }
            for _ in 0..3 {
                c.animation.frame += 1.;
                counter.sample(true, Some(config()), c, Some(hit));
            }
            assert!(counter.release_pending);
            counter.mapped_release();
        }
    }

    #[test]
    fn live_contact_requires_geometry_and_known_lifetime_for_prediction() {
        let hit = crate::autopilot_guard::Hitbox {
            start: [0., 8., 0.], end: [0., 8., 0.], radius: 1., velocity: [0.; 2],
        };
        for status in [crate::native_states::status::ATTACK_S3, crate::native_states::status::ATTACK_AIR] {
            assert!(melee_status(status));
            assert_eq!(incoming_contact(config(), [0.; 3], [0.; 2], hit,
                contact_horizon(None, false), None), Some(0.));
        }
        let passing = crate::autopilot_guard::Hitbox {
            start: [0., 25., 0.], end: [0., 25., 0.], velocity: [3., 0.], ..hit
        };
        assert_eq!(incoming_contact(config(), [0.; 3], [0.; 2], passing,
            contact_horizon(Some(7.), false), None), None);
        let approaching = crate::autopilot_guard::Hitbox {
            start: [10., 8., 0.], end: [10., 8., 0.], velocity: [-1., 0.], ..hit
        };
        assert_eq!(incoming_contact(config(), [0.; 3], [0.; 2], approaching,
            contact_horizon(None, false), None), None);
        assert_eq!(incoming_contact(config(), [0.; 3], [0.; 2], approaching,
            contact_horizon(Some(7.), false), None), Some(4.));
        assert_eq!(incoming_contact(config(), [0.; 3], [0.; 2], approaching,
            contact_horizon(Some(7.), true), None), None);
        assert_eq!(incoming_contact(config(), [0.; 3], [0.; 2], approaching,
            forecast_horizon(config(), None), None), Some(4.));
        assert_eq!(incoming_contact(config(), [0.; 3], [0.; 2], approaching,
            forecast_horizon(config(), Some(3.)), None), None);
    }

    #[test]
    fn falling_forecast_finds_contact_that_constant_velocity_misses() {
        let hit = crate::autopilot_guard::Hitbox {
            start: [0.0, 8.0, 0.0],
            end: [0.0, 8.0, 0.0],
            radius: 1.0,
            velocity: [0.0; 2],
        };
        let air = crate::gameplay::AirPhysics::new(1.0, 5.0);
        assert_eq!(active_melee_contact(config(), [0.0, 20.0, 0.0], [0.0; 2], hit, 7.0), None);
        let contact = incoming_contact(config(), [0.0, 20.0, 0.0], [0.0; 2], hit, 7.0, air).unwrap();
        assert!((4.6..4.8).contains(&contact));
        assert_eq!(incoming_contact(config(), [0.0, 20.0, 0.0], [0.0; 2], hit, 4.0, air), None);
        assert_eq!(incoming_contact(config(), [0.0, 20.0, 20.0], [0.0; 2], hit, 7.0, air), None);
        assert_eq!(incoming_contact(config(), [0.0, 20.0, 0.0], [3.0, 0.0], hit, 7.0, air), None);
    }

    #[test]
    fn fast_fall_sweeps_between_samples_without_reducing_observed_speed() {
        let hit = crate::autopilot_guard::Hitbox {
            start: [0.0; 3], end: [0.0; 3], radius: 0.1, velocity: [0.0; 2],
        };
        let body = Config { height: 0.1, ..config() };
        let air = crate::gameplay::AirPhysics::new(0.1, 2.0);
        let contact = incoming_contact(body, [0.0, 3.0, 0.0], [0.0, -8.0], hit, 1.0, air).unwrap();
        assert!((contact - 2.9 / 8.0).abs() < 0.001);
        // The body crosses this tiny hitbox entirely inside one half-frame.
        assert!(contact < 0.5);
    }

    #[test]
    fn relative_running_and_drift_forecasts_reject_retreat_and_invalid_physics() {
        let hit = crate::autopilot_guard::Hitbox {
            start: [20.0, 8.0, 0.0], end: [20.0, 8.0, 0.0],
            radius: 1.0, velocity: [0.0; 2],
        };
        assert_eq!(incoming_contact(config(), [0.0; 3], [0.0; 2], hit, 7.0, None), None);
        assert_eq!(incoming_contact(config(), [0.0; 3], [2.0, 0.0], hit, 7.0, None), Some(7.0));
        assert_eq!(incoming_contact(config(), [0.0; 3], [-2.0, 0.0], hit, 7.0, None), None);
        assert_eq!(incoming_contact(config(), [0.0; 3], [2.0, 0.0], hit, 7.0,
            Some(crate::gameplay::AirPhysics { gravity: f32::NAN, terminal_speed: 2.0 })), None);
        assert!(projectile_owner_near([0.0; 3], [40.0, 30.0, 0.0], 5.0));
        assert!(!projectile_owner_near([0.0; 3], [40.1, 0.0, 0.0], 5.0));
        assert!(!projectile_owner_near([0.0; 3], [81.0, 0.0, 0.0], 5.0));
    }

    #[test]
    fn imminent_contact_requests_best_effort_counter_on_ground_or_in_air() {
        for status in [
            crate::native_states::status::ATTACK_S3,
            crate::native_states::status::ATTACK_AIR,
        ] {
            let threat = predicted_attack(
                config(),
                [0.0; 3],
                [0.0; 2],
                [12.0, 0.0, 0.0],
                [0.0; 2],
                5,
                7,
                crate::timing::motion_hash("attack_air_f"),
                status,
                12.0,
                1.0,
            )
            .unwrap();
            for situation in [
                crate::native_states::situation::GROUND,
                crate::native_states::situation::AIR,
            ] {
                let output = Counter::new().sample(
                    true,
                    Some(config()),
                    Context {
                        kind: crate::fighter_kinds::LUCINA,
                        situation,
                        can_special: true,
                        stopped: false,
                        animation: AnimationSample {
                            object: 1,
                            status: if situation == crate::native_states::situation::GROUND {
                                crate::native_states::status::WAIT
                            } else {
                                crate::native_states::status::FALL
                            },
                            kind: 1,
                            frame: 1.0,
                            auxiliary: 0.0,
                        },
                    },
                    Some(threat),
                );
                assert_eq!(output.requested, MAPPER_B);
                assert_eq!(output.stick, Some([0.0, -1.0]));
            }
        }
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
            crate::native_states::status::WAIT,
            crate::native_states::status::WALK,
            crate::native_states::status::RUN,
            crate::native_states::status::TURN_RUN_BRAKE,
            crate::native_states::status::JUMP_SQUAT,
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
    fn air_jump_and_fall_request_counter_but_guard_does_not() {
        let threat = Threat {
            id: 7,
            motion: 9,
            status: crate::native_states::status::ATTACK_AIR,
            frame: 3.0,
            in_frames: 6.0,
        };
        for status in [
            crate::native_states::status::JUMP,
            crate::native_states::status::JUMP_AERIAL,
            crate::native_states::status::FALL,
            crate::native_states::status::FALL_AERIAL,
        ] {
            let output = Counter::new().sample(
                true,
                Some(config()),
                Context {
                    kind: crate::fighter_kinds::LUCINA,
                    situation: crate::native_states::situation::AIR,
                    can_special: true,
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
        }
        assert!(ready(
            crate::native_states::status::GUARD_OFF,
            crate::native_states::situation::GROUND
        ));
        assert!(ready(
            crate::native_states::status::GUARD,
            crate::native_states::situation::GROUND
        ));
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
            counter
                .sample(true, Some(config()), context, Some(threat))
                .requested
                & MAPPER_B,
            0
        );
        counter.mapped();
        counter.poll_receipt(1);
        context.animation.frame = 2.0;
        assert_eq!(
            counter
                .sample(true, Some(config()), context, Some(threat))
                .requested,
            0
        );
        counter.poll_receipt(2);
        context.animation.frame = 3.0;
        assert_eq!(
            counter
                .sample(true, Some(config()), context, None)
                .requested,
            0
        );
        counter.mapped_release();
        context.animation.frame = 4.0;
        assert_ne!(
            counter
                .sample(true, Some(config()), context, Some(threat))
                .requested
                & MAPPER_B,
            0
        );
        counter.mapped();
        context.animation.frame = 4.0;
        assert_eq!(
            counter
                .sample(true, Some(config()), context, Some(threat))
                .requested,
            0
        );
        context.animation.frame = 20.0;
        assert_eq!(
            counter
                .sample(true, Some(config()), context, Some(threat))
                .requested,
            0
        );
        assert_eq!(
            counter
                .sample(true, Some(config()), context, Some(threat))
                .requested,
            0
        );
    }

    #[test]
    fn a_temporarily_missing_hitbox_does_not_rearm_the_same_counter_attempt() {
        let mut counter = Counter::new();
        let threat = Threat {
            id: 7,
            motion: 9,
            status: crate::native_states::status::ATTACK,
            frame: 3.0,
            in_frames: 6.0,
        };
        let mut context = Context {
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
        };
        assert_ne!(
            counter
                .sample(true, Some(config()), context, Some(threat))
                .requested
                & MAPPER_B,
            0
        );
        counter.mapped();
        context.animation.frame = 2.0;
        counter.poll_receipt(1);
        counter.poll_receipt(2);
        assert_eq!(
            counter
                .sample(true, Some(config()), context, None)
                .requested,
            0
        );
        context.animation.frame = 3.0;
        assert_eq!(
            counter
                .sample(true, Some(config()), context, None)
                .requested,
            0
        );
        counter.mapped_release();
        context.animation.frame = 4.0;
        assert_ne!(
            counter
                .sample(true, Some(config()), context, Some(threat))
                .requested
                & MAPPER_B,
            0
        );
        counter.mapped();
        context.animation.frame = 5.0;
        assert_eq!(
            counter
                .sample(true, Some(config()), context, None)
                .requested,
            0
        );
        context.animation.frame = 6.0;
        assert_eq!(
            counter
                .sample(true, Some(config()), context, Some(threat))
                .requested,
            0
        );
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
            assert_eq!(
                counter
                    .sample(true, Some(config()), context, None)
                    .requested,
                0
            );
        }
        assert!(!counter.active());
        context.animation.status = crate::native_states::status::WAIT;
        context.animation.frame = 1.0;
        assert_eq!(
            counter
                .sample(true, Some(config()), context, Some(threat))
                .requested,
            0
        );
    }

    #[test]
    fn close_startup_uses_contact_time_not_counter_activation_time() {
        let motion = crate::timing::motion_hash("attack_air_f");
        let predict = |x, vx| predicted_attack(
            config(), [0.0; 3], [0.0; 2], [x, 0.0, 0.0], [vx, 0.0],
            5, 7, motion, crate::native_states::status::ATTACK_AIR, 14.0, 1.0,
        );
        // Overlap at contact must not be discarded because the attacker has
        // passed us by counter frame five.
        assert_eq!(predict(0.0, 6.0).unwrap().in_frames, 2.0);
        assert_eq!(predict(0.0, -6.0).unwrap().in_frames, 2.0);
        assert!(nearby_attacker(config(), [0.0; 3], [0.0; 2], [0.0; 3], [6.0, 0.0]));
        assert!(predict(0.0, 12.0).is_none());
        // An approaching root can enter after the first active frame.
        let later = predict(40.0, -5.0).unwrap().in_frames;
        assert!((later - 4.6).abs() < 0.001);
        // Crossing before the window opens is not a contact.
        assert!(predict(-20.0, -6.0).is_none());
        assert!(!nearby_attacker(config(), [0.0; 3], [0.0; 2],
            [-30.0, 45.0, 0.0], [20.0, -5.0]));
        for invalid in [Config { half_width: f32::NAN, ..config() },
            Config { input_delay: f32::INFINITY, ..config() }] {
            assert!(predicted_attack(invalid, [0.0; 3], [0.0; 2], [0.0; 3],
                [0.0; 2], 5, 7, motion, crate::native_states::status::ATTACK_AIR,
                14.0, 1.0).is_none());
        }
    }

    #[test]
    fn shield_and_tech_poses_rearm_the_next_identical_attack() {
        use crate::native_states::status;
        for pose in [status::GUARD, status::GUARD_OFF, status::PASSIVE, status::PASSIVE_FB] {
            let mut counter = Counter::new();
            let mut c = context();
            let hit = threat(7);
            assert_eq!(counter.sample(true, Some(config()), c, Some(hit)).requested, MAPPER_B);
            counter.mapped();
            assert!(counter.finished(status::SPECIAL_LW, 1.0));
            counter.observe_attacker_pose((hit.id, pose, crate::timing::motion_hash("wait"), 1.0));
            c.animation.frame += 1.0;
            assert_eq!(counter.sample(true, Some(config()), c, Some(hit)).requested, MAPPER_B,
                "pose {pose}");
        }
    }

    #[test]
    fn recovery_attacks_are_melee_but_passive_recovery_is_not() {
        use crate::native_states::status;
        for state in [status::DOWN_STAND_ATTACK, status::CLIFF_ATTACK] {
            assert!(melee_status(state), "attack status {state}");
        }
        for state in [status::GUARD, status::GUARD_OFF, status::DOWN_WAIT,
            status::DOWN_STAND, status::DOWN_STAND_FB, status::PASSIVE,
            status::PASSIVE_FB, status::PASSIVE_WALL, status::PASSIVE_WALL_JUMP] {
            assert!(!melee_status(state), "nonattack status {state}");
            assert!(predicted_attack(config(), [0.0; 3], [0.0; 2], [0.0; 3],
                [0.0; 2], 5, 7, crate::timing::motion_hash("attack_air_f"), state,
                12.0, 1.0).is_none());
        }
    }

    #[test]
    fn prediction_rejects_unknown_and_missed_attacks() {
        let motion = crate::timing::motion_hash("attack_air_f");
        let predict = |position, velocity, motion| {
            predicted_attack(
                config(),
                [0.0; 3],
                [0.0; 2],
                position,
                velocity,
                5,
                7,
                motion,
                crate::native_states::status::ATTACK_AIR,
                12.0,
                1.0,
            )
        };
        assert!(predict([40.0, 0.0, 0.0], [0.0; 2], motion).is_none());
        assert!(predict([12.0, 0.0, 0.0], [12.0, 0.0], motion).is_none());
        assert!(predict([12.0, 0.0, 0.0], [0.0; 2], 0).is_none());
        assert!(predict([40.0, 0.0, 0.0], [-5.0, 0.0], motion).is_some());
        assert!(predicted_attack(
            config(),
            [0.0; 3],
            [0.0; 2],
            [8.0, 0.0, 0.0],
            [0.0; 2],
            5,
            7,
            motion,
            crate::native_states::status::ATTACK_AIR,
            19.0,
            1.0,
        )
        .is_some());
    }

    #[test]
    fn startup_root_estimate_rejects_missed_ground_attack() {
        let motion = crate::timing::motion_hash("attack_air_f");
        let predict = |x, vx| {
            predicted_attack(
                config(),
                [0.0; 3],
                [0.0; 2],
                [x, 0.0, 0.0],
                [vx, 0.0],
                5,
                7,
                motion,
                crate::native_states::status::ATTACK_AIR,
                12.0,
                1.0,
            )
        };
        assert!(predict(12.0, 0.0).is_some());
        assert!(predict(40.0, -5.0).is_some());
        assert!(predict(22.0, 0.0).is_none());
    }

    #[test]
    fn nearby_attacker_rejects_disjoint_remote_attacks() {
        let own = [0.0; 3];
        assert!(!nearby_attacker(
            config(),
            own,
            [0.0; 2],
            [45.0, 0.0, 0.0],
            [0.0; 2]
        ));
        assert!(!nearby_attacker(
            config(),
            own,
            [0.0; 2],
            [0.0, 45.0, 0.0],
            [0.0; 2]
        ));
        assert!(nearby_attacker(
            config(),
            own,
            [0.0; 2],
            [30.0, 0.0, 0.0],
            [-3.0, 0.0]
        ));
        assert!(nearby_attacker(
            config(),
            own,
            [0.0; 2],
            [6.0, 0.0, 0.0],
            [0.0; 2]
        ));
    }

    #[test]
    fn movement_submits_input_without_overriding_action_lockouts() {
        use crate::native_states::{situation, status};
        for state in [
            status::WAIT,
            status::WALK,
            status::DASH,
            status::RUN,
            status::JUMP_SQUAT,
        ] {
            assert!(
                can_request(state, situation::GROUND, false)
                    == crate::native_states::moving_down_special_input(state, situation::GROUND),
                "status {state}"
            );
            assert!(
                can_request(state, situation::GROUND, true),
                "status {state}"
            );
        }
        for state in [status::JUMP, status::JUMP_AERIAL, status::FALL, status::FALL_AERIAL] {
            assert!(!can_request(state, situation::AIR, false));
            assert!(can_request(state, situation::AIR, true));
        }
        assert!(Config::for_fighter(crate::fighter_kinds::LUCINA, Tuning::default()).is_some());
        assert!(Config::for_fighter(crate::fighter_kinds::KIRBY, Tuning::default()).is_none());
        for state in [status::GUARD, status::LANDING, status::ATTACK, status::SPECIAL_LW] {
            assert!(!can_request(state, situation::GROUND, false));
        }
        assert!(can_request(status::GUARD, situation::GROUND, true));
        assert!(can_request(status::ATTACK, situation::GROUND, true));
        assert!(!can_request(status::DAMAGE, situation::GROUND, true));
        assert!(!can_request(status::CAPTURE_PULLED, situation::AIR, true));
    }

    #[test]
    fn active_disjoint_is_not_discarded_by_attacker_root_screen() {
        let origin = [0.0; 3];
        let far_root = [45.0, 0.0, 0.0];
        assert!(!attacker_relevant(
            config(),
            origin,
            [0.0; 2],
            far_root,
            [0.0; 2],
            false
        ));
        assert!(attacker_relevant(
            config(),
            origin,
            [0.0; 2],
            far_root,
            [0.0; 2],
            true
        ));
        assert!(active_melee_threat(
            config(),
            origin,
            [0.0; 2],
            crate::autopilot_guard::Hitbox {
                start: [19.0, 8.0, 0.0],
                end: [19.0, 8.0, 0.0],
                radius: 2.0,
                velocity: [-2.0, 0.0],
            }
        ));
    }

    #[test]
    fn active_melee_requires_hitbox_to_reach_grounded_counter() {
        let hitbox = |start: [f32; 3], velocity: [f32; 2]| crate::autopilot_guard::Hitbox {
            start,
            end: start,
            radius: 2.0,
            velocity,
        };
        assert!(!active_melee_threat(
            config(),
            [0.0; 3],
            [0.0; 2],
            hitbox([6.0, 8.0, 0.0], [0.0; 2]),
        ));
        assert!(!active_melee_threat(
            config(),
            [0.0; 3],
            [0.0; 2],
            hitbox([6.0, 25.0, 0.0], [0.0; 2]),
        ));
        assert!(!active_melee_threat(
            config(),
            [0.0; 3],
            [0.0; 2],
            hitbox([6.0, 25.0, 0.0], [-1.0, 0.0]),
        ));
        assert!(!active_melee_threat(
            config(),
            [0.0; 3],
            [0.0; 2],
            hitbox([20.0, 8.0, 0.0], [-1.0, 0.0]),
        ));
        assert!(!active_melee_threat(
            config(),
            [0.0; 3],
            [0.0; 2],
            hitbox([10.0, 8.0, 0.0], [-1.0, 0.0]),
        ));
        // An active hitbox that crosses the fighter before startup, then leaves,
        // cannot be countered by pressing down-special now.
        assert!(!active_melee_threat(
            config(),
            [0.0; 3],
            [0.0; 2],
            hitbox([6.0, 8.0, 0.0], [3.0, 0.0]),
        ));
        assert!(!active_melee_threat(
            config(),
            [0.0; 3],
            [0.0; 2],
            hitbox([40.0, 8.0, 0.0], [-3.0, 0.0]),
        ));
        assert!(active_melee_threat(
            config(),
            [0.0; 3],
            [0.0; 2],
            hitbox([25.0, 8.0, 0.0], [-3.0, 0.0]),
        ));
        assert!(!active_melee_threat(
            config(),
            [0.0; 3],
            [-3.0, 0.0],
            hitbox([25.0, 8.0, 0.0], [-3.0, 0.0]),
        ));
    }

    #[test]
    fn contact_before_activation_still_requests_best_effort_down_b() {
        let threat = Threat {
            id: 7,
            motion: 9,
            status: crate::native_states::status::ATTACK_AIR,
            frame: 14.0,
            in_frames: 0.0,
        };
        let output = Counter::new().sample(
            true,
            Some(config()),
            Context {
                kind: crate::fighter_kinds::LUCINA,
                situation: crate::native_states::situation::AIR,
                can_special: true,
                stopped: false,
                animation: AnimationSample {
                    object: 1,
                    status: crate::native_states::status::FALL,
                    kind: 1,
                    frame: 1.0,
                    auxiliary: 0.0,
                },
            },
            Some(threat),
        );
        assert_eq!(output.requested, MAPPER_B);
        assert_eq!(output.stick, Some([0.0, -1.0]));
    }

    fn context() -> Context {
        Context {
            kind: crate::fighter_kinds::LUCINA,
            situation: crate::native_states::situation::GROUND,
            can_special: true,
            stopped: false,
            animation: AnimationSample {
                object: 1,
                status: crate::native_states::status::WAIT,
                kind: 1,
                frame: 1.,
                auxiliary: 0.,
            },
        }
    }
    fn threat(id: u32) -> Threat {
        Threat {
            id,
            motion: 9,
            status: crate::native_states::status::ATTACK,
            frame: 3.,
            in_frames: 6.,
        }
    }

    #[test]
    fn repeated_identical_counter_threats_rearm_after_non_threatening_poses() {
        let mut counter = Counter::new();
        let mut c = context();
        let mut t = threat(7);
        counter.sample(true, Some(config()), c, Some(t));
        counter.mapped();
        assert!(counter.finished(crate::native_states::status::SPECIAL_LW, 1.));
        assert_eq!(counter.sample(true, Some(config()), c, Some(t)).requested, 0);
        for cycle in 0..20 {
            counter.observe_attacker_pose((7, crate::native_states::status::WAIT, 1, 10.));
            counter.observe_attacker_pose((7, t.status, t.motion, 0.));
            // The next threatening sample is not necessarily earlier than the
            // preceding attack's threatening sample, especially online.
            t.frame = 3. + cycle as f32;
            c.animation.frame += 1.;
            assert_eq!(counter.sample(true, Some(config()), c, Some(t)).requested, MAPPER_B);
            counter.mapped();
            assert!(counter.finished(crate::native_states::status::SPECIAL_LW, 1.));
        }
    }

    #[test]
    fn missing_geometry_and_target_switch_do_not_rearm_lingering_attacks() {
        let mut counter = Counter::new();
        let c = context();
        let t = threat(7);
        counter.sample(true, Some(config()), c, Some(t));
        counter.mapped();
        counter.finished(crate::native_states::status::SPECIAL_LW, 1.);
        counter.observe_attacker_pose((7, t.status, t.motion, 4.));
        counter.sample(true, Some(config()), c, None);
        assert_eq!(counter.sample(true, Some(config()), c, Some(Threat { frame: 4., ..t })).requested, 0);
        counter.sample(true, Some(config()), c, Some(threat(8)));
        counter.cancel();
        counter.observe_attacker_pose((7, crate::native_states::status::WAIT, 1, 1.));
        assert_eq!(counter.sample(true, Some(config()), c, Some(t)).requested, MAPPER_B);
    }

    #[test]
    fn duplicate_receipts_require_a_mapped_release_before_the_single_retry() {
        let mut counter = Counter::new();
        let mut c = context();
        assert_ne!(
            counter
                .sample(true, Some(config()), c, Some(threat(7)))
                .requested,
            0
        );
        counter.mapped();
        counter.mapped();
        assert_eq!(counter.attempts, 1);
        counter.poll_receipt(1);
        counter.poll_receipt(2);
        c.animation.frame = 2.;
        counter.sample(true, Some(config()), c, Some(threat(7)));
        c.animation.frame = 3.;
        let release = counter.sample(true, Some(config()), c, Some(threat(7)));
        assert_eq!(release.requested, 0);
        assert_eq!(release.owned, SEQUENCE_BUTTONS);
        c.animation.frame = 4.;
        assert_eq!(
            counter
                .sample(true, Some(config()), c, Some(threat(7)))
                .requested,
            0
        );
        counter.mapped_release();
        assert_ne!(
            counter
                .sample(true, Some(config()), c, Some(threat(7)))
                .requested,
            0
        );
        counter.mapped();
        assert_eq!(counter.attempts, 2);
    }

    #[test]
    fn configured_delay_defers_retry_and_late_native_start_cancels_release() {
        let mut counter = Counter::new();
        let mut c = context();
        let mut config = config();
        config.input_delay = 3.;
        let mut threat = threat(7);
        threat.in_frames = 9.;
        counter.sample(true, Some(config), c, Some(threat));
        counter.mapped();
        for tick in 1..=4 {
            counter.poll_receipt(tick);
            c.animation.frame += 1.;
            assert_eq!(counter.sample(true, Some(config), c, Some(threat)).owned, 0);
        }
        counter.poll_receipt(5);
        c.animation.frame += 1.;
        assert_eq!(
            counter.sample(true, Some(config), c, Some(threat)).owned,
            SEQUENCE_BUTTONS
        );
        c.animation.status = crate::native_states::status::SPECIAL_LW;
        c.animation.frame = 1.;
        assert_eq!(counter.sample(true, Some(config), c, None).owned, 0);
        assert!(!counter.active());
        assert_eq!(counter.attempts, u8::MAX);
    }

    #[test]
    fn undelivered_frozen_proposals_expire_and_target_switches_do_not_reset_budget() {
        let mut counter = Counter::new();
        let c = context();
        assert_ne!(
            counter
                .sample(true, Some(config()), c, Some(threat(7)))
                .requested,
            0
        );
        for tick in 0..24 {
            counter.poll_receipt(tick);
            counter.poll_receipt(tick);
        }
        assert!(!counter.active());
        assert_ne!(
            counter
                .sample(true, Some(config()), c, Some(threat(7)))
                .requested,
            0
        );
        assert_ne!(
            counter
                .sample(true, Some(config()), c, Some(threat(8)))
                .requested,
            0
        );
        counter.cancel();
        assert_ne!(
            counter
                .sample(true, Some(config()), c, Some(threat(7)))
                .requested,
            0
        );
        let mut next = threat(7);
        next.frame = 1.;
        assert_ne!(
            counter
                .sample(true, Some(config()), c, Some(next))
                .requested,
            0
        );
    }

    #[test]
    fn missing_invalid_hitstop_and_lost_ownership_cancel_pending_input() {
        for case in 0..5 {
            let mut counter = Counter::new();
            let mut c = context();
            counter.sample(true, Some(config()), c, Some(threat(7)));
            let mut t = threat(7);
            if case == 1 {
                t.frame = f32::NAN;
            }
            if case == 2 {
                c.stopped = true;
            }
            if case == 3 {
                c.can_special = false;
            }
            let out = counter.sample(case != 4, Some(config()), c, (case != 0).then_some(t));
            assert_eq!(out.requested, 0);
            assert!(!counter.active(), "case {case}");
        }
    }

    #[test]
    fn active_window_lifetime_is_rate_scaled_and_never_extends_past_deactivation() {
        let kind = crate::fighter_kinds::MARIO;
        let motion = crate::timing::motion_hash("attack_air_n");
        let data = crate::fighter_data::fighter(kind)
            .unwrap()
            .move_for_motion(motion)
            .unwrap();
        let window = data.active.last().unwrap();
        let end = f32::from(window.last);
        assert_eq!(active_remaining(kind, motion, end - 1., 2.), Some(0.5));
        assert_eq!(active_remaining(kind, motion, end + 1., 1.), None);
        assert_eq!(active_remaining(kind, motion, end, 0.), None);
        assert_eq!(active_remaining(kind, motion, f32::NAN, 1.), None);
    }

    #[test]
    fn contact_lifetime_delay_and_relative_motion_are_not_double_counted() {
        let hit = crate::autopilot_guard::Hitbox {
            start: [19., 8., 0.],
            end: [19., 8., 0.],
            radius: 2.,
            velocity: [-2., 0.],
        };
        let contact = active_melee_contact(config(), [0.; 3], [0.; 2], hit, 7.).unwrap();
        assert!((contact - 6.).abs() < 0.0001);
        assert!(active_melee_contact(config(), [0.; 3], [0.; 2], hit, 5.).is_none());
        assert!(active_melee_contact(config(), [0.; 3], [-2., 0.], hit, 7.).is_none());
        let mut delayed = config();
        delayed.input_delay = 2.;
        assert!(!active_melee_threat(delayed, [0.; 3], [0.; 2], hit));
        delayed.input_delay = f32::NAN;
        assert!(active_melee_contact(delayed, [0.; 3], [0.; 2], hit, 7.).is_none());
    }
}
