use kirby_assist_core::{
    fighter_profiles::FighterProfiles,
    ledge_menu::MenuSelection,
    ledge_trap::{
        LedgeContext, LedgeSettings, LedgeTrap, Opponent, CLIFF_CLIMB, CLIFF_JUMP1, CLIFF_JUMP3,
        CLIFF_WAIT,
    },
    native_input::{MAPPER_A, MAPPER_B, MAPPER_X, MAPPER_ZL},
};

fn settings(kind: i32, neutral_choice: usize) -> LedgeSettings {
    let catalog = FighterProfiles::default();
    let profile = catalog.profile(kind).unwrap();
    let mut selection = MenuSelection::new(&profile.menu_options);
    selection.choices[0] = neutral_choice;
    selection.settings(
        &profile.menu_options,
        profile.apply(LedgeSettings {
            enabled: true,
            manual_spacing: true,
            distance_agnostic: true,
            ..LedgeSettings::default()
        }),
    )
}

fn context() -> LedgeContext {
    LedgeContext {
        held: true,
        gameplay: true,
        can_act: true,
        can_control: true,
        stopped: false,
        status: 0,
        situation: 0,
        motion_frame: 0.0,
        motion_kind: 0,
        motion_rate: 1.0,
        position: [20., 0.],
        velocity: [0., 0.],
        ground_brake: Some(0.1),
        facing: -1.,
        previous_status: 0,
        can_grab: true,
        can_tilt: true,
        can_down_tilt: true,
        can_down_special: true,
        can_dash_attack: true,
        can_down_smash: true,
        input_delay: 0.,
        can_jump: true,
        can_air_attack: true,
        air_physics: None,
        short_hop_speed: None,
        full_hop_speed: None,
        shield_stun_frames: None,
        shield_attacker: None,
        hit_confirmed: false,
        safe_ground: [true, true],
    }
}

fn enemy(status: i32, remaining: f32) -> Opponent {
    Opponent {
        id: 2,
        status,
        previous_status: CLIFF_WAIT,
        situation: 0,
        position: [0., 0.],
        vulnerable: false,
        velocity: [0., 0.],
        cliff_position: (status == CLIFF_WAIT).then_some([0., 0.]),
        vulnerable_in: Some(remaining),
        motion: None,
        attack_active: Some(false),
        air_physics: None,
        actionable_in: Some(40.),
    }
}

#[test]
fn kirby_drop_dair_launches_without_attack_then_buffers_down_a_in_squat() {
    let catalog = FighterProfiles::default();
    let profile = catalog.profile(6).unwrap();
    let mut selection = MenuSelection::new(&profile.menu_options);
    selection.choices[4] = profile.menu_options.lists[4]
        .options
        .iter()
        .position(|v| v.id == "short_hop_dair")
        .unwrap();
    let s = selection.settings(&profile.menu_options, profile.apply(settings(6, 0)));
    for side in [-1.0, 1.0] {
        let mut trap = LedgeTrap::default();
        let ctx = LedgeContext {
            position: [side * 8.0, 0.0],
            facing: -side,
            motion_kind: kirby_assist_core::timing::motion_hash("wait"),
            ..context()
        };
        trap.sample(s, 0, ctx, Some(enemy(CLIFF_WAIT, 40.0)));
        let drop = Opponent {
            status: 0xe,
            situation: 2,
            position: [0.0, -4.0],
            ..enemy(0xe, 0.0)
        };
        let jump = trap.sample(s, 1, ctx, Some(drop));
        assert_eq!(jump.requested, MAPPER_X);
        assert_eq!(jump.stick, Some([0.0, 0.0]));
        let squat = LedgeContext {
            status: 0xa,
            can_act: false,
            can_jump: false,
            motion_kind: kirby_assist_core::timing::motion_hash("jump_squat"),
            ..ctx
        };
        let attack = trap.sample(s, 2, squat, None);
        assert_eq!(attack.requested, MAPPER_A);
        assert_eq!(attack.stick, Some([-side * 0.6, -1.0]));
        let accepted = LedgeContext {
            status: 0x36,
            situation: 2,
            motion_kind: kirby_assist_core::timing::motion_hash("attack_air_lw"),
            ..squat
        };
        let drift = trap.sample(s, 3, accepted, None);
        assert_eq!(drift.requested, 0);
        assert_eq!(drift.stick, Some([-side, 0.0]));
    }
}

