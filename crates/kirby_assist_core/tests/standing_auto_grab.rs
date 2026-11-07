use kirby_assist_core::{
    action_owner::{ActionOwnership, Intent, Owner},
    gameplay::GameplayContext,
    ledge_trap::SEQUENCE_BUTTONS,
    native_input::*,
    out_of_shield::{Context, OutOfShield},
    roll_read::Target,
};
fn c(frame: f32, status: i32) -> Context {
    Context {
        object: 1,
        kind: kirby_assist_core::fighter_kinds::BOWSER,
        status,
        motion: 1,
        frame,
        rate: 1.,
        stopped: false,
        grounded: true,
        position: [0.; 2],
        facing: 1.,
        shield_ready: true,
        can_grab: true,
        can_jump: false,
        can_up_special: false,
        threatened: false,
    }
}
fn lc(frame: f32, status: i32) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        status,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.,
        facing: 1.,
        ..Default::default()
    }
}
fn targets(status: i32, x: f32) -> [Option<Target>; 8] {
    let mut ts = [None; 8];
    ts[0] = Some(Target {
        id: 2,
        status,
        motion: 2,
        frame: 0.,
        rate: 1.,
        position: [x, 0.],
        velocity_x: 0.,
        vulnerable_in: Some(0.),
        actionable_in: None,
    });
    ts
}
#[test]
fn shielding_vulnerable_front_targets_can_be_grabbed_with_or_without_own_shield() {
    for own in [
        0,
        kirby_assist_core::native_states::GUARD_ON,
        kirby_assist_core::native_states::GUARD,
    ] {
        for enemy in [
            kirby_assist_core::native_states::GUARD_ON,
            kirby_assist_core::native_states::GUARD,
            kirby_assist_core::native_states::GUARD_DAMAGE,
        ] {
            let out = OutOfShield::new().sample_simple(
                true,
                c(0., own),
                lc(0., own),
                targets(enemy, 22.),
                None,
                22.,
                100.,
            );
            assert_eq!(
                out.requested,
                if matches!(own, 0x1b | 0x1c) {
                    MAPPER_R | MAPPER_ZR
                } else {
                    MAPPER_R
                }
            );
            assert_eq!(out.stick, Some([0.; 2]));
            assert_eq!(out.owned, SEQUENCE_BUTTONS);
        }
    }
}
#[test]
fn out_of_range_behind_invulnerable_and_captured_targets_never_start_an_approach() {
    for (status, x, vulnerable) in [
        (0x1c, 22.01, true),
        (0x1c, -1., true),
        (0x1c, 10., false),
        (0x41, 10., true),
        (0x53, 10., true),
        (0x77, 10., true),
        (0xb5, 10., true),
    ] {
        let mut ts = targets(status, x);
        if !vulnerable {
            ts[0].as_mut().unwrap().vulnerable_in = None;
        }
        let out = OutOfShield::new().sample_simple(
            true,
            c(0., kirby_assist_core::native_states::GUARD),
            lc(0., kirby_assist_core::native_states::GUARD),
            ts,
            None,
            22.,
            100.,
        );
        assert_eq!(out.owned, 0);
        assert!(out.stick.is_none());
    }
}
#[test]
fn shieldstun_does_not_queue_an_old_attacker_and_grabs_on_first_ready_sample() {
    let mut grab = OutOfShield::new();
    let out = grab.sample_simple(
        true,
        Context {
            can_grab: false,
            shield_ready: false,
            stopped: true,
            ..c(0., kirby_assist_core::native_states::GUARD_DAMAGE)
        },
        GameplayContext {
            stopped: true,
            ..lc(0., kirby_assist_core::native_states::GUARD_DAMAGE)
        },
        targets(kirby_assist_core::native_states::GUARD, 15.),
        None,
        22.,
        35.,
    );
    assert_eq!(out.owned, 0);
    assert!(grab.target().is_none());
    let out = grab.sample_simple(
        true,
        c(1., kirby_assist_core::native_states::GUARD),
        lc(1., kirby_assist_core::native_states::GUARD),
        targets(kirby_assist_core::native_states::GUARD, 15.),
        None,
        22.,
        35.,
    );
    assert_eq!(out.requested, MAPPER_R | MAPPER_ZR);
    assert_eq!(out.stick, Some([0.; 2]));
    let out = grab.sample_simple(
        true,
        c(2., kirby_assist_core::native_states::CATCH),
        lc(2., kirby_assist_core::native_states::CATCH),
        targets(kirby_assist_core::native_states::GUARD, 15.),
        None,
        22.,
        35.,
    );
    assert_eq!(out.requested, 0);
    assert!(grab.target().is_none());
}
#[test]
fn suspension_blocks_new_automatic_work_but_committed_owners_require_cancellation() {
    let automatic = [
        Owner::OutOfShield,
        Owner::RollRead,
        Owner::GrabDodge,
        Owner::OptionRead,
        Owner::Trip,
        Owner::JabLock,
        Owner::Fair,
        Owner::Finisher,
    ];
    for current in automatic {
        let mut owner = ActionOwnership::new();
        owner.select(
            true,
            &[Intent {
                owner: current,
                present: true,
                committed: true,
            }],
        );
        owner.set_automatic_suspended(true);
        assert_eq!(owner.current_owner(), Some(current));
        assert!(owner.allows(current));
        owner.release(current);
        assert!(owner.current_owner().is_none());
        assert!(
            !owner.ledge_priority(),
            "suspension does not need an acquired ledge"
        );
        for candidate in automatic {
            assert!(!owner.allows(candidate));
        }
        for candidate in [
            Owner::Ledge,
            Owner::Safety,
            Owner::Shortcut,
            Owner::Turbo,
            Owner::Posture,
        ] {
            assert!(owner.allows(candidate));
        }
        owner.set_automatic_suspended(false);
        assert!(owner.allows(Owner::RollRead));
        assert!(owner.allows(Owner::OutOfShield));
    }
}
#[test]
fn a_committed_grab_and_roll_reaction_cannot_take_over_each_other() {
    for (first, next) in [
        (Owner::OutOfShield, Owner::RollRead),
        (Owner::RollRead, Owner::OutOfShield),
    ] {
        let mut owner = ActionOwnership::new();
        owner.select(true, &[Intent::input_sequence(first, SEQUENCE_BUTTONS)]);
        assert!(!owner.allows(next));
        assert_eq!(
            owner.select(
                true,
                &[
                    Intent::input_sequence(first, SEQUENCE_BUTTONS),
                    Intent::input_sequence(next, SEQUENCE_BUTTONS)
                ]
            ),
            Some(first)
        );
    }
}
#[test]
fn approach_range_is_visible_and_remains_parseable_for_existing_files() {
    use kirby_assist_core::fighter_tuning::{Tuning, AUTO_GRAB_CLOSE, AUTO_GRAB_FAR};
    let mut t = Tuning::default();
    t.set("auto_grab_close_range", "22").unwrap();
    t.set("auto_grab_dash_range", "80").unwrap();
    assert!(Tuning::visible(15, AUTO_GRAB_CLOSE));
    assert!(Tuning::visible(15, AUTO_GRAB_FAR));
    assert_eq!(t.auto_grab_range(15, 18.), 22.);
}

