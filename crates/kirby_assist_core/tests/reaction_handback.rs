use kirby_assist_core::{
    gameplay::{GameplayContext, MotionTiming, Opponent},
    ledge_trap::{InputTimeline, LedgeSettings},
    manual_controls::{AssistGuardHandoff, ManualOverlay, TurboOutput},
    native_input::{ButtonTrace, MAPPER_B, MAPPER_ZR},
    sequence_config::{persist, Catalog},
};
fn config(
    actions: &str,
    ground: bool,
    reaction: bool,
) -> kirby_assist_core::sequence_config::Sequence {
    Catalog::parse(&format!(
        "[bowser.roll.test]\nstart_delay=0\nground_only={ground}\n{}actions={actions}\n",
        if reaction {
            "reaction=roll_vulnerable:grounded:0:24:10:any:1\n"
        } else {
            ""
        }
    ))
    .unwrap()
    .entries()[0]
        .sequence
}
fn enemy(frame: f32) -> Opponent {
    Opponent {
        id: 2,
        status: kirby_assist_core::native_states::ESCAPE_F,
        position: [20., 0.],
        motion: Some(MotionTiming {
            kind: 2,
            frame,
            rate: 1.,
            remaining: Some(8.),
        }),
        ..Default::default()
    }
}
fn ctx(frame: f32) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.,
        facing: 1.,
        safe_ground: [true; 2],
        ..Default::default()
    }
}
#[test]
fn fresh_manual_intent_cannot_cancel_or_mix_into_ordinary_reaction_tape() {
    let s = config("up_b:3", true, true);
    let mut t = InputTimeline::new(s, enemy(0.));
    assert_eq!(
        t.sample(LedgeSettings::default(), ctx(0.), Some(enemy(0.)), None, 0.)
            .requested
            & MAPPER_B,
        MAPPER_B
    );
    let c = GameplayContext {
        manual_input_epoch: 1,
        ..ctx(0.)
    };
    assert_eq!(
        t.sample(LedgeSettings::default(), c, Some(enemy(0.)), None, 0.)
            .owned,
        kirby_assist_core::ledge_trap::SEQUENCE_BUTTONS
    );
    assert!(!t.finished());
}
#[test]
fn ground_only_prevents_aerial_up_b_after_leaving_floor() {
    let mut t = InputTimeline::new(config("run:opponent:6,up_b:3", true, true), enemy(0.));
    assert_eq!(
        t.sample(LedgeSettings::default(), ctx(0.), Some(enemy(0.)), None, 0.)
            .requested,
        0
    );
    let c = GameplayContext {
        situation: kirby_assist_core::native_states::situation::AIR,
        position: [7., 0.],
        ..ctx(1.)
    };
    assert_eq!(
        t.sample(LedgeSettings::default(), c, Some(enemy(1.)), None, 0.)
            .owned,
        0
    );
    assert!(t.finished());
}
#[test]
fn existing_grounded_start_filters_do_not_cancel_aerial_sequences_without_opt_in() {
    let mut t = InputTimeline::new(config("short_hop:1,up_b:3", false, true), enemy(0.));
    t.sample(LedgeSettings::default(), ctx(0.), Some(enemy(0.)), None, 0.);
    let c = GameplayContext {
        situation: kirby_assist_core::native_states::situation::AIR,
        ..ctx(1.)
    };
    assert_eq!(
        t.sample(LedgeSettings::default(), c, Some(enemy(1.)), None, 0.)
            .requested
            & MAPPER_B,
        MAPPER_B
    );
    assert!(!t.finished());
    let mut literal = InputTimeline::new(config("up_b:3", false, false), enemy(0.));
    literal.sample(LedgeSettings::default(), ctx(0.), Some(enemy(0.)), None, 0.);
    assert_ne!(
        literal
            .sample(
                LedgeSettings::default(),
                GameplayContext {
                    manual_input_epoch: 1,
                    ..ctx(1.)
                },
                Some(enemy(1.)),
                None,
                0.
            )
            .owned,
        0
    );
}
#[test]
fn suppressed_held_shield_requires_a_real_release_and_does_not_gain_a_fresh_edge() {
    let mut h = AssistGuardHandoff::new();
    let mut overlay = ManualOverlay::default();
    let human = ButtonTrace {
        held: MAPPER_ZR,
        previous: MAPPER_ZR,
        ..Default::default()
    };
    let blocked = h.sample(true, human, MAPPER_ZR, MAPPER_ZR, 0);
    overlay.apply_controls(
        human,
        [1., 0.],
        TurboOutput {
            stick: Some([1., 0.]),
            ..Default::default()
        },
        0,
        0,
        blocked,
    );
    for _ in 0..4 {
        let blocked = h.sample(false, human, MAPPER_ZR, 0, 0);
        let out = overlay.apply_controls(human, [1., 0.], TurboOutput::default(), 0, 0, blocked);
        assert_eq!(out.buttons.held & MAPPER_ZR, 0);
        assert_eq!(out.buttons.pressed & MAPPER_ZR, 0);
    }
    let human = ButtonTrace::default();
    let blocked = h.sample(false, human, MAPPER_ZR, 0, 0);
    overlay.apply_controls(human, [0., 0.], TurboOutput::default(), 0, 0, blocked);
    let human = ButtonTrace {
        held: MAPPER_ZR,
        pressed: MAPPER_ZR,
        ..Default::default()
    };
    let blocked = h.sample(false, human, MAPPER_ZR, 0, 0);
    let out = overlay.apply_controls(human, [0., 0.], TurboOutput::default(), 0, 0, blocked);
    assert_eq!(out.buttons.pressed & MAPPER_ZR, MAPPER_ZR);
}
#[test]
fn ground_only_saves_roundtrip_without_duplicate_flags() {
    let source="[bowser.roll.test]\nstart_delay=0\nactions=up_b:3\nreaction=roll_vulnerable:grounded:0:24:10:any:1\n";
    let entry = Catalog::parse(source).unwrap().entries()[0].clone();
    let mut s = entry.sequence;
    s.ground_only = true;
    let text = persist::update(source, 15, entry.slot, &entry.choice, s, false).unwrap();
    assert!(
        Catalog::parse(&text).unwrap().entries()[0]
            .sequence
            .ground_only
    );
    s.ground_only = false;
    let text = persist::update(&text, 15, entry.slot, &entry.choice, s, false).unwrap();
    assert!(!text.contains("ground_only"));
    assert!(Catalog::parse(&source.replace(
        "start_delay=0",
        "ground_only=true\nground_only=false\nstart_delay=0"
    ))
    .is_err());
}

