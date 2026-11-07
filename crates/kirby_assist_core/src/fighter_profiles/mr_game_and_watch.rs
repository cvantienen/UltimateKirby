//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
use super::FighterProfile;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "mr_game_and_watch",
        fighter_kind: crate::fighter_kinds::MR_GAME_AND_WATCH,
        grab: super::FighterGrabData::from_kind(28),
        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
