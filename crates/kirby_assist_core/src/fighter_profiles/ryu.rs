//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
mod timing;
use super::FighterProfile;
pub use timing::*;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "ryu",
        fighter_kind: 60,
        ledge: LEDGE_DATA,
        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
