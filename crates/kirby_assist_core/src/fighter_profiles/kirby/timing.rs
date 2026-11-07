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
};

// Start Neutral get-up down smash this many frames earlier than midpoint alignment.
// Whiff-punish root-distance estimates, not native hitbox radii.
pub const WHIFF_TILT_REACH: f32 = 8.0;
pub const WHIFF_SMASH_REACH: f32 = 12.0;
pub const WHIFF_SMASH_STARTUP: u8 = 13;
// Separate onstage roll-through mode. Reach/height are conservative root-space
// estimates for calibration, not measured hurtbox intersection guarantees.
pub const ROLL_READ_DOWN_SMASH: crate::roll_read::FighterConfig = crate::roll_read::FighterConfig {
    back: None,
    first: 7,
    last: 19,
    reach: 14.0,
    height: 6.0,
    watch_distance: 64.0,
};
// Standing grab is active 6–7. Reach is an estimated root-space gate, not a
// native grabbox measurement; unlike ledge roll grabs, this never chases.
pub const ROLL_READ_GRAB: crate::roll_read::FighterConfig = crate::roll_read::FighterConfig {
    back: None,
    first: LEDGE_DATA.grab.standing,
    last: LEDGE_DATA.grab.standing + LEDGE_DATA.grab.active_frames - 1,
    reach: LEDGE_DATA.grab.standing_range,
    height: 4.0,
    watch_distance: 32.0,
};
// Optional grounded preparation while Neutral/Jump nair are already waiting.
// Reuses the fast-walk strength from Kirby's roll grab, below the dash flick.
// https://ultimateframedata.com/kirby.php : clean 6–8, late 9–12.
pub const JUMP_BAIR: MoveFrames = MoveFrames { first: 6, last: 12 };
// Stage the seven-frame bair window three frames before protection expires.
// Normal Inhale (without a copied ability): active from frame 10 and lingering.
pub const INHALE_STARTUP: u8 = 10;

// Down-air starts on frame 18: https://ultimateframedata.com/kirby
// Keep down stronger than horizontal while selecting the aerial. Once dair is
// observed, release down/A and use full horizontal drift through startup.
