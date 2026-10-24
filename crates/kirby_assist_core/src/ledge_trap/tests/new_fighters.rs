//! Real menu dispatch for the new profiles and their single-move variants.
use super::*;
use crate::{
    fighter_profiles::{presets::PresetSlot, FighterProfiles},
    ledge_menu::MenuSelection,
    timing::motion_hash,
};
fn configured(kind: i32, slot: PresetSlot, id: &str) -> LedgeSettings {
    let catalog = FighterProfiles::default();
    let profile = catalog.profile(kind).unwrap();
    let mut menu = MenuSelection::new(&profile.menu_options);
    menu.choices[slot as usize] = profile
        .menu_options
        .list(slot)
        .options
        .iter()
        .position(|choice| choice.id == id)
        .unwrap();
    menu.settings(
        &profile.menu_options,
        profile.apply(LedgeSettings {
            enabled: true,
            manual_spacing: true,
            distance_agnostic: true,
            ..LedgeSettings::default()
        }),
    )
}
fn own(frame: f32) -> LedgeContext {
    LedgeContext {
        motion_frame: frame,
        motion_kind: motion_hash("wait"),
        ..context()
    }
}

#[test]
fn expanded_roster_up_air_dispatch_buffers_paired_inputs_until_exact_native_acceptance() {
    let catalog = FighterProfiles::default();
    for kind in 0..=0x4c {
        let Some(profile) = catalog.profile(kind) else {
            continue;
        };
        if !profile
            .menu_options
            .list(PresetSlot::Jump)
            .options
            .iter()
            .any(|c| c.id == "up_air")
        {
            continue;
        }
        let config = configured(kind, PresetSlot::Jump, "up_air");
        let target = Opponent {
            situation: 2,
            actionable_in: Some(80.0),
            ..enemy(CLIFF_JUMP3, true)
        };
        let mut trap = LedgeTrap::default();
        let out = trap.sample(config, 0, own(0.0), Some(target));
        assert_eq!(
            out.requested,
            crate::move_input::SHORT_HOP,
            "kind {kind:#x}"
        );
        assert_eq!(out.stick, Some([0.0, 0.0]));
        let squat = LedgeContext {
            status: JUMP_SQUAT,
            stopped: true,
            can_act: false,
            ..own(1.0)
        };
        assert_eq!(trap.sample(config, 1, squat, None).requested, 0);
        let air = LedgeContext {
            status: 0xb,
            situation: 2,
            can_act: false,
            ..own(2.0)
        };
        assert_eq!(trap.sample(config, 2, air, None).requested, MAPPER_A);
        let accepted = LedgeContext {
            status: ATTACK_AIR,
            motion_kind: motion_hash("attack_air_hi"),
            ..air
        };
        assert_eq!(trap.sample(config, 3, accepted, None).requested, 0);
        assert!(trap.fighter_aerial_active());
    }
}

#[test]
fn front_only_roll_smash_waits_for_observed_facing_and_releases_the_turn_before_attacking() {
    let config = configured(0x3c, PresetSlot::Roll, "down_smash");
    let mut trap = LedgeTrap::default();
    let c = LedgeContext {
        position: [25.0, 0.0],
        facing: -1.0,
        ..own(0.0)
    };
    let target = Opponent {
        position: [27.0, 0.0],
        cliff_position: Some([0.0, 0.0]),
        vulnerable_in: Some(5.0),
        actionable_in: Some(15.0),
        ..enemy(CLIFF_ESCAPE, false)
    };
    let turn = trap.sample(config, 0, c, Some(target));
    assert_eq!(turn.requested, 0);
    assert_eq!(turn.stick, Some([0.3, 0.0]));
    assert_eq!(
        trap.sample(
            config,
            1,
            LedgeContext {
                motion_frame: 1.0,
                ..c
            },
            Some(target)
        )
        .requested,
        0
    );
    let facing = LedgeContext {
        facing: 1.0,
        motion_frame: 2.0,
        ..c
    };
    assert_eq!(trap.sample(config, 2, facing, Some(target)).requested, 0);
    let out = trap.sample(
        config,
        3,
        LedgeContext {
            motion_frame: 3.0,
            ..facing
        },
        Some(target),
    );
    assert_eq!(out.requested, MAPPER_A);
    assert_eq!(out.stick, Some([0.0, -1.0]));
}

#[test]
fn olimar_roll_body_tilt_does_not_require_a_smash_or_grab_transition() {
    let config = configured(0x2b, PresetSlot::Roll, "down_tilt");
    let mut trap = LedgeTrap::default();
    let c = LedgeContext {
        position: [25.0, 0.0],
        can_down_smash: false,
        can_grab: false,
        ..own(0.0)
    };
    let target = Opponent {
        position: [27.0, 0.0],
        cliff_position: Some([0.0, 0.0]),
        vulnerable_in: Some(6.0),
        actionable_in: Some(16.0),
        ..enemy(CLIFF_ESCAPE, false)
    };
    let out = trap.sample(config, 0, c, Some(target));
    assert_eq!(out.requested, MAPPER_A);
    assert_eq!(out.stick, Some([0.0, -0.6]));
    let started = LedgeContext {
        status: DOWN_TILT,
        can_act: false,
        motion_frame: 1.0,
        ..c
    };
    assert_eq!(trap.sample(config, 1, started, Some(target)).requested, 0);
}
#[test]
fn up_air_options_pair_jump_attack_and_survive_target_loss_through_acceptance() {
    for kind in [5, 0x45, 0x46] {
        let config = configured(kind, PresetSlot::Jump, "up_air");
        let mut trap = LedgeTrap::default();
        let target = Opponent {
            situation: 2,
            vulnerable_in: Some(0.0),
            actionable_in: Some(40.0),
            ..enemy(CLIFF_JUMP1, true)
        };
        let out = trap.sample(config, 0, own(0.0), Some(target));
        assert_eq!(out.requested, crate::move_input::SHORT_HOP);
        assert_eq!(out.stick, Some([0.0, 0.0]));
        assert!(trap.owns_jump_state(JUMP_SQUAT));
        let squat = LedgeContext {
            status: JUMP_SQUAT,
            motion_kind: motion_hash("jump_squat"),
            can_act: false,
            stopped: true,
            ..own(1.0)
        };
        assert_eq!(trap.sample(config, 1, squat, None).requested, 0);
        let air = LedgeContext {
            status: 0xb,
            situation: 2,
            can_act: false,
            can_air_attack: false,
            ..own(2.0)
        };
        assert_eq!(trap.sample(config, 2, air, None).requested, MAPPER_A);
        let accepted = LedgeContext {
            status: ATTACK_AIR,
            motion_kind: motion_hash("attack_air_hi"),
            ..air
        };
        assert_eq!(trap.sample(config, 3, accepted, None).requested, 0);
        assert!(trap.fighter_aerial_active());
        assert_eq!(
            trap.sample(
                config,
                4,
                LedgeContext {
                    status: 0x47,
                    can_control: false,
                    ..air
                },
                None
            ),
            LedgeOutput::default()
        );
        assert!(trap.manual_after_response());
    }
}
#[test]
fn funny_ground_moves_release_on_real_startup_instead_of_charging() {
    for (kind, id, first, startup, stick) in [
        (5, "forward_smash", 14.0, 0x2d, [1.0, 0.0]),
        (0x45, "belly_flop", 22.0, 0x30, [0.0, -1.0]),
        (0x46, "party_popper", 14.0, 0x2d, [1.0, 0.0]),
        (0x37, "ghost", 16.0, 0x2d, [1.0, 0.0]),
    ] {
        let config = configured(kind, PresetSlot::Neutral, id);
        let mut trap = LedgeTrap::default();
        let target = Opponent {
            vulnerable_in: Some(first + 1.0),
            actionable_in: Some(first + 6.0),
            ..enemy(CLIFF_CLIMB, false)
        };
        assert_eq!(trap.sample(config, 0, own(0.0), Some(target)).requested, 0);
        let out = trap.sample(config, 1, own(1.0), Some(target));
        assert_eq!(out.requested, MAPPER_A);
        assert_eq!(out.stick, Some(stick));
        assert_eq!(
            trap.sample(
                config,
                2,
                LedgeContext {
                    status: startup,
                    ..own(2.0)
                },
                None
            ),
            LedgeOutput::default()
        );
        assert!(trap.manual_after_response());
    }
}
#[test]
fn drop_chases_use_directional_jump_attack_together_for_each_fighter() {
    for kind in [5, 0x45, 0x46] {
        let config = configured(kind, PresetSlot::Drop, "short_hop_dair");
        let mut trap = LedgeTrap::default();
        trap.sample(config, 0, own(0.0), Some(enemy(CLIFF_WAIT, false)));
        let target = Opponent {
            previous_status: CLIFF_WAIT,
            situation: 2,
            position: [5.0, -4.0],
            ..enemy(0xe, true)
        };
        let out = trap.sample(config, 1, own(1.0), Some(target));
        assert_eq!(out.requested, crate::move_input::SHORT_HOP);
        assert_eq!(out.stick, Some([0.0, 0.0]));
    }
}

