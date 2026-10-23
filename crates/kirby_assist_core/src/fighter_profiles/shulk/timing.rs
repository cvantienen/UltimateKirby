//! shulk: compiled move intervals from https://ultimateframedata.com/shulk
//! Reference movement (neutral state): Gravity: 0.098; Walk Speed: 1.155; Run Speed: 1.672;
//! Initial Dash: 1.87; Air Speed: 1.113; Total Air Acceleration: 0.05; Fall Speed / Fast
//! Fall Speed: 1.58 / 2.528; SH / FH / SHFF / FHFF Frames: 37 / 50 / 26 / 36
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
//! No Monado switching. Reference movement values describe neutral art; active arts still
//! need device verification.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames {
        first: 13,
        last: 30,
    },
    down_tilt: MoveFrames {
        first: 10,
        last: 11,
    },
    forward_tilt: MoveFrames {
        first: 12,
        last: 13,
    },
    down_smash: MoveFrames {
        first: 18,
        last: 19,
    },
    down_smash_back: Some(MoveFrames {
        first: 23,
        last: 24,
    }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 12.5,
        standing: 7,
        dash: 10,
        turn_or_shield: 11,
        shield: 11,
        active_frames: 2,
    },
    normal_hit_lead_frames: 8,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames {
    first: 11,
    last: 23,
};
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 14,
    last: 15,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 14,
    last: 18,
};
pub const UP_AIR: MoveFrames = MoveFrames {
    first: 14,
    last: 16,
};
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 14,
    last: 15,
};
