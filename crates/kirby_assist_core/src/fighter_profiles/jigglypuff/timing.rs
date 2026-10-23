//! jigglypuff: compiled move intervals from https://ultimateframedata.com/jigglypuff
//! Reference movement (neutral state): Gravity: 0.053; Walk Speed: 0.735; Run Speed: 1.271;
//! Initial Dash: 1.65; Air Speed: 1.332; Total Air Acceleration: 0.13; Fall Speed / Fast
//! Fall Speed: 0.98 / 1.568; SH / FH / SHFF / FHFF Frames: 41 / 51 / 29 / 35
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 6, last: 30 },
    down_tilt: MoveFrames {
        first: 10,
        last: 12,
    },
    forward_tilt: MoveFrames { first: 7, last: 10 },
    down_smash: MoveFrames {
        first: 14,
        last: 16,
    },
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
pub const UP_TILT: MoveFrames = MoveFrames { first: 9, last: 13 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 16,
    last: 24,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 8, last: 20 };
pub const UP_AIR: MoveFrames = MoveFrames { first: 9, last: 21 };
pub const DOWN_AIR: MoveFrames = MoveFrames { first: 7, last: 8 };
