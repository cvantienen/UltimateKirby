use super::*;

fn config() -> Config {
    Config {
        startup: 5.0,
        input_delay: 0.0,
        margin: 2.0,
        half_width: 5.0,
        height: 16.0,
        half_depth: 5.0,
    }
}

#[test]
fn delivered_counter_receipt_is_cancelled_before_forced_state_can_reserve_inputs() {
    use crate::native_states::{situation, status};
    for invalid in [status::DAMAGE, status::CAPTURE_WAIT, status::REBIRTH] {
        let mut counter = Counter::new();
        let mut c = context();
        assert_eq!(
            counter
                .sample(true, Some(config()), c, Some(threat(7)))
                .requested,
            MAPPER_B
        );
        counter.mapped();
        c.animation.status = invalid;
        c.animation.frame += 3.0;
        counter.poll_receipt(1);
        counter.poll_receipt(2);
        assert_eq!(
            counter.sample(true, Some(config()), c, None),
            ControllerOutput::NONE
        );
        assert!(!counter.active());
    }
    let mut counter = Counter::new();
    let mut c = context();
    counter.sample(true, Some(config()), c, Some(threat(7)));
    counter.mapped();
    c.situation = situation::CLIFF;
    assert_eq!(
        counter.sample(true, Some(config()), c, None),
        ControllerOutput::NONE
    );
    assert!(!counter.active());
}

#[test]
fn native_startup_at_frame_zero_is_acknowledged_even_during_hitstop() {
    let mut counter = Counter::new();
    let mut c = context();
    counter.sample(true, Some(config()), c, Some(threat(7)));
    counter.mapped();
    c.animation.status = crate::native_states::status::SPECIAL_LW;
    c.animation.frame = 0.0;
    c.stopped = true;
    assert_eq!(
        counter.sample(true, Some(config()), c, None),
        ControllerOutput::NONE
    );
    assert!(!counter.active());
    c = context();
    assert_eq!(
        counter.sample(true, Some(config()), c, Some(threat(7))),
        ControllerOutput::NONE
    );
}

#[test]
fn delayed_mapping_starts_receipt_deadline_at_delivery_not_proposal() {
    let mut counter = Counter::new();
    let mut c = context();
    for _ in 0..5 {
        assert_eq!(
            counter
                .sample(true, Some(config()), c, Some(threat(7)))
                .requested,
            MAPPER_B
        );
        c.animation.frame += 1.0;
    }
    counter.mapped();
    counter.poll_receipt(1);
    counter.poll_receipt(2);
    assert_eq!(
        counter.sample(true, Some(config()), c, None),
        ControllerOutput::NONE
    );
    assert!(counter.awaiting_receipt());
    c.animation.frame += 1.0;
    assert_eq!(
        counter.sample(true, Some(config()), c, None).owned,
        SEQUENCE_BUTTONS
    );
    assert!(!counter.awaiting_receipt());
}

#[test]
fn held_counter_requests_immediately_for_ready_movement_and_air_states() {
    use crate::native_states::{situation, status};
    let configured = Config::for_fighter(crate::fighter_kinds::LUCINA, Tuning::default()).unwrap();
    assert_eq!(configured.input_delay, 0.0);
    for (situation, states) in [
        (
            situation::GROUND,
            &[
                status::WAIT,
                status::WALK,
                status::DASH,
                status::RUN,
                status::TURN,
                status::SQUAT,
                status::LANDING,
            ][..],
        ),
        (
            situation::AIR,
            &[
                status::JUMP,
                status::JUMP_AERIAL,
                status::FALL,
                status::FALL_AERIAL,
            ][..],
        ),
    ] {
        for &status in states {
            let mut counter = Counter::new();
            let mut c = Context {
                situation,
                ..context()
            };
            c.animation.status = status;
            let t = Threat {
                in_frames: 0.0,
                ..threat(7)
            };
            let out = counter.sample(true, Some(configured), c, Some(t));
            assert_eq!(out.requested, MAPPER_B, "status {status}");
            assert_eq!(out.stick, Some([0.0, -1.0]));
            counter.mapped();
            c.animation.status = crate::native_states::status::SPECIAL_LW;
            c.animation.frame = 0.0;
            c.stopped = true;
            assert_eq!(
                counter.sample(true, Some(configured), c, Some(t)),
                ControllerOutput::NONE
            );
            assert!(!counter.active());
        }
    }
}

#[test]
fn failed_counter_deliveries_keep_retrying_same_projected_threat() {
    let mut counter = Counter::new();
    let mut c = context();
    for attempt in 0..4 {
        let hit = threat(7);
        assert_eq!(
            counter.sample(true, Some(config()), c, Some(hit)).requested,
            MAPPER_B
        );
        counter.mapped();
        for tick in 0..2 {
            counter.poll_receipt(attempt * 3 + tick);
        }
        for _ in 0..3 {
            c.animation.frame += 1.;
            counter.sample(true, Some(config()), c, Some(hit));
        }
        assert!(counter.release_pending);
        counter.mapped_release();
    }
}

