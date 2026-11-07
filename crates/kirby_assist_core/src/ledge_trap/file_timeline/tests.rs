//! Regression tests for ordered file-authored ledge timelines.
use super::*;
use crate::ledge_trap::{
    tests::{context, enemy},
    LedgeTrap, CLIFF_CLIMB,
};
use crate::native_input::*;
use crate::sequence_config::Catalog;
fn sequence(actions: &str) -> Sequence {
    Catalog::parse(&format!(
        "[kirby.neutral.timeline]\nstart_delay=0\nactions={actions}"
    ))
    .unwrap()
    .entries()[0]
        .sequence
}
fn ctx(frame: f32) -> GameplayContext {
    GameplayContext {
        motion_kind: crate::timing::motion_hash("wait"),
        motion_frame: frame,
        ..context()
    }
}
fn sample(timeline: &mut Timeline, context: GameplayContext) -> ControllerOutput {
    timeline.sample(LedgeSettings::default(), context, None, None, 1.0)
}
fn timeline(actions: &str) -> Timeline {
    Timeline::new(sequence(actions), enemy(CLIFF_CLIMB, false))
}
fn moving_enemy(frame: f32) -> Opponent {
    Opponent {
        motion: Some(crate::gameplay::MotionTiming {
            kind: 99,
            frame,
            rate: 1.0,
            remaining: None,
        }),
        ..enemy(CLIFF_CLIMB, false)
    }
}
#[test]
fn while_held_loops_without_extra_frames_and_rebases_clock_until_released() {
    let mut seq = sequence("tap:a:0:-0.6:1:3:8");
    seq.while_held = true;
    seq.timeout = 10;
    let mut t = Timeline::new(seq, enemy(CLIFF_CLIMB, false));
    for f in 0..1000 {
        let out = sample(&mut t, ctx(f as f32));
        assert_eq!(
            out.requested,
            if f % 4 == 0 { MAPPER_A } else { 0 },
            "frame {f}"
        );
        assert!(!t.finished());
    }
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                held: false,
                ..ctx(1000.)
            }
        )
        .requested,
        0
    );
    assert!(t.finished());
    let mut seq = sequence("tap:a:0:0:1:1:2");
    seq.start_delay = 2;
    seq.while_held = true;
    let mut t = Timeline::new(seq, enemy(CLIFF_CLIMB, false));
    for f in 0..12 {
        assert_eq!(
            sample(&mut t, ctx(f as f32)).requested,
            if f % 4 == 2 { MAPPER_A } else { 0 },
            "delayed frame {f}"
        );
    }
}
#[test]
fn taps_release_between_pulses_pause_on_hitstop_and_continue_to_next_action() {
    let mut t = timeline("tap:a:0:-0.6:2:3:12,hold:b:1");
    for frame in 0..=12 {
        let out = sample(&mut t, ctx(frame as f32));
        let on = frame < 12 && frame % 5 < 2;
        assert_eq!(
            out.requested,
            if frame == 12 {
                MAPPER_B
            } else if on {
                MAPPER_A
            } else {
                0
            },
            "frame {frame}"
        );
        assert_eq!(out.stick, Some(if on { [0., -0.6] } else { [0.; 2] }));
        for _ in 0..4 {
            assert_eq!(sample(&mut t, ctx(frame as f32)), out);
        }
        assert_eq!(
            sample(
                &mut t,
                GameplayContext {
                    stopped: true,
                    ..ctx(frame as f32)
                }
            ),
            out
        );
    }
}
#[test]
fn taps_hold_mapping_only_during_a_pulse_and_never_catch_up_missed_edges() {
    let mut t = timeline("tap:a:0:0:2:2:8");
    for frame in 0..8 {
        sample(&mut t, ctx(frame as f32));
        assert_eq!(t.attack_hold(), frame % 4 < 2);
    }
    let mut t = timeline("tap:a:0:0:1:1:8");
    sample(&mut t, ctx(0.));
    assert_eq!(sample(&mut t, ctx(3.)).requested, 0);
    assert!(t.finished());
}
#[test]
fn explicit_conditions_require_fresh_enemy_and_continue_without_extra_frames() {
    let target = Opponent {
        status: crate::native_states::GUARD,
        ..moving_enemy(0.)
    };
    let mut t = Timeline::new(sequence("check:enemy_shielding,hold:a:4"), target);
    assert!(!t.inputs_only());
    assert!(!t.needs_ground());
    assert_eq!(
        t.sample(LedgeSettings::default(), ctx(0.), Some(target), None, 1.)
            .requested,
        MAPPER_A
    );
    assert!(t.inputs_only());
    for observation in [
        None,
        Some(Opponent {
            id: target.id + 1,
            ..target
        }),
    ] {
        let mut t = Timeline::new(sequence("check:enemy_shielding,hold:a:4"), target);
        assert_eq!(
            t.sample(LedgeSettings::default(), ctx(0.), observation, None, 1.)
                .requested,
            0
        );
        assert!(t.finished(), "never use the cached or a different enemy");
    }
    let mut t = timeline("check:enemy_shielding:3,hold:a:4");
    assert_eq!(sample(&mut t, ctx(0.)).requested, 0);
    assert_eq!(sample(&mut t, ctx(1.)).requested, 0);
    // A new observation may satisfy a predicate in the same native frame.
    assert_eq!(
        t.sample(LedgeSettings::default(), ctx(1.), Some(target), None, 1.)
            .requested,
        MAPPER_A
    );
    let mut t = timeline("check:enemy_shielding:2,hold:a:4");
    for frame in 0..=2 {
        assert_eq!(sample(&mut t, ctx(frame as f32)).requested, 0);
    }
    assert!(t.finished());
    let mut t = timeline("check:self_grounded,hold:a:4");
    assert!(t.inputs_only());
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                situation: crate::native_states::situation::GROUND,
                ..ctx(0.)
            }
        )
        .requested,
        MAPPER_A
    );
}
#[test]
fn condition_flags_distance_and_facing_use_only_copied_facts() {
    let own = GameplayContext {
        position: [0., 0.],
        facing: 1.,
        situation: crate::native_states::situation::GROUND,
        ..ctx(0.)
    };
    let target = Opponent {
        position: [10., 0.],
        situation: crate::native_states::situation::AIR,
        status: crate::native_states::GUARD,
        ..moving_enemy(0.)
    };
    for (condition, expected) in [
        (Condition::SelfGrounded, true),
        (Condition::SelfAirborne, false),
        (Condition::EnemyGrounded, false),
        (Condition::EnemyAirborne, true),
        (Condition::EnemyShielding, true),
        (Condition::EnemyNotShielding, false),
        (Condition::EnemyInFront, true),
        (Condition::EnemyBehind, false),
        (Condition::EnemyWithin, true),
        (Condition::EnemyOutside, true),
    ] {
        assert_eq!(condition_met(condition, own, Some(target), 10.), expected);
    }
    assert!(condition_met(
        Condition::EnemyBehind,
        GameplayContext { facing: -1., ..own },
        Some(target),
        0.
    ));
    assert!(!condition_met(
        Condition::EnemyShielding,
        own,
        Some(Opponent {
            status: crate::native_states::GUARD_OFF,
            ..target
        }),
        0.
    ));
    assert!(!condition_met(
        Condition::EnemyWithin,
        own,
        Some(target),
        f32::NAN
    ));
    assert!(timeline("walk:inward:4:50%").needs_ground());
    assert!(timeline("run:inward:4").needs_ground());
    assert!(!timeline("drift:inward:50%:4").needs_ground());
}
#[test]
fn fighter_range_uses_the_selected_fighter_profile() {
    let profiles = crate::fighter_profiles::FighterProfiles::default();
    for (kind, inside, outside) in [(6, 10.0, 13.0)] {
        let mut settings = LedgeSettings::default();
        settings.fighter = profiles.profile(kind).unwrap().ledge;
        for (distance, expected) in [(inside, MAPPER_R), (outside, 0)] {
            let opponent = Opponent {
                position: [distance, 0.0],
                ..enemy(CLIFF_CLIMB, true)
            };
            let mut t = Timeline::new(sequence("range:fighter_range,face:opponent,grab"), opponent);
            assert_eq!(
                t.sample(settings, ctx(0.0), Some(opponent), None, 0.0)
                    .requested,
                expected
            );
        }
        let opponent = Opponent {
            position: [inside, 0.0],
            ..enemy(CLIFF_CLIMB, true)
        };
        let mut t = Timeline::new(sequence("range:9,grab"), opponent);
        assert_eq!(
            t.sample(settings, ctx(0.0), Some(opponent), None, 0.0)
                .requested,
            0
        );
    }
}
#[test]
fn instant_grab_range_passes_now_or_cancels_without_a_late_grab() {
    let profiles = crate::fighter_profiles::FighterProfiles::default();
    for kind in [6] {
        let mut settings = LedgeSettings::default();
        settings.fighter = profiles.profile(kind).unwrap().ledge;
        let reach = settings.fighter.grab.standing_range;
        for actions in [
            "range:fighter_range_instant,grab",
            "grab:fighter_range_instant",
        ] {
            for distance in [reach, reach + 1.] {
                let opponent = Opponent {
                    position: [distance, 0.],
                    ..enemy(CLIFF_CLIMB, true)
                };
                let mut t = Timeline::new(sequence(actions), opponent);
                let out = t.sample(settings, ctx(0.), Some(opponent), None, 0.);
                if distance <= reach {
                    assert_eq!(
                        out.requested, MAPPER_R,
                        "in-range grab on first observation"
                    );
                    assert!(!t.finished());
                } else {
                    assert_eq!(out.requested, 0);
                    assert!(t.finished(), "out of range cancels on first observation");
                    let near = Opponent {
                        position: [1., 0.],
                        ..opponent
                    };
                    assert_eq!(
                        t.sample(settings, ctx(1.), Some(near), None, 0.).requested,
                        0
                    );
                }
            }
            let opponent = enemy(CLIFF_CLIMB, true);
            for observed in [
                None,
                Some(Opponent {
                    id: opponent.id + 1,
                    ..opponent
                }),
            ] {
                let mut t = Timeline::new(sequence(actions), opponent);
                assert_eq!(t.sample(settings, ctx(0.), observed, None, 0.).requested, 0);
                assert!(t.finished(), "missing current target cancels");
            }
        }
        let far = Opponent {
            position: [reach + 1., 0.],
            ..enemy(CLIFF_CLIMB, true)
        };
        let mut waiting = Timeline::new(sequence("range:fighter_range,grab"), far);
        assert_eq!(
            waiting
                .sample(settings, ctx(0.), Some(far), None, 0.)
                .requested,
            0
        );
        assert!(!waiting.finished(), "ledge range wait is preserved");
        let near = Opponent {
            position: [reach, 0.],
            ..far
        };
        assert_eq!(
            waiting
                .sample(settings, ctx(1.), Some(near), None, 0.)
                .requested,
            MAPPER_R
        );
    }
}
#[test]
fn instant_grab_range_fails_when_fighter_reference_is_unavailable() {
    let profiles = crate::fighter_profiles::FighterProfiles::default();
    let mut settings = LedgeSettings::default();
    settings.fighter = profiles.profile(5).unwrap().ledge;
    assert_eq!(settings.fighter.grab.standing_range, 0.0);
    for actions in [
        "range:fighter_range,grab",
        "range:fighter_range_instant,grab",
        "grab:fighter_range_instant",
    ] {
        let opponent = Opponent {
            position: [0.0, 0.0],
            ..enemy(CLIFF_CLIMB, true)
        };
        let mut timeline = Timeline::new(sequence(actions), opponent);
        let output = timeline.sample(settings, ctx(0.0), Some(opponent), None, 0.0);
        assert_eq!(output.requested, 0);
        assert!(timeline.finished());
    }
}
#[test]
fn movement_rate_changes_do_not_cancel_forward_or_reverse_walks_or_drift() {
    for facing in [-1.0, 1.0] {
        for direction in [-1.0, 1.0] {
            for strength in [0.2, 0.28, 0.3, 0.5, 0.79, 0.95] {
                for action in [
                    format!("walk:opponent:8:{strength}"),
                    format!("walk_to_opponent:{strength}:3:0"),
                    format!("drift:opponent:{strength}:3"),
                    "run:opponent:8".to_owned(),
                ] {
                    let opponent = Opponent {
                        position: [direction * 100.0, 0.0],
                        ..enemy(CLIFF_CLIMB, true)
                    };
                    let mut t = Timeline::new(sequence(&format!("{action},grab")), opponent);
                    let mut animation_frame = 0.0;
                    for (tick, rate) in [0.25, 0.5, 2.0, 1.0].into_iter().enumerate() {
                        if tick > 0 {
                            animation_frame += rate;
                        }
                        let c = GameplayContext {
                            facing,
                            status: if action.starts_with("run:") { 4 } else { 1 },
                            position: [direction * if tick == 3 { 8.0 } else { tick as f32 }, 0.0],
                            motion_kind: crate::timing::motion_hash("walk_middle"),
                            motion_frame: animation_frame,
                            motion_rate: rate,
                            ..ctx(tick as f32)
                        };
                        let out = t.sample(LedgeSettings::default(), c, Some(opponent), None, 0.0);
                        if tick < 3 {
                            assert!(
                                !t.finished(),
                                "{action}: facing {facing}, direction {direction}, tick {tick}"
                            );
                            let expected = if action.starts_with("run:") {
                                1.0
                            } else {
                                strength
                            };
                            assert_eq!(out.stick, Some([direction * expected, 0.0]));
                        } else {
                            assert_eq!(out.requested, MAPPER_R, "{action}");
                        }
                        for _ in 0..5 {
                            assert_eq!(
                                t.sample(LedgeSettings::default(), c, Some(opponent), None, 0.0),
                                out
                            );
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn locomotion_clock_still_rejects_real_gaps_and_does_not_change_attack_clock() {
    for status in [crate::native_states::status::WALK, crate::native_states::status::RUN, crate::native_states::status::REBOUND_STOP] {
        let mut t = timeline("drift:opponent:0.5:8,grab");
        sample(
            &mut t,
            GameplayContext {
                status,
                motion_rate: 0.25,
                ..ctx(0.0)
            },
        );
        // Two updates at the new rate, not a single rate-adjusted update.
        let out = sample(
            &mut t,
            GameplayContext {
                status,
                motion_rate: 2.0,
                ..ctx(4.0)
            },
        );
        assert!(t.finished());
        assert_eq!(out.requested, 0);
    }
    // Attack clocks retain the existing missed-frame behavior, even when
    // an animation frame delta happens to match a changed rate.
    let mut t = timeline("wait:8,grab");
    sample(
        &mut t,
        GameplayContext {
            status: crate::native_states::status::REBOUND_STOP,
            motion_rate: 0.25,
            ..ctx(0.0)
        },
    );
    sample(
        &mut t,
        GameplayContext {
            status: crate::native_states::status::REBOUND_STOP,
            motion_rate: 2.0,
            ..ctx(2.0)
        },
    );
    assert!(t.finished());
}
#[test]
fn file_static_pose_frame_changes_do_not_fabricate_game_frames() {
    let mut t = timeline("wait:1,up_b");
    let own = GameplayContext {
        motion_rate: 0.0,
        ..ctx(5.0)
    };
    assert_eq!(sample(&mut t, own).requested, 0);
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                motion_frame: 3.0,
                ..own
            }
        )
        .requested,
        0
    );
    assert!(!t.finished());
}

#[test]
fn file_zero_rate_guard_resumes_from_a_block_without_an_extra_timer_frame() {
    let mut t = timeline("shield_until_block:30,grab");
    let guard = GameplayContext {
        status: crate::native_states::GUARD,
        motion_rate: 0.0,
        ..ctx(0.0)
    };
    assert_eq!(sample(&mut t, guard).requested, MAPPER_ZR);
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                status: crate::native_states::GUARD_DAMAGE,
                stopped: true,
                ..guard
            }
        )
        .requested,
        MAPPER_ZR
    );
    assert_eq!(sample(&mut t, guard).requested, MAPPER_R | MAPPER_ZR);
    assert_eq!(
        sample(&mut t, guard).requested,
        MAPPER_R | MAPPER_ZR,
        "same input holds until a proven native frame advances"
    );
}

#[test]
fn file_zero_rate_pose_uses_locked_native_actor_clock_without_counting_polls() {
    let mut t = timeline("wait:2,up_b");
    for (frame, expected) in [0, 0, MAPPER_B, 0].into_iter().enumerate() {
        let own = GameplayContext {
            motion_rate: 0.0,
            ..ctx(0.0)
        };
        let e = Some(moving_enemy(frame as f32));
        let out = t.sample(LedgeSettings::default(), own, e, None, 1.0);
        assert_eq!(out.requested, expected, "frame {frame}");
        for _ in 0..30 {
            assert_eq!(t.sample(LedgeSettings::default(), own, e, None, 1.0), out);
        }
    }
}
#[test]
fn file_native_clocks_are_not_added_and_missing_samples_are_not_backfilled() {
    let mut t = timeline("wait:3,up_b");
    for frame in 0..4 {
        let out = t.sample(
            LedgeSettings::default(),
            ctx(frame as f32),
            Some(moving_enemy(frame as f32)),
            None,
            1.0,
        );
        assert_eq!(out.requested, if frame == 3 { MAPPER_B } else { 0 });
    }
    let mut t = timeline("wait:1,up_b");
    let own = GameplayContext {
        motion_rate: 0.0,
        ..ctx(0.0)
    };
    t.sample(
        LedgeSettings::default(),
        own,
        Some(moving_enemy(0.0)),
        None,
        1.0,
    );
    assert_eq!(
        t.sample(LedgeSettings::default(), own, None, None, 1.0)
            .requested,
        0
    );
    assert_eq!(
        t.sample(
            LedgeSettings::default(),
            own,
            Some(moving_enemy(10.0)),
            None,
            1.0
        )
        .requested,
        0
    );
    assert_eq!(
        t.sample(
            LedgeSettings::default(),
            own,
            Some(moving_enemy(11.0)),
            None,
            1.0
        )
        .requested,
        MAPPER_B
    );
}
#[test]
fn file_fractional_motion_rate_roundoff_does_not_add_a_wait_frame() {
    let mut t = timeline("wait:5,up_b,wait:2,short_hop_nair");
    let mut motion = 0.0;
    for frame in 0..=10 {
        let out = sample(
            &mut t,
            GameplayContext {
                motion_rate: 0.1,
                motion_frame: motion,
                ..ctx(0.0)
            },
        );
        assert_eq!(
            out.requested,
            match frame {
                5 => MAPPER_B,
                8 => MAPPER_X | MAPPER_A,
                _ => 0,
            },
            "frame {frame}"
        );
        motion += 0.1;
    }
}

#[test]
fn file_observation_demand_drops_after_the_final_spatial_step() {
    let mut t = timeline("face:opponent,shield_until_block:5,grab");
    assert!(!t.inputs_only());
    assert_eq!(sample(&mut t, ctx(0.0)).requested, MAPPER_ZR);
    assert!(
        t.inputs_only(),
        "block/inputs use own status only, without opponent or ground probes"
    );
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                safe_ground: [false; 2],
                ..ctx(1.0)
            }
        )
        .requested,
        MAPPER_ZR
    );
}

