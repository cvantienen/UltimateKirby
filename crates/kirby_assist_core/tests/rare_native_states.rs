use kirby_assist_core::{
    battle_objects::{append_fighter, insert_nearest, FighterRoster},
    gameplay::{GameplayContext, Opponent},
    ledge_trap::{InputTimeline, LedgeSettings},
    native_input::MAPPER_B,
    native_states,
    observation::RosterSnapshot,
    reaction_rules::{
        automatic_target_available, automatic_up_b_status, AutomaticObservationScope,
    },
    sequence_config::Catalog,
};
fn ctx(status: i32, frame: f32) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        can_act: true,
        status,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.,
        facing: 1.,
        ..Default::default()
    }
}
fn tape() -> InputTimeline {
    let c =
        Catalog::parse("[bowser.roll.test]\nstart_delay=0\nactions=up_b:4,drift:opponent:1:69\n")
            .unwrap();
    InputTimeline::new(
        c.entries()[0].sequence,
        Opponent {
            id: 2,
            ..Default::default()
        },
    )
}
#[test]
fn shield_break_sleep_bind_bury_freeze_and_special_captures_abort_without_resuming_old_inputs() {
    let states = [
        0x24, 0x5c, 0x5d, 0x5e, 0x5f, 0x60, 0x61, 0x62, 0x65, 0x66, 0x86, 0xab, 0xb3, 0xb9, 0xbd,
        0xbe, 0xc1, 0xc4, 0xc5, 0xc8, 0xc9, 0xcb, 0xcc, 0xcd, 0xd6, 0xe6, 0xed, 0xf0, 0xf3, 0xf4,
        0xf7, 0xfe, 0x108, 0x10e, 0x117, 0x146, 0x148, 0x156, 0x171, 0x176, 0x183, 0x18a, 0x19c,
        0x1a3, 0x1a8, 0x1be, 0x1bf, 0x1ca, 0x1d2, 0x1d6, 0x1d9, 0x1da, 0x1e0,
    ];
    for status in states {
        for stopped in [false, true] {
            let mut t = tape();
            assert_eq!(
                t.sample(LedgeSettings::default(), ctx(0, 0.), None, None, 0.)
                    .requested,
                MAPPER_B
            );
            let out = t.sample(
                LedgeSettings::default(),
                GameplayContext {
                    stopped,
                    ..ctx(status, 1.)
                },
                None,
                None,
                0.,
            );
            assert_eq!(out.owned, 0, "status {status:#x}");
            assert_eq!(out.requested, 0, "status {status:#x}");
            assert_eq!(out.stick, None);
            assert!(t.finished());
            assert_eq!(
                t.sample(LedgeSettings::default(), ctx(0, 2.), None, None, 0.)
                    .requested,
                0
            );
            let scope = AutomaticObservationScope::new(
                false,
                status,
                kirby_assist_core::native_states::status::WAIT,
                stopped,
                false,
                false,
                false,
                false,
                true,
            );
            assert!(!scope.acquire && !scope.follow, "status {status:#x}");
        }
    }
}
#[test]
fn stunned_opponents_remain_punishable_but_capture_and_scene_actors_are_not() {
    use kirby_assist_core::out_of_shield::automatic_grab_target_status_with_shield as grab;
    for status in [
        kirby_assist_core::native_states::status::SHIELD_BREAK_FLY,
        kirby_assist_core::native_states::status::SHIELD_BREAK_FALL,
        kirby_assist_core::native_states::status::SHIELD_BREAK_DOWN,
        kirby_assist_core::native_states::status::FURAFURA,
        kirby_assist_core::native_states::status::DAMAGE_SONG_START,
        kirby_assist_core::native_states::status::BIND,
        kirby_assist_core::native_states::status::BURY,
        0xcb,
        kirby_assist_core::native_states::status::SAVING_DAMAGE,
        0x1a4,
    ] {
        assert!(
            automatic_target_available(status),
            "hittable status {status:#x}"
        );
    }
    assert!(
        grab(0x60, true),
        "dizzy standing opponent remains grabbable"
    );
    for status in [
        kirby_assist_core::native_states::status::DOWN,
        kirby_assist_core::native_states::status::DOWN_WAIT,
        kirby_assist_core::native_states::status::SHIELD_BREAK_DOWN,
        kirby_assist_core::native_states::status::SLIP,
        kirby_assist_core::native_states::status::BURY,
        0xc9,
        0xcb,
        kirby_assist_core::native_states::status::ICE_JUMP,
    ] {
        assert!(!grab(status, true));
    }
    for status in [
        kirby_assist_core::native_states::status::CAPTURE_WAIT,
        kirby_assist_core::native_states::status::CLUNG_GANON,
        kirby_assist_core::native_states::status::CATCHED_GANON,
        0xc4,
        0xc6,
        kirby_assist_core::native_states::status::SWALLOWED,
        0xed,
        0xf4,
        kirby_assist_core::native_states::status::CATCHED_REFLET,
        0x10e,
        0x124,
        0x146,
        0x156,
        0x172,
        0x184,
        0x18b,
        0x19c,
        kirby_assist_core::native_states::status::CATCHED_PICKEL_TROLLEY,
        0x1ca,
        kirby_assist_core::native_states::status::STANDBY,
        0x1da,
        kirby_assist_core::native_states::status::FINAL,
    ] {
        assert!(!automatic_target_available(status), "status {status:#x}");
        assert!(!grab(status, true));
    }
    for status in [
        kirby_assist_core::native_states::status::DOWN_STAND,
        kirby_assist_core::native_states::status::DOWN_STAND_FB,
        kirby_assist_core::native_states::status::DOWN_STAND_ATTACK,
        kirby_assist_core::native_states::status::PASSIVE,
        kirby_assist_core::native_states::status::PASSIVE_FB,
        kirby_assist_core::native_states::status::SPECIAL_HI,
    ] {
        assert!(
            !native_states::sequence_interrupted(status),
            "ordinary recovery {status:#x}"
        );
    }
}
#[test]
fn custom_status_numbers_cannot_borrow_another_fighters_up_b_identity() {
    for kind in 0..kirby_assist_core::fighter_kinds::PLAYABLE_KIND_COUNT {
        assert!(automatic_up_b_status(kind, 0x1de, 0));
        for status in [
            kirby_assist_core::native_states::status::FINAL,
            0x3ff,
            0x7fffffff,
        ] {
            assert!(!automatic_up_b_status(
                kind,
                status,
                kirby_assist_core::native_states::status::SPECIAL_HI
            ));
        }
    }
    for (kind, status) in [
        (15, 0x1e1),
        (15, 0x1e2),
        (28, 0x1e1),
        (6, 0x1fc),
        (75, 0x1e4),
        (76, 0x1e4),
        (43, 0x1e5),
        (90, 0x1eb),
        (91, 0x1ed),
        (88, 0x20c),
    ] {
        assert!(automatic_up_b_status(
            kind,
            status,
            kirby_assist_core::native_states::status::WAIT
        ));
    }
    for (kind, status) in [
        (90, 0x1e1),
        (91, 0x1e1),
        (28, 0x1e3),
        (15, 0x1e3),
        (43, 0x1e1),
    ] {
        assert!(!automatic_up_b_status(
            kind,
            status,
            kirby_assist_core::native_states::status::SPECIAL_HI
        ));
    }
}
#[test]
fn sixteen_actor_roster_is_deduplicated_and_policy_candidates_stay_bounded_and_nearby() {
    let mut roster: FighterRoster = [None; 16];
    for id in 0..16 {
        append_fighter(&mut roster, id);
        append_fighter(&mut roster, id);
    }
    for id in [0x10000001, 0x40000001, u32::MAX] {
        append_fighter(&mut roster, id);
    }
    assert_eq!(roster, std::array::from_fn(|i| Some(i as u32)));
    let mut targets = [None; 8];
    for id in (0..16u32).rev() {
        insert_nearest(&mut targets, id, |id| id as f32);
    }
    let mut ids = targets.into_iter().flatten().collect::<Vec<_>>();
    ids.sort_unstable();
    assert_eq!(ids, (0..8).collect::<Vec<_>>());
    insert_nearest(&mut targets, 100, |_| f32::NAN);
    assert!(targets.into_iter().flatten().all(|id| id < 8));
    let snapshot = RosterSnapshot::<16>::default();
    let mut reads = 0;
    for _ in 0..100 {
        assert_eq!(
            snapshot.read(|| {
                reads += 1;
                Some(roster)
            }),
            Some(roster)
        );
    }
    assert_eq!(reads, 1);
    roster[1] = Some(77);
    assert_eq!(
        RosterSnapshot::<16>::default().read(|| Some(roster)),
        Some(roster)
    );
}

