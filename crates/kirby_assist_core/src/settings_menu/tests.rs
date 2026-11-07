fn file_profiles() -> crate::fighter_profiles::FighterProfiles {
    static PROFILES: std::sync::OnceLock<crate::fighter_profiles::FighterProfiles> =
        std::sync::OnceLock::new();
    PROFILES
        .get_or_init(|| {
            let catalog = crate::sequence_config::Catalog::parse(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../deployment/ultimate/kirby_assist/assembled/sequences.txt"
            )))
            .unwrap()
            .for_build(cfg!(feature = "diagnostic-logging"));
            let mut profiles = crate::fighter_profiles::FighterProfiles::default();
            profiles.add_sequence_choices(&catalog).unwrap();
            profiles
        })
        .clone()
}
use super::*;

#[test]
fn quick_selector_cycles_saved_recipes_only_and_cannot_edit_or_manage() {
    let catalog = crate::sequence_config::Catalog::parse("[yoshi.neutral.first]\nstart_delay=0\nactions=a\n[yoshi.neutral.second]\nstart_delay=2\nactions=b\n").unwrap();
    let mut profiles = crate::fighter_profiles::FighterProfiles::default();
    profiles.add_sequence_choices(&catalog).unwrap();
    let p = &profiles.profile(5).unwrap().menu_options;
    let mut menu = MenuSelection::new(p);
    let tap = |m: &mut MenuSelection, button| {
        m.input(0, p);
        m.input(button, p);
    };
    tap(&mut menu, MAPPER_MINUS);
    assert!(menu.quick_menu && !menu.menu_hidden);
    assert_eq!(menu.row, MenuRow::Neutral);
    tap(&mut menu, crate::manual_controls::DPAD_LEFT);
    assert_eq!(menu.choices[0], 1);
    tap(&mut menu, DPAD_RIGHT);
    assert_eq!(menu.choices[0], 0);
    for button in [MAPPER_Y, crate::native_input::MAPPER_B] {
        tap(&mut menu, button);
        assert!(!menu.recipe_editing && !menu.managing());
    }
    for row in [
        MenuRow::Jump,
        MenuRow::Roll,
        MenuRow::Attack,
        MenuRow::Drop,
        MenuRow::ButtonsPage,
    ] {
        tap(&mut menu, DPAD_DOWN);
        assert_eq!(menu.row, row);
    }
    let choices = menu.choices;
    tap(&mut menu, crate::manual_controls::DPAD_LEFT);
    assert!(menu.button_menu && menu.quick_menu);
    assert_eq!(menu.choices, choices);
    menu.row = MenuRow::ModesPage;
    tap(&mut menu, DPAD_RIGHT);
    assert!(menu.modes_menu && !menu.quick_menu);
    menu.row = MenuRow::LedgePage;
    tap(&mut menu, crate::manual_controls::DPAD_LEFT);
    assert!(menu.quick_menu && !menu.button_menu);
    assert_eq!(menu.row, MenuRow::Neutral);
    tap(&mut menu, MENU_STICK_CLICK);
    assert!(menu.recipe_editing);
    tap(&mut menu, MAPPER_MINUS);
    assert!(menu.menu_hidden && !menu.recipe_editing);
    tap(&mut menu, MAPPER_MINUS);
    assert!(menu.quick_menu && !menu.button_menu && !menu.modes_menu);
    assert_eq!(menu.choices, choices);
}

#[test]
fn diagnostic_only_rows_are_available_only_in_diagnostics() {
    let profiles = file_profiles();
    let presets = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.show_modes();
    menu.row = MenuRow::AutoTech;
    menu.input(0, presets);
    menu.input(DPAD_DOWN, presets);
    assert_eq!(menu.row, MenuRow::AutoMash);
    assert_eq!(
        menu.row_visible(MenuRow::Performance),
        cfg!(feature = "diagnostic-logging")
    );
    assert!(menu.row_visible(MenuRow::GrabDodge));
    assert!(!menu.auto_trump && !menu.performance && menu.grab_dodge && menu.out_of_shield);
    assert!(menu.spot_punish && menu.auto_tech);
}

