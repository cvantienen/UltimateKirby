//! Kirby-specific follow-up admission and state lifecycle.
//! Technical: Adapts native fighter observations into core Kirby trip and forward-air
//! follow-up state machines, then returns controller outputs.
//! Plain English: This module handles Kirby-only follow-up behavior using observed
//! game state and generated controller input.
use kirby_assist_core::{
    gameplay::ControllerOutput,
    kirby::{FairContext, FairFollowup, TripContext, TripFollowup, TripTarget},
    native_states::status,
    timing::{motion_hash, MOTION_ATTACK_AIR_F},
};

/// Compares a native fighter-kind value with Kirby's catalog ID.
///
///  Tells whether the observed fighter is Kirby.
pub(super) fn is_kirby(kind: i32) -> bool {
    kind == kirby_assist_core::fighter_kinds::KIRBY
}

/// Enables trip assistance only when it is not suspended, configured,
/// and the fighter kind is Kirby.
///
///  Allows the trip helper only for enabled Kirby play.
pub(super) fn trip_enabled(suspended: bool, enabled: bool, kind: i32) -> bool {
    !suspended && enabled && is_kirby(kind)
}

/// Admits trip watching for enabled side/down tilts or a queued follow-up.
///
///  Watches for a trip during those attacks or when one is already queued.
pub(super) fn trip_watch(enabled: bool, status: i32, queued: bool) -> bool {
    enabled && (matches!(status, status::ATTACK_S3 | status::ATTACK_LW3) || queued)
}

/// Reads the core trip-follow-up watch state when one exists.
///
///  Checks whether trip follow-up tracking is currently active.
pub(super) fn watching(state: &Option<TripFollowup>) -> bool {
    state.as_ref().is_some_and(TripFollowup::watching)
}

/// Reads the core forward-air follow-up active state when one exists.
///
///  Checks whether forward-air follow-up tracking is active.
pub(super) fn fair_active(state: &Option<FairFollowup>) -> bool {
    state.as_ref().is_some_and(FairFollowup::active)
}

/// Stores the copied native fighter fields consumed by follow-up policies.
///
///  This is a snapshot of the fighter's current state for the helpers.
pub(super) struct Observation {
    /// Native battle-object ID for the observed fighter.
    ///
    ///  Identifies which fighter this snapshot describes.
    pub object: u32,
    /// Native fighter-kind value used for fighter-specific policy gates.
    ///
    ///  Says which character is being observed.
    pub kind: i32,
    /// Native status identifier at observation time.
    ///
    ///  Describes the fighter's current action or state.
    pub status: i32,
    /// Native ground/air situation identifier.
    ///
    ///  Says whether the fighter is on the ground or in the air.
    pub situation: i32,
    /// Hash of the current native motion.
    ///
    ///  Identifies the animation currently playing.
    pub motion: u64,
    /// Current frame position within the native motion.
    ///
    ///  Shows how far the current animation has progressed.
    pub frame: f32,
    /// Native motion playback-rate value.
    ///
    ///  Shows how quickly the current animation is playing.
    pub rate: f32,
    /// Whether native game stop is active for this fighter.
    ///
    ///  Indicates whether the fighter is currently frozen.
    pub stopped: bool,
    /// Native facing direction used by directional follow-up logic.
    ///
    ///  Shows which way the fighter is facing.
    pub facing: f32,
}

/// Builds a trip-policy context from the observation and target snapshot.
///
///  Packages what trip assistance needs to decide what to do next.
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

/// Builds a forward-air policy context and evaluates hit state only inside
/// the supplied hit window.
///
///  Packages the fighter's current attack state and whether a hit happened.
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

/// Admits forward-air hit observation only for enabled Kirby forward-air
/// or its landing status when assistance is not suspended.
///
///  Decides when the forward-air helper is allowed to watch for a hit.
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

/// Samples or clears the fair-follow-up state machine and returns its
/// controller output.
///
///  Advances Kirby's forward-air helper or resets it when disabled.
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

/// Samples or clears the trip-follow-up state machine and returns its
/// controller output.
///
///  Advances Kirby's trip helper or resets it when disabled.
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
