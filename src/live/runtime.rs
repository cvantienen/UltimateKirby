//! Local teardown only: no game writes, scene hooks, HID changes or I/O.
//! Technical: Owns local runtime gating, participant suspension, and state reset
//! without modifying native game state.
//! Plain English: This module safely starts or pauses Kirby Assist for the local player.
use super::*;
/// Check if feature activity is allowed based on the runtime and local participant status.
/// Requires both the master runtime flag and local participant flag to be set.
///
///  Assistance is allowed only when the mod and this player's session are active.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub(crate) fn feature_activity_allowed() -> bool {
    RUNTIME_ENABLED.load(Ordering::Acquire) && LOCAL_PARTICIPANT.load(Ordering::Acquire)
}

/// Bind the selected mapper without creating a menu or starting an assistance
/// policy. A later native fighter callback supplies ownership; the next mapper
/// poll revalidates its copied ID before allowing any feature work.
/// Selects a fresh callback-discovered route only when its mapper
/// index and associated fighter still match.
///
///  Finds the currently verified fighter for the selected controller.
/// ### Arguments
/// - `index` (`i32`): Represents the zero-based position of an item; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Some(value)` when a value is available; otherwise, returns `None`.
pub(super) fn mapper_candidate(index: i32) -> Option<u32> {
    if !RUNTIME_ENABLED.load(Ordering::Acquire) || !(0..8).contains(&index) {
        return None;
    }
    let mut state = STATE.lock().ok()?;
    if state.mapping_index != Some(index) {
        state.cancel();
        state.associated_object = None;
        state.mapping_index = Some(index);
    }
    state.human_routes[index as usize]
        .filter(|(id, seen)| {
            Some(*id) == state.associated_object && seen.elapsed() < SESSION_OBSERVATION_GAP
        })
        .map(|(id, _)| id)
}
/// Reset the given state, preserving the session and observation sequence while clearing all other fields.
/// Ends menu state, replaces all runtime policy fields with defaults,
/// and advances the observation sequence while retaining the session.
///
///  Clears active assistance safely without forgetting the current match.
/// ### Arguments
/// - `state` (`&mut State`): Represents the current state of the operation; the function uses it as input to the operation described above.
fn reset_state(state: &mut State) {
    let mut session = state.session;
    session.update(false);
    let sequence = state.observation_sequence.wrapping_add(1);
    if let Some(mut menu) = state.settings_menu.take() {
        menu.display_battle(false);
        menu.end_match();
    }
    // Reset every policy, latch, handoff, pending input and discovered route.
    // Preserve monotonic fences so an in-flight old observation cannot publish
    // into a later enabled session with a coincidentally reused object ID.
    *state = State::new();
    state.session = session;
    state.observation_sequence = sequence;
}
/// Suspend local participation in the runtime, clearing any active state and resetting relevant flags.
/// Atomically revokes local participation, clears projectile tracking,
/// and resets state only when local runtime state was active.
///
///  Pauses this player's assistance and forgets its active session data.
pub(super) fn suspend_participation() {
    let was_active = LOCAL_PARTICIPANT.swap(false, Ordering::AcqRel);
    if let Ok(mut state) = STATE.lock() {
        if was_active || state.associated_object.is_some() || state.menu_battle_active {
            attack_sources::clear();
            reset_state(&mut state);
            TRAINING_PASSTHROUGH.store(false, Ordering::Relaxed);
            TRAINING_MODAL_HANDOFF.store(false, Ordering::Relaxed);
            #[cfg(feature = "diagnostic-logging")]
            {
                crate::performance::set_enabled(false);
                crate::performance::clear();
                crate::calibration::clear();
                crate::logging::clear_snapshot();
            }
        }
    }
}
/// Enable or disable the runtime, resetting state and clearing relevant flags as necessary.
/// Refuses toggles during a match, disables activity during teardown,
/// clears shared state, then publishes the requested gate value.
///
///  Turns assistance off or on outside battle after safely clearing old state.
/// ### Arguments
/// - `enabled` (`bool`): Represents whether the feature or option should be enabled; the function uses it as input to the operation described above.
pub(super) fn set_runtime_enabled(enabled: bool) {
    if MASTER_IN_MATCH.load(Ordering::Acquire) {
        return;
    }
    RUNTIME_ENABLED.store(false, Ordering::Release);
    LOCAL_PARTICIPANT.store(false, Ordering::Release);
    attack_sources::clear();
    if let Ok(mut state) = STATE.lock() {
        reset_state(&mut state);
    } else {
        settings_menu::show_master_status(false);
        return;
    }
    TRAINING_PASSTHROUGH.store(false, Ordering::Relaxed);
    TRAINING_MODAL_HANDOFF.store(false, Ordering::Relaxed);
    #[cfg(feature = "diagnostic-logging")]
    {
        crate::performance::set_enabled(false);
        crate::performance::clear();
        crate::calibration::clear();
        crate::logging::clear_snapshot();
    }
    RUNTIME_ENABLED.store(enabled, Ordering::Release);
    settings_menu::show_master_status(enabled);
}
