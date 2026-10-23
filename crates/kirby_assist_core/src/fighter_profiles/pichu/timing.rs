//! pichu: compiled move intervals from https://ultimateframedata.com/pichu
//! Reference movement (neutral state): Gravity: 0.140; Walk Speed: 1.302; Run Speed: 1.892;
//! Initial Dash: 1.98; Air Speed: 1.029; Total Air Acceleration: 0.1; Fall Speed / Fast
//! Fall Speed: 1.9 / 2.5; SH / FH / SHFF / FHFF Frames: 31 / 45 / 23 / 34
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 3, last: 27 },
    down_tilt: MoveFrames { first: 7, last: 8 },
    forward_tilt: MoveFrames { first: 5, last: 12 },
    down_smash: MoveFrames { first: 8, last: 9 },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 12.0,
        standing: 6,
        dash: 8,
        turn_or_shield: 9,
        shield: 10,
        active_frames: 2,
    },
    normal_hit_lead_frames: 8,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 7, last: 14 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 16,
    last: 18,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 10,
    last: 12,
};
pub const UP_AIR: MoveFrames = MoveFrames { first: 4, last: 9 };
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 14,
    last: 26,
};
