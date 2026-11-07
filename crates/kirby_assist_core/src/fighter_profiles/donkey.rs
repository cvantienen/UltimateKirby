//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
use super::FighterProfile;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "donkey_kong",
        fighter_kind: 1,
        ledge: super::LedgeFighterData::from_kind(1),

        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
