use super::*;
use crate::button_bindings::Builtin;

fn spacing_selection() -> (MenuSelection, MenuPresets) {
    let presets = MenuPresets::default();
    let mut selection = MenuSelection {
        menu_hidden: true,
        ..MenuSelection::default()
    };
    selection.choices[PresetSlot::DpadUp as usize] = presets.list(PresetSlot::DpadUp).options.len()
        + Builtin::LModeSwap.visible_template_offset().unwrap();
    selection.capture_binding_input(0, &presets);
    selection.input(DPAD_UP, &presets);
    selection.capture_binding_input(MAPPER_L, &presets);
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
        let spacing = base + Builtin::LModeSwap.visible_template_offset().unwrap();
        let choices = selection.visible_choices(&presets, slot);
        assert!(choices.contains(&spacing));
        assert_eq!(
            selection.choice_end(&presets, slot),
            base + slot.templates()
        );
        assert_eq!(
            selection.choice(&presets, slot, spacing).unwrap().id,
            Builtin::LModeSwap.choice_id()
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
            builtin: Some(Builtin::LModeSwap),
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
        menu.input(0, p);
        menu.input(MAPPER_Y, p);
        assert!(menu.l_spacing_selected());
        menu.input(MAPPER_L, p);
        assert!(menu.auto_spacing_held());
    }
}

#[test]
fn counter_suppresses_spacing_even_when_binding_starts_are_blocked() {
    use crate::action_owner::Owner;
    let (mut selection, presets) = spacing_selection();
    selection.choices[PresetSlot::X as usize] = presets.list(PresetSlot::X).options.len()
        + Builtin::Counter.visible_template_offset().unwrap();
    selection.block_binding_starts(true);
    selection.capture_binding_input(MAPPER_X | MAPPER_L, &presets);
    assert_eq!(selection.defensive_owner(), Some(Owner::Counter));
    assert!(!selection.auto_spacing_held());
    assert!(selection.reserves_automatic_modes());
    selection.capture_binding_input(MAPPER_X, &presets);
    assert_eq!(selection.defensive_owner(), Some(Owner::Counter));
    assert!(!selection.auto_spacing_held());
    selection.capture_binding_input(0, &presets);
    assert_eq!(selection.defensive_owner(), None);
    assert!(!selection.reserves_automatic_modes());
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
            (PresetSlot::L3, Builtin::Normal),
            (PresetSlot::DpadUp, Builtin::Normal),
            (PresetSlot::R3, Builtin::LModeSwap),
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
    assert_ne!(selection.builtin_consumed & MAPPER_L, 0);
    selection.process_bindings(&presets, MAPPER_L, MAPPER_L, true, 0);
    assert!(selection.auto_spacing_held());
    assert_eq!(selection.button_latch & MAPPER_L, 0);
    selection.capture_binding_input(0, &presets);
    assert!(!selection.auto_spacing_held());
    selection.capture_binding_input(MAPPER_L, &presets);
    assert!(selection.auto_spacing_held());
    selection.input(MAPPER_L | DPAD_UP, &presets);
    assert!(!selection.auto_spacing_held());
    assert!(
        !selection.ledge_held(MAPPER_L),
        "swap cancels the previous owner for this sample"
    );
    selection.input(MAPPER_L, &presets);
    selection.input(MAPPER_L, &presets);
    assert!(selection.ledge_held(MAPPER_L));
    selection.input(MAPPER_L | DPAD_UP, &presets);
    assert!(
        !selection.auto_spacing_held(),
        "held mode transition cancels for one sample"
    );
    selection.input(MAPPER_L, &presets);
    assert!(selection.auto_spacing_held());
    assert!(!selection.ledge_held(MAPPER_L));
    selection.reset_inputs();
    selection.capture_binding_input(0, &presets);
    selection.capture_binding_input(MAPPER_L, &presets);
    assert!(!selection.auto_spacing_held());
    assert_eq!(selection.native_buttons(MAPPER_L) & MAPPER_L, MAPPER_L);
}

