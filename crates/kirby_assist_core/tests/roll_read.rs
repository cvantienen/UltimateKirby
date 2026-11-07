use kirby_assist_core::{
    fighter_profiles::FighterProfiles,
    ledge_menu::{MenuRow, MenuSelection},
    native_input::MAPPER_A,
    roll_read::{self, Choice, Context, RollRead, Target, ESCAPE_F},
};
fn own(frame: f32) -> Context {
    Context {
        status: 0,
        situation: 0,
        motion: 123,
        frame,
        rate: 1.0,
        stopped: false,
        can_act: true,
        can_smash: true,
        can_grab: true,
        facing: 1.0,
        position: [0., 0.],
        movement: None,
    }
}

#[test]
fn target_first_readiness_preserves_both_roll_modes_and_target_loss() {
    for choice in [Choice::Grab, Choice::DownSmash] {
        let cfg = roll_read::config_for(6, choice).unwrap();
        let mut reference = RollRead::default();
        let mut gated = RollRead::default();
        for tick in 0..8 {
            let targets = if matches!(tick, 1 | 2 | 5 | 6) {
                targets(enemy())
            } else {
                [None; 8]
            };
            let c = Context {
                stopped: tick == 2 || tick == 3,
                ..own(tick as f32)
            };
            let relevant = targets.iter().any(Option::is_some);
            let observed = Context {
                can_smash: relevant && choice == Choice::DownSmash && c.can_smash,
                can_grab: relevant && choice == Choice::Grab && c.can_grab,
                ..c
            };
            assert_eq!(
                gated.sample_choice(choice, cfg, observed, targets),
                reference.sample_choice(choice, cfg, c, targets)
            );
        }
    }
}

#[test]
fn cancelled_grab_does_not_rearm_when_the_same_enemy_changes_roster_slot() {
    let cfg = roll_read::config_for(6, Choice::Grab).unwrap();
    let t = Target {
        position: [8.0, 0.0],
        velocity_x: -1.0,
        vulnerable_in: Some(7.0),
        ..enemy()
    };
    let mut read = RollRead::default();
    let prime = read.sample_choice(Choice::Grab, cfg, own(0.0), targets(t));
    assert_ne!(prime.owned, 0);
    read.cancel_pending();
    let mut reordered = [None; 8];
    reordered[4] = Some(Target {
        frame: t.frame + 1.0,
        ..t
    });
    assert_eq!(
        read.sample_choice(Choice::Grab, cfg, own(1.0), reordered)
            .owned,
        0
    );
}

#[test]
fn skipped_native_clock_gap_abandons_prepared_smash_instead_of_issuing_late() {
    let cfg = roll_read::fighter_config(6).unwrap();
    let mut read = RollRead::default();
    assert_ne!(read.sample(true, cfg, own(0.0), targets(enemy())).owned, 0);
    assert_eq!(read.sample(true, cfg, own(20.0), targets(enemy())).owned, 0);
}

