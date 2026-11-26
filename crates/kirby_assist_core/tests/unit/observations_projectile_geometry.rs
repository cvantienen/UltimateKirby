mod tests {
    use super::*;

    fn hit(start: [f32; 3], end: [f32; 3], velocity: [f32; 2]) -> Hitbox {
        Hitbox {
            start,
            end,
            radius: 1.,
            velocity,
        }
    }

    #[test]
    fn diagonal_bounds_are_not_capsule_contact() {
        let diagonal = hit([-10., -5., 0.], [5., 10., 0.], [0.; 2]);
        assert_eq!(
            swept_capsule_aabb(diagonal, [-1.; 3], [1.; 3], [0.; 2], 10.),
            None
        );
        let crossing = hit([-10., -10., 0.], [10., 10., 0.], [0.; 2]);
        assert_eq!(
            swept_capsule_aabb(crossing, [-1.; 3], [1.; 3], [0.; 2], 0.),
            Some(0.)
        );
    }

    #[test]
    fn earliest_contact_uses_relative_motion_and_rounded_corners() {
        let sphere = hit([5., 2., 0.], [5., 2., 0.], [-1., 0.]);
        let t = swept_capsule_aabb(sphere, [-1.; 3], [1.; 3], [0.; 2], 8.).unwrap();
        assert!((t - 4.).abs() < 0.0001, "{t}");
        let sphere = hit([10., 0., 0.], [10., 0., 0.], [-2., 0.]);
        let t = swept_capsule_aabb(sphere, [-1.; 3], [1.; 3], [2., 0.], 3.).unwrap();
        assert!((t - 2.).abs() < 0.0001, "{t}");
        assert_eq!(
            swept_capsule_aabb(sphere, [-1.; 3], [1.; 3], [2., 0.], 1.),
            None
        );
    }

    #[test]
    fn fast_crossing_and_single_instant_tangency_are_not_skipped() {
        let sphere = hit([-1000., 0., 0.], [-1000., 0., 0.], [10000., 0.]);
        let t = swept_capsule_aabb(sphere, [-1.; 3], [1.; 3], [0.; 2], 1.).unwrap();
        assert!((t - 0.0998).abs() < 0.000001, "{t}");
        let tangent = hit([-5., 1., 0.], [-5., 1., 0.], [2., 0.]);
        let t = swept_capsule_aabb(tangent, [0.; 3], [0.; 3], [0.; 2], 5.).unwrap();
        assert!((t - 2.5).abs() < 0.0001, "{t}");
    }

    #[test]
    fn moving_diagonal_uses_segment_interior_and_depth() {
        let diagonal = hit([-10., 5., 0.], [5., 20., 0.], [0., -1.]);
        let t = swept_capsule_aabb(diagonal, [-1.; 3], [1.; 3], [0.; 2], 20.).unwrap();
        let expected = 13. - 2.0_f32.sqrt();
        assert!((t - expected).abs() < 0.0001, "{t}");
        assert_eq!(
            swept_capsule_aabb(diagonal, [-1.; 3], [1.; 3], [0.; 2], 10.),
            None
        );
        let off_plane = Hitbox {
            start: [-10., 5., 3.],
            end: [5., 20., 3.],
            ..diagonal
        };
        assert_eq!(
            swept_capsule_aabb(off_plane, [-1.; 3], [1.; 3], [0.; 2], 20.),
            None
        );
    }

    #[test]
    fn contact_time_precision_covers_gameplay_horizons_and_tangencies() {
        for time in [0.01_f32, 1., 7.12345, 39.9, 119.9] {
            for speed in [0.01_f32, 1., 10000.] {
                let x = 2. + speed * time;
                let expected = (x - 2.) / speed;
                let sphere = hit([x, 0., 0.], [x, 0., 0.], [-speed, 0.]);
                let contact = swept_capsule_aabb(sphere, [-1.; 3], [1.; 3], [0.; 2], 120.).unwrap();
                assert!(
                    (contact - expected).abs() <= 0.00003,
                    "time={time} speed={speed}: {contact} vs {expected}"
                );
            }
        }
        // A tiny radius makes missing an isolated tangent easier than
        // missing an ordinary crossing; its center reaches the point at t=40.
        let tangent = Hitbox {
            radius: 0.001,
            ..hit([-40., 0.001, 0.], [-40., 0.001, 0.], [1., 0.])
        };
        let contact = swept_capsule_aabb(tangent, [0.; 3], [0.; 3], [0.; 2], 120.).unwrap();
        assert!((contact - 40.).abs() <= 0.00003, "{contact}");
    }

    #[test]
    fn estimated_body_dimensions_change_admission_and_contact_time() {
        let overhead = hit([0., 12., 0.], [0., 12., 0.], [0.; 2]);
        let small = ([-2., 0., -2.], [2., 6., 2.]);
        let large = ([-6., 0., -6.], [6., 18., 6.]);
        assert_eq!(
            swept_capsule_aabb(overhead, small.0, small.1, [0.; 2], 10.),
            None
        );
        assert_eq!(
            swept_capsule_aabb(overhead, large.0, large.1, [0.; 2], 10.),
            Some(0.)
        );
        let incoming = hit([20., 3., 0.], [20., 3., 0.], [-2., 0.]);
        let small_contact = swept_capsule_aabb(incoming, small.0, small.1, [0.; 2], 10.).unwrap();
        let large_contact = swept_capsule_aabb(incoming, large.0, large.1, [0.; 2], 10.).unwrap();
        assert!((small_contact - 8.5).abs() < 0.00003);
        assert!((large_contact - 6.5).abs() < 0.00003);
    }

    #[test]
    fn malformed_geometry_is_unavailable() {
        let valid = hit([0.; 3], [0.; 3], [0.; 2]);
        for horizon in [f32::NAN, f32::INFINITY, -1.] {
            assert_eq!(
                swept_capsule_aabb(valid, [-1.; 3], [1.; 3], [0.; 2], horizon),
                None
            );
        }
        for radius in [0., -1., f32::NAN, f32::INFINITY] {
            assert_eq!(
                swept_capsule_aabb(Hitbox { radius, ..valid }, [-1.; 3], [1.; 3], [0.; 2], 1.),
                None
            );
        }
        assert_eq!(
            swept_capsule_aabb(valid, [2.; 3], [1.; 3], [0.; 2], 1.),
            None
        );
        assert_eq!(
            swept_capsule_aabb(valid, [-1.; 3], [1.; 3], [f32::NAN, 0.], 1.),
            None
        );
        assert_eq!(
            swept_capsule_aabb(
                Hitbox {
                    end: [f32::INFINITY; 3],
                    ..valid
                },
                [-1.; 3],
                [1.; 3],
                [0.; 2],
                1.
            ),
            None
        );
    }
}