#[test]
fn live_contact_requires_geometry_and_known_lifetime_for_prediction() {
    let hit = crate::autopilot_guard::Hitbox {
        start: [0., 8., 0.],
        end: [0., 8., 0.],
        radius: 1.,
        velocity: [0.; 2],
    };
    for status in [
        crate::native_states::status::ATTACK_S3,
        crate::native_states::status::ATTACK_AIR,
    ] {
        assert!(melee_status(status));
        assert_eq!(
            incoming_contact(
                config(),
                [0.; 3],
                [0.; 2],
                hit,
                contact_horizon(None, false),
                None
            ),
            Some(0.)
        );
    }
    let passing = crate::autopilot_guard::Hitbox {
        start: [0., 25., 0.],
        end: [0., 25., 0.],
        velocity: [3., 0.],
        ..hit
    };
    assert_eq!(
        incoming_contact(
            config(),
            [0.; 3],
            [0.; 2],
            passing,
            contact_horizon(Some(7.), false),
            None
        ),
        None
    );
    let approaching = crate::autopilot_guard::Hitbox {
        start: [10., 8., 0.],
        end: [10., 8., 0.],
        velocity: [-1., 0.],
        ..hit
    };
    assert_eq!(
        incoming_contact(
            config(),
            [0.; 3],
            [0.; 2],
            approaching,
            contact_horizon(None, false),
            None
        ),
        None
    );
    assert_eq!(
        incoming_contact(
            config(),
            [0.; 3],
            [0.; 2],
            approaching,
            contact_horizon(Some(7.), false),
            None
        ),
        Some(4.)
    );
    assert_eq!(
        incoming_contact(
            config(),
            [0.; 3],
            [0.; 2],
            approaching,
            contact_horizon(Some(7.), true),
            None
        ),
        None
    );
    assert_eq!(
        incoming_contact(
            config(),
            [0.; 3],
            [0.; 2],
            approaching,
            forecast_horizon(config(), None),
            None
        ),
        Some(4.)
    );
    assert_eq!(
        incoming_contact(
            config(),
            [0.; 3],
            [0.; 2],
            approaching,
            forecast_horizon(config(), Some(3.)),
            None
        ),
        None
    );
}

#[test]
fn falling_forecast_finds_contact_that_constant_velocity_misses() {
    let hit = crate::autopilot_guard::Hitbox {
        start: [0.0, 8.0, 0.0],
        end: [0.0, 8.0, 0.0],
        radius: 1.0,
        velocity: [0.0; 2],
    };
    let air = crate::gameplay::AirPhysics::new(1.0, 5.0);
    assert_eq!(
        active_melee_contact(config(), [0.0, 20.0, 0.0], [0.0; 2], hit, 7.0),
        None
    );
    let contact = incoming_contact(config(), [0.0, 20.0, 0.0], [0.0; 2], hit, 7.0, air).unwrap();
    assert!((4.6..4.8).contains(&contact));
    assert_eq!(
        incoming_contact(config(), [0.0, 20.0, 0.0], [0.0; 2], hit, 4.0, air),
        None
    );
    assert_eq!(
        incoming_contact(config(), [0.0, 20.0, 20.0], [0.0; 2], hit, 7.0, air),
        None
    );
    assert_eq!(
        incoming_contact(config(), [0.0, 20.0, 0.0], [3.0, 0.0], hit, 7.0, air),
        None
    );
}

#[test]
fn fast_fall_sweeps_between_samples_without_reducing_observed_speed() {
    let hit = crate::autopilot_guard::Hitbox {
        start: [0.0; 3],
        end: [0.0; 3],
        radius: 0.1,
        velocity: [0.0; 2],
    };
    let body = Config {
        height: 0.1,
        ..config()
    };
    let air = crate::gameplay::AirPhysics::new(0.1, 2.0);
    let contact = incoming_contact(body, [0.0, 3.0, 0.0], [0.0, -8.0], hit, 1.0, air).unwrap();
    assert!((contact - 2.9 / 8.0).abs() < 0.001);
    // The body crosses this tiny hitbox entirely inside one half-frame.
    assert!(contact < 0.5);
}

#[test]
fn relative_running_and_drift_forecasts_reject_retreat_and_invalid_physics() {
    let hit = crate::autopilot_guard::Hitbox {
        start: [20.0, 8.0, 0.0],
        end: [20.0, 8.0, 0.0],
        radius: 1.0,
        velocity: [0.0; 2],
    };
    assert_eq!(
        incoming_contact(config(), [0.0; 3], [0.0; 2], hit, 7.0, None),
        None
    );
    assert_eq!(
        incoming_contact(config(), [0.0; 3], [2.0, 0.0], hit, 7.0, None),
        Some(7.0)
    );
    assert_eq!(
        incoming_contact(config(), [0.0; 3], [-2.0, 0.0], hit, 7.0, None),
        None
    );
    assert_eq!(
        incoming_contact(
            config(),
            [0.0; 3],
            [2.0, 0.0],
            hit,
            7.0,
            Some(crate::gameplay::AirPhysics {
                gravity: f32::NAN,
                terminal_speed: 2.0
            })
        ),
        None
    );
    assert!(projectile_owner_near([0.0; 3], [40.0, 30.0, 0.0], 5.0));
    assert!(!projectile_owner_near([0.0; 3], [40.1, 0.0, 0.0], 5.0));
    assert!(!projectile_owner_near([0.0; 3], [81.0, 0.0, 0.0], 5.0));
}

#[test]
fn imminent_contact_requests_best_effort_counter_on_ground_or_in_air() {
    for status in [
        crate::native_states::status::ATTACK_S3,
        crate::native_states::status::ATTACK_AIR,
    ] {
        let threat = predicted_attack(
            config(),
            [0.0; 3],
            [0.0; 2],
            [12.0, 0.0, 0.0],
            [0.0; 2],
            5,
            7,
            crate::timing::motion_hash("attack_air_f"),
            status,
            12.0,
            1.0,
        )
        .unwrap();
        for situation in [
            crate::native_states::situation::GROUND,
            crate::native_states::situation::AIR,
        ] {
            let output = Counter::new().sample(
                true,
                Some(config()),
                Context {
                    kind: crate::fighter_kinds::LUCINA,
                    situation,
                    can_special: true,
                    stopped: false,
                    animation: AnimationSample {
                        object: 1,
                        status: if situation == crate::native_states::situation::GROUND {
                            crate::native_states::status::WAIT
                        } else {
                            crate::native_states::status::FALL
                        },
                        kind: 1,
                        frame: 1.0,
                        auxiliary: 0.0,
                    },
                },
                Some(threat),
            );
            assert_eq!(output.requested, MAPPER_B);
            assert_eq!(output.stick, Some([0.0, -1.0]));
        }
    }
}

