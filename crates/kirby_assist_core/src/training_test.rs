//! Training compatibility in both builds: reserve Minus for OUR menu and hand the
//! complete controller back to Training for its own menus/save/record shortcuts.
//! No Training-private state, virtual L press or reduced gameplay config.
use crate::{
    ledge_trap::DPAD_DOWN,
    live_input::LiveConfig,
    manual_controls::DPAD_UP,
    native_input::{MAPPER_B, MAPPER_PLUS, MAPPER_ZL, MAPPER_ZR},
    settings_menu::menu_toggle_pressed,
};

pub fn active(installed: bool, training: bool) -> bool {
    installed && training
}
/// Determines if the Training compatibility is active based on the installed and training flags.
/// # Arguments
/// * `installed` - Indicates if the feature is installed.
/// * `training` - Indicates if the training mode is active.
///
/// # Returns
/// `true` if both installed and training are active, otherwise `false`.
pub fn passthrough(buttons: u32, pressed: u32, already: bool) -> bool {
    let overlay_chord =
        crate::native_input::MAPPER_L | DPAD_DOWN | crate::settings_menu::MENU_STICK_CLICK;
    let training_shortcut = buttons & overlay_chord == overlay_chord
        || buttons & MAPPER_PLUS != 0
        || (buttons & (MAPPER_B | MAPPER_ZL | MAPPER_ZR) != 0
            && buttons & (DPAD_UP | DPAD_DOWN) != 0);
    // Persist the handoff across releases: a Training menu/recording can stay
    // open after its shortcut is released. Fresh Minus resumes and toggles ours.
    training_shortcut || (already && !menu_toggle_pressed(buttons, pressed))
}

/// Default ZL+Up/Down save/load is momentary. Menus, recording/playback and
/// OS-overlay shortcuts can stay open after release and require a Minus resume.
/// Determines if a sticky shortcut is active based on the current button state.
/// # Arguments
/// * `buttons` - The current state of the buttons.
///
/// # Returns
/// `true` if a sticky shortcut is active, otherwise `false`.
pub fn sticky_shortcut(buttons: u32) -> bool {
    crate::settings_menu::monitor_chord(buttons)
        || buttons & MAPPER_PLUS != 0
        || buttons & MAPPER_B != 0 && buttons & (DPAD_UP | DPAD_DOWN) != 0
        || buttons & MAPPER_ZR != 0 && buttons & (DPAD_UP | DPAD_DOWN) != 0
}

/// Minus is the shared menu event in every build, including Training.
pub fn policy_buttons(physical: u32) -> u32 {
    physical
}

/// All gameplay features retain the same startup/menu settings in Training.
pub fn settings(config: LiveConfig, _test_mode: bool) -> LiveConfig {
    config
}
