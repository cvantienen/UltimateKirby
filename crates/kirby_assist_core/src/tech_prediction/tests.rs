//! Existing regression cases for `tech_prediction`. Kept outside the runtime code.
use super::*;

fn crosses_plane(ray: &CollisionRay, axis: usize, position: f32) -> bool {
    let t = (position - ray.origin[axis]) / ray.delta[axis];
    (0.0..=1.0).contains(&t)
}

#[test]
fn native_gate_reasons_distinguish_disabled_hitstop_and_non_tumble_hits() {
    assert_eq!(tech_gate(0x4b, 2, true, false), TechGate::PassiveDisabled);
    assert_eq!(tech_gate(0x4b, 0, false, false), TechGate::NotAirborne);
    assert_eq!(tech_gate(0x4b, 2, false, true), TechGate::Hitstop);
    assert_eq!(tech_gate(0x47, 0, false, false), TechGate::OutsideTumble);
    assert_eq!(tech_gate(0x48, 2, false, false), TechGate::OutsideTumble);
    assert_eq!(tech_gate(0x4b, 2, false, false), TechGate::Predict);
}

#[test]
fn sweep_catches_fast_floor_wall_and_ceiling_before_contact() {
    for (x, y, vx, vy, axis, plane) in [
        (0.0, 40.0, 0.0, -15.0, 1, 0.0),
        (0.0, 0.0, 20.0, 0.0, 0, 40.0),
        (0.0, 0.0, 0.0, 20.0, 1, 45.0),
    ] {
        assert!(tech_probes(x, y, vx, vy, 5.0)
            .iter()
            .flatten()
            .any(|r| crosses_plane(r, axis, plane)));
    }
    assert!(!tech_probes(0.0, 40.0, 0.0, -0.2, 5.0)
        .iter()
        .flatten()
        .any(|r| crosses_plane(r, 1, 0.0)));
}

#[test]
fn body_edge_probe_catches_a_ledge_that_center_ray_misses() {
    let probes = tech_probes(0.0, 10.0, 0.0, -5.0, 5.0);
    let hits_ledge =
        |r: &CollisionRay| crosses_plane(r, 1, 0.0) && (2.0..=3.0).contains(&r.origin[0]);
    assert!(!hits_ledge(probes[1].as_ref().unwrap()));
    assert!(hits_ledge(probes[2].as_ref().unwrap()));
}

#[test]
fn stationary_or_invalid_motion_does_not_request_collision_probes() {
    for (vx, horizon) in [
        (0.0, 5.0),
        (f32::NAN, 5.0),
        (2.0, 0.0),
        (2.0, f32::INFINITY),
        (f32::MAX, 5.0),
    ] {
        assert!(tech_probes(0.0, 0.0, vx, 0.0, horizon)
            .iter()
            .all(Option::is_none));
    }
}
#[test]
fn contact_refinement_finds_early_impact_inside_a_delay_extended_horizon() {
    let hit = first_contact_in(0.0, 0.0, 10.0, 0.0, 12.0, |ray| {
        crosses_plane(&ray, 0, 42.5)
    })
    .unwrap();
    // Wall body allowance adds 2.5: actual modeled time is four frames.
    assert!((4.0..=4.75).contains(&hit), "{hit}");
    assert!(
        first_contact_in(0.0, 0.0, 10.0, 0.0, 3.0, |ray| crosses_plane(&ray, 0, 42.5)).is_none()
    );
}
#[test]
fn contact_refinement_covers_walls_on_both_sides_ceiling_and_narrow_floor_edges() {
    for side in [-1.0, 1.0] {
        let hit = first_contact_in(0.0, 0.0, side * 10.0, 0.0, 12.0, |ray| {
            crosses_plane(&ray, 0, side * 42.5)
        })
        .unwrap();
        assert!((4.0..=4.75).contains(&hit));
    }
    assert!(
        first_contact_in(0.0, 0.0, 0.0, 15.0, 12.0, |ray| crosses_plane(
            &ray, 1, 52.0
        ))
        .is_some()
    );
    assert!(
        first_contact_in(0.0, 10.0, 0.0, -5.0, 12.0, |ray| crosses_plane(
            &ray, 1, 0.0
        ) && ray.origin[0] > 2.0)
        .is_some()
    );
    assert!(first_contact_in(0.0, 0.0, f32::NAN, 1.0, 5.0, |_| true).is_none());
}
