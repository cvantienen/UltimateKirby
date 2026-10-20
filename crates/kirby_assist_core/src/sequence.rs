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

/// A reactive combo step: send one input, then wait for the requested native
/// action (and optionally its hit). A rejected input or whiff cannot advance it.
#[derive(Clone, Copy, Debug)]
pub struct ConfirmedStep {
    pub recipe: InputRecipe,
    pub accepted_status: i32,
    pub require_hit: bool,
    pub timeout_frames: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SequenceObservation {
    /// A real simulation-frame ID supplied by the adapter, never a hook count.
    pub frame: u64,
    pub target: u32,
    pub status: i32,
    pub can_act: bool,
    pub hit_confirmed: bool,
    pub stopped: bool,
    pub interrupted: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SequenceState {
    Ready,
    WaitingForConfirmation,
    Complete,
    Cancelled,
}

/// Building block for measured combos. Not wired to live playback slots: each
/// combo still needs a verified trigger, simulation clock and move/hit evidence.
pub struct ConfirmedSequence<'a> {
    steps: &'a [ConfirmedStep],
    target: u32,
    index: usize,
    issued: bool,
    age: u64,
    last_frame: Option<u64>,
    state: SequenceState,
    output: InputRecipe,
}

impl<'a> ConfirmedSequence<'a> {
    pub fn new(steps: &'a [ConfirmedStep], target: u32) -> Self {
        Self {
            steps,
            target,
            index: 0,
            issued: false,
            age: 0,
            last_frame: None,
            state: if steps.is_empty() {
                SequenceState::Complete
            } else {
                SequenceState::Ready
            },
            output: InputRecipe::NONE,
        }
    }

    pub fn state(&self) -> SequenceState {
        self.state
    }

    pub fn sample(&mut self, observation: SequenceObservation) -> InputRecipe {
        // Cancellation wins over the duplicate-frame cache, including damage
        // or target reassignment between callbacks in the same simulation frame.
        if observation.interrupted
            || observation.target != self.target
            || self.last_frame.is_some_and(|last| observation.frame < last)
        {
            self.state = SequenceState::Cancelled;
        }
        if matches!(
            self.state,
            SequenceState::Complete | SequenceState::Cancelled
        ) {
            return InputRecipe::NONE;
        }
        if observation.stopped {
            self.last_frame = Some(observation.frame);
            self.output = InputRecipe::NONE;
            return self.output;
        }
        if self.last_frame == Some(observation.frame) {
            return self.output;
        }
        let elapsed = self.last_frame.map_or(0, |last| observation.frame - last);
        self.last_frame = Some(observation.frame);
        self.output = InputRecipe::NONE;
        let step = self.steps[self.index];
        self.age = self.age.saturating_add(elapsed);
        if self.issued {
            if self.age > u64::from(step.timeout_frames.max(1)) {
                self.state = SequenceState::Cancelled;
                return self.output;
            }
            if observation.status == step.accepted_status
                && (!step.require_hit || observation.hit_confirmed)
            {
                self.index += 1;
                self.issued = false;
                self.age = 0;
                self.state = if self.index == self.steps.len() {
                    SequenceState::Complete
                } else {
                    SequenceState::Ready
                };
                // Separate confirmation and the next request by a release frame.
                return self.output;
            }
            if self.age >= u64::from(step.timeout_frames.max(1)) {
                self.state = SequenceState::Cancelled;
            }
        } else if self.age >= u64::from(step.timeout_frames.max(1)) {
            self.state = SequenceState::Cancelled;
        } else if observation.can_act && observation.status != step.accepted_status {
            self.output = step.recipe;
            self.issued = true;
            self.state = SequenceState::WaitingForConfirmation;
        }
        self.output
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
