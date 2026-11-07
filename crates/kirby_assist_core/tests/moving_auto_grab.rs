use kirby_assist_core::{
    gameplay::{GameplayContext, MotionTiming, Opponent},
    ledge_trap::SEQUENCE_BUTTONS,
    manual_controls::{ManualOverlay, ManualTakeover, TurboOutput},
    native_input::*,
    out_of_shield::{automatic_grab_ready, Context, OutOfShield},
    reaction_rules::{Candidate, Engine, Resolved},
    roll_read::Target,
    sequence_config::Catalog,
    settings::AssistSettings,
};
fn c(status: i32) -> Context {
    Context {
        object: 1,
        kind: kirby_assist_core::fighter_kinds::BOWSER,
        status,
        motion: 1,
        frame: 0.,
        rate: 1.,
        stopped: false,
        grounded: true,
        position: [0.; 2],
        facing: 1.,
        shield_ready: true,
        can_grab: true,
        can_jump: false,
        can_up_special: true,
        threatened: false,
    }
}
fn lc(status: i32) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        status,
        motion_kind: 1,
        motion_frame: 0.,
        motion_rate: 1.,
        can_act: true,
        facing: 1.,
        ..Default::default()
    }
}
fn targets(x: f32) -> [Option<Target>; 8] {
    [
        Some(Target {
            id: 2,
            status: kirby_assist_core::native_states::GUARD,
            motion: 2,
            frame: 0.,
            rate: 1.,
            position: [x, 0.],
            velocity_x: 0.,
            vulnerable_in: Some(0.),
            actionable_in: None,
        }),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ]
}
#[test]
fn ground_locomotion_and_normal_landing_grab_on_the_first_observation_without_an_approach_pulse() {
    for status in [
        kirby_assist_core::native_states::status::WALK,
        kirby_assist_core::native_states::status::WALK_BRAKE,
        kirby_assist_core::native_states::status::DASH,
        kirby_assist_core::native_states::status::RUN,
        kirby_assist_core::native_states::status::RUN_BRAKE,
        kirby_assist_core::native_states::status::TURN,
        kirby_assist_core::native_states::status::TURN_DASH,
        kirby_assist_core::native_states::status::TURN_RUN,
        kirby_assist_core::native_states::status::TURN_RUN_BRAKE,
        kirby_assist_core::native_states::status::SQUAT,
        kirby_assist_core::native_states::status::SQUAT_WAIT,
        kirby_assist_core::native_states::status::LANDING,
        kirby_assist_core::native_states::status::LANDING_LIGHT,
    ] {
        for side in [-1., 1.] {
            let mut o = OutOfShield::new();
            let ctx = Context {
                facing: side,
                ..c(status)
            };
            let out = o.sample_automatic(
                true,
                ctx,
                GameplayContext {
                    facing: side,
                    ..lc(status)
                },
                targets(side * 15.),
                None,
                false,
                22.,
                35.,
                Some(8.),
            );
            assert_eq!(
                out.requested, MAPPER_R,
                "status {status:#x}: no neutral or approach-only packet"
            );
            assert!(!o.approach_pending());
            assert_eq!(out.owned, SEQUENCE_BUTTONS);
            for physical in [
                [0., 0.],
                [side * 0.3, 0.],
                [side * 0.6, 0.7],
                [side, -1.],
                [f32::NAN, 1.],
                [f32::INFINITY, -1.],
            ] {
                assert_eq!(o.with_grab_movement(out, physical).stick, Some([0., 0.]));
            }
        }
    }
    let mut o = OutOfShield::new();
    assert_eq!(
        o.sample_automatic(
            true,
            c(0),
            GameplayContext {
                velocity: [1., 0.],
                ..lc(0)
            },
            targets(15.),
            None,
            false,
            22.,
            35.,
            Some(8.)
        )
        .requested,
        MAPPER_R,
        "native WAIT with existing momentum must not introduce a new approach pulse"
    );
}
#[test]
fn native_dash_catch_readiness_is_used_without_waiting_for_standing_catch() {
    for status in [
        kirby_assist_core::native_states::status::WAIT,
        kirby_assist_core::native_states::status::WALK,
        kirby_assist_core::native_states::status::WALK_BRAKE,
        kirby_assist_core::native_states::status::DASH,
        kirby_assist_core::native_states::status::RUN,
        kirby_assist_core::native_states::status::RUN_BRAKE,
        kirby_assist_core::native_states::status::TURN,
        kirby_assist_core::native_states::status::TURN_DASH,
        kirby_assist_core::native_states::status::TURN_RUN,
        kirby_assist_core::native_states::status::TURN_RUN_BRAKE,
        kirby_assist_core::native_states::status::LANDING,
        kirby_assist_core::native_states::status::LANDING_LIGHT,
    ] {
        let mut seen = Vec::new();
        assert!(automatic_grab_ready(status, |term| {
            seen.push(term);
            term == kirby_assist_core::native_states::transition_terms::CONT_CATCH_DASH
        }));
        assert_eq!(
            seen,
            vec![
                kirby_assist_core::native_states::transition_terms::CONT_CATCH,
                kirby_assist_core::native_states::transition_terms::CONT_CATCH_DASH
            ]
        );
        assert!(!automatic_grab_ready(status, |_| false));
    }
    assert!(automatic_grab_ready(0x1c, |_| panic!(
        "native held shield remains a direct grab"
    )));
    assert!(
        !automatic_grab_ready(0x1b, |term| term
            == kirby_assist_core::native_states::transition_terms::CONT_CATCH_DASH),
        "guard startup cannot become a dash grab"
    );
}
#[test]
fn horizontal_intent_allows_ground_offense_but_shield_rolls_and_manual_attacks_are_preserved() {
    let mut input = ManualTakeover::new();
    assert!(!input.grounded_grab_assist(0));
    for status in [
        kirby_assist_core::native_states::status::WAIT,
        kirby_assist_core::native_states::status::WALK,
        kirby_assist_core::native_states::status::DASH,
        kirby_assist_core::native_states::status::RUN,
        kirby_assist_core::native_states::status::LANDING,
    ] {
        for x in [-1., -0.5, 0.5, 1.] {
            input.sample(0, [x, 0., 0., 0.]);
            assert!(input.grounded_grab_assist(status));
        }
    }
    for status in [
        kirby_assist_core::native_states::status::WAIT,
        kirby_assist_core::native_states::GUARD_ON,
        kirby_assist_core::native_states::GUARD,
    ] {
        for x in [-1., 1.] {
            input.sample(MAPPER_ZR, [x, 0., 0., 0.]);
            assert!(
                !input.grounded_grab_assist(status),
                "manual shield roll is not an automatic grab"
            );
        }
    }
    for button in [MAPPER_A, MAPPER_B, MAPPER_R] {
        input.sample(button, [1., 0., 0., 0.]);
        assert!(!input.grounded_grab_assist(3));
    }
    for axes in [[1., 1., 0., 0.], [1., 0., 1., 0.], [f32::NAN, 0., 0., 0.]] {
        input.sample(0, axes);
        assert!(!input.grounded_grab_assist(3));
    }
}
#[test]
fn airborne_snapshots_and_unready_movement_never_start_a_grab_and_landing_does_not_queue() {
    for mismatch in 0..3 {
        let mut o = OutOfShield::new();
        let mut ctx = c(3);
        let mut own = lc(3);
        match mismatch {
            0 => ctx.grounded = false,
            1 => own.situation = kirby_assist_core::native_states::situation::AIR,
            _ => ctx.can_grab = false,
        }
        assert_eq!(
            o.sample_automatic(
                true,
                ctx,
                own,
                targets(15.),
                None,
                false,
                22.,
                35.,
                Some(8.)
            )
            .owned,
            0
        );
        assert!(o.target().is_none());
        assert_eq!(
            o.sample_automatic(
                true,
                c(0x16),
                lc(0x16),
                targets(15.),
                None,
                false,
                22.,
                35.,
                Some(8.)
            )
            .requested,
            MAPPER_R
        );
    }
}
#[test]
fn both_layouts_neutralize_stick_during_unshielded_and_shielded_grabs() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for shielded in [false, true] {
            let status = if shielded {
                kirby_assist_core::native_states::GUARD
            } else {
                4
            };
            let mut o = OutOfShield::new();
            let raw = o.sample_automatic(
                true,
                c(status),
                lc(status),
                targets(15.),
                None,
                shielded,
                22.,
                35.,
                Some(8.),
            );
            let out = o.with_grab_movement(raw, [0.65, 0.]);
            let expected = [0.; 2];
            assert_eq!(out.stick, Some(expected));
            let physical_directions = (1 << 21) | (1 << 20);
            let mapped = ManualOverlay::default().apply_controls(
                ButtonTrace {
                    held: layout.physical_buttons(MAPPER_ZR) | physical_directions,
                    ..Default::default()
                },
                [0.65, 0.],
                TurboOutput {
                    stick: out.stick,
                    ..Default::default()
                },
                0,
                layout.physical_buttons(out.requested),
                layout.physical_owned_buttons(out.owned),
            );
            assert_eq!(mapped.stick, expected);
            assert_eq!(mapped.buttons.held & MAPPER_LEFT_STICK_DIRECTIONS, 0);
            assert_eq!(
                exclusive_sequence_stick(out.owned, out.stick),
                Some(expected)
            );
            assert_eq!(
                mapped.buttons.held & GAMEPLAY_BUTTONS,
                layout.physical_buttons(out.requested)
            );
            assert_eq!(
                out.requested,
                MAPPER_R | if shielded { MAPPER_ZR } else { 0 }
            );
            let native = mapped_literal_buttons(0, out.requested);
            assert_eq!(
                mapped_unshielded_auto_grab(native, out.requested, o.unshielded_grab_started()),
                MAPPED_CATCH | MAPPED_GUARD
            );
        }
    }
}

