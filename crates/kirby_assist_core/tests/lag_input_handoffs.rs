//! Released manual jumps and queued grabs under delayed native feedback.
use kirby_assist_core::{
    action_owner::{Intent, Owner},
    button_bindings::Builtin,
    defense_response::Response,
    fighter_profiles::{
        presets::{MenuPresets, PresetSlot},
        FighterProfiles,
    },
    gameplay::{ControllerOutput, GameplayContext, MotionTiming, Opponent},
    grab_handoff::GrabHandoff,
    ledge_menu::MenuSelection,
    ledge_trap::{InputTimeline, LedgeSettings},
    manual_controls::ManualJumpFence,
    native_input::*,
    reaction_rules::{Candidate, Engine, Resolved},
    roll_read::Target,
    sequence_config::{Catalog, Sequence},
};
fn c(frame: f32) -> GameplayContext {
    GameplayContext {
        gameplay: true,
        held: true,
        can_act: true,
        status: kirby_assist_core::native_states::status::WAIT,
        situation: 0,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.,
        facing: 1.,
        safe_ground: [true; 2],
        ..Default::default()
    }
}
fn enemy(frame: f32) -> Opponent {
    Opponent {
        id: 2,
        status: kirby_assist_core::native_states::ESCAPE_F,
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
fn sequence(actions: &str) -> Sequence {
    Catalog::parse(&format!(
        "[bowser.roll.test]\nstart_delay=0\nreaction=auto_roll:40:12\nactions={actions}\n"
    ))
    .unwrap()
    .entries()[0]
        .sequence
}
fn settings() -> LedgeSettings {
    LedgeSettings {
        fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
        ..Default::default()
    }
}
fn presets(text: &str) -> MenuPresets {
    let mut p = FighterProfiles::default();
    p.add_sequence_choices(&Catalog::parse(text).unwrap())
        .unwrap();
    p.profile(15).unwrap().menu_options
}
fn button(m: &mut MenuSelection, p: &MenuPresets, raw: u32, layout: ControlLayout) {
    m.set_binding_input(raw, layout);
    m.input(layout.policy_buttons(raw), p);
}
#[test]
fn manual_jump_roles_follow_every_short_hop_binding_and_fixed_layout_jump() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for (slot, token) in [
            (PresetSlot::X, "x"),
            (PresetSlot::Y, "y"),
            (PresetSlot::DpadUp, "dpad_up"),
            (PresetSlot::DpadDown, "dpad_down"),
            (PresetSlot::DpadLeft, "dpad_left"),
            (PresetSlot::DpadRight, "dpad_right"),
            (PresetSlot::L3, "l3"),
            (PresetSlot::R3, "r3"),
        ] {
            let p = presets(&format!(
                "[bowser.{token}.hop]\nbuiltin={}\ndefault=true\n",
                Builtin::ShortHop.token()
            ));
            let mut m = MenuSelection::new(&p);
            button(&mut m, &p, 0, layout);
            button(&mut m, &p, slot.button(), layout);
            assert_ne!(
                m.manual_native_actions(&p) & SEQUENCE_JUMP_BUTTONS,
                0,
                "{layout:?} {slot:?}"
            );
            button(&mut m, &p, 0, layout);
            assert_eq!(m.manual_native_actions(&p), 0);
        }
        let p = presets("");
        for raw in if layout == ControlLayout::Default {
            vec![MAPPER_Y, MAPPER_ZL]
        } else {
            vec![MAPPER_R, MAPPER_ZR]
        } {
            let mut m = MenuSelection::new(&p);
            button(&mut m, &p, 0, layout);
            button(&mut m, &p, raw, layout);
            assert_ne!(
                m.manual_native_actions(&p) & SEQUENCE_JUMP_BUTTONS,
                0,
                "{layout:?} {raw:#x}"
            );
        }
    }
    let p = presets("[bowser.x.normal]\nbuiltin=normal\ndefault=true\n");
    let mut m = MenuSelection::new(&p);
    button(&mut m, &p, 0, ControlLayout::Default);
    button(&mut m, &p, MAPPER_X, ControlLayout::Default);
    assert_ne!(m.manual_native_actions(&p) & MAPPER_X, 0);
}
#[test]
fn released_jump_remains_protected_during_lag_then_clears_on_native_jump() {
    let mut fence = ManualJumpFence::new();
    assert!(fence.sample(1, 15, true, c(0.)));
    for _ in 0..1000 {
        assert!(fence.sample(1, 15, false, c(0.)));
    }
    assert!(!fence.sample(
        1,
        15,
        false,
        GameplayContext {
            status: kirby_assist_core::native_states::status::JUMP_SQUAT,
            ..c(1.)
        }
    ));
    assert!(!fence.pending());
    assert!(!fence.sample(
        1,
        15,
        false,
        GameplayContext {
            situation: 2,
            ..c(2.)
        }
    ));
    assert!(
        !fence.sample(1, 15, false, c(3.)),
        "landing does not add a wait"
    );
}
#[test]
fn jump_fence_uses_native_progress_and_resets_on_actor_or_scope_change() {
    let mut fence = ManualJumpFence::new();
    fence.sample(1, 15, true, c(0.));
    for frame in 1..24 {
        assert!(fence.sample(1, 15, false, c(frame as f32)));
    }
    assert!(!fence.sample(1, 15, false, c(24.)));
    fence.sample(1, 15, true, c(25.));
    for _ in 0..100 {
        assert!(fence.sample(
            1,
            15,
            false,
            GameplayContext {
                stopped: true,
                ..c(25.)
            }
        ));
    }
    assert!(!fence.sample(1, 5, false, c(25.)));
    fence.sample(1, 5, true, c(25.));
    assert!(!fence.sample(2, 5, false, c(25.)));
    fence.sample(2, 5, true, c(25.));
    assert!(!fence.sample(
        2,
        5,
        false,
        GameplayContext {
            gameplay: false,
            ..c(25.)
        }
    ));
}
#[test]
fn delayed_automatic_grab_and_up_b_cancel_in_jump_startup_or_air() {
    for actions in ["wait:1,up_b:4", "wait:1,grab:4"] {
        for airborne in [false, true] {
            let next = if airborne {
                GameplayContext {
                    situation: 2,
                    ..c(1.)
                }
            } else {
                GameplayContext {
                    status: kirby_assist_core::native_states::status::JUMP_SQUAT,
                    ..c(1.)
                }
            };
            let mut response = Response::default();
            assert!(response.start_sequence(sequence(actions), c(0.), enemy(0.)));
            assert_eq!(
                response
                    .sample(true, settings(), c(0.), Some(enemy(0.)))
                    .requested,
                0
            );
            assert_eq!(
                response.sample(true, settings(), next, Some(enemy(1.))),
                ControllerOutput::default()
            );
            let mut rules = Resolved::default();
            rules
                .add(Candidate {
                    slot: 2,
                    index: 0,
                    sequence: sequence(actions),
                })
                .unwrap();
            let targets = |frame| {
                let mut t = [None; 8];
                t[0] = Some(Target {
                    id: 2,
                    status: kirby_assist_core::native_states::ESCAPE_F,
                    motion: 2,
                    frame,
                    rate: 1.,
                    position: [10., 0.],
                    velocity_x: 0.,
                    vulnerable_in: Some(0.),
                    actionable_in: None,
                });
                t
            };
            let mut engine = Engine::default();
            assert_eq!(
                engine
                    .sample_simple(
                        true,
                        &rules,
                        settings(),
                        c(0.),
                        targets(0.),
                        Some(enemy(0.))
                    )
                    .requested,
                0
            );
            assert_eq!(
                engine.sample_simple(true, &rules, settings(), next, targets(1.), Some(enemy(1.))),
                ControllerOutput::default()
            );
        }
    }
}
#[test]
fn literal_manual_tapes_retain_their_explicit_airborne_specials() {
    let mut tape = InputTimeline::new(sequence("wait:1,up_b:4"), enemy(0.));
    tape.sample(settings(), c(0.), Some(enemy(0.)), None, 0.);
    let out = tape.sample(
        settings(),
        GameplayContext {
            situation: 2,
            ..c(1.)
        },
        Some(enemy(1.)),
        None,
        0.,
    );
    assert_eq!(out.requested, MAPPER_B);
}
#[test]
fn up_b_tracking_protection_requires_a_delivered_packet_or_native_confirmation() {
    let s = sequence("up_b:4,drift:opponent:1:69");
    let mut response = Response::default();
    assert!(response.start_up_b(s, c(0.), enemy(0.)));
    assert!(!response.up_b_delivery_started());
    response.sample(true, settings(), c(0.), Some(enemy(0.)));
    response.attempted_up_special(0, 2);
    assert!(!response.up_b_delivery_started());
    response.mapped_up_special(1, 999);
    assert!(!response.up_b_delivery_started());
    response.mapped_up_special(1, 2);
    assert!(response.up_b_delivery_started());
    let mut native = Response::default();
    native.start_up_b(s, c(0.), enemy(0.));
    native.sample(
        true,
        settings(),
        GameplayContext {
            status: kirby_assist_core::native_states::status::SPECIAL_HI,
            ..c(1.)
        },
        Some(enemy(1.)),
    );
    assert!(native.up_b_delivery_started());
    let mut rules = Resolved::default();
    rules
        .add(Candidate {
            slot: 2,
            index: 0,
            sequence: s,
        })
        .unwrap();
    let mut targets = [None; 8];
    targets[0] = Some(Target {
        id: 2,
        status: kirby_assist_core::native_states::ESCAPE_F,
        motion: 2,
        frame: 0.,
        rate: 1.,
        position: [10., 0.],
        velocity_x: 0.,
        vulnerable_in: Some(0.),
        actionable_in: None,
    });
    let mut engine = Engine::default();
    engine.sample_simple(true, &rules, settings(), c(0.), targets, Some(enemy(0.)));
    assert!(!engine.up_b_delivery_started());
    engine.mapped_up_special(1, 2);
    assert!(engine.up_b_delivery_started());
}
fn grab() -> ControllerOutput {
    ControllerOutput {
        requested: MAPPER_R,
        stick: Some([0., 0.]),
        ..Default::default()
    }
}
#[test]
fn frozen_native_feedback_cannot_rearm_grab_or_leak_a_competing_special() {
    let mut h = GrabHandoff::new();
    h.observe_native(true, Some(c(0.)));
    for tick in 0..4 {
        assert!(h.permits(Some(Owner::OutOfShield), grab(), None));
        h.mapped(tick, Some(Owner::OutOfShield), true, true, true);
        h.observe_native(true, Some(c(0.)));
    }
    for tick in 4..1000 {
        h.observe_native(true, Some(c(0.)));
        h.observe_native(true, None);
        assert!(h.pending());
        assert!(!h.permits(Some(Owner::OutOfShield), grab(), None));
        assert!(!h.permits(
            Some(Owner::RollRead),
            ControllerOutput {
                requested: MAPPER_B,
                stick: Some([0., 1.]),
                ..Default::default()
            },
            None
        ));
        h.mapped(tick, None, false, false, true);
    }
    h.observe_native(
        true,
        Some(GameplayContext {
            status: kirby_assist_core::native_states::CATCH,
            ..c(1.)
        }),
    );
    assert!(!h.pending());
}
#[test]
fn native_grab_receipt_expires_only_after_actual_progress_or_scope_loss() {
    let mut h = GrabHandoff::new();
    h.observe_native(true, Some(c(0.)));
    h.mapped(0, None, true, true, true);
    for frame in 1..24 {
        h.observe_native(true, Some(c(frame as f32)));
        assert!(h.pending());
    }
    h.observe_native(true, Some(c(24.)));
    assert!(!h.pending());
    h.mapped(25, None, false, false, true);
    h.mapped(26, None, true, true, true);
    assert!(h.pending());
    h.observe_native(false, None);
    assert!(!h.pending());
}
#[test]
fn queued_grab_allows_manual_jump_without_clearing_receipt_or_allowing_special() {
    let mut h = GrabHandoff::new();
    h.observe_native(true, Some(c(0.)));
    h.mapped(0, Some(Owner::OutOfShield), true, true, true);
    let jump = ControllerOutput {
        requested: MAPPER_X,
        ..Default::default()
    };
    let mut intents = [
        Intent::input_sequence(Owner::Shortcut, 1),
        Intent::input_sequence(Owner::RollRead, 1),
    ];
    h.restrict_with_manual_jump(&mut intents, GrabHandoff::manual_jump_packet(jump, None));
    assert!(intents[0].present);
    assert!(!intents[1].present);
    assert!(h.permits(Some(Owner::Shortcut), jump, None));
    assert!(h.pending());
    for out in [
        ControllerOutput {
            requested: MAPPER_X | MAPPER_B,
            ..jump
        },
        ControllerOutput {
            stick: Some([0., 1.]),
            ..jump
        },
    ] {
        assert!(!h.permits(Some(Owner::Shortcut), out, None));
    }
}

