use super::*;
use crate::button_bindings::Builtin;

fn spacing_selection() -> (MenuSelection, MenuPresets) {
    let presets = MenuPresets::default();
    let mut selection = MenuSelection {
        menu_hidden: true,
        ..MenuSelection::default()
    };
    selection.choices[PresetSlot::L3 as usize] = presets.list(PresetSlot::L3).options.len()
        + Builtin::AutoSpacing.visible_template_offset().unwrap();
    selection.capture_binding_input(PresetSlot::L3.button(), &presets);
    (selection, presets)
}

#[test]
fn auto_spacing_choice_cycles_and_saved_default_reloads_by_identity() {
    use crate::{
        fighter_profiles::FighterProfiles,
        sequence_config::{persist, Catalog, Sequence},
    };
    let mut profiles = FighterProfiles::default();
    profiles
        .add_sequence_choices(&Catalog::parse("").unwrap())
        .unwrap();
    let presets = profiles
        .profile(crate::fighter_kinds::KIRBY)
        .unwrap()
        .menu_options;
    let mut selection = MenuSelection::new(&presets);
    for slot in PresetSlot::BUTTONS {
        let base = presets.list(slot).options.len();
        let spacing = base + Builtin::AutoSpacing.visible_template_offset().unwrap();
        let choices = selection.visible_choices(&presets, slot);
        assert!(choices.contains(&spacing));
        assert_eq!(
            selection.choice_end(&presets, slot),
            base + slot.templates()
        );
        assert_eq!(
            selection.choice(&presets, slot, spacing).unwrap().id,
            Builtin::AutoSpacing.choice_id()
        );
        selection.choices[slot as usize] = spacing;
        assert_eq!(selection.cycle_choice(&presets, slot, true), spacing + 1);
        selection.quick_menu = true;
        let (position, count) = selection.choice_position(&presets, slot);
        assert_eq!(position + 2, count);
        selection.quick_menu = false;
    }
    let saved = persist::update(
        "",
        crate::fighter_kinds::KIRBY,
        PresetSlot::Y,
        "saved_spacing",
        Sequence {
            builtin: Some(Builtin::AutoSpacing),
            ..Sequence::default()
        },
        true,
    )
    .unwrap();
    let catalog = Catalog::parse(&saved).unwrap();
    profiles.add_sequence_choices(&catalog).unwrap();
    let startup = profiles
        .profile(crate::fighter_kinds::KIRBY)
        .unwrap()
        .menu_options;
    let mut startup_selection = MenuSelection::new(&startup);
    selection.load_saved_recipes(&presets, &catalog, crate::fighter_kinds::KIRBY);
    for (menu, p) in [
        (&mut startup_selection, &startup),
        (&mut selection, &presets),
    ] {
        assert_eq!(
            menu.choice(p, PresetSlot::Y, menu.choices[PresetSlot::Y as usize])
                .unwrap()
                .id,
            "saved_spacing"
        );
        menu.menu_hidden = true;
        menu.capture_binding_input(MAPPER_Y, p);
        assert!(menu.auto_spacing_held());
    }
}

#[test]
fn defensive_holds_preempt_committed_bindings_and_release_cleanly() {
    use crate::action_owner::Owner;
    let (mut selection, presets) = spacing_selection();
    selection.choices[PresetSlot::X as usize] = presets.list(PresetSlot::X).options.len()
        + Builtin::Counter.visible_template_offset().unwrap();
    selection.block_binding_starts(true);
    let l3 = PresetSlot::L3.button();
    selection.capture_binding_input(MAPPER_X | l3 | MAPPER_Y, &presets);
    assert_eq!(selection.defensive_owner(), Some(Owner::Counter));
    assert!(selection.reserves_automatic_modes());
    assert!(!selection.auto_spacing_held());
    assert_eq!(selection.builtin_consumed & (MAPPER_X | l3), MAPPER_X | l3);
    selection.capture_binding_input(MAPPER_X, &presets);
    assert_eq!(selection.defensive_owner(), Some(Owner::Counter));
    assert!(selection.reserves_automatic_modes());
    selection.capture_binding_input(0, &presets);
    assert_eq!(selection.defensive_owner(), None);
    assert!(!selection.reserves_automatic_modes());
    selection.button_latch &= selection
        .binding_layout
        .binding_ownership(selection.binding_physical);
    selection.block_binding_starts(false);
    selection.capture_binding_input(PresetSlot::L3.button(), &presets);
    assert!(selection.auto_spacing_held());
    assert!(selection.reserves_automatic_modes());
}

