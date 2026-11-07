use kirby_assist_core::{
    gameplay::ControllerOutput,
    manual_controls::{ManualOverlay, TurboOutput},
    native_input::{ButtonTrace, MAPPER_A, MAPPER_B, MAPPER_ZR},
    spot_dodge::{contact, Context, SpotDodge, Target, ESCAPE},
};
fn context(frame: f32) -> Context {
    Context {
        object: 1,
        status: kirby_assist_core::native_states::status::WAIT,
        situation: kirby_assist_core::native_states::situation::GROUND,
        motion: 1,
        frame,
        rate: 1.0,
        stopped: false,
        can_spot_dodge: true,
        position: [0.0; 2],
        velocity: [0.0; 2],
    }
}
fn target() -> Target {
    Target {
        id: 2,
        status: kirby_assist_core::native_states::CATCH,
        motion: 2,
        frame: 2.0,
        rate: 1.0,
        position: [10.0, 0.0],
        velocity: [0.0; 2],
        facing: -1.0,
        first: 6,
        last: 7,
        reach: 12.0,
    }
}
fn roster(t: Target) -> [Option<Target>; 8] {
    let mut out = [None; 8];
    out[0] = Some(t);
    out
}
#[test]
fn target_first_readiness_preserves_acquisition_hitstop_and_target_loss() {
    let mut reference = SpotDodge::default();
    let mut gated = SpotDodge::default();
    for tick in 0..8 {
        let targets = if matches!(tick, 1 | 2 | 5 | 6) {
            roster(target())
        } else {
            [None; 8]
        };
        let c = Context {
            stopped: tick == 2 || tick == 3,
            ..context(tick as f32)
        };
        let observed = Context {
            can_spot_dodge: targets.iter().any(Option::is_some) && c.can_spot_dodge,
            ..c
        };
        assert_eq!(
            gated.sample(true, observed, targets),
            reference.sample(true, c, targets),
            "tick={tick}"
        );
    }
}

#[test]
fn standing_dash_and_pivot_grabs_trigger_at_the_forecast_boundary() {
    for (status, first) in [
        (kirby_assist_core::native_states::CATCH, 6),
        (kirby_assist_core::native_states::CATCH_DASH, 9),
        (kirby_assist_core::native_states::CATCH_TURN, 10),
    ] {
        let mut tool = SpotDodge::default();
        let t = Target {
            status,
            first,
            last: first + 1,
            frame: f32::from(first) - 7.0,
            ..target()
        };
        assert_eq!(
            tool.sample(true, context(0.0), roster(t)),
            ControllerOutput::default()
        );
        let out = tool.sample(
            true,
            context(1.0),
            roster(Target {
                frame: t.frame + 1.0,
                ..t
            }),
        );
        assert_eq!(out.requested, MAPPER_ZR);
        assert_eq!(out.stick, Some([0.0, -1.0]));
    }
}
#[test]
fn distant_behind_high_expired_unknown_and_malformed_grabs_do_not_trigger() {
    for t in [
        Target {
            position: [50.0, 0.0],
            ..target()
        },
        Target {
            facing: 1.0,
            ..target()
        },
        Target {
            position: [10.0, 20.0],
            ..target()
        },
        Target {
            frame: 8.0,
            ..target()
        },
        Target {
            status: kirby_assist_core::native_states::status::FINAL,
            ..target()
        },
        Target {
            motion: kirby_assist_core::timing::NO_MOTION_KIND,
            ..target()
        },
        Target {
            frame: f32::NAN,
            ..target()
        },
        Target {
            reach: f32::INFINITY,
            ..target()
        },
    ] {
        assert_eq!(
            SpotDodge::default().sample(true, context(0.0), roster(t)),
            ControllerOutput::default()
        );
    }
}
#[test]
fn movement_is_projected_to_the_same_time_as_grab_activity() {
    let near = Target {
        position: [20.0, 0.0],
        velocity: [-2.0, 0.0],
        ..target()
    };
    assert!(contact(context(0.0), near).is_some());
    let leaving = Target {
        position: [10.0, 0.0],
        velocity: [3.0, 0.0],
        ..target()
    };
    assert!(contact(context(0.0), leaving).is_none());
    assert!(contact(
        Context {
            velocity: [-3.0, 0.0],
            ..context(0.0)
        },
        target()
    )
    .is_none());
}
#[test]
fn no_air_dodges_damage_recovery_cancels_or_forced_dodge_terms() {
    for c in [
        Context {
            situation: kirby_assist_core::native_states::situation::AIR,
            ..context(0.0)
        },
        Context {
            status: kirby_assist_core::native_states::status::DAMAGE,
            can_spot_dodge: false,
            ..context(0.0)
        },
        Context {
            status: kirby_assist_core::native_states::status::ATTACK,
            can_spot_dodge: false,
            ..context(0.0)
        },
        Context {
            status: kirby_assist_core::native_states::GUARD_DAMAGE,
            can_spot_dodge: false,
            ..context(0.0)
        },
        Context {
            can_spot_dodge: false,
            ..context(0.0)
        },
        Context {
            stopped: true,
            ..context(0.0)
        },
    ] {
        assert_eq!(
            SpotDodge::default().sample(true, c, roster(target())),
            ControllerOutput::default()
        );
    }
}

