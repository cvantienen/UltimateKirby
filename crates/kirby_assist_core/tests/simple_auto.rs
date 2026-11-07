use kirby_assist_core::{
    gameplay::{GameplayContext, MotionTiming, Opponent},
    ledge_trap::LedgeSettings,
    native_input::*,
    reaction_rules::{Candidate, Engine, Resolved},
    roll_read::Target,
    sequence_config::Catalog,
};
fn c(n: f32) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        can_act: true,
        situation: 0,
        status: kirby_assist_core::native_states::status::WAIT,
        motion_kind: 1,
        motion_frame: n,
        motion_rate: 1.,
        facing: 1.,
        safe_ground: [true; 2],
        ..Default::default()
    }
}
fn t(status: i32, x: f32, n: f32) -> Target {
    Target {
        id: 2,
        status,
        motion: 2,
        frame: n,
        rate: 1.,
        position: [x, 0.],
        velocity_x: 0.,
        vulnerable_in: Some(999.),
        actionable_in: None,
    }
}
fn follow(t: Target) -> Opponent {
    Opponent {
        id: t.id,
        status: t.status,
        position: t.position,
        motion: Some(MotionTiming {
            kind: t.motion,
            frame: t.frame,
            rate: t.rate,
            remaining: None,
        }),
        ..Default::default()
    }
}
fn rules() -> Resolved {
    let mut source = String::new();
    for (event, x, y) in [
        ("auto_roll", 30, 10),
        ("auto_spot_dodge", 30, 10),
        ("auto_air_dodge", 40, 20),
    ] {
        source.push_str(&format!("[bowser.roll.{event}]\nstart_delay=0\ntimeout=180\nreaction={event}:{x}:{y}\nactions=up_b:4,drift:opponent:1:65\n"));
    }
    let cat = Catalog::parse(&source).unwrap();
    let mut r = Resolved::default();
    for (i, e) in cat.entries().iter().enumerate() {
        r.add(Candidate {
            slot: 2,
            index: i,
            sequence: e.sequence,
        })
        .unwrap();
    }
    r
}
fn sample(
    e: &mut Engine,
    r: &Resolved,
    n: f32,
    status: i32,
    x: f32,
    allowed: bool,
    own: GameplayContext,
) -> kirby_assist_core::gameplay::ControllerOutput {
    let target = t(status, x, n);
    let mut ts = [None; 8];
    ts[0] = Some(target);
    e.sample_simple(
        allowed,
        r,
        LedgeSettings::default(),
        own,
        ts,
        Some(follow(target)),
    )
}
#[test]
fn any_dodge_starts_near_up_b_without_vulnerability_or_facing_priority_checks() {
    let r = rules();
    for status in [
        kirby_assist_core::native_states::ESCAPE,
        kirby_assist_core::native_states::ESCAPE_F,
        kirby_assist_core::native_states::ESCAPE_B,
        kirby_assist_core::native_states::status::ESCAPE_AIR,
        kirby_assist_core::native_states::status::ESCAPE_AIR_SLIDE,
    ] {
        for own in [
            0,
            1,
            3,
            4,
            kirby_assist_core::native_states::GUARD,
            0x7e,
            0x7f,
        ] {
            let mut e = Engine::default();
            let out = sample(
                &mut e,
                &r,
                0.,
                status,
                -25.,
                true,
                GameplayContext {
                    status: own,
                    ..c(0.)
                },
            );
            assert_eq!(
                out.requested,
                MAPPER_B | if own == 0x1c { MAPPER_ZR } else { 0 },
                "event {status} own {own}"
            );
            assert_eq!(out.stick, Some([0., 1.]));
        }
    }
}

