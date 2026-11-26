use super::*;
use crate::{
    AutoMash, ButtonSet, ControllerFrame, FighterKind, FighterObservation, GameObservationSource,
    InputBridge, RunContext, Scenario,
};

/// Runtime Smash data carried by `Adapter`.
struct Adapter {
    /// Runtime observation value used by Adapter..
    observation: Option<FighterObservation>,
    /// Runtime human value used by Adapter..
    human: Option<ControllerFrame>,
    /// Runtime context value used by Adapter..
    context: Option<RunContext>,
    /// Runtime writes value used by Adapter..
    writes: Vec<ControllerFrame>,
}

impl GameObservationSource for Adapter {
    /// Observes the fighter.
    fn observe_fighter(&self) -> Option<FighterObservation> {
        self.observation
    }
}
impl InputBridge for Adapter {
    /// Reads the human input.
    fn read_human_input(&self) -> Option<ControllerFrame> {
        self.human
    }
    /// Writes the merged input.
    fn write_merged_input(&mut self, frame: ControllerFrame) {
        self.writes.push(frame);
    }
}
impl GameAdapter for Adapter {
    /// Reads the context.
    fn read_context(&self) -> Option<RunContext> {
        self.context
    }
}

/// Regression test for `runner_releases_pulses_and_handles_disable_exit_and_missing_state`.
#[test]
fn runner_releases_pulses_and_handles_disable_exit_and_missing_state() {
    let human = ControllerFrame {
        buttons: ButtonSet::JUMP,
        ..ControllerFrame::default()
    };
    let mut adapter = Adapter {
        observation: Some(FighterObservation::one(Scenario::Clatter, 0)),
        human: Some(human),
        context: Some(RunContext {
            is_locally_human_controlled: true,
            fighter: FighterKind::Kirby,
        }),
        writes: Vec::new(),
    };
    let mut runtime = AssistRuntime::default();
    runtime.config.enabled = true;
    assert!(runtime.config.set_rule(AutoMash::default().rule()));
    runtime.process_frame(&mut adapter).unwrap();
    assert_eq!(
        adapter.writes.last().unwrap().buttons,
        ButtonSet::JUMP.union(ButtonSet::ATTACK)
    );
    adapter.observation = Some(FighterObservation::one(Scenario::Clatter, 1));
    runtime.process_frame(&mut adapter).unwrap();
    assert_eq!(adapter.writes.last(), Some(&human));
    // A new scenario occurrence starts its timing at zero again.
    adapter.observation = Some(FighterObservation::one(Scenario::Clatter, 0));
    runtime.process_frame(&mut adapter).unwrap();
    assert!(adapter
        .writes
        .last()
        .unwrap()
        .buttons
        .contains(ButtonSet::ATTACK));
    runtime.config.enabled = false;
    runtime.process_frame(&mut adapter).unwrap();
    assert_eq!(adapter.writes.last(), Some(&human));
    runtime.config.enabled = true;
    adapter.observation = Some(FighterObservation::EMPTY);
    runtime.process_frame(&mut adapter).unwrap();
    assert_eq!(adapter.writes.last(), Some(&human));

    let count = adapter.writes.len();
    adapter.observation = None;
    assert!(runtime.process_frame(&mut adapter).is_none());
    adapter.observation = Some(FighterObservation::EMPTY);
    adapter.human = None;
    assert!(runtime.process_frame(&mut adapter).is_none());
    adapter.human = Some(human);
    adapter.context = None;
    assert!(runtime.process_frame(&mut adapter).is_none());
    assert_eq!(adapter.writes.len(), count + 2);
    assert_eq!(adapter.writes.last(), Some(&human));
}
