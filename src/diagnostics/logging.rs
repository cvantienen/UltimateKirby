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

/// SD-card destination for ordinary diagnostic messages.
const LOG_PATH: &str = "sd:/ultimate/kirby_assist/status.log";
/// SD-card destination for performance summaries.
const PERFORMANCE_PATH: &str = "sd:/ultimate/kirby_assist/performance.log";
/// Maximum number of pending messages in the nonblocking sync queue.
const QUEUE_CAPACITY: usize = 128;
/// Receiver timeout and periodic flush interval.
const FLUSH_INTERVAL: Duration = Duration::from_millis(250);
/// Typed queue payloads consumed by the background writer.
enum Message {
    /// Plain owned diagnostic text.
    Text(String),
    /// Mode report tagged with the snapshot-cache generation.
    Modes(usize, Box<kirby_assist_core::diagnostics::modes::Report>),
    /// Owned configuration diagnostic payload.
    Configuration(Box<kirby_assist_core::diagnostics::modes::Configuration>),
    /// Owned bounded grab-trace report.
    Grab(Box<crate::grab_diagnostics::Report>),
    /// Owned ledge calibration report.
    Calibration(Box<crate::calibration::Report>),
}
/// One-time sender used by callbacks to enqueue messages without blocking.
static SENDER: OnceLock<SyncSender<Message>> = OnceLock::new();
/// Counts messages rejected because the bounded queue was full.
static DROPPED: AtomicUsize = AtomicUsize::new(0);
static LIVE_LOGS: OnceLock<std::sync::Mutex<kirby_assist_core::diagnostic_http::LogBuffer>> = OnceLock::new();

fn live_output(text: &str) {
    let logs = LIVE_LOGS.get_or_init(|| std::sync::Mutex::new(Default::default()));
    logs.lock().unwrap_or_else(|e| e.into_inner()).push(text);
}

pub(crate) fn live_logs(after: u64) -> String {
    let logs = LIVE_LOGS.get_or_init(|| std::sync::Mutex::new(Default::default()));
    logs.lock().unwrap_or_else(|e| e.into_inner()).logs_json(after)
}

// Generation tags prevent a queued report from an old match repopulating the UI.
/// Generation counter invalidating stale mode snapshots.
static SNAPSHOT_EPOCH: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
/// Nonblocking cached card text paired with the generation that created it.
static SNAPSHOT_CARD: std::sync::Mutex<Option<(usize, std::ffi::CString)>> =
    std::sync::Mutex::new(None);
