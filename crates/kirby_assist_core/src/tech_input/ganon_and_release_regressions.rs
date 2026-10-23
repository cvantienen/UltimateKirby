//! Existing regression cases for `tech_input`. Kept outside the runtime code.
use super::*;
use crate::tech_prediction::grounded_ganon_contact_in;

fn capture(frame: f32, delay: f32) -> TechContext {
    TechContext {
        object: 1,
        status: CATCHED_GANON,
        // This motion clock comes from the linked captor; the victim can
        // remain in one frozen capture pose throughout the sequence.
        motion: 7,
        frame,
        rate: 1.0,
        stopped: false,
        remaining_stop: 0,
        contact_in: grounded_ganon_contact_in(CATCHED_GANON, 0x18, 0x1e2, frame, 1.0),
        input_delay: delay,
        previous_guard: false,
    }
}

#[test]
fn grounded_ganon_waits_for_tight_arrival_window_and_compensates_online_delay() {
    for delay in [0.0, 3.0, 6.0] {
        let mut input = TechInput::default();
        let due_frame = 31.0 - delay - 4.0;
        assert!(
            !input
                .sample(true, true, 40, capture(due_frame - 1.0, delay))
                .press
        );
        assert_eq!(input.cooldown_remaining(), 0.0);
        for _ in 0..20 {
            assert!(
                input
                    .sample(true, true, 40, capture(due_frame, delay))
                    .press
            );
        }
        input.mapped_press();
        assert_eq!(input.cooldown_remaining(), 40.0);
        let release = input.sample(true, true, 40, capture(due_frame, delay));
        assert!(!release.press);
        assert!(release.owns_guard);
    }
}

#[test]
fn cooldown_keeps_held_shield_aliases_released_until_the_collision_ends() {
    use crate::manual_controls::{ManualOverlay, TurboOutput};
    use crate::native_input::{ButtonTrace, MAPPER_L, MAPPER_R, MAPPER_ZL, MAPPER_ZR};
    let shoulders = MAPPER_L | MAPPER_R | MAPPER_ZL | MAPPER_ZR;
    let human = ButtonTrace {
        held: MAPPER_ZL | MAPPER_ZR,
        previous: MAPPER_ZL | MAPPER_ZR,
        ..ButtonTrace::default()
    };
    let mut input = TechInput::default();
    let mut overlay = ManualOverlay::default();
    let context = capture(27.0, 0.0);
    input.sample(
        true,
        true,
        40,
        TechContext {
            previous_guard: true,
            ..context
        },
    );
    overlay.apply_controls(human, [0.0; 2], TurboOutput::default(), 0, 0, shoulders);
    assert!(input.sample(true, true, 40, context).press);
    overlay.apply_controls(
        human,
        [0.0; 2],
        TurboOutput::default(),
        0,
        MAPPER_ZR,
        shoulders,
    );
    input.mapped_press();
    for frame in 28..=31 {
        let out = input.sample(true, true, 40, capture(frame as f32, 0.0));
        assert!(out.owns_guard && !out.press);
        let trace =
            overlay.apply_controls(human, [0.0; 2], TurboOutput::default(), 0, 0, shoulders);
        assert_eq!(trace.buttons.held & shoulders, 0);
        assert_eq!(
            trace.buttons.pressed & shoulders,
            0,
            "held physical shields must not fabricate another GUARD edge"
        );
    }
    let ended = input.sample(
        true,
        true,
        40,
        TechContext {
            status: 0x67,
            contact_in: None,
            ..context
        },
    );
    assert_eq!(ended, TechOutput::default());
    assert_eq!(input.cooldown_remaining(), 0.0);
}

#[test]
fn ordinary_tumble_retains_its_existing_arrival_window_and_invalid_capture_never_presses() {
    let mut input = TechInput::default();
    let ordinary = TechContext {
        status: 0x49,
        contact_in: Some(8.0),
        ..capture(20.0, 0.0)
    };
    assert!(input.sample(true, true, 40, ordinary).press);
    let mut input = TechInput::default();
    assert!(
        !input
            .sample(
                true,
                true,
                40,
                TechContext {
                    contact_in: None,
                    ..capture(27.0, 0.0)
                }
            )
            .press
    );
    assert!(!input.sample(false, true, 40, capture(27.0, 0.0)).press);
}
