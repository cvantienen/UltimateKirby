use kirby_assist_core::{
    gameplay::{GameplayContext, MotionTiming, Opponent},
    ledge_trap::{InputTimeline, LedgeSettings},
    native_input::*,
    sequence_config::{persist, Action, Catalog},
};
fn c(frame: f32, facing: f32) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.,
        facing,
        ..Default::default()
    }
}
fn enemy(frame: f32, x: f32) -> Opponent {
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
fn sequence(action: &str) -> kirby_assist_core::sequence_config::Sequence {
    Catalog::parse(&format!(
        "[kirby.x.test]\nstart_delay=0\nactions={action}\n"
    ))
    .unwrap()
    .entries()[0]
        .sequence
}
#[test]
fn buttons_and_both_sticks_are_sent_together_and_capture_their_own_reference() {
    let s = sequence("controller_input:a+b+x:0.5:-0.2:-0.75:0.4:4:facing:opponent");
    let mut t = InputTimeline::new(s, enemy(0., 10.));
    assert!(!t.inputs_only());
    let out = t.sample(
        LedgeSettings::default(),
        c(0., -1.),
        Some(enemy(0., 10.)),
        None,
        0.,
    );
    assert_eq!(out.stick, Some([-0.5, -0.2]));
    assert_eq!(t.right_stick(), [-0.75, 0.4]);
    assert_eq!(
        out.requested,
        MAPPER_A | MAPPER_B | MAPPER_X | MAPPER_RIGHT_STICK_LEFT | MAPPER_RIGHT_STICK_UP
    );
    let next = t.sample(
        LedgeSettings::default(),
        c(1., 1.),
        Some(enemy(1., -10.)),
        None,
        0.,
    );
    assert_eq!(next.stick, out.stick);
    assert_eq!(t.right_stick(), [-0.75, 0.4]);
    assert_eq!(next.requested, out.requested); // Turning/crossing cannot invert a held step.
    t.sample(LedgeSettings::default(), c(2., 1.), None, None, 0.);
    assert!(t.finished());
}
#[test]
fn screen_relative_controller_does_not_require_an_enemy_or_move_readiness() {
    let s = sequence("controller_input:b:1:0:0:0:3");
    let mut t = InputTimeline::new(s, enemy(0., 10.));
    assert!(t.inputs_only());
    let out = t.sample(
        LedgeSettings::default(),
        GameplayContext {
            situation: kirby_assist_core::native_states::situation::AIR,
            ..c(0., -1.)
        },
        None,
        None,
        0.,
    );
    assert_eq!(out.requested, MAPPER_B);
    assert_eq!(out.stick, Some([1., 0.]));
}
#[test]
fn controller_save_roundtrip_preserves_all_fields_and_rejects_invalid_values() {
    let source="[kirby.x.test]\nstart_delay=0\nactions=controller_input:a+b+x+y+l+r+zl+zr:0.5:-1:1:-0.5:255:opponent:facing\n";
    let entry = Catalog::parse(source).unwrap().entries()[0].clone();
    let saved =
        persist::update(source, 6, entry.slot, &entry.choice, entry.sequence, false).unwrap();
    assert_eq!(
        Catalog::parse(&saved).unwrap().entries()[0].sequence,
        entry.sequence
    );
    for bad in [
        "controller_input:a:2:0:0:0:4",
        "controller_input:a:0:0:0:0:0",
        "controller_input:a:0:0:0:NaN:4",
        "controller_input:a:0:0:0:0:4:ledge",
        "controller_input:plus:0:0:0:0:4",
    ] {
        assert!(
            Catalog::parse(&format!("[kirby.x.test]\nstart_delay=0\nactions={bad}\n")).is_err(),
            "{bad}"
        );
    }
    assert!(matches!(
        entry.sequence.actions()[0],
        Action::Controller { frames: 255, .. }
    ));
}
