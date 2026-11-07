//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
mod timing;
pub use timing::{
    INHALE_STARTUP, JUMP_BAIR, LEDGE_DATA, ROLL_READ_DOWN_SMASH, ROLL_READ_GRAB, WHIFF_SMASH_REACH,
    WHIFF_SMASH_STARTUP, WHIFF_TILT_REACH,
};

use super::FighterProfile;

pub const FIGHTER_KIND: i32 = 6;

pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "kirby",
        fighter_kind: FIGHTER_KIND,
        ledge: LEDGE_DATA,

        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
