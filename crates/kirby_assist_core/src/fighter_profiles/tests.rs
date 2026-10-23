//! Compiled catalog invariants. These inspect registration/data, not file I/O.
use super::*;
use crate::ledge_trap::GetupAttack;
use crate::{ledge_menu::MenuSelection, live_input::LiveConfig};
#[test]
fn compiled_kirby_keeps_moves_and_default_choices() {
    let profiles = FighterProfiles::default();
    assert_eq!(profiles.profiles.len(), 77); // 76 base-game forms plus Nana.
    assert!(profiles.validate().is_ok());
    let p = profiles.profile(6).unwrap();
    assert_eq!(p.name, "kirby");
    assert_eq!(p.ledge.neutral_air, MoveFrames { first: 8, last: 32 });
    assert_eq!(p.ledge.down_smash, MoveFrames { first: 7, last: 19 });
    assert_eq!(p.ledge.down_special_startup, Some(11));
    let base = p.apply(LedgeSettings {
        enabled: true,
        ..LedgeSettings::default()
    });
    let selected = MenuSelection::new(&p.menu_options).settings(&p.menu_options, base);
    assert_eq!(selected.getup_attack, GetupAttack::NeutralAir);
    assert_eq!(selected.fighter.normal_hit_lead_frames, 8);
    assert_eq!(selected.response_choices, [Some(0); 5]);
}

#[test]
fn every_base_game_native_kind_has_its_own_valid_compiled_choices() {
    use crate::fighter_profiles::presets::PresetSlot;
    let catalog = FighterProfiles::default();
    assert!(catalog.validate().is_ok());
    for kind in 0..=0x4c {
        let profile = catalog.profile(kind).unwrap();
        assert_eq!(
            profile.fighter_kind, kind,
            "unexpected Kirby fallback for {kind:#x}"
        );
        for slot in PresetSlot::ALL {
            assert!(
                profile.menu_options.list(slot).options.len() >= 2 ||
                // Existing profiles are intentionally unchanged by this expansion.
                matches!(kind, 0 | 1 | 2 | 3 | 5 | 6 | 7 | 8 | 9 | 11 | 0x29 | 0x34 | 0x37 | 0x45 | 0x46)
            );
        }
        let selected = MenuSelection::new(&profile.menu_options).settings(
            &profile.menu_options,
            profile.apply(LedgeSettings::default()),
        );
        assert!(
            !selected.enabled,
            "a new catalog must not override global disable"
        );
    }
    // No DLC profile was added; the previously authorized Kirby fallback remains.
    for kind in 0x51..=0x5d {
        assert_eq!(catalog.profile(kind).unwrap().fighter_kind, 6);
    }
}

