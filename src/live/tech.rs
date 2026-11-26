//! Native collision-timed auto-tech eligibility.
//!
//! Guard is requested only for a bounded, imminent stage collision. Native
//! transition flags act as an eligibility check; they are not, by themselves,
//! a reason to press Guard.

use crate::game_ffi as game;
use kirby_assist_core::tech_prediction::{
    first_contact_with_body, grounded_ganon_contact_in, grounded_ganon_release,
    late_damage_stop_prediction, prediction_velocity, tech_gate_with_floor, Body, TechGate,
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
    /// First predicted floor, wall, or ceiling contact, including remaining hitstop.
    pub(super) contact_in: Option<f32>,
    /// Remaining native damage-stop frames.
    pub(super) remaining_stop: u64,
    /// Stable diagnostic label for the native gate that matched.
    #[cfg_attr(not(feature = "diagnostic-logging"), allow(dead_code))]
    pub(super) reason: &'static str,
}

/// Reads the fighter's consumed Guard and independent damage-reaction clock.
/// # Safety
/// `object` must be a live fighter with the pinned native control/work modules.
pub(super) unsafe fn input_feedback(object: *mut game::Object, status: i32) -> (bool, Option<f32>) {
    if !kirby_assist_core::tech_prediction::is_tech_status(status) {
        return (false, None);
    }
    // CONTROL_PAD_BUTTON_GUARD / GUARD_HOLD in the pinned skyline-smash constants.
    let guard = game::check_button_on(object, 0x3) || game::check_button_on(object, 0x10);
    let reaction = kirby_assist_core::tech_prediction::is_tumble_status(status).then(|| {
        game::work_float(
            object,
            kirby_assist_core::native_states::work_ids::FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_REACTION_FRAME,
        )
    });
    (guard, reaction)
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
    body: Body,
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

    // A collision forecast alone cannot make Guard safe during native air escape.
    if game::transition_enabled(
        object,
        kirby_assist_core::native_states::transition_terms::CONT_ESCAPE_AIR,
    ) {
        return TechAssessment {
            status,
            situation,
            contact_in: None,
            remaining_stop: 0,
            reason: "air_escape_enabled",
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
    if status == kirby_assist_core::native_states::status::CATCHED_GANON {
        let contact_in = if situation == kirby_assist_core::native_states::situation::GROUND
            && !stopped
            && game::is_link(object, 0)
        {
            u32::try_from(game::link_node_object_id(object, 0))
                .ok()
                .filter(|id| {
                    *id >> 28 == 0
                        && *id != game::battle_object_id(object)
                        && game::is_active(*id)
                })
                .and_then(|id| {
                    let captor = game::module_accessor(id);
                    if captor.is_null()
                        || game::battle_object_id(captor) != id
                        || reads.situation_kind(captor)
                            != kirby_assist_core::native_states::situation::GROUND
                        || reads.is_stop(captor)
                    {
                        return None;
                    }
                    grounded_ganon_contact_in(
                        status,
                        game::fighter_kind(id),
                        reads.status_kind(captor),
                        reads.motion_frame(captor),
                        reads.motion_rate(captor),
                    )
                })
        } else {
            None
        };
        return TechAssessment {
            status,
            situation,
            contact_in,
            remaining_stop,
            reason: if contact_in.is_some() {
                "grounded_ganon_capture"
            } else {
                "unverified_ganon_capture"
            },
        };
    }
    if status == kirby_assist_core::native_states::status::CATCHED_AIR_END_GANON
        && grounded_ganon_release(status, game::previous_status(object, 0))
    {
        let contact_in = (situation == kirby_assist_core::native_states::situation::GROUND
            && floor_enabled
            && !passive_disabled
            && !stopped)
            .then_some(0.0);
        return TechAssessment {
            status,
            situation,
            contact_in,
            remaining_stop,
            reason: "grounded_ganon_release",
        };
    }
    let native_contact = kirby_assist_core::tech_input::native_contact_due(
        status,
        situation,
        passive_disabled,
        floor_enabled,
        game::transition_enabled(
            object,
            kirby_assist_core::native_states::transition_terms::PASSIVE_WALL,
        ),
        game::transition_enabled(
            object,
            kirby_assist_core::native_states::transition_terms::PASSIVE_CEIL,
        ),
    );
    if native_contact {
        return TechAssessment {
            status,
            situation,
            contact_in: Some(0.0),
            remaining_stop,
            reason: "native_tech_contact",
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
        first_contact_with_body(x, y, vx, vy, horizon, 0.0, body, |probe| {
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
        contact_in: contact_in.map(|frames| frames + remaining_stop as f32),
        remaining_stop,
        reason,
    }
}
