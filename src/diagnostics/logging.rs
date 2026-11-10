//! Diagnostic-only logger. Game hooks enqueue; a worker batches console/SD I/O.
//! Silent builds exclude this module and its thread/channel entirely.
//! Technical: Queues diagnostic messages and batches console/SD output on a worker
//! rather than inside native callbacks.
//! Plain English: This module writes optional logs in the background so gameplay
//! callbacks stay lightweight.
use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
        OnceLock,
    },
    time::{Duration, Instant},
};

/// Technical: SD-card destination for ordinary diagnostic messages.
/// Plain English: File where status and action logs are written.
const LOG_PATH: &str = "sd:/ultimate/kirby_assist/status.log";
/// Technical: SD-card destination for performance summaries.
/// Plain English: Separate file where timing reports are written.
const PERFORMANCE_PATH: &str = "sd:/ultimate/kirby_assist/performance.log";
/// Technical: Maximum number of pending messages in the nonblocking sync queue.
/// Plain English: Bounds how many log messages can wait for the background writer.
const QUEUE_CAPACITY: usize = 128;
/// Technical: Receiver timeout and periodic flush interval.
/// Plain English: The logger checks in and flushes about four times per second.
const FLUSH_INTERVAL: Duration = Duration::from_millis(250);
/// Technical: Typed queue payloads consumed by the background writer.
/// Plain English: Different kinds of diagnostic reports waiting to be written.
enum Message {
    /// Technical: Plain owned diagnostic text.
    /// Plain English: A regular log line.
    Text(String),
    /// Technical: Mode report tagged with the snapshot-cache generation.
    /// Plain English: A mode summary that can update the menu card if still current.
    Modes(usize, Box<kirby_assist_core::diagnostics::modes::Report>),
    /// Technical: Owned configuration diagnostic payload.
    /// Plain English: A settings snapshot queued for logging.
    Configuration(Box<kirby_assist_core::diagnostics::modes::Configuration>),
    /// Technical: Owned bounded grab-trace report.
    /// Plain English: A completed grab troubleshooting report.
    Grab(Box<crate::grab_diagnostics::Report>),
    /// Technical: Owned ledge calibration report.
    /// Plain English: A completed calibration troubleshooting report.
    Calibration(Box<crate::calibration::Report>),
}
/// Technical: One-time sender used by callbacks to enqueue messages without blocking.
/// Plain English: Lets game callbacks hand logs to the background writer.
static SENDER: OnceLock<SyncSender<Message>> = OnceLock::new();
/// Technical: Counts messages rejected because the bounded queue was full.
/// Plain English: Tracks log lines that could not be queued in time.
static DROPPED: AtomicUsize = AtomicUsize::new(0);

// Generation tags prevent a queued report from an old match repopulating the UI.
/// Technical: Generation counter invalidating stale mode snapshots.
/// Plain English: Prevents a report from an old match replacing the current card.
static SNAPSHOT_EPOCH: AtomicUsize = AtomicUsize::new(0);
#[cfg(any(feature = "imgui-menu", test))]
/// Technical: Nonblocking cached card text paired with the generation that created it.
/// Plain English: Stores the latest mode summary shown in the menu.
static SNAPSHOT_CARD: std::sync::Mutex<Option<(usize, std::ffi::CString)>> =
    std::sync::Mutex::new(None);
