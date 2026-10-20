//! Host regressions for the grounded ledge policy. These run without a Switch.
// Read one test as arrange -> sample -> assert: construct native-like facts,
// feed a sequence of observations, then check proposed controls and state.
// These fixtures do not advance a real game, simulate hitboxes, or press a pad.
// To reproduce a bug, vary only its relevant fields (e.g. vulnerable_in or
// shield_stun_frames), preserve required routing/eligibility, and include the
// next update to check release/recovery. A button request is not a confirmed hit.
use super::*;
// Fixture helpers keep repeated setup readable. `..context()` copies the
// defaults into fields not explicitly set by a test; an explicit field wins.
fn settings() -> LedgeSettings {
    LedgeSettings {
        enabled: true,
        getup_attack: GetupAttack::DownTilt,
        manual_spacing: false,
        distance_agnostic: false,
        ..LedgeSettings::default()
    }
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
        position: [0.0, 0.0],
        velocity: [0.0, 0.0],
        ground_brake: Some(0.1),
        facing: 1.0,
        previous_status: 0,
        can_grab: true,
        can_tilt: true,
        can_dash_attack: true,
        input_delay: 0.0,
        can_down_smash: true,
        can_jump: true,
        can_air_attack: true,
        air_physics: None,
        short_hop_speed: None,
        shield_stun_frames: None,
        shield_attacker: None,
        hit_confirmed: false,
        safe_ground: [true, true],
    }
}
fn enemy(status: i32, vulnerable: bool) -> Opponent {
    Opponent {
        id: 2,
        status,
        previous_status: 0,
        situation: 0,
        vulnerable,
        position: [5.0, 0.0],
        velocity: [0.0, 0.0],
        cliff_position: None,
        vulnerable_in: None,
        motion: None,
        attack_active: None,
        air_physics: None,
        actionable_in: None,
    }
}
// &mut retains one state machine across successive calls; constructing a new
// LedgeTrap each step would hide bugs in retry limits and previous-input history.
fn step(state: &mut LedgeTrap, tick: i64, ctx: LedgeContext, enemy: Opponent) -> LedgeOutput {
    state.sample(settings(), tick, ctx, Some(enemy))
}

#[test]
fn ledge_drop_defense_shields_through_double_jump_aerial_landing_and_block_hitstop() {
    let mut state = LedgeTrap::default();
    step(&mut state, 0, context(), enemy(CLIFF_WAIT, false));
    let drop = Opponent {
        situation: 2,
        previous_status: CLIFF_WAIT,
        position: [5.0, -4.0],
        ..enemy(0xe, true)
    };
    for (tick, status, previous_status, position) in [
        (1, 0xe, CLIFF_WAIT, [5.0, -4.0]),
        (2, 0xc, 0xe, [5.0, 2.0]),
        (3, 0x33, 0xc, [3.0, 6.0]),
        (4, 0x18, 0x33, [3.0, 0.0]),
    ] {
        let target = Opponent {
            status,
            previous_status,
            position,
            situation: if status == 0x18 { 0 } else { 2 },
            ..drop
        };
        let out = step(&mut state, tick, context(), target);
        assert_eq!(out.requested, MAPPER_ZR);
        assert_eq!(out.stick, Some([0.0, 0.0]));
        assert_eq!(state.action_name(), "shield_ledge_drop_aerial");
    }
    let stopped = LedgeContext {
        status: GUARD_DAMAGE,
        stopped: true,
        can_act: false,
        can_control: false,
        ..context()
    };
    assert_eq!(step(&mut state, 5, stopped, drop).requested, MAPPER_ZR);
    assert_eq!(
        step(
            &mut state,
            6,
            LedgeContext {
                held: false,
                ..context()
            },
            drop
        ),
        LedgeOutput::default()
    );
}

#[test]
fn ledge_drop_defense_requires_hang_evidence_and_can_be_disabled() {
    let drop = Opponent {
        situation: 2,
        ..enemy(0xe, true)
    };
    assert_eq!(step(&mut LedgeTrap::default(), 0, context(), drop).owned, 0);
    let missed_hang = Opponent {
        previous_status: CLIFF_WAIT,
        ..drop
    };
    assert_eq!(
        step(&mut LedgeTrap::default(), 0, context(), missed_hang).requested,
        MAPPER_ZR
    );
    // Remembering the last hang also works when native previous-status history
    // has already advanced through a very short drop before this observation.
    let mut state = LedgeTrap::default();
    step(&mut state, 0, context(), enemy(CLIFF_WAIT, false));
    assert_eq!(step(&mut state, 1, context(), drop).requested, MAPPER_ZR);
    let mut disabled = LedgeTrap::default();
    let config = LedgeSettings {
        drop_defense: false,
        ..settings()
    };
    disabled.sample(config, 0, context(), Some(enemy(CLIFF_WAIT, false)));
    assert_eq!(
        disabled
            .sample(config, 1, context(), Some(missed_hang))
            .requested,
        0
    );
    assert_ne!(disabled.action_name(), "shield_ledge_drop_aerial");
}

#[test]
fn ledge_drop_defense_yields_to_damage_regrabs_normal_jump_and_tracking_timeout() {
    let drop = Opponent {
        situation: 2,
        previous_status: CLIFF_WAIT,
        ..enemy(0xe, true)
    };
    let mut state = LedgeTrap::default();
    step(&mut state, 0, context(), drop);
    let damage = LedgeContext {
        status: 0x47,
        can_control: false,
        can_act: false,
        ..context()
    };
    assert_eq!(step(&mut state, 1, damage, drop).owned, 0);
    step(&mut state, 2, context(), enemy(CLIFF_WAIT, false));
    assert_ne!(state.action_name(), "shield_ledge_drop_aerial");
    step(
        &mut state,
        3,
        context(),
        Opponent {
            situation: 2,
            position: [5.0, 6.0],
            ..enemy(CLIFF_JUMP1, true)
        },
    );
    assert_eq!(state.action_name(), "short_hop_neutral_air");
    let mut expires = LedgeTrap::default();
    step(&mut expires, 0, context(), drop);
    for tick in 1..=i64::from(FOLLOW_UPDATES) + 1 {
        step(&mut expires, tick, context(), drop);
    }
    assert_eq!(expires.last_output.owned, 0);
    assert_eq!(expires.target(), None);
}

#[test]
fn ledge_drop_defense_config_is_optional_and_validated() {
    use crate::live_input::LiveConfig;
    assert!(
        LiveConfig::parse("ledge_trap = true")
            .unwrap()
            .ledge
            .drop_defense
    );
    assert!(
        !LiveConfig::parse("ledge_drop_defense = false")
            .unwrap()
            .ledge
            .drop_defense
    );
    assert!(LiveConfig::parse("ledge_drop_defense = sometimes").is_err());
}

#[test]
fn shield_grab_requires_a_real_block_and_waits_for_shieldstun_to_finish() {
    let mut state = LedgeTrap::default();
    let attack = enemy(CLIFF_ATTACK, true);
    assert_eq!(step(&mut state, 0, context(), attack).requested, MAPPER_ZR);
    let guard = LedgeContext {
        status: 0x1c,
        can_act: false,
        ..context()
    };
    assert_eq!(step(&mut state, 1, guard, attack).requested, MAPPER_ZR);
    let stun = LedgeContext {
        status: 0x1e,
        can_act: false,
        ..context()
    };
    assert_eq!(
        step(
            &mut state,
            2,
            LedgeContext {
                stopped: true,
                can_control: false,
                ..stun
            },
            attack
        )
        .requested,
        MAPPER_ZR
    );
    assert_eq!(step(&mut state, 3, stun, attack).requested, MAPPER_ZR);
    let grab = step(&mut state, 4, guard, attack);
    assert_eq!(grab.requested, MAPPER_ZR | MAPPER_R);
    assert_eq!(step(&mut state, 5, guard, attack).requested, MAPPER_ZR);
    step(
        &mut state,
        6,
        LedgeContext {
            status: 0x37,
            can_act: false,
            ..context()
        },
        attack,
    );
    assert_ne!(state.action_name(), "finished");
    step(
        &mut state,
        7,
        LedgeContext {
            status: 0x38,
            can_act: false,
            ..context()
        },
        attack,
    );
    assert_eq!(state.action_name(), "finished");
}
#[test]
fn a_whiffed_getup_attack_does_not_trigger_a_blind_grab() {
    let mut state = LedgeTrap::default();
    step(&mut state, 0, context(), enemy(CLIFF_ATTACK, true));
    for tick in 1..8 {
        let guard = LedgeContext {
            status: 0x1c,
            can_act: false,
            ..context()
        };
        assert_eq!(
            step(&mut state, tick, guard, enemy(0, true)).requested,
            MAPPER_ZR
        );
    }
}
#[test]
fn shield_hold_has_no_release_edge_through_hitstop_buffer_and_grab_retries() {
    use crate::manual_controls::{ManualOverlay, TurboOutput};
    use crate::native_input::ButtonTrace;
    let mut state = LedgeTrap::default();
    let mut overlay = ManualOverlay::default();
    let attack = enemy(CLIFF_ATTACK, true);
    let guard = LedgeContext {
        status: 0x1c,
        can_act: false,
        ..context()
    };
    let stop = LedgeContext {
        status: 0x1e,
        stopped: true,
        can_control: false,
        shield_stun_frames: Some(2),
        ..guard
    };
    let stun = LedgeContext {
        stopped: false,
        can_control: true,
        ..stop
    };
    // The target finishes its option during the shield-grab retry window.
    let frames = [
        (context(), attack, MAPPER_ZR),
        (guard, attack, MAPPER_ZR),
        (stop, attack, MAPPER_ZR),
        (stop, attack, MAPPER_ZR),
        (stun, attack, MAPPER_ZR | MAPPER_R),
        (guard, enemy(0, true), MAPPER_ZR),
        (guard, enemy(0, true), MAPPER_ZR | MAPPER_R),
        (guard, enemy(0, true), MAPPER_ZR),
    ];
    for (tick, (ctx, target, expected)) in frames.into_iter().enumerate() {
        let out = step(&mut state, tick as i64, ctx, target);
        assert_eq!(out.requested, expected, "update {tick}");
        let mapped = overlay.apply_controls(
            ButtonTrace::default(),
            [0.0, 0.0],
            TurboOutput::default(),
            DPAD_DOWN,
            out.requested,
            out.owned,
        );
        assert_eq!(mapped.buttons.held & MAPPER_ZR, MAPPER_ZR);
        assert_eq!(mapped.buttons.released & MAPPER_ZR, 0);
    }
}
#[test]
fn pushback_pursuit_does_not_raise_shield_when_target_moves_into_range_during_guard_off() {
    let mut state = LedgeTrap::default();
    let attack = enemy(CLIFF_ATTACK, true);
    step(&mut state, 0, context(), attack);
    let guard = LedgeContext {
        status: 0x1c,
        can_act: false,
        ..context()
    };
    step(
        &mut state,
        1,
        LedgeContext {
            status: 0x1e,
            ..guard
        },
        attack,
    );
    let far = Opponent {
        position: [10.0, 0.0],
        ..enemy(0, true)
    };
    assert_eq!(step(&mut state, 2, guard, far).requested, 0);
    let off = LedgeContext {
        status: 0x1d,
        ..guard
    };
    let close = enemy(0, true);
    for tick in 3..7 {
        assert_eq!(step(&mut state, tick, off, close).requested, 0);
    }
    assert_eq!(step(&mut state, 7, context(), close).requested, MAPPER_R);
    assert_eq!(step(&mut state, 8, context(), close).requested, 0);
    assert_eq!(step(&mut state, 9, context(), close).requested, MAPPER_R);
    assert_eq!(state.action_name(), "close_distance_then_grab");
}
#[test]
fn shield_hitstop_preservation_does_not_take_over_damage_or_ignore_trigger_release() {
    let mut state = LedgeTrap::default();
    let attack = enemy(CLIFF_ATTACK, true);
    step(&mut state, 0, context(), attack);
    let damage = LedgeContext {
        stopped: true,
        status: 0x47,
        can_control: false,
        ..context()
    };
    assert_eq!(step(&mut state, 1, damage, attack).owned, 0);
    let released = LedgeContext {
        held: false,
        status: 0x1e,
        ..damage
    };
    assert_eq!(
        step(&mut state, 2, released, attack),
        LedgeOutput::default()
    );
    assert_eq!(state.target(), None);
}
#[test]
fn recovered_shield_grab_whiff_retries_without_raising_shield_again() {
    let mut state = LedgeTrap::default();
    let attack = enemy(CLIFF_ATTACK, true);
    step(&mut state, 0, context(), attack);
    let guard = LedgeContext {
        status: 0x1c,
        can_act: false,
        ..context()
    };
    step(
        &mut state,
        1,
        LedgeContext {
            status: 0x1e,
            ..guard
        },
        attack,
    );
    assert_eq!(
        step(&mut state, 2, guard, attack).requested,
        MAPPER_ZR | MAPPER_R
    );
    let target = enemy(0, true);
    assert_eq!(
        step(
            &mut state,
            3,
            LedgeContext {
                status: 0x37,
                can_act: false,
                ..context()
            },
            target
        )
        .requested,
        0
    );
    assert_eq!(step(&mut state, 4, context(), target).requested, MAPPER_R);
}
#[test]
fn shield_grab_refuses_out_of_range_or_invincible_opponents() {
    let mut state = LedgeTrap::default();
    let attack = enemy(CLIFF_ATTACK, true);
    step(&mut state, 0, context(), attack);
    step(
        &mut state,
        1,
        LedgeContext {
            status: 0x1e,
            can_act: false,
            ..context()
        },
        attack,
    );
    let guard = LedgeContext {
        status: 0x1c,
        can_act: false,
        ..context()
    };
    assert_eq!(
        step(
            &mut state,
            2,
            guard,
            Opponent {
                position: [8.5, 0.0],
                ..attack
            }
        )
        .requested,
        MAPPER_ZR
    );
    assert_eq!(
        step(
            &mut state,
            3,
            guard,
            Opponent {
                vulnerable: false,
                ..attack
            }
        )
        .requested,
        MAPPER_ZR
    );
    assert_eq!(
        step(&mut state, 4, guard, attack).requested,
        MAPPER_ZR | MAPPER_R
    );
}
#[test]
fn rolls_do_not_grab_or_smash_after_the_punish_window_has_closed() {
    let mut state = LedgeTrap::default();
    step(&mut state, 0, context(), enemy(CLIFF_ESCAPE, false));
    for tick in 1..8 {
        let out = step(&mut state, tick, context(), enemy(0, true));
        assert_eq!(out.requested, 0);
        assert_eq!(out.stick, Some([0.0, 0.0]));
    }
    assert_eq!(state.action_name(), "roll_punish_window_missed");
}

