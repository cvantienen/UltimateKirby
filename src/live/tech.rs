//! Native collision-timed auto-tech eligibility.
//!
//! Guard is requested only for a bounded, imminent stage collision. Native
//! transition flags act as an eligibility check; they are not, by themselves,
//! a reason to press Guard.

use crate::game_ffi as game;
use kirby_assist_core::tech_prediction::{
    first_contact_in, late_damage_stop_prediction, prediction_velocity, tech_gate_with_floor,
    TechGate,
};

/// One mapper/controller frame accounts for the live callback and overlay
/// before the game consumes the Guard edge.
pub(super) const TECH_INPUT_DELAY_FRAMES: f32 = 1.0;

/// Native status and transition result copied into the observation pipeline.
#[derive(Clone, Copy)]
pub(super) struct TechAssessment {
    /// Native fighter status at assessment time.
    pub(super) status: i32,
    /// Native ground/air situation at assessment time.
    pub(super) situation: i32,
    /// First predicted floor, wall, or ceiling contact in frames.
    pub(super) contact_in: Option<f32>,
    /// Remaining native damage-stop frames.
    pub(super) remaining_stop: u64,
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
    horizon: f32,
    stopped: bool,
) -> TechAssessment {
    let status = reads.status_kind(object);
    let situation = reads.situation_kind(object);
    if !enabled {
        return TechAssessment {
            status,
            situation,
            contact_in: None,
            remaining_stop: 0,
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
    let remaining_stop = if stopped && game::is_damage_stop(object) {
        game::hitstop_remaining(object)
    } else {
        0
    };
    if (kirby_assist_core::native_states::status::PASSIVE
        ..=kirby_assist_core::native_states::status::PASSIVE_CEIL)
        .contains(&status)
    {
        return TechAssessment {
            status,
            situation,
            contact_in: None,
            remaining_stop,
            reason: "tech_status_observed",
        };
    }
    let gate = tech_gate_with_floor(status, situation, passive_disabled, stopped, floor_enabled);
    let late_hitstop = gate == TechGate::Hitstop
        && late_damage_stop_prediction(remaining_stop, TECH_INPUT_DELAY_FRAMES);
    if !matches!(gate, TechGate::Predict) && !late_hitstop {
        return TechAssessment {
            status,
            situation,
            contact_in: None,
            remaining_stop,
            reason: gate.label(),
        };
    }
    let x = reads.pos_x(object);
    let y = reads.pos_y(object);
    let current_velocity = [reads.speed_x(object), reads.speed_y(object)];
    let reserved_velocity = (stopped
        && remaining_stop > 0
        && kirby_assist_core::tech_prediction::is_tumble_status(status))
    .then(|| reads.damage_launch(object));
    let [vx, vy] = prediction_velocity(current_velocity, reserved_velocity);
    let horizon = horizon.clamp(1.0, 20.0);
    let contact_in = if [x, y, vx, vy].iter().all(|v| v.is_finite()) {
        first_contact_in(x, y, vx, vy, horizon, |probe| {
            let origin = game::Vector2 {
                x: probe.origin[0],
                y: probe.origin[1],
            };
            let delta = game::Vector2 {
                x: probe.delta[0],
                y: probe.delta[1],
            };
            game::ray_check(object, &origin, &delta, probe.passable) != 0
        })
    } else {
        None
    };
    let reason = if contact_in.is_some() {
        "collision_predicted"
    } else {
        gate.label()
    };
    TechAssessment {
        status,
        situation,
        contact_in,
        remaining_stop,
        reason,
    }
}