#[test]
fn spot_punish_starts_on_and_is_reachable_and_toggleable_in_both_builds() {
    let profiles = file_profiles();
    let presets = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.show_modes();
    menu.row = MenuRow::RollRules;
    menu.input(0, presets);
    menu.input(DPAD_DOWN, presets);
    assert_eq!(menu.row, MenuRow::ShieldPressure);
    menu.input(0, presets);
    menu.input(DPAD_DOWN, presets);
    assert_eq!(menu.row, MenuRow::SpotPunish);
    assert!(menu.spot_punish);
    menu.input(0, presets);
    menu.input(DPAD_RIGHT, presets);
    assert!(!menu.spot_punish);
    menu.cancel_actions();
    assert!(!menu.spot_punish);
}

#[test]
fn auto_tech_toggles_once_and_survives_input_reset() {
    let profiles = file_profiles();
    let presets = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.show_modes();
    menu.row = MenuRow::AutoTech;
    menu.input(0, presets);
    assert!(menu.auto_tech);
    menu.input(DPAD_RIGHT, presets);
    assert!(!menu.auto_tech && menu.cancel_pending);
    menu.input(DPAD_RIGHT, presets);
    assert!(!menu.auto_tech);
    menu.cancel_actions();
    assert!(!menu.auto_tech);
    menu.input(0, presets);
    menu.input(crate::manual_controls::DPAD_LEFT, presets);
    assert!(menu.auto_tech);
}

#[test]
fn yoshi_jab_lock_starts_off_and_is_a_separate_menu_toggle() {
    let profiles = file_profiles();
    let presets = &profiles.profile(5).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.show_modes();
    menu.y_fighter_kind = 5;
    menu.input(0, presets);
    assert!(!menu.jab_lock);
    let roll = menu.roll_read;
    menu.row = MenuRow::JabLock;
    menu.input(DPAD_RIGHT, presets);
    assert!(menu.jab_lock && menu.cancel_pending);
    assert_eq!(menu.roll_read, roll);
    menu.input(0, presets);
    menu.input(crate::manual_controls::DPAD_LEFT, presets);
    assert!(!menu.jab_lock);
    // Kirby navigation skips the Yoshi-only row.
    menu.y_fighter_kind = 6;
    menu.row = MenuRow::JabGrab;
    menu.input(0, presets);
    menu.input(DPAD_DOWN, presets);
    assert_ne!(menu.row, MenuRow::JabLock);
}

#[test]
fn kirby_helpers_default_on_and_classic_monitor_chord_does_not_toggle() {
    let profiles = file_profiles();
    let presets = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.show_modes();
    menu.input(0, presets);
    assert!(menu.idle_crouch && menu.tilt_turn && menu.jab_grab == crate::jab_grab::Followup::Grab);
    menu.row = MenuRow::IdleCrouch;
    menu.input(DPAD_RIGHT, presets);
    assert!(!menu.idle_crouch && menu.cancel_pending);
    menu.input(0, presets);
    menu.row = MenuRow::TiltTurn;
    menu.input(crate::manual_controls::DPAD_LEFT, presets);
    assert!(!menu.tilt_turn);
    menu.input(0, presets);
    menu.row = MenuRow::JabGrab;
    menu.input(DPAD_RIGHT, presets);
    assert_eq!(menu.jab_grab, crate::jab_grab::Followup::DownSmash);
    menu.input(0, presets);
    menu.input(DPAD_RIGHT, presets);
    assert_eq!(menu.jab_grab, crate::jab_grab::Followup::Grab);
    let hidden = menu.menu_hidden;
    menu.input(0, presets);
    menu.input(MAPPER_L | DPAD_DOWN | MENU_STICK_CLICK, presets);
    assert_eq!(menu.menu_hidden, hidden);
    menu.input(0, presets);
    menu.input(MENU_TOGGLE_CHORD, presets);
    assert_ne!(menu.menu_hidden, hidden);
}

#[test]
fn minus_alone_toggles_once_and_l3_alone_does_not() {
    let profiles = file_profiles();
    let presets = &profiles.profile(5).unwrap().menu_options;
    for first in [0, TWO_FRAME_TRIGGER] {
        let mut menu = MenuSelection::new(presets);
        menu.input(0, presets);
        menu.input(first, presets);
        assert!(menu.menu_hidden);
        menu.input(MAPPER_MINUS, presets);
        assert!(!menu.menu_hidden && menu.quick_menu);
        let revision = menu.revision;
        menu.input(MAPPER_MINUS, presets);
        assert_eq!(menu.revision, revision);
        menu.input(0, presets);
        menu.input(MAPPER_MINUS, presets);
        assert!(menu.menu_hidden);
    }
}