#[test]
fn down_tilt_prepares_during_invincibility_but_never_uses_acquisition_range_as_reach() {
    let mut state = LedgeTrap::default();
    assert_eq!(
        step(&mut state, 0, context(), enemy(CLIFF_CLIMB, false)).stick,
        Some([0.0, -0.6])
    );
    assert_eq!(
        step(&mut state, 1, context(), enemy(CLIFF_CLIMB, true)).requested,
        MAPPER_A
    );
    step(
        &mut state,
        2,
        LedgeContext {
            status: 0x2c,
            hit_confirmed: true,
            can_act: false,
            ..context()
        },
        enemy(CLIFF_CLIMB, true),
    );
    assert_eq!(
        step(&mut state, 5, context(), enemy(CLIFF_CLIMB, true)).requested,
        0
    );
    let mut state = LedgeTrap::default();
    let far = Opponent {
        position: [10.5, 0.0],
        ..enemy(CLIFF_CLIMB, true)
    };
    for tick in 0..5 {
        assert_eq!(step(&mut state, tick, context(), far).requested, 0);
    }
}
#[test]
fn vulnerable_hang_does_not_precommit_an_attack_before_a_roll_or_jump() {
    let mut state = LedgeTrap::default();
    for tick in 0..8 {
        assert_eq!(
            step(&mut state, tick, context(), enemy(CLIFF_WAIT, true)).requested,
            0
        );
    }
    let roll = step(&mut state, 8, context(), enemy(CLIFF_ESCAPE, false));
    assert_eq!(roll.requested, 0);
    assert_ne!(roll.stick, Some([0.0, 0.0]));
}
#[test]
fn release_damage_and_target_loss_stop_owned_inputs() {
    let mut state = LedgeTrap::default();
    let jump = enemy(CLIFF_JUMP1, true);
    step(&mut state, 0, context(), jump);
    let released = LedgeContext {
        held: false,
        ..context()
    };
    assert_eq!(step(&mut state, 1, released, jump), LedgeOutput::default());
    assert_eq!(state.target(), None);
    step(&mut state, 2, context(), jump);
    let damaged = LedgeContext {
        status: 0x49,
        can_act: false,
        can_control: false,
        ..context()
    };
    assert_eq!(step(&mut state, 3, damaged, jump).owned, 0);
    assert_eq!(state.sample(settings(), 4, context(), None).owned, 0);
    assert_eq!(state.target(), None);
}
#[test]
fn input_retries_are_bounded_and_stopped_polls_do_not_expire_the_target() {
    let mut state = LedgeTrap::default();
    let climb = enemy(CLIFF_CLIMB, true);
    let mut presses = 0;
    for tick in 0..15 {
        presses += usize::from(step(&mut state, tick, context(), climb).requested == MAPPER_A);
    }
    assert_eq!(presses, usize::from(MAX_ATTEMPTS));
    for tick in 15..100 {
        step(
            &mut state,
            tick,
            LedgeContext {
                stopped: true,
                can_control: false,
                ..context()
            },
            enemy(0, true),
        );
    }
    assert_eq!(state.target(), Some(2));
    for tick in 100..150 {
        step(&mut state, tick, context(), enemy(0, true));
    }
    assert_eq!(state.target(), None);
}
#[test]
fn cannot_acquire_an_unrelated_neutral_target_or_take_over_airborne_movement() {
    let mut state = LedgeTrap::default();
    assert_eq!(step(&mut state, 0, context(), enemy(0, true)).owned, 0);
    let ctx = LedgeContext {
        can_control: false,
        status: 0xb,
        situation: 2,
        can_act: false,
        ..context()
    };
    assert_eq!(
        step(&mut state, 1, ctx, enemy(CLIFF_JUMP1, true)).requested,
        0
    );
    assert_eq!(
        state.sample(
            LedgeSettings::default(),
            2,
            context(),
            Some(enemy(CLIFF_WAIT, true))
        ),
        LedgeOutput::default()
    );
}
#[test]
fn hanging_root_below_stage_acquires_by_cliff_anchor_and_aligns_on_supported_floor() {
    let mut state = LedgeTrap::default();
    let hang = Opponent {
        position: [0.0, -18.0],
        cliff_position: Some([0.0, 0.0]),
        ..enemy(CLIFF_WAIT, false)
    };
    let ctx = LedgeContext {
        position: [12.0, 0.0],
        ..context()
    };
    assert!(in_tracking_range(settings(), ctx.position, hang, false));
    let align = step(&mut state, 0, ctx, hang);
    assert_eq!(state.target(), Some(2));
    assert_eq!(align.stick, Some([-0.3, 0.0]));
    let unsafe_ctx = LedgeContext {
        safe_ground: [false, true],
        ..ctx
    };
    assert_eq!(
        step(&mut state, 1, unsafe_ctx, hang).stick,
        Some([0.0, 0.0])
    );
    let stance = LedgeContext {
        position: [8.0, 0.0],
        ..context()
    };
    assert_eq!(step(&mut state, 2, stance, hang).stick, Some([-0.3, 0.0]));
    assert_eq!(
        step(
            &mut state,
            3,
            LedgeContext {
                facing: -1.0,
                ..stance
            },
            hang
        )
        .requested,
        0
    );
}
#[test]
fn tracked_jump_survives_acquisition_box_without_requesting_an_aerial_sequence() {
    let mut state = LedgeTrap::default();
    let jump = enemy(CLIFF_JUMP1, true);
    step(&mut state, 0, context(), jump);
    let rising = Opponent {
        position: [20.0, 30.0],
        situation: 2,
        ..jump
    };
    assert!(!in_tracking_range(settings(), [0.0, 0.0], rising, false));
    assert!(in_tracking_range(settings(), [0.0, 0.0], rising, true));
    let pursuit = step(&mut state, 1, context(), rising);
    assert_eq!(state.target(), Some(2));
    assert_eq!(pursuit.stick, Some([settings().move_strength, 0.0]));
    assert_eq!(pursuit.requested, 0);
    let airborne = LedgeContext {
        status: 0xb,
        situation: 2,
        can_act: false,
        can_control: false,
        ..context()
    };
    assert_eq!(step(&mut state, 2, airborne, rising).owned, 0);
}

#[test]
fn previous_guard_damage_recovers_a_missed_shieldstun_observation() {
    let mut state = LedgeTrap::default();
    step(&mut state, 0, context(), enemy(CLIFF_ATTACK, true));
    let recovered = LedgeContext {
        status: 0x1c,
        previous_status: 0x1e,
        can_act: false,
        ..context()
    };
    assert_eq!(
        step(&mut state, 1, recovered, enemy(0, true)).requested,
        MAPPER_ZR | MAPPER_R
    );
}
#[test]
fn buffered_grab_only_presses_near_end_of_confirmed_shieldstun() {
    let mut state = LedgeTrap::default();
    let attack = enemy(CLIFF_ATTACK, true);
    step(&mut state, 0, context(), attack);
    let stun = LedgeContext {
        status: 0x1e,
        can_act: false,
        shield_attacker: Some(2),
        shield_stun_frames: Some(10),
        ..context()
    };
    assert_eq!(step(&mut state, 1, stun, attack).requested, MAPPER_ZR);
    let near_end = LedgeContext {
        shield_stun_frames: Some(2),
        ..stun
    };
    assert_eq!(
        step(&mut state, 2, near_end, attack).requested,
        MAPPER_ZR | MAPPER_R
    );
    let mut state = LedgeTrap::default();
    step(&mut state, 0, context(), attack);
    let other_attacker = LedgeContext {
        shield_attacker: Some(3),
        ..near_end
    };
    assert_eq!(
        step(&mut state, 1, other_attacker, attack).requested,
        MAPPER_ZR
    );
}
#[test]
fn shield_pushback_releases_guard_closes_distance_and_grabs_without_reentering() {
    let mut state = LedgeTrap::default();
    let attack = enemy(CLIFF_ATTACK, true);
    step(&mut state, 0, context(), attack);
    step(
        &mut state,
        1,
        LedgeContext {
            status: 0x1e,
            can_act: false,
            ..context()
        },
        attack,
    );
    let far = Opponent {
        position: [10.0, 0.0],
        ..enemy(0, true)
    };
    let guarding = LedgeContext {
        status: 0x1c,
        can_act: false,
        ..context()
    };
    assert_eq!(step(&mut state, 2, guarding, far).requested, 0);
    let off = LedgeContext {
        status: 0x1d,
        can_act: false,
        ..context()
    };
    assert_eq!(step(&mut state, 3, off, far).requested, 0);
    assert_eq!(
        step(&mut state, 4, context(), far).stick,
        Some([settings().move_strength, 0.0])
    );
    let closed = LedgeContext {
        position: [5.0, 0.0],
        ..context()
    };
    assert_eq!(step(&mut state, 5, closed, far).requested, MAPPER_R);
    assert_eq!(
        step(&mut state, 6, LedgeContext { ..closed }, far).requested,
        0
    );
    assert_eq!(
        step(&mut state, 7, LedgeContext { ..closed }, far).requested,
        MAPPER_R
    );
}
#[test]
fn roll_navigation_does_not_depend_on_legacy_auto_position_flag() {
    let mut state = common_roll_state(1.0);
    let config = LedgeSettings {
        auto_position: false,
        ..settings()
    };
    assert_eq!(
        state
            .sample(config, 0, context(), Some(enemy(CLIFF_ESCAPE, false)))
            .stick,
        Some([1.0, 0.0])
    );
    let travel = Opponent {
        position: [28.0, 0.0],
        ..enemy(CLIFF_ESCAPE, false)
    };
    let out = state.sample(config, 1, context(), Some(travel));
    assert_eq!(out.stick, Some([0.0, 0.0]));
    assert_eq!(out.requested, 0);
}
#[test]
fn mapped_grab_button_is_used_without_an_attack_button_request() {
    let mut state = LedgeTrap::default();
    let config = LedgeSettings {
        grab_button: MAPPER_L,
        ..settings()
    };
    let attack = enemy(CLIFF_ATTACK, true);
    state.sample(config, 0, context(), Some(attack));
    let blocked = LedgeContext {
        status: 0x1e,
        can_act: false,
        ..context()
    };
    state.sample(config, 1, blocked, Some(attack));
    let guard = LedgeContext {
        status: 0x1c,
        can_act: false,
        ..context()
    };
    let request = state.sample(config, 2, guard, Some(attack));
    assert_eq!(request.requested, MAPPER_ZR | MAPPER_L);
    assert_eq!(request.requested & (MAPPER_A | MAPPER_R), 0);
}
#[test]
fn direct_r_grab_gets_fresh_edges_even_if_the_physical_r_button_is_held() {
    use crate::manual_controls::{ManualOverlay, TurboOutput};
    use crate::native_input::ButtonTrace;
    let mut state = LedgeTrap::default();
    let mut overlay = ManualOverlay::default();
    let human = ButtonTrace {
        held: MAPPER_R | DPAD_DOWN,
        previous: MAPPER_R | DPAD_DOWN,
        ..ButtonTrace::default()
    };
    let attack = enemy(CLIFF_ATTACK, true);
    let shield = step(&mut state, 0, context(), attack);
    let prepared = overlay.apply_controls(
        human,
        [0.0, 0.0],
        TurboOutput::default(),
        DPAD_DOWN,
        shield.requested,
        shield.owned,
    );
    assert_eq!(prepared.buttons.held & MAPPER_R, 0);
    let guard = LedgeContext {
        status: 0x1c,
        previous_status: 0x1e,
        can_act: false,
        ..context()
    };
    let grab = step(&mut state, 1, guard, attack);
    assert_eq!(grab.requested, MAPPER_ZR | MAPPER_R);
    let applied = overlay.apply_controls(
        human,
        [0.0, 0.0],
        TurboOutput::default(),
        DPAD_DOWN,
        grab.requested,
        grab.owned,
    );
    assert_eq!(applied.buttons.pressed & MAPPER_R, MAPPER_R);
    assert_eq!(applied.buttons.held & MAPPER_A, 0);
    let release = step(&mut state, 2, guard, attack);
    let applied = overlay.apply_controls(
        human,
        [0.0, 0.0],
        TurboOutput::default(),
        DPAD_DOWN,
        release.requested,
        release.owned,
    );
    assert_eq!(applied.buttons.released & MAPPER_R, MAPPER_R);
}

#[test]
fn neutral_air_waits_for_vertical_reach_before_starting_short_hop() {
    let mut state = LedgeTrap::default();
    let ctx = LedgeContext {
        facing: -1.0,
        ..context()
    };
    step(&mut state, 0, ctx, enemy(CLIFF_WAIT, false));
    let low = Opponent {
        situation: 2,
        position: [5.0, -12.0],
        ..enemy(CLIFF_JUMP1, true)
    };
    assert_eq!(step(&mut state, 1, ctx, low).requested, 0);
    let high = Opponent {
        position: [5.0, 5.0],
        ..low
    };
    assert_eq!(
        step(&mut state, 2, ctx, high).requested,
        (MAPPER_X | MAPPER_A)
    );
}

#[test]
fn grab_retries_after_one_release_update_without_the_attack_cooldown() {
    let mut state = LedgeTrap::default();
    step(&mut state, 0, context(), enemy(CLIFF_ATTACK, false));
    state.plan = OptionPlan::Shield { blocked: true };
    state.shield_pursuing = true;
    assert_eq!(
        step(&mut state, 1, context(), enemy(0, true)).requested,
        MAPPER_R
    );
    assert_eq!(step(&mut state, 2, context(), enemy(0, true)).requested, 0);
    assert_eq!(
        step(&mut state, 3, context(), enemy(0, true)).requested,
        MAPPER_R
    );
}
#[test]
fn an_early_grab_buffer_does_not_consume_the_fresh_ready_guard_budget() {
    let mut state = LedgeTrap::default();
    let attack = enemy(CLIFF_ATTACK, true);
    step(&mut state, 0, context(), attack);
    let buffering = LedgeContext {
        status: 0x1e,
        shield_stun_frames: Some(3),
        can_act: false,
        ..context()
    };
    assert_eq!(
        step(&mut state, 1, buffering, attack).requested,
        MAPPER_ZR | MAPPER_R
    );
    assert_eq!(step(&mut state, 2, buffering, attack).requested, MAPPER_ZR);
    assert_eq!(step(&mut state, 3, buffering, attack).requested, MAPPER_ZR);
    let ready = LedgeContext {
        status: 0x1c,
        can_act: false,
        ..context()
    };
    assert_eq!(
        step(&mut state, 4, ready, attack).requested,
        MAPPER_ZR | MAPPER_R
    );
}
#[test]
fn whiffed_grab_animation_is_not_capture_and_can_retry_on_recovery() {
    let mut state = LedgeTrap::default();
    step(&mut state, 0, context(), enemy(CLIFF_ATTACK, false));
    state.plan = OptionPlan::Shield { blocked: true };
    state.shield_pursuing = true;
    let target = enemy(0, true);
    assert_eq!(step(&mut state, 1, context(), target).requested, MAPPER_R);
    let startup = LedgeContext {
        status: 0x37,
        can_act: false,
        ..context()
    };
    assert_eq!(step(&mut state, 2, startup, target).requested, 0);
    assert_ne!(state.action_name(), "finished");
    assert_eq!(step(&mut state, 3, context(), target).requested, MAPPER_R);
    let capture = LedgeContext {
        status: 0x38,
        can_act: false,
        ..context()
    };
    step(&mut state, 4, capture, target);
    assert_eq!(state.action_name(), "finished");
    assert_eq!(step(&mut state, 5, context(), target).requested, 0);
}
#[test]
fn actual_grab_whiff_retries_are_bounded_to_six_animation_starts() {
    let mut state = LedgeTrap::default();
    step(&mut state, 0, context(), enemy(CLIFF_ATTACK, false));
    state.plan = OptionPlan::Shield { blocked: true };
    state.shield_pursuing = true;
    let target = enemy(0, true);
    let startup = LedgeContext {
        status: 0x37,
        can_act: false,
        ..context()
    };
    for attempt in 0..6 {
        let tick = attempt * 2 + 1;
        assert_eq!(
            step(&mut state, tick, context(), target).requested,
            MAPPER_R
        );
        step(&mut state, tick + 1, startup, target);
    }
    assert_eq!(step(&mut state, 13, context(), target).requested, 0);
}

