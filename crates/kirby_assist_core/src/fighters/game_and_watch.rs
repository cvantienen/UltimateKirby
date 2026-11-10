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

/// Defines the `REFLECT_CONTACT_SAFETY_MARGIN_FRAMES` constant used by this module.
///
/// Provides the REFLECT CONTACT SAFETY MARGIN FRAMES constant value.
const REFLECT_CONTACT_SAFETY_MARGIN_FRAMES: f32 = 2.0;
/// Sets the upper bound for projectile radius.
///
/// Holds the MAX PROJECTILE RADIUS value.
const MAX_PROJECTILE_RADIUS: f32 = 100.0;
/// Sets the upper bound for reflector horizon frames.
///
/// Holds the MAX REFLECTOR HORIZON FRAMES value.
const MAX_REFLECTOR_HORIZON_FRAMES: f32 = 40.0;
/// Sets the upper bound for receipt poll count.
///
/// Holds the MAX RECEIPT POLL COUNT value.
const MAX_RECEIPT_POLL_COUNT: u8 = 24;
/// Sets the upper bound for unconfirmed reflect frames.
///
/// Holds the MAX UNCONFIRMED REFLECT FRAMES value.
const MAX_UNCONFIRMED_REFLECT_FRAMES: f32 = 24.0;
/// Sets the lower bound for reflect retry interval frames.
///
/// Holds the MIN REFLECT RETRY INTERVAL FRAMES value.
const MIN_REFLECT_RETRY_INTERVAL_FRAMES: f32 = 2.0;
/// Sets the upper bound for turning frames.
///
/// Holds the MAX TURNING FRAMES value.
const MAX_TURNING_FRAMES: f32 = 2.0;
/// Sets the upper bound for unsent reflect frames.
///
/// Holds the MAX UNSENT REFLECT FRAMES value.
const MAX_UNSENT_REFLECT_FRAMES: f32 = 4.0;
/// Defines the `TURNING_REFLECT_STARTUP_EXTRA_FRAMES` constant used by this module.
///
/// Provides the TURNING REFLECT STARTUP EXTRA FRAMES constant value.
const TURNING_REFLECT_STARTUP_EXTRA_FRAMES: f32 = 1.0;
/// Recently attempted shots remembered so a missed/consumed shot is not retried.
///
///  Recently attempted shots remembered so a missed/consumed shot is not retried.
const ATTEMPT_MEMORY: usize = 32;

/// Defines the `GAMEWATCH` constant used by this module.
///
/// Provides the GAMEWATCH constant value.
pub const GAMEWATCH: i32 = crate::fighter_kinds::MR_GAME_AND_WATCH;
/// Adding a fighter requires a verified native move/resource adapter too.
///
///  Adding a fighter requires a verified native move/resource adapter too.
/// ### Arguments
/// - `kind` (`i32`): Represents the kind or category being checked; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn supports(kind: i32) -> bool {
    kind == GAMEWATCH
}

/// Defines the `Config` data structure.
///
/// Holds the data for Config.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    /// Stores the `startup` value in `Config` as `f32`.
    ///
    ///
    ///  Holds the startup value used by Config.
    pub startup: f32,
    /// Stores the `margin` value in `Config` as `f32`.
    ///
    ///
    ///  Holds the margin value used by Config.
    pub margin: f32,
    /// Stores the `max_hold` value in `Config` as `f32`.
    ///
    ///
    ///  Holds the max hold value used by Config.
    pub max_hold: f32,
    /// Stores the `half_width` value in `Config` as `f32`.
    ///
    ///
    ///  Holds the half width value used by Config.
    pub half_width: f32,
    /// Stores the `height` value in `Config` as `f32`.
    ///
    ///
    ///  Holds the height value used by Config.
    pub height: f32,
    /// Stores the `half_depth` value in `Config` as `f32`.
    ///
    ///
    ///  Holds the half depth value used by Config.
    pub half_depth: f32,
}
impl Config {
    /// Returns the profile data for the requested fighter.
    ///
    ///
    ///  Gets the data for this fighter.
    /// ### Arguments
    /// - `kind` (`i32`): Represents the kind or category being checked; the function uses it as input to the operation described above.
    /// - `values` (`Tuning`): Represents the values; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
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
    /// Returns the prediction horizon used by the policy.
    ///
    ///
    ///  Gets how far ahead the policy looks.
    /// ### Arguments
    /// - `self` (`self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `f32` value.
    pub fn horizon(self) -> f32 {
        self.startup + self.margin + REFLECT_CONTACT_SAFETY_MARGIN_FRAMES
    }
}

/// Checks whether the fighter can move in the observed state.
///
/// Checks whether the fighter can move.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// - `situation` (`i32`): Represents the situation; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
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
/// Classifies the value into its bounded bucket.
///
/// Groups the value into a range.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn bucket(status: i32) -> bool {
    matches!(
        status,
        crate::native_states::status::SPECIAL_LW
            | crate::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_WAIT_START
            | crate::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_WAIT
    )
}

