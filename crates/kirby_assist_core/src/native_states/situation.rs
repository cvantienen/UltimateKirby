//! `SITUATION_KIND_*` values from the pinned `skyline-smash` lua_const table.
pub const GROUND: i32 = 0;
pub const CLIFF: i32 = 1;
pub const AIR: i32 = 2;
/// Adapter sentinel when no native situation observation is available.
pub const UNAVAILABLE: i32 = -1;

// AttackData.situation is a bitmask, not a SITUATION_KIND value.
pub const ATTACK_GROUND_MASK: u16 = 1;
pub const ATTACK_AIR_MASK: u16 = 2;
