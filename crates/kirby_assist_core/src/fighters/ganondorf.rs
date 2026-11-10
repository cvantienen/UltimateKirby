//! Read-only grounded Flame Choke release and contact timing.
use crate::native_states::status;

/// Defines the `CATCHED_GANON` constant used by this module.
///
/// Provides the CATCHED GANON constant value.
pub const CATCHED_GANON: i32 = status::CATCHED_GANON;
/// Defines the `CATCHED_AIR_END_GANON` constant used by this module.
///
/// Provides the CATCHED AIR END GANON constant value.
pub const CATCHED_AIR_END_GANON: i32 = status::CATCHED_AIR_END_GANON;
/// Sets the default value for ganon passive trigger frame.
///
/// Holds the DEFAULT GANON PASSIVE TRIGGER FRAME value.
pub const DEFAULT_GANON_PASSIVE_TRIGGER_FRAME: u16 = 7;

/// Classifies the current guard-lockout status.
///
/// Checks the current guard lockout.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub(crate) fn guard_lockout_status(status: i32) -> bool {
    matches!(status, CATCHED_GANON | CATCHED_AIR_END_GANON)
}

/// Returns the remaining passive-tech window.
///
/// Gets the time left to tech.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// - `normal` (`u16`): Represents the normal; the function uses it as input to the operation described above.
/// - `ganon` (`u16`): Represents the ganon; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `u16` value.
pub(crate) fn passive_window(status: i32, normal: u16, ganon: u16) -> u16 {
    if status == CATCHED_GANON {
        ganon
    } else {
        normal
    }
}

/// The release status is shared by grounded and aerial Flame Choke. Only the
/// grounded capture immediately preceding it identifies a techable release.
///
///  The release status is shared by grounded and aerial Flame Choke. Only the grounded capture immediately preceding it identifies a techable release.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// - `previous_status` (`i32`): Represents the previous status; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn grounded_ganon_release(status: i32, previous_status: i32) -> bool {
    status == CATCHED_AIR_END_GANON && previous_status == CATCHED_GANON
}

/// Grounded Flame Choke uses a capture status, not damage-fly or a falling ray.
/// Vanilla game_specials applies its throw at native motion frame 31:
/// <https://github.com/WuBoytH/SSBU-Dumped-Scripts/blob/main/smashline/lua2cpp_ganon/ganon/SpecialS.txt>
/// The caller must resolve the actual capture link, not a nearby Ganondorf.
///
///  Grounded Flame Choke uses a capture status, not damage-fly or a falling ray. Vanilla gamespecials applies its throw at native motion frame 31.
/// ### Arguments
/// - `victim_status` (`i32`): Represents the victim status; the function uses it as input to the operation described above.
/// - `captor_kind` (`i32`): Represents the captor kind; the function uses it as input to the operation described above.
/// - `captor_status` (`i32`): Represents the captor status; the function uses it as input to the operation described above.
/// - `frame` (`f32`): Represents the current frame or frame count; the function uses it as input to the operation described above.
/// - `rate` (`f32`): Represents the rate; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Some(value)` when a value is available; otherwise, returns `None`.
pub fn grounded_ganon_contact_in(
    victim_status: i32,
    captor_kind: i32,
    captor_status: i32,
    frame: f32,
    rate: f32,
) -> Option<f32> {
    if victim_status != CATCHED_GANON
        || captor_kind != crate::fighter_kinds::GANONDORF
        || captor_status != crate::native_states::fighter_status::GANON_SPECIAL_S_CATCH
        || !frame.is_finite()
        || !(0.0..=31.0).contains(&frame)
        || !rate.is_finite()
        || rate <= 0.0
    {
        return None;
    }
    let remaining = (31.0 - frame) / rate;
    (remaining.is_finite() && remaining <= 120.0).then_some(remaining)
}
