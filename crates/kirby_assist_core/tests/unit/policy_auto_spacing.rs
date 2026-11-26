use super::*;
const GROUND_WALK_STICK: f32 = 88. / 127.;

fn assert_ground_packet(out: Output, direction: [f32; 2], attack: bool) {
    use crate::native_input::*;
    assert_eq!(out.attack, attack);
    assert_eq!(out.controller.stick, Some(direction));
    assert_eq!(out.controller.requested, if attack { MAPPER_A } else { 0 });
    assert_eq!(out.right_stick, [0.; 2]);
    let raw = crate::manual_controls::ManualOverlay::default().apply_reaction_controls(
        ButtonTrace {
            held: MAPPER_LEFT_STICK_DIRECTIONS
                | MAPPER_RIGHT_STICK_DIRECTIONS
                | MAPPER_A
                | MAPPER_B,
            ..Default::default()
        },
        [-1., 1.],
        crate::manual_controls::TurboOutput {
            stick: out.controller.stick,
            ..Default::default()
        },
        MAPPER_LEFT_STICK_HORIZONTAL,
        out.controller.requested,
        out.controller.owned,
        true,
    );
    assert_eq!(raw.stick, direction);
    assert_eq!(
        raw.buttons.held & (MAPPER_LEFT_STICK_HORIZONTAL | MAPPER_RIGHT_STICK_DIRECTIONS),
        0
    );
    assert_eq!(
        raw.buttons.held & MAPPER_A,
        if attack { MAPPER_A } else { 0 }
    );
    let (buttons, left, right) = mapped_spacing_input(
        MAPPED_JUMP | MAPPED_SPECIAL | 0x80,
        out.attack,
        raw.stick,
        out.right_stick,
    );
    assert_eq!(left, direction);
    assert_eq!(right, [0.; 2]);
    assert_eq!(
        buttons,
        MAPPED_JUMP | if attack { MAPPED_ATTACK_HOLD_BITS } else { 0 }
    );
}

fn context() -> GameplayContext {
    GameplayContext {
        gameplay: true,
        can_control: true,
        can_act: true,
        can_air_attack: true,
        situation: situation::GROUND,
        status: status::WAIT,
        facing: 1.,
        ground_brake: Some(0.1),
        walk_stick: Some(GROUND_WALK_STICK),
        safe_ground: [true; 2],
        ..Default::default()
    }
}
fn config() -> Config {
    let a = AttackConfig {
        attack: Attack::Forward,
        startup: 8.,
        active_frames: 5.,
        reach: 20.,
        height: 9.,
        commitment: 33.,
    };
    Config {
        minimum_distance: 12.,
        engagement_range: 20.,
        reactive_retreat_distance: 12.,
        fighter_kind: -1,
        ground_speed: 1.,
        maximum_drift: 1.,
        minimum_drift: 0.1,
        ground_braking: true,
        air_braking: true,
        braking_distance: 10.,
        ground: a,
        aerial: AttackConfig {
            startup: 6.,
            active_frames: 3.,
            commitment: 30.,
            ..a
        },
    }
}
fn enemy(x: f32, vx: f32) -> Option<Opponent> {
    Some(Opponent {
        id: 1,
        position: [x, 0.],
        velocity: [vx, 0.],
        vulnerable: true,
        ..Default::default()
    })
}
#[test]
fn auto_spacing_direct_cap_survives_missing_thresholds_and_mapped_output() {
    for threshold in [None, Some(f32::NAN), Some(0.3), Some(0.8)] {
        for target in [None, enemy(40., 0.)] {
            let mut spacing = Spacing::default();
            let c = GameplayContext {
                walk_stick: threshold,
                ..context()
            };
            let out = spacing.sample(1, true, Config::default(), c, target);
            let expected = if threshold == Some(0.3) { 0.3 } else { 0.49 };
            assert_eq!(out.walk_cap, Some(expected));
            assert_eq!(out.movement_caps, Some([expected; 2]));
            for side in [-1., 1.] {
                let manual = manual_movement([side, -0.2], out.movement_caps.unwrap());
                assert_eq!(manual, [side * expected, -0.2]);
                assert_eq!(
                    mapped_walk_axis((side * 127.) as i8, expected),
                    crate::native_input::mapped_stick_axis(side * expected)
                );
            }
            assert_eq!(
                spacing.sample(2, false, Config::default(), c, target),
                Output::default()
            );
        }
    }
}

#[test]
fn auto_spacing_caps_existing_dash_without_forcing_neutral() {
    for state in [
        status::DASH,
        status::TURN_DASH,
        status::RUN,
        status::TURN_RUN,
        status::RUN_BRAKE,
        status::TURN_RUN_BRAKE,
    ] {
        for target in [None, enemy(40., 0.)] {
            let out = Spacing::default().sample(
                1,
                true,
                Config::default(),
                GameplayContext {
                    status: state,
                    ..context()
                },
                target,
            );
            assert_eq!(out.controller.stick, None);
            assert_eq!(out.walk_cap, Some(0.49));
            for x in [-1., -0.1, 0., 0.1, 1.] {
                assert_eq!(
                    manual_movement([x, 0.2], out.movement_caps.unwrap()),
                    [x.clamp(-0.49, 0.49), 0.2]
                );
            }
            assert!(!out.attack);
        }
    }
}

