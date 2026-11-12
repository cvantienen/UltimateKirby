//! Native auto-tech eligibility.
//!
//! This module deliberately does not forecast physics or collision timing. It
//! reads the game's passive-transition permissions and reports whether the live
//! input layer should pulse Guard on this controller frame.

use crate::game_ffi as game;
use kirby_assist_core::tech_input::{native_tech_state, NativeTechState};

/// Native status and transition result copied into the observation pipeline.
#[derive(Clone, Copy)]
pub(super) struct TechAssessment {
    /// Native fighter status at assessment time.
    pub(super) status: i32,
    /// Native ground/air situation at assessment time.
    pub(super) situation: i32,
    /// Authoritative native tech state.
    pub(super) state: NativeTechState,
    /// Stable diagnostic label for the native gate that matched.
    #[cfg_attr(not(feature = "diagnostic-logging"), allow(dead_code))]
    pub(super) reason: &'static str,
}

/// Reads only native status, work flags, and passive transition terms.
///
/// # Safety
/// `object` must remain a valid fighter module accessor for these reads.
pub(super) unsafe fn assess_tech(
    reads: &super::frame::FrameReads,
    object: *mut game::Object,
    enabled: bool,
) -> TechAssessment {
    let status = reads.status_kind(object);
    let situation = reads.situation_kind(object);
    if !enabled {
        return TechAssessment {
            status,
            situation,
            state: NativeTechState::Inactive,
            reason: "auto_tech_disabled",
        };
    }

    let passive_disabled = game::is_flag(
        object,
        kirby_assist_core::native_states::work_ids::FIGHTER_STATUS_DAMAGE_FLAG_FLY_DISABLE_PASSIVE,
    );
    let floor_enabled = game::transition_enabled(
        object,
        kirby_assist_core::native_states::transition_terms::PASSIVE,
    ) || game::transition_enabled(
        object,
        kirby_assist_core::native_states::transition_terms::PASSIVE_FB,
    );
    let wall_enabled = game::transition_enabled(
        object,
        kirby_assist_core::native_states::transition_terms::PASSIVE_WALL,
    );
    let ceiling_enabled = game::transition_enabled(
        object,
        kirby_assist_core::native_states::transition_terms::PASSIVE_CEIL,
    );
    let state = native_tech_state(
        status,
        situation,
        passive_disabled,
        floor_enabled,
        wall_enabled,
        ceiling_enabled,
    );
    let reason = match state {
        NativeTechState::Succeeded => "tech_status_observed",
        NativeTechState::Opportunity
            if status == kirby_assist_core::native_states::status::STOP_WALL =>
        {
            "native_wall_tech_available"
        }
        NativeTechState::Opportunity
            if status == kirby_assist_core::native_states::status::STOP_CEIL =>
        {
            "native_ceiling_tech_available"
        }
        NativeTechState::Opportunity if floor_enabled => "native_floor_tech_available",
        NativeTechState::Opportunity if wall_enabled => "native_wall_tech_available",
        NativeTechState::Opportunity => "native_ceiling_tech_available",
        NativeTechState::Inactive if passive_disabled => "native_passive_disabled",
        NativeTechState::Inactive => "native_tech_unavailable",
    };
    TechAssessment {
        status,
        situation,
        state,
        reason,
    }
}
