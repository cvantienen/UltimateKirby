mod tests {
    use super::*;
    use crate::kirby::FOLLOWUP_BUTTONS;
    /// Builds a Kirby trip observation for follow-up regression tests.
    fn ctx(status: i32, target_status: i32) -> TripContext {
        TripContext {
            object: 1,
            kirby: true,
            status,
            motion: 1,
            frame: 1.0,
            grounded: true,
            facing: 1.0,
            stopped: false,
            targets: [
                Some(TripTarget {
                    id: 2,
                    status: target_status,
                    previous_status: crate::native_states::status::WAIT,
                    relative_position: [8.0, 0.0],
                    grounded: true,
                    hit_confirmed: true,
                }),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        }
    }
    fn baseline(state: &mut TripFollowup, mut trip: TripContext) {
        trip.frame -= 0.5;
        trip.targets[0].as_mut().unwrap().status = status::WAIT;
        trip.targets[0].as_mut().unwrap().hit_confirmed = false;
        state.sample(true, trip);
    }
    /// Regression test for `either_tilt_trip_flicks_in_existing_facing_and_new_tilt_rearms`.
    #[test]
    fn either_tilt_trip_flicks_in_existing_facing_and_new_tilt_rearms() {
        for status in [
            crate::native_states::status::ATTACK_S3,
            crate::native_states::status::ATTACK_LW3,
        ] {
            for facing in [-1.0, 1.0] {
                for trip_status in [
                    crate::native_states::status::SLIP,
                    crate::native_states::status::SLIP_DAMAGE,
                    crate::native_states::status::SLIP_WAIT,
                ] {
                    let mut state = TripFollowup::default();
                    let trip = TripContext {
                        facing,
                        targets: {
                            let mut targets = ctx(status, trip_status).targets;
                            targets[0].as_mut().unwrap().relative_position[0] *= facing;
                            targets
                        },
                        ..ctx(status, trip_status)
                    };
                    baseline(&mut state, trip);
                    assert_eq!(state.sample(true, trip).requested, 0);
                    let attack = state.sample(true, TripContext { frame: 2.0, ..trip });
                    assert_eq!(attack.requested, MAPPER_A);
                    assert_eq!(attack.stick, Some([facing, 0.]));
                    state.sample(
                        true,
                        TripContext {
                            status: crate::native_states::status::ATTACK_S4_START,
                            ..trip
                        },
                    );
                    state.sample(
                        true,
                        TripContext {
                            status: crate::native_states::status::WAIT,
                            ..trip
                        },
                    );
                    baseline(&mut state, trip);
                    state.sample(true, trip);
                    assert!(state.watching());
                    assert_eq!(
                        state
                            .sample(true, TripContext { frame: 2.0, ..trip })
                            .requested,
                        MAPPER_A
                    );
                }
            }
        }
    }
    /// Regression test for `trip_takeover_suppresses_physical_actions_stick_and_taunts_through_release_gaps`.
    #[test]
    fn trip_takeover_suppresses_physical_actions_stick_and_taunts_through_release_gaps() {
        use crate::manual_controls::{ManualOverlay, TurboOutput, LEFT_STICK_DIRECTIONS};
        use crate::native_input::ButtonTrace;
        let mut state = TripFollowup::default();
        let mut overlay = ManualOverlay::default();
        let human = ButtonTrace {
            held: FOLLOWUP_BUTTONS | LEFT_STICK_DIRECTIONS,
            ..ButtonTrace::default()
        };
        baseline(&mut state, ctx(status::ATTACK_LW3, status::SLIP));
        for frame in 1..=5 {
            let out = state.sample(
                true,
                TripContext {
                    frame: frame as f32,
                    ..ctx(
                        crate::native_states::status::ATTACK_LW3,
                        crate::native_states::status::SLIP,
                    )
                },
            );
            let mapped = overlay.apply_controls(
                human,
                [-1.0, -1.0],
                TurboOutput { stick: out.stick },
                0,
                out.requested,
                out.owned,
            );
            assert_eq!(mapped.buttons.held & FOLLOWUP_BUTTONS, out.requested);
            assert_eq!(mapped.stick, out.stick.unwrap());
            assert_eq!(mapped.buttons.pressed & MAPPER_A, out.requested);
        }
        let off = state.sample(
            false,
            ctx(
                crate::native_states::status::ATTACK_LW3,
                crate::native_states::status::SLIP,
            ),
        );
        let restored = overlay.apply_controls(
            human,
            [-1.0, -1.0],
            TurboOutput { stick: off.stick },
            0,
            off.requested,
            off.owned,
        );
        assert_eq!(
            restored.buttons.held & FOLLOWUP_BUTTONS,
            human.held & FOLLOWUP_BUTTONS
        );
        assert_eq!(restored.stick, [-1.0, -1.0]);
    }
    /// Regression test for `no_trip_no_tilt_and_non_kirby_are_inert_and_interruptions_cancel`.
    #[test]
    fn no_trip_no_tilt_and_non_kirby_are_inert_and_interruptions_cancel() {
        for sample in [
            ctx(
                crate::native_states::status::ATTACK_LW3,
                crate::native_states::status::WAIT,
            ),
            ctx(
                crate::native_states::status::WAIT,
                crate::native_states::status::SLIP,
            ),
            TripContext {
                kirby: false,
                ..ctx(
                    crate::native_states::status::ATTACK_LW3,
                    crate::native_states::status::SLIP,
                )
            },
            // An unreadable facing must not guess "forward = right".
            TripContext {
                facing: 0.0,
                ..ctx(
                    crate::native_states::status::ATTACK_LW3,
                    crate::native_states::status::SLIP,
                )
            },
            TripContext {
                facing: f32::NAN,
                ..ctx(
                    crate::native_states::status::ATTACK_LW3,
                    crate::native_states::status::SLIP,
                )
            },
        ] {
            let mut state = TripFollowup::default();
            assert_eq!(state.sample(true, sample), ControllerOutput::default());
            assert!(!state.watching());
        }
        for terminal in [
            TripContext {
                grounded: false,
                ..ctx(
                    crate::native_states::status::JUMP,
                    crate::native_states::status::SLIP,
                )
            },
            ctx(
                crate::native_states::status::DAMAGE,
                crate::native_states::status::SLIP,
            ),
        ] {
            let mut state = TripFollowup::default();
            baseline(&mut state, ctx(status::ATTACK_LW3, status::SLIP));
            state.sample(
                true,
                ctx(
                    crate::native_states::status::ATTACK_LW3,
                    crate::native_states::status::SLIP,
                ),
            );
            assert_eq!(state.sample(true, terminal).requested, 0);
            assert!(!state.watching());
            assert_eq!(
                state
                    .sample(
                        true,
                        TripContext {
                            frame: 2.0,
                            ..terminal
                        }
                    )
                    .owned,
                0
            );
        }
        let mut state = TripFollowup::default();
        state.sample(
            true,
            ctx(
                crate::native_states::status::ATTACK_LW3,
                crate::native_states::status::SLIP,
            ),
        );
        assert_eq!(
            state
                .sample(
                    false,
                    ctx(
                        crate::native_states::status::ATTACK_LW3,
                        crate::native_states::status::SLIP,
                    )
                )
                .owned,
            0
        );
        assert!(!state.watching());
        state.sample(
            true,
            ctx(
                crate::native_states::status::ATTACK_LW3,
                crate::native_states::status::SLIP,
            ),
        );
        assert_eq!(
            state
                .sample(
                    true,
                    TripContext {
                        object: 3,
                        ..ctx(
                            crate::native_states::status::WAIT,
                            crate::native_states::status::WAIT,
                        )
                    }
                )
                .owned,
            0
        );
        assert!(!state.watching());
    }
}
