//! Existing regression cases for `slide`. Kept outside the runtime code.
use super::*;
#[test]
fn delayed_takeoff_and_dodge_readiness_still_complete_the_requested_slide() {
    let ground = SlideContext {
        gameplay: true,
        stopped: false,
        status: 0,
        situation: 0,
        motion_frame: 0.0,
        can_jump: true,
        can_air_dodge: false,
        safe_ground: [true; 2],
        input_delay: 5.0,
    };
    let mut slide = SlideState::default();
    assert_eq!(
        slide.sample(true, 0, DPAD_RIGHT, ground).requested,
        MAPPER_X
    );
    for tick in 1..10 {
        assert_eq!(
            slide
                .sample(
                    true,
                    tick,
                    0,
                    SlideContext {
                        can_jump: false,
                        motion_frame: tick as f32,
                        ..ground
                    }
                )
                .requested,
            0
        );
    }
    let air = SlideContext {
        status: 0xb,
        situation: 2,
        motion_frame: 3.0,
        can_jump: false,
        can_air_dodge: true,
        ..ground
    };
    let request = slide.sample(true, 10, 0, air);
    assert_eq!(request.requested, MAPPER_ZR);
    for tick in 11..15 {
        assert_eq!(
            slide
                .sample(
                    true,
                    tick,
                    0,
                    SlideContext {
                        motion_frame: (tick - 7) as f32,
                        ..air
                    }
                )
                .requested,
            MAPPER_ZR
        );
    }
    assert_eq!(
        slide
            .sample(
                true,
                15,
                0,
                SlideContext {
                    status: 0x23,
                    ..air
                }
            )
            .requested,
        0
    );
}
