//! Compiled mr_game_and_watch profile. No fighter file is read at runtime.
mod timing;
use super::FighterProfile;
pub use timing::*;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "mr_game_and_watch",
        fighter_kind: 28,
        aerial_execution: super::execution::AerialExecution::ADAPTIVE_JUMP,
        ledge: LEDGE_DATA,
        menu_options: crate::ledge_trap::fighters::mr_game_and_watch::MENU,
    }
}
