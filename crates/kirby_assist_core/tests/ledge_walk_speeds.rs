use kirby_assist_core::manual_controls::ledge_walk_stick;

fn near(a: f32, b: f32) {
    assert!((a - b).abs() < 0.000001, "{a} != {b}");
}
#[test]
fn default_baseline_has_slightly_slower_and_faster_caps_on_both_sides() {
    for direction in [-1.0, 1.0] {
        near(
            ledge_walk_stick([direction * 0.9, 0.], 0.35)[0],
            direction * 0.33,
        );
        near(ledge_walk_stick([direction, 0.], 0.35)[0], direction * 0.5);
        near(
            ledge_walk_stick([direction * 0.12, 0.], 0.35)[0],
            direction * 0.12,
        );
    }
}
#[test]
fn boost_needs_nearly_full_horizontal_and_preserves_vertical_control() {
    near(ledge_walk_stick([0.949, 0.], 0.35)[0], 0.33);
    near(ledge_walk_stick([0.95, 0.], 0.35)[0], 0.5);
    near(ledge_walk_stick([1., 0.15], 0.35)[0], 0.5);
    let diagonal = ledge_walk_stick([-1., -1.], 0.35);
    near(diagonal[0], -0.33);
    assert_eq!(diagonal[1], -1.);
    near(ledge_walk_stick([1., 0.151], 0.35)[0], 0.33);
}
#[test]
fn slow_config_baseline_and_fixed_fast_cap_remain_supported() {
    near(ledge_walk_stick([0.9, 0.], 0.4)[0], 0.38);
    near(ledge_walk_stick([1., 0.], 0.4)[0], 0.5);
    assert_eq!(ledge_walk_stick([1., 0.], 1.)[0], 0.5);
    assert_eq!(
        ledge_walk_stick([f32::NAN, f32::INFINITY], f32::NAN),
        [0., 0.]
    );
    near(ledge_walk_stick([0.9, 0.], f32::NAN)[0], 0.33);
    assert_eq!(ledge_walk_stick([0., 0.], 0.35), [0., 0.]);
}

#[test]
fn faster_walk_still_clears_dash_flick_bits_and_restores_physical_input_on_exit() {
    use kirby_assist_core::{
        manual_controls::{ManualOverlay, TurboOutput, LEFT_STICK_DIRECTIONS},
        native_input::{ButtonTrace, MAPPER_A},
    };
    let human = ButtonTrace {
        held: MAPPER_A | LEFT_STICK_DIRECTIONS,
        previous: MAPPER_A | LEFT_STICK_DIRECTIONS,
        ..ButtonTrace::default()
    };
    let mut overlay = ManualOverlay::default();
    let physical = [1., 0.];
    let frame = overlay.apply_controls(
        human,
        physical,
        TurboOutput {
            consume_dpad: false,
            stick: Some(ledge_walk_stick(physical, 0.35)),
        },
        (1 << 21) | (1 << 23), // Manual walk suppresses horizontal flick aliases.
        0,
        0,
    );
    near(frame.stick[0], 0.5);
    assert_eq!(frame.buttons.held & LEFT_STICK_DIRECTIONS, 0);
    assert_eq!(frame.buttons.held & MAPPER_A, MAPPER_A);
    let release = overlay.apply_controls(human, physical, TurboOutput::default(), 0, 0, 0);
    assert_eq!(release.stick, physical);
    assert_eq!(release.buttons.held, human.held);
}
