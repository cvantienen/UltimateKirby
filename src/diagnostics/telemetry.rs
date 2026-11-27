//! Session-only, read-only telemetry transport. Native pointers never enter this module.
#[allow(dead_code)]
#[path = "network.rs"]
mod network;
use kirby_assist_core::telemetry::{self, Config, Sample};
use network::socket_result;
use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Mutex,
    },
    thread::JoinHandle,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

struct Subscriber {
    peer: SocketAddrV4,
    renewed: Instant,
}
fn subscribe(subscriber: &mut Option<Subscriber>, bytes: &[u8], peer: SocketAddr, now: Instant) {
    if bytes != telemetry::SUBSCRIPTION {
        return;
    }
    if let SocketAddr::V4(peer) = peer {
        if telemetry::private_peer(peer) {
            *subscriber = Some(Subscriber { peer, renewed: now });
        }
    }
}
fn destination(subscriber: &mut Option<Subscriber>, now: Instant) -> Option<SocketAddrV4> {
    if subscriber
        .as_ref()
        .is_some_and(|s| now.duration_since(s.renewed) >= telemetry::SUBSCRIBER_LEASE)
    {
        *subscriber = None;
    }
    subscriber.as_ref().map(|s| s.peer)
}
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
// A renewed observation cannot authorize an older activation or queued sample.
static GENERATION: AtomicU64 = AtomicU64::new(0);
static ACTIVATION: AtomicU64 = AtomicU64::new(0);
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
    generation: 0,
});
struct Mailbox {
    values: [Sample; CAPACITY],
    read: usize,
    len: usize,
    sequence: u64,
    seen: Option<Instant>,
    started: Option<Instant>,
    generation: u64,
}
impl Mailbox {
    fn authorized(&self, generation: u64) -> bool {
        self.generation == generation
            && self.seen.is_some_and(|t| t.elapsed() < LEASE)
            && generation == GENERATION.load(Ordering::Acquire)
    }
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
        && ACTIVATION.load(Ordering::Acquire) == GENERATION.load(Ordering::Acquire)
}
pub(crate) fn ready() -> bool {
    enabled()
        && READY.load(Ordering::Acquire)
        && MAILBOX
            .try_lock()
            .ok()
            .is_some_and(|mailbox| mailbox.authorized(ACTIVATION.load(Ordering::Acquire)))
}
pub(crate) fn status() -> Option<String> {
    STATUS.try_lock().ok().and_then(|s| s.clone())
}
pub(crate) fn session() -> u64 {
    SESSION.load(Ordering::Acquire)
}
pub(crate) fn revoke() {
    GENERATION.fetch_add(1, Ordering::AcqRel);
    ENABLED.store(false, Ordering::Release);
    READY.store(false, Ordering::Release);
    if let Ok(mut mailbox) = MAILBOX.try_lock() {
        mailbox.seen = None;
        mailbox.len = 0;
    }
}
pub(crate) fn observe(training: bool) {
    if !training {
        revoke();
        return;
    }
    let generation = GENERATION.load(Ordering::Acquire);
    if let Ok(mut mailbox) = MAILBOX.try_lock() {
        let generation = if mailbox.generation == generation
            && mailbox.seen.is_some_and(|t| t.elapsed() >= LEASE)
        {
            match GENERATION.compare_exchange(
                generation,
                generation.wrapping_add(1),
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => generation.wrapping_add(1),
                Err(_) => return,
            }
        } else {
            generation
        };
        if mailbox.generation != generation {
            mailbox.len = 0;
        }
        mailbox.generation = generation;
        mailbox.seen = Some(Instant::now());
    }
}
fn authorized(generation: u64) -> bool {
    enabled()
        && MAILBOX
            .lock()
            .ok()
            .is_some_and(|mailbox| mailbox.authorized(generation))
}
pub(crate) fn publish(mut sample: Sample) {
    let generation = ACTIVATION.load(Ordering::Acquire);
    if !ready() {
        return;
    }
    if let Ok(mut mailbox) = MAILBOX.try_lock() {
        if !enabled() || !READY.load(Ordering::Acquire) || !mailbox.authorized(generation) {
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
    let generation = GENERATION.load(Ordering::Acquire);
    if !mailbox.authorized(generation) {
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
    ACTIVATION.store(generation, Ordering::Release);
    ENABLED.store(true, Ordering::Release);
    if !mailbox.authorized(generation) || !enabled() {
        ENABLED.store(false, Ordering::Release);
        return Err("Activate telemetry in Training only".into());
    }
    drop(mailbox);
    match std::thread::Builder::new()
        .name("kirby-telemetry".into())
        .stack_size(256 * 1024)
        .spawn(move || run(generation))
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
fn run(generation: u64) {
    let result = serve(generation);
    revoke();
    if let Err(error) = result {
        if let Ok(mut status) = STATUS.lock() {
            *status = Some(error);
        }
    }
}
fn serve(generation: u64) -> Result<(), String> {
    if !authorized(generation) {
        return Ok(());
    }
    let socket = socket_result(UdpSocket::bind((
        Ipv4Addr::UNSPECIFIED,
        telemetry::TELEMETRY_PORT,
    )))
    .map_err(|e| format!("Telemetry bind: {e}"))?;
    serve_socket(socket, None, generation)
}
#[cfg(test)]
fn serve_config(config: Config) -> Result<(), String> {
    let generation = ACTIVATION.load(Ordering::Acquire);
    if !authorized(generation) {
        return Ok(());
    }
    let socket = socket_result(UdpSocket::bind(config.switch))
        .map_err(|e| format!("Telemetry bind: {e}"))?;
    serve_socket(socket, Some(config), generation)
}
fn serve_socket(socket: UdpSocket, config: Option<Config>, generation: u64) -> Result<(), String> {
    if !authorized(generation) {
        return Ok(());
    }
    socket_result(socket.set_nonblocking(true))
        .map_err(|e| format!("Telemetry nonblocking setup: {e}"))?;
    let mut subscriber = None;
    let mut subscription = [0; 64];
    let mut bytes = [0; 1200];
    while authorized(generation) {
        // Bound receive work even under LAN flooding; oversized/truncated requests cannot match.
        for _ in 0..32 {
            if !authorized(generation) {
                return Ok(());
            }
            match socket_result(socket.recv_from(&mut subscription)) {
                Ok((n, peer)) => {
                    if !authorized(generation) {
                        return Ok(());
                    }
                    subscribe(&mut subscriber, &subscription[..n], peer, Instant::now())
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(format!("Telemetry subscription: {e}")),
            }
        }
        let peer = config
            .as_ref()
            .map(|c| c.computer)
            .or_else(|| destination(&mut subscriber, Instant::now()));
        READY.store(peer.is_some(), Ordering::Release);
        let mut mailbox = MAILBOX
            .lock()
            .map_err(|_| "Telemetry mailbox unavailable")?;
        if !enabled() || !mailbox.authorized(generation) {
            break;
        }
        let sample = if peer.is_some() {
            mailbox.pop()
        } else {
            mailbox.len = 0;
            None
        };
        drop(mailbox);
        if let (Some(sample), Some(peer)) = (sample, peer) {
            let key = config.as_ref().map_or(&[][..], |c| c.token.as_bytes());
            let n = telemetry::encode(&sample, key, &mut bytes).map_err(str::to_owned)?;
            if !authorized(generation) || sample.session != session() {
                break;
            }
            match socket_result(socket.send_to(&bytes[..n], peer)) {
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
    fn subscription_is_exact_private_and_expires_without_renewal() {
        let now = Instant::now();
        let mut subscriber = None;
        let peer: SocketAddr = "192.168.1.20:54321".parse().unwrap();
        for invalid in [
            "127.0.0.1:1234",
            "8.8.8.8:1234",
            "192.168.1.255:1234",
            "192.168.1.0:1234",
            "[::1]:1234",
        ] {
            subscribe(
                &mut subscriber,
                telemetry::SUBSCRIPTION,
                invalid.parse().unwrap(),
                now,
            );
            assert!(subscriber.is_none());
        }
        for invalid in [
            b"KATM SUBSCRIBE 1\n".as_slice(),
            b"KATM SUBSCRIBE 2",
            b"KATM SUBSCRIBE",
        ] {
            subscribe(&mut subscriber, invalid, peer, now);
            assert!(subscriber.is_none());
        }
        subscribe(&mut subscriber, telemetry::SUBSCRIPTION, peer, now);
        assert_eq!(
            destination(&mut subscriber, now),
            Some("192.168.1.20:54321".parse().unwrap())
        );
        subscribe(
            &mut subscriber,
            telemetry::SUBSCRIPTION,
            peer,
            now + Duration::from_secs(1),
        );
        let renewed_peer = Some("192.168.1.20:54321".parse().unwrap());
        assert_eq!(
            destination(&mut subscriber, now + telemetry::SUBSCRIBER_LEASE),
            renewed_peer
        );
        for invalid in ["8.8.8.8:54321", "127.0.0.1:54321", "[::1]:54321"] {
            subscribe(
                &mut subscriber,
                telemetry::SUBSCRIPTION,
                invalid.parse().unwrap(),
                now + telemetry::SUBSCRIBER_LEASE,
            );
            assert_eq!(
                destination(&mut subscriber, now + telemetry::SUBSCRIBER_LEASE),
                renewed_peer
            );
        }
        subscribe(
            &mut subscriber,
            b"KATM SUBSCRIBE 1 redirect=192.168.1.30:4243",
            peer,
            now + telemetry::SUBSCRIBER_LEASE,
        );
        assert_eq!(
            destination(&mut subscriber, now + telemetry::SUBSCRIBER_LEASE),
            renewed_peer
        );
        assert!(destination(
            &mut subscriber,
            now + telemetry::SUBSCRIBER_LEASE + Duration::from_secs(1)
        )
        .is_none());
    }
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
            generation: 0,
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
    fn revocation_discards_the_queue_and_requires_fresh_training_observation() {
        let _guard = TEST_GATE.lock().unwrap();
        {
            let mut mailbox = MAILBOX.lock().unwrap();
            mailbox.seen = Some(Instant::now());
            mailbox.len = 1;
        }
        ENABLED.store(true, Ordering::Release);
        READY.store(true, Ordering::Release);
        revoke();
        let mailbox = MAILBOX.lock().unwrap();
        assert!(mailbox.seen.is_none());
        assert_eq!(mailbox.len, 0);
        assert!(!ready());
    }
    #[test]
    fn contended_revocation_cannot_reuse_the_old_training_lease() {
        let _guard = TEST_GATE.lock().unwrap();
        observe(true);
        {
            let _mailbox = MAILBOX.lock().unwrap();
            revoke();
        }
        assert_eq!(toggle(), Err("Activate telemetry in Training only".into()));
        assert!(!enabled());
        observe(true);
        assert!(MAILBOX
            .lock()
            .unwrap()
            .authorized(GENERATION.load(Ordering::Acquire)));
        revoke();
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
    fn expired_or_revoked_startup_does_not_bind_or_reuse_queued_samples() {
        let _guard = TEST_GATE.lock().unwrap();
        revoke();
        observe(true);
        let old = GENERATION.load(Ordering::Acquire);
        ACTIVATION.store(old, Ordering::Release);
        ENABLED.store(true, Ordering::Release);
        {
            let mut mailbox = MAILBOX.lock().unwrap();
            mailbox.seen = Some(Instant::now() - LEASE);
            mailbox.push(EMPTY);
        }
        let occupied = UdpSocket::bind("127.0.0.1:0").unwrap();
        let SocketAddr::V4(address) = occupied.local_addr().unwrap() else {
            unreachable!()
        };
        let config = || Config {
            switch: address,
            computer: address,
            token: "abcdefghijklmnopqrstuvwxyz012345".into(),
        };
        assert!(serve_config(config()).is_ok());
        observe(true);
        assert!(!enabled());
        assert!(!authorized(old));
        assert_eq!(MAILBOX.lock().unwrap().len, 0);
        ACTIVATION.store(GENERATION.load(Ordering::Acquire), Ordering::Release);
        ENABLED.store(true, Ordering::Release);
        {
            let mut mailbox = MAILBOX.lock().unwrap();
            mailbox.push(EMPTY);
            revoke();
        }
        assert!(serve_config(config()).is_ok());
        observe(true);
        assert_eq!(MAILBOX.lock().unwrap().len, 0);
        assert!(!enabled());
        revoke();
    }

    #[test]
    fn old_worker_cannot_send_queued_data_after_reactivation() {
        let _guard = TEST_GATE.lock().unwrap();
        revoke();
        observe(true);
        let old = GENERATION.load(Ordering::Acquire);
        revoke();
        observe(true);
        ACTIVATION.store(GENERATION.load(Ordering::Acquire), Ordering::Release);
        ENABLED.store(true, Ordering::Release);
        SESSION.store(99, Ordering::Release);
        MAILBOX.lock().unwrap().push(EMPTY);
        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
        receiver.set_nonblocking(true).unwrap();
        let SocketAddr::V4(peer) = receiver.local_addr().unwrap() else {
            unreachable!()
        };
        let SocketAddr::V4(local) = sender.local_addr().unwrap() else {
            unreachable!()
        };
        assert!(serve_socket(
            sender,
            Some(Config {
                switch: local,
                computer: peer,
                token: "abcdefghijklmnopqrstuvwxyz012345".into()
            }),
            old
        )
        .is_ok());
        assert_eq!(
            receiver.recv(&mut [0; 1200]).unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
        assert_eq!(MAILBOX.lock().unwrap().len, 1);
        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        assert!(serve_socket(
            sender,
            Some(Config {
                switch: local,
                computer: peer,
                token: "abcdefghijklmnopqrstuvwxyz012345".into()
            }),
            GENERATION.load(Ordering::Acquire),
        )
        .is_ok());
        assert_eq!(
            receiver.recv(&mut [0; 1200]).unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
        assert_eq!(MAILBOX.lock().unwrap().len, 0);
        revoke();
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
        observe(true);
        SESSION.store(expected.session, Ordering::Release);
        ACTIVATION.store(GENERATION.load(Ordering::Acquire), Ordering::Release);
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