#[test]
fn unready_or_uninterruptible_grabs_never_reserve_the_controller() {
    for status in [
        kirby_assist_core::native_states::status::JUMP,
        kirby_assist_core::native_states::status::FALL,
        kirby_assist_core::native_states::status::LANDING_ATTACK_AIR,
        kirby_assist_core::native_states::status::LANDING_FALL_SPECIAL,
        kirby_assist_core::native_states::GUARD_OFF,
        kirby_assist_core::native_states::GUARD_DAMAGE,
        kirby_assist_core::native_states::status::ATTACK_LW3,
        kirby_assist_core::native_states::status::OTTOTTO,
        kirby_assist_core::native_states::status::OTTOTTO_WAIT,
    ] {
        let mut grab = OutOfShield::new();
        let out = grab.sample_simple(
            true,
            c(0., status),
            lc(0., status),
            targets(kirby_assist_core::native_states::GUARD, 10.),
            None,
            22.,
            35.,
        );
        assert_eq!(out.owned, 0, "{status:#x}");
        assert_eq!(out.stick, None);
        assert!(!kirby_assist_core::out_of_shield::automatic_grab_ready(
            status,
            |_| true
        ));
    }
    for ready in [false, true] {
        let mut grab = OutOfShield::new();
        let out = grab.sample_simple(
            true,
            Context {
                can_grab: ready,
                ..c(0., kirby_assist_core::native_states::GUARD)
            },
            lc(0., kirby_assist_core::native_states::GUARD),
            targets(kirby_assist_core::native_states::GUARD, 10.),
            None,
            22.,
            35.,
        );
        assert_eq!(out.requested, if ready { MAPPER_R | MAPPER_ZR } else { 0 });
        assert_eq!(out.owned != 0, ready, "no queue before native readiness");
    }
}

#[test]
fn a_grab_retry_cancels_if_target_leaves_range_or_becomes_invulnerable() {
    for (x, vulnerable) in [(23., true), (10., false), (-10., true)] {
        let mut grab = OutOfShield::new();
        assert_ne!(
            grab.sample_simple(
                true,
                c(0., 0),
                lc(0., 0),
                targets(kirby_assist_core::native_states::GUARD, 10.),
                None,
                22.,
                35.
            )
            .requested,
            0
        );
        let mut enemy = targets(kirby_assist_core::native_states::GUARD, x);
        if !vulnerable {
            enemy[0].as_mut().unwrap().vulnerable_in = None;
        }
        let out = grab.sample_simple(true, c(1., 0), lc(1., 0), enemy, None, 22., 35.);
        assert_eq!(out.owned, 0);
        assert!(grab.target().is_none());
    }
}