#[test]
fn native_cancel_term_does_not_bypass_stationary_only_grab_defense() {
    let out = SpotDodge::default().sample(
        true,
        Context {
            status: kirby_assist_core::native_states::status::ATTACK_LW3,
            can_spot_dodge: true,
            ..context(0.0)
        },
        roster(target()),
    );
    assert_eq!(out.requested, 0);
    assert_eq!(out.owned, 0);
}

#[test]
fn priority_cancellation_does_not_consume_an_unaccepted_grab_attempt() {
    let mut tool = SpotDodge::default();
    let first = tool.sample(true, context(0.0), roster(target()));
    assert_eq!(first.requested, MAPPER_ZR);
    tool.cancel_pending();
    let retry = tool.sample(
        true,
        context(1.0),
        roster(Target {
            frame: 3.0,
            ..target()
        }),
    );
    assert_eq!(retry.requested, MAPPER_ZR);
}
#[test]
fn native_acceptance_releases_and_the_same_grab_cannot_retrigger_after_roster_reordering() {
    let mut tool = SpotDodge::default();
    let t = Target {
        last: 40,
        ..target()
    };
    assert_eq!(
        tool.sample(true, context(0.0), roster(t)).requested,
        MAPPER_ZR
    );
    assert_eq!(
        tool.sample(
            true,
            Context {
                status: ESCAPE,
                motion: 3,
                ..context(0.0)
            },
            roster(t)
        )
        .requested,
        0
    );
    tool.cancel_pending();
    let mut moved = [None; 8];
    moved[5] = Some(Target { frame: 4.0, ..t });
    assert_eq!(tool.sample(true, context(1.0), moved).requested, 0);
    // A rewind alone (including rollback) cannot authorize a second dodge.
    tool.cancel_response();
    assert_eq!(
        tool.sample(true, context(2.0), roster(Target { frame: 0., ..t }))
            .requested,
        0
    );
    // A positively observed end rearms the next real grab.
    tool.observe_statuses(&[Some((t.id, 0)), None, None, None, None, None, None, None]);
    assert_eq!(
        tool.sample(true, context(3.0), roster(Target { frame: 0., ..t }))
            .requested,
        MAPPER_ZR
    );
}
#[test]
fn duplicate_polls_and_stop_do_not_spend_delivery_lifetime_or_repeat_attempts() {
    let mut tool = SpotDodge::default();
    let out = tool.sample(true, context(0.0), roster(target()));
    for _ in 0..100 {
        assert_eq!(tool.sample(true, context(0.0), roster(target())), out);
    }
    for _ in 0..100 {
        assert_eq!(
            tool.sample(
                true,
                Context {
                    stopped: true,
                    ..context(0.0)
                },
                roster(target())
            ),
            out
        );
    }
    let neutral = tool.sample(true, context(1.0), roster(target()));
    assert_eq!(neutral.requested, MAPPER_ZR);
    assert_eq!(neutral.stick, Some([0.0, -1.0]));
    assert_eq!(
        tool.sample(true, context(3.0), roster(target())).stick,
        Some([0.0, -1.0])
    );
    assert_eq!(
        tool.sample(true, context(4.0), roster(target())).requested,
        MAPPER_ZR
    );
}
#[test]
fn target_loss_disable_and_identity_change_have_bounded_behavior() {
    let mut tool = SpotDodge::default();
    tool.sample(true, context(0.0), roster(target()));
    assert_eq!(
        tool.sample(true, context(1.0), [None; 8]),
        ControllerOutput::default()
    );
    tool.sample(true, context(2.0), roster(target()));
    assert_eq!(
        tool.sample(false, context(3.0), roster(target())),
        ControllerOutput::default()
    );
    assert_eq!(
        tool.sample(
            true,
            Context {
                object: 3,
                ..context(0.0)
            },
            roster(target())
        )
        .requested,
        MAPPER_ZR
    );
}
#[test]
fn real_overlay_uses_down_guard_and_never_turns_physical_horizontal_into_a_roll() {
    let out = SpotDodge::default().sample(true, context(0.0), roster(target()));
    let mut overlay = ManualOverlay::default();
    let mapped = overlay.apply_controls(
        ButtonTrace {
            held: MAPPER_A | MAPPER_B | (1 << 21),
            ..Default::default()
        },
        [1.0, 0.0],
        TurboOutput {
            consume_dpad: false,
            stick: out.stick,
        },
        0,
        out.requested,
        out.owned,
    );
    assert_eq!(mapped.stick, [0.0, -1.0]);
    assert_eq!(
        mapped.buttons.held & (MAPPER_A | MAPPER_B | (1 << 21) | (1 << 23)),
        0
    );
    assert_eq!(mapped.buttons.held & MAPPER_ZR, MAPPER_ZR);
}
#[test]
fn constant_mode_defaults_on_and_can_be_disabled_in_config() {
    use kirby_assist_core::live_input::LiveConfig;
    assert!(LiveConfig::default().auto_spot_dodge);
    assert!(
        !LiveConfig::parse("auto_spot_dodge=false")
            .unwrap()
            .auto_spot_dodge
    );
    assert!(LiveConfig::parse("auto_spot_dodge=wrong").is_err());
    assert!(LiveConfig::parse(include_str!(
        "../../../deployment/ultimate/kirby_assist/config.txt"
    ))
    .is_ok());
}

