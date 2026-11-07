//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
use super::FighterProfile;
pub const FIGHTER_KIND: i32 = 0x46;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "isabelle",
        fighter_kind: FIGHTER_KIND,
        ledge: super::LedgeFighterData::from_kind(70),
        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
