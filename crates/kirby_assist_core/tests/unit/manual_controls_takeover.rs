use super::*;

#[test]
fn committed_auto_flick_rearms_against_held_manual_direction() {
    let right = 1 << 21;
    let held = ButtonTrace {
        held: right | crate::native_input::MAPPER_A,
        previous: right | crate::native_input::MAPPER_A,
        ..Default::default()
    };
    let mut overlay = ManualOverlay::default();
    let output = TurboOutput {
        stick: Some([1., 0.]),
        ..Default::default()
    };
    let frame = overlay.apply_reaction_controls(
        held,
        [1., 0.],
        output,
        0,
        0,
        crate::native_input::GAMEPLAY_BUTTONS,
        true,
    );
    assert_eq!(frame.buttons.held & crate::native_input::MAPPER_A, 0);
    assert_ne!(frame.buttons.pressed & right, 0);
}

#[test]
fn committed_reaction_rearms_held_manual_special_without_leaking_attack() {
    let held = ButtonTrace {
        held: crate::native_input::MAPPER_A | crate::native_input::MAPPER_B,
        previous: crate::native_input::MAPPER_A | crate::native_input::MAPPER_B,
        ..Default::default()
    };
    let mut overlay = ManualOverlay::default();
    let frame = overlay.apply_reaction_controls(
        held,
        [0., 0.],
        TurboOutput {
            stick: Some([0., -1.]),
        },
        0,
        crate::native_input::MAPPER_B,
        crate::ledge_trap::SEQUENCE_BUTTONS,
        true,
    );
    assert_eq!(frame.buttons.held & crate::native_input::MAPPER_A, 0);
    assert_eq!(
        frame.buttons.pressed & crate::native_input::MAPPER_B,
        crate::native_input::MAPPER_B
    );
    assert_eq!(frame.buttons.previous & crate::native_input::MAPPER_B, 0);
}
