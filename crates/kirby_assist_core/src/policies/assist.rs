//! Scenario selection and logical controller-frame proposals.
//! Read this after state.rs/config.rs: evaluate is a pure function of its inputs.
//! There is no remembered timer here; the adapter supplies each scenario's age.
//! This makes a rule easy to test using small, fabricated observations.

use crate::{
    config::{AssistConfig, RecordingSlot, ScenarioAction},
    input::{arbitrate, ArbitrationResult, ControllerFrame, InputPermissions, InputRecipe},
    state::{FighterObservation, RunContext, Scenario},
};

/// Why the engine did or did not propose an action this frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
// An enum is one named alternative at a time. Variants carrying (Scenario)
// also keep the reason's context, which is useful in tests and diagnostics.
///
pub enum DecisionStatus {
    /// `Disabled` option in the `DecisionStatus` Smash policy state.
    Disabled,
    /// `ScopeRejected` option in the `DecisionStatus` Smash policy state.
    ScopeRejected,
    /// `NoActiveScenario` option in the `DecisionStatus` Smash policy state.
    NoActiveScenario,
    /// `NoRule` option in the `DecisionStatus` Smash policy state.
    NoRule(
        /// Carries the scenario with no enabled rule in `DecisionStatus`.
        Scenario,
    ),
    /// `WaitingForTiming` option in the `DecisionStatus` Smash policy state.
    WaitingForTiming(
        /// Carries the scenario waiting for its timing window in `DecisionStatus`.
        Scenario,
    ),
    /// `ProposedController` option in the `DecisionStatus` Smash policy state.
    ProposedController(
        /// Carries the scenario producing a controller proposal in `DecisionStatus`.
        Scenario,
    ),
    /// `AwaitingPlaybackBackend` option in the `DecisionStatus` Smash policy state.
    AwaitingPlaybackBackend {
        /// Runtime scenario value used by DecisionStatus..
        scenario: Scenario,
        /// Runtime slot value used by DecisionStatus..
        slot: RecordingSlot,
    },
}

/// A transparent logical result. `proposed` is never written to Smash by this
/// crate; the platform adapter decides whether and how to use it. The live
/// policy consumes `recipe`, while generic examples consume `proposed`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameDecision {
    /// Runtime proposed value used by FrameDecision..
    pub proposed: ControllerFrame,
    /// Runtime recipe value used by FrameDecision..
    pub recipe: InputRecipe,
    /// Runtime arbitration value used by FrameDecision..
    pub arbitration: Option<ArbitrationResult>,
    /// Runtime selected scenario value used by FrameDecision..
    pub selected_scenario: Option<Scenario>,
    /// native fighter status.
    pub status: DecisionStatus,
}

/// Stateless policy engine. It deliberately has no Skyline, Smash, HID, or
/// memory access, making it easy to test and safe to evolve in isolation.
#[derive(Clone, Copy, Debug, Default)]
pub struct AssistEngine;

impl AssistEngine {
    // The early returns preserve the fresh human frame when a prerequisite is
    // missing. Do not accidentally substitute a previous automated frame here.
    /// Returns the `evaluate` result.
    pub fn evaluate(
        &self,
        config: AssistConfig,
        context: RunContext,
        observation: FighterObservation,
        user_input: ControllerFrame,
        permissions: InputPermissions,
    ) -> FrameDecision {
        if !config.enabled {
            return FrameDecision::unchanged(user_input, DecisionStatus::Disabled);
        }

        if !config.scope.allows(context) {
            return FrameDecision::unchanged(user_input, DecisionStatus::ScopeRejected);
        }

        // Select priority BEFORE looking up an enabled rule. A broad Grounded
        // rule must not take over just because a more specific Tumble rule is off.
        let Some(sample) = observation.highest_priority() else {
            return FrameDecision::unchanged(user_input, DecisionStatus::NoActiveScenario);
        };

        let Some(rule) = config.rule_for(sample.scenario) else {
            return FrameDecision::unchanged(user_input, DecisionStatus::NoRule(sample.scenario));
        };

        if !rule.timing.is_active_on(sample.frames_active) {
            return FrameDecision::unchanged(
                user_input,
                DecisionStatus::WaitingForTiming(sample.scenario),
            );
        }

        // `match` handles every action alternative. Rust will ask you to update
        // this expression if ScenarioAction gains another variant. Keep this
        // function pure: the adapter is responsible for applying the result.
        match rule.action {
            ScenarioAction::ControllerRecipe(recipe) => {
                let arbitration = arbitrate(user_input, recipe, permissions);
                FrameDecision {
                    proposed: arbitration.frame,
                    recipe,
                    arbitration: Some(arbitration),
                    selected_scenario: Some(sample.scenario),
                    status: DecisionStatus::ProposedController(sample.scenario),
                }
            }
            // Naming a playback slot is only a request to a future backend.
            // It returns the human input unchanged; no sequence runs here.
            ScenarioAction::PlaybackSlot(slot) => FrameDecision {
                proposed: user_input,
                recipe: InputRecipe::NONE,
                arbitration: None,
                selected_scenario: Some(sample.scenario),
                status: DecisionStatus::AwaitingPlaybackBackend {
                    scenario: sample.scenario,
                    slot,
                },
            },
        }
    }
}

impl FrameDecision {
    /// Returns the `unchanged` result.
    const fn unchanged(proposed: ControllerFrame, status: DecisionStatus) -> Self {
        Self {
            proposed,
            recipe: InputRecipe::NONE,
            arbitration: None,
            selected_scenario: None,
            status,
        }
    }
}

// Compiled only by cargo test on the host; these assertions are not live
// Switch logging or gameplay callbacks. Add regressions here for new behavior.
#[cfg(test)]
#[path = "../../tests/unit/policy_assist.rs"]
mod tests;
