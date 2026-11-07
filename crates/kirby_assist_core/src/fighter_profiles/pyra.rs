//! Pyra's own identity/reference facts. Recipes are owned by her text file.
mod timing;
use super::FighterProfile;
pub use timing::*;
pub const FIGHTER_KIND: i32 = 90;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "pyra",
        fighter_kind: FIGHTER_KIND,
        ledge: LEDGE_DATA,
        menu_options: crate::fighter_profiles::presets::MenuPresets::EMPTY,
    }
}
