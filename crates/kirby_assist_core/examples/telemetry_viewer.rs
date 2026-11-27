use kirby_assist_core::telemetry::{
    decode, fields, private_peer, Sample, ValueKind, FIELD_COUNT, SUBSCRIPTION, TELEMETRY_PORT,
};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const STALE: Duration = Duration::from_millis(250);
const EXPIRY: Duration = Duration::from_secs(1);
const MAX_GROUPS: usize = 512;
struct Observation {
    sample: Sample,
    received: Instant,
}
struct State {
    objects: HashMap<(u32, u16), Observation>,
    session: Option<u64>,
    target: Option<(std::net::SocketAddrV4, String)>,
    generation: u64,
    retired: HashSet<u64>,
    last_sequence: Option<u64>,
    received: Option<Instant>,
    started: Instant,
    packets: u64,
    bytes: u64,
    gaps: u64,
    reordered: u64,
    rejected: u64,
    wrong_source: u64,
    session_changes: u64,
    sender_dropped: u64,
    rate_started: Instant,
    rate_bytes: u64,
    rate_packets: u64,
    bytes_per_second: f64,
    packets_per_second: f64,
}
impl State {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            objects: HashMap::new(),
            session: None,
            target: None,
            generation: 0,
            retired: HashSet::new(),
            last_sequence: None,
            received: None,
            started: now,
            packets: 0,
            bytes: 0,
            gaps: 0,
            reordered: 0,
            rejected: 0,
            wrong_source: 0,
            session_changes: 0,
            sender_dropped: 0,
            rate_started: now,
            rate_bytes: 0,
            rate_packets: 0,
            bytes_per_second: 0.0,
            packets_per_second: 0.0,
        }
    }
    fn configure(&mut self, route: &str) -> Result<(), &'static str> {
        let endpoint = route
            .strip_prefix("/connect?endpoint=")
            .ok_or("invalid setup")?;
        let peer: std::net::SocketAddrV4 = endpoint
            .parse()
            .map_err(|_| "Enter numeric private Switch IPv4 and port")?;
        if !private_peer(peer) {
            return Err("Expected private unicast Switch IPv4 and nonzero port");
        }
        let generation = self.generation.wrapping_add(1);
        *self = Self::new();
        self.generation = generation;
        self.target = Some((peer, String::new()));
        Ok(())
    }
    fn renew(
        &self,
        udp: &UdpSocket,
        renewed: &mut Option<Instant>,
        now: Instant,
    ) -> io::Result<()> {
        if let Some((peer, key)) = self.target.as_ref() {
            if key.is_empty()
                && renewed.is_none_or(|t| now.duration_since(t) >= Duration::from_secs(1))
            {
                match udp.send_to(SUBSCRIPTION, peer) {
                    Ok(n) if n == SUBSCRIPTION.len() => *renewed = Some(now),
                    Ok(_) => {}
                    Err(e)
                        if matches!(
                            e.kind(),
                            io::ErrorKind::WouldBlock | io::ErrorKind::ConnectionRefused
                        ) => {}
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(())
    }
    fn receive(&mut self, bytes: &[u8], source: SocketAddr, now: Instant) {
        let Some((peer, key)) = self.target.as_ref() else {
            return;
        };
        if source != SocketAddr::V4(*peer) {
            self.wrong_source = self.wrong_source.saturating_add(1);
            return;
        }
        match decode(bytes, key.as_bytes()) {
            Ok(sample) => self.accept(sample, bytes.len(), now),
            Err(_) => self.rejected = self.rejected.saturating_add(1),
        }
    }
    fn accept(&mut self, sample: Sample, length: usize, now: Instant) {
        if self.session != Some(sample.session) {
            // Never let a delayed packet resurrect a retired session. Bound history by
            // refusing further session transitions when full, rather than forgetting it.
            if self.retired.contains(&sample.session) || self.retired.len() >= 128 {
                self.reordered = self.reordered.saturating_add(1);
                return;
            }
            if let Some(old) = self.session {
                self.retired.insert(old);
                self.session_changes += 1;
            }
            self.session = Some(sample.session);
            self.last_sequence = None;
            self.objects.clear();
            self.sender_dropped = 0;
        }
        if let Some(previous) = self.last_sequence {
            if sample.sequence <= previous {
                self.reordered = self.reordered.saturating_add(1);
                return;
            }
            self.gaps = self.gaps.saturating_add(sample.sequence - previous - 1);
        }
        self.last_sequence = Some(sample.sequence);
        self.received = Some(now);
        self.packets = self.packets.saturating_add(1);
        self.bytes = self.bytes.saturating_add(length as u64);
        self.sender_dropped = self.sender_dropped.max(sample.dropped);
        self.rate_packets += 1;
        self.rate_bytes += length as u64;
        let key = (sample.object_id, sample.group);
        if !self.objects.contains_key(&key) && self.objects.len() >= MAX_GROUPS {
            if let Some(oldest) = self
                .objects
                .iter()
                .min_by_key(|(_, v)| v.received)
                .map(|(k, _)| *k)
            {
                self.objects.remove(&oldest);
            }
        }
        self.objects.insert(
            key,
            Observation {
                sample,
                received: now,
            },
        );
    }
    fn tick(&mut self, now: Instant) {
        self.objects
            .retain(|_, v| now.duration_since(v.received) < EXPIRY);
        let elapsed = now.duration_since(self.rate_started).as_secs_f64();
        if elapsed >= 0.5 {
            self.bytes_per_second = self.rate_bytes as f64 / elapsed;
            self.packets_per_second = self.rate_packets as f64 / elapsed;
            self.rate_started = now;
            self.rate_bytes = 0;
            self.rate_packets = 0;
        }
    }
    fn snapshot(&self, now: Instant) -> Value {
        let mut objects: Vec<_> = self.objects.values().map(|v| {
            let sample = &v.sample;
            let age = now.duration_since(v.received);
            let values: Vec<_> = sample.values.iter().enumerate().map(|(i, value)| {
                value.map(|raw| {
                    let display = match fields(sample.group)[i].kind {
                        ValueKind::Float => format!("{}", f32::from_bits(raw as u32)),
                        ValueKind::Signed => format!("{}", raw as i64),
                        ValueKind::Unsigned => raw.to_string(),
                        ValueKind::Boolean => if raw == 0 { "false".into() } else { "true".into() },
                        ValueKind::Hex => format!("0x{raw:016x}"),
                    };
                    json!({ "raw": format!("0x{raw:016x}"), "display": display })
                })
            }).collect();
            json!({ "object_id": format!("0x{:08x}", sample.object_id), "group": sample.group,
                "category": sample.category, "kind": sample.kind, "session": format!("0x{:016x}", sample.session),
                "sequence": sample.sequence.to_string(), "sample_us": sample.sample_us.to_string(),
                "age_ms": age.as_millis() as u64, "stale": age >= STALE, "values": values })
        }).collect();
        objects.sort_by_key(|v| {
            (
                v["object_id"].as_str().unwrap_or("").to_owned(),
                v["group"].as_u64(),
            )
        });
        json!({ "status": if self.received.is_some_and(|t| now.duration_since(t) < STALE) { "streaming" } else { "stale / waiting for stream" },
            "session": self.session.map(|s| format!("0x{s:016x}")), "uptime_seconds": now.duration_since(self.started).as_secs(),
            "packets": self.packets.to_string(), "bytes": self.bytes.to_string(), "gaps": self.gaps.to_string(),
            "reordered": self.reordered.to_string(), "rejected": self.rejected.to_string(), "wrong_source": self.wrong_source.to_string(),
            "session_changes": self.session_changes.to_string(), "sender_dropped": self.sender_dropped.to_string(),
            "bytes_per_second": self.bytes_per_second, "packets_per_second": self.packets_per_second,
            "session_history_full": self.retired.len() >= 128, "objects": objects })
    }
}
fn schema() -> Value {
    let group = |g| {
        fields(g).iter().enumerate().filter(|(_, f)| f.name != "reserved")
        .map(|(i, f)| json!({ "index": i, "name": f.name, "kind": format!("{:?}", f.kind), "source": f.source })).collect::<Vec<_>>()
    };
    json!({ "object": group(0), "attack": group(1), "input": group(9), "field_count": FIELD_COUNT })
}
fn valid_headers(request: &str, port: u16) -> Option<&str> {
    let mut lines = request.split("\r\n");
    let mut first = lines.next()?.split(' ');
    let method = first.next()?;
    if !matches!(method, "GET" | "POST") {
        return None;
    }
    let route = first.next()?;
    if (method == "POST") != route.starts_with("/connect?") {
        return None;
    }
    if first.next()? != "HTTP/1.1" || first.next().is_some() {
        return None;
    }
    let mut host = None;
    let mut origin = None;
    for line in lines {
        if line.is_empty() {
            break;
        }
        let (name, value) = line.split_once(':')?;
        let value = value.trim();
        if name.eq_ignore_ascii_case("host") {
            if host.replace(value).is_some() {
                return None;
            }
        }
        if name.eq_ignore_ascii_case("origin") {
            if origin.replace(value).is_some() {
                return None;
            }
        }
        if name.eq_ignore_ascii_case("transfer-encoding")
            || (name.eq_ignore_ascii_case("content-length") && value != "0")
        {
            return None;
        }
        if name.eq_ignore_ascii_case("sec-fetch-site") && !matches!(value, "same-origin" | "none") {
            return None;
        }
    }
    let expected = format!("127.0.0.1:{port}");
    if host? != expected {
        return None;
    }
    if (method == "POST" && origin.is_none())
        || origin.is_some_and(|o| o != format!("http://{expected}"))
    {
        return None;
    }
    Some(route)
}
fn serve(mut stream: TcpStream, port: u16, state: &Mutex<State>, schema: &[u8]) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_millis(50)))?;
    stream.set_write_timeout(Some(Duration::from_millis(100)))?;
    let deadline = Instant::now() + Duration::from_millis(200);
    let mut request = Vec::new();
    let mut chunk = [0u8; 1024];
    while !request.windows(4).any(|w| w == b"\r\n\r\n") {
        if Instant::now() >= deadline || request.len() >= 8192 {
            return Ok(());
        }
        match stream.read(&mut chunk) {
            Ok(0) => return Ok(()),
            Ok(n) => request.extend_from_slice(&chunk[..n]),
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                continue
            }
            Err(e) => return Err(e),
        }
    }
    let route = std::str::from_utf8(&request)
        .ok()
        .and_then(|r| valid_headers(r, port));
    let (status, content_type, body) = match route {
        Some("/") => (
            "200 OK",
            "text/html; charset=utf-8",
            HTML.as_bytes().to_vec(),
        ),
        Some("/schema") => ("200 OK", "application/json", schema.to_vec()),
        Some("/state") => {
            let snapshot = state.lock().unwrap().snapshot(Instant::now());
            ("200 OK", "application/json", serde_json::to_vec(&snapshot)?)
        }
        Some(route) if route.starts_with("/connect?") => {
            match state.lock().unwrap().configure(route) {
                Ok(()) => (
                    "200 OK",
                    "text/plain",
                    b"Subscribed; enable telemetry in Training on Switch".to_vec(),
                ),
                Err(error) => ("400 Bad Request", "text/plain", error.as_bytes().to_vec()),
            }
        }
        Some(_) => ("404 Not Found", "text/plain", b"Not found".to_vec()),
        None => ("403 Forbidden", "text/plain", b"Forbidden".to_vec()),
    };
    let headers = format!("HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nCross-Origin-Resource-Policy: same-origin\r\nContent-Security-Policy: default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'\r\n\r\n", body.len());
    stream.write_all(headers.as_bytes())?;
    stream.write_all(&body)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() > 1 {
        return Err("usage: telemetry_viewer [local HTTP port; default 8765]".into());
    }
    let port: u16 = args.first().map(|p| p.parse()).transpose()?.unwrap_or(8765);
    if port == 0 {
        return Err("HTTP port must be 1..65535".into());
    }
    let udp = UdpSocket::bind((std::net::Ipv4Addr::UNSPECIFIED, 0))?;
    udp.set_nonblocking(true)?;
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))?;
    listener.set_nonblocking(true)?;
    let state = Arc::new(Mutex::new(State::new()));
    let http_state = Arc::clone(&state);
    let http_worker = thread::spawn(move || {
        let schema = serde_json::to_vec(&schema()).unwrap();
        loop {
            match listener.accept() {
                Ok((stream, peer)) if peer.ip().is_loopback() => {
                    let _ = serve(stream, port, &http_state, &schema);
                }
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5))
                }
                Err(_) => break,
            }
        }
    });
    println!("Read-only telemetry viewer: http://127.0.0.1:{port}/ (waiting for stream)");
    println!("Trusted private LAN only: default stream is NOT authenticated or encrypted; explicitly enable Training telemetry on Switch (UDP {TELEMETRY_PORT}).");
    let mut buffer = [0u8; 1201];
    let mut generation = 0;
    let mut renewed = None;
    loop {
        if http_worker.is_finished() {
            return Err("local HTTP worker stopped".into());
        }
        {
            let mut state = state.lock().unwrap();
            if generation != state.generation {
                // Discard queued observations before admitting the newly selected endpoint.
                let mut drained = false;
                for _ in 0..256 {
                    match udp.recv_from(&mut buffer) {
                        Ok(_) => {}
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            drained = true;
                            break;
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
                if !drained {
                    drop(state);
                    thread::sleep(Duration::from_millis(2));
                    continue;
                }
                generation = state.generation;
                renewed = None;
            }
            let now = Instant::now();
            state.renew(&udp, &mut renewed, now)?;
            for _ in 0..256 {
                match udp.recv_from(&mut buffer) {
                    Ok((n, source)) => state.receive(&buffer[..n], source, Instant::now()),
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                    Err(e) => return Err(e.into()),
                }
            }
            state.tick(Instant::now());
        }
        thread::sleep(Duration::from_millis(2));
    }
}

const HTML: &str = include_str!("telemetry_viewer.html");

#[cfg(test)]
mod tests {
    use super::*;
    fn sample(session: u64, sequence: u64, object_id: u32, group: u16) -> Sample {
        Sample {
            session,
            sequence,
            sample_us: sequence,
            object_id,
            category: 0,
            kind: 0,
            group,
            dropped: 0,
            values: [None; FIELD_COUNT],
        }
    }
    #[test]
    fn subscription_renews_and_receives_on_the_same_ephemeral_socket() {
        let server = UdpSocket::bind("127.0.0.1:0").unwrap();
        server
            .set_read_timeout(Some(Duration::from_millis(50)))
            .unwrap();
        let udp = UdpSocket::bind("127.0.0.1:0").unwrap();
        udp.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        let SocketAddr::V4(peer) = server.local_addr().unwrap() else {
            unreachable!()
        };
        let mut state = State::new();
        state.target = Some((peer, String::new()));
        let now = Instant::now();
        let mut renewed = None;
        let mut bytes = [0; 1200];
        state.renew(&udp, &mut renewed, now).unwrap();
        let (n, source) = server.recv_from(&mut bytes).unwrap();
        assert_eq!(&bytes[..n], SUBSCRIPTION);
        assert_eq!(source, udp.local_addr().unwrap());
        state
            .renew(&udp, &mut renewed, now + Duration::from_millis(999))
            .unwrap();
        assert!(server.recv_from(&mut bytes).is_err());
        state
            .renew(&udp, &mut renewed, now + Duration::from_secs(1))
            .unwrap();
        let (n, same_source) = server.recv_from(&mut bytes).unwrap();
        assert_eq!(&bytes[..n], SUBSCRIPTION);
        assert_eq!(same_source, source);
        let n =
            kirby_assist_core::telemetry::encode(&sample(1, 1, 42, 0), b"", &mut bytes).unwrap();
        server.send_to(&bytes[..n], source).unwrap();
        let (n, source) = udp.recv_from(&mut bytes).unwrap();
        state.receive(&bytes[..n], source, now);
        assert_eq!(state.objects.len(), 1);
    }
    #[test]
    fn setup_resets_observations_and_filters_endpoint_and_wire_mode() {
        let now = Instant::now();
        let mut state = State::new();
        state
            .configure("/connect?endpoint=192.168.1.10:4243")
            .unwrap();
        let mut bytes = [0; 1200];
        let n =
            kirby_assist_core::telemetry::encode(&sample(1, 1, 42, 0), b"", &mut bytes).unwrap();
        state.receive(&bytes[..n], "192.168.1.10:4242".parse().unwrap(), now);
        assert_eq!(state.wrong_source, 1);
        assert!(state.objects.is_empty());
        state.receive(&bytes[..n], "192.168.1.10:4243".parse().unwrap(), now);
        assert_eq!(state.objects.len(), 1);
        for endpoint in [
            "8.8.8.8:4243",
            "127.0.0.1:4243",
            "192.168.1.0:4243",
            "192.168.1.10:0",
            "[::1]:4243",
        ] {
            assert!(state
                .configure(&format!("/connect?endpoint={endpoint}"))
                .is_err());
            assert_eq!(state.objects.len(), 1);
        }
        state
            .configure("/connect?endpoint=192.168.1.11:4243")
            .unwrap();
        assert!(state.objects.is_empty());
        assert!(state.retired.is_empty());
        assert_eq!(state.last_sequence, None);
        assert_eq!(state.received, None);
        state.receive(&bytes[..n], "192.168.1.10:4243".parse().unwrap(), now);
        assert!(state.objects.is_empty());
        let n = kirby_assist_core::telemetry::encode(
            &sample(1, 1, 42, 0),
            b"0123456789abcdef0123456789abcdef",
            &mut bytes,
        )
        .unwrap();
        state.receive(&bytes[..n], "192.168.1.11:4243".parse().unwrap(), now);
        assert_eq!(state.rejected, 1);
    }
    #[test]
    fn ordering_sessions_and_expiry() {
        let now = Instant::now();
        let mut state = State::new();
        state.accept(sample(1, 1, 1, 0), 604, now);
        state.accept(sample(1, 4, 1, 1), 604, now);
        state.accept(sample(1, 3, 1, 0), 604, now);
        assert_eq!(state.gaps, 2);
        assert_eq!(state.reordered, 1);
        assert_eq!(state.objects.len(), 2);
        state.accept(sample(2, 0, 2, 0), 604, now);
        state.accept(sample(1, 5, 1, 0), 604, now);
        assert_eq!(state.session, Some(2));
        assert_eq!(state.objects.len(), 1);
        assert_eq!(state.snapshot(now + STALE)["objects"][0]["stale"], true);
        state.tick(now + EXPIRY);
        assert!(state.objects.is_empty());
    }
    #[test]
    fn bounded_storage_and_exact_values() {
        let now = Instant::now();
        let mut state = State::new();
        for i in 0..600 {
            state.accept(sample(1, i as u64, i, 0), 604, now);
        }
        assert_eq!(state.objects.len(), MAX_GROUPS);
        let mut s = sample(1, 600, 700, 0);
        s.values[3] = Some(u64::MAX);
        state.accept(s, 604, now);
        let snapshot = state.snapshot(now);
        let object = snapshot["objects"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["object_id"] == "0x000002bc")
            .unwrap();
        assert_eq!(object["values"][3]["raw"], "0xffffffffffffffff");
    }
    #[test]
    fn dashboard_snapshot_preserves_exact_unknown_and_nonfinite_values() {
        let now = Instant::now();
        let mut state = State::new();
        let mut s = sample(u64::MAX, u64::MAX, u32::MAX, 0);
        s.sample_us = u64::MAX;
        s.values[0] = Some(u64::MAX);
        s.values[4] = Some(f32::NAN.to_bits() as u64);
        s.values[5] = Some(f32::INFINITY.to_bits() as u64);
        s.values[14] = Some(u64::MAX);
        state.accept(s, 604, now);
        let snapshot = state.snapshot(now);
        let object = &snapshot["objects"][0];
        assert_eq!(object["session"], "0xffffffffffffffff");
        assert_eq!(object["sequence"], u64::MAX.to_string());
        assert_eq!(object["sample_us"], u64::MAX.to_string());
        assert_eq!(object["values"][0]["display"], "-1");
        assert_eq!(object["values"][4]["display"], "NaN");
        assert_eq!(object["values"][5]["display"], "inf");
        assert_eq!(object["values"][14]["display"], u64::MAX.to_string());
        assert!(object["values"][29].is_null());
        assert_eq!(
            state.snapshot(now + STALE - Duration::from_nanos(1))["objects"][0]["stale"],
            false
        );
        assert_eq!(state.snapshot(now + STALE)["objects"][0]["stale"], true);
        state.tick(now + EXPIRY - Duration::from_nanos(1));
        assert_eq!(state.objects.len(), 1);
        state.tick(now + EXPIRY);
        assert!(state.objects.is_empty());
    }
    #[test]
    fn throughput_and_sender_drops_are_distinct_from_sequence_gaps() {
        let now = Instant::now();
        let mut state = State::new();
        state.rate_started = now;
        let mut first = sample(1, 1, 1, 0);
        first.dropped = 7;
        state.accept(first, 100, now);
        state.accept(sample(1, 4, 1, 0), 200, now);
        state.tick(now + Duration::from_millis(500));
        assert_eq!(state.gaps, 2);
        assert_eq!(state.sender_dropped, 7);
        assert_eq!(state.packets_per_second, 4.0);
        assert_eq!(state.bytes_per_second, 600.0);
        state.tick(now + EXPIRY);
        assert_eq!(state.packets_per_second, 0.0);
        assert_eq!(state.bytes_per_second, 0.0);
    }
    #[test]
    fn serves_live_state_over_local_http_without_exposing_secrets() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let port = address.port();
        let worker = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut state = State::new();
            state.accept(sample(1, 1, 42, 0), 100, Instant::now());
            serve(
                stream,
                port,
                &Mutex::new(state),
                &serde_json::to_vec(&schema()).unwrap(),
            )
            .unwrap();
        });
        let mut client = TcpStream::connect(address).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        write!(
            client,
            "GET /state HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n"
        )
        .unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        worker.join().unwrap();
        assert!(response.starts_with("HTTP/1.1 200 OK"));
        let body: serde_json::Value =
            serde_json::from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
        assert_eq!(body["session"], "0x0000000000000001");
        assert_eq!(body["objects"][0]["object_id"], "0x0000002a");
        assert!(body.get("token").is_none());
    }
    #[test]
    fn local_headers_only() {
        assert_eq!(
            valid_headers("GET /state HTTP/1.1\r\nHost: 127.0.0.1:8765\r\n\r\n", 8765),
            Some("/state")
        );
        let setup = "POST /connect?endpoint=192.168.1.10:4243 HTTP/1.1\r\nHost: 127.0.0.1:8765\r\nOrigin: http://127.0.0.1:8765\r\nContent-Length: 0\r\n\r\n";
        assert_eq!(
            valid_headers(setup, 8765),
            Some("/connect?endpoint=192.168.1.10:4243")
        );
        assert!(valid_headers(&setup.replace("POST", "GET"), 8765).is_none());
        assert!(valid_headers(
            &setup.replace("Origin: http://127.0.0.1:8765\r\n", ""),
            8765
        )
        .is_none());
        assert!(valid_headers(
            &setup.replace("Content-Length: 0", "Content-Length: 12"),
            8765
        )
        .is_none());
        for headers in [
            "Host: evil.test",
            "Host: 127.0.0.1:8765\r\nOrigin: https://evil.test",
            "Host: 127.0.0.1:8765\r\nHost: evil.test",
            "Host: 127.0.0.1:8765\r\nSec-Fetch-Site: cross-site",
        ] {
            assert!(
                valid_headers(&format!("GET /state HTTP/1.1\r\n{headers}\r\n\r\n"), 8765).is_none()
            );
        }
    }
}
