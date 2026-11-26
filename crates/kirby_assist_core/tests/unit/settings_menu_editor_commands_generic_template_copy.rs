use super::*;
use crate::sequence_config::Catalog;
/// Returns the `sequence` result.
fn sequence(source: &str) -> Sequence {
    Catalog::parse(source).unwrap().entries()[0].sequence
}
/// Regression test for `generic_copy_keeps_destination_reaction_and_activation_and_frees_dialog_names`.
#[test]
fn generic_copy_keeps_destination_reaction_and_activation_and_frees_dialog_names() {
    let key = (5, 0);
    let original=sequence("[kirby.x.custom]\nwhile_held=true\nstart_delay=0\nreaction=auto_shield_grab:20:5\nactions=grab:4\n");
    let c = Catalog::parse("[template.fast_grab]\nname=Fast grab\nstart_delay=1\nactions=grab:2\n")
        .unwrap();
    let mut e = RecipeEditor::default();
    e.ensure(key, original);
    e.set_copy_templates(&c.templates);
    assert!(e.apply_generic_template(key, c.templates[0].sequence));
    let after = e.compiled(key).unwrap();
    assert_eq!(after.reaction, original.reaction);
    assert!(after.while_held);
    assert_eq!(after.start_delay, 1);
    assert_eq!(after.actions(), c.templates[0].sequence.actions());
    assert_eq!(e.copy_templates.capacity(), 0);
    assert_eq!(e.copy_sources.capacity(), 0);
}
/// Regression test for `a_ledge_template_cannot_replace_an_automatic_punish`.
#[test]
fn a_ledge_template_cannot_replace_an_automatic_punish() {
    let key = (2, 0);
    let original =
        sequence("[bowser.roll.custom]\nstart_delay=0\nreaction=auto_roll:30:12\nactions=up_b:4\n");
    let c =
        Catalog::parse("[template.spacing]\nstart_delay=0\nactions=run_to_ledge:10:1:1:2:120\n")
            .unwrap();
    let mut e = RecipeEditor::default();
    e.ensure(key, original);
    assert!(!e.apply_generic_template(key, c.templates[0].sequence));
    assert_eq!(e.compiled(key), Some(original));
}