#[test]
fn minus_toggles_menu_once_without_toggling_mash() {
    let profiles = file_profiles();
    let p = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(p);
    menu.input(0, p);
    let choices = menu.choices;
    menu.input(MAPPER_MINUS, p);
    let revision = menu.revision;
    for _ in 0..10 {
        menu.input(MAPPER_MINUS, p);
        assert_eq!(menu.revision, revision);
    }
    menu.input(0, p);
    menu.input(MAPPER_MINUS, p);
    assert!(menu.menu_hidden && !menu.toggle_mash);
    menu.input(0, p);
    menu.input(MAPPER_L | MAPPER_MINUS, p);
    assert!(!menu.menu_hidden && !menu.toggle_mash);
    assert_eq!(menu.choices, choices);
}

#[test]
fn minus_closes_after_editing_and_reopens_with_saved_choices() {
    let profiles = file_profiles();
    let p = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(p);
    menu.show_quick();
    let tap = |m: &mut MenuSelection, b| {
        m.input(0, p);
        m.input(b, p);
    };
    tap(&mut menu, MENU_STICK_CLICK);
    tap(&mut menu, DPAD_RIGHT);
    assert!(menu.recipe_modified(PresetSlot::Neutral));
    let choices = menu.choices;
    tap(&mut menu, MAPPER_MINUS);
    assert!(menu.menu_hidden && !menu.recipe_editing);
    for _ in 0..4 {
        menu.input(MAPPER_MINUS, p);
        assert!(menu.menu_hidden);
    }
    tap(&mut menu, MAPPER_MINUS);
    assert!(menu.quick_menu && !menu.button_menu);
    assert_eq!(menu.choices, choices);
    assert!(menu.recipe_modified(PresetSlot::Neutral));
}

#[test]
fn mash_is_a_menu_row_and_pad_release_does_not_toggle_twice() {
    assert_eq!(MenuRow::AutoMash.label(), "Auto-mash");
    let profiles = file_profiles();
    let presets = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.show_modes();
    let initial = menu.settings(presets, AssistSettings::default());
    menu.input(0, presets);
    menu.row = MenuRow::AutoMash;
    menu.input(DPAD_RIGHT, presets);
    assert!(menu.toggle_mash && menu.cancel_pending);
    assert_eq!(
        menu.settings(presets, AssistSettings::default())
            .response_choices,
        initial.response_choices
    );
    menu.input(DPAD_RIGHT, presets);
    assert!(!menu.toggle_mash);
    menu.input(0, presets);
    menu.input(crate::manual_controls::DPAD_LEFT, presets);
    assert!(menu.toggle_mash);
    menu.input(0, presets);
    menu.input(MENU_TOGGLE_CHORD, presets);
    assert!(menu.menu_hidden);
    menu.input(0, presets);
    menu.input(DPAD_RIGHT, presets);
    assert!(
        !menu.toggle_mash,
        "hidden sides belong to gameplay, not settings"
    );
}

#[test]
fn visible_pad_edits_only_and_close_quarantines_held_directions() {
    let profiles = file_profiles();
    let p = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(p);
    menu.show();
    menu.input(0, p);
    let before = menu.choices;
    assert_ne!(menu.input(DPAD_DOWN, p) & DPAD_DOWN, 0);
    assert_ne!(menu.choices, before);
    menu.input(MAPPER_MINUS | DPAD_DOWN, p);
    assert!(menu.menu_hidden);
    assert_ne!(menu.input(DPAD_DOWN, p) & DPAD_DOWN, 0);
    assert_ne!(menu.input(0, p) & DPAD_DOWN, 0);
    assert_eq!(menu.input(DPAD_DOWN, p) & DPAD_DOWN, 0); // Normal Down passes after menu quarantine releases.
}

