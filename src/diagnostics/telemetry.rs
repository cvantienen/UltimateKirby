//! Session-only, read-only telemetry transport. Native pointers never enter this module.
use kirby_assist_core::telemetry::{self, Config, Sample};
use std::{
    io::Read,
    net::UdpSocket,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Mutex,
    },
    thread::JoinHandle,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const CONFIG_PATH: &str = "sd:/ultimate/kirby_assist/telemetry.txt";
const LEASE: Duration = Duration::from_millis(250);
const CAPACITY: usize = 512;
const EMPTY: Sample = Sample {
    session: 0,
    sequence: 0,
    sample_us: 0,
    object_id: 0,
    category: 0,
    kind: 0,
    group: 0,
    dropped: 0,
    values: [None; 64],
};
static ENABLED: AtomicBool = AtomicBool::new(false);
static READY: AtomicBool = AtomicBool::new(false);
static SESSION: AtomicU64 = AtomicU64::new(0);
static DROPPED: AtomicU64 = AtomicU64::new(0);
static WORKER: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);
static STATUS: Mutex<Option<String>> = Mutex::new(None);
static MAILBOX: Mutex<Mailbox> = Mutex::new(Mailbox {
    values: [EMPTY; CAPACITY],
    read: 0,
    len: 0,
    sequence: 0,
    seen: None,
    started: None,
});
struct Mailbox {
    values: [Sample; CAPACITY],
    read: usize,
    len: usize,
    sequence: u64,
    seen: Option<Instant>,
    started: Option<Instant>,
}
impl Mailbox {
    fn push(&mut self, mut sample: Sample) {
        self.sequence = self.sequence.wrapping_add(1);
        sample.sequence = self.sequence;
        if self.len == self.values.len() {
            self.read = (self.read + 1) % self.values.len();
            self.len -= 1;
            DROPPED.fetch_add(1, Ordering::Relaxed);
        }
        sample.dropped = DROPPED.load(Ordering::Relaxed);
        self.values[(self.read + self.len) % self.values.len()] = sample;
        self.len += 1;
    }
    fn pop(&mut self) -> Option<Sample> {
        if self.len == 0 {
            return None;
        }
        let sample = self.values[self.read];
        self.read = (self.read + 1) % self.values.len();
        self.len -= 1;
        Some(sample)
    }
}
pub(crate) fn enabled() -> bool {
    ENABLED.load(Ordering::Acquire)
}
pub(crate) fn ready() -> bool {
    enabled() && READY.load(Ordering::Acquire)
}
pub(crate) fn status() -> Option<String> {
    STATUS.try_lock().ok().and_then(|s| s.clone())
}
pub(crate) fn revoke() {
    ENABLED.store(false, Ordering::Release);
    READY.store(false, Ordering::Release);
}
pub(crate) fn observe(training: bool) {
    if !training {
        revoke();
        return;
    }
    if let Ok(mut mailbox) = MAILBOX.try_lock() {
        mailbox.seen = Some(Instant::now());
    }
}
pub(crate) fn publish(mut sample: Sample) {
    if !ready() {
        return;
    }
    if let Ok(mut mailbox) = MAILBOX.try_lock() {
        if !ready() || !mailbox.seen.is_some_and(|t| t.elapsed() < LEASE) {
            return;
        }
        sample.session = SESSION.load(Ordering::Acquire);
        sample.sample_us = mailbox
            .started
            .map_or(0, |t| t.elapsed().as_micros() as u64);
        mailbox.push(sample);
    } else {
        DROPPED.fetch_add(1, Ordering::Relaxed);
    }
}
pub(crate) fn toggle() -> Result<(), String> {
    if enabled() {
        revoke();
        return Ok(());
    }
    let mut worker = WORKER
        .try_lock()
        .map_err(|_| "Telemetry worker busy; retry")?;
    if worker.as_ref().is_some_and(|h| !h.is_finished()) {
        return Err("Telemetry worker is closing; retry".into());
    }
    if let Some(handle) = worker.take() {
        let _ = handle.join();
    }
    let mut mailbox = MAILBOX
        .try_lock()
        .map_err(|_| "Telemetry sampler busy; retry")?;
    if !mailbox.seen.is_some_and(|t| t.elapsed() < LEASE) {
        return Err("Activate telemetry in Training only".into());
    }
    mailbox.len = 0;
    mailbox.read = 0;
    mailbox.sequence = 0;
    mailbox.started = Some(Instant::now());
    let session = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "System clock unavailable")?
        .as_nanos() as u64;
    SESSION.store(session, Ordering::Release);
    DROPPED.store(0, Ordering::Relaxed);
    if let Ok(mut status) = STATUS.try_lock() {
        *status = None;
    }
    ENABLED.store(true, Ordering::Release);
    match std::thread::Builder::new()
        .name("kirby-telemetry".into())
        .stack_size(256 * 1024)
        .spawn(run)
    {
        Ok(handle) => {
            *worker = Some(handle);
            Ok(())
        }
        Err(error) => {
            revoke();
            Err(format!("Cannot start telemetry: {error}"))
        }
    }
}
fn run() {
    let result = serve();
    revoke();
    if let Err(error) = result {
        if let Ok(mut status) = STATUS.lock() {
            *status = Some(error);
        }
    }
}
fn serve() -> Result<(), String> {
    let mut source = String::new();
    std::fs::File::open(CONFIG_PATH)
        .map_err(|_| "Cannot read telemetry.txt".to_owned())?
        .take(2049)
        .read_to_string(&mut source)
        .map_err(|_| "Invalid telemetry setup UTF-8".to_owned())?;
    if source.len() > 2048 {
        return Err("Telemetry setup exceeds 2048 bytes".into());
    }
    let config = Config::parse(&source).map_err(str::to_owned)?;
    serve_config(config)
}
fn serve_config(config: Config) -> Result<(), String> {
    if !enabled() {
        return Ok(());
    }
    let socket = UdpSocket::bind(config.switch).map_err(|e| format!("Telemetry bind: {e}"))?;
    socket
        .connect(config.computer)
        .map_err(|e| format!("Telemetry destination: {e}"))?;
    socket.set_nonblocking(true).map_err(|e| e.to_string())?;
    READY.store(true, Ordering::Release);
    let mut bytes = [0; 1200];
    while enabled() {
        let mut mailbox = MAILBOX
            .lock()
            .map_err(|_| "Telemetry mailbox unavailable")?;
        if !mailbox.seen.is_some_and(|t| t.elapsed() < LEASE) {
            break;
        }
        let sample = mailbox.pop();
        drop(mailbox);
        if let Some(sample) = sample {
            let n = telemetry::encode(&sample, config.token.as_bytes(), &mut bytes)
                .map_err(str::to_owned)?;
            if !ready() {
                break;
            }
            match socket.send(&bytes[..n]) {
                Ok(sent) if sent == n => {}
                Ok(_) => {
                    DROPPED.fetch_add(1, Ordering::Relaxed);
                }
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::ConnectionRefused
                    ) =>
                {
                    DROPPED.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => return Err(format!("Telemetry send: {e}")),
            }
        } else {
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    static TEST_GATE: Mutex<()> = Mutex::new(());
    #[test]
    fn bounded_mailbox_retains_newest_and_preserves_sequence() {
        let _guard = TEST_GATE.lock().unwrap();
        let mut mailbox = Mailbox {
            values: [EMPTY; CAPACITY],
            read: 0,
            len: 0,
            sequence: 0,
            seen: None,
            started: None,
        };
        for id in 0..CAPACITY as u32 + 6 {
            let mut sample = EMPTY;
            sample.object_id = id;
            mailbox.push(sample);
        }
        assert_eq!(mailbox.len, CAPACITY);
        let first = mailbox.pop().unwrap();
        assert_eq!((first.object_id, first.sequence), (6, 7));
        let mut last = first;
        while let Some(sample) = mailbox.pop() {
            last = sample;
        }
        assert_eq!(
            (last.object_id, last.sequence),
            (CAPACITY as u32 + 5, CAPACITY as u64 + 6)
        );
    }
    #[test]
    fn expired_training_lease_does_not_publish() {
        let _guard = TEST_GATE.lock().unwrap();
        revoke();
        let before = MAILBOX.lock().unwrap().len;
        publish(EMPTY);
        assert_eq!(MAILBOX.lock().unwrap().len, before);
        ENABLED.store(true, Ordering::Release);
        READY.store(true, Ordering::Release);
        MAILBOX.lock().unwrap().seen = Some(Instant::now() - LEASE);
        publish(EMPTY);
        assert_eq!(MAILBOX.lock().unwrap().len, before);
        observe(false);
        assert!(!ready());
    }
    #[test]
    fn worker_sends_authenticated_snapshot_and_stops_when_training_is_revoked() {
        let _guard = TEST_GATE.lock().unwrap();
        let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
        receiver
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        let local = match sender.local_addr().unwrap() {
            std::net::SocketAddr::V4(a) => a,
            _ => unreachable!(),
        };
        drop(sender);
        let peer = match receiver.local_addr().unwrap() {
            std::net::SocketAddr::V4(a) => a,
            _ => unreachable!(),
        };
        let key = "abcdefghijklmnopqrstuvwxyz012345";
        let config = Config {
            switch: local,
            computer: peer,
            token: key.into(),
        };
        let mut expected = EMPTY;
        expected.session = 1;
        expected.sequence = 1;
        expected.values[7] = Some(u64::from(2.5f32.to_bits()));
        {
            let mut mailbox = MAILBOX.lock().unwrap();
            mailbox.len = 0;
            mailbox.read = 0;
            mailbox.seen = Some(Instant::now());
            mailbox.values[0] = expected;
            mailbox.len = 1;
        }
        ENABLED.store(true, Ordering::Release);
        let worker = std::thread::spawn(move || serve_config(config));
        let mut bytes = [0; 1200];
        let (n, source) = receiver.recv_from(&mut bytes).unwrap();
        assert_eq!(source, std::net::SocketAddr::V4(local));
        assert_eq!(
            telemetry::decode(&bytes[..n], key.as_bytes()).unwrap(),
            expected
        );
        observe(false);
        assert!(worker.join().unwrap().is_ok());
        assert!(!ready());
        MAILBOX.lock().unwrap().len = 0;
    }
}