#[test]
fn moving_with_direction_requests_down_special_and_releases_on_receipt() {
    let threat = Threat {
        id: 7,
        motion: 9,
        status: crate::native_states::status::ATTACK,
        frame: 3.0,
        in_frames: 6.0,
    };
    for status in [
        crate::native_states::status::WAIT,
        crate::native_states::status::WALK,
        crate::native_states::status::RUN,
        crate::native_states::status::TURN_RUN_BRAKE,
        crate::native_states::status::JUMP_SQUAT,
        crate::native_states::status::SQUAT_WAIT,
    ] {
        let mut counter = Counter::new();
        let output = counter.sample(
            true,
            Some(config()),
            Context {
                kind: crate::fighter_kinds::LUCINA,
                situation: crate::native_states::situation::GROUND,
                can_special: crate::native_states::moving_down_special_input(
                    status,
                    crate::native_states::situation::GROUND,
                ),
                stopped: false,
                animation: AnimationSample {
                    object: 1,
                    status,
                    kind: 1,
                    frame: 3.0,
                    auxiliary: 0.0,
                },
            },
            Some(threat),
        );
        assert_eq!(output.requested & MAPPER_B, MAPPER_B, "status {status}");
        assert_eq!(output.stick, Some([0.0, -1.0]));
        counter.mapped();
        assert!(counter.finished(crate::native_states::status::SPECIAL_LW, 1.0));
    }
}

#[test]
fn air_jump_and_fall_request_counter_but_guard_does_not() {
    let threat = Threat {
        id: 7,
        motion: 9,
        status: crate::native_states::status::ATTACK_AIR,
        frame: 3.0,
        in_frames: 6.0,
    };
    for status in [
        crate::native_states::status::JUMP,
        crate::native_states::status::JUMP_AERIAL,
        crate::native_states::status::FALL,
        crate::native_states::status::FALL_AERIAL,
    ] {
        let output = Counter::new().sample(
            true,
            Some(config()),
            Context {
                kind: crate::fighter_kinds::LUCINA,
                situation: crate::native_states::situation::AIR,
                can_special: true,
                stopped: false,
                animation: AnimationSample {
                    object: 1,
                    status,
                    kind: 1,
                    frame: 3.0,
                    auxiliary: 0.0,
                },
            },
            Some(threat),
        );
        assert_eq!(output.requested & MAPPER_B, MAPPER_B, "status {status}");
    }
    assert!(ready(
        crate::native_states::status::GUARD_OFF,
        crate::native_states::situation::GROUND
    ));
    assert!(ready(
        crate::native_states::status::GUARD,
        crate::native_states::situation::GROUND
    ));
}

#[test]
fn retries_once_after_mapped_input_fails_to_start_counter() {
    let mut counter = Counter::new();
    let threat = Threat {
        id: 7,
        motion: 9,
        status: crate::native_states::status::ATTACK,
        frame: 3.0,
        in_frames: 7.0,
    };
    let mut context = Context {
        kind: crate::fighter_kinds::LUCINA,
        situation: crate::native_states::situation::GROUND,
        can_special: true,
        stopped: false,
        animation: AnimationSample {
            object: 1,
            status: crate::native_states::status::RUN,
            kind: 1,
            frame: 1.0,
            auxiliary: 0.0,
        },
    };
    assert_ne!(
        counter
            .sample(true, Some(config()), context, Some(threat))
            .requested
            & MAPPER_B,
        0
    );
    counter.mapped();
    counter.poll_receipt(1);
    context.animation.frame = 2.0;
    assert_eq!(
        counter
            .sample(true, Some(config()), context, Some(threat))
            .requested,
        0
    );
    counter.poll_receipt(2);
    context.animation.frame = 3.0;
    assert_eq!(
        counter
            .sample(true, Some(config()), context, None)
            .requested,
        0
    );
    counter.mapped_release();
    context.animation.frame = 4.0;
    assert_ne!(
        counter
            .sample(true, Some(config()), context, Some(threat))
            .requested
            & MAPPER_B,
        0
    );
    counter.mapped();
    context.animation.frame = 4.0;
    assert_eq!(
        counter
            .sample(true, Some(config()), context, Some(threat))
            .requested,
        0
    );
    context.animation.frame = 20.0;
    assert_eq!(
        counter
            .sample(true, Some(config()), context, Some(threat))
            .requested,
        0
    );
    assert_eq!(
        counter
            .sample(true, Some(config()), context, Some(threat))
            .requested,
        0
    );
}

#[test]
fn a_temporarily_missing_hitbox_does_not_rearm_the_same_counter_attempt() {
    let mut counter = Counter::new();
    let threat = Threat {
        id: 7,
        motion: 9,
        status: crate::native_states::status::ATTACK,
        frame: 3.0,
        in_frames: 6.0,
    };
    let mut context = Context {
        kind: crate::fighter_kinds::LUCINA,
        situation: crate::native_states::situation::GROUND,
        can_special: true,
        stopped: false,
        animation: AnimationSample {
            object: 1,
            status: crate::native_states::status::WAIT,
            kind: 1,
            frame: 1.0,
            auxiliary: 0.0,
        },
    };
    assert_ne!(
        counter
            .sample(true, Some(config()), context, Some(threat))
            .requested
            & MAPPER_B,
        0
    );
    counter.mapped();
    context.animation.frame = 2.0;
    counter.poll_receipt(1);
    counter.poll_receipt(2);
    assert_eq!(
        counter
            .sample(true, Some(config()), context, None)
            .requested,
        0
    );
    context.animation.frame = 3.0;
    assert_eq!(
        counter
            .sample(true, Some(config()), context, None)
            .requested,
        0
    );
    counter.mapped_release();
    context.animation.frame = 4.0;
    assert_ne!(
        counter
            .sample(true, Some(config()), context, Some(threat))
            .requested
            & MAPPER_B,
        0
    );
    counter.mapped();
    context.animation.frame = 5.0;
    assert_eq!(
        counter
            .sample(true, Some(config()), context, None)
            .requested,
        0
    );
    context.animation.frame = 6.0;
    assert_eq!(
        counter
            .sample(true, Some(config()), context, Some(threat))
            .requested,
        0
    );
}