#[test]
fn kirby_roll_grab_runs_to_twenty_units_and_waits_on_both_sides() {
    let catalog = FighterProfiles::default();
    for (kind, inset) in [(6, 20.0)] {
        let profile = catalog.profile(kind).unwrap();
        let mut selection = MenuSelection::new(&profile.menu_options);
        selection.choices[2] = 1; // Roll -> Grab.
        let s = selection.settings(&profile.menu_options, profile.apply(settings(kind, 0)));
        for inward in [-1.0, 1.0] {
            let mut trap = LedgeTrap::default();
            let ctx = LedgeContext {
                position: [inward * 6.0, 0.0],
                facing: inward,
                ..context()
            };
            trap.sample(s, 0, ctx, Some(enemy(CLIFF_WAIT, 40.0)));
            let roll = Opponent {
                position: [inward * 12.0, 0.0],
                velocity: [inward * 3.0, 0.0],
                ..enemy(kirby_assist_core::ledge_trap::CLIFF_ESCAPE, 30.0)
            };
            let approach = trap.sample(s, 1, ctx, Some(roll));
            assert_eq!(approach.stick, Some([inward, 0.0]));
            assert_eq!(
                approach.owned & kirby_assist_core::manual_controls::LEFT_STICK_DIRECTIONS,
                0,
                "the dash must retain horizontal flick aliases"
            );
            assert_eq!(approach.requested, 0);
            let running = LedgeContext {
                status: 0x4,
                can_act: false,
                position: [inward * (inset - 8.0), 0.0],
                velocity: [inward * 0.5, 0.0],
                ..ctx
            };
            assert_eq!(
                trap.sample(s, 2, running, Some(roll)).stick,
                Some([inward, 0.0])
            );
            let braking = LedgeContext {
                position: [inward * (inset - 4.0), 0.0],
                velocity: [inward * 2.0, 0.0],
                ..running
            };
            assert_eq!(
                trap.sample(s, 3, braking, Some(roll)).stick,
                Some([0.0, 0.0])
            );
            let near = LedgeContext {
                position: [inward * (inset - 2.5), 0.0],
                ..ctx
            };
            assert_eq!(
                trap.sample(s, 4, near, Some(roll)).stick,
                Some([inward * 0.3, 0.0])
            );
            // A deeper moving opponent must not extend the fixed destination.
            let arrived = LedgeContext {
                position: [inward * (inset - 1.0), 0.0],
                ..ctx
            };
            let deeper = Opponent {
                position: [inward * 42.0, 0.0],
                ..roll
            };
            let waiting = trap.sample(s, 5, arrived, Some(deeper));
            assert_eq!(waiting.stick, Some([0.0, 0.0]));
            assert_eq!(waiting.requested, 0);
        }
    }
}

#[test]
fn yoshi_tracked_roll_grab_takes_legal_catch_before_destination() {
    let catalog = FighterProfiles::default();
    for kind in [5] {
        let profile = catalog.profile(kind).unwrap();
        let mut selection = MenuSelection::new(&profile.menu_options);
        selection.choices[2] = 1;
        let s = selection.settings(&profile.menu_options, profile.apply(settings(kind, 0)));
        for inward in [-1.0, 1.0] {
            let mut trap = LedgeTrap::default();
            let ctx = LedgeContext {
                position: [inward * 6.0, 0.0],
                facing: inward,
                motion_kind: kirby_assist_core::timing::motion_hash("wait"),
                ..context()
            };
            trap.sample(s, 0, ctx, Some(enemy(CLIFF_WAIT, 40.0)));
            let roll = Opponent {
                position: [inward * 11.0, 0.0],
                ..enemy(kirby_assist_core::ledge_trap::CLIFF_ESCAPE, 30.0)
            };
            assert_eq!(
                trap.sample(s, 1, ctx, Some(roll)).stick,
                Some([inward * 0.65, 0.0])
            );
            let opens = f32::from(s.fighter.grab.standing) - 3.0;
            let legal = Opponent {
                vulnerable_in: Some(opens),
                ..roll
            };
            let released = trap.sample(
                s,
                2,
                LedgeContext {
                    motion_frame: 1.0,
                    ..ctx
                },
                Some(legal),
            );
            assert_eq!(released.stick, Some([0.0, 0.0]));
            assert_eq!(released.requested, 0);
            let grab = trap.sample(
                s,
                3,
                LedgeContext {
                    motion_frame: 2.0,
                    ..ctx
                },
                Some(Opponent {
                    vulnerable_in: Some(opens - 1.0),
                    ..legal
                }),
            );
            assert_ne!(
                grab.requested & kirby_assist_core::native_input::MAPPER_R,
                0,
                "tracked Yoshi grab must not wait for a fixed stance"
            );
        }
    }
}

#[test]
fn kirby_fixed_roll_grab_does_not_grab_before_arrival_even_with_local_contact() {
    let catalog = FighterProfiles::default();
    let profile = catalog.profile(6).unwrap();
    let mut selection = MenuSelection::new(&profile.menu_options);
    selection.choices[2] = 1;
    let s = selection.settings(&profile.menu_options, profile.apply(settings(6, 0)));
    for inward in [-1.0, 1.0] {
        let mut trap = LedgeTrap::default();
        let ctx = LedgeContext {
            position: [inward * 6.0, 0.0],
            facing: inward,
            ..context()
        };
        trap.sample(s, 0, ctx, Some(enemy(CLIFF_WAIT, 40.0)));
        let roll = Opponent {
            position: [inward * 11.0, 0.0],
            vulnerable_in: Some(f32::from(s.fighter.grab.standing) - 3.0),
            ..enemy(kirby_assist_core::ledge_trap::CLIFF_ESCAPE, 30.0)
        };
        // The same nearby contact allowed an early catch in the old experiment.
        for tick in 1..4 {
            let out = trap.sample(
                s,
                tick,
                LedgeContext {
                    motion_frame: tick as f32,
                    ..ctx
                },
                Some(roll),
            );
            assert_eq!(out.requested, 0, "must first reach the 20-unit stance");
        }
    }
}

