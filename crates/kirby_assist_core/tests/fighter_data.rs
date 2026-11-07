use kirby_assist_core::{
    fighter_data::{self, FighterDataCache, RecoveryObservation},
    timing::motion_hash,
};
#[test]
fn roster_has_exact_kinds_stats_and_gap_preserving_move_facts() {
    let all = fighter_data::all();
    assert_eq!(all.len(), 90);
    assert!(all.windows(2).all(|p| p[0].kind < p[1].kind));
    for p in all {
        assert!(p.numeric_stat("weight.weight").is_some(), "{}", p.name);
        assert!(
            p.numeric_stat("walkspeed.walkspeed").is_some(),
            "{}",
            p.name
        );
        assert!(p.moves.len() >= 10);
        for m in &p.moves {
            for s in &m.active {
                assert!(s.first > 0 && s.first <= s.last);
            }
            if m.whiff_eligible {
                assert!(m.total.unwrap() > m.last_active().unwrap());
            }
        }
    }
    let kirby = fighter_data::fighter(6).unwrap();
    assert!(
        kirby.move_for_motion(motion_hash("attack_s3_hi")).is_none(),
        "an angled variant must not inherit neutral-angle timing by guesswork"
    );
    let yoshi = fighter_data::fighter(5).unwrap();
    assert_eq!(kirby.numeric_stat("weight.weight"), Some(79.0));
    assert_eq!(yoshi.numeric_stat("airspeed.airspeed"), Some(1.344));
    let bair = yoshi.move_for_motion(motion_hash("attack_air_b")).unwrap();
    assert_eq!(
        bair.active
            .iter()
            .map(|s| (s.first, s.last))
            .collect::<Vec<_>>(),
        vec![(11, 11), (14, 14), (18, 19)]
    );
    assert_eq!(bair.landing_lag, Some(11));
    assert!(fighter_data::fighter(-1).is_none());
    assert!(fighter_data::fighter(0x4d).is_none());
    assert!(fighter_data::fighter(0x77).is_none()); // Random, never Kirby fallback.
    let mut cache = FighterDataCache::new();
    assert_eq!(cache.bind(0, 1, 6).unwrap().kind, 6);
    assert_eq!(cache.bind(0, 1, 5).unwrap().kind, 5); // Transformation/identity change.
    assert!(cache.bind(0, 2, 0x77).is_none());
    assert!(cache.bind(8, 1, 6).is_none());
}

#[test]
fn fighter_data_is_available_independent_of_match_role() {
    for (kind, expected_name) in [
        (0, "mario"),
        (6, "kirby"),
        (36, "squirtle"),
        (43, "olimar"),
        (81, "piranha_plant"),
        (93, "sora"),
    ] {
        let data = fighter_data::fighter(kind).unwrap();
        assert_eq!(data.name, expected_name);
        assert!(std::ptr::eq(fighter_data::fighter(kind).unwrap(), data));
    }
}

#[test]
fn ordinary_grab_lookup_uses_each_fighters_own_data_including_dlc() {
    for fighter in fighter_data::all() {
        for status in [0x37, 0x39, 0x3b] {
            assert!(
                fighter_data::ordinary_grab(fighter.kind, status).is_some(),
                "missing ordinary grab status {status:#x} for {}",
                fighter.name
            );
        }
    }
    assert_eq!(
        fighter_data::ordinary_grab(6, 0x37),
        Some(fighter_data::GrabReference {
            first: 6,
            last: 7,
            reach: Some(12.5),
        })
    );
    assert_eq!(fighter_data::ordinary_grab(43, 0x37).unwrap().first, 12);
    assert_eq!(
        fighter_data::ordinary_grab(82, 0x39),
        Some(fighter_data::GrabReference {
            first: 8,
            last: 9,
            reach: Some(12.3),
        }),
        "Joker must not inherit Kirby's dash-grab timing"
    );
    assert!(fighter_data::ordinary_grab(82, 0x1f).is_none());
    assert!(fighter_data::ordinary_grab(0x77, 0x37).is_none());
}
#[test]
fn recovery_rejects_contact_active_hits_cancel_and_changed_animations() {
    let m = fighter_data::fighter(6)
        .unwrap()
        .move_for_motion(motion_hash("attack_lw4"))
        .unwrap();
    let o = RecoveryObservation {
        motion: motion_hash("attack_lw4"),
        status: 0x32,
        grounded: true,
        frame: 20.0,
        rate: 1.0,
        end: 50.0,
        cancel: 0.0,
        can_cancel: false,
        active_attack: false,
        registered_contact: false,
        vulnerable: true,
    };
    assert_eq!(m.recovery(o), Some(30.0));
    assert_eq!(
        m.recovery(RecoveryObservation { rate: 0.5, ..o }),
        Some(60.0)
    );
    assert_eq!(
        m.recovery(RecoveryObservation { cancel: 40.0, ..o }),
        Some(20.0)
    );
    for bad in [
        RecoveryObservation { frame: 19.0, ..o },
        RecoveryObservation {
            registered_contact: true,
            ..o
        },
        RecoveryObservation {
            active_attack: true,
            ..o
        },
        RecoveryObservation {
            can_cancel: true,
            ..o
        },
        RecoveryObservation {
            grounded: false,
            ..o
        },
        RecoveryObservation {
            vulnerable: false,
            ..o
        },
        RecoveryObservation { end: 100.0, ..o },
        RecoveryObservation { cancel: 19.0, ..o },
        RecoveryObservation { rate: 0.0, ..o },
        RecoveryObservation { status: 0x30, ..o },
    ] {
        assert_eq!(m.recovery(bad), None);
    }
}