#[test]
fn native_motion_deadlines_respect_rates_and_reject_unknown_metadata() {
    assert_eq!(frames_until(10.0, 16.0, 0.5), Some(12.0));
    assert_eq!(frames_until(10.0, 16.0, 2.0), Some(3.0));
    for (frame, end, rate) in [
        (f32::NAN, 16.0, 1.0),
        (10.0, 16.0, 0.0),
        (10.0, 0.0, 1.0),
        (10.0, 9.0, 1.0),
        (10.0, 16.0, f32::NAN),
    ] {
        assert_eq!(frames_until(frame, end, rate), None);
    }
}

#[test]
fn small_fast_jump_is_predicted_before_current_root_enters_neutral_air_box() {
    let mut state = LedgeTrap::default();
    let ctx = LedgeContext {
        facing: -1.0,
        ..context()
    };
    step(&mut state, 0, ctx, enemy(CLIFF_WAIT, false));
    let rising = Opponent {
        situation: 2,
        position: [5.0, -15.0],
        velocity: [0.0, 2.0],
        vulnerable_in: Some(4.0),
        ..enemy(CLIFF_JUMP1, false)
    };
    assert_eq!(
        step(&mut state, 1, ctx, rising).requested,
        (MAPPER_X | MAPPER_A)
    );
}

#[test]
fn jump_that_will_leave_reach_is_followed_instead_of_poking_its_current_root() {
    let mut state = LedgeTrap::default();
    step(&mut state, 0, context(), enemy(CLIFF_WAIT, false));
    let escaping = Opponent {
        situation: 2,
        position: [7.0, 8.0],
        velocity: [3.0, 4.0],
        ..enemy(CLIFF_JUMP1, true)
    };
    let out = step(&mut state, 1, context(), escaping);
    assert_eq!(out.requested, 0);
    assert_eq!(out.stick, Some([settings().move_strength, 0.0]));
    let unsupported = LedgeContext {
        safe_ground: [true, false],
        ..context()
    };
    assert_eq!(
        step(&mut state, 2, unsupported, escaping).stick,
        Some([0.0, 0.0])
    );
}

#[test]
fn short_jump_status_is_recovered_from_native_history_and_remembered_while_kirby_is_busy() {
    let mut state = LedgeTrap::default();
    step(
        &mut state,
        0,
        LedgeContext {
            facing: -1.0,
            ..context()
        },
        enemy(CLIFF_WAIT, false),
    );
    let jumping = Opponent {
        situation: 2,
        position: [5.0, 5.0],
        ..enemy(CLIFF_JUMP1, true)
    };
    let busy = LedgeContext {
        status: GUARD_OFF,
        can_act: false,
        ..LedgeContext {
            facing: -1.0,
            ..context()
        }
    };
    step(&mut state, 1, busy, jumping);
    assert_eq!(state.action_name(), "short_hop_neutral_air");
    let air = Opponent {
        status: 0xb,
        previous_status: CLIFF_JUMP3,
        ..jumping
    };
    assert_eq!(
        step(
            &mut state,
            2,
            LedgeContext {
                facing: -1.0,
                ..context()
            },
            air
        )
        .requested,
        (MAPPER_X | MAPPER_A)
    );
    assert_eq!(
        step(
            &mut state,
            3,
            LedgeContext {
                facing: -1.0,
                ..context()
            },
            air
        )
        .requested,
        0
    );
    let mut missed = LedgeTrap::default();
    step(
        &mut missed,
        0,
        LedgeContext {
            facing: -1.0,
            ..context()
        },
        enemy(CLIFF_WAIT, false),
    );
    assert_eq!(
        step(
            &mut missed,
            1,
            LedgeContext {
                facing: -1.0,
                ..context()
            },
            air
        )
        .requested,
        (MAPPER_X | MAPPER_A)
    );
}

#[test]
fn normal_getup_waits_in_place_then_prepares_when_target_enters_reach() {
    let mut state = LedgeTrap::default();
    let far = Opponent {
        position: [12.0, 0.0],
        vulnerable_in: Some(4.0),
        ..enemy(CLIFF_CLIMB, false)
    };
    let waiting = step(&mut state, 0, context(), far);
    assert_eq!(waiting.stick.unwrap()[0], 0.0);
    assert_eq!(waiting.requested, 0);
    let close = LedgeContext {
        position: [6.0, 0.0],
        ..context()
    };
    assert_eq!(step(&mut state, 1, close, far).requested, 0);
    assert_eq!(step(&mut state, 2, close, far).requested, MAPPER_A);
}

#[test]
fn selectable_forward_tilt_sends_direction_with_attack_without_preparatory_walking() {
    let config = LedgeSettings {
        getup_attack: GetupAttack::ForwardTilt,
        ..settings()
    };
    let mut state = LedgeTrap::default();
    let climb = Opponent {
        vulnerable_in: Some(8.0),
        ..enemy(CLIFF_CLIMB, false)
    };
    let prep = state.sample(config, 0, context(), Some(climb));
    assert_eq!(prep.stick, Some([0.6, -0.3]));
    assert_eq!(prep.requested, MAPPER_A);
    assert_eq!(
        state
            .sample(
                config,
                1,
                context(),
                Some(Opponent {
                    vulnerable_in: Some(9.0),
                    ..climb
                })
            )
            .requested,
        0
    );
    assert_eq!(
        state.sample(config, 2, context(), Some(climb)).requested,
        MAPPER_A
    );
    let hit = LedgeContext {
        status: FORWARD_TILT,
        can_act: false,
        hit_confirmed: true,
        ..context()
    };
    state.sample(config, 3, hit, Some(climb));
    assert_eq!(state.action_name(), "finished");
    let mut low = LedgeTrap::default();
    step(&mut low, 0, context(), climb);
    assert_eq!(
        step(&mut low, 1, context(), climb).requested,
        0,
        "down tilt is inactive by expiry frame 8"
    );
}

#[test]
fn forward_tilt_waits_neutral_during_protection_recovery_and_retry_release() {
    let config = LedgeSettings {
        getup_attack: GetupAttack::ForwardTilt,
        ..settings()
    };
    for facing in [-1.0, 1.0] {
        let ctx = LedgeContext {
            facing,
            ..context()
        };
        let climb = Opponent {
            position: [facing * 5.0, 0.0],
            vulnerable_in: Some(9.0),
            ..enemy(CLIFF_CLIMB, false)
        };
        let mut state = LedgeTrap::default();
        let waiting = state.sample(config, 0, ctx, Some(climb));
        assert_eq!(waiting.stick, Some([0.0, 0.0]));
        assert_eq!(waiting.requested, 0);
        let ready = Opponent {
            vulnerable_in: Some(8.0),
            ..climb
        };
        let walking = LedgeContext { status: 1, ..ctx };
        let settling = state.sample(config, 1, walking, Some(ready));
        assert_eq!(settling.stick, Some([0.0, 0.0]));
        assert_eq!(settling.requested, 0);
        let attack = state.sample(config, 2, ctx, Some(ready));
        assert_eq!(attack.stick, Some([facing * 0.6, -0.3]));
        assert_eq!(attack.requested, MAPPER_A);
        let release = state.sample(config, 3, ctx, Some(ready));
        assert_eq!(release.stick, Some([0.0, 0.0]));
        assert_eq!(release.requested, 0);
    }
}

#[test]
fn forward_tilt_waits_out_of_range_and_turns_when_rear_target_can_be_hit() {
    let config = LedgeSettings {
        getup_attack: GetupAttack::ForwardTilt,
        ..settings()
    };
    for x in [15.0, -5.0] {
        let mut state = LedgeTrap::default();
        let climb = Opponent {
            position: [x, 0.0],
            ..enemy(CLIFF_CLIMB, true)
        };
        state.sample(config, 0, context(), Some(enemy(CLIFF_WAIT, false)));
        let out = state.sample(config, 1, context(), Some(climb));
        assert_eq!(out.stick, Some([if x < 0.0 { -0.3 } else { 0.0 }, 0.0]));
        assert_eq!(out.requested, 0);
    }
}

// Vulnerability and recovery are game-frame deadlines supplied by native motion.
fn timed_roll(vulnerable_in: f32, recovery: f32) -> Opponent {
    Opponent {
        position: [25.0, 0.0],
        velocity: [0.1, 0.0],
        vulnerable_in: Some(vulnerable_in),
        motion: Some(MotionTiming {
            kind: 1,
            frame: 10.0,
            rate: 1.0,
            remaining: Some(recovery),
        }),
        ..enemy(CLIFF_ESCAPE, false)
    }
}

#[test]
fn shield_grab_native_transition_terms_match_selected_movement() {
    for status in [0, 1, 2, GUARD, GUARD_DAMAGE] {
        assert_eq!(grab_transition_term(status), 0x1e00006f);
    }
    for status in [3, 4] {
        assert_eq!(grab_transition_term(status), 0x1e000070);
    }
    for status in [7, 8] {
        assert_eq!(grab_transition_term(status), 0x1e000071);
    }
}

#[test]
fn root_displacement_follows_animation_driven_rolls_when_kinetic_velocity_is_zero() {
    let mut state = LedgeTrap::default();
    let roll = Opponent {
        position: [2.0, 0.0],
        motion: Some(MotionTiming {
            kind: 7,
            frame: 10.0,
            rate: 0.5,
            remaining: Some(10.0),
        }),
        ..enemy(CLIFF_ESCAPE, false)
    };
    assert_eq!(step(&mut state, 0, context(), roll).requested, 0);
    let moving = Opponent {
        position: [4.0, 0.0],
        motion: Some(MotionTiming {
            kind: 7,
            frame: 10.5,
            rate: 0.5,
            remaining: Some(9.0),
        }),
        ..roll
    };
    step(&mut state, 1, context(), moving);
    assert_eq!(state.observed_velocity[0], 2.0);
    assert_eq!(step(&mut state, 2, context(), moving).requested, 0);
}

#[test]
fn stance_anticipates_kirbys_momentum_on_both_ledges_and_keeps_floor_safety() {
    for side in [-1.0, 1.0] {
        let mut state = LedgeTrap::default();
        let hanging = Opponent {
            position: [0.0, -10.0],
            cliff_position: Some([0.0, 0.0]),
            ..enemy(CLIFF_WAIT, false)
        };
        let ctx = LedgeContext {
            position: [8.0 * side, 0.0],
            velocity: [2.0 * side, 0.0],
            ground_brake: Some(0.1),
            facing: side,
            ..context()
        };
        assert_eq!(
            step(&mut state, 0, ctx, hanging).stick,
            Some([-0.3 * side, 0.0])
        );
        let unsafe_ctx = LedgeContext {
            safe_ground: [false, false],
            ..ctx
        };
        assert_eq!(
            step(&mut state, 1, unsafe_ctx, hanging).stick,
            Some([0.0, 0.0])
        );
    }
}

#[test]
fn whiffed_tilts_retry_on_recovery_but_are_bounded_and_hit_confirmation_ends_the_visit() {
    let mut state = LedgeTrap::default();
    let climb = enemy(CLIFF_CLIMB, true);
    step(&mut state, 0, context(), climb);
    for attempt in 0..i64::from(MAX_ATTEMPTS) {
        let tick = attempt * 3 + 1;
        assert_eq!(step(&mut state, tick, context(), climb).requested, MAPPER_A);
        let whiff = LedgeContext {
            status: DOWN_TILT,
            can_act: false,
            ..context()
        };
        assert_eq!(step(&mut state, tick + 1, whiff, climb).requested, 0);
        assert_ne!(state.action_name(), "finished");
        step(&mut state, tick + 2, context(), climb);
    }
    assert_eq!(
        step(
            &mut state,
            i64::from(MAX_ATTEMPTS) * 3 + 2,
            context(),
            climb
        )
        .requested,
        0
    );
    let mut hit = LedgeTrap::default();
    step(&mut hit, 0, context(), climb);
    step(&mut hit, 1, context(), climb);
    step(
        &mut hit,
        2,
        LedgeContext {
            status: DOWN_TILT,
            can_act: false,
            hit_confirmed: true,
            ..context()
        },
        climb,
    );
    assert_eq!(hit.action_name(), "finished");
}

#[test]
fn observed_getup_attack_whiff_punishes_after_hitboxes_clear_without_guessing_startup() {
    let mut state = LedgeTrap::default();
    let attack = Opponent {
        attack_active: Some(false),
        ..enemy(CLIFF_ATTACK, true)
    };
    step(&mut state, 0, context(), attack);
    let guard = LedgeContext {
        status: GUARD,
        can_act: false,
        ..context()
    };
    assert_eq!(step(&mut state, 1, guard, attack).requested, MAPPER_ZR);
    let active = Opponent {
        attack_active: Some(true),
        ..attack
    };
    assert_eq!(step(&mut state, 2, guard, active).requested, MAPPER_ZR);
    assert_eq!(
        step(&mut state, 3, guard, attack).requested,
        MAPPER_ZR | MAPPER_R
    );
}

#[test]
fn duplicate_native_animation_polls_keep_the_press_until_game_advances_then_release_before_retry() {
    let mut state = LedgeTrap::default();
    let climb = Opponent {
        motion: Some(MotionTiming {
            kind: 9,
            frame: 10.0,
            rate: 1.0,
            remaining: Some(20.0),
        }),
        ..enemy(CLIFF_CLIMB, true)
    };
    step(&mut state, 0, context(), climb);
    let advance = |frame| Opponent {
        motion: Some(MotionTiming {
            frame,
            ..climb.motion.unwrap()
        }),
        ..climb
    };
    assert_eq!(
        step(&mut state, 1, context(), advance(11.0)).requested,
        MAPPER_A
    );
    for tick in 2..10 {
        assert_eq!(
            step(&mut state, tick, context(), advance(11.0)).requested,
            MAPPER_A
        );
    }
    assert_eq!(
        step(&mut state, 10, context(), advance(12.0)).requested,
        0,
        "real release before another A edge"
    );
    assert_eq!(
        step(&mut state, 11, context(), advance(13.0)).requested,
        MAPPER_A
    );
    let released = LedgeContext {
        held: false,
        ..context()
    };
    assert_eq!(
        step(&mut state, 12, released, advance(13.0)),
        LedgeOutput::default()
    );
}

