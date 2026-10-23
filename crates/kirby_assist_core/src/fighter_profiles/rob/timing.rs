//! rob: compiled move intervals from https://ultimateframedata.com/rob
//! Reference movement (neutral state): Gravity: 0.090; Walk Speed: 1.178; Run Speed: 1.725;
//! Initial Dash: 2.002; Air Speed: 1.134; Total Air Acceleration: 0.085; Fall Speed / Fast
//! Fall Speed: 1.6 / 2.56; SH / FH / SHFF / FHFF Frames: 40 / 56 / 28 / 38
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames {
        first: 14,
        last: 32,
    },
    down_tilt: MoveFrames { first: 3, last: 3 },
    forward_tilt: MoveFrames { first: 7, last: 9 },
    down_smash: MoveFrames { first: 7, last: 8 },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 14.4,
        standing: 6,
        dash: 9,
        turn_or_shield: 10,
        shield: 10,
        active_frames: 2,
    },
    normal_hit_lead_frames: 8,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 4, last: 7 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 16,
    last: 17,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 6, last: 8 };
pub const UP_AIR: MoveFrames = MoveFrames { first: 7, last: 7 };
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 20,
    last: 26,
};
