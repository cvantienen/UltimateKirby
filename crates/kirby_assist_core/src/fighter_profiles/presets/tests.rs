//! Selection is a reference to file-authored inputs and timing, not a function choice.
use super::*;
use crate::{fighter_profiles::FighterProfiles, ledge_menu::MenuSelection};
#[test]
fn file_lists_have_unique_ids_and_copy_their_own_delays() {
    let file = crate::sequence_config::Catalog::parse("[kirby.neutral.first]\nstart_delay=0\nactions=jab\n[kirby.neutral.second]\nstart_delay=2\ndefault=true\nactions=short_hop, wait:1, up_b").unwrap();
    let mut profiles = FighterProfiles::default();
    profiles.add_sequence_choices(&file).unwrap();
    let p = profiles.profile(6).unwrap();
    let mut menu = MenuSelection::new(&p.menu_options);
    assert_eq!(menu.choices[0], 1);
    let second = menu.settings(&p.menu_options, crate::ledge_trap::LedgeSettings::default());
    assert_eq!(second.sequence_overrides[0].unwrap().start_delay, 2);
    menu.choices[0] = 0;
    let first = menu.settings(&p.menu_options, crate::ledge_trap::LedgeSettings::default());
    assert_eq!(first.sequence_overrides[0].unwrap().start_delay, 0);
    assert_ne!(first.sequence_overrides[0], second.sequence_overrides[0]);
    assert!(first.sequence_overrides[1..].iter().all(Option::is_none));
    assert!(
        !first.enabled,
        "loading a file never turns on a globally disabled tool"
    );
}
#[test]
fn empty_catalog_is_a_valid_empty_menu() {
    let menu = MenuPresets::default();
    assert!(menu
        .lists
        .iter()
        .all(|list| list.options.is_empty() && list.selected == 0));
}

#[test]
fn saved_nondefault_button_recipes_start_on_normal_button_and_pass_startup() {
    let source = "[kirby.x.saved]\nstart_delay=0\ndefault=false\nactions=hold:a:4\n[yoshi.y.menu_custom]\nstart_delay=0\ndefault=false\nactions=short_hop, wait:12, forward_air\n";
    let catalog = crate::sequence_config::Catalog::parse(source).unwrap();
    crate::sequence_config::persist::validate_output(source).unwrap();
    for diagnostic in [false, true] {
        let mut profiles = FighterProfiles::default();
        profiles
            .add_sequence_choices(&catalog.clone().for_build(diagnostic))
            .unwrap();
        profiles.validate().unwrap();
        for (kind, slot) in [(6, PresetSlot::X), (5, PresetSlot::Y)] {
            let p = profiles.profile(kind).unwrap();
            let menu = MenuSelection::new(&p.menu_options);
            let list = p.menu_options.list(slot);
            assert_eq!(list.selected, slot.default_index(list.options.len()));
            assert_eq!(
                menu.choice(&p.menu_options, slot, list.selected)
                    .unwrap()
                    .id,
                crate::button_bindings::Builtin::default_for(slot).choice_id()
            );
            assert!(menu
                .settings(&p.menu_options, Default::default())
                .sequence_overrides[slot as usize]
                .is_none());
        }
    }
}

#[test]
fn startup_still_rejects_button_selection_beyond_normal_and_invalid_ledge_selection() {
    let catalog = crate::sequence_config::Catalog::parse("[kirby.x.saved]\nstart_delay=0\nactions=jab\n[kirby.neutral.n]\nstart_delay=0\nactions=grab\n").unwrap();
    let mut profiles = FighterProfiles::default();
    profiles.add_sequence_choices(&catalog).unwrap();
    profiles.profiles.get_mut(&6).unwrap().menu_options.lists[PresetSlot::X as usize].selected =
        1 + PresetSlot::X.templates();
    assert!(profiles.validate().is_err());
    profiles.profiles.get_mut(&6).unwrap().menu_options.lists[PresetSlot::X as usize].selected = 1;
    profiles.profiles.get_mut(&6).unwrap().menu_options.lists[PresetSlot::Neutral as usize]
        .selected = 1;
    assert!(profiles.validate().is_err());
}
