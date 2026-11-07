#![cfg(feature = "diagnostic-logging")]
#![allow(unexpected_cfgs, dead_code)]
// Compile the production menu adapter without the Switch/ImGui runtime.
use kirby_assist_core::{fighter_profiles::FighterProfiles, ledge_trap::LedgeSettings};
use std::sync::{Mutex, OnceLock};
static PROFILES: OnceLock<FighterProfiles> = OnceLock::new();
static SEQUENCES: OnceLock<Mutex<kirby_assist_core::sequence_config::SequenceLibrary>> = OnceLock::new();
static SEQUENCES_UPDATED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static MASTER_IN_MATCH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
struct HostState { menu_battle_active: bool, ledge_menu: Option<live_menu::LedgeMenu> }
static STATE: Mutex<HostState> = Mutex::new(HostState{menu_battle_active:false,ledge_menu:None});
fn ledge_settings(_:Option<i32>,base:LedgeSettings)->LedgeSettings{base}
#[path = "../../../src/performance.rs"]
mod performance;
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
    let _save_guard = live_menu::save_test_guard();
    CONFIGS.lock().unwrap().clear();
    PROFILES.get_or_init(|| {
        let c=kirby_assist_core::sequence_config::Catalog::parse("[kirby.jump.first]\nstart_delay=0\nactions=short_hop\n[kirby.jump.second]\nstart_delay=0\nactions=nair\n").unwrap();
        let mut profiles=FighterProfiles::default();profiles.add_sequence_choices(&c).unwrap();profiles
    });
    let mut menu = live_menu::LedgeMenu::default();
    let mut sample = |kind, buttons| {
        menu.input(
            true,
            true,
            false,
            false,
            false,
            Some(kind), buttons, buttons,
            kirby_assist_core::native_input::ControlLayout::Default,
            &LedgeSettings::default(),
            true,
            true,
            true,
            true,
        );
    };
    sample(6, 0);
    assert_eq!(CONFIGS.lock().unwrap().len(), 1);
    sample(6, kirby_assist_core::native_input::MAPPER_MINUS);
    sample(6, 0);
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
    assert!(!reports[2].rows[1].is_empty());
}
