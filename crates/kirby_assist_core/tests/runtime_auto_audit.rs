//! Live-mode regressions: stalled delivery, event identity and physical handback.
use kirby_assist_core::{
    action_owner::{automatic_escape_request, Owner},
    defense_response::Response,
    gameplay::{GameplayContext, MotionTiming, Opponent},
    ledge_trap::LedgeSettings,
    native_input::{automatic_escape, ControlLayout, MAPPER_B, MAPPER_R, MAPPER_ZL, MAPPER_ZR},
    observation::{Fact, Field, ObservationSnapshot},
    reaction_rules::{Candidate, Engine, Resolved},
    roll_read::Target,
    sequence_config::{Catalog, Sequence},
    spot_dodge::{Context as DodgeContext, SpotDodge, Target as GrabTarget},
};
fn own(frame: f32, status: i32) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        can_act: true,
        status,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.,
        facing: 1.,
        ..Default::default()
    }
}
fn recipe(frames: u8) -> Sequence {
    Catalog::parse(&format!("[bowser.roll.auto]\nstart_delay=0\nreaction=auto_roll:40:12\nactions=up_b:{frames},drift:opponent:1:69\n")).unwrap().entries()[0].sequence
}
fn rules(frames: u8) -> Resolved {
    let mut out = Resolved::default();
    out.add(Candidate {
        slot: 2,
        index: 0,
        sequence: recipe(frames),
    })
    .unwrap();
    out
}
fn enemy(frame: f32) -> Opponent {
    Opponent {
        id: 2,
        status: kirby_assist_core::native_states::ESCAPE_F,
        position: [10., 0.],
        motion: Some(MotionTiming {
            kind: 2,
            frame,
            rate: 0.,
            remaining: None,
        }),
        ..Default::default()
    }
}
fn targets(frame: f32) -> [Option<Target>; 8] {
    let mut out = [None; 8];
    out[0] = Some(Target {
        id: 2,
        status: kirby_assist_core::native_states::ESCAPE_F,
        motion: 2,
        frame,
        rate: 0.,
        position: [10., 0.],
        velocity_x: 0.,
        vulnerable_in: None,
        actionable_in: None,
    });
    out
}
fn settings() -> LedgeSettings {
    LedgeSettings {
        fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
        ..Default::default()
    }
}

#[test]
fn frozen_native_clocks_cannot_hold_unconfirmed_automatic_up_b_forever() {
    let c = GameplayContext {
        motion_rate: 0.,
        ..own(0., kirby_assist_core::native_states::GUARD)
    };
    let mut engine = Engine::default();
    let rules = rules(4);
    let mut response = Response::default();
    assert!(response.start_up_b(recipe(4), c, enemy(0.)));
    for tick in 0..24 {
        for _ in 0..8 {
            assert_ne!(
                engine
                    .sample_simple(true, &rules, settings(), c, targets(0.), Some(enemy(0.)))
                    .requested
                    & MAPPER_B,
                0
            );
            assert_ne!(
                response
                    .sample(true, settings(), c, Some(enemy(0.)))
                    .requested
                    & MAPPER_B,
                0
            );
        }
        for _ in 0..8 {
            engine.mapped_up_special(tick, 999);
            response.mapped_up_special(tick, 999);
            engine.mapped_up_special(tick, 2);
            response.mapped_up_special(tick, 2);
        }
    }
    let stopped = GameplayContext { stopped: true, ..c };
    assert_eq!(
        engine
            .sample_simple(
                true,
                &rules,
                settings(),
                stopped,
                targets(0.),
                Some(enemy(0.))
            )
            .owned,
        0
    );
    assert_eq!(
        response
            .sample(true, settings(), stopped, Some(enemy(0.)))
            .owned,
        0
    );
    assert!(!engine.committed());
    assert!(!response.automatic_up_b());
    assert_eq!(
        engine
            .sample_simple(true, &rules, settings(), c, targets(0.), Some(enemy(0.)))
            .owned,
        0,
        "a timed-out event cannot immediately restart"
    );
}