/// Defines the `Threat` data structure.
///
/// Holds the data for Threat.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Threat {
    /// Stores the `id` value in `Threat` as `u32`.
    ///
    ///
    ///  Holds the id value used by Threat.
    pub id: u32,
    /// Stores the `generation` value in `Threat` as `u32`.
    ///
    ///
    ///  Holds the generation value used by Threat.
    pub generation: u32,
    /// Stores the `in_frames` value in `Threat` as `f32`.
    ///
    ///
    ///  Sets the in frames timing for Threat.
    pub in_frames: f32,
    /// Stores the `direction` value in `Threat` as `f32`.
    ///
    ///
    ///  Holds the direction value used by Threat.
    pub direction: f32,
}
/// A new native ATTACK on a reused object is a fresh opportunity, not the
/// previous shot. No wall-clock cooldown or permanent object-ID blacklist.
///
///  A new native ATTACK on a reused object is a fresh opportunity, not the previous shot. No wall-clock cooldown or permanent object-ID blacklist.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectileKey {
    /// Stores the `id` value in `ProjectileKey` as `u32`.
    ///
    ///
    ///  Holds the id value used by ProjectileKey.
    pub id: u32,
    /// Stores the `generation` value in `ProjectileKey` as `u32`.
    ///
    ///
    ///  Holds the generation value used by ProjectileKey.
    pub generation: u32,
}
impl Threat {
    /// Builds the lookup key for the observed state.
    ///
    ///
    ///  Creates the key used to find this state.
    /// ### Arguments
    /// - `self` (`self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ProjectileKey` value.
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
///
///  Swept capsule AABB against a configurable body envelope. Continuous slab intersections cannot skip a fast straight projectile between frame samples. Bounds are estimates, not a claim about the native reflector's hitbox.
/// ### Arguments
/// - `config` (`Config`): Represents the configuration being read or changed; the function uses it as input to the operation described above.
/// - `origin` (`[f32; 3]`): Represents the origin; the function uses it as input to the operation described above.
/// - `velocity` (`[f32; 2]`): Represents an object velocity or movement speed; the function uses it as input to the operation described above.
/// - `facing` (`f32`): Represents the facing; the function uses it as input to the operation described above.
/// - `id` (`u32`): Represents the identifier of an object or entry; the function uses it as input to the operation described above.
/// - `hit` (`Hitbox`): Represents the hit; the function uses it as input to the operation described above.
/// - `reflectable` (`bool`): Represents the reflectable; the function uses it as input to the operation described above.
/// - `absorbable` (`bool`): Represents the absorbable; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Some(value)` when a value is available; otherwise, returns `None`.
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

/// Defines the `Context` data structure.
///
/// Holds the data for Context.
#[derive(Clone, Copy)]
pub struct Context {
    /// Stores the `kind` value in `Context` as `i32`.
    ///
    ///
    ///  Holds the kind value used by Context.
    pub kind: i32,
    /// Stores the `situation` value in `Context` as `i32`.
    ///
    ///
    ///  Holds the situation value used by Context.
    pub situation: i32,
    /// Stores the `facing` value in `Context` as `f32`.
    ///
    ///
    ///  Holds the facing value used by Context.
    pub facing: f32,
    /// Stores the `can_special` value in `Context` as `bool`.
    ///
    ///
    ///  Holds the can special value used by Context.
    pub can_special: bool,
    /// Stores the `gauge` value in `Context` as `f32`.
    ///
    ///
    ///  Holds the gauge value used by Context.
    pub gauge: f32,
    /// Stores the `stopped` value in `Context` as `bool`.
    ///
    ///
    ///  Holds the stopped value used by Context.
    pub stopped: bool,
    /// Stores the `animation` value in `Context` as `AnimationSample`.
    ///
    ///
    ///  Holds the animation value used by Context.
    pub animation: AnimationSample,
}
/// One current input owner. Delivered/expired attempts remember their native
/// attack generation. Losing arbitration does not consume an unseen attack;
/// a fresh shot on the same object can rearm on the first legal observation.
///
///  One current input owner. Delivered/expired attempts remember their native attack generation. Losing arbitration does not consume an unseen attack; a fresh shot on the same object can rearm on the first legal observation.
pub struct Reflector {
    /// Stores the `clock` value in `Reflector` as `AnimationClock`.
    ///
    ///
    ///  Holds the clock value used by Reflector.
    clock: AnimationClock,
    /// Stores the `target` value in `Reflector` as `Option<ProjectileKey>`.
    ///
    ///
    ///  Optionally stores the target for Reflector; it may be absent.
    target: Option<ProjectileKey>,
    /// Stores the `began` value in `Reflector` as `i64`.
    ///
    ///
    ///  Holds the began value used by Reflector.
    began: i64,
    /// Stores the `turning` value in `Reflector` as `bool`.
    ///
    ///
    ///  Holds the turning value used by Reflector.
    turning: bool,
    /// Stores the `sent` value in `Reflector` as `bool`.
    ///
    ///
    ///  Holds the sent value used by Reflector.
    sent: bool,
    /// Stores the `confirmed` value in `Reflector` as `bool`.
    ///
    ///
    ///  Holds the confirmed value used by Reflector.
    confirmed: bool,
    /// Stores the `delivery` value in `Reflector` as `crate::move_input::DeliveryBudget`.
    ///
    ///
    ///  Holds the delivery value used by Reflector.
    delivery: crate::move_input::DeliveryBudget,
    /// Stores the `receipt_polls` value in `Reflector` as `u8`.
    ///
    ///
    ///  Holds the receipt polls value used by Reflector.
    receipt_polls: u8,
    /// Stores the `last_poll` value in `Reflector` as `Option<i64>`.
    ///
    ///
    ///  Optionally stores the last poll for Reflector; it may be absent.
    last_poll: Option<i64>,
    /// Stores the `attempted` value in `Reflector` as `[Option<ProjectileKey>; ATTEMPT_MEMORY]`.
    ///
    ///
    ///  Stores the attempted collection for Reflector.
    attempted: [Option<ProjectileKey>; ATTEMPT_MEMORY],
    /// Stores the `next` value in `Reflector` as `usize`.
    ///
    ///
    ///  Holds the next value used by Reflector.
    next: usize,
}
impl Default for Reflector {
    /// Returns the default value for this type.
    ///
    ///
    ///  Returns the default value for this type.
    /// ### Returns
    /// Returns a `Self` value representing the result of the operation.
    fn default() -> Self {
        Self::new()
    }
}
impl Reflector {
    /// Constructs a new value with the type’s initial state.
    ///
    ///
    ///  Creates a new value.
    /// ### Returns
    /// Returns a `Self` value representing the result of the operation.
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
    /// Checks whether the feature is active.
    ///
    ///
    ///  Checks whether the feature is on.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn active(&self) -> bool {
        self.target.is_some()
    }
    /// Returns the selected target observation.
    ///
    ///
    ///  Gets the chosen target.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
    pub fn target(&self) -> Option<u32> {
        self.target.map(|key| key.id)
    }
    /// Stores the latest eligible target observation.
    ///
    ///
    ///  Remembers the target that can be used next.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    fn remember_target(&mut self) {
        if let Some(id) = self
            .target
            .filter(|id| !self.attempted.contains(&Some(*id)))
        {
            self.attempted[self.next % ATTEMPT_MEMORY] = Some(id);
            self.next = self.next.wrapping_add(1);
        }
    }
    /// Cancels the pending operation and clears its state.
    ///
    ///
    ///  Stops the operation that was waiting.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    pub fn cancel(&mut self) {
        self.target = None;
        self.turning = false;
        self.sent = false;
        self.confirmed = false;
        self.delivery = crate::move_input::DeliveryBudget::new();
        self.receipt_polls = 0;
        self.last_poll = None;
    }
    /// Aborts the active sequence and resets its progress.
    ///
    ///
    ///  Stops the sequence and starts it over.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ControllerOutput` value.
    fn abort(&mut self) -> ControllerOutput {
        self.cancel();
        ControllerOutput::default()
    }
    /// Abort, remembering the target when a receipt poll already counted against it.
    ///
    ///  Abort, remembering the target when a receipt poll already counted against it.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ControllerOutput` value.
    fn abort_polled(&mut self) -> ControllerOutput {
        if self.receipt_polls > 0 {
            self.remember_target();
        }
        self.abort()
    }
    /// Last-resort receipt fence when BOTH native animation and projectile
    /// clocks are unavailable/frozen. This never advances a move or adds input.
    ///
    ///  Last-resort receipt fence when BOTH native animation and projectile clocks are unavailable/frozen. This never advances a move or adds input.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `tick` (`i64`): Represents the tick; the function uses it as input to the operation described above.
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
    /// Checks whether the requested input was acknowledged by the mapper.
    ///
    ///
    ///  Checks whether the controller mapping accepted the input.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `tick` (`i64`): Represents the tick; the function uses it as input to the operation described above.
    pub fn mapped(&mut self, tick: i64) {
        if self.target.is_some() {
            self.delivery.mapped(tick);
            if !self.sent {
                self.remember_target();
                self.sent = true;
            }
        }
    }
    /// Updates the toggle state from the current button sample and match state.
    ///
    ///
    ///  Checks the current buttons and updates whether the toggle should activate.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `allowed` (`bool`): Represents the allowed; the function uses it as input to the operation described above.
    /// - `config` (`Option<Config>`): Represents the configuration being read or changed; the function uses it as input to the operation described above.
    /// - `c` (`Context`): Represents the c; the function uses it as input to the operation described above.
    /// - `threat` (`Option<Threat>`): Represents the threat; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ControllerOutput` value.
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
    ///
    ///  Whether a new reflect started now covers this shot: Game & Watch can act, the shot is a fresh attempt, and it arrives inside the startup window (a turn-around costs extra startup and keeps a larger margin).
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `config` (`Config`): Represents the configuration being read or changed; the function uses it as input to the operation described above.
    /// - `c` (`Context`): Represents the c; the function uses it as input to the operation described above.
    /// - `threat` (`Threat`): Represents the threat; the function uses it as input to the operation described above.
    /// - `facing` (`bool`): Represents the facing; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
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