#[test]
fn rejected_tilt_retries_on_first_released_update_without_extra_timer() {
    let mut state = LedgeTrap::default();
    let climb = enemy(CLIFF_CLIMB, true);
    assert_eq!(step(&mut state, 0, context(), climb).requested, 0);
    assert_eq!(step(&mut state, 1, context(), climb).requested, MAPPER_A);
    assert_eq!(step(&mut state, 2, context(), climb).requested, 0);
    assert_eq!(step(&mut state, 3, context(), climb).requested, MAPPER_A);
}

#[test]
fn native_history_recovers_missed_roll_and_normal_getup_before_first_acquisition() {
    let mut roll = LedgeTrap::default();
    let after_roll = Opponent {
        previous_status: CLIFF_ESCAPE,
        ..enemy(0, true)
    };
    assert_eq!(step(&mut roll, 0, context(), after_roll).requested, 0);
    assert_eq!(roll.target(), Some(after_roll.id));

    let mut climb = LedgeTrap::default();
    let after_climb = Opponent {
        previous_status: CLIFF_CLIMB,
        ..enemy(0, true)
    };
    step(&mut climb, 0, context(), after_climb);
    assert_eq!(
        step(&mut climb, 1, context(), after_climb).requested,
        MAPPER_A
    );

    // An ordinary nearby idle fighter with no ledge evidence is still ignored.
    let mut unrelated = LedgeTrap::default();
    assert_eq!(step(&mut unrelated, 0, context(), enemy(0, true)).owned, 0);
}

#[test]
fn shield_grab_predicts_vulnerability_and_reach_including_remaining_shieldstun() {
    let stun = LedgeContext {
        status: GUARD_DAMAGE,
        shield_stun_frames: Some(2),
        can_act: false,
        ..context()
    };
    // Confirmed block; attack hitboxes have cleared, protection lasts until
    // the estimated grab startup + two remaining shieldstun frames.
    let attack = Opponent {
        attack_active: Some(false),
        vulnerable_in: Some(12.0),
        ..enemy(CLIFF_ATTACK, false)
    };
    let mut early = LedgeTrap::default();
    assert_eq!(
        step(&mut early, 0, stun, attack).requested,
        MAPPER_ZR | MAPPER_R
    );
    for rejected in [
        Opponent {
            vulnerable_in: Some(14.0),
            ..attack
        },
        Opponent {
            vulnerable_in: None,
            ..attack
        },
        Opponent {
            velocity: [2.0, 0.0],
            ..attack
        },
    ] {
        let mut state = LedgeTrap::default();
        assert_eq!(step(&mut state, 0, stun, rejected).requested, MAPPER_ZR);
    }
}

#[test]
fn confirmed_block_buffers_grab_before_getup_attack_hitboxes_clear() {
    let mut state = LedgeTrap::default();
    let attack = Opponent {
        attack_active: Some(true),
        ..enemy(CLIFF_ATTACK, true)
    };
    let guard = LedgeContext {
        status: GUARD,
        can_act: false,
        ..context()
    };
    // Active hitboxes alone are not block evidence.
    assert_eq!(step(&mut state, 0, guard, attack).requested, MAPPER_ZR);
    let stun = LedgeContext {
        status: GUARD_DAMAGE,
        shield_attacker: Some(attack.id),
        shield_stun_frames: Some(10),
        ..guard
    };
    assert_eq!(step(&mut state, 1, stun, attack).requested, MAPPER_ZR);
    let near_end = LedgeContext {
        shield_stun_frames: Some(2),
        ..stun
    };
    assert_eq!(
        step(&mut state, 2, near_end, attack).requested,
        MAPPER_ZR | MAPPER_R
    );
    // Preserve shield through the release gap, then retry as soon as ready,
    // even if the attacker's hitboxes are still reported active.
    assert_eq!(step(&mut state, 3, guard, attack).requested, MAPPER_ZR);
    assert_eq!(
        step(&mut state, 4, guard, attack).requested,
        MAPPER_ZR | MAPPER_R
    );
}

// Acquire the real ledge anchor, then request a shield grab from a stage-side
// position. This prepares native-like capture observations for both ledges.
fn shield_grab_at_ledge(anchor_x: f32, inward: f32) -> (LedgeTrap, LedgeContext, Opponent) {
    let mut state = LedgeTrap::default();
    let ctx = LedgeContext {
        position: [anchor_x + inward * 8.0, 0.0],
        facing: -inward,
        ..context()
    };
    let hanging = Opponent {
        position: [anchor_x, -4.0],
        cliff_position: Some([anchor_x, 0.0]),
        ..enemy(CLIFF_WAIT, false)
    };
    step(&mut state, 0, ctx, hanging);
    let rolling = Opponent {
        position: [anchor_x + inward * 4.0, 0.0],
        ..enemy(CLIFF_ATTACK, true)
    };
    let blocked = LedgeContext {
        status: GUARD_DAMAGE,
        shield_stun_frames: Some(0),
        ..ctx
    };
    assert_eq!(
        step(&mut state, 1, blocked, rolling).requested,
        MAPPER_ZR | MAPPER_R
    );
    (state, ctx, rolling)
}

#[test]
fn captured_shield_grab_immediately_returns_pummels_and_throw_selection_to_player() {
    use crate::manual_controls::{ManualOverlay, TurboOutput};
    use crate::native_input::ButtonTrace;
    for (anchor, inward) in [(0.0, 1.0), (100.0, -1.0)] {
        for legacy_throw_setting in [false, true] {
            for captured_status in [GRAB_PULL, DASH_GRAB_PULL, GRAB_HOLD, GRAB_PUMMEL] {
                let (mut state, ctx, _) = shield_grab_at_ledge(anchor, inward);
                let config = LedgeSettings {
                    throw_outward: legacy_throw_setting,
                    ..manual_settings()
                };
                let captured = LedgeContext {
                    status: captured_status,
                    can_act: false,
                    ..ctx
                };
                // Target metadata may disappear as the capture begins.
                let out = state.sample(config, 2, captured, None);
                assert_eq!(out, LedgeOutput::default());
                assert!(state.manual_after_grab());
                assert_eq!(state.action_name(), "manual_after_grab");
                let mut overlay = ManualOverlay::default();
                overlay.apply_controls(
                    ButtonTrace::default(),
                    [0.0; 2],
                    TurboOutput {
                        consume_dpad: false,
                        stick: Some([0.0; 2]),
                    },
                    0,
                    MAPPER_R | MAPPER_ZR,
                    LEDGE_BUTTONS,
                );
                for physical_stick in [[0.0, 0.0], [-1.0, 0.0], [1.0, 0.0], [0.0, -1.0], [0.0, 1.0]]
                {
                    let human = ButtonTrace {
                        held: MAPPER_A,
                        pressed: MAPPER_A,
                        ..ButtonTrace::default()
                    };
                    let mapped = overlay.apply_controls(
                        human,
                        physical_stick,
                        TurboOutput {
                            consume_dpad: false,
                            stick: out.stick,
                        },
                        0,
                        out.requested,
                        out.owned,
                    );
                    assert_eq!(
                        mapped.buttons.held & MAPPER_A,
                        MAPPER_A,
                        "manual pummel stays available"
                    );
                    assert_eq!(mapped.buttons.held & (MAPPER_R | MAPPER_ZR), 0);
                    assert_eq!(
                        mapped.stick, physical_stick,
                        "manual throw selection is uncapped"
                    );
                }
                assert_eq!(
                    state.sample(
                        config,
                        3,
                        LedgeContext {
                            stopped: true,
                            ..captured
                        },
                        None
                    ),
                    LedgeOutput::default()
                );
            }
        }
    }
}

#[test]
fn hit_confirmation_survives_target_loss_returns_to_stance_and_rearms_on_regrab() {
    let (mut state, ctx, rolling) = shield_grab_at_ledge(0.0, 1.0);
    // Switch to normal get-up, prepare and request the tilt.
    let climb = Opponent {
        status: CLIFF_CLIMB,
        ..rolling
    };
    step(&mut state, 2, ctx, climb);
    assert_eq!(step(&mut state, 3, ctx, climb).requested, MAPPER_A);
    let hit = LedgeContext {
        status: DOWN_TILT,
        can_act: false,
        hit_confirmed: true,
        ..ctx
    };
    state.sample(settings(), 4, hit, None);
    assert_eq!(state.action_name(), "finished");
    let displaced = LedgeContext {
        position: [16.0, 0.0],
        ..ctx
    };
    let returning = state.sample(settings(), 5, displaced, None);
    assert_eq!(returning.stick, Some([-settings().move_strength, 0.0]));
    assert_eq!(returning.requested, 0);
    let unsafe_floor = LedgeContext {
        safe_ground: [false, true],
        ..displaced
    };
    assert_eq!(
        state.sample(settings(), 6, unsafe_floor, None).stick,
        Some([0.0, 0.0])
    );
    let next_hang = Opponent {
        position: [0.0, -4.0],
        cliff_position: Some([0.0, 0.0]),
        ..enemy(CLIFF_WAIT, false)
    };
    step(&mut state, 7, ctx, next_hang);
    assert_ne!(state.action_name(), "finished");
    assert_eq!(
        step(
            &mut state,
            8,
            LedgeContext {
                status: GUARD_DAMAGE,
                shield_stun_frames: Some(0),
                ..ctx
            },
            rolling
        )
        .requested,
        MAPPER_ZR | MAPPER_R
    );
}

#[test]
fn normal_getup_entering_idle_keeps_the_prepared_punish_instead_of_becoming_a_hang() {
    let mut state = LedgeTrap::default();
    let preparing = Opponent {
        vulnerable_in: Some(7.0),
        ..enemy(CLIFF_CLIMB, false)
    };
    assert_eq!(step(&mut state, 0, context(), preparing).requested, 0);
    // Status 0 is idle, NOT a hang, even though 0 <= CLIFF_WAIT. Also prove
    // the remembered climb works when native previous-status metadata is gone.
    assert_eq!(
        step(&mut state, 1, context(), enemy(0, true)).requested,
        MAPPER_A
    );
}

#[test]
fn captured_grab_keeps_autopilot_off_until_y_is_released_and_rearmed() {
    let (mut state, ctx, rolling) = shield_grab_at_ledge(0.0, 1.0);
    let hold = LedgeContext {
        status: GRAB_HOLD,
        can_act: false,
        ..ctx
    };
    step(
        &mut state,
        2,
        hold,
        Opponent {
            status: 0x42,
            ..rolling
        },
    );
    step(
        &mut state,
        3,
        ctx,
        Opponent {
            status: 0x4b,
            situation: 2,
            ..rolling
        },
    );
    let jump = Opponent {
        status: CLIFF_JUMP1,
        situation: 2,
        ..rolling
    };
    let response = step(&mut state, 4, ctx, jump);
    assert_eq!(response, LedgeOutput::default());
    assert!(state.manual_after_grab());
    assert_eq!(
        step(&mut state, 5, LedgeContext { held: false, ..ctx }, jump),
        LedgeOutput::default()
    );
    assert!(!state.manual_after_grab());
    let response = step(&mut state, 6, ctx, jump);
    assert_eq!(response.requested, (MAPPER_X | MAPPER_A));
    assert_eq!(response.stick, Some([0.0, 0.0]));
}

#[test]
fn down_smash_duplicate_polls_reuse_one_press_and_release_when_animation_advances() {
    let mut state = common_roll_state(1.0);
    let roll = timed_roll(10.0, 30.0);
    let press = step(
        &mut state,
        0,
        LedgeContext {
            status: 0x3,
            ..roll_ready_context()
        },
        roll,
    );
    assert_eq!(press.requested, MAPPER_A);
    for tick in 1..5 {
        assert_eq!(
            step(
                &mut state,
                tick,
                LedgeContext {
                    status: 0x3,
                    ..roll_ready_context()
                },
                roll
            ),
            press
        );
        assert_eq!(state.attempts, 1);
    }
    let advanced = LedgeContext {
        motion_frame: 1.0,
        ..LedgeContext {
            status: 0x3,
            ..roll_ready_context()
        }
    };
    let release = step(&mut state, 5, advanced, roll);
    assert_eq!(release.requested, 0);
    assert_eq!(release.stick, Some([0.0, 0.0]));
}

#[test]
fn same_tick_damage_and_invalid_motion_do_not_reuse_a_cached_attack() {
    let mut state = common_roll_state(1.0);
    let roll = timed_roll(10.0, 30.0);
    assert_eq!(
        step(
            &mut state,
            0,
            LedgeContext {
                status: 0x3,
                ..roll_ready_context()
            },
            roll
        )
        .requested,
        MAPPER_A
    );
    let damaged = LedgeContext {
        status: 0x49,
        can_act: false,
        can_control: false,
        ..LedgeContext {
            status: 0x3,
            ..roll_ready_context()
        }
    };
    assert_eq!(step(&mut state, 0, damaged, roll).owned, 0);
    let bad = Opponent {
        velocity: [f32::NAN, 0.0],
        ..roll
    };
    assert_eq!(
        step(
            &mut state,
            0,
            LedgeContext {
                status: 0x3,
                ..roll_ready_context()
            },
            bad
        )
        .owned,
        0
    );
    assert_ne!(state.pending, Pending::DownSmash);
}

#[test]
fn losing_target_discards_pending_attack_and_cannot_confirm_an_unrelated_later_hit() {
    let mut state = LedgeTrap::default();
    let climb = enemy(CLIFF_CLIMB, true);
    step(&mut state, 0, context(), climb);
    step(&mut state, 1, context(), climb);
    assert_eq!(state.pending, Pending::DownTilt);
    state.sample(settings(), 2, context(), None);
    assert_eq!(state.pending, Pending::None);
    state.sample(
        settings(),
        3,
        LedgeContext {
            status: DOWN_TILT,
            hit_confirmed: true,
            ..context()
        },
        None,
    );
    assert!(!state.finished);
}

#[test]
fn own_native_clock_alone_deduplicates_inputs_when_opponent_motion_is_unknown() {
    let mut state = LedgeTrap::default();
    let ctx = LedgeContext {
        facing: -1.0,
        motion_kind: 1,
        ..context()
    };
    let jump = Opponent {
        situation: 2,
        ..enemy(CLIFF_JUMP1, true)
    };
    assert_eq!(
        step(&mut state, 0, ctx, jump).requested,
        (MAPPER_X | MAPPER_A)
    );
    for tick in 1..20 {
        assert_eq!(
            step(&mut state, tick, ctx, jump).requested,
            (MAPPER_X | MAPPER_A)
        );
    }
    assert_eq!(state.jump_attempts, 1);
    assert_eq!(
        step(
            &mut state,
            20,
            LedgeContext {
                motion_frame: 1.0,
                ..ctx
            },
            jump
        )
        .requested,
        0
    );
}

