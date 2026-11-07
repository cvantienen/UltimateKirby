//! Compile the production startup adapter on the host, without SD or hooks.
#![allow(dead_code)]
mod ledge_menu {
    pub fn recover_saved_file() -> Result<(), String> {
        panic!("host startup tests must not access the SD card")
    }
}
#[path = "../../../src/live/config.rs"]
mod config;