#[test]
fn file_shield_until_block_waits_through_hitstop_and_stun_then_sends_grab() {
    let mut t = timeline("shield_until_block:5, grab");
    assert_eq!(sample(&mut t, ctx(0.0)).requested, MAPPER_ZR);
    let impact = GameplayContext {
        status: crate::native_states::GUARD_DAMAGE,
        stopped: true,
        ..ctx(1.0)
    };
    for _ in 0..20 {
        assert_eq!(sample(&mut t, impact).requested, MAPPER_ZR);
    }
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                stopped: false,
                ..impact
            }
        )
        .requested,
        MAPPER_ZR
    );
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                status: crate::native_states::GUARD,
                ..ctx(0.0)
            }
        )
        .requested,
        MAPPER_R | MAPPER_ZR
    );
    let mut t = timeline("shield_until_block:2, grab");
    for n in 0..=2 {
        assert_ne!(sample(&mut t, ctx(n as f32)).requested, MAPPER_R);
    }
    assert!(t.finished());
}

#[test]
fn blocked_getup_attack_cancels_shield_into_up_b_on_the_first_guard_frame() {
    for action in [
        "up_b:1",
        "up_special:4",
        "input:b:0:1:4",
        "facing_input:b:0:1:4",
        "opponent_input:b:0:1:4",
        "controller_input:b:0:1:0:0:4",
        "tap:b:0:1:1:1:4",
    ] {
        let opponent = enemy(CLIFF_CLIMB, false);
        let mut t = Timeline::new(
            sequence(&format!("shield_until_block:30,{action}")),
            opponent,
        );
        let call =
            |t: &mut Timeline, c| t.sample(LedgeSettings::default(), c, Some(opponent), None, 1.);
        assert_eq!(call(&mut t, ctx(0.)).requested, MAPPER_ZR);
        let hit = GameplayContext {
            status: crate::native_states::GUARD_DAMAGE,
            stopped: true,
            ..ctx(1.)
        };
        for _ in 0..8 {
            assert_eq!(call(&mut t, hit).requested, MAPPER_ZR);
        }
        assert_eq!(
            call(
                &mut t,
                GameplayContext {
                    stopped: false,
                    ..hit
                }
            )
            .requested,
            MAPPER_ZR
        );
        let ready = GameplayContext {
            status: crate::native_states::GUARD,
            ..ctx(0.)
        };
        let out = call(&mut t, ready);
        assert_eq!(
            out.requested,
            MAPPER_B | MAPPER_ZR,
            "{action}: keep guard through direct OOS Up-B"
        );
        assert_eq!(out.stick, Some([0., 1.]));
        assert_eq!(
            call(&mut t, ready),
            out,
            "duplicate polls must not restart the input"
        );
        // A native status change with no clock advance must not retain a
        // synthetic shield bit once the special has started or is airborne.
        let started = GameplayContext {
            status: crate::native_states::status::SPECIAL_HI,
            situation: crate::native_states::situation::AIR,
            motion_kind: crate::timing::motion_hash("special_hi"),
            ..ctx(0.)
        };
        let after = call(&mut t, started);
        assert_eq!(
            after.requested & MAPPER_ZR,
            0,
            "{action}: no guard leak after acceptance"
        );
        assert!(
            matches!(after.requested, 0 | MAPPER_B),
            "{action}: retain only the configured pulse/gap"
        );
    }
}