#[test]
fn auto_spacing_uses_configured_distance_and_retreats_before_incoming_crosses_it() {
    let cfg = Config {
        minimum_distance: 30.,
        engagement_range: 0.,
        ..config()
    };
    let mut spacing = Spacing::default();
    let out = spacing.sample(1, true, cfg, context(), enemy(24., -4.));
    assert_eq!(out.controller.stick, Some([-GROUND_WALK_STICK, 0.]));
    assert!(
        !Spacing::default()
            .sample(2, true, cfg, context(), enemy(20., 0.))
            .attack
    );
}

#[test]
fn auto_spacing_direct_ground_cap_limits_both_directions() {
    for ground_speed in [0., 0.4, 1.] {
        for side in [-1., 1.] {
            let cfg = Config {
                ground_speed,
                minimum_drift: 0.2,
                ..config()
            };
            let maximum = ground_speed.min(GROUND_WALK_STICK);
            let far = movement_caps(cfg, context(), enemy(side * 40., 0.));
            assert_eq!(far[usize::from(side > 0.)], maximum);
            let retreat = maximum;
            assert_eq!(far[usize::from(side < 0.)], retreat);
            let close = movement_caps(cfg, context(), enemy(side * 17., 0.));
            assert_eq!(close[usize::from(side > 0.)], maximum);
            for (distance, status) in [(10., status::WAIT), (10., status::ATTACK_S3)] {
                let out = Spacing::default().sample(
                    1,
                    true,
                    Config {
                        ground_speed,
                        ..Config::default()
                    },
                    GameplayContext {
                        status,
                        ..context()
                    },
                    enemy(side * distance, 0.),
                );
                assert_eq!(out.controller.stick, Some([-side * ground_speed.min(GROUND_WALK_STICK), 0.]));
            }
            for cap in close {
                assert!(crate::native_input::mapped_stick_axis(cap) <= 88);
            }
        }
    }
    let cfg = Config {
        maximum_drift: 0.05,
        minimum_drift: 0.2,
        ..config()
    };
    let c = GameplayContext {
        situation: situation::AIR,
        ..context()
    };
    assert_eq!(movement_caps(cfg, c, enemy(12., 0.)), [1.; 2]);
    assert_eq!(movement_caps(cfg, c, None), [1.; 2]);
    let mut s = Spacing::default();
    assert_eq!(
        s.sample(1, false, cfg, c, enemy(12., 0.)),
        Output::default()
    );
}

#[test]
fn auto_spacing_never_approaches_without_attacks_or_to_turn() {
    for side in [-1., 1.] {
        for speed in [0., 0.4, 1.] {
            let cfg = Config {
                ground_speed: speed,
                minimum_drift: 0.,
                ..Config::default()
            };
            let c = GameplayContext {
                facing: -side,
                ..context()
            };
            for distance in [40., 25., 20.] {
                let out = Spacing::default().sample(1, true, cfg, c, enemy(side * distance, 0.));
                assert_eq!(out.controller.stick, None);
                assert!(!out.attack);
                assert_eq!(out.controller.requested, 0);
                assert_eq!(out.controller.owned & SEQUENCE_JUMP_BUTTONS, 0);
                assert_ne!(
                    out.controller.owned & crate::native_input::MAPPER_LEFT_STICK_DIRECTIONS,
                    0
                );
            }
            let blocked = Spacing::default().sample(
                1,
                true,
                cfg,
                GameplayContext {
                    safe_ground: [false; 2],
                    ..c
                },
                enemy(side * 40., 0.),
            );
            assert_eq!(blocked.controller.stick, None);
            let retreat = Spacing::default().sample(1, true, cfg, c, enemy(side * 14., 0.));
            assert_eq!(retreat.controller.stick, Some([-side * speed.min(GROUND_WALK_STICK), 0.]));
            let mut attack_cfg = config();
            attack_cfg.ground_speed = 0.;
            let turn = Spacing::default().sample(1, true, attack_cfg, c, enemy(side * 18., 0.));
            assert_eq!(turn.controller.stick, None);
            assert!(!turn.attack);
        }
    }
}

#[test]
fn auto_spacing_engagement_boundary_cancels_retries_but_keeps_ground_approach() {
    let mut s = Spacing::default();
    let cfg = Config {
        minimum_distance: 15.,
        engagement_range: 20.,
        ..config()
    };
    let attack = s.sample(1, true, cfg, context(), enemy(20., 0.));
    assert!(attack.attack);
    s.mapped(1);
    let outside = s.sample(2, true, cfg, context(), enemy(20.01, -10.));
    assert_eq!(outside.controller.requested, 0);
    assert_eq!(outside.controller.stick, Some([-GROUND_WALK_STICK, 0.]));
    assert!(!outside.attack);
    assert!(!s.sample(3, true, cfg, context(), enemy(20., 0.)).attack);
    let missing = s.sample(4, true, cfg, context(), None);
    assert_eq!(missing.controller, ControllerOutput::NONE);
    assert_eq!(missing.walk_cap, context().walk_stick);
}

