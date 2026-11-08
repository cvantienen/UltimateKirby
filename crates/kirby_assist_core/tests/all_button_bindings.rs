//! Physical binding routing and persistence, independent of native game calls.
use kirby_assist_core::{
    button_bindings::Builtin,
    fighter_profiles::{
        presets::{MenuPresets, PresetSlot},
        FighterProfiles,
    },
    manual_controls::TURBO_TRIGGER,
    native_input::*,
    sequence_config::{persist, Catalog, Sequence},
    settings_menu::{MenuRow, MenuSelection, MENU_STICK_CLICK},
};
fn token(slot: PresetSlot) -> &'static str {
    match slot {
        PresetSlot::X => "x",
        PresetSlot::Y => "y",
        PresetSlot::DpadUp => "dpad_up",
        PresetSlot::DpadDown => "dpad_down",
        PresetSlot::DpadLeft => "dpad_left",
        PresetSlot::DpadRight => "dpad_right",
        PresetSlot::L3 => "l3",
        PresetSlot::R3 => "r3",
        _ => unreachable!(),
    }
}
fn presets(text: &str) -> MenuPresets {
    let mut p = FighterProfiles::default();
    p.add_sequence_choices(&Catalog::parse(text).unwrap())
        .unwrap();
    p.profile(15).unwrap().menu_options
}
fn input(m: &mut MenuSelection, p: &MenuPresets, raw: u32, layout: ControlLayout) -> u32 {
    m.set_binding_input(raw, layout);
    m.input(layout.policy_buttons(raw), p)
}
#[test]
fn every_physical_button_runs_only_its_custom_recipe_in_both_layouts() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for slot in PresetSlot::BUTTONS {
            let p = presets(&format!(
                "[bowser.{}.saved]\nstart_delay=0\ndefault=true\nactions=up_b:4\n",
                token(slot)
            ));
            let mut m = MenuSelection::new(&p);
            input(&mut m, &p, 0, layout);
            let consumed = input(&mut m, &p, slot.button(), layout);
            assert_eq!(m.preview_slot, Some(slot), "{layout:?} {slot:?}");
            assert_ne!(
                layout.physical_buttons(consumed) & slot.button(),
                0,
                "source ownership {layout:?} {slot:?}"
            );
            assert_eq!(
                m.native_buttons(layout.policy_buttons(slot.button()))
                    & (MAPPER_X | MAPPER_L | TURBO_TRIGGER),
                0,
                "custom must not double-activate native helper: {layout:?} {slot:?}"
            );
        }
    }
}
#[test]
fn each_native_helper_can_be_assigned_to_every_physical_button() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for slot in PresetSlot::BUTTONS {
            for (builtin, expected) in [
                (Builtin::ShortHop, MAPPER_X),
                (Builtin::LedgeAssist, MAPPER_L),
                (Builtin::TurboTeabag, TURBO_TRIGGER),
            ] {
                let p = presets(&format!(
                    "[bowser.{}.native]\nbuiltin={}\ndefault=true\n",
                    token(slot),
                    builtin.token()
                ));
                let mut m = MenuSelection::new(&p);
                input(&mut m, &p, 0, layout);
                let consumed = input(&mut m, &p, slot.button(), layout);
                assert!(
                    m.preview_slot.is_none(),
                    "native must not start a tape: {layout:?} {slot:?} {builtin:?}"
                );
                assert_eq!(
                    m.native_buttons(layout.policy_buttons(slot.button()))
                        & (MAPPER_X | MAPPER_L | TURBO_TRIGGER),
                    expected,
                    "{layout:?} {slot:?} {builtin:?}"
                );
                assert_ne!(layout.physical_buttons(consumed) & slot.button(), 0);
            }
        }
    }
}
#[test]
fn normal_choice_removes_assistance_and_returns_physical_button_to_game() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for slot in PresetSlot::BUTTONS {
            let p = presets(&format!(
                "[bowser.{}.normal]\nbuiltin=normal\ndefault=true\n",
                token(slot)
            ));
            let mut m = MenuSelection::new(&p);
            input(&mut m, &p, 0, layout);
            let consumed = input(&mut m, &p, slot.button(), layout);
            assert_eq!(
                layout.physical_buttons(consumed) & slot.button(),
                0,
                "normal source must pass: {layout:?} {slot:?}"
            );
            assert_eq!(
                m.native_buttons(layout.policy_buttons(slot.button()))
                    & (MAPPER_X | MAPPER_L | TURBO_TRIGGER),
                0
            );
        }
    }
}
#[test]
fn fresh_profiles_use_requested_physical_defaults_in_both_layouts() {
    let p = presets("");
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for (source, expected) in [
            (
                MAPPER_L,
                if layout == ControlLayout::Default {
                    MAPPER_L
                } else {
                    0
                },
            ),
            (MAPPER_X, MAPPER_X),
            (4, 0),
            (TURBO_TRIGGER, TURBO_TRIGGER),
            (MAPPER_Y, 0),
            (MAPPER_R, 0),
            (MAPPER_ZR, 0),
        ] {
            let mut m = MenuSelection::new(&p);
            input(&mut m, &p, 0, layout);
            input(&mut m, &p, source, layout);
            assert_eq!(
                m.native_buttons(layout.policy_buttons(source))
                    & (MAPPER_X | MAPPER_L | TURBO_TRIGGER),
                expected,
                "{layout:?} source {source:#x}"
            );
        }
    }
}
#[test]
fn stick_click_menu_controls_win_only_when_menu_is_visible() {
    for slot in [PresetSlot::L3, PresetSlot::R3] {
        let p = presets(&format!(
            "[bowser.{}.saved]\nstart_delay=0\ndefault=true\nactions=grab:4\n",
            token(slot)
        ));
        let mut m = MenuSelection::new(&p);
        m.show_quick();
        input(&mut m, &p, 0, ControlLayout::Default);
        input(&mut m, &p, slot.button(), ControlLayout::Default);
        assert_ne!(
            m.preview_slot,
            Some(slot),
            "menu controls must not start their assigned gameplay recipe"
        );
        if slot == PresetSlot::R3 {
            assert!(m.preview_slot.is_none());
        }
    }
}
#[test]
fn builtin_save_defaults_round_trip_in_every_new_slot() {
    let original = "[bowser.neutral.perfect]\nstart_delay=7\ndefault=true\nactions=down_smash:4\n";
    for slot in PresetSlot::BUTTONS {
        for builtin in Builtin::ALL {
            let sequence = Sequence {
                builtin: Some(builtin),
                ..Default::default()
            };
            let updated = persist::update_named(
                original,
                15,
                slot,
                builtin.choice_id(),
                sequence,
                true,
                Some(builtin.label()),
            )
            .unwrap();
            let saved = persist::set_default(&updated, 15, slot, builtin.choice_id()).unwrap();
            let p = presets(&saved);
            let m = MenuSelection::new(&p);
            assert_eq!(
                m.choice(&p, slot, m.choices[slot as usize])
                    .unwrap()
                    .sequence
                    .builtin,
                Some(builtin)
            );
            assert_eq!(
                Catalog::parse(&saved)
                    .unwrap()
                    .find(15, PresetSlot::Neutral, "perfect")
                    .unwrap()
                    .start_delay,
                7
            );
        }
    }
}
#[test]
fn builtins_cannot_be_configured_as_conditioned_or_timed_attack_recipes() {
    for extra in [
        "actions=up_b:4",
        "start_delay=1",
        "ground_only=true",
        "while_held=true",
        "reaction=auto_roll:80:30",
        "mode=ledge_trump",
    ] {
        assert!(
            Catalog::parse(&format!("[bowser.r3.native]\nbuiltin=short_hop\n{extra}\n")).is_err(),
            "{extra}"
        );
    }
    assert!(Catalog::parse("[bowser.neutral.native]\nbuiltin=short_hop\n").is_err());
}
#[test]
fn builtins_can_be_selected_and_saved_as_default_from_editor() {
    let p = presets("");
    let mut m = MenuSelection::new(&p);
    m.show_quick();
    m.button_menu = true;
    m.row = MenuRow::R3Sequence;
    m.choices[PresetSlot::R3 as usize] =
        p.list(PresetSlot::R3).options.len() + Builtin::ShortHop.template_offset();
    m.input(0, &p);
    m.input(MENU_STICK_CLICK, &p);
    assert!(m.recipe_editing);
    m.input(0, &p);
    m.input(MAPPER_Y, &p);
    assert!(m.save_requested && m.default_requested);
    assert!(m
        .recipe_panel(&p)
        .header
        .iter()
        .any(|s| s.contains("Short hop")));
}

