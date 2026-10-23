//! ivysaur: compiled move intervals from https://ultimateframedata.com/ivysaur
//! Reference movement (neutral state): Gravity: 0.082; Walk Speed: 1.103; Run Speed: 1.595;
//! Initial Dash: 1.903; Air Speed: 0.998; Total Air Acceleration: 0.095; Fall Speed / Fast
//! Fall Speed: 1.38 / 2.208; SH / FH / SHFF / FHFF Frames: 39 / 54 / 27 / 38
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 7, last: 7 },
    down_tilt: MoveFrames { first: 4, last: 6 },
    forward_tilt: MoveFrames {
        first: 10,
        last: 10,
    },
    down_smash: MoveFrames {
        first: 13,
        last: 14,
    },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 23.0,
        standing: 13,
        dash: 13,
        turn_or_shield: 13,
        shield: 17,
        active_frames: 2,
    },
    normal_hit_lead_frames: 0,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 7, last: 13 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 15,
    last: 20,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 14,
    last: 18,
};
pub const UP_AIR: MoveFrames = MoveFrames {
    first: 12,
    last: 13,
};
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 11,
    last: 12,
};
