use kirby_assist_core::{
    action_owner::{ActionOwnership, Intent, Owner},
    gameplay::GameplayContext,
    ledge_trap::SEQUENCE_BUTTONS,
    manual_controls::{ManualOverlay, TurboOutput},
    native_input::*,
    out_of_shield::{AutomaticConfig, Context, OutOfShield},
    reaction_rules::{Candidate, Engine, Resolved},
    roll_read::Target,
    sequence_config::Catalog,
    settings::AssistSettings,
};
fn c(frame: f32) -> Context {
    Context {
        object: 1,
        kind: kirby_assist_core::fighter_kinds::BOWSER,
        status: kirby_assist_core::native_states::GUARD,
        motion: 1,
        frame,
        rate: 1.,
        stopped: false,
        grounded: true,
        position: [0.; 2],
        facing: 1.,
        shield_ready: true,
        can_grab: true,
        can_up_special: true,
        can_jump: false,
        threatened: false,
    }
}
fn lc(frame: f32) -> GameplayContext {
    GameplayContext {
        gameplay: true,
        held: true,
        status: kirby_assist_core::native_states::GUARD,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.,
        facing: 1.,
        can_act: true,
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
fn sample(
    o: &mut OutOfShield,
    allowed: bool,
    c: Context,
    lc: GameplayContext,
    t: [Option<Target>; 8],
) -> kirby_assist_core::gameplay::ControllerOutput {
    o.sample_automatic_configured(
        allowed,
        c,
        lc,
        t,
        None,
        true,
        22.,
        35.,
        Some(12.),
        AssistSettings {
            fighter_kind: c.kind,
            ..Default::default()
        },
        explicit_backup_for(c.kind),
    )
}
fn explicit_backup() -> AutomaticConfig<'static> {
    explicit_backup_for(15)
}
fn explicit_backup_for(kind: i32) -> AutomaticConfig<'static> {
    static BOWSER: std::sync::OnceLock<kirby_assist_core::sequence_config::Sequence> =
        std::sync::OnceLock::new();
    static OTHER: std::sync::OnceLock<kirby_assist_core::sequence_config::Sequence> =
        std::sync::OnceLock::new();
    let cell = if kind == kirby_assist_core::fighter_kinds::BOWSER {
        &BOWSER
    } else {
        &OTHER
    };
    AutomaticConfig{backup:Some(cell.get_or_init(||{
        let actions=if kind==15{"up_b:4,drift:opponent:1:69"}else{"up_b:4"};
        Catalog::parse(&format!("[bowser.roll.backup]\nstart_delay=0\nreaction=auto_shield_backup:22:24\nactions={actions}\n")).unwrap().entries()[0].sequence
    })),..Default::default()}
}
#[test]
fn front_grab_wins_and_close_ungrabbable_targets_receive_up_b_in_the_same_observation() {
    for (x, grab_ready, velocity, expected) in [
        (10., true, 0., MAPPER_R | MAPPER_ZR),
        (-10., true, 0., MAPPER_B | MAPPER_ZR),
        (0., true, 0., MAPPER_B | MAPPER_ZR),
        (10., false, 0., MAPPER_B | MAPPER_ZR),
        (20., true, 2., MAPPER_B | MAPPER_ZR),
    ] {
        let mut o = OutOfShield::new();
        let mut t = targets(x);
        t[0].as_mut().unwrap().velocity_x = velocity;
        let out = sample(
            &mut o,
            true,
            Context {
                can_grab: grab_ready,
                ..c(0.)
            },
            lc(0.),
            t,
        );
        assert_eq!(
            out.requested, expected,
            "x={x},grab={grab_ready},velocity={velocity}"
        );
        assert_eq!(out.owned, SEQUENCE_BUTTONS);
        assert_eq!(
            out.stick,
            Some(if expected & MAPPER_B != 0 {
                [0., 1.]
            } else {
                [0.; 2]
            })
        );
        assert_eq!(o.shield_up_b_pending(), expected & MAPPER_B != 0);
        assert_eq!(o.automatic_grab_pending(), expected & MAPPER_R != 0);
    }
}
#[test]
fn shieldstun_hitstop_and_unready_states_do_not_queue_or_neutralize_manual_control() {
    for case in 0..9 {
        let mut o = OutOfShield::new();
        let mut context = c(0.);
        let mut own = lc(0.);
        match case {
            0 => context.status = kirby_assist_core::native_states::GUARD_DAMAGE,
            1 => context.stopped = true,
            2 => own.stopped = true,
            3 => context.can_up_special = false,
            4 => context.shield_ready = false,
            5 => context.grounded = false,
            6 => context.status = kirby_assist_core::native_states::status::WAIT,
            7 => own.gameplay = false,
            _ => {}
        }
        assert_eq!(
            sample(&mut o, case != 8, context, own, targets(-10.)).owned,
            0,
            "case {case}"
        );
        assert!(o.target().is_none());
        let ready = sample(&mut o, true, c(1.), lc(1.), targets(-10.));
        assert_eq!(
            ready.requested,
            MAPPER_B | MAPPER_ZR,
            "first ready observation, case {case}"
        );
    }
}
#[test]
fn fallback_range_vulnerability_and_status_are_current_and_bound_on_both_sides() {
    for (x, y, vulnerable, status, expected) in [
        (-22., 24., true, 0, true),
        (-22.01, 0., true, 0, false),
        (-10., 24.01, true, 0, false),
        (-10., 0., false, 0x20, false),
        (-10., 0., true, 0x41, false),
        (-10., 0., true, 0x77, false),
        (10., 0., true, 0x2c, true),
        (10., 0., true, 0x53, true),
        (-10., 0., false, 0x53, false),
        (f32::NAN, 0., true, 0, false),
    ] {
        let mut o = OutOfShield::new();
        let mut ts = targets(x);
        let t = ts[0].as_mut().unwrap();
        t.position[1] = y;
        t.status = status;
        t.vulnerable_in = if vulnerable { Some(0.) } else { None };
        let out = sample(
            &mut o,
            true,
            Context {
                can_grab: false,
                ..c(0.)
            },
            lc(0.),
            ts,
        );
        assert_eq!(out.requested & MAPPER_B != 0, expected);
        assert_eq!(out.owned != 0, expected);
    }
}
#[test]
fn failed_or_crossed_grab_hands_to_up_b_without_mixing_inputs_or_adding_a_release_frame() {
    let mut o = OutOfShield::new();
    assert_eq!(
        sample(&mut o, true, c(0.), lc(0.), targets(10.)).requested,
        MAPPER_R | MAPPER_ZR
    );
    let out = sample(&mut o, true, c(1.), lc(1.), targets(-10.));
    assert_eq!(out.requested, MAPPER_B | MAPPER_ZR);
    assert!(!o.automatic_grab_pending());
    assert!(o.shield_up_b_pending());
    let mut o = OutOfShield::new();
    for n in 0..4 {
        assert_ne!(
            sample(&mut o, true, c(0.), lc(0.), targets(10.)).requested & MAPPER_R,
            0
        );
        o.mapped_grab(n, 2);
    }
    assert_eq!(
        sample(&mut o, true, c(0.), lc(0.), targets(10.)).requested,
        0,
        "delivered grab must settle without queuing a throw-producing Up-B"
    );
}
#[test]
fn fallback_is_bounded_by_delivered_packets_and_releases_on_native_start_or_manual_escape() {
    let mut o = OutOfShield::new();
    let ctx = Context { rate: 0., ..c(0.) };
    let own = GameplayContext {
        motion_rate: 0.,
        ..lc(0.)
    };
    for tick in 0..4 {
        for _ in 0..8 {
            assert_eq!(
                sample(&mut o, true, ctx, own, targets(-10.)).requested,
                MAPPER_B | MAPPER_ZR
            );
        }
        o.mapped_up_special(tick, 999);
        o.mapped_up_special(tick, 2);
        o.mapped_up_special(tick, 2);
    }
    assert_eq!(sample(&mut o, true, ctx, own, targets(-10.)).owned, 0);
    assert!(
        o.up_b_receipt_pending(),
        "a delivered packet may still receive native confirmation; no more B is sent"
    );
    for tick in 4..24 {
        o.delivery_tick(tick);
        assert_eq!(sample(&mut o, true, ctx, own, targets(-10.)).owned, 0);
    }
    assert!(
        o.target().is_none(),
        "passive confirmation is bounded even with frozen motion clocks"
    );
    for status in [
        kirby_assist_core::native_states::status::WAIT,
        kirby_assist_core::native_states::status::DASH,
        kirby_assist_core::native_states::GUARD_OFF,
        kirby_assist_core::native_states::GUARD_DAMAGE,
        kirby_assist_core::native_states::CATCH,
        kirby_assist_core::native_states::status::DAMAGE,
    ] {
        let mut o = OutOfShield::new();
        sample(&mut o, true, c(0.), lc(0.), targets(-10.));
        assert_eq!(
            sample(
                &mut o,
                true,
                Context { status, ..c(1.) },
                lc(1.),
                targets(-10.)
            )
            .owned,
            0,
            "{status:#x}"
        );
        assert!(o.target().is_none());
    }
    let mut o = OutOfShield::new();
    sample(&mut o, true, c(0.), lc(0.), targets(-10.));
    assert_eq!(sample(&mut o, false, c(1.), lc(1.), targets(-10.)).owned, 0);
}
#[test]
fn native_grab_acceptance_and_other_input_owners_are_never_replaced_by_the_fallback() {
    let mut o = OutOfShield::new();
    sample(&mut o, true, c(0.), lc(0.), targets(10.));
    assert_eq!(
        sample(
            &mut o,
            true,
            Context {
                status: kirby_assist_core::native_states::CATCH,
                ..c(1.)
            },
            lc(1.),
            targets(-10.)
        )
        .owned,
        0
    );
    for owner in [
        Owner::Safety,
        Owner::Ledge,
        Owner::RollRead,
        Owner::GrabDodge,
    ] {
        let mut arbiter = ActionOwnership::new();
        arbiter.select(true, &[Intent::input_sequence(owner, SEQUENCE_BUTTONS)]);
        let mut o = OutOfShield::new();
        assert_eq!(
            sample(
                &mut o,
                arbiter.allows(Owner::OutOfShield),
                c(0.),
                lc(0.),
                targets(-10.)
            )
            .owned,
            0,
            "{owner:?}"
        );
    }
}
#[test]
fn roll_up_b_wins_before_new_shield_fallback_and_active_fallback_cannot_become_a_grab() {
    let cat = Catalog::parse(
        "[bowser.roll.up_b]\nstart_delay=0\nreaction=auto_roll:40:12\nactions=up_b:4\n",
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
    let mut ts = targets(-10.);
    ts[0].as_mut().unwrap().status = kirby_assist_core::native_states::ESCAPE_F;
    let mut e = Engine::default();
    let mut o = OutOfShield::new();
    let reaction = e.sample_simple(true, &rules, AssistSettings::default(), lc(0.), ts, None);
    assert_ne!(reaction.requested & MAPPER_B, 0);
    assert_eq!(
        sample(&mut o, reaction.owned == 0, c(0.), lc(0.), ts).owned,
        0
    );
    let mut o = OutOfShield::new();
    sample(&mut o, true, c(0.), lc(0.), targets(-10.));
    let out = sample(&mut o, true, c(1.), lc(1.), targets(10.));
    assert_eq!(
        out.requested,
        MAPPER_B | MAPPER_ZR,
        "committed B pulse is not switched to a new grab"
    );
    assert!(!o.automatic_grab_pending());
}
#[test]
fn fallback_reaches_both_layouts_without_stale_grab_cstick_or_tap_jump_input() {
    let mut o = OutOfShield::new();
    let out = sample(&mut o, true, c(0.), lc(0.), targets(-10.));
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        let mut overlay = ManualOverlay::default();
        overlay.apply_controls(
            ButtonTrace::default(),
            [0.; 2],
            TurboOutput::default(),
            0,
            layout.physical_buttons(MAPPER_R),
            layout.physical_owned_buttons(SEQUENCE_BUTTONS),
        );
        let raw = overlay.apply_controls(
            ButtonTrace {
                held: GAMEPLAY_BUTTONS,
                previous: GAMEPLAY_BUTTONS,
                ..Default::default()
            },
            [-1., -1.],
            TurboOutput {
                stick: out.stick,
                ..Default::default()
            },
            0,
            layout.physical_buttons(out.requested),
            layout.physical_owned_buttons(out.owned),
        );
        assert_eq!(
            raw.buttons.held & GAMEPLAY_BUTTONS,
            layout.physical_buttons(MAPPER_B | MAPPER_ZR)
        );
        assert_ne!(raw.buttons.pressed & layout.physical_buttons(MAPPER_B), 0);
        assert_eq!(
            exclusive_sequence_stick(out.owned, out.stick),
            Some([0., 1.])
        );
        assert_eq!(
            mapped_automatic_actions(mapped_sequence_cstick(0xffff, [0.; 2]), out.requested)
                & (0x4 | 0x40 | 0x80),
            0
        );
        assert_ne!(
            mapped_automatic_actions(0xffff, out.requested) & 0x8,
            0,
            "explicit guard is retained for direct OOS"
        );
        assert_eq!(mapped_sequence_cstick(0x882, [0.; 2]), 0x802);
    }
}

