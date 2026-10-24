use super::*;
use crate::fighter_profiles::FighterProfiles;
use crate::manual_controls::DPAD_LEFT;

#[test]
fn yoshi_jab_lock_starts_off_and_is_a_separate_menu_toggle() {
    let profiles = FighterProfiles::default();
    let presets = &profiles.profile(5).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.y_fighter_kind = 5;
    menu.input(0, presets);
    assert!(!menu.jab_lock);
    let roll = menu.roll_read;
    menu.row = MenuRow::JabLock;
    menu.input(DPAD_RIGHT, presets);
    assert!(menu.jab_lock && menu.cancel_pending);
    assert_eq!(menu.roll_read, roll);
    menu.input(0, presets);
    menu.input(DPAD_LEFT, presets);
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
    let profiles = FighterProfiles::default();
    let presets = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.input(0, presets);
    assert!(menu.idle_crouch && menu.tilt_turn && menu.jab_grab == crate::jab_grab::Followup::Grab);
    menu.row = MenuRow::IdleCrouch;
    menu.input(DPAD_RIGHT, presets);
    assert!(!menu.idle_crouch && menu.cancel_pending);
    menu.input(0, presets);
    menu.row = MenuRow::TiltTurn;
    menu.input(DPAD_LEFT, presets);
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
    menu.input(MENU_STICK_CLICK, presets);
    assert_ne!(menu.menu_hidden, hidden);
}

#[test]
fn r3_alone_toggles_once_and_l3_alone_does_not() {
    let profiles = FighterProfiles::default();
    let presets = &profiles.profile(5).unwrap().menu_options;
    for first in [0, TWO_FRAME_TRIGGER] {
        let mut menu = MenuSelection::new(presets);
        menu.input(0, presets);
        menu.input(first, presets);
        assert!(!menu.menu_hidden);
        menu.input(MENU_TOGGLE_CHORD, presets);
        assert!(menu.menu_hidden);
        menu.input(MENU_TOGGLE_CHORD, presets);
        assert!(menu.menu_hidden);
        menu.input(0, presets);
        menu.input(MENU_TOGGLE_CHORD, presets);
        assert!(!menu.menu_hidden);
    }
    let menu = MenuSelection::new(presets);
    assert_eq!(presets.lists[2].options[menu.choices[2]].id, "grab");
    assert_eq!(presets.lists[3].options[menu.choices[3]].id, "shield_grab");
}

#[test]
fn r3_toggles_menu_once_without_toggling_mash() {
    let profiles = FighterProfiles::default();
    let presets = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.input(0, presets);
    let choices = menu.choices;
    assert_ne!(
        menu.input(MENU_TOGGLE_CHORD, presets) & MENU_TOGGLE_CHORD,
        0
    );
    assert!(menu.menu_hidden && !menu.toggle_mash);
    let revision = menu.revision;
    for _ in 0..10 {
        menu.input(MENU_TOGGLE_CHORD, presets);
        assert_eq!(menu.revision, revision);
    }
    menu.input(0, presets);
    menu.input(MENU_TOGGLE_CHORD, presets);
    assert!(!menu.menu_hidden && !menu.toggle_mash);
    menu.input(0, presets);
    menu.input(MAPPER_L | MENU_STICK_CLICK, presets);
    assert!(menu.menu_hidden, "R3 must toggle even while L is held");
    assert_eq!(menu.choices, choices);
}

#[test]
fn r3_closes_after_editing_and_reopens_with_saved_choices() {
    let profiles = FighterProfiles::default();
    let presets = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.input(0, presets);
    menu.input(MENU_TOGGLE_CHORD, presets);
    assert!(menu.menu_hidden);
    menu.input(0, presets);
    menu.input(MENU_TOGGLE_CHORD, presets);
    assert!(!menu.menu_hidden);
    menu.input(0, presets);
    menu.input(DPAD_RIGHT, presets);
    let edited = menu.choices;
    menu.input(0, presets);
    menu.input(MENU_TOGGLE_CHORD, presets);
    for _ in 0..4 {
        menu.cancel_actions();
        menu.input(MENU_TOGGLE_CHORD, presets);
        assert!(menu.menu_hidden);
    }
    menu.input(0, presets);
    menu.input(MENU_TOGGLE_CHORD, presets);
    assert!(!menu.menu_hidden);
    assert_eq!(menu.choices, edited);
}

