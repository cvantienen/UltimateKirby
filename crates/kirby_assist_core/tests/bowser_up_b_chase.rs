//! Real movement-intent history drives every automatic Bowser Up-B owner.
use kirby_assist_core::{
    defense_response::Response,
    gameplay::{GameplayContext, MotionTiming, Opponent},
    ledge_trap::{InputTimeline, LedgeSettings, SEQUENCE_BUTTONS},
    manual_controls::ManualTakeover,
    native_input::*,
    out_of_shield::{AutomaticConfig, Context as ShieldContext, OutOfShield},
    reaction_rules::{Candidate, Engine, Resolved},
    sequence_config::{Catalog, Sequence},
};
fn recipe(event: &str) -> Sequence {
    Catalog::parse(&format!("[bowser.roll.test]\nstart_delay=0\ntimeout=240\nground_only=true\nreaction={event}:30:12\nactions=up_b:4,drift:opponent:1:69\n")).unwrap().entries()[0].sequence
}
fn settings() -> LedgeSettings {
    LedgeSettings {
        fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
        ..Default::default()
    }
}
fn history() -> ManualTakeover {
    let mut h = ManualTakeover::default();
    h.sample(0, [0.; 4]);
    h.sample(MAPPER_ZR, [0.; 4]);
    h
}
fn context(frame: u32, h: &ManualTakeover) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        can_act: frame == 0,
        status: if frame == 0 {
            kirby_assist_core::native_states::status::GUARD
        } else if frame < 20 {
            kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_G
        } else {
            kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_A
        },
        situation: if frame < 20 { 0 } else { 2 },
        motion_kind: 1,
        motion_frame: frame as f32,
        motion_rate: 1.,
        facing: 1.,
        manual_input_epoch: h.epoch(),
        automatic_up_b_epoch: Some(h.action_epoch()),
        ..Default::default()
    }
}
fn enemy(frame: u32) -> Opponent {
    Opponent {
        id: 2,
        status: if frame == 0 {
            0x20
        } else if frame < 4 {
            kirby_assist_core::native_states::status::DAMAGE_FLY
        } else {
            kirby_assist_core::native_states::status::CLIFF_WAIT
        },
        position: [
            if frame < 12 { -10. } else { 80. },
            100. * f32::from(frame > 12),
        ],
        motion: Some(MotionTiming {
            kind: 2,
            frame: frame as f32,
            rate: 1.,
            remaining: None,
        }),
        ..Default::default()
    }
}
fn move_stick(h: &mut ManualTakeover, frame: u32) {
    h.sample(
        0,
        match frame % 3 {
            0 => [0.; 4],
            1 => [1., 0.8, 0., 0.],
            _ => [-1., -0.8, 0., 0.],
        },
    );
}
fn fight_controls(h: &mut ManualTakeover, frame: u32) {
    let buttons = [
        MAPPER_A,
        MAPPER_B,
        MAPPER_X,
        MAPPER_Y,
        MAPPER_R,
        MAPPER_ZR,
        MAPPER_ZL,
        1,
        2,
        4,
        8,
        MAPPER_MINUS,
        1 << 16,
        1 << 17,
    ];
    let button = buttons[(frame as usize - 1) % buttons.len()];
    h.sample(
        button,
        [
            if frame % 2 == 0 { -1. } else { 1. },
            0.8,
            if frame % 2 == 0 { 1. } else { -1. },
            -0.8,
        ],
    );
}
fn rules(event: &str) -> Resolved {
    let mut r = Resolved::default();
    r.add(Candidate {
        slot: 2,
        index: 0,
        sequence: recipe(event),
    })
    .unwrap();
    r
}
fn target(e: Opponent) -> [Option<kirby_assist_core::roll_read::Target>; 8] {
    let mut ts = [None; 8];
    ts[0] = Some(kirby_assist_core::roll_read::Target {
        id: e.id,
        status: e.status,
        motion: 2,
        frame: 0.,
        rate: 1.,
        position: e.position,
        velocity_x: 0.,
        vulnerable_in: None,
        actionable_in: None,
    });
    ts
}
#[test]
fn every_option_read_keeps_69_frames_of_live_full_drift_despite_movement_release_hits_and_enemy_ledge_grab(
) {
    for (event, status) in [
        ("auto_roll", 0x20),
        ("auto_spot_dodge", 0x1f),
        ("auto_air_dodge", 0x22),
    ] {
        let r = rules(event);
        let mut engine = Engine::default();
        let mut h = history();
        let mut start = enemy(0);
        start.status = status;
        assert_ne!(
            engine
                .sample_simple(
                    true,
                    &r,
                    settings(),
                    context(0, &h),
                    target(start),
                    Some(start)
                )
                .requested
                & MAPPER_B,
            0
        );
        for frame in 1..73 {
            fight_controls(&mut h, frame);
            let e = enemy(frame);
            let out =
                engine.sample_simple(true, &r, settings(), context(frame, &h), [None; 8], Some(e));
            assert_eq!(out.owned, SEQUENCE_BUTTONS, "{event} frame {frame}");
            if frame >= 4 {
                assert_eq!(out.requested, 0);
                assert_eq!(
                    out.stick,
                    Some([e.position[0].signum(), 0.]),
                    "{event} frame {frame}"
                );
            }
        }
        let release = engine.sample_simple(
            true,
            &r,
            settings(),
            context(73, &h),
            [None; 8],
            Some(enemy(73)),
        );
        assert_eq!(release.requested, 0);
        assert_eq!(release.stick, Some([0.; 2]));
        assert_eq!(
            engine
                .sample_simple(
                    true,
                    &r,
                    settings(),
                    context(74, &h),
                    [None; 8],
                    Some(enemy(74))
                )
                .owned,
            0
        );
        assert!(!engine.committed());
    }
}
#[test]
fn dodge_grab_followup_keeps_its_configured_69_frame_chase_and_target_lock() {
    let mut r = Response::default();
    let mut h = history();
    let start = GameplayContext {
        status: kirby_assist_core::native_states::ESCAPE,
        ..context(0, &h)
    };
    assert!(r.start_sequence(recipe("auto_grab_dodged"), start, enemy(0)));
    for frame in 0..73 {
        if frame > 0 {
            fight_controls(&mut h, frame);
        }
        let c = if frame == 0 {
            start
        } else {
            context(frame, &h)
        };
        let e = enemy(frame);
        let out = r.sample(true, settings(), c, Some(e));
        assert_eq!(out.owned, SEQUENCE_BUTTONS);
        if frame >= 4 {
            assert_eq!(out.requested, 0);
            assert_eq!(out.stick, Some([e.position[0].signum(), 0.]));
        }
    }
    let release = r.sample(true, settings(), context(73, &h), Some(enemy(73)));
    assert_eq!(release.requested, 0);
    assert_eq!(release.stick, Some([0.; 2]));
    assert_eq!(
        r.sample(true, settings(), context(74, &h), Some(enemy(74)))
            .owned,
        0
    );
    assert!(r.target().is_none());
}
fn shield(c: GameplayContext) -> ShieldContext {
    ShieldContext {
        object: 1,
        kind: kirby_assist_core::fighter_kinds::BOWSER,
        status: c.status,
        motion: 1,
        frame: c.motion_frame,
        rate: 1.,
        grounded: c.situation == 0,
        position: c.position,
        facing: 1.,
        shield_ready: c.can_act,
        can_up_special: c.can_act,
        can_grab: false,
        can_jump: false,
        threatened: false,
        stopped: c.stopped,
    }
}
#[test]
fn configured_shield_backup_tracks_for_69_confirmed_frames_without_rechecking_start_range_or_vulnerability(
) {
    let b = recipe("auto_shield_backup");
    let config = AutomaticConfig {
        backup: Some(&b),
        ..Default::default()
    };
    let mut o = OutOfShield::new();
    let mut h = history();
    let c = context(0, &h);
    let mut e = enemy(0);
    e.status = kirby_assist_core::native_states::status::WAIT;
    e.vulnerable = true;
    assert_ne!(
        o.sample_automatic_configured(
            true,
            shield(c),
            c,
            target(e),
            Some(e),
            true,
            22.,
            35.,
            Some(8.),
            settings(),
            config
        )
        .requested
            & MAPPER_B,
        0
    );
    for frame in 1..70 {
        fight_controls(&mut h, frame);
        let c = context(frame, &h);
        let e = enemy(frame);
        let out = o.sample_automatic_configured(
            true,
            shield(c),
            c,
            [None; 8],
            Some(e),
            false,
            22.,
            35.,
            Some(8.),
            settings(),
            config,
        );
        assert_eq!(out.owned, SEQUENCE_BUTTONS, "frame {frame}");
        assert_eq!(out.requested, 0);
        assert_eq!(out.stick, Some([e.position[0].signum(), 0.]));
    }
    let c = context(70, &h);
    assert_eq!(
        o.sample_automatic_configured(
            true,
            shield(c),
            c,
            [None; 8],
            Some(enemy(70)),
            false,
            22.,
            35.,
            Some(8.),
            settings(),
            config
        )
        .owned,
        0
    );
}
#[test]
fn other_fighters_keep_movement_handback_and_literal_manual_actions_keep_the_original_clock() {
    let mut h = history();
    let start = h.action_epoch();
    h.sample(0, [1., 0., 0., 0.]);
    assert_eq!(h.action_epoch(), start);
    h.sample(0, [0., 0., 1., 0.]);
    assert_ne!(h.action_epoch(), start);
    let mut r = Response::default();
    let mut h = history();
    let c = GameplayContext {
        automatic_up_b_epoch: None,
        ..context(0, &h)
    };
    assert!(r.start_up_b(recipe("auto_grab_dodged"), c, enemy(0)));
    for frame in 0..=4 {
        r.sample(
            true,
            LedgeSettings {
                fighter_kind: kirby_assist_core::fighter_kinds::KIRBY,
                ..settings()
            },
            GameplayContext {
                automatic_up_b_epoch: None,
                status: if frame == 0 {
                    kirby_assist_core::native_states::GUARD
                } else {
                    kirby_assist_core::native_states::status::SPECIAL_HI
                },
                ..context(frame, &h)
            },
            Some(Opponent {
                status: kirby_assist_core::native_states::status::WAIT,
                ..enemy(frame)
            }),
        );
    }
    move_stick(&mut h, 5);
    assert_eq!(
        r.sample(
            true,
            LedgeSettings {
                fighter_kind: kirby_assist_core::fighter_kinds::KIRBY,
                ..settings()
            },
            GameplayContext {
                status: kirby_assist_core::native_states::status::SPECIAL_HI,
                automatic_up_b_epoch: None,
                ..context(5, &h)
            },
            Some(enemy(1))
        )
        .owned,
        0
    );
}
#[test]
fn chasing_never_survives_target_loss_capture_death_damage_ledge_mode_or_native_spin_end() {
    for case in 0..6 {
        let mut h = history();
        let mut r = Response::default();
        assert!(r.start_up_b(recipe("auto_grab_dodged"), context(0, &h), enemy(0)));
        for frame in 0..=4 {
            r.sample(true, settings(), context(frame, &h), Some(enemy(frame)));
        }
        move_stick(&mut h, 5);
        let mut c = context(5, &h);
        let mut e = Some(enemy(5));
        match case {
            0 => e = None,
            1 => e.as_mut().unwrap().status = kirby_assist_core::native_states::CAPTURE_PULLED,
            2 => e.as_mut().unwrap().status = kirby_assist_core::native_states::status::DEAD,
            3 => c.status = kirby_assist_core::native_states::status::DAMAGE_FLY,
            5 => c.status = kirby_assist_core::native_states::status::WAIT,
            _ => {}
        }
        assert_eq!(
            r.sample(case != 4, settings(), c, e).owned,
            0,
            "case {case}"
        );
        assert!(r.target().is_none());
    }
    // The extra action clock never changes literal shield-until-manual behavior.
    let seq =
        Catalog::parse("[bowser.attack.literal]\nstart_delay=0\nactions=shield_until_manual\n")
            .unwrap()
            .entries()[0]
            .sequence;
    let mut h = history();
    let mut t = InputTimeline::new(seq, enemy(0));
    assert_ne!(
        t.sample(settings(), context(0, &h), Some(enemy(0)), None, 0.)
            .requested
            & MAPPER_ZR,
        0
    );
    move_stick(&mut h, 1);
    let c = GameplayContext {
        status: kirby_assist_core::native_states::GUARD,
        ..context(1, &h)
    };
    assert_eq!(t.sample(settings(), c, Some(enemy(1)), None, 0.).owned, 0);
}

