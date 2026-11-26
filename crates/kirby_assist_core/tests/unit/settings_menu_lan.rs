use super::*;

#[test]
fn lan_switch_is_diagnostic_training_only_and_never_autoactivates() {
    let presets = MenuPresets::EMPTY;
    let mut m = MenuSelection::new(&presets);
    m.begin_match();
    m.input_with_editing(0, &presets, true);
    assert!(m.menu_hidden);
    m.input(MAPPER_MINUS, &presets);
    m.input(0, &presets);
    m.set_training_category(4);
    let panel = m.recipe_panel(&presets);
    assert_eq!(
        panel
            .body
            .iter()
            .any(|line| line.contains("LAN editor HTTP")),
        cfg!(feature = "diagnostic-logging")
    );
    #[cfg(feature = "diagnostic-logging")]
    {
        assert!(!m.lan_enabled && !m.lan_toggle_requested);
        for _ in 0..crate::fighter_tuning::COUNT {
            m.input(0, &presets);
            m.input(DPAD_DOWN, &presets);
            if m.tuning_row == crate::fighter_tuning::COUNT {
                break;
            }
        }
        assert_eq!(m.tuning_row, crate::fighter_tuning::COUNT);
        m.input(0, &presets);
        let tuning = m.tuning;
        let consumed = m.input(MAPPER_A, &presets);
        assert!(m.lan_toggle_requested);
        assert_ne!(consumed & MAPPER_A, 0);
        m.input(MAPPER_A, &presets);
        assert!(!m.lan_toggle_requested, "held confirm must not retrigger");
        m.input(0, &presets);
        m.input(DPAD_RIGHT, &presets);
        assert!(m.lan_toggle_requested);
        for _ in 0..120 {
            m.input(DPAD_RIGHT, &presets);
            assert!(!m.lan_toggle_requested);
        }
        m.input(0, &presets);
        m.input(MAPPER_Y, &presets);
        assert!(!m.tuning_save_requested);
        m.input(0, &presets);
        m.input(MAPPER_X, &presets);
        assert_eq!(m.tuning, tuning);
        m.lan_enabled = true;
        m.lan_status = Some("Listener unavailable".into());
        let panel = m.recipe_panel(&presets);
        assert!(panel
            .body
            .iter()
            .any(|s| s.contains("HTTP: ON") && s.contains("Listener unavailable")));
        m.input_with_editing(0, &presets, false);
        assert!(!m.lan_toggle_requested && !m.lan_editing_allowed());
        assert!(!m
            .recipe_panel(&presets)
            .body
            .iter()
            .any(|s| s.contains("LAN editor")));
    }
}

#[cfg(feature = "diagnostic-logging")]
#[test]
fn remote_editing_requires_open_clean_training_menu() {
    let p = MenuPresets::EMPTY;
    let mut m = MenuSelection::new(&p);
    assert!(!m.lan_editing_allowed());
    m.input(0, &p);
    m.input(MAPPER_MINUS, &p);
    assert!(m.lan_editing_allowed());
    m.set_training_category(4);
    m.input(0, &p);
    m.input(DPAD_RIGHT, &p);
    assert!(!m.lan_editing_allowed());
    m.saved_tuning = m.tuning;
    assert!(m.lan_editing_allowed());
    let key = (PresetSlot::Roll as usize, 0);
    let e = m.editor.get_or_insert_with(Default::default);
    e.ensure(key, editor::RecipeEditor::custom());
    for cursor in 0..100 {
        e.cursor = cursor;
        e.adjust(key, true);
        if e.edited(key) {
            break;
        }
    }
    assert!(
        !m.lan_editing_allowed(),
        "nonselected unsaved recipe blocks remote overwrite"
    );
    m.editor.as_mut().unwrap().mark_saved(key);
    assert!(m.lan_editing_allowed());
    m.menu_hidden = true;
    assert!(!m.lan_editing_allowed());
}
