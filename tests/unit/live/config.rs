use super::*;
/// Verifies shared templates remain shared and unrelated menus stay unexpanded.
#[test]
fn startup_keeps_shared_templates_but_never_expands_other_fighter_menus() {
    let catalog=Catalog::parse("[template.defend]\nstart_delay=0\nactions=shield:4\n[bowser.roll.spin]\nstart_delay=0\nreaction=auto_roll:30:12\nactions=up_b:4,drift:opponent:1:69\n[lucina.neutral.poke]\nstart_delay=23\nactions=forward_tilt:1\n").unwrap();
    let (_, profiles, library) = prepare_runtime(LiveConfig::default(), catalog).unwrap();
    let mut count = 0;
    for kind in 0..kirby_assist_core::fighter_kinds::PLAYABLE_KIND_COUNT {
        let Some(profile) = profiles.profile(kind) else {
            continue;
        };
        count += 1;
        assert!(profile
            .menu_options
            .lists
            .iter()
            .all(|list| list.options.is_empty()));
        assert_eq!(profile.menu_options.templates.len(), 1);
    }
    assert_eq!(count, 90);
    assert_eq!(library.fighter_entries(15), 1);
    assert_eq!(library.fighter_entries(22), 1);
    assert_eq!(library.for_fighter(5).len(), 0);
    assert!(std::ptr::eq(
        profiles
            .profile(15)
            .unwrap()
            .menu_options
            .templates
            .as_ptr(),
        profiles
            .profile(22)
            .unwrap()
            .menu_options
            .templates
            .as_ptr()
    ));
}

#[test]
fn startup_migrates_legacy_mode_swap_and_preserves_conflicts_without_disabling_load() {
    use kirby_assist_core::fighter_profiles::presets::PresetSlot;
    let legacy = "[kirby.dpad_up.swap]\nname=Old swap\nbuiltin=l_mode_swap\ndefault=true\n";
    let (_, _, library) =
        prepare_runtime(LiveConfig::default(), Catalog::parse(legacy).unwrap()).unwrap();
    let catalog = library.for_fighter(kirby_assist_core::fighter_kinds::KIRBY);
    assert_eq!(catalog.entries()[0].slot, PresetSlot::R3);
    assert_eq!(catalog.entries()[0].choice, "swap");
    let conflict = format!("{legacy}[kirby.r3.explicit]\nname=My R3\nbuiltin=normal\n");
    let original = Catalog::parse(&conflict).unwrap();
    let expected_config = LiveConfig::parse("auto_tech=false").unwrap();
    let (config, _, library) = prepare_runtime(expected_config, original.clone()).unwrap();
    assert!(!config.controls.auto_tech);
    let preserved = library.for_fighter(kirby_assist_core::fighter_kinds::KIRBY);
    assert_eq!(preserved.entries(), original.entries());
    assert_eq!(preserved.entries()[0].slot, PresetSlot::DpadUp);
    assert_eq!(preserved.entries()[1].slot, PresetSlot::R3);
    let error = original.clone().migrate_mode_swap().unwrap_err();
    assert!(error.contains("kirby") && error.contains("Old swap") && error.contains("My R3"));
}
