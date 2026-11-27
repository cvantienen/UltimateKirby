use super::*;
#[test]
/// Exercises sampling enablement, report thresholds, epoch reset, and card freshness.
fn opt_in_sampling_reports_and_clears_without_stale_cards() {
    let _serial = TEST_LOCK.lock().unwrap();
    observe(true);
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
    observe(false);
}

#[test]
fn training_exit_stops_collection_and_invalidates_in_flight_measurements() {
    let _serial = TEST_LOCK.lock().unwrap();
    observe(false);
    set_enabled(true);
    assert!(!enabled());
    assert!(Scope::root().start.is_none());
    assert!(health::report().is_none());
    observe(true);
    TICK.store(0, Ordering::Relaxed);
    let root = Scope::root();
    assert!(root.start.is_some());
    observe(false);
    observe(true);
    drop(root);
    assert_eq!(STATS.lock().unwrap().calls[Part::Total as usize], 0);
    observe(false);
    set_enabled(false);
}
