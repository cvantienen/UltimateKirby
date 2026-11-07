//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
use super::FighterProfile;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "king_dedede",
        fighter_kind: crate::fighter_kinds::KING_DEDEDE,
        ledge: super::FighterGrabData::from_kind(42),
        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
