use super::*;
use crate::{
    fighter_profiles::{presets::PresetSlot, FighterProfiles},
    ledge_trap::DPAD_DOWN,
    manual_controls::DPAD_RIGHT,
    settings::AssistSettings,
    settings_menu::{MenuSelection, MENU_STICK_CLICK},
};

/// Regression test for `edited_actions_keep_their_matching_preset_kind`.
#[test]
fn edited_actions_keep_their_matching_preset_kind() {
    let forward_tilt = PRESETS
        .iter()
        .position(|(name, _)| *name == "Forward A (Fair / Fsmash)")
        .unwrap();
    let edited_input = Action::Input {
        buttons: MAPPER_A,
        stick: [1., 0.],
        frames: 7,
        relative: true,
    };
    assert_eq!(preset_kind(edited_input), Some(forward_tilt));

    assert!(same_preset_shape(
        Action::Range(RangeCheck::Verify(4.)),
        Action::Range(RangeCheck::Verify(10.))
    ));
    assert!(!same_preset_shape(
        Action::Range(RangeCheck::Verify(4.)),
        Action::Range(RangeCheck::Ignore)
    ));
}

/// Returns the `presets` result.
fn presets() -> crate::fighter_profiles::presets::MenuPresets {
    let file = crate::sequence_config::Catalog::parse(
        "[kirby.neutral.first]\nstart_delay=0\nactions=short_hop,wait:2,nair",
    )
    .unwrap();
    let mut p = FighterProfiles::default();
    p.add_sequence_choices(&file).unwrap();
    p.profile(6).unwrap().menu_options
}
/// Updates the `tap` state using the supplied inputs.
fn tap(menu: &mut MenuSelection, p: &crate::fighter_profiles::presets::MenuPresets, button: u32) {
    menu.input(0, p);
    menu.input(button, p);
}
/// Copies the menu can template a button into ledge without changing source or name.
#[test]
fn copy_menu_can_template_a_button_into_ledge_without_changing_source_or_name() {
    let file=crate::sequence_config::Catalog::parse("[kirby.neutral.target]\nname=Keep my name\nstart_delay=0\nactions=grab\n[kirby.x.source]\nname=Nice down smash\ndefault=true\nstart_delay=3\ntimeout=700\nspatial_timeout=80\nwhile_held=true\nactions=wait:2,down_smash:4").unwrap();
    let mut profiles = FighterProfiles::default();
    profiles.add_sequence_choices(&file).unwrap();
    let p = profiles.profile(6).unwrap().menu_options;
    let mut m = MenuSelection::new(&p);
    m.show();
    m.input(0, &p);
    tap(&mut m, &p, MENU_STICK_CLICK);
    let before = m.editor.as_ref().unwrap().compiled((0, 0)).unwrap();
    tap(&mut m, &p, MAPPER_A);
    for _ in 0..8 {
        tap(&mut m, &p, crate::ledge_trap::DPAD_DOWN);
    }
    tap(&mut m, &p, MAPPER_A);
    assert!(m.recipe_panel(&p).header.join(" ").contains("COPY RECIPE"));
    let e = m.editor.as_ref().unwrap();
    assert_eq!(e.copy_sources.len(), 1 + PresetSlot::BUTTONS.len());
    assert_eq!(e.copy_sources[0].name, "Nice down smash");
    assert!(e.copy_sources[1..].iter().all(|s| s.name == "B-reverse"));
    tap(&mut m, &p, MAPPER_A);
    let e = m.editor.as_ref().unwrap();
    let target = e.compiled((0, 0)).unwrap();
    let source = *p.lists[5].options[0].sequence;
    assert_eq!(target.actions(), source.actions());
    assert_eq!(target.start_delay, 3);
    assert_eq!(target.timeout, 700);
    assert_eq!(target.spatial_timeout, 80);
    assert!(!target.while_held);
    assert_eq!(
        m.choice(&p, crate::fighter_profiles::presets::PresetSlot::Neutral, 0)
            .unwrap()
            .name,
        "Keep my name"
    );
    assert_eq!(*p.lists[5].options[0].sequence, source);
    assert_ne!(target, before);
    assert!(!m.save_requested);
    assert_eq!(e.copy_sources.capacity(), 0);
}
#[test]
fn copy_picker_visits_stick_click_recipes_and_templates_without_aliasing() {
    let file = crate::sequence_config::Catalog::parse(
        "[kirby.neutral.target]\nstart_delay=0\nactions=grab\n[kirby.l3.left]\nname=Left click\nstart_delay=2\nactions=nair\n[kirby.r3.right]\nname=Right click\nstart_delay=3\nactions=up_b\n[template.generic]\nname=Generic\nstart_delay=4\nactions=down_b\n",
    ).unwrap();
    let mut profiles = FighterProfiles::default();
    profiles.add_sequence_choices(&file).unwrap();
    let p = profiles.profile(6).unwrap().menu_options;
    for (slot, label, name) in [
        (Some(PresetSlot::L3), "L3 button", "Left click"),
        (Some(PresetSlot::R3), "R3 button", "Right click"),
        (None, "Generic templates", "Generic"),
    ] {
        let mut m = MenuSelection::new(&p);
        m.show();
        tap(&mut m, &p, MENU_STICK_CLICK);
        tap(&mut m, &p, MAPPER_ZL | MAPPER_ZR);
        for _ in 0..8 {
            tap(&mut m, &p, DPAD_DOWN);
        }
        tap(&mut m, &p, MAPPER_A);
        let mut visited = false;
        for _ in 0..14 {
            let panel = m.recipe_panel(&p);
            if panel.header.iter().any(|line| line.contains(label)) {
                assert!(panel.body.iter().any(|line| line.contains(name)));
                visited = true;
                break;
            }
            tap(&mut m, &p, DPAD_RIGHT);
        }
        assert!(visited, "{slot:?} copy category is unreachable");
        tap(&mut m, &p, MAPPER_A);
        let copied = m.recipe_snapshot(&p, PresetSlot::Neutral, 0).unwrap();
        let source = slot.map_or(p.templates[0].sequence, |slot| {
            *p.list(slot).options[0].sequence
        });
        assert_eq!(copied.actions(), source.actions());
        assert_eq!(copied.start_delay, source.start_delay);
        assert!(!m.editor.as_ref().unwrap().command_active());
    }
}

