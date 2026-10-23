//! mii_gunner: compiled move intervals from https://ultimateframedata.com/mii_gunner
//! Reference movement (neutral state): Gravity: 0.098; Walk Speed: 1.00; Run Speed: 1.37;
//! Initial Dash: 1.63; Air Speed: 0.93; Total Air Acceleration: 0.053; Fall Speed / Fast
//! Fall Speed: 1.45 / 2.32; SH / FH / SHFF / FHFF Frames: 34 / 49 / 24 / 35
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
//! Only shared normals/aerials/grab; no assumptions about selected custom specials.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 8, last: 21 },
    down_tilt: MoveFrames { first: 8, last: 10 },
    forward_tilt: MoveFrames { first: 7, last: 12 },
    down_smash: MoveFrames { first: 9, last: 9 },
    down_smash_back: Some(MoveFrames {
        first: 23,
        last: 25,
    }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 14.8,
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
    last: 17,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 12,
    last: 20,
};
pub const UP_AIR: MoveFrames = MoveFrames {
    first: 17,
    last: 17,
};
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 20,
    last: 25,
};