#[test]
fn auto_spacing_turns_before_ground_attack_and_waits_for_native_facing() {
    for attack in [Attack::Neutral, Attack::Forward, Attack::Up, Attack::Down] {
        for side in [-1., 1.] {
            let mut s = Spacing::default();
            let mut cfg = config();
            cfg.ground.attack = attack;
            let c = GameplayContext {
                facing: -side,
                ..context()
            };
            let turn = s.sample(1, true, cfg, c, enemy(side * 18., 0.));
            assert!(!turn.attack);
            assert_eq!(turn.controller.requested, 0);
            assert_eq!(turn.controller.stick, Some([side * 0.6, 0.]));
            let blocked = Spacing::default().sample(
                1,
                true,
                cfg,
                GameplayContext {
                    safe_ground: [false; 2],
                    ..c
                },
                enemy(side * 18., 0.),
            );
            assert_eq!(blocked.controller.stick, None);
            assert!(!blocked.attack);
            let out = s.sample(
                2,
                true,
                cfg,
                GameplayContext { facing: side, ..c },
                enemy(side * 18., 0.),
            );
            let direction = match attack {
                Attack::Neutral => [0.; 2],
                Attack::Forward => [side * 0.6, 0.],
                Attack::Up => [0., 0.6],
                Attack::Down => [0., -0.6],
                _ => unreachable!(),
            };
            assert_ground_packet(out, direction, true);
            s.mapped(2);
            let out = s.sample(
                3,
                true,
                cfg,
                GameplayContext {
                    facing: side,
                    status: status::ATTACK_S3,
                    can_act: false,
                    ..c
                },
                enemy(side * 18., 0.),
            );
            assert!(!out.attack);
            assert_eq!(out.controller.stick, None);
        }
    }
}

#[test]
fn auto_spacing_fair_uses_direction_on_every_attempt_then_immediately_drifts_back() {
    for side in [-1., 1.] {
        let mut s = Spacing::default();
        let c = GameplayContext {
            facing: side,
            situation: situation::AIR,
            status: status::FALL,
            ..context()
        };
        for tick in 1..=3 {
            let out = s.sample(tick, true, config(), c, enemy(side * 10., 0.));
            assert_eq!(out.controller.stick, Some([side, 0.]));
            assert_eq!(out.right_stick, [0.; 2]);
            assert_eq!(
                out.controller.requested,
                if tick == 2 { 0 } else { MAPPER_A }
            );
            if out.attack {
                s.mapped(tick);
            }
        }
        let out = s.sample(
            4,
            true,
            config(),
            GameplayContext {
                status: status::ATTACK_AIR,
                can_air_attack: false,
                motion_kind: crate::timing::MOTION_ATTACK_AIR_F,
                motion_frame: 1.,
                ..c
            },
            enemy(side * 10., 0.),
        );
        assert!(!out.attack);
        assert_eq!(out.controller.stick, Some([-side, 0.]));
    }
}

#[test]
fn auto_spacing_buffered_aerial_keeps_aim_until_motion_starts() {
    for (attack, motion, direction) in [
        (Attack::Forward, "attack_air_f", [1., 0.]),
        (Attack::Back, "attack_air_b", [1., 0.]),
        (Attack::Neutral, "attack_air_n", [0., 0.]),
        (Attack::Up, "attack_air_hi", [0., 1.]),
        (Attack::Down, "attack_air_lw", [0., -1.]),
    ] {
        for side in [-1., 1.] {
            let mut s = Spacing::default();
            let mut cfg = config();
            cfg.aerial.attack = attack;
            let c = GameplayContext {
                situation: situation::AIR,
                status: status::FALL,
                facing: if attack == Attack::Back { -side } else { side },
                ..context()
            };
            let aim = [direction[0] * side, direction[1]];
            assert!(s.sample(1, true, cfg, c, enemy(side * 10., 0.)).attack);
            s.mapped(1);
            for (tick, target, status, frame) in [
                (2, enemy(side * 100., 0.), status::FALL, 0.),
                (3, None, status::FALL, 0.),
                (4, enemy(side * 10., 0.), status::ATTACK_AIR, 0.),
            ] {
                let out = s.sample(
                    tick,
                    true,
                    cfg,
                    GameplayContext {
                        status,
                        motion_frame: frame,
                        motion_kind: crate::timing::motion_hash(motion),
                        ..c
                    },
                    target,
                );
                assert_eq!(out.controller.stick, Some(aim));
                assert!(!out.attack);
            }
            let out = s.sample(
                5,
                true,
                cfg,
                GameplayContext {
                    status: status::ATTACK_AIR,
                    motion_frame: 1.,
                    motion_kind: crate::timing::motion_hash(motion),
                    ..c
                },
                enemy(side * 10., 0.),
            );
            assert_eq!(out.controller.stick, Some([-side, 0.]));
            assert!(!out.attack);
        }
    }
}