#[test]
fn fresh_defensive_defaults_preserve_saved_legacy_bindings() {
    use crate::{
        fighter_profiles::FighterProfiles,
        sequence_config::{persist, Catalog, Sequence},
    };
    let mut profiles = FighterProfiles::default();
    profiles
        .add_sequence_choices(&Catalog::parse("").unwrap())
        .unwrap();
    for kind in [crate::fighter_kinds::KIRBY, crate::fighter_kinds::LUCINA] {
        let presets = profiles.profile(kind).unwrap().menu_options;
        let selection = MenuSelection::new(&presets);
        for (slot, expected) in [
            (PresetSlot::L3, Builtin::AutoSpacing),
            (PresetSlot::DpadDown, Builtin::Normal),
        ] {
            assert_eq!(
                selection
                    .choice(&presets, slot, selection.choices[slot as usize])
                    .unwrap()
                    .id,
                expected.choice_id()
            );
        }
        let mut fresh = MenuSelection::new(&presets);
        fresh.load_saved_recipes(&presets, &Catalog::parse("").unwrap(), kind);
        let expected = if crate::auto_counter::supports(kind) {
            Builtin::Counter
        } else {
            Builtin::Normal
        };
        assert_eq!(
            fresh
                .choice(
                    &presets,
                    PresetSlot::Y,
                    fresh.choices[PresetSlot::Y as usize]
                )
                .unwrap()
                .id,
            expected.choice_id()
        );
        fresh.menu_hidden = true;
        fresh.capture_binding_input(PresetSlot::Y.button(), &presets);
        assert_eq!(fresh.counter_held(), crate::auto_counter::supports(kind));
        let saved = persist::update(
            "",
            kind,
            PresetSlot::L3,
            "saved_turbo",
            Sequence {
                builtin: Some(Builtin::TurboTeabag),
                ..Default::default()
            },
            true,
        )
        .unwrap();
        let catalog = Catalog::parse(&saved).unwrap();
        let mut selection = selection;
        selection.load_saved_recipes(&presets, &catalog, kind);
        assert_eq!(
            selection
                .choice(
                    &presets,
                    PresetSlot::L3,
                    selection.choices[PresetSlot::L3 as usize]
                )
                .unwrap()
                .id,
            "saved_turbo"
        );
    }
}

#[test]
fn auto_spacing_hold_releases_without_toggling_or_latching() {
    let (mut selection, presets) = spacing_selection();
    assert!(selection.auto_spacing_held());
    assert_ne!(selection.builtin_consumed & PresetSlot::L3.button(), 0);
    selection.process_bindings(&presets, PresetSlot::L3.button(), PresetSlot::L3.button(), true, 0);
    assert!(selection.auto_spacing_held());
    assert_eq!(selection.button_latch & PresetSlot::L3.button(), 0);
    selection.capture_binding_input(0, &presets);
    assert!(!selection.auto_spacing_held());
    selection.capture_binding_input(PresetSlot::L3.button(), &presets);
    assert!(selection.auto_spacing_held());
    selection.capture_binding_input(PresetSlot::L3.button() | MAPPER_L, &presets);
    assert!(!selection.auto_spacing_held());
}

#[test]
fn auto_spacing_respects_menu_cancel_and_quarantine() {
    let (mut selection, presets) = spacing_selection();
    selection.menu_hidden = false;
    assert!(!selection.auto_spacing_held());
    selection.menu_hidden = true;
    selection.recipe_editing = true;
    assert!(!selection.auto_spacing_held());
    selection.recipe_editing = false;
    selection.tuning_page = true;
    assert!(!selection.auto_spacing_held());
    selection.tuning_page = false;
    selection.cancel_pending = true;
    assert!(!selection.auto_spacing_held());
    selection.cancel_pending = false;
    selection.pad_quarantine = PresetSlot::L3.button();
    assert!(!selection.auto_spacing_held());
    selection.pad_quarantine = 0;
    selection.block_binding_starts(true);
    selection.capture_binding_input(PresetSlot::L3.button(), &presets);
    assert!(
        selection.auto_spacing_held(),
        "held spacing stays armed through hitstun"
    );
    selection.block_binding_starts(false);
    assert!(selection.auto_spacing_held());
    selection.observe_physical_passthrough(0, ControlLayout::Default);
    assert!(!selection.auto_spacing_held());
    selection.capture_binding_input(PresetSlot::L3.button(), &presets);
    assert!(selection.auto_spacing_held());
}
