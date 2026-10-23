//! A fighter's function lists are compiled and every selection is independent.
use super::*;
use crate::{
    fighter_profiles::FighterProfiles,
    ledge_menu::MenuSelection,
    ledge_trap::{GetupAttack, LedgeDropAction},
};
use std::collections::BTreeSet;
#[test]
fn compiled_lists_have_unique_nonempty_ids_and_valid_defaults() {
    let p = FighterProfiles::default();
    let p = p.profile(6).unwrap();
    for slot in PresetSlot::ALL {
        let list = p.menu_options.list(slot);
        assert!(!list.options.is_empty());
        assert!(list.selected < list.options.len());
        let mut ids = BTreeSet::new();
        for choice in list.options {
            assert!(!choice.id.is_empty() && !choice.name.is_empty());
            assert!(ids.insert(choice.id));
        }
    }
}
#[test]
fn independent_compiled_selections_restore_their_own_timing() {
    let profiles = FighterProfiles::default();
    let p = profiles.profile(6).unwrap();
    let base = p.apply(LedgeSettings {
        enabled: true,
        ..LedgeSettings::default()
    });
    let mut menu = MenuSelection::new(&p.menu_options);
    menu.choices[0] = 3;
    menu.choices[4] = 1;
    let stone = menu.settings(&p.menu_options, base);
    assert_eq!(stone.getup_attack, GetupAttack::DownSpecial);
    assert_eq!(stone.fighter.normal_hit_lead_frames, 0);
    assert_eq!(stone.drop_action, LedgeDropAction::DownTiltShield);
    menu.choices[0] = 0;
    let nair = menu.settings(&p.menu_options, base);
    assert_eq!(nair.getup_attack, GetupAttack::NeutralAir);
    assert_eq!(nair.fighter.normal_hit_lead_frames, 8);
    assert_eq!(nair.drop_action, LedgeDropAction::DownTiltShield);
}
#[test]
fn every_compiled_response_list_excludes_off_and_preserves_global_disable() {
    let profiles = FighterProfiles::default();
    for kind in [6, 9, 11] {
        let profile = profiles.profile(kind).unwrap();
        for slot in PresetSlot::ALL {
            let list = profile.menu_options.list(slot);
            assert!(!list.options.is_empty());
            for choice in list.options {
                assert_ne!(choice.id, "off");
                assert_ne!(choice.name, "Off");
                assert!(!(choice.prepare)(LedgeSettings::default()).enabled);
            }
        }
    }
}
