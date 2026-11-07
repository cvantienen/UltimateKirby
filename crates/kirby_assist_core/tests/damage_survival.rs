use kirby_assist_core::{
    action_owner::safety_output,
    auto_di::{
        choose_trajectory, predicted_launch, AutoDi, BlastZone, Context, Settings, Trajectory,
    },
    combat_assist::{SdiContext, SdiState},
    tech_input::{TechContext, TechInput, TechOutput},
    tech_prediction::{first_contact_swept, Body},
};
fn bounds() -> BlastZone {
    BlastZone {
        left: -240.,
        right: 240.,
        bottom: -140.,
        top: 180.,
    }
}
fn trajectory(position: [f32; 2]) -> Trajectory {
    Trajectory {
        position,
        blast: bounds(),
        hitstun: 60.,
        gravity: 0.08,
        fall_speed: 1.6,
    }
}
fn context(launch: [f32; 2], remaining: u64) -> Context {
    Context {
        object: 1,
        damage_stop: true,
        remaining,
        launch,
        outward: 0.,
    }
}
fn sdi_context(remaining: u64) -> SdiContext {
    SdiContext {
        object: 1,
        damage_stop: true,
        remaining,
        input_delay: 0.,
        outward: 1.,
    }
}
fn tech_context() -> TechContext {
    TechContext {
        object: 1,
        status: kirby_assist_core::native_states::status::DAMAGE_FLY,
        motion: 1,
        frame: 0.,
        rate: 1.,
        stopped: false,
        remaining_stop: 0,
        contact_in: Some(3.),
        input_delay: 0.,
        previous_guard: false,
        prepare_guard: true,
    }
}
#[test]
fn native_death_query_recovers_translated_bounds_with_bounded_error_and_work() {
    let b = BlastZone {
        left: 320.,
        right: 800.,
        bottom: 100.,
        top: 420.,
    };
    let mut calls = 0;
    let observed = BlastZone::observe([600., 250.], |p| {
        calls += 1;
        if p[1] >= b.top {
            0
        } else if p[1] <= b.bottom {
            1
        } else if p[0] <= b.left {
            2
        } else if p[0] >= b.right {
            3
        } else {
            -1
        }
    })
    .unwrap();
    for (a, b) in [
        (observed.left, b.left),
        (observed.right, b.right),
        (observed.bottom, b.bottom),
        (observed.top, b.top),
    ] {
        assert!((a - b).abs() <= 0.25);
    }
    assert!(calls <= 90, "bounded per-hit query work: {calls}");
    assert!(BlastZone::observe([f32::NAN, 0.], |_| panic!("invalid position queried")).is_none());
    assert!(BlastZone::observe([0.; 2], |_| 0).is_none());
    assert!(
        BlastZone::observe([0.; 2], |_| -1).is_none(),
        "missing bounds must not create a stage"
    );
}
#[test]
fn stage_di_never_turns_a_predicted_survivor_into_a_predicted_ko() {
    for x in [-215., -100., 0., 100., 215.] {
        for y in [-110., 0., 150.] {
            for launch in [
                [4., 1.],
                [-4., 1.],
                [0., 5.],
                [0., -4.],
                [3., 3.],
                [-3., 3.],
                [4., -2.],
                [-4., -2.],
            ] {
                let t = trajectory([x, y]);
                let d = choose_trajectory(Settings::default(), context(launch, 3), t).unwrap();
                assert!(d.stick.iter().all(|v| v.is_finite()));
                assert!(d.stick[0].hypot(d.stick[1]) <= 1.00001);
                assert!(
                    t.score(predicted_launch(launch, d.stick)) + 0.001 >= t.score(launch),
                    "position {:?}, launch {launch:?}, stick {:?}",
                    t.position,
                    d.stick
                );
            }
        }
    }
}
#[test]
fn vertical_survival_uses_room_toward_stage_center_without_attacker_attribution() {
    for side in [-1., 1.] {
        let t = trajectory([side * 210., 100.]);
        let c = context([0., 5.], 3);
        let d = choose_trajectory(Settings::default(), c, t).unwrap();
        let launch = predicted_launch(c.launch, d.stick);
        assert!(
            launch[0] * side < 0.,
            "must flatten toward open space: {d:?}"
        );
        assert!(launch[1] < 5.);
    }
}
#[test]
fn final_di_is_stable_through_zero_count_then_releases_and_rearms_for_another_hit() {
    let mut di = AutoDi::default();
    let mut t = trajectory([210., 100.]);
    let settings = Settings::default();
    assert_eq!(
        di.sample_exclusive_with_trajectory(settings, context([0., 5.], 8), Some(t)),
        None
    );
    let first = di
        .sample_exclusive_with_trajectory(settings, context([0., 5.], 3), Some(t))
        .unwrap();
    t.position = [-210., 100.]; // A refresh cannot rewrite an already committed launch input.
    for remaining in [3, 2, 2, 1, 0, 0] {
        assert_eq!(
            di.sample_exclusive_with_trajectory(settings, context([0., 5.], remaining), Some(t)),
            Some(first)
        );
    }
    assert_eq!(
        di.sample_exclusive_with_trajectory(
            settings,
            Context {
                damage_stop: false,
                ..context([0., 5.], 0)
            },
            Some(t)
        ),
        None
    );
    let next = di
        .sample_exclusive_with_trajectory(settings, context([0., 5.], 2), Some(t))
        .unwrap();
    assert!(next[0] * first[0] < 0.);
}
#[test]
fn launch_model_is_finite_mirrored_and_respects_vertical_lsi_exclusion() {
    assert_eq!(predicted_launch([0.; 2], [1., 1.]), [0.; 2]);
    for launch in [[4., 1.], [2., 4.], [4., -2.], [0., 5.]] {
        for stick in [[1., 0.], [0., -1.], [-0.70710677, 0.70710677]] {
            let right = predicted_launch(launch, stick);
            let left = predicted_launch([-launch[0], launch[1]], [-stick[0], stick[1]]);
            assert!((right[0] + left[0]).abs() < 0.00001 && (right[1] - left[1]).abs() < 0.00001);
        }
    }
    assert!((predicted_launch([0., 5.], [0., -1.])[1] - 5.).abs() < 0.00001);
    assert!((predicted_launch([4., 0.], [0., -1.])[0] - 3.68).abs() < 0.00001);
}
#[test]
fn sdi_uses_full_strength_flicks_every_four_native_frames_and_reuses_duplicate_polls() {
    let mut sdi = SdiState::default();
    for elapsed in 0..17 {
        let c = sdi_context(24 - elapsed);
        let expected = if elapsed % 4 == 0 {
            Some([1., 0.])
        } else {
            Some([0.; 2])
        };
        for _ in 0..12 {
            assert_eq!(sdi.sample_exclusive(true, c), expected);
        }
    }
    assert_eq!(
        sdi.sample_exclusive(
            true,
            SdiContext {
                damage_stop: false,
                ..sdi_context(8)
            }
        ),
        None
    );
    assert_eq!(sdi.sample_exclusive(true, sdi_context(8)), Some([1., 0.]));
}
#[test]
fn sdi_cannot_skip_its_neutral_rearm_or_replay_a_stale_flick_after_disable() {
    let mut sdi = SdiState::default();
    assert_eq!(sdi.sample_exclusive(true, sdi_context(20)), Some([1., 0.]));
    assert_eq!(sdi.sample_exclusive(true, sdi_context(15)), Some([0.; 2]));
    assert_eq!(sdi.sample_exclusive(true, sdi_context(14)), Some([1., 0.]));
    assert_eq!(sdi.sample_exclusive(false, sdi_context(13)), None);
    assert_eq!(sdi.sample_exclusive(true, sdi_context(13)), Some([1., 0.]));
}
#[test]
fn sdi_releases_an_already_held_game_visible_stick_before_the_first_and_next_hit() {
    let mut sdi = SdiState::default();
    for _ in 0..12 {
        assert_eq!(
            sdi.sample_exclusive_with_previous(true, sdi_context(20), None, [1., 0.]),
            Some([0.; 2])
        );
    }
    assert_eq!(
        sdi.sample_exclusive_with_previous(true, sdi_context(19), None, [0.; 2]),
        Some([1., 0.])
    );
    assert_eq!(
        sdi.sample_exclusive_with_previous(true, sdi_context(22), None, [1., 0.]),
        Some([0.; 2]),
        "a new hit must not reuse the preceding held flick"
    );
    assert_eq!(
        sdi.sample_exclusive_with_previous(true, sdi_context(21), None, [0.; 2]),
        Some([1., 0.])
    );
}
#[test]
fn sdi_bends_away_from_the_nearest_blast_zone() {
    let mut sdi = SdiState::default();
    let p = [238., 0.];
    let stick = sdi
        .sample_exclusive_with_geometry(true, sdi_context(20), Some((p, bounds())))
        .unwrap();
    assert!(
        stick[0] < 0.,
        "attacker separation must not SDI out of bounds: {stick:?}"
    );
    assert!(bounds().contains([p[0] + stick[0] * 8., p[1] + stick[1] * 8.]));
}
#[test]
fn every_safety_combination_emits_exactly_one_response_and_only_winning_tech_gets_acknowledged() {
    let tech = TechOutput {
        press: true,
        owns_guard: true,
    };
    for mask in 0..16 {
        let mash = (mask & 1 != 0).then_some([1., 0.]);
        let di = (mask & 2 != 0).then_some([0., 1.]);
        let sdi = (mask & 4 != 0).then_some([-1., 0.]);
        let tech = if mask & 8 != 0 {
            tech
        } else {
            TechOutput::default()
        };
        let (packet, ack) = safety_output(tech, mash, di, sdi);
        if mash.is_some() || di.is_some() {
            assert_eq!(packet.stick, mash.or(di));
            assert_eq!(packet.requested, 0);
            assert_eq!(ack, None);
        } else if tech.press {
            assert!(packet.requested != 0);
            assert_eq!(packet.stick, None);
            assert_eq!(ack, Some(tech));
        } else {
            assert_eq!(packet.stick, sdi);
            assert_eq!(packet.requested, 0);
        }
    }
}
#[test]
fn tech_window_and_lockout_follow_native_parameters_without_poll_or_transition_ticks() {
    let mut tech = TechInput::default();
    let c = TechContext {
        contact_in: Some(10.),
        ..tech_context()
    };
    assert!(
        !tech
            .sample_with_native_window(true, true, 28, c, None, 8, 7)
            .press
    );
    assert!(
        tech.sample_with_native_window(true, true, 28, c, None, 14, 7)
            .press
    );
    tech.mapped_press();
    assert_eq!(tech.cooldown_remaining(), 28.);
    for _ in 0..20 {
        tech.sample_with_native_window(true, true, 28, c, None, 14, 7);
    }
    assert_eq!(tech.cooldown_remaining(), 28.);
    tech.sample_with_native_window(
        true,
        true,
        28,
        TechContext {
            motion: 2,
            status: kirby_assist_core::native_states::status::DAMAGE_FALL,
            frame: 0.,
            ..c
        },
        None,
        14,
        7,
    );
    assert_eq!(
        tech.cooldown_remaining(),
        28.,
        "a transition is not evidence of one elapsed frame"
    );
}
#[test]
fn disabling_tech_releases_ownership_without_erasing_a_delivered_native_lockout() {
    let mut tech = TechInput::default();
    let c = tech_context();
    assert!(
        tech.sample_with_native_window(true, true, 28, c, None, 12, 7)
            .press
    );
    tech.mapped_press();
    assert_eq!(
        tech.sample_with_native_window(false, true, 28, c, None, 12, 7),
        TechOutput::default()
    );
    assert_eq!(tech.cooldown_remaining(), 28.);
    assert!(
        !tech
            .sample_with_native_window(true, true, 28, c, None, 12, 7)
            .press
    );
    tech.sample_with_native_window(false, true, 28, TechContext { frame: 1., ..c }, None, 12, 7);
    assert_eq!(tech.cooldown_remaining(), 27.);
    assert!(
        !tech
            .sample_with_native_window(true, true, 28, TechContext { frame: 1., ..c }, None, 12, 7)
            .press
    );
}
#[test]
fn tech_sweeps_cover_narrow_midflight_platforms_and_respect_surface_eligibility() {
    let platform = |r: kirby_assist_core::tech_prediction::CollisionRay| {
        if !r.passable || r.delta[1] >= 0. {
            return false;
        }
        let t = (6. - r.origin[1]) / r.delta[1];
        let x = r.origin[0] + r.delta[0] * t;
        (0.0..=1.).contains(&t) && (11.0..=12.).contains(&x)
    };
    let hit = first_contact_swept(
        0.,
        10.,
        5.,
        -2.,
        5.,
        0.,
        Body::default(),
        [true; 3],
        platform,
    );
    assert!(hit.is_some_and(|t| (1.0..=5.).contains(&t)), "{hit:?}");
    assert_eq!(
        first_contact_swept(
            0.,
            10.,
            5.,
            -2.,
            5.,
            0.,
            Body::default(),
            [false, true, true],
            platform
        ),
        None
    );
    for side in [-1., 1.] {
        assert!(first_contact_swept(
            0.,
            0.,
            side * 10.,
            0.,
            5.,
            0.,
            Body::default(),
            [false, true, false],
            |r| {
                let t = (side * 24. - r.origin[0]) / r.delta[0];
                (0.0..=1.).contains(&t)
            }
        )
        .is_some());
    }
    assert!(first_contact_swept(
        0.,
        0.,
        0.,
        5.,
        5.,
        0.,
        Body::default(),
        [false, false, true],
        |r| {
            let t = (18. - r.origin[1]) / r.delta[1];
            (0.0..=1.).contains(&t)
        }
    )
    .is_some());
}
