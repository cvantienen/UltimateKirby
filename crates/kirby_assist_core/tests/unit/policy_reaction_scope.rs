use super::*;
use crate::native_states::{situation, status};

#[test]
fn own_floor_recovery_blocks_follow_but_enemy_recovery_stays_observable() {
    for status in [
        status::PASSIVE,
        status::PASSIVE_FB,
        status::PASSIVE_WALL,
        status::PASSIVE_WALL_JUMP,
        status::PASSIVE_CEIL,
        status::DOWN_STAND,
        status::DOWN_STAND_FB,
        status::DOWN_STAND_ATTACK,
    ] {
        assert!(automatic_target_available(status), "enemy status {status}");
        assert!(
            !AutomaticObservationScope::new(
                false,
                status,
                situation::GROUND,
                false,
                false,
                false,
                true
            )
            .observe_targets(),
            "own status {status}"
        );
        assert!(!automatic_up_b_receipt_status(
            status,
            situation::GROUND,
            false
        ));
    }
    assert!(
        AutomaticObservationScope::new(
            false,
            status::ATTACK,
            situation::GROUND,
            false,
            false,
            false,
            true
        )
        .follow
    );
}

#[test]
fn grounded_automatic_modes_recognize_transitions_without_admitting_lockouts() {
    for status in [
        status::WAIT,
        status::WALK,
        status::TURN,
        status::TURN_DASH,
        status::RUN,
        status::TURN_RUN_BRAKE,
        status::JUMP_SQUAT,
        status::SQUAT_RV,
        status::LANDING,
        status::LANDING_LIGHT,
        status::GUARD_ON,
        status::GUARD,
        status::OTTOTTO_WAIT,
    ] {
        assert!(
            automatic_start_allowed(status, situation::GROUND, false, false),
            "status {status}"
        );
        assert!(crate::spot_dodge::ground_status(status), "status {status}");
    }
    for status in [
        status::LANDING_ATTACK_AIR,
        status::GUARD_DAMAGE,
        status::ATTACK,
        status::ESCAPE,
        status::FALL_SPECIAL,
    ] {
        assert!(!automatic_start_status(status), "status {status}");
        assert!(!crate::spot_dodge::ground_status(status), "status {status}");
    }
    assert!(!automatic_start_allowed(
        status::FALL,
        situation::AIR,
        false,
        false
    ));
    assert!(!automatic_start_allowed(
        status::WAIT,
        situation::GROUND,
        true,
        false
    ));
    assert!(!automatic_start_allowed(
        status::WAIT,
        situation::GROUND,
        false,
        true
    ));
}
