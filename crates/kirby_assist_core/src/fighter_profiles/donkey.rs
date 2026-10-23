//! Compiled donkey kong profile. No fighter file is read at runtime.
mod timing;
use super::FighterProfile;
pub use timing::LEDGE_DATA;
pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "donkey_kong",
        fighter_kind: 1,
        aerial_execution: super::execution::AerialExecution::ADAPTIVE_JUMP,
        ledge: LEDGE_DATA,

        menu_options: crate::ledge_trap::fighters::donkey::MENU,
    }
}
