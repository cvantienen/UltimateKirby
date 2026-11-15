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
fn explicit_grab_clears_stale_attack_bits() {
    let stale = MAPPED_ATTACK | MAPPED_ATTACK_RAW;
    assert_eq!(
        kirby_assist_core::native_input::mapped_literal_buttons(stale, MAPPER_R),
        MAPPED_CATCH | kirby_assist_core::native_input::MAPPED_GUARD
    );
}
