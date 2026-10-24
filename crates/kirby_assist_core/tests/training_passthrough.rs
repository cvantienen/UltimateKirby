use kirby_assist_core::{
    fighter_profiles::FighterProfiles,
    ledge_menu::{MenuRow, MenuSelection},
    ledge_trap::DPAD_DOWN,
    native_input::{MAPPER_L, MAPPER_MINUS},
};

#[test]
fn physical_release_during_training_passthrough_rearms_autopilot() {
    let catalog = FighterProfiles::default();
    let presets = &catalog.profile(6).unwrap().menu_options;
    let mut selection = MenuSelection::new(presets);
    selection.input(MAPPER_L, presets);
    selection.input(MAPPER_L, presets);
    assert!(
        !selection.ledge_held(MAPPER_L),
        "unobserved release must not arm"
    );
    selection.observe_passthrough(0);
    selection.input(MAPPER_L, presets);
    assert!(
        !selection.ledge_held(MAPPER_L),
        "first press keeps normal one-sample cancellation"
    );
    selection.input(MAPPER_L, presets);
    assert!(selection.ledge_held(MAPPER_L));
    selection.cancel_actions();
    selection.observe_passthrough(0);
    selection.input(MAPPER_L, presets);
    selection.input(MAPPER_L, presets);
    assert!(
        selection.ledge_held(MAPPER_L),
        "release must rearm after cancellation too"
    );
}

#[test]
fn passthrough_observation_does_not_edit_menu_or_replay_held_directions() {
    let catalog = FighterProfiles::default();
    let presets = &catalog.profile(6).unwrap().menu_options;
    let mut selection = MenuSelection::new(presets);
    selection.input(0, presets);
    let choices = selection.choices;
    selection.observe_passthrough(MAPPER_MINUS | DPAD_DOWN);
    assert!(!selection.toggle_mash);
    assert_eq!(selection.row, MenuRow::Neutral);
    assert_eq!(selection.choices, choices);
    selection.input(MAPPER_L | DPAD_DOWN, presets);
    assert_eq!(selection.row, MenuRow::Neutral);
    selection.input(MAPPER_L, presets);
    selection.input(MAPPER_L | DPAD_DOWN, presets);
    assert_eq!(selection.row, MenuRow::Jump);
}
