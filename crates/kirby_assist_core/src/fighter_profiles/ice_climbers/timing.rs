//! ice_climbers: compiled move intervals from https://ultimateframedata.com/ice_climbers
//! Reference movement (neutral state): Gravity: 0.082; Walk Speed: 1.008 (1.1088); Run
//! Speed: 1.53 (1.683); Initial Dash: 1.68; Air Speed: 0.83 (0.8715); Total Air
//! Acceleration: 0.09 (0.0945); Fall Speed / Fast Fall Speed: 1.3 / 2.08; SH / FH / SHFF /
//! FHFF Frames: 41 / 62 / 29 / 44
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
//! Leader/follower use the same listed move frames. No desync, partner control or
//! guaranteed double hit.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 6, last: 23 },
    down_tilt: MoveFrames { first: 8, last: 11 },
    forward_tilt: MoveFrames { first: 9, last: 10 },
    down_smash: MoveFrames { first: 9, last: 11 },
    down_smash_back: Some(MoveFrames {
        first: 16,
        last: 16,
    }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 10.0,
        standing: 8,
        dash: 10,
        turn_or_shield: 10,
        shield: 12,
        active_frames: 2,
    },
    normal_hit_lead_frames: 8,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 8, last: 9 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 11,
    last: 13,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 19,
    last: 20,
};
pub const UP_AIR: MoveFrames = MoveFrames { first: 7, last: 11 };