#[test]
fn bowser_up_b_protection_is_owner_specific_and_never_admits_a_new_reaction() {
    use kirby_assist_core::action_owner::{bowser_up_b_owner, ActionOwnership, Owner};
    for owner in [Owner::RollRead, Owner::OptionRead, Owner::OutOfShield] {
        let flags = (
            owner == Owner::RollRead,
            owner == Owner::OptionRead,
            owner == Owner::OutOfShield,
        );
        for current in [None, Some(owner)] {
            assert_eq!(
                bowser_up_b_owner(Some(15), current, flags.0, flags.1, flags.2),
                Some(owner)
            );
        }
        assert_eq!(
            bowser_up_b_owner(Some(15), Some(Owner::Safety), flags.0, flags.1, flags.2),
            None
        );
        for kind in [None, Some(5), Some(6), Some(22)] {
            assert_eq!(
                bowser_up_b_owner(kind, Some(owner), flags.0, flags.1, flags.2),
                None
            );
        }
        // L suspension is unconditional even when physical escape is bypassed.
        let mut arbiter = ActionOwnership::new();
        arbiter.set_automatic_suspended(true);
        assert!(!arbiter.allows(owner));
    }
    assert_eq!(bowser_up_b_owner(Some(15), None, false, false, false), None);
    for flags in [
        (true, true, false),
        (false, true, true),
        (true, false, true),
        (true, true, true),
    ] {
        assert_eq!(
            bowser_up_b_owner(Some(15), None, flags.0, flags.1, flags.2),
            None
        );
    }
}