#[test]
fn walks_during_wait_but_launches_on_existing_deadline_even_when_still_far_away() {
    let s = settings(6, 0);
    let mut trap = LedgeTrap::default();
    trap.sample(s, 0, context(), Some(enemy(CLIFF_WAIT, 40.)));
    let walk = trap.sample(s, 1, context(), Some(enemy(CLIFF_CLIMB, 30.)));
    assert_eq!(walk.stick, Some([-0.65, 0.]));
    assert_eq!(walk.requested, 0);
    assert!(
        !trap.committed_input_pending(),
        "walk preparation is not a committed input"
    );
    let launch = trap.sample(s, 2, context(), Some(enemy(CLIFF_CLIMB, 21.)));
    assert_eq!(
        launch.requested & (MAPPER_X | MAPPER_A),
        MAPPER_X | MAPPER_A
    );
    assert_eq!(launch.stick, Some([0., 0.]));
    assert!(trap.committed_input_pending());
}

#[test]
fn stops_at_inset_brakes_for_momentum_and_refuses_unsafe_ground() {
    let s = settings(6, 0);
    for ctx in [
        LedgeContext {
            position: [6., 0.],
            ..context()
        },
        LedgeContext {
            position: [10., 0.],
            velocity: [-1., 0.],
            ..context()
        },
        LedgeContext {
            safe_ground: [false, true],
            ..context()
        },
    ] {
        let mut trap = LedgeTrap::default();
        trap.sample(s, 0, context(), Some(enemy(CLIFF_WAIT, 40.)));
        let out = trap.sample(s, 1, ctx, Some(enemy(CLIFF_CLIMB, 30.)));
        assert_eq!(out.stick, Some([0., 0.]));
        assert_eq!(out.requested, 0);
    }
}

#[test]
fn mirrors_toward_right_ledge_and_leaves_other_choices_alone() {
    let s = settings(6, 0);
    let ctx = LedgeContext {
        position: [-20., 0.],
        facing: 1.,
        ..context()
    };
    let mut trap = LedgeTrap::default();
    trap.sample(s, 0, ctx, Some(enemy(CLIFF_WAIT, 40.)));
    assert_eq!(
        trap.sample(s, 1, ctx, Some(enemy(CLIFF_CLIMB, 30.))).stick,
        Some([0.65, 0.])
    );
    for (kind, choice) in [(0, 0), (6, 1)] {
        let s = settings(kind, choice);
        let mut trap = LedgeTrap::default();
        trap.sample(s, 0, context(), Some(enemy(CLIFF_WAIT, 40.)));
        let out = trap.sample(s, 1, context(), Some(enemy(CLIFF_CLIMB, 30.)));
        assert_ne!(out.stick, Some([-0.65, 0.]));
    }
}

#[test]
fn yoshi_neutral_nair_pairs_attack_without_waiting_for_protection_or_online_polls() {
    for opens in [None, Some(30.0), Some(100.0)] {
        let mut s = settings(5, 0);
        s.aerial_execution.neutral = kirby_assist_core::fighter_profiles::execution::Hop::Full;
        let ctx = LedgeContext {
            motion_kind: kirby_assist_core::timing::motion_hash("wait"),
            ..context()
        };
        let mut trap = LedgeTrap::default();
        trap.sample(s, 0, ctx, Some(enemy(CLIFF_WAIT, 40.)));
        let target = Opponent {
            vulnerable_in: opens,
            ..enemy(CLIFF_CLIMB, 30.)
        };
        let launch = trap.sample(s, 1, ctx, Some(target));
        assert_eq!(launch.requested, MAPPER_X | MAPPER_A);
        assert_eq!(launch.stick, Some([0., 0.]));
        for tick in 2..40 {
            assert_eq!(trap.sample(s, tick, ctx, Some(target)), launch);
        }
        let squat = LedgeContext {
            status: 0xa,
            motion_kind: kirby_assist_core::timing::motion_hash("jump_squat"),
            can_act: false,
            ..ctx
        };
        assert_eq!(trap.sample(s, 40, squat, None).requested, MAPPER_A);
        let air = LedgeContext {
            status: 0xb,
            situation: 2,
            motion_kind: kirby_assist_core::timing::motion_hash("jump_f"),
            can_act: false,
            ..ctx
        };
        assert_eq!(trap.sample(s, 41, air, None).requested, MAPPER_A);
        let landed = trap.sample(
            s,
            42,
            LedgeContext {
                motion_frame: 1.,
                ..ctx
            },
            None,
        );
        assert_eq!(landed.requested, 0);
        assert_eq!(
            trap.sample(
                s,
                43,
                LedgeContext {
                    motion_frame: 2.,
                    ..ctx
                },
                Some(target)
            )
            .requested,
            0
        );
    }
}

#[test]
fn yoshi_both_nairs_confirm_ledge_facing_before_launch_on_either_side() {
    for direction in [-1., 1.] {
        for option in [CLIFF_CLIMB, CLIFF_JUMP1] {
            let s = settings(5, 0);
            let mut trap = LedgeTrap::default();
            let ctx = LedgeContext {
                position: [-direction * 20., 0.],
                facing: -direction,
                motion_kind: kirby_assist_core::timing::motion_hash("wait"),
                ..context()
            };
            trap.sample(s, 0, ctx, Some(enemy(CLIFF_WAIT, 40.)));
            let turn = trap.sample(s, 1, ctx, Some(enemy(option, 100.)));
            assert_eq!(turn.requested, 0);
            assert_eq!(turn.stick, Some([direction * 0.6, 0.]));
            let launch = trap.sample(
                s,
                2,
                LedgeContext {
                    facing: direction,
                    status: 0x7,
                    can_act: false,
                    ..ctx
                },
                Some(enemy(option, 99.)),
            );
            assert_eq!(launch.requested, MAPPER_X | MAPPER_A);
            assert_eq!(launch.stick, Some([0., 0.]));
        }
    }
}

