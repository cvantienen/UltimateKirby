//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
use super::FighterProfile;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "bowser",
        fighter_kind: crate::fighter_kinds::BOWSER,
        grab: super::FighterGrabData::from_kind(15),
        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
