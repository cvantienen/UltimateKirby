//! Compiled Kirby move/timing defaults. These numbers describe the game; they
//! do not change its hitboxes. Runtime numeric adjustments are in config.txt.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};

pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 8, last: 32 },
    down_tilt: MoveFrames { first: 4, last: 6 },
    forward_tilt: MoveFrames { first: 5, last: 8 },
    down_smash: MoveFrames { first: 7, last: 19 },
    down_smash_back: None,
    // Grounded Stone, not aerial Stone: https://ultimateframedata.com/kirby
    down_special_startup: Some(11),
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 12.5,
        standing: 6,
        dash: 9,
        turn_or_shield: 10,
        shield: 10,
        active_frames: 2,
    },
    normal_hit_lead_frames: 8,
    common_roll_distance: 25.0,
};

pub const NAIR_HIT_LEAD: u8 = 8;
pub const STONE_HIT_LEAD: u8 = 0;
// https://ultimateframedata.com/kirby.php : clean 6–8, late 9–12.
pub const JUMP_BAIR: MoveFrames = MoveFrames { first: 6, last: 12 };
pub const JUMP_BAIR_DRIFT: f32 = 0.35;
// Stage the seven-frame bair window three frames before protection expires.
pub const NEUTRAL_BAIR_LEAD: u8 = 3;
pub const NEUTRAL_BAIR_DRIFT: f32 = 0.0;
// Normal Inhale (without a copied ability): active from frame 10 and lingering.
pub const INHALE_STARTUP: u8 = 10;
pub const INHALE_HIT_LEAD: u8 = 8;

// Down-air starts on frame 18: https://ultimateframedata.com/kirby
pub const DROP_DAIR_FIRST_ACTIVE: f32 = 18.0;
// Keep down stronger than horizontal while selecting the aerial. Once dair is
// observed, release down/A and use full horizontal drift through startup.
pub const DROP_DAIR_SELECT_DRIFT: f32 = 0.6;
