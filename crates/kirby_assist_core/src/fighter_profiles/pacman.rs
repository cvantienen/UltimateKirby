//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
use super::FighterProfile;
pub const FIGHTER_KIND: i32 = crate::fighter_kinds::PAC_MAN;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "pacman",
        fighter_kind: FIGHTER_KIND,
        ledge: super::FighterGrabData::from_kind(55),
        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
