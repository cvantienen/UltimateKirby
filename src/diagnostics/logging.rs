//! Bounded Training-session diagnostics retained only in RAM for the live HTTP view.
//! Hooks use a nonblocking queue; report formatting stays on the lazy worker.
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc::{self, Receiver, SyncSender, TrySendError},
    Mutex, OnceLock,
};
use std::time::Duration;

const QUEUE_CAPACITY: usize = 128;
const REPORT_INTERVAL: Duration = Duration::from_millis(250);
// Odd generations are active. Every transition invalidates in-flight work.
static SESSION: AtomicUsize = AtomicUsize::new(0);
static SENDER: OnceLock<SyncSender<Envelope>> = OnceLock::new();
static STARTED: AtomicBool = AtomicBool::new(false);
static LIVE_LOGS: OnceLock<Mutex<LiveLogs>> = OnceLock::new();
static SNAPSHOT_EPOCH: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
static SNAPSHOT_CARD: Mutex<Option<(usize, usize, std::ffi::CString)>> = Mutex::new(None);
#[cfg(test)]
pub(crate) static TEST_LOCK: Mutex<()> = Mutex::new(());

#[derive(Default)]
struct LiveLogs {
    session: usize,
    buffer: kirby_assist_core::diagnostic_http::LogBuffer,
}

struct Envelope {
    session: usize,
    message: Message,
}

enum Message {
    Wake,
    Text(String),
    Modes(usize, Box<kirby_assist_core::diagnostics::modes::Report>),
    Configuration(Box<kirby_assist_core::diagnostics::modes::Configuration>),
    Grab(Box<crate::grab_diagnostics::Report>),
    Calibration(Box<crate::calibration::Report>),
}

pub fn active() -> bool {
    session() != 0
}

/// Zero is inactive; nonzero tokens identify one uninterrupted Training session.
pub fn session() -> u64 {
    let generation = SESSION.load(Ordering::Acquire);
    if generation & 1 != 0 {
        generation as u64
    } else {
        0
    }
}

fn current(session: usize) -> bool {
    session & 1 != 0 && SESSION.load(Ordering::Acquire) == session
}

/// Called on every frame, including inactive frames, before diagnostic producers.
pub fn observe(training: bool) {
    let previous = SESSION.load(Ordering::Acquire);
    if (previous & 1 != 0) == training {
        return;
    }
    if SESSION
        .compare_exchange(
            previous,
            previous.wrapping_add(1),
            Ordering::AcqRel,
            Ordering::Acquire,
        )
        .is_err()
    {
        return;
    }
    clear_snapshot();
    crate::calibration::clear();
    crate::performance::observe(training);
    if let Some(logs) = LIVE_LOGS.get() {
        if let Ok(mut logs) = logs.try_lock() {
            *logs = LiveLogs::default();
        }
    }
    if training {
        start();
    }
    enqueue(|| Message::Wake);
}

/// Never creates a worker or emits startup output outside Training.
pub fn start() {
    if !active() || STARTED.swap(true, Ordering::AcqRel) {
        return;
    }
    let (sender, receiver) = mpsc::sync_channel(QUEUE_CAPACITY);
    if std::thread::Builder::new()
        .name("kirby-assist-log".into())
        .spawn(move || writer_loop(receiver))
        .is_ok()
    {
        let _ = SENDER.set(sender);
    }
}

fn live_output(session: usize, text: &str) {
    if !current(session) {
        return;
    }
    let logs = LIVE_LOGS.get_or_init(|| Mutex::new(LiveLogs::default()));
    let mut logs = logs.lock().unwrap_or_else(|e| e.into_inner());
    if !current(session) {
        return;
    }
    if logs.session != session {
        *logs = LiveLogs {
            session,
            ..Default::default()
        };
    }
    logs.buffer.push(text);
}

pub(crate) fn live_logs(after: u64) -> String {
    let session = SESSION.load(Ordering::Acquire);
    let empty = || kirby_assist_core::diagnostic_http::LogBuffer::default().logs_json(after);
    if !current(session) {
        return empty();
    }
    let Some(logs) = LIVE_LOGS.get() else {
        return empty();
    };
    let logs = logs.lock().unwrap_or_else(|e| e.into_inner());
    if logs.session != session || !current(session) {
        return empty();
    }
    let json = logs.buffer.logs_json(after);
    if current(session) {
        json
    } else {
        empty()
    }
}

pub fn clear_snapshot() {
    SNAPSHOT_EPOCH.fetch_add(1, Ordering::AcqRel);
    #[cfg(test)]
    if let Ok(mut card) = SNAPSHOT_CARD.try_lock() {
        *card = None;
    }
}

