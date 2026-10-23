//! Native move data: https://ultimateframedata.com/king_k_rool
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};
pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 7, last: 30 },
    down_tilt: MoveFrames {
        first: 13,
        last: 15,
    },
    forward_tilt: MoveFrames {
        first: 12,
        last: 13,
    },
    // Body hit only: frame-25 ground shockwave has different reach/geometry.
    down_smash: MoveFrames {
        first: 22,
        last: 24,
    },
    down_smash_back: None,
    down_special_startup: None, // Counter window is not an attacking hitbox.
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 20.0,
        standing: 8,
        dash: 11,
        turn_or_shield: 12,
        shield: 12,
        active_frames: 3,
    },
    normal_hit_lead_frames: 8,
    common_roll_distance: 25.0,
};
pub const UP_AIR: MoveFrames = MoveFrames { first: 7, last: 19 };
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 11,
    last: 17,
};
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 14,
    last: 21,
};
