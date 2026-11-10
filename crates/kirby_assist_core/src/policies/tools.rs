//! Built-in tools compile into the same rules as custom automation.
//! A tool here is a preset, not a background worker. To add another preset,
//! return a ScenarioRule with its situation, timing, and logical InputRecipe.
//! Also wire native observation/output in the live adapter; a preset alone does
//! not make the Switch generate input for the new scenario.
use crate::{ButtonSet, InputRecipe, RuleTiming, Scenario, ScenarioRule};

/// Press shield when the adapter reports a tech opportunity.
/// The adapter must report the actionable window, not just general tumble.
/// # Fields
/// * `timing` - The timing rule for when to press shield.
///
///  Press shield when the adapter reports a tech opportunity. The adapter must report the actionable window, not just general tumble. # Fields  timing - The timing rule for when to press shield.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AutoTech {
    /// Stores the `timing` value in `AutoTech` as `RuleTiming`.
    ///
    ///
///  Holds the timing value used by AutoTech.
    pub timing: RuleTiming,
}

impl Default for AutoTech {
    /// Returns the default value for this type.
    ///
    ///
///  Returns the default value for this type.
    fn default() -> Self {
        Self {
            timing: RuleTiming::IMMEDIATELY_ONCE,
        }
    }
}

// Preset constructors only describe a rule. Native collision detection lives
// in src/live.rs + tech_prediction.rs; the live scheduler maps SHIELD to ZR.
impl AutoTech {
    /// Constructs a `ScenarioRule` for the auto-tech preset based on the current timing.
    /// # Returns
    /// A `ScenarioRule` configured for the auto-tech preset.
    ///
///  Constructs a ScenarioRule for the auto-tech preset based on the current timing. # Returns A ScenarioRule configured for the auto-tech preset.
    pub const fn rule(self) -> ScenarioRule {
        ScenarioRule::controller(
            Scenario::Tech,
            self.timing,
            InputRecipe::buttons(ButtonSet::SHIELD),
        )
    }
}

/// Repeated button pulses while trapped in a mashable/clatter state.
/// A pulse interval of at least two frames leaves an assist-release gap.
/// preset: live_input.rs translates its selected Clatter decision into a
/// left-stick cycle instead of sending these buttons on the Switch.
/// # Fields
/// * `buttons` - The set of buttons to pulse.
/// * `interval_frames` - The interval in frames between pulses.
///
///  Repeated button pulses while trapped in a mashable/clatter state. A pulse interval of at least two frames leaves an assist-release gap. preset: liveinput.rs translates its selected Clatter decision into a left-stick cycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AutoMash {
    /// Stores the `buttons` value in `AutoMash` as `ButtonSet`.
    ///
    ///
///  Holds the buttons value used by AutoMash.
    pub buttons: ButtonSet,
    /// Stores the `interval_frames` value in `AutoMash` as `u16`.
    ///
    ///
///  Sets the interval frames timing for AutoMash.
    pub interval_frames: u16,
}

impl Default for AutoMash {
    /// Constructs the default auto-mash preset with standard buttons(attack) and interval(3).
    ///
///  Constructs the default auto-mash preset with standard buttons(attack) and interval(3).
    fn default() -> Self {
        Self {
            buttons: ButtonSet::ATTACK,
            interval_frames: 3,
        }
    }
}

impl AutoMash {
    /// Constructs a `ScenarioRule` for the auto-mash preset based on the current buttons and interval.
    /// # Returns
    /// A `ScenarioRule` configured for the auto-mash preset.
    ///
///  Constructs a ScenarioRule for the auto-mash preset based on the current buttons and interval. # Returns A ScenarioRule configured for the auto-mash preset.
    pub fn rule(self) -> ScenarioRule {
        ScenarioRule::controller(
            Scenario::Clatter,
            // In this generic BUTTON example, interval 2 means press, release,
            // repeat. Live analog mash replaces this timing with every update
            // in RawAutoControls, so changing this interval won't speed it up.
            RuleTiming::pulse_every(self.interval_frames.max(2)),
            InputRecipe::buttons(self.buttons),
        )
    }
}

// Compiled only by cargo test on the host; these assertions are not live
// Switch logging or gameplay callbacks. Add regressions here for new behavior.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AssistConfig, AssistEngine, ControllerFrame, DecisionStatus, FighterKind,
        FighterObservation, InputPermissions, RunContext, ScenarioSample,
    };

    /// Regression test for `tools_obey_priority_release_gaps_and_independent_toggles`.
    ///
    ///
///  Checks that tools obey priority release gaps and independent toggles.
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
    ///
    ///
///  Checks that mash always leaves a release frame.
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
}
