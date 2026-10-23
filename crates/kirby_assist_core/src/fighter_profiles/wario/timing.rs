//! wario: compiled move intervals from https://ultimateframedata.com/wario
//! Reference movement (neutral state): Gravity: 0.107; Walk Speed: 0.889; Run Speed: 1.65;
//! Initial Dash: 1.837; Air Speed: 1.271; Total Air Acceleration: 0.12; Fall Speed / Fast
//! Fall Speed: 1.61 / 2.576; SH / FH / SHFF / FHFF Frames: 33 / 46 / 23 / 32
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 4, last: 17 },
    down_tilt: MoveFrames { first: 5, last: 5 },
    forward_tilt: MoveFrames {
        first: 12,
        last: 15,
    },
    down_smash: MoveFrames { first: 8, last: 36 },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 14.0,
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
pub const UP_TILT: MoveFrames = MoveFrames { first: 8, last: 15 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 18,
    last: 19,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 5, last: 17 };
pub const UP_AIR: MoveFrames = MoveFrames { first: 8, last: 9 };