#[test]
fn yoshi_drop_fair_faces_outward_and_hands_back_after_its_actual_active_window() {
    let config = configured(5, PresetSlot::Drop, "drop_fair");
    let mut trap = LedgeTrap::default();
    let ctx = LedgeContext {
        facing: -1.0,
        ..own(0.0)
    };
    let hanging = Opponent {
        position: [-5.0, 0.0],
        ..enemy(CLIFF_WAIT, false)
    };
    trap.sample(config, 0, ctx, Some(hanging));
    let dropped = Opponent {
        previous_status: CLIFF_WAIT,
        status: 0xe,
        situation: 2,
        position: [-5.0, -4.0],
        vulnerable: true,
        ..hanging
    };
    let out = trap.sample(
        config,
        1,
        LedgeContext {
            motion_frame: 1.0,
            ..ctx
        },
        Some(dropped),
    );
    assert_eq!(out.requested, crate::move_input::SHORT_HOP);
    assert_eq!(out.stick, Some([0.0, 0.0]));
    let accepted = LedgeContext {
        situation: 2,
        status: ATTACK_AIR,
        motion_kind: motion_hash("attack_air_f"),
        motion_frame: 16.0,
        can_act: false,
        ..ctx
    };
    let out = trap.sample(config, 2, accepted, None);
    assert_eq!(out.requested, 0); // Accepted aerial is not repeatedly pressed.
    assert_eq!(out.stick, Some([-1.0, 0.0]));
    assert!(trap.fighter_aerial_active());
    assert_eq!(
        trap.sample(
            config,
            3,
            LedgeContext {
                motion_frame: 21.0,
                ..accepted
            },
            None
        ),
        LedgeOutput::default()
    );
    assert!(trap.manual_after_response());
}

#[test]
fn little_mac_jump_cover_stays_grounded_and_releases_on_up_tilt_acceptance() {
    let config = configured(0x34, PresetSlot::Jump, "up_tilt");
    let mut trap = LedgeTrap::default();
    let target = Opponent {
        situation: 2,
        vulnerable_in: Some(0.0),
        actionable_in: Some(30.0),
        ..enemy(CLIFF_JUMP1, true)
    };
    assert_eq!(trap.sample(config, 0, own(0.0), Some(target)).requested, 0);
    let out = trap.sample(config, 1, own(1.0), Some(target));
    assert_eq!(out.requested, MAPPER_A);
    assert_eq!(out.stick, Some([0.0, 0.6]));
    assert_eq!(
        trap.sample(
            config,
            2,
            LedgeContext {
                status: 0x2b,
                ..own(2.0)
            },
            None
        ),
        LedgeOutput::default()
    );
    assert!(trap.manual_after_response());
}

#[test]
fn pacman_and_mac_shield_specials_require_a_block_and_hand_back_on_startup() {
    for kind in [0x37, 0x34] {
        let config = configured(kind, PresetSlot::Attack, "shield_up_b");
        let mut trap = LedgeTrap::default();
        let guard = LedgeContext {
            status: GUARD,
            can_act: false,
            ..own(0.0)
        };
        let target = enemy(CLIFF_ATTACK, false);
        assert_eq!(
            trap.sample(config, 0, guard, Some(target)).requested,
            MAPPER_ZR
        );
        trap.plan = OptionPlan::Shield { blocked: true };
        let out = trap.sample(
            config,
            1,
            LedgeContext {
                motion_frame: 1.0,
                ..guard
            },
            Some(target),
        );
        assert_eq!(out.requested, MAPPER_ZR | MAPPER_B);
        assert_eq!(out.stick, Some([0.0, 1.0]));
        assert_eq!(
            trap.sample(
                config,
                2,
                LedgeContext {
                    status: 0x1de,
                    can_control: false,
                    ..own(2.0)
                },
                None
            ),
            LedgeOutput::default()
        );
        assert!(trap.manual_after_response());
    }
}

#[test]
fn roll_grabs_stop_short_by_own_reach_on_both_ledges() {
    for kind in 0..=0x4c {
        let catalog = FighterProfiles::default();
        let data = catalog.profile(kind).unwrap().ledge;
        for inward in [-1.0, 1.0] {
            let mut trap = LedgeTrap::default();
            trap.anchor = Some([0.0, 0.0]);
            trap.inward = inward;
            let settings = LedgeSettings {
                fighter: data,
                ..settings()
            };
            let target = Opponent {
                cliff_position: Some([0.0, 0.0]),
                position: [25.0 * inward, 0.0],
                ..enemy(CLIFF_ESCAPE, false)
            };
            let ctx = LedgeContext {
                position: [inward, 0.0],
                ..context()
            };
            trap.respond_to_roll_grab(settings, ctx, target, &mut LedgeOutput::default());
            let spot = trap.roll_intercept.unwrap();
            assert!(
                (spot.stand_x - inward * (25.0 - data.grab.roll_standoff())).abs() < 0.001,
                "kind {kind:#x}"
            );
            assert!(spot.stand_x.abs() < spot.endpoint.abs());
        }
    }
    assert!(
        crate::fighter_profiles::yoshi::LEDGE_DATA
            .grab
            .roll_standoff()
            > crate::fighter_profiles::isabelle::LEDGE_DATA
                .grab
                .roll_standoff()
    );
}