#[test]
fn mapped_unshielded_grab_fix_is_limited_to_active_grab_packets() {
    let native = 0x800;
    assert_eq!(mapped_unshielded_auto_grab(native, MAPPER_R, false), native);
    assert_eq!(mapped_unshielded_auto_grab(native, 0, true), native);
    assert_eq!(mapped_unshielded_auto_grab(native, MAPPER_A, true), native);
    assert_eq!(
        mapped_unshielded_auto_grab(native, MAPPER_R | MAPPER_ZR, true),
        native
    );
    assert_eq!(
        mapped_unshielded_auto_grab(native, MAPPER_R, true),
        native | MAPPED_CATCH | MAPPED_GUARD
    );
}
#[test]
fn ground_up_b_admission_keeps_tracking_after_native_takeoff_without_starting_new_air_reactions() {
    let cat=Catalog::parse("[bowser.roll.up_b]\nstart_delay=0\nreaction=auto_roll:40:12\nactions=up_b:4,drift:opponent:1:30\n").unwrap();
    let mut rules = Resolved::default();
    rules
        .add(Candidate {
            slot: 2,
            index: 0,
            sequence: cat.entries()[0].sequence,
        })
        .unwrap();
    for start in [0, 1, 3, 4, 0x16, 0x1c] {
        let mut e = Engine::default();
        let mut ts = targets(20.);
        ts[0].as_mut().unwrap().status = kirby_assist_core::native_states::ESCAPE_F;
        let out = e.sample_simple(true, &rules, AssistSettings::default(), lc(start), ts, None);
        assert_ne!(out.requested & MAPPER_B, 0, "grounded status {start:#x}");
        for frame in 1..=5 {
            let own = GameplayContext {
                status: kirby_assist_core::native_states::status::SPECIAL_HI,
                situation: kirby_assist_core::native_states::situation::AIR,
                motion_frame: frame as f32,
                ..lc(start)
            };
            let follow = Opponent {
                id: 2,
                status: kirby_assist_core::native_states::ESCAPE_F,
                position: [if frame == 5 { -20. } else { 20. }, 0.],
                motion: Some(MotionTiming {
                    kind: 2,
                    frame: frame as f32,
                    rate: 1.,
                    remaining: None,
                }),
                ..Default::default()
            };
            let out = e.sample_simple(
                true,
                &rules,
                AssistSettings::default(),
                own,
                ts,
                Some(follow),
            );
            if frame >= 4 {
                assert_eq!(
                    out.stick,
                    Some([if frame == 5 { -1. } else { 1. }, 0.]),
                    "tracking follows a crossing target after native takeoff"
                );
            }
        }
        let mut fresh = Engine::default();
        assert_eq!(
            fresh
                .sample_simple(
                    true,
                    &rules,
                    AssistSettings::default(),
                    GameplayContext {
                        situation: kirby_assist_core::native_states::situation::AIR,
                        ..lc(start)
                    },
                    ts,
                    None
                )
                .owned,
            0
        );
    }
}