#[test]
fn damage_aborts_the_owned_hop_instead_of_resuming_it_after_hitstun() {
    let mut state = LedgeTrap::default();
    let ctx = LedgeContext {
        facing: -1.0,
        ..context()
    };
    let jump = Opponent {
        situation: 2,
        ..enemy(CLIFF_JUMP1, true)
    };
    step(&mut state, 0, ctx, jump);
    assert!(state.owns_jump_state(0xb));
    step(
        &mut state,
        1,
        LedgeContext {
            status: 0x49,
            situation: 2,
            can_control: false,
            can_act: false,
            ..ctx
        },
        jump,
    );
    assert!(!state.owns_jump_state(0xe));
}

#[test]
fn committed_nair_does_not_recheck_kirbys_travel_after_the_launch() {
    let mut state = LedgeTrap::default();
    let ctx = LedgeContext {
        facing: -1.0,
        ..context()
    };
    let jump = Opponent {
        situation: 2,
        position: [5.0, 10.0],
        ..enemy(CLIFF_JUMP1, true)
    };
    assert_eq!(
        step(&mut state, 0, ctx, jump).requested,
        (MAPPER_X | MAPPER_A)
    );
    let rising = LedgeContext {
        status: 0xb,
        situation: 2,
        can_act: false,
        velocity: [0.0, 3.0],
        ..ctx
    };
    assert_eq!(step(&mut state, 1, rising, jump).requested, MAPPER_A);
}

fn manual_settings() -> LedgeSettings {
    LedgeSettings {
        manual_spacing: true,
        // Manual mode overrides even an old config's explicit navigation flag.
        auto_position: true,
        ..settings()
    }
}

#[test]
fn manual_tilts_start_from_walk_without_preparation_and_release_after_pulse() {
    for attack in [GetupAttack::ForwardTilt, GetupAttack::DownTilt] {
        let config = LedgeSettings {
            getup_attack: attack,
            ..manual_settings()
        };
        let ctx = LedgeContext {
            status: 1,
            velocity: [0.1, 0.0],
            ..context()
        };
        let target = enemy(CLIFF_CLIMB, true);
        let mut state = LedgeTrap::default();
        assert_eq!(
            state.sample(config, 0, ctx, Some(target)).requested,
            MAPPER_A
        );
        // No accepted animation yet: release A but return horizontal movement.
        let out = state.sample(
            config,
            1,
            LedgeContext {
                motion_frame: 1.0,
                ..ctx
            },
            Some(target),
        );
        assert_eq!(out.requested, 0);
        assert_eq!(out.stick, None);
        assert_eq!(out.owned, LEDGE_BUTTONS);
        let blocked = LedgeContext {
            can_tilt: false,
            ..ctx
        };
        assert_eq!(
            LedgeTrap::default().sample(config, 0, blocked, Some(target)),
            LedgeOutput {
                consume_trigger: true,
                ..LedgeOutput::default()
            }
        );
    }
}

#[test]
fn manual_jump_requires_native_readiness_and_no_observed_active_hitbox() {
    let target = Opponent {
        situation: 2,
        position: [-5.0, 5.0],
        ..enemy(CLIFF_JUMP1, true)
    };
    assert_eq!(
        LedgeTrap::default()
            .sample(manual_settings(), 0, context(), Some(target))
            .requested,
        (MAPPER_X | MAPPER_A)
    );
    for (ctx, target) in [
        (
            LedgeContext {
                can_jump: false,
                ..context()
            },
            target,
        ),
        (
            context(),
            Opponent {
                attack_active: Some(true),
                ..target
            },
        ),
    ] {
        assert_eq!(
            LedgeTrap::default().sample(manual_settings(), 0, ctx, Some(target)),
            LedgeOutput {
                consume_trigger: true,
                ..LedgeOutput::default()
            }
        );
    }
}

#[test]
fn neutral_air_hops_without_turning_and_attacks_neutrally_on_either_side() {
    for side in [-1.0, 1.0] {
        for facing in [-1.0, 1.0] {
            let mut state = LedgeTrap::default();
            let ctx = LedgeContext {
                facing,
                ..context()
            };
            let target = Opponent {
                position: [side * 5.0, 6.0],
                situation: 2,
                ..enemy(CLIFF_JUMP1, true)
            };
            let hop = state.sample(manual_settings(), 0, ctx, Some(target));
            assert_eq!(hop.requested, (MAPPER_X | MAPPER_A));
            assert_eq!(hop.stick, Some([0.0, 0.0]));
            let squat = LedgeContext {
                status: JUMP_SQUAT,
                can_act: false,
                ..ctx
            };
            assert_eq!(
                state
                    .sample(manual_settings(), 1, squat, Some(target))
                    .requested,
                MAPPER_A
            );
            let air = LedgeContext {
                status: 0xb,
                situation: 2,
                can_act: false,
                ..ctx
            };
            let attack = state.sample(manual_settings(), 2, air, Some(target));
            assert_eq!(attack.requested, MAPPER_A);
            assert_eq!(attack.stick, Some([0.0, 0.0]));
            let actual = LedgeContext {
                status: ATTACK_AIR,
                motion_kind: NEUTRAL_AIR_MOTION,
                hit_confirmed: true,
                ..air
            };
            assert_eq!(
                state
                    .sample(manual_settings(), 3, actual, Some(target))
                    .requested,
                0
            );
            assert!(state.finished);
        }
    }
}

#[test]
fn neutral_air_requires_its_exact_native_motion_and_a_hit_to_finish() {
    for (motion, hit, finished) in [
        (0x0c33f869bc, true, false),
        (NEUTRAL_AIR_MOTION, false, false),
        (NEUTRAL_AIR_MOTION, true, true),
    ] {
        let mut state = LedgeTrap::default();
        let target = Opponent {
            position: [5.0, 6.0],
            situation: 2,
            ..enemy(CLIFF_JUMP1, true)
        };
        step(&mut state, 0, context(), target);
        let air = LedgeContext {
            status: 0xb,
            situation: 2,
            can_act: false,
            ..context()
        };
        assert_eq!(step(&mut state, 1, air, target).requested, MAPPER_A);
        step(
            &mut state,
            2,
            LedgeContext {
                status: ATTACK_AIR,
                motion_kind: motion,
                hit_confirmed: hit,
                ..air
            },
            target,
        );
        assert_eq!(state.finished, finished);
    }
}

#[test]
fn neutral_air_lingering_window_requires_protection_and_range_at_contact() {
    for (expiry, x, expected) in [
        (Some(35.0), 5.0, (MAPPER_X | MAPPER_A)),
        (Some(36.0), 5.0, 0),
        (None, 5.0, 0),
        (Some(11.0), 30.0, 0),
    ] {
        let target = Opponent {
            position: [x, 6.0],
            situation: 2,
            vulnerable_in: expiry,
            ..enemy(CLIFF_JUMP1, false)
        };
        assert_eq!(
            LedgeTrap::default()
                .sample(manual_settings(), 0, context(), Some(target))
                .requested,
            expected
        );
    }
}

#[test]
fn dash_staging_releases_immediately_on_damage_target_loss_and_y_release() {
    for (ctx, target) in [
        (
            LedgeContext {
                held: false,
                ..context()
            },
            Some(timed_roll(12.0, 30.0)),
        ),
        (
            LedgeContext {
                status: 0x47,
                can_control: false,
                can_act: false,
                ..context()
            },
            Some(timed_roll(12.0, 30.0)),
        ),
        (context(), None),
    ] {
        let mut state = LedgeTrap::default();
        state.sample(
            manual_settings(),
            0,
            context(),
            Some(timed_roll(12.0, 30.0)),
        );
        let out = state.sample(manual_settings(), 1, ctx, target);
        assert_eq!(out.requested, 0);
        assert_eq!(out.owned, 0);
        assert_eq!(out.stick, None);
    }
}

#[test]
fn manual_drop_fallback_holds_shield_through_stop_but_known_roll_and_jump_override_it() {
    let drop = Opponent {
        situation: 2,
        previous_status: CLIFF_WAIT,
        position: [5.0, -3.0],
        ..enemy(0xe, true)
    };
    for option in [CLIFF_ESCAPE, CLIFF_JUMP1, CLIFF_CLIMB] {
        let mut state = LedgeTrap::default();
        assert_eq!(
            state
                .sample(manual_settings(), 0, context(), Some(drop))
                .requested,
            MAPPER_ZR
        );
        let guard = LedgeContext {
            status: GUARD,
            can_act: false,
            ..context()
        };
        assert_eq!(
            state
                .sample(
                    manual_settings(),
                    1,
                    LedgeContext {
                        stopped: true,
                        ..guard
                    },
                    Some(drop)
                )
                .requested,
            MAPPER_ZR
        );
        let target = Opponent {
            previous_status: CLIFF_WAIT,
            position: [5.0, 0.0],
            situation: if option == CLIFF_JUMP1 { 2 } else { 0 },
            status: option,
            ..timed_roll(20.0, 40.0)
        };
        assert_eq!(
            state
                .sample(manual_settings(), 2, guard, Some(target))
                .requested
                & MAPPER_ZR,
            0
        );
        assert!(!matches!(state.plan, OptionPlan::DropShield));
        assert_eq!(
            state
                .sample(
                    manual_settings(),
                    3,
                    LedgeContext {
                        held: false,
                        ..guard
                    },
                    Some(target)
                )
                .owned,
            0
        );
    }
}

#[test]
fn directional_tilts_turn_to_target_then_attack_after_native_facing_confirmation() {
    for attack in [GetupAttack::DownTilt, GetupAttack::ForwardTilt] {
        for side in [-1.0, 1.0] {
            let config = LedgeSettings {
                getup_attack: attack,
                ..manual_settings()
            };
            let mut state = LedgeTrap::default();
            let target = Opponent {
                position: [side * 5.0, 0.0],
                ..enemy(CLIFF_CLIMB, true)
            };
            let ctx = LedgeContext {
                facing: -side,
                ..context()
            };
            let turn = state.sample(config, 0, ctx, Some(target));
            assert_eq!(turn.requested, 0);
            assert_eq!(turn.stick, Some([side * 0.3, 0.0]));
            let input = state.sample(
                config,
                1,
                LedgeContext {
                    facing: side,
                    ..ctx
                },
                Some(target),
            );
            assert_eq!(input.requested, MAPPER_A);
        }
    }
}

#[test]
fn roll_dash_range_key_accepts_old_config_alias() {
    use crate::live_input::LiveConfig;
    assert_eq!(
        LiveConfig::parse("ledge_roll_dash_range = 9.5")
            .unwrap()
            .ledge
            .roll_dash_range,
        9.5
    );
    assert_eq!(
        LiveConfig::parse("ledge_roll_smash_range = 9.5")
            .unwrap()
            .ledge
            .roll_dash_range,
        9.5
    );
}

#[test]
fn native_turn_failures_do_not_take_movement_forever() {
    for target in [Opponent {
        position: [-5.0, 0.0],
        ..enemy(CLIFF_CLIMB, true)
    }] {
        let mut state = LedgeTrap::default();
        for tick in 0..=i64::from(MAX_ATTEMPTS) {
            let ctx = LedgeContext {
                motion_kind: 10,
                motion_frame: tick as f32,
                ..context()
            };
            let out = state.sample(manual_settings(), tick, ctx, Some(target));
            assert_eq!(out.requested, 0);
            if tick == i64::from(MAX_ATTEMPTS) {
                assert_eq!(out.stick, None);
                assert_eq!(out.owned, 0);
            }
        }
    }
}

#[test]
fn crossed_up_getup_attack_releases_shield_turns_and_grabs_with_confirmed_facing() {
    let mut state = LedgeTrap::default();
    let target = Opponent {
        position: [-5.0, 0.0],
        attack_active: Some(true),
        ..enemy(CLIFF_ATTACK, true)
    };
    state.sample(manual_settings(), 0, context(), Some(target));
    // A real block establishes permission to punish. Hold shield through stun.
    let stunned = LedgeContext {
        status: GUARD_DAMAGE,
        can_act: false,
        shield_attacker: Some(2),
        shield_stun_frames: Some(8),
        ..context()
    };
    state.sample(manual_settings(), 1, stunned, Some(target));
    let ended = Opponent {
        attack_active: Some(false),
        ..target
    };
    let guard = LedgeContext {
        status: GUARD,
        can_act: false,
        ..context()
    };
    assert_eq!(
        state
            .sample(manual_settings(), 2, guard, Some(ended))
            .requested,
        0
    );
    let turn = state.sample(manual_settings(), 3, context(), Some(ended));
    assert_eq!(turn.stick, Some([-0.3, 0.0]));
    let grab = state.sample(
        manual_settings(),
        4,
        LedgeContext {
            facing: -1.0,
            ..context()
        },
        Some(ended),
    );
    assert_eq!(grab.requested, MAPPER_R);
}

#[test]
fn rising_jump_is_intercepted_before_landing_with_native_gravity_and_own_hop_rise() {
    let physics = AirPhysics::new(0.1, 1.5);
    let mut state = LedgeTrap::default();
    let ground = LedgeContext {
        air_physics: physics,
        short_hop_speed: Some(2.5),
        ..context()
    };
    let launch = Opponent {
        position: [5.0, 6.0],
        velocity: [0.0, 3.0],
        situation: 2,
        vulnerable_in: Some(12.0),
        ..enemy(CLIFF_JUMP1, false)
    };
    assert_eq!(
        state
            .sample(manual_settings(), 0, ground, Some(launch))
            .requested,
        (MAPPER_X | MAPPER_A)
    );
    let rising = Opponent {
        status: CLIFF_JUMP3,
        position: [5.0, 15.0],
        velocity: [0.0, 2.7],
        vulnerable_in: Some(8.0),
        actionable_in: Some(16.0),
        air_physics: physics,
        ..launch
    };
    let own_air = LedgeContext {
        status: 0xb,
        situation: 2,
        can_act: false,
        position: [0.0, 2.5],
        velocity: [0.0, 2.4],
        ..ground
    };
    let attack = state.sample(manual_settings(), 1, own_air, Some(rising));
    assert_eq!(attack.requested, MAPPER_A);
    assert_eq!(attack.stick, Some([0.0, 0.0]));
}

#[test]
fn committed_jump_punish_finishes_even_if_enemy_recovers_or_lands() {
    let mut state = LedgeTrap::default();
    let launch = Opponent {
        position: [5.0, 6.0],
        situation: 2,
        ..enemy(CLIFF_JUMP1, true)
    };
    assert_eq!(
        state
            .sample(manual_settings(), 0, context(), Some(launch))
            .requested,
        (MAPPER_X | MAPPER_A)
    );
    let own_air = LedgeContext {
        status: 0xb,
        situation: 2,
        can_act: false,
        ..context()
    };
    let expired = Opponent {
        status: CLIFF_JUMP3,
        actionable_in: Some(8.0),
        ..launch
    };
    assert_eq!(
        state
            .sample(manual_settings(), 1, own_air, Some(expired))
            .requested,
        MAPPER_A
    );
    let landed = Opponent {
        status: 0,
        situation: 0,
        position: [5.0, 0.0],
        ..launch
    };
    let out = state.sample(manual_settings(), 2, own_air, Some(landed));
    assert_eq!(out.requested, MAPPER_A);
    assert_eq!(out.owned, LEDGE_BUTTONS);
    assert!(state.owns_jump_state(0xb));
    let own_landed = LedgeContext {
        status: 0x16,
        situation: 0,
        ..context()
    };
    assert_eq!(
        state
            .sample(manual_settings(), 3, own_landed, Some(landed))
            .requested,
        0
    );
    assert!(!state.owns_jump_state(0xb));
}

