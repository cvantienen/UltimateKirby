//! palutena: compiled move intervals from https://ultimateframedata.com/palutena
//! Reference movement (neutral state): Gravity: 0.120; Walk Speed: 1.271; Run Speed: 2.077;
//! Initial Dash: 2.035; Air Speed: 1.0; Total Air Acceleration: 0.115; Fall Speed / Fast
//! Fall Speed: 1.55 / 2.48; SH / FH / SHFF / FHFF Frames: 34 / 50 / 24 / 35
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 5, last: 5 },
    down_tilt: MoveFrames {
        first: 14,
        last: 27,
    },
    forward_tilt: MoveFrames {
        first: 14,
        last: 35,
    },
    down_smash: MoveFrames {
        first: 17,
        last: 19,
    },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 13.0,
        standing: 7,
        dash: 10,
        turn_or_shield: 11,
        shield: 11,
        active_frames: 2,
    },
    normal_hit_lead_frames: 0,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 8, last: 8 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 18,
    last: 20,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 9, last: 11 };
pub const UP_AIR: MoveFrames = MoveFrames {
    first: 10,
    last: 10,
};
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 10,
    last: 11,
};