#[test]
fn yoshi_roll_grab_rechecks_overshoot_and_waits_for_momentum() {
    let mut settings = settings();
    settings.fighter = crate::fighter_profiles::yoshi::LEDGE_DATA;
    let mut trap = LedgeTrap::default();
    trap.anchor = Some([0.0, 0.0]);
    trap.inward = 1.0;
    let target = Opponent {
        position: [25.0, 0.0],
        vulnerable_in: Some(14.0),
        actionable_in: Some(25.0),
        ..enemy(CLIFF_ESCAPE, false)
    };
    let stand_x = 25.0 - settings.fighter.grab.roll_standoff();
    let coasting = LedgeContext {
        position: [stand_x, 0.0],
        velocity: [3.0, 0.0],
        ..context()
    };
    let mut out = LedgeOutput::default();
    trap.respond_to_roll_grab(settings, coasting, target, &mut out);
    assert_eq!(out.requested, 0);
    assert_eq!(out.stick, Some([0.0, 0.0]));
    assert_eq!(trap.roll_phase, "roll_settle_grab");
    assert!(trap.roll_positioned);

    // A prior arrival cannot authorize a grab after delayed input carries us on.
    let overshot = LedgeContext {
        position: [25.0 + settings.fighter.grab.roll_standoff() + 3.0, 0.0],
        velocity: [0.0, 0.0],
        ..context()
    };
    out = LedgeOutput::default();
    trap.respond_to_roll_grab(settings, overshot, target, &mut out);
    assert!(!trap.roll_positioned);
    assert_eq!(out.requested, 0);
    assert_eq!(out.stick, Some([-0.3, 0.0]));
    assert_eq!(trap.roll_intercept.unwrap().stand_x, stand_x);
}

#[test]
fn kirby_shared_grab_approaches_from_nearest_side_then_holds_until_native_acceptance() {
    // Yoshi now uses its dedicated fixed-spot response, covered below.
    for kind in [6] {
        for inward in [-1.0, 1.0] {
            let settings = configured(kind, PresetSlot::Roll, "grab");
            let mut trap = LedgeTrap::default();
            let roll = Opponent {
                cliff_position: Some([0.0, 0.0]),
                position: [25.0 * inward, 0.0],
                actionable_in: Some(45.0),
                ..enemy(CLIFF_ESCAPE, true)
            };
            let approach = LedgeContext {
                position: [50.0 * inward, 0.0],
                facing: -inward,
                ..own(0.0)
            };
            let out = trap.sample(settings, 0, approach, Some(roll));
            assert_eq!(out.requested, 0);
            assert_eq!(out.stick, Some([-inward, 0.0]));
            let stand = (25.0 + settings.fighter.grab.roll_standoff()) * inward;
            assert!((trap.roll_intercept.unwrap().stand_x - stand).abs() < 0.001);
            assert!(trap.owns_roll_navigation());
            let arrived = LedgeContext {
                position: [stand, 0.0],
                motion_frame: 1.0,
                ..approach
            };
            assert_eq!(trap.sample(settings, 1, arrived, Some(roll)).requested, 0);
            let ready = LedgeContext {
                motion_frame: 2.0,
                ..arrived
            };
            assert_eq!(
                trap.sample(settings, 2, ready, Some(roll)).requested,
                settings.grab_button
            );
            let delivery = LedgeContext {
                motion_frame: 3.0,
                ..arrived
            };
            assert_eq!(
                trap.sample(settings, 3, delivery, Some(roll)).requested,
                settings.grab_button
            );
            assert_eq!(
                trap.sample(settings, 3, delivery, Some(roll)).requested,
                settings.grab_button
            );
            let accepted = LedgeContext {
                status: GRAB_START,
                motion_kind: motion_hash("catch"),
                motion_frame: 0.0,
                can_act: false,
                ..arrived
            };
            assert_eq!(trap.sample(settings, 4, accepted, Some(roll)).requested, 0);
            let captured = LedgeContext {
                status: GRAB_PULL,
                motion_kind: motion_hash("catch_pull"),
                ..accepted
            };
            assert_eq!(
                trap.sample(settings, 5, captured, None),
                LedgeOutput::default()
            );
            assert!(trap.manual_after_grab());
        }
    }
}

#[test]
fn already_in_grab_range_turns_without_crossing_target_or_waiting_for_zero_speed() {
    // Yoshi now uses its dedicated fixed-spot response, covered below.
    for kind in [6] {
        let settings = configured(kind, PresetSlot::Roll, "grab");
        let reach = settings.fighter.grab.roll_standoff();
        let roll = Opponent {
            cliff_position: Some([0.0, 0.0]),
            position: [25.0, 0.0],
            actionable_in: Some(45.0),
            ..enemy(CLIFF_ESCAPE, true)
        };
        let c = LedgeContext {
            position: [25.0 + reach / 2.0, 0.0],
            ..own(0.0)
        };
        let mut trap = LedgeTrap::default();
        let turn = trap.sample(settings, 0, c, Some(roll));
        assert_eq!(trap.roll_intercept.unwrap().stand_x, c.position[0]);
        assert_eq!(turn.stick, Some([-0.3, 0.0]));
        let turned = LedgeContext {
            status: 3,
            facing: -1.0,
            velocity: [-0.4, 0.0],
            motion_frame: 1.0,
            ..c
        };
        assert_eq!(trap.sample(settings, 1, turned, Some(roll)).requested, 0);
        let grab = trap.sample(
            settings,
            2,
            LedgeContext {
                motion_frame: 2.0,
                ..turned
            },
            Some(roll),
        );
        assert_eq!(grab.requested, settings.grab_button, "kind {kind}");
    }
}

#[test]
fn missed_roll_grab_releases_stick_and_buttons_instead_of_freezing_at_the_spot() {
    let settings = configured(6, PresetSlot::Roll, "grab");
    let mut trap = LedgeTrap::default();
    let roll = Opponent {
        cliff_position: Some([0.0, 0.0]),
        position: [25.0, 0.0],
        vulnerable_in: Some(40.0),
        actionable_in: Some(45.0),
        ..enemy(CLIFF_ESCAPE, false)
    };
    let first = trap.sample(settings, 0, own(0.0), Some(roll));
    assert_eq!(first.requested, 0);
    let exited = Opponent {
        status: 0,
        previous_status: CLIFF_ESCAPE,
        ..roll
    };
    let output = trap.sample(settings, 1, own(1.0), Some(exited));
    assert_eq!(output.owned, 0);
    assert_eq!(output.stick, None);
    assert_eq!(output.requested, 0);
    assert!(
        trap.owns_roll_navigation(),
        "stance preparation cannot steal this missed response"
    );
}

