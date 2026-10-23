//! ridley: compiled move intervals from https://ultimateframedata.com/ridley
//! Reference movement (neutral state): Gravity: 0.090; Walk Speed: 1.1; Run Speed: 2.2;
//! Initial Dash: 1.8; Air Speed: 1.05; Total Air Acceleration: 0.07; Fall Speed / Fast Fall
//! Speed: 1.78 / 2.848; SH / FH / SHFF / FHFF Frames: 35 / 51 / 23 / 34
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 8, last: 20 },
    down_tilt: MoveFrames { first: 9, last: 11 },
    forward_tilt: MoveFrames {
        first: 10,
        last: 11,
    },
    down_smash: MoveFrames {
        first: 24,
        last: 27,
    },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 16.5,
        standing: 8,
        dash: 11,
        turn_or_shield: 12,
        shield: 12,
        active_frames: 3,
    },
    normal_hit_lead_frames: 8,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 8, last: 12 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 18,
    last: 21,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 10,
    last: 11,
};
pub const UP_AIR: MoveFrames = MoveFrames {
    first: 11,
    last: 13,
};
