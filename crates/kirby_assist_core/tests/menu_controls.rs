// Exercise the production selector and its local regressions independently of
// unrelated pre-existing lib-test compilation failures.
pub use kirby_assist_core::{
    button_modes, fighter_profiles, ledge_trap, manual_controls, native_input, roll_read,
};
#[path = "../src/ledge_menu.rs"]
mod ledge_menu;
