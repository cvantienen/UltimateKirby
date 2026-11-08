#[path = "../../../src/live/fighters/kirby/followups.rs"]
mod followups;

use kirby_assist_core::{
    fighter_kinds,
    gameplay::ControllerOutput,
    kirby::{FairContext, TripContext, TripTarget},
    native_states::{situation, status},
    timing::{motion_hash, MOTION_ATTACK_AIR_F},
};

#[test]
fn contexts_preserve_fighter_gates_and_only_query_hits_in_the_fair_window() {
    let observation = followups::Observation {
        object: 1,
        kind: fighter_kinds::KIRBY,
        status: status::ATTACK_LW3,
        situation: situation::GROUND,
        motion: 7,
        frame: 2.0,
        rate: 1.0,
        stopped: false,
        facing: -1.0,
    };
    assert!(followups::is_kirby(observation.kind));
    assert!(followups::trip_enabled(false, true, observation.kind));
    assert!(!followups::trip_enabled(true, true, observation.kind));
    assert!(!followups::trip_enabled(false, true, fighter_kinds::YOSHI));
    assert!(followups::trip_watch(true, status::ATTACK_LW3, false));
    assert!(followups::trip_watch(true, status::WAIT, true));
    assert!(!followups::trip_watch(false, status::ATTACK_LW3, true));
    assert!(!followups::watching(&None));
    assert!(!followups::fair_active(&None));
    let trip = followups::trip_context(&observation, true, [None; 8]);
    assert!(trip.kirby && trip.grounded);
    assert_eq!(
        (trip.object, trip.status, trip.motion, trip.facing),
        (1, status::ATTACK_LW3, 7, -1.0)
    );
    let fair = followups::fair_context(&observation, false, || panic!("hit read outside window"));
    assert!(fair.kirby && fair.grounded && !fair.hit_now);
    assert!(followups::fair_context(&observation, true, || true).hit_now);
}

#[test]
fn fair_hit_window_requires_kirby_enabled_and_matching_motion() {
    for (status, motion) in [
        (status::ATTACK_AIR, MOTION_ATTACK_AIR_F),
        (status::LANDING_ATTACK_AIR, motion_hash("landing_air_f")),
    ] {
        assert!(followups::fair_hit_window(
            false, true, true, status, motion
        ));
        assert!(!followups::fair_hit_window(
            true, true, true, status, motion
        ));
        assert!(!followups::fair_hit_window(
            false, false, true, status, motion
        ));
        assert!(!followups::fair_hit_window(
            false, true, false, status, motion
        ));
        assert!(!followups::fair_hit_window(false, true, true, status, 0));
    }
    assert!(!followups::fair_hit_window(
        false,
        true,
        true,
        status::WAIT,
        MOTION_ATTACK_AIR_F
    ));
}

#[test]
fn disabling_or_disallowing_followups_releases_state_and_inputs() {
    let fair = FairContext {
        object: 1,
        kirby: true,
        status: status::ATTACK_AIR,
        motion: MOTION_ATTACK_AIR_F,
        frame: 10.0,
        rate: 1.0,
        grounded: false,
        stopped: false,
        facing: 1.0,
        hit_now: true,
    };
    let mut fair_state = None;
    followups::fair(&mut fair_state, true, true, fair, -1.0);
    assert!(fair_state.as_ref().unwrap().active());
    assert_eq!(
        followups::fair(&mut fair_state, true, false, fair, -1.0),
        ControllerOutput::default()
    );
    assert!(!fair_state.as_ref().unwrap().active());
    followups::fair(&mut fair_state, true, true, fair, -1.0);
    assert_eq!(
        followups::fair(&mut fair_state, false, true, fair, -1.0),
        ControllerOutput::default()
    );
    assert!(fair_state.is_none());

    let mut targets = [None; 8];
    targets[0] = Some(TripTarget {
        id: 2,
        status: status::SLIP,
        previous_status: status::WAIT,
    });
    let trip = TripContext {
        object: 1,
        kirby: true,
        status: status::ATTACK_LW3,
        motion: 1,
        frame: 1.0,
        grounded: true,
        facing: 1.0,
        stopped: false,
        targets,
    };
    let mut trip_state = None;
    assert_ne!(followups::trip(&mut trip_state, true, true, trip).owned, 0);
    assert!(trip_state.as_ref().unwrap().watching());
    assert_eq!(
        followups::trip(&mut trip_state, true, false, trip),
        ControllerOutput::default()
    );
    assert!(!trip_state.as_ref().unwrap().watching());
    assert_ne!(followups::trip(&mut trip_state, true, true, trip).owned, 0);
    assert_eq!(
        followups::trip(&mut trip_state, false, true, trip),
        ControllerOutput::default()
    );
    assert!(trip_state.is_none());
}
