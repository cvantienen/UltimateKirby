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

// See docs/DEVELOPER_GUIDE.md for the architecture, execution flow, and module map.
//
// Every module keeps its original crate-root path through the re-exports below,
// so `kirby_assist_core::native_input` and similar callers are unaffected.

pub mod config;
pub mod diagnostics;
/// Fighter reference facts, profiles, native state tables and fighter-specific policies.
pub mod fighters {
    pub mod auto_counter;
    pub mod fighter_data;
    pub mod fighter_kinds;
    pub mod fighter_profiles;
    pub mod fighter_tuning;
    pub mod game_and_watch;
    pub mod kirby;
    pub mod native_states;

    /// Read-only grounded Flame Choke release and contact timing.
    pub mod ganondorf {
        use crate::native_states::status;

        /// Victim status while held by grounded Flame Choke.
        pub const CATCHED_GANON: i32 = status::CATCHED_GANON;
        /// Shared victim release status at the end of Flame Choke.
        pub const CATCHED_AIR_END_GANON: i32 = status::CATCHED_AIR_END_GANON;
        /// Default frame for scheduling a tech after grounded Flame Choke.
        pub const DEFAULT_GANON_PASSIVE_TRIGGER_FRAME: u16 = 7;

        /// Classifies the current guard-lockout status.
        pub(crate) fn guard_lockout_status(status: i32) -> bool {
            matches!(status, CATCHED_GANON | CATCHED_AIR_END_GANON)
        }

        /// Selects the remaining tech window for grounded Flame Choke or normal tumble.
        pub(crate) fn passive_window(status: i32, normal: u16, ganon: u16) -> u16 {
            if status == CATCHED_GANON {
                ganon
            } else {
                normal
            }
        }

        /// The release status is shared by grounded and aerial Flame Choke. Only the
        /// grounded capture immediately preceding it identifies a techable release.
        pub fn grounded_ganon_release(status: i32, previous_status: i32) -> bool {
            status == CATCHED_AIR_END_GANON && previous_status == CATCHED_GANON
        }

        /// Grounded Flame Choke uses a capture status, not damage-fly or a falling ray.
        pub fn grounded_ganon_contact_in(
            victim_status: i32,
            captor_kind: i32,
            captor_status: i32,
            frame: f32,
            rate: f32,
        ) -> Option<f32> {
            if victim_status != CATCHED_GANON
                || captor_kind != crate::fighter_kinds::GANONDORF
                || captor_status != crate::native_states::fighter_status::GANON_SPECIAL_S_CATCH
                || !frame.is_finite()
                || !(0.0..=31.0).contains(&frame)
                || !rate.is_finite()
                || rate <= 0.0
            {
                return None;
            }
            let remaining = (31.0 - frame) / rate;
            (remaining.is_finite() && remaining <= 120.0).then_some(remaining)
        }
    }

    /// Native facing is ±1; anything else is an unreadable observation.
    pub(crate) fn valid_facing(facing: f32) -> bool {
        facing.is_finite() && facing.abs() >= 0.5
    }
}
pub mod live_input;
pub mod manual_controls;
/// Independent assistance policies and their decision helpers.
pub mod policies {
    pub mod assist;
    pub mod autopilot_guard;
    pub mod defense_response;
    pub mod grab_handoff;
    pub mod ledge_position;
    pub mod ledge_trap;
    pub mod ledge_trump;
    pub mod move_input;
    pub mod out_of_shield;
    pub mod posture;
    pub mod reaction_rules;
    pub mod shield_pressure;
    pub mod spot_dodge;
    pub mod spot_punish;
    pub mod tools;
}
pub mod runtime;
pub mod sequence_config;
pub mod settings_menu;
/// Contracts and survival responses shared by every policy.
pub mod shared {
    pub mod controller;
    pub mod native_input;
    pub mod observations;
    pub mod survival;
    pub mod timing;
}

pub use config::{button_bindings, option_read, settings};
pub use fighters::auto_counter;
pub use fighters::game_and_watch as projectile_reflect;
pub(crate) use fighters::ganondorf as flame_choke;
pub use fighters::{
    fighter_data, fighter_kinds, fighter_profiles, fighter_tuning, kirby, native_states,
};
pub use policies::spot_dodge::roll_read;
pub use policies::{
    assist, autopilot_guard, defense_response, grab_handoff, ledge_position, ledge_trap,
    ledge_trump, move_input, out_of_shield, posture, reaction_rules, shield_pressure, spot_dodge,
    spot_punish, tools,
};
pub use runtime::{runtime_toggle, session, training_test};
pub use sequence_config::sequence;
pub(crate) use shared::controller::up_b_receipt;
pub use shared::controller::{action_owner, input};
pub(crate) use shared::observations::projectile_geometry;
pub use shared::observations::{battle_objects, gameplay, observation, state};
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