#[test]
fn compact_helpers_keep_their_bindings_and_training_l3_tests_the_hovered_recipe() {
    let p=presets("[bowser.neutral.test]\nstart_delay=0\nactions=grab:4\n[bowser.x.hop]\nbuiltin=short_hop\ndefault=true\n");
    let mut training = MenuSelection::new(&p);
    training.show_quick();
    input(&mut training, &p, 0, ControlLayout::Default);
    input(&mut training, &p, MAPPER_X, ControlLayout::Default);
    assert_ne!(training.native_buttons(MAPPER_X) & MAPPER_X, 0);
    assert!(training.preview_slot.is_none());
    input(&mut training, &p, 0, ControlLayout::Default);
    input(&mut training, &p, 1 << 16, ControlLayout::Default);
    assert_eq!(training.preview_slot, Some(PresetSlot::Neutral));
    let mut battle = MenuSelection::new(&p);
    battle.input_with_editing(0, &p, false);
    battle.show_quick();
    battle.input_with_editing(MAPPER_X, &p, false);
    assert_ne!(battle.native_buttons(MAPPER_X) & MAPPER_X, 0);
}
#[test]
fn new_trigger_loops_stop_on_release_and_do_not_replay_passthrough_holds() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for slot in [PresetSlot::L3, PresetSlot::R3] {
            let p=presets(&format!("[bowser.{}.loop]\nstart_delay=0\ndefault=true\nwhile_held=true\nactions=grab:4,wait:1\n",token(slot)));
            let mut m = MenuSelection::new(&p);
            input(&mut m, &p, 0, layout);
            m.observe_physical_passthrough(slot.button(), layout);
            input(&mut m, &p, slot.button(), layout);
            assert!(m.preview_slot.is_none());
            input(&mut m, &p, 0, layout);
            input(&mut m, &p, slot.button(), layout);
            assert_eq!(m.preview_slot, Some(slot));
            input(&mut m, &p, slot.button(), layout);
            assert!(!m.cancel_pending);
            input(&mut m, &p, 0, layout);
            assert!(
                m.cancel_pending && m.preview_slot.is_none(),
                "release {layout:?} {slot:?}"
            );
        }
    }
}
#[test]
fn builtin_in_compact_selector_is_not_started_as_an_empty_test_tape() {
    let p = presets("");
    let mut m = MenuSelection::new(&p);
    m.show_quick();
    m.button_menu = true;
    m.row = MenuRow::L3Sequence;
    m.input(0, &p);
    m.input(MAPPER_X, &p);
    assert!(m.preview_slot.is_none());
}

