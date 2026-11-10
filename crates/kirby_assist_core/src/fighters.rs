//! Fighter reference facts, profiles, native state tables and fighter-specific policies.

pub mod fighter_data;
pub mod fighter_kinds;
pub mod fighter_profiles;
pub mod fighter_tuning;
pub mod game_and_watch;
pub mod ganondorf;
pub mod kirby;
pub mod native_states;

/// Native facing is ±1; anything else is an unreadable observation.
///
///  Native facing is ±1; anything else is an unreadable observation.
pub(crate) fn valid_facing(facing: f32) -> bool {
    facing.is_finite() && facing.abs() >= 0.5
}
