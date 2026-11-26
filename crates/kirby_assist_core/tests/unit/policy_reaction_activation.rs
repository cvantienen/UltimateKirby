use super::*;
use crate::{
    fighter_profiles::presets::PresetSlot,
    sequence_config::{persist, Catalog},
};
/// Regression test for `old_seven_fields_stay_menu_and_all_explicit_controls_roundtrip`.
#[test]
fn old_seven_fields_stay_menu_and_all_explicit_controls_roundtrip() {
    let old = "roll:grounded:0:20:12:any:1";
    assert_eq!(Rule::parse(old).unwrap().control, Control::Menu);
    assert_eq!(Rule::parse(old).unwrap().token(), old);
    for c in Control::ALL {
        let mut r = Rule::parse(old).unwrap();
        r.control = c;
        assert_eq!(Rule::parse(&r.token()).unwrap(), r);
        let source = "[yoshi.roll.rule]\nstart_delay=0\nactions=jab:4\n";
        let mut seq = Catalog::parse(source).unwrap().entries()[0].sequence;
        seq.reaction = r;
        let file = persist::update(source, 5, PresetSlot::Roll, "rule", seq, false).unwrap();
        assert_eq!(Catalog::parse(&file).unwrap().entries()[0].sequence, seq);
    }
    assert!(Rule::parse(&format!("{old}:toggle_zr")).is_err());
}
/// Regression test for `only_enabled_control_groups_can_start_and_disabling_cancels_playback`.
#[test]
fn only_enabled_control_groups_can_start_and_disabling_cancels_playback() {
    let mut seq = Sequence::default();
    seq.len = 1;
    seq.actions[0] = Action::Input {
        buttons: crate::native_input::MAPPER_A,
        stick: [0.; 2],
        frames: 10,
        relative: true,
    };
    seq.reaction = Rule::parse("roll:any:0:20:12:any:1:toggle_x").unwrap();
    let mut rules = Resolved::default();
    rules
        .add(Candidate {
            slot: 0,
            index: 0,
            sequence: seq,
        })
        .unwrap();
    let mut e = Engine::default();
    let mut c = GameplayContext {
        held: true,
        gameplay: true,
        situation: crate::native_states::situation::GROUND,
        status: crate::native_states::status::WAIT,
        motion_kind: 1,
        motion_frame: 1.,
        motion_rate: 1.,
        facing: 1.,
        ..GameplayContext::default()
    };
    let t = Target {
        id: 1,
        status: ESCAPE_F,
        motion: 456,
        frame: 1.,
        rate: 1.,
        position: [5., 0.],
        velocity_x: 0.,
        vulnerable_in: Some(0.),
        actionable_in: Some(5.),
    };
    let targets = [Some(t), None, None, None, None, None, None, None];
    assert_eq!(
        e.sample(true, &rules, AssistSettings::default(), c, targets, None)
            .requested,
        0
    );
    rules.enabled_controls = Control::ToggleX.bit();
    c.motion_frame = 2.;
    assert_ne!(
        e.sample(true, &rules, AssistSettings::default(), c, targets, None)
            .requested,
        0
    );
    rules.enabled_controls = 0;
    c.motion_frame = 3.;
    assert_eq!(
        e.sample(true, &rules, AssistSettings::default(), c, targets, None)
            .requested,
        0
    );
    assert!(!e.committed());
}
