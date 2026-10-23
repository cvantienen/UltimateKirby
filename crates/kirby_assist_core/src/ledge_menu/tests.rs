use super::*;
use crate::fighter_profiles::FighterProfiles;
use crate::manual_controls::DPAD_LEFT;

#[test]
fn y_is_reserved_for_b_reverse_and_does_not_run_ledge_or_two_frame() {
    let catalog = FighterProfiles::default();
    let presets = &catalog.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.input(0, presets);
    assert_ne!(menu.input(MAPPER_Y, presets) & MAPPER_Y, 0);
    assert!(
        !menu.cancel_pending,
        "Y must not cancel an active slide/trump"
    );
    menu.input(MAPPER_L | MAPPER_Y, presets);
    menu.input(MAPPER_L | MAPPER_Y, presets);
    assert!(!menu.ledge_held(MAPPER_L | MAPPER_Y));
    menu.input(MAPPER_L, presets);
    assert!(menu.ledge_held(MAPPER_L));
    assert!(!menu.ledge_held(MAPPER_L | SHORT_HOP_TRIGGER));
    assert_eq!(MenuRow::ALL.len(), 5);
}

#[test]
fn visible_pad_edits_only_and_hidden_pad_returns_to_gameplay_after_release() {
    let catalog = FighterProfiles::default();
    let presets = &catalog.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.input(0, presets);
    let before = menu.choices;
    let consumed = menu.input(MAPPER_L | DPAD_RIGHT, presets);
    assert_ne!(menu.choices, before);
    assert_ne!(consumed & DPAD_RIGHT, 0);
    assert!(!menu.ledge_held(MAPPER_L | DPAD_RIGHT));
    menu.input(MAPPER_L | DPAD_RIGHT | MENU_STICK_CLICK, presets);
    assert!(menu.menu_hidden);
    assert_ne!(menu.input(DPAD_RIGHT, presets) & DPAD_RIGHT, 0);
    assert_ne!(menu.input(0, presets) & DPAD_RIGHT, 0); // Swallow release edge.
    assert_eq!(menu.input(DPAD_RIGHT, presets) & DPAD_RIGHT, 0);
    assert_eq!(menu.input(DPAD_LEFT, presets) & DPAD_LEFT, 0);
    assert_eq!(menu.input(DPAD_UP, presets) & DPAD_UP, 0);
    assert_eq!(menu.input(DPAD_DOWN, presets) & DPAD_DOWN, 0);
}

#[test]
fn stock_reset_preserves_choices_visibility_and_menu_press_history() {
    let catalog = FighterProfiles::default();
    let presets = &catalog.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.input(0, presets);
    menu.input(DPAD_DOWN, presets);
    assert_eq!(menu.row, MenuRow::Jump);
    menu.cancel_actions();
    menu.input(DPAD_DOWN, presets);
    assert_eq!(menu.row, MenuRow::Jump);
    menu.input(MAPPER_L | MENU_STICK_CLICK, presets);
    let choices = menu.choices;
    menu.cancel_actions();
    assert!(menu.menu_hidden);
    menu.begin_match();
    assert!(!menu.menu_hidden);
    assert_eq!(menu.choices, choices);
}

#[test]
fn menu_chord_works_in_either_order_once_per_press_and_consumes_r3_release() {
    let catalog = FighterProfiles::default();
    let presets = &catalog.profile(6).unwrap().menu_options;
    for first in [MAPPER_L, MENU_STICK_CLICK] {
        let mut menu = MenuSelection::new(presets);
        menu.input(0, presets);
        menu.input(first, presets);
        assert!(!menu.menu_hidden);
        let consumed = menu.input(MENU_CHORD, presets);
        assert!(menu.menu_hidden);
        assert_ne!(consumed & MENU_STICK_CLICK, 0);
        assert!(!menu.ledge_held(MENU_CHORD));
        assert!(!menu.toggle_mash);
        let revision = menu.revision;
        for _ in 0..10 {
            menu.input(MENU_CHORD, presets);
            assert_eq!(menu.revision, revision);
            assert!(!menu.ledge_held(MENU_CHORD));
        }
        assert_ne!(menu.input(0, presets) & MENU_STICK_CLICK, 0);
        assert_eq!(menu.input(MENU_STICK_CLICK, presets) & MENU_STICK_CLICK, 0);
        assert!(menu.menu_hidden, "R3 alone must not toggle visibility");
        menu.input(MENU_CHORD, presets);
        assert!(!menu.menu_hidden);
    }
}

#[test]
fn minus_requests_one_mash_toggle_without_changing_menu_visibility_or_choices() {
    let catalog = FighterProfiles::default();
    let presets = &catalog.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.input(0, presets);
    let choices = menu.choices;
    for hidden in [false, true] {
        menu.menu_hidden = hidden;
        let consumed = menu.input(MAPPER_MINUS, presets);
        assert!(menu.toggle_mash);
        assert_ne!(consumed & MAPPER_MINUS, 0);
        assert_eq!(menu.menu_hidden, hidden);
        assert_eq!(menu.choices, choices);
        menu.input(MAPPER_MINUS, presets);
        assert!(!menu.toggle_mash);
        menu.input(0, presets);
        assert!(!menu.toggle_mash);
    }
}

#[test]
fn consumed_y_still_reaches_readiness_observer_but_not_the_game_mapping() {
    use crate::manual_controls::{ManualOverlay, TurboOutput};
    use crate::native_input::ButtonTrace;
    let catalog = FighterProfiles::default();
    let presets = &catalog.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.input(0, presets);
    let consumed = menu.input(MAPPER_Y | DPAD_RIGHT, presets);
    let observed = observation_buttons(MAPPER_Y | DPAD_RIGHT, consumed);
    assert_ne!(
        observed & MAPPER_Y,
        0,
        "first Y press must open native special-readiness reads"
    );
    assert_eq!(
        observed & DPAD_RIGHT,
        0,
        "menu navigation must not start a slide"
    );
    let mapped = ManualOverlay::default().apply_controls(
        ButtonTrace {
            held: MAPPER_Y,
            pressed: MAPPER_Y,
            ..ButtonTrace::default()
        },
        [0.0, 0.0],
        TurboOutput::default(),
        consumed,
        0,
        0,
    );
    assert_eq!(mapped.buttons.held & MAPPER_Y, 0);
    assert_eq!(mapped.buttons.pressed & MAPPER_Y, 0);
    assert_eq!(
        observation_buttons(SHORT_HOP_TRIGGER, SHORT_HOP_TRIGGER),
        SHORT_HOP_TRIGGER
    );
    assert_eq!(observation_buttons(MENU_STICK_CLICK, MENU_STICK_CLICK), 0);
}

#[test]
fn l3_two_frame_cover_is_held_and_releases_immediately_without_rebinding_y() {
    let catalog = FighterProfiles::default();
    let presets = &catalog.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.input(0, presets);
    menu.input(TWO_FRAME_TRIGGER, presets);
    menu.input(TWO_FRAME_TRIGGER, presets);
    assert!(menu.two_frame_held(TWO_FRAME_TRIGGER));
    assert!(!menu.ledge_held(MAPPER_L | TWO_FRAME_TRIGGER));
    assert_ne!(
        menu.input(TWO_FRAME_TRIGGER, presets) & TWO_FRAME_TRIGGER,
        0
    );
    menu.input(0, presets);
    assert!(!menu.two_frame_held(0));
    assert!(!menu.two_frame_held(MAPPER_Y));
    assert_ne!(
        observation_buttons(TWO_FRAME_TRIGGER, TWO_FRAME_TRIGGER) & TWO_FRAME_TRIGGER,
        0
    );
}
