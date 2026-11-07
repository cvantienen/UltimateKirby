use kirby_assist_core::{
    observation::{Fact, Field, GroundSupportSnapshot, ObservationSnapshot},
    timing::{
        motion_elapsed, motion_progress, AnimationClock, AnimationSample, MotionProgress,
        MotionStamp,
    },
};
use std::cell::Cell;

#[test]
fn repeated_command_callbacks_without_a_roster_upgrade_setup_only_once() {
    use kirby_assist_core::observation::ObservationSetup;
    let mut setup = ObservationSetup::new();
    let mut evaluations = 0;
    let mut upgrades = 0;
    for tick in 0..100 {
        let mut evaluated = false;
        // Mapper followed by repeated command-category queries. There is no
        // roster at all while all candidate policies are idle/unactionable.
        for discover in std::iter::once(false).chain(std::iter::repeat(true).take(100)) {
            if !setup.needed(tick, evaluated, discover) {
                continue;
            }
            setup.complete(tick, discover);
            if evaluated {
                upgrades += 1;
            } else {
                evaluations += 1;
                evaluated = true;
            }
        }
    }
    assert_eq!(evaluations, 100);
    assert_eq!(upgrades, 100);
    setup.clear();
    assert!(
        setup.needed(99, true, true),
        "new actor/session must not reuse old discovery"
    );
    assert!(
        setup.needed(100, false, false),
        "new input updates always refresh native facts"
    );
}

#[test]
fn floor_queries_are_shared_only_for_identical_inputs_within_one_observation() {
    let calls = Cell::new(0);
    let cache = GroundSupportSnapshot::default();
    let query = || {
        calls.set(calls.get() + 1);
        [true, false]
    };
    assert_eq!(cache.read(1, [4.0, 0.0], 0.5, query), [true, false]);
    for _ in 0..12 {
        assert_eq!(cache.read(1, [4.0, 0.0], 0.5, query), [true, false]);
    }
    assert_eq!(calls.get(), 1);
    cache.read(1, [4.0, 1.0], 0.5, query);
    cache.read(1, [4.0, 1.0], 0.8, query);
    cache.read(2, [4.0, 1.0], 0.8, query);
    assert_eq!(calls.get(), 4);
    // Moving platforms/geometry are queried again on the next callback, even
    // when the fighter's pose is identical to the preceding observation.
    assert_eq!(
        GroundSupportSnapshot::default().read(2, [4.0, 1.0], 0.8, || [false; 2]),
        [false; 2]
    );
}

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
        status: kirby_assist_core::native_states::status::WAIT,
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

#[test]
fn identity_cursor_and_kind_cache_remain_correct_when_interleaved_reentrant_and_recreated() {
    let s = ObservationSnapshot::default();
    for id in 0..8 {
        assert_eq!(
            s.read(id, Field::Kind, || Fact::Int(id as i32)),
            Fact::Int(id as i32)
        );
    }
    for id in [7, 0, 6, 1, 5, 2, 4, 3, 0, 7] {
        assert_eq!(
            s.read(id, Field::Kind, || panic!("kind read repeated")),
            Fact::Int(id as i32)
        );
    }
    assert_eq!(
        s.read(1, Field::X, || {
            assert_eq!(
                s.read(7, Field::Kind, || panic!("reentrant cache miss")),
                Fact::Int(7)
            );
            Fact::Float(12.5)
        }),
        Fact::Float(12.5)
    );
    assert_eq!(
        s.read(1, Field::X, || panic!("outer slot overwritten")),
        Fact::Float(12.5)
    );
    assert_eq!(s.read(99, Field::Kind, || Fact::Int(10)), Fact::Int(10));
    assert_eq!(s.read(99, Field::Kind, || Fact::Int(11)), Fact::Int(11));
    assert_eq!(
        ObservationSnapshot::default().read(1, Field::Kind, || Fact::Int(55)),
        Fact::Int(55)
    );
}

#[test]
fn roster_reads_once_per_observation_including_unavailable_and_refreshes_next_time() {
    use kirby_assist_core::observation::RosterSnapshot;
    let calls = Cell::new(0);
    for roster in [
        None,
        Some([Some(1), Some(9), None, None, None, None, None, None]),
    ] {
        let snapshot = RosterSnapshot::default();
        for _ in 0..12 {
            assert_eq!(
                snapshot.read(|| {
                    calls.set(calls.get() + 1);
                    roster
                }),
                roster
            );
        }
    }
    assert_eq!(calls.get(), 2);
    let changed = Some([Some(3); 8]);
    assert_eq!(RosterSnapshot::default().read(|| changed), changed);
}
