//! Compiled pacman ledge profile. No fighter files load during a match.
mod timing;
use super::FighterProfile;
pub use timing::*;
pub const FIGHTER_KIND: i32 = 0x37;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "pacman",
        fighter_kind: FIGHTER_KIND,
        aerial_execution: super::execution::AerialExecution::ADAPTIVE_JUMP,
        ledge: LEDGE_DATA,
        menu_options: crate::ledge_trap::fighters::pacman::MENU,
    }
}