#[test]
fn every_button_slot_executes_its_saved_tape_on_ground_and_in_air() {
    use kirby_assist_core::gameplay::{GameplayContext, MotionTiming, Opponent};
    use kirby_assist_core::ledge_trap::LedgeTrap;
    use kirby_assist_core::settings::AssistSettings;
    for slot in PresetSlot::BUTTONS {
        for situation in [
            kirby_assist_core::native_states::situation::GROUND,
            kirby_assist_core::native_states::situation::AIR,
        ] {
            let p = presets(&format!(
                "[bowser.{}.saved]\nstart_delay=0\ndefault=true\nactions=b:4\n",
                token(slot)
            ));
            let m = MenuSelection::new(&p);
            let settings = m.settings(
                &p,
                AssistSettings {
                    enabled: true,
                    ..Default::default()
                },
            );
            let mut trap = LedgeTrap::default();
            trap.request_preview(slot);
            for n in 0..5 {
                let own = GameplayContext {
                    held: true,
                    gameplay: true,
                    situation,
                    facing: 1.,
                    motion_kind: 1,
                    motion_frame: n as f32,
                    motion_rate: 1.,
                    ..Default::default()
                };
                let enemy = Opponent {
                    id: 2,
                    position: [10., 0.],
                    motion: Some(MotionTiming {
                        kind: 2,
                        frame: n as f32,
                        rate: 1.,
                        remaining: None,
                    }),
                    ..Default::default()
                };
                assert_eq!(
                    trap.sample(settings, n, own, Some(enemy)).requested,
                    if n < 4 { MAPPER_B } else { 0 },
                    "{slot:?} situation {situation} frame {n}"
                );
            }
        }
    }
}

#[test]
fn busy_owner_does_not_queue_custom_buttons_or_resurrect_held_helpers() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for slot in PresetSlot::BUTTONS {
            let p = presets(&format!(
                "[bowser.{}.test]\nstart_delay=0\ndefault=true\nactions=up_b:4\n",
                token(slot)
            ));
            let mut m = MenuSelection::new(&p);
            input(&mut m, &p, 0, layout);
            m.block_binding_starts(true);
            input(&mut m, &p, slot.button(), layout);
            assert!(m.preview_slot.is_none(), "{layout:?} {slot:?}");
            assert!(!m.cancel_pending);
            m.block_binding_starts(false);
            for _ in 0..4 {
                input(&mut m, &p, slot.button(), layout);
                assert!(m.preview_slot.is_none());
            }
            input(&mut m, &p, 0, layout);
            input(&mut m, &p, slot.button(), layout);
            assert_eq!(
                m.preview_slot,
                Some(slot),
                "fresh release rearms {layout:?} {slot:?}"
            );
        }
        for builtin in [Builtin::ShortHop, Builtin::TurboTeabag] {
            let p = presets(&format!(
                "[bowser.y.helper]\nbuiltin={}\ndefault=true\n",
                builtin.token()
            ));
            let mut m = MenuSelection::new(&p);
            input(&mut m, &p, 0, layout);
            m.block_binding_starts(true);
            input(&mut m, &p, MAPPER_Y, layout);
            m.block_binding_starts(false);
            input(&mut m, &p, MAPPER_Y, layout);
            let dest = if builtin == Builtin::ShortHop {
                MAPPER_X
            } else {
                TURBO_TRIGGER
            };
            assert_eq!(m.native_buttons(layout.policy_buttons(MAPPER_Y)) & dest, 0);
            input(&mut m, &p, 0, layout);
            input(&mut m, &p, MAPPER_Y, layout);
            assert_ne!(m.native_buttons(layout.policy_buttons(MAPPER_Y)) & dest, 0);
        }
    }
}
#[test]
fn busy_bindings_keep_held_ledge_helper_and_menu_navigation_available() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        let p = presets("[bowser.y.ledge]\nbuiltin=ledge_assist\ndefault=true\n");
        let mut m = MenuSelection::new(&p);
        input(&mut m, &p, 0, layout);
        m.block_binding_starts(true);
        input(&mut m, &p, MAPPER_Y, layout);
        assert_ne!(
            m.native_buttons(layout.policy_buttons(MAPPER_Y)) & MAPPER_L,
            0
        );
        input(&mut m, &p, 0, layout);
        input(&mut m, &p, MAPPER_MINUS, layout);
        assert!(!m.menu_hidden);
    }
}