#[test]
fn shielded_roll_read_releases_held_shield_then_smashes_on_native_readiness() {
    use kirby_assist_core::{
        manual_controls::{ManualOverlay, TurboOutput},
        native_input::{ButtonTrace, MAPPER_ZR},
    };
    let cfg = roll_read::fighter_config(6).unwrap();
    let mut r = RollRead::default();
    let shield = Context {
        status: 0x1c,
        can_act: false,
        can_smash: false,
        ..own(0.0)
    };
    let t = Target {
        position: [8.0, 0.0],
        velocity_x: -0.2,
        vulnerable_in: Some(20.0),
        actionable_in: Some(35.0),
        ..enemy()
    };
    let release = r.sample(true, cfg, shield, targets(t));
    assert_ne!(release.owned, 0);
    assert_eq!(release.requested, 0);
    assert_eq!(release.stick, Some([0.0; 2]));
    assert!(
        r.committed(),
        "shield release cannot be stolen by automatic shielding"
    );
    let mapped = ManualOverlay::default().apply_controls(
        ButtonTrace {
            held: MAPPER_ZR,
            ..Default::default()
        },
        [0.0; 2],
        TurboOutput {
            consume_dpad: false,
            stick: release.stick,
        },
        0,
        release.requested,
        release.owned,
    );
    assert_eq!(
        mapped.buttons.held & MAPPER_ZR,
        0,
        "override physically held shield"
    );
    for frame in 0..11 {
        let out = r.sample(
            true,
            cfg,
            Context {
                status: 0x1d,
                frame: frame as f32,
                ..shield
            },
            targets(Target {
                frame: t.frame + frame as f32 + 1.0,
                ..t
            }),
        );
        assert_eq!(out.requested, 0, "no Attack during shield release");
        assert!(r.committed());
    }
    let out = r.sample(
        true,
        cfg,
        own(0.0),
        targets(Target {
            vulnerable_in: Some(7.0),
            actionable_in: Some(20.0),
            frame: 20.0,
            ..t
        }),
    );
    assert_eq!(
        out.requested, MAPPER_A,
        "no extra primer once release already primed the smash"
    );
    assert_eq!(out.stick, Some([0.0, -1.0]));
}

#[test]
fn shield_stun_and_rolls_too_late_to_drop_shield_are_not_forced() {
    let cfg = roll_read::fighter_config(6).unwrap();
    let t = Target {
        position: [8.0, 0.0],
        velocity_x: -0.2,
        vulnerable_in: Some(3.0),
        actionable_in: Some(15.0),
        ..enemy()
    };
    let mut r = RollRead::default();
    let shield = Context {
        status: 0x1c,
        can_act: false,
        can_smash: false,
        ..own(0.0)
    };
    assert_eq!(
        r.sample(true, cfg, shield, targets(t)).owned,
        0,
        "11-frame shield drop cannot fit"
    );
    let mut r = RollRead::default();
    assert_eq!(
        r.sample(
            true,
            cfg,
            Context {
                status: 0x1e,
                ..shield
            },
            targets(Target {
                actionable_in: Some(40.0),
                ..t
            })
        )
        .owned,
        0
    );
    let mut r = RollRead::default();
    r.sample(
        true,
        cfg,
        shield,
        targets(Target {
            actionable_in: Some(40.0),
            ..t
        }),
    );
    assert_eq!(
        r.sample(
            true,
            cfg,
            Context {
                status: 0x1d,
                ..shield
            },
            [None; 8]
        )
        .owned,
        0,
        "target loss cancels shield-drop plan"
    );
}

#[test]
fn yoshi_uses_separate_front_and_rear_sweeps_and_tongue_grab_data() {
    let cfg = roll_read::fighter_config(5).unwrap();
    assert_eq!((cfg.first, cfg.last, cfg.back), (7, 8, Some((22, 23))));
    let grab = roll_read::config_for(5, Choice::Grab).unwrap();
    assert_eq!((grab.first, grab.last), (14, 21));
    assert_eq!(grab.reach, 26.0);
    for (x, v, opens, end, eligible) in [
        (8.0, -0.2, 6.0, 12.0, true),
        (-8.0, 0.2, 6.0, 12.0, false),
        (-8.0, 0.2, 21.0, 26.0, true),
    ] {
        for facing in [-1.0, 1.0] {
            let mut r = RollRead::default();
            let t = Target {
                position: [x * facing, 0.0],
                velocity_x: v * facing,
                vulnerable_in: Some(opens),
                actionable_in: Some(end),
                ..enemy()
            };
            let prime = r.sample(true, cfg, Context { facing, ..own(0.0) }, targets(t));
            let out = r.sample(
                true,
                cfg,
                Context { facing, ..own(1.0) },
                targets(Target {
                    frame: t.frame + 1.0,
                    ..t
                }),
            );
            if eligible {
                assert_ne!(prime.owned, 0);
                assert_eq!(out.requested, MAPPER_A);
            } else {
                assert_eq!(
                    out.requested, 0,
                    "rear sweep cannot use frame-7 front timing"
                );
            }
        }
    }
}

