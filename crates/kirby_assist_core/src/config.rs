//! Declarative rules: what situation activates a tool, when it fires, and what
//! logical input it requests. These types neither detect game state nor install
//! hooks. Shared text settings are parsed in live_input/config.rs; fighter
//! response functions are compiled Rust registrations.
//!
//! Extending the Scenario enum also requires updating its priority and this
//! fixed rule capacity; adding a new field alone does not wire a new live tool.
use crate::{
    input::InputRecipe,
    state::{FighterKind, RunContext, Scenario},
};

/// Maximum number of scenarios that can be configured without a heap allocator.
pub const MAX_SCENARIO_RULES: usize = 11;

/// Top-level configuration. Rules are disabled by default.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssistConfig {
    pub enabled: bool,
    pub scope: AssistScope,
    // Fixed-size storage avoids allocating a vector whenever policy is evaluated.
    pub rules: [ScenarioRule; MAX_SCENARIO_RULES],
}

impl Default for AssistConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            scope: AssistScope::default(),
            rules: [ScenarioRule::DISABLED; MAX_SCENARIO_RULES],
        }
    }
}

impl AssistConfig {
    /// Replaces the matching rule, or uses the first empty slot.
    ///
    /// Returns `false` for `Scenario::None` or when no empty slot remains.
    /// Disabled rules retain their settings until explicitly removed.
    pub fn set_rule(&mut self, rule: ScenarioRule) -> bool {
        if rule.scenario == Scenario::None {
            return false;
        }

        // iter_mut() visits mutable references. find() returns the first match
        // as Option<&mut ScenarioRule>; `*slot = rule` replaces that array entry,
        // not the entire config. Search existing entries first to avoid duplicates.
        if let Some(slot) = self
            .rules
            .iter_mut()
            .find(|current| current.scenario == rule.scenario)
        {
            *slot = rule;
            return true;
        }

        if let Some(slot) = self
            .rules
            .iter_mut()
            .find(|current| current.scenario == Scenario::None)
        {
            *slot = rule;
            return true;
        }

        false
    }

    /// Toggle a tool without losing its recipe or timing.
    pub fn set_rule_enabled(&mut self, scenario: Scenario, enabled: bool) -> bool {
        if scenario == Scenario::None {
            return false;
        }
        if let Some(rule) = self.rules.iter_mut().find(|rule| rule.scenario == scenario) {
            rule.enabled = enabled;
            return true;
        }
        false
    }

    /// Remove a tool and free its configuration slot.
    pub fn remove_rule(&mut self, scenario: Scenario) -> bool {
        if scenario == Scenario::None {
            return false;
        }
        if let Some(rule) = self.rules.iter_mut().find(|rule| rule.scenario == scenario) {
            *rule = ScenarioRule::DISABLED;
            return true;
        }
        false
    }

    // Disabled entries are intentionally ignored, but retain their settings so
    // toggling a tool back on does not discard its timing/custom recipe.
    pub fn rule_for(&self, scenario: Scenario) -> Option<ScenarioRule> {
        self.rules
            .iter()
            .copied()
            .find(|rule| rule.enabled && rule.scenario == scenario)
    }
}

/// Select which fighter/controller may receive assistance, in any game mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssistScope {
    pub require_locally_human_controlled: bool,
    pub require_kirby: bool,
}

impl AssistScope {
    // `!flag || condition` means "only require condition when flag is true".
    // This scope is independent of game mode; the live adapter separately
    // validates its supported native controller/fighter before using the engine.
    pub fn allows(self, context: RunContext) -> bool {
        (!self.require_locally_human_controlled || context.is_locally_human_controlled)
            && (!self.require_kirby || context.fighter == FighterKind::Kirby)
    }
}

impl Default for AssistScope {
    fn default() -> Self {
        Self {
            require_locally_human_controlled: true,
            require_kirby: false,
        }
    }
}

/// The action to take after a rule's timing condition has been met.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScenarioAction {
    /// One logical controller frame, merged with the human player's frame.
    ControllerRecipe(InputRecipe),
    /// A future feature for replaying a saved multi-frame input sequence.
    /// It is deliberately non-functional until a playback backend is written.
    PlaybackSlot(RecordingSlot),
}

/// Slot names reserve an understandable API for a future input recorder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordingSlot {
    One,
    Two,
    Three,
    Four,
    Five,
}

/// How a scenario's active-frame counter selects controller frames.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuleTiming {
    /// Wait this many scenario frames before the first action frame.
    pub start_after_frames: u16,
    /// Number of action frames to emit each time the rule activates.
    pub active_for_frames: u16,
    /// Zero means "run once". Any other value repeats from the first frame.
    pub repeat_every_frames: u16,
}

impl RuleTiming {
    pub const IMMEDIATELY_ONCE: Self = Self {
        start_after_frames: 0,
        active_for_frames: 1,
        repeat_every_frames: 0,
    };

    pub const fn after(start_after_frames: u16) -> Self {
        Self {
            start_after_frames,
            ..Self::IMMEDIATELY_ONCE
        }
    }

    pub const fn pulse_every(repeat_every_frames: u16) -> Self {
        Self {
            repeat_every_frames,
            ..Self::IMMEDIATELY_ONCE
        }
    }

    // Frame zero belongs to the scenario. After the initial delay, modulo (%)
    // repeats a pulse pattern: interval 3 / duration 1 means on, off, off, repeat.
    // The live caller supplies observed input updates, not a universal game clock.
    pub fn is_active_on(self, frames_active: u32) -> bool {
        if frames_active < self.start_after_frames as u32 {
            return false;
        }

        // Example: delay 2, duration 1, repeat 3 fires at ages 2, 5, 8, ...
        // .max(1) prevents a zero-duration action or a modulo-by-zero panic.
        // These generic recipe timings do not tune the live analog mash cycle.
        let duration = self.active_for_frames.max(1) as u32;
        let elapsed = frames_active - self.start_after_frames as u32;
        if self.repeat_every_frames == 0 {
            elapsed < duration
        } else {
            elapsed % (self.repeat_every_frames.max(1) as u32) < duration
        }
    }
}

/// A readable declaration: "in this scenario, propose this action at this time."
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScenarioRule {
    pub enabled: bool,
    pub scenario: Scenario,
    pub timing: RuleTiming,
    pub action: ScenarioAction,
}

impl ScenarioRule {
    pub const DISABLED: Self = Self {
        enabled: false,
        scenario: Scenario::None,
        timing: RuleTiming::IMMEDIATELY_ONCE,
        action: ScenarioAction::ControllerRecipe(InputRecipe::NONE),
    };

    pub const fn controller(scenario: Scenario, timing: RuleTiming, recipe: InputRecipe) -> Self {
        Self {
            enabled: true,
            scenario,
            timing,
            action: ScenarioAction::ControllerRecipe(recipe),
        }
    }

    pub const fn playback(scenario: Scenario, timing: RuleTiming, slot: RecordingSlot) -> Self {
        Self {
            enabled: true,
            scenario,
            timing,
            action: ScenarioAction::PlaybackSlot(slot),
        }
    }
}

// Compiled only by cargo test on the host; these assertions are not live
// Switch logging or gameplay callbacks. Add regressions here for new behavior.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::ButtonSet;

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
            Scenario::LedgeTrump,
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
}