#[test]
fn direct_up_b_guard_retention_is_grounded_status_scoped_and_keeps_literal_shields() {
    for status in [
        crate::native_states::GUARD_ON,
        crate::native_states::GUARD,
        crate::native_states::status::WAIT,
        crate::native_states::status::SPECIAL_HI,
    ] {
        let mut t = timeline("up_b:4");
        assert_eq!(
            sample(&mut t, GameplayContext { status, ..ctx(0.) }).requested,
            MAPPER_B
                | if matches!(
                    status,
                    crate::native_states::GUARD_ON | crate::native_states::GUARD
                ) {
                    MAPPER_ZR
                } else {
                    0
                }
        );
    }
    // Other specials still use their existing shield-drop path.
    for actions in ["side_special:1", "down_special:1", "neutral_special:1"] {
        let mut t = timeline(actions);
        assert_eq!(
            sample(
                &mut t,
                GameplayContext {
                    status: crate::native_states::GUARD,
                    ..ctx(0.)
                }
            )
            .requested,
            0
        );
    }
    // Explicitly authored guard is not the adapter's temporary guard bit.
    let mut t = timeline("input:b+zr:0:1:4");
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                status: crate::native_states::status::SPECIAL_HI,
                situation: crate::native_states::situation::AIR,
                ..ctx(0.)
            }
        )
        .requested,
        MAPPER_B | MAPPER_ZR
    );
}