#[test]
fn noncanonical_configured_shield_backup_keeps_chasing_with_buttons_and_cstick() {
    // This path runs the shared Response instead of the optimized ShieldUpB tracker.
    let catalog=Catalog::parse("[bowser.roll.backup]\nstart_delay=0\ntimeout=240\nreaction=auto_shield_backup:30:12\nactions=facing_input:b:0:1:4,drift:opponent:1:30,drift:opponent:1:39\n").unwrap();
    let b = catalog.entries()[0].sequence;
    let config = AutomaticConfig {
        backup: Some(&b),
        ..Default::default()
    };
    let mut o = OutOfShield::new();
    let mut h = history();
    let c = context(0, &h);
    let mut e = enemy(0);
    e.status = kirby_assist_core::native_states::status::WAIT;
    e.vulnerable = true;
    assert_ne!(
        o.sample_automatic_configured(
            true,
            shield(c),
            c,
            target(e),
            Some(e),
            true,
            22.,
            35.,
            Some(8.),
            settings(),
            config
        )
        .requested
            & MAPPER_B,
        0
    );
    for frame in 1..73 {
        fight_controls(&mut h, frame);
        let c = context(frame, &h);
        let e = enemy(frame);
        let out = o.sample_automatic_configured(
            true,
            shield(c),
            c,
            [None; 8],
            Some(e),
            false,
            22.,
            35.,
            Some(8.),
            settings(),
            config,
        );
        assert_eq!(out.owned, SEQUENCE_BUTTONS, "frame {frame}");
        if frame >= 4 {
            assert_eq!(out.requested, 0);
            assert_eq!(out.stick, Some([e.position[0].signum(), 0.]));
        }
    }
}

