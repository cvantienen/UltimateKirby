//! Compiled mii_swordfighter profile. No fighter file is read at runtime.
mod timing;
use super::FighterProfile;
pub use timing::*;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "mii_swordfighter",
        fighter_kind: 73,
        aerial_execution: super::execution::AerialExecution::ADAPTIVE_JUMP,
        ledge: LEDGE_DATA,
        menu_options: crate::ledge_trap::fighters::mii_swordfighter::MENU,
    }
}
