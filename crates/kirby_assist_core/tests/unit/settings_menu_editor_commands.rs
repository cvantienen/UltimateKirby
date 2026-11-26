use super::*;
/// `KEY` is a configuration value used by the Smash runtime.
const KEY: (usize, usize) = (5, 0);
/// Returns the `editor` result.
fn editor() -> RecipeEditor {
    let source=crate::sequence_config::Catalog::parse("[kirby.x.test]\nstart_delay=2\nwhile_held=true\nactions=hold:a:7,wait:3,hold:b:9,wait:2,hold:r:4").unwrap().entries()[0].sequence;
    let mut e = RecipeEditor::default();
    e.ensure(KEY, source);
    e
}
/// Returns an optional `tap` value.
fn tap(e: &mut RecipeEditor, button: u32) -> Option<(bool, bool)> {
    e.command_input(KEY, 0, 0);
    e.command_input(KEY, button, button)
}
/// Regression test for `delete_middle_preserves_following_parameters_and_undo_restores_exact_sequence`.
#[test]
fn delete_middle_preserves_following_parameters_and_undo_restores_exact_sequence() {
    let mut e = editor();
    let before = e.compiled(KEY).unwrap();
    assert!(e.structural(KEY, 1, 6));
    let after = e.compiled(KEY).unwrap();
    assert_eq!(
        after.actions(),
        &[
            before.actions()[0],
            before.actions()[3],
            before.actions()[4]
        ]
    );
    assert!(after.while_held);
    assert_eq!(after.start_delay, 2);
    assert!(e.undo_edit(KEY));
    assert_eq!(e.compiled(KEY).unwrap(), before);
}
/// Regression test for `moves_duplicate_and_inserts_carry_delays_kinds_and_values`.
#[test]
fn moves_duplicate_and_inserts_carry_delays_kinds_and_values() {
    let mut e = editor();
    let before = e.recipes[&KEY].clone();
    assert!(e.structural(KEY, 1, 4));
    let moved = &e.recipes[&KEY];
    assert_eq!(moved.sequence.actions[0], before.sequence.actions[1]);
    assert_eq!(moved.before[0], 3);
    assert_eq!(moved.kinds[0], before.kinds[1]);
    assert!(e.structural(KEY, 0, 3));
    let r = &e.recipes[&KEY];
    assert_eq!(r.sequence.actions[0], r.sequence.actions[1]);
    assert_eq!(r.before[1], 3);
    assert!(e.structural(KEY, 1, 1));
    let r = &e.recipes[&KEY];
    assert_eq!(r.sequence.actions[1], Action::Empty);
    assert_eq!(r.before[1], 0);
    assert_eq!(r.before[2], 3);
}
/// Regression test for `full_capacity_or_expanded_limit_refuses_without_losing_original`.
#[test]
fn full_capacity_or_expanded_limit_refuses_without_losing_original() {
    let mut e = editor();
    let r = e.recipes.get_mut(&KEY).unwrap();
    r.sequence.len = 32;
    r.sequence.actions.fill(key(MAPPER_A, 0., 0., 4));
    r.before.fill(0);
    let before = e.compiled(KEY).unwrap();
    assert!(!e.structural(KEY, 1, 3));
    assert_eq!(e.compiled(KEY).unwrap(), before);
    let r = e.recipes.get_mut(&KEY).unwrap();
    r.sequence.len = 16;
    r.before.fill(0);
    for i in 0..16 {
        r.before[i] = 1;
    }
    let before = e.compiled(KEY).unwrap();
    assert!(!e.structural(KEY, 1, 3));
    assert_eq!(e.compiled(KEY).unwrap(), before);
}
/// Regression test for `picker_is_complete_and_cancel_or_apply_same_action_preserves_hold_and_delay`.
#[test]
fn picker_is_complete_and_cancel_or_apply_same_action_preserves_hold_and_delay() {
    let mut e = editor();
    e.focus_slot(KEY, 1);
    let before = e.compiled(KEY).unwrap();
    assert_eq!(tap(&mut e, MAPPER_A), Some((true, false)));
    assert!(matches!(e.modal, Some(Modal::Picker { slot: 1, .. })));
    assert_eq!(
        e.command_input(KEY, 0, 0),
        Some((false, false)),
        "idle modal does not request a rebuild"
    );
    tap(&mut e, MAPPER_A);
    assert_eq!(
        e.compiled(KEY).unwrap(),
        before,
        "choosing current action must not restore preset timing"
    );
    let all: Vec<_> = (0..8)
        .flat_map(|g| candidates(&e.recipes[&KEY], g))
        .collect();
    for kind in 0..PRESETS.len() {
        assert!(all.contains(&kind));
    }
    tap(&mut e, MAPPER_A);
    tap(&mut e, DPAD_RIGHT);
    tap(&mut e, MAPPER_B);
    assert_eq!(e.compiled(KEY).unwrap(), before);
}
#[test]
fn replacing_manual_hold_with_non_shield_input_uses_finite_duration() {
    let mut e = RecipeEditor::default();
    let mut source = Sequence::default();
    source.len = 1;
    source.actions[0] = Action::ShieldUntilManual;
    e.ensure(KEY, source);
    let kind = PRESETS
        .iter()
        .position(|(name, _)| *name == "Neutral A / Nair")
        .unwrap();
    assert_eq!(e.apply_kind(KEY, 0, kind), Some((true, true)));
    assert_eq!(
        e.compiled(KEY).unwrap().actions(),
        &[key(MAPPER_A, 0., 0., 4)]
    );
    let kind = PRESETS
        .iter()
        .position(|(name, _)| *name == "Shield / air dodge")
        .unwrap();
    e.recipes.get_mut(&KEY).unwrap().sequence.actions[0] = Action::ShieldUntilManual;
    assert_eq!(e.apply_kind(KEY, 0, kind), Some((true, true)));
    assert_eq!(
        e.compiled(KEY).unwrap().actions(),
        &[key(MAPPER_ZR, 0., 0., 0)]
    );
}

