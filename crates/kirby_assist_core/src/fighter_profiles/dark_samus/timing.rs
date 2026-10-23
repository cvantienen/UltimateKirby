//! dark_samus: compiled move intervals from https://ultimateframedata.com/dark_samus
//! Reference movement (neutral state): Gravity: 0.075; Walk Speed: 1.115; Run Speed: 1.654;
//! Initial Dash: 1.87; Air Speed: 1.103; Total Air Acceleration: 0.09; Fall Speed / Fast
//! Fall Speed: 1.33 / 2.128; SH / FH / SHFF / FHFF Frames: 44 / 61 / 31 / 43
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 8, last: 11 },
    down_tilt: MoveFrames { first: 6, last: 8 },
    forward_tilt: MoveFrames { first: 8, last: 10 },
    down_smash: MoveFrames { first: 9, last: 10 },
    down_smash_back: Some(MoveFrames {
        first: 17,
        last: 18,
    }),
    down_special_startup: None,
    up_special_startup: Some(4),
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 42.2,
        standing: 15,
        dash: 17,
        turn_or_shield: 18,
        shield: 19,
        active_frames: 8,
    },
    normal_hit_lead_frames: 3,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames {
    first: 15,
    last: 18,
};
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 10,
    last: 11,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 6, last: 7 };
pub const UP_AIR: MoveFrames = MoveFrames { first: 5, last: 5 };
