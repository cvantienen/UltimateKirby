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
        autopilot_guard::referenced_imminent_attack_with, fighter_data::named_move,
        timing::motion_hash,
    };
    let down_tilt = named_move(6, "Down Tilt").unwrap();
    let motion = motion_hash("attack_lw3");
    assert!(down_tilt.motions.contains(&motion));
    assert!(referenced_imminent_attack_with(6, motion, 1., 1., 5.));
    assert!(!referenced_imminent_attack_with(6, motion, 0., 1., 2.));

    let neutral_air = named_move(6, "Neutral Air").unwrap();
    let motion = motion_hash("attack_air_n");
    assert!(neutral_air.motions.contains(&motion));
    assert!(referenced_imminent_attack_with(6, motion, 5., 1., 5.));
    let forward_air = named_move(6, "Forward Air").unwrap();
    assert!(referenced_imminent_attack_with(
        6,
        forward_air.motions[0],
        5.,
        1.,
        5.
    ));

    let grab = named_move(6, "Grab").unwrap();
    assert!(!grab.has_damage_data());
    assert!(!referenced_imminent_attack_with(6, u64::MAX, 5., 1., 5.));
    assert!(!referenced_imminent_attack_with(
        6,
        motion,
        f32::NAN,
        1.,
        5.
    ));

    let samus_smash = named_move(3, "Down Smash").unwrap();
    assert!(samus_smash.active.len() > 1);
    let gap = samus_smash.active[0].last + 1;
    assert!(!referenced_imminent_attack_with(
        3,
        motion_hash("attack_lw4"),
        f32::from(gap),
        1.,
        0.
    ));
}

#[test]
fn guard_tail_bridges_gaps_in_native_frames_and_release_cancels_immediately() {
    use kirby_assist_core::autopilot_guard::{GuardClock, GuardHold};
    let clock = |frame| GuardClock {
        status: kirby_assist_core::native_states::GUARD,
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
        status: kirby_assist_core::native_states::GUARD,
        motion: 123,
        frame: 10.,
        rate: 1.,
        stopped: false,
    };
    let mut hold = GuardHold::default();
    assert!(hold.sample(true, true, c));
    assert!(!hold.sample(true, false, GuardClock { frame: 0., ..c }));
    assert!(hold.sample(true, true, c));
    assert!(!hold.sample(
        true,
        false,
        GuardClock {
            motion: kirby_assist_core::timing::NO_MOTION_KIND,
            ..c
        }
    ));
}
