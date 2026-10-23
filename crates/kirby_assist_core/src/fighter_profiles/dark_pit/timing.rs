//! dark_pit: compiled move intervals from https://ultimateframedata.com/dark_pit
//! Reference movement (neutral state): Gravity: 0.081; Walk Speed: 1.259; Run Speed: 1.828;
//! Initial Dash: 2.09; Air Speed: 0.935; Total Air Acceleration: 0.075; Fall Speed / Fast
//! Fall Speed: 1.48 / 2.368; SH / FH / SHFF / FHFF Frames: 38 / 52 / 26 / 37
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 4, last: 5 },
    down_tilt: MoveFrames { first: 6, last: 7 },
    forward_tilt: MoveFrames {
        first: 10,
        last: 14,
    },
    down_smash: MoveFrames { first: 5, last: 6 },
    down_smash_back: Some(MoveFrames {
        first: 18,
        last: 20,
    }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 13.0,
        standing: 6,
        dash: 9,
        turn_or_shield: 10,
        shield: 10,
        active_frames: 2,
    },
    normal_hit_lead_frames: 1,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 6, last: 8 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 10,
    last: 10,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 11,
    last: 12,
};
pub const UP_AIR: MoveFrames = MoveFrames {
    first: 10,
    last: 12,
};
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 10,
    last: 12,
};
