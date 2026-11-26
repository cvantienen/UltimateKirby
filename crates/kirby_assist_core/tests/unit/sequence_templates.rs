use super::*;
/// Regression test for `named_templates_are_inert_bounded_and_survive_organizing_and_saving`.
#[test]
fn named_templates_are_inert_bounded_and_survive_organizing_and_saving() {
    let source="[universal.settings]\nenabled=true\n[template.shield]\nname=Quick shield\nstart_delay=0\nactions=input:zr:0:0:4\n[kirby.neutral.test]\nstart_delay=0\nactions=jab:2\n";
    let c = Catalog::parse(source).unwrap();
    assert_eq!(c.len(), 1);
    assert_eq!(c.templates.len(), 1);
    assert_eq!(c.templates[0].id, "shield");
    assert_eq!(c.templates[0].name, "Quick shield");
    let organized = layout::organize(source).unwrap();
    let again = Catalog::parse(&organized).unwrap();
    assert_eq!(again.templates, c.templates);
    let updated = persist::update(
        source,
        6,
        PresetSlot::Neutral,
        "test",
        c.entries()[0].sequence,
        false,
    )
    .unwrap();
    assert_eq!(Catalog::parse(&updated).unwrap().templates, c.templates);
    for diagnostic in [false, true] {
        let c = c.clone().for_build(diagnostic);
        assert_eq!(c.templates.len(), 1);
        c.validate(&FighterProfiles::default()).unwrap();
    }
}
/// Regression test for `templates_cannot_activate_modes_reactions_or_defaults_and_reject_duplicate_ids`.
#[test]
fn templates_cannot_activate_modes_reactions_or_defaults_and_reject_duplicate_ids() {
    for value in [
        "default=true",
        "while_held=true",
        "reaction=auto_roll:30:12",
        "mode=auto_tech",
        "builtin=normal",
    ] {
        assert!(
            Catalog::parse(&format!(
                "[template.bad]\nstart_delay=0\nactions=jab\n{value}\n"
            ))
            .is_err(),
            "{value}"
        );
    }
    assert!(Catalog::parse(
        "[template.a]\nstart_delay=0\nactions=jab\n[template.a]\nstart_delay=0\nactions=grab\n"
    )
    .is_err());
    assert!(Catalog::parse("[template.a.b]\nstart_delay=0\nactions=jab\n").is_err());
    let text = (0..=MAX_TEMPLATES)
        .map(|i| format!("[template.test_{i}]\nstart_delay=0\nactions=jab\n"))
        .collect::<String>();
    assert!(Catalog::parse(&text)
        .unwrap_err()
        .contains("generic templates"));
}