#[test]
fn tracking_packets_override_physical_buttons_and_sticks_in_both_layouts() {
    use kirby_assist_core::{
        manual_controls::{ManualOverlay, TurboOutput},
        native_input::ButtonTrace,
    };
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        let mut overlay = ManualOverlay::default();
        let mut r = Response::default();
        let mut h = history();
        assert!(r.start_up_b(recipe("auto_grab_dodged"), context(0, &h), enemy(0)));
        for frame in 0..73 {
            if frame > 0 {
                fight_controls(&mut h, frame);
            }
            let out = r.sample(true, settings(), context(frame, &h), Some(enemy(frame)));
            let raw = overlay.apply_controls(
                ButtonTrace {
                    held: GAMEPLAY_BUTTONS & !MAPPER_L,
                    pressed: GAMEPLAY_BUTTONS & !MAPPER_L,
                    ..Default::default()
                },
                [-1., 1.],
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
                layout.physical_buttons(out.requested)
            );
            assert_eq!(raw.stick, out.stick.unwrap());
            if frame >= 4 {
                assert_eq!(raw.buttons.held & GAMEPLAY_BUTTONS, 0);
                assert_eq!(raw.stick, [enemy(frame).position[0].signum(), 0.]);
            }
            assert_eq!(mapped_sequence_cstick(0x80, [0.; 2]) & 0x80, 0);
        }
    }
}

#[test]
fn button_changes_during_grounded_startup_do_not_detach_tracking_from_the_special() {
    let mut h = history();
    let rules = rules("auto_roll");
    let mut engine = Engine::default();
    let mut response = Response::default();
    assert!(response.start_up_b(recipe("auto_grab_dodged"), context(0, &h), enemy(0)));
    let b = recipe("auto_shield_backup");
    let config = AutomaticConfig {
        backup: Some(&b),
        ..Default::default()
    };
    let mut o = OutOfShield::new();
    let mut e = enemy(0);
    e.status = kirby_assist_core::native_states::status::WAIT;
    e.vulnerable = true;
    for frame in 0..4 {
        if frame > 0 {
            fight_controls(&mut h, frame);
        }
        let c = GameplayContext {
            status: kirby_assist_core::native_states::GUARD,
            can_act: true,
            ..context(frame, &h)
        };
        let roll = engine.sample_simple(
            true,
            &rules,
            settings(),
            c,
            if frame == 0 {
                target(enemy(0))
            } else {
                [None; 8]
            },
            Some(enemy(0)),
        );
        let dodge = response.sample(true, settings(), c, Some(enemy(0)));
        let backup = o.sample_automatic_configured(
            true,
            shield(c),
            c,
            target(e),
            Some(e),
            true,
            22.,
            35.,
            Some(8.),
            settings(),
            config,
        );
        for out in [roll, dodge, backup] {
            assert_ne!(out.requested & MAPPER_B, 0, "startup frame {frame}");
        }
    }
    fight_controls(&mut h, 4);
    let c = context(4, &h);
    let e = enemy(4);
    for out in [
        engine.sample_simple(true, &rules, settings(), c, [None; 8], Some(e)),
        response.sample(true, settings(), c, Some(e)),
        o.sample_automatic_configured(
            true,
            shield(c),
            c,
            [None; 8],
            Some(e),
            false,
            22.,
            35.,
            Some(8.),
            settings(),
            config,
        ),
    ] {
        assert_eq!(out.owned, SEQUENCE_BUTTONS);
        assert_eq!(out.requested, 0);
        assert_eq!(out.stick, Some([-1., 0.]));
    }
}

