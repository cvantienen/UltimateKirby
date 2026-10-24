//! Kirby’s compiled profile. Timing constants live separately from executable
//! response functions and their static menu catalog. No external file is loaded.
mod timing;
pub use timing::{
    DROP_DAIR_FIRST_ACTIVE, DROP_DAIR_SELECT_DRIFT, INHALE_HIT_LEAD, INHALE_STARTUP, JUMP_BAIR,
    JUMP_BAIR_DRIFT, LEDGE_DATA, NAIR_HIT_LEAD, NEUTRAL_BAIR_DRIFT, NEUTRAL_BAIR_LEAD,
    NEUTRAL_DOWN_SMASH_EARLY_FRAMES, NEUTRAL_NAIR_EDGE_INSET, NEUTRAL_NAIR_WALK_STICK,
    ROLL_GRAB_LEDGE_INSET, ROLL_READ_DOWN_SMASH, ROLL_READ_GRAB, STONE_HIT_LEAD, WHIFF_SMASH_REACH,
    WHIFF_SMASH_STARTUP, WHIFF_TILT_REACH,
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
