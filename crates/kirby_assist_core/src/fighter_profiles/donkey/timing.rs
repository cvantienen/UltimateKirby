//! Verified move windows: https://ultimateframedata.com/donkey_kong
//! Only continuous active intervals are used, never gaps between multihits.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};
pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames {
        first: 10,
        last: 26,
    },
    down_tilt: MoveFrames { first: 6, last: 7 },
    forward_tilt: MoveFrames { first: 7, last: 9 },
    down_smash: MoveFrames {
        first: 11,
        last: 14,
    },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 19.5,
        standing: 8,
        dash: 11,
        turn_or_shield: 12,
        shield: 12,
        active_frames: 3,
    },
    normal_hit_lead_frames: 8,
    common_roll_distance: 25.0,
};

// Native throw release or tilt hit frame, and the paired aerial first hit.
