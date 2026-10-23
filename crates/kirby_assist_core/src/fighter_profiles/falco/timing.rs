//! falco: compiled move intervals from https://ultimateframedata.com/falco
//! Reference movement (neutral state): Gravity: 0.130; Walk Speed: 1.344; Run Speed: 1.619;
//! Initial Dash: 2.035; Air Speed: 0.977; Total Air Acceleration: 0.1; Fall Speed / Fast
//! Fall Speed: 1.8 / 2.88; SH / FH / SHFF / FHFF Frames: 32 / 57 / 23 / 41
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 3, last: 13 },
    down_tilt: MoveFrames { first: 8, last: 10 },
    forward_tilt: MoveFrames { first: 6, last: 8 },
    down_smash: MoveFrames { first: 8, last: 10 },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 12.0,
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
pub const UP_TILT: MoveFrames = MoveFrames { first: 5, last: 9 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 17,
    last: 20,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 7, last: 7 };
pub const UP_AIR: MoveFrames = MoveFrames { first: 7, last: 11 };
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 10,
    last: 24,
};