#[test]
fn auto_spacing_attacks_from_braking_turning_and_double_jump_fall() {
    let mut tuning = crate::fighter_tuning::Tuning::default();
    tuning.set("auto_spacing_ground_move", "2").unwrap();
    tuning.set("auto_spacing_aerial_move", "2").unwrap();
    let cfg = tuning.auto_spacing(crate::fighter_kinds::LUCINA);
    for (situation, statuses) in [
        (
            situation::GROUND,
            &[status::WALK_BRAKE, status::TURN, status::SQUAT][..],
        ),
        (situation::AIR, &[status::FALL_AERIAL][..]),
    ] {
        for &status in statuses {
            let ready = attack_ready(status, situation, false);
            let c = GameplayContext {
                status,
                situation,
                can_act: ready,
                can_air_attack: ready,
                ..context()
            };
            let out = Spacing::default().sample(1, true, cfg, c, enemy(18., 0.));
            assert!(out.attack, "status {status}");
            assert_eq!(out.controller.requested, MAPPER_A);
        }
    }
    for (situation, status) in [
        (situation::GROUND, status::DASH),
        (situation::GROUND, status::LANDING_ATTACK_AIR),
        (situation::AIR, status::FALL_SPECIAL),
    ] {
        assert!(!attack_ready(status, situation, false));
    }
    assert!(attack_ready(
        status::LANDING_ATTACK_AIR,
        situation::GROUND,
        true
    ));
}

#[test]
fn auto_spacing_pending_attack_rechecks_facing_side_and_configuration() {
    for aerial in [false, true] {
        let c = GameplayContext {
            situation: if aerial {
                situation::AIR
            } else {
                situation::GROUND
            },
            status: if aerial { status::FALL } else { status::WAIT },
            ..context()
        };
        let mut s = Spacing::default();
        assert!(s.sample(1, true, config(), c, enemy(18., 0.)).attack);
        s.mapped(1);
        let out = s.sample(
            3,
            true,
            config(),
            GameplayContext { facing: -1., ..c },
            enemy(18., 0.),
        );
        assert!(!out.attack);
        assert_eq!(out.controller.requested, 0);

        let mut s = Spacing::default();
        assert!(s.sample(1, true, config(), c, enemy(18., 0.)).attack);
        s.mapped(1);
        let mut changed = config();
        if aerial {
            changed.aerial.attack = Attack::Disabled;
        } else {
            changed.ground.attack = Attack::Disabled;
        }
        let out = s.sample(3, true, changed, c, enemy(18., 0.));
        assert!(!out.attack);
        assert_eq!(out.controller.requested, 0);
    }
    let mut cfg = config();
    cfg.aerial.attack = Attack::Up;
    let c = GameplayContext {
        situation: situation::AIR,
        status: status::FALL,
        ..context()
    };
    let mut s = Spacing::default();
    assert!(s.sample(1, true, cfg, c, enemy(10., 0.)).attack);
    s.mapped(1);
    // Crossing cancels retries, but cannot rewrite an already buffered aerial.
    let out = s.sample(2, true, cfg, c, enemy(-10., 0.));
    assert!(!out.attack);
    assert_eq!(out.controller.stick, Some([0., 1.]));
}

#[test]
fn auto_spacing_crossing_enemy_cancels_the_old_attack_direction() {
    let mut s = Spacing::default();
    let c = GameplayContext {
        situation: situation::AIR,
        status: status::FALL,
        ..context()
    };
    assert!(s.sample(1, true, config(), c, enemy(10., 0.)).attack);
    s.mapped(1);
    for tick in 2..=3 {
        let out = s.sample(tick, true, config(), c, enemy(-10., 0.));
        assert!(!out.attack);
        assert_eq!(out.controller.requested, 0);
    }
}

#[test]
fn auto_spacing_native_walk_cap_is_maximal_and_preserves_manual_direction() {
    use crate::native_input::{fastest_walk_stick, mapped_stick_axis};
    for dash in (25..=120).map(|byte| byte as f32 / 120.) {
        let cap = fastest_walk_stick(0.2, dash).unwrap();
        let mapped = mapped_stick_axis(cap);
        assert!(f32::from(mapped) / 120. < dash);
        assert!(f32::from(mapped + 1) / 120. >= dash);
        for x in [-1., -0.1, 0., 0.1, 1.] {
            let result = manual_walk([x, 0.4], cap);
            assert_eq!(result, [x.clamp(-cap, cap), 0.4]);
        }
    }
    for (walk, dash) in [(0.2, f32::NAN), (0.2, 0.), (0.8, 0.5), (0.2, 1.1)] {
        assert_eq!(fastest_walk_stick(walk, dash), None);
    }
    let mut s = Spacing::default();
    let unknown = s.sample(
        1,
        true,
        config(),
        GameplayContext {
            walk_stick: None,
            ..context()
        },
        enemy(40., 0.),
    );
    assert_eq!(unknown.walk_cap, Some(0.49));
    assert_eq!(
        s.sample(2, false, config(), context(), enemy(40., 0.)),
        Output::default()
    );
}