/// Regression test for `editor_tools_consume_controls_cancel_safely_and_do_not_rebuild_while_idle`.
#[test]
fn editor_tools_consume_controls_cancel_safely_and_do_not_rebuild_while_idle() {
    let p = presets();
    let mut menu = MenuSelection::new(&p);
    menu.show();
    menu.input(0, &p);
    tap(&mut menu, &p, MENU_STICK_CLICK);
    assert!(menu.recipe_editing);
    tap(&mut menu, &p, MAPPER_A);
    assert!(menu
        .recipe_panel(&p)
        .header
        .join(" ")
        .contains("STEP TOOLS"));
    assert!(!menu.recipe_panel(&p).footer.join(" ").contains("B: Manage"));
    let revision = menu.revision;
    for _ in 0..120 {
        menu.input(0, &p);
    }
    assert_eq!(menu.revision, revision);
    tap(&mut menu, &p, MAPPER_Y);
    assert!(
        menu.save_requested,
        "Y saves the current recipe from tools too"
    );
    tap(&mut menu, &p, MAPPER_B);
    assert!(!menu.managing());
    assert!(!menu.recipe_editing);
    tap(&mut menu, &p, MENU_STICK_CLICK);
    let mask = menu.input(MAPPER_ZR, &p);
    assert_ne!(mask & MAPPER_ZR, 0);
    assert!(menu.recipe_panel(&p).header.join(" ").contains("STEP 1 /"));
    menu.input(MAPPER_ZR | MAPPER_MINUS, &p);
    assert!(menu.menu_hidden);
    assert_ne!(menu.input(MAPPER_ZR, &p) & MAPPER_ZR, 0);
    menu.input(0, &p);
    assert_eq!(menu.input(MAPPER_ZR, &p) & MAPPER_ZR, 0);
}

