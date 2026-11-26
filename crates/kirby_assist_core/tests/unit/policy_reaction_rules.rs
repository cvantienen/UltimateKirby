use super::*;
use crate::{
    fighter_profiles::{presets::PresetSlot, FighterProfiles},
    native_input::*,
    sequence_config::{persist, Catalog},
};
/// Returns the `sequence` result.
fn sequence(trigger: &str, actions: &str) -> Sequence {
    Catalog::parse(&format!(
        "[yoshi.roll.rule]\nstart_delay=0\nreaction={trigger}\nactions={actions}\n"
    ))
    .unwrap()
    .entries()[0]
        .sequence
}
/// Returns the `context` result.
fn context(frame: f32) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        status: crate::native_states::status::WAIT,
        situation: crate::native_states::situation::GROUND,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.,
        facing: 1.,
        position: [0., 0.],
        safe_ground: [true; 2],
        ..GameplayContext::default()
    }
}
/// Returns the selected target observation.
fn target(frame: f32, x: f32) -> Target {
    Target {
        id: 7,
        status: ESCAPE_F,
        motion: 456,
        frame,
        rate: 1.,
        position: [x, 0.],
        velocity_x: 0.,
        vulnerable_in: Some(0.),
        actionable_in: Some(5.),
    }
}
/// Regression test for `event_identity_requires_matching_motion_status_and_nonrewound_frame`.
#[test]
fn event_identity_requires_matching_motion_status_and_nonrewound_frame() {
    let target = target(5., 1.);
    let track = Track {
        id: target.id,
        status: target.status,
        motion: target.motion,
        frame: target.frame,
        dx: 1.,
        crossed: false,
        used: true,
    };
    assert!(track.continues(&target));
    assert!(track.continues(&Target {
        frame: 6.,
        ..target
    }));
    assert!(!track.continues(&Target {
        frame: 4.,
        ..target
    }));
    assert!(!track.continues(&Target {
        frame: f32::NAN,
        ..target
    }));
    assert!(!track.continues(&Target { id: 8, ..target }));
    assert!(!track.continues(&Target {
        status: ESCAPE_B,
        ..target
    }));
    assert!(!track.continues(&Target {
        motion: 457,
        ..target
    }));
}

