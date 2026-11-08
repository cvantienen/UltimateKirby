// Exercise the production selector and its local regressions independently of
// unrelated pre-existing lib-test compilation failures.
pub use kirby_assist_core::{
    button_bindings, fighter_data, fighter_kinds, fighter_profiles, fighter_tuning, gameplay,
    ledge_trap, manual_controls, native_input, native_states, out_of_shield, projectile_reflect,
    reaction_rules, roll_read, sequence_config, settings, timing,
};
#[path = "../src/settings_menu.rs"]
mod settings_menu;