#[test]
fn special_roster_cases_do_not_pretend_resources_projectiles_or_shoto_taps_are_body_moves() {
    use crate::fighter_profiles::presets::PresetSlot;
    let catalog = FighterProfiles::default();
    let mega = catalog.profile(0x31).unwrap();
    for slot in PresetSlot::ALL {
        assert!(mega
            .menu_options
            .list(slot)
            .options
            .iter()
            .all(|c| !matches!(c.id, "nair" | "shield_nair" | "drop_nair" | "forward_tilt")));
    }
    let mut selection = MenuSelection::new(&mega.menu_options);
    selection.choices[PresetSlot::Attack as usize] = 0; // Grab / rear shield.
    assert!(
        !selection
            .settings(&mega.menu_options, mega.apply(LedgeSettings::default()))
            .responses
            .rear_block_nair
    );
    for kind in [0x3c, 0x3d] {
        let p = catalog.profile(kind).unwrap();
        for list in p.menu_options.lists {
            assert!(list.options.iter().all(|c| !matches!(
                c.id,
                "down_tilt" | "forward_tilt" | "up_tilt" | "down_tilt_shield"
            )));
        }
    }
    let meta = catalog.profile(0x1d).unwrap();
    for list in meta.menu_options.lists {
        assert!(list
            .options
            .iter()
            .all(|c| !matches!(c.id, "down_tilt" | "forward_tilt" | "down_tilt_shield")));
    }
    let olimar = catalog.profile(0x2b).unwrap();
    let roll = olimar.menu_options.list(PresetSlot::Roll);
    assert_eq!(roll.options[roll.selected].id, "down_tilt"); // Works without Pikmin.
    assert_eq!(
        catalog.profile(0xc).unwrap().ledge.neutral_air,
        MoveFrames { first: 6, last: 30 }
    );
    let ness = &catalog.profile(0xa).unwrap().ledge;
    assert_eq!(
        ness.down_smash,
        MoveFrames {
            first: 23,
            last: 25
        }
    );
    assert_eq!(
        ness.down_smash_back,
        Some(MoveFrames {
            first: 12,
            last: 15
        })
    );
    // Conservative first pulse rather than inventing 5-30 continuous coverage.
    assert_eq!(
        catalog.profile(0x36).unwrap().ledge.neutral_air,
        MoveFrames { first: 5, last: 5 }
    );
}
#[test]
fn unconfigured_fighters_use_kirby_but_invalid_identity_and_global_disable_stay_inert() {
    let profiles = FighterProfiles::default();
    let base = LedgeSettings {
        enabled: true,
        ..LedgeSettings::default()
    };
    for kind in [None, Some(-1), Some(0x5e), Some(255)] {
        assert!(!profiles.ledge_settings(kind, base).enabled);
    }
    for kind in [0x51, 0x52, 0x5d] {
        let fallback = profiles.profile(kind).unwrap();
        assert_eq!(fallback.fighter_kind, 6);
        assert_eq!(fallback.name, "kirby");
        let settings = profiles.ledge_settings(Some(kind), base);
        assert!(settings.enabled);
        assert_eq!(settings.fighter, profiles.profile(6).unwrap().ledge);
        assert!(
            !profiles
                .ledge_settings(Some(kind), LedgeSettings::default())
                .enabled
        );
    }
    let p = profiles.profile(6).unwrap();
    let selected = MenuSelection::new(&p.menu_options)
        .settings(&p.menu_options, p.apply(LedgeSettings::default()));
    assert!(!selected.enabled);
}
#[test]
fn numeric_adjustments_change_timing_without_changing_registered_functions() {
    let config = LiveConfig::parse("ledge_trap = true\nledge_nair_hit_lead_frames = 10\nledge_stone_startup_frames = 13\nledge_stone_hit_lead_frames = 1\nledge_drop_request_timeout_frames = 40").unwrap();
    let profiles = FighterProfiles::default();
    let p = profiles.profile(6).unwrap();
    let base = p.apply(config.ledge);
    let mut menu = MenuSelection::new(&p.menu_options);
    let nair = menu.settings(&p.menu_options, base);
    assert_eq!(nair.fighter.normal_hit_lead_frames, 10);
    menu.choices[0] = 3;
    let stone = menu.settings(&p.menu_options, base);
    assert_eq!(stone.getup_attack, GetupAttack::DownSpecial);
    assert_eq!(stone.fighter.down_special_startup, Some(13));
    assert_eq!(stone.fighter.normal_hit_lead_frames, 1);
    assert_eq!(stone.drop_request_timeout_frames, 40);
    assert_eq!(p.menu_options.lists[0].options[3].id, "stone");
}
#[test]
fn invalid_numeric_adjustments_are_rejected() {
    for key in [
        "ledge_nair_hit_lead_frames = 21",
        "ledge_stone_hit_lead_frames = 21",
        "ledge_stone_startup_frames = 0",
        "ledge_stone_startup_frames = 121",
        "ledge_drop_request_timeout_frames = 0",
        "ledge_drop_request_timeout_frames = 121",
    ] {
        assert!(LiveConfig::parse(key).is_err(), "{key}");
    }
}

#[test]
fn invalid_compiled_move_data_fails_before_hooks_install() {
    let mut catalog = FighterProfiles::default();
    catalog.profiles.get_mut(&6).unwrap().ledge.neutral_air.last = 255;
    assert!(catalog.validate().is_err());
}

#[test]
fn new_fighters_use_their_own_move_windows_and_default_responses() {
    let profiles = FighterProfiles::default();
    for (kind, name, normal) in [
        (9, "luigi", GetupAttack::NeutralAir),
        (11, "captain_falcon", GetupAttack::ForwardTilt),
    ] {
        let p = profiles.profile(kind).unwrap();
        assert_eq!(p.name, name);
        let menu = MenuSelection::new(&p.menu_options);
        let base = p.apply(LedgeSettings {
            enabled: true,
            ..LedgeSettings::default()
        });
        let selected = menu.settings(&p.menu_options, base);
        assert!(selected.enabled);
        assert_eq!(selected.getup_attack, normal);
        assert_eq!(selected.fighter, p.ledge);
        for list in p.menu_options.lists {
            assert!(list.options.iter().all(|entry| entry.id != "stone"));
        }
    }
    let luigi = &profiles.profile(9).unwrap().ledge;
    assert_eq!(luigi.neutral_air, MoveFrames { first: 3, last: 31 });
    assert_eq!(luigi.down_smash, MoveFrames { first: 6, last: 7 });
    assert_eq!(luigi.grab.turn_or_shield, 18);
    let falcon = profiles.profile(11).unwrap();
    let mut menu = MenuSelection::new(&falcon.menu_options);
    menu.choices[0] = 1; // Select Falcon's nair variation, not Kirby's entry index.
    let mut base = falcon.apply(LedgeSettings::default());
    base.response_adjustments.nair_hit_lead = Some(8);
    let selected = menu.settings(&falcon.menu_options, base);
    assert_eq!(selected.fighter.normal_hit_lead_frames, 1);
    assert_eq!(
        selected.fighter.neutral_air,
        MoveFrames { first: 7, last: 8 }
    );
    assert_eq!(
        selected.fighter.down_smash,
        MoveFrames {
            first: 19,
            last: 20
        }
    );
}