#[test]
fn roll_clock_can_deliver_with_a_frozen_own_idle_pose_and_duplicate_polls_cannot() {
    let cfg = roll_read::fighter_config(6).unwrap();
    let mut read = RollRead::default();
    let c = own(0.0);
    let prime = read.sample(true, cfg, c, targets(enemy()));
    assert_eq!(prime.requested, 0);
    assert_eq!(read.sample(true, cfg, c, targets(enemy())), prime);
    let fire = read.sample(
        true,
        cfg,
        c,
        targets(Target {
            frame: 3.0,
            vulnerable_in: Some(13.0),
            ..enemy()
        }),
    );
    assert_eq!(fire.requested, MAPPER_A);
}

#[test]
fn toward_roll_stopping_short_and_roll_through_remain_eligible_but_departed_targets_cancel() {
    let cfg = roll_read::fighter_config(6).unwrap();
    for side in [-1.0, 1.0] {
        for (x, speed) in [(side * 12.0, -side * 0.2), (side * 2.0, -side)] {
            let mut read = RollRead::default();
            let t = Target {
                position: [x, 0.0],
                velocity_x: speed,
                ..enemy()
            };
            assert_ne!(read.sample(true, cfg, own(0.0), targets(t)).owned, 0);
            // The first roll stops short; the second crosses Kirby's body.
            let t = Target {
                frame: 3.0,
                position: [if x.abs() > 5.0 { side * 11.0 } else { -side }, 0.0],
                velocity_x: if x.abs() > 5.0 { 0.0 } else { speed },
                vulnerable_in: Some(13.0),
                ..t
            };
            assert_eq!(
                read.sample(true, cfg, own(1.0), targets(t)).requested,
                MAPPER_A
            );
            assert_eq!(
                read.sample(
                    true,
                    cfg,
                    own(1.0),
                    targets(Target {
                        position: [side * 60.0, 0.0],
                        ..t
                    })
                )
                .requested,
                0,
                "new geometry cancels even on a duplicate own frame"
            );
        }
    }
}

#[test]
fn roll_cancel_and_roster_reordering_cannot_reissue_the_same_roll() {
    let cfg = roll_read::fighter_config(6).unwrap();
    let mut read = RollRead::default();
    read.sample(true, cfg, own(0.0), targets(enemy()));
    read.cancel_pending();
    let mut moved = [None; 8];
    moved[3] = Some(Target {
        frame: 3.0,
        ..enemy()
    });
    assert_eq!(read.sample(true, cfg, own(1.0), moved).owned, 0);
    // A real new roll has a rewound native roll age and is eligible again.
    assert_ne!(
        read.sample(
            true,
            cfg,
            own(2.0),
            targets(Target {
                frame: 0.0,
                ..enemy()
            })
        )
        .owned,
        0
    );
}

#[test]
fn already_primed_roll_can_use_last_startup_opportunity_without_an_extra_primer() {
    let cfg = roll_read::fighter_config(6).unwrap();
    let mut read = RollRead::default();
    let t = Target {
        vulnerable_in: Some(7.0),
        actionable_in: Some(9.0),
        ..enemy()
    };
    assert_eq!(read.sample(true, cfg, own(0.0), targets(t)).requested, 0);
    assert_eq!(
        read.sample(
            true,
            cfg,
            own(1.0),
            targets(Target {
                frame: 3.0,
                vulnerable_in: Some(6.0),
                actionable_in: Some(8.0),
                ..t
            })
        )
        .requested,
        MAPPER_A
    );
}
fn enemy() -> Target {
    Target {
        id: 2,
        status: ESCAPE_F,
        motion: 456,
        frame: 2.,
        rate: 1.0,
        position: [-8., 0.],
        velocity_x: 1.,
        vulnerable_in: Some(14.),
        actionable_in: Some(25.),
    }
}
fn targets(t: Target) -> [Option<Target>; 8] {
    let mut a = [None; 8];
    a[0] = Some(t);
    a
}
fn walking(frame: f32, x: f32) -> Context {
    Context {
        position: [x, 0.],
        movement: Some(roll_read::Movement {
            walk_speed: 1.2,
            velocity_x: 0.,
            brake: 0.1,
            safe_ground: [true; 2],
        }),
        ..own(frame)
    }
}

