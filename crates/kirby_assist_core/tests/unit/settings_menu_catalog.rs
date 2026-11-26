use super::*;
use crate::{
    fighter_profiles::FighterProfiles,
    native_input::{MAPPER_A as A, MAPPER_B as B, MAPPER_X, MAPPER_Y as Y},
    sequence_config::Catalog as FileCatalog,
};
/// Returns the `presets` result.
fn presets() -> MenuPresets {
    let file=FileCatalog::parse("[yoshi.neutral.alpha]\nname=Alpha\nstart_delay=0\nactions=a\n[yoshi.neutral.beta]\nname=Beta\nstart_delay=0\nactions=b\n").unwrap();
    let mut profiles = FighterProfiles::default();
    profiles.add_sequence_choices(&file).unwrap();
    profiles.profile(5).unwrap().menu_options
}
/// Updates the `tap` state using the supplied inputs.
fn tap(menu: &mut MenuSelection, p: &MenuPresets, button: u32) {
    menu.input(button, p);
    menu.input(0, p);
}
/// Returns the `menu` result.
fn menu(p: &MenuPresets) -> MenuSelection {
    let mut m = MenuSelection::new(p);
    m.show();
    m.input(0, p);
    m
}
/// Regression test for `added_unsaved_recipes_can_be_switched_from_compact`.
#[test]
fn added_unsaved_recipes_can_be_switched_from_compact() {
    let p = presets();
    let mut m = menu(&p);
    m.row = MenuRow::Neutral;
    tap(&mut m, &p, MENU_STICK_CLICK);
    // Add using the same events as B -> Add -> A.
    let base_count = m.choice_position(&p, PresetSlot::Neutral).1;
    m.manage_input(
        &p,
        crate::native_input::MAPPER_B,
        crate::native_input::MAPPER_B,
    );
    m.catalog.as_mut().unwrap().mode = Mode::Actions(1);
    m.manage_input(
        &p,
        crate::native_input::MAPPER_A,
        crate::native_input::MAPPER_A,
    );
    let added = m.choices[PresetSlot::Neutral as usize];
    assert!(m.choice(&p, PresetSlot::Neutral, added).unwrap().allow_new);
    m.show_quick();
    let count = m.choice_position(&p, PresetSlot::Neutral).1;
    // The untouched blank template is hidden; the explicit new draft is visible.
    assert_eq!(count, base_count);
    let next = m.cycle_quick_choice(&p, PresetSlot::Neutral, true);
    assert_ne!(next, added);
    m.choices[PresetSlot::Neutral as usize] = next;
    assert_eq!(m.cycle_quick_choice(&p, PresetSlot::Neutral, false), added);
}
/// Regression test for `diagnostic_add_and_duplicate_keep_other_option_indices_and_edits_stable`.
#[test]
fn diagnostic_add_and_duplicate_keep_other_option_indices_and_edits_stable() {
    let p = presets();
    let mut m = menu(&p);
    tap(&mut m, &p, B);
    tap(&mut m, &p, DPAD_DOWN);
    m.input(A, &p);
    assert!(m.recipe_editing);
    assert!(!m.managing());
    assert_eq!(m.choices[0], 3);
    assert_ne!(m.input(A, &p) & A, 0); // held Select cannot become a gameplay attack
    m.input(0, &p);
    tap(&mut m, &p, DPAD_DOWN);
    tap(&mut m, &p, DPAD_RIGHT);
    let first_new = m.choices[0];
    let configured = m.settings(&p, AssistSettings::default()).sequence_overrides[0].unwrap();
    assert!(matches!(
        configured.actions(),
        [crate::sequence_config::Action::Input {
            buttons: MAPPER_X,
            ..
        }]
    ));
    m.manage_input(&p, B, B);
    tap(&mut m, &p, DPAD_DOWN);
    tap(&mut m, &p, DPAD_DOWN);
    tap(&mut m, &p, A);
    assert_eq!(m.choices[0], 4);
    assert_eq!(
        m.settings(&p, AssistSettings::default()).sequence_overrides[0]
            .unwrap()
            .actions(),
        configured.actions()
    );
    assert_ne!(
        m.choice(&p, PresetSlot::Neutral, first_new).unwrap().id,
        m.choice(&p, PresetSlot::Neutral, 4).unwrap().id
    );
    assert_eq!(m.choice(&p, PresetSlot::Neutral, 0).unwrap().id, "alpha");
    assert_eq!(m.choice(&p, PresetSlot::Neutral, 1).unwrap().id, "beta");
}
/// Regression test for `diagnostic_rename_changes_label_not_id_and_requires_explicit_file_save`.
#[test]
fn diagnostic_rename_changes_label_not_id_and_requires_explicit_file_save() {
    let p = presets();
    let mut m = menu(&p);
    tap(&mut m, &p, B);
    tap(&mut m, &p, A);
    tap(&mut m, &p, DPAD_DOWN);
    tap(&mut m, &p, Y);
    assert!(m.recipe_editing);
    assert_eq!(m.choice(&p, PresetSlot::Neutral, 0).unwrap().name, "Blpha");
    assert_eq!(m.choice(&p, PresetSlot::Neutral, 0).unwrap().id, "alpha");
    assert!(!m.save_requested);
    m.input(Y, &p);
    assert!(m.save_requested);
}
/// Regression test for `diagnostic_delete_needs_confirmation_and_only_hides_after_successful_file_write`.
#[test]
fn diagnostic_delete_needs_confirmation_and_only_hides_after_successful_file_write() {
    let p = presets();
    let mut m = menu(&p);
    tap(&mut m, &p, B);
    for _ in 0..3 {
        tap(&mut m, &p, DPAD_DOWN);
    }
    tap(&mut m, &p, A);
    assert!(!m.delete_requested);
    assert!(m.managing());
    m.input(A, &p);
    assert!(m.delete_requested);
    assert!(m.choice(&p, PresetSlot::Neutral, 0).is_some());
    m.complete_file_save(&p, PresetSlot::Neutral, 0, true);
    assert!(m.choice(&p, PresetSlot::Neutral, 0).is_none());
    assert_eq!(m.choice(&p, PresetSlot::Neutral, 1).unwrap().id, "beta");
    assert_ne!(m.choices[0], 0);
    assert_eq!(m.choice_position(&p, PresetSlot::Neutral).1, 2);
}
/// Regression test for `closing_or_match_reset_exits_manager_without_deleting_or_renaming`.
#[test]
fn closing_or_match_reset_exits_manager_without_deleting_or_renaming() {
    let p = presets();
    let mut m = menu(&p);
    tap(&mut m, &p, B);
    assert!(m.managing());
    m.reset_inputs();
    assert!(!m.managing());
    assert!(!m.delete_requested);
    assert_eq!(m.choice(&p, PresetSlot::Neutral, 0).unwrap().name, "Alpha");
}
/// Regression test for `empty_ness_and_falcon_catalogs_can_create_edit_preview_and_queue_a_recipe`.
#[test]
fn empty_ness_and_falcon_catalogs_can_create_edit_preview_and_queue_a_recipe() {
    use crate::{
        gameplay::{GameplayContext, Opponent},
        ledge_trap::LedgeTrap,
    };
    for kind in [10, 11] {
        let profiles = crate::fighter_profiles::FighterProfiles::default();
        let profile = profiles.profile(kind).unwrap();
        let presets = &profile.menu_options;
        assert!(presets.list(PresetSlot::Neutral).options.is_empty());
        let base = profile.apply(AssistSettings {
            enabled: true,
            ..AssistSettings::default()
        });
        assert!(base.enabled);
        let mut m = menu(presets);
        m.auto_ledge = false;
        tap(&mut m, presets, B);
        tap(&mut m, presets, DPAD_DOWN);
        tap(&mut m, presets, A); // add a distinct editable draft
        tap(&mut m, presets, DPAD_DOWN);
        tap(&mut m, presets, DPAD_RIGHT); // Empty -> Short hop
        let index = m.choices[0];
        let choice = m.choice(presets, PresetSlot::Neutral, index).unwrap();
        assert!(choice.allow_new);
        assert!(choice.name.starts_with("New option"));
        assert!(
            !m.ledge_mode(),
            "availability does not enable automatic mode"
        );
        tap(&mut m, presets, crate::native_input::MAPPER_X); // explicit Test
        assert_eq!(m.preview_slot, Some(PresetSlot::Neutral));
        let settings = m.settings(presets, base);
        assert!(
            settings.enabled,
            "explicit test cannot depend on existing file rows"
        );
        let mut trap = LedgeTrap::default();
        trap.request_preview(PresetSlot::Neutral);
        let out = trap.sample(
            settings,
            0,
            GameplayContext {
                gameplay: true,
                held: true,
                facing: 1.,
                motion_rate: 1.,
                motion_kind: crate::timing::motion_hash("wait"),
                ..GameplayContext::default()
            },
            Some(Opponent {
                id: 2,
                ..Opponent::default()
            }),
        );
        assert_ne!(out.requested & crate::native_input::MAPPER_X, 0);
        m.preview_slot = None;
        m.input(crate::native_input::MAPPER_Y, presets);
        assert!(m.save_requested);
        assert!(!m.sequence_enabled(false));
    }
}

