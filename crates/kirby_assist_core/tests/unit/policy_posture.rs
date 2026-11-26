use super::*;
use crate::native_states::{situation, status};

fn grounded(facing: f32) -> ManualContext {
    ManualContext {
        status: status::WAIT,
        situation: situation::GROUND,
        motion: 0,
        frame: 0.0,
        rate: 1.0,
        stopped: false,
        facing,
    }
}

#[test]
fn configured_forward_a_tilt_aims_without_a_smash_flick() {
    let settings = TiltSettings {
        forward: true,
        ..TiltSettings::default()
    };
    let mut posture = Posture::default();
    let output = posture.sample_configured(
        true,
        false,
        true,
        settings,
        MAPPER_A,
        [-1.0, 0.0],
        [0.0; 2],
        Some(1.0),
        grounded(-1.0),
    );
    assert_eq!(output.stick, None);
    assert_eq!(output.requested, 0);
    let reverse = posture.sample_configured(
        true,
        false,
        true,
        settings,
        MAPPER_A,
        [1.0, 0.0],
        [0.0; 2],
        Some(-1.0),
        grounded(1.0),
    );
    assert_eq!(reverse.stick, None);
}

#[test]
fn configured_up_a_tilt_and_down_c_stick_keep_their_attack_directions() {
    let mut posture = Posture::default();
    let up = TiltSettings {
        up: true,
        ..TiltSettings::default()
    };
    for (facing, target, physical) in [
        (1.0, -1.0, [0.0, 1.0]),
        (-1.0, 1.0, [0.4, 0.9]),
        (1.0, 1.0, [0.0, 1.0]),
    ] {
        let output = posture.sample_configured(
            true,
            false,
            true,
            up,
            MAPPER_A,
            physical,
            [0.0; 2],
            Some(target),
            grounded(facing),
        );
        assert_eq!(output.stick, None);
        assert_eq!(output.requested, 0);
    }
    let down = TiltSettings {
        down: true,
        ..TiltSettings::default()
    };
    assert_eq!(
        Posture::configured_c_stick(down, 0, [0.0; 2], [0.0, -1.0], Some(-1.0), grounded(1.0),),
        Some(([0.0; 2], [-0.2, -0.8]))
    );
}

#[test]
fn configured_up_c_stick_never_combines_attack_with_sideways_turn() {
    let settings = TiltSettings {
        up: true,
        ..TiltSettings::default()
    };
    assert_eq!(
        Posture::configured_c_stick(settings, 0, [0.0; 2], [0.0, 1.0], Some(-1.0), grounded(1.0),),
        Some(([0.0, 0.0], [-0.2, 0.8]))
    );
    assert_eq!(
        Posture::configured_c_stick(settings, 0, [0.0; 2], [0.0, 1.0], Some(1.0), grounded(1.0),),
        Some(([0.0, 0.0], [0.2, 0.8]))
    );
}
