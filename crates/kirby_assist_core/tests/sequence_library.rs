//! Dormant fighter definitions are compact; only a requested identity expands.
use kirby_assist_core::{
    fighter_profiles::FighterProfiles,
    sequence_config::{Catalog, SequenceLibrary, MAX_ACTIONS},
};

#[test]
fn whole_roster_round_trips_one_fighter_at_a_time_without_static_menu_publication() {
    let source = include_str!("../../../deployment/ultimate/kirby_assist/assembled/sequences.txt");
    let profiles = FighterProfiles::default();
    for diagnostic in [false, true] {
        let catalog = Catalog::parse(source).unwrap().for_build(diagnostic);
        let library = SequenceLibrary::from_catalog(catalog.clone());
        assert_eq!(library.len(), catalog.len());
        assert_eq!(library.templates, catalog.templates);
        assert!(
            library.action_slots() < library.len() * MAX_ACTIONS / 3,
            "dormant definitions must not retain full tapes"
        );
        let mut fighters = 0;
        for kind in 0..0x5e {
            let Some(profile) = profiles.profile(kind) else {
                continue;
            };
            fighters += 1;
            assert!(
                profile
                    .menu_options
                    .lists
                    .iter()
                    .all(|list| list.options.is_empty()),
                "base registry must not publish fighter recipes"
            );
            let active = library.for_fighter(kind);
            let expected: Vec<_> = catalog
                .entries()
                .iter()
                .filter(|e| e.fighter_kind == kind)
                .cloned()
                .collect();
            assert_eq!(active.entries(), expected, "fighter {kind}");
            assert_eq!(library.fighter_entries(kind), expected.len());
            assert!(
                active.templates.is_empty(),
                "templates remain shared, not cloned into fighter snapshots"
            );
            assert_eq!(active.tuning.get(&kind), catalog.tuning.get(&kind));
            assert!(active.tuning.keys().all(|k| *k == kind));
        }
        assert_eq!(fighters, 79);
        for kind in [-1, 0x7fff] {
            assert_eq!(library.for_fighter(kind).len(), 0);
            assert_eq!(library.fighter_entries(kind), 0);
        }
    }
}

#[test]
fn compact_storage_preserves_all_timing_binding_and_reaction_fields() {
    let source="[template.held]\nname=Generic hold\nstart_delay=3\nactions=hold_until_manual:a:0:0\n[kirby.x.track]\nname=Custom\nstart_delay=4\ntimeout=99\nspatial_timeout=13\nground_only=true\nwhile_held=true\nactions=controller_input:a:0.2:0.6:1:0:4:opponent:facing,wait:2\n[bowser.roll.auto]\nstart_delay=0\nreaction=auto_roll:30:12\nactions=up_b:4,drift:opponent:1:69\n[kirby.y.normal]\nbuiltin=normal\ndefault=true\n[kirby.dpad_up.mode]\nmode=ledge_trap\nactivation=toggle\n[kirby.settings]\nwalk_strength=0.3\n[bowser.settings]\nwalk_strength=0.4\n";
    let catalog = Catalog::parse(source).unwrap();
    let library = SequenceLibrary::from_catalog(catalog.clone());
    for kind in [6, 15] {
        let active = library.for_fighter(kind);
        assert_eq!(
            active.entries(),
            catalog
                .entries()
                .iter()
                .filter(|e| e.fighter_kind == kind)
                .cloned()
                .collect::<Vec<_>>()
        );
        assert_eq!(active.tuning.len(), 1);
        assert_eq!(active.tuning.get(&kind), catalog.tuning.get(&kind));
    }
}

#[test]
fn every_registered_fighter_reference_belongs_to_the_same_native_identity() {
    let profiles = FighterProfiles::default();
    for kind in 0..0x5e {
        let Some(profile) = profiles.profile(kind) else {
            continue;
        };
        let expected = match profile.name {
            "krool" => "king_k_rool",
            "littlemac" => "little_mac",
            "pacman" => "pac_man",
            "ice_climbers_nana" => "ice_climbers",
            name => name,
        };
        assert_eq!(
            profile.reference().unwrap().name,
            expected,
            "native kind {kind} must not borrow another fighter's facts"
        );
    }
}