#[test]
fn timeline_online_status_transitions_and_idle_wraps_preserve_schedule() {
    let mut t = timeline("up_special:2, wait:1, jab");
    assert_eq!(sample(&mut t, ctx(0.0)).requested, MAPPER_B);
    let startup = GameplayContext {
        status: crate::native_states::status::SPECIAL_HI,
        motion_kind: crate::timing::motion_hash("special_hi"),
        can_control: false,
        can_act: false,
        ..ctx(0.0)
    };
    assert_eq!(sample(&mut t, startup).requested, MAPPER_B);
    for _ in 0..50 {
        assert_eq!(sample(&mut t, startup).requested, MAPPER_B);
    }
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                motion_frame: 1.0,
                ..startup
            }
        )
        .requested,
        0
    );
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                status: crate::native_states::fighter_status::KIRBY_SPECIAL_HI2,
                motion_kind: crate::timing::motion_hash("special_hi2"),
                ..startup
            }
        )
        .requested,
        MAPPER_A
    );
    let mut t = timeline("wait:2, up_special");
    assert_eq!(sample(&mut t, ctx(8.0)).requested, 0);
    assert_eq!(sample(&mut t, ctx(9.0)).requested, 0);
    assert_eq!(sample(&mut t, ctx(0.0)).requested, MAPPER_B);
    assert_eq!(sample(&mut t, ctx(0.0)).requested, MAPPER_B);
}
#[test]
fn timeline_missing_file_recipe_never_dispatches_a_compiled_option() {
    let mut trap = LedgeTrap::default();
    let settings = LedgeSettings {
        enabled: true,
        ..LedgeSettings::default()
    };
    for frame in 0..5 {
        assert_eq!(
            trap.sample(
                settings,
                frame,
                ctx(frame as f32),
                Some(enemy(CLIFF_CLIMB, false))
            )
            .requested,
            0
        );
    }
    assert!(!trap.file_timeline_active());
}

