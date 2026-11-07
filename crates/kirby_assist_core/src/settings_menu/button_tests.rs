use super::*;
use crate::{fighter_profiles::FighterProfiles, native_input::*, sequence_config::Catalog};
fn presets(source: &str) -> MenuPresets {
    let mut profiles = FighterProfiles::default();
    profiles
        .add_sequence_choices(&Catalog::parse(source).unwrap())
        .unwrap();
    profiles.profile(6).unwrap().menu_options
}

fn configured() -> MenuPresets {
    presets("[kirby.neutral.n]\nstart_delay=0\nactions=grab\n")
}
#[test]
fn x_is_available_for_short_hop_and_y_passes_through_without_any_button_recipe() {
    let p = configured();
    let mut m = MenuSelection::new(&p);
    m.input(0, &p);
    assert_ne!(m.input(MAPPER_X, &p) & MAPPER_X, 0);
    assert!(m.preview_slot.is_none());
    m.input(0, &p);
    assert_eq!(m.input(MAPPER_Y, &p) & MAPPER_Y, 0);
    assert!(m.preview_slot.is_none());
    assert_eq!(MenuSelection::QUICK_ROWS.len(), PresetSlot::COUNT);
}
#[test]
fn legacy_combo_binding_is_hidden_and_y_passes_through_without_a_recipe() {
    let p = presets("");
    let mut m = MenuSelection::new(&p);
    m.auto_duck_available = true;
    m.input(0, &p);
    assert_eq!(m.input(MAPPER_Y, &p) & MAPPER_Y, 0);
    assert!(!m.finisher_held(MAPPER_Y));
    m.modes_menu = true;
    assert!(!m.row_visible(MenuRow::YMode));
    assert!(!m.row_visible(MenuRow::JabGrab));
}

#[test]
fn compact_selector_preserves_unassigned_manual_buttons() {
    let p = configured();
    let mut m = MenuSelection::new(&p);
    m.show_quick();
    m.input(0, &p);
    let mut overlay = crate::manual_controls::ManualOverlay::default();
    for button in [MAPPER_A, MAPPER_B, MAPPER_Y, MAPPER_R, MAPPER_ZL, MAPPER_ZR] {
        m.input(0, &p);
        let consumed = m.input(button, &p);
        let frame = overlay.apply_controls(
            crate::native_input::ButtonTrace {
                held: button,
                pressed: button,
                ..Default::default()
            },
            [0.0; 2],
            crate::manual_controls::TurboOutput::default(),
            consumed,
            0,
            0,
        );
        assert_eq!(frame.buttons.held & button, button);
        assert_eq!(frame.buttons.pressed & button, button);
        assert!(m.preview_slot.is_none() && !m.managing() && !m.save_requested);
    }
}

#[test]
fn compact_l3_tests_highlighted_recipe_once_and_manual_attack_cancels_without_swallowing_a() {
    let p = configured();
    let mut m = MenuSelection::new(&p);
    m.show_quick();
    m.input(0, &p);
    assert_ne!(m.input(TWO_FRAME_TRIGGER, &p) & TWO_FRAME_TRIGGER, 0);
    assert_eq!(m.preview_slot, Some(PresetSlot::Neutral));
    let revision = m.revision;
    for _ in 0..20 {
        m.input(TWO_FRAME_TRIGGER, &p);
    }
    assert_eq!(m.revision, revision);
    m.input(0, &p);
    m.input(TWO_FRAME_TRIGGER, &p);
    assert!(m.preview_slot.is_none()); // Second press stops the test.
    m.input(0, &p);
    m.input(TWO_FRAME_TRIGGER, &p);
    let consumed = m.input(MAPPER_A, &p);
    assert!(m.preview_slot.is_none() && m.cancel_pending);
    assert_eq!(consumed & MAPPER_A, 0);
    m.input(0, &p);
}

