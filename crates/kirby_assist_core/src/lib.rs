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

// Source layout (see docs/CODE_MAP.md):
// - shared/: controller, observation, native-input, timing and survival contracts.
// - fighters/: reference data, profiles, native state tables, fighter-specific rules.
// - policies/: independent assistance policies (ledge, out-of-shield, reactions, ...).
// - config.rs, sequence_config.rs, settings_menu.rs, manual_controls.rs: user configuration.
// - runtime.rs, live_input.rs, diagnostics.rs: stateful orchestration and logging.
// examples/walkthrough.rs is a complete, pretend frame loop on your computer.
//
// Every module keeps its original crate-root path through the re-exports below,
// so `kirby_assist_core::native_input` and similar callers are unaffected.

pub mod config;
pub mod diagnostics;
pub mod fighters;
pub mod live_input;
pub mod manual_controls;
pub mod policies;
pub mod runtime;
pub mod sequence_config;
pub mod settings_menu;
pub mod shared;

pub use config::{button_bindings, option_read, settings};
pub use fighters::{
    fighter_data, fighter_kinds, fighter_profiles, fighter_tuning, kirby, native_states,
};
pub use fighters::game_and_watch as projectile_reflect;
pub(crate) use fighters::ganondorf as flame_choke;
pub use policies::{
    assist, autopilot_guard, defense_response, grab_handoff, ledge_position, ledge_trap,
    ledge_trump, move_input, out_of_shield, posture, reaction_rules, shield_pressure, spot_dodge,
    spot_punish, tools,
};
pub use policies::spot_dodge::roll_read;
pub use runtime::{runtime_toggle, session, training_test};
pub use sequence_config::sequence;
pub use shared::controller::{action_owner, input};
pub(crate) use shared::controller::up_b_receipt;
pub use shared::observations::{battle_objects, gameplay, observation, state};
pub(crate) use shared::observations::projectile_geometry;
pub use shared::survival::{auto_di, combat_assist, tech_input, tech_prediction};
pub use shared::{native_input, timing};

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
