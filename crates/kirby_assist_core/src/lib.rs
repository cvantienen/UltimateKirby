//! Platform-independent assistance policy for Kirby Assist.
//!
//! The types in this crate are deliberately *logical*, not reverse-engineered
//! Switch controller structs. The implemented Switch adapter in `src/live.rs`
//! converts requests using `native_input`; examples use a pretend adapter.
//!
//! `pub mod` exposes a module; `pub use` re-exports selected names so callers can
//! write `kirby_assist_core::AutoMash` instead of the full module path.
//!
//! To add a tool: define/choose a Scenario, supply a ScenarioRule, implement live
//! detection and output translation, then test transitions and cancellation.
//! This crate can be tested on a computer without Skyline or a connected Switch.

// Suggested learning order inside this crate:
// 1. state.rs / input.rs: the small values passed between components.
// 2. config.rs / assist.rs: select a rule and compute one logical proposal.
// 3. examples/walkthrough.rs: a complete, pretend frame loop on your computer.
// 4. live_input.rs: real mash/tech scheduling; manual_controls.rs: D-pad tools.
// 5. ledge_trap.rs: the larger state machine, backed by ledge_trap/tests.rs.
// `pub` is Rust's visibility keyword. Tests can exercise these public helpers
// without loading the Switch plugin or depending on a native game allocation.
// Common Rust syntax in these files: #[derive(...)] asks the compiler to
// implement traits. Copy permits copying a small value, Debug supports {:?}
// formatting, Default supplies initial values, and PartialEq supports == checks.
pub mod assist;
pub mod config;
pub mod input;
pub mod live_input;
pub mod runtime;
pub mod sequence;
pub mod state;
pub mod tools;
pub use runtime::AssistRuntime;
pub use sequence::{InputSequence, InputStep};
pub use tools::{AutoMash, AutoTech};

pub use assist::{AssistEngine, DecisionStatus, FrameDecision};
pub use config::{
    AssistConfig, AssistScope, RecordingSlot, RuleTiming, ScenarioAction, ScenarioRule,
    MAX_SCENARIO_RULES,
};
pub use input::{
    arbitrate, ArbitrationResult, ButtonSet, ControllerFrame, InputPermissions, InputRecipe,
    LeftStickIntent, StickPosition,
};
pub use state::{
    FighterKind, FighterObservation, GameAdapter, GameObservationSource, InputBridge, RunContext,
    Scenario, ScenarioSample,
};

pub mod native_input;
pub mod tech_prediction;

pub mod manual_controls;

// Toggle-L3 opponent-state policy; the native adapter supplies observations.
pub mod ledge_position;
pub mod ledge_trap;
pub mod ledge_trump;
