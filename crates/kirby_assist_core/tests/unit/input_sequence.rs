use super::*;
use crate::ButtonSet;

/// Regression test for `sequence_has_release_gaps_and_ends_without_stuck_inputs`.
#[test]
fn sequence_has_release_gaps_and_ends_without_stuck_inputs() {
    let attack = InputRecipe::buttons(ButtonSet::ATTACK);
    let steps = [
        InputStep::hold(1, attack),
        InputStep::wait(2),
        InputStep::hold(1, attack),
    ];
    let sequence = InputSequence { steps: &steps };
    assert_eq!(sequence.recipe_at(0), Some(attack));
    assert_eq!(sequence.recipe_at(1), Some(InputRecipe::NONE));
    assert_eq!(sequence.recipe_at(2), Some(InputRecipe::NONE));
    assert_eq!(sequence.recipe_at(3), Some(attack));
    assert_eq!(sequence.recipe_at(4), None);
    assert_eq!(sequence.recipe_at(u32::MAX), None);
}