#[test]
fn grab_clock_retries_a_delayed_dodge_while_own_idle_is_frozen() {
    let mut tool = SpotDodge::default();
    tool.sample(true, context(0.0), roster(target()));
    assert_eq!(
        tool.sample(
            true,
            context(0.0),
            roster(Target {
                frame: 5.0,
                ..target()
            })
        )
        .stick,
        Some([0.0, -1.0])
    );
    let retry = tool.sample(
        true,
        context(0.0),
        roster(Target {
            frame: 6.0,
            ..target()
        }),
    );
    assert_eq!(retry.requested, MAPPER_ZR);
    assert_eq!(
        tool.sample(
            true,
            context(0.0),
            roster(Target {
                frame: 6.0,
                ..target()
            })
        ),
        retry
    );
    assert_eq!(
        tool.sample(
            true,
            Context {
                status: ESCAPE,
                ..context(0.0)
            },
            roster(target())
        )
        .owned,
        0
    );
}

#[test]
fn fast_dash_grab_that_reaches_only_in_the_last_two_forecast_frames_is_seen() {
    let t = Target {
        first: 6,
        last: 13,
        frame: 6.0,
        position: [36.0, 0.0],
        velocity: [-4.0, 0.0],
        ..target()
    };
    assert_eq!(contact(context(1.0), t), Some(5.5));
    let out = SpotDodge::default().sample(true, context(1.0), roster(t));
    assert_eq!(out.requested, MAPPER_ZR);
    assert_eq!(out.stick, Some([0.0, -1.0]));
}

