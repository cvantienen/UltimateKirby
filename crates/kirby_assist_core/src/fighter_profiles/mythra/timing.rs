//! Independent reference facts: https://ultimateframedata.com/mythra
//! Nair 8-19 is a multihit envelope, not proof every enclosed frame is active.
//! Actual move acceptance and motion remain native; no physics/timing writes.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};
pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 8, last: 19 },
    down_tilt: MoveFrames { first: 5, last: 7 },
    forward_tilt: MoveFrames { first: 8, last: 9 },
    down_smash: MoveFrames { first: 8, last: 10 },
    down_smash_back: Some(MoveFrames {
        first: 14,
        last: 16,
    }),
    down_special_startup: None,
    up_special_startup: Some(10),
    grab: GrabTiming {
        standing_range: 12.,
        standing: 6,
        dash: 11,
        turn_or_shield: 12,
        shield: 10,
        active_frames: 2,
    },
    common_roll_distance: 25.,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 7, last: 12 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 14,
    last: 17,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 8, last: 11 };
pub const UP_AIR: MoveFrames = MoveFrames { first: 9, last: 13 };