#[test]
fn delivered_shield_up_b_receipt_survives_shield_hitstop_until_native_confirmation() {
    let b = recipe("auto_shield_backup");
    let cfg = AutomaticConfig {
        backup: Some(&b),
        ..Default::default()
    };
    let mut o = OutOfShield::new();
    let h = history();
    let start = context(0, &h);
    let mut e = enemy(0);
    e.status = kirby_assist_core::native_states::status::WAIT;
    e.vulnerable = true;
    assert_ne!(
        o.sample_automatic_configured(
            true,
            shield(start),
            start,
            target(e),
            Some(e),
            true,
            22.,
            35.,
            Some(8.),
            settings(),
            cfg
        )
        .requested
            & MAPPER_B,
        0
    );
    o.mapped_up_special(0, e.id);
    let pause = GameplayContext {
        status: kirby_assist_core::native_states::GUARD_DAMAGE,
        stopped: true,
        can_act: false,
        ..context(1, &h)
    };
    let out = o.sample_automatic_configured(
        true,
        shield(pause),
        pause,
        [None; 8],
        Some(e),
        true,
        22.,
        35.,
        Some(8.),
        settings(),
        cfg,
    );
    assert_eq!(
        out.owned, 0,
        "receipt observation must not hold inputs through shield recovery"
    );
    assert!(
        o.shield_up_b_pending(),
        "keep the already delivered packet's target until confirmation"
    );
    let c = context(2, &h);
    let out = o.sample_automatic_configured(
        true,
        shield(c),
        c,
        [None; 8],
        Some(enemy(2)),
        true,
        22.,
        35.,
        Some(8.),
        settings(),
        cfg,
    );
    assert_eq!(out.requested, 0);
    assert_eq!(out.stick, Some([-1., 0.]));
    assert_eq!(out.owned, SEQUENCE_BUTTONS);
}
#[test]
fn delivered_roll_and_grab_dodge_receipts_survive_hitstop_without_resending_b() {
    let h = history();
    let r = rules("auto_roll");
    let mut engine = Engine::default();
    let mut response = Response::default();
    engine.sample_simple(
        true,
        &r,
        settings(),
        context(0, &h),
        target(enemy(0)),
        Some(enemy(0)),
    );
    assert!(response.start_up_b(recipe("auto_grab_dodged"), context(0, &h), enemy(0)));
    response.sample(true, settings(), context(0, &h), Some(enemy(0)));
    engine.mapped_up_special(0, 2);
    response.mapped_up_special(0, 2);
    let pause = GameplayContext {
        status: kirby_assist_core::native_states::GUARD_DAMAGE,
        stopped: true,
        can_act: false,
        ..context(1, &h)
    };
    assert_eq!(
        engine
            .sample_simple(true, &r, settings(), pause, [None; 8], Some(enemy(1)))
            .owned,
        0
    );
    assert!(engine.committed());
    assert_eq!(
        response
            .sample(true, settings(), pause, Some(enemy(1)))
            .owned,
        0
    );
    assert!(response.target().is_some());
    for out in [
        engine.sample_simple(
            true,
            &r,
            settings(),
            context(2, &h),
            [None; 8],
            Some(enemy(2)),
        ),
        response.sample(true, settings(), context(2, &h), Some(enemy(2))),
    ] {
        assert_eq!(out.requested, 0, "do not replay B after native startup");
        assert_eq!(out.stick, Some([-1., 0.]));
    }
}