#[test]
fn fresh_stationary_or_away_rolls_do_not_trigger_smashes() {
    let cfg = roll_read::fighter_config(6).unwrap();
    for side in [-1.0, 1.0] {
        for speed in [0.0, side * 0.25] {
            let t = Target {
                position: [side * 10.0, 0.0],
                velocity_x: speed,
                vulnerable_in: Some(18.0),
                actionable_in: Some(30.0),
                ..enemy()
            };
            let mut read = RollRead::default();
            assert_eq!(read.sample(true, cfg, own(0.0), targets(t)).owned, 0);
            assert_eq!(
                read.sample(true, cfg, own(1.0), targets(Target { frame: 3.0, ..t }))
                    .requested,
                0
            );
        }
    }
}

#[test]
fn reachable_far_roll_walks_then_brakes_primes_and_smashes() {
    let cfg = roll_read::fighter_config(6).unwrap();
    let mut read = RollRead::default();
    let t = Target {
        position: [-28., 0.],
        vulnerable_in: Some(22.),
        actionable_in: Some(36.),
        ..enemy()
    };
    let walk = read.sample(true, cfg, walking(0., 0.), targets(t));
    assert_eq!(walk.stick, Some([-0.65, 0.]));
    assert_eq!(walk.requested, 0);
    let prime = read.sample(
        true,
        cfg,
        walking(1., -6.),
        targets(Target {
            frame: 4.,
            position: [-24., 0.],
            vulnerable_in: Some(18.),
            ..t
        }),
    );
    assert_eq!(prime.stick, Some([0.; 2]));
    assert_eq!(prime.requested, 0);
    let press = read.sample(
        true,
        cfg,
        walking(2., -6.),
        targets(Target {
            frame: 5.,
            position: [-23., 0.],
            vulnerable_in: Some(17.),
            ..t
        }),
    );
    assert_eq!(press.requested, MAPPER_A);
    let release = read.sample(
        true,
        cfg,
        Context {
            status: 0x30,
            ..walking(0., -6.)
        },
        targets(t),
    );
    assert_eq!(release.requested, 0);
}

#[test]
fn timing_alone_never_triggers_an_out_of_range_or_unreachable_smash() {
    let cfg = roll_read::fighter_config(6).unwrap();
    for side in [-1.0, 1.0] {
        for c in [own(0.0), walking(0.0, 0.0)] {
            let t = Target {
                position: [side * 45.0, 0.0],
                velocity_x: -side * 0.1,
                vulnerable_in: Some(16.0),
                actionable_in: Some(24.0),
                ..enemy()
            };
            let mut read = RollRead::default();
            assert_eq!(read.sample(true, cfg, c, targets(t)).owned, 0);
            assert_eq!(
                read.sample(
                    true,
                    cfg,
                    Context { frame: 1.0, ..c },
                    targets(Target {
                        frame: 3.0,
                        vulnerable_in: Some(15.0),
                        ..t
                    })
                )
                .requested,
                0
            );
        }
    }
}

