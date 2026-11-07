//! Playable registrations are distinct from read-only DLC reference records.
use kirby_assist_core::{
    fighter_profiles::FighterProfiles,
    ledge_menu::MenuSelection,
    ledge_trap::{LedgeSettings, ResponseImplementation},
    projectile_reflect,
    sequence_config::{Action, Catalog, SequenceLibrary},
};

#[test]
fn both_native_aegis_identities_enable_their_own_editable_menus_without_inherited_attacks() {
    let profiles = FighterProfiles::default();
    profiles.validate().unwrap();
    let file = Catalog::parse(include_str!(
        "../../../deployment/ultimate/kirby_assist/assembled/sequences.txt"
    ))
    .unwrap();
    file.validate(&profiles).unwrap();
    let library = SequenceLibrary::from_catalog(file);
    for (kind, name, implementation, startup) in [
        (90, "pyra", ResponseImplementation::Pyra, 7),
        (91, "mythra", ResponseImplementation::Mythra, 6),
    ] {
        let profile = profiles.profile(kind).expect("playable form must exist");
        assert_eq!(profile.name, name);
        assert_eq!(profile.reference().unwrap().name, name);
        let base = profile.apply(LedgeSettings {
            enabled: true,
            ..Default::default()
        });
        assert!(base.enabled);
        assert_eq!(base.implementation, Some(implementation));
        assert_eq!(profile.ledge.grab.standing, startup);
        assert_eq!(profile.ledge.grab.standing_range, 12.);
        let selected = library.for_fighter(kind);
        assert_eq!(selected.len(), 5);
        assert!(selected
            .entries()
            .iter()
            .all(|e| e.fighter_kind == kind && e.fighter == name));
        let mut menu = MenuSelection::new(&profile.menu_options);
        menu.load_saved_recipes(&profile.menu_options, &selected, kind);
        let settings = menu.settings(&profile.menu_options, base);
        for slot in 0..5 {
            assert_eq!(
                settings.sequence_overrides[slot].unwrap().actions(),
                &[Action::Wait(0)]
            );
        }
        assert!(!menu
            .resolve_reactions(&profile.menu_options)
            .unwrap()
            .has_simple_events());
        assert!(
            !projectile_reflect::supports(kind),
            "swap/foresight must not be treated as a reflector"
        );
    }
}
