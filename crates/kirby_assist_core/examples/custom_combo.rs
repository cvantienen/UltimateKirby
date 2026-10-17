//! Run on your Mac with `cargo run --example custom_combo` in this crate.
//! Teaches sequence durations and additive merging using printed output only.
//! Editing this example does not install a combo or enable recorded-slot playback
//! in the Switch plugin; live triggering/cancellation would need its own wiring.
use kirby_assist_core::{
    arbitrate, ButtonSet, ControllerFrame, InputPermissions, InputRecipe, InputSequence, InputStep,
};

fn main() {
    // Illustrative timing only; this is not a verified Kirby combo.
    // Durations accumulate: the first Attack is at 0; wait occupies 1..8;
    // Jump is at 9; wait at 10..12; final Attack at 13. A wait releases the
    // assist's button request, but additive merging preserves human-held buttons.
    let steps = [
        InputStep::hold(1, InputRecipe::buttons(ButtonSet::ATTACK)),
        InputStep::wait(8),
        InputStep::hold(1, InputRecipe::buttons(ButtonSet::JUMP)),
        InputStep::wait(3),
        InputStep::hold(1, InputRecipe::buttons(ButtonSet::ATTACK)),
    ];
    // &steps borrows the array: the sampler reads it without taking ownership.
    let combo = InputSequence { steps: &steps };
    for frame in 0..20 {
        let Some(recipe) = combo.recipe_at(frame) else {
            println!("frame {frame}: finished; return to human input");
            break;
        };
        // A live adapter must read a fresh human frame each time.
        let human = ControllerFrame::default();
        let result = arbitrate(human, recipe, InputPermissions::default());
        println!("frame {frame}: {:?}", result.frame.buttons);
    }
}
