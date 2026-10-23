//! lucario: compiled move intervals from https://ultimateframedata.com/lucario
//! Reference movement (neutral state): Gravity: 0.084; Walk Speed: 1.103; Run Speed: 1.705;
//! Initial Dash: 2.255; Air Speed: 1.281; Total Air Acceleration: 0.09; Fall Speed / Fast
//! Fall Speed: 1.68 / 2.688; SH / FH / SHFF / FHFF Frames: 41 / 56 / 28 / 38
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames {
        first: 10,
        last: 14,
    },
    down_tilt: MoveFrames { first: 7, last: 10 },
    forward_tilt: MoveFrames {
        first: 12,
        last: 16,
    },
    down_smash: MoveFrames {
        first: 16,
        last: 18,
    },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 13.5,
        standing: 7,
        dash: 10,
        turn_or_shield: 11,
        shield: 11,
        active_frames: 2,
    },
    normal_hit_lead_frames: 4,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 6, last: 13 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 19,
    last: 22,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 7, last: 8 };
pub const UP_AIR: MoveFrames = MoveFrames {
    first: 10,
    last: 13,
};
pub const DOWN_AIR: MoveFrames = MoveFrames { first: 4, last: 5 };