#[test]
fn compact_assigned_l_autopilot_resumes_after_navigation_and_cancels_l3_preview() {
    let p = configured();
    let mut m = MenuSelection::new(&p);
    m.show_quick();
    m.input(0, &p);
    m.input(MAPPER_L, &p);
    let consumed = m.input(MAPPER_L, &p);
    assert!(m.ledge_held(MAPPER_L));
    assert_ne!(consumed & MAPPER_L, 0); // L stays an assist trigger, not native Shield.
    m.input(MAPPER_L | DPAD_DOWN, &p);
    assert_eq!(m.row, MenuRow::Jump);
    assert!(!m.ledge_held(MAPPER_L | DPAD_DOWN));
    m.input(MAPPER_L | DPAD_DOWN, &p);
    assert!(!m.ledge_held(MAPPER_L | DPAD_DOWN));
    m.input(MAPPER_L, &p);
    assert!(m.ledge_held(MAPPER_L)); // No L release required after navigation.
    m.input(MAPPER_L | TWO_FRAME_TRIGGER, &p);
    assert!(m.preview_slot.is_none()); // L3 cannot start a competing test with L held.
    m.input(MAPPER_L, &p);
    assert!(m.ledge_held(MAPPER_L));
    m.input(0, &p);
    m.row = MenuRow::Neutral;
    m.input(TWO_FRAME_TRIGGER, &p);
    assert_eq!(m.preview_slot, Some(PresetSlot::Neutral));
    m.input(MAPPER_L, &p);
    assert!(m.preview_slot.is_none());
    m.input(MAPPER_L, &p);
    assert!(m.ledge_held(MAPPER_L) && m.quick_menu && !m.menu_hidden);
}

#[test]
fn fighter_button_recipe_is_one_shot_and_suppresses_held_jump_until_release() {
    let p = presets("[kirby.x.route]\nname=My X\ndefault=true\nstart_delay=2\nactions=hold:a:4\n[kirby.y.route]\ndefault=true\nstart_delay=0\nactions=grab\n");
    let mut m = MenuSelection::new(&p);
    m.input(0, &p);
    assert_ne!(m.input(MAPPER_X, &p) & MAPPER_X, 0);
    assert_eq!(m.preview_slot, Some(PresetSlot::X));
    m.preview_slot = None; // Native timeline completion.
    for _ in 0..10 {
        assert_ne!(m.input(MAPPER_X, &p) & MAPPER_X, 0);
        assert!(
            m.preview_slot.is_none(),
            "holding never repeats a completed recipe"
        );
    }
    m.input(0, &p);
    m.input(MAPPER_Y, &p);
    assert_eq!(m.preview_slot, Some(PresetSlot::Y));
    assert_ne!(m.input(MAPPER_Y, &p) & MAPPER_Y, 0);
    m.reset_inputs();
    m.input(MAPPER_Y, &p);
    assert!(
        m.preview_slot.is_none(),
        "a reset must not synthesize a held press"
    );
}

#[test]
fn normal_button_is_permanent_and_has_a_saveable_builtin_page() {
    let p = configured();
    let mut m = MenuSelection::new(&p);
    m.show_quick();
    m.row = MenuRow::XSequence;
    m.choices[5] = p.list(PresetSlot::X).options.len();
    m.input(0, &p);
    assert_eq!(
        m.choice(&p, PresetSlot::X, m.choices[5]).unwrap().id,
        "__normal_button"
    );
    m.input(MENU_STICK_CLICK, &p);
    assert!(m.recipe_editing);
    assert_eq!(
        m.choice(&p, PresetSlot::X, m.choices[5]).unwrap().id,
        "__normal_button"
    );
    assert_eq!(
        m.choice(&p, PresetSlot::X, p.list(PresetSlot::X).options.len())
            .unwrap()
            .id,
        "__normal_button"
    );
}

#[test]
fn fighter_button_choices_and_edits_are_not_shared() {
    let c = Catalog::parse("[kirby.x.a]\ndefault=true\nstart_delay=1\nactions=jab\n[yoshi.x.a]\ndefault=true\nstart_delay=4\nactions=grab\n").unwrap();
    let mut profiles = FighterProfiles::default();
    profiles.add_sequence_choices(&c).unwrap();
    let kirby = profiles.profile(6).unwrap().menu_options;
    let yoshi = profiles.profile(5).unwrap().menu_options;
    let k = MenuSelection::new(&kirby).settings(&kirby, LedgeSettings::default());
    let y = MenuSelection::new(&yoshi).settings(&yoshi, LedgeSettings::default());
    assert_eq!(k.sequence_overrides[5].unwrap().start_delay, 1);
    assert_eq!(y.sequence_overrides[5].unwrap().start_delay, 4);
    assert_ne!(k.sequence_overrides[5], y.sequence_overrides[5]);
}

