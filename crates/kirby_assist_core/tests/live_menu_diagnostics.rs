#![allow(unexpected_cfgs, dead_code)]
// Compile the production menu adapter without the Switch/ImGui runtime.
use kirby_assist_core::{fighter_profiles::FighterProfiles, settings::AssistSettings};
use std::sync::{Mutex, OnceLock};
static PROFILES: OnceLock<FighterProfiles> = OnceLock::new();
static SEQUENCES: OnceLock<Mutex<kirby_assist_core::sequence_config::SequenceLibrary>> =
    OnceLock::new();
static SEQUENCES_UPDATED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static MASTER_IN_MATCH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
struct HostState {
    menu_battle_active: bool,
    settings_menu: Option<live_menu::SettingsMenu>,
}
static STATE: Mutex<HostState> = Mutex::new(HostState {
    menu_battle_active: false,
    settings_menu: None,
});
fn settings_for_fighter(_: Option<i32>, base: AssistSettings) -> AssistSettings {
    base
}
#[path = "../../../src/diagnostics/performance.rs"]
mod performance;
static CONFIGS: Mutex<Vec<kirby_assist_core::diagnostics::modes::Configuration>> =
    Mutex::new(Vec::new());
mod calibration {
    pub fn clear() {}
}
mod logging {
    static ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    pub fn observe(training: bool) {
        ACTIVE.store(training, std::sync::atomic::Ordering::Release);
        crate::performance::observe(training);
    }
    pub fn active() -> bool {
        ACTIVE.load(std::sync::atomic::Ordering::Acquire)
    }
    pub fn write_configuration(config: kirby_assist_core::diagnostics::modes::Configuration) {
        super::CONFIGS.lock().unwrap().push(config);
    }
}
#[path = "../../../src/live/settings_menu.rs"]
mod live_menu;

#[test]
fn configuration_logs_once_on_entry_and_on_choice_edits_but_not_navigation_or_polls() {
    let _save_guard = live_menu::save_test_guard();
    CONFIGS.lock().unwrap().clear();
    logging::observe(false);
    PROFILES.get_or_init(|| {
        let c=kirby_assist_core::sequence_config::Catalog::parse("[kirby.jump.first]\nstart_delay=0\nactions=short_hop\n[kirby.jump.second]\nstart_delay=0\nactions=nair\n").unwrap();
        let mut profiles=FighterProfiles::default();profiles.add_sequence_choices(&c).unwrap();profiles
    });
    let mut menu = live_menu::SettingsMenu::default();
    let mut sample = |kind, buttons| {
        menu.input(live_menu::MenuInput {
            available: true,
            gameplay: true,
            bindings_blocked: false,
            safety_jump: false,
            fighter_kind: Some(kind),
            buttons,
            physical: buttons,
            layout: kirby_assist_core::native_input::ControlLayout::Default,
            base: AssistSettings::default(),
            mash_default: true,
            grab_default: true,
            tech_default: true,
            damage_defaults: [false; 2],
            training: true,
        });
    };
    sample(6, 0);
    assert!(!logging::active(), "menu cannot bypass a pending native diagnostic reset");
    assert!(CONFIGS.lock().unwrap().is_empty());
    logging::observe(true);
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
    drop(sample);
    let before = menu.test_view_revision();
    menu.input(live_menu::MenuInput {
        available: true, gameplay: true, bindings_blocked: false, safety_jump: false,
        fighter_kind: Some(6), buttons: 0, physical: 0,
        layout: kirby_assist_core::native_input::ControlLayout::Default,
        base: AssistSettings::default(), mash_default: true, grab_default: true,
        tech_default: true, damage_defaults: [true, false], training: true,
    });
    assert_ne!(menu.test_view_revision(), before, "changed inherited DI must invalidate displayed content");
    let changed = menu.test_view_revision();
    menu.input(live_menu::MenuInput {
        available: true, gameplay: true, bindings_blocked: false, safety_jump: false,
        fighter_kind: Some(6), buttons: 0, physical: 0,
        layout: kirby_assist_core::native_input::ControlLayout::Default,
        base: AssistSettings::default(), mash_default: true, grab_default: true,
        tech_default: true, damage_defaults: [true, false], training: true,
    });
    assert_eq!(menu.test_view_revision(), changed, "unchanged defaults must not rebuild every poll");
    menu.input(live_menu::MenuInput {
        available: true, gameplay: true, bindings_blocked: false, safety_jump: false,
        fighter_kind: Some(5), buttons: 0, physical: 0,
        layout: kirby_assist_core::native_input::ControlLayout::Default,
        base: AssistSettings::default(), mash_default: true, grab_default: true,
        tech_default: true, damage_defaults: [false; 2], training: true,
    });
    let reports = CONFIGS.lock().unwrap();
    assert_eq!(reports.len(), 3);
    assert_eq!(reports[2].fighter, "yoshi");
    assert!(!reports[2].rows[1].is_empty());
    drop(reports);

    let mut input = live_menu::MenuInput {
        available: true, gameplay: true, bindings_blocked: false, safety_jump: false,
        fighter_kind: Some(5), buttons: 0, physical: 0,
        layout: kirby_assist_core::native_input::ControlLayout::Default,
        base: AssistSettings::default(), mash_default: true, grab_default: true,
        tech_default: true, damage_defaults: [false; 2], training: false,
    };
    menu.input(input);
    assert!(!logging::active());
    assert!(menu.diagnostic_selection(5).is_none());
    input.buttons = kirby_assist_core::ledge_trap::DPAD_DOWN;
    input.physical = input.buttons;
    menu.input(input);
    assert_eq!(CONFIGS.lock().unwrap().len(), 3, "no diagnostic sampling outside Training");
    input.training = true;
    logging::observe(true);
    input.buttons = 0;
    input.physical = 0;
    menu.input(input);
    assert!(logging::active());
    assert!(menu.diagnostic_selection(5).is_some());
    assert!(menu.diagnostic_selection(6).is_none());
    assert_eq!(CONFIGS.lock().unwrap().len(), 4, "new Training session republishes configuration");
    input.available = false;
    menu.input(input);
    assert!(!logging::active(), "unavailable menu revokes diagnostic permission");
    assert_eq!(CONFIGS.lock().unwrap().len(), 4);
    input.available = true;
    input.fighter_kind = None;
    menu.input(input);
    assert!(!logging::active(), "missing fighter cannot grant diagnostic permission");
}