#[test]
fn held_ledge_binding_survives_its_own_committed_sequence_on_every_source() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for slot in PresetSlot::BUTTONS {
            let p = presets(&format!(
                "[bowser.{}.ledge]\nbuiltin=ledge_assist\ndefault=true\n",
                token(slot)
            ));
            let mut m = MenuSelection::new(&p);
            input(&mut m, &p, 0, layout);
            input(&mut m, &p, slot.button(), layout);
            input(&mut m, &p, slot.button(), layout);
            assert!(
                m.ledge_held(layout.policy_buttons(slot.button())),
                "initial {layout:?} {slot:?}"
            );
            for _ in 0..5 {
                m.block_binding_starts(true);
                input(&mut m, &p, slot.button(), layout);
                assert!(
                    m.ledge_held(layout.policy_buttons(slot.button())),
                    "own sequence must retain L: {layout:?} {slot:?}"
                );
                assert!(!m.cancel_pending);
            }
            m.block_binding_starts(false);
            input(&mut m, &p, 0, layout);
            assert!(!m.ledge_held(0));
        }
    }
}
#[test]
fn logical_helper_routing_does_not_apply_a_swapped_mapping_twice() {
    let p=presets("[bowser.x.ledge]\nbuiltin=ledge_assist\ndefault=true\n[bowser.r3.hop]\nbuiltin=short_hop\ndefault=true\n");
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        let mut m = MenuSelection::new(&p);
        input(&mut m, &p, 0, layout);
        for (physical, expected) in [(MAPPER_X, MAPPER_L), (1 << 17, MAPPER_X)] {
            input(&mut m, &p, 0, layout);
            input(&mut m, &p, physical, layout);
            let logical = m.native_buttons(layout.policy_buttons(physical));
            assert_eq!(logical & (MAPPER_X | MAPPER_L), expected);
            assert_eq!(
                m.native_buttons(logical) & (MAPPER_X | MAPPER_L),
                expected,
                "second routing {layout:?}"
            );
        }
    }
}
#[test]
fn remapped_held_source_requires_actual_release_after_quarantine() {
    let p = presets("[bowser.x.turbo]\nbuiltin=turbo_teabag\ndefault=true\n");
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        let mut m = MenuSelection::new(&p);
        input(&mut m, &p, 0, layout);
        m.reset_inputs();
        for _ in 0..5 {
            input(&mut m, &p, MAPPER_X, layout);
            assert_eq!(
                m.native_buttons(layout.policy_buttons(MAPPER_X)) & TURBO_TRIGGER,
                0,
                "held source must stay quarantined: {layout:?}"
            );
        }
        input(&mut m, &p, 0, layout);
        input(&mut m, &p, MAPPER_X, layout);
        assert_ne!(
            m.native_buttons(layout.policy_buttons(MAPPER_X)) & TURBO_TRIGGER,
            0
        );
    }
}
#[test]
fn short_hop_works_on_every_source_without_becoming_an_extra_air_jump() {
    use kirby_assist_core::manual_controls::{ShortcutContext, Shortcuts};
    let context = |frame, status, situation| ShortcutContext {
        status,
        situation,
        motion: status as u64 + 1,
        frame,
        rate: 1.,
        stopped: false,
        facing: 1.,
        can_short_hop: situation == kirby_assist_core::native_states::situation::GROUND,
        input_delay: 0.,
        can_neutral_special: false,
    };
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for slot in PresetSlot::BUTTONS {
            let p = presets(&format!(
                "[bowser.{}.hop]\nbuiltin=short_hop\ndefault=true\n",
                token(slot)
            ));
            let mut m = MenuSelection::new(&p);
            let mut hops = Shortcuts::default();
            input(&mut m, &p, 0, layout);
            hops.sample(0, true, Some(context(0., 0, 0)));
            input(&mut m, &p, slot.button(), layout);
            let mut logical = m.native_buttons(layout.policy_buttons(slot.button()));
            assert_eq!(
                hops.sample(logical, true, Some(context(1., 0, 0)))
                    .requested,
                MAPPER_X,
                "initial hop {layout:?} {slot:?}"
            );
            m.block_binding_starts(true);
            m.continue_short_hop_binding(true);
            input(&mut m, &p, slot.button(), layout);
            logical = m.native_buttons(layout.policy_buttons(slot.button()));
            assert_ne!(logical & MAPPER_X, 0);
            let release = hops.sample(logical, true, Some(context(0., 0xa, 0)));
            assert_eq!(release.requested, 0);
            assert_ne!(release.owned & MAPPER_X, 0);
            m.block_binding_starts(true);
            m.continue_short_hop_binding(true);
            input(&mut m, &p, slot.button(), layout);
            logical = m.native_buttons(layout.policy_buttons(slot.button()));
            let air = hops.sample(logical, true, Some(context(0., 0xb, 2)));
            assert!(
                !jump_passthrough(logical, 0, air.owned, hops.consumed_buttons()),
                "held tail {layout:?} {slot:?}"
            );
            m.block_binding_starts(false);
            input(&mut m, &p, 0, layout);
            hops.sample(0, true, Some(context(1., 0xb, 2)));
            input(&mut m, &p, slot.button(), layout);
            logical = m.native_buttons(layout.policy_buttons(slot.button()));
            let fresh = hops.sample(logical, true, Some(context(2., 0xb, 2)));
            assert!(
                jump_passthrough(logical, 0, fresh.owned, hops.consumed_buttons()),
                "fresh aerial jump {layout:?} {slot:?}"
            );
        }
    }
}
#[test]
fn visible_lightweight_dpad_remaps_work_without_the_menu_modifier() {
    for slot in [
        PresetSlot::DpadUp,
        PresetSlot::DpadDown,
        PresetSlot::DpadLeft,
        PresetSlot::DpadRight,
    ] {
        let p = presets(&format!(
            "[bowser.{}.ledge]\nbuiltin=ledge_assist\ndefault=true\n",
            token(slot)
        ));
        let mut m = MenuSelection::new(&p);
        input(&mut m, &p, 0, ControlLayout::Default);
        m.show_quick();
        m.set_binding_input(slot.button(), ControlLayout::Default);
        m.input_with_editing(slot.button(), &p, false);
        assert_ne!(m.native_buttons(slot.button()) & MAPPER_L, 0);
    }
}
#[test]
fn assigned_l_remains_active_while_browsing_the_compact_selector() {
    let p = presets("[bowser.l.ledge]\nbuiltin=ledge_assist\ndefault=true\n");
    let layout = ControlLayout::Default;
    let mut m = MenuSelection::new(&p);
    let poll = |m: &mut MenuSelection, raw| {
        m.set_binding_input(raw, layout);
        m.input_with_editing(raw, &p, false);
    };
    poll(&mut m, 0);
    m.show_quick();
    poll(&mut m, MAPPER_L);
    poll(&mut m, MAPPER_L);
    assert!(m.ledge_held(MAPPER_L));
    for raw in [MAPPER_L, MAPPER_L | 4, MAPPER_L, MAPPER_L | 4, MAPPER_L] {
        m.block_binding_starts(true);
        poll(&mut m, raw);
        assert!(
            m.ledge_held(raw),
            "L remains available with menu navigation {raw:#x}"
        );
    }
}
#[test]
fn builtin_jump_is_delivered_after_source_consumption_without_a_fabricated_physical_jump() {
    use kirby_assist_core::manual_controls::{
        ManualOverlay, ShortcutContext, Shortcuts, TurboOutput,
    };
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for slot in PresetSlot::BUTTONS {
            let p = presets(&format!(
                "[bowser.{}.hop]\nbuiltin=short_hop\ndefault=true\n",
                token(slot)
            ));
            let mut m = MenuSelection::new(&p);
            input(&mut m, &p, 0, layout);
            let consumed = input(&mut m, &p, slot.button(), layout);
            let mut hop = Shortcuts::default();
            let proposal = hop.sample(
                m.native_buttons(layout.policy_buttons(slot.button())),
                true,
                Some(ShortcutContext {
                    status: kirby_assist_core::native_states::status::WAIT,
                    situation: kirby_assist_core::native_states::situation::GROUND,
                    motion: 1,
                    frame: 1.,
                    rate: 1.,
                    stopped: false,
                    facing: 1.,
                    can_short_hop: true,
                    input_delay: 0.,
                    can_neutral_special: false,
                }),
            );
            assert_ne!(proposal.requested & MAPPER_X, 0);
            let raw = ManualOverlay::default().apply_controls(
                ButtonTrace {
                    held: slot.button(),
                    pressed: slot.button(),
                    ..Default::default()
                },
                [0.; 2],
                TurboOutput::default(),
                layout.physical_buttons(consumed),
                layout.physical_buttons(proposal.requested & !MAPPER_X),
                layout.physical_owned_buttons(proposal.owned | MAPPER_X),
            );
            assert_eq!(
                raw.buttons.held, 0,
                "no source action or invented physical jump: {layout:?} {slot:?}"
            );
            assert_eq!(mapped_builtin_jump(0, Some(true), true), 0x44);
            assert_eq!(
                mapped_builtin_jump(0x45, Some(false), true),
                0x1,
                "release Jump/Mini without clearing Attack"
            );
        }
    }
}
#[test]
fn simulated_sequences_use_default_button_meanings_without_running_custom_bindings() {
    use kirby_assist_core::{
        ledge_trap::SEQUENCE_BUTTONS,
        manual_controls::{ManualOverlay, TurboOutput},
    };
    let p=presets("[bowser.l.jump]\nbuiltin=short_hop\ndefault=true\n[bowser.x.ledge]\nbuiltin=ledge_assist\ndefault=true\n[bowser.dpad_down.turbo]\nbuiltin=turbo_teabag\ndefault=true\n");
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        let mut m = MenuSelection::new(&p);
        let consumed = input(&mut m, &p, 0, layout);
        for requested in [MAPPER_X, MAPPER_L, MAPPER_R, MAPPER_ZR, MAPPER_Y, 4] {
            let before = m.native_buttons(0);
            let frame = ManualOverlay::default().apply_controls(
                ButtonTrace::default(),
                [0.; 2],
                TurboOutput::default(),
                layout.physical_buttons(consumed),
                layout.physical_buttons(requested),
                layout.physical_owned_buttons(SEQUENCE_BUTTONS),
            );
            assert_eq!(frame.buttons.held, layout.physical_buttons(requested));
            assert!(m.preview_slot.is_none());
            assert_eq!(m.native_buttons(0), before);
            assert!(!jump_passthrough(MAPPER_X, SEQUENCE_BUTTONS, 0, 0));
            assert_eq!(
                mapped_builtin_jump(0x123, None, true),
                0x123,
                "ordinary mapped sequence is untouched"
            );
        }
    }
}
#[test]
fn mapped_builtin_jump_never_overrides_a_sequence_or_safety_owner() {
    use kirby_assist_core::action_owner::{permits_builtin_jump, Owner};
    for owner in [
        Owner::Safety,
        Owner::GrabDodge,
        Owner::OptionRead,
        Owner::Trip,
        Owner::JabLock,
        Owner::Fair,
        Owner::Trump,
        Owner::Shortcut,
        Owner::RollRead,
        Owner::Defense,
        Owner::OutOfShield,
    ] {
        assert!(
            !permits_builtin_jump(Some(owner), false),
            "must not inject mapped jump into {owner:?}"
        );
    }
    assert!(permits_builtin_jump(None, false));
    assert!(permits_builtin_jump(Some(Owner::Posture), false));
    assert!(permits_builtin_jump(Some(Owner::Ledge), false));
    assert!(!permits_builtin_jump(Some(Owner::Ledge), true));
}

