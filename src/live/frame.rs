//! Per-observation lazy native reads, shared by every active policy. Drop this
//! at callback exit: never reuse it for a mapper poll or later game update.
//! Native accessors are validated by callers and are never stored here. Gates,
//! hit/contact flags and transitions remain live queries. Identical floor probes
//! share results only within this synchronous observation, never between frames.
use crate::game_ffi as game;
use kirby_assist_core::observation::{
    Fact, Field, GroundSupportSnapshot, ObservationSnapshotSized, RosterSnapshot,
};
/// Defines per-observation cached native readers.
///
/// Each entry names the method, the identifier bindings it needs (`id` is the
/// battle-object ID, `object` the accessor), the returned type, and the
/// [`Field`]/[`Fact`] pair used to memoize it. The generated body reads through
/// the fact cache and unwraps the matching variant, which cannot fail because
/// one field is only ever written with one variant.
macro_rules! cached_reads {
    ($(
        $(#[$meta:meta])*
        $name:ident($id:tt, $object:tt) -> $ret:ty = $field:ident: $variant:ident($read:expr);
    )*) => {$(
        $(#[$meta])*
        /// # Safety
        /// `object` must be a live native object for the duration of the read.
        pub unsafe fn $name(&self, object: *mut game::Object) -> $ret {
            let $id = game::battle_object_id(object);
            let $object = object;
            match self.facts.read($id, Field::$field, || Fact::$variant($read)) {
                Fact::$variant(value) => value,
                _ => unreachable!("snapshot field type"),
            }
        }
    )*};
}

/// Holds short-lived caches shared by policies during one accepted
/// native observation; never persists a native accessor pointer.
#[derive(Default)]
pub(super) struct FrameReads {
    /// Bounded cache of copied native facts keyed by object and field.
    facts: ObservationSnapshotSized<16>,
    /// Per-observation cache of swept ground-support probes.
    ground: GroundSupportSnapshot,
    /// Per-observation cache of copied fighter-roster IDs.
    roster: RosterSnapshot<16>,
    /// Lazily cached reserved damage launch vector for one object.
    damage_launch: std::cell::Cell<Option<(u32, [f32; 2])>>,
    /// Lazily cached validated ground-brake parameter for one object.
    ground_brake: std::cell::Cell<Option<(u32, Option<f32>)>>,
}
impl FrameReads {
    cached_reads! {
        /// Caches whether either capture-related link slot is active.
        ///
        /// Slots 0 and 2 are LINK_NO_CAPTURE and LINK_NO_CAPTURE_STOP. The
        /// latter also covers a captor-side stop link; IC partner/article
        /// links are different slots.
        capture_link(_id, object) -> bool =
            CaptureLink: Bool(game::is_link(object, 0) || game::is_link(object, 2));
    }
    /// Returns copied object/status pairs already present in the fact cache.
    pub fn known_statuses(&self) -> [Option<(u32, i32)>; 16] {
        self.facts.known_statuses()
    }
    /// Converts a cached status/motion/frame/rate tuple into a guard clock.
    pub fn known_guard_clock(
        &self,
        own: u32,
    ) -> Option<(u32, kirby_assist_core::autopilot_guard::GuardClock)> {
        self.facts
            .known_progress_clock(own)
            .map(|(id, status, motion, frame, rate)| {
                (
                    id,
                    kirby_assist_core::autopilot_guard::GuardClock {
                        status,
                        motion,
                        frame,
                        rate,
                        stopped: false,
                    },
                )
            })
    }
    /// Share copied roster IDs only within this accepted observation. Every
    /// consumer still revalidates native identity/team before reading fighters.
    /// Lazily obtains the roster snapshot stored in shared callback state.
    pub fn roster(&self) -> Option<kirby_assist_core::battle_objects::FightersInGame> {
        self.roster.read(|| {
            super::STATE
                .lock()
                .ok()
                .and_then(|s| s.ledge_roster.map(|r| r.0))
        })
    }
    /// Both sequence entry points use the same fighter braking parameter.
    /// Lazy and local to this observation; no extra reads for Drift-only tapes.
    /// Reads and validates the fighter ground-brake parameter once per object.
    /// # Safety
    /// `object` must be a live native object for the duration of this read.
    pub unsafe fn ground_brake(&self, object: *mut game::Object) -> Option<f32> {
        let id = game::battle_object_id(object);
        if let Some((_, value)) = self.ground_brake.get().filter(|(cached, _)| *cached == id) {
            return value;
        }
        let brake = game::param_float(object, super::native_params::PARAM_GROUND_BRAKE, 0);
        let value = (brake.is_finite() && (0.001..=3.).contains(&brake)).then_some(brake);
        self.ground_brake.set(Some((id, value)));
        value
    }
    /// Reserved launch vector during victim hitstop for directional influence.
    /// Caches the native reserved X/Y damage speeds for this observation.
    /// # Safety
    /// `object` must be a live native object for the duration of this read.
    pub unsafe fn damage_launch(&self, object: *mut game::Object) -> [f32; 2] {
        let id = game::battle_object_id(object);
        if let Some((_, velocity)) = self.damage_launch.get().filter(|(cached, _)| *cached == id) {
            return velocity;
        }
        let velocity = [
            game::work_float(
                object,
                kirby_assist_core::native_states::work_ids::FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_SPEED_RESERVED_X,
            ),
            game::work_float(
                object,
                kirby_assist_core::native_states::work_ids::FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_SPEED_RESERVED_Y,
            ),
        ];
        self.damage_launch.set(Some((id, velocity)));
        velocity
    }
    /// Caches and returns the two swept support results for the supplied path.
    /// # Safety
    /// `object` must be a live native object for the duration of the probe.
    pub unsafe fn ground_support(
        &self,
        object: *mut game::Object,
        position: [f32; 2],
        vx: f32,
    ) -> [bool; 2] {
        let id = game::battle_object_id(object);
        self.ground.read(id, position, vx, || {
            super::observations::ground_support(object, position, vx)
        })
    }
    cached_reads! {
        /// Reads or reuses the native fighter-kind fact for this object.
        fighter_kind(id, _object) -> i32 = Kind: Int(game::fighter_kind(id));
        /// Reads or reuses the object's native status ID.
        status_kind(_id, object) -> i32 = Status: Int(game::status_kind(object));
        /// Reads or reuses the object's native situation ID.
        situation_kind(_id, object) -> i32 = Situation: Int(game::situation_kind(object));
        /// Reads or reuses the native motion hash.
        motion_kind(_id, object) -> u64 = Motion: Motion(game::motion_kind(object));
        /// Reads or reuses the fractional frame within the native motion.
        motion_frame(_id, object) -> f32 = Frame: Float(game::motion_frame(object));
        /// Reads or reuses the native motion playback rate.
        motion_rate(_id, object) -> f32 = Rate: Float(game::motion_rate(object));
        /// Reads or reuses the object's horizontal position.
        pos_x(_id, object) -> f32 = X: Float(game::pos_x(object));
        /// Reads or reuses the object's vertical position.
        pos_y(_id, object) -> f32 = Y: Float(game::pos_y(object));
        /// Reads or reuses the object's facing direction.
        facing(_id, object) -> f32 = Facing: Float(game::facing(object));
        /// Reads or reuses the object's native team identifier.
        team(_id, object) -> u64 = Team: Motion(game::team(object));
        /// Reads or reuses the object's native stop flag.
        is_stop(_id, object) -> bool = Stop: Bool(game::is_stop(object));
        /// Reads or reuses the native end frame for the current motion.
        motion_end_frame(_id, object) -> f32 = EndFrame: Float(game::motion_end_frame(object));
        /// Reads or reuses horizontal speed across all kinetic reserves.
        speed_x(_id, object) -> f32 =
            Vx: Float(game::speed_x(object, game::ALL_KINETIC_ENERGY_RESERVES));
        /// Reads or reuses vertical speed across all kinetic reserves.
        speed_y(_id, object) -> f32 =
            Vy: Float(game::speed_y(object, game::ALL_KINETIC_ENERGY_RESERVES));
    }
    /// Reads a requested previous status, caching the latest-history entry.
    /// # Safety
    /// `object` must be a live native object for the duration of the read.
    pub unsafe fn previous_status(&self, object: *mut game::Object, history: u32) -> i32 {
        if history != 0 {
            return game::previous_status(object, history);
        }
        let id = game::battle_object_id(object);
        match self.facts.read(id, Field::PreviousStatus, || {
            Fact::Int(game::previous_status(
                object,
                game::LATEST_PREVIOUS_STATUS_HISTORY_INDEX,
            ))
        }) {
            Fact::Int(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
}
