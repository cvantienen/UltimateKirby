//! Distinct recipes use native facts, not the requested button as confirmation.
use super::*;
use crate::fighter_profiles::{presets::PresetSlot, FighterProfiles};
use crate::{ledge_menu::MenuSelection, timing::motion_hash};

fn configured(slot: PresetSlot, id: &str) -> LedgeSettings {
    let profiles = FighterProfiles::default();
    let profile = profiles.profile(6).unwrap();
    let base = profile.apply(LedgeSettings {
        enabled: true,
        ..LedgeSettings::default()
    });
    let mut selected = MenuSelection::new(&profile.menu_options);
    selected.choices[slot as usize] = profile
        .menu_options
        .list(slot)
        .options
        .iter()
        .position(|choice| choice.id == id)
        .unwrap();
    selected.settings(&profile.menu_options, base)
}
fn own(frame: f32) -> LedgeContext {
    LedgeContext {
        motion_kind: motion_hash("wait"),
        motion_frame: frame,
        ..context()
    }
}
fn normal(left: f32) -> Opponent {
    Opponent {
        vulnerable_in: Some(left),
        actionable_in: Some(left + 2.0),
        ..enemy(CLIFF_CLIMB, false)
    }
}
fn drop() -> Opponent {
    Opponent {
        previous_status: CLIFF_WAIT,
        situation: 2,
        position: [5.0, -4.0],
        ..enemy(0xe, true)
    }
}
#[test]
fn stone_uses_its_own_startup_and_releases_to_manual_control_on_native_confirmation() {
    let s = configured(PresetSlot::Neutral, "stone");
    assert_eq!(s.fighter.down_special_startup, Some(11));
    assert_eq!(s.fighter.normal_hit_lead_frames, 0);
    let ctx = LedgeContext {
        input_delay: 2.0,
        ..own(0.0)
    };
    let mut trap = LedgeTrap::default();
    assert_eq!(trap.sample(s, 0, ctx, Some(normal(14.0))).requested, 0);
    let out = trap.sample(
        s,
        1,
        LedgeContext {
            motion_frame: 1.0,
            ..ctx
        },
        Some(normal(13.0)),
    );
    assert_eq!(out.requested, MAPPER_B);
    assert_eq!(out.stick, Some([0.0, -1.0]));
    assert!(!trap.manual_after_response(), "request is not acceptance");
    let started = LedgeContext {
        status: 0x1df,
        motion_kind: motion_hash("special_lw"),
        motion_frame: 1.0,
        can_act: false,
        can_control: false,
        ..ctx
    };
    assert_eq!(trap.sample(s, 2, started, None), LedgeOutput::default());
    assert!(trap.manual_after_response());
    assert_eq!(
        trap.sample(s, 3, own(4.0), Some(normal(0.0))),
        LedgeOutput::default()
    );
    trap.sample(
        s,
        4,
        LedgeContext {
            held: false,
            ..own(5.0)
        },
        None,
    );
    assert_eq!(
        trap.sample(s, 5, own(6.0), Some(normal(11.0))).requested,
        MAPPER_B
    );
}
#[test]
fn stone_respects_variant_timing_grounded_actionability_and_known_vulnerability() {
    let mut s = configured(PresetSlot::Neutral, "stone");
    s.fighter.down_special_startup = Some(17);
    assert_eq!(
        LedgeTrap::default()
            .sample(s, 0, own(0.0), Some(normal(18.0)))
            .requested,
        0
    );
    assert_eq!(
        LedgeTrap::default()
            .sample(s, 0, own(0.0), Some(normal(17.0)))
            .requested,
        MAPPER_B
    );
    for ctx in [
        LedgeContext {
            can_act: false,
            ..own(0.0)
        },
        LedgeContext {
            situation: 2,
            ..own(0.0)
        },
        LedgeContext {
            stopped: true,
            ..own(0.0)
        },
        LedgeContext {
            motion_kind: 0,
            ..own(0.0)
        },
    ] {
        assert_eq!(
            LedgeTrap::default()
                .sample(s, 0, ctx, Some(normal(17.0)))
                .requested,
            0
        );
    }
    for target in [
        Opponent {
            vulnerable_in: None,
            ..normal(17.0)
        },
        Opponent {
            status: 0,
            previous_status: CLIFF_CLIMB,
            ..normal(0.0)
        },
    ] {
        assert_eq!(
            LedgeTrap::default()
                .sample(s, 0, own(0.0), Some(target))
                .requested,
            0
        );
    }
}
#[test]
fn ledge_drop_requests_one_confirmed_tilt_then_shields_after_whiff_or_hit() {
    let s = configured(PresetSlot::Drop, "down_tilt_shield");
    for hit in [false, true] {
        let mut trap = LedgeTrap::default();
        let first = trap.sample(s, 0, own(0.0), Some(drop()));
        assert_eq!(first.requested, MAPPER_A);
        assert_eq!(first.stick, Some([0.0, -0.6]));
        let tilt = LedgeContext {
            status: DOWN_TILT,
            motion_kind: motion_hash("attack_lw3"),
            motion_frame: 4.0,
            can_act: false,
            hit_confirmed: hit,
            ..own(0.0)
        };
        assert_eq!(trap.sample(s, 1, tilt, Some(drop())).requested, 0);
        assert!(!trap.finished, "a hit still needs the shield step");
        let recovered = LedgeContext {
            previous_status: DOWN_TILT,
            ..own(0.0)
        };
        assert_eq!(
            trap.sample(s, 2, recovered, Some(drop())).requested,
            MAPPER_ZR
        );
        for frame in 1..5 {
            assert_eq!(
                trap.sample(s, frame + 2, own(frame as f32), Some(drop()))
                    .requested,
                MAPPER_ZR
            );
        }
    }
}
#[test]
fn drop_variant_waits_for_readiness_and_duplicate_polls_never_spend_retry_time() {
    let s = configured(PresetSlot::Drop, "down_tilt_shield");
    let mut trap = LedgeTrap::default();
    let busy = LedgeContext {
        can_down_tilt: false,
        ..own(0.0)
    };
    assert_eq!(trap.sample(s, 0, busy, Some(drop())).requested, 0);
    let first = trap.sample(s, 1, own(1.0), Some(drop()));
    assert_eq!(first.requested, MAPPER_A);
    for tick in 2..100 {
        // Opponent motion changes, but our native clock is frozen: no extra
        // pulse/retry or artificial timeout is allowed.
        let target = Opponent {
            velocity: [tick as f32 / 100.0, 0.0],
            ..drop()
        };
        assert_eq!(
            trap.sample(s, tick, own(1.0), Some(target)).requested,
            MAPPER_A
        );
    }
    assert_eq!(trap.sample(s, 100, own(2.0), Some(drop())).requested, 0);
    assert_eq!(
        trap.sample(
            s,
            101,
            LedgeContext {
                held: false,
                ..own(3.0)
            },
            Some(drop())
        ),
        LedgeOutput::default()
    );
}
#[test]
fn rejected_drop_inputs_timeout_to_shield_without_repeating_an_accepted_tilt() {
    let mut s = configured(PresetSlot::Drop, "down_tilt_shield");
    s.drop_request_timeout_frames = 3;
    let mut trap = LedgeTrap::default();
    assert_eq!(
        trap.sample(s, 0, own(0.0), Some(drop())).requested,
        MAPPER_A
    );
    for frame in 1..=3 {
        assert_eq!(
            trap.sample(s, frame, own(frame as f32), Some(drop()))
                .requested,
            0
        );
    }
    assert_eq!(
        trap.sample(s, 4, own(4.0), Some(drop())).requested,
        MAPPER_ZR
    );
    assert_eq!(
        trap.sample(s, 5, own(5.0), Some(drop())).requested,
        MAPPER_ZR
    );
}

