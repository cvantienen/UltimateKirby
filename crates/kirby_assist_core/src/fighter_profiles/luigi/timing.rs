//! Move frames verified against https://ultimateframedata.com/luigi .
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    // Only continuous active intervals: never count gaps between separate hits.
    neutral_air: MoveFrames { first: 3, last: 31 },
    down_tilt: MoveFrames { first: 5, last: 8 },
    forward_tilt: MoveFrames { first: 5, last: 7 },
    down_smash: MoveFrames { first: 6, last: 7 },
    down_smash_back: Some(MoveFrames {
        first: 14,
        last: 15,
    }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 29.8,
        standing: 14,
        dash: 16,
        turn_or_shield: 18,
        shield: 18,
        active_frames: 2,
    },
    normal_hit_lead_frames: 8,
    common_roll_distance: 25.0,
};
