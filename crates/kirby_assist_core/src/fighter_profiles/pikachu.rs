//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
mod timing;
use super::FighterProfile;
pub use timing::LEDGE_DATA;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "pikachu",
        fighter_kind: 8,
        ledge: LEDGE_DATA,

        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