#[test]
fn absent_targets_native_catch_start_and_air_takeover_release_horizontal_control_immediately() {
    let mut o = OutOfShield::new();
    let absent = o.sample_automatic(
        true,
        c(4),
        lc(4),
        targets(36.),
        None,
        false,
        22.,
        35.,
        Some(8.),
    );
    assert_eq!(absent.owned, 0);
    assert_eq!(o.with_grab_movement(absent, [1., 0.]).stick, None);
    for (status, grounded, situation) in [
        (kirby_assist_core::native_states::CATCH_DASH, true, 0),
        (0xe, false, 2),
    ] {
        let mut o = OutOfShield::new();
        assert_eq!(
            o.sample_automatic(
                true,
                c(4),
                lc(4),
                targets(15.),
                None,
                false,
                22.,
                35.,
                Some(8.)
            )
            .requested,
            MAPPER_R
        );
        let ended = o.sample_automatic(
            true,
            Context {
                status,
                grounded,
                ..c(4)
            },
            GameplayContext {
                situation,
                ..lc(status)
            },
            targets(15.),
            None,
            false,
            22.,
            35.,
            Some(8.),
        );
        assert_eq!(ended.owned, 0);
        assert!(o.target().is_none());
        assert_eq!(o.with_grab_movement(ended, [1., 0.]).stick, None);
    }
}