#[test]
fn native_confirmation_wins_at_delivery_limit_and_preserves_all_69_drift_frames() {
    let mut engine = Engine::default();
    let rules = rules(4);
    let mut response = Response::default();
    assert!(response.start_up_b(recipe(4), own(0., 0), enemy(0.)));
    engine.sample_simple(
        true,
        &rules,
        settings(),
        own(0., 0),
        targets(0.),
        Some(enemy(0.)),
    );
    assert_eq!(
        response
            .sample(true, settings(), own(0., 0), Some(enemy(0.)))
            .requested,
        MAPPER_B
    );
    for tick in 0..24 {
        engine.mapped_up_special(tick, 2);
        response.mapped_up_special(tick, 2);
    }
    for frame in 1..4 {
        let c = own(
            frame as f32,
            kirby_assist_core::native_states::status::SPECIAL_HI,
        );
        assert_eq!(
            engine
                .sample_simple(
                    true,
                    &rules,
                    settings(),
                    c,
                    [None; 8],
                    Some(enemy(frame as f32))
                )
                .requested,
            MAPPER_B
        );
        assert_eq!(
            response
                .sample(true, settings(), c, Some(enemy(frame as f32)))
                .requested,
            MAPPER_B
        );
        engine.mapped_up_special(frame, 2);
        response.mapped_up_special(frame, 2);
    }
    for frame in 4..73 {
        let c = GameplayContext {
            situation: 2,
            ..own(
                frame as f32,
                kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_G,
            )
        };
        let a = engine.sample_simple(
            true,
            &rules,
            settings(),
            c,
            [None; 8],
            Some(enemy(frame as f32)),
        );
        let b = response.sample(true, settings(), c, Some(enemy(frame as f32)));
        assert_eq!(a.stick, Some([1., 0.]), "frame {frame}");
        assert_eq!(b.stick, a.stick);
        assert_eq!(a.requested, 0);
        assert_eq!(b.requested, 0);
    }
    engine.sample_simple(
        true,
        &rules,
        settings(),
        own(73., kirby_assist_core::native_states::status::SPECIAL_HI),
        [None; 8],
        Some(enemy(73.)),
    );
    response.sample(
        true,
        settings(),
        own(73., kirby_assist_core::native_states::status::SPECIAL_HI),
        Some(enemy(73.)),
    );
    assert!(!engine.committed());
    assert!(!response.automatic_up_b());
}

#[test]
fn used_roll_survives_a_roster_gap_but_a_known_native_end_rearms_the_next_roll() {
    let mut engine = Engine::default();
    let mut rule = recipe(1);
    // A one-step recipe makes native completion independent of a drift tail.
    rule.len = 1;
    let mut rules = Resolved::default();
    rules
        .add(Candidate {
            slot: 2,
            index: 0,
            sequence: rule,
        })
        .unwrap();
    assert_eq!(
        engine
            .sample_simple(
                true,
                &rules,
                settings(),
                own(0., 0),
                targets(2.),
                Some(enemy(2.))
            )
            .requested,
        MAPPER_B
    );
    engine.sample_simple(
        true,
        &rules,
        settings(),
        own(1., kirby_assist_core::native_states::status::SPECIAL_HI),
        targets(3.),
        Some(enemy(3.)),
    );
    assert!(!engine.committed());
    assert_eq!(
        engine
            .sample_simple(true, &rules, settings(), own(2., 0), [None; 8], None)
            .owned,
        0
    );
    let mut moved = [None; 8];
    moved[6] = targets(4.)[0];
    assert_eq!(
        engine
            .sample_simple(true, &rules, settings(), own(3., 0), moved, Some(enemy(4.)))
            .owned,
        0,
        "missing/reordered roster must not replay the same roll"
    );
    engine.observe_statuses(&[Some((2, 0)), None, None, None, None, None, None, None]);
    assert_eq!(
        engine
            .sample_simple(
                true,
                &rules,
                settings(),
                own(4., 0),
                targets(6.),
                Some(enemy(6.))
            )
            .requested,
        MAPPER_B,
        "a later roll may first be observed at a larger frame"
    );
}

