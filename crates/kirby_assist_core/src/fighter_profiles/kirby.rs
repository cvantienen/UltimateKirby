//! Kirby’s compiled profile. Timing constants live separately from executable
//! response functions and their static menu catalog. No external file is loaded.
mod timing;
pub use timing::{
    DROP_DAIR_FIRST_ACTIVE, DROP_DAIR_SELECT_DRIFT, INHALE_HIT_LEAD, INHALE_STARTUP, JUMP_BAIR,
    JUMP_BAIR_DRIFT, LEDGE_DATA, NAIR_HIT_LEAD, NEUTRAL_BAIR_DRIFT, NEUTRAL_BAIR_LEAD,
    STONE_HIT_LEAD,
};

use super::FighterProfile;

pub const FIGHTER_KIND: i32 = 6;

pub fn profile() -> FighterProfile {
    FighterProfile {
        name: "kirby",
        fighter_kind: FIGHTER_KIND,
        aerial_execution: super::execution::AerialExecution::SHORT,
        ledge: LEDGE_DATA,

        menu_options: crate::ledge_trap::fighters::kirby::MENU,
    }
}