#[test]
fn timeline_spatial_timeout_cancels_without_starting_a_late_attack() {
    let mut s = sequence("face:backward, up_special");
    s.spatial_timeout = 2;
    let mut t = Timeline::new(s, enemy(CLIFF_CLIMB, false));
    for frame in 0..=2 {
        assert_eq!(sample(&mut t, ctx(frame as f32)).requested, 0);
    }
    assert!(t.finished());
}
#[test]
fn backward_drift_keeps_entry_direction_through_turnaround_and_exact_duration() {
    for facing in [-1.0, 1.0] {
        for situation in [crate::native_states::situation::GROUND, crate::native_states::situation::AIR] {
            let mut t = timeline("drift:backward:0.5:3,grab");
            for frame in 0..3 {
                let c = GameplayContext {
                    facing: if frame == 0 { facing } else { -facing },
                    situation,
                    ..ctx(frame as f32)
                };
                assert_eq!(sample(&mut t, c).stick, Some([-facing * 0.5, 0.]));
                assert_eq!(sample(&mut t, c).stick, Some([-facing * 0.5, 0.]));
            }
            assert_eq!(
                sample(
                    &mut t,
                    GameplayContext {
                        facing: -facing,
                        situation,
                        ..ctx(3.)
                    }
                )
                .requested,
                MAPPER_R
            );
        }
    }
}
#[test]
fn backward_drift_recaptures_facing_for_each_step_and_leaves_facing_drift_dynamic() {
    let mut t = timeline("drift:backward:0.25:1,wait:1,drift:backward:0.75:1,grab");
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                facing: 1.,
                ..ctx(0.)
            }
        )
        .stick,
        Some([-0.25, 0.])
    );
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                facing: -1.,
                ..ctx(1.)
            }
        )
        .stick,
        Some([0., 0.])
    );
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                facing: -1.,
                ..ctx(2.)
            }
        )
        .stick,
        Some([0.75, 0.])
    );
    assert_eq!(sample(&mut t, ctx(3.)).requested, MAPPER_R);
    let mut t = timeline("drift:facing:0.5:3");
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                facing: 1.,
                ..ctx(0.)
            }
        )
        .stick,
        Some([0.5, 0.])
    );
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                facing: -1.,
                ..ctx(1.)
            }
        )
        .stick,
        Some([-0.5, 0.])
    );
}
#[test]
fn timeline_canonical_move_direction_is_captured_at_entry() {
    let mut t = timeline("fair:2");
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                facing: -1.0,
                ..ctx(0.0)
            }
        )
        .stick,
        Some([-1.0, 0.0])
    );
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                facing: 1.0,
                ..ctx(1.0)
            }
        )
        .stick,
        Some([-1.0, 0.0])
    );
}

