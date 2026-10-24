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

#[test]
fn dodge_buffers_in_late_native_squat_before_airborne_and_transition_flags() {
    for (button, direction) in [(DPAD_LEFT, -1.0), (DPAD_RIGHT, 1.0)] {
        let mut slide = SlideState::default();
        let ground = super::tests::ground();
        assert_eq!(slide.sample(true, 0, button, ground).requested, MAPPER_X);
        let squat = SlideContext {
            status: 0xa,
            can_jump: false,
            motion_frame: 1.0,
            ..ground
        };
        assert_eq!(slide.sample(true, 1, 0, squat).requested, 0);
        let late = SlideContext {
            motion_frame: 2.0,
            ..squat
        };
        let buffered = slide.sample(true, 2, 0, late);
        assert_eq!(buffered.requested, MAPPER_ZR);
        assert_eq!(buffered.stick, Some([direction * 0.9588, -0.2839]));
        for _ in 0..100 {
            assert_eq!(slide.sample(true, 2, 0, late), buffered);
        }
        // Native jump may precede the situation flag; do not call this a landing.
        let mixed = SlideContext {
            status: 0xb,
            motion_frame: 0.0,
            ..late
        };
        assert_eq!(slide.sample(true, 3, 0, mixed), buffered);
        assert_eq!(
            slide.sample(
                true,
                4,
                0,
                SlideContext {
                    situation: 2,
                    motion_frame: 1.0,
                    ..mixed
                }
            ),
            buffered
        );
        assert_eq!(
            slide
                .sample(
                    true,
                    5,
                    0,
                    SlideContext {
                        status: 0x23,
                        situation: 2,
                        ..mixed
                    }
                )
                .requested,
            0
        );
    }
}

#[test]
fn missed_squat_observation_uses_first_native_jump_without_waiting_for_readiness() {
    let mut slide = SlideState::default();
    let ground = super::tests::ground();
    slide.sample(true, 0, DPAD_RIGHT, ground);
    let early = SlideContext {
        status: 0xb,
        situation: 0,
        motion_frame: 0.0,
        can_jump: false,
        can_air_dodge: false,
        ..ground
    };
    assert_eq!(slide.sample(true, 1, 0, early).requested, MAPPER_ZR);
}

#[test]
fn unavailable_clock_preserves_only_existing_input_through_native_stop() {
    let mut slide = SlideState::default();
    let ground = super::tests::ground();
    slide.sample(true, 0, DPAD_RIGHT, ground);
    assert_eq!(
        slide.frozen_output(SlideContext {
            stopped: true,
            ..ground
        }),
        super::owned()
    );
    let late = SlideContext {
        status: 0xa,
        motion_frame: 2.0,
        can_jump: false,
        ..ground
    };
    let buffered = slide.sample(true, 1, 0, late);
    let stopped = SlideContext {
        stopped: true,
        ..late
    };
    for _ in 0..100 {
        assert_eq!(slide.frozen_output(stopped), buffered);
        assert_eq!(slide.sample(true, 1, 0, stopped), buffered);
    }
    for bad in [
        SlideContext {
            status: 0x47,
            ..stopped
        },
        SlideContext {
            gameplay: false,
            ..stopped
        },
        SlideContext {
            status: 0x23,
            ..stopped
        },
        late, // Missing clock without actual native stop is not delivery evidence.
    ] {
        assert_eq!(slide.frozen_output(bad), LedgeOutput::default());
    }
}
