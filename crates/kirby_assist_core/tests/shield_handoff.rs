//! Shield release is shared by every file-authored sequence consumer.
use kirby_assist_core::{
    defense_response::Response,
    ledge_trap::{InputTimeline, LedgeContext, LedgeSettings, Opponent},
    native_input::*,
    reaction_rules::{Candidate, Resolved, Trigger},
    sequence_config::Catalog,
};
fn context(status: i32, frame: f32) -> LedgeContext {
    LedgeContext {
        held: true,
        gameplay: true,
        status,
        situation: 0,
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
            LedgeSettings::default(),
            context(status, frame),
            None,
            None,
            0.,
        )
    };
    assert_eq!(sample(&mut t, 0x1c, 0.).requested, MAPPER_ZR);
    assert_eq!(sample(&mut t, 0x1e, 1.).requested, MAPPER_ZR);
    for (status, frame) in [(0x1c, 2.), (0x1d, 3.), (0x1d, 4.)] {
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
        ("grab:4", MAPPER_R),
        ("up_b:4", MAPPER_B),
        ("up_smash:4", MAPPER_A),
        ("short_hop:1", MAPPER_X),
        ("input:zl:0:0:1", MAPPER_ZL),
    ] {
        let mut t = InputTimeline::new(
            sequence(&format!("shield_until_block:30,{action}")),
            Opponent::default(),
        );
        t.sample(LedgeSettings::default(), context(0x1e, 0.), None, None, 0.);
        let o = t.sample(LedgeSettings::default(), context(0x1c, 1.), None, None, 0.);
        assert_eq!(o.requested & button, button, "{action}");
        assert_eq!(o.requested & MAPPER_ZR,MAPPER_ZR,"{action}: direct grab, special, up-smash and jump keep guard through native OOS cancellation");
    }
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
        assert!(r.start(&rules, Trigger::ShieldHit, context(0x1c, 0.), enemy));
        for (status, frame) in [(0x1c, 0.), (0x1d, 1.)] {
            let o = r.sample(
                true,
                LedgeSettings::default(),
                context(status, frame),
                Some(enemy),
            );
            assert_eq!(o.requested, 0, "{action}");
            assert_eq!(o.stick, Some([0.; 2]));
            assert_eq!(r.right_stick(), Some([0.; 2]));
        }
    }
}
