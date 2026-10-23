//! bowser: compiled move intervals from https://ultimateframedata.com/bowser
//! Reference movement (neutral state): Gravity: 0.125; Walk Speed: 0.901; Run Speed: 1.971;
//! Initial Dash: 2.255; Air Speed: 1.155; Total Air Acceleration: 0.05; Fall Speed / Fast
//! Fall Speed: 1.77 / 2.832; SH / FH / SHFF / FHFF Frames: 31 / 45 / 22 / 31
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 8, last: 29 },
    down_tilt: MoveFrames {
        first: 10,
        last: 12,
    },
    forward_tilt: MoveFrames {
        first: 10,
        last: 14,
    },
    down_smash: MoveFrames {
        first: 12,
        last: 14,
    },
    down_smash_back: Some(MoveFrames {
        first: 28,
        last: 30,
    }),
    down_special_startup: None,
    up_special_startup: Some(6),
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 20.0,
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
pub const UP_TILT: MoveFrames = MoveFrames {
    first: 11,
    last: 16,
};
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 22,
    last: 27,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 11,
    last: 14,
};
pub const UP_AIR: MoveFrames = MoveFrames { first: 9, last: 13 };