#[test]
fn explicit_sequence_dpad_is_not_suppressed_as_a_custom_physical_binding() {
    use kirby_assist_core::{
        ledge_trap::SEQUENCE_BUTTONS,
        manual_controls::{sequence_taunt_reservation, suppress_reserved_taunts},
    };
    for (dpad, mapped) in [(1, 0x400), (4, 0x1000), (2, 0x6000), (8, 0x6000)] {
        assert_eq!(suppress_reserved_taunts(mapped, 0xf), 0);
        let reserved = sequence_taunt_reservation(0xf, dpad, SEQUENCE_BUTTONS);
        assert_eq!(suppress_reserved_taunts(mapped, reserved), mapped);
        assert_eq!(
            sequence_taunt_reservation(0xf, dpad, 0),
            0xf,
            "unowned physical input stays reserved"
        );
    }
}

#[test]
fn compact_menu_preserves_fixed_l_and_the_retained_x_short_hop_default() {
    let p = presets("");
    let mut m = MenuSelection::new(&p);
    let layout = ControlLayout::Default;
    let poll = |m: &mut MenuSelection, raw| {
        m.set_binding_input(raw, layout);
        m.input_with_editing(raw, &p, false);
    };
    poll(&mut m, 0);
    m.show_quick();
    poll(&mut m, 0);
    poll(&mut m, MAPPER_L);
    assert!(
        m.cancel_pending,
        "fixed L performs legacy autopilot takeover"
    );
    poll(&mut m, MAPPER_L);
    assert!(m.ledge_held(MAPPER_L));
    poll(&mut m, 0);
    poll(&mut m, MAPPER_X);
    assert_ne!(
        m.native_buttons(MAPPER_X) & MAPPER_X,
        0,
        "unconfigured X defaults to short hop"
    );
}

