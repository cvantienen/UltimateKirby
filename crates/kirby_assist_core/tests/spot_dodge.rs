use kirby_assist_core::{
    ledge_trap::LedgeOutput,
    manual_controls::{ManualOverlay, TurboOutput},
    native_input::{ButtonTrace, MAPPER_A, MAPPER_B, MAPPER_ZR},
    spot_dodge::{contact, Context, SpotDodge, Target, ESCAPE},
};
fn context(frame: f32) -> Context {
    Context {
        object: 1,
        status: 0,
        situation: 0,
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
        status: 0x37,
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
fn standing_dash_and_pivot_grabs_trigger_at_the_forecast_boundary() {
    for (status, first) in [(0x37, 6), (0x39, 9), (0x3b, 10)] {
        let mut tool = SpotDodge::default();
        let t = Target {
            status,
            first,
            last: first + 1,
            frame: f32::from(first) - 5.0,
            ..target()
        };
        assert_eq!(
            tool.sample(true, context(0.0), roster(t)),
            LedgeOutput::default()
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
            status: 0x1e0,
            ..target()
        },
        Target {
            motion: 0,
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
            LedgeOutput::default()
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
            situation: 2,
            ..context(0.0)
        },
        Context {
            status: 0x47,
            ..context(0.0)
        },
        Context {
            status: 0x27,
            ..context(0.0)
        },
        Context {
            status: 0x1e,
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
            LedgeOutput::default()
        );
    }
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
    // A new native grab animation rewinds its age and may create a new attempt.
    assert_eq!(
        tool.sample(true, context(2.0), roster(Target { frame: 0.0, ..t }))
            .requested,
        0
    );
    assert_eq!(
        tool.sample(true, context(3.0), roster(t)).requested,
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
    assert_eq!(tool.sample(true, context(1.0), roster(target())), out);
    assert_eq!(
        tool.sample(true, context(3.0), roster(target())).requested,
        0
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
        LedgeOutput::default()
    );
    tool.sample(true, context(2.0), roster(target()));
    assert_eq!(
        tool.sample(false, context(3.0), roster(target())),
        LedgeOutput::default()
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
        .requested,
        0
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
