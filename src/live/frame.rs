//! Per-observation lazy native reads, shared by every active policy. Drop this
//! at callback exit: never reuse it for a mapper poll or later game update.
//! Native accessors are validated by callers and are never stored here. Gates,
//! hit/contact flags and transitions remain live queries. Identical floor probes
//! share results only within this synchronous observation, never between frames.
use crate::game_ffi as game;
use kirby_assist_core::observation::{
    Fact, Field, GroundSupportSnapshot, ObservationSnapshotSized, RosterSnapshot,
};
#[derive(Default)]
pub(super) struct FrameReads {
    facts: ObservationSnapshotSized<16>,
    ground: GroundSupportSnapshot,
    roster: RosterSnapshot<16>,
    damage_launch: std::cell::Cell<Option<(u32, [f32; 2])>>,
    ground_brake: std::cell::Cell<Option<(u32, Option<f32>)>>,
}
impl FrameReads {
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
    pub fn known_statuses(&self) -> [Option<(u32, i32)>; 16] {
        self.facts.known_statuses()
    }
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
    pub unsafe fn situation_kind(&self, object: *mut game::Object) -> i32 {
        let id = game::battle_object_id(object);
        match self.facts.read(id, Field::Situation, || {
            Fact::Int(game::situation_kind(object))
        }) {
            Fact::Int(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    pub unsafe fn motion_kind(&self, object: *mut game::Object) -> u64 {
        let id = game::battle_object_id(object);
        match self.facts.read(id, Field::Motion, || {
            Fact::Motion(game::motion_kind(object))
        }) {
            Fact::Motion(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
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
    pub unsafe fn motion_end_frame(&self, object: *mut game::Object) -> f32 {
        let id = game::battle_object_id(object);
        match self.facts.read(id, Field::EndFrame, || {
            Fact::Float(game::motion_end_frame(object))
        }) {
            Fact::Float(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    pub unsafe fn speed_x(&self, object: *mut game::Object) -> f32 {
        let id = game::battle_object_id(object);
        match self.facts.read(id, Field::Vx, || {
            Fact::Float(game::speed_x(object, game::ALL_KINETIC_ENERGY_RESERVES))
        }) {
            Fact::Float(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    pub unsafe fn speed_y(&self, object: *mut game::Object) -> f32 {
        let id = game::battle_object_id(object);
        match self.facts.read(id, Field::Vy, || {
            Fact::Float(game::speed_y(object, game::ALL_KINETIC_ENERGY_RESERVES))
        }) {
            Fact::Float(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
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