#[test]
fn yoshi_both_nairs_use_brief_full_stick_nudge_then_release() {
    for option in [CLIFF_CLIMB, CLIFF_JUMP1] {
        let s = settings(5, 0);
        let ctx = LedgeContext {
            motion_kind: kirby_assist_core::timing::motion_hash("wait"),
            ..context()
        };
        let mut trap = LedgeTrap::default();
        trap.sample(s, 0, ctx, Some(enemy(CLIFF_WAIT, 40.)));
        trap.sample(s, 1, ctx, Some(enemy(option, 22.)));
        let air = LedgeContext {
            status: 0x36,
            situation: 2,
            motion_kind: kirby_assist_core::timing::motion_hash("attack_air_n"),
            motion_frame: 1.,
            velocity: [0., 0.5],
            ..ctx
        };
        let pulse = trap.sample(s, 2, air, Some(enemy(option, 21.)));
        assert_eq!(pulse.stick, Some([-1., 0.]), "option {option:#x}");
        for frame in 2..4 {
            let tail = trap.sample(
                s,
                frame + 1,
                LedgeContext {
                    motion_frame: frame as f32,
                    ..air
                },
                Some(enemy(option, 0.)),
            );
            assert_eq!(
                tail.stick,
                Some([0., 0.]),
                "option {option:#x}, frame {frame}"
            );
        }
        let released = trap.sample(
            s,
            5,
            LedgeContext {
                motion_frame: 4.,
                ..air
            },
            Some(enemy(option, 20.)),
        );
        assert_eq!(
            released.stick.unwrap_or([0., 0.]),
            [0., 0.],
            "option {option:#x}"
        );
    }
}

#[test]
fn yoshi_neutral_down_smash_is_appended_and_releases_charge_on_startup() {
    let catalog = FighterProfiles::default();
    let options = catalog.profile(5).unwrap().menu_options.lists[0].options;
    assert_eq!(options[3].id, "down_smash");
    let s = settings(5, 3);
    let ctx = LedgeContext {
        motion_kind: kirby_assist_core::timing::motion_hash("wait"),
        ..context()
    };
    let mut trap = LedgeTrap::default();
    trap.sample(s, 0, ctx, Some(enemy(CLIFF_WAIT, 40.)));
    let early = trap.sample(s, 1, ctx, Some(enemy(CLIFF_CLIMB, 30.)));
    assert_eq!(early.requested, 0);
    // Front activity 7..8 plus one neutral primer update.
    let primer = trap.sample(s, 2, ctx, Some(enemy(CLIFF_CLIMB, 8.)));
    assert_eq!(primer.requested, 0);
    let press = trap.sample(
        s,
        3,
        LedgeContext {
            motion_frame: 1.,
            ..ctx
        },
        Some(enemy(CLIFF_CLIMB, 7.)),
    );
    assert_eq!(press.requested, MAPPER_A);
    assert_eq!(press.stick, Some([0., -1.]));
    let release = trap.sample(
        s,
        4,
        LedgeContext {
            status: 0x30,
            motion_frame: 0.,
            ..ctx
        },
        Some(enemy(CLIFF_CLIMB, 6.)),
    );
    assert_eq!(release.requested, 0);
}

#[test]
fn yoshi_neutral_nair_keeps_the_same_descent_as_jump_nair() {
    let s = settings(5, 0);
    let ctx = LedgeContext {
        motion_kind: kirby_assist_core::timing::motion_hash("wait"),
        ..context()
    };
    let mut trap = LedgeTrap::default();
    trap.sample(s, 0, ctx, Some(enemy(CLIFF_WAIT, 40.)));
    trap.sample(s, 1, ctx, Some(enemy(CLIFF_CLIMB, 22.)));
    let air = LedgeContext {
        status: 0x36,
        situation: 2,
        motion_kind: kirby_assist_core::timing::motion_hash("attack_air_n"),
        motion_frame: 14.,
        velocity: [0., 0.5],
        ..ctx
    };
    let rising = trap.sample(s, 2, air, Some(enemy(CLIFF_CLIMB, 5.)));
    assert_ne!(rising.stick, Some([0., -1.]));
    let falling = trap.sample(
        s,
        3,
        LedgeContext {
            motion_frame: 15.,
            velocity: [0., -0.5],
            ..air
        },
        Some(enemy(CLIFF_CLIMB, 4.)),
    );
    assert_eq!(falling.stick, Some([0., 0.]));
    assert_eq!(falling.requested, 0);
}

