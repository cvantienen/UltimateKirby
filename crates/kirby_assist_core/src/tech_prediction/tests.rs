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

#[test]
fn one_way_platform_is_only_used_by_descending_floor_probes() {
    let platform = |ray: CollisionRay| {
        ray.passable && ray.origin[1] >= 10.0 && ray.origin[1] + ray.delta[1] <= 10.0
    };
    assert!(first_contact_in(0.0, 20.0, 2.0, -2.0, 10.0, platform).is_some());
    assert!(first_contact_in(0.0, 0.0, 2.0, 2.0, 10.0, platform).is_none());
    assert!(first_contact_in(0.0, 9.0, 2.0, 0.0, 10.0, platform).is_none());
    // Disabling passable geometry must not remove real solid wall/ceiling checks.
    assert!(tech_probes(0.0, 0.0, 2.0, 2.0, 5.0)[3..]
        .iter()
        .flatten()
        .all(|r| !r.passable));
    assert!(
        first_contact_in(0.0, 0.0, 2.0, 0.0, 10.0, |ray| ray.origin[0] + ray.delta[0]
            >= 10.0)
        .is_some()
    );
}

#[test]
fn native_air_escape_availability_blocks_tech_guard_even_during_tumble() {
    for status in 0x49..=0x4f {
        assert!(tech_press_allowed(status, false));
        assert!(!tech_press_allowed(status, true));
    }
    assert!(!tech_press_allowed(0xe, false));
    assert!(tech_press_allowed(CATCHED_GANON, false));
}