#[test]
fn timeline_inputs_are_immediate_even_without_readiness_or_ground() {
    let mut t = timeline("up_special:2, release, down_special");
    for frame in 0..=1 {
        let out = sample(
            &mut t,
            GameplayContext {
                situation: crate::native_states::situation::AIR,
                status: crate::native_states::status::FALL,
                can_act: false,
                can_control: false,
                can_jump: false,
                can_air_attack: false,
                safe_ground: [false; 2],
                ..ctx(frame as f32)
            },
        );
        assert_eq!(out.requested, MAPPER_B);
        assert_eq!(out.stick, Some([0.0, 1.0]));
    }
    assert_eq!(sample(&mut t, ctx(2.0)).requested, 0);
    assert_eq!(sample(&mut t, ctx(3.0)).stick, Some([0.0, -1.0]));
    assert_eq!(sample(&mut t, ctx(4.0)).requested, 0);
    assert!(t.finished());
}
#[test]
fn timeline_hop_wait_up_b_and_multiple_attacks_keep_exact_boundaries() {
    let mut t = timeline("short_hop, wait:2, up_special, wait:1, jab:2");
    let expected = [MAPPER_X, 0, 0, MAPPER_B, 0, MAPPER_A, MAPPER_A, 0];
    for (frame, buttons) in expected.into_iter().enumerate() {
        assert_eq!(
            sample(&mut t, ctx(frame as f32)).requested,
            buttons,
            "frame {frame}"
        );
    }
    assert!(t.finished());
}
#[test]
fn timeline_online_delay_and_repeated_polls_do_not_change_frame_schedule() {
    for delay in [0.0, 2.0, 6.0, 12.0] {
        let mut t = timeline("wait:2, up_special:2, release, jab");
        for (frame, expected) in [0, 0, MAPPER_B, MAPPER_B, 0, MAPPER_A, 0]
            .into_iter()
            .enumerate()
        {
            let c = GameplayContext {
                input_delay: delay,
                ..ctx(frame as f32)
            };
            let out = sample(&mut t, c);
            assert_eq!(out.requested, expected, "delay {delay}, frame {frame}");
            for _ in 0..40 {
                assert_eq!(sample(&mut t, c), out);
            }
        }
    }
}
#[test]
fn timeline_hitstop_and_motion_rates_pause_without_losing_button_holds() {
    let mut t = timeline("up_special:2, jab");
    let c = GameplayContext {
        motion_rate: 0.5,
        ..ctx(0.0)
    };
    let held = sample(&mut t, c);
    for _ in 0..20 {
        assert_eq!(sample(&mut t, GameplayContext { stopped: true, ..c }), held);
    }
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                motion_frame: 0.5,
                ..c
            }
        ),
        held
    );
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                motion_frame: 1.0,
                ..c
            }
        )
        .requested,
        MAPPER_A
    );
}
#[test]
fn timeline_clock_gaps_invalid_data_damage_and_release_cancel_without_bursting() {
    for changed in [
        ctx(4.0),
        GameplayContext {
            motion_frame: f32::NAN,
            ..ctx(1.0)
        },
        GameplayContext {
            status: crate::native_states::status::DAMAGE_FLY,
            ..ctx(1.0)
        },
        GameplayContext {
            held: false,
            ..ctx(1.0)
        },
    ] {
        let mut t = timeline("wait:2, up_special, jab");
        sample(&mut t, ctx(0.0));
        assert_eq!(sample(&mut t, changed).requested, 0);
        assert!(t.finished());
    }
}
#[test]
fn timeline_start_delay_and_repeated_buttons_have_explicit_release_frames() {
    let mut s = sequence("jab, release, jab");
    s.start_delay = 2;
    let mut t = Timeline::new(s, enemy(CLIFF_CLIMB, false));
    for (frame, expected) in [0, 0, MAPPER_A, 0, MAPPER_A, 0].into_iter().enumerate() {
        assert_eq!(sample(&mut t, ctx(frame as f32)).requested, expected);
    }
}
#[test]
fn timeline_face_captures_direction_and_continues_on_same_confirmed_update() {
    let mut t = timeline("face:backward, jab");
    assert_eq!(sample(&mut t, ctx(0.0)).stick, Some([-0.3, 0.0]));
    let out = sample(
        &mut t,
        GameplayContext {
            facing: -1.0,
            ..ctx(1.0)
        },
    );
    assert_eq!(out.requested, MAPPER_A, "no extra confirmation delay");
    assert_eq!(out.stick, Some([0.0, 0.0]));
}
#[test]
fn timeline_walk_then_wait_preserves_order_and_stops_at_explicit_gap() {
    let target = Opponent {
        position: [10.0, 0.0],
        ..enemy(CLIFF_CLIMB, false)
    };
    let mut t = Timeline::new(
        sequence("walk:opponent:8:0.8:4, wait:2, up_special"),
        target,
    );
    assert_eq!(sample(&mut t, ctx(0.0)).stick, Some([0.8, 0.0]));
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                position: [6.0, 0.0],
                ..ctx(1.0)
            }
        )
        .requested,
        0
    );
    assert_eq!(sample(&mut t, ctx(2.0)).requested, 0);
    assert_eq!(sample(&mut t, ctx(3.0)).requested, MAPPER_B);
}
#[test]
fn timeline_cstick_has_real_direction_bits_axes_and_duplicate_reuse() {
    let mut t = timeline("cstick:-1:0:1, wait:1");
    let out = sample(&mut t, ctx(0.0));
    assert_eq!(out.requested, MAPPER_RIGHT_STICK_LEFT);
    assert_eq!(t.right_stick(), [-1.0, 0.0]);
    assert_eq!(sample(&mut t, ctx(0.0)), out);
    assert_eq!(t.right_stick(), [-1.0, 0.0]);
    sample(&mut t, ctx(1.0));
    assert_eq!(t.right_stick(), [0.0; 2]);
}
#[test]
fn timeline_dispatch_survives_takeoff_target_loss_and_hit_confirm_then_hands_back() {
    let mut trap = LedgeTrap::default();
    let mut settings = LedgeSettings {
        enabled: true,
        manual_spacing: true,
        ..LedgeSettings::default()
    };
    settings.sequence_overrides[0] = Some(sequence("short_hop, wait:2, up_special"));
    assert_eq!(
        trap.sample(settings, 0, ctx(0.0), Some(enemy(CLIFF_CLIMB, false)))
            .requested,
        MAPPER_X
    );
    for frame in 1..=2 {
        let out = trap.sample(
            settings,
            100 * frame,
            GameplayContext {
                situation: crate::native_states::situation::AIR,
                can_control: false,
                can_act: false,
                hit_confirmed: true,
                ..ctx(frame as f32)
            },
            None,
        );
        assert_eq!(out.requested, 0);
        assert_eq!(out.owned, SEQUENCE_BUTTONS, "wait still owns controls");
    }
    assert_eq!(
        trap.sample(
            settings,
            999,
            GameplayContext {
                situation: crate::native_states::situation::AIR,
                can_control: false,
                can_act: false,
                ..ctx(3.0)
            },
            None
        )
        .requested,
        MAPPER_B
    );
    assert_eq!(trap.sample(settings, 1000, ctx(4.0), None).requested, 0);
    assert!(trap.manual_after_response());
    assert_eq!(
        trap.sample(settings, 1001, ctx(5.0), None),
        ControllerOutput::default()
    );
}
#[test]
fn timeline_groundless_raw_dash_does_not_wait_for_native_dash_or_retry() {
    let mut t = timeline("dash_attack:2:1");
    for frame in 0..=1 {
        let out = sample(
            &mut t,
            GameplayContext {
                can_act: false,
                safe_ground: [false; 2],
                ..ctx(frame as f32)
            },
        );
        assert_eq!(out.requested, 0);
        assert_eq!(out.stick, Some([1.0, 0.0]));
    }
    assert_eq!(sample(&mut t, ctx(2.0)).requested, MAPPER_A);
    assert_eq!(sample(&mut t, ctx(3.0)).requested, 0);
}
#[cfg(feature = "diagnostic-logging")]
#[test]
fn diagnostic_finish_reason_distinguishes_completion_from_missed_native_updates() {
    let mut done = timeline("wait:1,jab");
    for frame in 0..=2 {
        sample(&mut done, ctx(frame as f32));
    }
    assert_eq!(done.end_reason, Some("completed"));
    let mut gap = timeline("wait:8,jab");
    sample(&mut gap, ctx(0.0));
    sample(&mut gap, ctx(3.0));
    assert_eq!(gap.end_reason, Some("native_clock_gap"));
}
#[test]
fn held_a_survives_internal_observation_gaps_without_retriggering() {
    use crate::native_input::{ButtonTrace, MapperButtonOverlay};
    let mut t = timeline("hold:a:60");
    let mut overlay = MapperButtonOverlay::default();
    let first = sample(&mut t, ctx(0.0));
    let edge = overlay.apply_raw(ButtonTrace::default(), first.requested, first.owned);
    assert_eq!(edge.pressed & MAPPER_A, MAPPER_A);
    for frame in [
        1., 4., 7., 8., 12., 16., 20., 24., 28., 32., 36., 40., 44., 48., 52., 56., 59.,
    ] {
        let out = sample(&mut t, ctx(frame));
        assert!(!t.finished());
        let mapped = overlay.apply_raw(ButtonTrace::default(), out.requested, out.owned);
        assert_eq!(mapped.held & MAPPER_A, MAPPER_A);
        assert_eq!(mapped.pressed & MAPPER_A, 0);
        assert_eq!(mapped.released & MAPPER_A, 0);
    }
    let out = sample(&mut t, ctx(60.));
    let mapped = overlay.apply_raw(ButtonTrace::default(), out.requested, out.owned);
    assert_eq!(mapped.held & MAPPER_A, 0);
    assert_eq!(mapped.released & MAPPER_A, MAPPER_A);
    assert!(t.finished());
}

