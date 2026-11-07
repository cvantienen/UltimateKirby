//! Training Modpack handoff state shared by the mapper's passthrough paths.
use super::{settings_menu, STATE, TRAINING_MODAL_HANDOFF, TRAINING_PASSTHROUGH};
use kirby_assist_core::{
    native_input::{ButtonTrace, ControlLayout},
    training_test,
};
use std::{sync::atomic::Ordering, time::Instant};

pub(super) fn enter_passthrough(physical: ButtonTrace, index: i32, layout: ControlLayout) -> bool {
    if !training_test::passthrough(
        physical.held,
        physical.pressed,
        TRAINING_MODAL_HANDOFF.load(Ordering::Relaxed),
    ) {
        return false;
    }
    if training_test::sticky_shortcut(physical.held) {
        TRAINING_MODAL_HANDOFF.store(true, Ordering::Relaxed);
    }
    let entering_passthrough = !TRAINING_PASSTHROUGH.swap(true, Ordering::Relaxed);
    if let Ok(mut state) = STATE.lock() {
        if entering_passthrough {
            state.cancel();
        }
        state
            .settings_menu
            .get_or_insert_with(settings_menu::SettingsMenu::default)
            .observe_passthrough(training_test::policy_buttons(physical.held), layout);
        if let Some(menu) = state.settings_menu.as_ref() {
            menu.display_battle(
                state.menu_battle_active && !TRAINING_MODAL_HANDOFF.load(Ordering::Relaxed),
            );
        }
        // Keep the controller session alive during a long Training handoff.
        state.mapping_index = Some(index);
        state.raw_tick = state.raw_tick.wrapping_add(1);
        state.raw_buttons = training_test::policy_buttons(physical.held);
        state.raw_seen = Some(Instant::now());
        state.request_seen = state.raw_seen;
        state.raw_connected = true;
    }
    if entering_passthrough {
        log!("Training shortcut handoff: save/load resumes on release; menus/recording require Minus after closing");
    }
    true
}

pub(super) fn finish_passthrough(training_test_mode: bool) {
    if TRAINING_PASSTHROUGH.swap(false, Ordering::Relaxed) {
        let was_modal = TRAINING_MODAL_HANDOFF.swap(false, Ordering::Relaxed);
        if training_test_mode && !was_modal {
            if let Ok(mut state) = STATE.lock() {
                state
                    .settings_menu
                    .get_or_insert_with(Default::default)
                    .resume_saved_state();
            }
        }
        if training_test_mode && was_modal {
            if let Ok(mut state) = STATE.lock() {
                state
                    .settings_menu
                    .get_or_insert_with(settings_menu::SettingsMenu::default)
                    .request_open();
            }
        }
        log!("Training passthrough ended; selected input policy resumed");
    }
}
