//! Platform-independent assistance policy for Kirby Assist.
//!
//! The types in this crate are deliberately *logical*, not reverse-engineered
//! Switch controller structs. The implemented Switch adapter in `src/live.rs`
//! converts requests using `native_input`; examples use a pretend adapter.
//!
//! `pub mod` exposes a module; `pub use` re-exports selected names so callers can
//! write `kirby_assist_core::AutoMash` instead of the full module path.
//!
//! Generic scenario tools use Scenario/ScenarioRule; the live ledge and
//! combat tools have dedicated state machines. Both need live detection and
//! input translation, plus checks for transitions and cancellation.
//! This crate can be tested on a computer without Skyline or a connected Switch.

// Suggested learning order:
// 1. shared/observations/ and shared/controller/: values passed between components.
// 2. config.rs / assist.rs: choose a rule and compute one logical proposal.
// 3. examples/walkthrough.rs: a complete, pretend frame loop on your computer.
// 4. live_input.rs: mash/tech scheduling; manual_controls.rs: D-pad tools.
// 5. ledge_trap.rs: the larger state machine, backed by ledge_trap/tests.rs.
//    ledge_trap.rs contains its range checks and response routing.
//    sequence_config/parser.rs loads and validates the startup recipe file.
// See docs/CODE_MAP.md for module responsibilities and docs/FIGHTER_CONFIGS.md
// for compiled fighter catalogs.
//
// The declarations below are grouped by responsibility to make this crate
// easier to navigate. They remain public at their existing paths so this
// organization does not break callers that already use the crate.
//
// `pub` is Rust's visibility keyword. Tests can exercise these helpers without
// loading the Switch plugin or depending on a native game allocation. Common
// Rust syntax: #[derive(...)] asks the compiler to implement traits. Copy
// permits copying a small value, Debug supports {:?} formatting, Default
// supplies initial values, and PartialEq supports == checks.

// Shared controller and observation contracts passed between policies.
#[path = "shared/controller/action_owner.rs"]
pub mod action_owner;
#[path = "shared/observations/battle_objects.rs"]
pub mod battle_objects;
pub mod fighter_data;
#[path = "shared/observations/gameplay.rs"]
pub mod gameplay;
#[path = "shared/controller/input.rs"]
pub mod input;
#[path = "shared/controller/native_input.rs"]
pub mod native_input;
pub mod native_states;
#[path = "shared/observations/observation.rs"]
pub mod observation;
#[path = "shared/observations/projectile_geometry.rs"]
mod projectile_geometry;
#[path = "shared/observations/state.rs"]
pub mod state;
pub mod timing;

// Fighter-specific reference facts, move data, and startup catalogs.
pub mod fighter_kinds;
pub mod fighter_profiles;
pub mod fighter_tuning;
#[path = "fighters/tuning_defaults.rs"]
mod fighter_tuning_defaults;
#[path = "fighters/ganondorf/flame_choke.rs"]
mod flame_choke;
#[path = "fighters/kirby/mod.rs"]
pub mod kirby;
#[path = "shared/controller/up_b_receipt.rs"]
mod up_b_receipt;

// Independent assistance policies and their decision helpers.
pub mod assist;
pub mod autopilot_guard;
pub mod defense_response;
pub mod grab_handoff;
pub mod jab_lock;
pub mod move_input;
pub mod out_of_shield;
pub mod posture;
#[path = "fighters/game_and_watch/projectile_reflect.rs"]
pub mod projectile_reflect;
pub mod reaction_rules;
pub mod roll_read;
pub mod shield_pressure;
pub mod spot_dodge;
pub mod spot_punish;

// Shared survival responses, independent of fighter-specific follow-ups.
#[path = "shared/survival/auto_di.rs"]
pub mod auto_di;
#[path = "shared/survival/combat_assist.rs"]
pub mod combat_assist;
#[path = "shared/survival/tech_input.rs"]
pub mod tech_input;
#[path = "shared/survival/tech_prediction.rs"]
pub mod tech_prediction;

// User configuration, menu choices, and input bindings.
pub mod button_bindings;
pub mod config;
pub mod manual_controls;
pub mod option_read;
pub mod sequence_config;
pub mod settings;
pub mod settings_menu;

// Stateful orchestration, diagnostics, and reusable examples/tools.
pub mod diagnostics;
pub mod live_input;
pub mod runtime;
pub mod runtime_toggle;
pub mod sequence;
pub mod session;
pub mod tools;
pub mod training_test;

// Related ledge behaviors share observations, but keep separate policies.
pub mod ledge_position;
pub mod ledge_trap;
pub mod ledge_trump;

// Convenient root-level re-exports; module-qualified paths remain supported.
pub use assist::{AssistEngine, DecisionStatus, FrameDecision};
pub use config::{
    AssistConfig, AssistScope, RecordingSlot, RuleTiming, ScenarioAction, ScenarioRule,
    MAX_SCENARIO_RULES,
};
pub use input::{
    arbitrate, ArbitrationResult, ButtonSet, ControllerFrame, InputPermissions, InputRecipe,
    LeftStickIntent, StickPosition,
};
pub use runtime::AssistRuntime;
pub use sequence::{
    ConfirmedSequence, ConfirmedStep, InputSequence, InputStep, SequenceObservation, SequenceState,
};
pub use state::{
    FighterKind, FighterObservation, GameAdapter, GameObservationSource, InputBridge, RunContext,
    Scenario, ScenarioSample,
};
pub use tools::{AutoMash, AutoTech};