#[cfg(test)]
fn publish_snapshot(session: usize, epoch: usize, text: String) {
    if !current(session) || epoch != SNAPSHOT_EPOCH.load(Ordering::Acquire) {
        return;
    }
    if let Ok(text) = std::ffi::CString::new(text) {
        if let Ok(mut card) = SNAPSHOT_CARD.lock() {
            if current(session) && epoch == SNAPSHOT_EPOCH.load(Ordering::Acquire) {
                *card = Some((session, epoch, text));
            }
        }
    }
}

#[cfg(test)]
pub fn with_snapshot(draw: impl FnOnce(&std::ffi::CStr)) -> bool {
    if let Ok(card) = SNAPSHOT_CARD.try_lock() {
        if let Some((session, epoch, text)) = card.as_ref() {
            if current(*session) && *epoch == SNAPSHOT_EPOCH.load(Ordering::Acquire) {
                draw(text);
                return true;
            }
        }
    }
    false
}

fn bounded_text(message: &str) -> &str {
    let mut end = message
        .len()
        .min(kirby_assist_core::diagnostic_http::MAX_LOG_EVENT_BYTES);
    while !message.is_char_boundary(end) {
        end -= 1;
    }
    &message[..end]
}

pub fn write(message: &str) {
    enqueue(|| Message::Text(bounded_text(message).to_owned()));
}

pub fn write_owned(message: String) {
    write_owned_for_session(session(), message);
}

pub fn write_owned_for_session(session: u64, mut message: String) {
    enqueue_for_session(session, || {
        message.truncate(bounded_text(&message).len());
        // Do not retain the caller's potentially unbounded allocation in the queue.
        Message::Text(message.into_boxed_str().into_string())
    });
}

pub fn write_grab(report: crate::grab_diagnostics::Report) {
    enqueue(|| Message::Grab(Box::new(report)));
}

pub fn write_calibration(report: crate::calibration::Report) {
    enqueue(|| Message::Calibration(Box::new(report)));
}

pub fn write_modes(report: kirby_assist_core::diagnostics::modes::Report) {
    write_modes_for_session(session(), report);
}

pub fn write_modes_for_session(
    session: u64,
    report: kirby_assist_core::diagnostics::modes::Report,
) {
    enqueue_for_session(session, || {
        Message::Modes(SNAPSHOT_EPOCH.load(Ordering::Acquire), Box::new(report))
    });
}

pub fn write_configuration(config: kirby_assist_core::diagnostics::modes::Configuration) {
    enqueue(|| Message::Configuration(Box::new(config)));
}

fn enqueue(message: impl FnOnce() -> Message) {
    enqueue_for_session(session(), message);
}

fn enqueue_for_session(session: u64, message: impl FnOnce() -> Message) {
    let Ok(session) = usize::try_from(session) else {
        return;
    };
    if !current(session) {
        return;
    }
    if let Some(sender) = SENDER.get() {
        enqueue_message(
            sender,
            Envelope {
                session,
                message: message(),
            },
        );
    }
}

fn enqueue_message(sender: &SyncSender<Envelope>, message: Envelope) {
    if !current(message.session) {
        return;
    }
    crate::performance::health::queue_enter();
    if let Err(error) = sender.try_send(message) {
        crate::performance::health::queue_leave();
        if matches!(error, TrySendError::Full(_)) {
            crate::performance::health::event(crate::performance::health::Event::LogDropped);
        }
    }
}

fn writer_loop(receiver: Receiver<Envelope>) {
    loop {
        let received = if active() {
            receiver.recv_timeout(REPORT_INTERVAL)
        } else {
            receiver
                .recv()
                .map_err(|_| mpsc::RecvTimeoutError::Disconnected)
        };
        match received {
            Ok(message) => {
                crate::performance::health::queue_leave();
                output_message(message);
                for message in receiver.try_iter().take(63) {
                    crate::performance::health::queue_leave();
                    output_message(message);
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        let session = SESSION.load(Ordering::Acquire);
        if current(session) {
            if let Some(report) = crate::performance::report() {
                live_output(session, &report);
            }
        } else if let Some(logs) = LIVE_LOGS.get() {
            if let Ok(mut logs) = logs.try_lock() {
                if !active() {
                    *logs = LiveLogs::default();
                }
            }
        }
    }
}

fn output_message(envelope: Envelope) {
    let session = envelope.session;
    if !current(session) {
        return;
    }
    let _work = crate::performance::health::Trace::new(crate::performance::health::Lane::LogWork);
    let text = match envelope.message {
        Message::Wake => return,
        Message::Text(text) => text,
        Message::Modes(epoch, report) => {
            #[cfg(test)]
            publish_snapshot(session, epoch, report.card_text());
            #[cfg(not(test))]
            let _ = epoch;
            report.format()
        }
        Message::Configuration(config) => config.format(),
        Message::Grab(report) => report.format(),
        Message::Calibration(report) => {
            if current(session) {
                crate::calibration::publish(&report);
            }
            report.format()
        }
    };
    live_output(session, &text);
}

#[cfg(test)]
#[path = "../../tests/unit/diagnostics/logging.rs"]
mod tests;
