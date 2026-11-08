//! Kirby-specific follow-up admission and state lifecycle.
use kirby_assist_core::{
    gameplay::ControllerOutput,
    kirby::{FairContext, FairFollowup, TripContext, TripFollowup, TripTarget},
    native_states::status,
    timing::{motion_hash, MOTION_ATTACK_AIR_F},
};

pub(super) fn is_kirby(kind: i32) -> bool {
    kind == kirby_assist_core::fighter_kinds::KIRBY
}

pub(super) fn trip_enabled(suspended: bool, enabled: bool, kind: i32) -> bool {
    !suspended && enabled && is_kirby(kind)
}

pub(super) fn trip_watch(enabled: bool, status: i32, queued: bool) -> bool {
    enabled && (matches!(status, status::ATTACK_S3 | status::ATTACK_LW3) || queued)
}

pub(super) fn watching(state: &Option<TripFollowup>) -> bool {
    state.as_ref().is_some_and(TripFollowup::watching)
}

pub(super) fn fair_active(state: &Option<FairFollowup>) -> bool {
    state.as_ref().is_some_and(FairFollowup::active)
}

pub(super) struct Observation {
    pub object: u32,
    pub kind: i32,
    pub status: i32,
    pub situation: i32,
    pub motion: u64,
    pub frame: f32,
    pub rate: f32,
    pub stopped: bool,
    pub facing: f32,
}

pub(super) fn trip_context(
    observation: &Observation,
    enabled: bool,
    targets: [Option<TripTarget>; 8],
) -> TripContext {
    TripContext {
        object: observation.object,
        kirby: enabled,
        status: observation.status,
        motion: observation.motion,
        frame: observation.frame,
        grounded: observation.situation == kirby_assist_core::native_states::situation::GROUND,
        stopped: observation.stopped,
        facing: observation.facing,
        targets,
    }
}

pub(super) fn fair_context(
    observation: &Observation,
    hit_window: bool,
    hit: impl FnOnce() -> bool,
) -> FairContext {
    FairContext {
        object: observation.object,
        kirby: is_kirby(observation.kind),
        status: observation.status,
        motion: observation.motion,
        frame: observation.frame,
        rate: observation.rate,
        grounded: observation.situation == kirby_assist_core::native_states::situation::GROUND,
        stopped: observation.stopped,
        facing: observation.facing,
        hit_now: hit_window && hit(),
    }
}

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
