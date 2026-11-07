//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
use super::FighterProfile;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "rosalina_and_luma",
        fighter_kind: crate::fighter_kinds::ROSALINA_AND_LUMA,
        grab: super::FighterGrabData::from_kind(51),
        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
