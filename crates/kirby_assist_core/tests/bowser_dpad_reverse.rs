use kirby_assist_core::{
    fighter_profiles::FighterProfiles,
    gameplay::{GameplayContext, MotionTiming, Opponent},
    ledge_trap::InputTimeline,
    native_input::MAPPER_B,
    sequence_config::{persist, Catalog},
    settings::AssistSettings,
};
fn context(frame: f32, facing: f32) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        situation: kirby_assist_core::native_states::situation::AIR,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.,
        facing,
        ..Default::default()
    }
}
fn enemy(x: f32, frame: f32) -> Opponent {
    Opponent {
        id: 2,
        position: [x, 0.],
        motion: Some(MotionTiming {
            kind: 2,
            frame,
            rate: 1.,
            remaining: None,
        }),
        ..Default::default()
    }
}
#[test]
fn bowser_defaults_deliver_mirrored_input_edges_without_a_facing_wait() {
    let source = "[bowser.dpad_left.reverse_fire_breath]\nstart_delay=0\ndefault=true\nactions=opponent_input:none:-1:0:1,input:b:0:0:1,opponent_input:none:1:0:2\n[bowser.dpad_right.reverse_flying_slam]\nstart_delay=0\ndefault=true\nactions=opponent_input:b:-1:0:1,opponent_input:none:1:0:2\n";
    let catalog = Catalog::parse(source).unwrap();
    let entries: Vec<_> = catalog
        .entries()
        .iter()
        .filter(|e| {
            e.fighter_kind == kirby_assist_core::fighter_kinds::BOWSER
                && matches!(
                    e.choice.as_str(),
                    "reverse_fire_breath" | "reverse_flying_slam"
                )
        })
        .collect();
    assert_eq!(entries.len(), 2);
    let mut profiles = FighterProfiles::default();
    profiles.add_sequence_choices(&catalog).unwrap();
    for entry in entries {
        assert!(entry.default);
        let saved =
            persist::update(source, 15, entry.slot, &entry.choice, entry.sequence, false).unwrap();
        let parsed = Catalog::parse(&saved).unwrap();
        assert_eq!(
            parsed
                .entries()
                .iter()
                .find(
                    |e| e.fighter_kind == kirby_assist_core::fighter_kinds::BOWSER
                        && e.choice == entry.choice
                )
                .unwrap()
                .sequence
                .actions(),
            entry.sequence.actions()
        );
        for side in [-1., 1.] {
            for facing in [-1., 1.] {
                let mut timeline = InputTimeline::new(entry.sequence, enemy(side * 10., 0.));
                assert!(!timeline.inputs_only());
                let neutral = entry.choice == "reverse_fire_breath";
                for frame in 0..if neutral { 4 } else { 3 } {
                    let frame = frame as f32;
                    let c = context(frame, facing);
                    let output = timeline.sample(
                        AssistSettings::default(),
                        c,
                        Some(enemy(side * 10., frame)),
                        None,
                        0.,
                    );
                    let (x, b) = if neutral {
                        if frame == 0. {
                            (-side, 0)
                        } else if frame == 1. {
                            (0., MAPPER_B)
                        } else {
                            (side, 0)
                        }
                    } else if frame == 0. {
                        (-side, MAPPER_B)
                    } else {
                        (side, 0)
                    };
                    assert_eq!(output.stick, Some([x, 0.]));
                    assert_eq!(output.requested, b);
                    assert_eq!(
                        timeline.sample(
                            AssistSettings::default(),
                            c,
                            Some(enemy(-side * 10., frame)),
                            None,
                            0.
                        ),
                        output
                    );
                }
            }
        }
    }
}
#[test]
fn opponent_input_cancels_when_the_target_disappears_and_rejects_bad_values() {
    let text = "[bowser.dpad_left.test]\nstart_delay=0\nactions=opponent_input:b:1:0:2\n";
    let sequence = Catalog::parse(text).unwrap().entries()[0].sequence;
    let mut timeline = InputTimeline::new(sequence, enemy(10., 0.));
    assert_eq!(
        timeline
            .sample(AssistSettings::default(), context(0., 1.), None, None, 0.)
            .requested,
        0
    );
    assert!(timeline.finished());
    assert!(Catalog::parse(&text.replace(":1:0:2", ":2:0:2")).is_err());
    assert!(Catalog::parse(&text.replace(":1:0:2", ":1:0:0")).is_err());
}
