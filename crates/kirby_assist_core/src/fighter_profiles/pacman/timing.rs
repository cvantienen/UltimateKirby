//! Native move windows: https://ultimateframedata.com/pac_man
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};
pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 3, last: 19 },
    down_tilt: MoveFrames { first: 7, last: 9 },
    forward_tilt: MoveFrames { first: 5, last: 7 },
    // Clean/late ghosts remain active without a gap, with different strength.
    down_smash: MoveFrames {
        first: 15,
        last: 28,
    },
    down_smash_back: None,
    down_special_startup: None, // Hydrant placement needs state/geometry handling.
    up_special_startup: Some(4),
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 28.2,
        standing: 12,
        dash: 14,
        turn_or_shield: 15,
        shield: 16,
        active_frames: 22,
    },
    normal_hit_lead_frames: 8,
    common_roll_distance: 25.0,
};
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 16,
    last: 29,
};
pub const UP_AIR: MoveFrames = MoveFrames { first: 9, last: 16 };
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 5, last: 8 };