#[test]
fn never_retries_when_native_counter_has_started() {
    let mut counter = Counter::new();
    let threat = Threat {
        id: 7,
        motion: 9,
        status: crate::native_states::status::ATTACK,
        frame: 3.0,
        in_frames: 7.0,
    };
    let mut context = Context {
        kind: crate::fighter_kinds::LUCINA,
        situation: crate::native_states::situation::GROUND,
        can_special: true,
        stopped: false,
        animation: AnimationSample {
            object: 1,
            status: crate::native_states::status::RUN,
            kind: 1,
            frame: 1.0,
            auxiliary: 0.0,
        },
    };
    counter.sample(true, Some(config()), context, Some(threat));
    counter.mapped();
    context.animation.status = crate::native_states::status::SPECIAL_LW;
    for frame in 1..=4 {
        context.animation.frame = frame as f32;
        assert_eq!(
            counter
                .sample(true, Some(config()), context, None)
                .requested,
            0
        );
    }
    assert!(!counter.active());
    context.animation.status = crate::native_states::status::WAIT;
    context.animation.frame = 1.0;
    assert_eq!(
        counter
            .sample(true, Some(config()), context, Some(threat))
            .requested,
        0
    );
}

#[test]
fn close_slow_startup_requests_counter_at_animation_start() {
    use crate::native_states::status;
    for state in [status::ATTACK_AIR, status::SPECIAL_S, status::FINAL + 1] {
        let predict = |x, frame| predicted_attack(
            config(), [0.; 3], [0.; 2], [x, 0., 0.], [0.; 2],
            crate::fighter_kinds::YOSHI, 7, crate::timing::motion_hash("attack_air_f"),
            state, frame, 1.,
        );
        let hit = predict(12., 0.).expect("close startup must not wait for the contact horizon");
        let mut counter = Counter::new();
        assert_eq!(counter.sample(true, Some(config()), context(), Some(hit)).requested, MAPPER_B);
        assert!(predict(40., 0.).is_none());
        assert!(predict(12., 16.).is_none(), "missing active geometry is not startup");
        assert!(predict(12., 80.).is_none(), "recovery is not startup");
    }
    for state in [status::FINAL, status::GUARD, status::CAPTURE_WAIT] {
        assert!(predicted_attack(
            config(), [0.; 3], [0.; 2], [12., 0., 0.], [0.; 2],
            crate::fighter_kinds::YOSHI, 7, crate::timing::motion_hash("attack_air_f"),
            state, 0., 1.,
        ).is_none());
    }
}

#[test]
fn slow_startup_prediction_checks_relative_motion_at_the_active_window() {
    let predict = |own_velocity, enemy_velocity| predicted_attack(
        config(), [0.; 3], own_velocity, [12., 0., 0.], enemy_velocity,
        crate::fighter_kinds::YOSHI, 7, crate::timing::motion_hash("attack_air_f"),
        crate::native_states::status::ATTACK_AIR, 0., 1.,
    );
    assert!(predict([0.; 2], [0.; 2]).is_some());
    assert!(predict([0.; 2], [4., 0.]).is_none());
    assert!(predict([0.; 2], [-4., 0.]).is_none());
    assert!(predict([0.; 2], [0., 4.]).is_none());
    assert!(predict([4., 0.], [4., 0.]).is_some());
}

#[test]
fn delivered_counter_watchdog_requires_clean_release_with_frozen_animations() {
    let mut counter = Counter::new();
    let c = context();
    counter.sample(true, Some(config()), c, Some(threat(7)));
    counter.mapped();
    for tick in 0..24 {
        counter.poll_receipt(tick);
    }
    let release = counter.sample(true, Some(config()), c, Some(threat(7)));
    assert_eq!(release.requested, 0);
    assert_eq!(release.owned, SEQUENCE_BUTTONS);
    assert!(counter.active());
    counter.poll_receipt(24);
    assert_eq!(counter.sample(true, Some(config()), c, Some(threat(7))), release);
    counter.mapped_release();
    assert_eq!(counter.sample(true, Some(config()), c, Some(threat(7))).requested, MAPPER_B);
}

#[test]
fn close_startup_uses_contact_time_not_counter_activation_time() {
    let motion = crate::timing::motion_hash("attack_air_f");
    let predict = |x, vx| {
        predicted_attack(
            config(),
            [0.0; 3],
            [0.0; 2],
            [x, 0.0, 0.0],
            [vx, 0.0],
            5,
            7,
            motion,
            crate::native_states::status::ATTACK_AIR,
            14.0,
            1.0,
        )
    };
    // Overlap at contact must not be discarded because the attacker has
    // passed us by counter frame five.
    assert_eq!(predict(0.0, 6.0).unwrap().in_frames, 2.0);
    assert_eq!(predict(0.0, -6.0).unwrap().in_frames, 2.0);
    assert!(nearby_attacker(
        config(),
        [0.0; 3],
        [0.0; 2],
        [0.0; 3],
        [6.0, 0.0]
    ));
    assert!(predict(0.0, 12.0).is_none());
    // An approaching root can enter after the first active frame.
    let later = predict(40.0, -5.0).unwrap().in_frames;
    assert!((later - 4.6).abs() < 0.001);
    // Crossing before the window opens is not a contact.
    assert!(predict(-20.0, -6.0).is_none());
    assert!(!nearby_attacker(
        config(),
        [0.0; 3],
        [0.0; 2],
        [-30.0, 45.0, 0.0],
        [20.0, -5.0]
    ));
    for invalid in [
        Config {
            half_width: f32::NAN,
            ..config()
        },
        Config {
            input_delay: f32::INFINITY,
            ..config()
        },
    ] {
        assert!(predicted_attack(
            invalid,
            [0.0; 3],
            [0.0; 2],
            [0.0; 3],
            [0.0; 2],
            5,
            7,
            motion,
            crate::native_states::status::ATTACK_AIR,
            14.0,
            1.0
        )
        .is_none());
    }
}

