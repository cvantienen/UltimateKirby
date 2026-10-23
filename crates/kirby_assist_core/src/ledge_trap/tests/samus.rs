//! Exercise the real Samus catalog and native input acceptance/handoff paths.
use super::*;
use crate::fighter_profiles::{presets::PresetSlot, FighterProfiles};
use crate::{ledge_menu::MenuSelection, timing::motion_hash};

fn configured(slot: PresetSlot, id: &str) -> LedgeSettings {
    let profiles = FighterProfiles::default();
    let p = profiles.profile(3).unwrap();
    let base = p.apply(LedgeSettings {
        enabled: true,
        manual_spacing: true,
        distance_agnostic: true,
        ..LedgeSettings::default()
    });
    let mut selection = MenuSelection::new(&p.menu_options);
    selection.choices[slot as usize] = p
        .menu_options
        .list(slot)
        .options
        .iter()
        .position(|c| c.id == id)
        .unwrap();
    selection.settings(&p.menu_options, base)
}
fn own(frame: f32) -> LedgeContext {
    LedgeContext {
        motion_kind: motion_hash("wait"),
        motion_frame: frame,
        ..context()
    }
}
#[test]
fn ranged_missiles_fire_from_far_without_roll_chasing_and_finish_on_native_startup() {
    for (id, amount, startup) in [
        ("homing_missile", 0.45, 0x1eb),
        ("super_missile", 1.0, 0x1ed),
    ] {
        for slot in [
            PresetSlot::Neutral,
            PresetSlot::Jump,
            PresetSlot::Roll,
            PresetSlot::Drop,
        ] {
            for facing in [-1.0, 1.0] {
                let s = configured(slot, id);
                let mut trap = LedgeTrap::default();
                let c = LedgeContext {
                    facing,
                    input_delay: 3.0,
                    can_jump: false, // A ground missile must never require a hop.
                    ..own(0.0)
                };
                let hanging = Opponent {
                    position: [60.0 * facing, 0.0],
                    ..enemy(CLIFF_WAIT, false)
                };
                trap.sample(s, 0, c, Some(hanging));
                let target = Opponent {
                    status: match slot {
                        PresetSlot::Neutral => CLIFF_CLIMB,
                        PresetSlot::Jump => CLIFF_JUMP1,
                        PresetSlot::Roll => CLIFF_ESCAPE,
                        PresetSlot::Drop => 0xe,
                        _ => unreachable!(),
                    },
                    previous_status: CLIFF_WAIT,
                    situation: if matches!(slot, PresetSlot::Jump | PresetSlot::Drop) {
                        2
                    } else {
                        0
                    },
                    // No guessed invulnerability expiry or travel-time promise.
                    vulnerable_in: None,
                    actionable_in: None,
                    ..hanging
                };
                let primer = trap.sample(
                    s,
                    1,
                    LedgeContext {
                        motion_frame: 1.0,
                        ..c
                    },
                    Some(target),
                );
                assert_eq!(primer.requested, 0);
                assert_eq!(primer.stick, Some([0.0, 0.0])); // No approach.
                                                            // Once committed, target loss must not drop the B+direction.
                let out = trap.sample(
                    s,
                    2,
                    LedgeContext {
                        motion_frame: 2.0,
                        ..c
                    },
                    None,
                );
                assert_eq!(out.requested, MAPPER_B);
                assert_eq!(out.stick, Some([amount * facing, 0.0]));
                let frozen = LedgeContext {
                    motion_frame: 2.0,
                    stopped: true,
                    can_act: false,
                    ..c
                };
                assert_eq!(trap.sample(s, 3, frozen, None), out);
                let accepted = LedgeContext {
                    status: startup,
                    motion_kind: motion_hash("special_s"),
                    motion_frame: 0.0,
                    can_act: false,
                    ..c
                };
                assert_eq!(trap.sample(s, 4, accepted, None), LedgeOutput::default());
                assert!(trap.manual_after_response());
                assert_eq!(
                    trap.sample(
                        s,
                        5,
                        LedgeContext {
                            motion_frame: 5.0,
                            ..c
                        },
                        Some(target)
                    ),
                    LedgeOutput::default()
                );
            }
        }
    }
}