/// Regression test for `custom_saved_inputs_remain_available_after_reorder_and_replace`.
#[test]
fn custom_saved_inputs_remain_available_after_reorder_and_replace() {
    let mut e = editor();
    let mut source = e.compiled(KEY).unwrap();
    source.actions[0] = key(MAPPER_A | MAPPER_R, 0.37, -0.21, 13);
    e = RecipeEditor::default();
    e.ensure(KEY, source);
    assert!(candidates(&e.recipes[&KEY], 8).len() >= 1);
    assert!(e.structural(KEY, 0, 5));
    let r = &e.recipes[&KEY];
    let saved = candidates(r, 8)[0];
    assert_eq!(r.imported[saved - PRESETS.len()], source.actions[0]);
}
/// Regression test for `step_shortcuts_skip_settings_and_numeric_edits_are_undoable`.
#[test]
fn step_shortcuts_skip_settings_and_numeric_edits_are_undoable() {
    let mut e = editor();
    e.focus_slot(KEY, 0);
    tap(&mut e, MAPPER_ZR);
    assert_eq!(e.selected_slot(KEY), Some(1));
    tap(&mut e, MAPPER_ZL);
    assert_eq!(e.selected_slot(KEY), Some(0));
    e.cursor = 0;
    let old = e.compiled(KEY).unwrap();
    assert!(e.adjust(KEY, true));
    assert!(e.undo_edit(KEY));
    assert_eq!(e.compiled(KEY).unwrap(), old);
}
/// Regression test for `shoulder_reorder_is_direct_and_repeats_only_on_navigation_updates`.
#[test]
fn shoulder_reorder_is_direct_and_repeats_only_on_navigation_updates() {
    let mut e = editor();
    assert_eq!(
        e.command_input(KEY, MAPPER_R | DPAD_DOWN, DPAD_DOWN),
        Some((false, false)),
        "sequence-level settings must not reorder a step"
    );
    e.focus_slot(KEY, 1);
    let before = e.recipes[&KEY].clone();
    assert_eq!(
        e.command_input(KEY, MAPPER_R, MAPPER_R),
        Some((false, false))
    );
    assert_eq!(
        e.command_input(KEY, MAPPER_R | DPAD_UP, DPAD_UP),
        Some((true, true))
    );
    assert_eq!(e.selected_slot(KEY), Some(0));
    assert_eq!(
        e.recipes[&KEY].sequence.actions[0],
        before.sequence.actions[1]
    );
    assert_eq!(e.recipes[&KEY].before[0], 3);
    for _ in 0..12 {
        assert_eq!(
            e.command_input(KEY, MAPPER_R | DPAD_UP, 0),
            Some((false, false))
        );
    }
    assert!(e.undo_edit(KEY));
    assert_eq!(e.compiled(KEY).unwrap(), before.compile().unwrap());
}
/// Regression test for `insertion_and_deleting_final_step_preserve_other_steps_and_duration`.
#[test]
fn insertion_and_deleting_final_step_preserve_other_steps_and_duration() {
    let mut e = editor();
    let before = e.compiled(KEY).unwrap();
    assert!(e.structural(KEY, 1, 2));
    assert_eq!(
        e.compiled(KEY).unwrap(),
        before,
        "an empty inserted slot adds no time"
    );
    assert_eq!(e.recipes[&KEY].before[3], 2);
    let len = usize::from(e.recipes[&KEY].sequence.len);
    assert!(e.structural(KEY, len - 1, 6));
    assert!(e.undo_edit(KEY));
    assert_eq!(e.compiled(KEY).unwrap(), before);
}
/// Regression test for `action_edits_and_undo_only_mark_affected_steps`.
#[test]
fn action_edits_and_undo_only_mark_affected_steps() {
    let mut e = editor();
    let mut next = e.recipes[&KEY].clone();
    next.sequence.actions[0] = key(MAPPER_A, 0., 0., 14);
    assert!(e.commit_command(KEY, next, 0, "Changed hold"));
    assert_eq!(e.recipes[&KEY].edited, 1);
    assert!(e.undo_edit(KEY));
    assert_eq!(e.recipes[&KEY].edited, 1);
}
/// Copies the template uses unsaved source timings and preserves destination activation.
#[test]
fn copy_template_uses_unsaved_source_timings_and_preserves_destination_activation() {
    let mut e = editor();
    let target_before = e.compiled(KEY).unwrap();
    let source_key = (0, 0);
    let mut source = target_before;
    source.start_delay = 9;
    source.timeout = 900;
    source.spatial_timeout = 70;
    source.while_held = false;
    source.actions[0] = key(MAPPER_A, 0., -1., 17);
    e.ensure(source_key, source);
    // An in-memory edit must take precedence over file fallback.
    e.recipes.get_mut(&source_key).unwrap().before[1] = 13;
    let source_before = e.compiled(source_key).unwrap();
    assert!(e.apply_template(KEY, source_key, target_before));
    let mut expected = source_before;
    expected.while_held = true;
    assert_eq!(e.compiled(KEY).unwrap(), expected);
    assert_eq!(e.compiled(source_key).unwrap(), source_before);
    assert!(e.undo_edit(KEY));
    assert_eq!(e.compiled(KEY).unwrap(), target_before);
    assert!(!e.apply_template(KEY, KEY, source));
}
/// Regression test for `button_to_ledge_copy_cannot_enable_looping_and_releases_source_list`.
#[test]
fn button_to_ledge_copy_cannot_enable_looping_and_releases_source_list() {
    let mut e = editor();
    let source = e.compiled(KEY).unwrap();
    let destination = (1, 0);
    e.ensure(destination, RecipeEditor::custom());
    e.copy_sources.push(CopySource {
        slot: crate::fighter_profiles::presets::PresetSlot::X,
        index: 0,
        name: "Source".into(),
    });
    assert!(e.apply_template(destination, KEY, source));
    let target = e.compiled(destination).unwrap();
    assert_eq!(target.actions(), source.actions());
    assert!(!target.while_held);
    assert!(!e.recipes[&destination].button_recipe);
    assert_eq!(e.copy_sources.capacity(), 0);
    assert!(e.modal.is_none());
}
/// Regression test for `cancelling_copy_drops_temporary_names_without_changing_recipes`.
#[test]
fn cancelling_copy_drops_temporary_names_without_changing_recipes() {
    let mut e = editor();
    let before = e.compiled(KEY).unwrap();
    e.modal = Some(Modal::Copy { group: 5, item: 0 });
    e.copy_sources.push(CopySource {
        slot: crate::fighter_profiles::presets::PresetSlot::X,
        index: 1,
        name: "Saved source".into(),
    });
    assert_eq!(e.command_input(KEY, 0, 0), Some((false, false)));
    assert_eq!(tap(&mut e, MAPPER_B), Some((true, false)));
    assert_eq!(e.copy_sources.capacity(), 0);
    assert_eq!(e.compiled(KEY).unwrap(), before);
}