#[test]
fn yoshi_neutral_dair_turns_then_short_hops_without_timing_gate() {
    let catalog = FighterProfiles::default();
    let options = catalog.profile(5).unwrap().menu_options.lists[0].options;
    let ids: Vec<_> = options.iter().map(|choice| choice.id).collect();
    assert_eq!(
        ids,
        [
            "nair",
            "down_tilt",
            "forward_smash",
            "down_smash",
            "down_air"
        ]
    );
    for direction in [-1., 1.] {
        let s = settings(5, 4);
        let ctx = LedgeContext {
            position: [-direction * 20., 0.],
            facing: -direction,
            motion_kind: kirby_assist_core::timing::motion_hash("wait"),
            ..context()
        };
        let mut trap = LedgeTrap::default();
        trap.sample(s, 0, ctx, Some(enemy(CLIFF_WAIT, 40.)));
        let target = Opponent {
            vulnerable_in: None,
            actionable_in: None,
            ..enemy(CLIFF_CLIMB, 100.)
        };
        let turn = trap.sample(s, 1, ctx, Some(target));
        assert_eq!(turn.requested, 0);
        assert_eq!(turn.stick, Some([direction * 0.6, 0.]));
        let facing = LedgeContext {
            status: 0x7,
            can_act: false,
            facing: direction,
            ..ctx
        };
        let jump = trap.sample(s, 2, facing, Some(target));
        assert_eq!(jump.requested, MAPPER_X | MAPPER_ZL);
        assert_eq!(jump.stick, Some([0., 0.]));
        let dair = trap.sample(
            s,
            3,
            LedgeContext {
                status: 0xa,
                ..facing
            },
            Some(target),
        );
        assert_eq!(dair.requested, MAPPER_A);
        assert_eq!(dair.stick, Some([0., -1.]));
        let dair = trap.sample(
            s,
            4,
            LedgeContext {
                status: 0xb,
                situation: 2,
                motion_frame: 1.0,
                ..facing
            },
            Some(target),
        );
        assert_eq!(dair.requested, MAPPER_A);
        assert_eq!(dair.stick, Some([0., -1.]));
    }
}

#[test]
fn kirby_jump_preserves_spacing_during_wait_and_launches_immediately() {
    let s = settings(6, 0);
    let airborne = |status, remaining| Opponent {
        situation: 2,
        ..enemy(status, remaining)
    };
    let mut trap = LedgeTrap::default();
    trap.sample(s, 0, context(), Some(enemy(CLIFF_WAIT, 40.)));
    let walk = trap.sample(s, 1, context(), Some(airborne(CLIFF_JUMP3, 100.)));
    assert_eq!(walk.requested, 0);
    assert_eq!(walk.stick, None);

    let mut immediate = LedgeTrap::default();
    immediate.sample(s, 0, context(), Some(enemy(CLIFF_WAIT, 40.)));
    let launch = immediate.sample(s, 1, context(), Some(airborne(CLIFF_JUMP1, 40.)));
    assert_eq!(
        launch.requested & (MAPPER_X | MAPPER_A),
        MAPPER_X | MAPPER_A
    );
    assert_eq!(launch.stick, Some([0., 0.]));
}

#[test]
fn kirby_neutral_bair_turns_without_any_ground_attack_then_buffers_bair() {
    let s = settings(6, 5);
    for side in [-1., 1.] {
        for already_away in [false, true] {
            let ctx = LedgeContext {
                position: [side * 20., 0.],
                facing: if already_away { side } else { -side },
                motion_kind: kirby_assist_core::timing::motion_hash("wait"),
                ..context()
            };
            let mut trap = LedgeTrap::default();
            trap.sample(s, 0, ctx, Some(enemy(CLIFF_WAIT, 40.)));
            let prepare = trap.sample(s, 1, ctx, Some(enemy(CLIFF_CLIMB, 30.)));
            assert_eq!(prepare.requested, 0);
            assert_eq!(
                prepare.stick,
                if already_away {
                    None
                } else {
                    Some([side * 0.6, 0.])
                }
            );
            let back = LedgeContext {
                facing: side,
                status: 0x7,
                can_act: false,
                ..ctx
            };
            let hop = trap.sample(s, 2, back, Some(enemy(CLIFF_CLIMB, 1.)));
            assert_eq!(hop.requested, MAPPER_X | MAPPER_ZL);
            assert_eq!(hop.stick, Some([0., 0.]));
            let bair = trap.sample(
                s,
                3,
                LedgeContext {
                    status: 0xa,
                    ..back
                },
                Some(enemy(CLIFF_CLIMB, 0.)),
            );
            assert_eq!(bair.requested, 0);
            let bair = trap.sample(
                s,
                4,
                LedgeContext {
                    status: 0xb,
                    situation: 2,
                    motion_frame: 1.0,
                    ..back
                },
                Some(enemy(CLIFF_CLIMB, 0.)),
            );
            assert_eq!(bair.requested, MAPPER_A);
            assert_eq!(bair.stick, Some([-side, 0.]));
        }
    }
}

#[test]
fn inhale_turns_toward_ledge_before_neutral_b_from_either_side() {
    let s = settings(6, 4);
    for side in [-1.0, 1.0] {
        let backwards = LedgeContext {
            position: [side * 20., 0.],
            facing: side,
            motion_kind: kirby_assist_core::timing::motion_hash("wait"),
            ..context()
        };
        let mut trap = LedgeTrap::default();
        trap.sample(s, 0, backwards, Some(enemy(CLIFF_WAIT, 40.)));
        let turn = trap.sample(s, 1, backwards, Some(enemy(CLIFF_CLIMB, 30.)));
        assert_eq!(turn.stick, Some([-side * 0.6, 0.]));
        assert_eq!(turn.requested, 0);
        let facing_ledge = LedgeContext {
            facing: -side,
            motion_frame: 1.,
            ..backwards
        };
        let inhale = trap.sample(s, 2, facing_ledge, Some(enemy(CLIFF_CLIMB, 1.)));
        assert_eq!(inhale.requested & MAPPER_B, MAPPER_B);
        assert_eq!(inhale.stick, Some([0., 0.]));
    }
}

