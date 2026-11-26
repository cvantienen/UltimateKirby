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
        "unshielded local status cannot authorize automatic grabs"
    );
}
#[test]
fn movement_landing_and_fresh_physical_shield_cannot_authorize_automatic_grabs() {
    use kirby_assist_core::native_states::status;
    for native in [
        status::WAIT,
        status::WALK,
        status::WALK_BRAKE,
        status::DASH,
        status::RUN,
        status::RUN_BRAKE,
        status::TURN,
        status::TURN_DASH,
        status::TURN_RUN,
        status::TURN_RUN_BRAKE,
        status::SQUAT,
        status::SQUAT_WAIT,
        status::LANDING,
        status::LANDING_LIGHT,
    ] {
        assert!(!automatic_grab_ready(native, |_| true));
        for shielded in [false, true] {
            for x in [15., 30.] {
                let mut tool = OutOfShield::new();
                let out = tool.sample_automatic(
                    true,
                    c(native),
                    lc(native),
                    targets(x),
                    None,
                    shielded,
                    22.,
                    35.,
                    Some(8.),
                );
                assert_eq!(
                    out,
                    Default::default(),
                    "native {native:#x} shielded {shielded} x {x}"
                );
                assert!(tool.target().is_none());
            }
        }
    }
}
#[test]
fn only_native_shield_states_use_standing_catch_readiness() {
    use kirby_assist_core::native_states::{status, transition_terms};
    assert!(automatic_grab_ready(status::GUARD, |_| panic!(
        "held guard needs no transition query"
    )));
    let mut seen = Vec::new();
    assert!(automatic_grab_ready(status::GUARD_ON, |term| {
        seen.push(term);
        term == transition_terms::CONT_CATCH
    }));
    assert_eq!(seen, [transition_terms::CONT_CATCH]);
    assert!(!automatic_grab_ready(status::GUARD_ON, |term| term
        == transition_terms::CONT_CATCH_DASH));
    assert!(
        !kirby_assist_core::out_of_shield::automatic_grab_ready_with_shield(
            status::GUARD,
            false,
            |_| true
        )
    );
}
#[test]
fn shield_grab_neutralizes_both_layouts_and_releases_on_catch_or_air_takeover() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        let mut tool = OutOfShield::new();
        let guard = kirby_assist_core::native_states::GUARD;
        let out = tool.sample_automatic(
            true,
            c(guard),
            lc(guard),
            targets(15.),
            None,
            true,
            22.,
            35.,
            Some(8.),
        );
        assert_eq!(out.requested, MAPPER_R | MAPPER_ZR);
        assert_eq!(out.owned, SEQUENCE_BUTTONS);
        let mapped = ManualOverlay::default().apply_controls(
            ButtonTrace {
                held: GAMEPLAY_BUTTONS,
                ..Default::default()
            },
            [0.65, -1.],
            TurboOutput { stick: out.stick },
            0,
            layout.physical_buttons(out.requested),
            layout.physical_owned_buttons(out.owned),
        );
        assert_eq!(mapped.stick, [0.; 2]);
        assert_eq!(
            mapped.buttons.held & GAMEPLAY_BUTTONS,
            layout.physical_buttons(out.requested)
        );
        tool.mapped_grab(0, 2);
        let caught = kirby_assist_core::native_states::CATCH;
        assert_eq!(
            tool.sample_automatic(
                true,
                c(caught),
                lc(caught),
                targets(15.),
                None,
                true,
                22.,
                35.,
                Some(8.)
            ),
            Default::default()
        );
        assert!(tool.target().is_none());
    }
    let mut tool = OutOfShield::new();
    let guard = kirby_assist_core::native_states::GUARD;
    assert_ne!(
        tool.sample_automatic(
            true,
            c(guard),
            lc(guard),
            targets(15.),
            None,
            true,
            22.,
            35.,
            Some(8.)
        )
        .owned,
        0
    );
    assert_eq!(
        tool.sample_automatic(
            true,
            Context {
                grounded: false,
                ..c(guard)
            },
            GameplayContext {
                situation: kirby_assist_core::native_states::situation::AIR,
                ..lc(guard)
            },
            targets(15.),
            None,
            true,
            22.,
            35.,
            Some(8.)
        ),
        Default::default()
    );
}
