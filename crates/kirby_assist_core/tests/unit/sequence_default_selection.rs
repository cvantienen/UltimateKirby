use super::*;
/// Regression test for `startup_default_is_unique_and_other_fighters_categories_and_actions_are_untouched`.
#[test]
fn startup_default_is_unique_and_other_fighters_categories_and_actions_are_untouched() {
    let source = "# preserve\n[kirby.neutral.first]\nstart_delay=0\ndefault=true\nactions=grab\n[kirby.neutral.second]\nstart_delay=0\nname=Second\nactions=wait:2,jab\n[yoshi.neutral.first]\nstart_delay=0\ndefault=true\nactions=grab\n[kirby.x.first]\nstart_delay=0\ndefault=true\nactions=nair\n";
    let result = set_default(source, 6, PresetSlot::Neutral, "second").unwrap();
    let parsed = Catalog::parse(&result).unwrap();
    assert!(!parsed.entries()[0].default);
    assert!(parsed.entries()[1].default);
    assert_eq!(
        parsed.entries()[1].sequence,
        Catalog::parse(source).unwrap().entries()[1].sequence
    );
    assert_eq!(
        result.split("[yoshi.").nth(1),
        source.split("[yoshi.").nth(1)
    );
    assert!(set_default(source, 6, PresetSlot::Neutral, "missing").is_err());
    assert_eq!(
        set_default(&result, 6, PresetSlot::Neutral, "second").unwrap(),
        result
    );
}
