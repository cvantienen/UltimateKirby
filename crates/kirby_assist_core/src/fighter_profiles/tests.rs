//! File-only menu and reference-data invariants.
use super::*;
use crate::{live_input::LiveConfig, settings_menu::MenuSelection};
#[test]
fn registered_fighters_have_no_embedded_options_or_scheduling_fallback() {
    let profiles = FighterProfiles::default();
    assert_eq!(profiles.profiles.len(), 79);
    profiles.validate().unwrap();
    for profile in profiles.profiles.values() {
        assert!(profile
            .menu_options
            .lists
            .iter()
            .all(|list| list.options.is_empty()));
        assert!(
            profile
                .apply(AssistSettings {
                    enabled: true,
                    ..AssistSettings::default()
                })
                .enabled
        );
    }
}
#[test]
fn startup_builds_every_fighter_and_five_menus_from_the_file_only() {
    let profiles = FighterProfiles::from_test_file();
    profiles.validate().unwrap();
    for profile in profiles.profiles.values() {
        for list in profile.menu_options.lists.iter().take(5) {
            assert!(!list.options.is_empty(), "{}", profile.name);
            assert!(list.selected < list.options.len());
            assert!(list.options.iter().all(|choice| choice.sequence.len != 0));
        }
    }
}
#[test]
fn non_ledge_numeric_settings_cannot_override_selected_file_scheduling() {
    let config = LiveConfig::parse("ledge_trap=true\nledge_nair_hit_lead_frames=19\nledge_stone_startup_frames=13\nledge_stone_hit_lead_frames=1").unwrap();
    let profiles = FighterProfiles::from_test_file();
    let profile = profiles.profile(crate::fighter_kinds::KIRBY).unwrap();
    let menu = MenuSelection::new(&profile.menu_options);
    let a = menu.settings(&profile.menu_options, profile.apply(config.assist));
    let b = menu.settings(
        &profile.menu_options,
        profile.apply(AssistSettings {
            enabled: true,
            ..AssistSettings::default()
        }),
    );
    assert_eq!(a.sequence_overrides, b.sequence_overrides);
}
#[test]
fn file_menu_rebuild_replaces_removed_sections_instead_of_retaining_choices() {
    let mut profiles = FighterProfiles::from_test_file();
    let file =
        crate::sequence_config::Catalog::parse("[kirby.neutral.only]\nstart_delay=2\nactions=jab")
            .unwrap();
    profiles.add_sequence_choices(&file).unwrap();
    let kirby = profiles.profile(crate::fighter_kinds::KIRBY).unwrap();
    assert_eq!(kirby.menu_options.lists[0].options.len(), 1);
    assert_eq!(kirby.menu_options.lists[0].options[0].id, "only");
    assert!(kirby.menu_options.lists[1].options.is_empty());
    assert!(profiles
        .profile(crate::fighter_kinds::YOSHI)
        .unwrap()
        .menu_options
        .lists
        .iter()
        .all(|list| list.options.is_empty()));
}
#[test]
fn retired_ledge_timing_keys_are_ignored_in_older_config_files() {
    for key in [
        "ledge_nair_hit_lead_frames",
        "ledge_stone_hit_lead_frames",
        "ledge_stone_startup_frames",
        "ledge_drop_request_timeout_frames",
        "ledge_roll_hit_window_frames",
        "ledge_grab_buffer_frames",
        "ledge_getup_attack",
    ] {
        assert!(LiveConfig::parse(&format!("{key}=retired")).is_ok());
    }
}

#[test]
fn invalid_fighter_grab_data_fails_before_hooks_install() {
    let mut catalog = FighterProfiles::default();
    catalog
        .profiles
        .get_mut(&crate::fighter_kinds::KIRBY)
        .unwrap()
        .grab
        .grab
        .standing_range = f32::NAN;
    assert!(catalog.validate().is_err());
}

#[test]
fn profile_grab_facts_follow_the_matching_reference_row() {
    let profiles = FighterProfiles::default();

    for profile in profiles.profiles.values() {
        let grab_data = profile.grab.grab;

        if let Some(reference) =
            crate::fighter_data::ordinary_grab(profile.fighter_kind, crate::native_states::CATCH)
        {
            assert_eq!(
                grab_data.standing_startup, reference.first,
                "{}",
                profile.name
            );
            assert_eq!(
                grab_data.standing_active_frames,
                reference.last - reference.first + 1,
                "{}",
                profile.name
            );
            assert_eq!(
                grab_data.standing_range,
                reference.reach.unwrap_or(0.0),
                "{}",
                profile.name
            );
        }

        for (status, reference_frame, profile_frame) in [
            (
                crate::native_states::CATCH_DASH,
                crate::fighter_data::ordinary_grab(
                    profile.fighter_kind,
                    crate::native_states::CATCH_DASH,
                ),
                grab_data.dash_startup,
            ),
            (
                crate::native_states::CATCH_TURN,
                crate::fighter_data::ordinary_grab(
                    profile.fighter_kind,
                    crate::native_states::CATCH_TURN,
                ),
                grab_data.pivot_startup,
            ),
        ] {
            if let Some(reference) = reference_frame {
                assert_eq!(
                    profile_frame, reference.first,
                    "{} status {status:#x}",
                    profile.name
                );
            }
        }

        let reference_shield_grab = profile
            .fighter_data()
            .and_then(|reference| reference.stat("outofshield.grabpostshieldstun"))
            .and_then(|frame| frame.parse::<u8>().ok())
            .filter(|frame| (1..=120).contains(frame));
        if let Some(reference_frame) = reference_shield_grab {
            assert_eq!(
                grab_data.shield_startup, reference_frame,
                "{}",
                profile.name
            );
        }
    }

    assert!(profiles.profile(-1).is_none());
    assert!(profiles
        .profile(crate::fighter_kinds::PLAYABLE_KIND_COUNT)
        .is_none());
}
