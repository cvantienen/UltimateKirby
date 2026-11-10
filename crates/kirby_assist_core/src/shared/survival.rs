//! Shared survival responses, independent of fighter-specific follow-ups.

pub mod auto_di;
pub mod combat_assist;
pub mod tech_input;
pub mod tech_prediction;

/// Last hitstop frames, counted down to the native release, in which launch DI
/// owns the stick; earlier frames belong to SDI.
///
///  Last hitstop frames, counted down to the native release, in which launch DI owns the stick; earlier frames belong to SDI.
pub(crate) const LAUNCH_DI_FRAMES: u64 = 3;
/// Longest damage hitstop either DI response will track.
///
///  Longest damage hitstop either DI response will track.
pub(crate) const MAX_DAMAGE_STOP_DI_FRAMES: u64 = 120;
/// Physical stick magnitude at which the player's own DI input takes priority.
///
///  Physical stick magnitude at which the player's own DI input takes priority.
const PHYSICAL_STICK_DEFLECTION: f32 = 0.2;

/// A new hit begins when the victim changes or its hitstop counter restarts upward.
///
///  A new hit begins when the victim changes or its hitstop counter restarts upward.
pub(crate) fn is_new_hit(last: Option<(u32, u64)>, object: u32, remaining: u64) -> bool {
    last.is_none_or(|(id, left)| id != object || remaining > left)
}

/// Determines whether the `stick_engaged` condition holds.
///
/// Checks whether stick engaged.
pub(crate) fn stick_engaged(stick: [f32; 2]) -> bool {
    stick[0].hypot(stick[1]) >= PHYSICAL_STICK_DEFLECTION
}
