use super::*;
use crate::native_input::{MAPPER_A as A, MAPPER_X as X, MAPPER_Y as Y, MAPPER_ZL};
/// Returns the `rename` result.
fn rename(name: &str) -> Catalog {
    let mut c = Catalog {
        mode: Mode::Actions(0),
        ..Default::default()
    };
    c.input(A, A, name);
    c
}
#[test]
fn rename_picker_switch_invalidates_menu_content_without_editing_recipe() {
    let p = MenuPresets::EMPTY;
    let mut m = MenuSelection::new(&p);
    m.show();
    m.row = MenuRow::DpadDown;
    m.choices[PresetSlot::DpadDown as usize] = PresetSlot::BUTTON_CUSTOM_OFFSET;
    m.catalog_mut().mode = Mode::Name {
        text: b"Name".to_vec(),
        cursor: 0,
        picker: NamePicker::Alphabet,
    };
    m.input(0, &p);
    let recipes = m.recipe_revision;
    for expected in ["predefined term", "letter"] {
        let before = m.content_revision;
        m.input(MAPPER_ZL, &p);
        assert_ne!(m.content_revision, before);
        assert_eq!(m.recipe_revision, recipes);
        assert!(m.recipe_panel(&p).into_lines().join(" ").contains(expected));
        let after = m.content_revision;
        m.input(MAPPER_ZL, &p);
        m.input(0, &p);
        assert_eq!(m.content_revision, after);
    }
}

/// Returns the `accepted` result.
fn accepted(c: &mut Catalog) -> String {
    match c.input(Y, Y, "") {
        Event::Rename(n) => n,
        _ => panic!("expected a valid name"),
    }
}
/// Regression test for `accepting_a_unicode_name_preserves_it_and_cursor_edits_whole_characters`.
#[test]
fn accepting_a_unicode_name_preserves_it_and_cursor_edits_whole_characters() {
    let name = "Shield → Up-B / café";
    assert_eq!(accepted(&mut rename(name)), name);
    let mut c = rename("A→é");
    c.input(DPAD_RIGHT, DPAD_RIGHT, "");
    assert!(c.lines()[1].contains("[→]"));
    c.input(A, A, "");
    assert_eq!(accepted(&mut c), "Aé");
    let mut c = rename("é");
    c.input(A, A, "");
    assert_eq!(accepted(&mut c), "é");
}
/// Regression test for `rename_inserts_at_cursor_obeys_byte_limit_and_ignores_held_direction_on_confirm`.
#[test]
fn rename_inserts_at_cursor_obeys_byte_limit_and_ignores_held_direction_on_confirm() {
    let mut c = rename("AB");
    c.input(X, X, "");
    assert!(c.lines()[1].contains("A[ ]B"));
    c.input(DPAD_DOWN, DPAD_DOWN, "");
    let expected = match &c.mode {
        Mode::Name { text, .. } => String::from_utf8(text.clone()).unwrap(),
        _ => panic!(),
    };
    let Event::Rename(actual) = c.input(Y | DPAD_DOWN, Y, "") else {
        panic!()
    };
    assert_eq!(actual, expected);
    let name = "é".repeat(32);
    let mut c = rename(&name);
    c.input(X, X, "");
    assert_eq!(accepted(&mut c), name);
    let mut c = rename(&"é".repeat(33));
    assert_eq!(accepted(&mut c), name);
}
/// Regression test for `predefined_term_picker_replaces_the_name_and_cycles`.
#[test]
fn predefined_term_picker_replaces_the_name_and_cycles() {
    let mut c = rename("Custom");
    c.input(MAPPER_ZL, MAPPER_ZL, "");
    assert!(c.lines()[2].contains("predefined term"));
    c.input(DPAD_DOWN, DPAD_DOWN, "");
    assert_eq!(accepted(&mut c), NAME_TERMS[0]);

    let mut c = rename(NAME_TERMS[0]);
    c.input(MAPPER_ZL, MAPPER_ZL, "");
    c.input(DPAD_UP, DPAD_UP, "");
    assert_eq!(accepted(&mut c), *NAME_TERMS.last().unwrap());
}
