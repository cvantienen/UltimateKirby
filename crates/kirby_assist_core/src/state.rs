//! Normalize game facts into small Rust values that policy can understand.
//! Scenario names describe observations, not game commands. Most variants are
//! extension points; the current live adapter publishes Clatter and Tech only.
//!
//! Traits at the bottom define a contract for the generic runtime/examples.
//! They do not automatically read game memory or install native callbacks.
use crate::input::ControllerFrame;

/// The targeted fighter. A platform adapter must prove this from game state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FighterKind {
    Kirby,
    Other,
    Unknown,
}

/// A normalized situation that can activate an assistance rule.
///
/// The priority order follows the Training Modpack's public mash selection
/// flow: specific defensive situations win over broad grounded/airborne ones.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum Scenario {
    Tech,
    Clatter,
    Tumble,
    Hitstun,
    Parry,
    Footstool,
    LedgeTrump,
    Landing,
    Ledge,
    Grounded,
    Airborne,
    // Sentinel for an unused observation/rule slot; it is not an active tool.
    #[default]
    None,
}

impl Scenario {
    /// Lower values have higher precedence when multiple samples are active.
    pub const fn priority(self) -> u8 {
        match self {
            Self::Tech => 0,
            Self::Clatter => 1,
            Self::Tumble => 2,
            Self::Hitstun => 3,
            Self::Parry => 4,
            Self::Footstool => 5,
            Self::LedgeTrump => 6,
            Self::Landing => 7,
            Self::Ledge => 8,
            Self::Grounded => 9,
            Self::Airborne => 10,
            Self::None => u8::MAX,
        }
    }
}

/// A situation supplied by a live or pretend game-state adapter.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
// Age belongs to a particular occurrence, not match time: active -> inactive
// -> active restarts at zero. The live ScenarioCounter implements that contract.
pub struct ScenarioSample {
    pub scenario: Scenario,
    /// Frame zero is the first frame in this scenario.
    pub frames_active: u32,
}

impl ScenarioSample {
    pub const EMPTY: Self = Self {
        scenario: Scenario::None,
        frames_active: 0,
    };

    pub const fn new(scenario: Scenario, frames_active: u32) -> Self {
        Self {
            scenario,
            frames_active,
        }
    }
}

/// A small snapshot with room for overlapping conditions (for example, tumble
/// and hitstun). The engine chooses the highest-priority active scenario.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FighterObservation {
    pub active: [ScenarioSample; Self::MAX_SAMPLES],
}

impl FighterObservation {
    pub const MAX_SAMPLES: usize = 3;

    pub const EMPTY: Self = Self {
        active: [ScenarioSample::EMPTY; Self::MAX_SAMPLES],
    };

    pub const fn one(scenario: Scenario, frames_active: u32) -> Self {
        Self {
            active: [
                ScenarioSample::new(scenario, frames_active),
                ScenarioSample::EMPTY,
                ScenarioSample::EMPTY,
            ],
        }
    }

    // Return the sample WITH its age; choosing only its name would lose the
    // timing information needed by delayed/pulsing rules.
    pub fn highest_priority(self) -> Option<ScenarioSample> {
        let mut best = ScenarioSample::EMPTY;
        for sample in self.active {
            if sample.scenario != Scenario::None
                && (best.scenario == Scenario::None
                    || sample.scenario.priority() < best.scenario.priority())
            {
                best = sample;
            }
        }
        (best.scenario != Scenario::None).then_some(best)
    }
}

impl Default for FighterObservation {
    fn default() -> Self {
        Self::EMPTY
    }
}

/// Facts that must be established before the engine may propose an action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RunContext {
    pub is_locally_human_controlled: bool,
    pub fighter: FighterKind,
}

/// Boundary to implement only after a current, documented state-reading route
/// has been verified. No game hook or memory layout is assumed here.
// A trait describes methods a type must supply. It is not a concrete adapter;
// examples/walkthrough.rs implements these methods using pretend data. &self
// permits reads; &mut self on write_merged_input permits changing adapter state.
pub trait GameObservationSource {
    fn observe_fighter(&self) -> Option<FighterObservation>;
}

/// Boundary to implement only after a current, documented input route has been
/// verified. The core requires a read-then-write flow so human input can be
/// preserved during arbitration.
pub trait InputBridge {
    fn read_human_input(&self) -> Option<ControllerFrame>;
    fn write_merged_input(&mut self, frame: ControllerFrame);
}

/// Combined adapter used by the frame runner. Return `None` when no valid
/// fighter/controller is available (for example during loading or in menus).
pub trait GameAdapter: GameObservationSource + InputBridge {
    fn read_context(&self) -> Option<RunContext>;
}

// Compiled only by cargo test on the host; these assertions are not live
// Switch logging or gameplay callbacks. Add regressions here for new behavior.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specific_scenario_beats_broad_scenario() {
        let observation = FighterObservation {
            active: [
                ScenarioSample::new(Scenario::Grounded, 12),
                ScenarioSample::new(Scenario::Tumble, 3),
                ScenarioSample::EMPTY,
            ],
        };

        assert_eq!(
            observation.highest_priority(),
            Some(ScenarioSample::new(Scenario::Tumble, 3))
        );
    }
}