#[cfg(feature = "diagnostic-logging")]
#[test]
fn retained_editor_payload_stays_stable_during_idle_and_reopening_same_recipe() {
    let p = configured();
    let mut m = MenuSelection::new(&p);
    assert_eq!(m.diagnostic_retained().0, 0);
    m.show_quick();
    m.input(0, &p);
    m.input(MENU_STICK_CLICK, &p);
    let retained = m.diagnostic_retained();
    assert_eq!(retained.0, 1);
    for _ in 0..100 {
        m.input(0, &p);
    }
    assert_eq!(m.diagnostic_retained(), retained);
    m.input(MENU_STICK_CLICK, &p); // Back to compact.
    m.input(0, &p);
    m.input(MENU_STICK_CLICK, &p); // Same editor; no duplicate retained recipe.
    assert_eq!(m.diagnostic_retained(), retained);
}

#[test]
fn while_held_button_releases_cancel_but_once_per_press_recipes_keep_running() {
    for hold in [false, true] {
        let p=presets(&format!("[kirby.y.repeat]\ndefault=true\nwhile_held={hold}\nstart_delay=0\nactions=tap:a:0:-0.6:1:11:120\n"));
        let mut m = MenuSelection::new(&p);
        m.input(0, &p);
        m.input(MAPPER_Y, &p);
        assert_eq!(m.preview_slot, Some(PresetSlot::Y));
        assert_eq!(
            m.held_binding,
            if hold { Some(PresetSlot::Y) } else { None }
        );
        let revision = m.revision;
        for _ in 0..5 {
            m.input(MAPPER_Y, &p);
        }
        assert_eq!(m.revision, revision);
        m.input(0, &p);
        assert_eq!(
            m.preview_slot,
            if hold { None } else { Some(PresetSlot::Y) }
        );
        assert_eq!(m.cancel_pending, hold);
    }
}

#[test]
fn training_only_editor_gate_keeps_compact_choices_and_closes_existing_editor() {
    let p = configured();
    let mut m = MenuSelection::new(&p);
    m.show_quick();
    m.row = MenuRow::XSequence;
    m.input_with_editing(0, &p, false);
    let choices = m.choices;
    m.input_with_editing(MENU_STICK_CLICK, &p, false);
    assert!(m.quick_menu && !m.recipe_editing && !m.menu_hidden);
    assert_eq!(
        m.choices, choices,
        "blocked editor must not create a recipe"
    );
    m.input_with_editing(0, &p, true);
    m.input_with_editing(MENU_STICK_CLICK, &p, true);
    assert!(m.recipe_editing, "Training retains the editor");
    let edited_choices = m.choices;
    m.input_with_editing(0, &p, false);
    assert!(m.quick_menu && !m.recipe_editing && !m.menu_hidden);
    assert_eq!(
        m.choices, edited_choices,
        "leaving Training preserves recipes"
    );
}

#[test]
fn lightweight_has_five_ledge_rows_and_separate_buttons_assists_without_editor_or_preview() {
    let p = configured();
    let mut m = MenuSelection::new(&p);
    m.input_with_editing(0, &p, false);
    assert!(m.lightweight && m.quick_menu && !m.menu_hidden);
    assert_eq!(
        MenuRow::ALL.iter().filter(|r| m.row_visible(**r)).count(),
        5
    );
    assert!(!m.row_visible(MenuRow::ButtonsPage));
    m.input_with_editing(0, &p, false);
    m.input_with_editing(MAPPER_X, &p, false);
    assert!(m.preview_slot.is_none() && m.editor.is_none());
    m.input_with_editing(0, &p, false);
    m.input_with_editing(MENU_STICK_CLICK, &p, false);
    assert!(m.button_menu && !m.recipe_editing && m.editor.is_none());
    assert!(m.row_visible(MenuRow::XSequence) && m.row_visible(MenuRow::AutoTech));
    assert!(!m.row_visible(MenuRow::Neutral));
    m.row = MenuRow::AutoTech;
    let enabled = m.auto_tech;
    m.input_with_editing(0, &p, false);
    m.input_with_editing(MAPPER_L | DPAD_RIGHT, &p, false);
    assert_eq!(m.auto_tech, !enabled);
    m.input_with_editing(0, &p, false);
    m.input_with_editing(MAPPER_Y | MAPPER_A | MAPPER_B, &p, false);
    assert!(!m.save_requested && !m.default_requested && !m.managing());
    m.input_with_editing(0, &p, false);
    m.input_with_editing(MAPPER_MINUS, &p, false);
    m.input_with_editing(0, &p, false);
    m.input_with_editing(MAPPER_MINUS, &p, false);
    assert!(!m.button_menu && m.row == MenuRow::Neutral);
}

