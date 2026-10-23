//! marth: compiled move intervals from https://ultimateframedata.com/marth
//! Reference movement (neutral state): Gravity: 0.075; Walk Speed: 1.575; Run Speed: 1.964;
//! Initial Dash: 2.255; Air Speed: 1.071; Total Air Acceleration: 0.08; Fall Speed / Fast
//! Fall Speed: 1.58 / 2.528; SH / FH / SHFF / FHFF Frames: 41 / 55 / 28 / 38
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 6, last: 7 },
    down_tilt: MoveFrames { first: 7, last: 8 },
    forward_tilt: MoveFrames { first: 8, last: 11 },
    down_smash: MoveFrames { first: 6, last: 7 },
    down_smash_back: Some(MoveFrames {
        first: 21,
        last: 23,
    }),
    down_special_startup: None,
    up_special_startup: Some(5),
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 14.5,
        standing: 6,
        dash: 9,
        turn_or_shield: 10,
        shield: 10,
        active_frames: 2,
    },
    normal_hit_lead_frames: 1,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 6, last: 12 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 10,
    last: 13,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 6, last: 8 };
pub const UP_AIR: MoveFrames = MoveFrames { first: 5, last: 9 };
pub const DOWN_AIR: MoveFrames = MoveFrames { first: 9, last: 13 };
