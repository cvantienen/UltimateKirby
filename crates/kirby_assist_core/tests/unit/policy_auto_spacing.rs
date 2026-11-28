use super::*;

#[test]
fn attacks_ignore_facing_and_retired_hit_estimates_in_both_situations() {
    for airborne in [false, true] {
        for facing in [-1., 1., 0., f32::NAN] {
            for side in [-1., 1.] {
                for attack in [Attack::Forward, Attack::Back, Attack::Up, Attack::Down, Attack::Neutral] {
                    let move_config = AttackConfig { attack, ..Default::default() };
                    let cfg = Config { ground: move_config, aerial: move_config, ..Config::default() };
                    let c = GameplayContext {
                        situation: if airborne { situation::AIR } else { situation::GROUND },
                        status: if airborne { status::FALL } else { status::WAIT },
                        facing, walk_stick: None, ..context()
                    };
                    let target = Opponent { position: [side * 20., 100.],
                        velocity: [side * 10., 10.], vulnerable: false,
                        status: status::GUARD, ..Default::default() };
                    let out = Spacing::default().sample_threat(1, true, cfg, c, Some(target), true);
                    assert!(out.attack);
                    assert_eq!(out.controller.stick, Some([0.; 2]));
                    assert_eq!(out.right_stick, match attack {
                        Attack::Forward | Attack::Back => [side, 0.],
                        Attack::Up => [0., 1.], Attack::Down => [0., -1.],
                        _ => [0.; 2],
                    });
                }
            }
        }
    }
}

#[test]
fn aerial_retries_release_cstick_and_resume_immediately_after_native_recovery() {
    let c = GameplayContext { situation: situation::AIR, status: status::FALL, ..context() };
    let cfg = Config { aerial: AttackConfig { attack: Attack::Back, ..Default::default() },
        ..Config::default() };
    let mut spacing = Spacing::default();
    for tick in 1..=27 {
        let out = spacing.sample(tick, true, cfg, c, enemy(-18., 0.));
        assert_eq!(out.attack, tick % 2 == 1);
        assert_eq!(out.right_stick, if out.attack { [-1., 0.] } else { [0.; 2] });
        if out.attack { spacing.mapped(tick); }
    }
    let active = GameplayContext { status: status::ATTACK_AIR,
        can_air_attack: false, ..c };
    assert!(!spacing.sample(28, true, cfg, active, enemy(-18., 0.)).attack);
    assert!(spacing.sample(29, true, cfg, c, enemy(-18., 0.)).attack);
    assert!(!spacing.sample(30, true, cfg, c, enemy(-20.01, 0.)).attack);
    assert!(spacing.sample(31, true, cfg, c, enemy(18., 0.)).attack);
    assert_eq!(spacing.output.right_stick, [1., 0.]);
}

#[test]
fn ground_walk_and_retreat_use_ledge_curve_without_affecting_air_retreat() {
    for speed in [0.1, 0.35, 0.45, 1.] {
        for stick in [[0.2, 0.], [-0.8, 0.], [1., 0.], [-1., 0.], [1., -1.]] {
            assert_eq!(manual_movement(stick, [speed; 2]),
                crate::manual_controls::ledge_walk_stick(stick, speed));
        }
        for side in [-1., 1.] {
            let cfg = Config { ground_speed: speed, ..Config::default() };
            let ground = Spacing::default().sample(1, true, cfg, context(), enemy(side * 5., 0.));
            assert_eq!(ground.controller.stick,
                Some(crate::manual_controls::ledge_walk_stick([-side, 0.], speed)));
            let air = GameplayContext { situation: situation::AIR, status: status::FALL, ..context() };
            assert_eq!(Spacing::default().sample(1, true, cfg, air, enemy(side * 5., 0.))
                .controller.stick, Some([-side, 0.]));
        }
    }
}
const GROUND_WALK_STICK: f32 = 88. / 127.;

