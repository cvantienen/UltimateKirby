use super::*;
use crate::{
    AssistConfig, AssistEngine, ControllerFrame, DecisionStatus, FighterKind, FighterObservation,
    InputPermissions, RunContext, ScenarioSample,
};

/// Regression test for `tools_obey_priority_release_gaps_and_independent_toggles`.
#[test]
fn tools_obey_priority_release_gaps_and_independent_toggles() {
    let mut config = AssistConfig {
        enabled: true,
        ..AssistConfig::default()
    };
    assert!(config.set_rule(AutoTech::default().rule()));
    assert!(config.set_rule(AutoMash::default().rule()));
    let evaluate = |config, observation| {
        AssistEngine.evaluate(
            config,
            RunContext {
                is_locally_human_controlled: true,
                fighter: FighterKind::Kirby,
            },
            observation,
            ControllerFrame::default(),
            InputPermissions::default(),
        )
    };
    let overlapping = FighterObservation {
        active: [
            ScenarioSample::new(Scenario::Clatter, 0),
            ScenarioSample::new(Scenario::Tech, 0),
            ScenarioSample::EMPTY,
        ],
    };
    let tech = evaluate(config, overlapping);
    assert_eq!(tech.proposed.buttons, ButtonSet::SHIELD);
    assert_eq!(tech.selected_scenario, Some(Scenario::Tech));

    for frame in 0..7 {
        let decision = evaluate(config, FighterObservation::one(Scenario::Clatter, frame));
        assert_eq!(
            decision.proposed.buttons,
            if frame % 3 == 0 {
                ButtonSet::ATTACK
            } else {
                ButtonSet::EMPTY
            }
        );
    }
    assert!(config.set_rule_enabled(Scenario::Tech, false));
    assert_eq!(
        evaluate(config, overlapping).status,
        DecisionStatus::NoRule(Scenario::Tech)
    );
    assert_eq!(
        evaluate(config, FighterObservation::one(Scenario::Clatter, 0))
            .proposed
            .buttons,
        ButtonSet::ATTACK
    );
    assert!(config.set_rule_enabled(Scenario::Tech, true));
    assert_eq!(
        evaluate(config, overlapping).proposed.buttons,
        ButtonSet::SHIELD
    );
}

/// Regression test for `mash_always_leaves_a_release_frame`.
#[test]
fn mash_always_leaves_a_release_frame() {
    for interval in [0, 1, 2] {
        let rule = AutoMash {
            interval_frames: interval,
            ..AutoMash::default()
        }
        .rule();
        assert!(rule.timing.is_active_on(0));
        assert!(!rule.timing.is_active_on(1));
        assert!(rule.timing.is_active_on(2));
    }
}