#[test]
fn airborne_and_ledge_getup_roll_cannot_start_any_response() {
    let r = rules();
    for (status, own) in [
        (0x7a, c(0.)),
        (
            0x20,
            GameplayContext {
                situation: 2,
                ..c(0.)
            },
        ),
    ] {
        let mut e = Engine::default();
        assert_eq!(sample(&mut e, &r, 0., status, 20., true, own).owned, 0);
    }
    let mut e = Engine::default();
    sample(
        &mut e,
        &r,
        0.,
        kirby_assist_core::native_states::ESCAPE_F,
        20.,
        true,
        c(0.),
    );
    assert_eq!(
        sample(
            &mut e,
            &r,
            1.,
            0x20,
            20.,
            true,
            GameplayContext {
                situation: 2,
                ..c(1.)
            }
        )
        .owned,
        0,
        "unconfirmed grounded input must not start Up-B after leaving the floor"
    );
}
#[test]
fn committed_response_ignores_new_events_and_does_not_queue_them() {
    let r = rules();
    let mut e = Engine::default();
    sample(
        &mut e,
        &r,
        0.,
        kirby_assist_core::native_states::ESCAPE_F,
        20.,
        true,
        c(0.),
    );
    for n in 1..70 {
        sample(
            &mut e,
            &r,
            n as f32,
            kirby_assist_core::native_states::ESCAPE,
            20.,
            true,
            c(n as f32),
        );
    }
    assert!(!e.committed());
    assert_eq!(
        sample(
            &mut e,
            &r,
            70.,
            kirby_assist_core::native_states::ESCAPE,
            20.,
            true,
            c(70.)
        )
        .owned,
        0
    );
}
#[test]
fn fixed_grab_mode_only_sends_standing_grab_inside_close_range() {
    use kirby_assist_core::out_of_shield::{Context, OutOfShield};
    for distance in [22., 35.] {
        let mut a = OutOfShield::new();
        let native = Context {
            object: 1,
            kind: kirby_assist_core::fighter_kinds::BOWSER,
            status: kirby_assist_core::native_states::status::WAIT,
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
            can_up_special: false,
            threatened: false,
        };
        let mut targets = [None; 8];
        targets[0] = Some(Target {
            vulnerable_in: Some(0.),
            ..t(0x1c, distance, 0.)
        });
        let out = a.sample_simple(true, native, c(0.), targets, None, 22., 35.);
        assert_eq!(out.requested, if distance <= 22. { MAPPER_R } else { 0 });
        assert_eq!(
            out.stick,
            if distance <= 22. { Some([0.; 2]) } else { None }
        );
    }
}

#[test]
fn frozen_enemy_animation_does_not_delay_observed_dodge_response() {
    let r = rules();
    let mut e = Engine::default();
    let target = Target {
        rate: 0.,
        ..t(0x20, 20., 0.)
    };
    let mut ts = [None; 8];
    ts[0] = Some(target);
    assert_eq!(
        e.sample_simple(
            true,
            &r,
            LedgeSettings::default(),
            c(0.),
            ts,
            Some(follow(target))
        )
        .requested,
        MAPPER_B
    );
}
#[test]
fn duplicate_event_configs_are_rejected_instead_of_competing_by_priority() {
    let r="[bowser.roll.a]\nstart_delay=0\nreaction=auto_roll:80:30\nactions=up_b:4\n[bowser.roll.b]\nstart_delay=0\nreaction=auto_roll:30:30\nactions=grab:4\n";
    let catalog = Catalog::parse(r).unwrap();
    assert!(catalog
        .validate(&kirby_assist_core::fighter_profiles::FighterProfiles::default())
        .is_err());
}
#[test]
fn confirmed_shield_contact_grabs_only_after_shieldstun_and_only_front_vulnerable() {
    use kirby_assist_core::out_of_shield::{Context, OutOfShield};
    let mut a = OutOfShield::new();
    for (n, status, ready, expected) in [
        (0, kirby_assist_core::native_states::GUARD_DAMAGE, false, 0),
        (
            1,
            kirby_assist_core::native_states::GUARD,
            true,
            MAPPER_R | MAPPER_ZR,
        ),
    ] {
        let native = Context {
            object: 1,
            kind: kirby_assist_core::fighter_kinds::BOWSER,
            status,
            motion: 1,
            frame: n as f32,
            rate: 1.,
            stopped: false,
            grounded: true,
            position: [0.; 2],
            facing: 1.,
            shield_ready: ready,
            can_grab: ready,
            can_jump: false,
            can_up_special: false,
            threatened: false,
        };
        let enemy = Opponent {
            vulnerable: true,
            ..follow(t(0x1c, 20., n as f32))
        };
        assert_eq!(
            a.sample_simple(
                true,
                native,
                GameplayContext {
                    status,
                    ..c(n as f32)
                },
                [None; 8],
                Some(enemy),
                22.,
                35.
            )
            .requested,
            expected
        );
    }
}
#[test]
fn old_far_grab_range_never_drops_shield_or_drives_an_approach() {
    use kirby_assist_core::out_of_shield::{Context, OutOfShield};
    let mut a = OutOfShield::new();
    for n in 0..8 {
        let native = Context {
            object: 1,
            kind: kirby_assist_core::fighter_kinds::BOWSER,
            status: kirby_assist_core::native_states::GUARD,
            motion: 1,
            frame: n as f32,
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
        };
        let mut targets = [None; 8];
        targets[0] = Some(Target {
            vulnerable_in: Some(0.),
            ..t(0x1c, 35., n as f32)
        });
        let out = a.sample_simple(
            true,
            native,
            GameplayContext {
                status: kirby_assist_core::native_states::GUARD,
                ..c(n as f32)
            },
            targets,
            None,
            22.,
            80.,
        );
        assert_eq!(out.owned, 0);
        assert_eq!(out.requested, 0);
        assert_eq!(out.stick, None);
    }
}

