//! inkling: compiled move intervals from https://ultimateframedata.com/inkling
//! Reference movement (neutral state): Gravity: 0.087; Walk Speed: 1.134; Run Speed: 1.925;
//! Initial Dash: 2.118; Air Speed: 1.208; Total Air Acceleration: 0.08; Fall Speed / Fast
//! Fall Speed: 1.58 / 2.528; SH / FH / SHFF / FHFF Frames: 40 / 52 / 27 / 36
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
//! Down-smash uses the conservative initial 11-12 / 20-21 intervals; no ink refill or
//! resource edits.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 6, last: 9 },
    down_tilt: MoveFrames { first: 5, last: 8 },
    forward_tilt: MoveFrames { first: 8, last: 10 },
    down_smash: MoveFrames {
        first: 11,
        last: 12,
    },
    down_smash_back: Some(MoveFrames {
        first: 20,
        last: 21,
    }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 12.0,
        standing: 8,
        dash: 9,
        turn_or_shield: 10,
        shield: 12,
        active_frames: 2,
    },
    normal_hit_lead_frames: 3,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 7, last: 12 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 16,
    last: 17,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 10,
    last: 21,
};
pub const UP_AIR: MoveFrames = MoveFrames {
    first: 12,
    last: 15,
};
