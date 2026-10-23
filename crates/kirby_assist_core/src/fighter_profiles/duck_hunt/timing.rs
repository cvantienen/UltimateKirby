//! duck_hunt: compiled move intervals from https://ultimateframedata.com/duck_hunt
//! Reference movement (neutral state): Gravity: 0.076; Walk Speed: 1.213; Run Speed: 1.793;
//! Initial Dash: 1.76; Air Speed: 1.155; Total Air Acceleration: 0.08; Fall Speed / Fast
//! Fall Speed: 1.65 / 2.64; SH / FH / SHFF / FHFF Frames: 46 / 56 / 31 / 38
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 6, last: 37 },
    down_tilt: MoveFrames { first: 6, last: 7 },
    forward_tilt: MoveFrames { first: 8, last: 11 },
    down_smash: MoveFrames {
        first: 12,
        last: 13,
    },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 14.5,
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
pub const UP_TILT: MoveFrames = MoveFrames { first: 7, last: 11 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 17,
    last: 18,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 7, last: 11 };
pub const UP_AIR: MoveFrames = MoveFrames { first: 6, last: 7 };
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 14,
    last: 15,
};
