#![cfg(feature = "diagnostic-logging")]
#![allow(unexpected_cfgs, dead_code)]
// Compile the production menu adapter without the Switch/ImGui runtime.
use kirby_assist_core::{fighter_profiles::FighterProfiles, ledge_trap::LedgeSettings};
use std::sync::{Mutex, OnceLock};
static PROFILES: OnceLock<FighterProfiles> = OnceLock::new();
static CONFIGS: Mutex<Vec<kirby_assist_core::diagnostics::modes::Configuration>> =
    Mutex::new(Vec::new());
mod calibration {
    pub fn clear() {}
}
mod logging {
    pub fn write_configuration(config: kirby_assist_core::diagnostics::modes::Configuration) {
        super::CONFIGS.lock().unwrap().push(config);
    }
}
#[path = "../../../src/live/ledge_menu.rs"]
mod live_menu;

#[test]
fn configuration_logs_once_on_entry_and_on_choice_edits_but_not_navigation_or_polls() {
    PROFILES.get_or_init(FighterProfiles::default);
    let mut menu = live_menu::LedgeMenu::default();
    let mut sample = |kind, buttons| {
        menu.input(
            true,
            true,
            Some(kind),
            buttons,
            LedgeSettings::default(),
            true,
            true,
        );
    };
    sample(6, 0);
    assert_eq!(CONFIGS.lock().unwrap().len(), 1);
    for _ in 0..600 {
        sample(6, 0);
    }
    // Down selects the Jump row; navigation alone does not change options.
    sample(6, kirby_assist_core::ledge_trap::DPAD_DOWN);
    sample(6, 0);
    assert_eq!(CONFIGS.lock().unwrap().len(), 1);
    let before = CONFIGS.lock().unwrap()[0].rows[1];
    sample(6, kirby_assist_core::manual_controls::DPAD_RIGHT);
    sample(6, 0);
    assert_eq!(CONFIGS.lock().unwrap().len(), 2);
    assert_ne!(CONFIGS.lock().unwrap()[1].rows[1], before);
    sample(5, 0);
    let reports = CONFIGS.lock().unwrap();
    assert_eq!(reports.len(), 3);
    assert_eq!(reports[2].fighter, "yoshi");
    assert_eq!(reports[2].rows[8], "unavailable for this fighter");
    assert_ne!(reports[2].rows[11], "unavailable for this fighter");
}