#[test]
fn fresh_session_loads_saved_changes_additions_deletions_and_defaults() {
    let initial = "[kirby.neutral.old]\nstart_delay=0\nname=Old\nactions=grab\n[kirby.neutral.remove]\nstart_delay=0\nactions=jab\n[kirby.x.first]\nstart_delay=0\nactions=jab\n";
    let p = presets(initial);
    let latest = Catalog::parse("[kirby.neutral.old]\nstart_delay=0\nname=Renamed\nactions=wait:4,grab\n[kirby.neutral.new]\nstart_delay=0\nname=New\ndefault=true\nactions=nair\n[kirby.x.first]\nstart_delay=0\nactions=jab\n[kirby.x.new]\nstart_delay=0\ndefault=true\nactions=down_tilt\n").unwrap();
    let mut m = MenuSelection::new(&p);
    m.load_saved_recipes(&p, &latest, 6);
    let selected = m.choice(&p, PresetSlot::Neutral, m.choices[0]).unwrap();
    assert_eq!(selected.id, "new");
    assert_eq!(
        m.choice(&p, PresetSlot::Neutral, 0).unwrap().name,
        "Renamed"
    );
    assert!(
        m.choice(&p, PresetSlot::Neutral, 1).is_none(),
        "deleted startup choice stays deleted"
    );
    assert_eq!(m.choice(&p, PresetSlot::X, m.choices[5]).unwrap().id, "new");
    assert!(m.editor.is_none());
    let saved_actions = m.settings(&p, LedgeSettings::default()).sequence_overrides[0];
    m.choices[0] = 0; // Temporary match selection must not replace startup default.
    drop(m);
    let mut next = MenuSelection::new(&p);
    next.load_saved_recipes(&p, &latest, 6);
    assert_eq!(
        next.settings(&p, LedgeSettings::default())
            .sequence_overrides[0],
        saved_actions
    );
    assert_eq!(
        next.choice(&p, PresetSlot::Neutral, next.choices[0])
            .unwrap()
            .id,
        "new"
    );
    assert!(next.editor.is_none() && next.save_status.is_none());
}

#[test]
fn l_stays_reserved_and_armed_hidden_compact_navigation_and_page_switches() {
    use crate::manual_controls::{ManualOverlay, TurboOutput};
    for allow_editing in [false, true] {
        let p = configured();
        let mut m = MenuSelection::new(&p);
        let mut overlay = ManualOverlay::default();
        m.input_with_editing(0, &p, allow_editing);
        for (step, buttons) in [
            MAPPER_L,
            MAPPER_L | MAPPER_MINUS,
            MAPPER_L,
            MAPPER_L | DPAD_DOWN,
            MAPPER_L,
            MAPPER_L | MAPPER_MINUS,
            MAPPER_L,
        ]
        .into_iter()
        .enumerate()
        {
            let consumed = m.input_with_editing(buttons, &p, allow_editing);
            assert_ne!(
                consumed & MAPPER_L,
                0,
                "L leaked with editing={allow_editing}, buttons={buttons:x}"
            );
            let out = overlay.apply_controls(
                ButtonTrace {
                    held: buttons,
                    pressed: buttons,
                    released: 0,
                    previous: 0,
                },
                [0.; 2],
                TurboOutput::default(),
                consumed,
                0,
                0,
            );
            assert_eq!(out.buttons.held & MAPPER_L, 0);
            assert_eq!(out.buttons.pressed & MAPPER_L, 0);
            if buttons == MAPPER_L && step > 0 {
                assert!(m.ledge_held(buttons));
            }
        }
        if !allow_editing {
            m.input_with_editing(MAPPER_L | MAPPER_MINUS, &p, false);
            m.input_with_editing(MAPPER_L, &p, false);
            let consumed = m.input_with_editing(MAPPER_L | MENU_STICK_CLICK, &p, false);
            assert_ne!(
                consumed & MAPPER_L,
                0,
                "L leaked on auxiliary-page transition"
            );
            let consumed = m.input_with_editing(MAPPER_L, &p, false);
            assert_ne!(consumed & MAPPER_L, 0);
            assert!(m.ledge_held(MAPPER_L));
        }
    }
}