/// Regression test for `modified_save_explains_rejection_in_editor_and_tools_without_losing_edits`.
#[test]
fn modified_save_explains_rejection_in_editor_and_tools_without_losing_edits() {
    let p = presets();
    for tools in [false, true] {
        let mut menu = MenuSelection::new(&p);
        menu.show();
        menu.input(0, &p);
        tap(&mut menu, &p, MENU_STICK_CLICK);
        let before = menu
            .recipe_snapshot(&p, crate::fighter_profiles::presets::PresetSlot::Neutral, 0)
            .unwrap();
        if tools {
            tap(&mut menu, &p, MAPPER_A);
        }
        tap(&mut menu, &p, crate::native_input::MAPPER_L | MAPPER_Y);
        assert!(!menu.save_requested);
        assert!(menu
            .save_status
            .as_deref()
            .unwrap()
            .contains("Release other buttons"));
        assert_eq!(
            menu.recipe_snapshot(&p, crate::fighter_profiles::presets::PresetSlot::Neutral, 0),
            Some(before)
        );
        tap(&mut menu, &p, MAPPER_Y);
        assert!(menu.save_requested);
    }
}
/// Regression test for `button_activation_is_editable_and_not_exposed_for_ledge_recipes`.
#[test]
fn button_activation_is_editable_and_not_exposed_for_ledge_recipes() {
    let mut e = RecipeEditor::default();
    let source = RecipeEditor::custom();
    e.ensure((0, 0), source);
    assert!(!fields(&e.recipes[&(0, 0)])
        .iter()
        .any(|f| matches!(f, Field::WhileHeld)));
    let key = (5, 0);
    e.ensure(key, source);
    e.cursor = fields(&e.recipes[&key])
        .iter()
        .position(|f| matches!(f, Field::WhileHeld))
        .unwrap();
    assert!(!e.repeatable(key));
    assert!(e.adjust(key, true));
    assert!(e.compiled(key).unwrap().while_held);
    assert!(e
        .panel(key, true)
        .body
        .join(" ")
        .contains("Loop while held"));
}
/// Regression test for `repeated_tap_picker_has_only_buttons_and_tilts_with_gap_and_total_fields`.
#[test]
fn repeated_tap_picker_has_only_buttons_and_tilts_with_gap_and_total_fields() {
    let source = crate::sequence_config::Catalog::parse(
        "[kirby.x.t]\nstart_delay=0\nactions=tap:a:0:0:1:11:120",
    )
    .unwrap()
    .entries()[0]
        .sequence;
    let mut e = RecipeEditor::default();
    let key = (5, 0);
    e.ensure(key, source);
    assert!(!PRESETS.iter().any(|(name, _)| *name == "Repeat down tilt"));
    let fs = fields(&e.recipes[&key]);
    assert!(!fs.iter().any(|f| matches!(f, Field::X(0) | Field::Y(0))));
    for (button, stick, label) in TAP_BUTTONS.iter().skip(1).chain(TAP_BUTTONS.iter().take(1)) {
        e.cursor = fields(&e.recipes[&key])
            .iter()
            .position(|f| matches!(f, Field::TapButtons(0)))
            .unwrap();
        assert!(e.adjust(key, true));
        assert!(
            matches!(e.compiled(key).unwrap().actions(), [Action::Tap {buttons,stick:axes,hold:1,..}] if buttons==button && axes==stick)
        );
        assert!(field_value(&e.recipes[&key], Field::TapButtons(0)).contains(label));
    }
    for field in [Field::TapGap(0), Field::Duration(0)] {
        e.cursor = fields(&e.recipes[&key])
            .iter()
            .position(|f| *f == field)
            .unwrap();
        assert!(e.adjust(key, true));
    }
    assert!(matches!(
        e.compiled(key).unwrap().actions(),
        [Action::Tap {
            gap: 12,
            frames: 121,
            ..
        }]
    ));
    assert!(e.panel(key, true).body.join(" ").contains("every 13f"));
}
/// Regression test for `condition_fields_roundtrip_and_slot_growth_cannot_erase_actions`.
#[test]
fn condition_fields_roundtrip_and_slot_growth_cannot_erase_actions() {
    let source = crate::sequence_config::Catalog::parse(
        "[yoshi.roll.t]\nstart_delay=0\nactions=check:enemy_within:12:8,hold:a:60\n",
    )
    .unwrap()
    .entries()[0]
        .sequence;
    let key = (0, 0);
    let mut editor = RecipeEditor::default();
    editor.ensure(key, source);
    assert_eq!(editor.compiled(key).unwrap().actions(), source.actions());
    let r = editor.recipes.get(&key).unwrap();
    assert!(action_name(r, 0).contains("distance within"));
    editor.cursor = fields(r)
        .iter()
        .position(|f| matches!(f, Field::Distance(0)))
        .unwrap();
    assert!(editor.adjust(key, true));
    assert!(matches!(
        editor.compiled(key).unwrap().actions()[0],
        Action::Check {
            value: 13.,
            wait: 8,
            ..
        }
    ));
    let r = editor.recipes.get(&key).unwrap();
    editor.cursor = fields(r)
        .iter()
        .position(|f| matches!(f, Field::Slots))
        .unwrap();
    // Filled trailing actions cannot be removed by reducing the count.
    for _ in 0..8 {
        editor.cursor = fields(&editor.recipes[&key]).len() - 1;
        editor.adjust(key, false);
    }
    assert_eq!(editor.recipes[&key].sequence.len, 2);
    for _ in 0..40 {
        let r = &editor.recipes[&key];
        editor.cursor = fields(r).len() - 1;
        editor.adjust(key, true);
    }
    assert_eq!(editor.recipes[&key].sequence.len, MAX_ACTIONS as u8);
    assert_eq!(
        editor.compiled(key).unwrap().actions(),
        &[
            Action::Check {
                condition: Condition::EnemyWithin,
                value: 13.,
                wait: 8
            },
            source.actions()[1],
        ]
    );
}
/// Regression test for `schedule_shows_input_frames_with_one_based_recognition_and_preserves_recipe`.
#[test]
fn schedule_shows_input_frames_with_one_based_recognition_and_preserves_recipe() {
    let source = crate::sequence_config::Catalog::parse(
        "[kirby.neutral.t]\nstart_delay=2\nactions=short_hop:1,wait:3,hold:a:4,wait:2\n",
    )
    .unwrap()
    .entries()[0]
        .sequence;
    let recipe = Recipe::new(source);
    assert_eq!(schedule_summary(&recipe), (12, false));
    let panel = recipe_panel(&recipe, &fields(&recipe), None, false, None);
    let text = panel.header.join("\n") + "\n" + &panel.body.join("\n");
    assert!(text.contains("Input schedule: 12f | done F13"));
    assert!(text.contains("F3"));
    assert!(text.contains("F7-10"));
    assert!(text.contains("F11-12"));
    assert_eq!(recipe.compile().unwrap().start_delay, source.start_delay);
    assert_eq!(recipe.compile().unwrap().actions(), source.actions());
    let mut limited = recipe;
    limited.sequence.timeout = 6;
    let header = recipe_panel(&limited, &fields(&limited), None, false, None)
        .header
        .join(" ");
    assert!(header.contains("timeout 6f"));
    assert!(!header.contains("done F"));
}
#[test]
fn manual_takeover_hold_has_variable_timing_and_no_exact_completion() {
    let mut source = Sequence::default();
    source.len = 2;
    source.actions[..2].copy_from_slice(&[key(MAPPER_ZR, 0., 0., 0), key(MAPPER_A, 0., 0., 4)]);
    let recipe = Recipe::new(source);
    assert_eq!(schedule_summary(&recipe), (4, true));
    let panel = recipe_panel(&recipe, &fields(&recipe), None, false, None);
    assert!(panel.header.join(" ").contains("variable checks"));
    assert!(!panel.header.join(" ").contains("done F"));
    assert!(panel.body.join(" ").contains("after checks"));
    assert_eq!(recipe.compile(), Some(source));
}

