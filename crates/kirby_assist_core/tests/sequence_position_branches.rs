//! Native-input scheduling, spatial ledge placement and zero-frame branches.
use kirby_assist_core::{
    gameplay::{ControllerOutput, GameplayContext, MotionTiming, Opponent},
    ledge_trap::{InputTimeline, LedgeSettings},
    native_input::{MAPPER_A, MAPPER_B, MAPPER_R, MAPPER_ZR},
    sequence_config::{persist, Action, Catalog, Sequence},
};
fn recipe(actions: &str) -> Sequence {
    Catalog::parse(&format!(
        "[bowser.attack.test]\nstart_delay=0\ntimeout=3600\nactions={actions}\n"
    ))
    .unwrap()
    .entries()[0]
        .sequence
}
fn own(frame: f32, x: f32, facing: f32) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        status: kirby_assist_core::native_states::status::WAIT,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.,
        position: [x, 0.],
        facing,
        ground_brake: Some(0.2),
        safe_ground: [true; 2],
        ..Default::default()
    }
}
fn enemy(frame: f32, x: f32) -> Opponent {
    Opponent {
        id: 2,
        status: kirby_assist_core::native_states::status::WAIT,
        position: [x, 0.],
        motion: Some(MotionTiming {
            kind: 2,
            frame,
            rate: 1.,
            remaining: None,
        }),
        ..Default::default()
    }
}
fn sample(
    t: &mut InputTimeline,
    c: GameplayContext,
    x: f32,
    anchor: Option<[f32; 2]>,
    inward: f32,
) -> ControllerOutput {
    t.sample(
        LedgeSettings::default(),
        c,
        Some(enemy(c.motion_frame, x)),
        anchor,
        inward,
    )
}
#[test]
fn satisfied_checks_and_both_facing_blocks_emit_their_action_in_the_same_observation() {
    for (actions, x, facing, button) in [
        ("face:away_from_opponent,input:b:0:1:4", 10., -1., MAPPER_B),
        (
            "check:enemy_within:12,range:12,input:a:0:0:4",
            10.,
            1.,
            MAPPER_A,
        ),
        (
            "if_facing_ledge,input:a:0:0:4,else_facing,input:b:0:0:4,end_facing",
            10.,
            1.,
            MAPPER_A,
        ),
        (
            "if_facing_ledge,input:a:0:0:4,else_facing,input:b:0:0:4,end_facing",
            10.,
            -1.,
            MAPPER_B,
        ),
        (
            "if_facing_opponent,input:a:0:0:4,else_facing,input:b:0:0:4,end_facing",
            10.,
            1.,
            MAPPER_A,
        ),
        (
            "if_facing_opponent,input:a:0:0:4,else_facing,input:b:0:0:4,end_facing",
            -10.,
            1.,
            MAPPER_B,
        ),
        (
            "if_back_to_opponent,input:a:0:0:4,end_facing,input:b:0:0:4",
            10.,
            1.,
            MAPPER_B,
        ),
    ] {
        let mut t = InputTimeline::new(recipe(actions), enemy(0., x));
        let out = sample(&mut t, own(0., 0., facing), x, Some([50., 0.]), 1.);
        assert_eq!(out.requested, button, "{actions}");
    }
}
#[test]
fn a_facing_step_only_waits_until_native_turnaround_and_preserves_the_following_hold() {
    let mut t = InputTimeline::new(
        recipe("face:away_from_opponent,input:b:0:1:4"),
        enemy(0., 10.),
    );
    let first = sample(&mut t, own(0., 0., 1.), 10., None, 0.);
    assert_eq!(first.stick, Some([-0.3, 0.]));
    assert_eq!(first.requested, 0);
    // A short polling gap did not miss a scheduled attack: Face only held turn.
    for frame in 2..6 {
        assert_eq!(
            sample(&mut t, own(frame as f32, 0., -1.), 10., None, 0.).requested,
            MAPPER_B
        );
    }
    assert_eq!(sample(&mut t, own(6., 0., -1.), 10., None, 0.).requested, 0);
    assert!(t.finished());
    let mut t = InputTimeline::new(
        recipe("face:away_from_opponent,input:a:0:0:1"),
        enemy(0., 10.),
    );
    sample(&mut t, own(0., 0., 1.), 10., None, 0.);
    assert_eq!(
        sample(&mut t, own(0., 0., -1.), 10., None, 0.).requested,
        MAPPER_A,
        "facing facts need no timer advancement"
    );
}
#[test]
fn failed_checks_keep_their_explicit_wait_and_passed_checks_do_not_catch_up_over_inputs() {
    let mut t = InputTimeline::new(
        recipe("check:enemy_within:12:8,input:a:0:0:3"),
        enemy(0., 30.),
    );
    assert_eq!(sample(&mut t, own(0., 0., 1.), 30., None, 0.).requested, 0);
    for frame in 3..6 {
        assert_eq!(
            sample(&mut t, own(frame as f32, 0., 1.), 10., None, 0.).requested,
            MAPPER_A
        );
    }
    let mut t = InputTimeline::new(
        recipe("check:enemy_within:12:2,input:a:0:0:3"),
        enemy(0., 30.),
    );
    for frame in 0..=2 {
        assert_eq!(
            sample(&mut t, own(frame as f32, 0., 1.), 30., None, 0.).requested,
            0
        );
    }
    assert!(t.finished());
    let mut t = InputTimeline::new(recipe("wait:1,input:a:0:0:1,input:b:0:0:1"), enemy(0., 10.));
    sample(&mut t, own(0., 0., 1.), 10., None, 0.);
    assert_eq!(sample(&mut t, own(3., 0., 1.), 10., None, 0.).requested, 0);
    assert!(t.finished(), "timed actions keep strict gap protection");
}
#[test]
fn otherwise_is_exclusive_even_when_the_first_branch_changes_native_facing() {
    let seq = recipe(
        "if_facing_opponent,input:a:0:0:2,else_facing,input:b:0:0:2,end_facing,input:r:0:0:1",
    );
    let mut t = InputTimeline::new(seq, enemy(0., 10.));
    assert_eq!(
        sample(&mut t, own(0., 0., 1.), 10., None, 0.).requested,
        MAPPER_A
    );
    assert_eq!(
        sample(&mut t, own(1., 0., -1.), 10., None, 0.).requested,
        MAPPER_A
    );
    assert_eq!(
        sample(&mut t, own(2., 0., -1.), 10., None, 0.).requested,
        MAPPER_R
    );
    let mut t = InputTimeline::new(seq, enemy(0., -10.));
    for frame in 0..2 {
        assert_eq!(
            sample(&mut t, own(frame as f32, 0., 1.), -10., None, 0.).requested,
            MAPPER_B
        );
    }
    assert_eq!(
        sample(&mut t, own(2., 0., 1.), -10., None, 0.).requested,
        MAPPER_R
    );
}
#[test]
fn opponent_branches_require_current_target_and_preserve_ledge_snapshot_semantics() {
    let mut t = InputTimeline::new(
        recipe("if_facing_opponent,input:a:0:0:1,end_facing"),
        enemy(0., 10.),
    );
    assert_eq!(
        t.sample(LedgeSettings::default(), own(0., 0., 1.), None, None, 0.)
            .requested,
        0
    );
    assert!(t.finished());
    let mut t = InputTimeline::new(
        recipe("wait:1,if_facing_opponent,input:a:0:0:1,else_facing,input:b:0:0:1,end_facing"),
        enemy(0., 10.),
    );
    sample(&mut t, own(0., 0., 1.), 10., None, 0.);
    assert_eq!(
        sample(&mut t, own(1., 0., 1.), -10., None, 0.).requested,
        MAPPER_B,
        "use current opponent side at branch entry"
    );
    let mut t = InputTimeline::new(
        recipe("wait:1,if_facing_ledge,input:a:0:0:1,end_facing"),
        enemy(0., 10.),
    );
    sample(&mut t, own(0., 0., 1.), 10., Some([50., 0.]), 1.);
    assert_eq!(
        sample(&mut t, own(1., 0., -1.), 10., Some([50., 0.]), 1.).requested,
        MAPPER_A,
        "existing ledge facing remains captured at sequence start"
    );
}
#[test]
fn run_to_ledge_uses_both_stage_sides_and_approaches_the_same_point_from_either_direction() {
    for side in [-1., 1.] {
        for start in [5., 30.] {
            let anchor = [side * 100., 0.];
            let goal = side * 88.;
            let x = side * (100. - start);
            let mut t = InputTimeline::new(
                recipe("run_to_ledge:12:1:1:0,input:a:0:0:1"),
                enemy(0., side * 100.),
            );
            assert!(t.needs_ground());
            assert!(!t.inputs_only());
            assert_eq!(
                sample(&mut t, own(0., x, 1.), side * 100., Some(anchor), -side).stick,
                Some([(goal - x).signum(), 0.])
            );
            assert_eq!(
                sample(&mut t, own(1., goal, 1.), side * 50., Some(anchor), -side).requested,
                MAPPER_A,
                "goal is locked to ledge, not moving opponent"
            );
        }
    }
}
#[test]
fn arrived_ledge_position_skips_shield_drop_and_brakes_only_when_configured() {
    let seq = recipe("run_to_ledge:12:1:1:6,up_b:4");
    let mut t = InputTimeline::new(seq, enemy(0., 100.));
    let c = GameplayContext {
        status: kirby_assist_core::native_states::GUARD,
        ..own(0., 88., 1.)
    };
    assert_eq!(
        sample(&mut t, c, 100., Some([100., 0.]), -1.).requested,
        MAPPER_B | MAPPER_ZR,
        "arrival is an instant no-op, not a shield-drop step"
    );
    let mut t = InputTimeline::new(recipe("run_to_ledge:12:1:1:6"), enemy(0., 100.));
    let out = sample(
        &mut t,
        GameplayContext {
            velocity: [2., 0.],
            ..own(0., 80., 1.)
        },
        100.,
        Some([100., 0.]),
        -1.,
    );
    assert_eq!(
        out.stick,
        Some([0.; 2]),
        "coast while native braking distance exceeds remaining gap"
    );
    let mut t = InputTimeline::new(recipe("run_to_ledge:12:1:1:0"), enemy(0., 100.));
    assert_eq!(
        sample(
            &mut t,
            GameplayContext {
                velocity: [2., 0.],
                ..own(0., 80., 1.)
            },
            100.,
            Some([100., 0.]),
            -1.
        )
        .stick,
        Some([1., 0.])
    );
}
#[test]
fn positioning_cancels_on_air_unsafe_floor_missing_ledge_or_timeout_without_followup() {
    let seq = recipe("run_to_ledge:12:1:1:6:2,input:b:0:0:1");
    for mode in 0..4 {
        let mut t = InputTimeline::new(seq, enemy(0., 100.));
        let mut c = own(0., 50., 1.);
        let mut anchor = Some([100., 0.]);
        match mode {
            0 => c.situation = 2,
            1 => c.safe_ground = [false; 2],
            2 => anchor = None,
            _ => {
                sample(&mut t, c, 100., anchor, -1.);
                c.motion_frame = 2.;
            }
        }
        let out = sample(&mut t, c, 100., anchor, -1.);
        assert_eq!(out.requested, 0);
        assert!(t.finished());
    }
    let mut bad = seq;
    bad.actions[0] = Action::LedgePosition {
        offset: 12.,
        strength: f32::NAN,
        tolerance: 1.,
        brake: 6.,
        frames: 180,
    };
    let mut t = InputTimeline::new(bad, enemy(0., 100.));
    assert_eq!(
        sample(&mut t, own(0., 50., 1.), 100., Some([100., 0.]), -1.).requested,
        0
    );
    assert!(t.finished());
}
#[test]
fn new_tokens_roundtrip_and_reject_malformed_blocks_and_unsafe_parameters() {
    let text="[bowser.attack.test]\nstart_delay=0\nactions=run_to_ledge:12:0.8:1.5:8:240,if_facing_opponent,input:a:0:0:1,else_facing,input:b:0:0:1,end_facing\n";
    let first = Catalog::parse(text).unwrap();
    let saved = persist::update(
        text,
        15,
        kirby_assist_core::fighter_profiles::presets::PresetSlot::Attack,
        "test",
        first.entries()[0].sequence,
        false,
    )
    .unwrap();
    assert_eq!(
        Catalog::parse(&saved).unwrap().entries()[0].sequence,
        first.entries()[0].sequence
    );
    for actions in [
        "else_facing",
        "if_facing_opponent,input:a:0:0:1",
        "if_facing_ledge,if_facing_opponent,end_facing,end_facing",
        "if_facing_opponent,else_facing,else_facing,end_facing",
        "run_to_ledge:12:0.1",
        "run_to_ledge:-1:1",
        "run_to_ledge:12:1:NaN",
        "run_to_ledge:12:1:1:6:0",
    ] {
        assert!(
            Catalog::parse(&format!("[bowser.attack.test]\nactions={actions}\n")).is_err(),
            "{actions}"
        );
    }
}
#[test]
fn braking_does_not_finish_while_fast_momentum_would_carry_the_fighter_past_the_point() {
    let mut t = InputTimeline::new(
        recipe("run_to_ledge:12:1:1:6,input:a:0:0:1"),
        enemy(0., 100.),
    );
    let fast = GameplayContext {
        velocity: [2., 0.],
        ..own(0., 88., 1.)
    };
    let out = sample(&mut t, fast, 100., Some([100., 0.]), -1.);
    assert_eq!(out.requested, 0);
    assert_eq!(out.stick, Some([0.; 2]));
    let slow = GameplayContext {
        velocity: [0.1, 0.],
        ..own(1., 88.5, 1.)
    };
    assert_eq!(
        sample(&mut t, slow, 100., Some([100., 0.]), -1.).requested,
        MAPPER_A,
        "actual braking readiness, no settling timer"
    );
}

