//! Verified move windows: https://ultimateframedata.com/pikachu
//! Only continuous active intervals are used, never gaps between multihits.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};
pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 3, last: 6 },
    down_tilt: MoveFrames { first: 7, last: 8 },
    forward_tilt: MoveFrames { first: 6, last: 8 },
    down_smash: MoveFrames { first: 8, last: 9 },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 11.9,
        standing: 7,
        dash: 11,
        turn_or_shield: 12,
        shield: 11,
        active_frames: 2,
    },
    normal_hit_lead_frames: 3,
    common_roll_distance: 25.0,
};

// Native throw release or tilt hit frame, and the paired aerial first hit.