#[test]
fn grab_dodge_ignores_running_even_if_the_native_escape_term_is_enabled() {
    use kirby_assist_core::spot_dodge::{Context as DodgeContext, SpotDodge, Target as GrabTarget};
    let enemy = GrabTarget {
        id: 2,
        status: kirby_assist_core::native_states::CATCH,
        motion: 2,
        frame: 5.,
        rate: 1.,
        position: [10., 0.],
        velocity: [0.; 2],
        facing: -1.,
        first: 7,
        last: 8,
        reach: 20.,
    };
    for status in [
        kirby_assist_core::native_states::status::WAIT,
        kirby_assist_core::native_states::GUARD_ON,
        kirby_assist_core::native_states::GUARD,
        kirby_assist_core::native_states::status::WALK,
        kirby_assist_core::native_states::status::DASH,
        kirby_assist_core::native_states::status::RUN,
        kirby_assist_core::native_states::GUARD_OFF,
        kirby_assist_core::native_states::GUARD_DAMAGE,
        kirby_assist_core::native_states::status::ATTACK_LW3,
    ] {
        let context = DodgeContext {
            object: 1,
            status,
            situation: kirby_assist_core::native_states::situation::GROUND,
            motion: 1,
            frame: 0.,
            rate: 1.,
            stopped: false,
            can_spot_dodge: true,
            position: [0.; 2],
            velocity: [0.; 2],
        };
        let out = SpotDodge::default().sample(
            true,
            context,
            [Some(enemy), None, None, None, None, None, None, None],
        );
        assert_eq!(
            out.owned != 0,
            matches!(
                status,
                kirby_assist_core::native_states::status::WAIT
                    | kirby_assist_core::native_states::GUARD_ON
                    | kirby_assist_core::native_states::GUARD
            ),
            "{status:#x}"
        );
    }
}

#[test]
fn endlag_clears_grab_attempt_and_cannot_replay_an_expired_grab_threat() {
    let mut grab = OutOfShield::new();
    assert_ne!(
        grab.sample_simple(
            true,
            c(0., 0),
            lc(0., 0),
            targets(kirby_assist_core::native_states::GUARD, 10.),
            None,
            22.,
            35.
        )
        .requested,
        0
    );
    let endlag = Context {
        can_grab: false,
        ..c(1., 0x2f)
    };
    assert_eq!(
        grab.sample_simple(
            true,
            endlag,
            lc(1., 0x2f),
            targets(kirby_assist_core::native_states::GUARD, 10.),
            None,
            22.,
            35.
        )
        .owned,
        0
    );
    assert!(grab.target().is_none());
    assert_eq!(
        grab.sample_simple(true, c(2., 0), lc(2., 0), [None; 8], None, 22., 35.)
            .owned,
        0
    );
    use kirby_assist_core::spot_dodge::{Context as DodgeContext, SpotDodge, Target as GrabTarget};
    let enemy = GrabTarget {
        id: 2,
        status: kirby_assist_core::native_states::CATCH,
        motion: 2,
        frame: 5.,
        rate: 1.,
        position: [10., 0.],
        velocity: [0.; 2],
        facing: -1.,
        first: 7,
        last: 8,
        reach: 20.,
    };
    let context = DodgeContext {
        object: 1,
        status: kirby_assist_core::native_states::status::ATTACK_S4,
        situation: kirby_assist_core::native_states::situation::GROUND,
        motion: 1,
        frame: 0.,
        rate: 1.,
        stopped: false,
        can_spot_dodge: false,
        position: [0.; 2],
        velocity: [0.; 2],
    };
    let mut dodge = SpotDodge::default();
    assert_eq!(
        dodge
            .sample(
                true,
                context,
                [Some(enemy), None, None, None, None, None, None, None]
            )
            .owned,
        0
    );
    let expired = GrabTarget { frame: 9., ..enemy };
    assert_eq!(
        dodge
            .sample(
                true,
                DodgeContext {
                    status: kirby_assist_core::native_states::status::WAIT,
                    frame: 1.,
                    can_spot_dodge: true,
                    ..context
                },
                [Some(expired), None, None, None, None, None, None, None]
            )
            .owned,
        0
    );
    assert!(dodge.target().is_none());
}

#[test]
fn stationary_defense_preserves_run_intent_and_shield_roll_inputs() {
    use kirby_assist_core::manual_controls::ManualTakeover;
    let mut input = ManualTakeover::new();
    assert!(!input.stationary_assist(0));
    input.sample(0, [0.; 4]);
    assert!(input.stationary_assist(0));
    for x in [-1., -0.5, 0.5, 1.] {
        input.sample(0, [x, 0., 0., 0.]);
        assert!(!input.stationary_assist(0));
        assert!(
            !input.stationary_assist(0x1c),
            "released shield plus movement must not start a grab"
        );
        input.sample(MAPPER_ZR, [x, 0., 0., 0.]);
        assert_eq!(
            input.stationary_assist(0x1c),
            x.abs() <= 0.7,
            "shield tilt is allowed; full roll input is not intercepted"
        );
    }
    input.sample(0, [0.; 4]);
    assert!(input.stationary_assist(0));
    for button in [MAPPER_A, MAPPER_B, MAPPER_R] {
        input.sample(button, [0.; 4]);
        assert!(!input.stationary_assist(0x1c));
    }
    input.sample(MAPPER_ZR, [0., 0., 1., 0.]);
    assert!(!input.stationary_assist(0x1c));
    input.sample(MAPPER_ZR, [0.; 4]);
    assert!(input.stationary_assist(0x1c));
}

