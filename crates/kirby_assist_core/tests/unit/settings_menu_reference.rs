use super::*;
/// Regression test for `correct_fighter_move_and_special_variants_are_used`.
#[test]
fn correct_fighter_move_and_special_variants_are_used() {
    let kirby = hints(6, PresetSlot::X, Some("Dtilt")).join(" ");
    let yoshi = hints(5, PresetSlot::X, Some("Dtilt")).join(" ");
    assert!(kirby.contains("kirby / Down Tilt"));
    assert!(yoshi.contains("yoshi / Down Tilt"));
    assert_ne!(
        timing(find_move(6, "Down Tilt").unwrap()),
        timing(find_move(5, "Down Tilt").unwrap())
    );
    let bowser = hints(15, PresetSlot::X, Some("Up B")).join(" ");
    assert!(bowser.contains("Whirling Fortress"));
    assert!(bowser.contains("Air"));
}
/// Regression test for `roster_reference_is_labeled_and_does_not_claim_live_countdown`.
#[test]
fn roster_reference_is_labeled_and_does_not_claim_live_countdown() {
    let hint = hints(6, PresetSlot::Neutral, None).join(" ");
    assert!(hint.contains("roster avg"));
    assert!(hint.contains("invuln through"));
    assert!(hint.contains("Not a live countdown"));
    assert!(hints(6, PresetSlot::Y, None).is_empty());
    assert!(hints(6, PresetSlot::Drop, None).is_empty());
    assert_eq!(ledge_frames("1-32 intangible / 33 total"), Some((32, 33)));
    assert_eq!(ledge_frames("?"), None);
}
/// Regression test for `input_aliases_never_claim_ground_or_air_state_is_known`.
#[test]
fn input_aliases_never_claim_ground_or_air_state_is_known() {
    let hint = hints(6, PresetSlot::X, Some("Neutral A / Nair")).join(" ");
    assert!(hint.contains("Jab 1"));
    assert!(hint.contains("Neutral Air"));
    assert!(hints(6, PresetSlot::X, Some("Controller input"))
        .join(" ")
        .contains("game state"));
}
