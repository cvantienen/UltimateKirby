use super::*;
use crate::input::ButtonSet;

/// Regression test for `all_scenarios_fit_and_disabled_settings_survive_other_tools`.
#[test]
fn all_scenarios_fit_and_disabled_settings_survive_other_tools() {
    let mut config = AssistConfig::default();
    let scenarios = [
        Scenario::Tech,
        Scenario::Clatter,
        Scenario::Tumble,
        Scenario::Hitstun,
        Scenario::Parry,
        Scenario::Footstool,
        Scenario::Landing,
        Scenario::Ledge,
        Scenario::Grounded,
        Scenario::Airborne,
    ];
    let tech = ScenarioRule::controller(
        Scenario::Tech,
        RuleTiming::after(2),
        InputRecipe::buttons(ButtonSet::SHIELD),
    );
    assert!(config.set_rule(tech));
    assert!(config.set_rule_enabled(Scenario::Tech, false));
    for scenario in &scenarios[1..] {
        assert!(config.set_rule(ScenarioRule::controller(
            *scenario,
            RuleTiming::IMMEDIATELY_ONCE,
            InputRecipe::NONE,
        )));
    }
    assert!(config.set_rule_enabled(Scenario::Tech, true));
    assert_eq!(config.rule_for(Scenario::Tech), Some(tech));
    assert!(config.remove_rule(Scenario::Tech));
    assert!(!config.remove_rule(Scenario::Tech));
    assert_eq!(config.rule_for(Scenario::Tech), None);
    assert!(!config.set_rule(ScenarioRule::DISABLED));
    assert!(!config.set_rule_enabled(Scenario::None, true));
    assert!(!config.remove_rule(Scenario::None));
    assert!(config.set_rule(tech));
}

/// Constructs the rule replaces old rule for the same scenario.
#[test]
fn new_rule_replaces_old_rule_for_the_same_scenario() {
    let mut config = AssistConfig::default();
    assert!(config.set_rule(ScenarioRule::controller(
        Scenario::Clatter,
        RuleTiming::IMMEDIATELY_ONCE,
        InputRecipe::buttons(ButtonSet::ATTACK),
    )));
    assert!(config.set_rule(ScenarioRule::controller(
        Scenario::Clatter,
        RuleTiming::pulse_every(3),
        InputRecipe::buttons(ButtonSet::JUMP),
    )));

    assert_eq!(
        config.rule_for(Scenario::Clatter).unwrap().timing,
        RuleTiming::pulse_every(3)
    );
}

/// Regression test for `timing_supports_delays_and_pulses`.
#[test]
fn timing_supports_delays_and_pulses() {
    let timing = RuleTiming {
        start_after_frames: 2,
        active_for_frames: 1,
        repeat_every_frames: 3,
    };

    assert!(!timing.is_active_on(0));
    assert!(!timing.is_active_on(1));
    assert!(timing.is_active_on(2));
    assert!(!timing.is_active_on(3));
    assert!(timing.is_active_on(5));
}