#[test]
fn full_editor_reserves_l_without_running_ledge_hold() {
    let p = configured();
    let mut m = MenuSelection::new(&p);
    m.input(0, &p);
    m.input(MAPPER_MINUS, &p);
    m.input(0, &p);
    m.input(MENU_STICK_CLICK, &p);
    m.input(0, &p);
    let consumed = m.input(MAPPER_L, &p);
    assert!(m.recipe_editing);
    assert_ne!(consumed & MAPPER_L, 0);
    assert!(!m.ledge_held(MAPPER_L));
}

#[test]
fn l_press_does_not_cancel_or_reset_when_autopilot_is_already_on() {
    let p = configured();
    let mut m = MenuSelection::new(&p);
    m.input(0, &p);
    m.auto_ledge = true;
    let consumed = m.input(MAPPER_L, &p);
    assert!(!m.cancel_pending);
    assert!(m.ledge_mode());
    assert!(m.ledge_held(MAPPER_L));
    assert_ne!(consumed & MAPPER_L, 0);
    m.input(0, &p);
    assert!(!m.cancel_pending && m.ledge_mode());
    assert!(!m.ledge_held(0));
    m.auto_ledge = false;
    m.input(MAPPER_L, &p);
    assert!(m.cancel_pending); // Original held-L activation takeover remains.
    m.input(MAPPER_L, &p);
    assert!(m.ledge_held(MAPPER_L));
}