/// Regression test for `simple_tracking_retains_consumed_missing_actor_until_status_changes`.
#[test]
fn simple_tracking_retains_consumed_missing_actor_until_status_changes() {
    let mut engine = Engine::default();
    let first = target(5., 1.);
    let mut targets = [None; 8];
    targets[0] = Some(first);
    engine.observe_simple_events(&targets);
    engine.tracks[0].as_mut().unwrap().used = true;

    targets[0] = Some(Target { id: 8, ..first });
    engine.observe_simple_events(&targets);
    assert!(engine
        .tracks
        .iter()
        .flatten()
        .any(|t| t.id == first.id && t.used));

    targets[1] = Some(Target { frame: 6., ..first });
    engine.observe_simple_events(&targets);
    assert!(engine.tracks[1].unwrap().used);

    targets[1] = Some(Target { frame: 1., ..first });
    engine.observe_simple_events(&targets);
    assert!(!engine.tracks[1].unwrap().used);

    engine.tracks[1].as_mut().unwrap().used = true;
    targets[1] = Some(Target { status: 0, ..first });
    engine.observe_simple_events(&targets);
    assert!(!engine.tracks.iter().flatten().any(|t| t.id == first.id));
}
/// Returns the `resolved` result.
fn resolved(s: Sequence) -> Resolved {
    let mut r = Resolved::default();
    r.add(Candidate {
        slot: 0,
        index: 0,
        sequence: s,
    })
    .unwrap();
    r
}
/// Updates the toggle state from the current button sample and match state.
fn sample(e: &mut Engine, r: &Resolved, c: GameplayContext, t: Target) -> ControllerOutput {
    e.sample(
        true,
        r,
        AssistSettings::default(),
        c,
        [Some(t), None, None, None, None, None, None, None],
        None,
    )
}
/// Regression test for `roll_sequence_in_static_shield_uses_the_real_opponent_clock_from_its_first_sample`.
#[test]
fn roll_sequence_in_static_shield_uses_the_real_opponent_clock_from_its_first_sample() {
    for repeats in [0, 20] {
        let rules = resolved(sequence("roll:shielding:0:20:12:any:1", "wait:2,up_b:1"));
        let mut engine = Engine::default();
        let c = GameplayContext {
            status: crate::native_states::GUARD,
            motion_frame: 0.0,
            motion_rate: 1.0,
            ..context(0.0)
        };
        // Direct Up-B retains guard through its native shield cancel.
        for (frame, expected) in [0, 0, MAPPER_B | MAPPER_ZR, 0].into_iter().enumerate() {
            let t = target(frame as f32, 5.0);
            let following = Opponent {
                id: t.id,
                status: t.status,
                position: t.position,
                motion: Some(crate::gameplay::MotionTiming {
                    kind: t.motion,
                    frame: t.frame,
                    rate: t.rate,
                    remaining: t.actionable_in,
                }),
                ..Opponent::default()
            };
            let targets = [Some(t), None, None, None, None, None, None, None];
            // Discovery does not yet have a locked follow ID. The target
            // snapshot itself must seed this first frame's native clock.
            let follow = (frame > 0).then_some(following);
            let out = engine.sample(true, &rules, AssistSettings::default(), c, targets, follow);
            assert_eq!(out.requested, expected, "native frame {frame}");
            for _ in 0..repeats {
                let duplicate = engine.sample(
                    true,
                    &rules,
                    AssistSettings::default(),
                    c,
                    targets,
                    Some(following),
                );
                assert_eq!(duplicate.requested, out.requested);
            }
        }
    }
}
#[test]
fn new_spatial_roll_response_replaces_unrelated_follow_but_keeps_matching_live_facts() {
    let rules = resolved(sequence("roll:any:0:20:12:any:1", "face:opponent,wait:2"));
    let selected = target(0., 5.);
    for id in [selected.id, 99] {
        let mut engine = Engine::default();
        let live = Opponent {
            id,
            position: [-5., 0.],
            ..Opponent::default()
        };
        let out = engine.sample(
            true,
            &rules,
            AssistSettings::default(),
            context(0.),
            [Some(selected), None, None, None, None, None, None, None],
            Some(live),
        );
        assert!(engine.committed());
        assert_eq!(engine.target(), Some(selected.id));
        assert_ne!(engine.feedback, Feedback::MissingTarget);
        assert_eq!(
            out.stick,
            Some(if id == selected.id {
                [-0.3, 0.]
            } else {
                [0.; 2]
            })
        );
    }
}

/// Regression test for `different_native_roll_motion_does_not_inherit_the_previous_rolls_used_flag`.
#[test]
fn different_native_roll_motion_does_not_inherit_the_previous_rolls_used_flag() {
    let rules = resolved(sequence("roll:any:0:20:12:any:1", "up_b:1"));
    let mut engine = Engine::default();
    assert_eq!(
        sample(&mut engine, &rules, context(0.0), target(0.0, 5.0)).requested,
        MAPPER_B
    );
    sample(&mut engine, &rules, context(1.0), target(1.0, 5.0));
    let t = Target {
        motion: 789,
        ..target(2.0, 5.0)
    };
    assert_eq!(
        sample(&mut engine, &rules, context(2.0), t).requested,
        MAPPER_B
    );
}
#[test]
fn recovery_rolls_preserve_crossing_and_vulnerability_checks() {
    use crate::native_states::status;
    for status in [
        ESCAPE_F,
        ESCAPE_B,
        status::PASSIVE_FB,
        status::DOWN_STAND_FB,
    ] {
        let rules = resolved(sequence("roll_cross_vulnerable:any:0:20:12:any:1", "jab:1"));
        let mut engine = Engine::default();
        let first = Target {
            status,
            vulnerable_in: Some(2.),
            ..target(0., 5.)
        };
        assert_eq!(sample(&mut engine, &rules, context(0.), first).requested, 0);
        assert_eq!(engine.feedback, Feedback::Crossing);
        let crossed = Target {
            frame: 1.,
            position: [-5., 0.],
            ..first
        };
        assert_eq!(
            sample(&mut engine, &rules, context(1.), crossed).requested,
            0
        );
        assert_eq!(engine.feedback, Feedback::Invulnerable);
        let vulnerable = Target {
            frame: 2.,
            vulnerable_in: Some(0.),
            ..crossed
        };
        assert_eq!(
            sample(&mut engine, &rules, context(2.), vulnerable).requested,
            MAPPER_A
        );
        assert_eq!(
            sample(
                &mut engine,
                &rules,
                context(3.),
                Target {
                    frame: 3.,
                    ..vulnerable
                }
            )
            .requested,
            0
        );
    }
    let rules = resolved(sequence("roll:any:0:20:12:any:1", "jab:1"));
    for status in [
        status::PASSIVE,
        status::DOWN_STAND,
        status::DOWN_STAND_ATTACK,
    ] {
        assert_eq!(
            sample(
                &mut Engine::default(),
                &rules,
                context(0.),
                Target {
                    status,
                    ..target(0., 5.)
                }
            )
            .requested,
            0
        );
    }
}

