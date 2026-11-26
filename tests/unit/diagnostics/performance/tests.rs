use super::*;
#[test]
/// Exercises sampling enablement, report thresholds, epoch reset, and card freshness.
fn opt_in_sampling_reports_and_clears_without_stale_cards() {
    set_enabled(false);
    {
        let _root = Scope::root();
        ray();
    }
    assert!(report().is_none());
    set_enabled(true);
    TICK.store(0, Ordering::Relaxed);
    {
        let _root = Scope::root();
        let _tech = Scope::new(Part::Tech);
        ray();
    }
    {
        let mut stats = STATS.lock().unwrap();
        assert_eq!(stats.calls[Part::Tech as usize], 1);
        assert_eq!(stats.rays[Part::Tech as usize], 1);
        stats.since = Some(Instant::now() - Duration::from_secs(6));
    }
    let text = report().unwrap();
    health::exercise_report();
    assert!(text.contains("Tech prediction"));
    let mut shown = false;
    with_card(|_| shown = true);
    assert!(shown);
    clear();
    with_card(|_| panic!("old session card leaked"));
    assert!(report().is_none());
    set_enabled(false);
}
