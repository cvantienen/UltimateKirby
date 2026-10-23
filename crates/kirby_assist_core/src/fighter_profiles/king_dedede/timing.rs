//! king_dedede: compiled move intervals from https://ultimateframedata.com/king_dedede
//! Reference movement (neutral state): Gravity: 0.097; Walk Speed: 1.029; Run Speed: 1.496;
//! Initial Dash: 1.815; Air Speed: 0.735; Total Air Acceleration: 0.05; Fall Speed / Fast
//! Fall Speed: 1.95 / 3.12; SH / FH / SHFF / FHFF Frames: 36 / 48 / 24 / 32
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 7, last: 29 },
    down_tilt: MoveFrames { first: 6, last: 11 },
    forward_tilt: MoveFrames {
        first: 12,
        last: 21,
    },
    down_smash: MoveFrames {
        first: 14,
        last: 22,
    },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 17.0,
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
pub const UP_TILT: MoveFrames = MoveFrames { first: 7, last: 13 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 40,
    last: 44,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 13,
    last: 15,
};
pub const UP_AIR: MoveFrames = MoveFrames {
    first: 10,
    last: 10,
};
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 22,
    last: 23,
};
