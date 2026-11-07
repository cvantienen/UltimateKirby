//! Mythra's own identity/reference facts. Recipes are owned by her text file.
mod timing;
use super::FighterProfile;
pub use timing::*;
pub const FIGHTER_KIND: i32 = 91;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "mythra",
        fighter_kind: FIGHTER_KIND,
        ledge: LEDGE_DATA,
        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