#[test]
fn samus_catalog_has_all_zair_variants_and_defaults_to_roll_grab_and_drop_bomb() {
    let profiles = FighterProfiles::default();
    assert!(profiles.validate().is_ok());
    let p = profiles.profile(3).unwrap();
    for slot in PresetSlot::ALL {
        assert!(p
            .menu_options
            .list(slot)
            .options
            .iter()
            .any(|c| c.id == "zair" || c.id == "shield_zair"));
    }
    let roll = p.menu_options.list(PresetSlot::Roll);
    assert_eq!(roll.options[roll.selected].id, "grab");
    let drop = p.menu_options.list(PresetSlot::Drop);
    assert_eq!(drop.options[drop.selected].id, "bomb");
    assert_eq!(
        configured(PresetSlot::Drop, "bomb").drop_action,
        LedgeDropAction::Bomb
    );
}
#[test]
fn drop_bomb_is_immediate_despite_facing_or_unknown_protection_and_hands_back_on_start() {
    for native_bomb in [0x1df, 0x1e6, 0x1e7] {
        let s = configured(PresetSlot::Drop, "bomb");
        let mut trap = LedgeTrap::default();
        let c = LedgeContext {
            facing: -1.0,
            ..own(0.0)
        };
        trap.sample(s, 0, c, Some(enemy(CLIFF_WAIT, false)));
        let drop = Opponent {
            previous_status: CLIFF_WAIT,
            situation: 2,
            position: [5.0, -4.0],
            ..enemy(0xe, false)
        };
        let request = trap.sample(
            s,
            1,
            LedgeContext {
                motion_frame: 1.0,
                ..c
            },
            Some(drop),
        );
        assert_eq!(request.requested, MAPPER_B);
        assert_eq!(request.stick, Some([0.0, -1.0]));
        assert!(!trap.manual_after_response());
        let started = LedgeContext {
            status: native_bomb,
            situation: if native_bomb == 0x1e7 { 2 } else { 0 },
            motion_kind: motion_hash("special_lw"),
            motion_frame: 1.0,
            can_act: false,
            can_control: false,
            ..c
        };
        assert_eq!(trap.sample(s, 2, started, None), LedgeOutput::default());
        assert!(trap.manual_after_response());
    }
}
#[test]
fn neutral_zair_uses_middle_of_lingering_window_then_airborne_grab_without_rejump() {
    let s = configured(PresetSlot::Neutral, "zair");
    let mut trap = LedgeTrap::default();
    let climb = |left| Opponent {
        vulnerable_in: Some(left),
        actionable_in: Some(left + 3.0),
        ..enemy(CLIFF_CLIMB, false)
    };
    assert_eq!(trap.sample(s, 0, own(0.0), Some(climb(18.0))).requested, 0);
    let jump = trap.sample(s, 1, own(1.0), Some(climb(17.0)));
    assert_eq!(jump.requested, MAPPER_X);
    assert_eq!(jump.requested & (MAPPER_A | MAPPER_R), 0);
    let squat = LedgeContext {
        status: JUMP_SQUAT,
        motion_kind: motion_hash("jump_squat"),
        motion_frame: 1.0,
        can_act: false,
        ..own(2.0)
    };
    assert_eq!(trap.sample(s, 2, squat, None).requested, 0);
    let air = LedgeContext {
        status: 0xb,
        situation: 2,
        motion_kind: motion_hash("jump_f"),
        motion_frame: 1.0,
        can_act: false,
        ..own(3.0)
    };
    let grab = trap.sample(s, 3, air, None);
    assert_eq!(grab.requested, MAPPER_R);
    assert_eq!(grab.requested & (MAPPER_X | MAPPER_A), 0);
    for frame in 2..=7 {
        let delayed = LedgeContext {
            motion_frame: frame as f32,
            ..air
        };
        assert_eq!(
            trap.sample(s, frame + 2, delayed, None).requested,
            MAPPER_R,
            "keep R until native lasso acceptance, even with delayed delivery"
        );
    }
    let stopped = LedgeContext {
        stopped: true,
        motion_frame: 7.0,
        ..air
    };
    assert_eq!(trap.sample(s, 10, stopped, None).requested, MAPPER_R);
    assert!(trap.owns_jump_state(0xf8));
    for frame in [8.0, 15.0, 19.0] {
        let lasso = LedgeContext {
            status: 0xf8,
            motion_kind: motion_hash("air_catch"),
            motion_frame: frame,
            ..air
        };
        assert_eq!(trap.sample(s, 5 + frame as i64, lasso, None).requested, 0);
        assert!(!trap.manual_after_response());
    }
    let finished = LedgeContext {
        status: 0xf8,
        motion_kind: motion_hash("air_catch"),
        motion_frame: 20.0,
        ..air
    };
    assert_eq!(trap.sample(s, 26, finished, None), LedgeOutput::default());
    assert!(trap.manual_after_response());
}
#[test]
fn jump_and_drop_zair_dispatch_a_jump_and_shield_zair_waits_for_real_block() {
    for slot in [PresetSlot::Jump, PresetSlot::Drop] {
        let s = configured(slot, "zair");
        let mut trap = LedgeTrap::default();
        trap.sample(s, 0, own(0.0), Some(enemy(CLIFF_WAIT, false)));
        let option = if slot == PresetSlot::Jump {
            Opponent {
                situation: 2,
                vulnerable_in: Some(14.0),
                actionable_in: Some(23.0),
                ..enemy(CLIFF_JUMP1, false)
            }
        } else {
            Opponent {
                situation: 2,
                previous_status: CLIFF_WAIT,
                position: [5.0, -4.0],
                ..enemy(0xe, true)
            }
        };
        assert_eq!(
            trap.sample(s, 1, own(1.0), Some(option)).requested,
            MAPPER_X
        );
    }
    let s = configured(PresetSlot::Attack, "shield_zair");
    let mut trap = LedgeTrap::default();
    let attack = enemy(CLIFF_ATTACK, false);
    assert_eq!(
        trap.sample(s, 0, own(0.0), Some(attack)).requested,
        MAPPER_ZR
    );
    let stun = LedgeContext {
        status: GUARD_DAMAGE,
        previous_status: GUARD,
        can_act: false,
        ..own(1.0)
    };
    assert_eq!(trap.sample(s, 1, stun, Some(attack)).requested, MAPPER_ZR);
    let recovered = LedgeContext {
        status: GUARD,
        previous_status: GUARD_DAMAGE,
        can_act: false,
        ..own(2.0)
    };
    assert_eq!(
        trap.sample(s, 2, recovered, Some(attack)).requested,
        MAPPER_X
    );
}