#[test]
fn waiting_roll_walks_closer_inside_attack_reach_but_not_past_projection() {
    let cfg = roll_read::fighter_config(6).unwrap();
    for x in [-10.0, 10.0] {
        let mut read = RollRead::default();
        let t = Target {
            position: [x, 0.0],
            velocity_x: -x.signum() * 0.1,
            vulnerable_in: Some(30.0),
            actionable_in: Some(45.0),
            ..enemy()
        };
        assert_eq!(
            read.sample(true, cfg, walking(0.0, 0.0), targets(t)).stick,
            Some([x.signum() * 0.65, 0.0])
        );
        let arrived = read.sample(
            true,
            cfg,
            walking(1.0, x),
            targets(Target {
                frame: 3.0,
                velocity_x: 0.0,
                ..t
            }),
        );
        assert_eq!(arrived.stick, Some([0.0, 0.0]));
        assert_eq!(arrived.requested, 0);
        assert_ne!(
            arrived.owned, 0,
            "keep control while waiting for the punish"
        );
    }
}

#[test]
fn native_smash_readiness_can_deliver_when_generic_movement_is_unavailable() {
    let cfg = roll_read::fighter_config(6).unwrap();
    let mut read = RollRead::default();
    let c = Context {
        can_act: false,
        can_smash: true,
        ..own(0.0)
    };
    let t = enemy();
    let prime = read.sample(true, cfg, c, targets(t));
    assert_ne!(prime.owned, 0);
    assert_eq!(prime.requested, 0);
    assert_eq!(read.sample(true, cfg, c, targets(t)), prime);
    let press = read.sample(
        true,
        cfg,
        Context { frame: 1.0, ..c },
        targets(Target {
            frame: 3.0,
            vulnerable_in: Some(13.0),
            ..t
        }),
    );
    assert_eq!(press.requested, MAPPER_A);
}

#[test]
fn approach_refuses_unreachable_floorless_targets_and_cancels_when_target_leaves() {
    let cfg = roll_read::fighter_config(6).unwrap();
    let t = Target {
        position: [-28., 0.],
        vulnerable_in: Some(22.),
        actionable_in: Some(36.),
        ..enemy()
    };
    for c in [
        own(0.),
        Context {
            movement: Some(roll_read::Movement {
                safe_ground: [false; 2],
                ..walking(0., 0.).movement.unwrap()
            }),
            ..walking(0., 0.)
        },
    ] {
        let waiting = RollRead::default().sample(true, cfg, c, targets(t));
        assert_eq!(waiting.owned, 0);
        assert_eq!(waiting.requested, 0);
    }
    assert_eq!(
        RollRead::default()
            .sample(
                true,
                cfg,
                walking(0., 0.),
                targets(Target {
                    actionable_in: Some(10.),
                    ..t
                })
            )
            .owned,
        0
    );
    let mut read = RollRead::default();
    read.sample(true, cfg, walking(0., 0.), targets(t));
    assert_eq!(read.sample(true, cfg, walking(1., 0.), [None; 8]).owned, 0);
}
#[test]
fn retired_fixed_roll_menu_stays_off_and_hidden_after_match_reset() {
    let profiles = FighterProfiles::default();
    let presets = &profiles.profile(6).unwrap().menu_options;
    let mut menu = MenuSelection::new(presets);
    menu.roll_read_available = true;
    assert_eq!(menu.roll_read, Choice::Off);
    assert!(!menu.row_visible(MenuRow::RollRead));
    menu.begin_match();
    assert_eq!(menu.roll_read, Choice::Off);
    assert!(!menu.row_visible(MenuRow::RollRead));
    // Portable prototype facts remain testable, without live registration.
    assert!(roll_read::fighter_config(6).is_some());
    assert!(roll_read::fighter_config(5).is_some());
}

