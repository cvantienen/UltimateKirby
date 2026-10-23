//! diddy_kong: compiled move intervals from https://ultimateframedata.com/diddy_kong
//! Reference movement (neutral state): Gravity: 0.125; Walk Speed: 1.313; Run Speed: 2.006;
//! Initial Dash: 2.09; Air Speed: 0.924; Total Air Acceleration: 0.05; Fall Speed / Fast
//! Fall Speed: 1.75 / 2.8; SH / FH / SHFF / FHFF Frames: 36 / 51 / 26 / 36
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 8, last: 20 },
    down_tilt: MoveFrames { first: 4, last: 5 },
    forward_tilt: MoveFrames {
        first: 10,
        last: 16,
    },
    down_smash: MoveFrames { first: 7, last: 8 },
    down_smash_back: Some(MoveFrames {
        first: 12,
        last: 13,
    }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 11.5,
        standing: 7,
        dash: 10,
        turn_or_shield: 11,
        shield: 11,
        active_frames: 2,
    },
    normal_hit_lead_frames: 8,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 6, last: 11 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 12,
    last: 13,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 6, last: 16 };
pub const UP_AIR: MoveFrames = MoveFrames { first: 4, last: 7 };
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 15,
    last: 16,
};
