//! peach: compiled move intervals from https://ultimateframedata.com/peach
//! Reference movement (neutral state): Gravity: 0.070; Walk Speed: 0.924; Run Speed: 1.595;
//! Initial Dash: 1.826; Air Speed: 1.029; Total Air Acceleration: 0.12; Fall Speed / Fast
//! Fall Speed: 1.19 / 1.904; SH / FH / SHFF / FHFF Frames: 41 / 57 / 28 / 40
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 5, last: 19 },
    down_tilt: MoveFrames { first: 8, last: 9 },
    forward_tilt: MoveFrames { first: 7, last: 7 },
    down_smash: MoveFrames { first: 6, last: 7 },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 12.0,
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
pub const UP_TILT: MoveFrames = MoveFrames { first: 9, last: 13 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 15,
    last: 17,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 16,
    last: 20,
};
pub const UP_AIR: MoveFrames = MoveFrames {
    first: 10,
    last: 13,
};
