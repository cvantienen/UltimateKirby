//! Compiled luigi registration. No fighter files load from SD.
mod timing;
use super::FighterProfile;
pub use timing::LEDGE_DATA;

pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "luigi",
        fighter_kind: 9,
        aerial_execution: super::execution::AerialExecution::ADAPTIVE_JUMP,
        ledge: LEDGE_DATA,

        menu_options: crate::ledge_trap::fighters::luigi::MENU,
    }
}
