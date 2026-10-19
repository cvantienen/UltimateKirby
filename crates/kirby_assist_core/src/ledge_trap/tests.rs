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
        position: [0.0, 0.0],
        velocity: [0.0, 0.0],
        facing: 1.0,
        previous_status: 0,
        can_grab: true,
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
                position: [8.0, 0.0],
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
fn rolls_never_down_tilt_and_are_punished_only_after_exit_and_at_grab_range() {
    let mut state = LedgeTrap::default();
    assert_eq!(
        step(&mut state, 0, context(), enemy(CLIFF_ESCAPE, false)).requested,
        0
    );
    let far = Opponent {
        position: [9.0, 0.0],
        ..enemy(0, true)
    };
    let wait = step(&mut state, 1, context(), far);
    assert_eq!(wait.stick, Some([0.65, 0.0]));
    assert_eq!(wait.requested, 0);
    assert_eq!(
        step(&mut state, 2, context(), enemy(0, true)).requested,
        MAPPER_R
    );
    let guard = LedgeContext {
        status: 0x1c,
        can_act: false,
        ..context()
    };
    assert_eq!(
        step(&mut state, 3, guard, enemy(0, true)).requested,
        MAPPER_ZR
    );
    assert_eq!(
        step(&mut state, 5, guard, enemy(0, true)).requested,
        MAPPER_ZR | MAPPER_R
    );
}
#[test]
fn crossed_up_roll_turns_before_entering_shield() {
    let mut state = LedgeTrap::default();
    step(&mut state, 0, context(), enemy(CLIFF_ESCAPE, false));
    let behind = Opponent {
        position: [-5.0, 0.0],
        ..enemy(0, true)
    };
    let turn = step(&mut state, 1, context(), behind);
    assert_eq!(turn.requested, 0);
    assert_eq!(turn.stick, Some([-0.3, 0.0]));
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
        position: [10.0, 0.0],
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
    assert_eq!(roll.stick, Some([0.0, 0.0]));
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
    assert_eq!(presses, 3);
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
    assert_eq!(pursuit.stick, Some([0.65, 0.0]));
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
fn roll_travel_is_followed_beyond_acquisition_range_without_attacking_during_roll() {
    let mut state = LedgeTrap::default();
    step(&mut state, 0, context(), enemy(CLIFF_ESCAPE, false));
    let travel = Opponent {
        position: [28.0, 0.0],
        velocity: [2.0, 0.0],
        ..enemy(CLIFF_ESCAPE, false)
    };
    let following = step(&mut state, 1, context(), travel);
    assert_eq!(following.requested, 0);
    assert_eq!(following.stick, Some([0.65, 0.0]));
    assert_eq!(state.target(), Some(2));
    let unsupported = LedgeContext {
        safe_ground: [true, false],
        ..context()
    };
    assert_eq!(
        step(&mut state, 2, unsupported, travel).stick,
        Some([0.0, 0.0])
    );
    let endpoint = Opponent {
        position: [28.0, 0.0],
        ..enemy(0, true)
    };
    let closed = LedgeContext {
        position: [22.0, 0.0],
        ..context()
    };
    assert_eq!(step(&mut state, 3, closed, endpoint).requested, MAPPER_R);
    step(
        &mut state,
        4,
        LedgeContext {
            status: 0x1c,
            can_act: false,
            ..closed
        },
        endpoint,
    );
    assert_eq!(
        step(
            &mut state,
            6,
            LedgeContext {
                status: 0x1c,
                can_act: false,
                ..closed
            },
            endpoint
        )
        .requested,
        MAPPER_ZR | MAPPER_R
    );
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
    assert_eq!(step(&mut state, 4, context(), far).stick, Some([0.65, 0.0]));
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
fn disabled_positioning_still_tracks_but_does_not_chase_rolls() {
    let mut state = LedgeTrap::default();
    let config = LedgeSettings {
        auto_position: false,
        ..settings()
    };
    state.sample(config, 0, context(), Some(enemy(CLIFF_ESCAPE, false)));
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
fn default_jump_response_requests_short_hop_neutral_air_at_the_predicted_window() {
    let mut state = LedgeTrap::default();
    let config = LedgeSettings {
        enabled: true,
        ..LedgeSettings::default()
    };
    let hang = state.sample(config, 0, context(), Some(enemy(CLIFF_WAIT, true)));
    assert_eq!(hang.stick, Some([0.0, 0.6]));
    assert_eq!(hang.requested, 0);
    let jump = Opponent {
        situation: 2,
        position: [5.0, 8.0],
        ..enemy(CLIFF_JUMP1, true)
    };
    let attack = state.sample(config, 1, context(), Some(jump));
    assert_eq!(attack.stick, Some([0.0, 0.0]));
    assert_eq!(attack.requested, MAPPER_X | MAPPER_A);
    state.sample(
        config,
        2,
        LedgeContext {
            status: ATTACK_AIR,
            hit_confirmed: true,
            can_act: false,
            ..context()
        },
        Some(jump),
    );
    assert_eq!(state.action_name(), "finished");
}
#[test]
fn grounded_anti_air_waits_neutral_until_a_jump_rises_into_reach() {
    let mut state = LedgeTrap::default();
    let config = LedgeSettings {
        enabled: true,
        ..LedgeSettings::default()
    };
    let low = Opponent {
        situation: 2,
        position: [5.0, -12.0],
        ..enemy(CLIFF_JUMP1, true)
    };
    // Acquire via a hanging snapshot first, then follow the rising target.
    state.sample(config, 0, context(), Some(enemy(CLIFF_WAIT, true)));
    let waiting = state.sample(config, 1, context(), Some(low));
    assert_eq!(waiting.stick, Some([0.0, 0.0]));
    assert_eq!(waiting.requested, 0);
    let in_reach = Opponent {
        position: [5.0, 5.0],
        ..low
    };
    assert_eq!(
        state.sample(config, 2, context(), Some(in_reach)).requested,
        MAPPER_X | MAPPER_A
    );
}
#[test]
fn neutral_air_range_invulnerability_and_release_gates_are_respected() {
    let mut state = LedgeTrap::default();
    let config = LedgeSettings {
        enabled: true,
        ..LedgeSettings::default()
    };
    let jump = Opponent {
        position: [-5.0, 8.0],
        situation: 2,
        ..enemy(CLIFF_JUMP1, false)
    };
    let prepare = state.sample(config, 0, context(), Some(jump));
    assert_eq!(prepare.requested, 0);
    assert_eq!(prepare.stick, Some([0.0, 0.0]));
    let protected = Opponent {
        vulnerable: false,
        vulnerable_in: Some(40.0),
        ..jump
    };
    assert_eq!(
        state
            .sample(config, 1, context(), Some(protected))
            .requested,
        0
    );
    let high = Opponent {
        position: [-5.0, 30.0],
        ..jump
    };
    assert_eq!(state.sample(config, 2, context(), Some(high)).requested, 0);
    assert_eq!(
        state
            .sample(
                config,
                3,
                context(),
                Some(Opponent {
                    vulnerable: true,
                    ..jump
                })
            )
            .requested,
        MAPPER_X | MAPPER_A
    );
    assert_eq!(
        state.sample(
            config,
            4,
            LedgeContext {
                held: false,
                ..context()
            },
            Some(jump)
        ),
        LedgeOutput::default()
    );
}
#[test]
fn grab_retries_after_one_release_update_without_the_attack_cooldown() {
    let mut state = LedgeTrap::default();
    step(&mut state, 0, context(), enemy(CLIFF_ESCAPE, false));
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
    step(&mut state, 0, context(), enemy(CLIFF_ESCAPE, false));
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
fn actual_grab_whiff_retries_are_bounded_to_three_animation_starts() {
    let mut state = LedgeTrap::default();
    step(&mut state, 0, context(), enemy(CLIFF_ESCAPE, false));
    let target = enemy(0, true);
    let startup = LedgeContext {
        status: 0x37,
        can_act: false,
        ..context()
    };
    for attempt in 0..3 {
        let tick = attempt * 2 + 1;
        assert_eq!(
            step(&mut state, tick, context(), target).requested,
            MAPPER_R
        );
        step(&mut state, tick + 1, startup, target);
    }
    assert_eq!(step(&mut state, 7, context(), target).requested, 0);
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
fn small_fast_jump_is_intercepted_before_current_root_enters_the_old_box() {
    // Synthetic low-root/fast-rise case for a Pichu-like target, not recorded
    // Pichu hurtbox data. Prediction must see the root at attack startup.
    let mut state = LedgeTrap::default();
    step(&mut state, 0, context(), enemy(CLIFF_WAIT, false));
    let rising = Opponent {
        situation: 2,
        position: [5.0, -15.0],
        velocity: [0.0, 2.0],
        vulnerable_in: Some(4.0),
        ..enemy(CLIFF_JUMP1, false)
    };
    assert_eq!(
        step(&mut state, 1, context(), rising).requested,
        MAPPER_X | MAPPER_A
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
    assert_eq!(out.stick, Some([0.65, 0.0]));
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
fn jump_prediction_overlaps_active_frames_with_reported_expiry_and_requires_known_protection() {
    let mut state = LedgeTrap::default();
    step(&mut state, 0, context(), enemy(CLIFF_WAIT, false));
    let jump = Opponent {
        situation: 2,
        position: [5.0, 0.0],
        velocity: [0.0, 0.5],
        ..enemy(CLIFF_JUMP1, false)
    };
    assert_eq!(step(&mut state, 1, context(), jump).requested, 0);
    assert_eq!(
        step(
            &mut state,
            2,
            context(),
            Opponent {
                vulnerable_in: Some(40.0),
                ..jump
            }
        )
        .requested,
        0
    );
    assert_eq!(
        step(
            &mut state,
            3,
            context(),
            Opponent {
                vulnerable_in: Some(30.0),
                ..jump
            }
        )
        .requested,
        MAPPER_X | MAPPER_A
    );
}

#[test]
fn short_jump_status_is_recovered_from_native_history_and_remembered_while_kirby_is_busy() {
    let mut state = LedgeTrap::default();
    step(&mut state, 0, context(), enemy(CLIFF_WAIT, false));
    let jumping = Opponent {
        situation: 2,
        position: [5.0, 5.0],
        ..enemy(CLIFF_JUMP1, true)
    };
    let busy = LedgeContext {
        status: GUARD_OFF,
        can_act: false,
        ..context()
    };
    step(&mut state, 1, busy, jumping);
    assert_eq!(state.action_name(), "short_hop_neutral_air");
    let air = Opponent {
        status: 0xb,
        previous_status: CLIFF_JUMP3,
        ..jumping
    };
    assert_eq!(
        step(&mut state, 2, context(), air).requested,
        MAPPER_X | MAPPER_A
    );
    assert_eq!(step(&mut state, 3, context(), air).requested, 0);
    let mut missed = LedgeTrap::default();
    step(&mut missed, 0, context(), enemy(CLIFF_WAIT, false));
    assert_eq!(
        step(&mut missed, 1, context(), air).requested,
        MAPPER_X | MAPPER_A
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
fn forward_tilt_does_not_chase_or_turn_walk_after_normal_getup_starts() {
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
        let out = state.sample(config, 0, context(), Some(climb));
        assert_eq!(out.stick, Some([0.0, 0.0]));
        assert_eq!(out.requested, 0);
    }
}

#[test]
fn roll_grab_can_start_before_roll_exit_when_known_expiry_and_capped_travel_match_grab_startup() {
    let mut state = LedgeTrap::default();
    let roll = Opponent {
        position: [2.0, 0.0],
        velocity: [1.0, 0.0],
        vulnerable_in: Some(6.0),
        motion: Some(MotionTiming {
            kind: 1,
            frame: 20.0,
            rate: 1.0,
            remaining: Some(2.0),
        }),
        ..enemy(CLIFF_ESCAPE, false)
    };
    assert_eq!(
        step(
            &mut state,
            0,
            context(),
            Opponent {
                vulnerable_in: Some(8.0),
                ..roll
            }
        )
        .requested,
        0
    );
    assert_eq!(step(&mut state, 1, context(), roll).requested, MAPPER_R);
    let mut dash = LedgeTrap::default();
    let running = LedgeContext {
        status: 0x3,
        ..context()
    };
    assert_eq!(
        step(
            &mut dash,
            0,
            running,
            Opponent {
                vulnerable_in: Some(9.0),
                ..roll
            }
        )
        .requested,
        0
    );
}

#[test]
fn roll_grab_uses_standing_timing_and_waits_until_idle() {
    for status in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, GUARD_ON, GUARD, GUARD_OFF] {
        let mut state = LedgeTrap::default();
        let ctx = LedgeContext {
            status,
            ..context()
        };
        let roll = Opponent {
            motion: Some(MotionTiming {
                kind: 1,
                frame: 40.0,
                rate: 1.0,
                remaining: Some(0.0),
            }),
            vulnerable_in: Some(8.0),
            ..enemy(CLIFF_ESCAPE, false)
        };
        assert_eq!(step(&mut state, 0, ctx, roll).requested, 0);
        assert_eq!(state.last_output.stick, Some([0.0, 0.0]));
        let ready = Opponent {
            vulnerable_in: Some(7.0),
            ..roll
        };
        let out = step(&mut state, 1, ctx, ready);
        assert_eq!(out.requested, if status == 0 { MAPPER_R } else { 0 });
        assert_eq!(out.stick, Some([0.0, 0.0]));
        if status != 0 {
            let out = step(&mut state, 2, context(), ready);
            assert_eq!(out.requested, MAPPER_R);
            assert_eq!(out.stick, Some([0.0, 0.0]));
        }
        assert_eq!(state.expected_grab_status(), Some(GRAB_START));
    }
}

#[test]
fn roll_grab_unknown_protection_stops_movement_in_reach_without_spending_attempts() {
    let mut state = LedgeTrap::default();
    let roll = enemy(CLIFF_ESCAPE, false);
    for tick in 0..8 {
        let out = step(&mut state, tick, context(), roll);
        assert_eq!(out.stick, Some([0.0, 0.0]));
        assert_eq!(out.requested, 0);
    }
    assert_eq!(state.attempts, 0);
    assert_eq!(state.action_name(), "roll_wait_endpoint_data");
    assert_eq!(
        step(&mut state, 8, context(), enemy(CLIFF_ESCAPE, true)).requested,
        MAPPER_R
    );
}

#[test]
fn roll_grab_waits_for_native_readiness_then_requests_without_an_extra_prep_update() {
    let mut state = LedgeTrap::default();
    let roll = enemy(CLIFF_ESCAPE, true);
    let not_ready = LedgeContext {
        can_grab: false,
        ..context()
    };
    for tick in 0..12 {
        let out = step(&mut state, tick, not_ready, roll);
        assert_eq!(out.requested, 0);
        assert_eq!(out.stick, Some([0.0, 0.0]));
    }
    assert_eq!(state.attempts, 0);
    assert_eq!(state.action_name(), "roll_wait_grab_ready");
    assert_eq!(step(&mut state, 12, context(), roll).requested, MAPPER_R);
    assert_eq!(step(&mut state, 13, context(), roll).requested, 0);
    assert_eq!(step(&mut state, 14, context(), roll).requested, MAPPER_R);
}

#[test]
fn roll_grab_second_active_frame_is_checked_before_deciding_to_turn() {
    let mut state = LedgeTrap::default();
    let roll = Opponent {
        position: [-7.0, 0.0],
        velocity: [1.0, 0.0],
        motion: Some(MotionTiming {
            kind: 1,
            frame: 20.0,
            rate: 1.0,
            remaining: Some(7.0),
        }),
        vulnerable_in: Some(7.0),
        ..enemy(CLIFF_ESCAPE, false)
    };
    // At startup frame 6 the predicted root is behind; at frame 7 it is
    // within the policy's forward gate and vulnerable. Do not preempt it with a turn.
    let out = step(&mut state, 0, context(), roll);
    assert_eq!(out.requested, MAPPER_R);
    assert_eq!(out.stick, Some([0.0, 0.0]));
}

#[test]
fn roll_grab_brakes_instead_of_turning_toward_a_future_crossing() {
    let mut state = LedgeTrap::default();
    let running = LedgeContext {
        status: 4,
        velocity: [1.0, 0.0],
        ..context()
    };
    let out = step(&mut state, 0, running, enemy(CLIFF_ESCAPE, true));
    assert_eq!(out.requested, 0);
    assert_eq!(out.stick, Some([0.0, 0.0]));
    assert_eq!(state.action_name(), "roll_settle_for_standing_grab");
}

#[test]
fn roll_grab_turns_for_an_actual_crossup_and_waits_out_native_turn_status() {
    let mut state = LedgeTrap::default();
    let roll = Opponent {
        position: [-5.0, 0.0],
        ..enemy(CLIFF_ESCAPE, true)
    };
    let turning = step(&mut state, 0, context(), roll);
    assert_eq!(turning.stick, Some([-0.3, 0.0]));
    assert_eq!(turning.requested, 0);
    let ctx = LedgeContext {
        status: 6,
        facing: -1.0,
        ..context()
    };
    assert_eq!(step(&mut state, 1, ctx, roll).requested, 0);
    assert_eq!(state.last_output.stick, Some([0.0, 0.0]));
    assert_eq!(
        step(&mut state, 2, LedgeContext { status: 0, ..ctx }, roll).requested,
        MAPPER_R
    );
}

#[test]
fn roll_grab_releases_run_and_shield_without_spending_attempts() {
    for status in [3, 4, GUARD_ON, GUARD, GUARD_OFF] {
        let mut state = LedgeTrap::default();
        let roll = enemy(CLIFF_ESCAPE, true);
        for tick in 0..3 {
            let out = step(
                &mut state,
                tick,
                LedgeContext {
                    status,
                    ..context()
                },
                roll,
            );
            assert_eq!(out.requested, 0);
            assert_eq!(out.stick, Some([0.0, 0.0]));
            assert_eq!(state.expected_grab_status(), None);
        }
        let out = step(&mut state, 3, context(), roll);
        assert_eq!(out.requested, MAPPER_R);
        assert_eq!(out.stick, Some([0.0, 0.0]));
        assert_eq!(state.expected_grab_status(), Some(GRAB_START));
    }
}

#[test]
fn roll_grab_approaches_only_while_separated_and_brakes_for_closing_momentum() {
    let roll = Opponent {
        position: [12.0, 0.0],
        velocity: [1.0, 0.0],
        motion: Some(MotionTiming {
            kind: 1,
            frame: 20.0,
            rate: 1.0,
            remaining: Some(1.0),
        }),
        ..enemy(CLIFF_ESCAPE, false)
    };
    let mut state = LedgeTrap::default();
    assert!(step(&mut state, 0, context(), roll).stick.unwrap()[0] > 0.0);
    let closing = LedgeContext {
        position: [3.0, 0.0],
        velocity: [1.5, 0.0],
        status: 5,
        ..context()
    };
    let out = step(&mut state, 1, closing, roll);
    assert_eq!(out.stick, Some([0.0, 0.0]));
    assert_eq!(out.requested, 0);
}

#[test]
fn roll_grab_native_transition_terms_match_selected_movement() {
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
            remaining: Some(20.0),
        }),
        ..enemy(CLIFF_ESCAPE, false)
    };
    assert_eq!(step(&mut state, 0, context(), roll).stick, Some([0.0, 0.0]));
    let moving = Opponent {
        position: [4.0, 0.0],
        motion: Some(MotionTiming {
            kind: 7,
            frame: 10.5,
            rate: 0.5,
            remaining: Some(19.0),
        }),
        ..roll
    };
    assert_eq!(
        step(&mut state, 1, context(), moving).stick,
        Some([0.65, 0.0])
    );
    assert_eq!(
        step(&mut state, 2, context(), moving).stick,
        Some([0.65, 0.0]),
        "another poll of the same animation frame keeps the estimate"
    );
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
    for attempt in 0..3 {
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
    assert_eq!(step(&mut state, 11, context(), climb).requested, 0);
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
    assert_eq!(
        step(&mut roll, 0, context(), after_roll).requested,
        MAPPER_R
    );
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

// Acquire the real ledge anchor, then request a roll grab from a stage-side
// position. This prepares native-like capture observations for both ledges.
fn roll_grab_at_ledge(anchor_x: f32, inward: f32) -> (LedgeTrap, LedgeContext, Opponent) {
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
        ..enemy(CLIFF_ESCAPE, true)
    };
    assert_eq!(step(&mut state, 1, ctx, rolling).requested, MAPPER_R);
    (state, ctx, rolling)
}

#[test]
fn captured_roll_throws_toward_each_ledge_from_either_facing_without_pummeling() {
    for (anchor, inward) in [(0.0, 1.0), (100.0, -1.0)] {
        for facing in [-1.0, 1.0] {
            let (mut state, ctx, rolling) = roll_grab_at_ledge(anchor, inward);
            let captured = Opponent {
                status: 0x42,
                ..rolling
            };
            let pull = LedgeContext {
                status: GRAB_PULL,
                can_act: false,
                facing,
                ..ctx
            };
            assert_eq!(step(&mut state, 2, pull, captured).stick, Some([0.0, 0.0]));
            let hold = LedgeContext {
                status: GRAB_HOLD,
                ..pull
            };
            let throw = step(&mut state, 3, hold, captured);
            assert_eq!(throw.stick, Some([-inward, 0.0]));
            assert_eq!(throw.requested, 0, "direction only: no A pummel or R");
            assert_eq!(throw.owned, LEDGE_BUTTONS);
            let released = LedgeContext {
                held: false,
                ..hold
            };
            assert_eq!(
                step(&mut state, 4, released, captured),
                LedgeOutput::default()
            );
        }
    }
}

#[test]
fn outward_throw_respects_setting_hitstop_and_missing_ledge_anchor() {
    let (mut state, ctx, rolling) = roll_grab_at_ledge(0.0, 1.0);
    let hold = LedgeContext {
        status: GRAB_HOLD,
        can_act: false,
        ..ctx
    };
    let config = LedgeSettings {
        throw_outward: false,
        ..settings()
    };
    assert_eq!(
        state.sample(config, 2, hold, Some(rolling)).stick,
        Some([0.0, 0.0])
    );
    let stopped = LedgeContext {
        stopped: true,
        ..hold
    };
    assert_eq!(step(&mut state, 3, stopped, rolling).stick, None);
    assert_eq!(step(&mut state, 4, hold, rolling).stick, Some([-1.0, 0.0]));

    let mut no_anchor = LedgeTrap::default();
    step(&mut no_anchor, 0, context(), enemy(CLIFF_ESCAPE, true));
    let hold = LedgeContext {
        status: GRAB_HOLD,
        can_act: false,
        ..context()
    };
    assert_eq!(
        step(&mut no_anchor, 1, hold, enemy(0x42, false)).stick,
        Some([0.0, 0.0])
    );
}

#[test]
fn hit_confirmation_survives_target_loss_returns_to_stance_and_rearms_on_regrab() {
    let (mut state, ctx, rolling) = roll_grab_at_ledge(0.0, 1.0);
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
    assert_eq!(returning.stick, Some([-0.65, 0.0]));
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
    assert_eq!(step(&mut state, 8, ctx, rolling).requested, MAPPER_R);
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
fn completed_punish_rearms_when_new_ledge_visit_skips_the_hanging_sample() {
    let (mut state, ctx, rolling) = roll_grab_at_ledge(0.0, 1.0);
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
    assert_eq!(state.action_name(), "short_hop_neutral_air");
    assert_eq!(response.requested, MAPPER_X | MAPPER_A);
    assert_eq!(step(&mut state, 5, ctx, jump).requested, 0);
}

#[test]
fn roll_interception_commits_on_recognition_and_never_chases_current_root() {
    let config = LedgeSettings {
        roll_distance: Some(25.0),
        ..settings()
    };
    let mut state = LedgeTrap::default();
    state.prepare_stance([0.0, 0.0], 1.0);
    let ctx = LedgeContext {
        position: [10.0, 0.0],
        facing: -1.0,
        ..context()
    };
    let roll = enemy(CLIFF_ESCAPE, false);
    let first = state.sample(config, 0, ctx, Some(roll));
    assert!(first.stick.unwrap()[0] > 0.0);
    let locked = state.roll_interception().unwrap();
    assert_eq!(locked, [25.0, 18.5, 1.0]);
    for (tick, root) in [(1, 3.0), (2, 15.0), (3, 30.0)] {
        state.sample(
            config,
            tick,
            ctx,
            Some(Opponent {
                position: [root, 0.0],
                ..roll
            }),
        );
        assert_eq!(state.roll_interception(), Some(locked));
    }
    let arrived = LedgeContext {
        position: [18.5, 0.0],
        facing: 1.0,
        ..ctx
    };
    let waiting = state.sample(config, 4, arrived, Some(roll));
    assert_eq!(waiting.stick, Some([0.0, 0.0]));
    assert_eq!(waiting.requested, 0);
    assert_eq!(state.roll_interception(), Some(locked));
}

#[test]
fn roll_interception_times_first_eligible_standing_grab_before_roll_ends() {
    let config = LedgeSettings {
        roll_distance: Some(25.0),
        ..settings()
    };
    let mut state = LedgeTrap::default();
    state.prepare_stance([0.0, 0.0], 1.0);
    let roll = enemy(CLIFF_ESCAPE, false);
    let initial = LedgeContext {
        position: [10.0, 0.0],
        ..context()
    };
    state.sample(config, 0, initial, Some(roll));
    let arrived = LedgeContext {
        position: [18.5, 0.0],
        ..context()
    };
    let protected = Opponent {
        position: [20.0, 0.0],
        velocity: [1.0, 0.0],
        vulnerable_in: Some(8.0),
        motion: Some(MotionTiming {
            kind: 1,
            frame: 20.0,
            rate: 1.0,
            remaining: Some(5.0),
        }),
        ..roll
    };
    let waiting = state.sample(config, 1, arrived, Some(protected));
    assert_eq!(waiting.stick, Some([0.0, 0.0]));
    assert_eq!(waiting.requested, 0);
    let ready = Opponent {
        vulnerable_in: Some(7.0),
        ..protected
    };
    let request = state.sample(config, 2, arrived, Some(ready));
    assert_eq!(request.stick, Some([0.0, 0.0]));
    assert_eq!(request.requested, MAPPER_R);
    assert_eq!(state.expected_grab_status(), Some(GRAB_START));
}

#[test]
fn roll_interception_auto_latches_once_from_reliable_motion() {
    let mut state = LedgeTrap::default();
    let initial = enemy(CLIFF_ESCAPE, false);
    assert_eq!(step(&mut state, 0, context(), initial).requested, 0);
    assert_eq!(state.roll_interception(), None);
    let moving = Opponent {
        position: [5.0, 0.0],
        velocity: [1.0, 0.0],
        motion: Some(MotionTiming {
            kind: 1,
            frame: 10.0,
            rate: 1.0,
            remaining: Some(20.0),
        }),
        ..initial
    };
    step(&mut state, 1, context(), moving);
    let locked = state.roll_interception().unwrap();
    assert_eq!(locked[0], 25.0);
    step(
        &mut state,
        2,
        context(),
        Opponent {
            position: [10.0, 0.0],
            velocity: [3.0, 0.0],
            motion: None,
            ..moving
        },
    );
    assert_eq!(state.roll_interception(), Some(locked));
}
