use super::*;

struct Session;
impl Session {
    fn begin() -> Self {
        let old = SESSION.load(Ordering::Acquire);
        SESSION.store((old.wrapping_add(2)) | 1, Ordering::Release);
        Self
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        observe(false);
    }
}

#[test]
fn live_logs_are_bounded_plain_text_and_session_scoped() {
    let _serial = TEST_LOCK.lock().unwrap();
    let _performance = crate::performance::TEST_LOCK.lock().unwrap();
    let _session = Session::begin();
    let epoch = SESSION.load(Ordering::Acquire);
    let token = session();
    assert_ne!(token, 0);
    live_output(epoch, "ordinary <script>");
    live_output(epoch, "RUNTIME HEALTH: timing evidence");
    let batch: serde_json::Value = serde_json::from_str(&live_logs(0)).unwrap();
    assert_eq!(batch["entries"][0]["text"], "ordinary <script>");
    assert_eq!(
        batch["entries"][1]["text"],
        "RUNTIME HEALTH: timing evidence"
    );
    let next = batch["next"].as_u64().unwrap();
    let after: serde_json::Value = serde_json::from_str(&live_logs(next)).unwrap();
    assert!(after["entries"].as_array().unwrap().is_empty());
    for _ in 0..200 {
        live_output(epoch, &"é".repeat(4000));
    }
    let batch: serde_json::Value = serde_json::from_str(&live_logs(0)).unwrap();
    let entries = batch["entries"].as_array().unwrap();
    assert!(entries.len() <= kirby_assist_core::diagnostic_http::MAX_LOG_ENTRIES);
    let bytes: usize = entries
        .iter()
        .map(|e| e["text"].as_str().unwrap().len())
        .sum();
    assert!(bytes <= kirby_assist_core::diagnostic_http::MAX_LOG_TEXT_BYTES);
    assert!(batch["dropped"].as_u64().unwrap() > 0);
    observe(false);
    assert!(!active());
    assert_eq!(session(), 0);
    let hidden: serde_json::Value = serde_json::from_str(&live_logs(0)).unwrap();
    assert!(hidden["entries"].as_array().unwrap().is_empty());
    let _next = Session::begin();
    output_message(Envelope {
        session: epoch,
        message: Message::Text("stale work".into()),
    });
    let fresh = SESSION.load(Ordering::Acquire);
    assert_ne!(session(), token);
    enqueue_for_session(token, || {
        panic!("stale deferred payload must not be prepared")
    });
    live_output(epoch, "late publication");
    live_output(fresh, "new session only");
    let batch: serde_json::Value = serde_json::from_str(&live_logs(0)).unwrap();
    assert_eq!(batch["entries"].as_array().unwrap().len(), 1);
    assert_eq!(batch["entries"][0]["text"], "new session only");
}

#[test]
fn inactive_collection_never_constructs_payloads_and_has_no_io_sinks() {
    let _serial = TEST_LOCK.lock().unwrap();
    let _performance = crate::performance::TEST_LOCK.lock().unwrap();
    observe(false);
    start();
    enqueue(|| panic!("inactive enqueue must not allocate or format"));
    let (sender, receiver) = mpsc::sync_channel(1);
    enqueue_message(
        &sender,
        Envelope {
            session: SESSION.load(Ordering::Acquire),
            message: Message::Text("inactive".into()),
        },
    );
    assert!(matches!(
        receiver.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));
    let source = include_str!("../../../src/diagnostics/logging.rs");
    for capability in [
        "std::fs",
        "fs::",
        "OpenOptions",
        "BufWriter",
        "stdout",
        "stderr",
        "println!",
        "sd:/",
    ] {
        assert!(
            !source.contains(capability),
            "logger regained output capability: {capability}"
        );
    }
}

#[test]
fn snapshot_cache_rejects_previous_sessions_and_never_blocks_rendering() {
    let _serial = TEST_LOCK.lock().unwrap();
    let _performance = crate::performance::TEST_LOCK.lock().unwrap();
    let _session = Session::begin();
    clear_snapshot();
    let epoch = SNAPSHOT_EPOCH.load(Ordering::Acquire);
    let session = SESSION.load(Ordering::Acquire);
    publish_snapshot(session, epoch, "LATEST SNAPSHOT: roll read".into());
    assert!(with_snapshot(|card| assert!(card
        .to_str()
        .unwrap()
        .contains("roll read"))));
    {
        let _busy = SNAPSHOT_CARD.lock().unwrap();
        assert!(!with_snapshot(|_| panic!("must not wait for cache")));
    }
    observe(false);
    let _next = Session::begin();
    publish_snapshot(session, epoch, "stale match".into());
    assert!(!with_snapshot(|_| panic!("stale card leaked")));
}

#[test]
fn bounded_queue_drops_full_disconnected_and_stale_work_without_waiting() {
    let _serial = TEST_LOCK.lock().unwrap();
    let _performance = crate::performance::TEST_LOCK.lock().unwrap();
    let _session = Session::begin();
    let session = SESSION.load(Ordering::Acquire);
    let (sender, receiver) = mpsc::sync_channel(1);
    let message = |text: &str| Envelope {
        session,
        message: Message::Text(text.into()),
    };
    enqueue_message(&sender, message("first"));
    enqueue_message(&sender, message("overflow"));
    let Message::Text(text) = receiver.try_recv().unwrap().message else {
        panic!("wrong payload")
    };
    assert_eq!(text, "first");
    assert!(receiver.try_recv().is_err());
    observe(false);
    let _next = Session::begin();
    enqueue_message(&sender, message("stale"));
    assert!(receiver.try_recv().is_err());
    drop(receiver);
    enqueue_message(
        &sender,
        Envelope {
            session: SESSION.load(Ordering::Acquire),
            message: Message::Text("disconnected".into()),
        },
    );
    assert_eq!(
        bounded_text(&"é".repeat(3000)).len(),
        kirby_assist_core::diagnostic_http::MAX_LOG_EVENT_BYTES
    );
}

#[test]
fn lazy_worker_delivers_only_training_messages_across_reentry() {
    let _serial = TEST_LOCK.lock().unwrap();
    let _performance = crate::performance::TEST_LOCK.lock().unwrap();
    observe(false);
    write("outside before entry");
    observe(true);
    write("first training message");
    let first_session = session();
    let wait_for = |needle: &str| {
        for _ in 0..200 {
            if live_logs(0).contains(needle) {
                return;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("worker did not deliver {needle}");
    };
    wait_for("first training message");
    let first = live_logs(0);
    assert!(!first.contains("outside before entry"));
    observe(false);
    write("outside after exit");
    observe(true);
    write_owned_for_session(first_session, "old deferred preparation".into());
    write_owned_for_session(session(), "second training message".into());
    wait_for("second training message");
    let second = live_logs(0);
    assert!(!second.contains("first training message"));
    assert!(!second.contains("outside after exit"));
    assert!(!second.contains("old deferred preparation"));
    observe(false);
}