#[test]
fn mash_is_a_menu_row_and_pad_release_does_not_toggle_twice() {
    assert_eq!(MenuRow::AutoMash.label(), "Auto-mash");
    let profiles = FighterProfiles::default();
    let presets = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    let initial = menu.settings(presets, LedgeSettings::default());
    menu.input(0, presets);
    menu.row = MenuRow::AutoMash;
    menu.input(DPAD_RIGHT, presets);
    assert!(menu.toggle_mash && menu.cancel_pending);
    assert_eq!(
        menu.settings(presets, LedgeSettings::default())
            .response_choices,
        initial.response_choices
    );
    menu.input(DPAD_RIGHT, presets);
    assert!(!menu.toggle_mash);
    menu.input(0, presets);
    menu.input(DPAD_LEFT, presets);
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
    let profiles = FighterProfiles::default();
    let presets = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.input(0, presets);
    let before = menu.choices;
    assert_ne!(menu.input(DPAD_RIGHT, presets) & DPAD_RIGHT, 0);
    assert_ne!(menu.choices, before);
    menu.input(MENU_TOGGLE_CHORD | DPAD_RIGHT, presets);
    assert!(menu.menu_hidden);
    assert_ne!(menu.input(DPAD_RIGHT, presets) & DPAD_RIGHT, 0);
    assert_ne!(menu.input(0, presets) & DPAD_RIGHT, 0);
    assert_eq!(menu.input(DPAD_RIGHT, presets) & DPAD_RIGHT, 0);
    assert_eq!(menu.input(DPAD_DOWN, presets) & DPAD_DOWN, 0);
}

#[test]
fn stock_reset_preserves_choices_and_force_show_resumes_visible_menu() {
    let profiles = FighterProfiles::default();
    let presets = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.input(0, presets);
    menu.input(DPAD_DOWN, presets);
    assert_eq!(menu.row, MenuRow::Jump);
    menu.cancel_actions();
    menu.input(DPAD_DOWN, presets);
    assert_eq!(menu.row, MenuRow::Jump);
    menu.input(MENU_TOGGLE_CHORD, presets);
    let choices = menu.choices;
    assert!(menu.menu_hidden);
    menu.show();
    assert!(!menu.menu_hidden && menu.cancel_pending);
    menu.begin_match();
    menu.observe_passthrough(0);
    assert!(!menu.menu_hidden);
    assert_eq!(menu.choices, choices);
}

#[test]
fn y_remains_available_to_its_mode_but_is_not_a_menu_or_ledge_trigger() {
    let profiles = FighterProfiles::default();
    let presets = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.auto_duck_available = true;
    menu.input(0, presets);
    let consumed = menu.input(MAPPER_Y | DPAD_RIGHT, presets);
    assert_eq!(
        observation_buttons(MAPPER_Y | DPAD_RIGHT, consumed),
        MAPPER_Y
    );
    assert!(!menu.ledge_held(MAPPER_Y));
    menu.input(0, presets);
    menu.input(MAPPER_Y, presets);
    assert!(menu.finisher_held(MAPPER_Y));
    assert!(!menu.auto_duck_held(MAPPER_Y));
    assert_eq!(MenuRow::ALL.len(), 14);
    assert_eq!(
        observation_buttons(SHORT_HOP_TRIGGER, SHORT_HOP_TRIGGER),
        SHORT_HOP_TRIGGER
    );
}

#[test]
fn normal_r3_is_menu_and_does_not_disable_ledge_autopilot() {
    let profiles = FighterProfiles::default();
    let presets = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.input(0, presets);
    assert_eq!(
        menu.input(MENU_TOGGLE_CHORD, presets) & MENU_TOGGLE_CHORD,
        MENU_TOGGLE_CHORD
    );
    menu.input(MAPPER_L | MENU_TOGGLE_CHORD, presets);
    menu.input(MAPPER_L | MENU_TOGGLE_CHORD, presets);
    assert!(menu.ledge_held(MAPPER_L | MENU_TOGGLE_CHORD));
}
