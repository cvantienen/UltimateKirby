//! corrin: compiled move intervals from https://ultimateframedata.com/corrin
//! Reference movement (neutral state): Gravity: 0.092; Walk Speed: 1.208; Run Speed: 1.595;
//! Initial Dash: 1.892; Air Speed: 1.019; Total Air Acceleration: 0.05; Fall Speed / Fast
//! Fall Speed: 1.65 / 2.64; SH / FH / SHFF / FHFF Frames: 37 / 50 / 26 / 35
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 6, last: 19 },
    down_tilt: MoveFrames { first: 5, last: 7 },
    forward_tilt: MoveFrames { first: 8, last: 9 },
    down_smash: MoveFrames {
        first: 13,
        last: 15,
    },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 11.8,
        standing: 7,
        dash: 10,
        turn_or_shield: 11,
        shield: 11,
        active_frames: 2,
    },
    normal_hit_lead_frames: 8,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 7, last: 16 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 17,
    last: 18,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 9, last: 12 };
pub const UP_AIR: MoveFrames = MoveFrames { first: 7, last: 12 };
