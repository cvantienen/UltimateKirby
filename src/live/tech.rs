//! Native tech eligibility and collision/capture timing observations.
//!
//! This detector reports a possible contact. The portable TechInput scheduler
//! separately decides whether to request/release shield and tracks mapper
//! acknowledgment. Prediction, button delivery and successful tech are distinct.
//! Technical: Combines native eligibility/timing reads with portable swept-contact
//! prediction to produce a tech assessment.
//! Plain English: This module estimates whether a fighter may hit the ground and
//! whether the tech helper should prepare input.
use super::native_params::{
    PARAM_COMMON, PARAM_GANON_SPECIAL_S_PASSIVE_TRIGGER_FRAME, PARAM_NO_RAPID_FRAME_VALUE,
    PARAM_PASSIVE_TRIGGER_FRAME,
};
use crate::game_ffi as game;
use kirby_assist_core::tech_prediction::{
    first_contact_swept, late_damage_stop_prediction, tech_gate_with_floor, tech_press_allowed,
    FloorMotionHistory, FloorMotionSample, TechGate,
};
mod flame_choke {
    //! Read-only capture-link clock for grounded Flame Choke.
    //! Technical: Handles command-grab capture/release timing from the captor's native motion clock.
    //! Plain English: Checks when a grounded Flame Choke grab is about to release.
    use super::{game, TechAssessment};
    use kirby_assist_core::tech_prediction::{
        grounded_ganon_contact_in, grounded_ganon_release, CATCHED_AIR_END_GANON, CATCHED_GANON,
    };

    /// Technical: Applies the special grounded/aerial capture rules to an assessment.
    /// Plain English: Marks whether this Ganondorf grab can lead to a tech attempt.
    ///
    /// # Safety
    /// `object` must remain a valid fighter accessor for the native reads.
    pub(super) unsafe fn assess_capture(
        reads: &super::super::frame::FrameReads,
        object: *mut game::Object,
        assessment: &mut TechAssessment,
        stopped: bool,
        victim_stop: bool,
    ) -> bool {
        let status = assessment.status;
        if status == CATCHED_GANON {
            assessment.reason = "ganon_ground_capture_timing_unavailable";
            if let Some((contact, motion, frame, rate)) = timing(reads, object) {
                assessment.contact_in = Some(contact);
                assessment.capture_clock = Some((motion, frame, rate));
                assessment.prepare_guard = true;
                assessment.reason = "collision_predicted";
            }
            return true;
        }
        // Both throws share this release status; only a grounded predecessor
        // establishes an immediately techable release.
        if status == CATCHED_AIR_END_GANON
            && grounded_ganon_release(
                status,
                game::previous_status(object, game::LATEST_PREVIOUS_STATUS_HISTORY_INDEX),
            )
        {
            assessment.reason = "ganon_ground_release_wait";
            if !stopped || victim_stop {
                assessment.contact_now("collision_predicted");
            }
            return true;
        }
        if (kirby_assist_core::native_states::status::CATCHED_AIR_GANON
            ..=kirby_assist_core::native_states::status::CATCHED_AIR_END_GANON)
            .contains(&status)
        {
            assessment.reason = "ganon_aerial_slam_untechable";
            return true;
        }
        false
    }