#[test]
fn stationary_shield_can_request_grab_dodge_without_releasing_shield_first() {
    for status in [
        kirby_assist_core::native_states::GUARD_ON,
        kirby_assist_core::native_states::GUARD,
    ] {
        let c = Context {
            status,
            frame: 0.0,
            rate: 0.0,
            ..context(0.0)
        };
        let mut dodge = SpotDodge::default();
        let first = dodge.sample(true, c, roster(target()));
        assert_eq!(first.requested, MAPPER_ZR);
        assert_eq!(first.stick, Some([0.0, -1.0]));
        for _ in 0..10 {
            assert_eq!(dodge.sample(true, c, roster(target())), first);
        }
        let mut t = target();
        t.frame = 5.0; // Three advancing attacker frames, while shield stays frozen.
        let release = dodge.sample(true, c, roster(t));
        assert_eq!(release.requested, MAPPER_ZR);
        assert_eq!(release.stick, Some([0.0, -1.0]));
        t.frame = 6.0;
        assert_eq!(dodge.sample(true, c, roster(t)).requested, MAPPER_ZR);
        let accepted = Context {
            status: ESCAPE,
            rate: 1.0,
            ..c
        };
        assert_eq!(dodge.sample(true, accepted, roster(t)).owned, 0);
        assert_eq!(dodge.sample(true, c, roster(t)).owned, 0); // One accepted dodge per grab.
    }
}

#[test]
fn stationary_shield_exception_does_not_allow_air_shieldstun_or_disabled_escape() {
    let shield = Context {
        status: kirby_assist_core::native_states::GUARD,
        frame: 0.0,
        rate: 0.0,
        ..context(0.0)
    };
    for c in [
        Context {
            situation: kirby_assist_core::native_states::situation::AIR,
            ..shield
        },
        Context {
            can_spot_dodge: false,
            ..shield
        },
        Context {
            status: kirby_assist_core::native_states::GUARD_DAMAGE,
            ..shield
        },
        Context {
            status: kirby_assist_core::native_states::status::WAIT,
            ..shield
        },
        Context {
            rate: f32::NAN,
            ..shield
        },
        Context {
            rate: -1.0,
            ..shield
        },
    ] {
        assert_eq!(
            SpotDodge::default().sample(true, c, roster(target())),
            ControllerOutput::default()
        );
    }
    assert_eq!(
        SpotDodge::default().sample(false, shield, roster(target())),
        ControllerOutput::default()
    );
}

#[test]
fn pending_shield_dodge_holds_through_hitstop_but_releases_on_air() {
    let c = Context {
        status: kirby_assist_core::native_states::GUARD,
        frame: 0.0,
        rate: 0.0,
        ..context(0.0)
    };
    let mut dodge = SpotDodge::default();
    let initial = dodge.sample(true, c, roster(target()));
    for _ in 0..12 {
        assert_eq!(
            dodge.sample(true, Context { stopped: true, ..c }, roster(target())),
            initial
        );
    }
    assert_eq!(
        dodge.sample(true, Context { situation: kirby_assist_core::native_states::situation::AIR, ..c }, roster(target())),
        ControllerOutput::default()
    );
}

#[test]
fn fast_grab_crossing_between_half_frame_samples_is_detected() {
    let t = Target {
        frame: 5.0,
        rate: 4.0,
        first: 6,
        last: 7,
        position: [20.0, 0.0],
        velocity: [-50.0, 0.0],
        reach: 1.0,
        ..target()
    };
    let at = contact(context(0.0), t).expect("contact between sample times");
    assert!((at - 0.34).abs() < 0.0001);
    // Horizontal and vertical overlap must occur at the same instant.
    assert!(contact(
        context(0.0),
        Target {
            position: [20.0, 40.0],
            velocity: [-50.0, -50.0],
            ..t
        }
    )
    .is_none());
}

