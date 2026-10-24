use kirby_assist_core::{
    observation::{Fact, Field, ObservationSnapshot},
    timing::{
        motion_elapsed, motion_progress, AnimationClock, AnimationSample, MotionProgress,
        MotionStamp,
    },
};
use std::cell::Cell;

#[test]
fn observations_read_once_per_field_and_identity_then_refresh_next_callback() {
    let calls = Cell::new(0);
    let read = || {
        calls.set(calls.get() + 1);
        Fact::Int(calls.get())
    };
    let snapshot = ObservationSnapshot::default();
    for _ in 0..20 {
        assert_eq!(snapshot.read(1, Field::Status, read), Fact::Int(1));
    }
    assert_eq!(calls.get(), 1);
    assert_eq!(snapshot.read(2, Field::Status, read), Fact::Int(2));
    assert_eq!(snapshot.read(1, Field::Situation, read), Fact::Int(3));
    // A new observation does not inherit a fighter's stale status.
    assert_eq!(
        ObservationSnapshot::default().read(1, Field::Status, read),
        Fact::Int(4)
    );
    // Re-entrant reads cannot hold a RefCell borrow or deadlock a mutex.
    assert_eq!(
        snapshot.read(1, Field::Frame, || {
            assert_eq!(snapshot.read(1, Field::Status, read), Fact::Int(1));
            Fact::Float(5.0)
        }),
        Fact::Float(5.0)
    );
}

#[test]
fn overflow_reads_through_without_evicting_valid_facts_or_allocating_new_slots() {
    let snapshot = ObservationSnapshot::default();
    for id in 0..8 {
        snapshot.read(id, Field::Status, || Fact::Int(id as i32));
    }
    assert_eq!(
        snapshot.read(99, Field::Status, || Fact::Int(10)),
        Fact::Int(10)
    );
    assert_eq!(
        snapshot.read(99, Field::Status, || Fact::Int(11)),
        Fact::Int(11)
    );
    assert_eq!(
        snapshot.read(0, Field::Status, || panic!("cached fact was evicted")),
        Fact::Int(0)
    );
}

#[test]
fn shared_progress_matches_existing_valid_sequence_timing_without_added_frames() {
    for rate in [0.25, 0.5, 1.0, 1.5, 2.0] {
        for frames in [0.0, 0.5, 1.0, 2.0, 4.0] {
            let old = (7, 11, 10.0, rate);
            let current = (7, 11, 10.0 + frames * rate, 1.0);
            // Use the previous interval's rate, even if this sample changes it.
            assert_eq!(motion_elapsed(Some(old), current, 0.0), frames);
            assert_eq!(motion_elapsed(Some(old), (8, 12, 0.0, rate), 0.0), 1.0);
        }
    }
    assert_eq!(motion_elapsed(None, (0, 1, 0.0, 1.0), 0.0), 0.0);
    assert_eq!(motion_elapsed(None, (0, 1, 0.0, 1.0), 1.0), 1.0);
    let stamp = MotionStamp::new(0, 1, 10.0, 1.0);
    assert_eq!(
        motion_progress(Some(stamp), stamp),
        MotionProgress::Duplicate
    );
    assert_eq!(
        motion_progress(
            Some(stamp),
            MotionStamp {
                frame: 0.0,
                ..stamp
            }
        ),
        MotionProgress::Rewind
    );
    assert_eq!(
        motion_progress(
            Some(stamp),
            MotionStamp {
                rate: f32::NAN,
                ..stamp
            }
        ),
        MotionProgress::Invalid
    );
}

#[test]
fn shared_event_clock_preserves_mash_slide_turbo_ticks_and_auxiliary_progress() {
    let mut clock = AnimationClock::default();
    let sample = AnimationSample {
        object: 1,
        status: 0,
        kind: 1,
        frame: 8.0,
        auxiliary: 0.0,
    };
    let first = clock.observe(sample, false).unwrap();
    for _ in 0..20 {
        assert_eq!(clock.observe(sample, false), Some(first));
    }
    assert_eq!(
        clock.observe(
            AnimationSample {
                frame: 9.0,
                ..sample
            },
            true
        ),
        None
    );
    assert_eq!(
        clock.observe(
            AnimationSample {
                frame: 9.0,
                ..sample
            },
            false
        ),
        Some(first + 1)
    );
    let frozen = AnimationSample {
        frame: 9.0,
        auxiliary: 5.0,
        ..sample
    };
    assert_eq!(clock.observe(frozen, false), Some(first + 2));
    assert_eq!(
        clock.observe(
            AnimationSample {
                frame: 0.0,
                auxiliary: 5.0,
                ..sample
            },
            false
        ),
        Some(first + 3)
    );
}

#[test]
fn simulation_frames_are_separate_from_animation_and_mapper_polls() {
    use kirby_assist_core::timing::simulation_elapsed;
    assert_eq!(simulation_elapsed(None, 100), Some(0));
    assert_eq!(simulation_elapsed(Some(100), 100), Some(0));
    assert_eq!(simulation_elapsed(Some(100), 103), Some(3));
    assert_eq!(simulation_elapsed(Some(100), 99), None);
}