#[test]
fn shield_and_tech_poses_rearm_the_next_identical_attack() {
    use crate::native_states::status;
    for pose in [
        status::GUARD,
        status::GUARD_OFF,
        status::PASSIVE,
        status::PASSIVE_FB,
    ] {
        let mut counter = Counter::new();
        let mut c = context();
        let hit = threat(7);
        assert_eq!(
            counter.sample(true, Some(config()), c, Some(hit)).requested,
            MAPPER_B
        );
        counter.mapped();
        assert!(counter.finished(status::SPECIAL_LW, 1.0));
        counter.observe_attacker_pose((hit.id, pose, crate::timing::motion_hash("wait"), 1.0));
        c.animation.frame += 1.0;
        assert_eq!(
            counter.sample(true, Some(config()), c, Some(hit)).requested,
            MAPPER_B,
            "pose {pose}"
        );
    }
}

#[test]
fn recovery_attacks_are_melee_but_passive_recovery_is_not() {
    use crate::native_states::status;
    for state in [status::DOWN_STAND_ATTACK, status::CLIFF_ATTACK] {
        assert!(melee_status(state), "attack status {state}");
    }
    for state in [
        status::GUARD,
        status::GUARD_OFF,
        status::DOWN_WAIT,
        status::DOWN_STAND,
        status::DOWN_STAND_FB,
        status::PASSIVE,
        status::PASSIVE_FB,
        status::PASSIVE_WALL,
        status::PASSIVE_WALL_JUMP,
    ] {
        assert!(!melee_status(state), "nonattack status {state}");
        assert!(predicted_attack(
            config(),
            [0.0; 3],
            [0.0; 2],
            [0.0; 3],
            [0.0; 2],
            5,
            7,
            crate::timing::motion_hash("attack_air_f"),
            state,
            12.0,
            1.0
        )
        .is_none());
    }
}

#[test]
fn prediction_rejects_unknown_and_missed_attacks() {
    let motion = crate::timing::motion_hash("attack_air_f");
    let predict = |position, velocity, motion| {
        predicted_attack(
            config(),
            [0.0; 3],
            [0.0; 2],
            position,
            velocity,
            5,
            7,
            motion,
            crate::native_states::status::ATTACK_AIR,
            12.0,
            1.0,
        )
    };
    assert!(predict([40.0, 0.0, 0.0], [0.0; 2], motion).is_none());
    assert!(predict([12.0, 0.0, 0.0], [12.0, 0.0], motion).is_none());
    assert!(predict([12.0, 0.0, 0.0], [0.0; 2], 0).is_none());
    assert!(predict([40.0, 0.0, 0.0], [-5.0, 0.0], motion).is_some());
    assert!(predicted_attack(
        config(),
        [0.0; 3],
        [0.0; 2],
        [8.0, 0.0, 0.0],
        [0.0; 2],
        5,
        7,
        motion,
        crate::native_states::status::ATTACK_AIR,
        19.0,
        1.0,
    )
    .is_none());
}

#[test]
fn startup_root_estimate_rejects_missed_ground_attack() {
    let motion = crate::timing::motion_hash("attack_air_f");
    let predict = |x, vx| {
        predicted_attack(
            config(),
            [0.0; 3],
            [0.0; 2],
            [x, 0.0, 0.0],
            [vx, 0.0],
            5,
            7,
            motion,
            crate::native_states::status::ATTACK_AIR,
            12.0,
            1.0,
        )
    };
    assert!(predict(12.0, 0.0).is_some());
    assert!(predict(40.0, -5.0).is_some());
    assert!(predict(22.0, 0.0).is_none());
}

#[test]
fn nearby_attacker_rejects_disjoint_remote_attacks() {
    let own = [0.0; 3];
    assert!(!nearby_attacker(
        config(),
        own,
        [0.0; 2],
        [45.0, 0.0, 0.0],
        [0.0; 2]
    ));
    assert!(!nearby_attacker(
        config(),
        own,
        [0.0; 2],
        [0.0, 45.0, 0.0],
        [0.0; 2]
    ));
    assert!(nearby_attacker(
        config(),
        own,
        [0.0; 2],
        [30.0, 0.0, 0.0],
        [-3.0, 0.0]
    ));
    assert!(nearby_attacker(
        config(),
        own,
        [0.0; 2],
        [6.0, 0.0, 0.0],
        [0.0; 2]
    ));
}

#[test]
fn damaging_special_startup_is_not_discarded_by_normal_attack_status_filter() {
    use crate::native_states::status;
    let t = predicted_attack(
        config(),
        [0.; 3],
        [0.; 2],
        [12., 0., 0.],
        [0.; 2],
        5,
        7,
        crate::timing::motion_hash("attack_air_f"),
        status::SPECIAL_S,
        12.,
        1.,
    );
    assert!(t.is_some());
    assert!(predicted_attack(
        config(),
        [0.; 3],
        [0.; 2],
        [12., 0., 0.],
        [0.; 2],
        5,
        7,
        crate::timing::motion_hash("wait"),
        status::SPECIAL_S,
        0.,
        1.
    )
    .is_none());
}

