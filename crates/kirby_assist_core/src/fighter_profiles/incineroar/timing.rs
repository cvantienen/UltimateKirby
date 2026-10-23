//! incineroar: compiled move intervals from https://ultimateframedata.com/incineroar
//! Reference movement (neutral state): Gravity: 0.126; Walk Speed: 0.62; Run Speed: 1.18;
//! Initial Dash: 1.76; Air Speed: 0.88; Total Air Acceleration: 0.07; Fall Speed / Fast
//! Fall Speed: 1.76 / 2.816; SH / FH / SHFF / FHFF Frames: 30 / 43 / 21 / 31
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 5, last: 25 },
    down_tilt: MoveFrames { first: 9, last: 11 },
    forward_tilt: MoveFrames {
        first: 12,
        last: 14,
    },
    down_smash: MoveFrames {
        first: 18,
        last: 20,
    },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 16.0,
        standing: 7,
        dash: 11,
        turn_or_shield: 12,
        shield: 11,
        active_frames: 3,
    },
    normal_hit_lead_frames: 8,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 6, last: 11 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 16,
    last: 18,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 8, last: 14 };
pub const UP_AIR: MoveFrames = MoveFrames { first: 7, last: 11 };
