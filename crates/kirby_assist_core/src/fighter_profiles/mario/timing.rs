//! Verified move windows: https://ultimateframedata.com/mario
//! Only continuous active intervals are used, never gaps between multihits.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};
pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 3, last: 27 },
    down_tilt: MoveFrames { first: 5, last: 7 },
    forward_tilt: MoveFrames { first: 5, last: 7 },
    down_smash: MoveFrames { first: 5, last: 6 },
    down_smash_back: Some(MoveFrames {
        first: 14,
        last: 14,
    }),
    down_special_startup: None,
    up_special_startup: Some(3),
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 12.0,
        standing: 6,
        dash: 9,
        turn_or_shield: 10,
        shield: 10,
        active_frames: 2,
    },
    normal_hit_lead_frames: 8,
    common_roll_distance: 25.0,
};

// Native throw release or tilt hit frame, and the paired aerial first hit.
