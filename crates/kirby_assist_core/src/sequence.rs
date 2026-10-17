//! Frame-indexed input sequences for experimenting with custom combos.
//! These describe proposed inputs, not guaranteed fighter actions.
//! This sampler is used by examples. Recorded-slot playback is not connected
//! to the live plugin, and adding steps here does not automatically run a combo.
//! `InputSequence<'a>` borrows a slice; the steps must outlive the sequence.
use crate::InputRecipe;

/// Hold a recipe for this many simulation frames. An empty recipe is a wait.
#[derive(Clone, Copy, Debug)]
pub struct InputStep {
    pub frames: u16,
    pub recipe: InputRecipe,
}

impl InputStep {
    pub const fn hold(frames: u16, recipe: InputRecipe) -> Self {
        Self { frames, recipe }
    }

    pub const fn wait(frames: u16) -> Self {
        Self::hold(frames, InputRecipe::NONE)
    }
}

/// A borrowed sequence with an explicit frame cursor supplied by its caller.
/// Start at zero when the trigger fires. Cancel by stopping evaluation.
/// Advance only once per simulation frame, never once per hook invocation.
pub struct InputSequence<'a> {
    pub steps: &'a [InputStep],
}

impl InputSequence<'_> {
    /// Returns None when finished. Zero-length steps are skipped.
    pub fn recipe_at(&self, mut frame: u32) -> Option<InputRecipe> {
        // Subtract whole step durations until `frame` lies inside a step.
        // This stateless lookup can be restarted by supplying frame zero again.
        // For hold(1), wait(2), hold(1), ages 0 / 1..2 / 3 select those
        // respective steps; age 4 returns None. Some(NONE) is a deliberate wait,
        // whereas None means there are no more steps at all.
        for step in self.steps {
            if frame < u32::from(step.frames) {
                return Some(step.recipe);
            }
            frame -= u32::from(step.frames);
        }
        None
    }
}

// Compiled only by cargo test on the host; these assertions are not live
// Switch logging or gameplay callbacks. Add regressions here for new behavior.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ButtonSet;

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
}
