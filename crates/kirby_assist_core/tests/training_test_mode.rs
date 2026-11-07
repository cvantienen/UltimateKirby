use kirby_assist_core::{
    fighter_profiles::FighterProfiles,
    ledge_trap::DPAD_DOWN,
    live_input::LiveConfig,
    manual_controls::DPAD_UP,
    native_input::{MAPPER_L, MAPPER_MINUS, MAPPER_PLUS, MAPPER_X, MAPPER_Y, MAPPER_ZL, MAPPER_ZR},
    settings_menu::{MenuSelection, MENU_STICK_CLICK, MENU_TOGGLE_CHORD},
    training_test,
};

#[test]
fn compatibility_requires_installed_training_without_a_logging_dependency() {
    for installed in [false, true] {
        for training in [false, true] {
            let active = training_test::active(installed, training);
            assert_eq!(active, installed && training);
            let original=LiveConfig::parse("auto_mash = true\nauto_tech = true\nauto_sdi = true\nr3_turbo = true\ndpad_slide = true\nkirby_auto_duck = true").unwrap();
            assert_eq!(
                format!("{original:?}"),
                format!("{:?}", training_test::settings(original, active)),
                "compatibility must not silently disable gameplay features"
            );
        }
    }
}

#[test]
fn minus_is_a_policy_menu_event_and_preserves_real_l_x_y_inputs() {
    use kirby_assist_core::{
        manual_controls::{ManualOverlay, TurboOutput},
        native_input::ButtonTrace,
    };
    assert_eq!(
        training_test::policy_buttons(MENU_TOGGLE_CHORD | MAPPER_X | MAPPER_Y),
        MENU_TOGGLE_CHORD | MAPPER_X | MAPPER_Y
    );
    assert_eq!(
        training_test::policy_buttons(MAPPER_L | MAPPER_Y),
        MAPPER_L | MAPPER_Y
    );
    assert_eq!(
        training_test::policy_buttons(MAPPER_MINUS),
        MAPPER_MINUS,
        "real Minus belongs to the assist menu"
    );
    let profiles = FighterProfiles::default();
    let presets = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.input(0, presets);
    let minus = training_test::policy_buttons(MENU_TOGGLE_CHORD);
    let consumed = menu.input(minus, presets);
    let mapped = ManualOverlay::default().apply_controls(
        ButtonTrace {
            held: MENU_TOGGLE_CHORD,
            pressed: MENU_TOGGLE_CHORD,
            ..ButtonTrace::default()
        },
        [0.; 2],
        TurboOutput::default(),
        consumed | MENU_TOGGLE_CHORD,
        0,
        0,
    );
    assert_eq!(
        mapped.buttons.held & MAPPER_MINUS,
        0,
        "virtual Minus is only a menu event, never a game/Training button"
    );
    assert!(!menu.menu_hidden && !menu.toggle_mash);
    menu.input(minus, presets);
    assert!(!menu.menu_hidden, "held Minus must not repeat");
    menu.input(0, presets);
    menu.input(minus, presets);
    assert!(menu.menu_hidden);
    menu.input(0, presets);
    menu.input(MAPPER_L, presets);
    menu.input(MAPPER_L, presets);
    assert!(
        menu.ledge_held(MAPPER_L),
        "real L runs autopilot without R3 held"
    );
}

#[test]
fn training_shortcuts_hand_off_until_a_fresh_minus_tap() {
    assert!(
        !training_test::passthrough(0, 0, false),
        "normal gameplay does not require R3"
    );
    for chord in [
        MAPPER_PLUS,
        MAPPER_ZR | DPAD_UP,
        MAPPER_L | DPAD_DOWN | MENU_STICK_CLICK,
    ] {
        assert!(training_test::passthrough(chord, chord, false));
        assert!(
            training_test::passthrough(0, 0, true),
            "menu/recording can outlive its button press"
        );
        assert!(
            training_test::passthrough(MENU_TOGGLE_CHORD, 0, true),
            "an unpressed Minus cannot resume us"
        );
        assert!(!training_test::passthrough(
            MENU_TOGGLE_CHORD,
            MENU_TOGGLE_CHORD,
            true
        ));
        assert!(
            training_test::passthrough(chord | MENU_TOGGLE_CHORD, MENU_TOGGLE_CHORD, true),
            "Training shortcut wins"
        );
    }
}

#[test]
fn default_save_load_is_momentary_and_held_l_resumes_without_reopening_menu() {
    for pad in [DPAD_UP, DPAD_DOWN] {
        let chord = MAPPER_ZL | pad;
        assert!(!training_test::sticky_shortcut(chord));
        assert!(training_test::passthrough(chord, chord, false));
        assert!(!training_test::passthrough(0, 0, false));
    }
    let profiles = FighterProfiles::default();
    let presets = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.input(0, presets);
    menu.input(MAPPER_L, presets);
    menu.input(MAPPER_L, presets);
    menu.cancel_actions();
    menu.observe_passthrough(MAPPER_L | MAPPER_ZL | DPAD_UP);
    menu.resume_training_ledge_hold();
    menu.input(MAPPER_L, presets);
    assert!(menu.ledge_held(MAPPER_L));
    assert!(menu.menu_hidden);
    assert!(!menu.finisher_held(MAPPER_Y), "save/load cannot arm held Y");
}
