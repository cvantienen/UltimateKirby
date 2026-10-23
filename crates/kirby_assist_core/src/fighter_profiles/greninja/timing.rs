//! greninja: compiled move intervals from https://ultimateframedata.com/greninja
//! Reference movement (neutral state): Gravity: 0.180; Walk Speed: 1.502; Run Speed: 2.288;
//! Initial Dash: 2.178; Air Speed: 1.239; Total Air Acceleration: 0.08; Fall Speed / Fast
//! Fall Speed: 1.85 / 2.96; SH / FH / SHFF / FHFF Frames: 32 / 49 / 24 / 35
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames {
        first: 12,
        last: 19,
    },
    down_tilt: MoveFrames { first: 5, last: 6 },
    forward_tilt: MoveFrames {
        first: 10,
        last: 12,
    },
    down_smash: MoveFrames {
        first: 11,
        last: 13,
    },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 18.5,
        standing: 10,
        dash: 13,
        turn_or_shield: 14,
        shield: 14,
        active_frames: 2,
    },
    normal_hit_lead_frames: 7,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 9, last: 12 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 13,
    last: 15,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 16,
    last: 17,
};
pub const UP_AIR: MoveFrames = MoveFrames { first: 7, last: 8 };