#[test]
fn movement_animation_wraps_do_not_poison_roll_grab_requests() {
    // Yoshi now uses its dedicated fixed-spot response, covered below.
    for kind in [6] {
        let settings = configured(kind, PresetSlot::Roll, "grab");
        let mut trap = LedgeTrap::default();
        let target = Opponent {
            cliff_position: Some([0.0, 0.0]),
            position: [25.0, 0.0],
            actionable_in: Some(45.0),
            ..enemy(CLIFF_ESCAPE, true)
        };
        let walking = LedgeContext {
            status: 1,
            position: [0.0, 0.0],
            motion_kind: motion_hash("walk_slow"),
            motion_frame: 18.0,
            ..own(0.0)
        };
        assert_eq!(trap.sample(settings, 0, walking, Some(target)).requested, 0);
        let stand = 25.0 - settings.fighter.grab.roll_standoff();
        let wrapped = LedgeContext {
            position: [stand, 0.0],
            motion_frame: 0.0,
            ..walking
        };
        assert_eq!(trap.sample(settings, 1, wrapped, Some(target)).requested, 0);
        let ready = LedgeContext {
            motion_frame: 1.0,
            ..wrapped
        };
        assert_eq!(
            trap.sample(settings, 2, ready, Some(target)).requested,
            settings.grab_button
        );
        // A second loop during pending delivery must not cancel the button.
        let delivery_wrap = LedgeContext {
            motion_frame: 0.0,
            ..ready
        };
        assert_eq!(
            trap.sample(settings, 3, delivery_wrap, Some(target))
                .requested,
            settings.grab_button
        );
        assert_eq!(
            trap.sample(settings, 3, delivery_wrap, Some(target))
                .requested,
            settings.grab_button
        );
    }
}

#[test]
fn spent_dash_budget_cannot_prevent_roll_grab_turn_and_native_request() {
    let settings = configured(6, PresetSlot::Roll, "grab");
    let mut trap = LedgeTrap::default();
    let c = LedgeContext {
        position: [25.0 + settings.fighter.grab.roll_standoff(), 0.0],
        facing: 1.0,
        ..own(0.0)
    };
    let target = Opponent {
        cliff_position: Some([0.0, 0.0]),
        position: [25.0, 0.0],
        actionable_in: Some(45.0),
        ..enemy(CLIFF_ESCAPE, true)
    };
    trap.sample(settings, 0, c, Some(target));
    trap.movement_attempts = MAX_ATTEMPTS;
    trap.roll_turn_attempts = 0;
    let turn = trap.sample(
        settings,
        1,
        LedgeContext {
            motion_frame: 1.0,
            ..c
        },
        Some(target),
    );
    assert_eq!(
        turn.stick,
        Some([0.0, 0.0]),
        "release prior turn before next pulse"
    );
    let next = trap.sample(
        settings,
        2,
        LedgeContext {
            motion_frame: 2.0,
            ..c
        },
        Some(target),
    );
    assert_eq!(next.stick, Some([-0.3, 0.0]));
    let faced = LedgeContext {
        facing: -1.0,
        motion_frame: 3.0,
        ..c
    };
    assert_eq!(trap.sample(settings, 3, faced, Some(target)).requested, 0);
    assert_eq!(
        trap.sample(
            settings,
            4,
            LedgeContext {
                motion_frame: 4.0,
                ..faced
            },
            Some(target)
        )
        .requested,
        settings.grab_button
    );
}

#[test]
fn roll_grab_uses_native_guard_readiness_instead_of_movement_readiness() {
    let settings = configured(6, PresetSlot::Roll, "grab");
    let mut trap = LedgeTrap::default();
    let c = LedgeContext {
        status: GUARD,
        can_act: false,
        can_grab: true,
        position: [25.0 - settings.fighter.grab.roll_standoff(), 0.0],
        ..own(0.0)
    };
    let target = Opponent {
        cliff_position: Some([0.0, 0.0]),
        position: [25.0, 0.0],
        actionable_in: Some(45.0),
        ..enemy(CLIFF_ESCAPE, true)
    };
    let out = trap.sample(settings, 0, c, Some(target));
    assert_eq!(out.requested, settings.grab_button | MAPPER_ZR);
    assert_eq!(
        trap.sample(
            settings,
            1,
            LedgeContext {
                motion_frame: 1.0,
                ..c
            },
            Some(target)
        )
        .requested,
        settings.grab_button | MAPPER_ZR
    );
}

#[test]
fn rejected_roll_grab_has_one_release_then_retries_without_extra_eight_frame_timer() {
    let settings = configured(6, PresetSlot::Roll, "grab");
    let mut trap = LedgeTrap::default();
    let c = LedgeContext {
        position: [25.0 - settings.fighter.grab.roll_standoff(), 0.0],
        ..own(0.0)
    };
    let target = Opponent {
        cliff_position: Some([0.0, 0.0]),
        position: [25.0, 0.0],
        actionable_in: Some(45.0),
        ..enemy(CLIFF_ESCAPE, true)
    };
    for (tick, buttons) in [
        (0, settings.grab_button),
        (1, settings.grab_button),
        (2, 0),
        (3, settings.grab_button),
    ] {
        let out = trap.sample(
            settings,
            tick,
            LedgeContext {
                motion_frame: tick as f32,
                ..c
            },
            Some(target),
        );
        assert_eq!(out.requested, buttons, "tick {tick}");
    }
    assert_eq!(trap.attempts, 2);
}

#[test]
fn catch_readiness_accepts_native_brake_transition_without_permitting_guard_dash_terms() {
    for status in [0, 1, 3, 4, 5, 6, 7, 8, 9, 0x11] {
        for ready in [0x1e00006f, 0x1e000070, 0x1e000071] {
            assert!(grab_input_ready(status, |term| term == ready));
        }
    }
    assert!(!grab_input_ready(GUARD, |term| term == 0x1e000070));
    assert!(grab_input_ready(GUARD, |term| term == 0x1e00006f));
    assert!(!grab_input_ready(0, |_| false));
}

#[test]
fn native_ready_roll_grab_reaches_mapper_r_press_hold_release_and_retry() {
    use crate::manual_controls::{ManualOverlay, TurboOutput};
    use crate::native_input::{ButtonTrace, MAPPER_L, MAPPER_Y};
    let settings = configured(6, PresetSlot::Roll, "grab");
    let mut trap = LedgeTrap::default();
    let mut mapper = ManualOverlay::default();
    let target = Opponent {
        cliff_position: Some([0.0, 0.0]),
        position: [25.0, 0.0],
        actionable_in: Some(45.0),
        ..enemy(CLIFF_ESCAPE, true)
    };
    let c = LedgeContext {
        position: [25.0 - settings.fighter.grab.roll_standoff(), 0.0],
        ..own(0.0)
    };
    for tick in 0..4 {
        let out = trap.sample(
            settings,
            tick,
            LedgeContext {
                motion_frame: tick as f32,
                ..c
            },
            Some(target),
        );
        let actual = mapper
            .apply_controls(
                ButtonTrace {
                    held: MAPPER_L,
                    previous: MAPPER_L,
                    ..ButtonTrace::default()
                },
                [0.0, 0.0],
                TurboOutput {
                    stick: out.stick,
                    ..TurboOutput::default()
                },
                MAPPER_L | MAPPER_Y,
                out.requested,
                out.owned,
            )
            .buttons;
        assert_eq!(actual.held & settings.grab_button != 0, tick != 2);
        assert_eq!(
            actual.pressed & settings.grab_button != 0,
            tick == 0 || tick == 3
        );
        assert_eq!(actual.released & settings.grab_button != 0, tick == 2);
        assert_eq!(actual.held & MAPPER_L, 0);
    }
}

