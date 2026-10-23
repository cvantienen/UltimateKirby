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
const QUEUE_CAPACITY: usize = 128;
const FLUSH_INTERVAL: Duration = Duration::from_millis(250);
enum Message {
    Text(String),
    Grab(Box<crate::grab_diagnostics::Report>),
    Calibration(Box<crate::calibration::Report>),
}
static SENDER: OnceLock<SyncSender<Message>> = OnceLock::new();
static DROPPED: AtomicUsize = AtomicUsize::new(0);

/// Startup I/O happens before hooks are installed. Gameplay never opens files.
pub fn start() {
    if SENDER.get().is_some() {
        return;
    }
    if let Err(error) = std::fs::create_dir_all("sd:/ultimate/kirby_assist") {
        eprintln!("[kirby_assist] diagnostic log directory failed: {error}; expected {LOG_PATH}");
    }
    if let Err(error) = std::fs::write(
        LOG_PATH,
        concat!(
            "Kirby Assist diagnostic build ",
            env!("CARGO_PKG_VERSION"),
            "; on-device validation required; queued logging, 250ms flush\n"
        ),
    ) {
        eprintln!("[kirby_assist] diagnostic startup log failed: {error}; expected {LOG_PATH}");
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
        enqueue(sender, &DROPPED, Message::Text(message));
    }
}

pub fn write_grab(report: crate::grab_diagnostics::Report) {
    if let Some(sender) = SENDER.get() {
        enqueue(sender, &DROPPED, Message::Grab(Box::new(report)));
    }
}
pub fn write_calibration(report: crate::calibration::Report) {
    if let Some(sender) = SENDER.get() {
        enqueue(sender, &DROPPED, Message::Calibration(Box::new(report)));
    }
}

fn enqueue<T>(sender: &SyncSender<T>, dropped: &AtomicUsize, message: T) {
    // No waiting for a slow/full logger. Memory is bounded by queue capacity;
    // the worker reports dropped messages when it can make progress again.
    if let Err(TrySendError::Full(_)) = sender.try_send(message) {
        dropped.fetch_add(1, Ordering::Relaxed);
    }
}

fn writer_loop(receiver: Receiver<Message>) {
    // Keep one file open and buffer batches instead of opening/writing/closing
    // the SD file for every frame. All output failures remain nonfatal.
    let mut file = match OpenOptions::new().create(true).append(true).open(LOG_PATH) {
        Ok(file) => Some(BufWriter::with_capacity(16 * 1024, file)),
        Err(error) => {
            eprintln!("[kirby_assist] diagnostic log open failed: {error}; console only; expected {LOG_PATH}");
            None
        }
    };
    let mut console = BufWriter::with_capacity(16 * 1024, std::io::stdout());
    let mut flushed = Instant::now();
    loop {
        match receiver.recv_timeout(FLUSH_INTERVAL) {
            Ok(message) => {
                output_message(&mut file, &mut console, message);
                for message in receiver.try_iter().take(63) {
                    output_message(&mut file, &mut console, message);
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                if let Some(file) = file.as_mut() {
                    let _ = file.flush();
                }
                let _ = console.flush();
                break;
            }
        }
        let dropped = DROPPED.swap(0, Ordering::Relaxed);
        if dropped != 0 {
            output(
                &mut file,
                &mut console,
                &format!("diagnostic queue: dropped {dropped} messages; gameplay did not wait"),
            );
        }
        if flushed.elapsed() >= FLUSH_INTERVAL {
            if let Some(file) = file.as_mut() {
                let _ = file.flush();
            }
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
        Message::Grab(report) => output(file, console, &report.format()),
        Message::Calibration(report) => {
            let formatted = report.format();
            crate::calibration::publish(&report);
            output(file, console, &formatted);
        }
    }
}

fn output(
    file: &mut Option<BufWriter<std::fs::File>>,
    console: &mut BufWriter<std::io::Stdout>,
    message: &str,
) {
    // BufWriter owns a stdout handle, not its lock. No global console lock is
    // held while waiting for another queue message. Batch flushes run here only.
    let _ = writeln!(console, "[kirby_assist] {message}");
    if let Some(file) = file.as_mut() {
        let _ = writeln!(file, "{message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
