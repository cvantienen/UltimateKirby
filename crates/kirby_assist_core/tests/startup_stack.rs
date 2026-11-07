//! The Switch's loader thread has a 64 KiB stack. Never materialize all fighter
//! profiles in one temporary just to parse a small saved button configuration.
use kirby_assist_core::{fighter_profiles::FighterProfiles, sequence_config::Catalog};
#[test]
fn constructor_and_saved_binding_validation_fit_a_small_loader_stack() {
    std::thread::Builder::new().stack_size(64*1024).spawn(|| {
        let mut profiles=FighterProfiles::default();
        profiles.validate().unwrap();
        let saved="[kirby.y.saved]\nstart_delay=0\ndefault=false\nactions=short_hop,wait:12,nair\n[yoshi.dpad_up.mode]\nmode=ledge_trump\nactivation=toggle\n";
        let catalog=Catalog::parse(saved).unwrap();
        profiles.add_sequence_choices(&catalog).unwrap();
        profiles.validate().unwrap();
        assert!(profiles.profile(6).is_some());
        assert!(profiles.profile(5).is_some());
        assert_eq!((0..kirby_assist_core::fighter_kinds::PLAYABLE_KIND_COUNT).filter(|kind|profiles.profile(*kind).is_some()).count(),79);
    }).unwrap().join().unwrap();
}

#[test]
fn full_deployment_catalog_loads_on_the_temporary_config_worker_stack() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let catalog = Catalog::parse(include_str!(
                "../../../deployment/ultimate/kirby_assist/assembled/sequences.txt"
            ))
            .unwrap();
            assert!(catalog.len() > 1000);
            let mut profiles = FighterProfiles::default();
            profiles.add_sequence_choices(&catalog).unwrap();
            profiles.validate().unwrap();
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn bowser_dpad_up_and_y_defaults_are_independent_and_not_legacy_modes() {
    use kirby_assist_core::{fighter_profiles::presets::PresetSlot, settings_menu::MenuSelection};
    const SAVED: &str = "[bowser.dpad_up.grab]\nname=Grab\nstart_delay=0\ndefault=true\nactions=check:enemy_within:45:0,drift:opponent:1:1,opponent_input:r:0:0:1\n[bowser.y.menu_custom]\nname=Neutral air dodge\nstart_delay=0\ndefault=true\nground_only=false\nactions=wait:1,shield:1\n[bowser.dpad_left.saved]\nstart_delay=0\ndefault=true\nactions=b:4\n[bowser.dpad_right.saved]\nstart_delay=0\ndefault=true\nactions=side_b:4\n";
    // Putting Grab under Y is the configuration mistake found in the repo.
    let invalid = SAVED.replace("bowser.dpad_up.grab", "bowser.y.grab");
    assert!(Catalog::parse(&invalid)
        .unwrap()
        .validate(&FighterProfiles::default())
        .unwrap_err()
        .contains("multiple defaults"));
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(|| {
            for diagnostic in [false, true] {
                let catalog = Catalog::parse(SAVED).unwrap().for_build(diagnostic);
                let mut profiles = FighterProfiles::default();
                profiles.add_sequence_choices(&catalog).unwrap();
                let presets = profiles.profile(15).unwrap().menu_options;
                let mut menu = MenuSelection::new(&presets);
                menu.input(0, &presets);
                assert!(menu.preview_slot.is_none());
                for (slot, id) in [
                    (PresetSlot::DpadUp, "grab"),
                    (PresetSlot::Y, "menu_custom"),
                    (PresetSlot::DpadLeft, "saved"),
                    (PresetSlot::DpadRight, "saved"),
                ] {
                    let list = presets.list(slot);
                    let selected = &list.options[menu.choices[slot as usize]];
                    assert_eq!(selected.id, id);
                    assert!(selected.sequence.mode.is_none());
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
