use super::{copied_target, target_within_range, AutomaticConfig, Context, OutOfShield};
use crate::{
    gameplay::{GameplayContext, MotionTiming, Opponent},
    native_input::MAPPER_ZR,
    roll_read::Target as ObservedTarget,
    settings::AssistSettings,
};

#[test]
fn scoped_regression_shield_tail_requires_native_progress() {
    let mut oos = OutOfShield::new();
    let mut c = Context {
        object: 1,
        kind: crate::fighter_kinds::LUCINA,
        status: crate::native_states::GUARD,
        motion: 1,
        frame: 0.,
        rate: 0.,
        stopped: false,
        grounded: true,
        position: [0.; 2],
        facing: 1.,
        shield_ready: true,
        can_grab: true,
        can_jump: true,
        can_up_special: true,
        threatened: true,
    };
    let mut lc = GameplayContext {
        gameplay: true,
        situation: crate::native_states::situation::GROUND,
        ..Default::default()
    };
    let enemy = |frame| {
        Some(Opponent {
            id: 2,
            status: crate::native_states::status::ATTACK,
            motion: Some(MotionTiming {
                kind: 2,
                frame,
                rate: 1.,
                remaining: None,
            }),
            ..Default::default()
        })
    };
    let sample = |oos: &mut OutOfShield, c, lc, follow| {
        oos.sample_automatic_configured(
            true,
            c,
            lc,
            [None; 8],
            follow,
            true,
            10.,
            20.,
            None,
            AssistSettings::default(),
            AutomaticConfig::default(),
        )
    };
    assert_eq!(sample(&mut oos, c, lc, enemy(0.)).requested, MAPPER_ZR);
    c.threatened = false;
    for _ in 0..30 {
        assert_eq!(sample(&mut oos, c, lc, enemy(0.)).requested, MAPPER_ZR);
    }
    for frame in 1..=2 {
        assert_eq!(
            sample(&mut oos, c, lc, enemy(frame as f32)).requested,
            MAPPER_ZR
        );
    }
    for _ in 0..30 {
        assert_eq!(sample(&mut oos, c, lc, None).requested, MAPPER_ZR);
    }
    lc.stopped = true;
    assert_eq!(sample(&mut oos, c, lc, enemy(20.)).requested, MAPPER_ZR);
    lc.stopped = false;
    assert_eq!(sample(&mut oos, c, lc, enemy(20.)).requested, MAPPER_ZR);
    for frame in 21..24 {
        assert_eq!(
            sample(&mut oos, c, lc, enemy(frame as f32)).requested,
            MAPPER_ZR
        );
    }
    assert_eq!(sample(&mut oos, c, lc, enemy(24.)).owned, 0);
}

#[test]
fn threatened_shield_stays_held_without_a_punish_target() {
    let mut oos = OutOfShield::new();
    let context = Context {
        object: 1,
        kind: crate::fighter_kinds::LUCINA,
        status: crate::native_states::GUARD,
        motion: 1,
        frame: 1.,
        rate: 1.,
        stopped: false,
        grounded: true,
        position: [0.; 2],
        facing: 1.,
        shield_ready: true,
        can_grab: true,
        can_jump: true,
        can_up_special: true,
        threatened: true,
    };
    let output = oos.sample_automatic_configured(
        true,
        context,
        GameplayContext {
            gameplay: true,
            ..Default::default()
        },
        [None; 8],
        None,
        true,
        10.,
        20.,
        None,
        AssistSettings::default(),
        AutomaticConfig::default(),
    );
    assert_eq!(output.requested & MAPPER_ZR, MAPPER_ZR);
    assert_eq!(output.owned, crate::ledge_trap::SEQUENCE_BUTTONS);
    assert_eq!(output.stick, Some([0.; 2]));
    let mut safe = context;
    safe.threatened = false;
    for frame in 2..=6 {
        safe.frame = frame as f32;
        let output = oos.sample_automatic_configured(
            true,
            safe,
            GameplayContext {
                gameplay: true,
                ..Default::default()
            },
            [None; 8],
            None,
            true,
            10.,
            20.,
            None,
            AssistSettings::default(),
            AutomaticConfig::default(),
        );
        assert_eq!(output.requested & MAPPER_ZR, MAPPER_ZR);
    }
    safe.frame = 7.;
    let output = oos.sample_automatic_configured(
        true,
        safe,
        GameplayContext {
            gameplay: true,
            ..Default::default()
        },
        [None; 8],
        None,
        true,
        10.,
        20.,
        None,
        AssistSettings::default(),
        AutomaticConfig::default(),
    );
    assert_eq!(output.owned, 0);
    let mut stunned = context;
    stunned.status = crate::native_states::GUARD_DAMAGE;
    stunned.threatened = false;
    let output = oos.sample_automatic_configured(
        true,
        stunned,
        GameplayContext {
            gameplay: true,
            ..Default::default()
        },
        [None; 8],
        None,
        true,
        10.,
        20.,
        None,
        AssistSettings::default(),
        AutomaticConfig::default(),
    );
    assert_eq!(output.requested & MAPPER_ZR, MAPPER_ZR);
    let mut paused = context;
    paused.stopped = true;
    let output = oos.sample_automatic_configured(
        true,
        paused,
        GameplayContext {
            gameplay: true,
            ..Default::default()
        },
        [None; 8],
        None,
        true,
        10.,
        20.,
        None,
        AssistSettings::default(),
        AutomaticConfig::default(),
    );
    assert_eq!(output.requested & MAPPER_ZR, MAPPER_ZR);
}

/// Regression test for `target_range_is_inclusive_and_rejects_invalid_facts`.
#[test]
fn target_range_is_inclusive_and_rejects_invalid_facts() {
    assert!(target_within_range([0., 0.], [3., -2.], 3., 2.));
    assert!(!target_within_range([0., 0.], [3.01, 0.], 3., 2.));
    assert!(!target_within_range([0., 0.], [0., 0.], 0., 2.));
    assert!(!target_within_range([0., 0.], [0., 0.], 2., -1.));
    assert!(!target_within_range([f32::NAN, 0.], [0., 0.], 2., 2.));
}

/// Regression test for `copied_target_preserves_common_observation_facts`.
#[test]
fn copied_target_preserves_common_observation_facts() {
    let opponent = copied_target(&ObservedTarget {
        id: 7,
        status: 12,
        motion: 34,
        frame: 5.5,
        rate: 1.25,
        position: [2., -3.],
        velocity_x: 0.5,
        vulnerable_in: Some(0.),
        actionable_in: Some(4.),
    });

    assert_eq!(opponent.id, 7);
    assert_eq!(opponent.status, 12);
    assert_eq!(opponent.position, [2., -3.]);
    assert_eq!(opponent.velocity, [0.5, 0.]);
    assert!(opponent.vulnerable);
    assert_eq!(
        opponent.motion,
        Some(MotionTiming {
            kind: 34,
            frame: 5.5,
            rate: 1.25,
            remaining: None,
        })
    );
    assert_eq!(opponent.actionable_in, None);
}
