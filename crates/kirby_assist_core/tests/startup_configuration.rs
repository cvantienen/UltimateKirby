//! Compile the production startup adapter on the host, without SD or hooks.
#![allow(dead_code)]
mod settings_menu {
    pub fn recover_saved_file() -> Result<(), String> {
        panic!("host startup tests must not access the SD card")
    }
}
#[path = "../../../src/live/config.rs"]
mod config;

#[test]
fn retired_auto_duck_setting_is_inert_and_validated() {
    use kirby_assist_core::live_input::LiveConfig;

    let expected = format!("{:?}", LiveConfig::parse("auto_tech = false").unwrap());
    for value in ["true", "false"] {
        let config =
            LiveConfig::parse(&format!("auto_tech = false\nkirby_auto_duck = {value}")).unwrap();
        assert_eq!(format!("{config:?}"), expected);
    }
    let error = LiveConfig::parse("kirby_auto_duck = maybe").unwrap_err();
    assert!(error.contains("invalid config line 1"));
}
