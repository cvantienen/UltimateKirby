//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
mod timing;
use super::FighterProfile;
pub use timing::*;
pub const FIGHTER_KIND: i32 = 0x45;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "krool",
        fighter_kind: FIGHTER_KIND,
        ledge: LEDGE_DATA,
        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