/// Regression test for `schedule_never_claims_exact_frames_after_a_spatial_or_block_check`.
#[test]
fn schedule_never_claims_exact_frames_after_a_spatial_or_block_check() {
    let source = crate::sequence_config::Catalog::parse(
        "[yoshi.roll.t]\nstart_delay=0\nactions=walk:inward:4:50%,face:opponent,wait:2,grab\n",
    )
    .unwrap()
    .entries()[0]
        .sequence;
    let recipe = Recipe::new(source);
    let panel = recipe_panel(&recipe, &fields(&recipe), None, false, None);
    assert!(panel.header.join(" ").contains("variable checks"));
    let text = panel.body.join("\n");
    assert!(text.contains("F1 check"));
    assert!(text.contains("after checks"));
    assert!(!panel.header.join(" ").contains("done F"));
}
/// Regression test for `imported_input_is_descriptive_and_can_be_reselected_after_cycling`.
#[test]
fn imported_input_is_descriptive_and_can_be_reselected_after_cycling() {
    let action = Action::Input {
        buttons: MAPPER_A | MAPPER_ZR,
        stick: [0.35, -0.2],
        frames: 19,
        relative: false,
    };
    let mut source = Sequence::default();
    source.len = 1;
    source.actions[0] = action;
    let mut editor = RecipeEditor::default();
    let key = (0, 0);
    editor.ensure(key, source);
    editor.cursor = 1;
    let name = editor.lines(key).join("\n");
    assert!(name.contains("A+ZR | X:35% Y:-20% (screen)"));
    assert!(editor.adjust(key, true));
    assert_eq!(editor.compiled(key).unwrap().len, 0);
    assert!(editor.adjust(key, false));
    assert_eq!(editor.compiled(key).unwrap().actions(), &[action]);
    // Full forward cycle also keeps the original selectable.
    for _ in 0..PRESETS.len() + 1 {
        editor.adjust(key, true);
    }
    assert_eq!(editor.compiled(key).unwrap().actions(), &[action]);
}
#[test]
fn wait_inside_facing_branch_remains_in_that_branch() {
    let sequence = crate::sequence_config::Catalog::parse(
        "[kirby.x.branch]\nstart_delay=0\nactions=if_facing_opponent,wait:3,a:1,else_facing,b:1,end_facing",
    ).unwrap().entries()[0].sequence;
    let recipe = Recipe::new(sequence);
    assert_eq!(recipe.compile().unwrap().actions(), sequence.actions());
    assert_eq!(recipe.before[1], 0);
}

#[test]
fn full_recipe_with_waits_can_open_and_compile_without_overflow() {
    let mut sequence = Sequence::default();
    sequence.len = MAX_ACTIONS as u8;
    for (i, action) in sequence.actions.iter_mut().enumerate() {
        *action = if i % 2 == 0 {
            Action::Wait(255)
        } else {
            key(MAPPER_A, 0., 0., 1)
        };
    }
    let recipe = Recipe::new(sequence);
    assert_eq!(recipe.compile().unwrap().actions(), sequence.actions());
}

