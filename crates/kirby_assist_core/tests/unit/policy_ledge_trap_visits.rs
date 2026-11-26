use super::*;
use crate::ledge_trap::{
    tests::{context, enemy},
    CLIFF_ATTACK, CLIFF_CLIMB, CLIFF_WAIT,
};
use crate::{
    fighter_profiles::presets::PresetSlot, native_input::MAPPER_A, sequence_config::Catalog,
};
/// Returns the current settings view.
fn settings() -> AssistSettings {
    let c = Catalog::parse("[kirby.neutral.test]\nstart_delay=0\nactions=a:1").unwrap();
    let mut s = AssistSettings {
        enabled: true,
        ..Default::default()
    };
    s.sequence_overrides[PresetSlot::Neutral as usize] = Some(c.entries()[0].sequence);
    s
}
/// Returns the `ctx` result.
fn ctx(frame: f32) -> GameplayContext {
    GameplayContext {
        motion_kind: 123,
        motion_frame: frame,
        ..context()
    }
}
/// Regression test for `completed_visit_stays_consumed_after_release_reset_and_sticky_previous_status`.
#[test]
fn completed_visit_stays_consumed_after_release_reset_and_sticky_previous_status() {
    let mut v = LedgeVisits::default();
    v.set_owner(1, 6);
    let mut trap = LedgeTrap::default();
    let getup = Opponent {
        previous_status: CLIFF_WAIT,
        ..enemy(CLIFF_CLIMB, true)
    };
    assert_eq!(
        trap.sample_with_visits(&mut v, settings(), 0, ctx(0.0), Some(getup))
            .requested,
        MAPPER_A
    );
    trap.sample_with_visits(&mut v, settings(), 1, ctx(1.0), Some(getup));
    assert!(trap.complete);
    let standing = Opponent {
        previous_status: CLIFF_CLIMB,
        ..enemy(0, true)
    };
    trap.sample_with_visits(
        &mut v,
        settings(),
        2,
        GameplayContext {
            held: false,
            ..ctx(2.0)
        },
        Some(standing),
    );
    trap = LedgeTrap::default();
    for frame in 3..100 {
        assert_eq!(
            trap.sample_with_visits(&mut v, settings(), frame, ctx(frame as f32), Some(standing))
                .requested,
            0
        );
        trap = LedgeTrap::default(); // Also model another assist taking ownership.
    }
}
/// Regression test for `interrupting_an_active_response_does_not_restart_it_and_new_grab_rearms`.
#[test]
fn interrupting_an_active_response_does_not_restart_it_and_new_grab_rearms() {
    let mut v = LedgeVisits::default();
    let mut t = LedgeTrap::default();
    let getup = Opponent {
        previous_status: CLIFF_WAIT,
        ..enemy(CLIFF_CLIMB, true)
    };
    assert_eq!(
        t.sample_with_visits(&mut v, settings(), 0, ctx(0.0), Some(getup))
            .requested,
        MAPPER_A
    );
    t.sample_with_visits(
        &mut v,
        settings(),
        1,
        GameplayContext {
            held: false,
            ..ctx(1.0)
        },
        Some(getup),
    );
    assert_eq!(
        t.sample_with_visits(&mut v, settings(), 2, ctx(2.0), Some(getup))
            .requested,
        0
    );
    assert!(!t.reserves_response(Some(getup)));
    assert_eq!(t.action_name(), "await_new_ledge_visit");
    let hang = Opponent {
        cliff_position: Some([-10.0, 0.0]),
        ..enemy(CLIFF_WAIT, false)
    };
    t.sample_with_visits(&mut v, settings(), 3, ctx(3.0), Some(hang));
    assert_eq!(
        t.sample_with_visits(&mut v, settings(), 4, ctx(4.0), Some(getup))
            .requested,
        MAPPER_A
    );
}
/// Regression test for `standing_previous_getup_alone_cannot_create_a_visit`.
#[test]
fn standing_previous_getup_alone_cannot_create_a_visit() {
    let mut v = LedgeVisits::default();
    let mut t = LedgeTrap::default();
    let standing = Opponent {
        previous_status: CLIFF_CLIMB,
        ..enemy(0, true)
    };
    assert_eq!(
        t.sample_with_visits(&mut v, settings(), 0, ctx(0.0), Some(standing))
            .requested,
        0
    );
    assert!(t.file_timeline.is_none());
}
/// Regression test for `ledge_attack_seen_without_its_hang_is_still_a_response_visit`.
#[test]
fn ledge_attack_seen_without_its_hang_is_still_a_response_visit() {
    let c = Catalog::parse("[kirby.attack.test]\nstart_delay=0\nactions=y:1").unwrap();
    let mut s = settings();
    s.sequence_overrides[PresetSlot::Attack as usize] = Some(c.entries()[0].sequence);
    let attack = Opponent {
        previous_status: CLIFF_WAIT,
        ..enemy(CLIFF_ATTACK, true)
    };
    let mut v = LedgeVisits::default();
    let mut t = LedgeTrap::default();
    assert_eq!(
        t.sample_with_visits(&mut v, s, 0, ctx(0.0), Some(attack))
            .requested,
        crate::native_input::MAPPER_Y
    );
    // The same attack continuing, or following its own hang, is one visit.
    let epoch = v.visit_epoch();
    assert!(!v.observe(attack, [0.0; 2]).new_visit);
    assert_eq!(v.visit_epoch(), epoch);
    let mut seen_hang = LedgeVisits::default();
    seen_hang.observe(enemy(CLIFF_WAIT, false), [0.0; 2]);
    let epoch = seen_hang.visit_epoch();
    assert!(!seen_hang.observe(attack, [0.0; 2]).new_visit);
    assert_eq!(seen_hang.visit_epoch(), epoch);
}
/// Regression test for `walking_anchor_only_comes_from_a_real_observed_hang_and_survives_input_release`.
#[test]
fn walking_anchor_only_comes_from_a_real_observed_hang_and_survives_input_release() {
    let mut v = LedgeVisits::default();
    v.set_owner(1, 6);
    let stand = Opponent {
        cliff_position: Some([99.0, 20.0]),
        ..enemy(0, true)
    };
    v.observe(stand, [0.0; 2]);
    assert!(v.walking_ledge().is_none());
    let hang = Opponent {
        cliff_position: Some([0.0; 2]),
        ..enemy(CLIFF_WAIT, false)
    };
    v.observe(hang, [10.0, 0.0]);
    assert_eq!(
        v.walking_ledge(),
        Some(LedgeSpot {
            edge: [0.0; 2],
            inward: 1.0
        })
    );
    v.observe(hang, [-2.0, 0.0]); // Never flip the ledge's inward direction at the boundary.
    assert_eq!(v.walking_ledge().unwrap().inward, 1.0);
    v.observe(enemy(0, true), [10.0, 0.0]);
    assert_eq!(v.walking_ledge().unwrap().edge, [0.0; 2]);
    v.set_owner(1, 6);
    assert!(v.walking_ledge().is_some());
    v.set_owner(4, 5);
    assert!(v.walking_ledge().is_none());
}
#[test]
fn opponent_regrab_does_not_restart_committed_side_special_sequence() {
    let mut visits = LedgeVisits::default();
    let mut trap = LedgeTrap::default();
    let mut settings = settings();
    settings.sequence_overrides[PresetSlot::Neutral as usize] = Some(
        Catalog::parse(
            "[kirby.neutral.test]\nstart_delay=0\nactions=side_special:1,wait:2,side_special:1",
        )
        .unwrap()
        .entries()[0]
            .sequence,
    );
    let getup = enemy(CLIFF_CLIMB, true);
    let statuses = [
        getup,
        enemy(0, true),
        enemy(CLIFF_WAIT, false),
        getup,
        getup,
    ];
    for (tick, opponent) in statuses.into_iter().enumerate() {
        let out = trap.sample_with_visits(
            &mut visits,
            settings,
            tick as i64,
            ctx(tick as f32),
            Some(opponent),
        );
        assert_eq!(
            out.requested,
            if tick == 0 || tick == 3 {
                crate::native_input::MAPPER_B
            } else {
                0
            }
        );
    }
    assert!(trap.complete);
}
/// Regression test for `completed_engine_rearms_for_next_observed_hang_without_releasing_l`.
#[test]
fn completed_engine_rearms_for_next_observed_hang_without_releasing_l() {
    let mut v = LedgeVisits::default();
    let mut t = LedgeTrap::default();
    for base in [0, 10] {
        let hang = enemy(CLIFF_WAIT, false);
        t.sample_with_visits(&mut v, settings(), base, ctx(base as f32), Some(hang));
        let getup = Opponent {
            previous_status: CLIFF_WAIT,
            ..enemy(CLIFF_CLIMB, true)
        };
        assert_eq!(
            t.sample_with_visits(
                &mut v,
                settings(),
                base + 1,
                ctx((base + 1) as f32),
                Some(getup)
            )
            .requested,
            MAPPER_A
        );
        t.sample_with_visits(
            &mut v,
            settings(),
            base + 2,
            ctx((base + 2) as f32),
            Some(getup),
        );
        assert!(t.complete);
    }
}
