//! Read-only capture-link clock for grounded Flame Choke.
use super::{game, TechAssessment};
use kirby_assist_core::tech_prediction::{
    grounded_ganon_contact_in, grounded_ganon_release, CATCHED_AIR_END_GANON, CATCHED_GANON,
};

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
            assessment.contact_in = Some(assessment.remaining_stop as f32);
            assessment.prepare_guard = true;
            assessment.reason = "collision_predicted";
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
