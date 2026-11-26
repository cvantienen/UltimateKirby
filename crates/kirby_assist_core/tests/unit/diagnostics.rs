use super::*;
/// Regression test for `changing_progress_keys_cannot_bypass_the_formatting_rate_limit`.
#[test]
fn changing_progress_keys_cannot_bypass_the_formatting_rate_limit() {
    let mut gate = DiagnosticGate::new();
    let emissions = (0..=1000)
        .filter_map(|ms| gate.sample_throttled(ms, ms, true, 125))
        .collect::<Vec<_>>();
    assert_eq!(emissions.len(), 9);
    assert_eq!(emissions[1].coalesced_updates, 124);
    // A backwards/reset clock rearms instead of silencing diagnostics.
    assert!(gate.sample_throttled(2000, 0, true, 125).is_some());
}
/// Regression test for `waiting_frames_are_coalesced_but_attack_and_hit_events_are_immediate`.
#[test]
fn waiting_frames_are_coalesced_but_attack_and_hit_events_are_immediate() {
    let mut gate = DiagnosticGate::new();
    assert!(!gate.sample("roll_wait", 0, true).unwrap().periodic);
    for frame in 1..30 {
        assert!(gate.sample("roll_wait", frame * 16, true).is_none());
    }
    let attack = gate.sample("smash_request", 480, true).unwrap();
    assert!(!attack.periodic);
    assert_eq!(attack.coalesced_updates, 29);
    assert!(!gate.sample("hit_confirmed", 496, true).unwrap().periodic);
}
/// Regression test for `verbose_waits_emit_at_most_two_periodic_snapshots_per_second`.
#[test]
fn verbose_waits_emit_at_most_two_periodic_snapshots_per_second() {
    let mut gate = DiagnosticGate::new();
    let emissions = (0..=1000)
        .filter_map(|ms| gate.sample(1, ms, true))
        .collect::<Vec<_>>();
    assert_eq!(emissions.len(), 3); // Initial event, then 500ms and 1000ms.
    assert_eq!(emissions.iter().filter(|event| event.periodic).count(), 2);
    assert_eq!(emissions[1].coalesced_updates, 499);
}
/// Regression test for `nonverbose_mode_only_logs_events_and_reset_rearms_first_snapshot`.
#[test]
fn nonverbose_mode_only_logs_events_and_reset_rearms_first_snapshot() {
    let mut gate = DiagnosticGate::new();
    assert!(gate.sample(1, 0, false).is_some());
    assert!(gate.sample(1, 10_000, false).is_none());
    assert!(gate.sample(2, 10_001, false).is_some());
    gate = DiagnosticGate::new();
    assert!(gate.sample(2, 10_002, false).is_some());
}
