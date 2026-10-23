//! Move frames verified against https://ultimateframedata.com/captain_falcon .
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    // Only continuous active intervals: never count gaps between separate hits.
    neutral_air: MoveFrames { first: 7, last: 8 },
    down_tilt: MoveFrames {
        first: 11,
        last: 12,
    },
    forward_tilt: MoveFrames { first: 7, last: 10 },
    down_smash: MoveFrames {
        first: 19,
        last: 20,
    },
    down_smash_back: Some(MoveFrames {
        first: 29,
        last: 30,
    }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 13.0,
        standing: 6,
        dash: 9,
        turn_or_shield: 10,
        shield: 10,
        active_frames: 2,
    },
    normal_hit_lead_frames: 1,
    common_roll_distance: 25.0,
};