#[test]
fn movement_submits_input_without_overriding_action_lockouts() {
    use crate::native_states::{situation, status};
    for state in [
        status::WAIT,
        status::WALK,
        status::DASH,
        status::RUN,
        status::JUMP_SQUAT,
    ] {
        assert!(
            can_request(state, situation::GROUND, false)
                == crate::native_states::moving_down_special_input(state, situation::GROUND),
            "status {state}"
        );
        assert!(
            can_request(state, situation::GROUND, true),
            "status {state}"
        );
    }
    for state in [
        status::JUMP,
        status::JUMP_AERIAL,
        status::FALL,
        status::FALL_AERIAL,
    ] {
        assert!(can_request(state, situation::AIR, false));
        assert!(can_request(state, situation::AIR, true));
    }
    assert!(Config::for_fighter(crate::fighter_kinds::LUCINA, Tuning::default()).is_some());
    assert!(Config::for_fighter(crate::fighter_kinds::KIRBY, Tuning::default()).is_none());
    for state in [
        status::GUARD,
        status::LANDING,
        status::ATTACK,
        status::SPECIAL_LW,
    ] {
        assert!(!can_request(state, situation::GROUND, false));
    }
    assert!(can_request(status::GUARD, situation::GROUND, true));
    assert!(can_request(status::ATTACK, situation::GROUND, true));
    assert!(!can_request(status::DAMAGE, situation::GROUND, true));
    assert!(!can_request(status::CAPTURE_PULLED, situation::AIR, true));
}

#[test]
fn active_disjoint_is_not_discarded_by_attacker_root_screen() {
    let origin = [0.0; 3];
    let far_root = [45.0, 0.0, 0.0];
    assert!(!attacker_relevant(
        config(),
        origin,
        [0.0; 2],
        far_root,
        [0.0; 2],
        false
    ));
    assert!(attacker_relevant(
        config(),
        origin,
        [0.0; 2],
        far_root,
        [0.0; 2],
        true
    ));
    assert!(active_melee_threat(
        config(),
        origin,
        [0.0; 2],
        crate::autopilot_guard::Hitbox {
            start: [19.0, 8.0, 0.0],
            end: [19.0, 8.0, 0.0],
            radius: 2.0,
            velocity: [-2.0, 0.0],
        }
    ));
}

#[test]
fn active_melee_requires_hitbox_to_reach_grounded_counter() {
    let hitbox = |start: [f32; 3], velocity: [f32; 2]| crate::autopilot_guard::Hitbox {
        start,
        end: start,
        radius: 2.0,
        velocity,
    };
    assert!(!active_melee_threat(
        config(),
        [0.0; 3],
        [0.0; 2],
        hitbox([6.0, 8.0, 0.0], [0.0; 2]),
    ));
    assert!(!active_melee_threat(
        config(),
        [0.0; 3],
        [0.0; 2],
        hitbox([6.0, 25.0, 0.0], [0.0; 2]),
    ));
    assert!(!active_melee_threat(
        config(),
        [0.0; 3],
        [0.0; 2],
        hitbox([6.0, 25.0, 0.0], [-1.0, 0.0]),
    ));
    assert!(!active_melee_threat(
        config(),
        [0.0; 3],
        [0.0; 2],
        hitbox([20.0, 8.0, 0.0], [-1.0, 0.0]),
    ));
    assert!(!active_melee_threat(
        config(),
        [0.0; 3],
        [0.0; 2],
        hitbox([10.0, 8.0, 0.0], [-1.0, 0.0]),
    ));
    // An active hitbox that crosses the fighter before startup, then leaves,
    // cannot be countered by pressing down-special now.
    assert!(!active_melee_threat(
        config(),
        [0.0; 3],
        [0.0; 2],
        hitbox([6.0, 8.0, 0.0], [3.0, 0.0]),
    ));
    assert!(!active_melee_threat(
        config(),
        [0.0; 3],
        [0.0; 2],
        hitbox([40.0, 8.0, 0.0], [-3.0, 0.0]),
    ));
    assert!(active_melee_threat(
        config(),
        [0.0; 3],
        [0.0; 2],
        hitbox([25.0, 8.0, 0.0], [-3.0, 0.0]),
    ));
    assert!(!active_melee_threat(
        config(),
        [0.0; 3],
        [-3.0, 0.0],
        hitbox([25.0, 8.0, 0.0], [-3.0, 0.0]),
    ));
}

#[test]
fn contact_before_activation_still_requests_best_effort_down_b() {
    let threat = Threat {
        id: 7,
        motion: 9,
        status: crate::native_states::status::ATTACK_AIR,
        frame: 14.0,
        in_frames: 0.0,
    };
    let output = Counter::new().sample(
        true,
        Some(config()),
        Context {
            kind: crate::fighter_kinds::LUCINA,
            situation: crate::native_states::situation::AIR,
            can_special: true,
            stopped: false,
            animation: AnimationSample {
                object: 1,
                status: crate::native_states::status::FALL,
                kind: 1,
                frame: 1.0,
                auxiliary: 0.0,
            },
        },
        Some(threat),
    );
    assert_eq!(output.requested, MAPPER_B);
    assert_eq!(output.stick, Some([0.0, -1.0]));
}

#[test]
fn fresh_counter_can_buffer_from_airborne_damage_but_not_unavailable_or_captured_states() {
    use crate::native_states::{situation, status};
    for state in [status::DAMAGE_AIR, status::DAMAGE_FLY, status::DAMAGE_FALL] {
        let mut counter = Counter::new();
        let mut c = context();
        c.situation = situation::AIR;
        c.animation.status = state;
        c.can_special = false;
        assert_eq!(
            counter.sample(true, Some(config()), c, Some(threat(7))),
            ControllerOutput::NONE
        );
        c.can_special = true;
        assert_eq!(
            counter.sample(true, Some(config()), c, None),
            ControllerOutput::NONE
        );
        let out = counter.sample(true, Some(config()), c, Some(threat(7)));
        assert_eq!(out.requested, MAPPER_B);
        assert_eq!(out.stick, Some([0., -1.]));
        c.animation.status = status::CAPTURE_WAIT;
        assert_eq!(
            counter.sample(true, Some(config()), c, Some(threat(7))),
            ControllerOutput::NONE
        );
        assert!(!counter.active());
    }
}