#[test]
fn stock_reset_preserves_choices_and_force_show_resumes_visible_menu() {
    let profiles = file_profiles();
    let p = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(p);
    menu.show();
    menu.input(0, p);
    menu.input(DPAD_RIGHT, p);
    assert_eq!(menu.row, MenuRow::Jump);
    menu.cancel_actions();
    menu.input(DPAD_RIGHT, p);
    assert_eq!(menu.row, MenuRow::Jump);
    menu.input(MAPPER_MINUS, p);
    assert!(menu.menu_hidden);
    let choices = menu.choices;
    menu.show();
    assert!(!menu.menu_hidden && menu.cancel_pending);
    menu.begin_match();
    assert!(menu.menu_hidden);
    assert_eq!(menu.choices, choices);
    menu.observe_passthrough(0);
    menu.input(0, p);
    menu.input(MAPPER_L, p);
    menu.input(MAPPER_L, p);
    assert!(menu.ledge_held(MAPPER_L));
}

#[test]
fn y_remains_available_to_its_mode_but_is_not_a_menu_or_ledge_trigger() {
    let profiles = file_profiles();
    let presets = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.show();
    menu.auto_duck_available = true;
    menu.input(0, presets);
    let consumed = menu.input(MAPPER_Y | DPAD_DOWN, presets);
    assert_eq!(
        observation_buttons(MAPPER_Y | DPAD_DOWN, consumed),
        0 // Menu-owned Y and D-pad must not reach gameplay observers.
    );
    assert!(!menu.ledge_held(MAPPER_Y));
    menu.input(0, presets);
    menu.input(MAPPER_Y, presets);
    assert!(!menu.finisher_held(MAPPER_Y));
    assert!(!menu.auto_duck_held(MAPPER_Y));
    assert_eq!(MenuRow::ALL.len(), 35);
    assert_eq!(
        observation_buttons(SHORT_HOP_TRIGGER, SHORT_HOP_TRIGGER),
        0 // A menu-owned X test must not also start the short-hop helper.
    );
}

#[test]
fn minus_is_menu_and_does_not_disable_ledge_autopilot_after_release() {
    let profiles = file_profiles();
    let presets = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.input(0, presets);
    assert_eq!(
        menu.input(MENU_TOGGLE_CHORD, presets) & MENU_TOGGLE_CHORD,
        MENU_TOGGLE_CHORD
    );
    menu.input(MAPPER_L | MENU_TOGGLE_CHORD, presets);
    menu.input(MAPPER_L, presets);
    assert!(menu.ledge_held(MAPPER_L)); // Compact selection preserves held-L autopilot.
    menu.input(0, presets);
    menu.input(MENU_TOGGLE_CHORD, presets); // Modes.
    menu.input(0, presets);
    menu.input(MENU_TOGGLE_CHORD, presets); // Closed.
    menu.input(0, presets);
    menu.input(MAPPER_L, presets);
    menu.input(MAPPER_L, presets);
    assert!(menu.ledge_held(MAPPER_L));
}

#[test]
fn editor_and_modes_pages_keep_navigation_and_settings_separate() {
    let profiles = file_profiles();
    let p = &profiles.profile(5).unwrap().menu_options;
    let mut m = MenuSelection::new(p);
    m.y_fighter_kind = 5;
    m.show();
    for _ in 0..14 {
        assert!(m.row.slot().is_some() || MenuSelection::page_link(m.row));
        m.input(0, p);
        m.input(DPAD_RIGHT, p);
    }
    m.open_page(MenuRow::ModesPage);
    let choices = m.choices;
    m.row = MenuRow::AutoTech;
    let before = m.auto_tech;
    m.input(0, p);
    m.input(crate::manual_controls::DPAD_LEFT, p);
    assert_eq!(m.auto_tech, !before);
    for _ in 0..24 {
        assert!(m.row.slot().is_none() && m.row_visible(m.row));
        m.input(0, p);
        m.input(DPAD_DOWN, p);
    }
    m.row = MenuRow::AutoTech;
    m.input(0, p);
    m.input(MENU_STICK_CLICK, p);
    assert!(!m.recipe_editing && m.preview_slot.is_none());
    assert_eq!(m.choices, choices);
    m.input(0, p);
    m.input(MAPPER_MINUS, p);
    assert!(m.menu_hidden);
}

