use super::*;
use crate::{
    config::{RuleTiming, ScenarioRule},
    input::{ButtonSet, StickPosition},
    state::{FighterKind, ScenarioSample},
};

/// `LOCAL_KIRBY` is a configuration value used by the Smash runtime.
const LOCAL_KIRBY: RunContext = RunContext {
    is_locally_human_controlled: true,
    fighter: FighterKind::Kirby,
};

/// `PLAYER_INPUT` is a configuration value used by the Smash runtime.
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

/// Returns the `enabled_config` result.
fn enabled_config(rule: ScenarioRule) -> AssistConfig {
    let mut config = AssistConfig {
        enabled: true,
        ..AssistConfig::default()
    };
    assert!(config.set_rule(rule));
    config
}

/// Regression test for `defaults_are_a_no_op`.
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

/// Regression test for `configured_clatter_pulse_proposes_an_additive_input`.
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

/// Regression test for `higher_priority_scenario_blocks_broader_rule`.
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

/// Regression test for `scope_rejects_other_controllers`.
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

/// Regression test for `playback_rule_stays_inert_without_a_backend`.
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

/// Regression test for `local_fighters_are_allowed_without_a_character_or_game_mode_requirement`.
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