#[test]
fn jump_bair_turns_without_tilt_then_pairs_jump_and_buffers_back_air() {
    let mut s = settings(6, 0);
    s.response_choices[1] = Some(1); // Kirby Jump -> Bair.
    for side in [-1.0, 1.0] {
        let facing_ledge = LedgeContext {
            position: [side * 20., 0.],
            facing: -side,
            motion_kind: kirby_assist_core::timing::motion_hash("wait"),
            ..context()
        };
        let jumping = Opponent {
            situation: 2,
            ..enemy(CLIFF_JUMP1, 30.)
        };
        let mut trap = LedgeTrap::default();
        trap.sample(s, 0, facing_ledge, Some(enemy(CLIFF_WAIT, 40.)));
        let turn = trap.sample(s, 1, facing_ledge, Some(jumping));
        assert_eq!(turn.requested, 0, "no grounded tilt or premature jump");
        assert_eq!(turn.stick, Some([side * 0.6, 0.]));
        let turned = LedgeContext {
            facing: side,
            status: 6,
            can_act: false,
            motion_frame: 1.,
            ..facing_ledge
        };
        let jump = trap.sample(s, 2, turned, Some(jumping));
        assert_eq!(jump.requested, MAPPER_X | MAPPER_ZL);
        assert_eq!(jump.stick, Some([0., 0.]));
        let squat = LedgeContext {
            status: 0xa,
            motion_frame: 2.,
            ..turned
        };
        let bair = trap.sample(s, 3, squat, Some(jumping));
        assert_eq!(bair.requested, MAPPER_A);
        assert_eq!(bair.stick, Some([-side, 0.]));
        let bair = trap.sample(
            s,
            4,
            LedgeContext {
                status: 0xb,
                situation: 2,
                motion_frame: 3.0,
                ..turned
            },
            Some(jumping),
        );
        assert_eq!(bair.requested, MAPPER_A);
        assert_eq!(bair.stick, Some([-side, 0.]));
    }
}

#[test]
fn neutral_down_smash_starts_two_frames_earlier_and_aligns_expiry_at_frame_fifteen() {
    let catalog = FighterProfiles::default();
    let options = catalog.profile(6).unwrap().menu_options.lists[0].options;
    assert_eq!(options[2].name, "Down smash");
    let s = settings(6, 2);
    let ctx = LedgeContext {
        motion_kind: kirby_assist_core::timing::motion_hash("wait"),
        ..context()
    };
    let mut trap = LedgeTrap::default();
    trap.sample(s, 0, ctx, Some(enemy(CLIFF_WAIT, 40.)));
    let wait = trap.sample(s, 1, ctx, Some(enemy(CLIFF_CLIMB, 17.)));
    assert_eq!(wait.requested, 0);
    assert_eq!(wait.stick, None);
    let prime = trap.sample(
        s,
        2,
        LedgeContext {
            motion_frame: 1.,
            ..ctx
        },
        Some(enemy(CLIFF_CLIMB, 16.)),
    );
    assert_eq!(prime.requested, 0);
    assert_eq!(prime.stick, Some([0., 0.]));
    let smash = trap.sample(
        s,
        3,
        LedgeContext {
            motion_frame: 2.,
            ..ctx
        },
        Some(enemy(CLIFF_CLIMB, 15.)),
    );
    assert_eq!(smash.requested, MAPPER_A);
    assert_eq!(smash.stick, Some([0., -1.]));
    let started = trap.sample(
        s,
        4,
        LedgeContext {
            status: 0x30,
            motion_frame: 1.,
            ..ctx
        },
        Some(enemy(CLIFF_CLIMB, 14.)),
    );
    assert_eq!(
        started.requested, 0,
        "release on native startup, no charging"
    );
}

#[test]
fn kirby_down_smash_walk_never_delays_deadline_for_spacing_or_momentum() {
    let mut s = settings(6, 2);
    s.manual_spacing = false;
    s.distance_agnostic = false;
    s.horizontal_range = 128.;
    s.follow_range = 128.;
    let ctx = LedgeContext {
        position: [80., 0.],
        velocity: [-1., 0.],
        motion_kind: kirby_assist_core::timing::motion_hash("wait"),
        ..context()
    };
    let mut trap = LedgeTrap::default();
    trap.sample(s, 0, ctx, Some(enemy(CLIFF_WAIT, 40.)));
    let walk = trap.sample(s, 1, ctx, Some(enemy(CLIFF_CLIMB, 30.)));
    assert_eq!(walk.stick, Some([-0.65, 0.]));
    assert_ne!(
        walk.owned & kirby_assist_core::manual_controls::LEFT_STICK_DIRECTIONS,
        0
    );
    let frame = kirby_assist_core::manual_controls::ManualOverlay::default().apply_controls(
        kirby_assist_core::native_input::ButtonTrace {
            held: (1 << 21) | (1 << 23),
            ..Default::default()
        },
        [1., 0.],
        kirby_assist_core::manual_controls::TurboOutput {
            stick: walk.stick,
            ..Default::default()
        },
        walk.owned & ((1 << 21) | (1 << 23)),
        walk.requested,
        walk.owned,
    );
    assert_eq!(frame.stick, [-0.65, 0.]);
    assert_eq!(frame.buttons.held & ((1 << 21) | (1 << 23)), 0);
    let prime = trap.sample(
        s,
        2,
        LedgeContext {
            motion_frame: 1.,
            ..ctx
        },
        Some(enemy(CLIFF_CLIMB, 16.)),
    );
    assert_eq!(prime.stick, Some([0., 0.]));
    let smash = trap.sample(
        s,
        3,
        LedgeContext {
            motion_frame: 2.,
            ..ctx
        },
        Some(enemy(CLIFF_CLIMB, 15.)),
    );
    assert_eq!(smash.requested, MAPPER_A);
    assert_eq!(smash.stick, Some([0., -1.]));
}

