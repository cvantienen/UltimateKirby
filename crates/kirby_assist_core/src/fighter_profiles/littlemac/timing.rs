//! Native move windows: https://ultimateframedata.com/little_mac
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};
pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 2, last: 2 }, // Tiny one-frame hit, not a lingering default.
    down_tilt: MoveFrames { first: 3, last: 4 },
    forward_tilt: MoveFrames { first: 4, last: 5 }, // Second hit 12-13; never count the gap.
    down_smash: MoveFrames {
        first: 10,
        last: 11,
    },
    down_smash_back: Some(MoveFrames {
        first: 17,
        last: 18,
    }),
    down_special_startup: None, // Counter startup is not an attacking hitbox.
    up_special_startup: Some(3),
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 12.9,
        standing: 9,
        dash: 13,
        turn_or_shield: 14,
        shield: 13,
        active_frames: 2,
    },
    normal_hit_lead_frames: 0,
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 4, last: 10 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 14,
    last: 15,
};
// Down-angled forward smash is slower; never use the neutral smash timing.
pub const LOW_FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 15,
    last: 15,
};
pub const UP_AIR: MoveFrames = MoveFrames { first: 5, last: 8 };
