use super::*;
/// Regression test for `combined_trigger_roundtrips_and_requires_both_crossing_and_vulnerability`.
#[test]
fn combined_trigger_roundtrips_and_requires_both_crossing_and_vulnerability() {
    let rule = Rule::parse("roll_cross_vulnerable:grounded:0:20:12:any:1").unwrap();
    assert_eq!(Rule::parse(&rule.token()).unwrap(), rule);
    let mut sequence = Sequence::default();
    sequence.reaction = rule;
    sequence.len = 1;
    sequence.actions[0] = Action::Input {
        buttons: crate::native_input::MAPPER_A,
        stick: [0., -1.],
        frames: 4,
        relative: true,
    };
    let mut rules = Resolved::default();
    rules
        .add(Candidate {
            slot: 0,
            index: 0,
            sequence,
        })
        .unwrap();
    let mut engine = Engine::default();
    let mut ctx = GameplayContext {
        held: true,
        gameplay: true,
        status: crate::native_states::status::WAIT,
        situation: crate::native_states::situation::GROUND,
        motion_kind: 1,
        motion_frame: 1.,
        motion_rate: 1.,
        facing: 1.,
        ..GameplayContext::default()
    };
    for (frame, x, vuln, expected) in [
        (1., 5., 0., false),
        (2., -4., 3., false),
        (3., -4., 0., true),
    ] {
        ctx.motion_frame = frame;
        let target = Target {
            id: 3,
            status: ESCAPE_F,
            motion: 456,
            frame,
            rate: 1.,
            position: [x, 0.],
            velocity_x: 0.,
            vulnerable_in: Some(vuln),
            actionable_in: Some(6.),
        };
        let out = engine.sample(
            true,
            &rules,
            AssistSettings::default(),
            ctx,
            [Some(target), None, None, None, None, None, None, None],
            None,
        );
        assert_eq!(out.requested != 0, expected);
    }
}