#[test]
fn match_cursor_pages_and_close_keep_held_l_running_without_rebuilding_values() {
    let p = configured();
    let mut m = MenuSelection::new(&p);
    m.input_with_editing(0, &p, false);
    m.input_with_editing(MAPPER_L, &p, false);
    assert!(!m.menu_hidden);
    m.input_with_editing(MAPPER_L, &p, false);
    assert!(!m.cancel_pending);
    let choices = m.choices;
    let content = m.content_revision;
    let revision = m.revision;
    m.input_with_editing(MAPPER_L, &p, false);
    m.input_with_editing(MAPPER_L | DPAD_DOWN, &p, false);
    assert_eq!(m.row, MenuRow::Jump);
    assert!(!m.cancel_pending);
    assert_eq!(m.content_revision, content);
    assert!(m.revision > revision);
    assert_eq!(m.choices, choices);
    assert!(m.ledge_held(MAPPER_L | DPAD_DOWN));
    m.input_with_editing(MAPPER_L, &p, false);
    m.input_with_editing(MAPPER_L | MENU_STICK_CLICK, &p, false);
    assert!(m.button_menu);
    assert!(!m.cancel_pending);
    assert!(m.content_revision > content);
    m.input_with_editing(MAPPER_L, &p, false);
    m.input_with_editing(MAPPER_L | MAPPER_MINUS, &p, false);
    assert!(m.menu_hidden);
    assert!(!m.cancel_pending);
    assert!(m.ledge_held(MAPPER_L | MAPPER_MINUS));
    assert!(m.editor.is_none() && m.catalog.is_none());
}
#[test]
fn match_single_choice_swap_is_noop_and_unavailable_choice_does_not_claim_first_position() {
    let p = configured();
    let mut m = MenuSelection::new(&p);
    m.input_with_editing(0, &p, false);
    m.input_with_editing(MAPPER_L, &p, false);
    m.input_with_editing(MAPPER_L, &p, false);
    let rev = m.revision;
    let content = m.content_revision;
    m.input_with_editing(MAPPER_L | DPAD_RIGHT, &p, false);
    assert!(!m.cancel_pending);
    assert_eq!(m.revision, rev);
    assert_eq!(m.content_revision, content);
    m.choices[0] = p.lists[0].options.len(); // Unsaved template is not a saved choice.
    let (index, count) = m.choice_position(&p, PresetSlot::Neutral);
    assert_eq!(count, 1);
    assert_eq!(index, count);
}
#[test]
fn match_choice_switches_match_actual_timeline_and_require_fresh_dpad_edges() {
    let p=presets("[kirby.neutral.a]\nname=First\nstart_delay=0\ndefault=true\nactions=hold:a:3\n[kirby.neutral.b]\nname=Second\nstart_delay=2\nactions=hold:b:7\n[kirby.x.one]\nname=Button One\nstart_delay=0\ndefault=true\nactions=up_b:3\n[kirby.x.two]\nname=Button Two\nstart_delay=0\nactions=hold:r:4\n");
    let mut m = MenuSelection::new(&p);
    m.input_with_editing(0, &p, false);
    for (row, slot) in [
        (MenuRow::Neutral, PresetSlot::Neutral),
        (MenuRow::XSequence, PresetSlot::X),
    ] {
        if slot.is_button() {
            m.input_with_editing(0, &p, false);
            m.input_with_editing(MENU_STICK_CLICK, &p, false);
        }
        m.row = row;
        for _ in 0..5 {
            m.input_with_editing(0, &p, false);
            let before = m.choices[slot as usize];
            let content = m.content_revision;
            m.input_with_editing(MAPPER_L | DPAD_RIGHT, &p, false);
            assert_ne!(m.choices[slot as usize], before);
            assert!(m.cancel_pending);
            assert!(m.content_revision > content);
            let chosen = m.choice(&p, slot, m.choices[slot as usize]).unwrap();
            let actual = m.settings(&p, LedgeSettings::default()).sequence_overrides[slot as usize];
            if chosen.sequence.len == 0 {
                assert!(actual.is_none());
            } else {
                assert_eq!(actual, Some(chosen.sequence));
            }
            let snapshot = (m.revision, m.content_revision, m.choices);
            for _ in 0..20 {
                m.input_with_editing(MAPPER_L | DPAD_RIGHT, &p, false);
                assert_eq!((m.revision, m.content_revision, m.choices), snapshot);
                assert!(!m.cancel_pending);
            }
        }
    }
    assert!(m.editor.is_none() && m.catalog.is_none());
    assert!(!m.save_requested && !m.delete_requested);
}
#[test]
fn match_assist_switches_toggle_exactly_one_mode_once_per_press() {
    fn flags(m: &MenuSelection) -> [bool; 8] {
        [
            m.roll_rules,
            m.shield_pressure,
            m.spot_punish,
            m.out_of_shield,
            m.grab_dodge,
            m.auto_ledge,
            m.auto_tech,
            m.auto_trump,
        ]
    }
    let rows = [
        MenuRow::RollRules,
        MenuRow::ShieldPressure,
        MenuRow::SpotPunish,
        MenuRow::OutOfShield,
        MenuRow::GrabDodge,
        MenuRow::AutoLedge,
        MenuRow::AutoTech,
        MenuRow::AutoTrump,
    ];
    let p = configured();
    let mut m = MenuSelection::new(&p);
    m.input_with_editing(0, &p, false);
    m.input_with_editing(0, &p, false);
    m.input_with_editing(MENU_STICK_CLICK, &p, false);
    for (index, row) in rows.into_iter().enumerate() {
        assert!(m.row_visible(row));
        m.row = row;
        m.input_with_editing(0, &p, false);
        let before = flags(&m);
        let content = m.content_revision;
        m.input_with_editing(MAPPER_L | DPAD_LEFT, &p, false);
        let mut expected = before;
        expected[index] = !expected[index];
        assert_eq!(flags(&m), expected);
        assert!(m.content_revision > content);
        for _ in 0..12 {
            m.input_with_editing(MAPPER_L | DPAD_LEFT, &p, false);
            assert_eq!(flags(&m), expected);
            assert!(!m.cancel_pending);
        }
        m.input_with_editing(0, &p, false);
        m.input_with_editing(MAPPER_L | DPAD_RIGHT, &p, false);
        assert_eq!(flags(&m), before);
    }
    m.row = MenuRow::AutoMash;
    m.input_with_editing(0, &p, false);
    m.input_with_editing(MAPPER_L | DPAD_LEFT, &p, false);
    assert!(m.toggle_mash);
    m.input_with_editing(MAPPER_L | DPAD_LEFT, &p, false);
    assert!(!m.toggle_mash);
    assert!(m.editor.is_none() && m.catalog.is_none());
}
#[test]
fn idle_match_polls_and_live_feedback_never_invalidate_selector_contents() {
    let p = configured();
    let mut m = MenuSelection::new(&p);
    m.input_with_editing(0, &p, false);
    m.input_with_editing(0, &p, false);
    let stamp = (m.revision, m.content_revision, m.recipe_revision);
    for n in 0..240 {
        m.record_rule_feedback(
            if n % 2 == 0 {
                crate::reaction_rules::Feedback::Range
            } else {
                crate::reaction_rules::Feedback::Watching
            },
            Some(1),
        );
        m.input_with_editing(0, &p, false);
        assert_eq!((m.revision, m.content_revision, m.recipe_revision), stamp);
    }
}

