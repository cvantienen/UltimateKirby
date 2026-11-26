use super::*;
#[test]
fn live_logs_include_ordinary_and_performance_reports_as_plain_text() {
    let mut file = None;
    let mut console = BufWriter::new(std::io::stdout());
    output(&mut file, &mut console, "live-route-test ordinary <script>");
    output_performance(&mut file, &mut console, "live-route-test performance");
    let batch: serde_json::Value = serde_json::from_str(&live_logs(0)).unwrap();
    let texts: Vec<_> = batch["entries"].as_array().unwrap().iter()
        .map(|entry| entry["text"].as_str().unwrap()).collect();
    assert!(texts.contains(&"live-route-test ordinary <script>"));
    assert!(texts.contains(&"live-route-test performance"));
    let next = batch["next"].as_u64().unwrap();
    let after: serde_json::Value = serde_json::from_str(&live_logs(next)).unwrap();
    assert!(after["entries"].as_array().unwrap().iter()
        .all(|entry| entry["id"].as_u64().unwrap() > next));
}

#[test]
/// Confirms ordinary and performance reports use distinct file paths.
fn performance_and_ordinary_reports_write_to_separate_files() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!("kirby-log-test-{}-{unique}", std::process::id()));
    std::fs::create_dir(&base).unwrap();
    let status_path = base.join("status.log");
    let perf_path = base.join("performance.log");
    let mut status = open_log(status_path.to_str().unwrap());
    let mut perf = open_log(perf_path.to_str().unwrap());
    let mut console = BufWriter::new(std::io::stdout());
    output(&mut status, &mut console, "ordinary action evidence");
    output_performance(&mut perf, &mut console, "RUNTIME HEALTH: timing evidence");
    status.as_mut().unwrap().flush().unwrap();
    perf.as_mut().unwrap().flush().unwrap();
    assert_eq!(
        std::fs::read_to_string(&status_path).unwrap(),
        "ordinary action evidence\n"
    );
    assert_eq!(
        std::fs::read_to_string(&perf_path).unwrap(),
        "RUNTIME HEALTH: timing evidence\n"
    );
    drop((status, perf));
    std::fs::remove_dir_all(base).unwrap();
}
#[test]
/// Confirms stale generations are hidden and rendering uses try-lock.
fn snapshot_cache_rejects_previous_matches_and_never_blocks_rendering() {
    clear_snapshot();
    let epoch = SNAPSHOT_EPOCH.load(Ordering::Acquire);
    publish_snapshot(epoch, "LATEST SNAPSHOT: roll read".into());
    assert!(with_snapshot(|card| assert!(card
        .to_str()
        .unwrap()
        .contains("roll read"))));
    {
        let _busy = SNAPSHOT_CARD.lock().unwrap();
        assert!(!with_snapshot(|_| panic!(
            "rendering must not wait for a busy cache"
        )));
    }
    clear_snapshot();
    publish_snapshot(epoch, "stale match".into());
    assert!(!with_snapshot(|_| panic!(
        "old queued reports must remain hidden"
    )));
}
#[test]
/// Confirms queue overflow drops the new item without blocking or losing old items.
fn full_queue_drops_instead_of_waiting_and_preserves_queued_event() {
    let (sender, receiver) = mpsc::sync_channel::<String>(1);
    let dropped = AtomicUsize::new(0);
    enqueue(&sender, &dropped, "smash requested".into());
    enqueue(&sender, &dropped, "periodic wait snapshot".into());
    assert_eq!(dropped.load(Ordering::Relaxed), 1);
    assert_eq!(receiver.try_recv().unwrap(), "smash requested");
    enqueue(&sender, &dropped, "hit confirmed".into());
    assert_eq!(receiver.try_recv().unwrap(), "hit confirmed");
}
#[test]
/// Confirms sends to a disconnected logger are safely ignored.
fn disconnected_logger_does_not_panic_or_block_gameplay() {
    let (sender, receiver) = mpsc::sync_channel::<String>(1);
    drop(receiver);
    enqueue(&sender, &AtomicUsize::new(0), "event".into());
}
