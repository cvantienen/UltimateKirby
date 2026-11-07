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
    // The remaining raw values are intentionally unmapped native status IDs;
    // this regression exercises the conservative fallback for unknown states.
    let states = [
        native_states::status::REBOUND_STOP, native_states::status::SHIELD_BREAK_FLY, native_states::status::SHIELD_BREAK_FALL, native_states::status::SHIELD_BREAK_DOWN, native_states::status::FURAFURA_STAND, native_states::status::FURAFURA, native_states::status::FURAFURA_END, native_states::status::DAMAGE_SONG_START, native_states::status::DAMAGE_SONG_END, native_states::status::BIND, native_states::status::SLIP, native_states::status::SWIM_DIVE, 0xb3, native_states::status::TREAD_DAMAGE, native_states::status::CLUNG_GANON,
        native_states::status::CATCHED_GANON, native_states::status::CATCHED_AIR_END_GANON, 0xc4, 0xc5, native_states::status::BURY, 0xc9, 0xcb, native_states::status::ICE_JUMP, 0xcd, 0xd6, native_states::status::SWALLOWED, 0xed, 0xf0, 0xf3, 0xf4,
        native_states::status::CAPTURE_DAMAGE_YOSHI, native_states::status::CATCHED_REFLET, 0x108, 0x10e, 0x117, 0x146, native_states::status::SAVING_DAMAGE, 0x156, native_states::status::CATCHED_RIDLEY, native_states::status::SWING_GAOGAEN_CATCHED, 0x183, 0x18a, 0x19c,
        native_states::status::DAMAGE_SLEEP_START, 0x1a8, native_states::status::CATCHED_PICKEL_TROLLEY, 0x1bf, 0x1ca, native_states::status::CAPTURE_TRAIL_KEYHOLE, native_states::status::STANDBY, native_states::status::ENTRY, 0x1da, native_states::status::FINAL,
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
        assert!(automatic_up_b_status(kind, native_states::status::SPECIAL_HI, 0));
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
        (kirby_assist_core::fighter_kinds::BOWSER, kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_G),
        (kirby_assist_core::fighter_kinds::BOWSER, kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_A),
        (kirby_assist_core::fighter_kinds::MR_GAME_AND_WATCH, kirby_assist_core::native_states::fighter_status::GAMEWATCH_SPECIAL_HI_FALL),
        (kirby_assist_core::fighter_kinds::KIRBY, kirby_assist_core::native_states::fighter_status::KIRBY_SPECIAL_HI2),
        (kirby_assist_core::fighter_kinds::ICE_CLIMBERS_POPO, kirby_assist_core::native_states::fighter_status::POPO_SPECIAL_HI_PARTNER),
        (kirby_assist_core::fighter_kinds::ICE_CLIMBERS_NANA, kirby_assist_core::native_states::fighter_status::POPO_SPECIAL_HI_PARTNER),
        (kirby_assist_core::fighter_kinds::OLIMAR, kirby_assist_core::native_states::fighter_status::PIKMIN_SPECIAL_HI_WAIT),
        (kirby_assist_core::fighter_kinds::PYRA, kirby_assist_core::native_states::fighter_status::EFLAME_SPECIAL_HI_JUMP),
        (kirby_assist_core::fighter_kinds::MYTHRA, kirby_assist_core::native_states::fighter_status::ELIGHT_SPECIAL_HI_ATTACK2),
        (kirby_assist_core::fighter_kinds::STEVE, kirby_assist_core::native_states::fighter_status::PICKEL_SPECIAL_HI_GLIDING),
    ] {
        assert!(automatic_up_b_status(
            kind,
            status,
            kirby_assist_core::native_states::status::WAIT
        ));
    }
    for (kind, status) in [
        (kirby_assist_core::fighter_kinds::PYRA, kirby_assist_core::native_states::fighter_status::EFLAME_SPECIAL_LW_STANDBY),
        (kirby_assist_core::fighter_kinds::MYTHRA, kirby_assist_core::native_states::fighter_status::ELIGHT_SPECIAL_LW_STANDBY),
        (kirby_assist_core::fighter_kinds::MR_GAME_AND_WATCH, kirby_assist_core::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_WAIT_START),
        (kirby_assist_core::fighter_kinds::BOWSER, kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_LW_G),
        (kirby_assist_core::fighter_kinds::OLIMAR, kirby_assist_core::native_states::fighter_status::PIKMIN_FINAL_FLY),
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
        (kirby_assist_core::fighter_kinds::DIDDY_KONG, kirby_assist_core::native_states::fighter_status::DIDDY_SPECIAL_HI_CHARGE_DAMAGE),
        (kirby_assist_core::fighter_kinds::DIDDY_KONG, kirby_assist_core::native_states::fighter_status::DIDDY_SPECIAL_HI_UPPER_DAMAGE),
        (kirby_assist_core::fighter_kinds::GRENINJA, kirby_assist_core::native_states::fighter_status::GEKKOUGA_SPECIAL_HI_WALL_DAMAGE),
        (kirby_assist_core::fighter_kinds::BOWSER_JR, kirby_assist_core::native_states::fighter_status::KOOPAJR_SPECIAL_HI_DAMAGE_END),
        (kirby_assist_core::fighter_kinds::SQUIRTLE, kirby_assist_core::native_states::fighter_status::PZENIGAME_SPECIAL_LW_STANDBY),
        (kirby_assist_core::fighter_kinds::IVYSAUR, kirby_assist_core::native_states::fighter_status::PFUSHIGISOU_SPECIAL_LW_STANDBY),
        (kirby_assist_core::fighter_kinds::CHARIZARD, kirby_assist_core::native_states::fighter_status::PLIZARDON_SPECIAL_LW_STANDBY),
        (kirby_assist_core::fighter_kinds::PYRA, kirby_assist_core::native_states::fighter_status::EFLAME_SPECIAL_LW_STANDBY),
        (kirby_assist_core::fighter_kinds::MYTHRA, kirby_assist_core::native_states::fighter_status::ELIGHT_SPECIAL_LW_OUT),
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
    assert!(!native_states::fighter_sequence_interrupted(kirby_assist_core::fighter_kinds::BOWSER, kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_G));
    assert!(automatic_up_b_status(kirby_assist_core::fighter_kinds::BOWSER, native_states::fighter_status::KOOPA_SPECIAL_HI_G, 0));
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
        (kirby_assist_core::fighter_kinds::BOWSER, kirby_assist_core::native_states::fighter_status::KOOPA_FINAL_END),
        (kirby_assist_core::fighter_kinds::MR_GAME_AND_WATCH, kirby_assist_core::native_states::fighter_status::GAMEWATCH_FINAL_WAIT),
        (kirby_assist_core::fighter_kinds::ICE_CLIMBERS_POPO, kirby_assist_core::native_states::fighter_status::POPO_FINAL_PARTNER),
        (kirby_assist_core::fighter_kinds::ICE_CLIMBERS_NANA, kirby_assist_core::native_states::fighter_status::POPO_FINAL_PARTNER),
        (kirby_assist_core::fighter_kinds::PYRA, kirby_assist_core::native_states::fighter_status::EFLAME_FINAL_READY),
        (kirby_assist_core::fighter_kinds::MYTHRA, kirby_assist_core::native_states::fighter_status::ELIGHT_FINAL_SCENE03),
        (kirby_assist_core::fighter_kinds::SQUIRTLE, kirby_assist_core::native_states::fighter_status::PZENIGAME_SPECIAL_LW_STANDBY),
        (kirby_assist_core::fighter_kinds::PYRA, kirby_assist_core::native_states::fighter_status::EFLAME_SPECIAL_LW_STANDBY),
    ] {
        assert!(!automatic_fighter_target_available(kind, status));
        assert!(native_states::fighter_sequence_interrupted(kind, status));
    }
    assert!(
        automatic_fighter_target_available(kirby_assist_core::fighter_kinds::PYRA, kirby_assist_core::native_states::fighter_status::EFLAME_SPECIAL_LW_OUT),
        "active swap-out recovery may be punished when native hit status is vulnerable"
    );
    assert!(
        automatic_fighter_target_available(kirby_assist_core::fighter_kinds::BOWSER, kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_G),
        "Bowser spin shares a swap form's number"
    );
    assert!(
        automatic_fighter_target_available(kirby_assist_core::fighter_kinds::PYRA, kirby_assist_core::native_states::fighter_status::EFLAME_SPECIAL_HI_JUMP),
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
