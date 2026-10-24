//! Diagnostic Training compatibility: route R3 to OUR menu only, and hand the
//! complete controller back to Training for its own menus/save/record shortcuts.
//! No Training-private state, virtual L press or reduced gameplay config.
use crate::{
    ledge_menu::menu_toggle_pressed,
    ledge_trap::DPAD_DOWN,
    live_input::LiveConfig,
    manual_controls::DPAD_UP,
    native_input::{MAPPER_B, MAPPER_MINUS, MAPPER_PLUS, MAPPER_ZL, MAPPER_ZR},
};

pub fn active(diagnostic: bool, installed: bool, training: bool) -> bool {
    diagnostic && installed && training
}

pub fn passthrough(buttons: u32, pressed: u32, already: bool) -> bool {
    let overlay_chord = crate::native_input::MAPPER_L | DPAD_DOWN | crate::ledge_menu::MENU_STICK_CLICK;
    let training_shortcut = buttons & overlay_chord == overlay_chord
        || buttons & (MAPPER_PLUS | MAPPER_MINUS) != 0
        || (buttons & (MAPPER_B | MAPPER_ZL | MAPPER_ZR) != 0
            && buttons & (DPAD_UP | DPAD_DOWN) != 0);
    // Persist the handoff across releases: a Training menu/recording can stay
    // open after its shortcut is released. A fresh R3 tap explicitly resumes us.
    training_shortcut || (already && !menu_toggle_pressed(buttons, pressed))
}

/// Default ZL+Up/Down save/load is momentary. Menus, recording/playback and
/// OS-overlay shortcuts can stay open after release and require an R3 resume.
pub fn sticky_shortcut(buttons: u32) -> bool {
    crate::ledge_menu::monitor_chord(buttons)
        || buttons & (MAPPER_PLUS | MAPPER_MINUS) != 0
        || buttons & MAPPER_B != 0 && buttons & (DPAD_UP | DPAD_DOWN) != 0
        || buttons & MAPPER_ZR != 0 && buttons & (DPAD_UP | DPAD_DOWN) != 0
}

/// R3 is the shared menu event in every build. Real Minus belongs to Training.
pub fn policy_buttons(physical: u32) -> u32 {
    physical & !MAPPER_MINUS
}

/// All gameplay features retain the same startup/menu settings in Training.
pub fn settings(config: LiveConfig, _test_mode: bool) -> LiveConfig {
    config
}
