//! snake: compiled move intervals from https://ultimateframedata.com/snake
//! Reference movement (neutral state): Gravity: 0.080; Walk Speed: 0.882; Run Speed: 1.595;
//! Initial Dash: 1.76; Air Speed: 0.987; Total Air Acceleration: 0.03; Fall Speed / Fast
//! Fall Speed: 1.73 / 2.768; SH / FH / SHFF / FHFF Frames: 37 / 42 / 23 / 28
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames {
        first: 10,
        last: 11,
    },
    down_tilt: MoveFrames { first: 6, last: 7 },
    forward_tilt: MoveFrames { first: 4, last: 5 },
    down_smash: MoveFrames { first: 8, last: 10 },
    down_smash_back: Some(MoveFrames {
        first: 20,
        last: 22,
    }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 15.5,
        standing: 8,
        dash: 11,
        turn_or_shield: 9,
        shield: 12,
        active_frames: 2,
    },
    normal_hit_lead_frames: 1,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 6, last: 13 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 41,
    last: 43,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 23,
    last: 26,
};
pub const UP_AIR: MoveFrames = MoveFrames {
    first: 10,
    last: 23,
};
