use super::*;

fn context(status: i32, frame: f32) -> Context {
    Context {
        kind: GAMEWATCH,
        situation: crate::native_states::situation::GROUND,
        facing: 1.0,
        can_special: status == crate::native_states::status::WAIT,
        gauge: 0.0,
        stopped: false,
        animation: AnimationSample {
            object: 1,
            status,
            kind: if status == crate::native_states::status::WAIT {
                1
            } else {
                2
            },
            frame,
            auxiliary: 0.0,
        },
    }
}

fn config() -> Config {
    Config {
        startup: 6.0,
        margin: 4.0,
        max_hold: 45.0,
        half_width: 5.0,
        height: 16.0,
        half_depth: 5.0,
    }
}

#[test]
fn projectile_requests_down_special_and_keeps_receipt_when_it_disappears() {
    let mut reflector = Reflector::new();
    let threat = Threat {
        id: 0x10000005,
        generation: 1,
        in_frames: 8.0,
        direction: 1.0,
    };
    let initial = reflector.sample(
        true,
        Some(config()),
        context(crate::native_states::status::WAIT, 1.0),
        Some(threat),
    );
    assert_eq!(initial.requested & MAPPER_B, MAPPER_B);
    assert_eq!(initial.stick, Some([0.0, -1.0]));
    reflector.mapped(1);
    assert!(reflector.awaiting_receipt());
    let pending = reflector.sample(
        true,
        Some(config()),
        context(crate::native_states::status::WAIT, 2.0),
        None,
    );
    assert_eq!(pending.requested, 0);
    assert!(reflector.awaiting_receipt());
    reflector.sample(
        true,
        Some(config()),
        context(crate::native_states::status::SPECIAL_LW, 1.0),
        None,
    );
    assert!(!reflector.active());
}

#[test]
fn movement_requests_oil_panic_when_transition_term_lags() {
    let threat = Threat {
        id: 0x10000005,
        generation: 1,
        in_frames: 8.0,
        direction: 1.0,
    };
    for status in [
        crate::native_states::status::WALK,
        crate::native_states::status::RUN,
        crate::native_states::status::TURN_RUN_BRAKE,
        crate::native_states::status::JUMP_SQUAT,
        crate::native_states::status::SQUAT_WAIT,
    ] {
        let mut reflector = Reflector::new();
        let mut moving = context(status, 2.0);
        moving.can_special =
            crate::native_states::moving_down_special_input(status, moving.situation);
        let output = reflector.sample(true, Some(config()), moving, Some(threat));
        assert_eq!(output.requested & MAPPER_B, MAPPER_B, "status {status}");
        assert_eq!(output.stick, Some([0.0, -1.0]));
        reflector.mapped(1);
        assert!(reflector.awaiting_receipt());
        reflector.sample(
            true,
            Some(config()),
            context(crate::native_states::status::SPECIAL_LW, 1.0),
            None,
        );
        assert!(!reflector.active());
    }
}

#[test]
fn air_jump_and_fall_request_oil_panic_but_guard_does_not() {
    let threat = Threat {
        id: 7,
        generation: 1,
        in_frames: 8.0,
        direction: 1.0,
    };
    for status in [
        crate::native_states::status::JUMP,
        crate::native_states::status::JUMP_AERIAL,
        crate::native_states::status::FALL,
        crate::native_states::status::FALL_AERIAL,
    ] {
        let mut airborne = context(status, 2.0);
        airborne.situation = crate::native_states::situation::AIR;
        airborne.can_special = true;
        let output = Reflector::new().sample(true, Some(config()), airborne, Some(threat));
        assert_eq!(output.requested & MAPPER_B, MAPPER_B, "status {status}");
    }
    assert!(movable(
        crate::native_states::status::GUARD_OFF,
        crate::native_states::situation::GROUND
    ));
    assert!(!movable(
        crate::native_states::status::GUARD,
        crate::native_states::situation::GROUND
    ));
}

#[test]
fn approaching_projectile_geometry_requests_oil_panic() {
    let projectile = Hitbox {
        start: [28.0, 8.0, 0.0],
        end: [28.0, 8.0, 0.0],
        radius: 2.0,
        velocity: [-3.0, 0.0],
    };
    let threat = predict(
        config(),
        [0.0; 3],
        [0.0; 2],
        1.0,
        0x10000005,
        projectile,
        true,
        false,
    )
    .unwrap();
    assert!((config().startup..=config().startup + config().margin).contains(&threat.in_frames));
    let output = Reflector::new().sample(
        true,
        Some(config()),
        context(crate::native_states::status::WAIT, 1.0),
        Some(threat),
    );
    assert_eq!(output.requested & MAPPER_B, MAPPER_B);
}

#[test]
fn full_bucket_blocks_input_but_late_projectile_requests_immediately() {
    let threat = Threat {
        id: 0x10000005,
        generation: 1,
        in_frames: 2.0,
        direction: 1.0,
    };
    let mut reflector = Reflector::new();
    assert_eq!(
        reflector
            .sample(
                true,
                Some(config()),
                context(crate::native_states::status::WAIT, 1.0),
                Some(threat)
            )
            .requested
            & MAPPER_B,
        MAPPER_B
    );
    let mut full = context(crate::native_states::status::WAIT, 1.0);
    full.gauge = 3.0;
    assert_eq!(
        reflector
            .sample(
                true,
                Some(config()),
                full,
                Some(Threat {
                    in_frames: 8.0,
                    ..threat
                })
            )
            .requested,
        0
    );
}