#[test]
fn interrupted_drop_requests_do_not_throw_a_late_attack_after_damage() {
    let s = configured(PresetSlot::Drop, "down_tilt_shield");
    let mut trap = LedgeTrap::default();
    assert_eq!(
        trap.sample(s, 0, own(0.0), Some(drop())).requested,
        MAPPER_A
    );
    let damaged = LedgeContext {
        status: 0x47,
        can_act: false,
        can_control: false,
        motion_kind: motion_hash("damage"),
        ..own(1.0)
    };
    assert_eq!(trap.sample(s, 1, damaged, Some(drop())).requested, 0);
    assert_eq!(
        trap.sample(s, 2, own(2.0), Some(drop())).requested,
        MAPPER_ZR
    );
    assert_eq!(
        LedgeTrap::default()
            .sample(
                s,
                0,
                own(0.0),
                Some(Opponent {
                    previous_status: 0,
                    ..drop()
                })
            )
            .requested,
        0,
        "a generic fall without ledge evidence is not a drop trigger"
    );
}

#[test]
fn compiled_roll_selection_dispatches_the_down_smash_function() {
    let active = configured(PresetSlot::Roll, "down_smash");
    let out =
        common_roll_state(1.0).sample(active, 0, roll_ready_context(), Some(timed_roll(8.0, 20.0)));
    assert_eq!(out.requested, MAPPER_A);
    assert_eq!(out.stick, Some([0.0, -1.0]));
}