#[test]
fn grab_startup_forecast_keeps_stationary_grabs_but_rejects_escape_and_crossing() {
    for (x, speed, should_grab) in [
        (22., 0., true),
        (20., 1., false),
        (5., -2., false),
        (10., 0.5, true),
        (10., -0.5, true),
    ] {
        let mut grab = OutOfShield::new();
        let mut ts = targets(kirby_assist_core::native_states::GUARD, x);
        ts[0].as_mut().unwrap().velocity_x = speed;
        let out = grab.sample_simple_timed(
            true,
            c(0., kirby_assist_core::native_states::GUARD),
            lc(0., kirby_assist_core::native_states::GUARD),
            ts,
            None,
            22.,
            Some(12.),
        );
        assert_eq!(out.requested != 0, should_grab, "x={x} speed={speed}");
        if !should_grab {
            assert_eq!(out.owned, 0);
            assert!(grab.target().is_none());
        }
    }
}

#[test]
fn automatic_grab_finishes_with_no_controller_or_target_left_reserved() {
    let mut grab = OutOfShield::new();
    let ts = targets(kirby_assist_core::native_states::GUARD, 10.);
    for n in 0..4 {
        assert_ne!(
            grab.sample_simple(true, c(n as f32, 0), lc(n as f32, 0), ts, None, 22., 35.)
                .requested,
            0
        );
    }
    let ended = grab.sample_simple(true, c(4., 0), lc(4., 0), ts, None, 22., 35.);
    assert_eq!(ended.owned, 0);
    assert_eq!(ended.stick, None);
    assert!(grab.target().is_none());
}

#[test]
fn close_roll_up_b_is_chosen_before_new_grab_in_front_across_or_behind_shield() {
    use kirby_assist_core::{
        reaction_rules::{Candidate, Engine, Resolved},
        sequence_config::Catalog,
        settings::AssistSettings,
    };
    let cat=Catalog::parse("[bowser.roll.close]\nstart_delay=0\nreaction=auto_roll:40:12\nactions=up_b:4,drift:opponent:1:30\n").unwrap();
    let mut rules = Resolved::default();
    rules
        .add(Candidate {
            slot: 2,
            index: 0,
            sequence: cat.entries()[0].sequence,
        })
        .unwrap();
    for own in [0, kirby_assist_core::native_states::GUARD] {
        for x in [-20., 0., 20.] {
            for vulnerable in [false, true] {
                let mut engine = Engine::default();
                let mut grab = OutOfShield::new();
                let mut ts = targets(kirby_assist_core::native_states::ESCAPE_F, x);
                if !vulnerable {
                    ts[0].as_mut().unwrap().vulnerable_in = None;
                }
                let context = GameplayContext {
                    can_act: true,
                    ..lc(0., own)
                };
                let reaction = engine.sample_simple(
                    true,
                    &rules,
                    AssistSettings {
                        fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
                        ..Default::default()
                    },
                    context,
                    ts,
                    None,
                );
                assert_ne!(reaction.requested & MAPPER_B, 0, "own={own:#x} x={x}");
                let fallback = grab.sample_simple(
                    reaction.owned == 0,
                    c(0., own),
                    context,
                    ts,
                    None,
                    22.,
                    35.,
                );
                assert_eq!(
                    fallback.owned, 0,
                    "a new grab cannot compete with the selected roll reaction"
                );
            }
        }
    }
    let mut engine = Engine::default();
    let mut grab = OutOfShield::new();
    let ts = targets(kirby_assist_core::native_states::GUARD, 10.);
    let context = GameplayContext {
        can_act: true,
        ..lc(0., kirby_assist_core::native_states::GUARD)
    };
    let reaction = engine.sample_simple(true, &rules, AssistSettings::default(), context, ts, None);
    assert_eq!(reaction.owned, 0);
    assert_eq!(
        grab.sample_simple(
            true,
            c(0., kirby_assist_core::native_states::GUARD),
            context,
            ts,
            None,
            22.,
            35.
        )
        .requested,
        MAPPER_R | MAPPER_ZR,
        "stationary shield grabs stay unchanged"
    );
}

