//! Existing regression cases for `ledge_position`. Kept outside the runtime code.
use super::*;
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
        motion_kind: 1,
        motion_rate: 1.0,
        position: [50.0, 0.0],
        velocity: [0.0, 0.0],
        ground_brake: Some(0.1),
        facing: 1.0,
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
fn settings() -> LedgeSettings {
    LedgeSettings {
        enabled: true,
        stand_distance: 10.0,
        ..LedgeSettings::default()
    }
}
#[test]
fn selects_current_floor_edge_on_both_sides_and_checks_stance_support() {
    for (x, edge) in [(20.0, 60.0), (-20.0, -60.0)] {
        let spot = nearest_ground_ledge(
            [x, 0.0],
            1.0,
            |p| p[1] == 0.0 && (-60.0..=60.0).contains(&p[0]),
            10.0,
        )
        .unwrap();
        assert!((spot.edge[0] - edge).abs() < 0.02);
        assert_eq!(spot.inward, -edge.signum());
    }
    assert!(nearest_ground_ledge([0.0, 0.0], 1.0, |_| true, 10.0).is_none());
    assert!(
        nearest_ground_ledge([0.0, 0.0], 1.0, |p| (-2.0..=2.0).contains(&p[0]), 10.0).is_none()
    );
}
#[test]
fn air_has_no_inputs_and_ground_approach_starts_on_landing() {
    let mut nav = LedgePosition::default();
    let air = LedgeContext {
        status: 0xe,
        situation: 2,
        ..context()
    };
    assert_eq!(nav.sample(0, air, settings()), LedgeOutput::default());
    nav.lock(Some(LedgeSpot {
        edge: [60.0, 0.0],
        inward: -1.0,
    }));
    let ground = LedgeContext {
        position: [20.0, 0.0],
        ..context()
    };
    let out = nav.sample(1, ground, settings());
    assert!(out.stick.unwrap()[0] > 0.0);
    assert_eq!(out.requested, 0);
    assert_eq!(nav.sample(1, air, settings()), LedgeOutput::default());
    assert_eq!(nav.spot(), None);
}
#[test]
fn waits_facing_the_edge_and_never_jumps_over_unsupported_ground() {
    let mut nav = LedgePosition::default();
    nav.lock(Some(LedgeSpot {
        edge: [60.0, 0.0],
        inward: -1.0,
    }));
    assert_eq!(nav.sample(0, context(), settings()).stick, Some([0.0, 0.0]));
    assert_eq!(nav.action_name(), "ready_at_ledge");
    let displaced = LedgeContext {
        position: [30.0, 0.0],
        safe_ground: [false, false],
        ..context()
    };
    let out = nav.sample(1, displaced, settings());
    assert_eq!(out.requested, 0);
    assert_eq!(out.stick, Some([0.0, 0.0]));
}
#[test]
fn same_tick_damage_and_toggle_off_release_navigation_and_invalid_spots_are_rejected() {
    let mut nav = LedgePosition::default();
    nav.lock(Some(LedgeSpot {
        edge: [f32::NAN, 0.0],
        inward: 1.0,
    }));
    assert_eq!(nav.spot(), None);
    nav.lock(Some(LedgeSpot {
        edge: [60.0, 0.0],
        inward: -1.0,
    }));
    let ctx = LedgeContext {
        position: [20.0, 0.0],
        ..context()
    };
    assert!(nav.sample(0, ctx, settings()).stick.unwrap()[0] > 0.0);
    assert_eq!(
        nav.sample(
            0,
            LedgeContext {
                can_act: false,
                ..ctx
            },
            settings()
        ),
        LedgeOutput::default()
    );
    assert_eq!(
        nav.sample(0, LedgeContext { held: false, ..ctx }, settings()),
        LedgeOutput::default()
    );
}
