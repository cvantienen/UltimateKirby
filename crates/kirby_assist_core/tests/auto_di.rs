use kirby_assist_core::auto_di::{
    choose, predicted_launch, AutoDi, Context, Mode, Reason, Settings,
};
fn context(remaining: u64) -> Context {
    Context {
        object: 1,
        damage_stop: true,
        remaining,
        launch: [3.0, 1.0],
        outward: 1.0,
    }
}
#[test]
fn adaptive_separates_escape_from_survival_and_respects_forced_modes() {
    let weak = Context {
        launch: [1.0, 1.0],
        ..context(3)
    };
    assert_eq!(
        choose(Settings::default(), weak).unwrap().reason,
        Reason::ComboEscape
    );
    assert_eq!(
        choose(Settings::default(), context(3)).unwrap().reason,
        Reason::SideSurvival
    );
    assert_eq!(
        choose(
            Settings {
                mode: Mode::Survival,
                ..Settings::default()
            },
            weak
        )
        .unwrap()
        .reason,
        Reason::SideSurvival
    );
    assert_eq!(
        choose(
            Settings {
                mode: Mode::Escape,
                ..Settings::default()
            },
            context(3)
        )
        .unwrap()
        .reason,
        Reason::ComboEscape
    );
}
#[test]
fn survival_reduces_the_threatened_axis_without_an_upward_lsi_boost() {
    for side in [-1.0, 1.0] {
        for launch in [[side * 4.0, 1.0], [side * 0.2, 4.0]] {
            let c = Context {
                launch,
                outward: side,
                ..context(3)
            };
            let d = choose(
                Settings {
                    mode: Mode::Survival,
                    ..Settings::default()
                },
                c,
            )
            .unwrap();
            let after = predicted_launch(launch, d.stick);
            let axis = usize::from(launch[1].abs() > launch[0].abs());
            assert!(after[axis].abs() < launch[axis].abs());
            assert!(d.stick[0].hypot(d.stick[1]) <= 1.00001);
        }
    }
}
#[test]
fn escape_favors_separation_from_either_attributed_attacker() {
    for side in [-1.0, 1.0] {
        let c = Context {
            launch: [side, 1.0],
            outward: side,
            ..context(3)
        };
        let d = choose(
            Settings {
                mode: Mode::Escape,
                ..Settings::default()
            },
            c,
        )
        .unwrap();
        assert!(predicted_launch(c.launch, d.stick)[0] * side >= c.launch[0] * side);
    }
}
#[test]
fn sdi_keeps_early_frames_di_is_stable_at_release_and_normal_control_returns_afterward() {
    let mut di = AutoDi::default();
    for left in (4..=12).rev() {
        assert_eq!(
            di.sample(Settings::default(), context(left), [0.0; 2]),
            None
        );
    }
    let chosen = di.sample(Settings::default(), context(3), [0.0; 2]);
    assert!(chosen.is_some());
    for left in [3, 3, 2, 2, 1, 1] {
        assert_eq!(
            di.sample(Settings::default(), context(left), [0.0; 2]),
            chosen
        );
    }
    assert_eq!(
        di.sample(
            Settings::default(),
            Context {
                damage_stop: false,
                ..context(1)
            },
            [0.0; 2]
        ),
        None
    );
    assert!(di.decision().is_none());
}
#[test]
fn deliberate_manual_change_yields_for_the_hit_but_pre_hit_stick_is_not_an_override() {
    let mut di = AutoDi::default();
    di.sample(Settings::default(), context(8), [1.0, 0.0]);
    assert!(di
        .sample(Settings::default(), context(3), [1.0, 0.0])
        .is_some());
    assert_eq!(
        di.sample(Settings::default(), context(2), [-1.0, 0.0]),
        None
    );
    assert!(di.manual_override());
    assert_eq!(di.sample(Settings::default(), context(1), [0.0; 2]), None);
    // A fresh hit (increased hitstop) starts a separate decision.
    di.sample(Settings::default(), context(8), [0.0; 2]);
    assert!(!di.manual_override());
    assert!(di
        .sample(Settings::default(), context(3), [0.0; 2])
        .is_some());
}
#[test]
fn unknown_malformed_disabled_or_non_victim_inputs_do_not_take_over() {
    assert!(choose(
        Settings::default(),
        Context {
            launch: [f32::NAN, 2.0],
            ..context(3)
        }
    )
    .is_none());
    assert!(choose(
        Settings::default(),
        Context {
            launch: [0.0; 2],
            ..context(3)
        }
    )
    .is_none());
    assert!(choose(
        Settings {
            mode: Mode::Escape,
            ..Settings::default()
        },
        Context {
            outward: 0.0,
            ..context(3)
        }
    )
    .is_none());
    assert!(choose(
        Settings {
            mode: Mode::Survival,
            ..Settings::default()
        },
        Context {
            launch: [0.0, 4.0],
            outward: 0.0,
            ..context(3)
        }
    )
    .is_none());
    let mut di = AutoDi::default();
    for c in [
        Context {
            damage_stop: false,
            ..context(3)
        },
        context(0),
        context(121),
    ] {
        assert!(di.sample(Settings::default(), c, [0.0; 2]).is_none());
    }
    assert!(di
        .sample(
            Settings {
                enabled: false,
                ..Settings::default()
            },
            context(3),
            [0.0; 2]
        )
        .is_none());
}
#[test]
fn config_supports_disabling_and_mode_selection_and_rejects_invalid_thresholds() {
    use kirby_assist_core::live_input::LiveConfig;
    let c = LiveConfig::parse("auto_di=false\nauto_di_mode=escape\nauto_di_survival_speed=2.5")
        .unwrap();
    assert!(!c.auto_di.enabled);
    assert_eq!(c.auto_di.mode, Mode::Escape);
    assert_eq!(c.auto_di.survival_speed, 2.5);
    for value in ["NaN", "inf", "0", "31"] {
        assert!(LiveConfig::parse(&format!("auto_di_survival_speed={value}")).is_err());
    }
    assert!(LiveConfig::parse("auto_di_mode=unknown").is_err());
    assert!(LiveConfig::parse(include_str!(
        "../../../deployment/ultimate/kirby_assist/config.txt"
    ))
    .is_ok());
}

#[test]
fn late_attacker_attribution_can_rearm_the_same_launch_without_extra_game_frames() {
    let mut di = AutoDi::default();
    let c = Context {
        launch: [1.0, 1.0],
        outward: 0.0,
        ..context(3)
    };
    assert!(di.sample(Settings::default(), c, [0.0; 2]).is_none());
    assert!(di
        .sample(
            Settings::default(),
            Context { outward: -1.0, ..c },
            [0.0; 2]
        )
        .is_some());
}