#[test]
fn hop_preparation_does_not_require_the_launch_pose_to_extrapolate_into_attack_range() {
    let mut state = LedgeTrap::default();
    let ctx = LedgeContext {
        air_physics: AirPhysics::new(0.08, 1.2),
        short_hop_speed: None,
        ..context()
    };
    let target = Opponent {
        position: [5.0, -4.0],
        velocity: [0.0, 4.0],
        situation: 2,
        ..enemy(CLIFF_JUMP1, false)
    };
    assert_eq!(
        state
            .sample(manual_settings(), 0, ctx, Some(target))
            .requested,
        (MAPPER_X | MAPPER_A)
    );
    let air = LedgeContext {
        status: 0xb,
        situation: 2,
        can_act: false,
        velocity: [0.0, 2.0],
        ..ctx
    };
    assert_eq!(
        state
            .sample(manual_settings(), 1, air, Some(target))
            .requested,
        0,
        "preparation does not assume protection already ended"
    );
}

#[test]
fn native_cancel_deadline_wins_over_longer_animation_end() {
    assert_eq!(recovery_frames(10.0, 50.0, 30.0, 2.0), Some(10.0));
    assert_eq!(recovery_frames(30.0, 50.0, 30.0, 1.0), Some(0.0));
    assert_eq!(recovery_frames(35.0, 50.0, 30.0, 1.0), Some(0.0));
    assert_eq!(recovery_frames(10.0, 50.0, 0.0, 2.0), Some(20.0));
    assert_eq!(recovery_frames(10.0, f32::NAN, 30.0, 1.0), None);
    assert_eq!(recovery_frames(10.0, 50.0, 30.0, 0.0), None);
}

#[test]
fn native_gravity_is_validated_and_terminal_fall_speed_is_respected() {
    for (gravity, terminal) in [
        (0.0, 1.5),
        (f32::NAN, 1.5),
        (0.1, -1.0),
        (0.1, f32::INFINITY),
    ] {
        assert!(AirPhysics::new(gravity, terminal).is_none());
    }
    let physics = AirPhysics::new(0.1, 1.5).unwrap();
    assert!((physics.displacement(-1.5, 10.0) + 15.0).abs() < 0.001);
    assert!(physics.displacement(2.5, 30.0) < 2.5 * 30.0);
}

#[test]
fn shield_grab_is_timed_before_known_getup_attack_recovery_expires() {
    for (deadline, should_grab) in [(10.0, false), (11.0, true)] {
        let mut state = LedgeTrap::default();
        let target = Opponent {
            attack_active: Some(true),
            actionable_in: Some(30.0),
            ..enemy(CLIFF_ATTACK, true)
        };
        state.sample(manual_settings(), 0, context(), Some(target));
        let stunned = LedgeContext {
            status: GUARD_DAMAGE,
            shield_attacker: Some(2),
            shield_stun_frames: Some(8),
            can_act: false,
            ..context()
        };
        state.sample(manual_settings(), 1, stunned, Some(target));
        let guard = LedgeContext {
            status: GUARD,
            can_act: false,
            ..context()
        };
        let out = state.sample(
            manual_settings(),
            2,
            guard,
            Some(Opponent {
                actionable_in: Some(deadline),
                ..target
            }),
        );
        assert_eq!(out.requested & MAPPER_R != 0, should_grab);
        assert_eq!(out.requested & MAPPER_ZR, MAPPER_ZR);
    }
}

#[test]
fn neutral_air_forecast_does_not_keep_its_hitbox_after_predicted_landing() {
    let physics = AirPhysics::new(0.1, 1.5);
    let ctx = LedgeContext {
        status: 0xe,
        situation: 2,
        position: [0.0, 2.0],
        velocity: [0.0, -1.5],
        air_physics: physics,
        ..context()
    };
    let target = Opponent {
        position: [5.0, 0.0],
        situation: 2,
        ..enemy(CLIFF_JUMP3, true)
    };
    let window = AttackWindow {
        first: NEUTRAL_AIR_FIRST,
        last: NEUTRAL_AIR_LAST,
        range: 12.0,
        height: [-6.0, 18.0],
        front_only: false,
    };
    assert!(!jump_contact(window, ctx, target, false, Some(0.0)));
}

#[test]
fn normal_getup_keeps_the_existing_tilt_but_respects_an_earlier_cancel_deadline() {
    let config = LedgeSettings {
        getup_attack: GetupAttack::ForwardTilt,
        ..manual_settings()
    };
    for (deadline, expected) in [(5.0, 0), (6.0, MAPPER_A)] {
        let target = Opponent {
            actionable_in: Some(deadline),
            ..enemy(CLIFF_CLIMB, true)
        };
        assert_eq!(
            LedgeTrap::default()
                .sample(config, 0, context(), Some(target))
                .requested,
            expected
        );
    }
}

fn timing_only_settings() -> LedgeSettings {
    LedgeSettings {
        distance_agnostic: true,
        getup_attack: GetupAttack::ForwardTilt,
        ..manual_settings()
    }
}

#[test]
fn timing_only_normal_getup_ignores_distance_height_and_acquisition_box() {
    for position in [[5.0, 0.0], [500.0, 200.0]] {
        let mut state = LedgeTrap::default();
        let target = Opponent {
            position,
            vulnerable_in: Some(5.0),
            actionable_in: Some(10.0),
            ..enemy(CLIFF_CLIMB, false)
        };
        let out = state.sample(timing_only_settings(), 0, context(), Some(target));
        assert_eq!(out.requested, MAPPER_A);
        assert_eq!(out.stick, Some([0.6, -0.3]));
    }
}

#[test]
fn timing_only_jump_hops_and_requests_nair_at_any_distance() {
    for position in [[5.0, 6.0], [500.0, 200.0]] {
        let mut state = LedgeTrap::default();
        let target = Opponent {
            situation: 2,
            position,
            vulnerable_in: Some(12.0),
            actionable_in: Some(20.0),
            ..enemy(CLIFF_JUMP1, false)
        };
        assert_eq!(
            state
                .sample(timing_only_settings(), 0, context(), Some(target))
                .requested,
            (MAPPER_X | MAPPER_A)
        );
        let air = LedgeContext {
            status: 0xb,
            situation: 2,
            can_act: false,
            ..context()
        };
        let later = Opponent {
            vulnerable_in: Some(8.0),
            actionable_in: Some(12.0),
            ..target
        };
        assert_eq!(
            state
                .sample(timing_only_settings(), 1, air, Some(later))
                .requested,
            MAPPER_A
        );
    }
}

#[test]
fn distance_independent_timing_still_obeys_unknown_protection_readiness_and_recovery() {
    let target = Opponent {
        position: [500.0, 200.0],
        vulnerable_in: Some(5.0),
        actionable_in: Some(10.0),
        ..enemy(CLIFF_CLIMB, false)
    };
    for (ctx, target) in [
        (
            LedgeContext {
                can_tilt: false,
                ..context()
            },
            target,
        ),
        (
            context(),
            Opponent {
                actionable_in: Some(5.0),
                ..target
            },
        ),
        (
            context(),
            Opponent {
                vulnerable_in: None,
                ..target
            },
        ),
        (
            LedgeContext {
                held: false,
                ..context()
            },
            target,
        ),
    ] {
        assert_eq!(
            LedgeTrap::default()
                .sample(timing_only_settings(), 0, ctx, Some(target))
                .requested,
            0
        );
    }
}

#[test]
fn confirmed_getup_attack_block_buffers_grab_without_distance_protection_or_recovery_gates() {
    let mut state = LedgeTrap::default();
    let target = Opponent {
        position: [500.0, 100.0],
        situation: 3,
        actionable_in: Some(0.0),
        attack_active: Some(true),
        ..enemy(CLIFF_ATTACK, false)
    };
    assert_eq!(
        state
            .sample(timing_only_settings(), 0, context(), Some(target))
            .requested,
        MAPPER_ZR
    );
    let stun = LedgeContext {
        status: GUARD_DAMAGE,
        can_act: false,
        can_grab: false,
        shield_attacker: Some(2),
        shield_stun_frames: Some(8),
        ..context()
    };
    assert_eq!(
        state
            .sample(timing_only_settings(), 1, stun, Some(target))
            .requested,
        MAPPER_ZR
    );
    let buffer = LedgeContext {
        shield_stun_frames: Some(2),
        ..stun
    };
    assert_eq!(
        state
            .sample(timing_only_settings(), 2, buffer, Some(target))
            .requested,
        MAPPER_ZR | MAPPER_R
    );
    assert_eq!(
        state
            .sample(
                timing_only_settings(),
                3,
                LedgeContext {
                    motion_frame: 1.0,
                    ..buffer
                },
                Some(target)
            )
            .requested,
        MAPPER_ZR
    );
}

#[test]
fn distance_independent_policy_is_default_and_can_be_disabled_in_config() {
    use crate::live_input::LiveConfig;
    assert!(LiveConfig::parse("").unwrap().ledge.distance_agnostic);
    assert!(
        !LiveConfig::parse("ledge_distance_agnostic = false")
            .unwrap()
            .ledge
            .distance_agnostic
    );
    assert!(LiveConfig::parse("ledge_distance_agnostic = maybe").is_err());
}

#[test]
fn rear_getup_attack_block_jumps_directly_out_of_shield_then_nairs_without_turning() {
    for side in [-1.0, 1.0] {
        let mut state = LedgeTrap::default();
        let config = timing_only_settings();
        let ctx = LedgeContext {
            facing: -side,
            ..context()
        };
        let target = Opponent {
            position: [side * 50.0, 0.0],
            attack_active: Some(true),
            ..enemy(CLIFF_ATTACK, true)
        };
        state.sample(config, 0, ctx, Some(target));
        let stun = LedgeContext {
            status: GUARD_DAMAGE,
            can_act: false,
            shield_attacker: Some(2),
            shield_stun_frames: Some(1),
            ..ctx
        };
        assert_eq!(
            state.sample(config, 1, stun, Some(target)).requested,
            MAPPER_ZR
        );
        let guard = LedgeContext {
            status: GUARD,
            can_act: false,
            can_jump: true,
            ..ctx
        };
        let hop = state.sample(config, 2, guard, Some(target));
        assert_eq!(hop.requested, MAPPER_X);
        assert_eq!(hop.stick, Some([0.0, 0.0]));
        assert!(state.owns_jump_state(JUMP_SQUAT));
        let squat = LedgeContext {
            status: JUMP_SQUAT,
            can_act: false,
            ..ctx
        };
        assert_eq!(state.sample(config, 3, squat, Some(target)).requested, 0);
        let air = LedgeContext {
            status: 0xb,
            situation: 2,
            can_act: false,
            can_air_attack: true,
            ..ctx
        };
        let attack = state.sample(config, 4, air, Some(target));
        assert_eq!(attack.requested, MAPPER_A);
        assert_eq!(attack.stick, Some([0.0, 0.0]));
        assert_eq!(state.pending, Pending::NeutralAir);
        let actual = LedgeContext {
            status: ATTACK_AIR,
            motion_kind: NEUTRAL_AIR_MOTION,
            ..air
        };
        assert_eq!(state.sample(config, 5, actual, Some(target)).requested, 0);
        let landed = state.sample(config, 6, ctx, Some(target));
        assert_eq!(landed.requested, 0);
        assert_eq!(landed.owned, 0);
    }
}

#[test]
fn rear_block_nair_sequence_preserves_hitstop_shield_and_cancels_on_y_release() {
    let mut state = LedgeTrap::default();
    let config = timing_only_settings();
    let target = Opponent {
        position: [-5.0, 0.0],
        attack_active: Some(true),
        ..enemy(CLIFF_ATTACK, true)
    };
    state.sample(config, 0, context(), Some(target));
    let stun = LedgeContext {
        status: GUARD_DAMAGE,
        can_act: false,
        shield_attacker: Some(2),
        shield_stun_frames: Some(1),
        ..context()
    };
    state.sample(config, 1, stun, Some(target));
    assert_eq!(
        state
            .sample(
                config,
                2,
                LedgeContext {
                    stopped: true,
                    ..stun
                },
                Some(target)
            )
            .requested,
        MAPPER_ZR
    );
    let out = state.sample(
        config,
        3,
        LedgeContext {
            held: false,
            ..context()
        },
        Some(target),
    );
    assert_eq!(out.owned, 0);
    assert_eq!(out.requested, 0);
    assert_eq!(out.stick, None);
}

#[test]
fn queued_nair_stays_held_until_native_acceptance_or_hop_cancellation() {
    use crate::native_input::{ButtonTrace, MapperButtonOverlay};

    for option in [CLIFF_CLIMB, CLIFF_JUMP1, CLIFF_ATTACK] {
        for end in [ATTACK_AIR, 0, 0x47, -1] {
            let mut trap = LedgeTrap::default();
            let config = LedgeSettings {
                getup_attack: GetupAttack::NeutralAir,
                ..timing_only_settings()
            };
            let target = Opponent {
                position: [-5.0, 0.0],
                situation: if option == CLIFF_JUMP1 { 2 } else { 0 },
                vulnerable_in: Some(14.0),
                actionable_in: Some(30.0),
                ..enemy(option, false)
            };
            let mut ctx = context();
            if option == CLIFF_ATTACK {
                trap.sample(config, 0, ctx, Some(target));
                ctx.status = GUARD_DAMAGE;
                ctx.shield_attacker = Some(target.id);
                trap.sample(config, 1, ctx, Some(target));
                ctx.status = GUARD;
            }
            assert_eq!(
                trap.sample(config, 2, ctx, Some(target)).requested,
                if option == CLIFF_ATTACK {
                    MAPPER_X
                } else {
                    MAPPER_X | MAPPER_A
                }
            );
            let air = LedgeContext {
                status: 0xb,
                situation: 2,
                can_act: false,
                motion_kind: 1,
                can_air_attack: false,
                ..context()
            };
            assert_eq!(
                trap.sample(config, 3, air, Some(target)).requested,
                MAPPER_A
            );
            let mut overlay = MapperButtonOverlay::default();
            // Delayed acceptance must not exhaust six pulses. Changing native
            // frames bypasses duplicate-sample caching, as real animation does.
            for frame in 0..20 {
                let out = trap.sample(
                    config,
                    4 + frame,
                    LedgeContext {
                        motion_frame: frame as f32,
                        status: if frame < 10 { 0xb } else { 0xe },
                        can_air_attack: frame == 0,
                        ..air
                    },
                    Some(target),
                );
                assert_eq!(out.requested, MAPPER_A, "option={option:#x}, frame={frame}");
                assert_eq!(out.stick, Some([0.0, 0.0]));
                let mapped = overlay.apply_raw(ButtonTrace::default(), out.requested, out.owned);
                assert_eq!(mapped.held & MAPPER_A, MAPPER_A);
                assert_eq!(
                    mapped.pressed & MAPPER_A,
                    if frame == 0 { MAPPER_A } else { 0 }
                );
                assert_eq!(trap.attempts, 1);
            }
            let terminal = LedgeContext {
                held: end != -1,
                status: end,
                situation: if end == 0 { 0 } else { 2 },
                can_control: end != 0x47,
                motion_kind: if end == ATTACK_AIR {
                    NEUTRAL_AIR_MOTION
                } else {
                    1
                },
                motion_frame: 0.0,
                ..air
            };
            let out = trap.sample(config, 24, terminal, Some(target));
            assert_eq!(
                out.requested & MAPPER_A,
                0,
                "option={option:#x}, end={end:#x}"
            );
            let mapped = overlay.apply_raw(ButtonTrace::default(), out.requested, out.owned);
            assert_eq!(mapped.held & MAPPER_A, 0);
            assert_eq!(mapped.released & MAPPER_A, MAPPER_A);
            assert!(
                !trap.finished,
                "a requested/accepted nair alone is not a hit"
            );
            if end == 0 && option != CLIFF_JUMP1 {
                assert_eq!(
                    trap.sample(config, 25, context(), Some(target)).requested,
                    0
                );
                let manual_hop = LedgeContext {
                    can_air_attack: true,
                    ..air
                };
                assert_eq!(
                    trap.sample(config, 26, manual_hop, Some(target)).requested,
                    0
                );
            }
        }
    }
}