#[test]
fn shield_receipt_after_the_four_frame_pulse_gets_all_69_drift_frames_and_no_extra_b() {
    let b = recipe("auto_shield_backup");
    let cfg = AutomaticConfig {
        backup: Some(&b),
        ..Default::default()
    };
    let mut o = OutOfShield::new();
    let h = history();
    let mut e = enemy(0);
    e.status = kirby_assist_core::native_states::status::WAIT;
    e.vulnerable = true;
    for frame in 0..4 {
        let c = GameplayContext {
            status: kirby_assist_core::native_states::GUARD,
            can_act: true,
            ..context(frame, &h)
        };
        assert_ne!(
            o.sample_automatic_configured(
                true,
                shield(c),
                c,
                target(e),
                Some(e),
                true,
                22.,
                35.,
                Some(8.),
                settings(),
                cfg
            )
            .requested
                & MAPPER_B,
            0
        );
        o.mapped_up_special(frame as i64, 2);
    }
    for frame in 4..9 {
        o.delivery_tick(frame as i64);
        let c = GameplayContext {
            status: kirby_assist_core::native_states::GUARD,
            can_act: true,
            ..context(frame, &h)
        };
        let out = o.sample_automatic_configured(
            true,
            shield(c),
            c,
            target(e),
            Some(e),
            true,
            22.,
            35.,
            Some(8.),
            settings(),
            cfg,
        );
        assert_eq!(out.owned, 0);
        assert_eq!(out.requested, 0);
        assert!(o.up_b_receipt_pending());
    }
    for frame in 9..78 {
        o.delivery_tick(frame as i64);
        let c = context(frame, &h);
        let e = enemy(frame);
        let out = o.sample_automatic_configured(
            true,
            shield(c),
            c,
            [None; 8],
            Some(e),
            false,
            22.,
            35.,
            Some(8.),
            settings(),
            cfg,
        );
        assert_eq!(out.owned, SEQUENCE_BUTTONS, "drift frame {}", frame - 9);
        assert_eq!(out.requested, 0);
        assert_eq!(out.stick, Some([e.position[0].signum(), 0.]));
    }
    let c = context(78, &h);
    assert_eq!(
        o.sample_automatic_configured(
            true,
            shield(c),
            c,
            [None; 8],
            Some(enemy(78)),
            false,
            22.,
            35.,
            Some(8.),
            settings(),
            cfg
        )
        .owned,
        0
    );
}
#[test]
fn all_late_confirmed_sequence_tails_keep_their_full_budget_after_hitstop() {
    let h = history();
    let r = rules("auto_roll");
    let mut engine = Engine::default();
    let mut response = Response::default();
    engine.sample_simple(
        true,
        &r,
        settings(),
        context(0, &h),
        target(enemy(0)),
        Some(enemy(0)),
    );
    response.start_up_b(recipe("auto_grab_dodged"), context(0, &h), enemy(0));
    response.sample(true, settings(), context(0, &h), Some(enemy(0)));
    engine.mapped_up_special(0, 2);
    response.mapped_up_special(0, 2);
    for frame in 1..9 {
        engine.delivery_tick(frame as i64);
        response.delivery_tick(frame as i64);
        let c = GameplayContext {
            status: kirby_assist_core::native_states::GUARD_DAMAGE,
            stopped: true,
            can_act: false,
            ..context(frame, &h)
        };
        for out in [
            engine.sample_simple(true, &r, settings(), c, [None; 8], Some(enemy(frame))),
            response.sample(true, settings(), c, Some(enemy(frame))),
        ] {
            assert_eq!(out.owned, 0);
        }
    }
    for frame in 9..78 {
        for out in [
            engine.sample_simple(
                true,
                &r,
                settings(),
                context(frame, &h),
                [None; 8],
                Some(enemy(frame)),
            ),
            response.sample(true, settings(), context(frame, &h), Some(enemy(frame))),
        ] {
            assert_eq!(out.owned, SEQUENCE_BUTTONS, "frame {frame}");
            assert_eq!(out.requested, 0);
            assert_eq!(out.stick, Some([enemy(frame).position[0].signum(), 0.]));
        }
    }
    engine.sample_simple(
        true,
        &r,
        settings(),
        context(78, &h),
        [None; 8],
        Some(enemy(78)),
    );
    response.sample(true, settings(), context(78, &h), Some(enemy(78)));
    assert!(!engine.committed());
    assert!(!response.automatic_up_b());
}
#[test]
fn pending_receipt_never_survives_ledge_stop_damage_capture_target_loss_or_airborne_start() {
    for case in 0..6 {
        let b = recipe("auto_shield_backup");
        let cfg = AutomaticConfig {
            backup: Some(&b),
            ..Default::default()
        };
        let mut o = OutOfShield::new();
        let h = history();
        let c = context(0, &h);
        let mut e = enemy(0);
        e.status = kirby_assist_core::native_states::status::WAIT;
        e.vulnerable = true;
        o.sample_automatic_configured(
            true,
            shield(c),
            c,
            target(e),
            Some(e),
            true,
            22.,
            35.,
            Some(8.),
            settings(),
            cfg,
        );
        o.mapped_up_special(0, 2);
        let mut c = GameplayContext {
            status: kirby_assist_core::native_states::GUARD_DAMAGE,
            stopped: true,
            can_act: false,
            ..context(1, &h)
        };
        let mut e = Some(e);
        match case {
            0 => {}
            1 => c.status = kirby_assist_core::native_states::status::DAMAGE_FLY,
            2 => c.status = kirby_assist_core::native_states::CAPTURE_PULLED,
            3 => e = None,
            4 => c.situation = 2,
            _ => e.as_mut().unwrap().status = kirby_assist_core::native_states::CAPTURE_PULLED,
        }
        let out = o.sample_automatic_configured(
            case != 0,
            shield(c),
            c,
            [None; 8],
            e,
            true,
            22.,
            35.,
            Some(8.),
            settings(),
            cfg,
        );
        assert_eq!(out.owned, 0);
        assert!(!o.shield_up_b_pending(), "case {case}");
        let c = context(2, &h);
        assert_eq!(
            o.sample_automatic_configured(
                true,
                shield(c),
                c,
                [None; 8],
                Some(enemy(2)),
                false,
                22.,
                35.,
                Some(8.),
                settings(),
                cfg
            )
            .owned,
            0,
            "no stale tracking can restart"
        );
    }
}
#[test]
fn passive_receipt_scope_does_not_acquire_and_reserves_priority_without_owning_inputs() {
    use kirby_assist_core::{
        action_owner::{ActionOwnership, Intent, Owner},
        reaction_rules::AutomaticObservationScope,
    };
    for status in [
        kirby_assist_core::native_states::GUARD_ON,
        kirby_assist_core::native_states::GUARD,
        kirby_assist_core::native_states::GUARD_OFF,
        kirby_assist_core::native_states::GUARD_DAMAGE,
    ] {
        let scope = AutomaticObservationScope::new(
            false,
            status,
            kirby_assist_core::native_states::status::WAIT,
            true,
            false,
            true,
            false,
            false,
            false,
        )
        .with_up_b_receipt(
            false,
            false,
            true,
            status,
            kirby_assist_core::native_states::status::WAIT,
            false,
        );
        assert!(!scope.acquire);
        assert!(scope.follow);
    }
    for (suspended, linked, status, situation, receipt) in [
        (true, false, 0x1e, 0, true),
        (false, true, 0x1e, 0, true),
        (false, false, 0x49, 0, true),
        (false, false, 0x41, 0, true),
        (false, false, 0x1e, 2, true),
        (false, false, 0x1e, 0, false),
    ] {
        let scope = AutomaticObservationScope::new(
            suspended, status, situation, true, linked, true, false, false, false,
        )
        .with_up_b_receipt(suspended, linked, receipt, status, situation, false);
        assert!(!scope.acquire && !scope.follow);
    }
    for owner in [Owner::RollRead, Owner::OptionRead, Owner::OutOfShield] {
        let mut arbiter = ActionOwnership::new();
        let receipt = Intent::sequence_receipt(owner, 0, true);
        assert_eq!(arbiter.select(true, &[receipt]), Some(owner));
        assert!(!arbiter.allows(Owner::Shortcut));
        assert!(!arbiter.allows(Owner::GrabDodge));
        assert_eq!(
            arbiter.select(
                true,
                &[
                    receipt,
                    Intent::input_sequence(Owner::Safety, SEQUENCE_BUTTONS)
                ]
            ),
            Some(Owner::Safety)
        );
        arbiter.set_automatic_suspended(true);
        assert!(!arbiter.allows(owner));
    }
    assert!(!mapped_up_special_delivered(MAPPED_GUARD, 127));
    assert!(!mapped_up_special_delivered(MAPPED_SPECIAL, 0));
    assert!(!mapped_up_special_delivered(MAPPED_SPECIAL, -127));
    assert!(mapped_up_special_delivered(MAPPED_SPECIAL, 127));
}

