use kirby_assist_core::{
    jab_lock::*,
    native_input::{MAPPER_A, MAPPER_R},
    timing::motion_hash,
};
fn context(frame: f32) -> Context {
    Context {
        object: 1,
        kind: 5,
        status: 0x2c,
        motion: motion_hash("attack_lw3"),
        frame,
        rate: 1.0,
        stopped: false,
        grounded: true,
        hit: true,
        facing: 1.0,
        position: [0.0; 2],
        can_move: false,
        can_jab: false,
        can_smash: false,
        can_grab: false,
        safe_ground: [true; 2],
    }
}
fn target(status: i32, x: f32) -> Target {
    Target {
        id: 2,
        status,
        frame: 1.0,
        position: [x, 0.0],
        velocity_x: 0.0,
        grounded: true,
        lock_count: 0,
        tech_available: false,
        vulnerable_in: Some(0.0),
        remaining: Some(30.0),
        hit_by_us: true,
    }
}
fn targets(t: Target) -> [Option<Target>; 8] {
    let mut a = [None; 8];
    a[0] = Some(t);
    a
}
fn idle(frame: f32) -> Context {
    Context {
        status: 0,
        motion: motion_hash("wait"),
        hit: false,
        can_move: true,
        can_jab: true,
        can_smash: true,
        can_grab: true,
        ..context(frame)
    }
}
#[test]
fn only_confirmed_attributed_down_tilts_arm_and_off_is_inert() {
    for (enabled, hit, attributed, kind) in [
        (false, true, true, 5),
        (true, false, true, 5),
        (true, true, false, 5),
        (true, true, true, 6),
    ] {
        let mut j = JabLock::default();
        let c = Context {
            hit,
            kind,
            ..context(8.0)
        };
        let out = j.sample(
            enabled,
            c,
            targets(Target {
                hit_by_us: attributed,
                ..target(0x47, 12.0)
            }),
        );
        assert_eq!(out.owned, 0);
        assert!(!j.watching());
    }
    let mut j = JabLock::default();
    j.sample(true, context(8.0), targets(target(0x47, 12.0)));
    assert!(j.watching());
}
#[test]
fn jab_eligibility_rejects_tech_invulnerability_air_and_exhausted_locks() {
    let t = target(0x50, 5.0);
    assert!(jab_lockable(t));
    for t in [
        Target {
            tech_available: true,
            ..t
        },
        Target {
            grounded: false,
            ..t
        },
        Target { lock_count: 2, ..t },
        Target {
            lock_count: -1,
            ..t
        },
        Target {
            vulnerable_in: None,
            ..t
        },
        Target {
            vulnerable_in: Some(2.0),
            ..t
        },
        target(0x67, 5.0),
    ] {
        assert!(!jab_lockable(t));
    }
}

