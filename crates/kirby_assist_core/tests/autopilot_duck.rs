use kirby_assist_core::{
    autopilot_guard::duck::{
        candidate, environment_allows_duck, DefenseThreat, DuckEnemy, DuckHold, FALCON_PUNCH_MOTION,
    },
    live_input::LiveConfig,
};

fn punch(frame: f32) -> DuckEnemy {
    DuckEnemy {
        id: 10,
        kind: 0xb,
        status: 0x1dc,
        situation: 0,
        motion: FALCON_PUNCH_MOTION,
        frame,
        rate: 1.0,
        facing: 1.0,
        position: [-15.0, 0.0],
        velocity: [0.0; 2],
    }
}
fn threats(enemy: DuckEnemy) -> DefenseThreat {
    DefenseThreat {
        shield: false,
        duck: candidate([0.0; 2], [0.0; 2], enemy),
    }
}

#[test]
fn early_crouch_requires_native_confirmation_and_survives_duplicate_polls() {
    let mut hold = DuckHold::default();
    assert!(hold.sample(true, true, false, threats(punch(38.0))));
    for _ in 0..100 {
        assert!(hold.sample(true, false, false, threats(punch(38.0))));
    }
    assert!(hold.sample(true, true, false, threats(punch(49.0))));
    // By frame 50 the native fully crouched status is mandatory.
    assert!(hold.sample(true, true, true, threats(punch(50.0))));
    assert!(hold.sample(true, false, true, threats(punch(53.0))));
    assert!(hold.sample(true, true, true, threats(punch(57.0))));
    assert!(!hold.sample(true, true, true, threats(punch(58.0))));

    assert!(hold.sample(true, true, false, threats(punch(40.0))));
    assert!(!hold.sample(true, true, false, threats(punch(50.0))));
    assert!(
        threats(punch(50.0)).needs_defense(),
        "failed crouch still requests shield fallback"
    );
}

#[test]
fn late_detection_release_interruptions_and_clock_changes_cannot_latch_duck() {
    let mut hold = DuckHold::default();
    assert!(!hold.sample(true, true, false, threats(punch(46.0))));
    assert!(!hold.sample(true, false, false, threats(punch(40.0))));
    assert!(hold.sample(true, true, false, threats(punch(40.0))));
    assert!(!hold.sample(false, true, true, threats(punch(41.0))));
    assert!(hold.sample(true, true, false, threats(punch(40.0))));
    assert!(!hold.sample(true, true, true, threats(punch(39.0))));
    assert!(hold.sample(true, true, false, threats(punch(40.0))));
    assert!(!hold.sample(
        true,
        true,
        true,
        threats(DuckEnemy {
            id: 11,
            frame: 50.0,
            ..punch(40.0)
        })
    ));
    assert!(hold.sample(true, true, false, threats(punch(40.0))));
    assert!(!hold.sample(true, true, true, DefenseThreat::default()));
}

#[test]
fn another_warning_or_attacker_always_wins_over_ducking() {
    let mut hold = DuckHold::default();
    let mut incoming = threats(punch(40.0));
    assert!(hold.sample(true, true, false, incoming));
    incoming.shield = true;
    assert!(!hold.sample(true, true, true, incoming));
    assert!(incoming.needs_defense());
    // Same aggregation result regardless of which opponent was visited first.
    let mut reversed = DefenseThreat {
        shield: true,
        duck: None,
    };
    reversed.add_duck(candidate([0.; 2], [0.; 2], punch(40.0)).unwrap());
    assert!(!hold.sample(true, true, true, reversed));
    let mut multiple = threats(punch(40.0));
    multiple.add_duck(
        candidate(
            [0.; 2],
            [0.; 2],
            DuckEnemy {
                id: 11,
                ..punch(40.0)
            },
        )
        .unwrap(),
    );
    assert!(multiple.shield);
    assert!(!hold.sample(true, true, true, multiple));
}