/// Regression test for `saved_custom_can_be_renamed_saved_again_and_deleted_without_restart`.
#[test]
fn saved_custom_can_be_renamed_saved_again_and_deleted_without_restart() {
    use crate::sequence_config::persist;
    let source = "[yoshi.neutral.alpha]\nname=Alpha\nstart_delay=0\nactions=a\n[yoshi.neutral.beta]\nname=Beta\nstart_delay=0\nactions=b\n";
    let p = presets();
    let mut m = menu(&p);
    let slot = PresetSlot::Neutral;
    let index = p.list(slot).options.len(); // built-in Custom template
    m.choices[slot as usize] = index;
    tap(&mut m, &p, MENU_STICK_CLICK);
    tap(&mut m, &p, DPAD_DOWN);
    tap(&mut m, &p, DPAD_RIGHT); // Short hop
    let sequence = m.settings(&p, AssistSettings::default()).sequence_overrides[0].unwrap();
    let added =
        persist::update_named(source, 5, slot, "menu_custom", sequence, true, None).unwrap();
    m.complete_file_save(&p, slot, index, false);
    assert!(!m.choice(&p, slot, index).unwrap().allow_new);
    m.manage_input(&p, B, B);
    tap(&mut m, &p, A); // rename
    tap(&mut m, &p, DPAD_DOWN); // change first letter
    tap(&mut m, &p, Y); // accept name, no automatic disk write
    assert!(!m.save_requested);
    let choice = m.choice(&p, slot, index).unwrap();
    assert_eq!(choice.id, "menu_custom");
    let renamed = persist::update_named(
        &added,
        5,
        slot,
        choice.id,
        sequence,
        choice.allow_new,
        m.name_override(slot, index),
    )
    .unwrap();
    let parsed = FileCatalog::parse(&renamed).unwrap();
    let entry = parsed
        .entries()
        .iter()
        .find(|e| e.choice == "menu_custom")
        .unwrap();
    assert_eq!(entry.sequence.actions(), sequence.actions());
    assert_eq!(entry.name.as_deref(), m.name_override(slot, index));
    m.complete_file_save(&p, slot, index, false);
    m.manage_input(&p, B, B);
    for _ in 0..3 {
        tap(&mut m, &p, DPAD_DOWN);
    }
    tap(&mut m, &p, A); // ask to delete
    m.input(A, &p);
    assert!(
        m.delete_requested,
        "saved Custom is a real recipe, not an undeletable template"
    );
    let deleted = persist::delete(&renamed, 5, slot, "menu_custom", false).unwrap();
    assert!(
        deleted.starts_with(source),
        "unrelated choices remain byte-for-byte intact"
    );
    assert!(deleted[source.len()..].trim().is_empty());
    m.complete_file_save(&p, slot, index, true);
    let blank = m.choice(&p, slot, index).unwrap();
    assert!(blank.allow_new);
    assert_eq!(blank.id, "menu_custom");
    assert!(m.settings(&p, AssistSettings::default()).sequence_overrides[0].is_none());
}

