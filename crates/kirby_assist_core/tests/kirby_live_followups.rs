#[path = "../../../src/live/fighters/kirby/followups.rs"]
mod followups;

use kirby_assist_core::{
    gameplay::ControllerOutput,
    kirby::{FairContext, TripContext, TripTarget},
    native_states::status,
    timing::{motion_hash, MOTION_ATTACK_AIR_F},
};

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
