use super::*;
use kirby_assist_core::native_input::MAPPER_R;
use std::time::Duration;

/// Creates a minimal target-bearing fixture for trace tests.
fn snapshot(phase: &'static str) -> RollGrabSnapshot {
    RollGrabSnapshot {
        target: Some(42),
        phase,
        grab_button: MAPPER_R,
        can_grab: true,
        ..RollGrabSnapshot::default()
    }
}
#[test]
/// Verifies duplicate polls do not add repeated request or catch edges.
fn repeated_polls_do_not_fill_trace_or_invent_request_edges() {
    let now = Instant::now();
    let mut trace = GrabTrace::new();
    let s = snapshot("roll_request_grab");
    for _ in 0..1000 {
        trace.observe(Some(s), Some(6), now);
        trace.mapped(MAPPER_R, MAPPER_R, 0, MAPPED_CATCH, [0, 0], "ledge", now);
    }
    let report = trace.active.as_ref().unwrap();
    assert_eq!(report.requests, 1);
    assert_eq!(report.catch_edges, 1);
    assert_eq!(report.written, 2);
    assert!(report
        .format()
        .contains("mapped_CATCH_no_native_start_observed"));
}
#[test]
/// Verifies reports distinguish undelivered overlays from wrong button mapping.
fn mapper_evidence_distinguishes_expired_overlay_from_wrong_mapping() {
    let now = Instant::now();
    let mut trace = GrabTrace::new();
    trace.observe(Some(snapshot("roll_request_grab")), Some(6), now);
    trace.mapped(MAPPER_R, 0, 0, 0, [0, 0], "expired_observation", now);
    assert!(trace
        .active
        .as_ref()
        .unwrap()
        .format()
        .contains("overlay_not_delivered"));
    // GUARD is not CATCH; a shield mapping must not confirm grab delivery.
    trace.mapped(
        MAPPER_R,
        MAPPER_R,
        MAPPER_R,
        kirby_assist_core::native_input::MAPPED_GUARD,
        [0, 0],
        "ledge",
        now,
    );
    assert!(trace
        .active
        .as_ref()
        .unwrap()
        .format()
        .contains("raw_button_not_mapped_to_CATCH"));
    trace.mapped(
        MAPPER_R,
        MAPPER_R,
        MAPPER_R,
        MAPPED_CATCH,
        [0, 0],
        "ledge",
        now,
    );
    assert_eq!(trace.active.as_ref().unwrap().catch_edges, 1);
}
#[test]
/// Verifies capture evidence remains independent from bounded ring history.
fn acceptance_evidence_survives_a_full_ring_and_capture_is_separate() {
    let now = Instant::now();
    let mut trace = GrabTrace::new();
    trace.observe(Some(snapshot("roll_request_grab")), Some(5), now);
    let mut s = snapshot("roll_wait_native_catch");
    s.native_grabbing = true;
    s.animations = 1;
    trace.observe(Some(s), Some(5), now + Duration::from_millis(16));
    let report = trace.active.as_mut().unwrap();
    for ms in 0..100 {
        report.push(ms, Event::Observation(s));
    }
    let text = report.format();
    assert!(text.contains("first_request"));
    assert!(text.contains("first_native_start"));
    assert!(text.contains("native_grab_started_no_capture_observed"));
    assert_eq!(report.checkpoints.len(), CHECKPOINTS);
    s.captured = true;
    report.last = s;
    assert!(report.format().contains("result=capture_confirmed"));
}
#[test]
/// Verifies progress sampling and held-input completion behavior.
fn idle_progress_is_sampled_and_completed_trace_does_not_restart_while_held() {
    let now = Instant::now();
    let mut trace = GrabTrace::new();
    let s = snapshot("roll_fast_walk_to_grab");
    for ms in 0..1000 {
        trace.observe(Some(s), Some(6), now + Duration::from_millis(ms));
    }
    assert_eq!(trace.active.as_ref().unwrap().written, 8);
    trace.finish("test_release");
    trace.observe(Some(s), Some(6), now);
    assert!(!trace.active());
    trace.observe(None, Some(6), now);
    trace.observe(Some(s), Some(6), now);
    assert!(trace.active());
}