#[test]
fn explicit_rule_control_is_not_treated_as_a_manual_jump() {
    let p = presets("[bowser.roll.rule]\nstart_delay=0\nreaction=roll:any:0:20:12:any:1:toggle_x\nactions=jab:4\n");
    let mut m = MenuSelection::new(&p);
    button(&mut m, &p, 0, ControlLayout::Default);
    button(&mut m, &p, MAPPER_X, ControlLayout::Default);
    assert_eq!(m.manual_native_actions(&p), 0);
}
#[test]
fn duplicate_receipts_cannot_spend_the_four_packet_grab_budget() {
    let mut h = GrabHandoff::new();
    h.observe_native(true, Some(c(0.)));
    for tick in 0..4 {
        assert!(h.permits(Some(Owner::Ledge), grab(), None));
        for _ in 0..100 {
            h.mapped(tick, Some(Owner::Ledge), true, true, true);
        }
    }
    assert!(!h.permits(Some(Owner::Ledge), grab(), None));
}

#[test]
fn committed_automatic_binding_locks_block_manual_short_hop_starts() {
    use kirby_assist_core::action_owner::ActionOwnership;
    for owner in [
        Owner::RollRead,
        Owner::OutOfShield,
        Owner::OptionRead,
        Owner::Reflect,
        Owner::GrabDodge,
        Owner::Trip,
        Owner::JabLock,
        Owner::Fair,
        Owner::Finisher,
    ] {
        let mut ownership = ActionOwnership::new();
        ownership.select(true, &[Intent::input_sequence(owner, 1)]);
        assert!(!ownership.manual_jump_preempts(false));
        assert!(!ownership.manual_jump_preempts(true));
        for (source, slot) in [(MAPPER_X, "x"), (PresetSlot::DpadUp.button(), "dpad_up")] {
            let p = presets(&format!(
                "[bowser.{slot}.hop]\nbuiltin=short_hop\ndefault=true\n"
            ));
            let mut m = MenuSelection::new(&p);
            button(&mut m, &p, 0, ControlLayout::Default);
            m.block_binding_starts(true);
            m.preserve_safety_jump(ownership.manual_jump_preempts(false));
            button(&mut m, &p, source, ControlLayout::Default);
            assert_eq!(
                m.manual_native_actions(&p) & SEQUENCE_JUMP_BUTTONS,
                0,
                "{owner:?} {slot}"
            );
            assert_eq!(
                m.native_buttons(source) & MAPPER_X,
                0,
                "an unrelated jump cannot enter the active tape"
            );
        }
    }
    for owner in [Owner::Ledge, Owner::Shortcut, Owner::Safety, Owner::Trump] {
        let mut ownership = ActionOwnership::new();
        ownership.select(true, &[Intent::input_sequence(owner, 1)]);
        assert!(!ownership.manual_jump_preempts(false));
    }
}
