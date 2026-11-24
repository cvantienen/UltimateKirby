//! Shield release is shared by every file-authored sequence consumer.
use kirby_assist_core::{
    defense_response::Response,
    gameplay::{GameplayContext, Opponent},
    ledge_trap::InputTimeline,
    native_input::*,
    reaction_rules::{Candidate, Resolved, Trigger},
    sequence_config::Catalog,
    settings::AssistSettings,
};
fn context(status: i32, frame: f32) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        status,
        situation: kirby_assist_core::native_states::situation::GROUND,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.,
        facing: 1.,
        ..Default::default()
    }
}
fn sequence(actions: &str) -> kirby_assist_core::sequence_config::Sequence {
    Catalog::parse(&format!(
        "[bowser.attack.test]\nstart_delay=0\ntimeout=60\nactions={actions}\n"
    ))
    .unwrap()
    .entries()[0]
        .sequence
}
#[test]
fn block_to_down_smash_releases_neutral_until_guard_off_ends_then_holds_full_duration() {
    let mut t = InputTimeline::new(
        sequence("shield_until_block:30,down_smash:4"),
        Opponent::default(),
    );
    let sample = |t: &mut InputTimeline, status, frame| {
        t.sample(
            AssistSettings::default(),
            context(status, frame),
            None,
            None,
            0.,
        )
    };
    assert_eq!(
        sample(&mut t, kirby_assist_core::native_states::GUARD, 0.).requested,
        MAPPER_ZR
    );
    assert_eq!(
        sample(&mut t, kirby_assist_core::native_states::GUARD_DAMAGE, 1.).requested,
        MAPPER_ZR
    );
    for (status, frame) in [
        (kirby_assist_core::native_states::GUARD, 2.),
        (kirby_assist_core::native_states::GUARD_OFF, 3.),
        (kirby_assist_core::native_states::GUARD_OFF, 4.),
    ] {
        let o = sample(&mut t, status, frame);
        assert_eq!(o.requested, 0);
        assert_eq!(o.stick, Some([0.; 2]));
        assert_eq!(t.right_stick(), [0.; 2]);
        assert!(!t.finished());
    }
    for frame in 5..9 {
        let o = sample(&mut t, 0, frame as f32);
        assert_eq!(o.requested, MAPPER_A);
        assert_eq!(o.stick, Some([0., -1.]));
    }
    assert_eq!(sample(&mut t, 0, 9.).requested, 0);
    assert!(t.finished());
}
#[test]
fn direct_oos_inputs_do_not_gain_an_extra_release_frame() {
    for (action, button) in [
        ("input:r:0:0:4", MAPPER_R),
        ("up_b:4", MAPPER_B),
        ("up_smash:4", MAPPER_A),
        ("short_hop:1", MAPPER_X),
        ("input:zl:0:0:1", MAPPER_ZL),
    ] {
        let mut t = InputTimeline::new(
            sequence(&format!("shield_until_block:30,{action}")),
            Opponent::default(),
        );
        assert_eq!(
            t.sample(
                AssistSettings::default(),
                context(kirby_assist_core::native_states::GUARD, 0.),
                None,
                None,
                0.,
            )
            .requested,
            MAPPER_ZR,
            "{action}: initial shield"
        );
        assert_eq!(
            t.sample(
                AssistSettings::default(),
                context(kirby_assist_core::native_states::GUARD_DAMAGE, 1.),
                None,
                None,
                0.,
            )
            .requested,
            MAPPER_ZR,
            "{action}: shield hit"
        );
        let o = t.sample(
            AssistSettings::default(),
            context(kirby_assist_core::native_states::GUARD, 2.),
            None,
            None,
            0.,
        );
        assert_eq!(o.requested & button, button, "{action}");
        assert_eq!(o.requested & MAPPER_ZR,MAPPER_ZR,"{action}: direct grab, special, up-smash and jump keep guard through native OOS cancellation");
    }
}
#[test]
fn automatic_attacks_buffer_after_shield_release_without_spending_their_hold() {
    for action in ["down_smash:4", "side_b:4", "cstick:0:-1:4"] {
        let mut t = InputTimeline::new(sequence(action), Opponent::default());
        t.allow_continuous_progress();
        let sample = |t: &mut InputTimeline, status, frame| {
            t.sample(
                AssistSettings::default(),
                context(status, frame),
                None,
                None,
                0.,
            )
        };
        let released = sample(&mut t, kirby_assist_core::native_states::GUARD, 0.);
        assert_eq!(released.requested, 0, "{action}");
        assert_eq!(released.stick, Some([0.; 2]));
        let buffered = sample(&mut t, kirby_assist_core::native_states::GUARD_OFF, 1.);
        assert_ne!(buffered.requested, 0, "{action}");
        assert_eq!(buffered.requested & SEQUENCE_GUARD_INPUTS, 0, "{action}");
        for frame in [1., 2., 5., 10.] {
            assert_eq!(
                sample(&mut t, kirby_assist_core::native_states::GUARD_OFF, frame),
                buffered,
                "{action}: recovery and duplicate polls preserve the buffered input"
            );
            assert!(!t.finished());
        }
        for frame in 11..15 {
            assert_eq!(sample(&mut t, 0, frame as f32), buffered, "{action}");
        }
        assert_eq!(sample(&mut t, 0, 15.).requested, 0, "{action}");
        assert!(t.finished());
    }
}