#[test]
fn turbo_binding_alternates_each_local_input_update_without_delayed_pose_confirmation() {
    use kirby_assist_core::{
        manual_controls::{TurboSettings, TurboState},
        timing::{AnimationClock, AnimationSample},
    };
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for slot in PresetSlot::BUTTONS {
            let p = presets(&format!(
                "[bowser.{}.tbag]\nbuiltin=turbo_teabag\ndefault=true\n",
                token(slot)
            ));
            let mut m = MenuSelection::new(&p);
            input(&mut m, &p, 0, layout);
            let mut turbo = TurboState::default();
            let settings = TurboSettings {
                enabled: true,
                phase_frames: 99,
                ..Default::default()
            };
            let mut animation = AnimationClock::default();
            let frozen = AnimationSample {
                object: 1,
                status: kirby_assist_core::native_states::status::SQUAT_WAIT,
                kind: 1,
                frame: 10.,
                auxiliary: 0.,
            };
            let pose_tick = animation.observe(frozen, false);
            for input_tick in 0..20 {
                input(&mut m, &p, slot.button(), layout);
                let held =
                    m.native_buttons(layout.policy_buttons(slot.button())) & TURBO_TRIGGER != 0;
                assert_eq!(
                    animation.observe(frozen, false),
                    pose_tick,
                    "a delayed/frozen pose is not an input clock"
                );
                let out = turbo.sample_teabag(settings, input_tick, held, 1., true, true);
                assert_eq!(
                    out.stick.unwrap()[1],
                    if input_tick % 2 == 0 { -1. } else { 0. },
                    "{layout:?} {slot:?} input {input_tick}"
                );
                assert_eq!(
                    turbo.sample_teabag(settings, input_tick, held, 1., true, true),
                    out,
                    "duplicate update cannot consume another phase"
                );
            }
            assert!(turbo
                .sample_teabag(settings, 20, false, 1., true, true)
                .stick
                .is_none());
            assert!(turbo
                .sample_teabag(settings, 21, true, 1., true, false)
                .stick
                .is_none());
        }
    }
}

#[test]
fn builtin_hop_survives_the_runtime_menu_consumption_filter_on_every_source() {
    use kirby_assist_core::{
        manual_controls::{ShortcutContext, Shortcuts},
        settings_menu::observation_buttons,
    };
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for slot in PresetSlot::BUTTONS {
            let p = presets(&format!(
                "[bowser.{}.native]\nbuiltin=short_hop\ndefault=true\n",
                token(slot)
            ));
            let mut m = MenuSelection::new(&p);
            input(&mut m, &p, 0, layout);
            let consumed = input(&mut m, &p, slot.button(), layout);
            let policy = m.native_buttons(layout.policy_buttons(slot.button()));
            let ui_consumed = consumed & !m.builtin_consumed;
            let observed = observation_buttons(policy, ui_consumed);
            let buttons = policy & !MAPPER_Y & !(ui_consumed & (0xf | MAPPER_X)) & !0xa;
            assert_ne!(
                observed & MAPPER_X,
                0,
                "observation: {layout:?} {slot:?} consumed={consumed:#x}"
            );
            let mut tool = Shortcuts::default();
            let out = tool.sample(
                buttons,
                policy & (5 | MAPPER_MINUS) == 0,
                Some(ShortcutContext {
                    status: kirby_assist_core::native_states::status::WAIT,
                    situation: kirby_assist_core::native_states::situation::GROUND,
                    motion: 1,
                    frame: 10.,
                    rate: 1.,
                    stopped: false,
                    facing: 1.,
                    can_short_hop: true,
                    input_delay: 0.,
                    can_neutral_special: false,
                }),
            );
            assert_ne!(
                out.requested & MAPPER_X,
                0,
                "delivery: {layout:?} {slot:?} policy={policy:#x} consumed={consumed:#x}"
            );
        }
    }
}

