//! chrom: compiled move intervals from https://ultimateframedata.com/chrom
//! Reference movement (neutral state): Gravity: 0.114; Walk Speed: 1.208; Run Speed: 2.145;
//! Initial Dash: 2.2; Air Speed: 1.302; Total Air Acceleration: 0.05; Fall Speed / Fast
//! Fall Speed: 1.8 / 2.88; SH / FH / SHFF / FHFF Frames: 30 / 44 / 20 / 31
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 6, last: 7 },
    down_tilt: MoveFrames { first: 7, last: 8 },
    forward_tilt: MoveFrames { first: 8, last: 10 },
    down_smash: MoveFrames { first: 6, last: 7 },
    down_smash_back: Some(MoveFrames {
        first: 21,
        last: 22,
    }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 14.2,
        standing: 7,
        dash: 10,
        turn_or_shield: 11,
        shield: 11,
        active_frames: 2,
    },
    normal_hit_lead_frames: 1,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 6, last: 11 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 13,
    last: 14,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 10,
    last: 12,
};
pub const UP_AIR: MoveFrames = MoveFrames { first: 5, last: 12 };