#[test]
fn temporary_uncancelable_state_does_not_consume_an_unaccepted_dodge() {
    let mut tool = SpotDodge::default();
    assert_eq!(
        tool.sample(true, context(0.0), roster(target())).requested,
        MAPPER_ZR
    );
    assert_eq!(
        tool.sample(
            true,
            Context {
                status: kirby_assist_core::native_states::GUARD_DAMAGE,
                can_spot_dodge: false,
                ..context(1.0)
            },
            roster(Target {
                frame: 3.0,
                ..target()
            })
        )
        .owned,
        0
    );
    assert_eq!(
        tool.sample(
            true,
            context(2.0),
            roster(Target {
                frame: 4.0,
                ..target()
            })
        )
        .requested,
        MAPPER_ZR
    );
}

#[test]
fn predicted_range_change_does_not_consume_an_unaccepted_dodge() {
    let mut tool = SpotDodge::default();
    assert_eq!(
        tool.sample(true, context(0.0), roster(target())).requested,
        MAPPER_ZR
    );
    assert_eq!(
        tool.sample(
            true,
            context(1.0),
            roster(Target {
                frame: 3.0,
                position: [50.0, 0.0],
                ..target()
            })
        )
        .owned,
        0
    );
    assert_eq!(
        tool.sample(
            true,
            context(2.0),
            roster(Target {
                frame: 4.0,
                ..target()
            })
        )
        .requested,
        MAPPER_ZR
    );
}

#[test]
fn dk_grabs_use_dk_windows_and_keep_guard_held_for_bowsers_retry() {
    use kirby_assist_core::fighter_data;
    for (status, first, last) in [
        (kirby_assist_core::native_states::CATCH, 8, 10),
        (kirby_assist_core::native_states::CATCH_DASH, 11, 13),
        (kirby_assist_core::native_states::CATCH_TURN, 12, 14),
    ] {
        let facts = fighter_data::ordinary_grab(1, status).unwrap();
        assert_eq!(
            (facts.first, facts.last, facts.reach),
            (first, last, Some(19.5))
        );
        let reach = facts
            .reach
            .expect("Donkey Kong grab reach should be present");
        for own_status in [
            0,
            kirby_assist_core::native_states::GUARD_ON,
            kirby_assist_core::native_states::GUARD,
        ] {
            let mut dodge = SpotDodge::default();
            let mut c = Context {
                status: own_status,
                ..context(0.0)
            };
            if own_status != 0 {
                c.rate = 0.0;
            }
            let t = Target {
                status,
                frame: f32::from(first) - 6.0,
                first,
                last,
                reach,
                position: [19.0, 0.0],
                ..target()
            };
            for age in 0..=4 {
                let out = dodge.sample_with_shape(
                    true,
                    c,
                    roster(Target {
                        frame: t.frame + age as f32,
                        ..t
                    }),
                    7.0,
                    10.0,
                    8.0,
                );
                assert_eq!(out.requested, MAPPER_ZR, "status={own_status:x} age={age}");
                assert_eq!(out.stick, Some([0.0, -1.0]));
                // Standstill can use its own clock; held shield uses DK's clock.
                if own_status == 0 {
                    c.frame += 1.0;
                }
            }
            assert_eq!(
                dodge
                    .sample(
                        true,
                        Context {
                            status: ESCAPE,
                            rate: 1.0,
                            ..c
                        },
                        roster(t)
                    )
                    .owned,
                0
            );
        }
    }
}

#[test]
fn final_active_grab_frame_remains_live_at_fractional_motion_times() {
    let t = Target {
        first: 6,
        last: 7,
        frame: 7.5,
        ..target()
    };
    assert_eq!(contact(context(0.0), t), Some(0.0));
    assert_eq!(contact(context(0.0), Target { frame: 8.0, ..t }), None);
    // Contact precisely when the active window ends is already too late.
    assert_eq!(
        contact(
            context(0.0),
            Target {
                position: [15.0, 0.0],
                velocity: [-2.0, 0.0],
                ..t
            }
        ),
        None
    );
}

