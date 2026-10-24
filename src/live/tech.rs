//! Native tech eligibility and collision/capture timing observations.
//!
//! This detector reports a possible contact. The portable TechInput scheduler
//! separately decides whether to request/release shield and tracks mapper
//! acknowledgment. Prediction, button delivery and successful tech are distinct.
use crate::game_ffi as game;
use kirby_assist_core::tech_prediction::{
    first_contact_in, grounded_ganon_contact_in, tech_gate, TechGate, CATCHED_GANON,
};

// Diagnostic snapshot only: the string describes our detector's decision, not
// a complete explanation of all native tech restrictions or proof of success.
pub(super) struct TechAssessment {
    pub(super) status: i32,
    pub(super) situation: i32,
    pub(super) velocity: [f32; 2],
    pub(super) reason: &'static str,
    pub(super) contact_in: Option<f32>,
    pub(super) remaining_stop: u64,
    // A grounded command grab can freeze the victim's animation. Its linked
    // captor's native clock supplies progress without counting controller polls.
    pub(super) capture_clock: Option<(u64, f32, f32)>,
}

unsafe fn grounded_ganon_timing(
    reads: &super::frame::FrameReads,
    object: *mut game::Object,
) -> Option<(f32, u64, f32, f32)> {
    if !game::is_link(object, 0) {
        return None;
    }
    let id = u32::try_from(game::link_object_id(object, 0)).ok()?;
    let own_id = object.cast::<u8>().add(8).cast::<u32>().read();
    if id == own_id || id >> 28 != 0 || !game::is_active(id) {
        return None;
    }
    let captor = game::module_accessor(id);
    if captor.is_null() || captor.cast::<u8>().add(8).cast::<u32>().read() != id {
        return None;
    }
    let motion = reads.motion_kind(captor);
    if motion == 0 {
        return None;
    }
    let frame = reads.motion_frame(captor);
    let rate = reads.motion_rate(captor);
    let remaining = grounded_ganon_contact_in(
        reads.status_kind(object),
        game::fighter_kind(id),
        reads.status_kind(captor),
        frame,
        rate,
    )?;
    Some((remaining, motion, frame, rate))
}

// Start with native status/eligibility gates, then test swept stage rays. Keep
// the fixed body offsets in tech_prediction.rs separate so they can be tested
// on the host and tuned without scattering geometry through native FFI code.
pub(super) unsafe fn assess_tech(
    reads: &super::frame::FrameReads,
    object: *mut game::Object,
    horizon: f32,
    stopped: bool,
    input_delay: f32,
) -> TechAssessment {
    let status = reads.status_kind(object);
    let situation = reads.situation_kind(object);
    let disabled = (0x49..=0x4f).contains(&status) && game::is_flag(object, 0x21000016);
    let gate = tech_gate(status, situation, disabled, stopped);
    let mut assessment = TechAssessment {
        status,
        situation,
        velocity: [0.0; 2],
        reason: gate.label(),
        contact_in: None,
        remaining_stop: if stopped && game::is_damage_stop(object) {
            game::hitstop_remaining(object)
        } else {
            0
        },
        capture_clock: None,
    };
    if (0x67..=0x6b).contains(&status) {
        assessment.reason = "tech_status_observed";
        return assessment;
    }
    if status == CATCHED_GANON {
        assessment.reason = "ganon_ground_capture_timing_unavailable";
        if let Some((contact, motion, frame, rate)) = grounded_ganon_timing(reads, object) {
            assessment.contact_in = Some(contact);
            assessment.capture_clock = Some((motion, frame, rate));
            assessment.reason = "collision_predicted";
        }
        return assessment;
    }
    if (0xbf..=0xc1).contains(&status) {
        assessment.reason = "ganon_aerial_slam_untechable";
        return assessment;
    }
    // Read damage velocity for diagnostics even when the native flag blocks tech.
    if (0x47..=0x5b).contains(&status) {
        // ALL includes damage knockback as well as main/gravity velocity.
        // MAIN (1) alone can omit the force carrying Kirby toward the surface.
        assessment.velocity = [reads.speed_x(object, -1), reads.speed_y(object, -1)];
    }
    let late_hitstop = gate == TechGate::Hitstop
        && assessment.remaining_stop > 0
        && assessment.remaining_stop as f32 <= input_delay.clamp(0.0, 12.0) + 2.0;
    if gate != TechGate::Predict && !late_hitstop {
        return assessment;
    }
    let x = reads.pos_x(object);
    let y = reads.pos_y(object);
    let [vx, vy] = assessment.velocity;
    if ![x, y, vx, vy].iter().all(|value| value.is_finite()) {
        assessment.reason = "invalid_motion";
        return assessment;
    }
    assessment.reason = "no_collision_predicted";
    let horizon = (horizon + input_delay.clamp(0.0, 12.0)).clamp(1.0, 20.0);
    assessment.contact_in = first_contact_in(x, y, vx, vy, horizon, |probe| {
        let origin = game::Vector2 {
            x: probe.origin[0],
            y: probe.origin[1],
        };
        let path = game::Vector2 {
            x: probe.delta[0],
            y: probe.delta[1],
        };
        game::ray_check(object, &origin, &path, true) != 0
    })
    .map(|flight| flight + assessment.remaining_stop as f32);
    if assessment.contact_in.is_some() {
        assessment.reason = "collision_predicted";
    }
    assessment
}
