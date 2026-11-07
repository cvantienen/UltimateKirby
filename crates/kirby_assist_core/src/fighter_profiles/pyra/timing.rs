//! Independent reference facts: https://ultimateframedata.com/pyra
//! Actual move acceptance and motion remain native; no physics/timing writes.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};
pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames {
        first: 11,
        last: 22,
    },
    down_tilt: MoveFrames { first: 9, last: 11 },
    forward_tilt: MoveFrames {
        first: 12,
        last: 13,
    },
    down_smash: MoveFrames {
        first: 12,
        last: 14,
    },
    down_smash_back: Some(MoveFrames {
        first: 18,
        last: 20,
    }),
    down_special_startup: None, // Swap is never an automatic attack/reflector.
    up_special_startup: Some(13),
    grab: GrabTiming {
        standing_range: 12.,
        standing: 7,
        dash: 12,
        turn_or_shield: 13,
        shield: 11,
        active_frames: 2,
    },
};
pub const UP_TILT: MoveFrames = MoveFrames {
    first: 11,
    last: 16,
};
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 20,
    last: 23,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 11,
    last: 14,
};
pub const UP_AIR: MoveFrames = MoveFrames {
    first: 13,
    last: 17,
};
