//! mewtwo: compiled move intervals from https://ultimateframedata.com/mewtwo
//! Reference movement (neutral state): Gravity: 0.082; Walk Speed: 1.26; Run Speed: 2.255;
//! Initial Dash: 2.255; Air Speed: 1.313; Total Air Acceleration: 0.07; Fall Speed / Fast
//! Fall Speed: 1.55 / 2.48; SH / FH / SHFF / FHFF Frames: 41 / 52 / 29 / 35
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 7, last: 8 },
    down_tilt: MoveFrames { first: 6, last: 7 },
    forward_tilt: MoveFrames {
        first: 10,
        last: 11,
    },
    down_smash: MoveFrames {
        first: 21,
        last: 23,
    },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 14.3,
        standing: 7,
        dash: 10,
        turn_or_shield: 11,
        shield: 11,
        active_frames: 2,
    },
    normal_hit_lead_frames: 1,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 8, last: 13 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 19,
    last: 22,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 7, last: 9 };
pub const UP_AIR: MoveFrames = MoveFrames {
    first: 10,
    last: 14,
};
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 15,
    last: 18,
};