#[test]
fn any_legal_front_grab_wins_over_a_nearer_rear_fallback_and_lost_targets_release() {
    let mut ts = targets(-1.);
    ts[1] = ts[0].map(|t| Target {
        id: 3,
        position: [20., 0.],
        ..t
    });
    let mut o = OutOfShield::new();
    assert_eq!(
        sample(&mut o, true, c(0.), lc(0.), ts).requested,
        MAPPER_R | MAPPER_ZR
    );
    assert_eq!(o.target(), Some(3));
    for invalid in 0..4 {
        let mut o = OutOfShield::new();
        sample(&mut o, true, c(0.), lc(0.), targets(-10.));
        let mut next = targets(-10.);
        match invalid {
            0 => next[0] = None,
            1 => next[0].as_mut().unwrap().vulnerable_in = None,
            2 => next[0].as_mut().unwrap().position[0] = -23.,
            _ => next[0].as_mut().unwrap().id = 3,
        }
        assert_eq!(sample(&mut o, true, c(1.), lc(1.), next).owned, 0);
        assert!(o.target().is_none());
    }
}

#[test]
fn cancelled_fallback_can_choose_a_new_legal_grab_without_an_empty_decision_frame() {
    let mut o = OutOfShield::new();
    sample(&mut o, true, c(0.), lc(0.), targets(-10.));
    let mut next = targets(10.);
    next[0].as_mut().unwrap().id = 3;
    let out = sample(&mut o, true, c(1.), lc(1.), next);
    assert_eq!(out.requested, MAPPER_R | MAPPER_ZR);
    assert_eq!(o.target(), Some(3));
    assert!(!o.shield_up_b_pending());
}