#[test]
fn single_probe_refinement_matches_the_full_probe_generator_bit_for_bit() {
    for x in [-100.0, 0.0, 99.5] {
        for vx in [-20.0, -0.11, 0.0, 0.11, 20.0] {
            for vy in [-20.0, -0.11, 0.0, 0.11, 20.0] {
                for horizon in [1.0, 1.25, 3.0, 5.0, 12.0, 20.0] {
                    let full = tech_probes(x, 12.25, vx, vy, horizon);
                    for (index, ray) in full.into_iter().enumerate() {
                        let Some(ray) = ray else { continue };
                        for t in [1.0, (1.0 + horizon) * 0.5, horizon] {
                            let expected = tech_probes(x, 12.25, vx, vy, t)[index].unwrap();
                            let actual = refined_probe(ray, index, vx, vy, t, Body::default());
                            assert_eq!(
                                actual.origin.map(f32::to_bits),
                                expected.origin.map(f32::to_bits)
                            );
                            assert_eq!(
                                actual.delta.map(f32::to_bits),
                                expected.delta.map(f32::to_bits)
                            );
                            assert_eq!(actual.passable, expected.passable);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn accelerating_descent_catches_floor_missed_by_constant_velocity() {
    let floor = |ray: CollisionRay| crosses_plane(&ray, 1, 0.0);
    assert!(first_contact_in(0.0, 3.0, 0.0, -0.2, 5.0, floor).is_none());
    let mut calls = 0;
    let contact = first_contact_with_floor_acceleration(0.0, 3.0, 0.0, -0.2, 5.0, -0.2, |ray| {
        calls += 1;
        assert!(ray.passable);
        floor(ray)
    });
    assert!(contact.is_some_and(|t| (1.0..=5.0).contains(&t)));
    assert!(calls <= 15);
    let mut calls = 0;
    assert!(
        first_contact_with_floor_acceleration(0.0, 100.0, 0.0, -0.2, 5.0, -0.2, |_| {
            calls += 1;
            false
        })
        .is_none()
    );
    assert_eq!(calls, 15);
}

#[test]
fn floor_adjustment_preserves_fallback_wall_ceiling_and_platform_direction() {
    for acceleration in [0.0, f32::NAN, -2.0, 0.1] {
        let mut original = Vec::new();
        let mut adjusted = Vec::new();
        let a = first_contact_in(0.0, 3.0, 2.0, -0.2, 5.0, |ray| {
            original.push((ray.origin, ray.delta, ray.passable));
            false
        });
        let b =
            first_contact_with_floor_acceleration(0.0, 3.0, 2.0, -0.2, 5.0, acceleration, |ray| {
                adjusted.push((ray.origin, ray.delta, ray.passable));
                false
            });
        assert_eq!(a, b);
        assert_eq!(original, adjusted);
    }
    // Ascending through a platform still cannot produce a floor-tech request.
    assert!(
        first_contact_with_floor_acceleration(0.0, 0.0, 0.0, 2.0, 5.0, -0.2, |ray| ray.passable)
            .is_none()
    );
    for vy in [-0.2, 2.0] {
        let mut old_solid = Vec::new();
        let mut new_solid = Vec::new();
        first_contact_in(0.0, 20.0, 2.0, vy, 5.0, |r| {
            if !r.passable {
                old_solid.push((r.origin, r.delta));
            }
            false
        });
        first_contact_with_floor_acceleration(0.0, 20.0, 2.0, vy, 5.0, -0.2, |r| {
            if !r.passable {
                new_solid.push((r.origin, r.delta));
            }
            false
        });
        assert_eq!(old_solid, new_solid);
    }
}

#[test]
fn floor_acceleration_requires_recent_matching_native_progress() {
    let sample = FloorMotionSample {
        object: 1,
        status: 0x49,
        motion: 1,
        frame: 1.0,
        rate: 1.0,
        vy: 0.0,
    };
    let next = FloorMotionSample {
        frame: 2.0,
        vy: -0.2,
        ..sample
    };
    let mut history = FloorMotionHistory::default();
    assert_eq!(history.sample(sample), 0.0);
    assert!((history.sample(next) + 0.2).abs() < 0.001);
    for _ in 0..20 {
        assert!((history.sample(next) + 0.2).abs() < 0.001);
    }
    for bad in [
        FloorMotionSample { object: 2, ..next },
        FloorMotionSample {
            status: 0x4a,
            ..next
        },
        FloorMotionSample { motion: 2, ..next },
        FloorMotionSample {
            frame: 20.0,
            ..next
        },
        FloorMotionSample { frame: 0.0, ..next },
        FloorMotionSample { rate: 0.0, ..next },
        FloorMotionSample {
            vy: f32::NAN,
            ..next
        },
        FloorMotionSample { vy: -20.0, ..next },
    ] {
        let mut history = FloorMotionHistory::default();
        history.sample(sample);
        assert_eq!(history.sample(bad), 0.0);
    }
}

#[test]
fn slow_eligible_approaches_still_probe_floor_wall_and_ceiling() {
    for (x,y,vx,vy,axis,plane) in [
        (0.,0.4,0.,-0.02,1,0.),
        (-2.55,0.,0.02,0.,0,0.),
        (0.,-6.95,0.,0.02,1,0.),
    ] {
        let mut calls=0;
        let contact=first_contact_in(x,y,vx,vy,5.,|ray|{calls+=1;crosses_plane(&ray,axis,plane)});
        assert!(contact.is_some(),"missed slow velocity ({vx},{vy})");
        assert!(calls<=54,"existing nine-ray/refinement ceiling must hold");
    }
    assert!(tech_probes(0.,0.,0.,0.,5.).iter().all(Option::is_none));
    assert!(tech_probes(0.,0.,0.,0.02,5.).iter().flatten().all(|p|!p.passable));
}
#[test]
fn accelerating_from_zero_vertical_speed_can_reach_floor_without_extra_probe_slots() {
    let mut calls=0;
    let contact=first_contact_with_floor_acceleration(0.,0.8,0.,0.,5.,-0.1,|ray|{
        calls+=1;assert!(ray.passable);assert!(ray.delta[1]<0.);crosses_plane(&ray,1,0.)
    });
    assert!(contact.is_some());assert!(calls<=15);
}

#[test]
fn imminent_narrow_platform_contact_is_not_vetoed_by_a_long_diagonal_ray() {
    let mut calls=0;
    let contact=first_contact_in(0.,2.,4.,-0.5,5.,|ray|{
        calls+=1;
        if !ray.passable{return false;}
        let t=(1.5-ray.origin[1])/ray.delta[1];
        let x=ray.origin[0]+t*ray.delta[0];
        (0. ..=1.).contains(&t)&&(x-3.).abs()<0.01
    });
    assert_eq!(contact,Some(1.));assert!(calls<=54);
}

#[test]
fn late_hitstop_uses_finite_reserved_launch_instead_of_frozen_motion() {
    assert_eq!(prediction_velocity([0.;2],Some([0.,-12.])),[0.,-12.]);
    assert_eq!(prediction_velocity([2.,1.],Some([-4.,-3.])),[-4.,-3.]);
    assert_eq!(prediction_velocity([2.,1.],Some([f32::NAN,-3.])),[2.,1.]);
    assert_eq!(prediction_velocity([2.,1.],Some([0.;2])),[2.,1.]);
    let v=prediction_velocity([0.;2],Some([0.,-12.]));
    assert!(first_contact_in(0.,10.,v[0],v[1],5.,|r|crosses_plane(&r,1,0.)).is_some());
}
#[test]
fn slow_upward_tumble_predicts_floor_only_after_turning_downward() {
    let mut floor_calls=0;
    let contact=first_contact_with_floor_acceleration(0.,2.,1.,0.2,5.,-0.2,|ray| {
        if !ray.passable {return false;}
        floor_calls+=1;
        // Apex at t=1: each floor segment begins at/after it, never while rising.
        assert!(ray.origin[0]>=1.-Body::default().half_width);
        assert!(ray.delta[1]<0.);
        crosses_plane(&ray,1,0.)
    });
    assert!(contact.is_some());assert!(floor_calls<=15);
    let mut ascending_floor_queries=0;
    assert!(first_contact_with_floor_acceleration(0.,2.,0.,2.,5.,-0.2,|r| {
        if r.passable {ascending_floor_queries+=1;}false
    }).is_none());
    assert_eq!(ascending_floor_queries,0);
}
