//! Kirby-specific follow-up admission and state lifecycle.
use kirby_assist_core::{
    gameplay::ControllerOutput,
    kirby::{FairContext, FairFollowup, TripContext, TripFollowup},
    native_states::status,
    timing::{motion_hash, MOTION_ATTACK_AIR_F},
};

pub(super) fn fair_hit_window(
    suspended: bool,
    is_kirby: bool,
    enabled: bool,
    status: i32,
    motion: u64,
) -> bool {
    !suspended
        && is_kirby
        && enabled
        && (status == status::ATTACK_AIR && motion == MOTION_ATTACK_AIR_F
            || status == status::LANDING_ATTACK_AIR && motion == motion_hash("landing_air_f"))
}

pub(super) fn fair(
    state: &mut Option<FairFollowup>,
    enabled: bool,
    allowed: bool,
    context: FairContext,
    velocity_y: f32,
) -> ControllerOutput {
    if enabled {
        state
            .get_or_insert_with(FairFollowup::default)
            .sample_with_velocity(allowed, context, velocity_y)
    } else {
        *state = None;
        ControllerOutput::default()
    }
}

pub(super) fn trip(
    state: &mut Option<TripFollowup>,
    enabled: bool,
    allowed: bool,
    context: TripContext,
) -> ControllerOutput {
    if enabled {
        state
            .get_or_insert_with(TripFollowup::default)
            .sample(allowed, context)
    } else {
        *state = None;
        ControllerOutput::default()
    }
}