/// Technical: Advances the cache generation and clears any currently cached card.
/// Plain English: Hides old diagnostic text after a match or state reset.
pub fn clear_snapshot() {
    SNAPSHOT_EPOCH.fetch_add(1, Ordering::AcqRel);
    #[cfg(any(feature = "imgui-menu", test))]
    if let Ok(mut card) = SNAPSHOT_CARD.try_lock() {
        *card = None;
    }
}
#[cfg(any(feature = "imgui-menu", test))]
/// Technical: Publishes text only if its generation remains current across locking.
/// Plain English: Caches a report only when it still belongs to the active match.
fn publish_snapshot(epoch: usize, text: String) {
    if epoch != SNAPSHOT_EPOCH.load(Ordering::Acquire) {
        return;
    }
    if let Ok(text) = std::ffi::CString::new(text) {
        if let Ok(mut card) = SNAPSHOT_CARD.lock() {
            if epoch == SNAPSHOT_EPOCH.load(Ordering::Acquire) {
                *card = Some((epoch, text));
            }
        }
    }
}
#[cfg(any(feature = "imgui-menu", test))]
/// Technical: Calls `draw` only while a nonblocking lock holds a current cached card.
/// Plain English: Shows the latest valid summary without making rendering wait.
pub fn with_snapshot(draw: impl FnOnce(&std::ffi::CStr)) -> bool {
    if let Ok(card) = SNAPSHOT_CARD.try_lock() {
        if let Some((epoch, text)) = card.as_ref() {
            if *epoch == SNAPSHOT_EPOCH.load(Ordering::Acquire) {
                draw(text);
                return true;
            }
        }
    }
    false
}

/// Startup I/O happens before hooks are installed. Gameplay never opens files.
/// Technical: Creates diagnostic files, starts the bounded channel, and launches its writer.
/// Plain English: Prepares background logging before gameplay callbacks begin.
pub fn start() {
    if SENDER.get().is_some() {
        return;
    }
    if let Err(error) = std::fs::create_dir_all("sd:/ultimate/kirby_assist") {
        eprintln!("[kirby_assist] diagnostic log directory failed: {error}; expected {LOG_PATH}");
    }
    for path in [LOG_PATH, PERFORMANCE_PATH] {
        if let Err(error) = std::fs::write(
            path,
            concat!(
                "Kirby Assist diagnostic build ",
                env!("CARGO_PKG_VERSION"),
                "; queued logging; files reset at plugin start\n"
            ),
        ) {
            eprintln!("[kirby_assist] diagnostic startup log failed: {error}; expected {path}");
        }
    }
    let (sender, receiver) = mpsc::sync_channel(QUEUE_CAPACITY);
    // Spawn failure disables logging instead of panicking or falling back to
    // blocking file writes from a gameplay hook. Dropping the handle detaches.
    match std::thread::Builder::new()
        .name("kirby-assist-log".into())
        .spawn(move || writer_loop(receiver))
    {
        Ok(_) => { let _ = SENDER.set(sender); }
        Err(error) => eprintln!("[kirby_assist] diagnostic writer thread failed: {error}; startup header only at {LOG_PATH}"),
    }
}

/// Technical: Copies borrowed text and queues it for asynchronous output.
/// Plain English: Sends a log message to be written later.
pub fn write(message: &str) {
    write_owned(message.to_owned());
}

/// Technical: Enqueues owned text if the background logger has started.
/// Plain English: Hands an already-created log message to the writer.
pub fn write_owned(message: String) {
    if let Some(sender) = SENDER.get() {
        enqueue_message(sender, &DROPPED, Message::Text(message));
    }
}

/// Technical: Transfers a completed grab report into the bounded writer queue.
/// Plain English: Sends a grab troubleshooting report to the logger.
pub fn write_grab(report: crate::grab_diagnostics::Report) {
    if let Some(sender) = SENDER.get() {
        enqueue_message(sender, &DROPPED, Message::Grab(Box::new(report)));
    }
}
/// Technical: Transfers a completed calibration report into the bounded writer queue.
/// Plain English: Sends a calibration report to the logger.
pub fn write_calibration(report: crate::calibration::Report) {
    if let Some(sender) = SENDER.get() {
        enqueue_message(sender, &DROPPED, Message::Calibration(Box::new(report)));
    }
}

/// Technical: Queues a mode report tagged for generation-safe card publication.
/// Plain English: Sends a mode summary to the logger and menu card.
pub fn write_modes(report: kirby_assist_core::diagnostics::modes::Report) {
    if let Some(sender) = SENDER.get() {
        enqueue_message(
            sender,
            &DROPPED,
            Message::Modes(SNAPSHOT_EPOCH.load(Ordering::Acquire), Box::new(report)),
        );
    }
}
/// Technical: Queues an owned configuration snapshot for asynchronous formatting.
/// Plain English: Sends current settings details to the logger.
pub fn write_configuration(config: kirby_assist_core::diagnostics::modes::Configuration) {
    if let Some(sender) = SENDER.get() {
        enqueue_message(sender, &DROPPED, Message::Configuration(Box::new(config)));
    }
}