#[test]
fn automatic_recovery_rolls_rearm_without_confusing_getup_attacks() {
    use crate::native_states::status;
    let rules = resolved(sequence("auto_roll:20:12", "jab:1"));
    let mut engine = Engine::default();
    for (frame, status, expected) in [
        (0., status::PASSIVE_FB, MAPPER_A),
        (1., status::PASSIVE_FB, 0),
        (2., status::DOWN_STAND_ATTACK, 0),
        (3., status::DOWN_STAND_FB, MAPPER_A),
        (4., status::DOWN_STAND_FB, 0),
        (5., status::PASSIVE, 0),
        (6., status::DOWN_STAND, 0),
    ] {
        let target = Target {
            status,
            ..target(frame, 5.)
        };
        let targets = [Some(target), None, None, None, None, None, None, None];
        let readiness = simple_readiness(rules.simple_starts(), context(frame), &targets, || false);
        assert_eq!(
            readiness[0],
            matches!(status, status::PASSIVE_FB | status::DOWN_STAND_FB)
        );
        assert_eq!(
            engine
                .sample_simple_ready(
                    true,
                    &rules,
                    AssistSettings::default(),
                    context(frame),
                    targets,
                    None,
                    readiness
                )
                .requested,
            expected
        );
    }
}

/// Regression test for `reaction_persists_roundtrip_and_disabling_replaces_old_metadata`.
#[test]
fn reaction_persists_roundtrip_and_disabling_replaces_old_metadata() {
    let src="[yoshi.roll.rule]\nstart_delay=0\nactions=jab:4\n[kirby.roll.keep]\nstart_delay=0\nactions=grab\n";
    let seq = sequence("roll:grounded:2:20:12:front:2", "face:opponent,grab");
    let saved = persist::update(src, 5, PresetSlot::Roll, "rule", seq, false).unwrap();
    assert_eq!(Catalog::parse(&saved).unwrap().entries()[0].sequence, seq);
    let mut disabled = seq;
    disabled.reaction = Rule::default();
    let off = persist::update(&saved, 5, PresetSlot::Roll, "rule", disabled, false).unwrap();
    assert!(!off.contains("reaction ="));
    assert!(off.ends_with("[kirby.roll.keep]\nstart_delay=0\nactions=grab\n"));
}
/// Regression test for `invalid_rules_and_more_than_eight_are_rejected`.
#[test]
fn invalid_rules_and_more_than_eight_are_rejected() {
    for s in [
        "roll:any:20:10:12:any:1",
        "roll:any:0:NaN:12:any:1",
        "roll:any:0:20:12:any:0",
        "roll:wat:0:20:12:any:1",
    ] {
        assert!(Rule::parse(s).is_err());
    }
    let mut src = String::new();
    for i in 0..9 {
        src.push_str(&format!(
            "[yoshi.roll.r{i}]\nstart_delay=0\nreaction=roll:any:0:20:12:any:1\nactions=jab:4\n"
        ));
    }
    assert!(Catalog::parse(&src)
        .unwrap()
        .validate(&FighterProfiles::default())
        .is_err());
    assert!(validate_response(sequence("roll:any:0:20:12:any:1", "walk:ledge:8:50%")).is_err());
}
/// Regression test for `one_attempt_per_roll_and_new_roll_rearms`.
#[test]
fn scoped_regression_roll_dispatch_ignores_automatic_event_recipes() {
    for trigger in Trigger::AUTO_EVENTS {
        let mut r = resolved(sequence(&format!("{}:20:12", trigger.token()), "a:1"));
        let mut e = Engine::default();
        assert_eq!(sample(&mut e, &r, context(0.), target(0., 5.)).requested, 0);
        assert!(!e.committed());
        r.add(Candidate {
            slot: 0,
            index: 1,
            sequence: sequence("roll:any:0:20:12:any:2", "up_b:1"),
        })
        .unwrap();
        assert_eq!(
            sample(&mut e, &r, context(1.), target(1., 5.)).requested,
            MAPPER_B
        );
    }
}