#[test]
fn latency_trial_label_is_diagnostic_only_and_does_not_change_recipe_timing() {
    let profiles = file_profiles();
    let p = &profiles.profile(6).unwrap().menu_options;
    let mut m = MenuSelection::new(p);
    m.show_modes();
    assert_eq!(
        m.row_visible(MenuRow::TestLatency),
        cfg!(feature = "diagnostic-logging")
    );
    if cfg!(feature = "diagnostic-logging") {
        let before = m.settings(p, Default::default());
        m.row = MenuRow::TestLatency;
        m.input(0, p);
        m.input(DPAD_RIGHT, p);
        assert_eq!(m.test_latency, 1); // External 0f, not a timing adjustment.
        let after = m.settings(p, Default::default());
        assert_eq!(before.sequence_overrides, after.sequence_overrides);
        assert_eq!(before.response_choices, after.response_choices);
    }
}

#[test]
fn fighter_settings_edit_reset_save_and_training_gate() {
    let profile = crate::fighter_profiles::yoshi::profile();
    let mut m = MenuSelection::new(&profile.menu_options);
    m.tuning_values =
        crate::fighter_tuning::Tuning::values(profile.apply(AssistSettings::default()));
    m.input(0, &profile.menu_options);
    m.open_page(MenuRow::FighterSettings);
    assert!(m.tuning_page && m.recipe_editing);
    assert!(!m.ledge_held(MAPPER_L));
    m.input(DPAD_RIGHT, &profile.menu_options);
    assert!(m.tuning.0[0].is_some());
    assert_eq!(
        m.settings(
            &profile.menu_options,
            profile.apply(AssistSettings::default())
        )
        .walk_strength,
        m.tuning.0[0].unwrap()
    );
    m.input(0, &profile.menu_options);
    m.input(MAPPER_Y, &profile.menu_options);
    assert!(m.tuning_save_requested && !m.save_requested);
    m.input(0, &profile.menu_options);
    m.input(MAPPER_X, &profile.menu_options);
    assert_eq!(m.tuning.0[0], None);
    m.input_with_editing(0, &profile.menu_options, false);
    assert!(!m.tuning_page);
    assert!(!m.row_visible(MenuRow::FighterSettings));
    m.input_with_editing(MAPPER_Y, &profile.menu_options, false);
    assert!(!m.tuning_save_requested);
}

#[test]
fn fighter_settings_save_requires_y_alone_and_keeps_hints_after_status() {
    let profile = crate::fighter_profiles::yoshi::profile();
    let mut m = MenuSelection::new(&profile.menu_options);
    m.input(0, &profile.menu_options);
    m.open_page(MenuRow::FighterSettings);
    m.input(MAPPER_Y | MAPPER_L, &profile.menu_options);
    assert!(!m.tuning_save_requested);
    assert!(m.save_status.as_ref().unwrap().contains("Release other"));
    let lines = m.recipe_panel(&profile.menu_options).footer.join(" ");
    assert!(lines.contains("Y alone: save") && lines.contains("R3: back"));
    m.input(0, &profile.menu_options);
    m.input(MAPPER_Y, &profile.menu_options);
    assert!(m.tuning_save_requested);
}

#[test]
fn custom_roll_toggle_never_reactivates_a_retired_fixed_roll_helper() {
    for kind in [
        crate::fighter_kinds::YOSHI,
        crate::fighter_kinds::KIRBY,
        crate::fighter_kinds::BOWSER,
    ] {
        let profiles = file_profiles();
        let presets = &profiles.profile(kind).unwrap().menu_options;
        let mut menu = MenuSelection::new(presets);
        menu.y_fighter_kind = kind;
        menu.roll_read_available = true;
        assert_eq!(menu.roll_read, crate::roll_read::Choice::Off);
        assert!(!menu.row_visible(MenuRow::RollRead));
        menu.show_modes();
        menu.row = MenuRow::RollRules;
        menu.input(0, presets);
        menu.input(DPAD_LEFT, presets);
        assert!(!menu.roll_rules);
        assert_eq!(menu.roll_read, crate::roll_read::Choice::Off);
        for i in crate::fighter_tuning::PUNISH_DOWN_REACH..=crate::fighter_tuning::ROLL_GRAB_WATCH {
            assert!(!crate::fighter_tuning::Tuning::visible(kind, i));
        }
    }
}
