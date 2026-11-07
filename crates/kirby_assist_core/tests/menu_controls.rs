// Exercise the production selector and its local regressions independently of
// unrelated pre-existing lib-test compilation failures.
pub use kirby_assist_core::{
    button_bindings, button_modes, fighter_profiles, fighter_tuning, jab_grab, ledge_trap, manual_controls,
    native_input, opponent_data, out_of_shield, projectile_reflect, reaction_rules, roll_read, sequence_config, timing,
};
#[path = "../src/ledge_menu.rs"]
mod ledge_menu;
