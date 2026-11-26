mod tests {
    use super::*;
    /// Regression test for `later_contact_cancels_first_hit_followup_even_during_hitstop`.
    #[test]
    fn later_contact_cancels_first_hit_followup_even_during_hitstop() {
        let mut tool = FairFollowup::default();
        tool.sample(true, fair(10.0));
        let later = FairContext {
            stopped: true,
            ..fair(17.0)
        };
        assert_eq!(tool.sample(true, later).requested, 0);
        assert!(!tool.queued);
        let landing = FairContext {
            grounded: true,
            status: crate::native_states::status::LANDING_ATTACK_AIR,
            motion: 4,
            frame: 0.0,
            hit_now: false,
            ..fair(10.0)
        };
        assert_eq!(tool.sample(true, landing).requested, 0);
    }
    /// Builds a forward-air observation for follow-up regression tests.
    fn fair(frame: f32) -> FairContext {
        FairContext {
            object: 1,
            kirby: true,
            status: crate::native_states::status::ATTACK_AIR,
            motion: FAIR,
            frame,
            rate: 1.0,
            grounded: false,
            stopped: false,
            facing: 1.0,
            hit_now: true,
        }
    }
    /// Regression test for `manual_first_hit_survives_late_fair_skipped_frames_and_falling_until_grounded_smash`.
    #[test]
    fn manual_first_hit_survives_late_fair_skipped_frames_and_falling_until_grounded_smash() {
        for facing in [-1.0, 1.0] {
            let mut tool = FairFollowup::default();
            tool.sample(
                true,
                FairContext {
                    facing,
                    ..fair(10.0)
                },
            );
            for frame in [11.0, 13.0, 15.0, 17.0] {
                let c = FairContext {
                    facing,
                    frame,
                    hit_now: false,
                    ..fair(10.0)
                };
                assert_eq!(
                    tool.sample_with_velocity(true, c, -1.0).stick,
                    Some([0.0, -1.0])
                );
                assert!(tool.queued);
            }
            for frame in [0.0, 2.0, 0.0] {
                let c = FairContext {
                    status: crate::native_states::status::FALL,
                    motion: 3,
                    frame,
                    facing,
                    hit_now: false,
                    ..fair(10.0)
                };
                assert_eq!(tool.sample_with_velocity(true, c, -1.0).requested, 0);
                assert!(tool.queued);
            }
            let landing = FairContext {
                grounded: true,
                status: crate::native_states::status::LANDING_ATTACK_AIR,
                motion: 4,
                frame: 0.0,
                facing,
                hit_now: false,
                ..fair(10.0)
            };
            assert_eq!(tool.sample(true, landing).requested, MAPPER_A);
            assert_eq!(tool.output.stick, Some([facing, UP_ANGLE]));
            for _ in 0..50 {
                assert_eq!(tool.sample(true, landing).requested, MAPPER_A);
            }
            for frame in 1..12 {
                let out = tool.sample(
                    true,
                    FairContext {
                        frame: frame as f32,
                        ..landing
                    },
                );
                assert_eq!(out.requested, if frame % 2 == 0 { MAPPER_A } else { 0 });
            }
            let accepted = tool.sample(
                true,
                FairContext {
                    status: crate::native_states::status::ATTACK_S4_START,
                    motion: 5,
                    ..landing
                },
            );
            assert_eq!(accepted.requested, 0);
            assert_eq!(accepted.stick, Some([facing, UP_ANGLE]));
        }
    }
    /// Regression test for `first_hit_with_mixed_landing_flag_is_latched_without_sending_a_second_aerial`.
    #[test]
    fn first_hit_with_mixed_landing_flag_is_latched_without_sending_a_second_aerial() {
        let mut tool = FairFollowup::default();
        let contact = FairContext {
            grounded: true,
            ..fair(11.0)
        };
        assert_eq!(tool.sample(true, contact).requested, 0);
        assert!(tool.queued);
        let landing = FairContext {
            status: crate::native_states::status::LANDING_ATTACK_AIR,
            motion: 2,
            frame: 0.0,
            hit_now: false,
            ..contact
        };
        assert_eq!(tool.sample(true, landing).requested, MAPPER_A);
    }
    /// A delayed observation of the first-hit contact still arms the landing smash.
    #[test]
    fn delayed_first_hit_observation_arms_before_the_second_hit() {
        let mut tool = FairFollowup::default();
        let delayed_hit = FairContext {
            frame: 13.0,
            ..fair(10.0)
        };
        assert_eq!(tool.sample(true, delayed_hit).requested, 0);
        assert!(tool.queued);
    }
    /// Regression test for `fresh_first_hit_reported_on_immediate_landing_arms_but_later_hit_does_not`.
    #[test]
    fn fresh_first_hit_reported_on_immediate_landing_arms_but_later_hit_does_not() {
        for previous_frame in [11.0, 17.0] {
            let mut tool = FairFollowup::default();
            tool.sample(
                true,
                FairContext {
                    hit_now: false,
                    ..fair(previous_frame)
                },
            );
            let landing = FairContext {
                status: crate::native_states::status::LANDING_ATTACK_AIR,
                motion: crate::timing::motion_hash("landing_air_f"),
                frame: 0.0,
                grounded: true,
                ..fair(10.0)
            };
            tool.sample(true, landing);
            let out = tool.sample(
                true,
                FairContext {
                    frame: 1.0,
                    hit_now: false,
                    ..landing
                },
            );
            assert_eq!(
                out.requested,
                if previous_frame == 11.0 { MAPPER_A } else { 0 }
            );
        }
    }
    /// Regression test for `landing_hit_flag_changes_cannot_create_another_smash_input_frame`.
    #[test]
    fn landing_hit_flag_changes_cannot_create_another_smash_input_frame() {
        let mut tool = FairFollowup::default();
        tool.sample(true, fair(10.0));
        let landing = FairContext {
            grounded: true,
            status: crate::native_states::status::LANDING_ATTACK_AIR,
            motion: 2,
            frame: 0.0,
            hit_now: false,
            ..fair(10.0)
        };
        let press = tool.sample(true, landing);
        assert_eq!(press.requested, MAPPER_A);
        for hit_now in [true, false, true] {
            assert_eq!(tool.sample(true, FairContext { hit_now, ..landing }), press);
        }
        assert_eq!(
            tool.sample(
                true,
                FairContext {
                    frame: 1.0,
                    ..landing
                }
            )
            .requested,
            0
        );
    }
    /// Regression test for `first_hit_waits_for_landing_then_buffers_upward_smash_without_charging`.
    #[test]
    fn first_hit_waits_for_landing_then_buffers_upward_smash_without_charging() {
        for facing in [-1.0, 1.0] {
            let mut tool = FairFollowup::default();
            let hit = FairContext {
                facing,
                stopped: true,
                ..fair(10.0)
            };
            let armed = tool.sample(true, hit);
            assert_ne!(armed.owned, 0);
            assert_eq!(armed.requested, 0);
            assert_eq!(tool.sample(true, hit), armed);
            assert_eq!(
                tool.sample(
                    true,
                    FairContext {
                        stopped: false,
                        hit_now: false,
                        ..hit
                    }
                )
                .requested,
                0
            );
            let landing = FairContext {
                grounded: true,
                status: crate::native_states::status::LANDING_ATTACK_AIR,
                motion: 2,
                frame: 0.0,
                hit_now: false,
                stopped: false,
                ..hit
            };
            let out = tool.sample(true, landing);
            assert_eq!(out.requested, MAPPER_A);
            assert_eq!(out.stick, Some([facing, UP_ANGLE]));
            assert_eq!(
                tool.sample(
                    true,
                    FairContext {
                        frame: 1.0,
                        ..landing
                    }
                )
                .requested,
                0
            );
            assert_eq!(
                tool.sample(
                    true,
                    FairContext {
                        frame: 2.0,
                        ..landing
                    }
                )
                .requested,
                MAPPER_A
            );
            let startup = FairContext {
                status: crate::native_states::status::ATTACK_S4_START,
                motion: 3,
                frame: 0.0,
                ..landing
            };
            let accepted = tool.sample(true, startup);
            assert_eq!(accepted.requested, 0);
            assert_eq!(accepted.stick, Some([facing, UP_ANGLE]));
            let attack = FairContext {
                status: crate::native_states::status::ATTACK_S4,
                motion: 4,
                frame: 0.0,
                ..landing
            };
            assert_eq!(tool.sample(true, attack).requested, 0);
            assert_eq!(
                tool.sample(
                    true,
                    FairContext {
                        frame: 1.0,
                        ..attack
                    }
                )
                .owned,
                0,
                "acceptance must not leave a permanent release takeover"
            );
        }
    }
    /// Regression test for `shield_stale_or_later_hits_and_other_moves_cannot_arm`.
    #[test]
    fn shield_stale_or_later_hits_and_other_moves_cannot_arm() {
        for c in [
            FairContext {
                hit_now: false,
                ..fair(10.0)
            },
            fair(17.0),
            fair(25.0),
            FairContext {
                kirby: false,
                ..fair(10.0)
            },
            FairContext {
                motion: 0xc3a4e2597,
                ..fair(10.0)
            },
        ] {
            assert_eq!(
                FairFollowup::default().sample(true, c),
                ControllerOutput::default()
            );
        }
    }
    /// Regression test for `damage_still_cancels_the_confirm`.
    #[test]
    fn damage_still_cancels_the_confirm() {
        for changed in [FairContext {
            status: crate::native_states::status::DAMAGE_FLY,
            motion: 5,
            frame: 0.0,
            ..fair(10.0)
        }] {
            let mut tool = FairFollowup::default();
            tool.sample(true, fair(10.0));
            assert_eq!(tool.sample(true, changed).requested, 0);
            assert!(!tool.queued);
            let ground = FairContext {
                grounded: true,
                status: crate::native_states::status::WAIT,
                motion: 6,
                frame: 1.0,
                hit_now: false,
                ..changed
            };
            assert_eq!(tool.sample(true, ground).owned, 0);
        }
    }
}
