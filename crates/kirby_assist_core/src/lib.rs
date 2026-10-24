//! Platform-independent assistance policy for Kirby Assist.
//!
//! The types in this crate are deliberately *logical*, not reverse-engineered
//! Switch controller structs. The implemented Switch adapter in `src/live.rs`
//! converts requests using `native_input`; examples use a pretend adapter.
//!
//! `pub mod` exposes a module; `pub use` re-exports selected names so callers can
//! write `kirby_assist_core::AutoMash` instead of the full module path.
//!
//! Generic scenario tools use Scenario/ScenarioRule; the live ledge, slide and
//! combat tools have dedicated state machines. Both need live detection and
//! input translation, plus checks for transitions and cancellation.
//! This crate can be tested on a computer without Skyline or a connected Switch.

// Suggested learning order inside this crate:
// 1. state.rs / input.rs: the small values passed between components.
// 2. config.rs / assist.rs: select a rule and compute one logical proposal.
// 3. examples/walkthrough.rs: a complete, pretend frame loop on your computer.
// 4. live_input.rs: real mash/tech scheduling; manual_controls.rs: D-pad tools.
// 5. ledge_trap.rs: the larger state machine, backed by ledge_trap/tests.rs.
//    ledge_trap/types.rs defines its data; its response files contain recipes.
// See docs/CODE_MAP.md for module roles and docs/FIGHTER_CONFIGS.md for compiled fighter catalogs.
// `pub` is Rust's visibility keyword. Tests can exercise these public helpers
// without loading the Switch plugin or depending on a native game allocation.
// Common Rust syntax in these files: #[derive(...)] asks the compiler to
// implement traits. Copy permits copying a small value, Debug supports {:?}
// formatting, Default supplies initial values, and PartialEq supports == checks.
pub mod action_owner;
pub mod assist;
pub mod auto_di;
pub mod autopilot_guard;
pub mod button_modes;
pub mod combat_assist;
pub mod config;
pub mod diagnostics;
pub mod finisher;
pub mod ledge_menu;
pub mod opponent_data;
pub mod spot_dodge;
pub mod option_read;
pub mod whiff_punish;
// Compiled fighter registrations, named responses, move data.
pub mod fighter_profiles;
pub mod input;

pub mod live_input;
pub mod observation;
pub mod roll_read;
pub mod runtime;
pub mod sequence;
pub mod session;
pub mod slide;
pub mod state;
pub mod timing;
pub mod tools;
pub mod training_test;
pub use runtime::AssistRuntime;
pub use sequence::{
    ConfirmedSequence, ConfirmedStep, InputSequence, InputStep, SequenceObservation, SequenceState,
};
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
pub mod tech_input;
pub mod tech_prediction;

pub mod jab_grab;
pub mod jab_lock;
pub mod kirby_posture;
pub mod manual_controls;
pub mod move_input;

// Held-L ledge autopilot; the recovery 2-frame policy remains available to core callers.
pub mod ledge_position;
pub mod ledge_trap;
pub mod ledge_trump;
pub mod two_frame;
