use super::*;
use crate::{fighter_profiles::presets::PresetSlot, settings_menu::MenuSelection};
/// Regression test for `assist_editor_tunes_roll_conditions_without_moving_rule_to_manual_choices`.
#[test]
fn assist_editor_tunes_roll_conditions_without_moving_rule_to_manual_choices() {
    let source = crate::sequence_config::Catalog::parse(
        "[yoshi.roll.test]\nstart_delay=2\nreaction=roll:any:0:20:18:any:1\nactions=face:opponent,grab\n",
    )
    .unwrap()
    .entries()[0]
        .sequence;
    let key = (2, 0);
    let mut e = RecipeEditor::default();
    e.ensure(key, source);
    e.cursor = fields(e.recipes.get(&key).unwrap())
        .iter()
        .position(|f| matches!(f, Field::Reaction(0)))
        .unwrap();
    assert!(e.adjust(key, true));
    assert_eq!(
        e.compiled(key).unwrap().reaction.trigger,
        crate::reaction_rules::Trigger::Crossed
    );
    e.cursor = fields(e.recipes.get(&key).unwrap())
        .iter()
        .position(|f| matches!(f, Field::Reaction(1)))
        .unwrap();
    e.adjust(key, true);
    let compiled = e.compiled(key).unwrap();
    assert_eq!(
        compiled.reaction.state,
        crate::reaction_rules::SelfState::Grounded
    );
    assert_eq!(compiled.actions(), source.actions());
    assert_eq!(compiled.start_delay, 2);
    assert!(e
        .panel(key, true)
        .into_lines()
        .iter()
        .any(|l| l.contains("Auto reaction: roll")));
    e.cursor = fields(e.recipes.get(&key).unwrap())
        .iter()
        .position(|f| matches!(f, Field::Reaction(0)))
        .unwrap();
    e.adjust(key, false);
    assert_eq!(
        e.compiled(key).unwrap().reaction.trigger,
        crate::reaction_rules::Trigger::Roll
    );
}
/// Regression test for `facing_block_editor_preserves_marker_names_and_roundtrip`.
#[test]
fn facing_block_editor_preserves_marker_names_and_roundtrip() {
    let c=crate::sequence_config::Catalog::parse("[yoshi.neutral.test]\nstart_delay=0\nactions=if_facing_ledge, a:4, end_facing, if_back_to_ledge, b:4, end_facing").unwrap();
    let sequence = c.entries()[0].sequence;
    let r = Recipe::new(sequence);
    assert_eq!(action_name(&r, 0), "If facing ledge");
    assert_eq!(action_name(&r, 3), "If back to ledge");
    assert_eq!(action_name(&r, 5), "End facing block");
    assert_eq!(r.compile().unwrap(), sequence);
    assert!(recipe_panel(&r, &fields(&r), None, false, None)
        .header
        .join(" ")
        .contains("Conditional timing"));
    let mut d = D::Backward;
    direction(&mut d, true, false);
    assert_eq!(d, D::SavedLedge);
    direction(&mut d, true, false);
    assert_eq!(d, D::AwayFromLedge);
    direction(&mut d, true, false);
    assert_eq!(d, D::Opponent);
}

/// Regression test for `shield_hold_selector_round_trips_without_losing_neighbor_actions`.
#[test]
fn shield_hold_selector_round_trips_without_losing_neighbor_actions() {
    let source = crate::sequence_config::Catalog::parse(
        "[bowser.drop.test]\nstart_delay=0\nactions=shield:20,jab:4",
    )
    .unwrap()
    .entries()[0]
        .sequence;
    let key = (4, 0);
    let mut e = RecipeEditor::default();
    e.ensure(key, source);
    e.cursor = fields(&e.recipes[&key])
        .iter()
        .position(|f| matches!(f, Field::ShieldHold(0)))
        .unwrap();
    assert!(e.adjust(key, true));
    let compiled = e.compiled(key).unwrap();
    assert!(matches!(
        compiled.actions[0],
        Action::Input {
            buttons: MAPPER_ZR,
            frames: 0,
            ..
        }
    ));
    assert_eq!(compiled.actions[1], source.actions[1]);
    assert!(e.adjust(key, false));
    assert!(matches!(
        e.compiled(key).unwrap().actions[0],
        Action::Input {
            buttons: MAPPER_ZR,
            frames: 4,
            ..
        }
    ));
}