#[test]
fn regrabbing_ledge_stops_an_already_committed_auto_response() {
    let r = rules();
    let mut engine = Engine::default();
    sample(
        &mut engine,
        &r,
        0.,
        kirby_assist_core::native_states::ESCAPE_F,
        20.,
        true,
        c(0.),
    );
    let hanging = Opponent {
        status: kirby_assist_core::native_states::status::CLIFF_WAIT,
        ..follow(t(0x20, 20., 1.))
    };
    assert_eq!(
        engine
            .sample_simple(
                true,
                &r,
                LedgeSettings::default(),
                c(1.),
                [None; 8],
                Some(hanging)
            )
            .owned,
        0
    );
    assert!(!engine.committed());
}

#[test]
fn shield_grab_readiness_uses_guard_even_when_ordinary_catch_term_is_disabled() {
    use kirby_assist_core::out_of_shield::automatic_grab_ready;
    assert!(automatic_grab_ready(0x1c, |_| panic!(
        "held guard needs no ground-catch query"
    )));
    assert!(!automatic_grab_ready(0, |_| false));
    assert!(automatic_grab_ready(0, |_| true));
}

#[test]
fn unavailable_up_b_never_reserves_controls_or_queues_a_finished_enemy_roll() {
    let r = rules();
    for own in [
        kirby_assist_core::native_states::ESCAPE,
        kirby_assist_core::native_states::ESCAPE_F,
        kirby_assist_core::native_states::ESCAPE_B,
        kirby_assist_core::native_states::GUARD_ON,
        kirby_assist_core::native_states::GUARD_OFF,
        kirby_assist_core::native_states::GUARD_DAMAGE,
        0x2c,
    ] {
        let mut e = Engine::default();
        for n in 0..7 {
            let out = sample(
                &mut e,
                &r,
                n as f32,
                0x20,
                20.,
                true,
                GameplayContext {
                    status: own,
                    can_act: false,
                    ..c(n as f32)
                },
            );
            assert_eq!(out.owned, 0);
            assert_eq!(out.stick, None);
            assert!(!e.committed());
        }
        assert_eq!(
            sample(&mut e, &r, 7., 0, 20., true, c(7.)).owned,
            0,
            "old roll is not queued"
        );
        assert_eq!(
            sample(
                &mut e,
                &r,
                8.,
                kirby_assist_core::native_states::ESCAPE_F,
                20.,
                true,
                c(8.)
            )
            .requested,
            MAPPER_B,
            "fresh current roll reacts immediately"
        );
    }
}

