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
/// Plain English: Why the engine did or did not propose an action this frame.
pub enum DecisionStatus {
    /// Technical: Represents the `Disabled` case of `DecisionStatus`.
    ///
    /// Plain English: Represents the Disabled option in DecisionStatus.
    Disabled,
    /// Technical: Represents the `ScopeRejected` case of `DecisionStatus`.
    ///
    /// Plain English: Represents the ScopeRejected option in DecisionStatus.
    ScopeRejected,
    /// Technical: Represents the `NoActiveScenario` case of `DecisionStatus`.
    ///
    /// Plain English: Represents the NoActiveScenario option in DecisionStatus.
    NoActiveScenario,
    /// Technical: Represents the `NoRule` case of `DecisionStatus`.
    ///
    /// Plain English: Represents the NoRule option in DecisionStatus.
    NoRule(
        /// Technical: Carries the scenario with no enabled rule in `DecisionStatus`.
        ///
        /// Plain English: Holds the scenario with no enabled rule for this `DecisionStatus` choice.
        Scenario,
    ),
    /// Technical: Represents the `WaitingForTiming` case of `DecisionStatus`.
    ///
    /// Plain English: Represents the WaitingForTiming option in DecisionStatus.
    WaitingForTiming(
        /// Technical: Carries the scenario waiting for its timing window in `DecisionStatus`.
        ///
        /// Plain English: Holds the scenario waiting for its timing window for this `DecisionStatus` choice.
        Scenario,
    ),
    /// Technical: Represents the `ProposedController` case of `DecisionStatus`.
    ///
    /// Plain English: Represents the ProposedController option in DecisionStatus.
    ProposedController(
        /// Technical: Carries the scenario producing a controller proposal in `DecisionStatus`.
        ///
        /// Plain English: Holds the scenario producing a controller proposal for this `DecisionStatus` choice.
        Scenario,
    ),
    /// Technical: Represents the `AwaitingPlaybackBackend` case of `DecisionStatus`.
    ///
    /// Plain English: Represents the AwaitingPlaybackBackend option in DecisionStatus.
    AwaitingPlaybackBackend {
        /// Technical: Stores the `scenario` value in `DecisionStatus` as `Scenario`.
        ///
        /// Plain English: Holds the scenario value used by DecisionStatus.
        scenario: Scenario,
        /// Technical: Stores the `slot` value in `DecisionStatus` as `RecordingSlot`.
        ///
        /// Plain English: Holds the slot value used by DecisionStatus.
        slot: RecordingSlot,
    },
}

/// A transparent logical result. `proposed` is never written to Smash by this
/// crate; the platform adapter decides whether and how to use it. The live
/// policy consumes `recipe`, while generic examples consume `proposed`.
/// Plain English: A transparent logical result. proposed is never written to Smash by this crate; the platform adapter decides whether and how to use it. The live policy consumes recipe, while generic examples consume proposed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameDecision {
    /// Technical: Stores the `proposed` value in `FrameDecision` as `ControllerFrame`.
    ///
    /// Plain English: Holds the proposed value used by FrameDecision.
    pub proposed: ControllerFrame,
    /// Technical: Stores the `recipe` value in `FrameDecision` as `InputRecipe`.
    ///
    /// Plain English: Holds the recipe value used by FrameDecision.
    pub recipe: InputRecipe,
    /// Technical: Stores the `arbitration` value in `FrameDecision` as `Option<ArbitrationResult>`.
    ///
    /// Plain English: Optionally stores the arbitration for FrameDecision; it may be absent.
    pub arbitration: Option<ArbitrationResult>,
    /// Technical: Stores the `selected_scenario` value in `FrameDecision` as `Option<Scenario>`.
    ///
    /// Plain English: Optionally stores the selected scenario for FrameDecision; it may be absent.
    pub selected_scenario: Option<Scenario>,
    /// Technical: Stores the `status` value in `FrameDecision` as `DecisionStatus`.
    ///
    /// Plain English: Holds the status value used by FrameDecision.
    pub status: DecisionStatus,
}

/// Stateless policy engine. It deliberately has no Skyline, Smash, HID, or
/// memory access, making it easy to test and safe to evolve in isolation.
/// Plain English: Stateless policy engine. It deliberately has no Skyline, Smash, HID, or memory access, making it easy to test and safe to evolve in isolation.
#[derive(Clone, Copy, Debug, Default)]
pub struct AssistEngine;

impl AssistEngine {
    // The early returns preserve the fresh human frame when a prerequisite is
    // missing. Do not accidentally substitute a previous automated frame here.
    /// Technical: Returns the `evaluate` result.
    ///
    /// Plain English: Gets the evaluate result.
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
    /// Technical: Returns the `unchanged` result.
    ///
    /// Plain English: Gets the unchanged result.
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
mod tests {
    use super::*;
    use crate::{
        config::{RuleTiming, ScenarioRule},
        input::{ButtonSet, StickPosition},
        state::{FighterKind, ScenarioSample},
    };

    /// Technical: Defines the `LOCAL_KIRBY` constant used by this module.
    ///
    /// Plain English: Provides the LOCAL KIRBY constant value.
    const LOCAL_KIRBY: RunContext = RunContext {
        is_locally_human_controlled: true,
        fighter: FighterKind::Kirby,
    };

    /// Technical: Defines the `PLAYER_INPUT` constant used by this module.
    ///
    /// Plain English: Provides the PLAYER INPUT constant value.
    const PLAYER_INPUT: ControllerFrame = ControllerFrame {
        buttons: ButtonSet::JUMP,
        left_stick: StickPosition {
            x: 2_000,
            y: -2_000,
        },
        right_stick: StickPosition { x: 0, y: 4_000 },
        left_trigger: 0,
        right_trigger: 42,
    };

