//! In-match handoff regressions: final release delivery and incapacitation.
use kirby_assist_core::{
    action_owner::{ActionOwnership, Intent, Owner},
    ledge_trap::{
        InputTimeline, LedgeContext, LedgeSettings, MotionTiming, Opponent, SEQUENCE_BUTTONS,
    },
    native_input::{MAPPER_B, MAPPER_R, MAPPER_ZR},
    reaction_rules::{Candidate, Engine, Resolved},
    roll_read::Target,
    sequence_config::Catalog,
};
fn context(frame: f32, status: i32) -> LedgeContext {
    LedgeContext {
        held: true,
        gameplay: true,
        can_act: true,
        status,
        situation: 0,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.,
        facing: 1.,
        safe_ground: [true; 2],
        ..Default::default()
    }
}
fn enemy(frame: f32) -> Opponent {
    Opponent {
        id: 2,
        status: 0x20,
        position: [20., 0.],
        motion: Some(MotionTiming {
            kind: 2,
            frame,
            rate: 1.,
            remaining: None,
        }),
        ..Default::default()
    }
}
fn sequence(actions: &str) -> kirby_assist_core::sequence_config::Sequence {
    Catalog::parse(&format!(
        "[bowser.roll.test]\nstart_delay=0\nactions={actions}\n"
    ))
    .unwrap()
    .entries()[0]
        .sequence
}
#[test]
fn finished_auto_react_keeps_its_neutral_release_owned_through_duplicate_mapper_polls() {
    let cat = Catalog::parse(
        "[bowser.roll.auto]\nstart_delay=0\nreaction=auto_roll:30:10\nactions=up_b:1\n",
    )
    .unwrap();
    let mut rules = Resolved::default();
    rules
        .add(Candidate {
            slot: 2,
            index: 0,
            sequence: cat.entries()[0].sequence,
        })
        .unwrap();
    let mut engine = Engine::default();
    let mut owner = ActionOwnership::new();
    for frame in 0..=1 {
        let frame = frame as f32;
        let e = enemy(frame);
        let mut targets = [None; 8];
        targets[0] = Some(Target {
            id: 2,
            status: e.status,
            motion: 2,
            frame,
            rate: 1.,
            position: e.position,
            velocity_x: 0.,
            vulnerable_in: None,
            actionable_in: None,
        });
        let out = engine.sample_simple(
            true,
            &rules,
            LedgeSettings::default(),
            context(frame, if frame == 0. { 0 } else { 0x1de }),
            targets,
            Some(e),
        );
        assert_eq!(out.requested, if frame == 0. { MAPPER_B } else { 0 });
        assert_eq!(out.owned, SEQUENCE_BUTTONS);
        if frame == 1. {
            assert!(!engine.committed());
        }
        for _ in 0..4 {
            let current = Intent::input_sequence(Owner::RollRead, out.owned);
            let competitors = if frame == 0. {
                vec![current]
            } else {
                vec![
                    current,
                    Intent {
                        owner: Owner::GrabDodge,
                        present: true,
                        committed: true,
                    },
                    Intent {
                        owner: Owner::Shortcut,
                        present: true,
                        committed: true,
                    },
                ]
            };
            assert_eq!(owner.select(true, &competitors), Some(Owner::RollRead));
        }
    }
    let idle = engine.sample_simple(
        true,
        &rules,
        LedgeSettings::default(),
        context(2., 0),
        [None; 8],
        Some(enemy(2.)),
    );
    assert_eq!(idle.owned, 0);
    // The next accepted observation releases completed owners BEFORE asking
    // other policies whether they may start, avoiding an extra rejected frame.
    assert!(!engine.committed());
    owner.release(Owner::RollRead);
    assert!(owner.allows(Owner::OutOfShield));
    assert_eq!(
        owner.select(
            true,
            &[
                Intent::input_sequence(Owner::RollRead, idle.owned),
                Intent {
                    owner: Owner::Shortcut,
                    present: true,
                    committed: true
                }
            ]
        ),
        Some(Owner::Shortcut)
    );
}
#[test]
fn only_native_safety_can_preempt_a_committed_final_release() {
    for next in [Owner::Safety, Owner::Ledge] {
        let mut owner = ActionOwnership::new();
        let release = Intent::input_sequence(Owner::RollRead, SEQUENCE_BUTTONS);
        owner.select(true, &[release]);
        if next == Owner::Ledge {
            owner.set_ledge_priority(true);
        }
        assert_eq!(
            owner.select(
                true,
                &[
                    release,
                    Intent {
                        owner: next,
                        present: true,
                        committed: true
                    }
                ]
            ),
            Some(if next == Owner::Safety {
                next
            } else {
                Owner::RollRead
            })
        );
    }
}
#[test]
fn knockdown_and_rebound_cancel_existing_inputs_but_getups_and_specials_do_not() {
    for status in [
        0x41, 0x46, 0x47, 0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x5a, 0x5b, 0x1bd, 0xb5, 0xb6,
    ] {
        for actions in ["up_b:4,drift:opponent:1:65", "hold_until_manual:b:0:0"] {
            let mut tape = InputTimeline::new(sequence(actions), enemy(0.));
            assert_ne!(
                tape.sample(
                    LedgeSettings::default(),
                    context(0., 0),
                    Some(enemy(0.)),
                    None,
                    0.
                )
                .requested,
                0
            );
            let out = tape.sample(
                LedgeSettings::default(),
                context(1., status),
                Some(enemy(1.)),
                None,
                0.,
            );
            assert!(tape.finished(), "status {status:#x}");
            assert_eq!(out.requested, 0);
            assert_eq!(
                out.owned, 0,
                "damage must return manual jump/dodge immediately"
            );
            assert_eq!(out.stick, None);
        }
    }
    for status in [0, 3, 0x1c, 0x20, 0x57, 0x58, 0x59, 0x7e, 0x7f, 0x1de] {
        let mut tape = InputTimeline::new(sequence("up_b:4"), enemy(0.));
        tape.sample(
            LedgeSettings::default(),
            context(0., 0),
            Some(enemy(0.)),
            None,
            0.,
        );
        let out = tape.sample(
            LedgeSettings::default(),
            context(1., status),
            Some(enemy(1.)),
            None,
            0.,
        );
        assert!(!tape.finished(), "ordinary state {status:#x}");
        assert_eq!(
            out.requested,
            MAPPER_B | if status == 0x1c { MAPPER_ZR } else { 0 }
        );
    }
}
#[test]
fn grounded_damage_interrupts_a_fixed_automatic_grab() {
    use kirby_assist_core::out_of_shield::{Context, OutOfShield};
    for status in [0x47, 0x50, 0x5a, 0x1bd] {
        let mut assist = OutOfShield::new();
        let mut native = Context {
            object: 1,
            kind: 15,
            status: 0,
            motion: 1,
            frame: 0.,
            rate: 1.,
            stopped: false,
            grounded: true,
            position: [0.; 2],
            facing: 1.,
            shield_ready: true,
            can_grab: true,
            can_jump: false,
            can_up_special: false,
            threatened: false,
        };
        let mut targets = [None; 8];
        targets[0] = Some(Target {
            id: 2,
            status: 0x1c,
            motion: 2,
            frame: 0.,
            rate: 1.,
            position: [20., 0.],
            vulnerable_in: Some(0.),
            velocity_x: 0.,
            actionable_in: None,
        });
        assert_eq!(
            assist
                .sample_simple(true, native, context(0., 0), targets, None, 22., 35.)
                .requested,
            MAPPER_R
        );
        native.status = status;
        native.frame = 1.;
        let out = assist.sample_simple(true, native, context(1., status), targets, None, 22., 35.);
        assert_eq!(out.requested, 0);
        assert_eq!(out.owned, 0);
        assert_eq!(out.stick, None);
        assert!(assist.target().is_none());
    }
}