#[test]
fn a_failed_pending_grab_releases_for_a_same_observation_roll_fallback() {
    use kirby_assist_core::{
        reaction_rules::{Candidate, Engine, Resolved},
        sequence_config::Catalog,
        settings::AssistSettings,
    };
    let cat=Catalog::parse("[bowser.roll.fallback]\nstart_delay=0\nreaction=auto_roll:40:12\nactions=up_b:4,drift:opponent:1:30\n").unwrap();
    let mut rules = Resolved::default();
    rules
        .add(Candidate {
            slot: 2,
            index: 0,
            sequence: cat.entries()[0].sequence,
        })
        .unwrap();
    let mut owner = ActionOwnership::new();
    let mut grab = OutOfShield::new();
    let mut engine = Engine::default();
    let started = grab.sample_simple(
        true,
        c(0., kirby_assist_core::native_states::GUARD),
        lc(0., kirby_assist_core::native_states::GUARD),
        targets(kirby_assist_core::native_states::GUARD, 10.),
        None,
        22.,
        35.,
    );
    owner.select(
        true,
        &[Intent::input_sequence(Owner::OutOfShield, started.owned)],
    );
    let rolled = targets(kirby_assist_core::native_states::ESCAPE_F, -10.);
    let context = GameplayContext {
        can_act: true,
        ..lc(1., kirby_assist_core::native_states::GUARD)
    };
    let rejected = grab.sample_simple(
        true,
        c(1., kirby_assist_core::native_states::GUARD),
        context,
        rolled,
        None,
        22.,
        35.,
    );
    assert_eq!(rejected.owned, 0);
    assert!(grab.target().is_none());
    owner.release(Owner::OutOfShield);
    let out = engine.sample_simple(
        owner.allows(Owner::RollRead),
        &rules,
        AssistSettings::default(),
        context,
        rolled,
        None,
    );
    assert_ne!(
        out.requested & MAPPER_B,
        0,
        "no extra frame between failed grab and usable Up-B"
    );
    assert_eq!(
        owner.select(true, &[Intent::input_sequence(Owner::RollRead, out.owned)]),
        Some(Owner::RollRead)
    );
}

#[test]
fn frozen_shields_cannot_hold_an_unaccepted_grab_forever_and_duplicate_polls_do_not_count() {
    let mut grab = OutOfShield::new();
    let mut shield = c(0., kirby_assist_core::native_states::GUARD);
    shield.rate = 0.;
    let context = GameplayContext {
        motion_rate: 0.,
        ..lc(0., kirby_assist_core::native_states::GUARD)
    };
    let mut ts = targets(kirby_assist_core::native_states::GUARD, 10.);
    ts[0].as_mut().unwrap().rate = 0.;
    for tick in 0..4 {
        for _ in 0..20 {
            assert_ne!(
                grab.sample_simple(true, shield, context, ts, None, 22., 35.)
                    .requested,
                0
            );
        }
        grab.mapped_grab(tick, 999); // Unrelated/stale acknowledgment must not count.
        grab.mapped_grab(tick, 2);
        grab.mapped_grab(tick, 2);
    }
    let ended = grab.sample_simple(true, shield, context, ts, None, 22., 35.);
    assert_eq!(ended.owned, 0);
    assert_eq!(ended.stick, None);
    assert!(grab.grab_delivery_pending());
    for tick in 0..24 {
        grab.delivery_tick(tick);
        grab.delivery_tick(tick);
    }
    assert_eq!(
        grab.sample_simple(true, shield, context, ts, None, 22., 35.)
            .owned,
        0
    );
    assert!(grab.target().is_none());
}

