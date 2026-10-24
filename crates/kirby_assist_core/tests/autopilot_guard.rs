use kirby_assist_core::{autopilot_guard::approaching_active_attack, live_input::LiveConfig};

#[test]
fn old_configs_enable_defense_and_explicit_disable_remains_available() {
    assert!(
        LiveConfig::parse("enabled = true")
            .unwrap()
            .autopilot_shield
    );
    assert!(
        !LiveConfig::parse("autopilot_shield = false")
            .unwrap()
            .autopilot_shield
    );
    assert!(LiveConfig::parse("autopilot_shield = maybe").is_err());
    assert!(
        LiveConfig::parse(include_str!(
            "../../../deployment/ultimate/kirby_assist/config.txt"
        ))
        .unwrap()
        .autopilot_shield
    );
}

#[test]
fn relative_motion_predicts_approach_from_either_side_but_not_distant_attacks() {
    assert!(approaching_active_attack(
        [0., 0.],
        [1., 0.],
        [29., 0.],
        [-1., 0.]
    ));
    assert!(approaching_active_attack(
        [0., 0.],
        [-1., 0.],
        [-29., 0.],
        [1., 0.]
    ));
    assert!(!approaching_active_attack(
        [0., 0.],
        [1., 0.],
        [29., 0.],
        [1., 0.]
    ));
    assert!(!approaching_active_attack(
        [0., 0.],
        [0., 0.],
        [0., 60.],
        [0., -1.]
    ));
}

#[test]
fn imminent_moves_use_real_startup_and_rate_without_predicting_unknown_aerials() {
    use kirby_assist_core::{
        autopilot_guard::imminent_attack,
        fighter_profiles::{samus, FighterProfiles},
    };
    let data = FighterProfiles::default().profile(6).unwrap().ledge;
    assert!(imminent_attack(data, 0x2c, 1, 1., 1.)); // Dtilt 4, three frames away.
    assert!(!imminent_attack(data, 0x2c, 1, 0., 1.));
    assert!(imminent_attack(data, 0x36, 0xc3a4e2597, 5., 1.)); // Kirby nair 8.
    assert!(!imminent_attack(data, 0x36, 0xc3495ada5, 5., 1.)); // Fair is not nair.
    assert!(imminent_attack(data, 0x32, 1, 5., 1.));
    assert!(!imminent_attack(data, 0x32, 1, 5., 0.5));
    assert!(!imminent_attack(data, 0x37, 1, 5., 1.)); // Grabs excluded.
    assert!(!imminent_attack(data, 0x2c, 1, f32::NAN, 1.));
    // Samus's rear sweep is a separate window, not an invented continuous hit.
    assert!(imminent_attack(samus::LEDGE_DATA, 0x32, 1, 14., 1.));
    assert!(!imminent_attack(samus::LEDGE_DATA, 0x32, 1, 11., 1.));
}

#[test]
fn guard_tail_bridges_gaps_in_native_frames_and_release_cancels_immediately() {
    use kirby_assist_core::autopilot_guard::{GuardClock, GuardHold};
    let clock = |frame| GuardClock {
        status: 0x1c,
        motion: 123,
        frame,
        rate: 1.,
        stopped: false,
    };
    let mut hold = GuardHold::default();
    assert!(hold.sample(true, true, clock(0.)));
    for _ in 0..100 {
        assert!(hold.sample(true, false, clock(0.)));
    }
    assert!(hold.sample(true, false, clock(1.)));
    assert!(hold.sample(
        true,
        false,
        GuardClock {
            stopped: true,
            ..clock(1.)
        }
    ));
    assert!(hold.sample(true, false, clock(3.)));
    assert!(!hold.sample(true, false, clock(4.)));
    assert!(hold.sample(true, true, clock(5.)));
    assert!(
        !hold.sample(false, true, clock(5.)),
        "release/readiness/cancellation wins"
    );
    assert!(!hold.sample(true, false, clock(6.)));
}

#[test]
fn unknown_or_rewound_clocks_do_not_leave_a_shield_latched() {
    use kirby_assist_core::autopilot_guard::{GuardClock, GuardHold};
    let c = GuardClock {
        status: 0x1c,
        motion: 123,
        frame: 10.,
        rate: 1.,
        stopped: false,
    };
    let mut hold = GuardHold::default();
    assert!(hold.sample(true, true, c));
    assert!(!hold.sample(true, false, GuardClock { frame: 0., ..c }));
    assert!(hold.sample(true, true, c));
    assert!(!hold.sample(true, false, GuardClock { motion: 0, ..c }));
}