    /// Technical: Resolves the linked captor and predicts grounded capture contact.
    /// Plain English: Uses the captor's animation to estimate when the grab releases.
    ///
    /// # Safety
    /// `object` must be a live victim object; any linked captor is revalidated by ID.
    unsafe fn timing(
        reads: &super::super::frame::FrameReads,
        object: *mut game::Object,
    ) -> Option<(f32, u64, f32, f32)> {
        if !game::is_link(object, 0) {
            return None;
        }
        let id = u32::try_from(game::link_object_id(object, 0)).ok()?;
        let own_id = game::battle_object_id(object);
        if id == own_id
            || id >> 28 != kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
            || !game::is_active(id)
        {
            return None;
        }
        let captor = game::module_accessor(id);
        if captor.is_null() || game::battle_object_id(captor) != id {
            return None;
        }
        let motion = reads.motion_kind(captor);
        if motion == kirby_assist_core::timing::NO_MOTION_KIND {
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
}
/// Technical: Smallest accepted native tech-trigger parameter value.
/// Plain English: Rejects implausibly short game timing values.
const MIN_NATIVE_TRIGGER_FRAME: u16 = 2;
/// Technical: Largest accepted native tech-trigger parameter value.
/// Plain English: Rejects implausibly long game timing values.
const MAX_NATIVE_TRIGGER_FRAME: u16 = 120;
/// Technical: Tolerance for classifying vertical velocity as non-upward.
/// Plain English: Treats tiny upward movement as effectively level.
const FLOOR_CONTACT_UPWARD_SPEED_TOLERANCE: f32 = 0.0001;
/// Technical: Maximum absolute ground-offset magnitude accepted from native reads.
/// Plain English: Ignores unrealistic stage offsets.
const MAX_GROUND_OFFSET_MAGNITUDE: f32 = 100.0;
/// Technical: Minimum horizon used for predicted floor contact.
/// Plain English: Always checks at least one frame ahead.
const MIN_CONTACT_HORIZON_FRAMES: f32 = 1.0;
/// Technical: Maximum horizon used for predicted floor contact.
/// Plain English: Does not predict a landing more than 20 frames ahead.
const MAX_CONTACT_HORIZON_FRAMES: f32 = 20.0;

/// Diagnostic snapshot only: the string describes our detector's decision, not
/// a complete explanation of all native tech restrictions or proof of success.
#[derive(Clone, Copy)]
/// Technical: Copied diagnostic/prediction result for one tech observation.
/// Plain English: Summarizes the fighter's state and why a tech may be expected.
pub(super) struct TechAssessment {
    /// Technical: Native fighter status at assessment time.
    /// Plain English: The fighter's current action/state code.
    pub(super) status: i32,
    /// Technical: Native ground/air situation at assessment time.
    /// Plain English: Whether the fighter is on the ground or in the air.
    pub(super) situation: i32,
    /// Technical: Copied X/Y velocity used for contact prediction.
    /// Plain English: The fighter's current movement speed and direction.
    pub(super) velocity: [f32; 2],
    /// Technical: Stable diagnostic identifier for the detector's result.
    /// Plain English: A short reason explaining the current tech assessment.
    pub(super) reason: &'static str,
    /// Technical: Predicted frames until contact, when available.
    /// Plain English: How soon the fighter may hit the ground.
    pub(super) contact_in: Option<f32>,
    /// Technical: Native victim-stop frames remaining.
    /// Plain English: How much hit pause remains for the fighter.
    pub(super) remaining_stop: u64,
    /// Technical: Whether guard should be prepared before the predicted contact.
    /// Plain English: Whether the helper should get shield ready.
    pub(super) prepare_guard: bool,
    /// Technical: Predicted remaining native reaction time, when known.
    /// Plain English: How long the fighter remains in its hit reaction.
    pub(super) reaction_remaining: Option<f32>,
    // A grounded command grab can freeze the victim's animation. Its linked
    // captor's native clock supplies progress without counting controller polls.
    /// Technical: Captor motion identity, frame, and rate used to estimate capture release timing.
    /// Plain English: The grabber's animation timing used to predict when the grab releases.
    pub(super) capture_clock: Option<(u64, f32, f32)>,
}
impl TechAssessment {
    /// Technical: Marks contact as immediate using remaining hitstop as the estimate.
    /// Plain English: Records that the landing/contact is happening now.
    fn contact_now(&mut self, reason: &'static str) {
        self.contact_in = Some(self.remaining_stop as f32);
        self.prepare_guard = true;
        self.reason = reason;
    }
}

/// Pinned common.prc integer parameters, read without changing native timing.
/// Plain English: Reads the game's timing settings and substitutes safe defaults when values are invalid.
pub(super) unsafe fn native_timing(object: *mut game::Object, fallback: u16) -> (u16, u16, u16) {
    let bounded = |value: i32, fallback: u16| {
        u16::try_from(value)
            .ok()
            .filter(|v| (MIN_NATIVE_TRIGGER_FRAME..=MAX_NATIVE_TRIGGER_FRAME).contains(v))
            .unwrap_or(fallback)
    };
    (
        bounded(
            game::param_int(object, PARAM_COMMON, PARAM_NO_RAPID_FRAME_VALUE),
            fallback,
        ), // no_rapid_frame_value
        bounded(
            game::param_int(object, PARAM_COMMON, PARAM_PASSIVE_TRIGGER_FRAME),
            kirby_assist_core::tech_input::DEFAULT_PASSIVE_TRIGGER_FRAME,
        ), // passive_trigger_frame
        bounded(
            game::param_int(
                object,
                PARAM_COMMON,
                PARAM_GANON_SPECIAL_S_PASSIVE_TRIGGER_FRAME,
            ),
            kirby_assist_core::tech_input::DEFAULT_GANON_PASSIVE_TRIGGER_FRAME,
        ),
    ) // ganon_special_s_passive_trigger_frame
}

/// Start with native status/eligibility gates, then test swept stage rays. Keep
/// the fixed body offsets in tech_prediction.rs separate so they can be tested
/// on the host and tuned without scattering geometry through native FFI code.
/// Plain English: Checks native fighter state and estimates whether the fighter can tech a landing.
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
    let floor_tech = situation == kirby_assist_core::native_states::situation::GROUND
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
        && game::previous_situation_kind(object)
            == kirby_assist_core::native_states::situation::AIR;
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
            let remaining = game::work_float(
                object,
                kirby_assist_core::native_states::work_ids::FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_REACTION_FRAME,
            );
            (remaining.is_finite()
                && remaining > 0.
                && remaining <= kirby_assist_core::tech_input::MAX_REACTION_REMAINING_FRAMES)
                .then_some(remaining)
        } else {
            None
        },
    };
    // The native STOP_WALL / STOP_CEIL statuses still admit passive input at
    // collision. Damage-fly-only gates used to discard this final opportunity.
    // Native transition terms, not the status name alone, authorize the input.
    if situation == kirby_assist_core::native_states::situation::AIR
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
            assessment.contact_now("collision_predicted");
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
        let vy = reads.speed_y(object);
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
            || velocity[1].is_finite() && velocity[1] <= FLOOR_CONTACT_UPWARD_SPEED_TOLERANCE
        {
            assessment.velocity = velocity;
            assessment.contact_now("collision_predicted");
            return assessment;
        }
    }
    if flame_choke::assess_capture(reads, object, &mut assessment, stopped, victim_stop) {
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
    let (floor_surface_enabled, wall_surface_enabled, ceiling_surface_enabled) = (
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
    );
    let supported = (kirby_assist_core::native_states::status::DAMAGE_FLY
        ..=kirby_assist_core::native_states::status::DAMAGE_FALL)
        .contains(&status)
        && !disabled
        && (situation == kirby_assist_core::native_states::situation::AIR
            || situation == kirby_assist_core::native_states::situation::GROUND);
    // Use the game's current collision shape, including character animation,
    // scale and terrain corrections. Contact can precede a situation/status
    // change and frozen tumble poses need not have an advancing motion clock.
    let can_observe_contact = supported && (!stopped || victim_stop);
    let floor_touched = can_observe_contact
        && floor_surface_enabled
        && game::passive_ground_touch(
            object,
            kirby_assist_core::native_states::ground_touch::DOWN_SIDE,
        );
    let wall_touched = can_observe_contact
        && wall_surface_enabled
        && (game::ground_touch(
            object,
            kirby_assist_core::native_states::ground_touch::LEFT_SIDE,
        ) || game::ground_touch(
            object,
            kirby_assist_core::native_states::ground_touch::RIGHT_SIDE,
        ));
    let ceiling_touched = can_observe_contact
        && ceiling_surface_enabled
        && game::ground_touch(
            object,
            kirby_assist_core::native_states::ground_touch::UP_SIDE,
        );
    let any_surface_touched = floor_touched || wall_touched || ceiling_touched;
    let predict = gate == TechGate::Predict || late_hitstop;
    // Diagnostics retain rejected-state velocity evidence. Silent only needs
    // these reads when prediction actually runs; no eligibility/window changes.
    if (predict || any_surface_touched || cfg!(feature = "diagnostic-logging"))
        && (kirby_assist_core::native_states::status::DAMAGE
            ..=kirby_assist_core::native_states::status::DOWN_REFLECT_LR)
            .contains(&status)
    {
        // ALL includes damage knockback as well as main/gravity velocity.
        // MAIN (1) alone can omit the force carrying Kirby toward the surface.
        assessment.velocity = [reads.speed_x(object), reads.speed_y(object)];
    }
    if !predict && !any_surface_touched {
        return assessment;
    }
    if victim_stop && (late_hitstop || any_surface_touched) {
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
    let floor_contact = floor_touched
        && (status == kirby_assist_core::native_states::status::DAMAGE_FLY_REFLECT_D
            || assessment.velocity[1].is_finite()
                && assessment.velocity[1] <= FLOOR_CONTACT_UPWARD_SPEED_TOLERANCE);
    if floor_contact || wall_touched || ceiling_touched {
        assessment.contact_now("native_collision_contact");
        return assessment;
    }
    if !predict {
        return assessment;
    }
    predict_contact(
        reads,
        object,
        horizon,
        stopped,
        input_delay,
        floor_motion,
        body,
        [
            floor_surface_enabled,
            wall_surface_enabled,
            ceiling_surface_enabled,
        ],
        assessment,
    )
}

#[inline]
/// Technical: Predicts ground contact from native motion, body geometry, and bounded floor probes.
/// Plain English: Estimates when and where the fighter may land.
///
/// # Safety
/// `object` must be a live fighter accessor for the duration of native reads.
unsafe fn predict_contact(
    reads: &super::frame::FrameReads,
    object: *mut game::Object,
    horizon: f32,
    stopped: bool,
    input_delay: f32,
    floor_motion: &mut FloorMotionHistory,
    body: kirby_assist_core::tech_prediction::Body,
    surfaces: [bool; 3],
    mut assessment: TechAssessment,
) -> TechAssessment {
    // Existing native gates establish safe preparation even when this frame's
    // bounded rays find no surface. Never infer this from status alone.
    assessment.prepare_guard = true;
    #[cfg(feature = "diagnostic-logging")]
    let _performance = crate::performance::Scope::new(crate::performance::Part::Tech);
    // Copy the animated ECB base afresh for this observation. Generic probes
    // anchored only to PostureModule::pos miss shifted/large/small fighters.
    let offset = [game::ground_offset_x(object), game::ground_offset_y(object)];
    let offset = if offset
        .iter()
        .all(|v| v.is_finite() && v.abs() <= MAX_GROUND_OFFSET_MAGNITUDE)
    {
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
    let horizon = (horizon
        + input_delay.clamp(0.0, kirby_assist_core::tech_input::MAX_INPUT_DELAY_FRAMES))
    .clamp(MIN_CONTACT_HORIZON_FRAMES, MAX_CONTACT_HORIZON_FRAMES);
    let acceleration = if stopped {
        0.0
    } else {
        floor_motion.sample(FloorMotionSample {
            object: game::battle_object_id(object),
            status: assessment.status,
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
