use kirby_assist_core::{
    gameplay::GameplayContext,
    native_input::MAPPER_R,
    out_of_shield::{Context, OutOfShield},
    roll_read::Target,
};
fn own(kind: i32, status: i32) -> Context {
    Context {
        object: 1,
        kind,
        status,
        motion: 1,
        frame: 0.,
        rate: 1.,
        stopped: false,
        grounded: true,
        position: [0.; 2],
        facing: 1.,
        shield_ready: true,
        can_grab: true,
        can_jump: false,
        can_up_special: false,
        threatened: false,
    }
}
fn context(status: i32) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        status,
        motion_kind: 1,
        motion_rate: 1.,
        facing: 1.,
        safe_ground: [true; 2],
        ..Default::default()
    }
}
fn target(x: f32) -> [Option<Target>; 8] {
    let mut t = [None; 8];
    t[0] = Some(Target {
        id: 2,
        status: kirby_assist_core::native_states::GUARD,
        motion: 2,
        frame: 0.,
        rate: 1.,
        position: [x, 0.],
        velocity_x: 0.,
        vulnerable_in: Some(0.),
        actionable_in: None,
    });
    t
}
fn sample(
    o: &mut OutOfShield,
    kind: i32,
    status: i32,
    x: f32,
) -> kirby_assist_core::gameplay::ControllerOutput {
    o.sample_automatic(
        true,
        own(kind, status),
        context(status),
        target(x),
        None,
        false,
        22.,
        35.,
        Some(8.),
    )
}
#[test]
fn automatic_grabs_never_start_runs_or_unshielded_inputs_for_any_fighter() {
    for kind in 0..kirby_assist_core::fighter_kinds::PLAYABLE_KIND_COUNT {
        let mut tool = OutOfShield::new();
        for status in [0, 1, 3, 4] {
            for x in [10., 30.] {
                for tick in 0..100 {
                    assert_eq!(
                        sample(&mut tool, kind, status, x),
                        Default::default(),
                        "kind {kind} status {status} tick {tick}"
                    );
                    tool.delivery_tick(tick);
                    assert!(tool.target().is_none());
                }
            }
        }
    }
}
#[test]
fn shield_grab_does_not_leave_run_input_after_native_catch_or_actor_replacement() {
    let mut tool = OutOfShield::new();
    let guard = kirby_assist_core::native_states::GUARD;
    let out = sample(&mut tool, 15, guard, 10.);
    assert_eq!(
        out.requested,
        MAPPER_R | kirby_assist_core::native_input::MAPPER_ZR
    );
    assert_eq!(out.stick, Some([0.; 2]));
    tool.mapped_grab(0, 2);
    assert_eq!(
        sample(&mut tool, 15, kirby_assist_core::native_states::CATCH, 10.),
        Default::default()
    );
    assert_eq!(sample(&mut tool, 22, 0, 10.), Default::default());
    assert!(tool.target().is_none());
}
#[test]
fn default_grab_calculations_use_each_fighters_references_and_saved_reach() {
    use kirby_assist_core::{
        fighter_data, fighter_profiles::FighterProfiles, fighter_tuning::Tuning,
    };
    let profiles = FighterProfiles::default();
    let t = Tuning::default();
    for kind in 0..kirby_assist_core::fighter_kinds::PLAYABLE_KIND_COUNT {
        let Some(p) = profiles.profile(kind) else {
            continue;
        };
        let range = p.grab.grab.standing_range;
        let known_reaches =
            fighter_data::ordinary_grab(kind, kirby_assist_core::native_states::CATCH)
                .zip(fighter_data::ordinary_grab(
                    kind,
                    kirby_assist_core::native_states::CATCH_DASH,
                ))
                .and_then(|(stand, dash)| Some((stand.reach?, dash.reach?)))
                .filter(|(stand, dash)| *stand > 0.0 && *dash > 0.0);
        for scale in [0.5, 1., 1.5] {
            let requested = range * scale;
            let (close, far) = t.auto_grab_ranges(kind, requested);
            if let Some((stand, dash)) = known_reaches {
                assert_eq!(close, requested, "own standing reference {kind}");
                let expected = (close * dash / stand).max(close);
                assert!((far - expected).abs() < 0.001, "own dash reference {kind}");
            } else {
                assert_eq!(
                    (close, far),
                    (0.0, 0.0),
                    "missing fighter reach must not create an automatic range for {kind}"
                );
            }
        }
    }
    assert_eq!(
        t.auto_grab_range(15, 9.),
        9.,
        "Bowser no longer forces a 22-unit fallback"
    );
    assert_eq!(
        t.auto_grab_ranges(999, 9.),
        (0., 0.),
        "unknown fighter data does not authorize an automatic grab range"
    );
}

#[test]
fn pivot_cleanup_never_replays_full_run_after_new_manual_direction_or_strength() {
    use kirby_assist_core::manual_controls::DashPivotAssist;
    for same_tick in [true, false] {
        for physical in [[-0.2, 0.], [0.9, 0.]] {
            let mut pivot = DashPivotAssist::default();
            pivot.sample(1, 3, 0, false, [1., 0.]);
            assert_eq!(
                pivot.sample(2, 3, 0, false, [-0.9, 0.]).stick,
                Some([-1., 0.])
            );
            let out = pivot.sample(if same_tick { 2 } else { 3 }, 3, 0, false, physical);
            if let Some(stick) = out.stick {
                assert!(stick[0] * physical[0] >= 0.);
                if physical[0].abs() < 0.8 {
                    assert!(stick[0].abs() < 0.8);
                }
            }
        }
    }
    let mut pivot = DashPivotAssist::default();
    pivot.sample(1, 3, 0, false, [1., 0.]);
    pivot.sample(2, 3, 0, false, [0.2, 0.]);
    assert_eq!(
        pivot.sample(2, 3, 0, false, [1., 0.]).stick,
        None,
        "an old walk cap cannot swallow a fresh full run input"
    );
}
