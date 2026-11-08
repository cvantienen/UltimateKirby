//! Read-only observations for Kirby's independently enabled duck defense.
use super::{frame::FrameReads, observations};
use crate::game_ffi as game;
use kirby_assist_core::{
    auto_duck::{DefenseThreat, DuckContext},
    autopilot_guard::{Envelope, GuardClock},
    fighter_kinds::KIRBY,
    native_states::{self, situation, status, transition_terms},
};

pub(super) struct Observation {
    pub enabled: bool,
    pub kind: i32,
    pub object: u32,
    pub status: i32,
    pub situation: i32,
    pub motion: u64,
    pub frame: f32,
    pub rate: f32,
    pub stopped: bool,
    pub clatter: bool,
    pub tech: bool,
}

pub(super) unsafe fn observe(
    reads: &FrameReads,
    object: *mut game::Object,
    observation: &Observation,
    envelope: impl FnOnce() -> Envelope,
) -> Option<(DuckContext, DefenseThreat)> {
    if !observation.enabled
        || observation.kind != KIRBY
        || observation.situation != situation::GROUND
        || observation.clatter
        || observation.tech
    {
        return None;
    }
    let can_shield = matches!(
        observation.status,
        native_states::GUARD_ON | native_states::GUARD | native_states::GUARD_DAMAGE
    ) || (!observation.stopped
        && game::transition_enabled(object, transition_terms::CONT_GUARD_ON));
    let can_crouch = matches!(observation.status, status::SQUAT | status::SQUAT_WAIT)
        || (!observation.stopped
            && matches!(observation.status, status::WAIT..=status::TURN_RUN)
            && game::transition_enabled(object, transition_terms::CONT_SQUAT));
    let threats = if can_shield || can_crouch {
        observations::incoming_fighter_attack(
            reads,
            object,
            observation.object,
            [reads.pos_x(object), reads.pos_y(object)],
            [reads.speed_x(object), reads.speed_y(object)],
            true,
            envelope(),
        )
    } else {
        DefenseThreat::default()
    };
    Some((
        DuckContext {
            clock: GuardClock {
                status: observation.status,
                motion: observation.motion,
                frame: observation.frame,
                rate: observation.rate,
                stopped: observation.stopped,
            },
            can_crouch,
            crouched: observation.status == status::SQUAT_WAIT,
            can_shield,
        },
        threats,
    ))
}
