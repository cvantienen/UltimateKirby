use kirby_assist_core::{native_input::MAPPER_A, option_read::*};
fn c(frame: f32) -> Context {
    Context {
        object: 1,
        status: 0,
        motion: 1,
        frame,
        rate: 1.0,
        stopped: false,
        grounded: true,
        can_act: true,
        down_ready: true,
        forward_ready: true,
        facing: 1.0,
        position: [0.0; 2],
        velocity: [0.0; 2],
    }
}
fn cfg() -> Config {
    Config {
        down: [7, 19],
        forward: [13, 15],
        down_reach: 14.0,
        forward_reach: 22.0,
    }
}
fn t(kind: Kind, x: f32) -> Target {
    Target {
        id: 2,
        kind,
        status: if kind == Kind::Spot {
            0x1f
        } else if kind == Kind::Roll {
            0x20
        } else {
            0x37
        },
        motion: 2,
        frame: 10.0,
        rate: 1.0,
        position: [x, 0.0],
        velocity: [0.0; 2],
        opens: 10.0,
        closes: 30.0,
    }
}
fn targets(t: Target) -> [Option<Target>; 8] {
    let mut a = [None; 8];
    a[0] = Some(t);
    a
}
#[test]
fn all_three_options_select_down_near_forward_far_and_release_on_acceptance() {
    for kind in [Kind::Spot, Kind::Grab, Kind::Roll] {
        for x in [10.0, 20.0] {
            if kind == Kind::Roll && x == 10.0 {
                continue;
            }
            let mut p = Punisher::default();
            assert_eq!(
                p.sample(true, c(0.0), cfg(), targets(t(kind, x))).requested,
                0
            );
            let out = p.sample(true, c(1.0), cfg(), targets(t(kind, x)));
            assert_eq!(out.requested, MAPPER_A);
            assert_eq!(
                out.stick,
                Some(if x > 14.0 { [1.0, 0.0] } else { [0.0, -1.0] })
            );
            assert_eq!(
                p.sample(
                    true,
                    Context {
                        status: if x > 14.0 { 0x2d } else { 0x30 },
                        motion: 3,
                        ..c(0.0)
                    },
                    cfg(),
                    targets(t(kind, x))
                )
                .owned,
                0
            );
            assert_eq!(
                p.sample(true, c(2.0), cfg(), targets(t(kind, x))).requested,
                0
            );
        }
    }
}
#[test]
fn rejects_expired_unreachable_unknown_protection_and_airborne() {
    for target in [
        t(Kind::Spot, 40.0),
        Target {
            closes: 5.0,
            ..t(Kind::Spot, 10.0)
        },
        Target {
            opens: f32::NAN,
            ..t(Kind::Spot, 10.0)
        },
        Target {
            position: [10.0, 20.0],
            ..t(Kind::Spot, 10.0)
        },
    ] {
        let mut p = Punisher::default();
        assert_eq!(p.sample(true, c(0.0), cfg(), targets(target)).owned, 0);
    }
    assert_eq!(
        Punisher::default()
            .sample(
                true,
                Context {
                    grounded: false,
                    ..c(0.0)
                },
                cfg(),
                targets(t(Kind::Spot, 10.0))
            )
            .owned,
        0
    );
}
#[test]
fn forward_smash_turns_first_and_never_presses_attack_into_wrong_facing() {
    let mut p = Punisher::default();
    let target = t(Kind::Spot, -20.0);
    let out = p.sample(true, c(0.0), cfg(), targets(target));
    assert_eq!(out.requested, 0);
    assert_eq!(out.stick, Some([-0.3, 0.0]));
    let turned = Context {
        facing: -1.0,
        ..c(1.0)
    };
    assert_eq!(p.sample(true, turned, cfg(), targets(target)).requested, 0);
    let out = p.sample(
        true,
        Context {
            frame: 2.0,
            ..turned
        },
        cfg(),
        targets(target),
    );
    assert_eq!(out.requested, MAPPER_A);
    assert_eq!(out.stick, Some([-1.0, 0.0]));
}
#[test]
fn duplicate_polls_stop_and_disabled_toggles_do_not_repeat_a_punish() {
    let mut p = Punisher::default();
    let target = t(Kind::Grab, 10.0);
    p.sample(true, c(0.0), cfg(), targets(target));
    let out = p.sample(true, c(1.0), cfg(), targets(target));
    for _ in 0..100 {
        assert_eq!(p.sample(true, c(1.0), cfg(), targets(target)), out);
    }
    assert_eq!(
        p.sample(
            true,
            Context {
                stopped: true,
                ..c(1.0)
            },
            cfg(),
            targets(target)
        ),
        out
    );
    assert_eq!(p.sample(false, c(2.0), cfg(), targets(target)).owned, 0);
    assert_eq!(p.sample(true, c(3.0), cfg(), targets(target)).requested, 0);
}
#[test]
fn toggles_are_saved_and_independent() {
    use kirby_assist_core::{
        fighter_profiles::FighterProfiles,
        ledge_menu::{MenuRow, MenuSelection},
        manual_controls::DPAD_RIGHT,
    };
    let profiles = FighterProfiles::default();
    let menu = &profiles.profile(6).unwrap().menu_options;
    let mut s = MenuSelection::new(menu);
    s.input(0, menu);
    s.row = MenuRow::SpotPunish;
    s.input(DPAD_RIGHT, menu);
    assert!(!s.spot_punish);
    assert!(s.grab_dodge);
    s.input(0, menu);
    s.row = MenuRow::GrabDodge;
    s.input(DPAD_RIGHT, menu);
    assert!(!s.grab_dodge);
}

