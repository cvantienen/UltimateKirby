//! mr_game_and_watch: compiled move intervals from https://ultimateframedata.com/mr_game_and_watch
//! Reference movement (neutral state): Gravity: 0.080; Walk Speed: 1.18; Run Speed: 1.679;
//! Initial Dash: 1.98; Air Speed: 1.176; Total Air Acceleration: 0.1; Fall Speed / Fast
//! Fall Speed: 1.24 / 1.984; SH / FH / SHFF / FHFF Frames: 36 / 51 / 25 / 36
//! Runtime hop/braking forecasts use native instance facts, not these reference
//! values. Stationary aerial recipes use zero added drift and no fast-fall.
//! Bomb fair and projectile up-air are not registered as timed body hitboxes. Bury hammers
//! are a selectable roll/drop alternative.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 7, last: 10 },
    down_tilt: MoveFrames { first: 6, last: 8 },
    forward_tilt: MoveFrames { first: 8, last: 20 },
    down_smash: MoveFrames {
        first: 12,
        last: 16,
    },
    down_smash_back: None,
    down_special_startup: None,
    up_special_startup: Some(3),
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 12.5,
        standing: 6,
        dash: 9,
        turn_or_shield: 10,
        shield: 10,
        active_frames: 2,
    },
    normal_hit_lead_frames: 3,
    // Coverage point for the opponent's roll, not this fighter's movement speed.
    common_roll_distance: 25.0,
};
pub const UP_TILT: MoveFrames = MoveFrames {
    first: 10,
    last: 12,
};
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 17,
    last: 18,
};
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 10,
    last: 13,
};
pub const UP_AIR: MoveFrames = MoveFrames { first: 9, last: 9 };
