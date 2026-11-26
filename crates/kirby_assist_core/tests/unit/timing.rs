use super::*;

/// Regression test for `motion_event_continuation_does_not_accept_rewinds_or_nan`.
#[test]
fn motion_event_continuation_does_not_accept_rewinds_or_nan() {
    let previous = (7, 2, 3, 5.0);
    assert!(same_motion_event(previous, (7, 2, 3, 5.0)));
    assert!(same_motion_event(previous, (7, 2, 3, 6.0)));
    assert!(!same_motion_event(previous, (7, 2, 3, 4.0)));
    assert!(!same_motion_event(previous, (7, 2, 3, f32::NAN)));
    assert!(!same_motion_event((7, 2, 3, f32::NAN), previous));
    assert!(!same_motion_event(previous, (8, 2, 3, 6.0)));
    assert!(!same_motion_event(previous, (7, 4, 3, 6.0)));
    assert!(!same_motion_event(previous, (7, 2, 4, 6.0)));
}

/// Regression test for `contact_uses_same_frame_for_geometry_protection_delay_and_recovery`.
#[test]
fn contact_uses_same_frame_for_geometry_protection_delay_and_recovery() {
    let active = ActiveWindow { first: 6, last: 12 };
    let recovery = RecoveryWindow {
        vulnerable_in: Some(9.0),
        actionable_in: Some(11.0),
    };
    assert_eq!(
        active.first_contact(1.0, recovery, |f| f >= 10.0),
        Some(PredictedContact {
            in_frames: 10.0,
            recovery_margin: Some(1.0)
        })
    );
    assert_eq!(active.first_contact(1.0, recovery, |f| f >= 11.0), None);
    assert_eq!(
        active.first_contact(
            0.0,
            RecoveryWindow {
                vulnerable_in: None,
                ..recovery
            },
            |_| true
        ),
        None
    );
    for invalid in [f32::NAN, f32::INFINITY, -1.0] {
        assert_eq!(active.first_contact(invalid, recovery, |_| true), None);
        assert_eq!(
            active.first_contact(
                0.0,
                RecoveryWindow {
                    vulnerable_in: Some(invalid),
                    ..recovery
                },
                |_| true
            ),
            None
        );
    }
}

/// Regression test for `clock_reuses_polls_freezes_in_stop_and_accepts_loops_and_identity_changes`.
#[test]
fn clock_reuses_polls_freezes_in_stop_and_accepts_loops_and_identity_changes() {
    let mut clock = AnimationClock::default();
    let sample = AnimationSample {
        object: 1,
        status: crate::native_states::status::WAIT,
        kind: 1,
        frame: 5.0,
        auxiliary: 0.0,
    };
    let first = clock.observe(sample, false);
    for _ in 0..20 {
        assert_eq!(clock.observe(sample, false), first);
    }
    assert_eq!(clock.observe(sample, true), None);
    assert_eq!(clock.observe(sample, false), first);
    assert_ne!(
        clock.observe(
            AnimationSample {
                frame: 0.0,
                ..sample
            },
            false
        ),
        first
    );
    let loop_tick = clock.observe(sample, false);
    assert_ne!(
        clock.observe(
            AnimationSample {
                object: 2,
                ..sample
            },
            false
        ),
        loop_tick
    );
    assert_eq!(
        clock.observe(
            AnimationSample {
                frame: f32::NAN,
                ..sample
            },
            false
        ),
        None
    );
}
/// Regression test for `native_timer_advances_mash_in_a_frozen_pose_without_counting_mapper_polls`.
#[test]
fn native_timer_advances_mash_in_a_frozen_pose_without_counting_mapper_polls() {
    let mut clock = AnimationClock::default();
    let sample = AnimationSample {
        object: 1,
        status: crate::native_states::status::WALK,
        kind: 1,
        frame: 0.0,
        auxiliary: 30.0,
    };
    let first = clock.observe(sample, false);
    assert_eq!(clock.observe(sample, false), first);
    assert_ne!(
        clock.observe(
            AnimationSample {
                auxiliary: 29.0,
                ..sample
            },
            false
        ),
        first
    );
    assert_eq!(
        clock.observe(
            AnimationSample {
                auxiliary: f32::NAN,
                ..sample
            },
            false
        ),
        None
    );
}