#[test]
fn counter_aim_is_opt_in_supported_and_stops_before_retaliation() {
    use crate::native_states::status;
    let motion = crate::timing::motion_hash("special_lw");
    let aim = aim_confirmed(
        crate::fighter_kinds::LUCINA,
        status::SPECIAL_LW,
        motion,
        true,
        -12.,
    );
    assert_eq!(aim.stick, Some([-1., 0.]));
    assert_eq!(aim.requested, 0);
    for (kind, state, animation, enabled) in [
        (crate::fighter_kinds::LUCINA, status::WAIT, motion, true),
        (
            crate::fighter_kinds::LUCINA,
            status::SPECIAL_LW,
            motion,
            false,
        ),
        (
            crate::fighter_kinds::KIRBY,
            status::SPECIAL_LW,
            motion,
            true,
        ),
        (
            crate::fighter_kinds::LUCINA,
            status::SPECIAL_LW,
            crate::timing::motion_hash("special_lw_hit"),
            true,
        ),
    ] {
        assert_eq!(
            aim_confirmed(kind, state, animation, enabled, 12.),
            ControllerOutput::NONE
        );
    }
}

fn context() -> Context {
    Context {
        kind: crate::fighter_kinds::LUCINA,
        situation: crate::native_states::situation::GROUND,
        can_special: true,
        stopped: false,
        animation: AnimationSample {
            object: 1,
            status: crate::native_states::status::WAIT,
            kind: 1,
            frame: 1.,
            auxiliary: 0.,
        },
    }
}
fn threat(id: u32) -> Threat {
    Threat {
        id,
        motion: 9,
        status: crate::native_states::status::ATTACK,
        frame: 3.,
        in_frames: 6.,
    }
}

#[test]
fn repeated_identical_counter_threats_rearm_after_non_threatening_poses() {
    let mut counter = Counter::new();
    let mut c = context();
    let mut t = threat(7);
    counter.sample(true, Some(config()), c, Some(t));
    counter.mapped();
    assert!(counter.finished(crate::native_states::status::SPECIAL_LW, 1.));
    assert_eq!(
        counter.sample(true, Some(config()), c, Some(t)).requested,
        0
    );
    for cycle in 0..20 {
        counter.observe_attacker_pose((7, crate::native_states::status::WAIT, 1, 10.));
        counter.observe_attacker_pose((7, t.status, t.motion, 0.));
        // The next threatening sample is not necessarily earlier than the
        // preceding attack's threatening sample, especially online.
        t.frame = 3. + cycle as f32;
        c.animation.frame += 1.;
        assert_eq!(
            counter.sample(true, Some(config()), c, Some(t)).requested,
            MAPPER_B
        );
        counter.mapped();
        assert!(counter.finished(crate::native_states::status::SPECIAL_LW, 1.));
    }
}

#[test]
fn missing_geometry_and_target_switch_do_not_rearm_lingering_attacks() {
    let mut counter = Counter::new();
    let c = context();
    let t = threat(7);
    counter.sample(true, Some(config()), c, Some(t));
    counter.mapped();
    counter.finished(crate::native_states::status::SPECIAL_LW, 1.);
    counter.observe_attacker_pose((7, t.status, t.motion, 4.));
    counter.sample(true, Some(config()), c, None);
    assert_eq!(
        counter
            .sample(true, Some(config()), c, Some(Threat { frame: 4., ..t }))
            .requested,
        0
    );
    counter.sample(true, Some(config()), c, Some(threat(8)));
    counter.cancel();
    counter.observe_attacker_pose((7, crate::native_states::status::WAIT, 1, 1.));
    assert_eq!(
        counter.sample(true, Some(config()), c, Some(t)).requested,
        MAPPER_B
    );
}

#[test]
fn duplicate_receipts_require_a_mapped_release_before_the_single_retry() {
    let mut counter = Counter::new();
    let mut c = context();
    assert_ne!(
        counter
            .sample(true, Some(config()), c, Some(threat(7)))
            .requested,
        0
    );
    counter.mapped();
    counter.mapped();
    assert_eq!(counter.attempts, 1);
    counter.poll_receipt(1);
    counter.poll_receipt(2);
    c.animation.frame = 2.;
    counter.sample(true, Some(config()), c, Some(threat(7)));
    c.animation.frame = 3.;
    let release = counter.sample(true, Some(config()), c, Some(threat(7)));
    assert_eq!(release.requested, 0);
    assert_eq!(release.owned, SEQUENCE_BUTTONS);
    c.animation.frame = 4.;
    assert_eq!(
        counter
            .sample(true, Some(config()), c, Some(threat(7)))
            .requested,
        0
    );
    counter.mapped_release();
    assert_ne!(
        counter
            .sample(true, Some(config()), c, Some(threat(7)))
            .requested,
        0
    );
    counter.mapped();
    assert_eq!(counter.attempts, 2);
}

#[test]
fn configured_delay_defers_retry_and_late_native_start_cancels_release() {
    let mut counter = Counter::new();
    let mut c = context();
    let mut config = config();
    config.input_delay = 3.;
    let mut threat = threat(7);
    threat.in_frames = 9.;
    counter.sample(true, Some(config), c, Some(threat));
    counter.mapped();
    for tick in 1..=4 {
        counter.poll_receipt(tick);
        c.animation.frame += 1.;
        assert_eq!(counter.sample(true, Some(config), c, Some(threat)).owned, 0);
    }
    counter.poll_receipt(5);
    c.animation.frame += 1.;
    assert_eq!(
        counter.sample(true, Some(config), c, Some(threat)).owned,
        SEQUENCE_BUTTONS
    );
    c.animation.status = crate::native_states::status::SPECIAL_LW;
    c.animation.frame = 1.;
    assert_eq!(counter.sample(true, Some(config), c, None).owned, 0);
    assert!(!counter.active());
    assert_eq!(counter.attempts, u8::MAX);
}