#[test]
fn held_shield_grabs_an_approaching_shield_and_releases_synthetic_guard_on_catch() {
    use kirby_assist_core::out_of_shield::{automatic_grab_ready, Context, OutOfShield};
    let mut a = OutOfShield::new();
    for (n, status, expected) in [
        (0, kirby_assist_core::native_states::ESCAPE_F, 0),
        (
            1,
            kirby_assist_core::native_states::GUARD,
            MAPPER_R | MAPPER_ZR,
        ),
        (2, kirby_assist_core::native_states::CATCH, 0),
    ] {
        let lc = GameplayContext {
            status,
            ..c(n as f32)
        };
        let native = Context {
            object: 1,
            kind: kirby_assist_core::fighter_kinds::BOWSER,
            status,
            motion: 1,
            frame: n as f32,
            rate: 1.,
            stopped: false,
            grounded: true,
            position: [0.; 2],
            facing: 1.,
            shield_ready: true,
            can_grab: automatic_grab_ready(status, |_| false),
            can_jump: false,
            can_up_special: false,
            threatened: false,
        };
        let mut targets = [None; 8];
        targets[0] = Some(Target {
            vulnerable_in: Some(0.),
            ..t(0x1c, 22., n as f32)
        });
        let out = a.sample_simple(true, native, lc, targets, None, 22., 35.);
        assert_eq!(out.requested, expected);
        if n == 1 {
            assert_eq!(out.owned, kirby_assist_core::ledge_trap::SEQUENCE_BUTTONS);
        }
        if n == 2 {
            assert_eq!(out.owned, 0);
            assert_eq!(out.stick, None);
        }
    }
}

#[test]
fn exact_event_ranges_and_heights_have_no_far_approach() {
    let r = rules();
    for (status, x, y) in [
        (kirby_assist_core::native_states::ESCAPE_F, 30., 10.),
        (kirby_assist_core::native_states::ESCAPE, 30., 10.),
        (0x22, 40., 20.),
    ] {
        for (dx, dy, expected) in [
            (x, y, true),
            (-x, -y, true),
            (x + 0.01, 0., false),
            (0., y + 0.01, false),
        ] {
            let mut engine = Engine::default();
            let target = Target {
                position: [dx, dy],
                ..t(status, dx, 0.)
            };
            let mut targets = [None; 8];
            targets[0] = Some(target);
            let out = engine.sample_simple(
                true,
                &r,
                LedgeSettings::default(),
                c(0.),
                targets,
                Some(follow(target)),
            );
            assert_eq!(out.requested, if expected { MAPPER_B } else { 0 });
            assert_eq!(engine.committed(), expected);
        }
    }
}
#[test]
fn preempted_event_is_not_replayed_when_ledge_priority_ends() {
    let r = rules();
    let mut e = Engine::default();
    assert_eq!(
        sample(
            &mut e,
            &r,
            0.,
            kirby_assist_core::native_states::ESCAPE_F,
            20.,
            false,
            c(0.)
        )
        .owned,
        0
    );
    assert_eq!(
        sample(
            &mut e,
            &r,
            1.,
            kirby_assist_core::native_states::ESCAPE_F,
            20.,
            true,
            c(1.)
        )
        .owned,
        0
    );
}
#[test]
fn drift_keeps_exclusive_ownership_across_interior_clock_gap_and_air_transition() {
    let r = rules();
    let mut e = Engine::default();
    for n in 0..5 {
        sample(
            &mut e,
            &r,
            n as f32,
            0x20,
            20.,
            true,
            GameplayContext {
                status: if n == 0 {
                    0
                } else {
                    kirby_assist_core::native_states::status::SPECIAL_HI
                },
                ..c(n as f32)
            },
        );
    }
    let out = sample(
        &mut e,
        &r,
        7.,
        0,
        20.,
        true,
        GameplayContext {
            status: kirby_assist_core::native_states::status::SPECIAL_HI,
            situation: 2,
            ..c(7.)
        },
    );
    assert_eq!(out.owned, kirby_assist_core::ledge_trap::SEQUENCE_BUTTONS);
    assert_eq!(out.stick, Some([1., 0.]));
    assert!(e.committed());
    // At a crossed B-release boundary, begin drift now without skipping its hold.
    let mut e = Engine::default();
    for n in 0..4 {
        sample(
            &mut e,
            &r,
            n as f32,
            0x20,
            20.,
            true,
            GameplayContext {
                status: if n == 0 {
                    0
                } else {
                    kirby_assist_core::native_states::status::SPECIAL_HI
                },
                ..c(n as f32)
            },
        );
    }
    let resumed = sample(
        &mut e,
        &r,
        5.,
        0,
        20.,
        true,
        GameplayContext {
            status: kirby_assist_core::native_states::status::SPECIAL_HI,
            ..c(5.)
        },
    );
    assert!(e.committed());
    assert_eq!(resumed.requested, 0);
    assert_eq!(resumed.stick, Some([1., 0.]));
}
#[test]
fn fully_owned_sequences_exclude_global_guard_jump_and_attack_requests() {
    use kirby_assist_core::{ledge_trap::SEQUENCE_BUTTONS, native_input::selected_requests};
    assert_eq!(
        selected_requests(SEQUENCE_BUTTONS, MAPPER_B, MAPPER_ZR | MAPPER_X | MAPPER_A),
        MAPPER_B
    );
    assert_eq!(
        selected_requests(0, 0, MAPPER_ZR),
        MAPPER_ZR,
        "native safety owner keeps its request"
    );
}
#[test]
fn direct_shield_up_b_keeps_a_continuously_held_guard_after_handoff() {
    use kirby_assist_core::manual_controls::AssistGuardHandoff;
    let mut h = AssistGuardHandoff::new();
    let g = MAPPER_ZR;
    let held = ButtonTrace {
        held: g,
        previous: g,
        ..Default::default()
    };
    assert_eq!(h.sample(true, held, g, g, g), 0);
    assert_eq!(h.sample(false, held, g, 0, 0), 0);
    assert_eq!(h.sample(false, ButtonTrace::default(), g, 0, 0), 0);
}

