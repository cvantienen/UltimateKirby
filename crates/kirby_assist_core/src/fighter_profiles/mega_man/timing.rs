//! mega_man: compiled move intervals from https://ultimateframedata.com/mega_man
//! Reference movement (neutral state): Gravity: 0.107; Walk Speed: 1.132; Run Speed: 1.602;
//! Initial Dash: 1.958; Air Speed: 1.155; Total Air Acceleration: 0.11; Fall Speed / Fast
//! Fall Speed: 1.8 / 2.88; SH / FH / SHFF / FHFF Frames: 29 / 47 / 19 / 33
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
//! Pellet nair/forward-tilt and rear-nair fallback are excluded because their native action
//! states differ. Tornado up-air is not a body anti-air.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 7, last: 9 },
    down_tilt: MoveFrames { first: 5, last: 21 },
    forward_tilt: MoveFrames { first: 7, last: 8 },
    down_smash: MoveFrames {
        first: 17,
        last: 33,
    },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 12.5,
        standing: 6,
        dash: 9,
        turn_or_shield: 10,
        shield: 10,
        active_frames: 2,
    },
    normal_hit_lead_frames: 2,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 6, last: 16 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 19,
    last: 19,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 9, last: 17 };
pub const UP_AIR: MoveFrames = MoveFrames {
    first: 11,
    last: 11,
};
