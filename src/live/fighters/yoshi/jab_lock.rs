//! Read-only observations and input-only Yoshi jab-lock lifecycle.
use super::{frame::FrameReads, observations};
use crate::game_ffi as game;
use kirby_assist_core::{
    fighter_kinds::YOSHI,
    fighter_tuning::{Tuning, JAB_LOCK_HEIGHT},
    gameplay::ControllerOutput,
    jab_lock::{Config, Context, JabLock, Target},
    manual_controls::turbo_movement_allowed,
    native_states::{situation, status, transition_terms},
};

pub(super) fn enabled(
    suspended: bool,
    kind: i32,
    controls: bool,
    selected: impl FnOnce() -> bool,
) -> bool {
    !suspended && kind == YOSHI && controls && selected()
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
    pub clatter: bool,
    pub tech: bool,
}

pub(super) unsafe fn observe(
    reads: &FrameReads,
    object: *mut game::Object,
    observation: &Observation,
    config: Option<Config>,
    tuning: Tuning,
    watching: impl FnOnce() -> bool,
) -> (Context, [Option<Target>; 8]) {
    let position = [reads.pos_x(object), reads.pos_y(object)];
    let watching = watching();
    let tilt_hit = observation.status == status::ATTACK_LW3
        && game::attack_infliction_now(object, game::COLLISION_KIND_MASK_HIT);
    let targets = observations::jab_lock_targets(
        reads,
        object,
        observation.object,
        watching || tilt_hit,
        config.map_or(60., |c| c.chase_distance),
        tuning.value(JAB_LOCK_HEIGHT, 24.),
    );
    let relevant = watching || tilt_hit || targets.iter().any(Option::is_some);
    let grounded = observation.situation == situation::GROUND;
    (
        Context {
            object: observation.object,
            kind: observation.kind,
            status: observation.status,
            motion: observation.motion,
            frame: observation.frame,
            rate: observation.rate,
            stopped: observation.stopped || observation.clatter || observation.tech,
            grounded,
            hit: tilt_hit,
            facing: reads.facing(object),
            position,
            can_move: turbo_movement_allowed(
                observation.status,
                observation.situation,
                observation.stopped,
            ),
            can_jab: relevant
                && grounded
                && game::transition_enabled(object, transition_terms::CONT_ATTACK),
            can_smash: relevant
                && grounded
                && game::transition_enabled(object, transition_terms::CONT_ATTACK_LW4_START),
            can_grab: relevant
                && grounded
                && game::transition_enabled(object, transition_terms::CONT_CATCH),
            safe_ground: if relevant && grounded {
                reads.ground_support(object, position, reads.speed_x(object))
            } else {
                [false; 2]
            },
        },
        targets,
    )
}

pub(super) fn sample(
    state: &mut Option<JabLock>,
    observation: Option<(Context, [Option<Target>; 8])>,
    allowed: bool,
    config: Option<Config>,
    tuning: Tuning,
) -> ControllerOutput {
    if let Some((context, targets)) = observation {
        state
            .get_or_insert_with(Default::default)
            .sample_configured(
                allowed,
                context,
                targets,
                config,
                tuning.value(JAB_LOCK_HEIGHT, 24.),
            )
    } else {
        *state = None;
        ControllerOutput::default()
    }
}
