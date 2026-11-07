//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
use super::FighterProfile;
pub const FIGHTER_KIND: i32 = 0x34;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "littlemac",
        fighter_kind: FIGHTER_KIND,
        ledge: super::LedgeFighterData::from_kind(52),
        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