#[test]
fn rejected_mapper_packets_never_arm_late_tracking_and_wrong_target_receipts_are_ignored() {
    let h = history();
    let r = rules("auto_roll");
    let mut engine = Engine::default();
    let mut response = Response::default();
    engine.sample_simple(
        true,
        &r,
        settings(),
        context(0, &h),
        target(enemy(0)),
        Some(enemy(0)),
    );
    response.start_up_b(recipe("auto_grab_dodged"), context(0, &h), enemy(0));
    response.sample(true, settings(), context(0, &h), Some(enemy(0)));
    let b = recipe("auto_shield_backup");
    let cfg = AutomaticConfig {
        backup: Some(&b),
        ..Default::default()
    };
    let mut o = OutOfShield::new();
    let mut e = enemy(0);
    e.status = kirby_assist_core::native_states::status::WAIT;
    e.vulnerable = true;
    o.sample_automatic_configured(
        true,
        shield(context(0, &h)),
        context(0, &h),
        target(e),
        Some(e),
        true,
        22.,
        35.,
        Some(8.),
        settings(),
        cfg,
    );
    engine.attempted_up_special(0, 2);
    response.attempted_up_special(0, 2);
    o.attempted_up_special(0, 2);
    engine.mapped_up_special(0, 999);
    response.mapped_up_special(0, 999);
    o.mapped_up_special(0, 999);
    assert!(
        !engine.up_b_receipt_pending()
            && !response.up_b_receipt_pending()
            && !o.up_b_receipt_pending()
    );
    let c = GameplayContext {
        status: kirby_assist_core::native_states::GUARD_DAMAGE,
        stopped: true,
        can_act: false,
        ..context(1, &h)
    };
    engine.sample_simple(true, &r, settings(), c, [None; 8], Some(e));
    response.sample(true, settings(), c, Some(e));
    o.sample_automatic_configured(
        true,
        shield(c),
        c,
        [None; 8],
        Some(e),
        true,
        22.,
        35.,
        Some(8.),
        settings(),
        cfg,
    );
    assert!(!engine.committed() && response.target().is_none() && o.target().is_none());
}
#[test]
fn receipt_expiry_is_mapper_bounded_but_native_confirmation_wins_at_the_deadline() {
    for confirms in [false, true] {
        let h = history();
        let r = rules("auto_roll");
        let mut engine = Engine::default();
        let mut response = Response::default();
        engine.sample_simple(
            true,
            &r,
            settings(),
            context(0, &h),
            target(enemy(0)),
            Some(enemy(0)),
        );
        response.start_up_b(recipe("auto_grab_dodged"), context(0, &h), enemy(0));
        response.sample(true, settings(), context(0, &h), Some(enemy(0)));
        engine.mapped_up_special(0, 2);
        response.mapped_up_special(0, 2);
        for tick in 1..=23 {
            for _ in 0..10 {
                engine.delivery_tick(tick);
                response.delivery_tick(tick);
            }
            let c = if tick == 23 && confirms {
                context(1, &h)
            } else {
                GameplayContext {
                    status: kirby_assist_core::native_states::GUARD_DAMAGE,
                    stopped: true,
                    can_act: false,
                    ..context(1, &h)
                }
            };
            for out in [
                engine.sample_simple(true, &r, settings(), c, [None; 8], Some(enemy(1))),
                response.sample(true, settings(), c, Some(enemy(1))),
            ] {
                assert_eq!(out.requested, 0);
                assert_eq!(
                    out.owned,
                    if tick == 23 && confirms {
                        SEQUENCE_BUTTONS
                    } else {
                        0
                    }
                );
            }
            if tick < 23 {
                assert!(engine.committed() && response.automatic_up_b());
            }
        }
        assert_eq!(engine.committed(), confirms);
        assert_eq!(response.automatic_up_b(), confirms);
    }
}
#[test]
fn losing_the_locked_target_during_passive_receipt_releases_priority_immediately() {
    let h = history();
    let r = rules("auto_roll");
    let mut engine = Engine::default();
    engine.sample_simple(
        true,
        &r,
        settings(),
        context(0, &h),
        target(enemy(0)),
        Some(enemy(0)),
    );
    engine.mapped_up_special(0, 2);
    let c = GameplayContext {
        status: kirby_assist_core::native_states::GUARD_DAMAGE,
        stopped: true,
        can_act: false,
        ..context(1, &h)
    };
    assert_eq!(
        engine
            .sample_simple(true, &r, settings(), c, [None; 8], None)
            .owned,
        0
    );
    assert!(!engine.committed());
}

