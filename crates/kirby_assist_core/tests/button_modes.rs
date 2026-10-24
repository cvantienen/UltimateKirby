use kirby_assist_core::{
    button_modes::YMode,
    fighter_profiles::FighterProfiles,
    ledge_menu::{MenuRow, MenuSelection, TWO_FRAME_TRIGGER},
    manual_controls::DPAD_RIGHT,
    native_input::{MAPPER_L, MAPPER_Y},
};

#[test]
fn held_y_mode_is_independent_of_l_and_requires_release_after_edit_or_reset() {
    let profiles = FighterProfiles::default();
    let presets = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.auto_duck_available = true;
    menu.input(0, presets);
    menu.input(MAPPER_Y, presets);
    assert!(menu.finisher_held(MAPPER_Y));
    assert!(
        menu.finisher_held(MAPPER_Y | kirby_assist_core::manual_controls::LEFT_STICK_DIRECTIONS)
    );
    assert!(!menu.ledge_held(MAPPER_Y));
    assert!(!menu.finisher_held(0));
    menu.row = MenuRow::YMode;
    menu.input(MAPPER_Y | DPAD_RIGHT, presets);
    assert!(!menu.finisher_held(MAPPER_Y));
    menu.input(MAPPER_Y, presets);
    assert!(
        !menu.finisher_held(MAPPER_Y),
        "held Y cannot start a newly selected mode"
    );
    menu.input(0, presets);
    menu.input(MAPPER_Y, presets);
    assert!(menu.finisher_held(MAPPER_Y));
    menu.begin_match();
    menu.input(MAPPER_Y, presets);
    assert!(!menu.finisher_held(MAPPER_Y));
    menu.input(0, presets);
    menu.input(MAPPER_L, presets);
    menu.input(MAPPER_L, presets);
    assert!(menu.ledge_held(MAPPER_L));
    assert_eq!(
        menu.input(TWO_FRAME_TRIGGER, presets) & TWO_FRAME_TRIGGER,
        0,
        "L3 is no longer reserved by Kirby Assist"
    );
    assert_eq!(YMode::options(true), &[YMode::FairSmash, YMode::DairSmash]);
    assert!(YMode::options(false).is_empty());
    assert_eq!(YMode::options_for_kind(5), &[YMode::YoshiBairSmash]);
    menu.auto_duck_available = false;
    assert!(!menu.finisher_held(MAPPER_Y));
    assert!(!menu.auto_duck_held(MAPPER_Y));
}

#[test]
fn whiff_punish_is_not_a_live_menu_row() {
    assert_eq!(MenuRow::ALL.len(), 12);
    assert!(MenuRow::ALL.iter().all(|row| row.label() != "Whiff punish"));
}
