//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
use super::FighterProfile;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "duck_hunt",
        fighter_kind: crate::fighter_kinds::DUCK_HUNT,
        ledge: super::FighterGrabData::from_kind(59),
        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
