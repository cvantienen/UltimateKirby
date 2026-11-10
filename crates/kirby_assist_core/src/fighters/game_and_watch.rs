//! Game & Watch's bounded, input-only projectile defense.
//! No native writes or queued attacks.
use crate::{
    autopilot_guard::Hitbox,
    fighter_tuning::{self as tuning, Tuning},
    gameplay::ControllerOutput,
    ledge_trap::SEQUENCE_BUTTONS,
    native_input::MAPPER_B,
    timing::{AnimationClock, AnimationSample},
};

const REFLECT_CONTACT_SAFETY_MARGIN_FRAMES: f32 = 2.0;
const MAX_PROJECTILE_RADIUS: f32 = 100.0;
const MAX_REFLECTOR_HORIZON_FRAMES: f32 = 40.0;
const MAX_RECEIPT_POLL_COUNT: u8 = 24;
const MAX_UNCONFIRMED_REFLECT_FRAMES: f32 = 24.0;
const MIN_REFLECT_RETRY_INTERVAL_FRAMES: f32 = 2.0;
const MAX_TURNING_FRAMES: f32 = 2.0;
const MAX_UNSENT_REFLECT_FRAMES: f32 = 4.0;
const TURNING_REFLECT_STARTUP_EXTRA_FRAMES: f32 = 1.0;
/// Recently attempted shots remembered so a missed/consumed shot is not retried.
const ATTEMPT_MEMORY: usize = 32;

pub const GAMEWATCH: i32 = crate::fighter_kinds::MR_GAME_AND_WATCH;
/// Adding a fighter requires a verified native move/resource adapter too.
pub fn supports(kind: i32) -> bool {
    kind == GAMEWATCH
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    pub startup: f32,
    pub margin: f32,
    pub max_hold: f32,
    pub half_width: f32,
    pub height: f32,
    pub half_depth: f32,
}
impl Config {
    pub fn for_fighter(kind: i32, values: Tuning) -> Option<Self> {
        // Explicit saved configuration, never a global/cross-fighter fallback.
        if !supports(kind) || values.0[tuning::REFLECT_DEFAULT].is_none() {
            return None;
        }
        Some(Self {
            startup: values
                .get(tuning::REFLECT_STARTUP)
                .max(tuning::MIN_GAMEWATCH_REFLECT_STARTUP_FRAMES),
            margin: values.get(tuning::REFLECT_MARGIN),
            max_hold: values.get(tuning::REFLECT_MAX_HOLD),
            half_width: values.get(tuning::REFLECT_HALF_WIDTH),
            height: values.get(tuning::REFLECT_HEIGHT),
            half_depth: values.get(tuning::REFLECT_HALF_DEPTH),
        })
    }
    pub fn horizon(self) -> f32 {
        self.startup + self.margin + REFLECT_CONTACT_SAFETY_MARGIN_FRAMES
    }
}