#[test]
fn committed_online_hop_completes_nair_after_enemy_becomes_actionable() {
    let mut trap = LedgeTrap::default();
    let launch = Opponent {
        situation: 2,
        vulnerable_in: Some(15.0),
        ..enemy(CLIFF_JUMP1, false)
    };
    assert_eq!(
        trap.sample(timing_only_settings(), 0, context(), Some(launch))
            .requested,
        (MAPPER_X | MAPPER_A)
    );
    let airborne = LedgeContext {
        status: 0xb,
        situation: 2,
        can_act: false,
        motion_frame: 3.0,
        ..context()
    };
    let target = Opponent {
        status: 0xe,
        previous_status: CLIFF_JUMP3,
        actionable_in: Some(0.0),
        ..launch
    };
    let attack = trap.sample(timing_only_settings(), 1, airborne, Some(target));
    assert_eq!(attack.requested, MAPPER_A);
    assert_eq!(attack.stick, Some([0.0, 0.0]));
}

#[test]
fn roll_moves_to_common_spot_then_releases_horizontal_and_times_down_smash() {
    for side in [-1.0, 1.0] {
        let config = timing_only_settings();
        let mut trap = common_roll_state(side);
        let roll = Opponent {
            position: [side * 2.0, 0.0],
            velocity: [0.0; 2],
            ..timed_roll(25.0, 35.0)
        };
        let first = trap.sample(config, 0, context(), Some(roll));
        assert_eq!(first.stick, Some([side, 0.0]));
        assert_eq!(trap.roll_interception().unwrap()[1], side * 25.0);
        let arrived = LedgeContext {
            position: [side * 25.0, 0.0],
            ..context()
        };
        let wait = trap.sample(config, 1, arrived, Some(roll));
        assert_eq!(wait.requested, 0);
        assert_eq!(wait.stick, Some([0.0, 0.0]));
        let due = Opponent {
            vulnerable_in: Some(10.0),
            actionable_in: Some(20.0),
            ..roll
        };
        let attack = trap.sample(config, 2, arrived, Some(due));
        assert_eq!(attack.requested, MAPPER_A);
        assert_eq!(attack.stick, Some([0.0, -1.0]));
        let actual = LedgeContext {
            status: DOWN_SMASH,
            motion_frame: 7.0,
            ..arrived
        };
        assert_eq!(trap.sample(config, 3, actual, Some(due)).requested, 0);
        trap.sample(
            config,
            4,
            LedgeContext {
                hit_confirmed: true,
                ..actual
            },
            Some(due),
        );
        assert!(trap.finished);
    }
}

#[test]
fn learned_delay_requests_roll_smash_earlier_and_rechecks_live_recovery() {
    let roll = Opponent {
        velocity: [0.1, 0.0],
        actionable_in: Some(26.0),
        ..timed_roll(14.0, 26.0)
    };
    let config = timing_only_settings();
    let mut baseline = common_roll_state(1.0);
    assert_eq!(
        baseline
            .sample(config, 0, roll_ready_context(), Some(roll))
            .requested,
        0
    );
    let mut delayed = common_roll_state(1.0);
    let ctx = LedgeContext {
        input_delay: 5.0,
        ..roll_ready_context()
    };
    assert_eq!(
        delayed.sample(config, 0, ctx, Some(roll)).requested,
        MAPPER_A
    );
    let recovered = Opponent {
        actionable_in: Some(4.0),
        vulnerable_in: Some(0.0),
        vulnerable: true,
        ..roll
    };
    assert_eq!(
        delayed
            .sample(
                config,
                1,
                LedgeContext {
                    motion_frame: 1.0,
                    ..ctx
                },
                Some(recovered)
            )
            .requested,
        0
    );
}

#[test]
fn roll_down_smash_is_facing_independent_and_never_falls_back_to_grab() {
    for facing in [-1.0, 1.0] {
        let mut trap = common_roll_state(1.0);
        let roll = Opponent {
            vulnerable: true,
            actionable_in: Some(18.0),
            ..timed_roll(0.0, 18.0)
        };
        let ctx = LedgeContext {
            facing,
            safe_ground: [false; 2],
            ..roll_ready_context()
        };
        let out = trap.sample(timing_only_settings(), 0, ctx, Some(roll));
        assert_eq!(out.requested, MAPPER_A);
        assert_eq!(out.stick, Some([0.0, -1.0]));
    }
}

#[test]
fn roll_brakes_uncancellable_dash_before_requesting_down_smash() {
    let roll = Opponent {
        actionable_in: Some(30.0),
        ..timed_roll(10.0, 30.0)
    };
    let mut trap = common_roll_state(1.0);
    let dash = LedgeContext {
        status: 0x3,
        can_down_smash: false,
        ..roll_ready_context()
    };
    let brake = trap.sample(timing_only_settings(), 0, dash, Some(roll));
    assert_eq!(brake.requested, 0);
    assert_eq!(brake.stick, Some([0.0, 0.0]));
    assert_eq!(trap.action_name(), "roll_wait_smash_ready");
    let ready = trap.sample(timing_only_settings(), 1, roll_ready_context(), Some(roll));
    assert_eq!(ready.requested, MAPPER_A);
    assert_eq!(ready.stick, Some([0.0, -1.0]));
    let start = LedgeContext {
        status: 0x30,
        motion_frame: 1.0,
        can_down_smash: false,
        ..roll_ready_context()
    };
    let release = trap.sample(timing_only_settings(), 2, start, Some(roll));
    assert_eq!(
        release.requested, 0,
        "release A rather than charge the smash"
    );
    assert!(!trap.finished);
}

#[test]
fn normal_getup_nair_launches_eight_frames_before_its_first_hitbox_is_needed() {
    for delay in [0.0, 3.0, 6.0] {
        for facing in [-1.0, 1.0] {
            let mut trap = LedgeTrap::default();
            let config = LedgeSettings {
                enabled: true,
                ..LedgeSettings::default()
            };
            let ctx = LedgeContext {
                input_delay: delay,
                facing,
                can_tilt: false,
                ..context()
            };
            let launch_lead = 3.0 + 8.0 + delay + 8.0;
            // Relative to protection ending, target active frames -8..=16.
            // Up to seven frames of late execution still start before expiry.
            let first_active = delay + f32::from(JUMP_SQUAT_FRAMES + NEUTRAL_AIR_FIRST);
            let last_active = delay + f32::from(JUMP_SQUAT_FRAMES + NEUTRAL_AIR_LAST);
            assert_eq!(first_active - launch_lead, -8.0);
            assert_eq!(last_active - launch_lead, 16.0);
            for expiry in [32.0, 33.0, 34.0] {
                assert_eq!(expiry + first_active - launch_lead, expiry - 8.0);
            }
            for late_frames in [1.0, 2.0, 5.0, 7.0] {
                assert!(first_active + late_frames < launch_lead);
                assert!(last_active + late_frames >= launch_lead);
            }
            let target = Opponent {
                position: [100.0, 0.0],
                vulnerable_in: Some(launch_lead + 1.0),
                actionable_in: Some(launch_lead + 12.0),
                ..enemy(CLIFF_CLIMB, false)
            };
            assert_eq!(trap.sample(config, 0, ctx, Some(target)).requested, 0);
            let due = Opponent {
                vulnerable_in: Some(launch_lead),
                ..target
            };
            let jump = trap.sample(config, 1, ctx, Some(due));
            assert_eq!(jump.requested, (MAPPER_X | MAPPER_A));
            assert_eq!(jump.stick, Some([0.0, 0.0]));
            let squat = LedgeContext {
                status: JUMP_SQUAT,
                can_act: false,
                motion_frame: 1.0,
                ..ctx
            };
            assert_eq!(trap.sample(config, 2, squat, Some(due)).requested, MAPPER_A);
            // Protection metadata can change while an online jump is queued.
            // Complete the committed nair, even after the normal get-up ends.
            let recovered = Opponent {
                status: 0,
                previous_status: CLIFF_CLIMB,
                vulnerable_in: None,
                actionable_in: Some(0.0),
                ..due
            };
            let air = LedgeContext {
                status: 0xb,
                situation: 2,
                can_act: false,
                can_air_attack: true,
                ..ctx
            };
            let attack = trap.sample(config, 3, air, Some(recovered));
            assert_eq!(attack.requested, MAPPER_A);
            assert_eq!(attack.stick, Some([0.0, 0.0]));
            let accepted = LedgeContext {
                status: ATTACK_AIR,
                motion_kind: NEUTRAL_AIR_MOTION,
                motion_frame: 1.0,
                ..air
            };
            assert_eq!(
                trap.sample(config, 4, accepted, Some(recovered)).requested,
                0
            );
            assert!(!trap.finished);
            trap.sample(
                config,
                5,
                LedgeContext {
                    hit_confirmed: true,
                    motion_frame: 8.0,
                    ..accepted
                },
                Some(recovered),
            );
            assert!(trap.finished);
        }
    }
}

#[test]
fn normal_getup_nair_waits_for_timing_readiness_and_respects_release() {
    let config = LedgeSettings {
        enabled: true,
        ..LedgeSettings::default()
    };
    for target in [enemy(CLIFF_WAIT, true), enemy(CLIFF_CLIMB, false)] {
        assert_eq!(
            LedgeTrap::default()
                .sample(config, 0, context(), Some(target))
                .requested,
            0
        );
    }
    let due = Opponent {
        vulnerable_in: Some(14.0),
        actionable_in: Some(30.0),
        ..enemy(CLIFF_CLIMB, false)
    };
    let mut trap = LedgeTrap::default();
    assert_eq!(
        trap.sample(
            config,
            0,
            LedgeContext {
                can_jump: false,
                ..context()
            },
            Some(due)
        )
        .requested,
        0
    );
    assert_eq!(
        trap.sample(config, 1, context(), Some(due)).requested,
        (MAPPER_X | MAPPER_A)
    );
    assert!(trap.owns_jump_state(0xb));
    assert_eq!(
        trap.sample(
            config,
            2,
            LedgeContext {
                held: false,
                ..context()
            },
            Some(due)
        )
        .owned,
        0
    );
    assert!(!trap.owns_jump_state(0xb));
}

#[test]
fn normal_nair_retries_unaccepted_jump_without_target_rechecks_or_six_attempt_cutoff() {
    let config = LedgeSettings {
        getup_attack: GetupAttack::NeutralAir,
        ..timing_only_settings()
    };
    let due = Opponent {
        vulnerable_in: Some(14.0),
        actionable_in: Some(0.0),
        ..enemy(CLIFF_CLIMB, false)
    };
    let mut trap = LedgeTrap::default();
    let ctx = LedgeContext {
        motion_kind: 1,
        ..context()
    };
    assert_eq!(
        trap.sample(config, 0, ctx, Some(due)).requested,
        (MAPPER_X | MAPPER_A)
    );
    for frame in 1..=20 {
        let out = trap.sample(
            config,
            frame,
            LedgeContext {
                motion_frame: frame as f32,
                ..ctx
            },
            None,
        );
        assert_eq!(
            out.requested,
            if frame % 2 == 0 {
                (MAPPER_X | MAPPER_A)
            } else {
                0
            }
        );
        assert_eq!(out.owned, LEDGE_BUTTONS);
        assert_eq!(out.stick, Some([0.0, 0.0]));
    }
    assert!(trap.jump_attempts > MAX_ATTEMPTS);
    let squat = LedgeContext {
        status: JUMP_SQUAT,
        can_act: false,
        ..ctx
    };
    assert_eq!(trap.sample(config, 21, squat, None).requested, MAPPER_A);
    let air = LedgeContext {
        status: 0xb,
        situation: 2,
        can_act: false,
        can_air_attack: false,
        ..ctx
    };
    assert_eq!(trap.sample(config, 22, air, None).requested, MAPPER_A);
    // An opponent starting a different option cannot replace an owned nair.
    let different_option = enemy(CLIFF_ATTACK, true);
    assert_eq!(
        trap.sample(
            config,
            23,
            LedgeContext {
                motion_frame: 1.0,
                ..air
            },
            Some(different_option)
        )
        .requested,
        MAPPER_A
    );
    let stop = LedgeContext {
        stopped: true,
        can_control: false,
        ..air
    };
    let held = trap.sample(config, 24, stop, None);
    assert_eq!(held.requested, MAPPER_A);
    assert_eq!(held.owned, LEDGE_BUTTONS);
    let accepted = LedgeContext {
        status: ATTACK_AIR,
        motion_kind: NEUTRAL_AIR_MOTION,
        ..air
    };
    assert_eq!(trap.sample(config, 25, accepted, None).requested, 0);
    assert!(!trap.finished);
    assert_eq!(
        trap.sample(
            config,
            26,
            LedgeContext {
                hit_confirmed: true,
                motion_frame: 8.0,
                ..accepted
            },
            None
        )
        .requested,
        0
    );
    assert!(trap.finished);
}

#[test]
fn late_normal_getup_recognition_still_hops_without_recovery_deadline_veto() {
    let mut trap = LedgeTrap::default();
    let config = LedgeSettings {
        getup_attack: GetupAttack::NeutralAir,
        ..timing_only_settings()
    };
    let recovered = Opponent {
        status: 0,
        previous_status: CLIFF_CLIMB,
        vulnerable: true,
        vulnerable_in: None,
        actionable_in: Some(0.0),
        ..enemy(CLIFF_CLIMB, false)
    };
    assert_eq!(
        trap.sample(config, 0, context(), Some(recovered)).requested,
        (MAPPER_X | MAPPER_A)
    );
}