#[test]
fn kirby_jump_nair_restores_brief_nudge_without_drifting_through_activity() {
    for side in [-1., 1.] {
        let s = settings(6, 0);
        let ctx = LedgeContext {
            position: [side * 20., 0.],
            facing: -side,
            motion_kind: kirby_assist_core::timing::motion_hash("wait"),
            ..context()
        };
        let mut trap = LedgeTrap::default();
        trap.sample(s, 0, ctx, Some(enemy(CLIFF_WAIT, 40.)));
        let jumping = Opponent {
            situation: 2,
            ..enemy(CLIFF_JUMP1, 40.)
        };
        let launch = trap.sample(s, 1, ctx, Some(jumping));
        assert_eq!(launch.stick, Some([0., 0.]));
        for frame in [1., 12., 32.] {
            let air = LedgeContext {
                status: 0x36,
                situation: 2,
                motion_kind: kirby_assist_core::timing::motion_hash("attack_air_n"),
                motion_frame: frame,
                ..ctx
            };
            let drift = trap.sample(s, frame as i64 + 2, air, Some(jumping));
            assert_eq!(
                drift.stick,
                Some([if frame < 4. { -side * 0.25 } else { 0. }, 0.])
            );
        }
        let ended = trap.sample(
            s,
            36,
            LedgeContext {
                status: 0x36,
                situation: 2,
                motion_kind: kirby_assist_core::timing::motion_hash("attack_air_n"),
                motion_frame: 33.,
                ..ctx
            },
            Some(jumping),
        );
        assert_eq!(ended.stick, Some([0., 0.]));
    }
}

#[test]
fn samus_roll_grab_turns_inward_immediately_from_either_ledge() {
    let s = settings(3, 0); // Samus defaults to Roll -> Grab.
    for side in [-1.0, 1.0] {
        let outward = LedgeContext {
            position: [side * 20., 0.],
            facing: -side,
            motion_kind: kirby_assist_core::timing::motion_hash("wait"),
            ..context()
        };
        let mut trap = LedgeTrap::default();
        trap.sample(s, 0, outward, Some(enemy(CLIFF_WAIT, 40.)));
        let roll = enemy(kirby_assist_core::ledge_trap::CLIFF_ESCAPE, 30.);
        let turn = trap.sample(s, 1, outward, Some(roll));
        assert_eq!(turn.stick, Some([side * 0.6, 0.]));
        assert_eq!(turn.requested, 0, "never grab toward the ledge");
        let inward = LedgeContext {
            facing: side,
            motion_frame: 1.,
            ..outward
        };
        let next = trap.sample(s, 2, inward, Some(roll));
        assert_ne!(next.stick, Some([-side * 0.6, 0.]));
        assert_ne!(
            next.stick,
            Some([side * 0.6, 0.]),
            "stop explicit turning after facing confirms"
        );
    }
}

#[test]
fn samus_homing_pairs_side_b_immediately_for_all_four_rows_and_both_facings() {
    for kind in [3, 4] {
        for (row, choice, status, situation) in [
            (0, 4, CLIFF_CLIMB, 0),
            (1, 4, CLIFF_JUMP1, 2),
            (2, 3, kirby_assist_core::ledge_trap::CLIFF_ESCAPE, 0),
            (4, 4, 0xe, 2),
        ] {
            let catalog = FighterProfiles::default();
            let profile = catalog.profile(kind).unwrap();
            let mut menu = MenuSelection::new(&profile.menu_options);
            menu.choices[row] = choice;
            let s = menu.settings(
                &profile.menu_options,
                profile.apply(LedgeSettings {
                    enabled: true,
                    manual_spacing: true,
                    distance_agnostic: true,
                    ..LedgeSettings::default()
                }),
            );
            for side in [-1.0, 1.0] {
                for facing in [-1.0, 1.0] {
                    let ctx = LedgeContext {
                        position: [side * 20., 0.],
                        facing,
                        motion_kind: kirby_assist_core::timing::motion_hash("wait"),
                        ..context()
                    };
                    let mut trap = LedgeTrap::default();
                    trap.sample(s, 0, ctx, Some(enemy(CLIFF_WAIT, 40.)));
                    let target = Opponent {
                        situation,
                        vulnerable_in: None,
                        actionable_in: Some(1.),
                        ..enemy(status, 40.)
                    };
                    let fire = trap.sample(s, 1, ctx, Some(target));
                    assert_eq!(
                        fire.requested, MAPPER_B,
                        "kind={kind}, row={row}, side={side}, facing={facing}"
                    );
                    assert_eq!(fire.stick, Some([-side * 0.70, 0.]));
                    // Duplicate native samples preserve paired B+direction.
                    assert_eq!(trap.sample(s, 2, ctx, Some(target)).stick, fire.stick);
                    let accepted = trap.sample(
                        s,
                        3,
                        LedgeContext {
                            status: 0x1eb,
                            motion_frame: 1.,
                            ..ctx
                        },
                        Some(target),
                    );
                    assert_eq!(accepted.requested, 0);
                }
            }
        }
    }
}