#[test]
fn original_eight_have_distinct_compiled_profiles_and_ledge_choices() {
    let profiles = FighterProfiles::default();
    assert!(profiles.validate().is_ok());
    for (kind, name) in [
        (0, "mario"),
        (1, "donkey_kong"),
        (2, "link"),
        (3, "samus"),
        (5, "yoshi"),
        (6, "kirby"),
        (7, "fox"),
        (8, "pikachu"),
    ] {
        let profile = profiles.profile(kind).unwrap();
        assert_eq!(
            profile.fighter_kind, kind,
            "must not silently use Kirby fallback"
        );
        assert_eq!(profile.name, name);
        for list in profile.menu_options.lists {
            assert!(!list.options.is_empty());
            assert!(list.options.iter().all(|c| c.id != "off"));
        }
        let settings = profiles.ledge_settings(Some(kind), LedgeSettings::default());
        assert!(
            !settings.enabled,
            "a profile must not enable a globally disabled tool"
        );
    }
}
#[test]
fn rear_down_smash_and_shield_grab_use_distinct_verified_timings() {
    let profiles = FighterProfiles::default();
    let mario = profiles.profile(0).unwrap().ledge;
    assert_eq!(
        mario.down_smash_toward(8.0, 1.0),
        MoveFrames { first: 5, last: 6 }
    );
    assert_eq!(
        mario.down_smash_toward(-8.0, 1.0),
        MoveFrames {
            first: 14,
            last: 14
        }
    );
    let samus = profiles.profile(3).unwrap().ledge;
    assert_eq!(samus.grab.startup(0x7), 18);
    assert_eq!(samus.grab.startup(0x1b), 19);
    assert_eq!(
        profiles.profile(8).unwrap().ledge.neutral_air,
        MoveFrames { first: 3, last: 6 }
    );
}

#[test]
fn sonic_uses_own_windows_and_omits_non_damaging_grounded_spring_punish() {
    use crate::fighter_profiles::presets::PresetSlot;
    use crate::ledge_trap::ResponseImplementation;
    let catalog = FighterProfiles::default();
    assert!(catalog.validate().is_ok());
    let profile = catalog.profile(sonic::FIGHTER_KIND).unwrap();
    assert_eq!(profile.name, "sonic");
    assert_eq!(profile.fighter_kind, 0x29);
    let menu = MenuSelection::new(&profile.menu_options);
    let settings = menu.settings(
        &profile.menu_options,
        profile.apply(LedgeSettings {
            enabled: true,
            ..LedgeSettings::default()
        }),
    );
    assert_eq!(settings.implementation, Some(ResponseImplementation::Sonic));
    assert_eq!(
        settings.fighter.neutral_air,
        MoveFrames { first: 6, last: 38 }
    );
    assert_eq!(
        settings.fighter.forward_tilt,
        MoveFrames { first: 8, last: 11 }
    );
    assert_eq!(
        settings.fighter.down_smash,
        MoveFrames {
            first: 12,
            last: 13
        }
    );
    assert_eq!(settings.fighter.grab.startup(0x1b), 11);
    assert_eq!(settings.fighter.up_special_startup, None);
    assert_eq!(settings.fighter.down_special_startup, None);
    assert_eq!(settings.getup_attack, GetupAttack::NeutralAir);
    assert!(profile
        .menu_options
        .list(PresetSlot::Roll)
        .options
        .iter()
        .any(|choice| choice.id == "grab"));
    assert!(!profile
        .menu_options
        .list(PresetSlot::Attack)
        .options
        .iter()
        .any(|choice| choice.id == "shield_up_b"));
}

