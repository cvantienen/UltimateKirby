use kirby_assist_core::native_input::{
    mapped_short_hop, MAPPED_ATTACK, MAPPED_JUMP, MAPPED_JUMP_MINI,
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
