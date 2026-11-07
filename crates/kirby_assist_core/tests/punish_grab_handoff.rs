//! Native acceptance, single-dodge delivery and manual throw handoff regressions.
use kirby_assist_core::{
    action_owner::{native_grab_active, ActionOwnership, Intent, Owner},
    defense_response::Response,
    gameplay::{ControllerOutput, GameplayContext, MotionTiming, Opponent},
    ledge_trap::{InputTimeline, LedgeSettings},
    manual_controls::{ManualOverlay, TurboOutput},
    native_input::{ButtonTrace, MAPPER_B, MAPPER_R, MAPPER_ZR},
    out_of_shield::{Context as GrabContext, OutOfShield},
    reaction_rules::{Candidate, Engine, Resolved},
    roll_read::Target as ReactionTarget,
    sequence_config::{Catalog, Sequence},
    spot_dodge::{Context, SpotDodge, Target, ESCAPE},
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
        safe_ground: [true; 2],
        ..Default::default()
    }
}
fn dodge_context(frame: f32, status: i32) -> Context {
    Context {
        object: 1,
        status,
        situation: 0,
        motion: 1,
        frame,
        rate: 1.,
        stopped: false,
        can_spot_dodge: true,
        position: [0.; 2],
        velocity: [0.; 2],
    }
}
fn threat(frame: f32) -> [Option<Target>; 8] {
    let mut out = [None; 8];
    out[0] = Some(Target {
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
fn enemy(frame: f32, status: i32) -> Opponent {
    Opponent {
        id: 2,
        status,
        position: [10., 0.],
        vulnerable: true,
        motion: Some(MotionTiming {
            kind: 2,
            frame,
            rate: 1.,
            remaining: None,
        }),
        ..Default::default()
    }
}
fn recipe() -> Sequence {
    Catalog::parse("[bowser.roll.auto_spot_dodge]\nstart_delay=0\ntimeout=180\nreaction=auto_spot_dodge:35:12\nactions=up_b:4,drift:opponent:1:69\n")
        .unwrap().entries()[0].sequence
}
fn settings() -> LedgeSettings {
    LedgeSettings {
        fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
        ..Default::default()
    }
}
fn accepted() -> SpotDodge {
    let mut dodge = SpotDodge::default();
    assert_eq!(
        dodge
            .sample(true, dodge_context(0., 0), threat(2.))
            .requested,
        MAPPER_ZR
    );
    dodge.mapped_dodge(0, 2);
    assert_eq!(
        dodge
            .sample(true, dodge_context(1., ESCAPE), threat(3.))
            .owned,
        0
    );
    dodge
}
fn punish(dodge: &mut SpotDodge, c: GameplayContext) -> ControllerOutput {
    dodge.punish_automatic(
        true,
        Some(&recipe()),
        settings(),
        c,
        Some(enemy(
            c.motion_frame,
            kirby_assist_core::native_states::CATCH,
        )),
    )
}
fn grab_context(c: GameplayContext) -> GrabContext {
    GrabContext {
        object: 1,
        kind: kirby_assist_core::fighter_kinds::BOWSER,
        status: c.status,
        motion: c.motion_kind,
        frame: c.motion_frame,
        rate: c.motion_rate,
        stopped: c.stopped,
        grounded: true,
        position: c.position,
        facing: c.facing,
        shield_ready: true,
        can_grab: true,
        can_jump: false,
        can_up_special: true,
        threatened: false,
    }
}
fn targets(status: i32) -> [Option<ReactionTarget>; 8] {
    let mut out = [None; 8];
    out[0] = Some(ReactionTarget {
        id: 2,
        status,
        motion: 2,
        frame: 0.,
        rate: 1.,
        position: [10., 0.],
        velocity_x: 0.,
        vulnerable_in: Some(0.),
        actionable_in: None,
    });
    out
}

#[test]
fn one_delivered_down_guard_packet_then_native_recovery_up_b_on_first_legal_observation() {
    let mut dodge = SpotDodge::default();
    let mut last_y = 0.;
    let mut down_edges = 0;
    for tick in 0..5 {
        let out = dodge.sample(
            true,
            dodge_context(tick as f32, 0),
            threat(2. + tick as f32),
        );
        let y = out.stick.unwrap()[1];
        if y < -0.5 && last_y >= -0.5 {
            down_edges += 1;
        }
        last_y = y;
        assert_eq!(out.requested, if tick == 0 { MAPPER_ZR } else { 0 });
        if tick == 0 {
            dodge.mapped_dodge(tick, 2);
        } else {
            dodge.delivery_tick(tick);
        }
    }
    assert_eq!(down_edges, 1);
    assert_eq!(
        dodge
            .sample(true, dodge_context(5., ESCAPE), threat(7.))
            .owned,
        0
    );
    for frame in 5..9 {
        assert_eq!(
            punish(
                &mut dodge,
                GameplayContext {
                    can_act: false,
                    ..own(frame as f32, ESCAPE)
                }
            )
            .owned,
            0
        );
    }
    let out = punish(&mut dodge, own(9., 0));
    assert_eq!(out.requested, MAPPER_B);
    assert_eq!(out.stick, Some([0., 1.]));
    assert!(dodge.direct_up_b());
    assert_eq!(out.requested & (MAPPER_R | MAPPER_ZR), 0);
}

#[test]
fn native_dodge_cancel_readiness_does_not_add_an_idle_wait_frame() {
    let mut dodge = accepted();
    assert_eq!(
        punish(
            &mut dodge,
            GameplayContext {
                can_act: false,
                ..own(1., ESCAPE)
            }
        )
        .owned,
        0
    );
    assert_eq!(punish(&mut dodge, own(2., ESCAPE)).requested, MAPPER_B);
}

#[test]
fn unaccepted_or_unrelated_manual_dodge_never_claims_an_automatic_punish() {
    let mut dodge = SpotDodge::default();
    dodge.sample(true, dodge_context(0., ESCAPE), threat(2.));
    assert_eq!(punish(&mut dodge, own(1., 0)).owned, 0);
    dodge.sample(true, dodge_context(2., 0), threat(2.));
    dodge.cancel_pending();
    assert_eq!(punish(&mut dodge, own(3., 0)).owned, 0);
    assert!(!dodge.direct_up_b());
}

#[test]
fn lost_out_of_range_captured_airborne_and_disabled_followups_do_not_queue() {
    for case in 0..6 {
        let mut dodge = accepted();
        let mut c = GameplayContext {
            can_act: false,
            ..own(1., ESCAPE)
        };
        assert_eq!(punish(&mut dodge, c).owned, 0);
        c = own(2., 0);
        let mut e = enemy(2., kirby_assist_core::native_states::CATCH);
        let mut enabled = true;
        match case {
            0 => e.id = 99,
            1 => e.position[0] = 35.01,
            2 => e.position[1] = 12.01,
            3 => e.status = kirby_assist_core::native_states::CAPTURE_PULLED,
            4 => c.situation = 2,
            5 => enabled = false,
            _ => unreachable!(),
        }
        assert_eq!(
            dodge
                .punish_automatic(enabled, Some(&recipe()), settings(), c, Some(e))
                .owned,
            0,
            "case {case}"
        );
        assert_eq!(
            punish(&mut dodge, own(3., 0)).owned,
            0,
            "no queued retry, case {case}"
        );
        assert!(dodge.follow_target().is_none());
    }
}

#[test]
fn consumed_grab_stays_consumed_after_a_missing_roster_sample() {
    let mut dodge = accepted();
    dodge.cancel_response();
    assert_eq!(dodge.sample(true, dodge_context(2., 0), [None; 8]).owned, 0);
    assert_eq!(
        dodge.sample(true, dodge_context(3., 0), threat(5.)).owned,
        0
    );
    assert_eq!(
        dodge
            .sample(true, dodge_context(4., 0), threat(0.))
            .requested,
        0
    );
    dodge.observe_statuses(&[Some((2, 0)), None, None, None, None, None, None, None]);
    assert_eq!(
        dodge
            .sample(true, dodge_context(5., 0), threat(0.))
            .requested,
        MAPPER_ZR
    );
}

#[test]
fn duplicate_observations_do_not_exhaust_neutral_acceptance_wait_but_real_mapper_ticks_do() {
    let mut dodge = SpotDodge::default();
    for tick in 0..24 {
        for _ in 0..10 {
            assert_eq!(
                dodge
                    .sample(true, dodge_context(0., 0), threat(2.))
                    .requested,
                if tick == 0 { MAPPER_ZR } else { 0 }
            );
        }
        if tick == 0 {
            dodge.mapped_dodge(tick, 999);
            dodge.mapped_dodge(tick, 2);
        }
        dodge.delivery_tick(tick);
        dodge.delivery_tick(tick);
    }
    assert_eq!(
        dodge
            .sample(
                true,
                Context {
                    stopped: true,
                    ..dodge_context(0., 0)
                },
                threat(2.)
            )
            .owned,
        0
    );
    assert_eq!(
        dodge.sample(true, dodge_context(0., 0), threat(2.)).owned,
        0
    );
    assert_eq!(punish(&mut dodge, own(1., 0)).owned, 0);
}

#[test]
fn confirmed_up_b_tracks_for_69_native_frames_and_can_follow_air_spin() {
    let mut dodge = accepted();
    assert_eq!(punish(&mut dodge, own(0., 0)).requested, MAPPER_B);
    for frame in 1..4 {
        assert_eq!(
            punish(&mut dodge, own(frame as f32, 0x1de)).requested,
            MAPPER_B
        );
    }
    for frame in 4..73 {
        let x = if frame % 2 == 0 { 10. } else { -10. };
        let e = Opponent {
            position: [x, 0.],
            ..enemy(frame as f32, 0)
        };
        let out = dodge.punish_automatic(
            true,
            Some(&recipe()),
            settings(),
            GameplayContext {
                situation: 2,
                ..own(frame as f32, 0x1e1)
            },
            Some(e),
        );
        assert_eq!(out.requested, 0, "frame {frame}");
        assert_eq!(out.stick, Some([x.signum(), 0.]), "frame {frame}");
    }
    assert_eq!(punish(&mut dodge, own(73., 0x1de)).requested, 0);
    assert!(!dodge.direct_up_b());
}

#[test]
fn unconfirmed_up_b_never_turns_into_a_drift_chase() {
    let mut dodge = accepted();
    for frame in 0..4 {
        assert_eq!(punish(&mut dodge, own(frame as f32, 0)).requested, MAPPER_B);
    }
    assert_eq!(punish(&mut dodge, own(4., 0)).owned, 0);
    assert!(!dodge.direct_up_b());
}

#[test]
fn mapped_grab_cannot_be_replaced_by_up_b_before_native_acceptance() {
    let mut grab = OutOfShield::new();
    let c = own(0., kirby_assist_core::native_states::GUARD);
    let out = grab.sample_automatic(
        true,
        grab_context(c),
        c,
        targets(kirby_assist_core::native_states::GUARD),
        None,
        true,
        22.,
        35.,
        Some(12.),
    );
    assert_eq!(out.requested, MAPPER_R | MAPPER_ZR);
    grab.mapped_grab(0, 2);
    let mut owner = ActionOwnership::new();
    owner.select(
        true,
        &[Intent::input_sequence(Owner::OutOfShield, out.owned)],
    );
    let mut rules = Resolved::default();
    rules
        .add(Candidate {
            slot: 2,
            index: 0,
            sequence: recipe(),
        })
        .unwrap();
    assert!(!Engine::default().preempt_automatic_grab(
        true,
        &rules,
        c,
        &targets(ESCAPE),
        None,
        &mut grab,
        &mut owner
    ));
    // The target crosses behind after R is in flight: no backup B+Up.
    // Dropping enemy shield alone is now still a legal shield-grab target.
    let mut lost = targets(0);
    lost[0].as_mut().unwrap().position[0] = -10.;
    for tick in 1..24 {
        let out = grab.sample_automatic(
            true,
            grab_context(c),
            c,
            lost,
            None,
            true,
            22.,
            35.,
            Some(12.),
        );
        assert_eq!(out.owned, 0);
        assert!(grab.grab_delivery_pending());
        for _ in 0..5 {
            grab.delivery_tick(tick);
        }
    }
    grab.delivery_tick(24);
    let out = grab.sample_automatic(
        true,
        grab_context(c),
        c,
        lost,
        None,
        true,
        22.,
        35.,
        Some(12.),
    );
    assert_eq!(
        out.owned, 0,
        "release observation cannot immediately emit backup B"
    );
    assert!(!grab.grab_delivery_pending());
}

#[test]
fn zero_input_receipt_reservation_blocks_competing_punishes_with_exclusive_neutral_output() {
    let mut grab = OutOfShield::new();
    let c = own(0., kirby_assist_core::native_states::GUARD);
    let first = grab.sample_automatic(
        true,
        grab_context(c),
        c,
        targets(kirby_assist_core::native_states::GUARD),
        None,
        true,
        22.,
        35.,
        Some(12.),
    );
    let mut owner = ActionOwnership::new();
    owner.select(
        true,
        &[Intent::input_sequence(Owner::OutOfShield, first.owned)],
    );
    grab.mapped_grab(0, 2);
    let stun = GameplayContext {
        can_act: false,
        ..own(1., kirby_assist_core::native_states::GUARD_DAMAGE)
    };
    let out = grab.sample_automatic(
        true,
        grab_context(stun),
        stun,
        targets(0),
        None,
        true,
        22.,
        35.,
        Some(12.),
    );
    assert_eq!(out, ControllerOutput::default());
    let packet =
        kirby_assist_core::action_owner::exclusive_output(Some(Owner::OutOfShield), true, out);
    assert_eq!(
        packet.owned,
        kirby_assist_core::ledge_trap::SEQUENCE_BUTTONS
    );
    assert_eq!(packet.stick, Some([0.; 2]));
    assert!(
        grab.grab_delivery_pending(),
        "renewed shield damage cannot allow queued R to become Up+B"
    );
    let receipt = Intent {
        owner: Owner::OutOfShield,
        present: grab.grab_delivery_pending(),
        committed: true,
    };
    for next in [
        Owner::RollRead,
        Owner::GrabDodge,
        Owner::OptionRead,
        Owner::JabLock,
        Owner::Fair,
        Owner::Trip,
        Owner::Ledge,
    ] {
        assert_eq!(
            owner.select(true, &[receipt, Intent::input_sequence(next, 1)]),
            Some(Owner::OutOfShield)
        );
    }
    for status in kirby_assist_core::native_states::CATCH..=kirby_assist_core::native_states::THROW
    {
        let mut delivered = OutOfShield::new();
        delivered.sample_automatic(
            true,
            grab_context(c),
            c,
            targets(kirby_assist_core::native_states::GUARD),
            None,
            true,
            22.,
            35.,
            Some(12.),
        );
        delivered.mapped_grab(0, 2);
        let caught = own(1., status);
        let out = delivered.sample_automatic(
            true,
            grab_context(caught),
            caught,
            targets(0),
            None,
            true,
            22.,
            35.,
            Some(12.),
        );
        assert_eq!(out, ControllerOutput::default());
        assert!(!delivered.grab_delivery_pending());
        assert!(delivered.target().is_none());
    }
}

#[test]
fn native_grab_cancels_every_tape_and_active_up_b_and_leaves_throw_input_manual() {
    let mut rules = Resolved::default();
    rules
        .add(Candidate {
            slot: 2,
            index: 0,
            sequence: recipe(),
        })
        .unwrap();
    for status in kirby_assist_core::native_states::CATCH..=kirby_assist_core::native_states::THROW
    {
        assert!(native_grab_active(status));
        let mut timeline = InputTimeline::new(recipe(), enemy(0., ESCAPE));
        let mut response = Response::default();
        assert!(response.start_up_b(recipe(), own(0., 0), enemy(0., ESCAPE)));
        let mut engine = Engine::default();
        assert_eq!(
            engine
                .sample_simple(true, &rules, settings(), own(0., 0), targets(ESCAPE), None)
                .requested,
            MAPPER_B
        );
        assert_eq!(
            timeline
                .sample(settings(), own(0., 0), Some(enemy(0., ESCAPE)), None, 0.)
                .requested,
            MAPPER_B
        );
        assert_eq!(
            response
                .sample(true, settings(), own(0., 0), Some(enemy(0., ESCAPE)))
                .requested,
            MAPPER_B
        );
        let release = timeline.sample(
            settings(),
            own(1., status),
            Some(enemy(1., kirby_assist_core::native_states::CAPTURE_PULLED)),
            None,
            0.,
        );
        assert_eq!(
            release,
            ControllerOutput::default(),
            "native status {status:#x}"
        );
        assert!(timeline.finished());
        assert_eq!(
            response
                .sample(
                    true,
                    settings(),
                    own(1., status),
                    Some(enemy(1., kirby_assist_core::native_states::CAPTURE_PULLED))
                )
                .owned,
            0
        );
        assert_eq!(
            engine
                .sample_simple(
                    true,
                    &rules,
                    settings(),
                    own(1., status),
                    targets(ESCAPE),
                    None
                )
                .owned,
            0
        );
        let mut overlay = ManualOverlay::default();
        overlay.apply_controls(
            ButtonTrace::default(),
            [0.; 2],
            TurboOutput {
                stick: Some([0., 1.]),
                ..Default::default()
            },
            0,
            MAPPER_B,
            kirby_assist_core::ledge_trap::SEQUENCE_BUTTONS,
        );
        let manual = overlay.apply_controls(
            ButtonTrace::default(),
            [0.8, 0.],
            TurboOutput::default(),
            0,
            release.requested,
            release.owned,
        );
        assert_eq!(manual.buttons.held & (MAPPER_B | MAPPER_R | MAPPER_ZR), 0);
        assert_eq!(manual.stick, [0.8, 0.], "physical throw direction restored");
        let mut owner = ActionOwnership::new();
        let intents = [
            Intent::input_sequence(Owner::RollRead, 1),
            Intent::input_sequence(Owner::Ledge, 1),
        ];
        assert_eq!(owner.select(!native_grab_active(status), &intents), None);
    }
}

#[test]
fn manual_input_during_confirmed_dodge_recovery_cannot_cancel_its_configured_punish() {
    let mut dodge = accepted();
    let c = GameplayContext {
        can_act: false,
        ..own(1., ESCAPE)
    };
    assert_eq!(punish(&mut dodge, c).owned, 0);
    assert!(dodge.punish_pending());
    let c = GameplayContext {
        manual_input_epoch: 42,
        ..own(2., 0)
    };
    let out = punish(&mut dodge, c);
    assert_ne!(out.requested & MAPPER_B, 0);
    assert!(dodge.punish_committed());
}
