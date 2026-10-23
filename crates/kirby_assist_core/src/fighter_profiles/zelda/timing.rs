//! zelda: compiled move intervals from https://ultimateframedata.com/zelda
//! Reference movement (neutral state): Gravity: 0.071; Walk Speed: 0.914; Run Speed: 1.43;
//! Initial Dash: 1.958; Air Speed: 1.092; Total Air Acceleration: 0.065; Fall Speed / Fast
//! Fall Speed: 1.35 / 2.16; SH / FH / SHFF / FHFF Frames: 41 / 56 / 29 / 39
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 6, last: 7 },
    down_tilt: MoveFrames { first: 5, last: 11 },
    forward_tilt: MoveFrames {
        first: 12,
        last: 13,
    },
    down_smash: MoveFrames { first: 5, last: 6 },
    down_smash_back: Some(MoveFrames {
        first: 13,
        last: 14,
    }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 15.3,
        standing: 10,
        dash: 13,
        turn_or_shield: 14,
        shield: 14,
        active_frames: 2,
    },
    normal_hit_lead_frames: 1,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 7, last: 19 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 16,
    last: 16,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 6, last: 10 };
pub const UP_AIR: MoveFrames = MoveFrames {
    first: 14,
    last: 19,
};