#[test]
fn approach_primes_then_smashes_once_and_releases_on_native_startup() {
    let cfg = roll_read::fighter_config(6).unwrap();
    let mut read = RollRead::default();
    let prime = read.sample(true, cfg, own(0.), targets(enemy()));
    assert_eq!(prime.requested, 0);
    assert_ne!(prime.owned, 0);
    assert_eq!(prime.stick, Some([0., 0.]));
    assert_eq!(
        read.sample(true, cfg, own(0.), targets(enemy())),
        prime,
        "duplicate polls don't advance"
    );
    let fire = read.sample(
        true,
        cfg,
        own(1.),
        targets(Target {
            frame: 3.,
            vulnerable_in: Some(13.),
            ..enemy()
        }),
    );
    assert_eq!(fire.requested, MAPPER_A);
    assert_eq!(fire.stick, Some([0., -1.]));
    let released = read.sample(
        true,
        cfg,
        Context {
            status: 0x30,
            can_act: false,
            ..own(0.)
        },
        targets(enemy()),
    );
    assert_eq!(released.requested, 0);
    let again = read.sample(
        true,
        cfg,
        own(2.),
        targets(Target {
            frame: 5.,
            ..enemy()
        }),
    );
    assert_eq!(again.owned, 0, "one attempt per roll");
}
#[test]
fn refuses_outside_vicinity_high_ledge_spot_dodge_and_unknown_timing() {
    let cfg = roll_read::fighter_config(6).unwrap();
    for t in [
        Target {
            position: [-65., 0.],
            ..enemy()
        },
        Target {
            position: [-8., 10.],
            ..enemy()
        },
        Target {
            status: 0x7a,
            ..enemy()
        },
        Target {
            status: 0x1f,
            ..enemy()
        },
        Target {
            vulnerable_in: None,
            ..enemy()
        },
        Target {
            actionable_in: None,
            ..enemy()
        },
    ] {
        let out = RollRead::default().sample(true, cfg, own(0.), targets(t));
        assert_eq!(out.owned, 0);
    }
}
#[test]
fn recognizes_body_crossing_but_off_and_target_loss_cancel_before_press() {
    let cfg = roll_read::fighter_config(6).unwrap();
    let mut read = RollRead::default();
    read.sample(
        true,
        cfg,
        own(0.),
        targets(Target {
            vulnerable_in: Some(20.),
            ..enemy()
        }),
    );
    let cross = read.sample(
        true,
        cfg,
        own(1.),
        targets(Target {
            frame: 8.,
            position: [2., 0.],
            velocity_x: 0.,
            ..enemy()
        }),
    );
    assert_ne!(cross.owned, 0);
    assert_eq!(
        read.sample(true, cfg, own(2.), [None; 8]).owned,
        0,
        "target gone during primer"
    );
    let mut read = RollRead::default();
    read.sample(true, cfg, own(0.), targets(enemy()));
    assert_eq!(read.sample(false, cfg, own(1.), targets(enemy())).owned, 0);
}

#[test]
fn skipped_approach_sample_still_recognizes_same_roll_crossing_kirbys_body() {
    let cfg = roll_read::fighter_config(6).unwrap();
    let mut read = RollRead::default();
    let before = Target {
        position: [-3.0, 0.0],
        velocity_x: 0.0,
        vulnerable_in: Some(15.0),
        actionable_in: Some(28.0),
        ..enemy()
    };
    assert_eq!(read.sample(true, cfg, own(0.0), targets(before)).owned, 0);
    let crossed = Target {
        frame: before.frame + 4.0,
        position: [3.0, 0.0],
        velocity_x: 1.0,
        vulnerable_in: Some(11.0),
        actionable_in: Some(24.0),
        ..before
    };
    let prime = read.sample(true, cfg, own(1.0), targets(crossed));
    assert_ne!(prime.owned, 0);
    assert_eq!(prime.requested, 0);
    let press = read.sample(
        true,
        cfg,
        own(2.0),
        targets(Target {
            frame: crossed.frame + 1.0,
            vulnerable_in: Some(10.0),
            ..crossed
        }),
    );
    assert_eq!(press.requested, MAPPER_A);
}

#[test]
fn shield_faceoff_detects_an_overlapping_roll_already_moving_past_kirby() {
    let cfg = roll_read::fighter_config(6).unwrap();
    let shield = Context {
        status: 0x1c,
        can_act: false,
        can_smash: false,
        ..own(0.0)
    };
    let passing = Target {
        frame: 6.0,
        position: [2.0, 0.0],
        velocity_x: 0.8,
        vulnerable_in: Some(20.0),
        actionable_in: Some(40.0),
        ..enemy()
    };
    let release = RollRead::default().sample(true, cfg, shield, targets(passing));
    assert_ne!(release.owned, 0);
    assert_eq!(release.requested, 0);
    assert_eq!(release.stick, Some([0.0, 0.0]));
}