#[test]
fn inhale_times_its_frame_ten_catch_with_eight_frame_early_coverage() {
    let s = configured(PresetSlot::Neutral, "inhale");
    let mut trap = LedgeTrap::default();
    assert_eq!(trap.sample(s, 0, own(0.0), Some(normal(19.0))).requested, 0);
    let out = trap.sample(s, 1, own(1.0), Some(normal(18.0)));
    assert_eq!(out.requested, MAPPER_B);
    assert_eq!(out.stick, Some([0.0, 0.0]));
    let started = LedgeContext {
        status: 0x1dc,
        motion_kind: motion_hash("special_n_start"),
        can_act: false,
        can_control: false,
        ..own(2.0)
    };
    assert_eq!(trap.sample(s, 2, started, None), LedgeOutput::default());
    assert!(trap.manual_after_response());
}

#[test]
fn specials_ignore_spacing_facing_and_narrow_movement_transition_gates() {
    for (id, startup) in [("stone", 11.0), ("inhale", 18.0)] {
        let s = configured(PresetSlot::Neutral, id);
        let ctx = LedgeContext {
            status: 3,
            position: [400.0, 100.0],
            facing: -1.0,
            can_down_special: false,
            can_tilt: false,
            ..own(0.0)
        };
        let out = LedgeTrap::default().sample(s, 0, ctx, Some(normal(startup)));
        assert_eq!(
            out.requested, MAPPER_B,
            "{id} still requests the move while running/facing away"
        );
    }
}

#[test]
fn paired_special_survives_duplicate_polls_stop_and_target_loss_but_releases() {
    let s = configured(PresetSlot::Neutral, "stone");
    let mut trap = LedgeTrap::default();
    assert_eq!(
        trap.sample(s, 0, own(0.0), Some(normal(11.0))).requested,
        MAPPER_B
    );
    for tick in 1..5 {
        assert_eq!(trap.sample(s, tick, own(0.0), None).requested, MAPPER_B);
    }
    let stopped = LedgeContext {
        stopped: true,
        can_act: false,
        can_control: false,
        ..own(0.0)
    };
    assert_eq!(trap.sample(s, 5, stopped, None).requested, MAPPER_B);
    assert_eq!(trap.sample(s, 6, own(1.0), None).requested, MAPPER_B);
    assert_eq!(trap.sample(s, 7, own(2.0), None).requested, 0);
    assert_eq!(trap.sample(s, 8, own(3.0), None).requested, 0);
}

#[test]
fn roll_grab_arrives_before_request_and_capture_returns_all_controls() {
    let s = configured(PresetSlot::Roll, "grab");
    let mut trap = LedgeTrap::default();
    let target = Opponent {
        cliff_position: Some([0.0, 0.0]),
        position: [25.0, 0.0],
        vulnerable_in: Some(7.0),
        actionable_in: Some(15.0),
        ..enemy(CLIFF_ESCAPE, false)
    };
    let approaching = LedgeContext {
        position: [8.0, 0.0],
        can_down_smash: false,
        ..own(0.0)
    };
    let out = trap.sample(s, 0, approaching, Some(target));
    assert_eq!(out.requested, 0);
    assert_eq!(out.stick, Some([1.0, 0.0]));
    let arrived = LedgeContext {
        position: [25.0 - s.fighter.grab.roll_standoff(), 0.0],
        can_down_smash: false,
        ..own(1.0)
    };
    assert_eq!(
        trap.sample(s, 1, arrived, Some(target)).requested,
        0,
        "release approach before grab"
    );
    let ready = LedgeContext {
        motion_frame: 2.0,
        ..arrived
    };
    assert_eq!(
        trap.sample(s, 2, ready, Some(target)).requested,
        s.grab_button
    );
    let captured = LedgeContext {
        status: GRAB_PULL,
        can_act: false,
        ..own(3.0)
    };
    assert_eq!(trap.sample(s, 3, captured, None), LedgeOutput::default());
    assert!(trap.manual_after_grab());
    assert_eq!(
        trap.sample(s, 4, own(4.0), Some(target)),
        LedgeOutput::default()
    );
}

