//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
mod timing;
use super::FighterProfile;
pub use timing::*;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "wii_fit_trainer",
        fighter_kind: 50,
        ledge: LEDGE_DATA,
        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
