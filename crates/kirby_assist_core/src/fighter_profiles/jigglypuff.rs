//! Compiled jigglypuff profile. No fighter file is read at runtime.
mod timing;
use super::FighterProfile;
pub use timing::*;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "jigglypuff",
        fighter_kind: 12,
        aerial_execution: super::execution::AerialExecution::ADAPTIVE_JUMP,
        ledge: LEDGE_DATA,
        menu_options: crate::ledge_trap::fighters::jigglypuff::MENU,
    }
}