/// Advances the cache generation and clears any currently cached card.
pub fn clear_snapshot() {
    SNAPSHOT_EPOCH.fetch_add(1, Ordering::AcqRel);
    #[cfg(test)]
    if let Ok(mut card) = SNAPSHOT_CARD.try_lock() {
        *card = None;
    }
}
#[cfg(test)]
/// Publishes text only if its generation remains current across locking.
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
#[cfg(test)]
/// Calls `draw` only while a nonblocking lock holds a current cached card.
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
/// Creates diagnostic files, starts the bounded channel, and launches its writer.
pub fn start() {
    if SENDER.get().is_some() {
        return;
    }
    live_output(concat!("Kirby Assist diagnostic build ", env!("CARGO_PKG_VERSION"), "; bounded live logging; LAN disabled until Training editor activation"));
    if let Err(error) = std::fs::create_dir_all("sd:/ultimate/kirby_assist") {
        eprintln!("[kirby_assist] diagnostic log directory failed: {error}; expected {LOG_PATH}");
        live_output(&format!("LOG DIRECTORY ERROR: {error}; expected {LOG_PATH}"));
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
            live_output(&format!("LOG STARTUP ERROR: {error}; expected {path}"));
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

/// Copies borrowed text and queues it for asynchronous output.
pub fn write(message: &str) {
    write_owned(message.to_owned());
}

/// Enqueues owned text if the background logger has started.
pub fn write_owned(message: String) {
    if let Some(sender) = SENDER.get() {
        enqueue_message(sender, &DROPPED, Message::Text(message));
    }
}

/// Transfers a completed grab report into the bounded writer queue.
pub fn write_grab(report: crate::grab_diagnostics::Report) {
    if let Some(sender) = SENDER.get() {
        enqueue_message(sender, &DROPPED, Message::Grab(Box::new(report)));
    }
}
/// Transfers a completed calibration report into the bounded writer queue.
pub fn write_calibration(report: crate::calibration::Report) {
    if let Some(sender) = SENDER.get() {
        enqueue_message(sender, &DROPPED, Message::Calibration(Box::new(report)));
    }
}

/// Queues a mode report tagged for generation-safe card publication.
pub fn write_modes(report: kirby_assist_core::diagnostics::modes::Report) {
    if let Some(sender) = SENDER.get() {
        enqueue_message(
            sender,
            &DROPPED,
            Message::Modes(SNAPSHOT_EPOCH.load(Ordering::Acquire), Box::new(report)),
        );
    }
}
/// Queues an owned configuration snapshot for asynchronous formatting.
pub fn write_configuration(config: kirby_assist_core::diagnostics::modes::Configuration) {
    if let Some(sender) = SENDER.get() {
        enqueue_message(sender, &DROPPED, Message::Configuration(Box::new(config)));
    }
}

/// Attempts a nonblocking send and accounts for rejected queue entries.
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
/// Test-only generic variant for verifying bounded queue semantics.
fn enqueue<T>(sender: &SyncSender<T>, dropped: &AtomicUsize, message: T) {
    if let Err(TrySendError::Full(_)) = sender.try_send(message) {
        dropped.fetch_add(1, Ordering::Relaxed);
    }
}
/// Opens an append-only buffered log and degrades to console on failure.
fn open_log(path: &str) -> Option<BufWriter<std::fs::File>> {
    match OpenOptions::new().create(true).append(true).open(path) {
        Ok(file) => Some(BufWriter::with_capacity(16 * 1024, file)),
        Err(error) => {
            eprintln!(
                "[kirby_assist] diagnostic log open failed: {error}; console only; expected {path}"
            );
            live_output(&format!("LOG OPEN ERROR: {error}; expected {path}"));
            None
        }
    }
}
/// Flushes a file buffer and disables that file sink if flushing fails.
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
        live_output(&format!("LOG FLUSH ERROR: {error}; SD log may be incomplete: {path}"));
        *file = None;
    }
}

/// Drains the bounded queue, formats reports, samples performance,
/// and periodically flushes output sinks.
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

/// Dispatches each typed queue message to formatting and output sinks.
fn output_message(
    file: &mut Option<BufWriter<std::fs::File>>,
    console: &mut BufWriter<std::io::Stdout>,
    message: Message,
) {
    match message {
        Message::Text(message) => output(file, console, &message),
        Message::Modes(epoch, report) => {
            #[cfg(test)]
            publish_snapshot(epoch, report.card_text());
            #[cfg(not(test))]
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

/// Writes formatted performance data to the dedicated performance log.
fn output_performance(
    file: &mut Option<BufWriter<std::fs::File>>,
    console: &mut BufWriter<std::io::Stdout>,
    report: &str,
) {
    output_to(file, console, report, PERFORMANCE_PATH);
}

/// Writes ordinary diagnostic text to the status log.
fn output(
    file: &mut Option<BufWriter<std::fs::File>>,
    console: &mut BufWriter<std::io::Stdout>,
    message: &str,
) {
    output_to(file, console, message, LOG_PATH);
}
/// Writes to console and optional file, retiring a failed file sink.
fn output_to(
    file: &mut Option<BufWriter<std::fs::File>>,
    console: &mut BufWriter<std::io::Stdout>,
    message: &str,
    path: &str,
) {
    live_output(message);
    // BufWriter owns a stdout handle, not its lock. No global console lock is
    // held while waiting for another queue message. Batch flushes run here only.
    let _ = writeln!(console, "[kirby_assist] {message}");
    if let Some(error) = file
        .as_mut()
        .and_then(|file| writeln!(file, "{message}").err())
    {
        let _ = writeln!(console, "[kirby_assist] LOG FILE ERROR: {error}. Continuing console logging; SD log may be incomplete: {path}");
        live_output(&format!("LOG FILE ERROR: {error}; SD log may be incomplete: {path}"));
        *file = None;
    }
}

#[cfg(test)]
/// Logging queue, report-routing, and stale-snapshot regression tests.
#[path = "../../tests/unit/diagnostics/logging.rs"]
mod tests;
