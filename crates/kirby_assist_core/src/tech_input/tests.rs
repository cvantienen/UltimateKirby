//! Existing regression cases for `tech_input`. Kept outside the runtime code.
use super::*;
fn ctx(frame: f32) -> TechContext {
    TechContext {
        object: 1,
        status: 0x49,
        motion: 1,
        frame,
        rate: 1.0,
        stopped: false,
        remaining_stop: 0,
        contact_in: Some(3.0),
        input_delay: 0.0,
        previous_guard: false,
    }
}
#[test]
fn unmapped_requests_do_not_spend_cooldown_or_disappear_on_duplicate_polls() {
    let mut input = TechInput::default();
    for _ in 0..30 {
        assert!(input.sample(true, true, 40, ctx(1.0)).press);
        assert_eq!(input.cooldown_remaining(), 0.0);
    }
    assert!(input.sample(true, true, 40, ctx(2.0)).press);
    input.mapped_press();
    assert_eq!(input.cooldown_remaining(), 40.0);
    assert!(!input.sample(true, true, 40, ctx(2.0)).press);
    assert_eq!(input.cooldown_remaining(), 40.0);
}
#[test]
fn held_guard_gets_release_then_press_and_hitstop_does_not_consume_lockout() {
    let mut input = TechInput::default();
    let release = input.sample(
        true,
        true,
        40,
        TechContext {
            previous_guard: true,
            ..ctx(1.0)
        },
    );
    assert!(!release.press && release.owns_guard);
    assert!(input.sample(true, true, 40, ctx(1.0)).press);
    input.mapped_press();
    for remaining in (1..=10).rev() {
        input.sample(
            true,
            true,
            40,
            TechContext {
                stopped: true,
                remaining_stop: remaining,
                ..ctx(1.0)
            },
        );
    }
    assert_eq!(input.cooldown_remaining(), 40.0);
}
#[test]
fn success_clears_synthetic_cooldown_and_changed_prediction_cancels_unmapped_press() {
    let mut input = TechInput::default();
    input.sample(true, true, 40, ctx(1.0));
    input.mapped_press();
    assert!(
        !input
            .sample(
                true,
                true,
                40,
                TechContext {
                    status: 0x69,
                    motion: 2,
                    frame: 1.0,
                    ..ctx(1.0)
                }
            )
            .press
    );
    assert!(input.sample(true, true, 40, ctx(2.0)).press);
    assert!(
        !input
            .sample(
                true,
                true,
                40,
                TechContext {
                    contact_in: None,
                    ..ctx(2.0)
                }
            )
            .press
    );
    assert_eq!(input.cooldown_remaining(), 0.0);
}
#[test]
fn delay_schedules_earlier_but_unknown_contact_and_disable_never_press() {
    let mut input = TechInput::default();
    let early = TechContext {
        contact_in: Some(14.0),
        ..ctx(1.0)
    };
    assert!(!input.sample(true, true, 40, early).press);
    assert!(
        input
            .sample(
                true,
                true,
                40,
                TechContext {
                    input_delay: 6.0,
                    ..early
                }
            )
            .press
    );
    assert!(!input.sample(false, true, 40, early).press);
    assert!(
        !input
            .sample(
                true,
                true,
                40,
                TechContext {
                    contact_in: None,
                    ..early
                }
            )
            .press
    );
}
#[test]
fn cooldown_uses_rate_normalized_motion_progress_not_poll_count() {
    let mut input = TechInput::default();
    let first = TechContext {
        rate: 0.5,
        ..ctx(1.0)
    };
    input.sample(true, true, 40, first);
    input.mapped_press();
    for _ in 0..20 {
        input.sample(true, true, 40, first);
    }
    input.sample(
        true,
        true,
        40,
        TechContext {
            frame: 3.0,
            ..first
        },
    );
    assert_eq!(input.cooldown_remaining(), 36.0);
}
#[test]
fn native_hitstop_allows_a_buffered_request_and_additive_mode_preserves_held_guard() {
    let mut input = TechInput::default();
    let stop = TechContext {
        stopped: true,
        motion: 0,
        rate: 0.0,
        remaining_stop: 2,
        ..ctx(1.0)
    };
    assert!(input.sample(true, true, 40, stop).press);
    input.mapped_press();
    assert_eq!(input.cooldown_remaining(), 40.0);
    let mut additive = TechInput::default();
    assert_eq!(
        additive.sample(
            true,
            false,
            40,
            TechContext {
                previous_guard: true,
                ..ctx(1.0)
            }
        ),
        TechOutput::default()
    );
}
#[test]
fn shared_overlay_releases_both_triggers_before_a_fresh_tech_edge() {
    use crate::manual_controls::{ManualOverlay, TurboOutput};
    use crate::native_input::{ButtonTrace, MAPPER_L, MAPPER_R, MAPPER_ZL, MAPPER_ZR};
    let shoulders = MAPPER_ZL | MAPPER_ZR | MAPPER_L | MAPPER_R;
    let human = ButtonTrace {
        held: MAPPER_ZL | MAPPER_ZR,
        previous: MAPPER_ZL | MAPPER_ZR,
        ..ButtonTrace::default()
    };
    let mut overlay = ManualOverlay::default();
    let mut input = TechInput::default();
    let release = input.sample(
        true,
        true,
        40,
        TechContext {
            previous_guard: true,
            ..ctx(1.0)
        },
    );
    assert!(release.owns_guard && !release.press);
    let mapped = overlay.apply_controls(human, [0.0; 2], TurboOutput::default(), 0, 0, shoulders);
    assert_eq!(mapped.buttons.held & shoulders, 0);
    let press = input.sample(true, true, 40, ctx(1.0));
    assert!(press.press);
    let mapped = overlay.apply_controls(
        human,
        [0.0; 2],
        TurboOutput::default(),
        0,
        MAPPER_ZR,
        shoulders,
    );
    assert_eq!(mapped.buttons.held & shoulders, MAPPER_ZR);
    assert_eq!(mapped.buttons.pressed & MAPPER_ZR, MAPPER_ZR);
}
#[test]
fn unavailable_animation_data_and_human_guard_do_not_erase_lockout_tracking() {
    let mut input = TechInput::default();
    input.sample(true, true, 40, ctx(1.0));
    input.mapped_press();
    assert!(
        !input
            .sample(
                true,
                true,
                40,
                TechContext {
                    motion: 0,
                    rate: 0.0,
                    ..ctx(1.0)
                }
            )
            .press
    );
    assert_eq!(input.cooldown_remaining(), 40.0);
    input.sample(true, true, 40, ctx(2.0));
    input.mapped_human_guard();
    assert_eq!(input.cooldown_remaining(), 40.0);
    assert!(!input.sample(true, true, 40, ctx(3.0)).press);
}
