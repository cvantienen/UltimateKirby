use kirby_assist_core::{
    kirby::{TripContext, TripFollowup, TripTarget},
    native_input::MAPPER_A,
};

#[test]
fn legacy_combat_assist_exports_are_the_same_kirby_types() {
    use kirby_assist_core::{combat_assist, kirby};
    use std::any::TypeId;

    assert_eq!(
        TypeId::of::<combat_assist::TripTarget>(),
        TypeId::of::<kirby::TripTarget>()
    );
    assert_eq!(
        TypeId::of::<combat_assist::TripContext>(),
        TypeId::of::<kirby::TripContext>()
    );
    assert_eq!(
        TypeId::of::<combat_assist::TripFollowup>(),
        TypeId::of::<kirby::TripFollowup>()
    );
    assert_eq!(
        TypeId::of::<combat_assist::FairContext>(),
        TypeId::of::<kirby::FairContext>()
    );
    assert_eq!(
        TypeId::of::<combat_assist::FairFollowup>(),
        TypeId::of::<kirby::FairFollowup>()
    );
}

fn context(tilt: i32) -> TripContext {
    let mut targets = [None; 8];
    targets[0] = Some(TripTarget {
        id: 2, status: 0x88, previous_status: 0,
        relative_position: [-8.0, 0.0], grounded: true, hit_confirmed: true,
    });
    TripContext {
        object: 1, kirby: true, status: tilt, motion: 123, frame: 10.0,
        grounded: true, facing: -1.0, stopped: false, targets,
    }
}
fn baseline(c: TripContext) -> TripContext {
    let mut b = c;
    b.frame -= 1.0;
    b.targets[0].as_mut().unwrap().status = 0;
    b.targets[0].as_mut().unwrap().hit_confirmed = false;
    b
}
fn arm(tool: &mut TripFollowup, c: TripContext) {
    assert_eq!(tool.sample(true, baseline(c)).owned, 0);
    assert_ne!(tool.sample(true, c).owned, 0);
    assert!(tool.watching());
}
#[test]
fn only_fresh_nearby_grounded_hit_confirmed_trip_during_either_tilt_arms() {
    for tilt in [0x2a, 0x2c] {
        let c = context(tilt);
        let mut tool = TripFollowup::default();
        assert_eq!(tool.sample(true, c).owned, 0, "existing trip is not attributable");
        arm(&mut tool, c);
        assert_eq!(tool.sample(true, c).requested, 0);
        let press = tool.sample(true, TripContext { frame: 11.0, ..c });
        assert_eq!(press.requested, MAPPER_A);
        assert_eq!(press.stick, Some([-1.0, 0.0]));
    }
    for case in 0..9 {
        let c = context(0x2c);
        let mut tool = TripFollowup::default();
        tool.sample(true, baseline(c));
        let mut rejected = c;
        match case {
            0 => rejected.targets[0].as_mut().unwrap().hit_confirmed = false,
            1 => rejected.targets[0].as_mut().unwrap().relative_position[0] = -19.0,
            2 => rejected.targets[0].as_mut().unwrap().relative_position[0] = 8.0,
            3 => rejected.targets[0].as_mut().unwrap().relative_position[1] = 9.0,
            4 => rejected.targets[0].as_mut().unwrap().relative_position[0] = f32::NAN,
            5 => rejected.targets[0].as_mut().unwrap().grounded = false,
            6 => rejected.grounded = false,
            7 => rejected.status = 0,
            _ => rejected.targets[0].as_mut().unwrap().status = 0x47,
        }
        assert_eq!(tool.sample(true, rejected).owned, 0, "case {case}");
        assert!(!tool.watching());
    }
}
#[test]
fn target_loss_recovery_replacement_range_and_facing_changes_cancel_without_resumption() {
    for case in 0..7 {
        let c = context(0x2c);
        let mut tool = TripFollowup::default();
        arm(&mut tool, c);
        let pressed = TripContext { frame: 11.0, ..c };
        assert_eq!(tool.sample(true, pressed).requested, MAPPER_A);
        let mut invalid = pressed;
        match case {
            0 => invalid.targets = [None; 8],
            1 => invalid.targets[0].as_mut().unwrap().status = 0,
            2 => invalid.targets[0].as_mut().unwrap().id = 3,
            3 => invalid.targets[0].as_mut().unwrap().relative_position[0] = -30.0,
            4 => invalid.facing = 1.0,
            5 => invalid.facing = f32::NAN,
            _ => invalid.targets[0].as_mut().unwrap().grounded = false,
        }
        assert_eq!(tool.sample(true, invalid).requested, 0);
        assert!(!tool.watching());
        for frame in 12..16 {
            assert_eq!(tool.sample(true, TripContext { frame: frame as f32, ..c }).owned, 0);
        }
    }
}
#[test]
fn retries_are_bounded_by_native_progress_and_release_permanently_for_spent_tilt() {
    let c = context(0x2c);
    let mut tool = TripFollowup::default();
    arm(&mut tool, c);
    let mut presses = 0;
    for frame in 11..40 {
        let next = TripContext { frame: frame as f32, ..c };
        let output = tool.sample(true, next);
        presses += usize::from(output.requested == MAPPER_A);
        assert_eq!(tool.sample(true, next), output);
        let stopped = TripContext { stopped: true, ..next };
        assert_eq!(tool.sample(true, stopped), output);
        // Target evidence changes do not manufacture another native frame.
        let mut evidence = stopped;
        evidence.targets[0].as_mut().unwrap().hit_confirmed = false;
        assert_eq!(tool.sample(true, evidence), output);
    }
    assert_eq!(presses, 6);
    assert!(!tool.watching());
    assert_eq!(tool.sample(true, TripContext { frame: 41.0, ..c }).owned, 0);
    // A genuinely new tilt may arm only after new non-trip -> trip evidence.
    tool.sample(true, TripContext { status: 0, frame: 0.0, ..c });
    arm(&mut tool, c);
}
#[test]
fn smash_startup_releases_and_prior_trip_damage_does_not_arm() {
    let c = context(0x2c);
    let mut tool = TripFollowup::default();
    arm(&mut tool, c);
    tool.sample(true, TripContext { frame: 11.0, ..c });
    let startup = TripContext {
        status: kirby_assist_core::native_states::status::ATTACK_S4_START, frame: 0.0, ..c
    };
    assert_eq!(tool.sample(true, startup).requested, 0);
    assert!(!tool.watching());
    assert_eq!(tool.sample(true, TripContext { frame: 1.0, ..startup }).owned, 0);
    let mut damage = c;
    damage.targets[0].as_mut().unwrap().status = 0x47;
    damage.targets[0].as_mut().unwrap().previous_status = 0x88;
    assert_eq!(TripFollowup::default().sample(true, damage).owned, 0);
}