#[test]
fn unshielded_grabs_require_enemy_shield_but_shielded_grabs_accept_other_vulnerable_fronts() {
    for status in [
        kirby_assist_core::native_states::status::WAIT,
        kirby_assist_core::native_states::status::DASH,
        kirby_assist_core::native_states::status::GUARD_ON,
        kirby_assist_core::native_states::status::GUARD,
        kirby_assist_core::native_states::status::GUARD_OFF,
        kirby_assist_core::native_states::status::GUARD_DAMAGE,
        kirby_assist_core::native_states::status::ESCAPE,
        kirby_assist_core::native_states::status::ESCAPE_F,
        kirby_assist_core::native_states::status::ESCAPE_B,
        kirby_assist_core::native_states::status::ESCAPE_AIR,
        kirby_assist_core::native_states::status::ESCAPE_AIR_SLIDE,
        kirby_assist_core::native_states::status::ATTACK_LW3,
        kirby_assist_core::native_states::status::CATCH,
        kirby_assist_core::native_states::status::DAMAGE,
    ] {
        for shielding in [false, true] {
            for vulnerable in [false, true] {
                for side in [-1., 1.] {
                    let mut grab = OutOfShield::new();
                    let own = if shielding { 0x1c } else { 0 };
                    let context = GameplayContext {
                        safe_ground: [true; 2],
                        ..lc(0., own)
                    };
                    let mut ts = targets(status, side * 10.);
                    if !vulnerable {
                        ts[0].as_mut().unwrap().vulnerable_in = None;
                    }
                    let out = grab.sample_automatic(
                        true,
                        c(0., own),
                        context,
                        ts,
                        None,
                        shielding,
                        22.,
                        35.,
                        Some(12.),
                    );
                    assert_eq!(out.owned!=0,vulnerable&&side>0.&&(shielding||matches!(status,kirby_assist_core::native_states::GUARD_ON|kirby_assist_core::native_states::GUARD|kirby_assist_core::native_states::GUARD_DAMAGE)),"enemy={status:#x} self-shield={shielding} vulnerable={vulnerable} side={side}");
                }
            }
        }
    }
    for status in [
        -kirby_assist_core::native_states::status::WALK,
        kirby_assist_core::native_states::status::CAPTURE_PULLED,
        kirby_assist_core::native_states::status::CAPTURE_WAIT,
        kirby_assist_core::native_states::status::THROWN,
        kirby_assist_core::native_states::status::DOWN,
        kirby_assist_core::native_states::status::DOWN_WAIT,
        kirby_assist_core::native_states::status::DOWN_EAT,
        kirby_assist_core::native_states::status::CLIFF_CATCH_MOVE,
        kirby_assist_core::native_states::status::CLIFF_WAIT,
        kirby_assist_core::native_states::status::CLIFF_CLIMB,
        kirby_assist_core::native_states::status::CLIFF_JUMP3,
        kirby_assist_core::native_states::status::DEAD,
        kirby_assist_core::native_states::status::REBIRTH,
        kirby_assist_core::native_states::status::ENTRY,
        0x1da,
        kirby_assist_core::native_states::status::LOSE,
    ] {
        let mut grab = OutOfShield::new();
        let out = grab.sample_automatic(
            true,
            c(0., 0),
            GameplayContext {
                safe_ground: [true; 2],
                ..lc(0., 0)
            },
            targets(status, 10.),
            None,
            false,
            22.,
            35.,
            Some(12.),
        );
        assert_eq!(out.owned, 0, "nongrabbable state {status:#x}");
    }
}

#[test]
fn approach_is_one_delivered_pulse_then_four_grab_packets_and_never_dashes_from_shield() {
    for side in [-1., 1.] {
        let mut grab = OutOfShield::new();
        let ts = targets(kirby_assist_core::native_states::GUARD, side * 30.);
        let context = GameplayContext {
            facing: side,
            safe_ground: [true; 2],
            ..lc(0., 0)
        };
        for _ in 0..10 {
            let o = grab.sample_automatic(
                true,
                Context {
                    facing: side,
                    ..c(0., 0)
                },
                context,
                ts,
                None,
                false,
                22.,
                35.,
                Some(8.),
            );
            assert_eq!(o.requested, 0);
            assert_eq!(o.stick, Some([side, 0.]));
        }
        grab.mapped_approach(0, 999);
        assert!(grab.approach_pending());
        grab.mapped_approach(0, 2);
        grab.mapped_approach(0, 2);
        assert!(
            grab.approach_pending(),
            "far Bowser grab still needs native dash confirmation"
        );
        for tick in 1..=4 {
            let mut running = Context {
                facing: side,
                ..c(0., 3)
            };
            running.can_grab = false;
            running.shield_ready = false;
            let o = grab.sample_automatic(
                true,
                running,
                GameplayContext {
                    status: kirby_assist_core::native_states::status::DASH,
                    ..context
                },
                ts,
                None,
                false,
                22.,
                35.,
                Some(8.),
            );
            assert_eq!(o.requested, MAPPER_R);
            assert_eq!(o.stick, Some([0.; 2]));
            assert_eq!(grab.with_grab_movement(o, [side, 0.8]).stick, Some([0.; 2]));
            grab.mapped_grab(tick, 2);
            grab.mapped_grab(tick, 2);
        }
        let o = grab.sample_automatic(true, c(0., 0), context, ts, None, false, 22., 35., Some(8.));
        assert_eq!(o.owned, 0);
        assert!(grab.grab_delivery_pending());
    }
    for own in [
        0,
        kirby_assist_core::native_states::GUARD_ON,
        kirby_assist_core::native_states::GUARD,
    ] {
        // Includes a fresh physical shield before native WAIT changes.
        let mut grab = OutOfShield::new();
        let context = lc(0., own);
        assert_eq!(
            grab.sample_automatic(
                true,
                c(0., own),
                context,
                targets(kirby_assist_core::native_states::GUARD, 30.),
                None,
                true,
                22.,
                35.,
                Some(12.)
            )
            .owned,
            0
        );
        let o = grab.sample_automatic(
            true,
            c(0., own),
            context,
            targets(kirby_assist_core::native_states::GUARD, 20.),
            None,
            true,
            22.,
            35.,
            Some(12.),
        );
        assert_ne!(o.requested & MAPPER_R, 0);
        assert_eq!(o.stick, Some([0.; 2]));
        assert!(!grab.approach_pending());
    }
}

