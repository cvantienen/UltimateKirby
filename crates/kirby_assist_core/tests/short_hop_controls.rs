use kirby_assist_core::native_input::{
    mapped_automatic_actions, mapped_short_hop, MAPPER_R, MAPPER_X, MAPPED_ATTACK, MAPPED_JUMP,
    MAPPED_ATTACK_RAW, MAPPED_CATCH, MAPPED_JUMP_MINI,
};

#[test]
fn configured_short_hop_marks_only_an_existing_jump() {
    assert_eq!(
        mapped_short_hop(MAPPED_JUMP, true),
        MAPPED_JUMP | MAPPED_JUMP_MINI
    );
    assert_eq!(mapped_short_hop(MAPPED_JUMP, false), MAPPED_JUMP);
    assert_eq!(mapped_short_hop(0, true), 0);
    assert_eq!(mapped_short_hop(MAPPED_ATTACK, true), MAPPED_ATTACK);
}

#[test]
fn explicit_sequence_jump_survives_a_one_frame_native_pulse() {
    let jump = mapped_automatic_actions(0, MAPPER_X);
    assert_eq!(jump, MAPPED_JUMP);
    assert_eq!(mapped_short_hop(jump, true), MAPPED_JUMP | MAPPED_JUMP_MINI);
    assert_eq!(mapped_short_hop(jump, false), MAPPED_JUMP);
}

#[test]
fn controller_and_tap_jump_packets_preserve_short_hop_and_release_metadata() {
    use kirby_assist_core::{gameplay::GameplayContext, ledge_trap::InputTimeline,
        native_states::{situation, status}, sequence_config::Catalog, settings::AssistSettings};
    for (actions, short) in [
        ("input:x:0:0:1,wait:2", true),
        ("controller_input:x:0:0:0:0:1,wait:2", true),
        ("controller_input:x:0:0:0:0:4,wait:2", false),
        ("controller_input:x+a:0:0:0:0:4,wait:2", true),
        ("tap:x:0:0:1:2:3", true),
        ("tap:x:0:0:2:2:4", false),
    ] {
        let catalog = Catalog::parse(&format!("[bowser.roll.test]\nstart_delay=0\nactions={actions}\n")).unwrap();
        let mut tape = InputTimeline::new(catalog.entries()[0].sequence, Default::default());
        let c = GameplayContext { gameplay: true, held: true, can_act: true,
            situation: situation::GROUND, status: status::WAIT, facing: 1.,
            motion_kind: 1, motion_rate: 1., ..Default::default() };
        let out = tape.sample(AssistSettings::default(), c, None, None, 0.);
        assert_eq!(out.native_short_hop, short, "{actions}");
        let mapped = mapped_automatic_actions(0, out.requested);
        assert_ne!(mapped & MAPPED_JUMP, 0, "{actions}");
        assert_eq!(mapped_short_hop(mapped, out.native_short_hop) & MAPPED_JUMP_MINI != 0, short);
        let release = tape.sample(AssistSettings::default(), GameplayContext { motion_frame: 4., ..c }, None, None, 0.);
        assert!(!release.native_short_hop, "{actions}");
    }
}

#[test]
fn explicit_grab_clears_stale_attack_bits() {
    let stale = MAPPED_ATTACK | MAPPED_ATTACK_RAW;
    assert_eq!(
        kirby_assist_core::native_input::mapped_literal_buttons(stale, MAPPER_R),
        MAPPED_CATCH | kirby_assist_core::native_input::MAPPED_GUARD
    );
}