#[test]
fn one_attempt_per_roll_and_new_roll_rearms() {
    let r = resolved(sequence("roll:any:0:20:12:any:1", "jab:1"));
    let mut e = Engine::default();
    assert_eq!(
        sample(&mut e, &r, context(1.), target(1., 5.)).requested,
        MAPPER_A
    );
    sample(&mut e, &r, context(2.), target(2., 5.));
    assert_eq!(sample(&mut e, &r, context(3.), target(3., 5.)).requested, 0);
    assert_eq!(
        sample(&mut e, &r, context(4.), target(0., 5.)).requested,
        MAPPER_A
    );
}
/// Regression test for `crossing_survives_exact_zero_and_priority_chooses_the_first_match`.
#[test]
fn crossing_survives_exact_zero_and_priority_chooses_the_first_match() {
    let r = resolved(sequence("roll_cross:any:0:20:12:any:1", "down_smash:4"));
    let mut e = Engine::default();
    assert_eq!(sample(&mut e, &r, context(1.), target(1., 4.)).requested, 0);
    sample(&mut e, &r, context(2.), target(2., 0.));
    assert_eq!(
        sample(&mut e, &r, context(3.), target(3., -3.)).requested,
        MAPPER_A
    );
    let mut r = resolved(sequence("roll:any:0:20:12:any:5", "jab:4"));
    r.add(Candidate {
        slot: 0,
        index: 1,
        sequence: sequence("roll:any:0:20:12:any:1", "up_b:4"),
    })
    .unwrap();
    assert_eq!(
        sample(&mut Engine::default(), &r, context(1.), target(1., 5.)).requested,
        MAPPER_B
    );
}
/// Regression test for `filters_wait_for_native_vulnerability_and_cancel_never_replays`.
#[test]
fn filters_wait_for_native_vulnerability_and_cancel_never_replays() {
    let r = resolved(sequence("roll_vulnerable:walking:3:10:5:front:1", "jab:4"));
    let mut e = Engine::default();
    let mut c = context(1.);
    assert_eq!(sample(&mut e, &r, c, target(1., 5.)).requested, 0);
    assert_eq!(e.feedback, Feedback::State);
    c.status = crate::native_states::status::WALK;
    c.motion_frame = 2.;
    let mut t = target(2., 5.);
    t.vulnerable_in = Some(2.);
    assert_eq!(sample(&mut e, &r, c, t).requested, 0);
    assert_eq!(e.feedback, Feedback::Invulnerable);
    c.motion_frame = 3.;
    assert_eq!(sample(&mut e, &r, c, target(3., 5.)).requested, MAPPER_A);
    e.cancel();
    c.motion_frame = 4.;
    assert_eq!(sample(&mut e, &r, c, target(4., 5.)).requested, 0);
}
/// Regression test for `spatial_response_requires_fresh_target_and_no_new_plan_starts_in_stop`.
#[test]
fn spatial_response_requires_fresh_target_and_no_new_plan_starts_in_stop() {
    let r = resolved(sequence(
        "roll:any:0:20:12:any:1",
        "face:opponent,wait:5,face:opponent",
    ));
    let mut e = Engine::default();
    let mut c = context(1.);
    c.stopped = true;
    assert_eq!(sample(&mut e, &r, c, target(1., 5.)).requested, 0);
    assert!(!e.committed());
    c.stopped = false;
    sample(&mut e, &r, c, target(1., 5.));
    assert!(e.committed());
    c.motion_frame = 2.;
    sample(&mut e, &r, c, target(2., 5.));
    assert!(!e.committed());
    assert_eq!(e.feedback, Feedback::MissingTarget);
}