#[test]
fn unknown_variants_movement_height_size_and_platforms_are_not_ducked() {
    assert!(threats(punch(40.0)).duck.is_some());
    let mirrored = DuckEnemy {
        position: [15., 0.],
        facing: -1.,
        ..punch(40.0)
    };
    assert!(threats(mirrored).duck.is_some());
    for enemy in [
        DuckEnemy {
            kind: 6,
            ..punch(40.)
        }, // copied Punch is a different height
        DuckEnemy {
            status: 0x1e1,
            ..punch(40.)
        }, // reverse
        DuckEnemy {
            motion: 0xe396c1260,
            ..punch(40.)
        },
        DuckEnemy {
            situation: 2,
            ..punch(40.)
        },
        DuckEnemy {
            position: [-15., -1.],
            ..punch(40.)
        },
        DuckEnemy {
            position: [-2., 0.],
            ..punch(40.)
        },
        DuckEnemy {
            position: [-25., 0.],
            ..punch(40.)
        },
        DuckEnemy {
            facing: -1.,
            ..punch(40.)
        },
        DuckEnemy {
            velocity: [0.5, 0.],
            ..punch(40.)
        },
        DuckEnemy {
            frame: f32::NAN,
            ..punch(40.)
        },
        DuckEnemy {
            rate: 0.,
            ..punch(40.)
        },
    ] {
        assert!(threats(enemy).duck.is_none(), "{enemy:?}");
    }
    assert!(candidate([0.; 2], [0.5, 0.], punch(40.)).is_none());
    assert!(candidate([f32::NAN, 0.], [0.; 2], punch(40.)).is_none());
    assert!(environment_allows_duck(1., 1., false, false));
    assert!(!environment_allows_duck(1., 1., true, false));
    assert!(!environment_allows_duck(1., 1., false, true));
    assert!(!environment_allows_duck(1.5, 1., false, false));
    assert!(!environment_allows_duck(1., 0.5, false, false));
    assert!(!environment_allows_duck(f32::NAN, 1., false, false));
}

#[test]
fn lead_uses_animation_rate_and_config_can_disable_the_exception() {
    let mut hold = DuckHold::default();
    // Animation frame 49 has eight real game frames left at half speed.
    assert!(hold.sample(
        true,
        true,
        false,
        threats(DuckEnemy {
            frame: 49.,
            rate: 0.5,
            ..punch(40.)
        })
    ));
    assert!(!DuckHold::default().sample(
        true,
        true,
        false,
        threats(DuckEnemy {
            frame: 45.,
            rate: 2.,
            ..punch(40.)
        })
    ));
    // Auto-duck was disabled before the shared-observation refactor. Keep the
    // configuration expectation aligned with that existing default.
    assert!(!LiveConfig::parse("enabled = true").unwrap().kirby_auto_duck);
    assert!(
        !LiveConfig::parse("kirby_auto_duck = false")
            .unwrap()
            .kirby_auto_duck
    );
    assert!(LiveConfig::parse("kirby_auto_duck = maybe").is_err());
}

#[test]
fn independent_held_mode_outputs_down_or_shield_and_release_clears_both() {
    use kirby_assist_core::{
        autopilot_guard::{
            duck::{DuckContext, DuckDefense},
            GuardClock,
        },
        native_input::MAPPER_ZR,
    };
    let mut defense = DuckDefense::default();
    let c = DuckContext {
        clock: GuardClock {
            status: 0,
            motion: 123,
            frame: 0.,
            rate: 1.,
            stopped: false,
        },
        can_crouch: true,
        crouched: false,
        can_shield: true,
    };
    let duck = defense.sample(true, c, threats(punch(40.)));
    assert_eq!(duck.stick, Some([0., -1.]));
    assert_eq!(duck.requested, 0);
    assert_eq!(defense.sample(false, c, threats(punch(40.))).owned, 0);
    let shield = defense.sample(
        true,
        c,
        DefenseThreat {
            shield: true,
            duck: None,
        },
    );
    assert_eq!(shield.requested, MAPPER_ZR);
    assert_eq!(shield.stick, Some([0.; 2]));
    let next = defense.sample(true, c, threats(punch(40.)));
    assert_eq!(
        next.requested, MAPPER_ZR,
        "raised shield must not turn into down+shield"
    );
    assert_eq!(defense.sample(false, c, DefenseThreat::default()).owned, 0);
}
