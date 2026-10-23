//! toon_link: compiled move intervals from https://ultimateframedata.com/toon_link
//! Reference movement (neutral state): Gravity: 0.081; Walk Speed: 1.288; Run Speed: 1.906;
//! Initial Dash: 1.914; Air Speed: 1.05; Total Air Acceleration: 0.09; Fall Speed / Fast
//! Fall Speed: 1.38 / 2.208; SH / FH / SHFF / FHFF Frames: 40 / 56 / 28 / 39
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 6, last: 7 },
    down_tilt: MoveFrames { first: 9, last: 10 },
    forward_tilt: MoveFrames { first: 9, last: 13 },
    down_smash: MoveFrames { first: 9, last: 10 },
    down_smash_back: Some(MoveFrames {
        first: 17,
        last: 18,
    }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 28.2,
        standing: 12,
        dash: 14,
        turn_or_shield: 15,
        shield: 16,
        active_frames: 7,
    },
    normal_hit_lead_frames: 1,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 8, last: 12 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 16,
    last: 17,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 14,
    last: 15,
};
pub const UP_AIR: MoveFrames = MoveFrames {
    first: 11,
    last: 40,
};