#[test]
fn automatic_grab_recovery_does_not_start_an_up_b_or_mask_manual_throws() {
    let r = rules();
    for status in [
        kirby_assist_core::native_states::CATCH,
        kirby_assist_core::native_states::CATCH_DASH,
        kirby_assist_core::native_states::status::CATCH_CUT,
        kirby_assist_core::native_states::THROW,
    ] {
        let mut e = Engine::default();
        let out = sample(
            &mut e,
            &r,
            0.,
            0x20,
            20.,
            true,
            GameplayContext { status, ..c(0.) },
        );
        assert_eq!(out.owned, 0);
        assert!(!e.committed());
        let out = sample(
            &mut e,
            &r,
            1.,
            kirby_assist_core::native_states::ESCAPE_F,
            20.,
            true,
            c(1.),
        );
        assert_ne!(out.requested & MAPPER_B, 0);
        assert!(e.committed());
    }
}

#[test]
fn damage_and_knockdown_do_not_admit_an_abort_then_neutral_mask() {
    let r = rules();
    for status in [
        kirby_assist_core::native_states::status::CAPTURE_PULLED,
        kirby_assist_core::native_states::status::THROWN,
        kirby_assist_core::native_states::status::DAMAGE,
        kirby_assist_core::native_states::status::DAMAGE_FLY,
        kirby_assist_core::native_states::status::DAMAGE_FALL,
        kirby_assist_core::native_states::status::DOWN,
        kirby_assist_core::native_states::status::DOWN_WAIT_CONTINUE,
        kirby_assist_core::native_states::status::DOWN_DAMAGE,
        kirby_assist_core::native_states::status::DOWN_REFLECT_LR,
        kirby_assist_core::native_states::status::DEAD,
        kirby_assist_core::native_states::status::REBIRTH,
        kirby_assist_core::native_states::status::DAMAGE_FLY_REFLECT_JUMP_BOARD,
    ] {
        let mut e = Engine::default();
        let out = sample(
            &mut e,
            &r,
            0.,
            0x20,
            20.,
            true,
            GameplayContext { status, ..c(0.) },
        );
        assert_eq!(out.owned, 0, "incapacitated own status {status:#x}");
        let recovered = sample(
            &mut e,
            &r,
            1.,
            kirby_assist_core::native_states::ESCAPE_F,
            20.,
            true,
            c(1.),
        );
        assert_ne!(
            recovered.requested & MAPPER_B,
            0,
            "a still-live roll can be handled when readiness returns {status:#x}"
        );
    }
}

