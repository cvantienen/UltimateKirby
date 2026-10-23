//! lucas: compiled move intervals from https://ultimateframedata.com/lucas
//! Reference movement (neutral state): Gravity: 0.090; Walk Speed: 0.893; Run Speed: 1.65;
//! Initial Dash: 1.815; Air Speed: 1.155; Total Air Acceleration: 0.095; Fall Speed / Fast
//! Fall Speed: 1.37 / 2.192; SH / FH / SHFF / FHFF Frames: 34 / 50 / 23 / 35
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 7, last: 7 },
    down_tilt: MoveFrames { first: 3, last: 4 },
    forward_tilt: MoveFrames { first: 7, last: 9 },
    down_smash: MoveFrames {
        first: 20,
        last: 22,
    },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 22.9,
        standing: 12,
        dash: 14,
        turn_or_shield: 15,
        shield: 16,
        active_frames: 6,
    },
    normal_hit_lead_frames: 0,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 4, last: 4 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 14,
    last: 15,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 9, last: 12 };
pub const UP_AIR: MoveFrames = MoveFrames { first: 7, last: 10 };
