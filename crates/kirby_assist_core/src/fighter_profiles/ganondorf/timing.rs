//! ganondorf: compiled move intervals from https://ultimateframedata.com/ganondorf
//! Reference movement (neutral state): Gravity: 0.108; Walk Speed: 0.767; Run Speed: 1.34;
//! Initial Dash: 1.87; Air Speed: 0.83; Total Air Acceleration: 0.04; Fall Speed / Fast
//! Fall Speed: 1.65 / 2.64; SH / FH / SHFF / FHFF Frames: 30 / 41 / 20 / 29
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 7, last: 12 },
    down_tilt: MoveFrames {
        first: 10,
        last: 12,
    },
    forward_tilt: MoveFrames {
        first: 10,
        last: 12,
    },
    down_smash: MoveFrames {
        first: 15,
        last: 18,
    },
    down_smash_back: Some(MoveFrames {
        first: 35,
        last: 38,
    }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 14.0,
        standing: 8,
        dash: 11,
        turn_or_shield: 12,
        shield: 12,
        active_frames: 3,
    },
    normal_hit_lead_frames: 5,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames {
    first: 60,
    last: 63,
};
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 29,
    last: 31,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 14,
    last: 19,
};
pub const UP_AIR: MoveFrames = MoveFrames { first: 8, last: 16 };
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 16,
    last: 18,
};