#[test]
fn drop_dair_immediately_pairs_jump_attack_down_and_outward_drift() {
    let s = configured(PresetSlot::Drop, "short_hop_dair");
    assert_eq!(s.drop_action, LedgeDropAction::ShortHopDair);
    let mut trap = LedgeTrap::default();
    let target = Opponent {
        position: [100.0, -4.0],
        vulnerable: false,
        ..drop()
    };
    let out = trap.sample(s, 0, own(0.0), Some(target));
    assert_eq!(out.requested, MAPPER_X | MAPPER_A);
    assert_eq!(
        out.stick,
        Some([crate::fighter_profiles::kirby::DROP_DAIR_SELECT_DRIFT, -1.0])
    );
    assert!(trap.drop_dair_active());
    assert_eq!(trap.action_name(), "short_hop_dair_ledge_drop");
    // Changed opponent coordinates must not consume our native request clock.
    for tick in 1..10 {
        let moved = Opponent {
            position: [100.0 + tick as f32, -20.0],
            ..target
        };
        assert_eq!(trap.sample(s, tick, own(0.0), Some(moved)), out);
    }
}

#[test]
fn drop_dair_uses_locked_ledge_direction_and_continues_without_a_target() {
    let s = configured(PresetSlot::Drop, "short_hop_dair");
    let mut trap = LedgeTrap::default();
    trap.prepare_stance([-8.0, 0.0], 1.0);
    let first = trap.sample(s, 0, own(0.0), Some(drop()));
    assert_eq!(first.requested, MAPPER_X | MAPPER_A);
    assert_eq!(
        first.stick.unwrap()[0],
        -crate::fighter_profiles::kirby::DROP_DAIR_SELECT_DRIFT
    );
    assert!(trap.owns_jump_state(JUMP_SQUAT));
    let squat = LedgeContext {
        status: JUMP_SQUAT,
        can_act: false,
        can_jump: false,
        motion_kind: motion_hash("jump_squat"),
        ..own(0.0)
    };
    let out = trap.sample(s, 1, squat, None);
    assert_eq!(out.requested, MAPPER_A);
    assert_eq!(out.stick.unwrap()[1], -1.0);
    let airborne = LedgeContext {
        status: 0xb,
        situation: 2,
        can_act: false,
        can_jump: false,
        motion_kind: motion_hash("jump_f"),
        ..own(0.0)
    };
    assert!(trap.owns_jump_state(airborne.status));
    assert_eq!(trap.sample(s, 2, airborne, None).requested, MAPPER_A);
    let dair = LedgeContext {
        status: ATTACK_AIR,
        motion_kind: motion_hash("attack_air_lw"),
        ..airborne
    };
    let out = trap.sample(s, 3, dair, None);
    assert_eq!(out.requested, 0, "release attack once dair starts");
    assert_eq!(
        out.stick,
        Some([-1.0, 0.0]),
        "drift without a fast-fall flick"
    );
    for frame in 1..18 {
        let out = trap.sample(
            s,
            frame + 3,
            LedgeContext {
                motion_frame: frame as f32,
                ..dair
            },
            None,
        );
        assert_eq!(out.stick, Some([-1.0, 0.0]));
    }
    let active = LedgeContext {
        motion_frame: 18.0,
        ..dair
    };
    assert_eq!(trap.sample(s, 21, active, None), LedgeOutput::default());
    assert!(trap.manual_after_response());
    assert!(!trap.drop_dair_active());
    assert_eq!(
        trap.sample(s, 22, own(1.0), Some(drop())),
        LedgeOutput::default(),
        "one attempt until L is released"
    );
}

#[test]
fn drop_dair_stop_preserves_directional_attack_without_an_extra_jump() {
    let s = configured(PresetSlot::Drop, "short_hop_dair");
    let mut trap = LedgeTrap::default();
    trap.sample(s, 0, own(0.0), Some(drop()));
    let squat = LedgeContext {
        status: JUMP_SQUAT,
        can_act: false,
        motion_kind: motion_hash("jump_squat"),
        stopped: true,
        ..own(0.0)
    };
    for tick in 1..20 {
        let out = trap.sample(s, tick, squat, None);
        assert_eq!(out.requested, MAPPER_A);
        assert_eq!(out.stick.unwrap()[1], -1.0);
    }
}