#[test]
fn krool_and_isabelle_register_distinct_moves_choices_and_forgiving_defaults() {
    use crate::fighter_profiles::presets::PresetSlot;
    use crate::ledge_trap::ResponseImplementation;
    let catalog = FighterProfiles::default();
    assert!(catalog.validate().is_ok());
    for (kind, implementation, nair, roll_default) in [
        (
            0x45,
            ResponseImplementation::KingKRool,
            MoveFrames { first: 7, last: 30 },
            "grab",
        ),
        (
            0x46,
            ResponseImplementation::Isabelle,
            MoveFrames { first: 5, last: 25 },
            "down_smash",
        ),
    ] {
        let profile = catalog.profile(kind).unwrap();
        assert_eq!(profile.fighter_kind, kind);
        let menu = MenuSelection::new(&profile.menu_options);
        let s = menu.settings(
            &profile.menu_options,
            profile.apply(LedgeSettings {
                enabled: true,
                ..LedgeSettings::default()
            }),
        );
        assert_eq!(s.implementation, Some(implementation));
        assert_eq!(s.fighter.neutral_air, nair);
        assert_eq!(s.getup_attack, GetupAttack::NeutralAir);
        let roll = profile.menu_options.list(PresetSlot::Roll);
        assert_eq!(roll.options[roll.selected].id, roll_default);
        for (slot, minimum) in [
            (PresetSlot::Neutral, 4),
            (PresetSlot::Jump, 3),
            (PresetSlot::Roll, 2),
            (PresetSlot::Attack, 3),
            (PresetSlot::Drop, 4),
        ] {
            let list = profile.menu_options.list(slot);
            assert!(list.options.len() >= minimum);
            assert!(list.options.iter().all(|choice| choice.id != "off"));
        }
        assert_eq!(s.fighter.up_special_startup, None);
        assert_eq!(s.fighter.down_special_startup, None);
    }
    assert_eq!(
        catalog.profile(0x45).unwrap().ledge.down_smash,
        MoveFrames {
            first: 22,
            last: 24
        }
    );
    assert_eq!(
        catalog.profile(0x46).unwrap().ledge.down_smash_back,
        Some(MoveFrames {
            first: 12,
            last: 13
        })
    );
}

#[test]
fn pacman_and_little_mac_defaults_use_their_actual_strengths() {
    use crate::fighter_profiles::presets::PresetSlot;
    use crate::ledge_trap::ResponseImplementation;
    let catalog = FighterProfiles::default();
    assert!(catalog.validate().is_ok());
    let pac = catalog.profile(0x37).unwrap();
    assert_eq!(pac.name, "pacman");
    assert_eq!(pac.fighter_kind, 0x37);
    assert_eq!(pac.ledge.neutral_air, MoveFrames { first: 3, last: 19 });
    assert_eq!(pac.ledge.grab.active_frames, 22);
    let roll = pac.menu_options.list(PresetSlot::Roll);
    assert_eq!(roll.options[roll.selected].id, "grab");
    let mut menu = MenuSelection::new(&pac.menu_options);
    menu.choices[PresetSlot::Roll as usize] = 0;
    let s = menu.settings(
        &pac.menu_options,
        pac.apply(LedgeSettings {
            enabled: true,
            ..LedgeSettings::default()
        }),
    );
    assert_eq!(s.implementation, Some(ResponseImplementation::PacMan));
    assert_eq!(s.roll_hit_window_frames, 13);
    assert_eq!(
        s.fighter.down_smash,
        MoveFrames {
            first: 15,
            last: 28
        }
    );
    let mac = catalog.profile(0x34).unwrap();
    assert_eq!(mac.name, "littlemac");
    assert_eq!(mac.fighter_kind, 0x34);
    for slot in [PresetSlot::Neutral, PresetSlot::Jump] {
        let list = mac.menu_options.list(slot);
        assert_eq!(list.options[list.selected].id, "up_tilt");
    }
    let attack = mac.menu_options.list(PresetSlot::Attack);
    assert_eq!(attack.options[attack.selected].id, "shield_up_b");
    assert_eq!(mac.ledge.up_special_startup, Some(3));
    assert_eq!(mac.ledge.neutral_air, MoveFrames { first: 2, last: 2 });
    assert_eq!(mac.ledge.normal_hit_lead_frames, 0);
    assert_eq!(
        mac.ledge.down_smash_back,
        Some(MoveFrames {
            first: 17,
            last: 18
        })
    );
    for profile in [pac, mac] {
        for list in profile.menu_options.lists {
            assert!(!list.options.is_empty());
            assert!(list.options.iter().all(|choice| choice.id != "off"));
        }
    }
}
