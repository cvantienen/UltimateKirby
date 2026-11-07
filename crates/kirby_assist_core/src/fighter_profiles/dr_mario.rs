//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
use super::FighterProfile;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "dr_mario",
        fighter_kind: 18,
        ledge: super::LedgeFighterData::from_kind(18),
        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
