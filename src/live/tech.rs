//! Native tech eligibility and collision/capture timing observations.
//!
//! This detector reports a possible contact. The portable TechInput scheduler
//! separately decides whether to request/release shield and tracks mapper
//! acknowledgment. Prediction, button delivery and successful tech are distinct.
use crate::game_ffi as game;
use kirby_assist_core::tech_prediction::{
    first_contact_swept, grounded_ganon_contact_in, grounded_ganon_release,
    late_damage_stop_prediction, tech_gate_with_floor, tech_press_allowed, FloorMotionHistory,
    FloorMotionSample, TechGate, CATCHED_AIR_END_GANON, CATCHED_GANON,
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
    pub(super) prepare_guard: bool,
    pub(super) reaction_remaining: Option<f32>,
    // A grounded command grab can freeze the victim's animation. Its linked
    // captor's native clock supplies progress without counting controller polls.
    pub(super) capture_clock: Option<(u64, f32, f32)>,
}

/// Pinned common.prc integer parameters, read without changing native timing.
pub(super) unsafe fn native_timing(object: *mut game::Object, fallback: u16) -> (u16, u16, u16) {
    let bounded = |value: i32, fallback: u16| {
        u16::try_from(value)
            .ok()
            .filter(|v| (2..=120).contains(v))
            .unwrap_or(fallback)
    };
    (
        bounded(game::param_int(object, 0x6e5ec7051, 0x14459066dd), fallback), // no_rapid_frame_value
        bounded(game::param_int(object, 0x6e5ec7051, 0x15ba3cdb02), 12), // passive_trigger_frame
        bounded(game::param_int(object, 0x6e5ec7051, 0x2535742c20), 7),
    ) // ganon_special_s_passive_trigger_frame
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
    if id == own_id
        || id >> 28 != kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
        || !game::is_active(id)
    {
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
        reads.fighter_kind(captor),
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
    enabled: bool,
    floor_motion: &mut FloorMotionHistory,
    body: kirby_assist_core::tech_prediction::Body,
) -> TechAssessment {
    let status = reads.status_kind(object);
    let situation = reads.situation_kind(object);
    if !enabled {
        *floor_motion = FloorMotionHistory::default();
        return TechAssessment {
            status,
            situation,
            velocity: [0.0; 2],
            reason: "auto_tech_disabled",
            contact_in: None,
            remaining_stop: 0,
            prepare_guard: false,
            reaction_remaining: None,
            capture_clock: None,
        };
    }
    // A zero countdown can still be the final native victim-stop observation.
    // Global stop with the same zero value must never manufacture eligibility.
    let victim_stop = stopped && game::is_damage_stop(object);
    let disabled = (kirby_assist_core::native_states::status::DAMAGE_FLY
        ..=kirby_assist_core::native_states::status::DAMAGE_FALL)
        .contains(&status)
        && game::is_flag(object, kirby_assist_core::native_states::work_ids::FIGHTER_STATUS_DAMAGE_FLAG_FLY_DISABLE_PASSIVE);
    // PASSIVE / PASSIVE_FB: read-only native floor-tech eligibility. A grounded
    // tumble contact must not be discarded solely because situation changed.
    // Require an actual airborne->ground transition, not a grounded launch
    // whose floor-tech term is enabled before the fighter leaves the stage.
    let floor_tech = situation == 0
        && (kirby_assist_core::native_states::status::DAMAGE_FLY
            ..=kirby_assist_core::native_states::status::DAMAGE_FALL)
            .contains(&status)
        && !disabled
        && (game::transition_enabled(
            object,
            kirby_assist_core::native_states::transition_terms::PASSIVE,
        ) || game::transition_enabled(
            object,
            kirby_assist_core::native_states::transition_terms::PASSIVE_FB,
        ))
        && game::previous_situation_kind(object) == 2;
    let gate = tech_gate_with_floor(status, situation, disabled, stopped, floor_tech);
    if gate != TechGate::Predict {
        *floor_motion = FloorMotionHistory::default();
    }
    let mut assessment = TechAssessment {
        status,
        situation,
        velocity: [0.0; 2],
        reason: gate.label(),
        contact_in: None,
        remaining_stop: if victim_stop {
            game::hitstop_remaining(object)
        } else {
            0
        },
        capture_clock: None,
        prepare_guard: false,
        // FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_REACTION_FRAME (pinned lua_const).
        reaction_remaining: if (kirby_assist_core::native_states::status::DAMAGE_FLY
            ..=kirby_assist_core::native_states::status::DAMAGE_FALL)
            .contains(&status)
        {
            let remaining = game::work_float(object, 0x13);
            (remaining.is_finite() && remaining > 0. && remaining <= 1200.).then_some(remaining)
        } else {
            None
        },
    };
    // The native STOP_WALL / STOP_CEIL statuses still admit passive input at
    // collision. Damage-fly-only gates used to discard this final opportunity.
    // Native transition terms, not the status name alone, authorize the input.
    if situation == 2
        && matches!(
            status,
            kirby_assist_core::native_states::status::STOP_WALL
                | kirby_assist_core::native_states::status::STOP_CEIL
        )
    {
        let term = if status == kirby_assist_core::native_states::status::STOP_WALL {
            kirby_assist_core::native_states::transition_terms::PASSIVE_WALL
        } else {
            kirby_assist_core::native_states::transition_terms::PASSIVE_CEIL
        };
        if game::transition_enabled(object, term) && (!stopped || victim_stop) {
            assessment.contact_in = Some(assessment.remaining_stop as f32);
            assessment.prepare_guard = true;
            assessment.reason = "collision_predicted";
        }
        return assessment;
    }
    if (kirby_assist_core::native_states::status::PASSIVE
        ..=kirby_assist_core::native_states::status::PASSIVE_CEIL)
        .contains(&status)
    {
        assessment.reason = "tech_status_observed";
        return assessment;
    }
    if floor_tech && (!stopped || victim_stop) {
        let vy = reads.speed_y(object, -1);
        let velocity = if victim_stop {
            kirby_assist_core::tech_prediction::prediction_velocity(
                [0., vy],
                Some(reads.damage_launch(object)),
            )
        } else {
            [0., vy]
        };
        // Previous situation can still be AIR when a newly landed fighter is
        // hit upward. That is a new launch, not an impact to buffer a tech for.
        // Reflect-down is the native exception after a real floor rebound.
        if status == kirby_assist_core::native_states::status::DAMAGE_FLY_REFLECT_D
            || velocity[1].is_finite() && velocity[1] <= 0.0001
        {
            assessment.velocity = velocity;
            assessment.contact_in = Some(assessment.remaining_stop as f32);
            assessment.prepare_guard = true;
            assessment.reason = "collision_predicted";
            return assessment;
        }
    }
    if status == CATCHED_GANON {
        assessment.reason = "ganon_ground_capture_timing_unavailable";
        if let Some((contact, motion, frame, rate)) = grounded_ganon_timing(reads, object) {
            assessment.contact_in = Some(contact);
            assessment.capture_clock = Some((motion, frame, rate));
            assessment.prepare_guard = true;
            assessment.reason = "collision_predicted";
        }
        return assessment;
    }
    if status == CATCHED_AIR_END_GANON
        && grounded_ganon_release(status, game::previous_status(object, 0))
    {
        // Both types of Flame Choke use this status. Never infer its origin
        // from proximity or situation; the native predecessor must be grounded.
        // Contact is now immediate after any known damage stop. No extra rays.
        assessment.reason = "ganon_ground_release_wait";
        if !stopped || victim_stop {
            assessment.contact_in = Some(assessment.remaining_stop as f32);
            assessment.prepare_guard = true;
            assessment.reason = "collision_predicted";
        }
        return assessment;
    }
    if (kirby_assist_core::native_states::status::CATCHED_AIR_GANON
        ..=kirby_assist_core::native_states::status::CATCHED_AIR_END_GANON)
        .contains(&status)
    {
        assessment.reason = "ganon_aerial_slam_untechable";
        return assessment;
    }
    if gate == TechGate::OutsideTumble
        && !(cfg!(feature = "diagnostic-logging")
            && (kirby_assist_core::native_states::status::DAMAGE
                ..=kirby_assist_core::native_states::status::DOWN_REFLECT_LR)
                .contains(&status))
    {
        return assessment;
    }
    let late_hitstop = gate == TechGate::Hitstop
        && victim_stop
        && (late_damage_stop_prediction(assessment.remaining_stop, input_delay)
            || assessment.remaining_stop == 0);
    let surfaces = [
        game::transition_enabled(
            object,
            kirby_assist_core::native_states::transition_terms::PASSIVE,
        ) || game::transition_enabled(
            object,
            kirby_assist_core::native_states::transition_terms::PASSIVE_FB,
        ),
        game::transition_enabled(
            object,
            kirby_assist_core::native_states::transition_terms::PASSIVE_WALL,
        ),
        game::transition_enabled(
            object,
            kirby_assist_core::native_states::transition_terms::PASSIVE_CEIL,
        ),
    ];
    let supported = (kirby_assist_core::native_states::status::DAMAGE_FLY
        ..=kirby_assist_core::native_states::status::DAMAGE_FALL)
        .contains(&status)
        && !disabled
        && (situation == 2 || situation == 0);
    // Use the game's current collision shape, including character animation,
    // scale and terrain corrections. Contact can precede a situation/status
    // change and frozen tumble poses need not have an advancing motion clock.
    let can_observe_contact = supported && (!stopped || victim_stop);
    let touched = [
        can_observe_contact && surfaces[0] && game::passive_ground_touch(object, 0xc8),
        can_observe_contact
            && surfaces[1]
            && (game::ground_touch(object, 0x52) || game::ground_touch(object, 0xa4)),
        can_observe_contact && surfaces[2] && game::ground_touch(object, 0x31),
    ];
    let predict = gate == TechGate::Predict || late_hitstop;
    // Diagnostics retain rejected-state velocity evidence. Silent only needs
    // these reads when prediction actually runs; no eligibility/window changes.
    if (predict || touched.iter().any(|v| *v) || cfg!(feature = "diagnostic-logging"))
        && (kirby_assist_core::native_states::status::DAMAGE
            ..=kirby_assist_core::native_states::status::DOWN_REFLECT_LR)
            .contains(&status)
    {
        // ALL includes damage knockback as well as main/gravity velocity.
        // MAIN (1) alone can omit the force carrying Kirby toward the surface.
        assessment.velocity = [reads.speed_x(object, -1), reads.speed_y(object, -1)];
    }
    if !predict && !touched.iter().any(|v| *v) {
        return assessment;
    }
    if victim_stop && (late_hitstop || touched.iter().any(|v| *v)) {
        // Kinetic energies may be frozen or still describe the previous hit.
        // Project the already reserved new launch, not zero/old motion.
        assessment.velocity = kirby_assist_core::tech_prediction::prediction_velocity(
            assessment.velocity,
            Some(reads.damage_launch(object)),
        );
    }
    // A transition term alone can be enabled before damage recovery permits
    // the transition. Require native END_REACTION / REACTION_CANCEL too.
    // All three keys are named in the pinned lua_const.rs; these are reads.
    let air_escape_available = game::transition_enabled(
        object,
        kirby_assist_core::native_states::transition_terms::CONT_ESCAPE_AIR,
    ) && (game::is_flag(
        object,
        kirby_assist_core::native_states::work_ids::FIGHTER_STATUS_DAMAGE_FLAG_END_REACTION,
    ) || game::is_flag(
        object,
        kirby_assist_core::native_states::work_ids::FIGHTER_STATUS_DAMAGE_FLAG_REACTION_CANCEL,
    ));
    if !tech_press_allowed(status, air_escape_available) {
        *floor_motion = FloorMotionHistory::default();
        assessment.reason = "air_dodge_available";
        assessment.prepare_guard = false;
        return assessment;
    }
    // Do not mistake an upward grounded launch's existing floor contact for
    // an impact. A reflect-down status itself is authoritative after rebound.
    let floor_contact = touched[0]
        && (status == kirby_assist_core::native_states::status::DAMAGE_FLY_REFLECT_D
            || assessment.velocity[1].is_finite() && assessment.velocity[1] <= 0.0001);
    if floor_contact || touched[1] || touched[2] {
        assessment.prepare_guard = true;
        assessment.contact_in = Some(assessment.remaining_stop as f32);
        assessment.reason = "native_collision_contact";
        return assessment;
    }
    if !predict {
        return assessment;
    }
    // Existing native gates establish safe preparation even when this frame's
    // bounded rays find no surface. Never infer this from status alone.
    assessment.prepare_guard = true;
    #[cfg(feature = "diagnostic-logging")]
    let _performance = crate::performance::Scope::new(crate::performance::Part::Tech);
    // Copy the animated ECB base afresh for this observation. Generic probes
    // anchored only to PostureModule::pos miss shifted/large/small fighters.
    let offset = [game::ground_offset_x(object), game::ground_offset_y(object)];
    let offset = if offset.iter().all(|v| v.is_finite() && v.abs() <= 100.) {
        offset
    } else {
        [0.; 2]
    };
    let x = reads.pos_x(object) + offset[0];
    let y = reads.pos_y(object) + offset[1];
    let body = body.scaled(game::scale(object));
    let [vx, vy] = assessment.velocity;
    if ![x, y, vx, vy].iter().all(|value| value.is_finite()) {
        *floor_motion = FloorMotionHistory::default();
        assessment.reason = "invalid_motion";
        return assessment;
    }
    assessment.reason = "no_collision_predicted";
    let horizon = (horizon + input_delay.clamp(0.0, 12.0)).clamp(1.0, 20.0);
    let acceleration = if stopped {
        0.0
    } else {
        floor_motion.sample(FloorMotionSample {
            object: object.cast::<u8>().add(8).cast::<u32>().read(),
            status,
            motion: reads.motion_kind(object),
            frame: reads.motion_frame(object),
            rate: reads.motion_rate(object),
            vy,
        })
    };
    assessment.contact_in = first_contact_swept(
        x,
        y,
        vx,
        vy,
        horizon,
        acceleration,
        body,
        surfaces,
        |probe| {
            let origin = game::Vector2 {
                x: probe.origin[0],
                y: probe.origin[1],
            };
            let path = game::Vector2 {
                x: probe.delta[0],
                y: probe.delta[1],
            };
            game::ray_check(object, &origin, &path, probe.passable) != 0
        },
    )
    .map(|flight| flight + assessment.remaining_stop as f32);
    if assessment.contact_in.is_some() {
        assessment.reason = "collision_predicted";
    }
    assessment
}