#[test]
fn match_all_eleven_saved_slots_select_the_matching_payload() {
    let mut source = String::new();
    for slot in [
        "neutral",
        "jump",
        "roll",
        "attack",
        "drop",
        "x",
        "y",
        "dpad_up",
        "dpad_down",
        "dpad_left",
        "dpad_right",
        "l",
        "r",
        "zl",
        "zr",
        "l3",
        "r3",
    ] {
        source.push_str(&format!("[kirby.{slot}.one]\nname={slot} One\nstart_delay=0\ndefault=true\nactions=hold:a:3\n[kirby.{slot}.two]\nname={slot} Two\nstart_delay=2\nactions=hold:b:7\n"));
    }
    let p = presets(&source);
    let mut m = MenuSelection::new(&p);
    m.input_with_editing(0, &p, false);
    for slot in PresetSlot::ALL {
        m.open_page(if slot.is_button() {
            MenuRow::ButtonsPage
        } else {
            MenuRow::LedgePage
        });
        m.row = MenuRow::ALL
            .into_iter()
            .find(|r| r.slot() == Some(slot))
            .unwrap();
        m.input_with_editing(0, &p, false);
        m.input_with_editing(MAPPER_L | DPAD_RIGHT, &p, false);
        let chosen = m.choice(&p, slot, m.choices[slot as usize]).unwrap();
        assert_eq!(chosen.id, "two");
        assert_eq!(
            m.settings(&p, LedgeSettings::default()).sequence_overrides[slot as usize],
            Some(chosen.sequence)
        );
        let (position, count) = m.choice_position(&p, slot);
        assert_eq!(position, 1);
        assert_eq!(count, if slot.is_button() { 6 } else { 2 });
    }
    assert!(m.editor.is_none() && m.catalog.is_none());
}
#[test]
fn match_optional_fighter_modes_and_diagnostic_controls_update_their_real_values() {
    let p = configured();
    let mut m = MenuSelection::new(&p);
    m.input_with_editing(0, &p, false);
    m.open_page(MenuRow::ButtonsPage);
    for (kind, row) in [
        (6, MenuRow::IdleCrouch),
        (6, MenuRow::TiltTurn),
        (5, MenuRow::TiltTurn),
        (5, MenuRow::JabLock),
    ] {
        m.y_fighter_kind = kind;
        m.row = row;
        assert!(m.row_visible(row));
        m.input_with_editing(0, &p, false);
        let before = (m.idle_crouch, m.tilt_turn, m.jab_lock);
        m.input_with_editing(MAPPER_L | DPAD_LEFT, &p, false);
        let expected = match row {
            MenuRow::IdleCrouch => (!before.0, before.1, before.2),
            MenuRow::TiltTurn => (before.0, !before.1, before.2),
            _ => (before.0, before.1, !before.2),
        };
        assert_eq!((m.idle_crouch, m.tilt_turn, m.jab_lock), expected);
    }
    for row in [MenuRow::Performance, MenuRow::TestLatency] {
        assert_eq!(m.row_visible(row), cfg!(feature = "diagnostic-logging"));
        if m.row_visible(row) {
            m.row = row;
            m.input_with_editing(0, &p, false);
            let old = (m.performance, m.test_latency);
            m.input_with_editing(MAPPER_L | DPAD_RIGHT, &p, false);
            assert_eq!(
                (m.performance, m.test_latency),
                if row == MenuRow::Performance {
                    (!old.0, old.1)
                } else {
                    (old.0, (old.1 + 1) % 32)
                }
            );
        }
    }
}

#[test]
fn match_autopilot_stays_active_for_consumed_menu_dpad_but_not_hidden_gameplay_dpad() {
    let p = configured();
    let mut m = MenuSelection::new(&p);
    m.input_with_editing(0, &p, false);
    m.auto_ledge = true;
    assert!(m.autopilot_held(0));
    assert!(!m.autopilot_held(DPAD_DOWN));
    assert!(!m.menu_hidden);
    m.input_with_editing(0, &p, false);
    m.input_with_editing(MAPPER_L | DPAD_DOWN, &p, false);
    assert!(!m.cancel_pending);
    assert!(m.autopilot_held(MAPPER_L | DPAD_DOWN));
    m.input_with_editing(0, &p, false);
    m.input_with_editing(DPAD_DOWN, &p, false);
    assert!(!m.autopilot_held(DPAD_DOWN));
    m.input_with_editing(0, &p, false);
    m.input_with_editing(MENU_STICK_CLICK, &p, false);
    assert!(m.autopilot_held(MENU_STICK_CLICK));
    m.input_with_editing(0, &p, false);
    m.input_with_editing(MAPPER_MINUS, &p, false);
    assert!(m.menu_hidden);
    assert!(m.autopilot_held(MAPPER_MINUS));
    m.auto_ledge = false;
    assert!(!m.autopilot_held(0));
}