#[test]
fn kirby_roll_approach_brakes_once_then_grabs_without_returning_to_original_spot() {
    for inward in [-1.0, 1.0] {
        let settings = configured(6, PresetSlot::Roll, "grab");
        let reach = settings.fighter.grab.roll_standoff();
        let target = Opponent {
            cliff_position: Some([0.0, 0.0]),
            position: [25.0 * inward, 0.0],
            actionable_in: Some(40.0),
            ..enemy(CLIFF_ESCAPE, true)
        };
        let mut trap = LedgeTrap::default();
        let start = LedgeContext {
            position: [2.0 * inward, 0.0],
            facing: inward,
            ..own(0.0)
        };
        assert_eq!(
            trap.sample(settings, 0, start, Some(target)).stick,
            Some([inward, 0.0])
        );
        let original = trap.roll_intercept.unwrap();
        // An approach near the destination still has momentum. Release rather
        // than drive through, and retain the braking latch for this roll.
        let brake = LedgeContext {
            status: 4,
            position: [19.0 * inward, 0.0],
            velocity: [1.0 * inward, 0.0],
            ground_brake: Some(0.1),
            motion_frame: 1.0,
            ..start
        };
        assert_eq!(
            trap.sample(settings, 1, brake, Some(target)).stick,
            Some([0.0, 0.0])
        );
        assert!(trap.roll_grab_braking);
        let arrived = LedgeContext {
            status: 0,
            // Past the original standing point, but still comfortably in reach.
            position: [(25.0 - reach / 2.0) * inward, 0.0],
            velocity: [0.0, 0.0],
            motion_frame: 2.0,
            ..brake
        };
        let grab = trap.sample(settings, 2, arrived, Some(target));
        assert_eq!(grab.stick, Some([0.0, 0.0]));
        assert_eq!(grab.requested, settings.grab_button);
        assert_eq!(trap.roll_intercept.unwrap(), original);
    }
}

#[test]
fn overshot_roll_grab_never_reverse_dashes_or_crosses_back_to_the_old_standing_side() {
    // Yoshi now uses its dedicated fixed-spot response, covered below.
    for kind in [6] {
        let settings = configured(kind, PresetSlot::Roll, "grab");
        let reach = settings.fighter.grab.roll_standoff();
        let target = Opponent {
            cliff_position: Some([0.0, 0.0]),
            position: [25.0, 0.0],
            actionable_in: Some(45.0),
            ..enemy(CLIFF_ESCAPE, true)
        };
        let mut trap = LedgeTrap::default();
        let start = LedgeContext {
            position: [1.0, 0.0],
            ..own(0.0)
        };
        trap.sample(settings, 0, start, Some(target));
        let overshot = LedgeContext {
            position: [25.0 + reach + 4.0, 0.0],
            velocity: [1.0, 0.0],
            motion_frame: 1.0,
            ..start
        };
        let settle = trap.sample(settings, 1, overshot, Some(target));
        assert_eq!(
            settle.stick,
            Some([0.0, 0.0]),
            "never turn-dash against momentum"
        );
        let stopped = LedgeContext {
            velocity: [0.0, 0.0],
            motion_frame: 2.0,
            ..overshot
        };
        let correction = trap.sample(settings, 2, stopped, Some(target));
        assert_eq!(correction.stick, Some([-0.3, 0.0]));
        let inside = LedgeContext {
            position: [25.0 + reach / 2.0, 0.0],
            facing: -1.0,
            motion_frame: 3.0,
            ..stopped
        };
        // Release the correction; then grab on THIS side, without returning to
        // the original ledge-side stand_x below 25.
        assert_eq!(
            trap.sample(settings, 3, inside, Some(target)).stick,
            Some([0.0, 0.0])
        );
        let grab = trap.sample(
            settings,
            4,
            LedgeContext {
                motion_frame: 4.0,
                ..inside
            },
            Some(target),
        );
        assert_eq!(grab.stick, Some([0.0, 0.0]));
        assert_eq!(grab.requested, settings.grab_button);
        assert!(trap.roll_intercept.unwrap().stand_x < 25.0);
    }
}

#[test]
fn kirby_roll_grab_active_window_overlaps_the_inset_vulnerability_target() {
    let settings = configured(6, PresetSlot::Roll, "grab");
    let ctx = LedgeContext {
        position: [25.0 - settings.fighter.grab.roll_standoff(), 0.0],
        ..own(0.0)
    };
    let target = Opponent {
        position: [25.0, 0.0],
        cliff_position: Some([0.0, 0.0]),
        actionable_in: Some(20.0),
        ..enemy(CLIFF_ESCAPE, false)
    };
    let mut trap = LedgeTrap::default();
    let startup = f32::from(settings.fighter.grab.standing);
    let too_early = Opponent {
        vulnerable_in: Some(startup - 1.0),
        ..target
    };
    assert_eq!(trap.sample(settings, 0, ctx, Some(too_early)).requested, 0);
    let centered = Opponent {
        vulnerable_in: Some(startup - 2.0),
        ..target
    };
    assert_eq!(
        trap.sample(
            settings,
            1,
            LedgeContext {
                motion_frame: 1.0,
                ..ctx
            },
            Some(centered)
        )
        .requested,
        settings.grab_button
    );
    // A two-frame vulnerable interval uses its midpoint, not the three-frame inset.
    let mut narrow = LedgeTrap::default();
    let short_window = Opponent {
        vulnerable_in: Some(startup - 1.0),
        actionable_in: Some(startup + 1.0),
        ..target
    };
    assert_eq!(
        narrow
            .sample(settings, 0, ctx, Some(short_window))
            .requested,
        settings.grab_button
    );
    let mut late = LedgeTrap::default();
    let expired = Opponent {
        vulnerable: true,
        actionable_in: Some(startup),
        ..target
    };
    assert_eq!(late.sample(settings, 0, ctx, Some(expired)).requested, 0);
}

#[test]
fn yoshi_jump_nair_starts_immediately_short_hops_and_drifts_after_native_nair_acceptance() {
    for direction in [-1.0, 1.0] {
        let settings = configured(5, PresetSlot::Jump, "nair");
        let mut trap = LedgeTrap::default();
        let target = Opponent {
            position: [direction * 20.0, 35.0],
            situation: 2,
            vulnerable_in: None,
            actionable_in: None,
            ..enemy(CLIFF_JUMP1, false)
        };
        let start = LedgeContext {
            facing: -direction,
            air_physics: AirPhysics::new(0.1, 2.0),
            short_hop_speed: Some(2.0),
            full_hop_speed: Some(4.0),
            ..own(0.0)
        };
        let jump = trap.sample(settings, 0, start, Some(target));
        assert_eq!(jump.requested, MAPPER_X | MAPPER_A);
        assert_eq!(jump.stick, Some([0.0, 0.0]));
        let squat = LedgeContext {
            status: JUMP_SQUAT,
            can_act: false,
            motion_kind: motion_hash("jump_squat"),
            motion_frame: 1.0,
            ..start
        };
        let attack = trap.sample(settings, 1, squat, None);
        assert_eq!(
            attack.requested, MAPPER_A,
            "release jump; preserve neutral attack"
        );
        assert_eq!(attack.stick, Some([0.0, 0.0]));
        let nair = LedgeContext {
            situation: 2,
            status: ATTACK_AIR,
            can_act: false,
            motion_kind: motion_hash("attack_air_n"),
            motion_frame: 0.0,
            ..start
        };
        let drift = trap.sample(settings, 2, nair, None);
        assert_eq!(drift.requested, 0);
        assert_eq!(drift.stick, Some([direction * 0.25, 0.0]));
        for frame in 1..=25 {
            let lingering = trap.sample(
                settings,
                2 + frame,
                LedgeContext {
                    motion_frame: frame as f32,
                    ..nair
                },
                None,
            );
            assert_eq!(lingering.requested, 0);
            assert_eq!(
                lingering.stick,
                Some([if frame < 4 { direction * 0.25 } else { 0.0 }, 0.0])
            );
        }
        assert_eq!(
            trap.sample(
                settings,
                28,
                LedgeContext {
                    motion_frame: 26.0,
                    ..nair
                },
                None
            ),
            LedgeOutput::default()
        );
    }
}

