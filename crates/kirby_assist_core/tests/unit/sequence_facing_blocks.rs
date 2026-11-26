use super::*;
/// Regression test for `markers_validate_and_save_roundtrip`.
#[test]
fn markers_validate_and_save_roundtrip() {
    let source="[yoshi.neutral.test]\nstart_delay=0\nactions=if_facing_ledge, drift:saved_ledge:0.25:4, end_facing, if_back_to_ledge, walk:away_from_ledge:4:0.3, end_facing";
    let c = Catalog::parse(source).unwrap();
    let s = c.entries()[0].sequence;
    let saved = persist::update(source, 5, PresetSlot::Neutral, "test", s, false).unwrap();
    assert_eq!(Catalog::parse(&saved).unwrap().entries()[0].sequence, s);
    for actions in [
        "end_facing",
        "if_facing_ledge, a",
        "if_facing_ledge, if_back_to_ledge, a, end_facing, end_facing",
    ] {
        assert!(Catalog::parse(&format!(
            "[yoshi.neutral.test]\nstart_delay=0\nactions={actions}"
        ))
        .is_err());
    }
}
