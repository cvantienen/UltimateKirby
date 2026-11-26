use super::*;
#[test]
/// Verifies distinct event and timing thresholds produce their labeled flags.
fn flags_separate_expiry_reprocessing_logger_and_growth_from_proof_of_heat() {
    let mut t = [Timing::default(); LANES];
    let mut calls = [0; LANES];
    calls[Lane::Draw as usize] = 300;
    let mut e = [0; EVENTS];
    e[Event::FullObservation as usize] = 600;
    e[Event::InputExpired as usize] = 1;
    e[Event::TimelineGap as usize] = 1;
    t[Lane::LogFlush as usize].max = 6_000_000;
    let text = review(&t, &calls, &e, 120_000, 1, 100, 3, 2);
    for code in [
        "INPUT_EXPIRY",
        "SEQUENCE_GAP",
        "EVALUATION_RATE",
        "LOGGER_COST",
        "LOGGER_PRESSURE",
        "RETAINED_GROWTH",
        "SAVE_WORKER",
        "CALLBACK_GAP",
    ] {
        assert!(text.contains(code), "{code}");
    }
    assert!(text.contains("not proof"));
}
#[test]
/// Verifies a quiet interval does not imply health and thresholds are contextual.
fn quiet_window_does_not_claim_system_health_and_expected_edits_need_context() {
    let text = review(
        &[Timing::default(); LANES],
        &[0; LANES],
        &[0; EVENTS],
        0,
        0,
        0,
        0,
        0,
    );
    assert!(text.contains("no configured threshold"));
    assert!(text.contains("not ruled out"));
    assert!(!text.contains("REVIEW ["));
}
