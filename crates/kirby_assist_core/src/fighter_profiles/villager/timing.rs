//! villager: compiled move intervals from https://ultimateframedata.com/villager
//! Reference movement (neutral state): Gravity: 0.078; Walk Speed: 1.092; Run Speed: 1.397;
//! Initial Dash: 1.815; Air Speed: 0.987; Total Air Acceleration: 0.06; Fall Speed / Fast
//! Fall Speed: 1.32 / 2.112; SH / FH / SHFF / FHFF Frames: 40 / 56 / 28 / 39
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
//! Slingshot and bowling ball are area-denial attempts; projectile travel/turnip count are
//! not simulated.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 3, last: 23 },
    down_tilt: MoveFrames { first: 9, last: 11 },
    forward_tilt: MoveFrames { first: 8, last: 11 },
    down_smash: MoveFrames { first: 9, last: 11 },
    down_smash_back: Some(MoveFrames {
        first: 29,
        last: 31,
    }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 19.0,
        standing: 14,
        dash: 16,
        turn_or_shield: 17,
        shield: 18,
        active_frames: 3,
    },
    normal_hit_lead_frames: 8,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 7, last: 18 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 25,
    last: 25,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 10,
    last: 23,
};
pub const UP_AIR: MoveFrames = MoveFrames { first: 6, last: 21 };
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 11,
    last: 26,
};
