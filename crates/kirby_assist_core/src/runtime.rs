//! Generic adapter runner for teaching, host tests, and future synchronous tools.
//! The real Switch integration does NOT call this runner: it coordinates game
//! queries and raw mapper input through its own shared State in src/live.rs.
//! `impl GameAdapter` accepts any type implementing the adapter traits.
use crate::{AssistConfig, AssistEngine, FrameDecision, GameAdapter, InputPermissions};

#[derive(Clone, Debug, Default)]
pub struct AssistRuntime {
    pub config: AssistConfig,
    pub permissions: InputPermissions,
}

impl AssistRuntime {
    /// Call once per simulation frame, using fresh human input rather than the
    /// previous merged output. The adapter owns frame counters and resets them
    /// on scenario re-entry. Missing fighter state releases to fresh human input when available.
    pub fn process_frame(&self, adapter: &mut impl GameAdapter) -> Option<FrameDecision> {
        // `?` unwraps Some(value), or returns None immediately if a read fails.
        // A missing human input read cannot safely reconstruct physical input;
        // adapters retaining output need a watchdog for that particular path.
        let human = adapter.read_human_input()?;
        let Some((context, observation)) = adapter.read_context().zip(adapter.observe_fighter())
        else {
            // Fresh physical input is available: release any previous recipe
            // even when a fighter observation/context disappears during loading.
            adapter.write_merged_input(human);
            return None;
        };
        // &self reads configuration; &mut adapter permits the later output
        // write. The engine returns a value rather than holding that mutable
        // borrow across observation or input application.
        let decision =
            AssistEngine.evaluate(self.config, context, observation, human, self.permissions);
        // Write the fresh human frame on waiting/disabled frames too, so an
        // earlier assist pulse is released instead of remaining latched.
        adapter.write_merged_input(decision.proposed);
        Some(decision)
    }
}

// Compiled only by cargo test on the host; these assertions are not live
// Switch logging or gameplay callbacks. Add regressions here for new behavior.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AutoMash, ButtonSet, ControllerFrame, FighterKind, FighterObservation,
        GameObservationSource, InputBridge, RunContext, Scenario,
    };

    struct Adapter {
        observation: Option<FighterObservation>,
        human: Option<ControllerFrame>,
        context: Option<RunContext>,
        writes: Vec<ControllerFrame>,
    }

    impl GameObservationSource for Adapter {
        fn observe_fighter(&self) -> Option<FighterObservation> {
            self.observation
        }
    }
    impl InputBridge for Adapter {
        fn read_human_input(&self) -> Option<ControllerFrame> {
            self.human
        }
        fn write_merged_input(&mut self, frame: ControllerFrame) {
            self.writes.push(frame);
        }
    }
    impl GameAdapter for Adapter {
        fn read_context(&self) -> Option<RunContext> {
            self.context
        }
    }

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
}