#[test]
fn auto_spacing_tapers_before_boundary_and_retreats_after_breach() {
    let mut s = Spacing::default();
    let c = context();
    let out = s.sample(1, true, Config::default(), c, enemy(20., 0.));
    let approach = 0.49;
    assert_eq!(out.controller.stick, None);
    assert_eq!(out.movement_caps, Some([0.49, approach]));
    assert!(!out.attack);
    let out = s.sample(2, true, Config::default(), context(), enemy(-8., 0.));
    assert_eq!(out.controller.stick, Some([0.49, 0.]));
    assert!(!out.attack);
}
#[test]
fn auto_spacing_attacks_approaches_and_breaches_without_recovery_gap_veto() {
    let mut cfg = config();
    cfg.ground.commitment = 12.;
    let mut s = Spacing::default();
    assert!(s.sample(1, true, cfg, context(), enemy(24., -0.4)).attack);
    s.reset();
    assert!(
        s.sample(2, true, config(), context(), enemy(20., -1.))
            .attack
    );
    assert!(
        !s.sample(3, true, config(), context(), enemy(18., 1.))
            .attack
    );
    cfg.minimum_distance = 21.;
    let out = s.sample(4, true, cfg, context(), enemy(20., 0.));
    assert_eq!(out.controller.stick, Some([-GROUND_WALK_STICK, 0.]));
    assert!(!out.attack);
}
#[test]
fn auto_spacing_outside_range_stays_manual_and_retreat_never_dashes() {
    let mut s = Spacing::default();
    for side in [-1., 1.] {
        let c = GameplayContext {
            velocity: [side, 0.],
            ..context()
        };
        let out = s.sample(
            if side < 0. { 1 } else { 2 },
            true,
            config(),
            c,
            enemy(side * 80., 0.),
        );
        assert!(!out.attack);
        assert_eq!(out.controller.stick, None);
        assert_eq!(out.controller.requested, 0);
        assert_eq!(out.walk_cap, Some(GROUND_WALK_STICK));
    }
    let ground = s.sample(3, true, Config::default(), context(), enemy(10., 0.));
    assert_eq!(ground.controller.stick, Some([-0.49, 0.]));
    let air = s.sample(
        4,
        true,
        Config::default(),
        GameplayContext {
            situation: situation::AIR,
            status: status::FALL,
            ..context()
        },
        enemy(10., 0.),
    );
    assert_eq!(air.controller.stick, Some([-1., 0.]));
}

#[test]
fn auto_spacing_retries_until_native_startup_not_readiness_loss() {
    let mut s = Spacing::default();
    s.sample(1, true, config(), context(), enemy(13., 0.));
    s.mapped(1);
    let c = GameplayContext {
        status: status::TURN,
        can_act: false,
        ..context()
    };
    let release = s.sample(2, true, config(), c, enemy(13., 0.));
    assert_ground_packet(release, [0.; 2], false);
    let retry = s.sample(3, true, config(), c, enemy(13., 0.));
    assert_ground_packet(retry, [0.6, 0.], true);
    for tick in 4..=25 {
        let out = s.sample(tick, true, config(), c, enemy(13., 0.));
        if out.attack {
            s.mapped(tick);
        }
    }
    assert_eq!(s.output.controller.stick, None);
    assert_eq!(
        s.sample(25, false, config(), c, enemy(13., 0.)),
        Output::default()
    );
}

#[test]
fn auto_spacing_same_tick_native_changes_override_cached_attack() {
    for (c, target) in [
        (context(), None),
        (context(), enemy(40., 0.)),
        (
            GameplayContext {
                stopped: true,
                ..context()
            },
            enemy(18., 0.),
        ),
        (
            GameplayContext {
                status: status::ATTACK_S3,
                ..context()
            },
            enemy(18., 0.),
        ),
    ] {
        let mut s = Spacing::default();
        assert!(
            s.sample(1, true, config(), context(), enemy(18., 0.))
                .attack
        );
        s.mapped(1);
        let out = s.sample(1, true, config(), c, target);
        assert!(!out.attack);
        assert_eq!(out.controller.requested, 0);
    }
}

#[test]
fn auto_spacing_changed_target_cannot_inherit_a_pending_attack() {
    let mut s = Spacing::default();
    assert!(
        s.sample(1, true, config(), context(), enemy(18., 0.))
            .attack
    );
    s.mapped(1);
    assert!(
        !s.sample(2, true, config(), context(), enemy(18., 0.))
            .attack
    );
    let replacement = Opponent {
        id: 2,
        vulnerable: false,
        ..enemy(18., 0.).unwrap()
    };
    let out = s.sample(3, true, config(), context(), Some(replacement));
    assert!(!out.attack);
    assert_eq!(out.controller.requested, 0);
}

#[test]
fn auto_spacing_same_tick_rechecks_do_not_spend_retry_budget() {
    let mut s = Spacing::default();
    s.sample(1, true, config(), context(), enemy(18., 0.));
    s.mapped(1);
    let released = s.sample(2, true, config(), context(), enemy(18., 0.));
    for _ in 0..40 {
        assert_eq!(
            s.sample(2, true, config(), context(), enemy(18., 0.)),
            released
        );
    }
    assert!(
        s.sample(3, true, config(), context(), enemy(18., 0.))
            .attack
    );
}