#[test]
fn roll_priority_is_rechecked_when_a_cancelled_fallback_proposes_a_fresh_grab() {
    let cat = Catalog::parse(
        "[bowser.roll.up_b]\nstart_delay=0\nreaction=auto_roll:40:12\nactions=up_b:4\n",
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
    let mut o = OutOfShield::new();
    let mut owner = ActionOwnership::new();
    let mut engine = Engine::default();
    let first = sample(&mut o, true, c(0.), lc(0.), targets(-10.));
    owner.select(
        true,
        &[Intent::input_sequence(Owner::OutOfShield, first.owned)],
    );
    let mut next = targets(10.);
    next[0].as_mut().unwrap().id = 3;
    next[1] = next[0].map(|t| Target {
        id: 4,
        status: kirby_assist_core::native_states::ESCAPE_F,
        position: [-15., 0.],
        ..t
    });
    assert!(
        !engine.preempt_automatic_grab(true, &rules, lc(1.), &next, None, &mut o, &mut owner),
        "protect the existing B pulse"
    );
    let proposed = sample(&mut o, true, c(1.), lc(1.), next);
    assert_ne!(proposed.requested & MAPPER_R, 0);
    assert!(
        !o.shield_up_b_pending(),
        "the old fallback was canceled before new admission"
    );
    owner.release(Owner::OutOfShield);
    assert!(
        engine.preempt_automatic_grab(true, &rules, lc(1.), &next, None, &mut o, &mut owner),
        "new grab still yields before mapping"
    );
    let out = engine.sample_simple(
        owner.allows(Owner::RollRead),
        &rules,
        AssistSettings::default(),
        lc(1.),
        next,
        None,
    );
    assert_eq!(out.requested, MAPPER_B | MAPPER_ZR);
    assert!(o.target().is_none());
}

#[test]
fn confirmed_bowser_shield_up_b_tracks_crossings_at_full_strength_for_69_frames() {
    let mut o = OutOfShield::new();
    sample(&mut o, true, c(0.), lc(0.), targets(-10.));
    for frame in 0..69 {
        let side = if frame < 35 { -1. } else { 1. };
        let own = Context {
            status: kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_A,
            grounded: false,
            can_up_special: false,
            shield_ready: false,
            ..c(frame as f32)
        };
        let context = GameplayContext {
            status: kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_A,
            situation: kirby_assist_core::native_states::situation::AIR,
            ..lc(frame as f32)
        };
        let out = o.sample_automatic_configured(
            true,
            own,
            context,
            targets(side * 50.),
            None,
            false,
            22.,
            35.,
            Some(8.),
            AssistSettings {
                fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
                ..Default::default()
            },
            explicit_backup(),
        );
        assert_eq!(out.requested, 0);
        assert_eq!(out.stick, Some([side, 0.]));
        assert_eq!(
            out.owned, SEQUENCE_BUTTONS,
            "tracking stays active at frame {frame}"
        );
    }
    let ended = o.sample_automatic_configured(
        true,
        Context {
            status: kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_A,
            grounded: false,
            ..c(69.)
        },
        GameplayContext {
            status: kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_A,
            situation: kirby_assist_core::native_states::situation::AIR,
            ..lc(69.)
        },
        targets(50.),
        None,
        false,
        22.,
        35.,
        Some(8.),
        AssistSettings {
            fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
            ..Default::default()
        },
        explicit_backup(),
    );
    assert_eq!(ended.owned, 0);
    assert!(o.target().is_none());
}

#[test]
fn bowser_shield_tracking_ends_on_native_finish_target_loss_or_ledge_cancellation_and_never_starts_in_air(
) {
    for case in 0..3 {
        let mut o = OutOfShield::new();
        sample(&mut o, true, c(0.), lc(0.), targets(-10.));
        let own = Context {
            status: kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_G,
            ..c(0.)
        };
        let context = GameplayContext {
            status: kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_G,
            ..lc(0.)
        };
        assert_eq!(
            o.sample_automatic_configured(
                true,
                own,
                context,
                targets(-10.),
                None,
                false,
                22.,
                35.,
                Some(8.),
                AssistSettings {
                    fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
                    ..Default::default()
                },
                explicit_backup()
            )
            .stick,
            Some([-1., 0.])
        );
        let mut next = Context { frame: 1., ..own };
        let ctx = GameplayContext {
            motion_frame: 1.,
            manual_input_epoch: 1,
            automatic_up_b_epoch: Some(1),
            ..context
        };
        let mut ts = targets(-10.);
        match case {
            0 => next.status = kirby_assist_core::native_states::status::FALL_SPECIAL,
            1 => ts = [None; 8],
            _ => {}
        }
        assert_eq!(
            o.sample_automatic_configured(
                case != 2,
                next,
                ctx,
                ts,
                None,
                false,
                22.,
                35.,
                Some(8.),
                AssistSettings {
                    fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
                    ..Default::default()
                },
                explicit_backup()
            )
            .owned,
            0
        );
    }
    let mut fresh = OutOfShield::new();
    assert_eq!(
        fresh
            .sample_automatic_configured(
                true,
                Context {
                    status: kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_A,
                    grounded: false,
                    ..c(0.)
                },
                GameplayContext {
                    status: kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_A,
                    situation: kirby_assist_core::native_states::situation::AIR,
                    ..lc(0.)
                },
                targets(-10.),
                None,
                false,
                22.,
                35.,
                Some(8.),
                AssistSettings {
                    fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
                    ..Default::default()
                },
                explicit_backup()
            )
            .owned,
        0
    );
    let mut other = OutOfShield::new();
    let other_ctx = Context {
        kind: kirby_assist_core::fighter_kinds::KIRBY,
        ..c(0.)
    };
    sample(&mut other, true, other_ctx, lc(0.), targets(-10.));
    assert_eq!(
        other
            .sample_automatic_configured(
                true,
                Context {
                    status: kirby_assist_core::native_states::status::SPECIAL_HI,
                    ..other_ctx
                },
                lc(1.),
                targets(-10.),
                None,
                false,
                22.,
                35.,
                Some(8.),
                AssistSettings {
                    fighter_kind: kirby_assist_core::fighter_kinds::KIRBY,
                    ..Default::default()
                },
                explicit_backup_for(6)
            )
            .owned,
        0,
        "other fighters retain their previous release-on-startup behavior"
    );
}

#[test]
fn supplied_bowser_dodge_sequences_have_only_the_requested_69_frame_drift_change() {
    use kirby_assist_core::sequence_config::{Action, Direction};
    let catalog = Catalog::parse(include_str!(
        "../../../deployment/ultimate/kirby_assist/assembled/sequences.txt"
    ))
    .unwrap();
    for id in ["auto_roll", "auto_spot_dodge", "auto_air_dodge"] {
        let e = catalog
            .entries()
            .iter()
            .find(|e| e.fighter_kind == kirby_assist_core::fighter_kinds::BOWSER && e.choice == id)
            .unwrap();
        assert_eq!(e.sequence.start_delay, 0);
        assert_eq!(e.sequence.actions().len(), 2);
        assert_eq!(
            e.sequence.actions()[1],
            Action::Drift {
                direction: Direction::Opponent,
                strength: 1.,
                frames: 69
            }
        );
    }
}

#[test]
fn bowser_spin_tracking_holds_direction_through_hitstop_without_spending_the_clock() {
    let mut o = OutOfShield::new();
    sample(&mut o, true, c(0.), lc(0.), targets(-10.));
    let own = Context {
        status: kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_G,
        ..c(0.)
    };
    let context = GameplayContext {
        status: kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_G,
        ..lc(0.)
    };
    assert_eq!(
        o.sample_automatic_configured(
            true,
            own,
            context,
            targets(-10.),
            None,
            false,
            22.,
            35.,
            Some(8.),
            AssistSettings {
                fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
                ..Default::default()
            },
            explicit_backup()
        )
        .stick,
        Some([-1., 0.])
    );
    for _ in 0..100 {
        assert_eq!(
            o.sample_automatic_configured(
                true,
                Context {
                    stopped: true,
                    ..own
                },
                GameplayContext {
                    stopped: true,
                    ..context
                },
                targets(50.),
                None,
                false,
                22.,
                35.,
                Some(8.),
                AssistSettings {
                    fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
                    ..Default::default()
                },
                explicit_backup()
            )
            .stick,
            Some([-1., 0.])
        );
    }
    assert_eq!(
        o.sample_automatic_configured(
            true,
            Context { frame: 1., ..own },
            GameplayContext {
                motion_frame: 1.,
                ..context
            },
            targets(50.),
            None,
            false,
            22.,
            35.,
            Some(8.),
            AssistSettings {
                fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
                ..Default::default()
            },
            explicit_backup()
        )
        .stick,
        Some([1., 0.])
    );
}
