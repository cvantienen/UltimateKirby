use super::*;
/// `SOURCE` is a configuration value used by the Smash runtime.
const SOURCE: &str = "# keep\n[yoshi.roll.grab]\nname = Tongue grab\ndefault=true\nstart_delay=0\nactions=grab\n\n[kirby.roll.a]\nstart_delay=0\nactions=a\n";
/// Returns the `sequence` result.
fn sequence(actions: &str) -> Sequence {
    Catalog::parse(&format!("[yoshi.roll.x]\nstart_delay=4\nactions={actions}"))
        .unwrap()
        .entries()[0]
        .sequence
}

/// Regression test for `while_held_survives_save_and_can_be_disabled_without_duplicate_keys`.
#[test]
fn while_held_survives_save_and_can_be_disabled_without_duplicate_keys() {
    let mut seq = sequence("tap:a:0:-0.6:1:11:120");
    seq.while_held = true;
    let text = update(
        "[yoshi.y.repeat]\nstart_delay=0\nactions=a\n",
        5,
        PresetSlot::Y,
        "repeat",
        seq,
        false,
    )
    .unwrap();
    assert!(
        Catalog::parse(&text).unwrap().entries()[0]
            .sequence
            .while_held
    );
    seq.while_held = false;
    let text = update(&text, 5, PresetSlot::Y, "repeat", seq, false).unwrap();
    assert!(!text.contains("while_held"));
    assert!(
        !Catalog::parse(&text).unwrap().entries()[0]
            .sequence
            .while_held
    );
}
/// Regression test for `taps_roundtrip_as_one_step_and_reject_zero_release_or_invalid_durations`.
#[test]
fn taps_roundtrip_as_one_step_and_reject_zero_release_or_invalid_durations() {
    let original = sequence("tap:a+b:0:-0.6:1:11:1800");
    assert_eq!(original.len, 1);
    let text = update(SOURCE, 5, PresetSlot::Roll, "grab", original, false).unwrap();
    assert_eq!(
        Catalog::parse(&text).unwrap().entries()[0]
            .sequence
            .actions(),
        original.actions()
    );
    for bad in [
        "tap:none:0:0:1:1:60",
        "tap:a:0:0:0:1:60",
        "tap:a:0:0:1:0:60",
        "tap:a:0:0:1:1:0",
        "tap:a:0:0:1:1:3601",
        "tap:a:0:0:256:1:60",
        "tap:a:NaN:0:1:1:60",
    ] {
        assert!(
            Catalog::parse(&format!("[yoshi.roll.bad]\nstart_delay=0\nactions={bad}")).is_err(),
            "{bad}"
        );
    }
}
/// Regression test for `controller_and_opponent_inputs_roundtrip_without_rewriting_button_names`.
#[test]
fn controller_and_opponent_inputs_roundtrip_without_rewriting_button_names() {
    let original =
        sequence("controller_input:a+b:0.25:-0.5:0.75:-1:4:facing:opponent,opponent_input:b:1:0:8");
    let text = update(SOURCE, 5, PresetSlot::Roll, "grab", original, false).unwrap();
    let saved = Catalog::parse(&text).unwrap().entries()[0].sequence;
    assert_eq!(saved.actions(), original.actions());
}
/// Regression test for `explicit_checks_serialize_and_validate`.
#[test]
fn explicit_checks_serialize_and_validate() {
    let original =
        sequence("check:enemy_within:12:4,check:enemy_shielding,check:self_grounded:255,hold:a:60");
    let text = update(SOURCE, 5, PresetSlot::Roll, "grab", original, false).unwrap();
    let parsed = Catalog::parse(&text).unwrap();
    assert_eq!(parsed.entries()[0].sequence.actions(), original.actions());
    for bad in [
        "check:unknown",
        "check:enemy_within",
        "check:enemy_within:NaN",
        "check:enemy_shielding:256",
        "check:enemy_shielding:0:1",
    ] {
        assert!(
            Catalog::parse(&format!("[yoshi.roll.bad]\nstart_delay=0\nactions={bad}")).is_err(),
            "{bad}"
        );
    }
}
#[test]
fn mode_and_builtin_reject_inert_recipe_settings() {
    for text in [
        "[kirby.x.test]\nmode = auto_mash\nground_only = true\n",
        "[kirby.x.test]\nmode = auto_mash\nwhile_held = true\n",
        "[kirby.x.test]\nbuiltin = normal\nauto_tilt = up\n",
    ] {
        assert!(Catalog::parse(text).is_err(), "{text}");
    }
}

