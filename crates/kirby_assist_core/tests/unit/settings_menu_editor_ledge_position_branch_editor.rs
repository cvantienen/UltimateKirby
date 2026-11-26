use super::*;
/// Regression test for `positioning_fields_edit_and_save_without_converting_to_an_imported_action`.
#[test]
fn positioning_fields_edit_and_save_without_converting_to_an_imported_action() {
    let c = crate::sequence_config::Catalog::parse(
        "[bowser.attack.test]\nstart_delay=0\nactions=run_to_ledge:12:1:1:6:180\n",
    )
    .unwrap();
    let source = c.entries()[0].sequence;
    let key = (0, 0);
    let mut e = RecipeEditor::default();
    e.ensure(key, source);
    assert!(e.recipes[&key].kinds[0] < PRESETS.len());
    for (field, up) in [
        (Field::Distance(0), true),
        (Field::Strength(0), false),
        (Field::Tolerance(0), true),
        (Field::Brake(0), true),
        (Field::PositionTimeout(0), true),
    ] {
        e.cursor = fields(&e.recipes[&key])
            .iter()
            .position(|f| *f == field)
            .unwrap();
        assert!(e.adjust(key, up));
    }
    let edited = e.compiled(key).unwrap();
    assert_eq!(
        edited.actions()[0],
        Action::LedgePosition {
            offset: 13.,
            strength: 0.99,
            tolerance: 1.1,
            brake: 6.5,
            frames: 181
        }
    );
    let saved = crate::sequence_config::persist::update(
        "",
        15,
        crate::fighter_profiles::presets::PresetSlot::Attack,
        "test",
        edited,
        true,
    )
    .unwrap();
    assert_eq!(
        crate::sequence_config::Catalog::parse(&saved)
            .unwrap()
            .entries()[0]
            .sequence,
        edited
    );
    let panel = e.panel(key, true).body.join(" ");
    for label in [
        "Inward distance from ledge",
        "Arrival tolerance",
        "Braking distance",
        "Movement timeout",
    ] {
        assert!(panel.contains(label), "{label}");
    }
}
/// Regression test for `opponent_branch_controls_are_zero_time_and_keep_single_branch_timing_label`.
#[test]
fn opponent_branch_controls_are_zero_time_and_keep_single_branch_timing_label() {
    let cat=crate::sequence_config::Catalog::parse("[bowser.attack.test]\nstart_delay=0\nactions=if_facing_opponent,input:a:0:0:1,else_facing,input:b:0:0:1,end_facing\n").unwrap();
    let key = (0, 0);
    let mut e = RecipeEditor::default();
    e.ensure(key, cat.entries()[0].sequence);
    assert!([0, 2, 4]
        .into_iter()
        .all(|i| e.recipes[&key].kinds[i] < PRESETS.len()));
    for a in [
        Action::IfFacingOpponent { away: false },
        Action::ElseFacing,
        Action::EndFacing,
    ] {
        assert_eq!(scheduled_frames(a), Some(0));
    }
    assert!(e
        .panel(key, true)
        .header
        .join(" ")
        .contains("only the matching facing block runs"));
    assert_eq!(
        e.compiled(key).unwrap().actions(),
        cat.entries()[0].sequence.actions()
    );
}