#[test]
fn brother_jump_buttons_remain_manual_takeover_intent_without_becoming_shortcut_inputs() {
    use kirby_assist_core::{
        manual_controls::ManualTakeover,
        native_input::{ControlLayout, MAPPER_R, MAPPER_ZR},
    };
    let layout = ControlLayout::Brother;
    let mut intent = ManualTakeover::new();
    intent.sample(layout.takeover_buttons(0), [0.; 4]);
    assert_eq!(layout.policy_buttons(MAPPER_R), 0);
    intent.sample(layout.takeover_buttons(MAPPER_R), [0.; 4]);
    assert_eq!(intent.epoch(), 1);
    intent.sample(layout.takeover_buttons(MAPPER_R | MAPPER_ZR), [0.; 4]);
    assert_eq!(intent.epoch(), 2);
}

#[test]
fn exclusive_tape_masks_all_physical_gameplay_buttons_through_wait_and_drift() {
    use kirby_assist_core::{
        ledge_trap::SEQUENCE_BUTTONS,
        native_input::{ControlLayout, MAPPER_MENU},
    };
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        let mut tape = InputTimeline::new(
            config("wait:2,up_b:2,drift:opponent:1:3", false, true),
            enemy(0.),
        );
        let mut noisy = ManualOverlay::default();
        let mut clean = ManualOverlay::default();
        for overlay in [&mut noisy, &mut clean] {
            overlay.apply_controls(
                ButtonTrace::default(),
                [0.; 2],
                TurboOutput::default(),
                0,
                0,
                0,
            );
        }
        for frame in 0..7 {
            let c = GameplayContext {
                manual_input_epoch: frame as u64,
                ..ctx(frame as f32)
            };
            let out = tape.sample(
                LedgeSettings::default(),
                c,
                Some(enemy(frame as f32)),
                None,
                0.,
            );
            assert_eq!(out.owned, SEQUENCE_BUTTONS);
            assert_eq!(tape.right_stick(), [0.; 2]);
            let buttons = layout.physical_buttons(SEQUENCE_BUTTONS);
            let human = ButtonTrace {
                held: buttons,
                previous: buttons,
                pressed: buttons,
                released: buttons,
            };
            let run = |overlay: &mut ManualOverlay, human, stick| {
                overlay.apply_controls(
                    human,
                    stick,
                    TurboOutput {
                        stick: out.stick,
                        ..Default::default()
                    },
                    0,
                    layout.physical_buttons(out.requested),
                    layout.physical_owned_buttons(out.owned),
                )
            };
            let a = run(&mut noisy, human, [-1., -1.]);
            let b = run(&mut clean, ButtonTrace::default(), [0.; 2]);
            assert_eq!(a.buttons, b.buttons, "frame {frame}");
            assert_eq!(a.stick, b.stick);
            assert_eq!(a.clear_autorepeat & buttons, buttons);
            assert_eq!(out.owned & MAPPER_MENU, 0);
        }
        assert!(!tape.finished());
        tape.sample(LedgeSettings::default(), ctx(7.), Some(enemy(7.)), None, 0.);
        assert!(tape.finished());
        let handback = noisy.apply_controls(
            ButtonTrace {
                held: MAPPER_B,
                pressed: MAPPER_B,
                ..Default::default()
            },
            [-0.5, 0.],
            TurboOutput::default(),
            0,
            0,
            0,
        );
        assert_eq!(handback.buttons.held & MAPPER_B, MAPPER_B);
        assert_eq!(handback.stick, [-0.5, 0.]);
    }
}