#[test]
fn every_held_ledge_binding_and_toggle_suspend_autopunish_before_acquisition() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for slot in PresetSlot::BUTTONS {
            let p = presets(&format!(
                "[bowser.{}.ledge]\nbuiltin=ledge_assist\ndefault=true\n",
                token(slot)
            ));
            let mut m = MenuSelection::new(&p);
            input(&mut m, &p, 0, layout);
            assert!(!m.reserves_automatic_modes());
            input(&mut m, &p, slot.button(), layout);
            assert!(
                m.reserves_automatic_modes(),
                "first press: {layout:?} {slot:?}"
            );
            input(&mut m, &p, 0, layout);
            assert!(!m.reserves_automatic_modes());
            m.auto_ledge = true;
            assert!(
                m.reserves_automatic_modes(),
                "ON stays reserved during manual pauses"
            );
            use kirby_assist_core::action_owner::{ActionOwnership, Intent, Owner};
            for read in [
                Owner::RollRead,
                Owner::OutOfShield,
                Owner::GrabDodge,
                Owner::OptionRead,
            ] {
                let mut owner = ActionOwnership::new();
                owner.set_automatic_suspended(m.reserves_automatic_modes());
                assert!(!owner.allows(read));
                assert_eq!(
                    owner.select(
                        true,
                        &[Intent {
                            owner: read,
                            present: true,
                            committed: true
                        }]
                    ),
                    None
                );
                owner.set_ledge_priority(true);
                assert_eq!(
                    owner.select(
                        true,
                        &[Intent {
                            owner: Owner::Ledge,
                            present: true,
                            committed: true
                        }]
                    ),
                    Some(Owner::Ledge)
                );
                owner.set_ledge_priority(false);
                assert!(
                    !owner.allows(read),
                    "completing a response does not disable the ON reservation"
                );
                owner.release(Owner::Ledge);
                owner.set_automatic_suspended(false);
                assert!(owner.allows(read));
            }
            m.auto_ledge = false;
            m.preview_slot = Some(slot);
            assert!(
                m.reserves_automatic_modes(),
                "explicit preview still reserves controls"
            );
        }
    }
}

#[test]
fn safety_preserves_builtin_and_literal_one_frame_jump_on_every_physical_binding() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for slot in PresetSlot::BUTTONS {
            for recipe in [
                "builtin=short_hop",
                "start_delay=0\nactions=facing_input:x:0:0:1",
            ] {
                let p = presets(&format!(
                    "[bowser.{}.jump]\ndefault=true\n{recipe}\n",
                    token(slot)
                ));
                let mut m = MenuSelection::new(&p);
                input(&mut m, &p, 0, layout);
                // The previous action had quarantined this held source. Damage safety
                // returns it as ordinary jump input, without running a custom tape.
                m.block_binding_starts(true);
                input(&mut m, &p, slot.button(), layout);
                m.block_binding_starts(true);
                m.preserve_safety_jump(true);
                let consumed = input(&mut m, &p, slot.button(), layout);
                let routed = m.native_buttons(layout.policy_buttons(slot.button()));
                assert_ne!(routed & MAPPER_X, 0, "{layout:?} {slot:?} {recipe}");
                assert_ne!(
                    kirby_assist_core::settings_menu::observation_buttons(
                        routed,
                        consumed & !m.builtin_consumed
                    ) & MAPPER_X,
                    0
                );
                assert!(m.preview_slot.is_none(), "safety must not start a new tape");
                // Deliberate normal use keeps the saved recipe/helper behavior.
                m.block_binding_starts(false);
                input(&mut m, &p, 0, layout);
                input(&mut m, &p, slot.button(), layout);
                if recipe.starts_with("start_delay") {
                    assert_eq!(m.preview_slot, Some(slot));
                }
            }
        }
    }
}

#[test]
fn safety_does_not_run_attacks_or_conditioned_or_delayed_jump_tapes() {
    for recipe in [
        "start_delay=0\nactions=grab:4",
        "start_delay=2\nactions=short_hop:1",
        "start_delay=0\nactions=check:enemy_within:30:0,short_hop:1",
        "start_delay=0\nactions=short_hop:1,up_b:4",
    ] {
        let p = presets(&format!("[bowser.r3.saved]\ndefault=true\n{recipe}\n"));
        let mut m = MenuSelection::new(&p);
        input(&mut m, &p, 0, ControlLayout::Default);
        m.block_binding_starts(true);
        m.preserve_safety_jump(true);
        input(&mut m, &p, 1 << 17, ControlLayout::Default);
        assert!(m.preview_slot.is_none());
        assert_eq!(m.native_buttons(1 << 17) & MAPPER_X, 0, "{recipe}");
    }
}

#[test]
fn retired_trigger_bindings_cannot_consume_jump_guard_or_grab_in_either_layout() {
    use kirby_assist_core::manual_controls::{ManualOverlay, TurboOutput};
    let source="[bowser.l.saved]\nbuiltin=short_hop\ndefault=true\n[bowser.r.saved]\nactions=not_an_action\n[bowser.zl.saved]\nbuiltin=turbo_teabag\n[bowser.zr.saved]\nbuiltin=ledge_assist\n[bowser.y.grab]\nstart_delay=0\nactions=grab:4\ndefault=true\n";
    let cat = Catalog::parse(source).unwrap();
    assert_eq!(cat.entries().len(), 1);
    let p = presets(source);
    assert_eq!(PresetSlot::BUTTONS.len(), 8);
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for source in [MAPPER_R, MAPPER_ZL, MAPPER_ZR] {
            let mut m = MenuSelection::new(&p);
            input(&mut m, &p, 0, layout);
            for blocked in [false, true] {
                m.block_binding_starts(blocked);
                let consumed = input(&mut m, &p, source, layout);
                assert!(m.preview_slot.is_none());
                let physical = ButtonTrace {
                    held: source,
                    pressed: source,
                    ..Default::default()
                };
                let out = ManualOverlay::default().apply_controls(
                    physical,
                    [0.; 2],
                    TurboOutput::default(),
                    layout.physical_buttons(consumed),
                    0,
                    0,
                );
                assert_eq!(
                    out.buttons.held & source,
                    source,
                    "native trigger must pass {layout:?} {source:#x}"
                );
            }
        }
    }
    let mut m = MenuSelection::new(&p);
    input(&mut m, &p, 0, ControlLayout::Default);
    input(&mut m, &p, MAPPER_L, ControlLayout::Default);
    input(&mut m, &p, MAPPER_L, ControlLayout::Default);
    assert!(
        m.ledge_held(MAPPER_L),
        "fixed L retains legacy ledge/walk assistance"
    );
    assert_eq!(
        m.native_buttons(MAPPER_L) & MAPPER_X,
        0,
        "old saved L short hop is inert"
    );
}

