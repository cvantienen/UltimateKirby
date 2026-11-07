//! Diagnostic-only logger. Game hooks enqueue; a worker batches console/SD I/O.
//! Silent builds exclude this module and its thread/channel entirely.
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

const LOG_PATH: &str = "sd:/ultimate/kirby_assist/status.log";
const PERFORMANCE_PATH: &str = "sd:/ultimate/kirby_assist/performance.log";
const QUEUE_CAPACITY: usize = 128;
const FLUSH_INTERVAL: Duration = Duration::from_millis(250);
enum Message {
    Text(String),
    Modes(usize, Box<kirby_assist_core::diagnostics::modes::Report>),
    Configuration(Box<kirby_assist_core::diagnostics::modes::Configuration>),
    Grab(Box<crate::grab_diagnostics::Report>),
    Calibration(Box<crate::calibration::Report>),
}
static SENDER: OnceLock<SyncSender<Message>> = OnceLock::new();
static DROPPED: AtomicUsize = AtomicUsize::new(0);

// Generation tags prevent a queued report from an old match repopulating the UI.
static SNAPSHOT_EPOCH: AtomicUsize = AtomicUsize::new(0);
#[cfg(any(feature = "imgui-menu", test))]
static SNAPSHOT_CARD: std::sync::Mutex<Option<(usize, std::ffi::CString)>> =
    std::sync::Mutex::new(None);
pub fn clear_snapshot() {
    SNAPSHOT_EPOCH.fetch_add(1, Ordering::AcqRel);
    #[cfg(any(feature = "imgui-menu", test))]
    if let Ok(mut card) = SNAPSHOT_CARD.try_lock() {
        *card = None;
    }
}
#[cfg(any(feature = "imgui-menu", test))]
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

pub fn write(message: &str) {
    write_owned(message.to_owned());
}

pub fn write_owned(message: String) {
    if let Some(sender) = SENDER.get() {
        enqueue_message(sender, &DROPPED, Message::Text(message));
    }
}

pub fn write_grab(report: crate::grab_diagnostics::Report) {
    if let Some(sender) = SENDER.get() {
        enqueue_message(sender, &DROPPED, Message::Grab(Box::new(report)));
    }
}
pub fn write_calibration(report: crate::calibration::Report) {
    if let Some(sender) = SENDER.get() {
        enqueue_message(sender, &DROPPED, Message::Calibration(Box::new(report)));
    }
}

pub fn write_modes(report: kirby_assist_core::diagnostics::modes::Report) {
    if let Some(sender) = SENDER.get() {
        enqueue_message(
            sender,
            &DROPPED,
            Message::Modes(SNAPSHOT_EPOCH.load(Ordering::Acquire), Box::new(report)),
        );
    }
}
pub fn write_configuration(config: kirby_assist_core::diagnostics::modes::Configuration) {
    if let Some(sender) = SENDER.get() {
        enqueue_message(sender, &DROPPED, Message::Configuration(Box::new(config)));
    }
}

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
fn enqueue<T>(sender: &SyncSender<T>, dropped: &AtomicUsize, message: T) {
    if let Err(TrySendError::Full(_)) = sender.try_send(message) {
        dropped.fetch_add(1, Ordering::Relaxed);
    }
}
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

fn output_performance(
    file: &mut Option<BufWriter<std::fs::File>>,
    console: &mut BufWriter<std::io::Stdout>,
    report: &str,
) {
    output_to(file, console, report, PERFORMANCE_PATH);
}

fn output(
    file: &mut Option<BufWriter<std::fs::File>>,
    console: &mut BufWriter<std::io::Stdout>,
    message: &str,
) {
    output_to(file, console, message, LOG_PATH);
}
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
mod tests {
    use super::*;
    #[test]
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
    fn disconnected_logger_does_not_panic_or_block_gameplay() {
        let (sender, receiver) = mpsc::sync_channel::<String>(1);
        drop(receiver);
        enqueue(&sender, &AtomicUsize::new(0), "event".into());
    }
}
