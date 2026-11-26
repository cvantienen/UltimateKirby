mod tests {
    use super::*;

    /// Regression test for `held_guard_and_teeter_poses_use_opponent_frames_without_counting_duplicates_or_hitstop`.
    #[test]
    fn held_guard_and_teeter_poses_use_opponent_frames_without_counting_duplicates_or_hitstop() {
        for status in [
            crate::native_states::GUARD_ON,
            crate::native_states::GUARD,
            crate::native_states::status::OTTOTTO,
            crate::native_states::status::OTTOTTO_WAIT,
        ] {
            let mut clock = NativeAttempt::default();
            let ctx = GameplayContext {
                status,
                motion_kind: 123,
                motion_frame: 20.0,
                motion_rate: 1.0,
                ..super::super::tests::context()
            };
            let opponent = |frame| {
                Some(Opponent {
                    motion: Some(crate::gameplay::MotionTiming {
                        kind: 456,
                        frame,
                        rate: 1.0,
                        remaining: None,
                    }),
                    ..super::super::tests::enemy(super::super::CLIFF_CLIMB, false)
                })
            };
            clock.advance_pause(ctx, opponent(0.0), 60);
            clock.advance_pause(ctx, opponent(1.0), 60);
            assert_eq!(clock.elapsed(), 1.0);
            for _ in 0..20 {
                clock.advance_pause(ctx, opponent(1.0), 60);
            }
            assert_eq!(clock.elapsed(), 1.0);
            clock.advance_pause(
                GameplayContext {
                    stopped: true,
                    ..ctx
                },
                opponent(5.0),
                60,
            );
            clock.advance_pause(ctx, opponent(5.0), 60);
            assert_eq!(clock.elapsed(), 1.0);
            clock.advance_pause(ctx, opponent(6.0), 60);
            assert_eq!(clock.elapsed(), 2.0);
            // Returning to a progressing pose must not add both clocks.
            clock.advance_pause(
                GameplayContext {
                    motion_frame: 21.0,
                    ..ctx
                },
                opponent(7.0),
                60,
            );
            assert_eq!(clock.elapsed(), 3.0);
            assert!(!clock.expired);
        }
        assert!(super::super::needs_opponent_clock(0, 0.0));
        assert!(!super::super::needs_opponent_clock(0, 1.0));
        assert!(!super::super::needs_opponent_clock(
            crate::native_states::status::FALL,
            1.0
        ));
    }
}
