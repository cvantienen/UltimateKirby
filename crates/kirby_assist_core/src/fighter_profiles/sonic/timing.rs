//! Move windows: https://ultimateframedata.com/sonic
//! These describe native attacks; they do not change hitboxes or recovery.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    // Three consecutive strength phases remain active without an inactive gap.
    neutral_air: MoveFrames { first: 6, last: 38 },
    down_tilt: MoveFrames { first: 6, last: 7 },
    // First hit is frame 6 only; schedule the continuous second hit at 8-11.
    // Never treat the frame-7 gap as coverage.
    forward_tilt: MoveFrames { first: 8, last: 11 },
    down_smash: MoveFrames {
        first: 12,
        last: 13,
    },
    // Both sides use the same active interval.
    down_smash_back: None,
    // Spin Charge needs a dedicated charge/release recipe, not Kirby's Stone.
    down_special_startup: None,
    // Grounded Spring Jump has no damaging hitbox; omit shield-Up-B punish.
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 12.0,
        standing: 7,
        dash: 10,
        turn_or_shield: 11,
        shield: 11,
        active_frames: 2,
    },
    normal_hit_lead_frames: 8,
    common_roll_distance: 25.0,
};
