//! Keep the copy/paste sheet tied to the real parser, not handwritten mocks.
use kirby_assist_core::{
    fighter_profiles::FighterProfiles,
    fighter_tuning::Tuning,
    reaction_rules::{Control, SelfState, Side, Trigger},
    sequence_config::Catalog,
};

const SHEET: &str = include_str!("../../../docs/SEQUENCE_SYNTAX_CHEATSHEET.md");

fn validate(text: &str) {
    let parsed = Catalog::parse(text).unwrap_or_else(|e| panic!("{e}\nExample:\n{text}"));
    for diagnostic in [false, true] {
        let mut profiles = FighterProfiles::default();
        profiles
            .add_sequence_choices(&parsed.clone().for_build(diagnostic))
            .unwrap_or_else(|e| panic!("{e}\nExample:\n{text}"));
    }
}
fn has_word(word: &str) -> bool {
    SHEET
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .any(|w| w == word)
}

#[test]
fn every_copyable_action_reaction_and_ini_example_passes_both_build_filters() {
    let mut checked = 0;
    for snippet in SHEET.split('`') {
        let snippet = snippet.trim();
        if snippet.contains('\n') || snippet.chars().any(|c| c.is_ascii_uppercase()) {
            continue;
        }
        if snippet.starts_with("actions = ") && !snippet.contains("ACTION")
            || snippet.starts_with("reaction = ")
                && !snippet.contains("EVENT")
                && snippet != "reaction = off"
        {
            validate(
                &format!(
                    "[bowser.neutral.reference]\nstart_delay=0\nactions=shield:4\n{snippet}\n"
                )
                .replace("actions=shield:4\nactions =", "actions ="),
            );
            checked += 1;
        }
    }
    for block in SHEET.split("```ini\n").skip(1) {
        validate(block.split("```").next().unwrap());
        checked += 1;
    }
    assert!(checked >= 65, "missing copyable examples: {checked}");
    assert!(Catalog::parse(
        "[bowser.neutral.invalid]\nstart_delay=0\nactions=shield\nreaction=off\n"
    )
    .is_err());
}

#[test]
fn action_aliases_directions_buttons_and_conditions_from_the_parser_are_covered() {
    let parser = include_str!("../src/sequence_config/parser/actions.rs");
    let action_source = parser
        .split("fn parse_action(")
        .nth(1)
        .unwrap()
        .split("fn parse_float(")
        .next()
        .unwrap();
    let mut checked = 0;
    for line in action_source
        .lines()
        .filter(|line| line.trim_start().starts_with('"') && line.contains("=>"))
    {
        for token in line
            .split("=>")
            .next()
            .unwrap()
            .split('"')
            .skip(1)
            .step_by(2)
        {
            assert!(
                has_word(token),
                "missing parsed action/direction alias: {token}"
            );
            checked += 1;
        }
    }
    assert!(checked > 70);
    let condition_source = include_str!("../src/sequence_config.rs")
        .split("impl Condition {")
        .nth(1)
        .unwrap()
        .split("pub fn from_token")
        .next()
        .unwrap();
    for token in condition_source
        .lines()
        .filter(|line| line.contains("=>"))
        .filter_map(|line| line.split('"').nth(1))
    {
        assert!(has_word(token), "missing condition: {token}");
    }
}

#[test]
fn every_reaction_token_state_side_and_control_has_a_reference_entry() {
    for trigger in Trigger::ALL {
        assert!(
            has_word(trigger.token()),
            "missing trigger {}",
            trigger.token()
        );
    }
    for state in SelfState::ALL {
        assert!(has_word(state.token()));
    }
    for side in Side::ALL {
        assert!(has_word(side.token()));
    }
    for control in Control::ALL {
        assert!(has_word(control.token()));
    }
}

#[test]
fn all_fighter_keys_and_their_exact_parser_bounds_are_documented() {
    let mut section = String::from("[bowser.settings]\n");
    for (i, key) in kirby_assist_core::fighter_tuning::KEYS
        .into_iter()
        .enumerate()
    {
        let row = SHEET
            .lines()
            .find(|line| line.starts_with(&format!("| `{key}` |")))
            .unwrap_or_else(|| panic!("missing setting row: {key}"));
        let (lo, hi, _) = Tuning::bounds(i);
        if Tuning::is_bool(i) {
            assert!(row.contains("`true` / `false`"));
            section.push_str(&format!("{key}=false\n"));
        } else {
            assert!(row.contains(&format!("{lo}–{hi}")), "wrong bounds: {row}");
            section.push_str(&format!("{key}={lo}\n"));
        }
    }
    validate(&section);
}

#[test]
fn all_recognized_universal_keys_including_compatibility_aliases_are_documented() {
    let source = include_str!("../src/live_input.rs");
    let source = source
        .split("match key.trim() {")
        .nth(1)
        .unwrap()
        .split("if !config.assist.walk_strength")
        .next()
        .unwrap();
    let mut checked = 0;
    for line in source.lines().filter(|line| {
        line.starts_with("                    \"") || line.starts_with("                    | \"")
    }) {
        for key in line
            .split("=>")
            .next()
            .unwrap()
            .split('"')
            .skip(1)
            .step_by(2)
        {
            assert!(has_word(key), "missing universal key: {key}");
            checked += 1;
        }
    }
    assert!(checked > 50);
}
