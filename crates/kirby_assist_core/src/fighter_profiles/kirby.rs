//! Fighter identity and independent reference facts. Ledge options and input
//! scheduling are loaded exclusively from sequences.txt at startup.
use super::FighterProfile;

pub const FIGHTER_KIND: i32 = crate::fighter_kinds::KIRBY;

pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "kirby",
        fighter_kind: FIGHTER_KIND,
        ledge: super::FighterGrabData::from_kind(6),

        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