#[test]
fn approach_obeys_35_range_ground_support_shield_takeover_and_live_target_state() {
    for (distance, safe, expected) in [(35., true, true), (35.01, true, false), (30., false, false)]
    {
        let mut grab = OutOfShield::new();
        let context = GameplayContext {
            safe_ground: [safe; 2],
            ..lc(0., 0)
        };
        let o = grab.sample_automatic(
            true,
            c(0., 0),
            context,
            targets(kirby_assist_core::native_states::GUARD, distance),
            None,
            false,
            22.,
            35.,
            Some(8.),
        );
        assert_eq!(o.owned != 0, expected);
    }
    let mut grab = OutOfShield::new();
    let context = GameplayContext {
        safe_ground: [true; 2],
        ..lc(0., 0)
    };
    assert_ne!(
        grab.sample_automatic(
            true,
            c(0., 0),
            context,
            targets(kirby_assist_core::native_states::GUARD, 30.),
            None,
            false,
            22.,
            35.,
            Some(8.)
        )
        .owned,
        0
    );
    assert_eq!(
        grab.sample_automatic(
            true,
            c(0., 0),
            context,
            targets(kirby_assist_core::native_states::GUARD, 30.),
            None,
            true,
            22.,
            35.,
            Some(12.)
        )
        .owned,
        0,
        "fresh shield cannot retain the far approach"
    );
    let mut grab = OutOfShield::new();
    grab.sample_automatic(
        true,
        c(0., 0),
        context,
        targets(kirby_assist_core::native_states::GUARD, 30.),
        None,
        false,
        22.,
        35.,
        Some(8.),
    );
    grab.mapped_approach(0, 2);
    assert_ne!(
        grab.sample_automatic(
            true,
            c(1., 3),
            context,
            targets(kirby_assist_core::native_states::GUARD, 30.),
            None,
            false,
            22.,
            35.,
            Some(8.)
        )
        .requested,
        0,
        "a shielding target remains eligible"
    );
    let mut invalid = targets(0x2c, 30.);
    invalid[0].as_mut().unwrap().vulnerable_in = None;
    assert_eq!(
        grab.sample_automatic(
            true,
            c(2., 3),
            context,
            invalid,
            None,
            false,
            22.,
            35.,
            Some(8.)
        )
        .owned,
        0
    );
    assert!(grab.target().is_none());
}

fn automatic_roll_rules(actions: &str) -> kirby_assist_core::reaction_rules::Resolved {
    use kirby_assist_core::{
        reaction_rules::{Candidate, Resolved},
        sequence_config::Catalog,
    };
    let catalog = Catalog::parse(&format!(
        "[bowser.roll.priority]\nstart_delay=0\nreaction=auto_roll:40:12\nactions={actions}\n",
    ))
    .unwrap();
    let mut rules = Resolved::default();
    rules
        .add(Candidate {
            slot: 2,
            index: 0,
            sequence: catalog.entries()[0].sequence,
        })
        .unwrap();
    rules
}

#[test]
fn usable_up_b_wins_over_an_uncommitted_grab_proposal_without_mixed_input() {
    use kirby_assist_core::{
        manual_controls::{ManualOverlay, TurboOutput},
        reaction_rules::Engine,
        settings::AssistSettings,
    };
    let rules = automatic_roll_rules("up_b:4,drift:opponent:1:30");
    for shielded in [false, true] {
        for other_target in [false, true] {
            let own = if shielded { 0x1c } else { 0 };
            let context = GameplayContext {
                can_act: true,
                safe_ground: [true; 2],
                ..lc(0., own)
            };
            let mut grab = OutOfShield::new();
            let started = grab.sample_automatic(
                true,
                c(0., own),
                context,
                targets(kirby_assist_core::native_states::GUARD, 10.),
                None,
                shielded,
                22.,
                35.,
                Some(8.),
            );
            assert_ne!(started.owned, 0);
            // Both are proposals; neither has acquired the controller yet.
            let mut owner = ActionOwnership::new();
            let mut ts = targets(kirby_assist_core::native_states::ESCAPE_F, 10.);
            if other_target {
                ts[1] = ts[0].map(|t| Target {
                    id: 3,
                    position: [-20., 0.],
                    ..t
                });
                ts[0].as_mut().unwrap().status = kirby_assist_core::native_states::status::WAIT;
            }
            let mut engine = Engine::default();
            assert!(engine
                .preempt_automatic_grab(true, &rules, context, &ts, None, &mut grab, &mut owner));
            assert!(grab.target().is_none());
            assert!(owner.allows(Owner::RollRead));
            let out =
                engine.sample_simple(true, &rules, AssistSettings::default(), context, ts, None);
            assert_ne!(
                out.requested & MAPPER_B,
                0,
                "Up-B starts in the handoff observation"
            );
            assert_eq!(out.requested & MAPPER_R, 0);
            assert_eq!(out.stick, Some([0., 1.]));
            owner.select(true, &[Intent::input_sequence(Owner::RollRead, out.owned)]);
            assert!(!owner.allows(Owner::OutOfShield));
            assert_eq!(
                grab.sample_automatic(
                    false,
                    c(0., own),
                    context,
                    ts,
                    None,
                    shielded,
                    22.,
                    35.,
                    Some(8.)
                )
                .owned,
                0
            );
            let mut overlay = ManualOverlay::default();
            overlay.apply_controls(
                ButtonTrace::default(),
                [0.; 2],
                TurboOutput::default(),
                0,
                MAPPER_R,
                SEQUENCE_BUTTONS,
            );
            let mapped = overlay.apply_reaction_controls(
                ButtonTrace::default(),
                [0.; 2],
                TurboOutput {
                    stick: out.stick,
                    ..Default::default()
                },
                0,
                out.requested,
                out.owned,
                true,
            );
            assert_eq!(
                mapped.buttons.held & MAPPER_R,
                0,
                "previous synthetic Grab is released"
            );
            assert_ne!(mapped.buttons.held & MAPPER_B, 0);
        }
    }
}