#[test]
fn native_movement_replaces_an_unmapped_idle_approach_without_waiting_for_an_ack() {
    let mut o = OutOfShield::new();
    let idle = o.sample_automatic(
        true,
        c(0),
        GameplayContext {
            safe_ground: [true; 2],
            ..lc(0)
        },
        targets(30.),
        None,
        false,
        22.,
        35.,
        Some(8.),
    );
    assert_eq!(idle.requested, 0);
    assert!(o.approach_pending());
    let moving = o.sample_automatic(
        true,
        c(1),
        lc(1),
        targets(30.),
        None,
        false,
        22.,
        35.,
        Some(8.),
    );
    assert_eq!(moving.requested, MAPPER_R);
    assert!(!o.approach_pending());
    assert_eq!(
        o.with_grab_movement(moving, [0.4, 0.]).stick,
        Some([0., 0.])
    );
}

#[test]
fn fresh_physical_shield_cannot_borrow_dash_readiness_before_native_guard_starts() {
    use kirby_assist_core::out_of_shield::automatic_grab_ready_with_shield;
    for status in [
        kirby_assist_core::native_states::status::WAIT,
        kirby_assist_core::native_states::status::WALK,
        kirby_assist_core::native_states::status::DASH,
        kirby_assist_core::native_states::status::RUN,
        kirby_assist_core::native_states::status::LANDING,
    ] {
        assert!(!automatic_grab_ready_with_shield(status, true, |term| {
            term == kirby_assist_core::native_states::transition_terms::CONT_CATCH_DASH
        }));
        assert!(automatic_grab_ready_with_shield(status, false, |term| {
            term == kirby_assist_core::native_states::transition_terms::CONT_CATCH_DASH
        }));
    }
}

