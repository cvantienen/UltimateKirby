use super::*;
use crate::ledge_trap::{
    tests::{context, enemy},
    CLIFF_CLIMB, CLIFF_WAIT,
};
use crate::{
    fighter_profiles::presets::PresetSlot, native_input::MAPPER_A, sequence_config::Catalog,
};
/// Regression test for `editor_previews_repeat_without_consuming_the_real_visit`.
#[test]
fn editor_previews_repeat_without_consuming_the_real_visit() {
    let c = Catalog::parse("[kirby.neutral.test]\nstart_delay=0\nactions=a:1").unwrap();
    let mut s = AssistSettings {
        enabled: true,
        ..Default::default()
    };
    s.sequence_overrides[0] = Some(c.entries()[0].sequence);
    let mut v = LedgeVisits::default();
    let e = Opponent {
        cliff_position: Some([-10.0, 0.0]),
        ..enemy(CLIFF_WAIT, false)
    };
    v.observe(e, [0.0; 2]);
    for tick in [0, 1] {
        let mut t = LedgeTrap::default();
        t.request_preview(PresetSlot::Neutral);
        let ctx = GameplayContext {
            motion_kind: 123,
            motion_frame: tick as f32,
            ..context()
        };
        assert_eq!(
            t.sample_with_visits(&mut v, s, tick, ctx, Some(e))
                .requested,
            MAPPER_A
        );
    }
    let mut t = LedgeTrap::default();
    let e = Opponent {
        previous_status: CLIFF_WAIT,
        ..enemy(CLIFF_CLIMB, true)
    };
    assert_eq!(
        t.sample_with_visits(
            &mut v,
            s,
            2,
            GameplayContext {
                motion_kind: 123,
                motion_frame: 2.0,
                ..context()
            },
            Some(e)
        )
        .requested,
        MAPPER_A
    );
}
/// Regression test for `opposite_ledge_replaces_inward_side_but_same_edge_motion_and_own_crossing_do_not`.
#[test]
fn opposite_ledge_replaces_inward_side_but_same_edge_motion_and_own_crossing_do_not() {
    let mut v = LedgeVisits::new();
    v.set_owner(1, 15);
    for (edge, inward) in [(80., -1.), (-80., 1.)] {
        let e = Opponent {
            cliff_position: Some([edge, 0.]),
            ..enemy(CLIFF_WAIT, false)
        };
        v.observe(e, [0., 0.]);
        let spot = v.walking_ledge().unwrap();
        assert_eq!(spot.inward, inward);
        for clearance in [-2., 0., 4., 8., 20.] {
            let position = [edge + inward * clearance, 0.];
            let away = [inward * 0.5, 0.];
            assert_eq!(
                crate::manual_controls::stop_walk_at_ledge(away, position, [0.; 2], spot, 8.),
                away
            );
            if clearance <= 8. {
                assert_eq!(
                    crate::manual_controls::stop_walk_at_ledge(
                        [-away[0], 0.],
                        position,
                        [0.; 2],
                        spot,
                        8.
                    ),
                    [0.; 2]
                );
            }
        }
        let moved = Opponent {
            id: 3,
            cliff_position: Some([edge + 0.5, 0.]),
            ..e
        };
        v.observe(moved, [edge - inward * 2., 0.]);
        assert_eq!(
            v.walking_ledge().unwrap().inward,
            inward,
            "same physical edge keeps direction even across opponents"
        );
    }
}