fn grab(frame: f32) -> [Option<GrabTarget>; 8] {
    let mut out = [None; 8];
    out[0] = Some(GrabTarget {
        id: 2,
        status: kirby_assist_core::native_states::CATCH,
        motion: 2,
        frame,
        rate: 1.,
        position: [10., 0.],
        velocity: [0.; 2],
        facing: -1.,
        first: 6,
        last: 7,
        reach: 12.,
    });
    out
}
fn dodge_context(status: i32) -> DodgeContext {
    DodgeContext {
        object: 1,
        status,
        situation: 0,
        motion: 1,
        frame: 0.,
        rate: 1.,
        stopped: false,
        can_spot_dodge: true,
        position: [0.; 2],
        velocity: [0.; 2],
    }
}
#[test]
fn known_grab_end_rearms_defense_without_rearming_for_unknown_or_stopped_targets() {
    let mut dodge = SpotDodge::default();
    assert_eq!(
        dodge.sample(true, dodge_context(0), grab(2.)).requested,
        MAPPER_ZR
    );
    dodge.sample(
        true,
        dodge_context(kirby_assist_core::native_states::ESCAPE),
        grab(3.),
    );
    dodge.cancel_response();
    dodge.observe_statuses(&[None; 8]);
    dodge.sample(true, dodge_context(0), [None; 8]);
    dodge.observe_statuses(&[
        Some((2, kirby_assist_core::native_states::CATCH)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ]);
    assert_eq!(dodge.sample(true, dodge_context(0), grab(4.)).owned, 0);
    dodge.observe_statuses(&[Some((99, 0)), None, None, None, None, None, None, None]);
    assert_eq!(dodge.sample(true, dodge_context(0), grab(5.)).owned, 0);
    dodge.observe_statuses(&[Some((2, 0)), None, None, None, None, None, None, None]);
    assert_eq!(
        dodge.sample(true, dodge_context(0), grab(6.)).requested,
        MAPPER_ZR
    );
}

#[test]
fn all_automatic_up_b_owners_use_the_same_manual_shield_and_jump_escape() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        let shield = layout.physical_buttons(MAPPER_ZR);
        for owner in [Owner::RollRead, Owner::OptionRead, Owner::OutOfShield] {
            let pulse =
                automatic_escape_request(Some(owner), MAPPER_B, MAPPER_B, Some(MAPPER_B)).unwrap();
            assert!(
                !automatic_escape(layout, shield, shield, true, true, pulse),
                "grounded startup remains protected: {owner:?}"
            );
            assert!(
                automatic_escape(layout, shield, shield, true, false, pulse),
                "airborne shield is a manual escape: {owner:?}"
            );
            let drift = automatic_escape_request(Some(owner), 0, 0, Some(0)).unwrap();
            assert!(automatic_escape(layout, shield, shield, true, true, drift));
            assert!(
                !automatic_escape(layout, shield, 0, true, true, drift),
                "an old shield hold cannot cancel drift"
            );
            let jump = if layout == ControlLayout::Default {
                MAPPER_ZL
            } else {
                MAPPER_R
            };
            assert!(
                automatic_escape(layout, jump, jump, true, true, pulse),
                "a physical jump escapes every automatic owner"
            );
        }
    }
    for owner in [
        Owner::Ledge,
        Owner::Shortcut,
        Owner::Safety,
        Owner::GrabDodge,
        Owner::Trip,
        Owner::Defense,
        Owner::Posture,
    ] {
        assert_eq!(
            automatic_escape_request(Some(owner), MAPPER_B, MAPPER_B, Some(MAPPER_B)),
            None
        );
    }
    assert_eq!(
        automatic_escape_request(Some(Owner::OutOfShield), 0, 0, None),
        None,
        "shield grabs retain their own manual gates"
    );
}

#[test]
fn known_status_export_only_uses_existing_facts_and_does_not_claim_unknown_fields() {
    let facts = ObservationSnapshot::default();
    assert_eq!(facts.known_statuses(), [None; 8]);
    facts.read(10, Field::X, || Fact::Float(3.));
    assert_eq!(facts.known_statuses(), [None; 8]);
    facts.read(20, Field::Status, || Fact::Int(0x20));
    for _ in 0..100 {
        assert_eq!(
            facts
                .known_statuses()
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            vec![(20, 0x20)]
        );
    }
    facts.read(20, Field::Status, || {
        panic!("export caused another native query")
    });
    assert_eq!(ObservationSnapshot::default().known_statuses(), [None; 8]);
}

