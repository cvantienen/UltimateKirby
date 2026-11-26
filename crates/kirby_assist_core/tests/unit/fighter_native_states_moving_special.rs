use super::*;

#[test]
fn permits_ground_movement_but_not_guard_landing_or_airborne_lockout() {
    for status in [
        status::WAIT,
        status::WALK,
        status::DASH,
        status::RUN,
        status::TURN_RUN_BRAKE,
        status::JUMP_SQUAT,
        status::SQUAT_WAIT,
    ] {
        assert!(moving_down_special_input(status, situation::GROUND));
    }
    for status in [status::LANDING, status::GUARD, status::SPECIAL_LW] {
        assert!(!moving_down_special_input(status, situation::GROUND));
    }
    assert!(!moving_down_special_input(status::FALL, situation::AIR));
}