#[test]
fn moving_reactions_must_begin_with_up_b_not_a_grab_or_approach() {
    for actions in [
        "grab:4",
        "drift:opponent:1:10,up_b:4",
        "up_b:4,drift:opponent:1:30",
    ] {
        let cat = Catalog::parse(&format!(
            "[bowser.roll.auto]\nstart_delay=0\nreaction=auto_roll:30:10\nactions={actions}\n"
        ))
        .unwrap();
        let mut rules = Resolved::default();
        rules
            .add(Candidate {
                slot: 2,
                index: 0,
                sequence: cat.entries()[0].sequence,
            })
            .unwrap();
        for own in [1, 3, 4] {
            let mut engine = Engine::default();
            let out = sample(
                &mut engine,
                &rules,
                0.,
                0x20,
                20.,
                true,
                GameplayContext {
                    status: own,
                    ..c(0.)
                },
            );
            assert_eq!(
                out.owned != 0,
                actions.starts_with("up_b"),
                "{own:#x} {actions}"
            );
        }
    }
}

#[test]
fn live_dodges_are_reconsidered_after_endlag_but_finished_events_are_never_queued() {
    let r = rules();
    for event in [0x20, 0x21, 0x1f, 0x22, 0x23] {
        for stopped in [false, true] {
            let blocked = GameplayContext {
                status: kirby_assist_core::native_states::status::ATTACK_S4,
                can_act: false,
                stopped,
                ..c(0.)
            };
            let mut live = Engine::default();
            assert_eq!(
                sample(&mut live, &r, 0., event, 20., true, blocked).owned,
                0
            );
            assert_ne!(
                sample(&mut live, &r, 1., event, 20., true, c(1.)).requested & MAPPER_B,
                0
            );
            let mut ended = Engine::default();
            assert_eq!(
                sample(&mut ended, &r, 0., event, 20., true, blocked).owned,
                0
            );
            assert_eq!(sample(&mut ended, &r, 1., 0, 20., true, c(1.)).owned, 0);
            assert_ne!(
                sample(&mut ended, &r, 2., event, 20., true, c(2.)).requested & MAPPER_B,
                0
            );
        }
    }
}

#[test]
fn unconfirmed_up_b_cannot_turn_into_a_walk_or_drift_chase() {
    let r = rules();
    let mut e = Engine::default();
    for n in 0..4 {
        assert_ne!(
            sample(
                &mut e,
                &r,
                n as f32,
                kirby_assist_core::native_states::ESCAPE_F,
                20.,
                true,
                c(n as f32)
            )
            .requested
                & MAPPER_B,
            0
        );
    }
    let out = sample(
        &mut e,
        &r,
        4.,
        kirby_assist_core::native_states::ESCAPE_F,
        20.,
        true,
        c(4.),
    );
    assert_eq!(out.owned, 0);
    assert_eq!(out.stick, None);
    assert!(!e.committed());
    assert_eq!(
        sample(
            &mut e,
            &r,
            5.,
            kirby_assist_core::native_states::ESCAPE_F,
            20.,
            true,
            c(5.)
        )
        .owned,
        0,
        "failed response is not retried on the same roll"
    );
}