pub fn movable(status: i32, situation: i32) -> bool {
    match situation {
        crate::native_states::situation::GROUND => {
            matches!(status, crate::native_states::status::WAIT..=crate::native_states::status::TURN_RUN_BRAKE | crate::native_states::status::SQUAT..=crate::native_states::status::LANDING_LIGHT | crate::native_states::status::OTTOTTO | crate::native_states::status::OTTOTTO_WAIT)
        }
        // Ordinary jump/fall only.
        crate::native_states::situation::AIR => crate::native_states::jump_or_fall(status),
        _ => false,
    }
}
pub fn bucket(status: i32) -> bool {
    matches!(
        status,
        crate::native_states::status::SPECIAL_LW
            | crate::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_WAIT_START
            | crate::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_WAIT
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Threat {
    pub id: u32,
    pub generation: u32,
    pub in_frames: f32,
    pub direction: f32,
}
/// A new native ATTACK on a reused object is a fresh opportunity, not the
/// previous shot. No wall-clock cooldown or permanent object-ID blacklist.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectileKey {
    pub id: u32,
    pub generation: u32,
}
impl Threat {
    fn key(self) -> ProjectileKey {
        ProjectileKey {
            id: self.id,
            generation: self.generation,
        }
    }
}

/// Swept capsule AABB against a configurable body envelope. Continuous slab
/// intersections cannot skip a fast straight projectile between frame samples.
/// Bounds are estimates, not a claim about the native reflector's hitbox.
pub fn predict(
    config: Config,
    origin: [f32; 3],
    velocity: [f32; 2],
    facing: f32,
    id: u32,
    hit: Hitbox,
    reflectable: bool,
    absorbable: bool,
) -> Option<Threat> {
    if !(reflectable || absorbable)
        || !origin
            .iter()
            .chain(velocity.iter())
            .chain(hit.start.iter())
            .chain(hit.end.iter())
            .chain(hit.velocity.iter())
            .chain([
                &facing,
                &hit.radius,
                &config.startup,
                &config.margin,
                &config.max_hold,
                &config.half_width,
                &config.height,
                &config.half_depth,
            ])
            .all(|v| v.is_finite())
        || hit.radius <= 0.
        || hit.radius > MAX_PROJECTILE_RADIUS
        || facing.abs() < 0.5
        || config.startup < tuning::MIN_GAMEWATCH_REFLECT_STARTUP_FRAMES
        || config.margin < 0.
        || config.horizon() > MAX_REFLECTOR_HORIZON_FRAMES
        || config.half_width <= 0.
        || config.height <= 0.
        || config.half_depth <= 0.
    {
        return None;
    }
    let body_lo = [
        origin[0] - config.half_width,
        origin[1],
        origin[2] - config.half_depth,
    ];
    let body_hi = [
        origin[0] + config.half_width,
        origin[1] + config.height,
        origin[2] + config.half_depth,
    ];
    let enter = crate::projectile_geometry::swept_capsule_aabb(
        hit,
        body_lo,
        body_hi,
        velocity,
        config.horizon(),
    )?;
    let center = (hit.start[0] + hit.end[0]) * 0.5 - origin[0];
    Some(Threat {
        id,
        generation: 0, // The native source adapter supplies the attack generation.
        in_frames: enter,
        direction: if center.abs() < 0.1 {
            facing.signum()
        } else {
            center.signum()
        },
    })
}

#[derive(Clone, Copy)]
pub struct Context {
    pub kind: i32,
    pub situation: i32,
    pub facing: f32,
    pub can_special: bool,
    pub gauge: f32,
    pub stopped: bool,
    pub animation: AnimationSample,
}
/// One current input owner. Delivered/expired attempts remember their native
/// attack generation. Losing arbitration does not consume an unseen attack;
/// a fresh shot on the same object can rearm on the first legal observation.
pub struct Reflector {
    clock: AnimationClock,
    target: Option<ProjectileKey>,
    began: i64,
    turning: bool,
    sent: bool,
    confirmed: bool,
    delivery: crate::move_input::DeliveryBudget,
    receipt_polls: u8,
    last_poll: Option<i64>,
    attempted: [Option<ProjectileKey>; ATTEMPT_MEMORY],
    next: usize,
}
impl Default for Reflector {
    fn default() -> Self {
        Self::new()
    }
}
impl Reflector {
    pub const fn new() -> Self {
        Self {
            clock: AnimationClock::new(),
            target: None,
            began: 0,
            turning: false,
            sent: false,
            confirmed: false,
            delivery: crate::move_input::DeliveryBudget::new(),
            receipt_polls: 0,
            last_poll: None,
            attempted: [None; ATTEMPT_MEMORY],
            next: 0,
        }
    }
    pub fn active(&self) -> bool {
        self.target.is_some()
    }
    pub fn target(&self) -> Option<u32> {
        self.target.map(|key| key.id)
    }
    fn remember_target(&mut self) {
        if let Some(id) = self
            .target
            .filter(|id| !self.attempted.contains(&Some(*id)))
        {
            self.attempted[self.next % ATTEMPT_MEMORY] = Some(id);
            self.next = self.next.wrapping_add(1);
        }
    }
    pub fn cancel(&mut self) {
        self.target = None;
        self.turning = false;
        self.sent = false;
        self.confirmed = false;
        self.delivery = crate::move_input::DeliveryBudget::new();
        self.receipt_polls = 0;
        self.last_poll = None;
    }
    fn abort(&mut self) -> ControllerOutput {
        self.cancel();
        ControllerOutput::default()
    }
    /// Abort, remembering the target when a receipt poll already counted against it.
    fn abort_polled(&mut self) -> ControllerOutput {
        if self.receipt_polls > 0 {
            self.remember_target();
        }
        self.abort()
    }
    /// Last-resort receipt fence when BOTH native animation and projectile
    /// clocks are unavailable/frozen. This never advances a move or adds input.
    pub fn poll_receipt(&mut self, tick: i64) {
        if self.target.is_some() && !self.confirmed && self.last_poll != Some(tick) {
            self.last_poll = Some(tick);
            self.receipt_polls = self.receipt_polls.saturating_add(1);
            if self.receipt_polls >= MAX_RECEIPT_POLL_COUNT {
                self.remember_target();
                self.cancel();
            }
        }
    }
    pub fn mapped(&mut self, tick: i64) {
        if self.target.is_some() {
            self.delivery.mapped(tick);
            if !self.sent {
                self.remember_target();
                self.sent = true;
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
        let Some(config) = config.filter(|_| {
            allowed
                && supports(c.kind)
                && c.gauge.is_finite()
                && (0.0..3.0).contains(&c.gauge)
                && super::valid_facing(c.facing)
        }) else {
            return self.abort();
        };
        if c.stopped {
            return Default::default();
        }
        let Some(now) = self.clock.observe(c.animation, false) else {
            return self.abort();
        };
        let status = c.animation.status;
        let elapsed = now.saturating_sub(self.began) as f32;
        if self.sent && !self.confirmed {
            if bucket(status) {
                self.confirmed = true;
            } else if !movable(status, c.situation)
                || elapsed >= MAX_UNCONFIRMED_REFLECT_FRAMES
                || self.delivery.exhausted()
            {
                return self.abort();
            } else if elapsed >= MIN_REFLECT_RETRY_INTERVAL_FRAMES || threat.is_none() {
                // An online-delayed packet is a receipt reservation, not a
                // neutral-stick hold or a stream of repeated special inputs.
                // Physical movement stays free while other auto punishes yield.
                return Default::default();
            }
        }
        let Some(threat) = threat.filter(|t| {
            t.in_frames.is_finite()
                && t.in_frames >= 0.
                && t.direction.is_finite()
                && t.direction.abs() == 1.
        }) else {
            return self.abort();
        };
        if self.target.is_some() && self.target() != Some(threat.id) {
            return self.abort();
        }
        let facing = threat.direction * c.facing > 0.;
        if self.target.is_none() {
            if !self.admissible(config, c, threat, facing) {
                return Default::default();
            }
            self.target = Some(threat.key());
            self.began = now;
            self.turning = !facing;
        }
        let elapsed = now.saturating_sub(self.began) as f32;
        if elapsed > config.max_hold
            || matches!(
                status,
                crate::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_CATCH
                    ..=crate::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_END
            )
        {
            return self.abort();
        }
        if self.turning {
            if !movable(status, c.situation)
                || c.situation != crate::native_states::situation::GROUND
                || elapsed >= MAX_TURNING_FRAMES
            {
                return self.abort_polled();
            }
            if !facing {
                return crate::move_input::output([threat.direction * 0.6, 0.], 0);
            }
            self.turning = false;
        }
        if bucket(status) && self.sent {
            self.confirmed = true;
        }
        if !facing
            || !self.confirmed
                && (!movable(status, c.situation)
                    || !c.can_special
                    || !self.sent && elapsed >= MAX_UNSENT_REFLECT_FRAMES
                    || self.delivery.exhausted())
            || self.confirmed && !bucket(status)
        {
            return self.abort_polled();
        }
        ControllerOutput {
            requested: MAPPER_B,
            owned: SEQUENCE_BUTTONS,
            stick: Some([0., -1.]),
            ..Default::default()
        }
    }
    /// Whether a new reflect started now covers this shot: Game & Watch can
    /// act, the shot is a fresh attempt, and it arrives inside the startup
    /// window (a turn-around costs extra startup and keeps a larger margin).
    fn admissible(&self, config: Config, c: Context, threat: Threat, facing: bool) -> bool {
        let (extra_startup, extra_margin) = if facing {
            (0., 0.)
        } else {
            (
                TURNING_REFLECT_STARTUP_EXTRA_FRAMES,
                REFLECT_CONTACT_SAFETY_MARGIN_FRAMES,
            )
        };
        movable(c.animation.status, c.situation)
            && c.can_special
            && !self.attempted.contains(&Some(threat.key()))
            && (facing || c.situation == crate::native_states::situation::GROUND)
            && (config.startup + extra_startup..=config.startup + config.margin + extra_margin)
                .contains(&threat.in_frames)
    }
}