#[test]
fn yoshi_neutral_nair_uses_seventeen_frame_lead_and_brief_nudge() {
    let mut settings = configured(5, PresetSlot::Neutral, "nair");
    let choice = FighterProfiles::default()
        .profile(5)
        .unwrap()
        .menu_options
        .list(PresetSlot::Neutral)
        .options[0];
    settings.response_adjustments.nair_hit_lead = Some(8);
    let settings = (choice.prepare)(settings);
    assert_eq!(settings.fighter.normal_hit_lead_frames, 17);
    assert_eq!(settings.fighter.neutral_air.first, 3);
    let early = Opponent {
        vulnerable_in: Some(24.0),
        position: [8.0, 0.0],
        ..enemy(CLIFF_CLIMB, false)
    };
    let mut trap = LedgeTrap::default();
    assert_eq!(trap.sample(settings, 0, own(0.0), Some(early)).requested, 0);
    let staged = Opponent {
        vulnerable_in: Some(23.0),
        ..early
    };
    assert_eq!(
        trap.sample(settings, 1, own(1.0), Some(staged)).requested,
        MAPPER_X | MAPPER_A
    );
    let squat = LedgeContext {
        status: JUMP_SQUAT,
        can_act: false,
        ..own(2.0)
    };
    let buffered = trap.sample(settings, 2, squat, None);
    assert_eq!(buffered.stick, Some([0.0, 0.0]));
    let air = LedgeContext {
        status: 0xb,
        situation: 2,
        can_act: false,
        ..own(3.0)
    };
    let awaiting_nair = trap.sample(settings, 3, air, None);
    assert_eq!(awaiting_nair.stick, Some([0.0, 0.0]));
    let nair = LedgeContext {
        status: ATTACK_AIR,
        motion_kind: NEUTRAL_AIR_MOTION,
        motion_frame: 1.0,
        ..air
    };
    let drifting = trap.sample(settings, 4, nair, None);
    assert_eq!(drifting.stick, Some([0.25, 0.0]));
    let settled = trap.sample(
        settings,
        5,
        LedgeContext {
            motion_frame: 4.0,
            ..nair
        },
        None,
    );
    assert_eq!(settled.stick, Some([0.0, 0.0]));
}

#[test]
fn yoshi_roll_down_smash_stands_two_units_closer_to_ledge() {
    for inward in [-1.0, 1.0] {
        let settings = configured(5, PresetSlot::Roll, "down_smash");
        let mut trap = LedgeTrap::default();
        let target = Opponent {
            cliff_position: Some([0.0, 0.0]),
            position: [inward * 25.0, 0.0],
            vulnerable_in: Some(30.0),
            actionable_in: Some(40.0),
            ..enemy(CLIFF_ESCAPE, false)
        };
        trap.sample(
            settings,
            0,
            LedgeContext {
                position: [inward, 0.0],
                facing: inward,
                ..own(0.0)
            },
            Some(target),
        );
        assert_eq!(trap.roll_interception().unwrap()[0], inward * 23.0);
    }
}

#[test]
fn shared_roll_grab_uses_full_approach_and_preserves_selected_facing() {
    for kind in [5, 6] {
        for inward in [-1.0, 1.0] {
            let settings = configured(kind, PresetSlot::Roll, "grab");
            let mut trap = LedgeTrap::default();
            let target = Opponent {
                cliff_position: Some([0.0, 0.0]),
                position: [inward * 25.0, 0.0],
                actionable_in: Some(45.0),
                ..enemy(CLIFF_ESCAPE, true)
            };
            let start = LedgeContext {
                position: [inward, 0.0],
                facing: inward,
                ..own(0.0)
            };
            trap.sample(settings, 0, start, Some(target));
            let spot = trap.roll_intercept.unwrap();
            let old_destination = inward * (25.0 - settings.fighter.grab.roll_standoff());
            assert!((spot.stand_x - old_destination).abs() < 0.001);
            // Kirby and Yoshi both preserve center-facing during the walk.
            let arrived = LedgeContext {
                position: [spot.stand_x, 0.0],
                facing: inward,
                ..own(1.0)
            };
            trap.sample(settings, 1, arrived, Some(target));
            let grab = trap.sample(
                settings,
                2,
                LedgeContext {
                    motion_frame: 2.0,
                    ..arrived
                },
                Some(target),
            );
            assert_eq!(grab.stick, Some([0.0, 0.0]));
            assert_eq!(grab.requested, settings.grab_button);
        }
    }
}

#[test]
fn kirby_roll_grab_fast_walks_then_stops_anywhere_inside_usable_reach() {
    let settings = configured(6, PresetSlot::Roll, "grab");
    for inward in [-1.0, 1.0] {
        let mut trap = LedgeTrap::default();
        let spot = RollIntercept {
            endpoint: inward * 25.0,
            stand_x: inward * 14.0,
            facing: inward,
        };
        let ctx = LedgeContext {
            position: [inward, 0.0],
            facing: inward,
            ..own(0.0)
        };
        let mut out = LedgeOutput::default();
        assert!(!trap.position_for_roll_grab(settings, ctx, spot, &mut out));
        assert_eq!(out.stick, Some([inward * 0.65, 0.0]));
        let past = LedgeContext {
            position: [inward * 18.0, 0.0],
            ..ctx
        };
        // Caller supplies neutral. Reaching the usable area is sufficient;
        // he need not walk all the way to the original standing point.
        out.stick = Some([0.0, 0.0]);
        assert!(trap.position_for_roll_grab(settings, past, spot, &mut out));
        assert_eq!(out.stick, Some([0.0, 0.0]));
        assert!(trap.roll_positioned);
    }
}