/// Regression test for `saving_one_recipe_is_not_stale_after_renaming_another`.
#[test]
fn saving_one_recipe_is_not_stale_after_renaming_another() {
    let p = presets();
    let mut m = menu(&p);
    let slot = PresetSlot::Neutral;
    let sequence = m.choice(&p, slot, 0).unwrap().sequence;
    let revision = m.recipe_revision;
    m.choices[0] = 1;
    tap(&mut m, &p, B);
    tap(&mut m, &p, A);
    tap(&mut m, &p, DPAD_DOWN);
    tap(&mut m, &p, Y);
    assert!(m.recipe_revision > revision);
    assert!(m.saved_recipe_matches(&p, slot, 0, sequence, None));
    let mut older = sequence;
    older.start_delay += 1;
    assert!(!m.saved_recipe_matches(&p, slot, 0, older, None));
}

/// Regression test for `deleting_only_saved_custom_restores_an_empty_reusable_template`.
#[test]
fn deleting_only_saved_custom_restores_an_empty_reusable_template() {
    let p = MenuPresets::EMPTY;
    let mut m = menu(&p);
    let slot = PresetSlot::Neutral;
    tap(&mut m, &p, MENU_STICK_CLICK);
    tap(&mut m, &p, DPAD_DOWN);
    tap(&mut m, &p, DPAD_RIGHT);
    assert!(m.settings(&p, AssistSettings::default()).sequence_overrides[0].is_some());
    m.complete_file_save(&p, slot, 0, false);
    assert!(!m.choice(&p, slot, 0).unwrap().allow_new);
    m.complete_file_save(&p, slot, 0, true);
    assert_eq!(m.choices[0], 0);
    assert!(m.choice(&p, slot, 0).unwrap().allow_new);
    assert!(m.settings(&p, AssistSettings::default()).sequence_overrides[0].is_none());
    tap(&mut m, &p, MENU_STICK_CLICK);
    assert!(m.recipe_editing);
}
