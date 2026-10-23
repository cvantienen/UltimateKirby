//! bayonetta: compiled move intervals from https://ultimateframedata.com/bayonetta
//! Reference movement (neutral state): Gravity: 0.120; Walk Speed: 0.945; Run Speed: 1.76;
//! Initial Dash: 1.936; Air Speed: 1.019; Total Air Acceleration: 0.095; Fall Speed / Fast
//! Fall Speed: 1.77 / 2.832; SH / FH / SHFF / FHFF Frames: 38 / 50 / 27 / 35
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
//! Uses the foot interval 22-25 for down-smash, excluding the earlier ground-only stomp.
//! Optional bullet extensions are excluded.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 9, last: 25 },
    down_tilt: MoveFrames { first: 7, last: 8 },
    forward_tilt: MoveFrames {
        first: 12,
        last: 12,
    },
    down_smash: MoveFrames {
        first: 22,
        last: 25,
    },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 12.5,
        standing: 7,
        dash: 10,
        turn_or_shield: 11,
        shield: 11,
        active_frames: 2,
    },
    normal_hit_lead_frames: 8,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames { first: 7, last: 15 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 17,
    last: 21,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames { first: 7, last: 9 };
pub const UP_AIR: MoveFrames = MoveFrames { first: 9, last: 18 };
