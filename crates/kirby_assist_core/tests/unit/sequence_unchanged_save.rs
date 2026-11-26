use super::*;
#[test]
fn scoped_regression_tilt_aliases_are_replaced_semantically() {
    for alias in ["auto_tilts", "tilt_turn"] {
        let original = format!("[kirby.neutral.custom]\nstart_delay=0\nactions=jab\n{alias}=down\n[kirby.roll.keep]\nstart_delay=0\nactions=grab\n{alias}=up\n");
        let catalog = Catalog::parse(&original).unwrap();
        for disable in [false, true] {
            let mut seq = catalog.entries()[0].sequence;
            seq.actions[0] = Action::Wait(2);
            if disable {
                seq.auto_tilt = None;
            }
            let saved = update(&original, 6, PresetSlot::Neutral, "custom", seq, false).unwrap();
            let parsed = Catalog::parse(&saved).unwrap();
            assert_eq!(parsed.entries()[0].sequence, seq);
            assert_eq!(parsed.entries()[1], catalog.entries()[1]);
            assert!(saved.ends_with(&format!(
                "[kirby.roll.keep]\nstart_delay=0\nactions=grab\n{alias}=up\n"
            )));
        }
    }
}

#[test]
fn same_sequence_and_name_preserve_original_bytes() {
    let original =
        "[yoshi.neutral.test]\n# preserve formatting\nname=Grab\nstart_delay=0\nactions=grab:4\n";
    let c = Catalog::parse(original).unwrap();
    let seq = c.entries()[0].sequence;
    assert_eq!(
        update_named(
            original,
            5,
            PresetSlot::Neutral,
            "test",
            seq,
            false,
            Some("Grab")
        )
        .unwrap(),
        original
    );
    let renamed = update_named(
        original,
        5,
        PresetSlot::Neutral,
        "test",
        seq,
        false,
        Some("Other grab"),
    )
    .unwrap();
    assert_eq!(
        Catalog::parse(&renamed).unwrap().entries()[0]
            .name
            .as_deref(),
        Some("Other grab")
    );
}
