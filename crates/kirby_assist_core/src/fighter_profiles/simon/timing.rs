//! simon: compiled move intervals from https://ultimateframedata.com/simon
//! Reference movement (neutral state): Gravity: 0.085; Walk Speed: 0.76; Run Speed: 1.52;
//! Initial Dash: 1.73; Air Speed: 0.94; Total Air Acceleration: 0.03; Fall Speed / Fast
//! Fall Speed: 1.85 / 2.96; SH / FH / SHFF / FHFF Frames: 41 / 51 / 29 / 36
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 8, last: 9 },
    down_tilt: MoveFrames { first: 7, last: 23 },
    forward_tilt: MoveFrames {
        first: 12,
        last: 13,
    },
    down_smash: MoveFrames {
        first: 14,
        last: 15,
    },
    down_smash_back: Some(MoveFrames {
        first: 20,
        last: 21,
    }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 12.0,
        standing: 10,
        dash: 13,
        turn_or_shield: 14,
        shield: 14,
        active_frames: 2,
    },
    normal_hit_lead_frames: 1,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames {
    first: 10,
    last: 22,
};
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 24,
    last: 25,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 14,
    last: 14,
};
pub const UP_AIR: MoveFrames = MoveFrames {
    first: 14,
    last: 16,
};
