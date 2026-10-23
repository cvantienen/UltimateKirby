//! robin: compiled move intervals from https://ultimateframedata.com/robin
//! Reference movement (neutral state): Gravity: 0.089; Walk Speed: 0.936; Run Speed: 1.265;
//! Initial Dash: 1.815; Air Speed: 1.05; Total Air Acceleration: 0.08; Fall Speed / Fast
//! Fall Speed: 1.5 / 2.4; SH / FH / SHFF / FHFF Frames: 38 / 52 / 26 / 36
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
//! Only common early sword intervals are used; no Levin recharge, tome or sword-form automation.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 7, last: 10 },
    down_tilt: MoveFrames { first: 7, last: 8 },
    forward_tilt: MoveFrames { first: 9, last: 10 },
    down_smash: MoveFrames {
        first: 16,
        last: 17,
    },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 13.0,
        standing: 7,
        dash: 10,
        turn_or_shield: 11,
        shield: 11,
        active_frames: 2,
    },
    normal_hit_lead_frames: 3,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 6, last: 14 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 16,
    last: 18,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 12,
    last: 15,
};
pub const UP_AIR: MoveFrames = MoveFrames {
    first: 10,
    last: 13,
};
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 13,
    last: 24,
};