#[test]
fn physical_l_swap_works_in_both_layouts_without_special_or_double_activation() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        let (mut menu, presets) = spacing_selection();
        menu.reset_inputs();
        menu.menu_hidden = true;
        menu.set_binding_input(0, layout);
        menu.input(0, &presets);
        menu.set_binding_input(DPAD_UP, layout);
        menu.input(DPAD_UP, &presets);
        menu.set_binding_input(MAPPER_L, layout);
        menu.input(layout.policy_buttons(MAPPER_L), &presets);
        assert!(menu.auto_spacing_held());
        assert_eq!(
            menu.native_buttons(layout.policy_buttons(MAPPER_L)) & (MAPPER_B | MAPPER_L),
            0
        );
        menu.set_binding_input(MAPPER_L | MAPPER_B, layout);
        menu.input(layout.policy_buttons(MAPPER_L | MAPPER_B), &presets);
        assert_ne!(
            menu.native_buttons(layout.policy_buttons(MAPPER_L | MAPPER_B)) & MAPPER_B,
            0
        );
        menu.set_binding_input(MAPPER_L | DPAD_UP, layout);
        menu.input(layout.policy_buttons(MAPPER_L | DPAD_UP), &presets);
        assert!(!menu.auto_spacing_held());
        assert_eq!(
            menu.native_buttons(layout.policy_buttons(MAPPER_L)) & MAPPER_L,
            MAPPER_L
        );
    }
}

#[test]
fn spacing_allows_custom_buttons_and_swap_respects_menu_and_press_edges() {
    use crate::{fighter_profiles::FighterProfiles, sequence_config::Catalog};
    let mut profiles = FighterProfiles::default();
    profiles
        .add_sequence_choices(
            &Catalog::parse("[kirby.y.manual]\nstart_delay=0\ndefault=true\nactions=fair:4\n")
                .unwrap(),
        )
        .unwrap();
    let presets = profiles
        .profile(crate::fighter_kinds::KIRBY)
        .unwrap()
        .menu_options;
    let mut menu = MenuSelection::new(&presets);
    menu.menu_hidden = true;
    menu.input(0, &presets);
    menu.input(MENU_STICK_CLICK, &presets);
    assert!(menu.l_spacing_selected());
    menu.input(MENU_STICK_CLICK, &presets);
    assert!(menu.l_spacing_selected());
    menu.input(MAPPER_L, &presets);
    assert!(menu.auto_spacing_held());
    menu.input(MAPPER_L | MAPPER_Y, &presets);
    assert_eq!(menu.preview_slot, Some(PresetSlot::Y));
    menu.input(MAPPER_L | MENU_STICK_CLICK, &presets);
    assert!(!menu.l_spacing_selected());
    assert!(menu.preview_slot.is_none());
    assert!(
        menu.cancel_pending,
        "switching while held cancels the previous owner"
    );
    menu.input(0, &presets);
    menu.input(MAPPER_MINUS, &presets);
    menu.input(0, &presets);
    menu.input(MENU_STICK_CLICK, &presets);
    assert!(
        !menu.l_spacing_selected(),
        "visible menu navigation cannot swap L"
    );
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
    selection.pad_quarantine = MAPPER_L;
    assert!(!selection.auto_spacing_held());
    selection.pad_quarantine = 0;
    selection.block_binding_starts(true);
    selection.capture_binding_input(MAPPER_L, &presets);
    assert!(
        selection.auto_spacing_held(),
        "held spacing stays armed through hitstun"
    );
    selection.block_binding_starts(false);
    assert!(selection.auto_spacing_held());
    selection.observe_physical_passthrough(0, ControlLayout::Default);
    assert!(!selection.auto_spacing_held());
    selection.capture_binding_input(MAPPER_L, &presets);
    assert!(selection.auto_spacing_held());
}