/// Regression test for `imported_opponent_input_keeps_target_axes_and_duration_editable`.
#[test]
fn imported_opponent_input_keeps_target_axes_and_duration_editable() {
    let source = crate::sequence_config::Catalog::parse(
        "[bowser.dpad_right.test]\nstart_delay=0\nactions=opponent_input:b:-1:0:1\n",
    )
    .unwrap()
    .entries()[0]
        .sequence;
    let key = (0, 0);
    let mut editor = RecipeEditor::default();
    editor.ensure(key, source);
    assert!(editor.lines(key).join(" ").contains("(opponent)"));
    editor.cursor = fields(&editor.recipes[&key])
        .iter()
        .position(|f| matches!(f, Field::Duration(0)))
        .unwrap();
    assert!(editor.adjust(key, true));
    assert!(matches!(
        editor.compiled(key).unwrap().actions[0],
        Action::OpponentInput { frames: 2, .. }
    ));
    editor.cursor = fields(&editor.recipes[&key])
        .iter()
        .position(|f| matches!(f, Field::X(0)))
        .unwrap();
    assert!(editor.adjust(key, true));
    assert!(
        matches!(editor.compiled(key).unwrap().actions[0],Action::OpponentInput{stick:[x,0.],..} if x > -1.)
    );
}

/// Regression test for `assist_ground_only_switch_preserves_other_recipe_fields`.
#[test]
fn assist_ground_only_switch_preserves_other_recipe_fields() {
    let source=crate::sequence_config::Catalog::parse("[bowser.roll.test]\nstart_delay=0\nreaction=roll_vulnerable:grounded:0:24:10:any:1\nactions=up_b:3\n").unwrap().entries()[0].sequence;
    let key = (0, 0);
    let mut editor = RecipeEditor::default();
    editor.ensure(key, source);
    editor.cursor = fields(&editor.recipes[&key])
        .iter()
        .position(|f| matches!(f, Field::GroundOnly))
        .unwrap();

    assert!(editor.adjust(key, true));
    let compiled = editor.compiled(key).unwrap();
    assert!(compiled.ground_only);
    assert_eq!(compiled.actions(), source.actions());
    assert_eq!(compiled.reaction, source.reaction);
    assert!(editor.lines(key).join(" ").contains("Stop if airborne: ON"));
    assert!(editor.adjust(key, false));
    assert!(!editor.compiled(key).unwrap().ground_only);
}

/// Regression test for `both_controller_sticks_and_references_are_editable_and_keep_saved_payload`.
#[test]
fn both_controller_sticks_and_references_are_editable_and_keep_saved_payload() {
    let source = crate::sequence_config::Catalog::parse(
        "[kirby.x.t]\nstart_delay=0\nactions=controller_input:a:0:0:0:0:19\n",
    )
    .unwrap()
    .entries()[0]
        .sequence;
    let key = (5, 0);
    let mut e = RecipeEditor::default();
    e.ensure(key, source);
    for target in [
        Field::Reference(0),
        Field::CReference(0),
        Field::X(0),
        Field::Y(0),
        Field::CX(0),
        Field::CY(0),
    ] {
        e.cursor = fields(&e.recipes[&key])
            .iter()
            .position(|f| *f == target)
            .unwrap();
        assert!(e.adjust(key, true));
    }
    assert!(matches!(
        e.compiled(key).unwrap().actions[0],
        Action::Controller {
            stick: [0.01, 0.01],
            cstick: [0.01, 0.01],
            frames: 19,
            reference: IR::Facing,
            cstick_reference: IR::Facing,
            ..
        }
    ));
}

