//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
mod timing;
use super::FighterProfile;
pub use timing::LEDGE_DATA;

// FIGHTER_KIND_SONIC in the pinned skyline-smash lua_const.rs.
pub const FIGHTER_KIND: i32 = 0x29;

pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "sonic",
        fighter_kind: FIGHTER_KIND,
        ledge: LEDGE_DATA,
        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
