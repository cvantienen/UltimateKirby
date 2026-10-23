//! Existing regression cases for `ledge_trump`. Kept outside the runtime code.
use super::*;

#[test]
fn configuration_is_explicit_and_rejects_invalid_values() {
    use crate::live_input::LiveConfig;
    assert!(LiveConfig::default().auto_ledge_trump);
    assert!(
        LiveConfig::parse("ledge_trap = true")
            .unwrap()
            .auto_ledge_trump
    );
    assert!(
        !LiveConfig::parse("auto_ledge_trump = false")
            .unwrap()
            .auto_ledge_trump
    );
    assert!(
        LiveConfig::parse("auto_ledge_trump = true")
            .unwrap()
            .auto_ledge_trump
    );
    assert!(LiveConfig::parse("auto_ledge_trump = yes").is_err());
}

fn context() -> LedgeContext {
    LedgeContext {
        held: false,
        gameplay: true,
        can_act: true,
        can_control: true,
        stopped: false,
        status: 0,
        situation: 0,
        motion_frame: 0.0,
        motion_kind: 1,
        motion_rate: 1.0,
        position: [8.0, 0.0],
        velocity: [0.0, 0.0],
        ground_brake: Some(0.1),
        facing: -1.0,
        previous_status: 0,
        can_grab: true,
        can_tilt: true,
        can_down_tilt: true,
        can_down_special: true,
        can_dash_attack: true,
        input_delay: 0.0,
        can_down_smash: true,
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

fn edge() -> [f32; 2] {
    [0.0, 0.0]
}

#[test]
fn tap_runs_both_ledges_and_releases_at_catch_without_a_followup() {
    for side in [-1.0, 1.0] {
        let mut state = LedgeTrump::default();
        let ground = LedgeContext {
            position: [side * 8.0, 0.0],
            ..context()
        };
        let start = state.sample(true, 0, true, ground, Some(edge()));
        assert_eq!(start.stick, Some([-side, 0.0]));
        assert_eq!(start.requested, 0);
        let falling = LedgeContext {
            situation: 2,
            status: 0xe,
            can_act: false,
            position: [-side, -0.1],
            ..ground
        };
        let down = state.sample(true, 1, false, falling, None);
        assert_eq!(down.stick, Some([0.0, -1.0]));
        // Extra mapper polls must not skip the Down input before a native frame.
        assert_eq!(state.sample(true, 2, false, falling, None), down);
        let inward = state.sample(
            true,
            3,
            false,
            LedgeContext {
                motion_frame: 1.0,
                ..falling
            },
            None,
        );
        assert_eq!(inward.stick, Some([side, 0.0]));
        let catch = LedgeContext {
            status: CLIFF_CATCH_MOVE,
            ..falling
        };
        assert_eq!(
            state.sample(true, 4, false, catch, None),
            LedgeOutput::default()
        );
        assert_eq!(
            state.sample(true, 5, false, catch, None),
            LedgeOutput::default()
        );
    }
}

#[test]
fn holding_down_cannot_relaunch_after_catch() {
    let mut state = LedgeTrump::default();
    state.sample(true, 0, true, context(), Some(edge()));
    state.sample(
        true,
        1,
        true,
        LedgeContext {
            status: CLIFF_WAIT,
            ..context()
        },
        None,
    );
    assert_eq!(
        state.sample(true, 2, true, context(), Some(edge())),
        LedgeOutput::default()
    );
    state.sample(true, 3, false, context(), Some(edge()));
    assert!(state
        .sample(true, 4, true, context(), Some(edge()))
        .stick
        .is_some());
}

#[test]
fn requires_detected_edge_and_grounded_actionability_but_no_opponent() {
    for target in [
        None,
        Some([f32::NAN, 0.0]),
        Some([120.0, 0.0]),
        Some([0.0, 10.0]),
    ] {
        assert_eq!(
            LedgeTrump::default().sample(true, 0, true, context(), target),
            LedgeOutput::default()
        );
    }
    assert_eq!(
        LedgeTrump::default().sample(false, 0, true, context(), Some(edge())),
        LedgeOutput::default()
    );
    assert_eq!(
        LedgeTrump::default().sample(
            true,
            0,
            true,
            LedgeContext {
                can_act: false,
                ..context()
            },
            Some(edge())
        ),
        LedgeOutput::default()
    );
}

#[test]
fn damage_and_excessive_drop_cancel_control() {
    for interrupted in [
        LedgeContext {
            can_act: false,
            status: 0x4b,
            situation: 2,
            ..context()
        },
        LedgeContext {
            can_act: false,
            status: 0xe,
            situation: 2,
            position: [0.0, -17.0],
            ..context()
        },
    ] {
        let mut state = LedgeTrump::default();
        state.sample(true, 0, true, context(), Some(edge()));
        assert_eq!(
            state.sample(true, 1, false, interrupted, None),
            LedgeOutput::default()
        );
    }
}

#[test]
fn grounded_attempt_times_out_instead_of_running_forever() {
    let mut state = LedgeTrump::default();
    for tick in 0..=120 {
        state.sample(
            true,
            tick,
            tick == 0,
            LedgeContext {
                motion_frame: tick as f32,
                ..context()
            },
            Some(edge()),
        );
    }
    assert_eq!(state.output, LedgeOutput::default());
}

#[test]
fn detects_nearest_empty_edge_and_uses_facing_only_for_equal_distances() {
    let floor = |p: [f32; 2]| (-60.0..=60.0).contains(&p[0]);
    for (x, facing, expected) in [
        (50.0, -1.0, 60.0),
        (-50.0, 1.0, -60.0),
        (0.0, 1.0, 60.0),
        (0.0, -1.0, -60.0),
    ] {
        let edge = nearest_floor_edge([x, 0.0], facing, floor).unwrap();
        assert!((edge[0] - expected).abs() < 0.02);
        assert_eq!(edge[1], 0.0);
    }
    assert_eq!(nearest_floor_edge([0.0, 0.0], 1.0, |_| true), None);
    assert_eq!(nearest_floor_edge([0.0, 0.0], 1.0, |_| false), None);
    assert_eq!(nearest_floor_edge([f32::NAN, 0.0], 1.0, floor), None);
}

#[test]
fn empty_edge_approach_continues_without_a_target_or_repeated_edge_query() {
    let mut state = LedgeTrump::default();
    let ctx = context();
    let edge = nearest_floor_edge(ctx.position, ctx.facing, |p| (-60.0..=60.0).contains(&p[0]));
    assert!(state.needs_edge(true));
    assert_eq!(
        state.sample(true, 0, true, ctx, edge).stick,
        Some([1.0, 0.0])
    );
    assert!(!state.needs_edge(true));
    assert_eq!(
        state.sample(true, 1, false, ctx, None).stick,
        Some([1.0, 0.0])
    );
    assert_eq!(state.action_name(), "run_to_edge");
}

#[test]
fn old_config_down_press_is_consumed_before_mapping_even_without_an_edge() {
    use crate::ledge_trap::DPAD_DOWN;
    use crate::live_input::LiveConfig;
    use crate::manual_controls::{
        suppress_reserved_taunts, DpadReservation, ManualOverlay, TurboOutput,
    };
    use crate::native_input::ButtonTrace;
    let config = LiveConfig::parse("enabled = true\nledge_trap = true").unwrap();
    let mut reservation = DpadReservation::default();
    let mut overlay = ManualOverlay::default();
    let mut policy = LedgeTrump::default();
    for (tick, held) in [(0, DPAD_DOWN), (1, 0)] {
        let consumed = reservation.sample(
            if config.auto_ledge_trump {
                DPAD_DOWN
            } else {
                0
            },
            held,
            true,
        );
        let output = policy.sample(config.auto_ledge_trump, tick, held != 0, context(), None);
        let mapped = overlay.apply_controls(
            ButtonTrace {
                held,
                previous: DPAD_DOWN,
                pressed: DPAD_DOWN,
                released: DPAD_DOWN,
            },
            [0.25, 0.0],
            TurboOutput::default(),
            consumed,
            output.requested,
            output.owned,
        );
        for word in [
            mapped.buttons.held,
            mapped.buttons.previous,
            mapped.buttons.pressed,
            mapped.buttons.released,
        ] {
            assert_eq!(word & DPAD_DOWN, 0);
        }
        assert_eq!(mapped.stick, [0.25, 0.0]);
        // Native history must not reintroduce the down-taunt action.
        assert_eq!(suppress_reserved_taunts(0x1000 | 1, consumed), 1);
    }
}
#[test]
fn hitstop_preserves_a_run_but_same_tick_damage_cancels_cached_movement() {
    let mut state = LedgeTrump::default();
    let ctx = context();
    assert_ne!(state.sample(true, 0, true, ctx, Some(edge())).owned, 0);
    let stopped = LedgeContext {
        stopped: true,
        can_act: false,
        ..ctx
    };
    assert_eq!(state.sample(true, 1, false, stopped, None).owned, 0);
    assert_ne!(state.sample(true, 2, false, ctx, None).owned, 0);
    let damage = LedgeContext {
        status: 0x49,
        can_act: false,
        ..ctx
    };
    assert_eq!(state.sample(true, 2, false, damage, None).owned, 0);
    assert_eq!(state.sample(true, 3, false, ctx, None).owned, 0);
}
