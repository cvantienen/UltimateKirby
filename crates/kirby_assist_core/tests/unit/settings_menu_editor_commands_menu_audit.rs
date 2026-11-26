use super::*;
/// `SRC` is a configuration value used by the Smash runtime.
const SRC: (usize, usize) = (5, 0);
/// `DST` is a configuration value used by the Smash runtime.
const DST: (usize, usize) = (0, 0);
/// Returns the `recipe` result.
fn recipe(a: &str) -> Sequence {
    crate::sequence_config::Catalog::parse(&format!(
        "[bowser.attack.test]\nstart_delay=0\nactions={a}\n"
    ))
    .unwrap()
    .entries()[0]
        .sequence
}
/// Returns an optional `tap` value.
fn tap(e: &mut RecipeEditor, key: (usize, usize), b: u32) -> Option<(bool, bool)> {
    e.command_input(key, 0, 0);
    e.command_input(key, b, b)
}
/// Regression test for `confirm_on_a_held_arrow_toggles_the_selected_button_without_moving_the_cursor`.
#[test]
fn confirm_on_a_held_arrow_toggles_the_selected_button_without_moving_the_cursor() {
    let mut e = RecipeEditor::default();
    e.ensure(DST, recipe("input:a:0:0:4"));
    e.cursor = fields(&e.recipes[&DST])
        .iter()
        .position(|f| matches!(f, Field::Buttons(0)))
        .unwrap();
    tap(&mut e, DST, MAPPER_A);
    tap(&mut e, DST, DPAD_DOWN);
    let Some(Modal::Buttons { item, .. }) = e.modal else {
        panic!()
    };
    let expected =
        input_buttons(e.recipes[&DST].sequence.actions[0]).unwrap() ^ INPUT_BUTTONS[item].0;
    assert_eq!(
        e.command_input(DST, DPAD_DOWN | MAPPER_A, MAPPER_A),
        Some((true, true))
    );
    assert!(matches!(e.modal,Some(Modal::Buttons{item:i,..})if i==item));
    assert_eq!(
        input_buttons(e.recipes[&DST].sequence.actions[0]),
        Some(expected)
    );
}
/// Regression test for `step_clipboard_preserves_imported_axes_references_duration_and_delay_across_recipes`.
#[test]
fn step_clipboard_preserves_imported_axes_references_duration_and_delay_across_recipes() {
    let source = recipe("wait:7,controller_input:a+b:0.4:-0.2:-0.6:0.8:17:opponent:facing");
    let dest = recipe("controller_input:r:0:0:0:0:4:screen:screen");
    let mut e = RecipeEditor::default();
    e.ensure(SRC, source);
    assert!(e.copy_step(SRC, 0));
    e.ensure(DST, dest);
    assert!(e.step_clipboard.is_some());
    assert!(e.paste_step(DST, 0, true));
    let compiled = e.compiled(DST).unwrap();
    assert_eq!(&compiled.actions()[1..], source.actions());
    assert_eq!(compiled.actions()[0], dest.actions()[0]);
    assert_eq!(e.compiled(SRC), Some(source));
    assert!(e.undo_edit(DST));
    assert_eq!(e.compiled(DST), Some(dest));
}
/// Regression test for `paste_limits_and_empty_clipboard_cannot_overwrite_or_lose_the_existing_recipe`.
#[test]
fn paste_limits_and_empty_clipboard_cannot_overwrite_or_lose_the_existing_recipe() {
    let mut e = RecipeEditor::default();
    let mut full = recipe("input:a:0:0:1");
    full.len = MAX_ACTIONS as u8;
    let action = full.actions[0];
    full.actions.fill(action);
    e.ensure(DST, full);
    assert!(!e.paste_step(DST, 0, true));
    assert_eq!(e.compiled(DST), Some(full));
    assert!(e.copy_step(DST, 0));
    assert!(!e.paste_step(DST, 0, false));
    assert_eq!(e.compiled(DST), Some(full));
    let mut expanded = recipe("input:a:0:0:1");
    expanded.len = (MAX_ACTIONS - 1) as u8;
    let action = expanded.actions[0];
    expanded.actions[..MAX_ACTIONS - 1].fill(action);
    e.ensure((1, 0), expanded);
    e.step_clipboard = Some((
        Action::Input {
            buttons: MAPPER_B,
            stick: [0.; 2],
            frames: 1,
            relative: false,
        },
        1,
    ));
    assert!(
        !e.paste_step((1, 0), 0, false),
        "pasted delay counts toward expanded step limit"
    );
    assert_eq!(e.compiled((1, 0)), Some(expanded));
}
/// Regression test for `back_returns_to_parent_tools_but_done_and_save_close_all_popups`.
#[test]
fn back_returns_to_parent_tools_but_done_and_save_close_all_popups() {
    let mut e = RecipeEditor::default();
    e.ensure(DST, recipe("input:a:0:0:4"));
    tap(&mut e, DST, MAPPER_ZL | MAPPER_ZR);
    tap(&mut e, DST, MAPPER_A);
    assert!(matches!(e.modal, Some(Modal::Picker { .. })));
    tap(&mut e, DST, MAPPER_B);
    assert!(matches!(e.modal, Some(Modal::Tools { item: 0, .. })));
    tap(&mut e, DST, MAPPER_A);
    assert_eq!(
        tap(&mut e, DST, crate::settings_menu::MENU_STICK_CLICK),
        None
    );
    assert!(e.modal.is_none());
    assert!(e.return_modal.is_none());
    tap(&mut e, DST, MAPPER_ZL | MAPPER_ZR);
    assert_eq!(tap(&mut e, DST, MAPPER_Y), None);
    assert!(e.modal.is_none());
}
/// Copies the category navigation skips empty groups in both directions.
#[test]
fn copy_category_navigation_skips_empty_groups_in_both_directions() {
    let mut e = RecipeEditor::default();
    e.ensure(DST, recipe("input:a:0:0:4"));
    e.modal = Some(Modal::Copy { group: 0, item: 0 });
    e.set_copy_sources(vec![
        CopySource {
            slot: super::super::super::PresetSlot::Roll,
            index: 0,
            name: "Roll".into(),
        },
        CopySource {
            slot: super::super::super::PresetSlot::Y,
            index: 0,
            name: "Y".into(),
        },
    ]);
    assert!(matches!(e.modal, Some(Modal::Copy { group: 2, .. })));
    tap(&mut e, DST, DPAD_RIGHT);
    assert!(matches!(e.modal, Some(Modal::Copy { group: 6, .. })));
    tap(&mut e, DST, DPAD_RIGHT);
    assert!(matches!(e.modal, Some(Modal::Copy { group: 2, .. })));
    tap(&mut e, DST, DPAD_LEFT);
    assert!(matches!(e.modal, Some(Modal::Copy { group: 6, .. })));
}
/// Regression test for `identical_template_is_a_clean_noop_and_incompatible_copy_keeps_the_destination`.
#[test]
fn identical_template_is_a_clean_noop_and_incompatible_copy_keeps_the_destination() {
    let original = recipe("input:a:0:0:4");
    let mut e = RecipeEditor::default();
    e.ensure(DST, original);
    e.ensure(SRC, original);
    e.modal = Some(Modal::Copy { group: 5, item: 0 });
    assert!(!e.apply_template(DST, SRC, original));
    assert!(!e.edited(DST));
    assert!(e.undo.is_none());
    assert!(e.modal.is_none());
    let mut response = original;
    response.reaction = crate::reaction_rules::Rule::parse("auto_roll:30:12").unwrap();
    e.ensure((2, 0), response);
    let ledge = recipe("run_to_ledge:12:1");
    e.ensure((3, 0), ledge);
    assert!(!e.apply_template((2, 0), (3, 0), ledge));
    assert_eq!(e.compiled((2, 0)), Some(response));
    assert!(e.copy_step((3, 0), 0));
    assert!(!e.paste_step((2, 0), 0, true));
}
#[test]
fn onstage_direct_kind_cycle_skips_ledge_presets() {
    let mut source = recipe("face:opponent,walk:inward:4:50%");
    source.reaction = crate::reaction_rules::Rule::parse("auto_roll:30:12").unwrap();
    let mut e = RecipeEditor::default();
    e.ensure(DST, source);
    e.cursor = fields(&e.recipes[&DST])
        .iter()
        .position(|f| *f == Field::Kind(0))
        .unwrap();
    let count = PRESETS.len() + e.recipes[&DST].imported.len();
    for forward in [true, false] {
        for _ in 0..count {
            assert!(e.adjust(DST, forward));
            assert!(!e.recipes[&DST].sequence.actions[0].needs_ledge());
        }
    }
    assert_eq!(e.recipes[&DST].sequence.actions[1], source.actions()[1]);
}

/// Regression test for `onstage_picker_and_direction_adjustment_only_offer_actions_that_can_execute_there`.
#[test]
fn onstage_picker_and_direction_adjustment_only_offer_actions_that_can_execute_there() {
    let mut source = recipe("face:opponent");
    source.reaction = crate::reaction_rules::Rule::parse("auto_roll:30:12").unwrap();
    let mut e = RecipeEditor::default();
    e.ensure(DST, source);
    for group in 0..8 {
        for kind in candidates(&e.recipes[&DST], group) {
            assert!(!PRESETS[kind].1.needs_ledge());
        }
    }
    e.cursor = fields(&e.recipes[&DST])
        .iter()
        .position(|f| matches!(f, Field::Direction(0)))
        .unwrap();
    for _ in 0..16 {
        assert!(e.adjust(DST, true));
        assert!(!e.compiled(DST).unwrap().actions()[0].needs_ledge());
    }
}
