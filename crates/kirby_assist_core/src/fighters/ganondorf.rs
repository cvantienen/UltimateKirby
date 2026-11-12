//! Read-only grounded Flame Choke release and contact timing.
use crate::native_states::status;

/// Victim status while held by grounded Flame Choke.
pub const CATCHED_GANON: i32 = status::CATCHED_GANON;
/// Shared victim release status at the end of Flame Choke.
pub const CATCHED_AIR_END_GANON: i32 = status::CATCHED_AIR_END_GANON;
/// Default frame for scheduling a tech after grounded Flame Choke.
pub const DEFAULT_GANON_PASSIVE_TRIGGER_FRAME: u16 = 7;

/// Classifies the current guard-lockout status.
/// Checks the current guard lockout.
pub(crate) fn guard_lockout_status(status: i32) -> bool {
    matches!(status, CATCHED_GANON | CATCHED_AIR_END_GANON)
}

/// Selects the remaining tech window for grounded Flame Choke or normal tumble.
pub(crate) fn passive_window(status: i32, normal: u16, ganon: u16) -> u16 {
    if status == CATCHED_GANON {
        ganon
    } else {
        normal
    }
}

/// The release status is shared by grounded and aerial Flame Choke. Only the
/// grounded capture immediately preceding it identifies a techable release.
pub fn grounded_ganon_release(status: i32, previous_status: i32) -> bool {
    status == CATCHED_AIR_END_GANON && previous_status == CATCHED_GANON
}

/// Grounded Flame Choke uses a capture status, not damage-fly or a falling ray.
/// Vanilla game_specials applies its throw at native motion frame 31:
/// <https://github.com/WuBoytH/SSBU-Dumped-Scripts/blob/main/smashline/lua2cpp_ganon/ganon/SpecialS.txt>
/// The caller must resolve the actual capture link, not a nearby Ganondorf.
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