#[test]
fn drop_dair_cancels_on_damage_or_release_and_does_not_trigger_on_generic_fall() {
    let s = configured(PresetSlot::Drop, "short_hop_dair");
    let mut trap = LedgeTrap::default();
    let fall = Opponent {
        previous_status: 0,
        ..drop()
    };
    assert_eq!(trap.sample(s, 0, own(0.0), Some(fall)).requested, 0);
    assert!(!trap.drop_dair_active());
    trap.sample(s, 1, own(1.0), Some(drop()));
    let interrupted = LedgeContext {
        status: 0x49,
        situation: 2,
        can_act: false,
        can_control: false,
        ..own(2.0)
    };
    assert_eq!(trap.sample(s, 2, interrupted, None), LedgeOutput::default());
    assert!(trap.manual_after_response());
    let release = LedgeContext {
        held: false,
        ..own(3.0)
    };
    trap.sample(s, 3, release, None);
    assert!(!trap.manual_after_response());
    assert!(!trap.drop_dair_active());
}

#[test]
fn drop_dair_rejected_inputs_have_releases_and_expire_in_native_time() {
    let s = configured(PresetSlot::Drop, "short_hop_dair");
    let mut trap = LedgeTrap::default();
    assert_eq!(
        trap.sample(s, 0, own(0.0), Some(drop())).requested,
        MAPPER_X | MAPPER_A
    );
    assert_eq!(trap.sample(s, 1, own(1.0), None).requested, 0);
    for frame in 2..35 {
        trap.sample(s, frame, own(frame as f32), None);
    }
    assert!(trap.manual_after_response());
    assert!(!trap.drop_dair_active());
}

#[test]
fn starting_cast_up_b_shield_punish_waits_for_block_then_hands_back_on_native_start() {
    for kind in [0, 2, 3] {
        let profiles = FighterProfiles::default();
        let p = profiles.profile(kind).unwrap();
        let selection = MenuSelection::new(&p.menu_options);
        let s = selection.settings(
            &p.menu_options,
            p.apply(LedgeSettings {
                enabled: true,
                ..LedgeSettings::default()
            }),
        );
        assert_eq!(
            p.menu_options.list(PresetSlot::Attack).options[selection.choices[3]].id,
            "shield_up_b"
        );
        let target = enemy(CLIFF_ATTACK, false);
        let guard = LedgeContext {
            status: GUARD,
            can_act: false,
            ..own(1.0)
        };
        let mut trap = LedgeTrap {
            target: Some(target.id),
            last_status: CLIFF_ATTACK,
            plan: OptionPlan::Shield { blocked: false },
            ..LedgeTrap::default()
        };
        assert_eq!(trap.sample(s, 0, guard, Some(target)).requested, MAPPER_ZR);
        trap.plan = OptionPlan::Shield { blocked: true };
        let stun = LedgeContext {
            status: GUARD_DAMAGE,
            ..own(2.0)
        };
        assert_eq!(trap.sample(s, 1, stun, Some(target)).requested, MAPPER_ZR);
        let out = trap.sample(
            s,
            2,
            LedgeContext {
                motion_frame: 3.0,
                ..guard
            },
            Some(target),
        );
        assert_eq!(out.requested, MAPPER_ZR | MAPPER_B);
        assert_eq!(out.stick, Some([0.0, 1.0]));
        assert_eq!(
            trap.sample(
                s,
                3,
                LedgeContext {
                    motion_frame: 4.0,
                    ..guard
                },
                Some(target)
            )
            .requested,
            MAPPER_ZR | MAPPER_B
        );
        assert_eq!(
            trap.sample(
                s,
                4,
                LedgeContext {
                    motion_frame: 5.0,
                    ..guard
                },
                Some(target)
            )
            .requested,
            MAPPER_ZR,
            "B release gaps must retain shield"
        );
        let special = LedgeContext {
            status: 0x1de,
            can_act: false,
            can_control: false,
            motion_kind: motion_hash("special_hi"),
            ..own(0.0)
        };
        assert_eq!(trap.sample(s, 5, special, None), LedgeOutput::default());
        assert!(trap.manual_after_response());
    }
}
