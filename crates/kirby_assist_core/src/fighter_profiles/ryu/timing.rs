//! ryu: compiled move intervals from https://ultimateframedata.com/ryu
//! Reference movement (neutral state): Gravity: 0.120; Walk Speed: 0.75; Run Speed: 1.6;
//! Initial Dash: 1.76; Air Speed: 1.12; Total Air Acceleration: 0.035; Fall Speed / Fast
//! Fall Speed: 1.6 / 2.24; SH / FH / SHFF / FHFF Frames: 31 / 40 / 23 / 30
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
//! Light/heavy tilts and command inputs are excluded from these lists; use ordinary
//! aerials/smashes.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 4, last: 31 },
    down_tilt: MoveFrames { first: 2, last: 3 },
    forward_tilt: MoveFrames { first: 8, last: 11 },
    down_smash: MoveFrames { first: 5, last: 6 },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 11.6,
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
pub const UP_TILT: MoveFrames = MoveFrames { first: 3, last: 6 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 15,
    last: 17,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 8, last: 14 };
pub const UP_AIR: MoveFrames = MoveFrames { first: 6, last: 7 };
pub const DOWN_AIR: MoveFrames = MoveFrames { first: 8, last: 12 };
