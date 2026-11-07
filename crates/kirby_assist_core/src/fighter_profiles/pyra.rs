//! Pyra's own identity/reference facts. Recipes are owned by her text file.
use super::FighterProfile;
pub const FIGHTER_KIND: i32 = 90;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "pyra",
        fighter_kind: FIGHTER_KIND,
        ledge: super::LedgeFighterData::from_kind(90),
        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
