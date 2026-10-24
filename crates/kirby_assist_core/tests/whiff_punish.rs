use kirby_assist_core::{
    button_modes::YMode,
    combat_assist::FairContext,
    finisher,
    ledge_trap::LedgeOutput,
    native_input::{MAPPER_A, MAPPER_X, MAPPER_ZL},
    timing::motion_hash,
    whiff_punish::{Context, Target, WhiffPunisher},
};
fn own(frame: f32) -> Context {
    Context {
        starter: finisher::Context {
            fighter_kind: 6,
            target_direction: None,
            fighter: FairContext {
                object: 1,
                kirby: true,
                status: 0,
                motion: 1,
                frame,
                rate: 1.0,
                grounded: true,
                stopped: false,
                facing: 1.0,
                hit_now: false,
            },
            velocity_y: -0.5,
            can_jump: true,
            can_start: true,
        },
        position: [0.0, 0.0],
        velocity_x: 0.0,
        walk_speed: Some(0.977),
        can_tilt: true,
        can_smash: true,
        safe_ground: [true; 2],
    }
}
fn target(frame: f32, remaining: f32, x: f32) -> Target {
    Target {
        id: 2,
        kind: 6,
        motion: motion_hash("attack_lw4"),
        frame,
        position: [x, 0.0],
        velocity_x: 0.0,
        remaining,
    }
}
fn targets(t: Target) -> [Option<Target>; 8] {
    let mut a = [None; 8];
    a[0] = Some(t);
    a
}
#[test]
fn kirby_selects_smash_or_tilt_by_actual_recovery_and_issues_once() {
    for (remaining, stick) in [(30.0, [1.0, 0.0]), (10.0, [0.0, -1.0])] {
        let mut p = WhiffPunisher::default();
        let t = target(50.0 - remaining, remaining, 6.0);
        let prime = p.sample(true, own(0.0), targets(t));
        assert_ne!(prime.owned, 0);
        assert_eq!(prime.requested, 0);
        assert_eq!(p.sample(true, own(0.0), targets(t)), prime);
        let fire = p.sample(
            true,
            own(1.0),
            targets(target(t.frame + 1.0, remaining - 1.0, 6.0)),
        );
        assert_eq!(fire.requested, MAPPER_A);
        assert_eq!(fire.stick, Some(stick));
        let mut accepted = own(0.0);
        accepted.starter.fighter.status = if remaining > 10.0 { 0x2d } else { 0x2c };
        accepted.starter.fighter.motion = 2;
        assert_eq!(
            p.sample(
                true,
                accepted,
                targets(target(t.frame + 2.0, remaining - 2.0, 6.0))
            )
            .requested,
            0
        );
        assert_eq!(
            p.sample(
                true,
                own(2.0),
                targets(target(t.frame + 3.0, remaining - 3.0, 6.0))
            )
            .owned,
            0
        );
    }
}
#[test]
fn approach_needs_time_and_floor_and_stops_before_attack() {
    let mut p = WhiffPunisher::default();
    let t = target(20.0, 30.0, 16.0);
    assert_eq!(
        p.sample(true, own(0.0), targets(t)).stick,
        Some([0.65, 0.0])
    );
    let mut c = own(1.0);
    c.position[0] = 5.0;
    assert_eq!(
        p.sample(true, c, targets(target(21.0, 29.0, 16.0)))
            .requested,
        0
    );
    c.starter.fighter.frame = 2.0;
    assert_eq!(
        p.sample(true, c, targets(target(22.0, 28.0, 16.0)))
            .requested,
        MAPPER_A
    );
    for (c, t) in [
        (own(0.0), target(44.0, 6.0, 8.0)),
        (own(0.0), target(20.0, 30.0, 50.0)),
    ] {
        assert_eq!(
            WhiffPunisher::default().sample(true, c, targets(t)),
            LedgeOutput::default()
        );
    }
    let mut c = own(0.0);
    c.safe_ground = [false; 2];
    assert_eq!(
        WhiffPunisher::default().sample(true, c, targets(t)).owned,
        0
    );
}
#[test]
fn off_unknown_active_motion_and_target_loss_cancel_without_attacking() {
    let t = target(20.0, 30.0, 6.0);
    for bad in [
        Target { kind: 0x77, ..t },
        Target {
            motion: motion_hash("unknown"),
            ..t
        },
        Target { frame: 18.0, ..t },
    ] {
        assert_eq!(
            WhiffPunisher::default()
                .sample(true, own(0.0), targets(bad))
                .owned,
            0
        );
    }
    let mut p = WhiffPunisher::default();
    p.sample(true, own(0.0), targets(t));
    assert_eq!(p.sample(false, own(1.0), targets(t)).owned, 0);
    p.sample(true, own(2.0), targets(t));
    assert_eq!(
        p.sample(true, own(2.0), [None; 8]).owned,
        0,
        "target loss releases even on a duplicate own-animation sample"
    );
}

#[test]
fn roster_slot_changes_do_not_rearm_the_same_enemy_move() {
    let mut p = WhiffPunisher::default();
    p.sample(true, own(0.0), targets(target(20.0, 30.0, 6.0)));
    assert_eq!(
        p.sample(true, own(1.0), targets(target(21.0, 29.0, 6.0)))
            .requested,
        MAPPER_A
    );
    let mut accepted = own(0.0);
    accepted.starter.fighter.status = 0x2d;
    accepted.starter.fighter.motion = 2;
    p.sample(true, accepted, targets(target(22.0, 28.0, 6.0)));
    let mut shifted = [None; 8];
    shifted[3] = Some(target(23.0, 27.0, 6.0));
    assert_eq!(p.sample(true, own(2.0), shifted).owned, 0);
}
#[test]
fn yoshi_bair_starter_turns_away_then_short_hops_without_a_grounded_attack() {
    let mut p = WhiffPunisher::default();
    let mut c = own(0.0);
    c.starter.fighter_kind = 5;
    c.starter.fighter.kirby = false;
    let turn = p.sample(true, c, targets(target(20.0, 30.0, 8.0)));
    assert_eq!(turn.requested, 0);
    assert_eq!(turn.stick, Some([-0.3, 0.0]));
    c.starter.fighter.frame = 1.0;
    c.starter.fighter.facing = -1.0;
    let hop = p.sample(true, c, targets(target(21.0, 29.0, 8.0)));
    assert_eq!(hop.requested, MAPPER_X | MAPPER_ZL);
    assert!(YMode::YoshiBairSmash.supports(5));
    assert!(!YMode::FairSmash.supports(5));
}