#[test]
fn roll_zair_locks_a_standoff_and_faces_the_destination_instead_of_chasing_roll_root() {
    let s = configured(PresetSlot::Roll, "zair");
    let mut trap = LedgeTrap::default();
    let hang = Opponent {
        position: [-5.0, 0.0],
        cliff_position: Some([-5.0, 0.0]),
        ..enemy(CLIFF_WAIT, false)
    };
    trap.sample(s, 0, own(0.0), Some(hang));
    let roll = Opponent {
        position: [-4.0, 0.0],
        vulnerable_in: Some(40.0),
        actionable_in: Some(45.0),
        ..enemy(CLIFF_ESCAPE, false)
    };
    assert_eq!(
        trap.sample(s, 1, own(1.0), Some(roll)).stick,
        Some([1.0, 0.0])
    );
    let locked = trap.roll_interception().unwrap();
    assert_eq!(locked[0], 20.0);
    assert_eq!(locked[1], 8.0);
    let arrived = LedgeContext {
        position: [8.0, 0.0],
        ..own(2.0)
    };
    let opening = Opponent {
        vulnerable_in: Some(17.0),
        actionable_in: Some(23.0),
        ..roll
    };
    assert_eq!(trap.sample(s, 2, arrived, Some(opening)).requested, 0);
    let jump = trap.sample(
        s,
        3,
        LedgeContext {
            motion_frame: 3.0,
            ..arrived
        },
        Some(opening),
    );
    assert_eq!(jump.requested, MAPPER_X);
    assert_eq!(trap.roll_interception().unwrap(), locked);
}

#[test]
fn roll_grab_uses_locked_distance_and_capture_returns_manual_control() {
    let s = configured(PresetSlot::Roll, "grab");
    let mut trap = LedgeTrap::default();
    let hang = Opponent {
        position: [-5.0, 0.0],
        cliff_position: Some([-5.0, 0.0]),
        ..enemy(CLIFF_WAIT, false)
    };
    trap.sample(s, 0, own(0.0), Some(hang));
    let roll = Opponent {
        position: [-4.0, 0.0],
        vulnerable_in: Some(40.0),
        actionable_in: Some(45.0),
        ..enemy(CLIFF_ESCAPE, false)
    };
    let moving = trap.sample(s, 1, own(1.0), Some(roll));
    assert_eq!(moving.stick, Some([1.0, 0.0]));
    assert_eq!(moving.requested, 0);
    let endpoint = trap.roll_interception().unwrap()[0];
    assert_eq!(endpoint, 20.0);
    let arrived = LedgeContext {
        position: [endpoint - s.fighter.grab.roll_standoff(), 0.0],
        ..own(2.0)
    };
    let late = Opponent {
        position: [endpoint, 0.0],
        vulnerable_in: Some(18.0),
        actionable_in: Some(23.0),
        ..roll
    };
    assert_eq!(trap.sample(s, 2, arrived, Some(late)).requested, 0);
    let grab = trap.sample(
        s,
        3,
        LedgeContext {
            motion_frame: 3.0,
            ..arrived
        },
        Some(late),
    );
    assert_eq!(grab.requested, MAPPER_R);
    let captured = LedgeContext {
        status: GRAB_PULL,
        motion_kind: motion_hash("catch_pull"),
        motion_frame: 1.0,
        can_act: false,
        ..arrived
    };
    assert_eq!(
        trap.sample(s, 4, captured, Some(late)),
        LedgeOutput::default()
    );
    assert!(trap.manual_after_grab());
}