#[test]
fn ambiguous_hit_victims_do_not_arm_and_cancelled_tilt_cannot_restart() {
    let mut j = JabLock::default();
    let mut ts = targets(target(0x47, 10.0));
    ts[1] = Some(Target {
        id: 3,
        ..target(0x47, 12.0)
    });
    j.sample(true, context(8.0), ts);
    assert!(!j.watching());
    let ts = targets(target(0x47, 10.0));
    j.sample(true, context(9.0), ts);
    assert!(j.watching());
    j.cancel_pending();
    j.sample(true, context(9.0), ts);
    assert!(!j.watching());
}
#[test]
fn chases_stops_at_floor_edge_turns_then_jabs_and_only_confirms_real_lock() {
    let mut j = JabLock::default();
    j.sample(true, context(8.0), targets(target(0x47, 20.0)));
    let out = j.sample(true, idle(1.0), targets(target(0x50, 20.0)));
    assert_eq!(out.stick, Some([0.65, 0.0]));
    assert_eq!(out.requested, 0);
    let out = j.sample(
        true,
        Context {
            safe_ground: [true, false],
            ..idle(2.0)
        },
        targets(target(0x50, 20.0)),
    );
    assert_eq!(out.stick, Some([0.0; 2]));
    let out = j.sample(
        true,
        Context {
            facing: -1.0,
            ..idle(3.0)
        },
        targets(target(0x50, 5.0)),
    );
    assert_eq!(out.requested, 0);
    assert_eq!(out.stick, Some([0.65, 0.0]));
    let out = j.sample(true, idle(4.0), targets(target(0x50, 5.0)));
    assert_eq!(out.requested, MAPPER_A);
    // Exact jab 1 startup releases A before the jab-2 chain window.
    let jab = Context {
        status: 0x27,
        motion: motion_hash("attack_11"),
        can_move: false,
        can_jab: false,
        can_smash: false,
        ..idle(1.0)
    };
    let out = j.sample(true, jab, targets(target(0x50, 5.0)));
    assert_eq!(out.requested, 0);
    let out = j.sample(true, idle(1.0), targets(target(0x50, 5.0)));
    assert_eq!(out.requested, 0, "jab input alone is not a lock");
    let out = j.sample(
        true,
        idle(2.0),
        targets(Target {
            lock_count: 1,
            ..target(0x5a, 5.0)
        }),
    );
    assert_eq!(out.requested, MAPPER_A);
    assert_eq!(out.stick, Some([0.0, -1.0]));
    let out = j.sample(
        true,
        Context {
            status: 0x30,
            ..idle(1.0)
        },
        targets(target(0x5a, 5.0)),
    );
    assert_eq!(out.owned, 0);
    assert!(!j.watching());
}
#[test]
fn tech_roll_grab_requires_reach_facing_and_a_real_recovery_window() {
    for opens in [None, Some(20.0), Some(0.0)] {
        let mut j = JabLock::default();
        j.sample(true, context(8.0), targets(target(0x47, 12.0)));
        let out = j.sample(
            true,
            idle(1.0),
            targets(Target {
                vulnerable_in: opens,
                ..target(0x68, 12.0)
            }),
        );
        assert_eq!(out.requested, if opens == Some(0.0) { MAPPER_R } else { 0 });
    }
    let mut j = JabLock::default();
    j.sample(true, context(8.0), targets(target(0x47, 12.0)));
    assert_eq!(
        j.sample(true, idle(1.0), targets(target(0x67, 12.0)))
            .requested,
        MAPPER_R
    );
}
#[test]
fn stopped_duplicate_samples_do_not_expire_attempt_and_target_loss_cancels() {
    let mut j = JabLock::default();
    j.sample(true, context(8.0), targets(target(0x47, 20.0)));
    let c = Context {
        stopped: true,
        ..idle(1.0)
    };
    for _ in 0..500 {
        j.sample(true, c, targets(target(0x47, 20.0)));
    }
    assert!(j.watching());
    j.sample(true, idle(2.0), [None; 8]);
    assert!(!j.watching());
    assert_eq!(
        j.sample(true, idle(3.0), targets(target(0x50, 5.0))).owned,
        0
    );
}
#[test]
fn accepted_jab_releases_a_even_in_hitstop_and_moving_rolls_need_future_reach() {
    let mut j = JabLock::default();
    j.sample(true, context(8.0), targets(target(0x47, 5.0)));
    assert_eq!(
        j.sample(true, idle(1.0), targets(target(0x50, 5.0)))
            .requested,
        MAPPER_A
    );
    let jab = Context {
        status: 0x27,
        motion: motion_hash("attack_11"),
        stopped: true,
        ..idle(1.0)
    };
    assert_eq!(j.sample(true, jab, targets(target(0x50, 5.0))).requested, 0);
    let mut j = JabLock::default();
    j.sample(true, context(8.0), targets(target(0x47, 12.0)));
    let out = j.sample(
        true,
        idle(1.0),
        targets(Target {
            velocity_x: 2.0,
            ..target(0x68, 12.0)
        }),
    );
    assert_eq!(
        out.requested, 0,
        "current reach alone cannot justify a grab on a moving roll"
    );
}

#[test]
fn delayed_jab_startup_keeps_chase_alive_until_native_acceptance() {
    let mut j = JabLock::default();
    j.sample(true, context(8.0), targets(target(0x47, 5.0)));
    assert_eq!(
        j.sample(true, idle(1.0), targets(target(0x50, 5.0)))
            .requested,
        MAPPER_A
    );
    for frame in 2..14 {
        let out = j.sample(true, idle(frame as f32), targets(target(0x50, 5.0)));
        assert_eq!(
            out.requested,
            if (frame - 1) % 4 < 3 { MAPPER_A } else { 0 }
        );
        assert!(
            j.watching(),
            "queued jab must not be abandoned after six frames"
        );
    }
    let jab = Context {
        status: 0x27,
        motion: motion_hash("attack_11"),
        can_smash: false,
        ..idle(0.0)
    };
    assert_eq!(
        j.sample(
            true,
            jab,
            targets(Target {
                lock_count: 1,
                ..target(0x5a, 5.0)
            })
        )
        .requested,
        0
    );
    assert_eq!(
        j.sample(
            true,
            idle(1.0),
            targets(Target {
                lock_count: 1,
                ..target(0x5a, 5.0)
            })
        )
        .requested,
        MAPPER_A
    );
}