#[test]
fn samus_super_missile_still_uses_neutral_primer_and_full_flick() {
    let s = settings(3, 5);
    let ctx = LedgeContext {
        motion_kind: kirby_assist_core::timing::motion_hash("wait"),
        ..context()
    };
    let mut trap = LedgeTrap::default();
    trap.sample(s, 0, ctx, Some(enemy(CLIFF_WAIT, 40.)));
    let prime = trap.sample(s, 1, ctx, Some(enemy(CLIFF_CLIMB, 40.)));
    assert_eq!(prime.requested, 0);
    let fire = trap.sample(
        s,
        2,
        LedgeContext {
            motion_frame: 1.,
            ..ctx
        },
        Some(enemy(CLIFF_CLIMB, 39.)),
    );
    assert_eq!(fire.requested, MAPPER_B);
    assert_eq!(fire.stick, Some([-1., 0.]));
}

#[test]
fn pacman_stored_fruit_throws_on_neutral_or_jump_recognition_without_timing_gate() {
    let catalog = FighterProfiles::default();
    let profile = catalog.profile(55).unwrap();
    for (row, status) in [(0, CLIFF_CLIMB), (1, CLIFF_JUMP1)] {
        let mut menu = MenuSelection::new(&profile.menu_options);
        menu.choices[row] = profile.menu_options.lists[row]
            .options
            .iter()
            .position(|choice| choice.id == "fruit_throw")
            .unwrap();
        let s = menu.settings(
            &profile.menu_options,
            profile.apply(LedgeSettings {
                enabled: true,
                manual_spacing: true,
                distance_agnostic: true,
                ..LedgeSettings::default()
            }),
        );
        for side in [-1.0, 1.0] {
            let ctx = LedgeContext {
                position: [side * 20., 0.],
                facing: -side,
                motion_kind: kirby_assist_core::timing::motion_hash("wait"),
                ..context()
            };
            let mut trap = LedgeTrap::default();
            trap.sample(s, 0, ctx, Some(enemy(CLIFF_WAIT, 40.)));
            let target = Opponent {
                situation: if row == 1 { 2 } else { 0 },
                vulnerable_in: None,
                actionable_in: Some(1.),
                ..enemy(status, 40.)
            };
            let fire = trap.sample(s, 1, ctx, Some(target));
            assert_eq!(fire.requested, MAPPER_B);
            assert_eq!(fire.stick, Some([0., 0.]), "neutral B, no side special");
            // Entry releases B rather than holding it into a fruit recharge.
            let accepted = trap.sample(
                s,
                2,
                LedgeContext {
                    status: 0x1dc,
                    motion_frame: 1.,
                    ..ctx
                },
                Some(target),
            );
            assert_eq!(accepted.requested, 0);
            assert!(trap.manual_after_response());
            let later = trap.sample(s, 3, ctx, Some(target));
            assert_eq!(later.requested, 0, "one throw per held-trigger attempt");
        }
    }
}

#[test]
fn pacman_fruit_faces_opponent_before_sending_neutral_b() {
    let s = settings(55, 4);
    let ctx = LedgeContext {
        facing: 1.,
        motion_kind: kirby_assist_core::timing::motion_hash("wait"),
        ..context()
    };
    let mut trap = LedgeTrap::default();
    trap.sample(s, 0, ctx, Some(enemy(CLIFF_WAIT, 40.)));
    let turn = trap.sample(s, 1, ctx, Some(enemy(CLIFF_CLIMB, 40.)));
    assert_eq!(turn.requested, 0);
    assert_eq!(turn.stick, Some([-0.6, 0.]));
    let fire = trap.sample(
        s,
        2,
        LedgeContext {
            facing: -1.,
            motion_frame: 1.,
            ..ctx
        },
        Some(enemy(CLIFF_CLIMB, 39.)),
    );
    assert_eq!(fire.requested, MAPPER_B);
    assert_eq!(fire.stick, Some([0., 0.]));
}

#[test]
fn yoshi_headbutt_turns_during_protection_instead_of_spending_startup_window() {
    let s = settings(5, 2);
    let ctx = LedgeContext {
        facing: 1.,
        motion_kind: kirby_assist_core::timing::motion_hash("wait"),
        ..context()
    };
    let mut trap = LedgeTrap::default();
    trap.sample(s, 0, ctx, Some(enemy(CLIFF_WAIT, 40.)));
    let turn = trap.sample(s, 1, ctx, Some(enemy(CLIFF_CLIMB, 30.)));
    assert_eq!(turn.requested, 0);
    assert_eq!(turn.stick, Some([-0.3, 0.]));
    let turned = LedgeContext {
        facing: -1.,
        motion_frame: 1.,
        ..ctx
    };
    let prime = trap.sample(s, 2, turned, Some(enemy(CLIFF_CLIMB, 17.)));
    assert_eq!(prime.requested, 0);
    let press = trap.sample(
        s,
        3,
        LedgeContext {
            motion_frame: 2.,
            ..turned
        },
        Some(enemy(CLIFF_CLIMB, 16.)),
    );
    assert_eq!(press.requested, MAPPER_A);
    assert_eq!(press.stick, Some([-1., 0.]));
}