#[test]
fn kirby_bair_cancel_requires_mapped_cstick_then_holds_short_hop_without_second_a() {
    use crate::native_input::{MAPPER_RIGHT_STICK_LEFT, MAPPER_RIGHT_STICK_RIGHT, MAPPER_ZL};
    let settings = configured(6, PresetSlot::Jump, "bair");
    for toward in [-1.0, 1.0] {
        let mut trap = LedgeTrap::default();
        let target = Opponent {
            position: [toward * 8.0, 6.0],
            situation: 2,
            ..enemy(CLIFF_JUMP1, true)
        };
        let ctx = LedgeContext {
            facing: toward,
            ..own(0.0)
        };
        let prime = trap.sample(settings, 0, ctx, Some(target));
        let backward_bit = if toward > 0.0 {
            MAPPER_RIGHT_STICK_LEFT
        } else {
            MAPPER_RIGHT_STICK_RIGHT
        };
        assert_eq!(prime.requested, backward_bit);
        assert_eq!(trap.attack_cancel_right_stick(), Some([-toward, 0.0]));
        // Native time alone cannot advance before the mapper sees the flick.
        let waiting = trap.sample(
            settings,
            1,
            LedgeContext {
                motion_frame: 1.0,
                ..ctx
            },
            None,
        );
        assert_eq!(waiting.requested, backward_bit);
        trap.mapped_attack_cancel(0x1, (-toward * 127.0) as i8);
        let still_waiting = trap.sample(
            settings,
            2,
            LedgeContext {
                motion_frame: 2.0,
                ..ctx
            },
            None,
        );
        assert_eq!(
            still_waiting.requested, backward_bit,
            "attack without mapped C-stick is not acceptance"
        );
        trap.mapped_attack_cancel(0x81, (-toward * 127.0) as i8);
        let tilt = LedgeContext {
            status: 0x2b,
            motion_kind: motion_hash("attack_s3_s"),
            motion_frame: 1.0,
            facing: -toward,
            can_act: false,
            can_jump: false,
            ..ctx
        };
        assert!(trap.owns_jump_state(tilt.status));
        let jump = trap.sample(settings, 3, tilt, None);
        assert_eq!(jump.requested, MAPPER_X | MAPPER_ZL);
        assert_eq!(jump.stick, Some([toward, 0.0]));
        assert_eq!(trap.attack_cancel_right_stick(), Some([0.0, 0.0]));
        let held = trap.sample(
            settings,
            4,
            LedgeContext {
                motion_frame: 2.0,
                ..tilt
            },
            None,
        );
        assert_eq!(held.requested, MAPPER_X | MAPPER_ZL);
        let squat = LedgeContext {
            status: JUMP_SQUAT,
            motion_kind: motion_hash("jump_squat"),
            motion_frame: 1.0,
            ..tilt
        };
        let buffered = trap.sample(settings, 5, squat, None);
        assert_eq!(
            buffered.requested, 0,
            "the cancel already buffered attack; no second A"
        );
        assert_eq!(buffered.stick, Some([toward, 0.0]));
        let bair = LedgeContext {
            status: ATTACK_AIR,
            situation: 2,
            motion_kind: motion_hash("attack_air_b"),
            motion_frame: 1.0,
            ..squat
        };
        assert_eq!(
            trap.sample(settings, 6, bair, None).stick,
            Some([toward * 0.35, 0.0])
        );
    }
}

#[test]
fn kirby_neutral_bair_short_hops_three_frames_early_without_drift() {
    let settings = configured(6, PresetSlot::Neutral, "bair");
    let mut trap = LedgeTrap::default();
    let ctx = LedgeContext {
        facing: -1.0,
        ..own(0.0)
    };
    let target = Opponent {
        position: [8.0, 0.0],
        vulnerable_in: Some(13.0),
        actionable_in: Some(30.0),
        ..enemy(CLIFF_CLIMB, false)
    };
    assert_eq!(trap.sample(settings, 0, ctx, Some(target)).requested, 0);
    let launch = trap.sample(
        settings,
        1,
        LedgeContext {
            motion_frame: 1.0,
            ..ctx
        },
        Some(Opponent {
            vulnerable_in: Some(12.0),
            ..target
        }),
    );
    assert_eq!(launch.requested, crate::move_input::SHORT_HOP);
    let squat = LedgeContext {
        status: JUMP_SQUAT,
        can_act: false,
        motion_kind: motion_hash("jump_squat"),
        motion_frame: 1.0,
        ..ctx
    };
    assert_eq!(trap.sample(settings, 2, squat, None).requested, 0);
    let bair = LedgeContext {
        status: ATTACK_AIR,
        situation: 2,
        motion_kind: motion_hash("attack_air_b"),
        motion_frame: 1.0,
        ..squat
    };
    let active = trap.sample(settings, 3, bair, None);
    assert_eq!(active.stick, Some([0.0, 0.0]));
    assert_eq!(active.requested, 0);
}

#[test]
fn kirby_bair_already_backwards_launches_without_turn_or_locomotion_readiness() {
    let settings = configured(6, PresetSlot::Jump, "bair");
    for toward in [-1.0, 1.0] {
        let mut trap = LedgeTrap::default();
        let target = Opponent {
            position: [toward * 8.0, 6.0],
            situation: 2,
            ..enemy(CLIFF_JUMP1, true)
        };
        let ctx = LedgeContext {
            facing: -toward,
            can_act: false,
            can_jump: true,
            safe_ground: [false, false],
            ..own(0.0)
        };
        let jump = trap.sample(settings, 0, ctx, Some(target));
        assert_eq!(jump.requested, crate::move_input::SHORT_HOP);
        assert_eq!(jump.stick, Some([0.0, 0.0]));
        assert!(
            trap.owns_jump_state(6),
            "turn state must retain adapter ownership"
        );
    }
}

#[test]
fn isabelle_offers_slingshot_in_all_five_lists_without_changing_defaults() {
    let profiles = FighterProfiles::default();
    let menu = &profiles.profile(0x46).unwrap().menu_options;
    for slot in [
        PresetSlot::Neutral,
        PresetSlot::Jump,
        PresetSlot::Roll,
        PresetSlot::Attack,
        PresetSlot::Drop,
    ] {
        let list = menu.list(slot);
        assert!(list
            .options
            .iter()
            .any(|choice| choice.name.to_lowercase().contains("slingshot")));
        assert_eq!(list.selected, 0);
    }
}

#[test]
fn isabelle_stationary_slingshot_covers_neutral_roll_and_drop_without_chasing() {
    for (slot, id, status) in [
        (PresetSlot::Neutral, "slingshot", CLIFF_CLIMB),
        (PresetSlot::Roll, "slingshot", CLIFF_ESCAPE),
        (PresetSlot::Drop, "slingshot", 0xe),
    ] {
        for back in [false, true] {
            let settings = configured(0x46, slot, id);
            let mut trap = LedgeTrap::default();
            let ctx = LedgeContext {
                facing: if back { -1.0 } else { 1.0 },
                ..own(0.0)
            };
            let target = Opponent {
                position: [20.0, 0.0],
                previous_status: if status == 0xe { CLIFF_WAIT } else { 0 },
                situation: if status == 0xe { 2 } else { 0 },
                vulnerable_in: Some(0.0),
                actionable_in: Some(40.0),
                ..enemy(status, true)
            };
            let launch = trap.sample(settings, 0, ctx, Some(target));
            assert_eq!(launch.requested, crate::move_input::SHORT_HOP);
            let squat = LedgeContext {
                status: JUMP_SQUAT,
                can_act: false,
                motion_kind: motion_hash("jump_squat"),
                motion_frame: 1.0,
                ..ctx
            };
            let attack = trap.sample(settings, 1, squat, None);
            assert_eq!(attack.requested, 0);
            assert_eq!(attack.stick, Some([0.0, 0.0]));
            let air = LedgeContext {
                status: ATTACK_AIR,
                situation: 2,
                motion_kind: motion_hash(if back { "attack_air_b" } else { "attack_air_f" }),
                motion_frame: 1.0,
                ..squat
            };
            assert_eq!(trap.sample(settings, 2, air, None).stick, Some([0.0, 0.0]));
        }
    }
}