#[test]
fn auto_spacing_release_is_immediate_even_on_same_tick_and_keeps_jump_manual() {
    let mut s = Spacing::default();
    let out = s.sample(1, true, config(), context(), enemy(18., 0.));
    assert!(out.attack);
    assert_eq!(out.controller.owned & SEQUENCE_JUMP_BUTTONS, 0);
    s.mapped(1);
    assert_eq!(
        s.sample(1, false, config(), context(), enemy(18., 0.)),
        Output::default()
    );
    assert!(
        s.sample(2, true, config(), context(), enemy(18., 0.))
            .attack
    );
}
#[test]
fn auto_spacing_repeats_only_after_delivery_and_native_recovery() {
    let mut s = Spacing::default();
    let mut c = context();
    assert!(s.sample(1, true, config(), c, enemy(18., 0.)).attack);
    s.mapped(1);
    let release = s.sample(2, true, config(), c, enemy(18., 0.));
    assert_ground_packet(release, [0.; 2], false);
    let retry = s.sample(3, true, config(), c, enemy(18., 0.));
    assert_ground_packet(retry, [0.6, 0.], true);
    s.mapped(3);
    c.status = status::ATTACK_S3;
    c.can_act = false;
    assert!(!s.sample(4, true, config(), c, enemy(13., 0.)).attack);
    assert_eq!(s.output.controller.stick, None);
    assert!(
        s.sample(5, true, config(), context(), enemy(18., 0.))
            .attack
    );
}
#[test]
fn auto_spacing_aerial_gravity_and_vertical_range_are_predicted() {
    let mut s = Spacing::default();
    let c = GameplayContext {
        situation: situation::AIR,
        status: status::FALL,
        velocity: [0., 3.],
        air_physics: crate::gameplay::AirPhysics::new(0.5, 3.),
        ..context()
    };
    let mut cfg = config();
    cfg.aerial.height = 3.;
    let target = Opponent {
        position: [18., 9.],
        vulnerable: true,
        ..Default::default()
    };
    assert!(s.sample(1, true, cfg, c, Some(target)).attack);
    assert!(
        !s.sample(
            2,
            true,
            cfg,
            GameplayContext {
                air_physics: None,
                ..c
            },
            Some(target)
        )
        .attack
    );
    assert!(
        !s.sample(
            3,
            true,
            cfg,
            c,
            Some(Opponent {
                vulnerable: false,
                ..target
            })
        )
        .attack
    );
}

#[test]
fn auto_spacing_ground_retreat_finishes_a_three_unit_step_without_facing_shuffle() {
    for side in [-1., 1.] {
        let cfg = Config {
            minimum_distance: 20.,
            braking_distance: 8.,
            ..config()
        };
        let mut s = Spacing::default();
        for (tick, x, enemy_x, walking) in [
            (1, 0., 19., true),
            (2, -0.1, 22., true),
            (3, -1., 22., true),
            (4, -2.9, 21., true),
            (5, -3., 21., false),
            (6, -3.2, 20.9, false),
        ] {
            let c = GameplayContext {
                position: [side * x, 0.],
                velocity: [-side * 0.2, 0.],
                facing: -side,
                ..context()
            };
            let out = s.sample(tick, true, cfg, c, enemy(side * enemy_x, 0.));
            assert_eq!(out.controller.stick, walking.then_some([-side * GROUND_WALK_STICK, 0.]));
            assert!(!out.attack);
            assert_eq!(out.controller.requested, 0);
        }
    }
}

#[test]
fn legacy_braking_settings_cannot_reduce_manual_walk_or_drift() {
    for airborne in [false, true] {
        for distance in [5., 15., 20., 100.] {
            let c = GameplayContext {
                situation: if airborne {
                    situation::AIR
                } else {
                    situation::GROUND
                },
                status: if airborne { status::FALL } else { status::WAIT },
                ..context()
            };
            let cfg = Config {
                ground_speed: GROUND_WALK_STICK,
                maximum_drift: 0.,
                minimum_drift: 0.,
                ground_braking: true,
                air_braking: true,
                braking_distance: 100.,
                ..Config::default()
            };
            let caps = movement_caps(cfg, c, enemy(distance, -10.));
            let maximum = if airborne { 1. } else { GROUND_WALK_STICK };
            assert_eq!(caps, [maximum; 2]);
            for x in [-1., -0.4, 0., 0.4, 1.] {
                assert_eq!(
                    manual_movement([x, 0.3], caps),
                    [x.clamp(-maximum, maximum), 0.3]
                );
            }
        }
    }
}