#[test]
fn bowser_far_idle_grab_waits_for_native_dash_then_sends_neutral_grab() {
    for side in [-1., 1.] {
        for speed in [0., 0.15] {
            let mut o = OutOfShield::new();
            let ctx = Context {
                facing: side,
                ..c(0)
            };
            let own = GameplayContext {
                facing: side,
                velocity: [side * speed, 0.],
                previous_status: kirby_assist_core::native_states::GUARD_OFF,
                safe_ground: [true; 2],
                ..lc(0)
            };
            let ts = targets(side * 30.);
            for tick in 0..3 {
                let out = o.sample_automatic(true, ctx, own, ts, None, false, 22., 35., Some(8.));
                assert_eq!(
                    out.requested, 0,
                    "a mapped forward packet must not turn into a standing grab"
                );
                assert_eq!(out.stick, Some([side, 0.]));
                assert!(o.approach_pending());
                o.mapped_approach(tick, 2);
                o.mapped_approach(tick, 2);
            }
            let dash = o.sample_automatic(
                true,
                Context {
                    status: kirby_assist_core::native_states::status::DASH,
                    ..ctx
                },
                GameplayContext {
                    status: kirby_assist_core::native_states::status::DASH,
                    ..own
                },
                ts,
                None,
                false,
                22.,
                35.,
                Some(8.),
            );
            assert_eq!(
                dash.requested, MAPPER_R,
                "grab on first observed dash, no extra timing delay"
            );
            assert!(!o.approach_pending());
            assert_eq!(
                o.with_grab_movement(dash, [0.; 2]).stick,
                Some([0., 0.]),
                "grab packet must not retain the assisted dash direction"
            );
        }
    }
}

#[test]
fn failed_bowser_dash_start_is_bounded_and_never_falls_through_to_a_standing_grab() {
    let mut o = OutOfShield::new();
    let own = GameplayContext {
        safe_ground: [true; 2],
        ..lc(0)
    };
    for tick in 0..4 {
        for _ in 0..10 {
            assert_eq!(
                o.sample_automatic(
                    true,
                    c(0),
                    own,
                    targets(30.),
                    None,
                    false,
                    22.,
                    35.,
                    Some(8.)
                )
                .requested,
                0
            );
        }
        o.mapped_approach(tick, 2);
    }
    let ended = o.sample_automatic(
        true,
        c(0),
        own,
        targets(30.),
        None,
        false,
        22.,
        35.,
        Some(8.),
    );
    assert_eq!(ended.owned, 0);
    assert!(o.target().is_none());
}

#[test]
fn tech_in_place_is_not_a_dodge_event_and_far_targets_do_not_own_grab_input() {
    let cat = Catalog::parse(
        "[bowser.roll.auto]\nstart_delay=0\nreaction=auto_roll:40:12\nactions=up_b:4\n",
    )
    .unwrap();
    let mut rules = Resolved::default();
    rules
        .add(Candidate {
            slot: 2,
            index: 0,
            sequence: cat.entries()[0].sequence,
        })
        .unwrap();
    let mut tech = targets(80.);
    tech[0].as_mut().unwrap().status = kirby_assist_core::native_states::status::PASSIVE;
    let mut engine = Engine::default();
    assert_eq!(
        engine
            .sample_simple(true, &rules, AssistSettings::default(), lc(4), tech, None)
            .owned,
        0
    );
    let mut o = OutOfShield::new();
    assert_eq!(
        o.sample_automatic(true, c(4), lc(4), tech, None, false, 22., 35., Some(8.))
            .owned,
        0
    );
    tech[0].as_mut().unwrap().position[0] = 30.;
    assert_eq!(
        o.sample_automatic(true, c(4), lc(4), tech, None, false, 22., 35., Some(8.))
            .requested,
        0,
        "automatic grabs now require a visibly shielding opponent"
    );
}