#[test]
fn saving_rejects_invalid_tap_instead_of_silently_normalizing_it() {
    let mut bad = sequence("tap:a:0:0:1:1:60");
    for action in [
        Action::Tap {
            buttons: 0,
            stick: [0.; 2],
            hold: 1,
            gap: 1,
            frames: 60,
        },
        Action::Tap {
            buttons: crate::native_input::MAPPER_A,
            stick: [0.; 2],
            hold: 0,
            gap: 1,
            frames: 60,
        },
        Action::Tap {
            buttons: crate::native_input::MAPPER_A,
            stick: [0.; 2],
            hold: 1,
            gap: 0,
            frames: 60,
        },
        Action::Tap {
            buttons: crate::native_input::MAPPER_A,
            stick: [0.; 2],
            hold: 1,
            gap: 1,
            frames: 0,
        },
    ] {
        bad.actions[0] = action;
        assert!(update(SOURCE, 5, PresetSlot::Roll, "grab", bad, false).is_err());
    }
}

#[test]
fn saved_range_and_condition_options_keep_immediate_defaults() {
    let original = sequence(
        "range:ignore,range:5,range:fighter_range,range:fighter_range_instant,check:self_grounded,check:enemy_within:10,grab:verified",
    );
    assert!(matches!(
        original.actions()[4],
        Action::Check { wait: 0, .. }
    ));
    assert!(matches!(
        original.actions()[5],
        Action::Check { wait: 0, .. }
    ));
    let text = update(SOURCE, 5, PresetSlot::Roll, "grab", original, false).unwrap();
    assert_eq!(
        Catalog::parse(&text).unwrap().entries()[0].sequence,
        original
    );
}

/// Regression test for `saving_a_ledge_recipe_preserves_ignored_retired_button_sections`.
#[test]
fn saving_a_ledge_recipe_preserves_ignored_retired_button_sections() {
    let archived = "[universal.x.old]\nname=Archived\nstart_delay=0\nactions=old_syntax\n";
    let source = format!("{archived}[kirby.neutral.keep]\nstart_delay=0\nactions=grab\n");
    let updated = update(
        &source,
        6,
        PresetSlot::Neutral,
        "keep",
        sequence("hold:a:4"),
        false,
    )
    .unwrap();
    assert!(updated.starts_with(archived));
    assert_eq!(Catalog::parse(&updated).unwrap().len(), 1);
}
/// Regression test for `diagnostic_rename_delete_and_add_preserve_other_sections_and_validate`.
#[test]
fn diagnostic_rename_delete_and_add_preserve_other_sections_and_validate() {
    let seq = sequence("grab");
    let renamed = update_named(
        SOURCE,
        5,
        PresetSlot::Roll,
        "grab",
        seq,
        false,
        Some("Perfect roll grab"),
    )
    .unwrap();
    let c = Catalog::parse(&renamed).unwrap();
    assert_eq!(c.entries()[0].name.as_deref(), Some("Perfect roll grab"));
    assert!(c.entries()[0].default);
    assert!(renamed.ends_with("[kirby.roll.a]\nstart_delay=0\nactions=a\n"));
    let added = update_named(
        &renamed,
        5,
        PresetSlot::Roll,
        "menu_added_7",
        seq,
        true,
        Some("New setup"),
    )
    .unwrap();
    let removed = delete(&added, 5, PresetSlot::Roll, "grab", false).unwrap();
    let c = Catalog::parse(&removed).unwrap();
    assert_eq!(c.entries().len(), 2);
    assert!(c.entries().iter().any(|e| e.choice == "menu_added_7"));
    assert!(!c.entries().iter().any(|e| e.choice == "grab"));
    assert_eq!(
        delete(&removed, 5, PresetSlot::Roll, "grab", true).unwrap(),
        removed
    );
    for bad in ["", "bad#name", "bad\nname", "bad\"name"] {
        assert!(update_named(SOURCE, 5, PresetSlot::Roll, "grab", seq, false, Some(bad)).is_err());
    }
}
/// Regression test for `saved_recipe_preserves_other_sections_label_and_default`.
#[test]
fn saved_recipe_preserves_other_sections_label_and_default() {
    let s = sequence("walk:inward:4:0.5,wait:2,face:opponent,grab:fighter_range");
    let result = update(SOURCE, 5, PresetSlot::Roll, "grab", s, false).unwrap();
    assert!(result.starts_with("# keep\n"));
    assert!(result.contains("name = Tongue grab\ndefault=true"));
    assert!(result.ends_with("[kirby.roll.a]\nstart_delay=0\nactions=a\n"));
    assert_eq!(Catalog::parse(&result).unwrap().entries()[0].sequence, s);
}
/// Regression test for `saved_recipe_roundtrips_palette_actions_and_custom_updates_without_duplicates`.
#[test]
fn saved_recipe_roundtrips_palette_actions_and_custom_updates_without_duplicates() {
    let s=sequence("facing_input:x+a:0.3:-0.5:4,input:none:0:0:2,cstick:1:0:4,walk_to_opponent:0.5:10:4,drift_to_opponent:0.5:10:4,run:ledge:4,drift:away:0.25:5,range:ignore,range:4,range:fighter_range,range:fighter_range_instant,shield_until_block:9");
    let first = update(SOURCE, 5, PresetSlot::Roll, "menu_custom", s, true).unwrap();
    let second = update(&first, 5, PresetSlot::Roll, "menu_custom", s, true).unwrap();
    assert_eq!(first, second);
    let catalog = Catalog::parse(&second).unwrap();
    assert_eq!(catalog.entries().len(), 3);
    assert_eq!(
        catalog
            .entries()
            .iter()
            .find(|e| e.choice == "menu_custom")
            .unwrap()
            .sequence,
        s
    );
}
/// Regression test for `saved_recipe_rejects_invalid_source_removed_sections_and_invalid_edits`.
#[test]
fn saved_recipe_rejects_invalid_source_removed_sections_and_invalid_edits() {
    let s = sequence("grab");
    assert!(update("invalid", 5, PresetSlot::Roll, "grab", s, false).is_err());
    assert!(update(SOURCE, 5, PresetSlot::Roll, "missing", s, false).is_err());
    assert!(update(
        SOURCE,
        6,
        PresetSlot::Roll,
        "grab",
        Sequence::default(),
        false
    )
    .is_err());
    let mut bad = s;
    bad.start_delay = 3601;
    assert!(update(SOURCE, 5, PresetSlot::Roll, "grab", bad, false).is_err());
}

