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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AutoTech {
    /// Runtime timing value used by AutoTech..
    pub timing: RuleTiming,
}

impl Default for AutoTech {
    /// Returns the default value for this type.
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AutoMash {
    /// Runtime buttons value used by AutoMash..
    pub buttons: ButtonSet,
    /// Runtime interval frames value used by AutoMash..
    pub interval_frames: u16,
}

impl Default for AutoMash {
    /// Constructs the default auto-mash preset with standard buttons(attack) and interval(3).
    fn default() -> Self {
        Self {
            buttons: ButtonSet::ATTACK,
            interval_frames: 3,
        }
    }
}

impl AutoMash {
    /// Constructs a `ScenarioRule` for the auto-mash preset based on the current buttons and interval.
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
#[path = "../../tests/unit/policy_tools.rs"]
mod tests;
