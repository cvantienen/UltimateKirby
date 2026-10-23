//! Sonic's compiled ledge registration; no fighter file loads at runtime.
mod timing;
use super::FighterProfile;
pub use timing::LEDGE_DATA;

// FIGHTER_KIND_SONIC in the pinned skyline-smash lua_const.rs.
pub const FIGHTER_KIND: i32 = 0x29;

pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "sonic",
        fighter_kind: FIGHTER_KIND,
        aerial_execution: super::execution::AerialExecution::ADAPTIVE_JUMP,
        ledge: LEDGE_DATA,
        menu_options: crate::ledge_trap::fighters::sonic::MENU,
    }
}