#[test]
fn one_delivered_dodge_releases_down_guard_and_waits_for_native_success_without_an_unconfigured_punish(
) {
    use kirby_assist_core::{
        gameplay::{GameplayContext, Opponent},
        ledge_trap::LedgeSettings,
    };
    let mut tool = SpotDodge::default();
    let t = Target {
        last: 40,
        ..target()
    };
    assert_eq!(
        tool.sample(true, context(0.), roster(t)).requested,
        MAPPER_ZR
    );
    tool.mapped_dodge(0, t.id);
    for frame in 1..8 {
        let c = Context {
            status: if frame >= 3 { ESCAPE } else { 0 },
            ..context(frame as f32)
        };
        let out = tool.sample(true, c, roster(t));
        assert_eq!(out.requested, 0);
        if frame < 3 {
            assert_eq!(out.stick, Some([0.; 2]));
        }
        tool.delivery_tick(frame);
        let punish = tool.punish_automatic(
            true,
            None,
            LedgeSettings::default(),
            GameplayContext {
                gameplay: true,
                can_act: true,
                status: c.status,
                motion_kind: 1,
                motion_rate: 1.,
                motion_frame: frame as f32,
                ..Default::default()
            },
            Some(Opponent {
                id: t.id,
                ..Default::default()
            }),
        );
        assert_eq!(punish.owned, 0);
        assert_eq!(punish.requested, 0);
    }
    assert_eq!(
        tool.sample(true, context(9.), roster(Target { frame: 0., ..t }))
            .requested,
        0
    );
}
#[test]
fn delivered_dodge_wait_has_a_mapper_budget_even_with_frozen_poses_and_never_repeats_a_guard_edge()
{
    let mut tool = SpotDodge::default();
    let t = Target {
        last: 40,
        ..target()
    };
    tool.sample(true, context(0.), roster(t));
    tool.mapped_dodge(1, t.id);
    for _ in 0..100 {
        tool.delivery_tick(1);
        let out = tool.sample(true, context(0.), roster(t));
        assert_eq!(out.requested, 0);
        assert_ne!(out.owned, 0);
    }
    for tick in 2..=25 {
        tool.delivery_tick(tick);
        assert_eq!(tool.sample(true, context(0.), roster(t)).requested, 0);
    }
    assert_eq!(tool.sample(true, context(0.), roster(t)).owned, 0);
}

#[test]
fn rejected_guard_packets_are_bounded_without_falsely_confirming_a_dodge_or_a_punish() {
    use kirby_assist_core::{
        gameplay::{GameplayContext, Opponent},
        ledge_trap::LedgeSettings,
        sequence_config::Catalog,
    };
    let mut tool = SpotDodge::default();
    let t = Target {
        last: 40,
        ..target()
    };
    for tick in 0..24 {
        assert_eq!(
            tool.sample(true, context(0.), roster(t)).requested,
            MAPPER_ZR
        );
        tool.attempted_dodge(tick, t.id);
    }
    assert_eq!(tool.sample(true, context(0.), roster(t)).owned, 0);
    let s = Catalog::parse(
        "[bowser.roll.after]\nstart_delay=0\nreaction=auto_grab_dodged:25:10\nactions=up_b:4\n",
    )
    .unwrap()
    .entries()[0]
        .sequence;
    let p = tool.punish_automatic(
        true,
        Some(&s),
        LedgeSettings::default(),
        GameplayContext {
            gameplay: true,
            can_act: true,
            motion_kind: 1,
            motion_rate: 1.,
            ..Default::default()
        },
        Some(Opponent {
            id: t.id,
            ..Default::default()
        }),
    );
    assert_eq!(p.owned, 0);
    assert!(!tool.punish_committed());
}