/// Regression test for `invalid_draft_never_falls_back_to_the_old_recipe_snapshot`.
#[test]
fn invalid_draft_never_falls_back_to_the_old_recipe_snapshot() {
    let catalog = crate::sequence_config::Catalog::parse(
        "[kirby.neutral.first]\nstart_delay=0\nactions=b:4\n",
    )
    .unwrap();
    let mut profiles = crate::fighter_profiles::FighterProfiles::default();
    profiles.add_sequence_choices(&catalog).unwrap();
    let p = profiles.profile(6).unwrap().menu_options;
    let mut m = MenuSelection::new(&p);
    let key = (0, 0);
    let mut editor = RecipeEditor::default();
    let mut sequence = RecipeEditor::custom();
    sequence.len = 32;
    sequence.actions.fill(Action::Input {
        buttons: MAPPER_B,
        stick: [0., 1.],
        frames: 4,
        relative: false,
    });
    editor.ensure(key, sequence);
    editor.recipes.get_mut(&key).unwrap().before[0] = 1;
    assert!(editor.compiled(key).is_none());
    m.editor = Some(Box::new(editor));
    assert!(m.recipe_snapshot(&p, PresetSlot::Neutral, 0).is_none());
    assert!(!m
        .template_sources(&p, (PresetSlot::X as usize, 0))
        .iter()
        .any(|source| source.slot == PresetSlot::Neutral && source.index == 0));
    m.show();
    m.open_editor(&p, PresetSlot::Neutral);
    for button in [
        MAPPER_B,
        crate::ledge_trap::DPAD_DOWN,
        crate::ledge_trap::DPAD_DOWN,
        MAPPER_A,
    ] {
        m.manage_input(&p, button, button);
    }
    assert_eq!(m.choices[PresetSlot::Neutral as usize], 0);
    assert!(m
        .save_status
        .as_ref()
        .unwrap()
        .starts_with("Cannot duplicate:"));
    assert!(m.recipe_snapshot(&p, PresetSlot::Neutral, 0).is_none());
    assert!(
        m.choice(&p, PresetSlot::Neutral, 0).is_some(),
        "an older saved recipe exists but must not be saved instead"
    );
}
/// Regression test for `rapid_axis_edits_across_drafts_compile_and_serialize_the_exact_snapshot`.
#[test]
fn rapid_axis_edits_across_drafts_compile_and_serialize_the_exact_snapshot() {
    let mut e = RecipeEditor::default();
    let mut source = RecipeEditor::custom();
    source.len = 1;
    source.actions[0] = Action::Controller {
        buttons: MAPPER_B,
        stick: [0.; 2],
        cstick: [0.; 2],
        frames: 255,
        reference: IR::Screen,
        cstick_reference: IR::Screen,
    };
    for round in 0..12 {
        let key = (5, round % 3);
        e.ensure(key, source);
        for target in [Field::X(0), Field::Y(0), Field::CX(0), Field::CY(0)] {
            e.cursor = fields(&e.recipes[&key])
                .iter()
                .position(|f| *f == target)
                .unwrap();
            for _ in 0..120 {
                e.adjust(key, round % 2 == 0);
            }
        }
        let snapshot = e.compiled(key).unwrap();
        let text = crate::sequence_config::persist::update(
            "[kirby.x.test]\nstart_delay=0\nactions=b:4\n",
            6,
            PresetSlot::X,
            "test",
            snapshot,
            false,
        )
        .unwrap();
        assert_eq!(
            crate::sequence_config::Catalog::parse(&text)
                .unwrap()
                .entries()[0]
                .sequence,
            snapshot
        );
        assert!(e.recipes.len() <= 3);
    }
}