#[test]
fn auto_spacing_braking_never_reduces_outward_walk_or_air_drift() {
    for airborne in [false, true] {
        for side in [-1., 1.] {
            for braking in [false, true] {
                for distance in [5., 20., 21., 40.] {
                    let cfg = Config {
                        minimum_distance: 20.,
                        ground_speed: 0.49,
                        maximum_drift: 0.2,
                        minimum_drift: 0.,
                        ground_braking: braking,
                        air_braking: braking,
                        ..Config::default()
                    };
                    let c = GameplayContext {
                        situation: if airborne {
                            situation::AIR
                        } else {
                            situation::GROUND
                        },
                        status: if airborne { status::FALL } else { status::WAIT },
                        velocity: [-side * 2., 0.],
                        facing: side,
                        ..context()
                    };
                    let out =
                        Spacing::default().sample(1, true, cfg, c, enemy(side * distance, -side));
                    let full = if airborne { 1. } else { 0.49 };
                    for magnitude in [0.1, 0.3, 1.] {
                        assert_eq!(
                            manual_movement([-side * magnitude, 0.4], out.movement_caps.unwrap()),
                            [-side * magnitude.min(full), 0.4]
                        );
                    }
                    if let Some(stick) = out.controller.stick {
                        assert_eq!(stick, [-side * full, 0.]);
                    }
                    assert!(!out.attack);
                }
            }
        }
    }
}

#[test]
fn auto_spacing_retreat_releases_on_target_loss_and_respects_ground_support() {
    let cfg = Config::default();
    let mut spacing = Spacing::default();
    let start = spacing.sample(1, true, cfg, context(), enemy(10., 0.));
    assert_eq!(start.controller.stick, Some([-0.49, 0.]));
    let blocked = spacing.sample(
        2,
        true,
        cfg,
        GameplayContext {
            safe_ground: [false; 2],
            ..context()
        },
        enemy(10., 0.),
    );
    assert_eq!(blocked.controller.stick, Some([0.; 2]));
    assert!(!blocked.attack);
    let missing = spacing.sample(3, true, cfg, context(), None);
    assert_eq!(missing.controller.stick, None);
    let outside = spacing.sample(4, true, cfg, context(), enemy(20., 0.));
    assert_eq!(outside.controller.stick, None);
    assert_eq!(
        spacing.sample(4, false, cfg, context(), enemy(10., 0.)),
        Output::default()
    );
}

#[test]
fn reactive_retreat_latches_all_status_entries_but_not_initial_idle() {
    let cfg = Config { reactive_retreat_distance: 34., ..config() };
    for airborne in [false, true] {
        let c = GameplayContext {
            situation: if airborne { situation::AIR } else { situation::GROUND },
            status: if airborne { status::FALL } else { status::WAIT },
            can_act: false,
            can_air_attack: false,
            ..context()
        };
        for state in [status::WALK, status::LANDING, status::ATTACK, status::WAIT] {
            let mut s = Spacing::default();
            let idle = Opponent { status: if state == status::WAIT { status::WALK } else { status::WAIT }, ..enemy(25., 0.).unwrap() };
            assert_eq!(s.sample(1, true, cfg, c, Some(idle)).controller.stick, None);
            let transitioned = Opponent { status: state, ..idle };
            let expected = [-if airborne { 1. } else { GROUND_WALK_STICK }, 0.];
            assert_eq!(s.sample(2, true, cfg, c, Some(transitioned)).controller.stick, Some(expected));
            assert_eq!(s.sample(3, true, cfg, c, Some(Opponent { position: [33., 0.], ..transitioned })).controller.stick, Some(expected));
            assert_eq!(s.sample(4, true, cfg, c, Some(Opponent { position: [34., 0.], ..transitioned })).controller.stick, None);
            s.sample(5, true, cfg, c, Some(Opponent { velocity: [-0.1, 0.], ..transitioned }));
            assert_eq!(s.sample(6, true, cfg, c, None).controller.stick, None);
            assert_eq!(s.sample(7, true, cfg, c, Some(idle)).controller.stick, None);
            s.sample(8, true, cfg, c, Some(transitioned));
            s.sample(8, false, cfg, c, Some(transitioned));
            assert_eq!(s.sample(9, true, cfg, c, Some(transitioned)).controller.stick, None);
        }
    }
}

#[test]
fn reactive_retreat_tracks_motion_changes_and_restarts() {
    let cfg = Config { reactive_retreat_distance: 34., ..Config::default() };
    let mut e = enemy(25., 0.).unwrap();
    e.motion = Some(crate::gameplay::MotionTiming { kind: 1, frame: 10., rate: 1., remaining: None });
    let mut s = Spacing::default();
    assert_eq!(s.sample(1, true, cfg, context(), Some(e)).controller.stick, None);
    e.motion.as_mut().unwrap().frame = 11.;
    assert_eq!(s.sample(2, true, cfg, context(), Some(e)).controller.stick, None);
    e.motion.as_mut().unwrap().frame = 0.;
    assert_eq!(s.sample(3, true, cfg, context(), Some(e)).controller.stick, Some([-0.49, 0.]));
    let replacement = Opponent { id: 2, ..e };
    assert_eq!(s.sample(4, true, cfg, context(), Some(replacement)).controller.stick, None);
}

