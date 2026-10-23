//! Compiled yoshi profile. No fighter file is read at runtime.
mod timing;
use super::FighterProfile;
pub use timing::*;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "yoshi",
        fighter_kind: 5,
        // Up-air/fair keep adaptive hops. Jump-nair has its own immediate
        // short-hop sequence in ledge_trap/fighters/yoshi.rs.
        aerial_execution: super::execution::AerialExecution::ADAPTIVE_JUMP,
        ledge: LEDGE_DATA,

        menu_options: crate::ledge_trap::fighters::yoshi::MENU,
    }
}