/// Exhaustive on purpose: adding an `Action` variant breaks this match until
/// the writer/reader round-trip below also covers it.
fn variant_index(action: &Action) -> usize {
    match action {
        Action::Empty => 0,
        Action::Input { .. } => 1,
        Action::OpponentInput { .. } => 2,
        Action::Controller { .. } => 3,
        Action::Tap { .. } => 4,
        Action::CStick { .. } => 5,
        Action::Chase { .. } => 6,
        Action::Wait(_) => 7,
        Action::Face(_) => 8,
        Action::IfFacingLedge { .. } => 9,
        Action::IfFacingOpponent { .. } => 10,
        Action::ElseFacing => 11,
        Action::EndFacing => 12,
        Action::LedgePosition { .. } => 13,
        Action::Approach { .. } => 14,
        Action::Walk { .. } => 15,
        Action::Run { .. } => 16,
        Action::Drift { .. } => 17,
        Action::Range(_) => 18,
        Action::ShieldUntilBlock(_) => 19,
        Action::ShieldUntilManual => 20,
        Action::Check { .. } => 21,
    }
}

/// Regression test for `every_action_variant_and_direction_roundtrips_through_writer_and_parser`.
#[test]
fn every_action_variant_and_direction_roundtrips_through_writer_and_parser() {
    let directions: Vec<Direction> = Direction::TOKENS.iter().map(|(d, _)| *d).collect();
    let input = |buttons, stick, frames, relative| Action::Input {
        buttons,
        stick,
        frames,
        relative,
    };
    use crate::native_input::{MAPPER_A as A, MAPPER_B as B, MAPPER_X as X};
    let mut groups: Vec<Vec<Action>> = vec![
        vec![
            input(A | X, [0.3, -0.5], 4, false),
            input(A | X, [0.3, -0.5], 4, true),
            input(0, [0.0, 0.0], 2, false),
            input(A, [1.0, 0.0], 0, false),
            input(B, [0.0, -1.0], 0, true),
            Action::OpponentInput {
                buttons: B,
                stick: [1.0, 0.0],
                frames: 8,
            },
            Action::Controller {
                buttons: A | B,
                stick: [0.25, -0.5],
                cstick: [0.75, -1.0],
                frames: 4,
                reference: InputReference::Facing,
                cstick_reference: InputReference::Opponent,
            },
            Action::Controller {
                buttons: 0,
                stick: [0.0; 2],
                cstick: [0.0; 2],
                frames: 255,
                reference: InputReference::Screen,
                cstick_reference: InputReference::Screen,
            },
            Action::Tap {
                buttons: A | B,
                stick: [0.0, -0.6],
                hold: 2,
                gap: 3,
                frames: 1800,
            },
            Action::CStick {
                stick: [1.0, 0.0],
                frames: 255,
            },
            Action::Chase {
                gap: 12.5,
                strength: 0.5,
                frames: 14,
            },
            Action::Wait(0),
            Action::Wait(255),
            Action::LedgePosition {
                offset: 12.5,
                strength: 0.65,
                tolerance: 1.0,
                brake: 6.0,
                frames: 180,
            },
            Action::Approach {
                airborne: false,
                strength: 0.5,
                frames: 3,
                gap: 10.0,
            },
            Action::Approach {
                airborne: true,
                strength: 1.0,
                frames: 3,
                gap: 0.0,
            },
            Action::Range(RangeCheck::Ignore),
            Action::Range(RangeCheck::Verify(4.5)),
            Action::Range(RangeCheck::FighterGrabRange),
            Action::Range(RangeCheck::FighterGrabRangeInstant),
            Action::ShieldUntilBlock(9),
            Action::ShieldUntilManual,
        ],
        vec![
            Action::IfFacingLedge { away: false },
            Action::ElseFacing,
            Action::EndFacing,
            Action::IfFacingLedge { away: true },
            Action::EndFacing,
            Action::IfFacingOpponent { away: false },
            Action::ElseFacing,
            Action::EndFacing,
            Action::IfFacingOpponent { away: true },
            Action::EndFacing,
        ],
        Condition::ALL
            .iter()
            .map(|&condition| Action::Check {
                condition,
                value: if condition.has_distance() { 12.5 } else { 0.0 },
                wait: if condition.has_distance() { 4 } else { 255 },
            })
            .collect(),
        directions.iter().map(|&d| Action::Face(d)).collect(),
        directions
            .iter()
            .map(|&direction| Action::Run {
                direction,
                units: 4.5,
            })
            .collect(),
        directions
            .iter()
            .map(|&direction| Action::Drift {
                direction,
                strength: 0.25,
                frames: 5,
            })
            .collect(),
        directions
            .iter()
            .map(|&direction| Action::Walk {
                direction,
                units: 4.0,
                strength: 0.35,
                min_gap: (direction == Direction::Opponent).then_some(4.5),
            })
            .collect(),
    ];
    groups.push(vec![Action::Walk {
        direction: Direction::Opponent,
        units: 100.0,
        strength: 0.95,
        min_gap: None,
    }]);
    let mut covered = std::collections::BTreeSet::from([variant_index(&Action::Empty)]);
    for group in &groups {
        let text = group
            .iter()
            .map(|a| action(*a).unwrap())
            .collect::<Vec<_>>()
            .join(", ");
        let parsed = Catalog::parse(&format!(
            "[kirby.neutral.rt]\nstart_delay=0\nactions = {text}\n"
        ))
        .unwrap_or_else(|e| panic!("{e}\n{text}"));
        assert_eq!(parsed.entries()[0].sequence.actions(), &group[..], "{text}");
        covered.extend(group.iter().map(variant_index));
    }
    assert_eq!(
        covered.len(),
        22,
        "an Action variant is missing from the round trip"
    );
    assert!(action(Action::Empty).is_err());
}

#[test]
fn mode_swap_text_migration_changes_only_legacy_headers() {
    let source = "# authored\r\n  [kirby.dpad_up.My_Swap] # keep\r\nname = My swap\r\nbuiltin = l_mode_swap\r\ndefault = true\r\n[kirby.dpad_up.taunt]\r\nbuiltin=normal\r\n[bowser.y.keep]\nbuiltin=normal";
    let expected = source.replace("[kirby.dpad_up.My_Swap]", "[kirby.r3.My_Swap]");
    let migrated = migrate_mode_swap(source).unwrap();
    assert_eq!(migrated, expected);
    assert_eq!(migrate_mode_swap(&migrated).unwrap(), migrated);
}