#[test]
fn wrong_native_moves_and_finished_up_b_return_control_without_extra_drift() {
    let r = rules();
    for status in [
        kirby_assist_core::native_states::status::JUMP,
        kirby_assist_core::native_states::status::JUMP_AERIAL,
        kirby_assist_core::native_states::status::FALL,
        kirby_assist_core::native_states::ESCAPE,
        kirby_assist_core::native_states::ESCAPE_F,
        kirby_assist_core::native_states::ESCAPE_B,
        kirby_assist_core::native_states::status::ESCAPE_AIR,
        kirby_assist_core::native_states::status::SPECIAL_N,
        0x1dd,
        kirby_assist_core::native_states::status::SPECIAL_LW,
    ] {
        let mut e = Engine::default();
        sample(
            &mut e,
            &r,
            0.,
            kirby_assist_core::native_states::ESCAPE_F,
            20.,
            true,
            c(0.),
        );
        assert_eq!(
            sample(
                &mut e,
                &r,
                1.,
                0x20,
                20.,
                true,
                GameplayContext { status, ..c(1.) }
            )
            .owned,
            0,
            "{status:#x}"
        );
    }
    for special in [
        kirby_assist_core::native_states::status::SPECIAL_HI,
        kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_G,
        kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_A,
    ] {
        let mut e = Engine::default();
        let settings = LedgeSettings {
            fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
            ..Default::default()
        };
        let target = t(0x20, 20., 0.);
        let targets = [Some(target), None, None, None, None, None, None, None];
        e.sample_simple(true, &r, settings, c(0.), targets, Some(follow(target)));
        for n in 1..5 {
            let out = e.sample_simple(
                true,
                &r,
                settings,
                GameplayContext {
                    status: special,
                    ..c(n as f32)
                },
                targets,
                Some(follow(target)),
            );
            assert_ne!(out.owned, 0);
        }
        let out = e.sample_simple(true, &r, settings, c(5.), targets, Some(follow(target)));
        assert_eq!(out.owned, 0, "special {special:#x} has ended");
        assert!(!e.committed());
    }
}

#[test]
fn instant_roll_grab_cancels_outside_reach_and_never_queues_the_same_roll() {
    let catalog = Catalog::parse(
        "[bowser.roll.instant_grab]\nstart_delay=0\nreaction=auto_roll:60:12\nactions=grab:fighter_range_instant\n",
    ).unwrap();
    let mut rules = Resolved::default();
    rules
        .add(Candidate {
            slot: 2,
            index: 0,
            sequence: catalog.entries()[0].sequence,
        })
        .unwrap();
    let settings = LedgeSettings::default();
    let reach = settings.fighter.grab.standing_range;
    let mut engine = Engine::default();
    let out = sample(
        &mut engine,
        &rules,
        0.,
        kirby_assist_core::native_states::ESCAPE_F,
        reach + 1.,
        true,
        c(0.),
    );
    assert_eq!(out.requested, 0);
    assert!(
        !engine.committed(),
        "out-of-range check releases the plan immediately"
    );
    assert!(engine.target().is_none());
    let out = sample(
        &mut engine,
        &rules,
        1.,
        kirby_assist_core::native_states::ESCAPE_F,
        reach,
        true,
        c(1.),
    );
    assert_eq!(
        out.owned, 0,
        "no neutral range wait on the next observation"
    );
    assert_eq!(
        out.requested, 0,
        "the consumed roll cannot start a late grab"
    );
    let mut fresh = Engine::default();
    let out = sample(
        &mut fresh,
        &rules,
        0.,
        kirby_assist_core::native_states::ESCAPE_F,
        reach,
        true,
        c(0.),
    );
    assert_eq!(
        out.requested, MAPPER_R,
        "in-range roll grab starts in the same observation"
    );
}

#[test]
fn manual_intent_hands_back_confirmed_up_b_drift_without_cutting_its_b_pulse() {
    let r = rules();
    for fresh_at in [2, 4, 5] {
        let mut engine = Engine::default();
        for n in 0..=5 {
            let context = GameplayContext {
                status: if n == 0 {
                    kirby_assist_core::native_states::GUARD
                } else {
                    kirby_assist_core::native_states::status::SPECIAL_HI
                },
                manual_input_epoch: u64::from(n >= fresh_at),
                ..c(n as f32)
            };
            let out = sample(
                &mut engine,
                &r,
                n as f32,
                kirby_assist_core::native_states::ESCAPE_F,
                10.,
                true,
                context,
            );
            if n < 4 {
                assert_ne!(
                    out.requested & MAPPER_B,
                    0,
                    "critical pulse is delivered in full"
                );
            } else if n >= fresh_at {
                assert_eq!(
                    out.owned, 0,
                    "manual input releases drift on that observation"
                );
                assert!(!engine.committed());
            } else {
                assert_eq!(out.stick, Some([1., 0.]));
            }
        }
    }
}
