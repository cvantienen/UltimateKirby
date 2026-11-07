//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
use super::FighterProfile;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "dark_samus",
        fighter_kind: 4,
        ledge: super::LedgeFighterData::from_kind(4),
        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
