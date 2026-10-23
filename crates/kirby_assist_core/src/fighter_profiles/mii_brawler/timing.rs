//! mii_brawler: compiled move intervals from https://ultimateframedata.com/mii_brawler
//! Reference movement (neutral state): Gravity: 0.192; Walk Speed: 1.26; Run Speed: 1.92;
//! Initial Dash: 1.87; Air Speed: 1.15; Total Air Acceleration: 0.068; Fall Speed / Fast
//! Fall Speed: 1.92 / 3.072; SH / FH / SHFF / FHFF Frames: 28 / 41 / 20 / 30
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
//! Only shared normals/aerials/grab; no assumptions about selected custom specials.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 3, last: 29 },
    down_tilt: MoveFrames { first: 7, last: 8 },
    forward_tilt: MoveFrames { first: 6, last: 8 },
    down_smash: MoveFrames { first: 9, last: 10 },
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
pub const UP_TILT: MoveFrames = MoveFrames { first: 5, last: 12 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 17,
    last: 18,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 8, last: 9 };
pub const UP_AIR: MoveFrames = MoveFrames { first: 6, last: 10 };
pub const DOWN_AIR: MoveFrames = MoveFrames {
    first: 16,
    last: 18,
};