#[test]
fn primer_requires_real_native_progress_but_can_use_the_opponents_clock() {
    let mut p = Punisher::default();
    let target = t(Kind::Spot, 10.0);
    let prime = p.sample(true, c(0.0), cfg(), targets(target));
    assert_eq!(prime.requested, 0);
    for _ in 0..100 {
        assert_eq!(p.sample(true, c(0.0), cfg(), targets(target)), prime);
    }
    assert_eq!(
        p.sample(
            true,
            c(0.0),
            cfg(),
            targets(Target {
                frame: 11.0,
                ..target
            })
        )
        .requested,
        MAPPER_A
    );
}
#[test]
fn native_readiness_does_not_cancel_uncancellable_own_attacks() {
    let mut p = Punisher::default();
    let own = Context {
        status: 0x27,
        can_act: false,
        ..c(0.0)
    };
    assert_eq!(
        p.sample(true, own, cfg(), targets(t(Kind::Spot, 10.0)))
            .owned,
        0
    );
}

#[test]
fn held_shield_releases_before_a_punish_and_does_not_add_another_primer() {
    let mut p = Punisher::default();
    let target = Target {
        opens: 15.0,
        closes: 35.0,
        ..t(Kind::Spot, 10.0)
    };
    let shield = Context {
        status: 0x1b,
        can_act: false,
        down_ready: false,
        forward_ready: false,
        ..c(0.0)
    };
    let release = p.sample(true, shield, cfg(), targets(target));
    assert_ne!(release.owned, 0);
    assert_eq!(release.requested, 0);
    assert_eq!(release.stick, Some([0.0; 2]));
    let off = Context {
        status: 0x1d,
        frame: 5.0,
        ..shield
    };
    assert_eq!(
        p.sample(
            true,
            off,
            cfg(),
            targets(Target {
                opens: 10.0,
                closes: 30.0,
                frame: 15.0,
                ..target
            })
        )
        .requested,
        0
    );
    assert_eq!(
        p.sample(
            true,
            Context {
                status: 0x1d,
                frame: 11.0,
                can_act: false,
                ..c(0.0)
            },
            cfg(),
            targets(Target {
                opens: 4.0,
                closes: 24.0,
                frame: 21.0,
                ..target
            })
        )
        .requested,
        MAPPER_A
    );
}

#[test]
fn delayed_smash_acceptance_retries_but_native_startup_and_expiry_stop_it() {
    for kind in [Kind::Spot, Kind::Grab, Kind::Roll] {
        let mut p = Punisher::default();
        let target = Target {
            opens: 0.0,
            closes: 60.0,
            ..t(kind, 20.0)
        };
        p.sample(true, c(0.0), cfg(), targets(target));
        assert_eq!(
            p.sample(true, c(1.0), cfg(), targets(target)).requested,
            MAPPER_A
        );
        assert_eq!(p.sample(true, c(4.0), cfg(), targets(target)).requested, 0);
        assert_eq!(
            p.sample(true, c(5.0), cfg(), targets(target)).requested,
            MAPPER_A
        );
        assert_eq!(
            p.sample(
                true,
                c(6.0),
                cfg(),
                targets(Target {
                    closes: 1.0,
                    ..target
                })
            )
            .owned,
            0
        );
    }
}