#[test]
fn committed_normal_nair_cancels_on_damage_or_y_release_even_without_target() {
    let config = LedgeSettings {
        getup_attack: GetupAttack::NeutralAir,
        ..timing_only_settings()
    };
    let due = Opponent {
        vulnerable_in: Some(14.0),
        ..enemy(CLIFF_CLIMB, false)
    };
    for terminal in [
        LedgeContext {
            status: 0x47,
            can_control: false,
            stopped: true,
            ..context()
        },
        LedgeContext {
            held: false,
            ..context()
        },
    ] {
        let mut trap = LedgeTrap::default();
        assert_eq!(
            trap.sample(config, 0, context(), Some(due)).requested,
            (MAPPER_X | MAPPER_A)
        );
        let out = trap.sample(config, 1, terminal, None);
        assert_eq!(out.requested, 0);
        assert_eq!(out.owned, 0);
        assert!(!trap.owns_jump_state(0xb));
    }
}

#[test]
fn roll_coverage_spot_stays_fixed_when_opponent_speed_position_or_timing_changes() {
    let config = timing_only_settings();
    let mut trap = common_roll_state(1.0);
    let roll = timed_roll(25.0, 35.0);
    trap.sample(config, 0, context(), Some(roll));
    let locked = trap.roll_interception().unwrap();
    for (tick, x, speed) in [(1, 8.0, 0.0), (2, 20.0, 2.0), (3, 40.0, -1.0)] {
        let changed = Opponent {
            position: [x, 0.0],
            velocity: [speed, 0.0],
            ..roll
        };
        trap.sample(
            config,
            tick,
            LedgeContext {
                status: 0x4,
                motion_frame: tick as f32,
                ..context()
            },
            Some(changed),
        );
        assert_eq!(trap.roll_interception().unwrap(), locked);
    }
    let arrived = roll_ready_context();
    assert_eq!(
        trap.sample(config, 4, arrived, Some(roll)).stick,
        Some([0.0, 0.0])
    );
    let recovered = Opponent {
        status: 0,
        previous_status: CLIFF_ESCAPE,
        ..roll
    };
    let out = trap.sample(config, 5, arrived, Some(recovered));
    assert_eq!(out.requested, 0);
    assert_eq!(out.owned, 0);
}

#[test]
fn roll_contact_window_opens_across_multiple_active_frames_and_closes_at_recovery() {
    for vulnerable_in in [7.0, 9.0, 11.0] {
        let mut trap = common_roll_state(1.0);
        let roll = Opponent {
            actionable_in: Some(20.0),
            ..timed_roll(vulnerable_in, 20.0)
        };
        assert_eq!(
            trap.sample(timing_only_settings(), 0, roll_ready_context(), Some(roll))
                .requested,
            MAPPER_A
        );
    }
    for (vulnerable_in, actionable_in) in [(12.0, 20.0), (0.0, 7.0), (0.0, 0.0)] {
        let mut trap = common_roll_state(1.0);
        let roll = Opponent {
            actionable_in: Some(actionable_in),
            ..timed_roll(vulnerable_in, 20.0)
        };
        assert_eq!(
            trap.sample(timing_only_settings(), 0, roll_ready_context(), Some(roll))
                .requested,
            0
        );
    }
}

#[test]
fn roll_starts_immediately_without_velocity_motion_or_protection_metadata() {
    for side in [-1.0, 1.0] {
        let mut trap = common_roll_state(side);
        let roll = Opponent {
            position: [side * 2.0, 0.0],
            velocity: [0.0; 2],
            motion: None,
            vulnerable_in: None,
            ..enemy(CLIFF_ESCAPE, false)
        };
        let out = trap.sample(timing_only_settings(), 0, context(), Some(roll));
        assert_eq!(out.stick, Some([side, 0.0]));
        assert_eq!(out.requested, 0);
        assert_eq!(trap.roll_interception().unwrap()[1], side * 25.0);
    }
}

#[test]
fn roll_open_window_does_not_cancel_movement_to_the_common_spot() {
    let config = timing_only_settings();
    let mut trap = common_roll_state(1.0);
    let roll = timed_roll(10.0, 25.0);
    let start = trap.sample(config, 0, context(), Some(roll));
    assert_eq!(start.stick, Some([1.0, 0.0]));
    assert_eq!(start.requested, 0);
    assert!(!trap.roll_positioned);
    let out = trap.sample(config, 1, roll_ready_context(), Some(roll));
    assert_eq!(out.requested, 0, "release approach first");
    assert_eq!(out.stick, Some([0.0, 0.0]));
    let out = trap.sample(
        config,
        2,
        LedgeContext {
            motion_frame: 1.0,
            ..roll_ready_context()
        },
        Some(roll),
    );
    assert_eq!(out.requested, MAPPER_A);
}

#[test]
fn configured_common_roll_distance_is_clamped_to_travel_budget() {
    let config = LedgeSettings {
        roll_distance: Some(80.0),
        ..timing_only_settings()
    };
    let mut trap = common_roll_state(1.0);
    let out = trap.sample(config, 0, context(), Some(timed_roll(30.0, 40.0)));
    assert_eq!(trap.roll_interception().unwrap()[1], config.follow_range);
    assert_eq!(out.stick, Some([1.0, 0.0]));
    assert_eq!(out.requested, 0);
}

#[test]
fn online_ledge_jump_sends_one_neutral_jump_attack_pair_and_finishes_without_target_or_term() {
    use crate::manual_controls::{ManualOverlay, TurboOutput};
    use crate::native_input::ButtonTrace;
    let config = timing_only_settings();
    let mut trap = LedgeTrap::default();
    let launch = Opponent {
        situation: 2,
        ..enemy(CLIFF_JUMP1, true)
    };
    let ground = LedgeContext {
        motion_kind: 1,
        ..context()
    };
    let paired = trap.sample(config, 0, ground, Some(launch));
    assert_eq!(paired.requested, MAPPER_X | MAPPER_A);
    let mut overlay = ManualOverlay::default();
    let physical = ButtonTrace {
        held: MAPPER_B | MAPPER_ZR,
        ..ButtonTrace::default()
    };
    let mapped = overlay.apply_controls(
        physical,
        [-1.0, 1.0],
        TurboOutput {
            consume_dpad: false,
            stick: paired.stick,
        },
        0,
        paired.requested,
        paired.owned,
    );
    assert_eq!(mapped.buttons.held & LEDGE_BUTTONS, MAPPER_X | MAPPER_A);
    assert_eq!(mapped.stick, [0.0, 0.0]);
    // Frozen online observations retain the paired sample, without extra attempts.
    for tick in 1..=100 {
        assert_eq!(trap.sample(config, tick, ground, Some(launch)), paired);
    }
    let squat = LedgeContext {
        status: JUMP_SQUAT,
        can_act: false,
        can_jump: false,
        ..ground
    };
    let out = trap.sample(config, 101, squat, None);
    assert_eq!(
        out.requested, MAPPER_A,
        "release jump, retain paired attack"
    );
    let air = LedgeContext {
        status: 0xb,
        situation: 2,
        can_act: false,
        can_air_attack: false,
        ..ground
    };
    for frame in 0..20 {
        let out = trap.sample(
            config,
            102 + frame,
            LedgeContext {
                motion_frame: frame as f32,
                ..air
            },
            None,
        );
        assert_eq!(out.requested, MAPPER_A);
        assert_eq!(out.owned, LEDGE_BUTTONS);
        assert_eq!(out.stick, Some([0.0, 0.0]));
    }
    let stopped = LedgeContext {
        stopped: true,
        can_control: false,
        ..air
    };
    assert_eq!(trap.sample(config, 122, stopped, None).requested, MAPPER_A);
    let accepted = LedgeContext {
        status: ATTACK_AIR,
        motion_kind: NEUTRAL_AIR_MOTION,
        ..air
    };
    assert_eq!(trap.sample(config, 123, accepted, None).requested, 0);
    assert!(!trap.finished, "acceptance and a hit remain separate");
    trap.sample(
        config,
        124,
        LedgeContext {
            hit_confirmed: true,
            motion_frame: 8.0,
            ..accepted
        },
        None,
    );
    assert!(trap.finished);
}

#[test]
fn rejected_online_ledge_jump_retries_paired_inputs_until_native_takeoff_without_six_attempt_limit()
{
    let config = timing_only_settings();
    let mut trap = LedgeTrap::default();
    let launch = Opponent {
        situation: 2,
        ..enemy(CLIFF_JUMP1, true)
    };
    let ground = LedgeContext {
        motion_kind: 1,
        ..context()
    };
    assert_eq!(
        trap.sample(config, 0, ground, Some(launch)).requested,
        MAPPER_X | MAPPER_A
    );
    for frame in 1..=20 {
        let out = trap.sample(
            config,
            frame,
            LedgeContext {
                motion_frame: frame as f32,
                ..ground
            },
            None,
        );
        assert_eq!(
            out.requested,
            if frame % 2 == 0 {
                MAPPER_X | MAPPER_A
            } else {
                0
            }
        );
    }
    assert!(trap.jump_attempts > MAX_ATTEMPTS);
    for terminal in [
        LedgeContext {
            held: false,
            ..ground
        },
        LedgeContext {
            status: 0x47,
            can_control: false,
            ..ground
        },
    ] {
        let mut active = LedgeTrap::default();
        active.sample(config, 0, ground, Some(launch));
        let out = active.sample(config, 21, terminal, None);
        assert_eq!(out.requested, 0);
        assert!(!active.owns_jump_state(0xb));
    }
}

#[test]
fn roll_brakes_ahead_of_the_fixed_common_coverage_spot() {
    for side in [-1.0, 1.0] {
        let config = timing_only_settings();
        let mut trap = common_roll_state(side);
        let roll = timed_roll(25.0, 35.0);
        trap.sample(config, 0, context(), Some(roll));
        let moving = LedgeContext {
            position: [side * 13.0, 0.0],
            velocity: [side * 2.0, 0.0],
            ground_brake: Some(0.5),
            input_delay: 2.0,
            status: 0x4,
            facing: side,
            ..context()
        };
        assert_eq!(roll_stopping_distance(moving, side), Some(11.0));
        let out = trap.sample(config, 1, moving, Some(roll));
        assert_eq!(out.stick, Some([0.0, 0.0]));
        assert_eq!(out.requested, 0);
        assert!(!trap.roll_positioned);
    }
}

#[test]
fn roll_stopping_estimate_accounts_for_delay_without_blocking_missing_brake_navigation() {
    let moving = LedgeContext {
        velocity: [2.0, 0.0],
        ground_brake: Some(0.5),
        ..context()
    };
    assert_eq!(roll_stopping_distance(moving, 1.0), Some(7.0));
    assert_eq!(
        roll_stopping_distance(
            LedgeContext {
                input_delay: 3.0,
                ..moving
            },
            1.0
        ),
        Some(13.0)
    );
    assert_eq!(roll_stopping_distance(moving, -1.0), Some(0.0));
    assert_eq!(
        roll_stopping_distance(
            LedgeContext {
                ground_brake: None,
                ..moving
            },
            1.0
        ),
        None
    );
    let mut trap = common_roll_state(1.0);
    let out = trap.sample(
        timing_only_settings(),
        0,
        LedgeContext {
            ground_brake: None,
            ..context()
        },
        Some(timed_roll(25.0, 35.0)),
    );
    assert_eq!(out.stick, Some([1.0, 0.0]));
}

#[test]
fn roll_ready_down_smash_is_not_vetoed_by_a_separate_velocity_threshold() {
    let config = timing_only_settings();
    let mut trap = common_roll_state(1.0);
    let early = timed_roll(25.0, 35.0);
    let arrived = roll_ready_context();
    trap.sample(config, 0, arrived, Some(early));
    let sliding = LedgeContext {
        velocity: [0.5, 0.0],
        ..arrived
    };
    let due = Opponent {
        vulnerable_in: Some(8.0),
        actionable_in: Some(20.0),
        ..early
    };
    assert_eq!(
        trap.sample(config, 1, sliding, Some(due)).requested,
        MAPPER_A
    );
}

// Roll fixtures use a remembered stage ledge, avoiding invented root-path data.
fn common_roll_state(inward: f32) -> LedgeTrap {
    let mut trap = LedgeTrap::default();
    trap.prepare_stance([0.0, 0.0], inward);
    trap
}
fn roll_ready_context() -> LedgeContext {
    LedgeContext {
        position: [25.0, 0.0],
        ..context()
    }
}

#[test]
fn normal_getup_pairs_jump_and_attack_once_and_keeps_attack_through_stopped_squat() {
    use crate::manual_controls::{ManualOverlay, TurboOutput};
    use crate::native_input::ButtonTrace;
    let config = LedgeSettings {
        getup_attack: GetupAttack::NeutralAir,
        ..timing_only_settings()
    };
    let mut trap = LedgeTrap::default();
    let target = Opponent {
        vulnerable_in: Some(14.0),
        ..enemy(CLIFF_CLIMB, false)
    };
    let paired = trap.sample(config, 0, context(), Some(target));
    assert_eq!(paired.requested, MAPPER_X | MAPPER_A);
    assert_eq!(paired.stick, Some([0.0, 0.0]));
    let mut overlay = ManualOverlay::default();
    let mapped = overlay.apply_controls(
        ButtonTrace::default(),
        [1.0, -1.0],
        TurboOutput {
            consume_dpad: false,
            stick: paired.stick,
        },
        0,
        paired.requested,
        paired.owned,
    );
    assert_eq!(
        mapped.buttons.pressed & (MAPPER_X | MAPPER_A),
        MAPPER_X | MAPPER_A
    );
    assert_eq!(mapped.stick, [0.0, 0.0]);
    let squat = LedgeContext {
        status: JUMP_SQUAT,
        stopped: true,
        can_control: false,
        can_act: false,
        ..context()
    };
    let held = trap.sample(config, 1, squat, None);
    assert_eq!(held.requested, MAPPER_A);
    assert_eq!(held.requested & MAPPER_X, 0);
    let air = LedgeContext {
        status: 0xb,
        situation: 2,
        can_air_attack: false,
        can_act: false,
        ..context()
    };
    assert_eq!(trap.sample(config, 2, air, None).requested, MAPPER_A);
}

#[test]
fn common_roll_distance_override_and_unsafe_floor_are_respected_without_velocity_prediction() {
    let config = LedgeSettings {
        roll_distance: Some(22.0),
        auto_position: false,
        ..timing_only_settings()
    };
    let roll = Opponent {
        velocity: [0.0; 2],
        motion: None,
        ..enemy(CLIFF_ESCAPE, false)
    };
    let mut trap = common_roll_state(1.0);
    let move_in = trap.sample(config, 0, context(), Some(roll));
    assert_eq!(trap.roll_interception().unwrap()[1], 22.0);
    assert_eq!(move_in.stick, Some([1.0, 0.0]));
    let mut blocked = common_roll_state(1.0);
    let stopped = blocked.sample(
        config,
        0,
        LedgeContext {
            safe_ground: [false; 2],
            ..context()
        },
        Some(roll),
    );
    assert_eq!(stopped.stick, Some([0.0, 0.0]));
    assert_eq!(stopped.requested, 0);
}