#[test]
fn saved_button_defaults_restore_by_identity_with_all_timings_and_loop_flags() {
    use crate::sequence_config::persist;
    for slot in PresetSlot::ALL.into_iter().filter(|s| s.is_button()) {
        let source = format!(
            "[kirby.{}.old]\nstart_delay=0\ndefault=true\nactions=jab:4\n",
            button_slot_token(slot)
        );
        let p = presets(&source);
        let mut seq=Catalog::parse("[kirby.x.new]\nstart_delay=3\nwhile_held=true\nground_only=true\nactions=facing_input:b:1:1:160,wait:7,drift:opponent:1:40\n").unwrap().entries()[0].sequence;
        seq.while_held = true;
        let updated = persist::update_named(
            &source,
            6,
            slot,
            "new",
            seq,
            true,
            Some("My edited default"),
        )
        .unwrap();
        let updated = persist::set_default(&updated, 6, slot, "new").unwrap();
        let saved = Catalog::parse(&updated).unwrap();
        for _ in 0..2 {
            let mut m = MenuSelection::new(&p);
            m.load_saved_recipes(&p, &saved, 6);
            let index = m.choices[slot as usize];
            let c = m.choice(&p, slot, index).unwrap();
            assert_eq!(c.id, "new");
            assert_eq!(c.name, "My edited default");
            assert_eq!(c.sequence, seq);
            assert_eq!(
                m.settings(&p, LedgeSettings::default()).sequence_overrides[slot as usize],
                Some(seq)
            );
            m.input(0, &p);
            m.input(slot.button(), &p);
            assert_eq!(m.preview_slot, Some(slot));
            m.choices[slot as usize] = 0; // Temporary swaps must not overwrite the saved default.
        }
    }
}
#[test]
fn reload_without_explicit_button_default_matches_cold_start_normal_button() {
    for slot in PresetSlot::ALL.into_iter().filter(|s| s.is_button()) {
        let token = button_slot_token(slot);
        let source = format!("[kirby.{token}.saved]\nstart_delay=0\ndefault=true\nactions=b:4\n");
        let p = presets(&source);
        let saved = Catalog::parse(&source.replace("default=true", "default=false")).unwrap();
        let mut next = MenuSelection::new(&p);
        next.load_saved_recipes(&p, &saved, 6);
        assert_eq!(
            next.choice(&p, slot, next.choices[slot as usize])
                .unwrap()
                .id,
            crate::button_bindings::Builtin::default_for(slot).choice_id()
        );
        assert!(next
            .settings(&p, LedgeSettings::default())
            .sequence_overrides[slot as usize]
            .is_none());
    }
}

fn button_slot_token(slot: PresetSlot) -> &'static str {
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

#[test]
fn lightweight_starts_visible_minus_still_toggles_and_training_starts_hidden() {
    let p = configured();
    let mut m = MenuSelection::new(&p);
    m.input_with_editing(0, &p, false);
    assert!(m.lightweight && !m.menu_hidden && m.quick_menu);
    m.input_with_editing(MAPPER_MINUS, &p, false);
    assert!(m.menu_hidden);
    for _ in 0..20 {
        m.input_with_editing(MAPPER_MINUS, &p, false);
        assert!(m.menu_hidden);
    }
    m.input_with_editing(0, &p, false);
    m.input_with_editing(MAPPER_MINUS, &p, false);
    assert!(!m.menu_hidden);
    m.input_with_editing(0, &p, false);
    m.input_with_editing(MAPPER_MINUS, &p, false);
    assert!(m.menu_hidden);
    m.input_with_editing(0, &p, true);
    m.input_with_editing(0, &p, false);
    assert!(
        m.menu_hidden,
        "scene changes preserve an explicitly hidden selector"
    );
    assert!(m.editor.is_none() && m.catalog.is_none());
    let mut training = MenuSelection::new(&p);
    training.input_with_editing(0, &p, true);
    assert!(training.menu_hidden && !training.lightweight);
}
