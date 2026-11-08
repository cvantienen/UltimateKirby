//! Local teardown only: no game writes, scene hooks, HID changes or I/O.
use super::*;

pub(crate) fn feature_activity_allowed() -> bool {
    RUNTIME_ENABLED.load(Ordering::Acquire) && LOCAL_PARTICIPANT.load(Ordering::Acquire)
}

/// Bind the selected mapper without creating a menu or starting an assistance
/// policy. A later native fighter callback supplies ownership; the next mapper
/// poll revalidates its copied ID before allowing any feature work.
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

pub(super) fn suspend_participation() {
    let was_active = LOCAL_PARTICIPANT.swap(false, Ordering::AcqRel);
    if let Ok(mut state) = STATE.lock() {
        if was_active || state.associated_object.is_some() || state.menu_battle_active {
            duck_hits::clear();
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

pub(super) fn set_runtime_enabled(enabled: bool) {
    if MASTER_IN_MATCH.load(Ordering::Acquire) {
        return;
    }
    RUNTIME_ENABLED.store(false, Ordering::Release);
    LOCAL_PARTICIPANT.store(false, Ordering::Release);
    duck_hits::clear();
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