#[test]
fn invalid_retreat_caps_are_inert_and_edge_safety_still_wins() {
    for invalid in [f32::NAN, f32::INFINITY, -0.01, 1.01] {
        for ground in [true, false] {
            let cfg = Config {
                ground_retreat_stick: if ground { invalid } else { 1. },
                air_retreat_stick: if ground { 1. } else { invalid },
                ..Config::default()
            };
            assert_eq!(Spacing::default().sample(1, true, cfg, context(), enemy(5., 0.)), Output::default());
        }
    }
    let c = GameplayContext { safe_ground: [false; 2], ..context() };
    assert_eq!(Spacing::default().sample(1, true, Config::default(), c, enemy(5., 0.)).controller.stick, Some([0.; 2]));
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
        fighter_kind: -1,
        ground_speed: 1.,
        ground_retreat_stick: 1.,
        air_retreat_stick: 1.,
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
fn auto_spacing_does_not_retreat_for_opponent_action_changes_or_approach_outside_spacing_range() {
    let config = Config::default();
    let mut spacing = Spacing::default();
    let mut opponent = enemy(25., 0.).unwrap();
    opponent.motion = Some(crate::gameplay::MotionTiming {
        kind: 1,
        frame: 10.,
        rate: 1.,
        remaining: None,
    });
    assert_eq!(
        spacing
            .sample(1, true, config, context(), Some(opponent))
            .controller
            .stick,
        None
    );

    opponent.status = status::ATTACK;
    opponent.motion.as_mut().unwrap().frame = 0.;
    assert_eq!(
        spacing
            .sample(2, true, config, context(), Some(opponent))
            .controller
            .stick,
        None
    );

    opponent.position[0] = 31.;
    opponent.velocity[0] = -0.1;
    assert_eq!(
        spacing
            .sample(3, true, config, context(), Some(opponent))
            .controller
            .stick,
        None
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
fn close_target_brakes_approaching_dash_before_selecting_a_tilt() {
    for status in [status::DASH, status::RUN, status::TURN_RUN, status::RUN_BRAKE] {
        let c = GameplayContext { status, can_act: false, velocity: [3., 0.], ..context() };
        let mut spacing = Spacing::default();
        let braking = spacing.sample(1, true, config(), c, enemy(18., 0.));
        assert!(!braking.attack);
        assert_eq!(braking.controller.stick, Some([0.; 2]));
        assert!(spacing.sample(2, true, config(), context(), enemy(18., 0.)).attack);
    }
}

#[test]
fn spacing_discards_buffered_selection_when_native_readiness_is_lost() {
    for airborne in [false, true] {
        let c = GameplayContext {
            situation: if airborne { situation::AIR } else { situation::GROUND },
            status: if airborne { status::FALL } else { status::WAIT },
            ..context()
        };
        let mut spacing = Spacing::default();
        assert!(spacing.sample(1, true, config(), c, enemy(18., 0.)).attack);
        spacing.mapped(1);
        let locked = GameplayContext {
            status: if airborne { status::CLIFF_JUMP2 } else { status::LANDING_ATTACK_AIR },
            can_act: false, can_air_attack: false, ..c
        };
        assert!(!spacing.sample(2, true, config(), locked, enemy(18., 0.)).attack);
        assert!(!spacing.pending);
        assert!(spacing.sample(3, true, config(), c, enemy(18., 0.)).attack);
    }
}

#[test]
fn manual_attack_selection_yields_then_allows_retreat_after_confirmed_startup() {
    let aerial = crate::timing::motion_hash("attack_air_f");
    assert!(manual_action_pending(MAPPER_A, [0.; 2], MAPPER_A, status::FALL, 0, 0.));
    assert!(manual_action_pending(MAPPER_A, [0.; 2], 0, status::ATTACK_AIR, aerial, 0.));
    assert!(!manual_action_pending(MAPPER_A, [0.; 2], 0, status::ATTACK_AIR, aerial, 1.));
    assert!(manual_action_pending(MAPPER_A, [0.; 2], MAPPER_A, status::ATTACK_AIR, aerial, 1.));
    assert!(!manual_action_pending(0, [1., 0.], 0, status::ATTACK_S3, 0, 1.));
    assert!(manual_action_pending(crate::native_input::MAPPER_B, [0.; 2], 0,
        status::ATTACK_S3, 0, 1.));
    let c = GameplayContext { status: status::ATTACK_AIR, situation: situation::AIR,
        motion_kind: aerial, motion_frame: 1., can_air_attack: false, ..context() };
    let out = Spacing::default().sample(1, true, config(), c, enemy(5., 0.));
    assert!(!out.attack);
    assert_eq!(out.controller.stick, Some([-1., 0.]));
}

#[test]
fn platform_fall_and_ledge_jump_resume_aerials_only_when_native_ready() {
    for status in [status::FALL, status::FALL_AERIAL, status::CLIFF_JUMP2, status::CLIFF_JUMP3] {
        let c = GameplayContext { status, situation: situation::AIR,
            can_air_attack: false, ..context() };
        let mut spacing = Spacing::default();
        assert!(!spacing.sample(1, true, config(), c, enemy(18., 0.)).attack);
        assert!(spacing.sample(2, true, config(), GameplayContext { can_air_attack: true, ..c },
            enemy(18., 0.)).attack);
    }
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