#[test]
fn undelivered_frozen_proposals_expire_and_target_switches_do_not_reset_budget() {
    let mut counter = Counter::new();
    let c = context();
    assert_ne!(
        counter
            .sample(true, Some(config()), c, Some(threat(7)))
            .requested,
        0
    );
    for tick in 0..24 {
        counter.poll_receipt(tick);
        counter.poll_receipt(tick);
    }
    assert!(!counter.active());
    assert_ne!(
        counter
            .sample(true, Some(config()), c, Some(threat(7)))
            .requested,
        0
    );
    assert_ne!(
        counter
            .sample(true, Some(config()), c, Some(threat(8)))
            .requested,
        0
    );
    counter.cancel();
    assert_ne!(
        counter
            .sample(true, Some(config()), c, Some(threat(7)))
            .requested,
        0
    );
    let mut next = threat(7);
    next.frame = 1.;
    assert_ne!(
        counter
            .sample(true, Some(config()), c, Some(next))
            .requested,
        0
    );
}

#[test]
fn missing_invalid_hitstop_and_lost_ownership_cancel_pending_input() {
    for case in 0..5 {
        let mut counter = Counter::new();
        let mut c = context();
        counter.sample(true, Some(config()), c, Some(threat(7)));
        let mut t = threat(7);
        if case == 1 {
            t.frame = f32::NAN;
        }
        if case == 2 {
            c.stopped = true;
        }
        if case == 3 {
            c.can_special = false;
        }
        let out = counter.sample(case != 4, Some(config()), c, (case != 0).then_some(t));
        assert_eq!(out.requested, 0);
        assert!(!counter.active(), "case {case}");
    }
}

#[test]
fn active_window_lifetime_is_rate_scaled_and_never_extends_past_deactivation() {
    let kind = crate::fighter_kinds::MARIO;
    let motion = crate::timing::motion_hash("attack_air_n");
    let data = crate::fighter_data::fighter(kind)
        .unwrap()
        .move_for_motion(motion)
        .unwrap();
    let window = data.active.last().unwrap();
    let end = f32::from(window.last);
    assert_eq!(active_remaining(kind, motion, end - 1., 2.), Some(1.0));
    assert_eq!(active_remaining(kind, motion, end, 1.), Some(1.0));
    assert_eq!(active_remaining(kind, motion, end + 0.5, 1.), Some(0.5));
    assert_eq!(active_remaining(kind, motion, end + 1., 1.), None);
    assert_eq!(active_remaining(kind, motion, end, 0.), None);
    assert_eq!(active_remaining(kind, motion, f32::NAN, 1.), None);
}

#[test]
fn contact_lifetime_delay_and_relative_motion_are_not_double_counted() {
    let hit = crate::autopilot_guard::Hitbox {
        start: [19., 8., 0.],
        end: [19., 8., 0.],
        radius: 2.,
        velocity: [-2., 0.],
    };
    let contact = active_melee_contact(config(), [0.; 3], [0.; 2], hit, 7.).unwrap();
    assert!((contact - 6.).abs() < 0.0001);
    assert!(active_melee_contact(config(), [0.; 3], [0.; 2], hit, 5.).is_none());
    assert!(active_melee_contact(config(), [0.; 3], [-2., 0.], hit, 7.).is_none());
    let mut delayed = config();
    delayed.input_delay = 2.;
    assert!(!active_melee_threat(delayed, [0.; 3], [0.; 2], hit));
    delayed.input_delay = f32::NAN;
    assert!(active_melee_contact(delayed, [0.; 3], [0.; 2], hit, 7.).is_none());
}

#[test]
fn invalid_earlier_contact_does_not_erase_incoming_attack() {
    let best = Some(threat(7));
    for candidate in [
        Threat { motion: 0, in_frames: 0.0, ..threat(8) },
        Threat { frame: f32::NAN, in_frames: 0.0, ..threat(8) },
        Threat { in_frames: f32::NAN, ..threat(8) },
        Threat { in_frames: 8.0, ..threat(8) },
    ] {
        assert_eq!(prefer_incoming(config(), best, candidate), best);
    }
    let immediate = Threat { in_frames: 0.0, ..threat(8) };
    assert_eq!(prefer_incoming(config(), best, immediate), Some(immediate));
}

#[test]
fn projectile_generation_rearms_same_object_without_native_motion() {
    use crate::projectile_reflect::ProjectileKey;
    let key = ProjectileKey { id: 0x1000_0007, generation: 0 };
    let first = projectile_threat(key, 3.0);
    let mut counter = Counter::new();
    let mut c = context();
    assert_eq!(counter.sample(true, Some(config()), c, Some(first)).requested, MAPPER_B);
    counter.mapped();
    assert!(counter.finished(crate::native_states::status::SPECIAL_LW, 0.0));
    c.animation.frame += 1.0;
    assert_eq!(counter.sample(true, Some(config()), c, Some(first)), ControllerOutput::NONE);
    let next = projectile_threat(ProjectileKey { generation: 1, ..key }, 3.0);
    assert_eq!(counter.sample(true, Some(config()), c, Some(next)).requested, MAPPER_B);
}

#[test]
fn attacker_progress_releases_failed_counter_with_frozen_defender_animation() {
    let mut counter = Counter::new();
    let c = context();
    let mut t = threat(7);
    assert_eq!(counter.sample(true, Some(config()), c, Some(t)).requested, MAPPER_B);
    counter.mapped();
    for tick in 1..=2 {
        counter.poll_receipt(tick);
        t.frame += 1.0;
        let output = counter.sample(true, Some(config()), c, Some(t));
        assert_eq!(output.requested, 0);
    }
    assert!(counter.release_pending);
    counter.mapped_release();
    assert_eq!(counter.sample(true, Some(config()), c, Some(t)).requested, MAPPER_B);
}