#[test]
fn reaching_the_target_in_air_never_runs_the_ground_positioning_followup() {
    for first_ground in [false, true] {
        let mut t = InputTimeline::new(recipe("run_to_ledge:12:1,up_b:4"), enemy(0., 100.));
        if first_ground {
            sample(&mut t, own(0., 60., 1.), 100., Some([100., 0.]), -1.);
        }
        let c = GameplayContext {
            situation: 2,
            ..own(1., 88., 1.)
        };
        assert_eq!(sample(&mut t, c, 100., Some([100., 0.]), -1.).requested, 0);
        assert!(t.finished());
    }
}
#[test]
fn zero_ledge_offset_already_at_the_anchor_is_an_immediate_noop_without_guessing_a_side() {
    let mut t = InputTimeline::new(recipe("run_to_ledge:0:1,input:a:0:0:1"), enemy(0., 100.));
    assert_eq!(
        sample(&mut t, own(0., 100., 1.), 100., Some([100., 0.]), 0.).requested,
        MAPPER_A
    );
}
#[test]
fn native_poll_gaps_do_not_abort_waiting_reach_or_confirmed_shield_hit_checks() {
    let mut t = InputTimeline::new(recipe("range:10,input:a:0:0:3"), enemy(0., 30.));
    assert_eq!(sample(&mut t, own(0., 0., 1.), 30., None, 0.).requested, 0);
    for frame in 3..6 {
        assert_eq!(
            sample(&mut t, own(frame as f32, 0., 1.), 5., None, 0.).requested,
            MAPPER_A
        );
    }
    let mut t = InputTimeline::new(
        recipe("shield_until_block:90,input:r:0:0:4"),
        enemy(0., 10.),
    );
    sample(
        &mut t,
        GameplayContext {
            status: kirby_assist_core::native_states::GUARD,
            ..own(0., 0., 1.)
        },
        10.,
        None,
        0.,
    );
    sample(
        &mut t,
        GameplayContext {
            status: kirby_assist_core::native_states::GUARD_DAMAGE,
            stopped: true,
            ..own(1., 0., 1.)
        },
        10.,
        None,
        0.,
    );
    let c = GameplayContext {
        status: kirby_assist_core::native_states::GUARD,
        ..own(3., 0., 1.)
    };
    assert_eq!(
        sample(&mut t, c, 10., None, 0.).requested,
        MAPPER_R | kirby_assist_core::native_input::MAPPER_ZR
    );
}

