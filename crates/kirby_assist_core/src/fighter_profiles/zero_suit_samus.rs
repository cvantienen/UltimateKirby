//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
use super::FighterProfile;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "zero_suit_samus",
        fighter_kind: crate::fighter_kinds::ZERO_SUIT_SAMUS,
        grab: super::FighterGrabData::from_kind(32),
        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
