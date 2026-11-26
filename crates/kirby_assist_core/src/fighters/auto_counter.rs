//! Input-only counter policy for explicitly configured fighters.
use crate::{
    fighter_tuning::{self as tuning, Tuning},
    gameplay::ControllerOutput,
    ledge_trap::SEQUENCE_BUTTONS,
    native_input::MAPPER_B,
    timing::{AnimationClock, AnimationSample},
};

/// Reports the currently verified down-special counter adapter: Lucina only.
///
/// This is narrower than universal held shield/air-dodge defense. Adding a
/// fighter requires verified native readiness/startup/confirmation behavior and
/// focused regression cases, not merely adding its name to this predicate.
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
        || !(melee_status(status)
            || (crate::native_states::status::SPECIAL_N..=crate::native_states::status::SPECIAL_LW).contains(&status))
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
        let last = ((f32::from(window.last) + 1.0 - frame) / rate).min(config.horizon());
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
        .find(|window| frame >= f32::from(window.first) && frame < f32::from(window.last) + 1.0)
        .map(|window| (f32::from(window.last) + 1.0 - frame) / rate)
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
        && (transition_enabled || crate::native_states::moving_down_special_input(status, situation)
            || situation == crate::native_states::situation::AIR
                && matches!(status, crate::native_states::status::JUMP
                    | crate::native_states::status::JUMP_AERIAL
                    | crate::native_states::status::FALL
                    | crate::native_states::status::FALL_AERIAL))
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

#[derive(Clone, Copy)]
struct AttackHistory {
    id: u32,
    motion: u64,
    status: i32,
    frame: f32,
    attempts: u8,
}

/// Retry with a clean release while contact remains projected and native
/// down-special readiness permits it; acknowledge native startup before retrying.
pub struct Counter {
    clock: AnimationClock,
    target: Option<(u32, u64)>,
    began: i64,
    observed_tick: i64,
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
    history: [Option<AttackHistory>; 16],
    attacker_poses: [Option<(u32, i32, u64, f32)>; 16],
    pose_cursor: usize,
}
impl Counter {
    pub const fn new() -> Self {
        Self {
            clock: AnimationClock::new(),
            target: None,
            began: 0,
            observed_tick: 0,
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
            && frame >= 0.0
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
        let (id, status, motion, frame) = pose;
        if motion == 0 || !frame.is_finite() || frame < 0. {
            return;
        }
        let index = self
            .attacker_poses
            .iter()
            .position(|old| old.is_some_and(|(old_id, _, _, _)| old_id == id));
        if let Some(old) = index.and_then(|i| self.attacker_poses[i]) {
            if !crate::timing::same_motion_event(old, pose) {
                if self.last_object == id {
                    self.last = None;
                    self.attempts = 0;
                    self.last_status = status;
                    self.last_motion = motion;
                    self.last_frame = frame;
                }
                for entry in &mut self.history {
                    if entry.is_some_and(|old| old.id == id) {
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
            let previous = AttackHistory {
                id: self.last_object,
                motion: self.last_motion,
                status: self.last_status,
                frame: self.last_frame,
                attempts: self.attempts,
            };
            if let Some(slot) = self
                .history
                .iter_mut()
                .find(|entry| entry.is_some_and(|old| old.id == self.last_object))
            {
                *slot = Some(previous);
            } else if self.last_motion != 0 {
                if let Some(slot) = self.history.iter_mut().find(|entry| entry.is_none()) {
                    *slot = Some(previous);
                }
            }
            let old = self.history.iter().flatten().find(|old| old.id == threat.id);
            self.attempts = old.map_or_else(
                || {
                    if self.history.iter().all(Option::is_some) {
                        2
                    } else {
                        0
                    }
                },
                |old| {
                    if old.motion == threat.motion
                        && old.status == threat.status
                        && threat.frame >= old.frame
                    {
                        old.attempts
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
                self.began = self.observed_tick;
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
        if self.finished(c.animation.status, c.animation.frame) {
            return ControllerOutput::default();
        }
        if !ready(c.animation.status, c.situation)
            || crate::native_states::fighter_sequence_interrupted(c.kind, c.animation.status)
            || c.stopped
        {
            self.cancel();
            return ControllerOutput::default();
        }
        let Some(now) = self.clock.observe(c.animation, false) else {
            self.cancel();
            return ControllerOutput::default();
        };
        self.observed_tick = now;
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
#[path = "../../tests/unit/fighter_auto_counter.rs"]
mod tests;
