//! Per-observation lazy native reads, shared by every active policy. Drop this
//! at callback exit: never reuse it for a mapper poll or later game update.
//! Native accessors are validated by callers and are never stored here. Gates,
//! hit/contact flags, transitions and geometry remain uncached live queries.
use crate::game_ffi as game;
use kirby_assist_core::observation::{Fact, Field, ObservationSnapshot};
#[derive(Default)]
pub(super) struct FrameReads {
    facts: ObservationSnapshot,
}
impl FrameReads {
    pub unsafe fn status_kind(&self, object: *mut game::Object) -> i32 {
        let id = object.cast::<u8>().add(8).cast::<u32>().read();
        match self
            .facts
            .read(id, Field::Status, || Fact::Int(game::status_kind(object)))
        {
            Fact::Int(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    pub unsafe fn situation_kind(&self, object: *mut game::Object) -> i32 {
        let id = object.cast::<u8>().add(8).cast::<u32>().read();
        match self.facts.read(id, Field::Situation, || {
            Fact::Int(game::situation_kind(object))
        }) {
            Fact::Int(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    pub unsafe fn motion_kind(&self, object: *mut game::Object) -> u64 {
        let id = object.cast::<u8>().add(8).cast::<u32>().read();
        match self.facts.read(id, Field::Motion, || {
            Fact::Motion(game::motion_kind(object))
        }) {
            Fact::Motion(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    pub unsafe fn motion_frame(&self, object: *mut game::Object) -> f32 {
        let id = object.cast::<u8>().add(8).cast::<u32>().read();
        match self
            .facts
            .read(id, Field::Frame, || Fact::Float(game::motion_frame(object)))
        {
            Fact::Float(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    pub unsafe fn motion_rate(&self, object: *mut game::Object) -> f32 {
        let id = object.cast::<u8>().add(8).cast::<u32>().read();
        match self
            .facts
            .read(id, Field::Rate, || Fact::Float(game::motion_rate(object)))
        {
            Fact::Float(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    pub unsafe fn pos_x(&self, object: *mut game::Object) -> f32 {
        let id = object.cast::<u8>().add(8).cast::<u32>().read();
        match self
            .facts
            .read(id, Field::X, || Fact::Float(game::pos_x(object)))
        {
            Fact::Float(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    pub unsafe fn pos_y(&self, object: *mut game::Object) -> f32 {
        let id = object.cast::<u8>().add(8).cast::<u32>().read();
        match self
            .facts
            .read(id, Field::Y, || Fact::Float(game::pos_y(object)))
        {
            Fact::Float(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    pub unsafe fn facing(&self, object: *mut game::Object) -> f32 {
        let id = object.cast::<u8>().add(8).cast::<u32>().read();
        match self
            .facts
            .read(id, Field::Facing, || Fact::Float(game::facing(object)))
        {
            Fact::Float(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    pub unsafe fn team(&self, object: *mut game::Object) -> u64 {
        let id = object.cast::<u8>().add(8).cast::<u32>().read();
        match self
            .facts
            .read(id, Field::Team, || Fact::Motion(game::team(object)))
        {
            Fact::Motion(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    pub unsafe fn is_stop(&self, object: *mut game::Object) -> bool {
        let id = object.cast::<u8>().add(8).cast::<u32>().read();
        match self
            .facts
            .read(id, Field::Stop, || Fact::Bool(game::is_stop(object)))
        {
            Fact::Bool(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    pub unsafe fn motion_end_frame(&self, object: *mut game::Object) -> f32 {
        let id = object.cast::<u8>().add(8).cast::<u32>().read();
        match self.facts.read(id, Field::EndFrame, || {
            Fact::Float(game::motion_end_frame(object))
        }) {
            Fact::Float(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    pub unsafe fn speed_x(&self, object: *mut game::Object, energy: i32) -> f32 {
        if energy != -1 {
            return game::speed_x(object, energy);
        }
        let id = object.cast::<u8>().add(8).cast::<u32>().read();
        match self
            .facts
            .read(id, Field::Vx, || Fact::Float(game::speed_x(object, -1)))
        {
            Fact::Float(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    pub unsafe fn speed_y(&self, object: *mut game::Object, energy: i32) -> f32 {
        if energy != -1 {
            return game::speed_y(object, energy);
        }
        let id = object.cast::<u8>().add(8).cast::<u32>().read();
        match self
            .facts
            .read(id, Field::Vy, || Fact::Float(game::speed_y(object, -1)))
        {
            Fact::Float(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
    pub unsafe fn previous_status(&self, object: *mut game::Object, history: u32) -> i32 {
        if history != 0 {
            return game::previous_status(object, history);
        }
        let id = object.cast::<u8>().add(8).cast::<u32>().read();
        match self.facts.read(id, Field::PreviousStatus, || {
            Fact::Int(game::previous_status(object, 0))
        }) {
            Fact::Int(value) => value,
            _ => unreachable!("snapshot field type"),
        }
    }
}