#[test]
fn leaving_ground_before_native_confirmation_cannot_turn_the_request_into_an_air_up_b() {
    for status in [
        kirby_assist_core::native_states::status::FALL_SPECIAL,
        kirby_assist_core::native_states::status::JUMP,
        kirby_assist_core::native_states::status::FALL,
    ] {
        let mut engine = Engine::default();
        let rules = rules(4);
        let mut response = Response::default();
        assert!(response.start_up_b(recipe(4), own(0., 0), enemy(0.)));
        assert_eq!(
            engine
                .sample_simple(
                    true,
                    &rules,
                    settings(),
                    own(0., 0),
                    targets(0.),
                    Some(enemy(0.))
                )
                .requested,
            MAPPER_B
        );
        assert_eq!(
            response
                .sample(true, settings(), own(0., 0), Some(enemy(0.)))
                .requested,
            MAPPER_B
        );
        let air = GameplayContext {
            situation: 2,
            ..own(1., status)
        };
        assert_eq!(
            engine
                .sample_simple(true, &rules, settings(), air, targets(1.), Some(enemy(1.)))
                .owned,
            0
        );
        assert_eq!(
            response
                .sample(true, settings(), air, Some(enemy(1.)))
                .owned,
            0
        );
        assert!(!engine.committed());
        assert!(!response.automatic_up_b());
    }
}

#[test]
fn another_fighter_reusing_a_roster_slot_cannot_erase_a_consumed_event() {
    let mut engine = Engine::default();
    let mut sequence = recipe(1);
    sequence.len = 1;
    let mut rules = Resolved::default();
    rules
        .add(Candidate {
            slot: 2,
            index: 0,
            sequence,
        })
        .unwrap();
    engine.sample_simple(
        true,
        &rules,
        settings(),
        own(0., 0),
        targets(2.),
        Some(enemy(2.)),
    );
    engine.sample_simple(
        true,
        &rules,
        settings(),
        own(1., kirby_assist_core::native_states::status::SPECIAL_HI),
        targets(3.),
        Some(enemy(3.)),
    );
    let mut other = targets(4.);
    other[0].as_mut().unwrap().id = 3;
    engine.sample_simple(
        true,
        &rules,
        settings(),
        GameplayContext {
            can_act: false,
            ..own(2., 0)
        },
        other,
        None,
    );
    let mut moved = [None; 8];
    moved[6] = targets(5.)[0];
    assert_eq!(
        engine
            .sample_simple(true, &rules, settings(), own(3., 0), moved, Some(enemy(5.)))
            .owned,
        0
    );

    let mut dodge = SpotDodge::default();
    dodge.sample(true, dodge_context(0), grab(2.));
    dodge.sample(
        true,
        dodge_context(kirby_assist_core::native_states::ESCAPE),
        grab(3.),
    );
    dodge.cancel_response();
    let mut other = grab(4.);
    other[0].as_mut().unwrap().id = 3;
    dodge.sample(
        true,
        DodgeContext {
            can_spot_dodge: false,
            ..dodge_context(0)
        },
        other,
    );
    let mut moved = [None; 8];
    moved[6] = grab(5.)[0];
    assert_eq!(dodge.sample(true, dodge_context(0), moved).owned, 0);
}

#[test]
fn sixteen_climber_grabs_remain_consumed_when_warning_candidates_change() {
    let mut dodge = SpotDodge::default();
    for id in 1..=16 {
        let mut targets = grab(2.);
        targets[0].as_mut().unwrap().id = id;
        assert_eq!(
            dodge.sample(true, dodge_context(0), targets).requested,
            MAPPER_ZR
        );
        dodge.sample(
            true,
            dodge_context(kirby_assist_core::native_states::ESCAPE),
            targets,
        );
        dodge.cancel_response();
    }
    for id in 1..=16 {
        let mut targets = grab(4.);
        targets[0].as_mut().unwrap().id = id;
        assert_eq!(
            dodge.sample(true, dodge_context(0), targets).requested,
            0,
            "the same ongoing grab from actor {id} cannot cause another spot dodge"
        );
    }
}
