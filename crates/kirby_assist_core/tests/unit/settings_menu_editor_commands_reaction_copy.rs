use super::*;
/// `KEY` is a configuration value used by the Smash runtime.
const KEY: (usize, usize) = (5, 0);
/// Returns the `editor` result.
fn editor() -> RecipeEditor {
    let source =
        crate::sequence_config::Catalog::parse("[kirby.x.test]\nstart_delay=2\nactions=hold:a:7\n")
            .unwrap()
            .entries()[0]
            .sequence;
    let mut e = RecipeEditor::default();
    e.ensure(KEY, source);
    e
}
/// Returns an optional `tap` value.
fn tap(e: &mut RecipeEditor, b: u32) -> Option<(bool, bool)> {
    e.command_input(KEY, 0, 0);
    e.command_input(KEY, b, b)
}
/// Regression test for `copying_inputs_keeps_destination_reaction_conditions`.
#[test]
fn copying_inputs_keeps_destination_reaction_conditions() {
    let mut source = RecipeEditor::custom();
    source.len = 1;
    source.actions[0] = PRESETS[3].1;
    source.reaction = crate::reaction_rules::Rule::parse("roll:any:0:20:12:any:1").unwrap();
    let mut target = source;
    target.actions[0] = PRESETS[16].1;
    target.reaction = crate::reaction_rules::Rule::default();
    for (destination, from) in [(target, source), (source, target)] {
        let mut e = RecipeEditor::default();
        e.ensure((0, 0), destination);
        e.ensure((5, 0), from);
        assert!(e.apply_template((0, 0), (5, 0), from));
        let result = e.compiled((0, 0)).unwrap();
        assert_eq!(result.reaction, destination.reaction);
        assert_eq!(result.actions(), from.actions());
    }
}

/// Regression test for `fixed_picker_has_unique_actions_names_and_one_category_each`.
#[test]
fn fixed_picker_has_unique_actions_names_and_one_category_each() {
    let e = editor();
    let all: Vec<_> = (0..8)
        .flat_map(|g| candidates(&e.recipes[&KEY], g))
        .collect();
    for (i, (name, action)) in PRESETS.iter().enumerate() {
        assert_eq!(all.iter().filter(|k| **k == i).count(), 1, "{name}");
        for (other, other_action) in &PRESETS[i + 1..] {
            assert_ne!(name, other);
            assert_ne!(action, other_action, "duplicate {name} / {other}");
        }
    }
    assert!(!DIRECTIONS.contains(&D::Ledge));
    let docs = include_str!("../../../../docs/EDITOR_ACTION_CHEATSHEET.md");
    // Documentation for these new actions is explicitly deferred by the user.
    for (name, action) in PRESETS.iter().filter(|(_, a)| {
        !matches!(
            a,
            Action::Chase { .. }
                | Action::IfFacingOpponent { .. }
                | Action::ElseFacing
                | Action::LedgePosition { .. }
        )
    }) {
        let _ = action;
        assert!(
            docs.contains(&format!("| {name} |")),
            "missing cheat-sheet entry {name}"
        );
    }
    assert!(
        std::mem::size_of::<Action>() <= 24,
        "keep controller input within existing action storage"
    );
}
/// Regression test for `button_panel_composes_all_gameplay_buttons_and_preserves_sticks_hold_and_delay`.
#[test]
fn button_panel_composes_all_gameplay_buttons_and_preserves_sticks_hold_and_delay() {
    let mut source = Sequence::default();
    source.len = 1;
    source.start_delay = 7;
    source.actions[0] = Action::Controller {
        buttons: 0,
        stick: [0.4, -0.3],
        cstick: [-0.8, 0.6],
        frames: 17,
        reference: IR::Opponent,
        cstick_reference: IR::Facing,
    };
    let mut e = RecipeEditor::default();
    e.ensure(KEY, source);
    e.cursor = fields(&e.recipes[&KEY])
        .iter()
        .position(|f| matches!(f, Field::Buttons(0)))
        .unwrap();
    assert_eq!(tap(&mut e, MAPPER_A), Some((true, false)));
    for i in 0..INPUT_BUTTONS.len() {
        e.modal = Some(Modal::Buttons { slot: 0, item: i });
        assert_eq!(tap(&mut e, MAPPER_A), Some((true, true)));
    }
    let result = e.compiled(KEY).unwrap();
    assert_eq!(result.start_delay, 7);
    assert!(
        matches!(result.actions[0],Action::Controller{buttons,stick:[0.4,-0.3],cstick:[-0.8,0.6],frames:17,reference:IR::Opponent,cstick_reference:IR::Facing} if buttons==INPUT_BUTTONS.iter().fold(0,|m,(b,_)|m|b))
    );
    tap(&mut e, MAPPER_B);
    assert!(e.modal.is_none());
    assert!(e.undo_edit(KEY));
}
/// Regression test for `jump_choices_use_distinct_pulses_instead_of_inheriting_attack_hold`.
#[test]
fn jump_choices_use_distinct_pulses_instead_of_inheriting_attack_hold() {
    let mut e = editor();
    for (label, frames) in [("Short hop", 1), ("Full hop hold", 4)] {
        let kind = PRESETS.iter().position(|(name, _)| *name == label).unwrap();
        let group = group(PRESETS[kind].1);
        let item = candidates(&e.recipes[&KEY], group)
            .iter()
            .position(|k| *k == kind)
            .unwrap();
        e.modal = Some(Modal::Picker {
            slot: 0,
            group,
            item,
        });
        tap(&mut e, MAPPER_A);
        assert!(
            matches!(e.compiled(KEY).unwrap().actions[0],Action::Input{buttons:MAPPER_X,frames:f,..} if f==frames)
        );
    }
}
/// Regression test for `legacy_saved_actions_are_retained_without_reintroducing_fixed_duplicates`.
#[test]
fn legacy_saved_actions_are_retained_without_reintroducing_fixed_duplicates() {
    let source = crate::sequence_config::Catalog::parse(
        "[kirby.x.t]\nstart_delay=0\nactions=range:ignore,shield_until_manual,range:ignore\n",
    )
    .unwrap()
    .entries()[0]
        .sequence;
    let mut e = RecipeEditor::default();
    e.ensure(KEY, source);
    assert_eq!(e.compiled(KEY).unwrap(), source);
    assert_eq!(candidates(&e.recipes[&KEY], 8).len(), 2);
    for (_, a) in PRESETS {
        assert!(!matches!(
            a,
            Action::Range(RangeCheck::Ignore) | Action::ShieldUntilManual
        ));
    }
}