#[test]
fn confirmed_spin_cannot_follow_into_an_unrelated_custom_move_or_final() {
    for status in [
        kirby_assist_core::native_states::status::FINAL,
        0x1e3,
        0x1e6,
    ] {
        let h = history();
        let rules = rules("auto_roll");
        let mut engine = Engine::default();
        let mut response = Response::default();
        let start = context(0, &h);
        assert!(response.start_up_b(recipe("auto_grab_dodged"), start, enemy(0)));
        let b = recipe("auto_shield_backup");
        let config = AutomaticConfig {
            backup: Some(&b),
            ..Default::default()
        };
        let mut oos = OutOfShield::new();
        let shield_enemy = Opponent {
            status: kirby_assist_core::native_states::status::WAIT,
            vulnerable: true,
            ..enemy(0)
        };
        engine.sample_simple(
            true,
            &rules,
            settings(),
            start,
            target(enemy(0)),
            Some(enemy(0)),
        );
        oos.sample_automatic_configured(
            true,
            shield(start),
            start,
            target(shield_enemy),
            Some(shield_enemy),
            true,
            22.,
            35.,
            Some(8.),
            settings(),
            config,
        );
        let confirmed = context(1, &h);
        engine.sample_simple(
            true,
            &rules,
            settings(),
            confirmed,
            [None; 8],
            Some(enemy(1)),
        );
        response.sample(true, settings(), confirmed, Some(enemy(1)));
        oos.sample_automatic_configured(
            true,
            shield(confirmed),
            confirmed,
            [None; 8],
            Some(enemy(1)),
            false,
            22.,
            35.,
            Some(8.),
            settings(),
            config,
        );
        let changed = GameplayContext {
            status,
            previous_status: kirby_assist_core::native_states::status::SPECIAL_HI,
            ..context(5, &h)
        };
        for output in [
            engine.sample_simple(true, &rules, settings(), changed, [None; 8], Some(enemy(5))),
            response.sample(true, settings(), changed, Some(enemy(5))),
            oos.sample_automatic_configured(
                true,
                shield(changed),
                changed,
                [None; 8],
                Some(enemy(5)),
                false,
                22.,
                35.,
                Some(8.),
                settings(),
                config,
            ),
        ] {
            assert_eq!(output.owned, 0, "status {status:#x}");
            assert_eq!(output.requested, 0);
            assert_eq!(output.stick, None);
        }
    }
}