#[test]
fn opponent_conditionals_are_valid_onstage_but_ledge_positioning_requires_a_ledge_recipe() {
    use kirby_assist_core::reaction_rules::{validate_response, Rule};
    let mut conditional =
        recipe("if_facing_opponent,input:a:0:0:1,else_facing,input:b:0:0:1,end_facing");
    conditional.reaction = Rule::parse("auto_roll:30:12").unwrap();
    assert!(validate_response(conditional).is_ok());
    let mut position = recipe("run_to_ledge:12:1,input:a:0:0:1");
    position.reaction = conditional.reaction;
    assert!(validate_response(position)
        .unwrap_err()
        .contains("ledge anchor"));
    let mut old = recipe("if_facing_ledge,input:a:0:0:1,end_facing");
    old.reaction = conditional.reaction;
    assert!(
        validate_response(old).is_err(),
        "existing ledge-only constraints remain in place"
    );
}

#[test]
fn arrival_tolerance_includes_projected_coast_instead_of_allowing_a_second_tolerance_of_overshoot()
{
    let mut t = InputTimeline::new(
        recipe("run_to_ledge:12:1:1:6,input:a:0:0:1"),
        enemy(0., 100.),
    );
    let c = GameplayContext {
        velocity: [0.6, 0.],
        ..own(0., 88.9, 1.)
    };
    assert_eq!(
        sample(&mut t, c, 100., Some([100., 0.]), -1.).requested,
        0,
        "coasting would stop outside the requested tolerance"
    );
}