#[test]
fn priority_handoff_never_cancels_native_grabs_or_bypasses_other_modes_and_readiness() {
    use kirby_assist_core::{reaction_rules::Engine, settings::AssistSettings};
    for case in 0..14 {
        let rules = if case == 12 {
            use kirby_assist_core::{
                reaction_rules::{Candidate, Resolved},
                sequence_config::Catalog,
            };
            let cat = Catalog::parse(
                "[bowser.roll.delayed]\nstart_delay=5\nreaction=auto_roll:40:12\nactions=up_b:4\n",
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
            rules
        } else {
            automatic_roll_rules(if case == 8 { "grab" } else { "up_b:4" })
        };
        let mut engine = Engine::default();
        let mut grab = OutOfShield::new();
        let mut owner = ActionOwnership::new();
        let started = grab.sample_simple(
            true,
            c(0., kirby_assist_core::native_states::GUARD),
            lc(0., kirby_assist_core::native_states::GUARD),
            targets(kirby_assist_core::native_states::GUARD, 10.),
            None,
            22.,
            35.,
        );
        assert_ne!(started.owned, 0);
        let mut context = GameplayContext {
            can_act: true,
            ..lc(1., kirby_assist_core::native_states::GUARD)
        };
        let mut ts = targets(kirby_assist_core::native_states::ESCAPE_F, 10.);
        match case {
            0 => context.stopped = true,
            1 => context.can_act = false,
            2 => context.status = kirby_assist_core::native_states::CATCH, // Native catch accepted: never replace it.
            3 => ts[0].as_mut().unwrap().position[0] = 41.,
            4 => {
                engine.sample_simple(false, &rules, AssistSettings::default(), context, ts, None);
            }
            5 => {
                owner.select(
                    true,
                    &[Intent::input_sequence(Owner::Safety, SEQUENCE_BUTTONS)],
                );
            }
            6 => owner.set_ledge_priority(true),
            7 => owner.set_automatic_suspended(true),
            8 => {} // Ordinary configured attacks cannot preempt a pending grab.
            9 => ts[0].as_mut().unwrap().frame = f32::NAN,
            10 => context.gameplay = false,
            13 => {
                owner.select(
                    true,
                    &[Intent::input_sequence(Owner::OutOfShield, started.owned)],
                );
            }
            _ => {} // Disabled or delayed reaction mode.
        }
        assert!(
            !engine.preempt_automatic_grab(
                case != 11,
                &rules,
                context,
                &ts,
                None,
                &mut grab,
                &mut owner
            ),
            "case {case}"
        );
        assert!(
            grab.target().is_some(),
            "rejected priority proposal does not mutate the grab"
        );
        if case == 2 {
            assert_eq!(
                grab.sample_simple(
                    true,
                    c(1., kirby_assist_core::native_states::CATCH),
                    context,
                    targets(kirby_assist_core::native_states::GUARD, 10.),
                    None,
                    22.,
                    35.
                )
                .owned,
                0
            );
            assert!(
                grab.target().is_none(),
                "accepted native grab releases normally"
            );
        }
    }
}

#[test]
fn approach_cancels_before_grab_when_live_target_crosses_disappears_or_becomes_ungrabbable() {
    let context = GameplayContext {
        safe_ground: [true; 2],
        ..lc(0., 0)
    };
    for next in [
        targets(0, -1.),
        targets(0, 36.),
        targets(kirby_assist_core::native_states::CAPTURE_PULLED, 10.),
        [None; 8],
    ] {
        let mut grab = OutOfShield::new();
        let started = grab.sample_automatic(
            true,
            c(0., 0),
            context,
            targets(kirby_assist_core::native_states::GUARD, 30.),
            None,
            false,
            22.,
            35.,
            Some(8.),
        );
        assert_ne!(started.owned, 0);
        grab.mapped_approach(0, 2);
        let out = grab.sample_automatic(
            true,
            c(1., 3),
            GameplayContext {
                status: kirby_assist_core::native_states::status::DASH,
                ..context
            },
            next,
            None,
            false,
            22.,
            35.,
            Some(8.),
        );
        assert_eq!(out.owned, 0);
        assert_eq!(out.requested, 0);
        assert!(grab.target().is_none());
    }
}