/// Regression test for `conditional_range_choices_keep_distinct_names_and_controls`.
#[test]
fn conditional_range_choices_keep_distinct_names_and_controls() {
    let mut source = Sequence::default();
    source.len = 4;
    source.actions[3] = Action::Range(RangeCheck::FighterGrabRangeInstant);
    source.actions[0] = Action::Range(RangeCheck::FighterGrabRange);
    source.actions[1] = Action::Range(RangeCheck::Verify(17.));
    source.actions[2] = Action::ShieldUntilBlock(60);
    let r = Recipe::new(source);
    assert_eq!(action_name(&r, 0), "Check fighter grab reach");
    assert_eq!(action_name(&r, 1), "Check opponent distance");
    assert_eq!(action_name(&r, 2), "Wait for shield hit");
    assert_eq!(action_name(&r, 3), "Check fighter grab reach instantly");
    let fs = fields(&r);
    assert!(!fs.iter().any(|f| matches!(f, Field::Distance(0))));
    assert!(fs.iter().any(|f| matches!(f, Field::Distance(1))));
    assert!(fs.iter().any(|f| matches!(f, Field::Duration(2))));
    assert_eq!(r.compile().unwrap().actions(), source.actions());
}
/// Regression test for `button_hold_duration_changes_only_the_input_hold`.
#[test]
fn button_hold_duration_changes_only_the_input_hold() {
    let mut source = Sequence::default();
    source.len = 1;
    source.actions[0] = key(MAPPER_B, 0., 0., 4);
    let mut editor = RecipeEditor::default();
    let key = (0, 0);
    editor.ensure(key, source);
    editor.cursor = fields(&editor.recipes[&key])
        .iter()
        .position(|f| matches!(f, Field::Duration(0)))
        .unwrap();
    for _ in 0..6 {
        assert!(editor.adjust(key, true));
    }
    let result = editor.compiled(key).unwrap();
    assert_eq!(result.actions(), &[super::key(MAPPER_B, 0., 0., 10)]);
    assert_eq!(result.start_delay, 0);
    assert!(editor.lines(key).join("\n").contains("Hold input: 10f"));
}
/// Regression test for `full_sequence_panel_retains_all_moves_settings_and_exact_focus`.
#[test]
fn full_sequence_panel_retains_all_moves_settings_and_exact_focus() {
    let mut sequence = RecipeEditor::custom();
    sequence.len = 32;
    sequence.actions.fill(key(MAPPER_A, 0.5, -0.25, 4));
    let mut editor = RecipeEditor::default();
    let key = (0, 0);
    editor.ensure(key, sequence);
    editor.cursor = fields(&editor.recipes[&key])
        .iter()
        .position(|f| matches!(f, Field::Before(31)))
        .unwrap(); // Exact field, independent of added settings.
    let panel = editor.panel(key, true);
    let body = panel.body.join("\n");
    assert!(body.contains("01  "));
    assert!(body.contains("32  "));
    assert_eq!(body.matches("Hold input: 4f").count(), 32);
    assert_eq!(body.matches("Left X: 50%").count(), 32);
    assert_eq!(body.matches("Left Y: -25%").count(), 32);
    let focused = &panel.body[panel.focus.unwrap()];
    assert!(focused.contains("[Delay: 0f]"));
    assert!(panel.header.iter().any(|s| s.contains("STEP 32 / 32")));
    let preview = editor.panel(key, false);
    assert!(preview.focus.is_none());
    assert!(
        preview.body.len() < panel.body.len(),
        "only the focused step expands its settings"
    );
}
/// Regression test for `numbered_slot_overview_keeps_hidden_edits_visible_and_marks_saved_state`.
#[test]
fn numbered_slot_overview_keeps_hidden_edits_visible_and_marks_saved_state() {
    let mut e = RecipeEditor::default();
    let key = (0, 0);
    e.ensure(key, RecipeEditor::custom());
    e.cursor = 1;
    assert!(e.adjust(key, true));
    let first = e.lines(key).join("\n");
    assert!(first.contains("STEP 1 / 6"));
    assert!(first.contains("01  Short hop *"));
    e.cursor = fields(&e.recipes[&key])
        .iter()
        .position(|f| matches!(f, Field::Kind(1)))
        .unwrap();
    let next = e.lines(key).join("\n");
    assert!(next.contains("STEP 2 / 6"));
    assert!(next.contains("01  Short hop *"));
    assert!(next.contains("Last change: slot 1"));
    assert!(e.edited(key));
    e.mark_saved(key);
    assert!(!e.edited(key));
    assert!(!e.lines(key).join("\n").contains("01  Short hop *"));
}
/// Regression test for `training_reference_tracks_focused_move_and_picker_without_changing_recipe`.
#[test]
fn training_reference_tracks_focused_move_and_picker_without_changing_recipe() {
    let p = presets();
    let mut menu = MenuSelection::new(&p);
    menu.show();
    menu.input(0, &p);
    tap(&mut menu, &p, MENU_STICK_CLICK);
    let recipe_key = (
        PresetSlot::Neutral as usize,
        menu.choices[PresetSlot::Neutral as usize],
    );
    let editor = menu.editor.as_mut().unwrap();
    let kind = PRESETS
        .iter()
        .position(|(label, _)| *label == "Dtilt")
        .unwrap();
    let r = editor.recipes.get_mut(&recipe_key).unwrap();
    r.kinds[0] = kind;
    r.sequence.actions[0] = PRESETS[kind].1;
    editor.cursor = fields(r)
        .iter()
        .position(|f| matches!(f, Field::Kind(0)))
        .unwrap();
    let original = editor.compiled(recipe_key).unwrap();
    let hint = menu.recipe_panel(&p).header.join(" ");
    assert!(hint.contains("kirby / Down Tilt"));
    assert!(hint.contains("roster avg"));
    assert_eq!(
        menu.editor.as_ref().unwrap().compiled(recipe_key).unwrap(),
        original
    );
    menu.y_fighter_kind = 5;
    assert!(menu
        .recipe_panel(&p)
        .header
        .join(" ")
        .contains("yoshi / Down Tilt"));
    let editor = menu.editor.as_mut().unwrap();
    let r = editor.recipes.get(&recipe_key).unwrap();
    let fair = PRESETS
        .iter()
        .position(|(label, _)| *label == "Short hop Fair")
        .unwrap();
    let item = commands::candidates(r, 1)
        .iter()
        .position(|i| *i == fair)
        .unwrap();
    editor.modal = Some(commands::Modal::Picker {
        slot: 0,
        group: 1,
        item,
    });
    assert!(menu
        .recipe_panel(&p)
        .header
        .join(" ")
        .contains("yoshi / Forward Air"));
    assert_eq!(
        menu.editor.as_ref().unwrap().compiled(recipe_key).unwrap(),
        original
    );
    menu.lightweight = true;
    let hint = menu.recipe_panel(&p).header.join(" ");
    assert!(!hint.contains("REF "));
    assert!(!hint.contains("roster avg"));
    menu.lightweight = false;
    menu.recipe_editing = false;
    assert!(!menu.recipe_panel(&p).header.join(" ").contains("REF "));
}

