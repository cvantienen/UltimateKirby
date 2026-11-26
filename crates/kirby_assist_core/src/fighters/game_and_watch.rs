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
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};

/// Extra frames added to Oil Panic startup when predicting projectile contact.
const REFLECT_CONTACT_SAFETY_MARGIN_FRAMES: f32 = 2.0;
/// Largest projectile hitbox radius accepted by Oil Panic prediction.
const MAX_PROJECTILE_RADIUS: f32 = 100.0;
/// Longest projectile-contact horizon accepted by Oil Panic prediction.
const MAX_REFLECTOR_HORIZON_FRAMES: f32 = 40.0;
/// Mapper-poll limit while awaiting Oil Panic input acknowledgement.
const MAX_RECEIPT_POLL_COUNT: u8 = 24;
/// Time limit for native state to confirm Oil Panic startup.
const MAX_UNCONFIRMED_REFLECT_FRAMES: f32 = 24.0;
/// Maximum time allowed for the pre-reflector turnaround.
const MAX_TURNING_FRAMES: f32 = 2.0;
/// Maximum native frames allowed to deliver the Oil Panic input.
const MAX_UNSENT_REFLECT_FRAMES: f32 = 4.0;
/// Recently attempted shots remembered so a missed/consumed shot is not retried.
const ATTEMPT_MEMORY: usize = 32;

/// Native fighter-kind ID for Mr. Game & Watch.
pub const GAMEWATCH: i32 = crate::fighter_kinds::MR_GAME_AND_WATCH;
/// Adding a fighter requires a verified native move/resource adapter too.
pub fn supports(kind: i32) -> bool {
    kind == GAMEWATCH
}

/// Mr. Game & Watch reflector timing and estimated body envelope.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    /// Native Oil Panic startup before reflection or absorption can begin.
    pub startup: f32,
    /// Extra prediction margin before projectile contact.
    pub margin: f32,
    /// Maximum frames to hold one confirmed Oil Panic request.
    pub max_hold: f32,
    /// Estimated horizontal half-width of Game & Watch's body.
    pub half_width: f32,
    /// Estimated body height above Game & Watch's root.
    pub height: f32,
    /// Estimated depth allowance for off-plane projectile checks.
    pub half_depth: f32,
}
impl Config {
    /// Builds Oil Panic timing and body estimates for supported Game & Watch settings.
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
    /// Frames of lead time needed to start Oil Panic before projectile contact.
    pub fn horizon(self) -> f32 {
        self.startup + self.margin + REFLECT_CONTACT_SAFETY_MARGIN_FRAMES
    }
}

/// Whether Game & Watch can turn and start Oil Panic from this native state.
pub fn movable(status: i32, situation: i32) -> bool {
    match situation {
        crate::native_states::situation::GROUND => {
            matches!(status, crate::native_states::status::WAIT..=crate::native_states::status::JUMP_SQUAT | crate::native_states::status::SQUAT..=crate::native_states::status::LANDING_LIGHT | crate::native_states::status::GUARD_OFF | crate::native_states::status::OTTOTTO | crate::native_states::status::OTTOTTO_WAIT)
        }
        // Ordinary jump/fall only.
        crate::native_states::situation::AIR => crate::native_states::jump_or_fall(status),
        _ => false,
    }
}
/// Whether Game & Watch is in an Oil Panic hold/absorb status.
pub fn bucket(status: i32) -> bool {
    matches!(
        status,
        crate::native_states::status::SPECIAL_LW
            | crate::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_WAIT_START
            | crate::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_WAIT
            | crate::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_CATCH
            | crate::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_REFLECT
    )
}

