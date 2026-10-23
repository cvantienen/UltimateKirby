//! Native move data: https://ultimateframedata.com/isabelle
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};
pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 5, last: 25 },
    down_tilt: MoveFrames { first: 9, last: 10 },
    forward_tilt: MoveFrames { first: 8, last: 11 },
    down_smash: MoveFrames { first: 8, last: 9 },
    down_smash_back: Some(MoveFrames {
        first: 12,
        last: 13,
    }),
    down_special_startup: None, // Plant/activation depends on prior trap state.
    up_special_startup: None,   // Balloon Trip is not a damaging punish.
    // UFD's dash startup summary says 15 but active row says 16-17; use 16.
    // Two active frames conservatively fit all three net-grab animations.
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 19.0,
        standing: 14,
        dash: 16,
        turn_or_shield: 17,
        shield: 18,
        active_frames: 2,
    },
    normal_hit_lead_frames: 8,
    common_roll_distance: 25.0,
};
pub const UP_AIR: MoveFrames = MoveFrames { first: 6, last: 31 };
// Slingshot emission/first projectile phase; travel to target is not modeled.
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 10,
    last: 12,
};
// First projectile phase. Bair is slower than fair; do not reuse fair startup.
pub const BACK_AIR: MoveFrames = MoveFrames {
    first: 13,
    last: 15,
};
pub const SLINGSHOT_EARLY_FRAMES: u8 = 2;
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 11,
    last: 28,
};
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 14,
    last: 19,
};
