//! ike: compiled move intervals from https://ultimateframedata.com/ike
//! Reference movement (neutral state): Gravity: 0.092; Walk Speed: 0.912; Run Speed: 1.507;
//! Initial Dash: 1.815; Air Speed: 1.134; Total Air Acceleration: 0.05; Fall Speed / Fast
//! Fall Speed: 1.65 / 2.64; SH / FH / SHFF / FHFF Frames: 37 / 48 / 26 / 33
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames {
        first: 10,
        last: 22,
    },
    down_tilt: MoveFrames { first: 7, last: 8 },
    forward_tilt: MoveFrames {
        first: 12,
        last: 13,
    },
    down_smash: MoveFrames {
        first: 13,
        last: 15,
    },
    down_smash_back: Some(MoveFrames {
        first: 32,
        last: 36,
    }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 11.5,
        standing: 7,
        dash: 10,
        turn_or_shield: 11,
        shield: 11,
        active_frames: 2,
    },
    normal_hit_lead_frames: 8,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames {
    first: 11,
    last: 21,
};
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 31,
    last: 35,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 11,
    last: 15,
};
pub const UP_AIR: MoveFrames = MoveFrames {
    first: 13,
    last: 18,
};