/// Predicted incoming projectile contact used to time Oil Panic.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Threat {
    /// Native battle-object ID of the threatening projectile.
    pub id: u32,
    /// Projectile attack generation used to detect a reused object.
    pub generation: u32,
    /// Predicted frames until projectile contact.
    pub in_frames: f32,
    /// Horizontal direction from Game & Watch toward the projectile.
    pub direction: f32,
}
/// A new native ATTACK on a reused object is a fresh opportunity, not the
/// previous shot. No wall-clock cooldown or permanent object-ID blacklist.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectileKey {
    /// Native projectile battle-object ID.
    pub id: u32,
    /// Attack generation for this projectile object.
    pub generation: u32,
}
impl Threat {
    /// Identifies this exact projectile attack generation.
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

/// Copied Mr. Game & Watch state used to decide whether Oil Panic can be requested.
#[derive(Clone, Copy)]
pub struct Context {
    /// Observed fighter's native kind ID.
    pub kind: i32,
    /// Observed ground, air, or ledge situation.
    pub situation: i32,
    /// Observed left/right facing sign.
    pub facing: f32,
    /// Whether native transitions currently permit a special move.
    pub can_special: bool,
    /// Current Oil Panic bucket gauge.
    pub gauge: f32,
    /// Whether native animation progress is stopped.
    pub stopped: bool,
    /// Current status, motion, frame, and playback-rate sample.
    pub animation: AnimationSample,
}
/// One current input owner. Delivered/expired attempts remember their native
/// attack generation. Losing arbitration does not consume an unseen attack;
/// a fresh shot on the same object can rearm on the first legal observation.
pub struct Reflector {
    /// Native animation clock used to age the current Oil Panic request.
    clock: AnimationClock,
    /// Projectile generation currently assigned to the reflector.
    target: Option<ProjectileKey>,
    /// Native tick on which the reflector request began.
    began: i64,
    /// Whether Game & Watch must turn before using Oil Panic.
    turning: bool,
    /// Whether the Oil Panic input has been delivered.
    sent: bool,
    /// Bounded retry budget for delivering the Oil Panic input.
    delivery: crate::move_input::DeliveryBudget,
    /// Mapper polls spent waiting for delivery acknowledgement.
    receipt_polls: u8,
    /// Last native tick counted toward receipt acknowledgement.
    last_poll: Option<i64>,
    /// Recent projectile generations that must not be retried.
    attempted: [Option<ProjectileKey>; ATTEMPT_MEMORY],
    /// Ring-buffer slot for the next remembered projectile.
    next: usize,
}
impl Default for Reflector {
    /// Starts with no active or remembered Oil Panic target.
    fn default() -> Self {
        Self::new()
    }
}
impl Reflector {
    /// Creates an idle Oil Panic state machine with no remembered projectile.
    pub const fn new() -> Self {
        Self {
            clock: AnimationClock::new(),
            target: None,
            began: 0,
            turning: false,
            sent: false,
            delivery: crate::move_input::DeliveryBudget::new(),
            receipt_polls: 0,
            last_poll: None,
            attempted: [None; ATTEMPT_MEMORY],
            next: 0,
        }
    }
    /// Whether Oil Panic currently owns a projectile target.
    pub fn active(&self) -> bool {
        self.target.is_some()
    }
    /// Whether the mapped down-special was delivered but native startup is pending.
    pub fn awaiting_receipt(&self) -> bool {
        self.target.is_some() && self.sent
    }
    /// Battle-object ID of the projectile currently assigned to Oil Panic.
    pub fn target(&self) -> Option<u32> {
        self.target.map(|key| key.id)
    }
    /// Exact projectile attack generation currently reserved.
    pub fn target_key(&self) -> Option<ProjectileKey> {
        self.target
    }
    /// Remembers the current projectile generation so it is not retried.
    fn remember_target(&mut self) {
        if let Some(id) = self
            .target
            .filter(|id| !self.attempted.contains(&Some(*id)))
        {
            self.attempted[self.next % ATTEMPT_MEMORY] = Some(id);
            self.next = self.next.wrapping_add(1);
        }
    }
    /// Releases Oil Panic ownership while preserving the recent-attempt memory.
    pub fn cancel(&mut self) {
        self.target = None;
        self.turning = false;
        self.sent = false;
        self.delivery = crate::move_input::DeliveryBudget::new();
        self.receipt_polls = 0;
        self.last_poll = None;
    }
    /// Aborts the active sequence and resets its progress.
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
        if self.awaiting_receipt() && self.last_poll != Some(tick) {
            self.last_poll = Some(tick);
            self.receipt_polls = self.receipt_polls.saturating_add(1);
            if self.receipt_polls >= MAX_RECEIPT_POLL_COUNT {
                self.remember_target();
                self.cancel();
            }
        }
    }
    /// Checks whether the requested input was acknowledged by the mapper.
    pub fn mapped(&mut self, tick: i64) {
        if self.target.is_some() {
            self.delivery.mapped(tick);
            if !self.sent {
                self.remember_target();
                self.sent = true;
            }
        }
    }
    /// Advances Oil Panic targeting and emits only the controller input needed now.
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
        if self.sent {
            if bucket(status) {
                self.cancel();
                return Default::default();
            } else if !movable(status, c.situation)
                || elapsed >= MAX_UNCONFIRMED_REFLECT_FRAMES
                || self.delivery.exhausted()
            {
                return self.abort();
            } else {
                // Keep the mapped input's receipt reservation while the
                // projectile disappears or native startup is delayed online.
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
        if self.target.is_some() && self.target != Some(threat.key()) {
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
        if !facing
            || !movable(status, c.situation)
            || !c.can_special
            || elapsed >= MAX_UNSENT_REFLECT_FRAMES
            || self.delivery.exhausted()
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
    /// Start on an observed incoming shot even if the estimated contact is
    /// sooner than startup; delaying cannot improve a late defense.
    fn admissible(&self, config: Config, c: Context, threat: Threat, facing: bool) -> bool {
        movable(c.animation.status, c.situation)
            && c.can_special
            && !self.attempted.contains(&Some(threat.key()))
            && (facing || c.situation == crate::native_states::situation::GROUND)
            && (0.0..=config.horizon()).contains(&threat.in_frames)
    }
}

/// Fixed number of projectile source entries retained by [`Sources`].
pub const SOURCE_CAPACITY: usize = 32;
/// Sentinel marking a vacant slot in the packed atomic record array.
const EMPTY_SOURCE: u64 = u64::MAX;

/// Bounded, allocation-free tracker of projectile identities that have
/// attacked, pairing each battle-object ID with the generation of its attack.
///
/// Records are packed into atomics so a concurrent snapshot always observes a
/// coherent ID/generation pair. Only copied IDs are retained, never native
/// pointers; callers revalidate identity and team before reading geometry.
pub struct Sources {
    /// Packed `generation << 32 | id` records; `EMPTY_SOURCE` marks vacancy.
    records: [AtomicU64; SOURCE_CAPACITY],
    /// Round-robin replacement cursor used once every record is occupied.
    next: AtomicUsize,
    /// Monotonic wrapping counter assigned to each observed attack.
    generation: AtomicU32,
}

impl Default for Sources {
    fn default() -> Self {
        Self::new()
    }
}

impl Sources {
    /// Constructs an empty tracker with generation zero.
    pub const fn new() -> Self {
        Self {
            records: [const { AtomicU64::new(EMPTY_SOURCE) }; SOURCE_CAPACITY],
            next: AtomicUsize::new(0),
            generation: AtomicU32::new(0),
        }
    }

    /// Records a weapon/item attack, advancing its generation. Non-projectile
    /// categories are ignored. An already tracked object is refreshed in place
    /// so a reused article or a fresh attack phase is not suppressed.
    pub fn remember(&self, id: u32) {
        if !matches!(
            id >> 28,
            crate::battle_objects::OBJECT_CATEGORY_WEAPON
                | crate::battle_objects::OBJECT_CATEGORY_ITEM
        ) {
            return;
        }
        let generation = self
            .generation
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1);
        let record = (u64::from(generation) << 32) | u64::from(id);
        let mut vacant = None;
        for (index, slot) in self.records.iter().enumerate() {
            let old = slot.load(Ordering::Relaxed);
            if old != EMPTY_SOURCE && old as u32 == id {
                slot.store(record, Ordering::Relaxed);
                return;
            }
            if old == EMPTY_SOURCE {
                vacant.get_or_insert(index);
            }
        }
        // Reuse a vacant slot before evicting a live projectile.
        let index =
            vacant.unwrap_or_else(|| self.next.fetch_add(1, Ordering::Relaxed) % SOURCE_CAPACITY);
        self.records[index].store(record, Ordering::Relaxed);
    }

    /// Copies every occupied record into an owned optional key array.
    pub fn snapshot(&self) -> [Option<ProjectileKey>; SOURCE_CAPACITY] {
        std::array::from_fn(|index| {
            let record = self.records[index].load(Ordering::Relaxed);
            (record != EMPTY_SOURCE).then_some(ProjectileKey {
                id: record as u32,
                generation: (record >> 32) as u32,
            })
        })
    }

    /// Clears the first record matching `id` without erasing a record that a
    /// concurrent `remember` has already replaced.
    pub fn forget(&self, id: u32) {
        for slot in &self.records {
            let old = slot.load(Ordering::Relaxed);
            if old != EMPTY_SOURCE
                && old as u32 == id
                && slot
                    .compare_exchange(old, EMPTY_SOURCE, Ordering::Relaxed, Ordering::Relaxed)
                    .is_ok()
            {
                break;
            }
        }
    }

    /// Empties every slot and resets the cursor. The generation counter is
    /// deliberately retained so reused IDs cannot collide with stale history.
    pub fn clear(&self) {
        for slot in &self.records {
            slot.store(EMPTY_SOURCE, Ordering::Relaxed);
        }
        self.next.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
#[path = "../../tests/unit/fighter_game_and_watch.rs"]
mod tests;
