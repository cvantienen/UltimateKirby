//! Compiled bowser profile. No fighter file is read at runtime.
mod timing;
use super::FighterProfile;
pub use timing::*;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "bowser",
        fighter_kind: 15,
        aerial_execution: super::execution::AerialExecution::ADAPTIVE_JUMP,
        ledge: LEDGE_DATA,
        menu_options: crate::ledge_trap::fighters::bowser::MENU,
    }
}
