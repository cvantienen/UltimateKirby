//! bowser_jr: compiled move intervals from https://ultimateframedata.com/bowser_jr
//! Reference movement (neutral state): Gravity: 0.092; Walk Speed: 0.924; Run Speed: 1.566;
//! Initial Dash: 1.76; Air Speed: 1.134; Total Air Acceleration: 0.08; Fall Speed / Fast
//! Fall Speed: 1.65 / 2.64; SH / FH / SHFF / FHFF Frames: 38 / 52 / 26 / 36
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 7, last: 19 },
    down_tilt: MoveFrames { first: 4, last: 6 },
    forward_tilt: MoveFrames { first: 7, last: 9 },
    down_smash: MoveFrames {
        first: 12,
        last: 14,
    },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 18.0,
        standing: 11,
        dash: 14,
        turn_or_shield: 15,
        shield: 15,
        active_frames: 3,
    },
    normal_hit_lead_frames: 8,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 7, last: 14 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 18,
    last: 18,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 10,
    last: 23,
};
pub const UP_AIR: MoveFrames = MoveFrames { first: 6, last: 13 };
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 15,
    last: 15,
};