/// Saves the wait consumes inputs without hidden edits or held save replay.
#[test]
fn save_wait_consumes_inputs_without_hidden_edits_or_held_save_replay() {
    let p = presets();
    let mut menu = MenuSelection::new(&p);
    menu.show();
    menu.input(0, &p);
    tap(&mut menu, &p, MENU_STICK_CLICK);
    menu.input(MAPPER_Y, &p);
    assert!(menu.save_requested);
    let revision = menu.recipe_revision;
    for _ in 0..10 {
        let buttons = MAPPER_Y | DPAD_RIGHT;
        assert_eq!(menu.wait_for_save(buttons), buttons);
        assert!(!menu.save_requested);
        assert!(!menu.delete_requested);
        assert_eq!(menu.recipe_revision, revision);
    }
    menu.input(MAPPER_Y, &p);
    assert!(
        !menu.save_requested,
        "held Save must not restart when worker exits"
    );
    menu.input(0, &p);
    menu.input(MAPPER_Y, &p);
    assert!(menu.save_requested);
}

/// Regression test for `menu_save_is_editor_only_once_per_press_and_cancels_preview`.
#[test]
fn menu_save_is_editor_only_once_per_press_and_cancels_preview() {
    let p = presets();
    let mut menu = MenuSelection::new(&p);
    menu.show();
    menu.input(0, &p);
    menu.input(MAPPER_Y, &p);
    assert!(!menu.save_requested);
    menu.input(0, &p);
    tap(&mut menu, &p, MENU_STICK_CLICK);
    tap(&mut menu, &p, MAPPER_X);
    assert!(menu.preview_slot.is_some());
    let consumed = menu.input(MAPPER_Y, &p);
    assert!(menu.save_requested);
    assert_ne!(consumed & MAPPER_Y, 0);
    assert!(menu.preview_slot.is_none());
    assert!(menu.cancel_pending);
    menu.input(MAPPER_Y, &p);
    assert!(!menu.save_requested);
    menu.input(0, &p);
    menu.input(MAPPER_Y, &p);
    assert!(menu.save_requested);
    menu.reset_inputs();
    assert!(!menu.save_requested);
}
/// Regression test for `menu_preview_button_is_one_shot_consumed_and_menu_changes_cancel`.
#[test]
fn menu_preview_button_is_one_shot_consumed_and_menu_changes_cancel() {
    let p = presets();
    let mut menu = MenuSelection::new(&p);
    menu.show();
    menu.input(0, &p);
    assert_ne!(menu.input(MAPPER_X, &p) & MAPPER_X, 0);
    assert_eq!(menu.preview_slot, Some(PresetSlot::Neutral));
    let revision = menu.revision;
    for _ in 0..10 {
        menu.input(MAPPER_X, &p);
    }
    assert_eq!(menu.revision, revision);
    menu.input(0, &p);
    tap(&mut menu, &p, MAPPER_X);
    assert_eq!(menu.preview_slot, None);
    tap(&mut menu, &p, MAPPER_X);
    assert!(menu.preview_slot.is_some());
    tap(&mut menu, &p, MENU_STICK_CLICK);
    assert!(menu.recipe_editing);
    assert_eq!(menu.preview_slot, None);
    tap(&mut menu, &p, MAPPER_X);
    assert!(menu.preview_slot.is_some());
    tap(&mut menu, &p, DPAD_DOWN);
    assert_eq!(menu.preview_slot, None);
    tap(&mut menu, &p, MAPPER_X);
    menu.reset_inputs();
    assert_eq!(menu.preview_slot, None);
    menu.input(0, &p);
    menu.input(MAPPER_X, &p);
    let consumed = menu.input(MAPPER_X | crate::native_input::MAPPER_MINUS, &p);
    assert!(menu.menu_hidden && menu.preview_slot.is_none());
    assert_ne!(consumed & MAPPER_X, 0);
    assert_eq!(menu.preview_slot, None);
    assert_ne!(consumed & MAPPER_X, 0);
    let consumed = menu.input(MAPPER_X, &p);
    assert_eq!(
        super::super::observation_buttons(MAPPER_X, consumed) & MAPPER_X,
        0
    );
    menu.input(0, &p);
    assert_eq!(menu.input(MAPPER_X, &p) & MAPPER_X, 0);
}
/// Regression test for `recipe_editor_numeric_hold_accelerates_and_release_resets`.
#[test]
fn recipe_editor_numeric_hold_accelerates_and_release_resets() {
    let p = presets();
    let mut menu = MenuSelection::new(&p);
    menu.show();
    tap(&mut menu, &p, MENU_STICK_CLICK);
    let delay = |m: &MenuSelection| {
        m.settings(&p, AssistSettings::default()).sequence_overrides[0]
            .unwrap()
            .start_delay
    };
    menu.input(DPAD_RIGHT, &p);
    assert_eq!(delay(&menu), 1);
    for _ in 0..23 {
        menu.input(DPAD_RIGHT, &p);
    }
    assert_eq!(delay(&menu), 1);
    menu.input(DPAD_RIGHT, &p);
    assert_eq!(delay(&menu), 2);
    for _ in 0..66 {
        menu.input(DPAD_RIGHT, &p);
    }
    let before = delay(&menu);
    for _ in 0..12 {
        menu.input(DPAD_RIGHT, &p);
    }
    assert_eq!(delay(&menu) - before, 6);
    menu.input(0, &p);
    tap(&mut menu, &p, DPAD_RIGHT);
    assert_eq!(delay(&menu), before + 7);
    tap(&mut menu, &p, DPAD_DOWN); // move picker stays tap-only
    menu.input(DPAD_RIGHT, &p);
    let revision = menu.recipe_revision;
    for _ in 0..120 {
        menu.input(DPAD_RIGHT, &p);
    }
    assert_eq!(menu.recipe_revision, revision);
}
/// Regression test for `recipe_editor_file_waits_become_delays_without_losing_frames`.
#[test]
fn recipe_editor_file_waits_become_delays_without_losing_frames() {
    let mut source = Sequence::default();
    source.len = 4;
    source.actions[..4].copy_from_slice(&[
        Action::Wait(255),
        Action::Wait(4),
        key(MAPPER_A, 0., 0., 1),
        Action::Wait(2),
    ]);
    let recipe = Recipe::new(source);
    assert!(!PRESETS.iter().any(|(_, a)| matches!(a, Action::Wait(_))));
    assert!(!recipe
        .sequence
        .actions()
        .iter()
        .any(|a| matches!(a, Action::Wait(_))));
    assert_eq!(recipe.compile().unwrap().actions(), source.actions());
    let mut editor = RecipeEditor::default();
    editor.ensure((0, 0), source);
    editor.navigate((0, 0), true);
    let lines = editor.lines((0, 0)).join("\n");
    assert!(!lines.contains("Slot "));
    assert!(lines.contains("Delay:"));
}
/// Regression test for `recipe_editor_changes_next_attempt_delay_without_touching_file_or_old_snapshot`.
#[test]
fn recipe_editor_changes_next_attempt_delay_without_touching_file_or_old_snapshot() {
    let p = presets();
    let mut menu = MenuSelection::new(&p);
    menu.show();
    let old = menu
        .settings(&p, AssistSettings::default())
        .sequence_overrides[0]
        .unwrap();
    tap(&mut menu, &p, MENU_STICK_CLICK);
    assert!(menu.recipe_editing);
    for _ in 0..4 {
        tap(&mut menu, &p, DPAD_RIGHT);
    }
    let next = menu
        .settings(&p, AssistSettings::default())
        .sequence_overrides[0]
        .unwrap();
    assert_eq!(next.start_delay, 4);
    assert_eq!(old.start_delay, 0);
    assert_eq!(p.lists[0].options[0].sequence.start_delay, 0);
    assert_eq!(menu.recipe_revision, 4);
    assert!(!menu.ledge_held(MAPPER_L));
    tap(&mut menu, &p, MENU_STICK_CLICK);
    assert!(!menu.recipe_editing);
    assert!(menu.recipe_modified(PresetSlot::Neutral));
}
/// Regression test for `recipe_editor_custom_slots_swap_actions_and_insert_exact_delays`.
#[test]
fn recipe_editor_custom_slots_swap_actions_and_insert_exact_delays() {
    let p = presets();
    let mut menu = MenuSelection::new(&p);
    menu.show();
    menu.choices[0] = p.lists[0].options.len();
    tap(&mut menu, &p, MENU_STICK_CLICK);
    tap(&mut menu, &p, DPAD_DOWN); // slot 1 type
    for _ in 0..1 {
        tap(&mut menu, &p, DPAD_RIGHT);
    } // short hop
    let k = (0, menu.choices[0]);
    {
        let e = menu.editor.as_mut().unwrap();
        e.cursor = fields(&e.recipes[&k])
            .iter()
            .position(|f| matches!(f, Field::Kind(1)))
            .unwrap();
    }
    for _ in 0..3 {
        tap(&mut menu, &p, DPAD_RIGHT);
    } // neutral A
    {
        let e = menu.editor.as_mut().unwrap();
        e.cursor = fields(&e.recipes[&k])
            .iter()
            .position(|f| matches!(f, Field::Before(1)))
            .unwrap();
    }
    for _ in 0..2 {
        tap(&mut menu, &p, DPAD_RIGHT);
    } // 2 frames before A
    let s = menu
        .settings(&p, AssistSettings::default())
        .sequence_overrides[0]
        .unwrap();
    assert_eq!(
        s.actions(),
        &[
            key(MAPPER_X, 0., 0., 1),
            Action::Wait(2),
            key(MAPPER_A, 0., 0., 4)
        ]
    );
}
/// Regression test for `recipe_editor_movement_bounds_and_expansion_limit_keep_valid_snapshots`.
#[test]
fn recipe_editor_movement_bounds_and_expansion_limit_keep_valid_snapshots() {
    let mut e = RecipeEditor::default();
    let mut source = RecipeEditor::custom();
    source.len = 1;
    source.actions[0] = Action::Approach {
        airborne: false,
        strength: 0.8,
        frames: 30,
        gap: 10.,
    };
    e.ensure((0, 0), source);
    e.cursor = 3;
    e.adjust((0, 0), true);
    e.cursor = 4;
    e.adjust((0, 0), false);
    e.cursor = 5;
    e.adjust((0, 0), false);
    assert_eq!(
        e.compiled((0, 0)).unwrap().actions[0],
        Action::Approach {
            airborne: false,
            strength: 0.81,
            frames: 29,
            gap: 9.
        }
    );
    e.cursor = 3;
    for _ in 0..50 {
        e.adjust((0, 0), true);
    }
    assert!(matches!(
        e.compiled((0, 0)).unwrap().actions[0],
        Action::Approach { strength: 0.95, .. }
    ));
    let mut full = Sequence::default();
    full.len = MAX_ACTIONS as u8;
    full.actions.fill(key(MAPPER_A, 0., 0., 1));
    e.ensure((0, 1), full);
    e.cursor = 2;
    assert!(!e.adjust((0, 1), true));
    assert!(e.limited);
    assert_eq!(e.compiled((0, 1)).unwrap(), full);
}
/// Regression test for `menu_edited_a_hold_continues_after_jab_into_the_next_action`.
#[test]
fn menu_edited_a_hold_continues_after_jab_into_the_next_action() {
    use crate::{
        fighter_profiles::presets::PresetSlot,
        gameplay::{GameplayContext, Opponent},
        ledge_trap::LedgeTrap,
        native_input::{ButtonTrace, MapperButtonOverlay},
        settings::AssistSettings,
    };
    let mut source = Sequence::default();
    source.len = 2;
    source.actions[0] = key(MAPPER_A, 0., 0., 4);
    source.actions[1] = key(MAPPER_B, 0., 0., 4);
    let mut editor = RecipeEditor::default();
    let recipe = (PresetSlot::X as usize, 0);
    editor.ensure(recipe, source);
    editor.cursor = fields(&editor.recipes[&recipe])
        .iter()
        .position(|f| matches!(f, Field::Duration(0)))
        .unwrap();
    for _ in 4..60 {
        assert!(editor.adjust(recipe, true));
    }
    let mut settings = AssistSettings::default();
    settings.enabled = true;
    settings.sequence_overrides[PresetSlot::X as usize] = editor.compiled(recipe);
    let mut trap = LedgeTrap::default();
    trap.request_preview(PresetSlot::X);
    let mut overlay = MapperButtonOverlay::default();
    for frame in 0..=64 {
        // A normal jab starts, finishes, then returns to idle. No action
        // readiness or continued physical trigger is needed for the hold.
        let (status, motion, native_frame) = if frame == 0 {
            (0, "wait", 0.)
        } else if frame < 15 {
            (
                crate::native_states::status::ATTACK,
                "attack_11",
                (frame - 1) as f32,
            )
        } else {
            (0, "wait", (frame - 15) as f32)
        };
        let out = trap.sample(
            settings,
            frame,
            GameplayContext {
                held: true,
                gameplay: true,
                facing: 1.,
                motion_rate: 1.,
                status,
                motion_kind: crate::timing::motion_hash(motion),
                motion_frame: native_frame,
                ..GameplayContext::default()
            },
            Some(Opponent {
                id: 2,
                ..Opponent::default()
            }),
        );
        assert_eq!(trap.requests_attack_hold(), frame < 60);
        let mapped = overlay.apply_raw(ButtonTrace::default(), out.requested, out.owned);
        assert_eq!(
            mapped.held & MAPPER_A,
            if frame < 60 { MAPPER_A } else { 0 },
            "frame {frame}"
        );
        assert_eq!(
            mapped.held & MAPPER_B,
            if (60..64).contains(&frame) {
                MAPPER_B
            } else {
                0
            },
            "frame {frame}"
        );
        assert_eq!(
            mapped.pressed & MAPPER_A,
            if frame == 0 { MAPPER_A } else { 0 }
        );
    }
    assert!(trap.manual_after_response());
}
