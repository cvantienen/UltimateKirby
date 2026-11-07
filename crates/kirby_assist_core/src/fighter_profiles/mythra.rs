//! Mythra's own identity/reference facts. Recipes are owned by her text file.
use super::FighterProfile;
pub const FIGHTER_KIND: i32 = crate::fighter_kinds::MYTHRA;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "mythra",
        fighter_kind: FIGHTER_KIND,
        grab: super::FighterGrabData::from_kind(91),
        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