#[test]
fn native_triggers_are_identical_with_selector_hidden_open_and_closed_again() {
    use kirby_assist_core::manual_controls::{ManualOverlay, TurboOutput};
    let p=presets("[bowser.zl.jump]\nbuiltin=normal\ndefault=true\n[bowser.r.bad]\nbuiltin=short_hop\ndefault=true\n[bowser.zr.bad]\nbuiltin=turbo_teabag\ndefault=true\n");
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for source in [MAPPER_R, MAPPER_ZL, MAPPER_ZR] {
            let mut m = MenuSelection::new(&p);
            let poll = |m: &mut MenuSelection, raw| {
                m.set_binding_input(raw, layout);
                m.input_with_editing(layout.policy_buttons(raw), &p, false)
            };
            poll(&mut m, 0);
            for state in 0..3 {
                if state > 0 {
                    poll(&mut m, 0);
                    poll(&mut m, MAPPER_MINUS);
                    poll(&mut m, 0);
                }
                for blocked in [false, true] {
                    m.block_binding_starts(blocked);
                    let consumed = poll(&mut m, source);
                    let out = ManualOverlay::default().apply_controls(
                        ButtonTrace {
                            held: source,
                            previous: source,
                            ..Default::default()
                        },
                        [0.; 2],
                        TurboOutput::default(),
                        layout.physical_buttons(consumed),
                        0,
                        0,
                    );
                    assert_eq!(
                        out.buttons.held & source,
                        source,
                        "{layout:?} trigger={source:#x} selector-state={state} blocked={blocked}"
                    );
                    assert!(m.preview_slot.is_none());
                }
            }
        }
    }
}

fn light_input(m: &mut MenuSelection, p: &MenuPresets, raw: u32, layout: ControlLayout) -> u32 {
    m.set_binding_input(raw, layout);
    m.input_with_editing(layout.policy_buttons(raw), p, false)
}
#[test]
fn visible_lightweight_custom_dpad_works_without_modifier_in_both_layouts() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for slot in [
            PresetSlot::DpadUp,
            PresetSlot::DpadDown,
            PresetSlot::DpadLeft,
            PresetSlot::DpadRight,
        ] {
            let p = presets(&format!(
                "[bowser.{}.saved]\ndefault=true\nstart_delay=0\nactions=grab:4\n",
                token(slot)
            ));
            let mut m = MenuSelection::new(&p);
            light_input(&mut m, &p, 0, layout);
            light_input(&mut m, &p, MAPPER_MINUS, layout);
            light_input(&mut m, &p, 0, layout);
            let row = m.row;
            let choices = m.choices;
            let consumed = light_input(&mut m, &p, slot.button(), layout);
            assert_eq!(m.preview_slot, Some(slot));
            assert_eq!(m.row, row);
            assert_eq!(m.choices, choices);
            assert_ne!(layout.physical_buttons(consumed) & slot.button(), 0);
            let revision = m.revision;
            for _ in 0..120 {
                light_input(&mut m, &p, slot.button(), layout);
                assert_eq!(m.revision, revision);
            }
            assert!(!m.recipe_editing && !m.managing());
        }
    }
}
#[test]
fn lightweight_navigation_dpad_requires_release_before_returning_to_its_custom_action() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        let p=presets("[bowser.dpad_down.saved]\ndefault=true\nstart_delay=0\nactions=grab:4\n[bowser.l3.ledge]\nbuiltin=ledge_assist\ndefault=true\n");
        let mut m = MenuSelection::new(&p);
        let modifier = layout.physical_buttons(MAPPER_L);
        let down = kirby_assist_core::ledge_trap::DPAD_DOWN;
        light_input(&mut m, &p, 0, layout);
        assert!(!m.menu_hidden);
        light_input(&mut m, &p, modifier, layout);
        let consumed = light_input(&mut m, &p, modifier | down, layout);
        assert_ne!(consumed & down, 0);
        assert_eq!(m.row, MenuRow::Jump);
        assert!(m.preview_slot.is_none());
        for _ in 0..10 {
            let consumed = light_input(&mut m, &p, down, layout);
            assert_ne!(consumed & down, 0);
            assert!(m.preview_slot.is_none());
        }
        light_input(&mut m, &p, 0, layout);
        light_input(&mut m, &p, down, layout);
        assert_eq!(m.preview_slot, Some(PresetSlot::DpadDown));
        // Re-entering L navigation cancels the custom tape before ledge input resumes.
        light_input(&mut m, &p, modifier, layout);
        assert!(m.preview_slot.is_none());
        light_input(&mut m, &p, modifier, layout);
        assert!(m.ledge_held(layout.policy_buttons(modifier)));
    }
}
#[test]
fn unassigned_dpad_passes_to_game_without_changing_the_open_lightweight_menu() {
    let p = presets("");
    let mut m = MenuSelection::new(&p);
    light_input(&mut m, &p, 0, ControlLayout::Default);
    assert!(!m.menu_hidden);
    light_input(&mut m, &p, 0, ControlLayout::Default);
    let choices = m.choices;
    let row = m.row;
    for button in [1, 2, 4, 8] {
        light_input(&mut m, &p, 0, ControlLayout::Default);
        assert_eq!(
            light_input(&mut m, &p, button, ControlLayout::Default) & button,
            0
        );
        assert_eq!(m.row, row);
        assert_eq!(m.choices, choices);
    }
}