#[test]
fn position_selection_preserves_lucina_horizontal_and_uses_catalog_verticals() {
    let cfg = Config {
        fighter_kind: crate::fighter_kinds::LUCINA,
        minimum_distance: 24., reactive_retreat_distance: 34., ground_speed: 0.49,
        ground_braking: false, minimum_drift: 0.11,
        ground: AttackConfig { startup: 10., active_frames: 5., reach: 28., height: 10., ..config().ground },
        aerial: AttackConfig { startup: 5., active_frames: 5., reach: 28., height: 20.5, ..config().aerial },
        ..config()
    };
    let ground = GameplayContext { walk_stick: Some(0.48999986), velocity: [-0.1, 0.], ..context() };
    let mut s = Spacing::default();
    assert!(s.sample(1, true, cfg, ground, enemy(25., 0.)).attack);
    assert_eq!(s.pending_attack, cfg.ground);
    let air = GameplayContext { situation: situation::AIR, status: status::FALL, ..context() };
    for (y, expected) in [(0., Attack::Forward), (15., Attack::Up), (-15., Attack::Down)] {
        let e = Opponent { position: [8., y], ..enemy(8., 0.).unwrap() };
        let selected = select_attack(cfg, air, e, 1.);
        assert_eq!(selected.attack, expected);
        if y == 0. { assert_eq!(selected, cfg.aerial); }
        let mut spacing = Spacing::default();
        let out = spacing.sample(1, true, cfg, air, Some(e));
        assert!(out.attack, "height {y}");
        spacing.mapped(1);
        let shifted = Opponent { position: [8., y + y.signum() * 0.1], ..e };
        spacing.sample(2, true, cfg, air, Some(shifted));
        assert_eq!(spacing.pending_attack, selected);
    }
    let behind = GameplayContext { facing: -1., ..air };
    assert_eq!(select_attack(cfg, behind, enemy(8., 0.).unwrap(), 1.).attack, Attack::Back);
    let above = Opponent { position: [8., 8.], ..enemy(8., 0.).unwrap() };
    assert_eq!(select_attack(cfg, ground, above, 1.).attack, Attack::Up);
    assert!(!Spacing::default().sample(1, true, cfg, ground, Some(above)).attack);
}

#[test]
fn auto_spacing_manual_actions_bypass_movement_caps_but_jump_does_not() {
    use crate::native_input::*;
    for button in [
        MAPPER_A,
        MAPPER_B,
        MAPPER_R,
        MAPPER_ZR,
        MAPPER_RIGHT_STICK_DOWN,
    ] {
        assert!(manual_action(button, [0.; 2]));
    }
    assert!(manual_action(0, [1., 0.]));
    assert!(!manual_action(
        SEQUENCE_JUMP_BUTTONS | MAPPER_STICK_CLICKS | MAPPER_DPAD_BUTTONS,
        [0.; 2]
    ));
    assert!(!manual_action(0, [0.; 2]));
}

#[test]
fn auto_spacing_ownership_and_mapped_attack_preserve_jump() {
    use crate::action_owner::{ActionOwnership, Intent, Owner};
    use crate::native_input::*;
    let mut ownership = ActionOwnership::new();
    let intent = Intent {
        owner: Owner::Spacing,
        present: true,
        committed: true,
    };
    ownership.set_automatic_suspended(true);
    assert_eq!(ownership.select(true, &[intent]), Some(Owner::Spacing));
    assert!(!ownership.allows(Owner::Ledge));
    assert!(!ownership.allows(Owner::RollRead));
    let safety = Intent {
        owner: Owner::Safety,
        present: true,
        committed: false,
    };
    assert_eq!(
        ownership.select(true, &[intent, safety]),
        Some(Owner::Safety)
    );
    ownership.release(Owner::Spacing);
    assert_eq!(ownership.select(true, &[]), None);
    let physical = MAPPED_JUMP | MAPPED_SPECIAL | MAPPED_GUARD | MAPPED_CATCH;
    let attack = mapped_spacing_attack(physical, true, true);
    assert_ne!(attack & MAPPED_JUMP, 0);
    assert_ne!(attack & MAPPED_ATTACK, 0);
    assert_eq!(attack & (MAPPED_SPECIAL | MAPPED_GUARD | MAPPED_CATCH), 0);
    assert_eq!(mapped_spacing_attack(attack, false, false), MAPPED_JUMP);
}

#[test]
fn auto_spacing_air_drift_facing_edges_and_invalid_observations() {
    let mut s = Spacing::default();
    let c = GameplayContext {
        situation: situation::AIR,
        status: status::FALL,
        ..context()
    };
    let out = s.sample(1, true, config(), c, enemy(13., 0.));
    assert_eq!(out.controller.stick, Some([1., 0.]));
    assert_eq!(out.right_stick, [0.; 2]);
    assert!(
        !s.sample(
            2,
            true,
            config(),
            GameplayContext { facing: -1., ..c },
            enemy(13., 0.)
        )
        .attack
    );
    let out = s.sample(
        3,
        true,
        config(),
        GameplayContext {
            safe_ground: [false; 2],
            ..context()
        },
        enemy(8., 0.),
    );
    assert_eq!(out.controller.stick, Some([0.; 2]));
    assert!(!out.attack);
    assert_eq!(
        s.sample(
            4,
            true,
            config(),
            GameplayContext {
                position: [f32::NAN, 0.],
                ..c
            },
            enemy(13., 0.)
        ),
        Output::default()
    );
    let out = s.sample(5, true, config(), c, None);
    assert_eq!(out.right_stick, [0.; 2]);
    assert!(!out.attack);
}
