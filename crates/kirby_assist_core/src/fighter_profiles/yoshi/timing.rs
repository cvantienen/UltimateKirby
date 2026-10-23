//! Verified move windows: https://ultimateframedata.com/yoshi
//! Only continuous active intervals are used, never gaps between multihits.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};
pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 3, last: 25 },
    down_tilt: MoveFrames { first: 8, last: 10 },
    forward_tilt: MoveFrames { first: 5, last: 7 },
    down_smash: MoveFrames { first: 7, last: 8 },
    down_smash_back: Some(MoveFrames {
        first: 22,
        last: 23,
    }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 26.0,
        standing: 14,
        dash: 16,
        turn_or_shield: 17,
        shield: 18,
        active_frames: 8,
    },
    normal_hit_lead_frames: NEUTRAL_NAIR_LEAD,
    common_roll_distance: 25.0,
};

pub const UP_TILT: MoveFrames = MoveFrames { first: 8, last: 15 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 14,
    last: 16,
};
pub const UP_AIR: MoveFrames = MoveFrames { first: 5, last: 6 };
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 16,
    last: 20,
};
// Dair has discrete hits, not continuous coverage from 16 through 41. It is
// registered only as an immediate drop chase, never a timed vulnerability window.
// The final frame is used solely to stop outward drift after the last hit.
pub const DOWN_AIR_FIRST: u8 = 16;
pub const DOWN_AIR_LAST: u8 = 41;

// Response choices, separate from native hitbox startup/active-frame data.
pub const NEUTRAL_NAIR_LEAD: u8 = 17;
// Horizontal stick after native neutral-getup nair is confirmed.
pub const NEUTRAL_NAIR_DRIFT: f32 = 0.25;
pub const JUMP_NAIR_DRIFT: f32 = 0.25;
// Stand slightly on the ledge side of the shared roll-coverage estimate.
// This is a spacing adjustment to calibrate on-device, not a hitbox edit.
pub const ROLL_DOWN_SMASH_STANDOFF: f32 = 2.0;
