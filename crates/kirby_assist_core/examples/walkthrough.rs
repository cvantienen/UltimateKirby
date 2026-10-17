//! A pretend game adapter for learning the complete frame-processing path.
//! This runs on your computer and never reads or writes a real controller.
//! Read the three trait implementations below as the contract between a data
//! source and AssistRuntime. The Switch integration uses callbacks instead, but
//! the separation of observation, fresh human input, and final output is shared.
use kirby_assist_core::{
    AssistRuntime, AutoMash, AutoTech, ButtonSet, ControllerFrame, FighterKind, FighterObservation,
    GameAdapter, GameObservationSource, InputBridge, RunContext, Scenario,
};

// An adapter normally wraps game data. This one stores pretend data instead.
struct DemoAdapter {
    observation: FighterObservation,
    human: ControllerFrame,
    output: ControllerFrame,
}

// `impl Trait for Type` promises that DemoAdapter supplies the trait's methods.
// &self borrows for reading; &mut self below permits updating output.
impl GameObservationSource for DemoAdapter {
    fn observe_fighter(&self) -> Option<FighterObservation> {
        Some(self.observation)
    }
}

impl InputBridge for DemoAdapter {
    fn read_human_input(&self) -> Option<ControllerFrame> {
        // Read the human's frame, never our last output.
        Some(self.human)
    }

    fn write_merged_input(&mut self, frame: ControllerFrame) {
        // A real adapter would apply input to the game's current input buffer.
        // Assignment here only changes a Rust variable.
        self.output = frame;
    }
}

impl GameAdapter for DemoAdapter {
    fn read_context(&self) -> Option<RunContext> {
        Some(RunContext {
            is_locally_human_controlled: true,
            fighter: FighterKind::Kirby,
        })
    }
}

fn main() {
    let mut runtime = AssistRuntime::default();
    runtime.config.enabled = true;
    assert!(runtime.config.set_rule(AutoTech::default().rule()));
    assert!(runtime.config.set_rule(AutoMash::default().rule()));

    // There are three separate values: observation (what happened), human
    // (what the player currently holds), output (merged result). Never assign
    // output back to human or a synthetic hold can feed itself indefinitely.
    let mut adapter = DemoAdapter {
        observation: FighterObservation::EMPTY,
        human: ControllerFrame {
            buttons: ButtonSet::JUMP,
            ..ControllerFrame::default()
        },
        output: ControllerFrame::default(),
    };

    println!("Demo only: the human holds Jump throughout.");
    println!("scenario | scenario frame | decision | output Jump/Attack/Shield");
    let frames = [
        (Scenario::Clatter, 0),
        (Scenario::Clatter, 1),
        (Scenario::Clatter, 2),
        (Scenario::Clatter, 3),
        (Scenario::None, 0),
        (Scenario::Clatter, 0), // Re-entry resets this scenario's counter.
        (Scenario::Tech, 0),
        (Scenario::Tech, 1),
    ];

    // Tuple destructuring gives names to each pretend scenario and its age.
    // if let Some(...) handles a successful frame; None means a required read
    // was unavailable, so the generic runtime did not write output.
    for (scenario, scenario_frame) in frames {
        adapter.observation = FighterObservation::one(scenario, scenario_frame);
        if let Some(decision) = runtime.process_frame(&mut adapter) {
            println!(
                "{scenario:?} | {scenario_frame} | {:?} | {}/{}/{}",
                decision.status,
                adapter.output.buttons.contains(ButtonSet::JUMP),
                adapter.output.buttons.contains(ButtonSet::ATTACK),
                adapter.output.buttons.contains(ButtonSet::SHIELD),
            );
        }
    }

    runtime.config.enabled = false;
    adapter.observation = FighterObservation::one(Scenario::Clatter, 0);
    if let Some(decision) = runtime.process_frame(&mut adapter) {
        println!(
            "Assistance off: {:?}; human Jump remains held.",
            decision.status
        );
    }
}
