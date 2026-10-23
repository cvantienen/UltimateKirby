//! rosalina_and_luma: compiled move intervals from https://ultimateframedata.com/rosalina_and_luma
//! Reference movement (neutral state): Gravity: 0.062; Walk Speed: 1.138; Run Speed: 1.795;
//! Initial Dash: 2.035; Air Speed: 1.05; Total Air Acceleration: 0.11; Fall Speed / Fast
//! Fall Speed: 1.2 / 1.92; SH / FH / SHFF / FHFF Frames: 50 / 71 / 36 / 49
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
//! Timings describe Rosalina, not a separately positioned Luma. No partner commands.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 9, last: 33 },
    down_tilt: MoveFrames { first: 5, last: 8 },
    forward_tilt: MoveFrames { first: 7, last: 9 },
    down_smash: MoveFrames { first: 6, last: 7 },
    down_smash_back: Some(MoveFrames {
        first: 17,
        last: 18,
    }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 14.0,
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
pub const UP_TILT: MoveFrames = MoveFrames { first: 7, last: 17 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 16,
    last: 18,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 10,
    last: 10,
};
pub const UP_AIR: MoveFrames = MoveFrames { first: 8, last: 19 };
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 17,
    last: 32,
};
