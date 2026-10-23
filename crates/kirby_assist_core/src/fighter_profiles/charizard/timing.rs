//! charizard: compiled move intervals from https://ultimateframedata.com/charizard
//! Reference movement (neutral state): Gravity: 0.110; Walk Speed: 1.187; Run Speed: 2.2;
//! Initial Dash: 2.288; Air Speed: 1.103; Total Air Acceleration: 0.06; Fall Speed / Fast
//! Fall Speed: 1.52 / 2.432; SH / FH / SHFF / FHFF Frames: 33 / 48 / 24 / 34
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 8, last: 20 },
    down_tilt: MoveFrames { first: 9, last: 10 },
    forward_tilt: MoveFrames {
        first: 12,
        last: 13,
    },
    down_smash: MoveFrames {
        first: 14,
        last: 16,
    },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 21.4,
        standing: 8,
        dash: 11,
        turn_or_shield: 12,
        shield: 12,
        active_frames: 3,
    },
    normal_hit_lead_frames: 8,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 9, last: 12 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 22,
    last: 24,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 8, last: 12 };
pub const UP_AIR: MoveFrames = MoveFrames {
    first: 12,
    last: 15,
};
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 18,
    last: 25,
};
