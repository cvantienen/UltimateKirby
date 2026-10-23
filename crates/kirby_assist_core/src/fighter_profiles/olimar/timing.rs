//! olimar: compiled move intervals from https://ultimateframedata.com/olimar
//! Reference movement (neutral state): Gravity: 0.068; Walk Speed: 0.945; Run Speed: 1.617;
//! Initial Dash: 1.606; Air Speed: 0.861; Total Air Acceleration: 0.08; Fall Speed / Fast
//! Fall Speed: 1.35 / 2.16; SH / FH / SHFF / FHFF Frames: 43 / 59 / 30 / 41
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
//! Nair/down-tilt are body moves. Pikmin-labelled choices require available Pikmin. No
//! plucking/ordering or resource writes.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 7, last: 8 },
    down_tilt: MoveFrames { first: 6, last: 12 },
    forward_tilt: MoveFrames {
        first: 15,
        last: 17,
    },
    down_smash: MoveFrames {
        first: 10,
        last: 13,
    },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 19.4,
        standing: 12,
        dash: 12,
        turn_or_shield: 12,
        shield: 16,
        active_frames: 12,
    },
    normal_hit_lead_frames: 1,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 6, last: 6 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 11,
    last: 13,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 7, last: 9 };
pub const UP_AIR: MoveFrames = MoveFrames { first: 8, last: 15 };
pub const DOWN_AIR: MoveFrames = MoveFrames { first: 9, last: 16 };