#[test]
fn grab_primes_once_then_hands_back_on_native_grab_without_pummeling() {
    use kirby_assist_core::native_input::MAPPER_R;
    let cfg = roll_read::config_for(6, Choice::Grab).unwrap();
    for side in [-1., 1.] {
        let mut read = RollRead::default();
        let t = Target {
            position: [8. * side, 0.],
            velocity_x: -side,
            vulnerable_in: Some(7.),
            ..enemy()
        };
        let c = Context {
            facing: side,
            can_smash: false,
            ..own(0.)
        };
        let prime = read.sample_choice(Choice::Grab, cfg, c, targets(t));
        assert_eq!(prime.requested, 0);
        assert_eq!(prime.stick, Some([0.; 2]), "no chasing movement");
        let t = Target {
            frame: 3.,
            position: [7. * side, 0.],
            vulnerable_in: Some(6.),
            ..t
        };
        let c = Context { frame: 1., ..c };
        let grab = read.sample_choice(Choice::Grab, cfg, c, targets(t));
        assert_eq!(grab.requested, MAPPER_R);
        assert_eq!(grab.stick, Some([0.; 2]));
        assert_eq!(read.sample_choice(Choice::Grab, cfg, c, targets(t)), grab);
        assert_eq!(
            read.sample_choice(
                Choice::Grab,
                cfg,
                Context {
                    stopped: true,
                    can_act: false,
                    ..c
                },
                targets(t)
            ),
            grab
        );
        for status in [0x37, 0x3c] {
            assert_eq!(
                read.sample_choice(
                    Choice::Grab,
                    cfg,
                    Context {
                        status,
                        frame: 0.,
                        can_act: false,
                        ..c
                    },
                    targets(t)
                )
                .owned,
                0
            );
        }
        assert_eq!(
            read.sample_choice(
                Choice::Grab,
                cfg,
                Context { frame: 2., ..c },
                targets(Target { frame: 4., ..t })
            )
            .owned,
            0,
            "one attempt per roll"
        );
    }
}

#[test]
fn grab_confirms_turn_then_releases_direction_before_requesting_r() {
    use kirby_assist_core::native_input::MAPPER_R;
    let cfg = roll_read::config_for(6, Choice::Grab).unwrap();
    let mut read = RollRead::default();
    let t = Target {
        position: [8., 0.],
        velocity_x: -1.,
        vulnerable_in: Some(7.),
        ..enemy()
    };
    let turn = read.sample_choice(
        Choice::Grab,
        cfg,
        Context {
            facing: -1.,
            ..own(0.)
        },
        targets(t),
    );
    assert_eq!(turn.requested, 0);
    assert_eq!(turn.stick, Some([0.3, 0.]));
    let t = Target {
        position: [7., 0.],
        frame: 3.,
        vulnerable_in: Some(6.),
        ..t
    };
    let release = read.sample_choice(Choice::Grab, cfg, own(1.), targets(t));
    assert_eq!(release.requested, 0);
    assert_eq!(release.stick, Some([0.; 2]));
    let grab = read.sample_choice(
        Choice::Grab,
        cfg,
        own(2.),
        targets(Target {
            position: [6., 0.],
            frame: 4.,
            vulnerable_in: Some(5.),
            ..t
        }),
    );
    assert_eq!(grab.requested, MAPPER_R);
    assert_eq!(grab.stick, Some([0.; 2]));
    // An unaccepted request expires in native frames, never by mapper polls.
    assert_eq!(
        read.sample_choice(Choice::Grab, cfg, own(5.), targets(t))
            .requested,
        0
    );
}

