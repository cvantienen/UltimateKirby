//! ness: compiled move intervals from https://ultimateframedata.com/ness
//! Reference movement (neutral state): Gravity: 0.077; Walk Speed: 0.907; Run Speed: 1.609;
//! Initial Dash: 1.826; Air Speed: 1.007; Total Air Acceleration: 0.1; Fall Speed / Fast
//! Fall Speed: 1.31 / 2.096; SH / FH / SHFF / FHFF Frames: 42 / 58 / 29 / 41
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
//! Front yo-yo interval 23-25; rear interval 12-15. No charge-hold trap is scripted.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 5, last: 15 },
    down_tilt: MoveFrames { first: 3, last: 4 },
    forward_tilt: MoveFrames { first: 7, last: 10 },
    down_smash: MoveFrames {
        first: 23,
        last: 25,
    },
    down_smash_back: Some(MoveFrames {
        first: 12,
        last: 15,
    }),
    down_special_startup: None,
    up_special_startup: None,
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
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 5, last: 8 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 21,
    last: 22,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 8, last: 8 };
pub const UP_AIR: MoveFrames = MoveFrames { first: 8, last: 8 };
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 18,
    last: 24,
};