#[test]
fn private_damage_and_swap_states_are_scoped_to_their_actual_native_fighter() {
    for (kind, status) in [
        (39, 0x1f1),
        (39, 0x1f3),
        (53, 0x1e8),
        (58, 0x1eb),
        (36, 0x1e1),
        (37, 0x1e1),
        (38, 0x1e1),
        (90, 0x1e1),
        (91, 0x1e2),
    ] {
        let mut t = tape();
        let settings = LedgeSettings {
            fighter_kind: kind,
            ..Default::default()
        };
        t.sample(settings, ctx(0, 0.), None, None, 0.);
        let out = t.sample(settings, ctx(status, 1.), None, None, 0.);
        assert_eq!(out.owned, 0);
        assert!(t.finished());
        assert!(!automatic_up_b_status(
            kind,
            status,
            kirby_assist_core::native_states::status::SPECIAL_HI
        ));
    }
    assert!(!native_states::fighter_sequence_interrupted(15, 0x1e1));
    assert!(automatic_up_b_status(15, 0x1e1, 0));
}

#[test]
fn both_climbers_status_changes_survive_a_full_sixteen_actor_observation() {
    use kirby_assist_core::observation::{Fact, Field, ObservationSnapshotSized};
    let facts = ObservationSnapshotSized::<16>::default();
    for id in 0..16 {
        facts.read(id, Field::Status, || Fact::Int(0x1c));
    }
    let known = facts.known_statuses();
    for id in 0..16 {
        assert!(known.contains(&Some((id, 0x1c))));
    }
    assert_eq!(known.into_iter().flatten().count(), 16);
}