#[test]
fn every_auto_react_event_buffers_down_smash_but_keeps_up_smash_immediate() {
    use kirby_assist_core::{reaction_rules::Engine, roll_read::Target};
    for (event, status) in [
        ("auto_roll", kirby_assist_core::native_states::ESCAPE_F),
        ("auto_spot_dodge", kirby_assist_core::native_states::ESCAPE),
        (
            "auto_air_dodge",
            kirby_assist_core::native_states::ESCAPE_AIR,
        ),
    ] {
        for (action, first) in [("down_smash:1", 0), ("up_smash:1", MAPPER_A | MAPPER_ZR)] {
            let mut s = sequence(action);
            s.reaction =
                kirby_assist_core::reaction_rules::Rule::parse(&format!("{event}:30:12")).unwrap();
            let mut rules = Resolved::default();
            rules
                .add(Candidate {
                    slot: 0,
                    index: 0,
                    sequence: s,
                })
                .unwrap();
            let target = Target {
                id: 2,
                status,
                motion: 2,
                frame: 0.,
                rate: 1.,
                position: [10., 0.],
                velocity_x: 0.,
                vulnerable_in: None,
                actionable_in: None,
            };
            let mut engine = Engine::default();
            let out = engine.sample_simple_ready(
                true,
                &rules,
                AssistSettings::default(),
                context(kirby_assist_core::native_states::GUARD, 0.),
                [Some(target), None, None, None, None, None, None, None],
                None,
                [true; 3],
            );
            assert_eq!(out.requested, first, "{event}, {action}");
            if first == 0 {
                let out = engine.sample_simple_ready(
                    true,
                    &rules,
                    AssistSettings::default(),
                    context(kirby_assist_core::native_states::GUARD_OFF, 1.),
                    [None; 8],
                    Some(Opponent {
                        id: 2,
                        status,
                        position: [10., 0.],
                        ..Default::default()
                    }),
                    [false; 3],
                );
                assert_eq!(out.requested, MAPPER_A, "{event}");
                assert_eq!(out.stick, Some([0., -1.]));
                assert!(engine.committed());
            }
        }
    }
}

#[test]
fn automatic_buffer_does_not_feed_attacks_into_shieldstun_or_shield_hold() {
    let mut t = InputTimeline::new(sequence("down_smash:1"), Opponent::default());
    t.allow_continuous_progress();
    for (status, frame, requested) in [
        (kirby_assist_core::native_states::GUARD, 0., 0),
        (kirby_assist_core::native_states::GUARD_OFF, 1., MAPPER_A),
        (kirby_assist_core::native_states::GUARD_DAMAGE, 2., 0),
        (kirby_assist_core::native_states::GUARD, 3., 0),
        (kirby_assist_core::native_states::GUARD_OFF, 4., MAPPER_A),
        (0, 5., MAPPER_A),
        (0, 6., 0),
    ] {
        let out = t.sample(
            AssistSettings::default(),
            context(status, frame),
            None,
            None,
            0.,
        );
        assert_eq!(out.requested, requested, "status {status}, frame {frame}");
        assert_eq!(out.requested & SEQUENCE_GUARD_INPUTS, 0);
        if status == kirby_assist_core::native_states::GUARD_OFF {
            let stopped = t.sample(
                AssistSettings::default(),
                GameplayContext {
                    stopped: true,
                    status: kirby_assist_core::native_states::GUARD_DAMAGE,
                    ..context(status, frame)
                },
                None,
                None,
                0.,
            );
            assert_eq!(stopped.requested, 0);
            assert_eq!(stopped.stick, Some([0.; 2]));
        }
    }
    assert!(t.finished());
}

#[test]
fn shared_response_and_movement_cannot_feed_directions_into_shield() {
    for action in [
        "down_smash:4",
        "side_b:4",
        "run:opponent:5",
        "drift:opponent:1:4",
        "cstick:0:-1:4",
    ] {
        let mut s = sequence(action);
        s.reaction =
            kirby_assist_core::reaction_rules::Rule::parse("shield_hit:grounded:0:30:12:any:1")
                .unwrap();
        let mut rules = Resolved::default();
        rules
            .add(Candidate {
                slot: 0,
                index: 0,
                sequence: s,
            })
            .unwrap();
        let enemy = Opponent {
            id: 2,
            position: [10., 0.],
            ..Default::default()
        };
        let mut r = Response::new();
        assert!(r.start(
            &rules,
            Trigger::ShieldHit,
            context(kirby_assist_core::native_states::GUARD, 0.),
            enemy
        ));
        for (status, frame) in [
            (kirby_assist_core::native_states::GUARD, 0.),
            (kirby_assist_core::native_states::GUARD_OFF, 1.),
        ] {
            let o = r.sample(
                true,
                AssistSettings::default(),
                context(status, frame),
                Some(enemy),
            );
            assert_eq!(o.requested, 0, "{action}");
            assert_eq!(o.stick, Some([0.; 2]));
            assert_eq!(r.right_stick(), Some([0.; 2]));
        }
    }
}
