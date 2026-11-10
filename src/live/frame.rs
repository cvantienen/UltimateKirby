//! Per-observation lazy native reads, shared by every active policy. Drop this
//! at callback exit: never reuse it for a mapper poll or later game update.
//! Native accessors are validated by callers and are never stored here. Gates,
//! hit/contact flags and transitions remain live queries. Identical floor probes
//! share results only within this synchronous observation, never between frames.
use crate::game_ffi as game;
use kirby_assist_core::observation::{
    Fact, Field, GroundSupportSnapshot, ObservationSnapshotSized, RosterSnapshot,
};
/// Holds short-lived caches shared by policies during one accepted
/// native observation; never persists a native accessor pointer.
///
///  Keeps repeated reads consistent while one game update is processed.
#[derive(Default)]
pub(super) struct FrameReads {
    /// Bounded cache of copied native facts keyed by object and field.
    ///
    ///  Avoids rereading the same fighter value during one observation.
    facts: ObservationSnapshotSized<16>,
    /// Per-observation cache of swept ground-support probes.
    ///
    ///  Reuses ground checks while processing this update.
    ground: GroundSupportSnapshot,
    /// Per-observation cache of copied fighter-roster IDs.
    ///
    ///  Shares the current fighter list among this update's policies.
    roster: RosterSnapshot<16>,
    /// Lazily cached reserved damage launch vector for one object.
    ///
    ///  Keeps repeated launch-speed reads consistent for this update.
    damage_launch: std::cell::Cell<Option<(u32, [f32; 2])>>,
    /// Lazily cached validated ground-brake parameter for one object.
    ///
    ///  Reads the fighter's braking value at most once this update.
    ground_brake: std::cell::Cell<Option<(u32, Option<f32>)>>,
}
impl FrameReads {
    /// Caches whether either capture-related link slot is active.
    ///
    ///  Checks whether the fighter is being held by a capture link.
    ///
    /// # Safety
    /// `object` must be a live native object for the duration of this read.
    pub unsafe fn capture_link(&self, object: *mut game::Object) -> bool {
        let id = game::battle_object_id(object);
        // LINK_NO_CAPTURE and LINK_NO_CAPTURE_STOP. The latter also covers a
        // captor-side stop link; IC partner/article links are different slots.
        match self.facts.read(id, Field::CaptureLink, || {
            Fact::Bool(game::is_link(object, 0) || game::is_link(object, 2))
        }) {
            Fact::Bool(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    /// Returns copied object/status pairs already present in the fact cache.
    ///
    ///  Lists status values this update has already read.
    pub fn known_statuses(&self) -> [Option<(u32, i32)>; 16] {
        self.facts.known_statuses()
    }
    /// Converts a cached status/motion/frame/rate tuple into a guard clock.
    ///
    ///  Builds a reusable animation timer from facts already observed.
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
    ///
    ///  Reuses the roster list for this update; later reads still check it.
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
    ///
    ///  Gets a usable braking value only when a sequence needs it.
    ///
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
    /// Reserved launch vector during victim hitstop; shared by DI and tech prediction.
    /// Caches the native reserved X/Y damage speeds for this observation.
    ///
    ///  Gets the fighter's launch direction and speed for prediction.
    ///
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
    ///
    ///  Checks whether the fighter's possible movement stays over ground.
    ///
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
    /// Reads or reuses the native fighter-kind fact for this object.
    ///
    ///  Gets which character this fighter is.
    ///
    /// # Safety
    /// `object` must be a live native object for the duration of the read.
    pub unsafe fn fighter_kind(&self, object: *mut game::Object) -> i32 {
        let id = game::battle_object_id(object);
        match self
            .facts
            .read(id, Field::Kind, || Fact::Int(game::fighter_kind(id)))
        {
            Fact::Int(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    /// Reads or reuses the object's native status ID.
    ///
    ///  Gets the fighter's current action/state.
    ///
    /// # Safety
    /// `object` must be a live native object for the duration of the read.
    pub unsafe fn status_kind(&self, object: *mut game::Object) -> i32 {
        let id = game::battle_object_id(object);
        match self
            .facts
            .read(id, Field::Status, || Fact::Int(game::status_kind(object)))
        {
            Fact::Int(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    /// Reads or reuses the object's native situation ID.
    ///
    ///  Gets whether the fighter is grounded, airborne, or elsewhere.
    ///
    /// # Safety
    /// `object` must be a live native object for the duration of the read.
    pub unsafe fn situation_kind(&self, object: *mut game::Object) -> i32 {
        let id = game::battle_object_id(object);
        match self.facts.read(id, Field::Situation, || {
            Fact::Int(game::situation_kind(object))
        }) {
            Fact::Int(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    /// Reads or reuses the native motion hash.
    ///
    ///  Identifies the animation playing now.
    ///
    /// # Safety
    /// `object` must be a live native object for the duration of the read.
    pub unsafe fn motion_kind(&self, object: *mut game::Object) -> u64 {
        let id = game::battle_object_id(object);
        match self.facts.read(id, Field::Motion, || {
            Fact::Motion(game::motion_kind(object))
        }) {
            Fact::Motion(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    /// Reads or reuses the fractional frame within the native motion.
    ///
    ///  Shows how far the current animation has progressed.
    ///
    /// # Safety
    /// `object` must be a live native object for the duration of the read.
    pub unsafe fn motion_frame(&self, object: *mut game::Object) -> f32 {
        let id = game::battle_object_id(object);
        match self
            .facts
            .read(id, Field::Frame, || Fact::Float(game::motion_frame(object)))
        {
            Fact::Float(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    /// Reads or reuses the native motion playback rate.
    ///
    ///  Shows how quickly the current animation is running.
    ///
    /// # Safety
    /// `object` must be a live native object for the duration of the read.
    pub unsafe fn motion_rate(&self, object: *mut game::Object) -> f32 {
        let id = game::battle_object_id(object);
        match self
            .facts
            .read(id, Field::Rate, || Fact::Float(game::motion_rate(object)))
        {
            Fact::Float(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    /// Reads or reuses the object's horizontal position.
    ///
    ///  Gets the fighter's left/right location.
    ///
    /// # Safety
    /// `object` must be a live native object for the duration of the read.
    pub unsafe fn pos_x(&self, object: *mut game::Object) -> f32 {
        let id = game::battle_object_id(object);
        match self
            .facts
            .read(id, Field::X, || Fact::Float(game::pos_x(object)))
        {
            Fact::Float(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    /// Reads or reuses the object's vertical position.
    ///
    ///  Gets the fighter's height above or below the stage.
    ///
    /// # Safety
    /// `object` must be a live native object for the duration of the read.
    pub unsafe fn pos_y(&self, object: *mut game::Object) -> f32 {
        let id = game::battle_object_id(object);
        match self
            .facts
            .read(id, Field::Y, || Fact::Float(game::pos_y(object)))
        {
            Fact::Float(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    /// Reads or reuses the object's facing direction.
    ///
    ///  Gets which way the fighter is facing.
    ///
    /// # Safety
    /// `object` must be a live native object for the duration of the read.
    pub unsafe fn facing(&self, object: *mut game::Object) -> f32 {
        let id = game::battle_object_id(object);
        match self
            .facts
            .read(id, Field::Facing, || Fact::Float(game::facing(object)))
        {
            Fact::Float(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    /// Reads or reuses the object's native team identifier.
    ///
    ///  Gets which team the object belongs to.
    ///
    /// # Safety
    /// `object` must be a live native object for the duration of the read.
    pub unsafe fn team(&self, object: *mut game::Object) -> u64 {
        let id = game::battle_object_id(object);
        match self
            .facts
            .read(id, Field::Team, || Fact::Motion(game::team(object)))
        {
            Fact::Motion(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    /// Reads or reuses the object's native stop flag.
    ///
    ///  Checks whether the game has frozen this object.
    ///
    /// # Safety
    /// `object` must be a live native object for the duration of the read.
    pub unsafe fn is_stop(&self, object: *mut game::Object) -> bool {
        let id = game::battle_object_id(object);
        match self
            .facts
            .read(id, Field::Stop, || Fact::Bool(game::is_stop(object)))
        {
            Fact::Bool(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    /// Reads or reuses the native end frame for the current motion.
    ///
    ///  Gets the configured final frame of the animation.
    ///
    /// # Safety
    /// `object` must be a live native object for the duration of the read.
    pub unsafe fn motion_end_frame(&self, object: *mut game::Object) -> f32 {
        let id = game::battle_object_id(object);
        match self.facts.read(id, Field::EndFrame, || {
            Fact::Float(game::motion_end_frame(object))
        }) {
            Fact::Float(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    /// Reads or reuses horizontal speed across all kinetic reserves.
    ///
    ///  Gets the fighter's overall left/right speed.
    ///
    /// # Safety
    /// `object` must be a live native object for the duration of the read.
    pub unsafe fn speed_x(&self, object: *mut game::Object) -> f32 {
        let id = game::battle_object_id(object);
        match self.facts.read(id, Field::Vx, || {
            Fact::Float(game::speed_x(object, game::ALL_KINETIC_ENERGY_RESERVES))
        }) {
            Fact::Float(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    /// Reads or reuses vertical speed across all kinetic reserves.
    ///
    ///  Gets the fighter's overall up/down speed.
    ///
    /// # Safety
    /// `object` must be a live native object for the duration of the read.
    pub unsafe fn speed_y(&self, object: *mut game::Object) -> f32 {
        let id = game::battle_object_id(object);
        match self.facts.read(id, Field::Vy, || {
            Fact::Float(game::speed_y(object, game::ALL_KINETIC_ENERGY_RESERVES))
        }) {
            Fact::Float(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    /// Reads a requested previous status, caching the latest-history entry.
    ///
    ///  Gets the earlier action from the fighter's status history.
    ///
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