#[test]
fn hold_gap_never_skips_release_or_delayed_press() {
    for recipe in ["hold:a:3,release,hold:a:8", "wait:2,hold:a:60"] {
        let mut t = timeline(recipe);
        sample(&mut t, ctx(0.));
        assert_eq!(sample(&mut t, ctx(4.)).requested, 0);
        assert!(t.finished());
    }
    let mut t = timeline("hold:a:60");
    sample(&mut t, ctx(0.));
    assert_eq!(sample(&mut t, ctx(9.)).requested, 0);
    assert!(t.finished(), "invalid native progress still aborts");
}
#[test]
fn facing_blocks_pick_only_starting_direction_and_add_no_delay() {
    let recipe =
        "if_facing_ledge, input:a:0:0:1, end_facing, if_back_to_ledge, input:b:0:0:1, end_facing";
    for side in [-1.0, 1.0] {
        for toward in [false, true] {
            let mut t = timeline(recipe);
            let mut own = ctx(0.0);
            own.position = [side * 10.0, 0.0];
            own.facing = if toward { -side } else { side };
            let first = t.sample(LedgeSettings::default(), own, None, Some([0.0; 2]), side);
            assert_eq!(first.requested, if toward { MAPPER_A } else { MAPPER_B });
            own.motion_frame = 1.0;
            own.facing = -own.facing;
            let second = t.sample(LedgeSettings::default(), own, None, Some([0.0; 2]), side);
            assert_eq!(
                second.requested, 0,
                "Turning must not enable the other block"
            );
            assert!(t.finished());
        }
    }
}
#[test]
fn skipped_block_waits_do_not_consume_time_and_missing_anchor_stops() {
    let mut t =
        timeline("if_facing_ledge, wait:100, a:4, end_facing, if_back_to_ledge, b:4, end_facing");
    let mut own = ctx(0.0);
    own.position = [10.0, 0.0];
    own.facing = 1.0;
    assert_eq!(
        t.sample(LedgeSettings::default(), own, None, Some([0.0; 2]), 1.0)
            .requested,
        MAPPER_B
    );
    let mut missing = timeline("if_back_to_ledge, a:4, end_facing");
    assert_eq!(sample(&mut missing, own).requested, 0);
    assert!(missing.finished());
}
#[test]
fn explicit_saved_ledge_directions_do_not_fall_back_to_opponent() {
    for token in [
        "drift:saved_ledge:0.25:4",
        "walk:away_from_ledge:4:0.3",
        "run:saved_ledge:4",
        "face:saved_ledge",
    ] {
        let mut t = timeline(token);
        assert_eq!(sample(&mut t, ctx(0.0)).requested, 0);
        assert!(t.finished(), "{token}");
    }
    let mut own = ctx(0.0);
    own.position = [10.0, 0.0];
    own.facing = 1.0;
    let mut toward = timeline("drift:saved_ledge:0.25:4");
    assert_eq!(
        toward
            .sample(LedgeSettings::default(), own, None, Some([0.0; 2]), 1.0)
            .stick,
        Some([-0.25, 0.0])
    );
    let mut away = timeline("drift:away_from_ledge:0.25:4");
    assert_eq!(
        away.sample(LedgeSettings::default(), own, None, Some([0.0; 2]), 1.0)
            .stick,
        Some([0.25, 0.0])
    );
}
