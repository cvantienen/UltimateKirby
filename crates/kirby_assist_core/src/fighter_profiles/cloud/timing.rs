//! cloud: compiled move intervals from https://ultimateframedata.com/cloud
//! Reference movement (neutral state): Gravity: 0.098 (Limit — 0.1078); Walk Speed: 1.155
//! (Limit — 1.32825); Run Speed: 2.167 (Limit — 2.3837); Initial Dash: 2.145 (Limit —
//! 2.3595); Air Speed: 1.155 (Limit — 1.386); Total Air Acceleration: 0.07 (Limit — 0.084);
//! Fall Speed / Fast Fall Speed: 1.68 (Limit — 1.848) / 2.688 (Limit — 2.9568); SH / FH /
//! SHFF / FHFF Frames: 37 / 49 / 26 / 34
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 5, last: 15 },
    down_tilt: MoveFrames { first: 7, last: 17 },
    forward_tilt: MoveFrames { first: 9, last: 10 },
    down_smash: MoveFrames { first: 8, last: 9 },
    down_smash_back: Some(MoveFrames {
        first: 21,
        last: 23,
    }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 12.0,
        standing: 9,
        dash: 12,
        turn_or_shield: 13,
        shield: 13,
        active_frames: 2,
    },
    normal_hit_lead_frames: 8,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 6, last: 9 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 19,
    last: 19,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 18,
    last: 25,
};
pub const UP_AIR: MoveFrames = MoveFrames { first: 8, last: 23 };