/// Technical: Attempts a nonblocking send and accounts for rejected queue entries.
/// Plain English: Adds a message if space is available; logging never stalls gameplay.
fn enqueue_message(sender: &SyncSender<Message>, dropped: &AtomicUsize, message: Message) {
    crate::performance::health::queue_enter();
    match sender.try_send(message) {
        Ok(()) => {}
        Err(error) => {
            crate::performance::health::queue_leave();
            if matches!(error, TrySendError::Full(_)) {
                dropped.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}
#[cfg(test)]
/// Technical: Test-only generic variant for verifying bounded queue semantics.
/// Plain English: Exercises queue behavior without starting the real writer.
fn enqueue<T>(sender: &SyncSender<T>, dropped: &AtomicUsize, message: T) {
    if let Err(TrySendError::Full(_)) = sender.try_send(message) {
        dropped.fetch_add(1, Ordering::Relaxed);
    }
}
/// Technical: Opens an append-only buffered log and degrades to console on failure.
/// Plain English: Tries to open a log file without making logging depend on the SD card.
fn open_log(path: &str) -> Option<BufWriter<std::fs::File>> {
    match OpenOptions::new().create(true).append(true).open(path) {
        Ok(file) => Some(BufWriter::with_capacity(16 * 1024, file)),
        Err(error) => {
            eprintln!(
                "[kirby_assist] diagnostic log open failed: {error}; console only; expected {path}"
            );
            None
        }
    }
}
/// Technical: Flushes a file buffer and disables that file sink if flushing fails.
/// Plain English: Saves pending log text and falls back to console if the file fails.
fn flush_log(
    file: &mut Option<BufWriter<std::fs::File>>,
    console: &mut BufWriter<std::io::Stdout>,
    path: &str,
) {
    if let Some(error) = file.as_mut().and_then(|f| f.flush().err()) {
        let _ = writeln!(
            console,
            "[kirby_assist] LOG FLUSH ERROR: {error}; continuing console logging; expected {path}"
        );
        *file = None;
    }
}

/// Technical: Drains the bounded queue, formats reports, samples performance,
/// and periodically flushes output sinks.
/// Plain English: Writes queued diagnostics in the background, away from gameplay.
fn writer_loop(receiver: Receiver<Message>) {
    // Both files share the existing worker; no extra logging thread/queue.
    let mut file = open_log(LOG_PATH);
    let mut performance_file = open_log(PERFORMANCE_PATH);
    let mut console = BufWriter::with_capacity(16 * 1024, std::io::stdout());
    let mut flushed = Instant::now();
    loop {
        match receiver.recv_timeout(FLUSH_INTERVAL) {
            Ok(message) => {
                let _batch_time = crate::performance::health::Trace::new(
                    crate::performance::health::Lane::LogWork,
                );
                crate::performance::health::queue_leave();
                output_message(&mut file, &mut console, message);
                for message in receiver.try_iter().take(63) {
                    crate::performance::health::queue_leave();
                    output_message(&mut file, &mut console, message);
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                flush_log(&mut file, &mut console, LOG_PATH);
                flush_log(&mut performance_file, &mut console, PERFORMANCE_PATH);
                let _ = console.flush();
                break;
            }
        }
        let dropped = DROPPED.swap(0, Ordering::Relaxed);
        if crate::live::feature_activity_allowed() {
            let _report_time =
                crate::performance::health::Trace::new(crate::performance::health::Lane::LogWork);
            if let Some(report) = crate::performance::report() {
                output_performance(&mut performance_file, &mut console, &report);
            }
        }
        if dropped != 0 {
            crate::performance::health::add_event(
                crate::performance::health::Event::LogDropped,
                dropped,
            );
            output(
                &mut file,
                &mut console,
                &format!("diagnostic queue: dropped {dropped} messages; gameplay did not wait"),
            );
        }
        if flushed.elapsed() >= FLUSH_INTERVAL {
            let _flush_time =
                crate::performance::health::Trace::new(crate::performance::health::Lane::LogFlush);
            flush_log(&mut file, &mut console, LOG_PATH);
            flush_log(&mut performance_file, &mut console, PERFORMANCE_PATH);
            let _ = console.flush();
            flushed = Instant::now();
        }
    }
}

/// Technical: Dispatches each typed queue message to formatting and output sinks.
/// Plain English: Turns a queued report into text and writes it.
fn output_message(
    file: &mut Option<BufWriter<std::fs::File>>,
    console: &mut BufWriter<std::io::Stdout>,
    message: Message,
) {
    match message {
        Message::Text(message) => output(file, console, &message),
        Message::Modes(epoch, report) => {
            #[cfg(any(feature = "imgui-menu", test))]
            publish_snapshot(epoch, report.card_text());
            #[cfg(not(any(feature = "imgui-menu", test)))]
            let _ = epoch;
            output(file, console, &report.format());
        }
        Message::Configuration(config) => output(file, console, &config.format()),
        Message::Grab(report) => output(file, console, &report.format()),
        Message::Calibration(report) => {
            let formatted = report.format();
            crate::calibration::publish(&report);
            output(file, console, &formatted);
        }
    }
}

/// Technical: Writes formatted performance data to the dedicated performance log.
/// Plain English: Keeps timing reports separate from ordinary status logs.
fn output_performance(
    file: &mut Option<BufWriter<std::fs::File>>,
    console: &mut BufWriter<std::io::Stdout>,
    report: &str,
) {
    output_to(file, console, report, PERFORMANCE_PATH);
}

/// Technical: Writes ordinary diagnostic text to the status log.
/// Plain English: Saves one regular status message.
fn output(
    file: &mut Option<BufWriter<std::fs::File>>,
    console: &mut BufWriter<std::io::Stdout>,
    message: &str,
) {
    output_to(file, console, message, LOG_PATH);
}
/// Technical: Writes to console and optional file, retiring a failed file sink.
/// Plain English: Prints a report and saves it when the file is available.
fn output_to(
    file: &mut Option<BufWriter<std::fs::File>>,
    console: &mut BufWriter<std::io::Stdout>,
    message: &str,
    path: &str,
) {
    // BufWriter owns a stdout handle, not its lock. No global console lock is
    // held while waiting for another queue message. Batch flushes run here only.
    let _ = writeln!(console, "[kirby_assist] {message}");
    if let Some(error) = file
        .as_mut()
        .and_then(|file| writeln!(file, "{message}").err())
    {
        let _ = writeln!(console, "[kirby_assist] LOG FILE ERROR: {error}. Continuing console logging; SD log may be incomplete: {path}");
        *file = None;
    }
}

#[cfg(test)]
/// Technical: Logging queue, report-routing, and stale-snapshot regression tests.
/// Plain English: Checks that logs stay nonblocking and reports reach the right place.
mod tests {
    use super::*;
    #[test]
    /// Technical: Confirms ordinary and performance reports use distinct file paths.
    /// Plain English: Ensures timing reports do not mix with status logs.
    fn performance_and_ordinary_reports_write_to_separate_files() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base =
            std::env::temp_dir().join(format!("kirby-log-test-{}-{unique}", std::process::id()));
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
    /// Technical: Confirms stale generations are hidden and rendering uses try-lock.
    /// Plain English: Ensures old cards disappear and the menu never waits on the logger.
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
    /// Technical: Confirms queue overflow drops the new item without blocking or losing old items.
    /// Plain English: Checks a full queue does not stall the game or erase earlier messages.
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
    /// Technical: Confirms sends to a disconnected logger are safely ignored.
    /// Plain English: Ensures logging failure does not crash or stall gameplay.
    fn disconnected_logger_does_not_panic_or_block_gameplay() {
        let (sender, receiver) = mpsc::sync_channel::<String>(1);
        drop(receiver);
        enqueue(&sender, &AtomicUsize::new(0), "event".into());
    }
}
