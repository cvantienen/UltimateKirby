use kirby_assist_core::{
    ledge_trap::LedgeContext,
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
fn context(status: i32) -> LedgeContext {
    LedgeContext {
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
        status: 0x1c,
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
) -> kirby_assist_core::ledge_trap::LedgeOutput {
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
fn close_idle_grab_does_not_create_a_run_on_any_fighter() {
    for kind in [15, 6, 5, 22, 28, 90, 91] {
        let mut o = OutOfShield::new();
        let out = sample(&mut o, kind, 0, 10.);
        assert_eq!(out.requested, MAPPER_R, "kind {kind}");
        assert_eq!(out.stick, Some([0.; 2]));
        assert!(!o.approach_pending());
    }
}
#[test]
fn failed_idle_dash_cannot_restart_forever_against_the_same_shield() {
    let mut o = OutOfShield::new();
    for tick in 0..4 {
        let out = sample(&mut o, 15, 0, 30.);
        assert_eq!(out.requested, 0);
        assert_eq!(out.stick, Some([1., 0.]));
        o.mapped_approach(tick, 2);
    }
    assert_eq!(sample(&mut o, 15, 0, 30.).owned, 0);
    for _ in 0..100 {
        assert_eq!(
            sample(&mut o, 15, 0, 30.).owned,
            0,
            "failed approach must not rearm itself"
        );
    }
}
#[test]
fn delivered_grab_cannot_keep_forcing_run_or_override_opposite_manual_input() {
    let mut o = OutOfShield::new();
    sample(&mut o, 15, 0, 30.);
    o.mapped_approach(0, 2);
    let grab = sample(&mut o, 15, 3, 30.);
    assert_eq!(grab.requested, MAPPER_R);
    assert_eq!(o.with_grab_movement(grab, [0.; 2]).stick, Some([1., 0.]));
    o.mapped_grab(1, 2);
    let tail = sample(&mut o, 15, 3, 30.);
    assert_eq!(o.with_grab_movement(tail, [0.; 2]).stick, Some([0.; 2]));
    assert_eq!(o.with_grab_movement(tail, [-0.7, 0.]).stick, None);
}

#[test]
fn lost_dash_and_grab_acknowledgements_are_bounded_even_when_native_animation_is_frozen() {
    for phase in [0, 1] {
        let mut o = OutOfShield::new();
        if phase == 1 {
            sample(&mut o, 15, 0, 30.);
            o.mapped_approach(-1, 2);
        }
        let status = if phase == 0 { 0 } else { 3 };
        for tick in 0..4 {
            let out = sample(&mut o, 15, status, 30.);
            assert_ne!(out.owned, 0);
            if phase == 1 && tick > 0 {
                assert_eq!(o.with_grab_movement(out, [0.; 2]).stick, Some([0.; 2]));
            }
            // The original mapper completed, but no request receipt arrived.
            o.delivery_tick(tick);
            o.delivery_tick(tick);
        }
        assert_eq!(sample(&mut o, 15, status, 30.).owned, 0);
        for _ in 0..50 {
            assert_eq!(sample(&mut o, 15, 0, 30.).owned, 0);
        }
    }
}
#[test]
fn failed_dash_rearms_only_on_new_manual_intent_or_a_positive_end_of_enemy_shield() {
    for rearm in [0, 1] {
        let mut o = OutOfShield::new();
        for tick in 0..4 {
            sample(&mut o, 22, 0, 30.);
            o.mapped_approach(tick, 2);
        }
        assert_eq!(sample(&mut o, 22, 0, 30.).owned, 0);
        o.observe_statuses(&[None; 16]);
        for _ in 0..20 {
            assert_eq!(sample(&mut o, 22, 0, 30.).owned, 0);
        }
        let mut c = context(0);
        if rearm == 0 {
            c.manual_input_epoch = 1;
        } else {
            o.observe_statuses(&[Some((2, 0))]);
        }
        assert_ne!(
            o.sample_automatic(
                true,
                own(22, 0),
                c,
                target(30.),
                None,
                false,
                22.,
                35.,
                Some(8.)
            )
            .owned,
            0
        );
    }
}
#[test]
fn a_new_fighter_using_the_same_object_id_does_not_inherit_pending_grabs_or_failures() {
    let mut o = OutOfShield::new();
    sample(&mut o, 15, 0, 30.);
    o.mapped_approach(0, 2);
    let out = o.sample_automatic(
        true,
        own(22, 0),
        context(0),
        [None; 8],
        None,
        false,
        22.,
        35.,
        Some(8.),
    );
    assert_eq!(out.owned, 0);
    assert!(!o.approach_pending());
    assert!(o.target().is_none());
    assert_eq!(sample(&mut o, 22, 0, 10.).requested, MAPPER_R);
}
#[test]
fn aborting_an_assisted_dash_for_manual_movement_cannot_admit_a_competing_throw_packet() {
    let mut o = OutOfShield::new();
    let approach = sample(&mut o, 15, 0, 30.);
    assert_eq!(o.with_grab_movement(approach, [-1., 0.]).owned, 0);
    assert!(o.target().is_none());
    sample(&mut o, 15, 0, 30.);
    o.mapped_approach(0, 2);
    let grab = sample(&mut o, 15, 3, 30.);
    o.mapped_grab(1, 2);
    assert_eq!(o.with_grab_movement(grab, [-1., 0.]).owned, 0);
    assert!(
        o.grab_delivery_pending(),
        "keep only the inert receipt against a competing Up-B"
    );
}
#[test]
fn default_grab_calculations_use_each_fighters_references_and_saved_reach() {
    use kirby_assist_core::{
        fighter_data, fighter_profiles::FighterProfiles, fighter_tuning::Tuning,
    };
    let profiles = FighterProfiles::default();
    let t = Tuning::default();
    for kind in 0..0x5e {
        let Some(p) = profiles.profile(kind) else {
            continue;
        };
        let range = p.ledge.grab.standing_range;
        let known_reaches = fighter_data::ordinary_grab(kind, 0x37)
            .zip(fighter_data::ordinary_grab(kind, 0x39))
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