    /// Technical: Returns the `enabled_config` result.
    ///
    /// Plain English: Gets the enabled config result.
    fn enabled_config(rule: ScenarioRule) -> AssistConfig {
        let mut config = AssistConfig {
            enabled: true,
            ..AssistConfig::default()
        };
        assert!(config.set_rule(rule));
        config
    }

    /// Technical: Regression test for `defaults_are_a_no_op`.
    ///
    /// Plain English: Checks that defaults are a no op.
    #[test]
    fn defaults_are_a_no_op() {
        let decision = AssistEngine.evaluate(
            AssistConfig::default(),
            LOCAL_KIRBY,
            FighterObservation::one(Scenario::Clatter, 0),
            PLAYER_INPUT,
            InputPermissions::default(),
        );

        assert_eq!(decision.proposed, PLAYER_INPUT);
        assert_eq!(decision.status, DecisionStatus::Disabled);
    }

    /// Technical: Regression test for `configured_clatter_pulse_proposes_an_additive_input`.
    ///
    /// Plain English: Checks that configured clatter pulse proposes an additive input.
    #[test]
    fn configured_clatter_pulse_proposes_an_additive_input() {
        let config = enabled_config(ScenarioRule::controller(
            Scenario::Clatter,
            RuleTiming::pulse_every(3),
            InputRecipe::buttons(ButtonSet::ATTACK),
        ));
        let decision = AssistEngine.evaluate(
            config,
            LOCAL_KIRBY,
            FighterObservation::one(Scenario::Clatter, 3),
            PLAYER_INPUT,
            InputPermissions::default(),
        );

        assert_eq!(
            decision.status,
            DecisionStatus::ProposedController(Scenario::Clatter)
        );
        assert!(decision.proposed.buttons.contains(ButtonSet::ATTACK));
        assert!(decision.proposed.buttons.contains(ButtonSet::JUMP));
        assert_eq!(decision.proposed.left_stick, PLAYER_INPUT.left_stick);
    }

    /// Technical: Regression test for `higher_priority_scenario_blocks_broader_rule`.
    ///
    /// Plain English: Checks that higher priority scenario blocks broader rule.
    #[test]
    fn higher_priority_scenario_blocks_broader_rule() {
        let config = enabled_config(ScenarioRule::controller(
            Scenario::Grounded,
            RuleTiming::IMMEDIATELY_ONCE,
            InputRecipe::buttons(ButtonSet::SHIELD),
        ));
        let observation = FighterObservation {
            active: [
                ScenarioSample::new(Scenario::Grounded, 10),
                ScenarioSample::new(Scenario::Tumble, 2),
                ScenarioSample::EMPTY,
            ],
        };
        let decision = AssistEngine.evaluate(
            config,
            LOCAL_KIRBY,
            observation,
            PLAYER_INPUT,
            InputPermissions::default(),
        );

        assert_eq!(decision.status, DecisionStatus::NoRule(Scenario::Tumble));
        assert_eq!(decision.proposed, PLAYER_INPUT);
    }

    /// Technical: Regression test for `scope_rejects_other_controllers`.
    ///
    /// Plain English: Checks that scope rejects other controllers.
    #[test]
    fn scope_rejects_other_controllers() {
        let config = enabled_config(ScenarioRule::controller(
            Scenario::Clatter,
            RuleTiming::IMMEDIATELY_ONCE,
            InputRecipe::buttons(ButtonSet::ATTACK),
        ));
        let other_controller = RunContext {
            is_locally_human_controlled: false,
            ..LOCAL_KIRBY
        };
        let decision = AssistEngine.evaluate(
            config,
            other_controller,
            FighterObservation::one(Scenario::Clatter, 0),
            PLAYER_INPUT,
            InputPermissions::default(),
        );

        assert_eq!(decision.status, DecisionStatus::ScopeRejected);
        assert_eq!(decision.proposed, PLAYER_INPUT);
    }

    /// Technical: Regression test for `playback_rule_stays_inert_without_a_backend`.
    ///
    /// Plain English: Checks that playback rule stays inert without a backend.
    #[test]
    fn playback_rule_stays_inert_without_a_backend() {
        let config = enabled_config(ScenarioRule::playback(
            Scenario::Ledge,
            RuleTiming::after(5),
            RecordingSlot::One,
        ));
        let decision = AssistEngine.evaluate(
            config,
            LOCAL_KIRBY,
            FighterObservation::one(Scenario::Ledge, 5),
            PLAYER_INPUT,
            InputPermissions::default(),
        );

        assert_eq!(
            decision.status,
            DecisionStatus::AwaitingPlaybackBackend {
                scenario: Scenario::Ledge,
                slot: RecordingSlot::One,
            }
        );
        assert_eq!(decision.proposed, PLAYER_INPUT);
    }

    /// Technical: Regression test for `local_fighters_are_allowed_without_a_character_or_game_mode_requirement`.
    ///
    /// Plain English: Checks that local fighters are allowed without a character or game mode requirement.
    #[test]
    fn local_fighters_are_allowed_without_a_character_or_game_mode_requirement() {
        for fighter in [FighterKind::Kirby, FighterKind::Other, FighterKind::Unknown] {
            assert!(crate::AssistScope::default().allows(RunContext {
                fighter,
                ..LOCAL_KIRBY
            }));
        }
        assert!(!crate::AssistScope::default().allows(RunContext {
            is_locally_human_controlled: false,
            ..LOCAL_KIRBY
        }));
        assert!(!crate::AssistScope {
            require_kirby: true,
            ..crate::AssistScope::default()
        }
        .allows(RunContext {
            fighter: FighterKind::Other,
            ..LOCAL_KIRBY
        }));
    }
}
