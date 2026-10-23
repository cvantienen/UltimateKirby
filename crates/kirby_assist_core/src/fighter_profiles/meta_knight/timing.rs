//! meta_knight: compiled move intervals from https://ultimateframedata.com/meta_knight
//! Reference movement (neutral state): Gravity: 0.110; Walk Speed: 1.239; Run Speed: 2.09;
//! Initial Dash: 2.211; Air Speed: 1.04; Total Air Acceleration: 0.075; Fall Speed / Fast
//! Fall Speed: 1.66 / 2.656; SH / FH / SHFF / FHFF Frames: 31 / 47 / 22 / 33
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 6, last: 20 },
    down_tilt: MoveFrames { first: 3, last: 4 },
    forward_tilt: MoveFrames { first: 6, last: 6 },
    down_smash: MoveFrames { first: 4, last: 4 },
    down_smash_back: Some(MoveFrames { first: 9, last: 9 }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 11.5,
        standing: 7,
        dash: 11,
        turn_or_shield: 12,
        shield: 11,
        active_frames: 2,
    },
    normal_hit_lead_frames: 8,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 8, last: 14 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 24,
    last: 24,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 9, last: 9 };
pub const UP_AIR: MoveFrames = MoveFrames { first: 6, last: 6 };
