//! Verified move windows: https://ultimateframedata.com/samus
//! Only continuous active intervals are used, never gaps between multihits.
use super::super::{GrabTiming, LedgeFighterData, MoveFrames};
pub const LEDGE_DATA: LedgeFighterData = LedgeFighterData {
    neutral_air: MoveFrames { first: 8, last: 11 },
    down_tilt: MoveFrames { first: 6, last: 8 },
    forward_tilt: MoveFrames { first: 8, last: 10 },
    down_smash: MoveFrames { first: 9, last: 10 },
    down_smash_back: Some(MoveFrames {
        first: 17,
        last: 18,
    }),
    down_special_startup: None,
    up_special_startup: Some(4),
    grab: GrabTiming {
        // Standing grab maximum reach reference: UFD stats / Grab Range.
        standing_range: 42.2,
        standing: 15,
        dash: 17,
        turn_or_shield: 18,
        shield: 19,
        active_frames: 8,
    },
    normal_hit_lead_frames: 3,
    common_roll_distance: 25.0,
};

// Only the first continuous hit is scheduled; later multihits have gaps.
pub const FORWARD_AIR_FIRST: MoveFrames = MoveFrames { first: 6, last: 7 };
pub const UP_AIR_FIRST: MoveFrames = MoveFrames { first: 5, last: 5 };
pub const FORWARD_SMASH: MoveFrames = MoveFrames {
    first: 10,
    last: 11,
};
// Projectile generation/collision startup is not its arrival time at the ledge.
pub const HOMING_MISSILE_STARTUP: u8 = 18;
// Clear side-special selection after mapping without a full smash-stick flick.
pub const HOMING_MISSILE_STICK: f32 = 0.70;
pub const SUPER_MISSILE_STARTUP: u8 = 21;

// Zair's two consecutive phases have no inactive gap: 8-15, then 16-19.
pub const ZAIR_ACTIVE: MoveFrames = MoveFrames { first: 8, last: 19 };
pub const ZAIR_HIT_LEAD: u8 = 5;
// Jump squat plus a separate airborne Grab sample (never grounded X+Grab).
pub const ZAIR_TAKEOFF_FRAMES: u8 = 4;
// Coverage preset in world units, not an asserted universal hitbox distance.
pub const ZAIR_ROLL_STANDOFF: f32 = 12.0;
pub const BOMB_GENERATED_FRAME: u8 = 11;
pub const BOMB_FIRST_COLLISION_FRAME: u8 = 33;