#[test]
fn grab_off_range_unknown_timing_and_binding_edits_release_pending_input() {
    let cfg = roll_read::config_for(6, Choice::Grab).unwrap();
    for t in [
        Target {
            vulnerable_in: None,
            ..enemy()
        },
        Target {
            position: [-30., 0.],
            ..enemy()
        },
    ] {
        assert_eq!(
            RollRead::default()
                .sample_choice(Choice::Grab, cfg, own(0.), targets(t))
                .owned,
            0
        );
    }
    assert_eq!(
        RollRead::default()
            .sample_choice(
                Choice::Grab,
                cfg,
                Context {
                    can_grab: false,
                    ..own(0.)
                },
                targets(enemy())
            )
            .owned,
        0
    );
    let mut read = RollRead::default();
    let t = Target {
        position: [8., 0.],
        velocity_x: -1.,
        vulnerable_in: Some(7.),
        ..enemy()
    };
    assert_ne!(
        read.sample_choice(Choice::Grab, cfg, own(0.), targets(t))
            .owned,
        0
    );
    assert_eq!(
        read.sample_choice(Choice::Off, cfg, own(1.), targets(t))
            .owned,
        0
    );
    assert_eq!(Choice::DownSmash.cycle(true), Choice::Grab);
    assert_eq!(Choice::Grab.cycle(false), Choice::DownSmash);
    assert_eq!(Choice::Off.cycle(false), Choice::Grab);
    assert!(roll_read::config_for(5, Choice::Grab).is_some());

    // The delivery limit is game time, not raw animation frames at a slow rate.
    let mut read = RollRead::default();
    let slow = Context {
        rate: 0.5,
        ..own(0.)
    };
    read.sample_choice(Choice::Grab, cfg, slow, targets(t));
    let t = Target {
        frame: 3.,
        vulnerable_in: Some(6.),
        ..t
    };
    let pressed = read.sample_choice(
        Choice::Grab,
        cfg,
        Context { frame: 0.5, ..slow },
        targets(t),
    );
    assert_eq!(pressed.requested, kirby_assist_core::native_input::MAPPER_R);
    assert_eq!(
        read.sample_choice(Choice::Grab, cfg, Context { frame: 1., ..slow }, targets(t))
            .requested,
        kirby_assist_core::native_input::MAPPER_R
    );
    assert_eq!(
        read.sample_choice(Choice::Grab, cfg, Context { frame: 2., ..slow }, targets(t))
            .requested,
        0
    );
}

#[test]
fn frozen_idle_roll_grab_retries_and_cancels_on_changed_reach() {
    use kirby_assist_core::native_input::MAPPER_R;
    let cfg = roll_read::config_for(6, Choice::Grab).unwrap();
    let mut read = RollRead::default();
    let t = Target {
        position: [8.0, 0.0],
        velocity_x: -0.1,
        vulnerable_in: Some(6.0),
        actionable_in: Some(50.0),
        ..enemy()
    };
    read.sample_choice(Choice::Grab, cfg, own(0.0), targets(t));
    let t = Target {
        frame: t.frame + 1.0,
        ..t
    };
    assert_eq!(
        read.sample_choice(Choice::Grab, cfg, own(0.0), targets(t))
            .requested,
        MAPPER_R
    );
    assert_eq!(
        read.sample_choice(
            Choice::Grab,
            cfg,
            own(0.0),
            targets(Target {
                frame: t.frame + 3.0,
                ..t
            })
        )
        .requested,
        0
    );
    let t = Target {
        frame: t.frame + 4.0,
        ..t
    };
    assert_eq!(
        read.sample_choice(Choice::Grab, cfg, own(0.0), targets(t))
            .requested,
        MAPPER_R
    );
    assert_eq!(
        read.sample_choice(
            Choice::Grab,
            cfg,
            own(0.0),
            targets(Target {
                position: [100.0, 0.0],
                ..t
            })
        )
        .owned,
        0
    );
}