#[test]
fn isabelle_slingshot_shields_until_confirmed_block_then_fires_from_either_facing() {
    let profiles = FighterProfiles::default();
    let choice = profiles
        .profile(0x46)
        .unwrap()
        .menu_options
        .list(PresetSlot::Attack)
        .options
        .iter()
        .find(|choice| choice.id == "slingshot")
        .unwrap();
    let settings = configured(0x46, PresetSlot::Attack, "slingshot");
    for back in [false, true] {
        let mut trap = LedgeTrap::default();
        trap.plan = OptionPlan::Shield { blocked: false };
        let ctx = LedgeContext {
            status: GUARD,
            can_act: false,
            facing: if back { -1.0 } else { 1.0 },
            ..own(0.0)
        };
        let target = Opponent {
            position: [8.0, 0.0],
            ..enemy(CLIFF_ATTACK, false)
        };
        let mut out = LedgeOutput::default();
        (choice.respond)(&mut trap, settings, ctx, target, &mut out);
        assert_eq!(out.requested, MAPPER_ZR);
        assert!(trap.fighter_move.is_none());
        trap.plan = OptionPlan::Shield { blocked: true };
        (choice.respond)(&mut trap, settings, ctx, target, &mut out);
        assert_eq!(out.requested, crate::move_input::SHORT_HOP);
        assert!(trap.fighter_move.is_some());
    }
}

#[test]
fn shared_roll_grabs_request_from_usable_area_without_walking_into_endpoint() {
    for kind in [6, 5, 0x46, 3] {
        for inward in [-1.0, 1.0] {
            let settings = configured(kind, PresetSlot::Roll, "grab");
            let reach = settings.fighter.grab.roll_usable_reach();
            let spot = RollIntercept {
                endpoint: inward * 25.0,
                stand_x: inward * (25.0 - settings.fighter.grab.roll_standoff()),
                facing: inward,
            };
            let mut trap = LedgeTrap::default();
            trap.anchor = Some([0.0, 0.0]);
            trap.inward = inward;
            trap.roll_intercept = Some(spot);
            let ctx = LedgeContext {
                position: [inward * (25.0 - reach + 0.25), 0.0],
                status: 1,
                facing: inward,
                velocity: [inward * 0.25, 0.0],
                ..own(0.0)
            };
            assert!(
                (ctx.position[0] - spot.stand_x).abs() > 1.5,
                "this fixture would fail the old exact-point requirement"
            );
            let target = Opponent {
                position: [inward * 25.0, 0.0],
                actionable_in: Some(40.0),
                ..enemy(CLIFF_ESCAPE, true)
            };
            let mut out = LedgeOutput::default();
            trap.respond_to_roll_grab(settings, ctx, target, &mut out);
            assert_eq!(out.stick, Some([0.0, 0.0]));
            assert_eq!(out.requested, settings.grab_button);
            assert!(trap.grab_delivery.is_some());
        }
    }
}

#[test]
fn yoshi_can_request_when_late_tongue_activity_reaches_vulnerable_recovery() {
    let settings = configured(5, PresetSlot::Roll, "grab");
    let mut trap = LedgeTrap::default();
    let target = Opponent {
        cliff_position: Some([0.0, 0.0]),
        position: [25.0, 0.0],
        vulnerable_in: Some(16.0),
        actionable_in: Some(25.0),
        ..enemy(CLIFF_ESCAPE, false)
    };
    let out = trap.sample(
        settings,
        0,
        LedgeContext {
            position: [12.0, 0.0],
            ..own(0.0)
        },
        Some(target),
    );
    // First activity is 14, but the native eight-frame window runs through 21.
    // Its later activity can reach the protection+3 target at frame 19.
    assert_eq!(out.requested, settings.grab_button);
    assert_eq!(out.stick, Some([0.0, 0.0]));
}

#[test]
fn kirby_yoshi_walk_does_not_brake_before_entering_grab_area() {
    for kind in [6, 5] {
        let settings = configured(kind, PresetSlot::Roll, "grab");
        for inward in [-1.0, 1.0] {
            let reach = settings.fighter.grab.roll_usable_reach();
            let spot = RollIntercept {
                endpoint: inward * 25.0,
                stand_x: inward * 20.0,
                facing: inward,
            };
            let ctx = LedgeContext {
                position: [inward * (25.0 - reach - 0.8), 0.0],
                facing: inward,
                velocity: [inward * 0.15, 0.0],
                ground_brake: Some(0.08),
                status: 1,
                ..own(0.0)
            };
            let mut trap = LedgeTrap::default();
            let mut out = LedgeOutput::default();
            assert!(!trap.position_for_roll_grab(settings, ctx, spot, &mut out));
            assert_eq!(out.stick, Some([inward * 0.65, 0.0]));
            assert_eq!(trap.roll_phase, "roll_fast_walk_to_grab");
        }
    }
}

#[test]
fn kirby_logged_late_miss_extends_approach_instead_of_grabbing_distant_opponent() {
    let settings = configured(6, PresetSlot::Roll, "grab");
    let mut trap = LedgeTrap::default();
    trap.prepare_stance([79.989304, 0.0], -1.0);
    let target = Opponent {
        position: [47.189384, 0.0],
        velocity: [-0.9, 0.0],
        actionable_in: Some(18.131863),
        ..enemy(CLIFF_ESCAPE, true)
    };
    let ctx = LedgeContext {
        position: [62.41135, 0.0],
        facing: -1.0,
        ..own(0.0)
    };
    let mut out = LedgeOutput::default();
    trap.respond_to_roll_grab(settings, ctx, target, &mut out);
    assert_eq!(out.requested & settings.grab_button, 0);
    assert_eq!(out.stick, Some([-0.65, 0.0]));
    assert!(trap.roll_intercept.unwrap().endpoint < 54.989304);
}

#[test]
fn roll_grab_can_catch_after_passing_old_common_point_without_turning_back() {
    let settings = configured(6, PresetSlot::Roll, "grab");
    for inward in [-1.0, 1.0] {
        let mut trap = LedgeTrap::default();
        trap.prepare_stance([0.0, 0.0], inward);
        let target = Opponent {
            position: [inward * 23.0, 0.0],
            velocity: [inward, 0.0],
            actionable_in: Some(30.0),
            ..enemy(CLIFF_ESCAPE, true)
        };
        let ctx = LedgeContext {
            position: [inward * 27.0, 0.0],
            facing: inward,
            ..own(0.0)
        };
        let mut out = LedgeOutput::default();
        trap.respond_to_roll_grab(settings, ctx, target, &mut out);
        assert_eq!(out.stick, Some([0.0, 0.0]));
        assert_eq!(out.requested, settings.grab_button);
    }
}
