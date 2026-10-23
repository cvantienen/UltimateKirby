//! Compiled isabelle ledge profile. No fighter config file loads at runtime.
mod timing;
use super::FighterProfile;
pub use timing::*;
pub const FIGHTER_KIND: i32 = 0x46;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "isabelle",
        fighter_kind: FIGHTER_KIND,
        aerial_execution: super::execution::AerialExecution::ADAPTIVE_JUMP,
        ledge: LEDGE_DATA,
        menu_options: crate::ledge_trap::fighters::isabelle::MENU,
    }
}