#[test]
fn shared_climber_entry_never_invents_attacker_direction_from_iteration_order() {
    use kirby_assist_core::battle_objects::EntryDirection;
    for sides in [[1., -1.], [-1., 1.]] {
        let mut direction = EntryDirection::default();
        for side in sides {
            direction.observe(side);
        }
        assert_eq!(direction.direction(), None);
        direction.observe(1.);
        assert_eq!(
            direction.direction(),
            None,
            "a third observation cannot erase ambiguity"
        );
    }
    let mut direction = EntryDirection::default();
    direction.observe(-1.);
    direction.observe(-1.);
    assert_eq!(direction.direction(), Some(-1.));
}

#[test]
fn final_phases_and_inactive_forms_cannot_masquerade_as_ordinary_grab_targets() {
    use kirby_assist_core::reaction_rules::automatic_fighter_target_available;
    for (kind, status) in [
        (15, 0x1ea),
        (28, 0x1e9),
        (75, 0x1ed),
        (76, 0x1ed),
        (90, 0x1e3),
        (91, 0x1ef),
        (36, 0x1e1),
        (90, 0x1e1),
    ] {
        assert!(!automatic_fighter_target_available(kind, status));
        assert!(native_states::fighter_sequence_interrupted(kind, status));
    }
    assert!(
        automatic_fighter_target_available(90, 0x1e2),
        "active swap-out recovery may be punished when native hit status is vulnerable"
    );
    assert!(
        automatic_fighter_target_available(15, 0x1e1),
        "Bowser spin shares a swap form's number"
    );
    assert!(
        automatic_fighter_target_available(90, 0x1eb),
        "Pyra Up-B is not another fighter's Final"
    );
}

#[test]
fn consumed_rolls_for_all_sixteen_actors_survive_candidate_pool_changes() {
    use kirby_assist_core::{
        ledge_trap::MotionTiming,
        reaction_rules::{Candidate, Engine, Resolved},
        roll_read::Target,
    };
    let sequence = Catalog::parse(
        "[bowser.roll.auto]\nstart_delay=0\nreaction=auto_roll:40:12\nactions=input:a:0:0:1\n",
    )
    .unwrap()
    .entries()[0]
        .sequence;
    let mut rules = Resolved::default();
    rules
        .add(Candidate {
            slot: 2,
            index: 0,
            sequence,
        })
        .unwrap();
    let mut engine = Engine::default();
    let settings = LedgeSettings {
        fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
        ..Default::default()
    };
    for id in 1..=16 {
        let t = Target {
            id,
            status: kirby_assist_core::native_states::ESCAPE_F,
            motion: 2,
            frame: 1.,
            rate: 1.,
            position: [10., 0.],
            velocity_x: 0.,
            vulnerable_in: None,
            actionable_in: None,
        };
        let e = Opponent {
            id,
            status: kirby_assist_core::native_states::ESCAPE_F,
            position: [10., 0.],
            motion: Some(MotionTiming {
                kind: 2,
                frame: 1.,
                rate: 1.,
                remaining: None,
            }),
            ..Default::default()
        };
        let mut targets = [None; 8];
        targets[0] = Some(t);
        assert_ne!(
            engine
                .sample_simple(
                    true,
                    &rules,
                    settings,
                    ctx(0, id as f32 * 4.),
                    targets,
                    Some(e)
                )
                .requested,
            0
        );
        for frame in 1..=3 {
            engine.sample_simple(
                true,
                &rules,
                settings,
                ctx(0, id as f32 * 4. + frame as f32),
                [None; 8],
                Some(e),
            );
        }
    }
    for id in 1..=16 {
        let mut targets = [None; 8];
        targets[0] = Some(Target {
            id,
            status: kirby_assist_core::native_states::ESCAPE_F,
            motion: 2,
            frame: 100.,
            rate: 1.,
            position: [10., 0.],
            velocity_x: 0.,
            vulnerable_in: None,
            actionable_in: None,
        });
        let out = engine.sample_simple(
            true,
            &rules,
            settings,
            ctx(0, 100. + id as f32),
            targets,
            None,
        );
        assert_eq!(
            out.requested, 0,
            "consumed actor {id} must not be rearmed by pool overflow"
        );
    }
}
