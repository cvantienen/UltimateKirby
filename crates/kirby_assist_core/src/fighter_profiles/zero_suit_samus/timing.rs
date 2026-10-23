//! zero_suit_samus: compiled move intervals from https://ultimateframedata.com/zero_suit_samus
//! Reference movement (neutral state): Gravity: 0.120; Walk Speed: 1.47; Run Speed: 2.31;
//! Initial Dash: 2.42; Air Speed: 1.26; Total Air Acceleration: 0.095; Fall Speed / Fast
//! Fall Speed: 1.7 / 2.72; SH / FH / SHFF / FHFF Frames: 38 / 55 / 27 / 39
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames {
        first: 10,
        last: 11,
    },
    down_tilt: MoveFrames { first: 8, last: 9 },
    forward_tilt: MoveFrames { first: 6, last: 8 },
    down_smash: MoveFrames {
        first: 20,
        last: 24,
    },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: Some(6),
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 37.0,
        standing: 15,
        dash: 17,
        turn_or_shield: 18,
        shield: 19,
        active_frames: 11,
    },
    normal_hit_lead_frames: 1,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 3, last: 4 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 13,
    last: 14,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 6, last: 7 };
pub const UP_AIR: MoveFrames = MoveFrames { first: 6, last: 12 };
