//! Verified move windows: https://ultimateframedata.com/yoshi
//! Only continuous active intervals are used, never gaps between multihits.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};
// 3 squat + 11 bair startup + turn/setup allowance. Landing Fsmash requires
// an early hit; this budget does not claim the follow-up is a true combo.
pub const WHIFF_BAIR_STARTUP_BUDGET: u8 = 3 + BACK_AIR_FIRST.first + 4;
pub const WHIFF_BAIR_REACH: f32 = 12.0;
pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 3, last: 25 },
    down_tilt: MoveFrames { first: 8, last: 10 },
    forward_tilt: MoveFrames { first: 5, last: 7 },
    down_smash: MoveFrames { first: 7, last: 8 },
    down_smash_back: Some(MoveFrames {
        first: 22,
        last: 23,
    }),
    down_special_startup: None,
    up_special_startup: None,
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 26.0,
        standing: 14,
        dash: 16,
        turn_or_shield: 17,
        shield: 18,
        active_frames: 8,
    },
};

pub const UP_TILT: MoveFrames = MoveFrames { first: 8, last: 15 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 14,
    last: 16,
};
pub const UP_AIR: MoveFrames = MoveFrames { first: 5, last: 6 };
pub const FORWARD_AIR: MoveFrames = MoveFrames {
    first: 16,
    last: 20,
};
// Bair is three discrete hits. Never model the gaps as active coverage.
pub const BACK_AIR_FIRST: MoveFrames = MoveFrames {
    first: 11,
    last: 11,
};
pub const BACK_AIR_MIDDLE: MoveFrames = MoveFrames {
    first: 14,
    last: 14,
};
pub const BACK_AIR_FINISH: MoveFrames = MoveFrames {
    first: 18,
    last: 19,
};
// Zero aims the middle hit at protection expiry; increase to stage it earlier.
// Dair has discrete hits, not continuous coverage from 16 through 41. It is
// registered only as an immediate drop chase, never a timed vulnerability window.
// The final frame is used solely to stop outward drift after the last hit.
pub const DOWN_AIR_FIRST: u8 = 16;
pub const DOWN_AIR_LAST: u8 = 41;

// Response choices, separate from native hitbox startup/active-frame data.
// Start earlier so protection expiry targets nair frame 19 on the descending
// portion instead of frame 11 while rising over small fighters. Native activity
// stays 3..=25; actual landing/height still needs on-device calibration.
// No extra post-expiry wait: fast fall as soon as the confirmed nair descends.
// The sequence still requires exact native nair and negative vertical speed.
pub const NEUTRAL_NAIR_FAST_FALL_FRAME: u8 = 3;
// Horizontal stick after native neutral-getup nair is confirmed.
// Brief full-stick nudge during the first two confirmed nair animation frames.
// Neutral dair retains its separate longer drift window.
// Walk only in the existing Neutral nair wait; never delay its launch for arrival.
// Stand slightly on the ledge side of the shared roll-coverage estimate.
// This is a spacing adjustment to calibrate on-device, not a hitbox edit.
pub const ROLL_DOWN_SMASH_STANDOFF: f32 = 2.0;

// Use the first continuous sweep only; frames 9..21 are NOT active.
// Root-space reach is a conservative calibration estimate, not a live hitbox.
pub const ROLL_READ_DOWN_SMASH: crate::roll_read::FighterConfig = crate::roll_read::FighterConfig {
    back: Some((22, 23)),
    first: 7,
    last: 8,
    reach: 13.0,
    height: 6.0,
    watch_distance: 64.0,
};
pub const ROLL_READ_GRAB: crate::roll_read::FighterConfig = crate::roll_read::FighterConfig {
    back: None,
    first: LEDGE_DATA.grab.standing,
    last: LEDGE_DATA.grab.standing + LEDGE_DATA.grab.active_frames - 1,
    reach: LEDGE_DATA.grab.standing_range,
    height: 4.0,
    watch_distance: 36.0,
};
