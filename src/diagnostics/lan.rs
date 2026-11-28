//! Explicitly activated, inbound-only diagnostic transport. No controller or game APIs.
#[path = "network.rs"]
mod network;
use network::socket_result;
#[path = "lan_udp.rs"]
mod udp;
use std::{
    io,
    net::{Ipv4Addr, SocketAddrV4, UdpSocket},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Mutex,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

const LEASE: Duration = Duration::from_millis(100);
static ENABLED: AtomicBool = AtomicBool::new(false);
static STATUS: Mutex<Option<String>> = Mutex::new(None);
static RELOAD: AtomicBool = AtomicBool::new(false);
// Revocation must invalidate leases even when their mutex cannot be acquired.
static GENERATION: AtomicU64 = AtomicU64::new(0);
static ACTIVATION: AtomicU64 = AtomicU64::new(0);
static OBSERVATION: Mutex<Option<(Instant, bool, u64)>> = Mutex::new(None);
static WORKER: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);

pub(crate) fn observe(training: bool, editing: bool) {
    if !training {
        revoke();
        return;
    }
    let generation = GENERATION.load(Ordering::Acquire);
    if let Ok(mut observation) = OBSERVATION.try_lock() {
        let generation = if observation
            .is_some_and(|(seen, _, old)| old == generation && seen.elapsed() >= LEASE)
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
        *observation = Some((Instant::now(), editing, generation));
    }
}

pub(crate) fn revoke() {
    GENERATION.fetch_add(1, Ordering::AcqRel);
    ENABLED.store(false, Ordering::Release);
    if let Ok(mut observation) = OBSERVATION.try_lock() {
        *observation = None;
    }
}

pub(crate) fn enabled() -> bool {
    ENABLED.load(Ordering::Acquire)
        && ACTIVATION.load(Ordering::Acquire) == GENERATION.load(Ordering::Acquire)
}

pub(crate) fn status() -> Option<String> {
    STATUS.try_lock().ok().and_then(|status| status.clone())
}

fn set_status(message: impl Into<String>) {
    *STATUS.lock().unwrap_or_else(|e| e.into_inner()) = Some(message.into());
}

fn listener_ready() {
    if enabled() && fresh(false) {
        set_status("LISTENING — connect from Workbench Training LAN");
    }
}

pub(crate) fn request_reload() {
    RELOAD.store(true, Ordering::Release);
}

pub(crate) fn reload_pending() -> bool {
    RELOAD.load(Ordering::Acquire)
}

pub(crate) fn take_reload() -> bool {
    RELOAD.swap(false, Ordering::AcqRel)
}

fn fresh(editing: bool) -> bool {
    OBSERVATION.try_lock().ok().is_some_and(|observation| {
        observation.is_some_and(|(seen, open, generation)| {
            generation == GENERATION.load(Ordering::Acquire)
                && seen.elapsed() < LEASE
                && (!editing || open)
        })
    })
}

fn authorized(generation: u64) -> bool {
    enabled() && fresh(false) && generation == GENERATION.load(Ordering::Acquire)
}

pub(crate) fn toggle() -> Result<(), String> {
    let result = toggle_inner();
    if let Err(error) = &result {
        set_status(error.clone());
    }
    result
}

fn toggle_inner() -> Result<(), String> {
    if enabled() {
        revoke();
        set_status("Disabled explicitly");
        return Ok(());
    }
    let generation = GENERATION.load(Ordering::Acquire);
    if !fresh(true) {
        return Err("Open the Training editor before activating LAN".into());
    }
    let mut worker = WORKER.try_lock().map_err(|_| "LAN worker busy; retry")?;
    if worker.as_ref().is_some_and(|handle| !handle.is_finished()) {
        return Err("LAN worker is still closing; retry".into());
    }
    if let Some(handle) = worker.take() {
        let _ = handle.join();
    }
    set_status("STARTING — opening UDP listener (default port 4242)");
    ACTIVATION.store(generation, Ordering::Release);
    ENABLED.store(true, Ordering::Release);
    if !authorized(generation) || !fresh(true) {
        ENABLED.store(false, Ordering::Release);
        return Err("Open the Training editor before activating LAN".into());
    }
    match std::thread::Builder::new()
        .name("kirby-diagnostic-udp".into())
        .stack_size(2 * 1024 * 1024)
        .spawn(move || run(generation))
    {
        Ok(handle) => {
            *worker = Some(handle);
            Ok(())
        }
        Err(error) => {
            revoke();
            let message = format!("Cannot start LAN worker: {error}");
            set_status(message.clone());
            Err(message)
        }
    }
}

struct Config {
    local: SocketAddrV4,
    peer: Ipv4Addr,
    token: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            local: SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 4242),
            peer: Ipv4Addr::UNSPECIFIED,
            token: String::new(),
        }
    }
}

impl Config {
    fn permits(&self, peer: Ipv4Addr) -> bool {
        if self.peer.is_unspecified() {
            peer.is_private()
        } else {
            peer == self.peer
        }
    }

    #[cfg(test)]
    fn parse(source: &str) -> Result<Self, String> {
        let mut local = None;
        let mut peer = None;
        let mut port = None;
        let mut token = None;
        for line in source.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = line.split_once('=').ok_or("LAN setup requires key=value")?;
            let value = value.trim();
            match key.trim() {
                "switch_ip" if local.is_none() => local = Some(private_ip(value)?),
                "computer_ip" if peer.is_none() => peer = Some(private_ip(value)?),
                "port" if port.is_none() => {
                    port = Some(
                        value
                            .parse::<u16>()
                            .ok()
                            .filter(|p| *p > 0)
                            .ok_or("LAN port must be 1..65535")?,
                    )
                }
                "token"
                    if token.is_none()
                        && (32..=128).contains(&value.len())
                        && value
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_') =>
                {
                    token = Some(value.to_owned())
                }
                _ => return Err("Unknown, duplicate or invalid LAN setup field".into()),
            }
        }
        let local = local.ok_or("Missing switch_ip")?;
        let peer = peer.ok_or("Missing computer_ip")?;
        if local == peer {
            return Err("Switch and computer IPs must differ".into());
        }
        Ok(Self {
            local: SocketAddrV4::new(local, port.ok_or("Missing port")?),
            peer,
            token: token.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
fn private_ip(text: &str) -> Result<Ipv4Addr, String> {
    text.parse::<Ipv4Addr>()
        .ok()
        .filter(|ip| ip.is_private())
        .ok_or_else(|| "Use a numeric RFC1918 private IPv4 address".into())
}

fn run(generation: u64) {
    finish(serve(generation));
}

fn finish(result: Result<(), String>) {
    let was_enabled = enabled();
    revoke();
    match result {
        Ok(()) => {
            if was_enabled {
                set_status("Training observation expired; reopen editor and activate again");
            }
        }
        Err(error) => {
            set_status(error.clone());
        }
    }
}

#[cfg(test)]
fn load_config(path: &std::path::Path) -> Result<Config, String> {
    use std::io::Read;
    let mut source = String::new();
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Config::default()),
        Err(_) => {
            return Err("Cannot open optional lan_editor.txt on SD; check file access".into())
        }
    };
    file.take(2049)
        .read_to_string(&mut source)
        .map_err(|_| "Cannot read LAN setup as UTF-8")?;
    if source.len() > 2048 {
        return Err("LAN setup exceeds 2048 bytes".into());
    }
    Config::parse(&source)
}

fn serve(generation: u64) -> Result<(), String> {
    // File-free activation must not inherit stale addresses or keys from an old setup.
    let config = Config::default();
    if !authorized(generation) || !fresh(true) {
        return Ok(());
    }
    // The installed Switch std backend owns socket/NIFM initialization. It may
    // wait on the OS; never run this on a game callback or finalize shared NIFM.
    let datagrams = socket_result(UdpSocket::bind(config.local))
        .map_err(|e| format!("Cannot bind UDP editor: {e}"))?;
    socket_result(datagrams.set_nonblocking(true))
        .map_err(|e| format!("Cannot set UDP editor nonblocking: {e}"))?;
    if !authorized(generation) {
        return Ok(());
    }
    listener_ready();
    udp::serve(datagrams, &config)
}

fn dispatch<W: std::io::Write>(
    stream: &mut W,
    config: &Config,
    request: kirby_assist_core::diagnostic_http::Request,
    started: Instant,
    generation: u64,
) -> Result<(), String> {
    use kirby_assist_core::diagnostic_http::{self as http, Method, Route};
    let allowed = || authorized(generation) && started.elapsed() < Duration::from_secs(2);
    if !allowed() {
        return Err("Training authorization expired".into());
    }
    if !config.token.is_empty() && !http::constant_time_eq(&request.token, &config.token) {
        return respond(
            stream,
            403,
            b"{\"error\":\"Pairing rejected\"}",
            None,
            allowed,
        );
    }
    match (request.method, request.route) {
        (Method::Get, Route::Status) => {
            let body = format!(
                "{{\"protocol\":1,\"training\":true,\"udp_port\":4242,\"telemetry_port\":4243,\"editing\":{}}}",
                fresh(true)
            );
            respond(stream, 200, body.as_bytes(), None, allowed)
        }
        (Method::Get, Route::Logs { after }) => {
            let body = crate::logging::live_logs(after);
            respond(stream, 200, body.as_bytes(), None, allowed)
        }
        (Method::Get, Route::Sequences) => match crate::live::settings_menu::lan_read() {
            Ok((body, revision)) => respond(stream, 200, &body, Some(&revision), allowed),
            Err(_) => respond(
                stream,
                409,
                b"{\"error\":\"File unavailable or save busy; see live logs\"}",
                None,
                allowed,
            ),
        },
        (Method::Put, Route::Sequences) => {
            if !fresh(true) {
                return respond(
                    stream,
                    403,
                    b"{\"error\":\"Open the Training editor before saving\"}",
                    None,
                    allowed,
                );
            }
            let save_allowed =
                || authorized(generation) && started.elapsed() < Duration::from_secs(10);
            match crate::live::settings_menu::lan_write(
                &request.body,
                request.revision.as_deref().unwrap_or(""),
                || save_allowed() && fresh(true),
            ) {
                Ok(()) => respond(
                    stream,
                    200,
                    b"{\"saved\":true,\"reloaded\":true}",
                    None,
                    save_allowed,
                ),
                Err(_) => respond(
                    stream,
                    409,
                    b"{\"error\":\"Save rejected; reload file and inspect live logs\"}",
                    None,
                    save_allowed,
                ),
            }
        }
        _ => respond(
            stream,
            405,
            b"{\"error\":\"Unsupported request\"}",
            None,
            allowed,
        ),
    }
}

fn respond<W: std::io::Write>(
    stream: &mut W,
    status: u16,
    body: &[u8],
    revision: Option<&str>,
    allowed: impl FnMut() -> bool,
) -> Result<(), String> {
    use kirby_assist_core::diagnostic_http::{write_response, ContentType, Response};
    let response = Response {
        status,
        content_type: if revision.is_some() {
            ContentType::Text
        } else {
            ContentType::Json
        },
        revision: revision.map(str::to_owned),
        body: body.to_vec(),
    };
    write_response(stream, &response, allowed).map_err(|e| format!("{e:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    static TEST_GATE: Mutex<()> = Mutex::new(());

    #[test]
    fn missing_setup_uses_file_free_private_peer_defaults() {
        let _serial = TEST_GATE.lock().unwrap();
        let directory = std::env::temp_dir().join(format!(
            "kirby-lan-setup-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let config = load_config(&directory.join("lan_editor.txt")).unwrap();
        assert!(config.token.is_empty());
        assert!(config.local.ip().is_unspecified());
        assert!(config.permits("192.168.1.20".parse().unwrap()));
        assert!(!config.permits("8.8.8.8".parse().unwrap()));
        assert!(!config.permits(Ipv4Addr::LOCALHOST));
    }

    #[test]
    fn listener_status_changes_only_after_binding_and_failures_hide_secrets() {
        let _serial = TEST_GATE.lock().unwrap();
        observe(true, true);
        ACTIVATION.store(GENERATION.load(Ordering::Acquire), Ordering::Release);
        ENABLED.store(true, Ordering::Release);
        set_status("STARTING");
        listener_ready();
        assert!(status().unwrap().contains("LISTENING"));
        let secret = "SECRET_VALUE_MUST_NEVER_APPEAR_IN_ERRORS";
        let source =
            format!("switch_ip=192.168.1.10\ncomputer_ip=192.168.1.20\nport=bad\ntoken={secret}\n");
        finish(Err(Config::parse(&source).err().unwrap()));
        assert!(!enabled());
        assert!(status().unwrap().contains("port"));
        assert!(!status().unwrap().contains(secret));
        observe(true, true);
        ACTIVATION.store(GENERATION.load(Ordering::Acquire), Ordering::Release);
        ENABLED.store(true, Ordering::Release);
        *OBSERVATION.lock().unwrap() = Some((
            Instant::now() - LEASE,
            true,
            GENERATION.load(Ordering::Acquire),
        ));
        finish(Ok(()));
        assert!(status().unwrap().contains("observation expired"));
    }

    #[test]
    fn activation_never_survives_training_exit_or_expired_observation() {
        let _serial = TEST_GATE.lock().unwrap();
        revoke();
        assert!(!enabled());
        assert!(!fresh(false));
        assert!(toggle().is_err());
        observe(true, true);
        assert!(fresh(true));
        ACTIVATION.store(GENERATION.load(Ordering::Acquire), Ordering::Release);
        ENABLED.store(true, Ordering::Release);
        observe(true, false);
        assert!(fresh(false));
        assert!(!fresh(true));
        *OBSERVATION.lock().unwrap() = Some((
            Instant::now() - LEASE,
            true,
            GENERATION.load(Ordering::Acquire),
        ));
        assert!(!fresh(false));
        observe(false, false);
        assert!(!enabled());
    }

    #[test]
    fn contended_revocation_and_expiry_cannot_reactivate_old_leases() {
        let _serial = TEST_GATE.lock().unwrap();
        observe(true, true);
        ACTIVATION.store(GENERATION.load(Ordering::Acquire), Ordering::Release);
        ENABLED.store(true, Ordering::Release);
        let old = GENERATION.load(Ordering::Acquire);
        {
            let _observation = OBSERVATION.lock().unwrap();
            revoke();
        }
        assert!(!fresh(true));
        assert!(toggle().is_err());
        observe(true, true);
        assert!(!authorized(old));
        assert!(!enabled());
        ACTIVATION.store(GENERATION.load(Ordering::Acquire), Ordering::Release);
        ENABLED.store(true, Ordering::Release);
        *OBSERVATION.lock().unwrap() = Some((
            Instant::now() - LEASE,
            true,
            GENERATION.load(Ordering::Acquire),
        ));
        observe(true, true);
        assert!(!enabled());
        assert!(fresh(true));
        revoke();
    }

    #[test]
    fn revoked_request_cannot_dispatch_or_send_udp_in_a_new_activation() {
        let _serial = TEST_GATE.lock().unwrap();
        observe(true, true);
        let old = GENERATION.load(Ordering::Acquire);
        revoke();
        observe(true, true);
        ACTIVATION.store(GENERATION.load(Ordering::Acquire), Ordering::Release);
        ENABLED.store(true, Ordering::Release);
        let request = kirby_assist_core::diagnostic_http::read_request(
            &mut std::io::Cursor::new(b"GET /sequences HTTP/1.1\r\n\r\n"),
            || true,
        )
        .unwrap();
        let mut output = Vec::new();
        assert!(dispatch(
            &mut output,
            &Config::default(),
            request,
            Instant::now(),
            old
        )
        .is_err());
        assert!(output.is_empty());
        let sender = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let receiver = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        receiver.set_nonblocking(true).unwrap();
        assert_eq!(
            udp::send(
                &sender,
                receiver.local_addr().unwrap(),
                2,
                1,
                0,
                1,
                &[],
                old
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::PermissionDenied
        );
        assert_eq!(
            receiver.recv(&mut [0; 64]).unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
        revoke();
        assert!(serve(old).is_ok());
    }

    #[test]
    fn disabling_lan_stops_udp_worker_with_an_incomplete_connected_transfer() {
        use std::net::UdpSocket;

        let _serial = TEST_GATE.lock().unwrap();
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        socket.set_nonblocking(true).unwrap();
        let address = socket.local_addr().unwrap();
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        client.connect(address).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let config = Config {
            peer: Ipv4Addr::LOCALHOST,
            ..Config::default()
        };
        observe(true, true);
        let generation = GENERATION.load(Ordering::Acquire);
        ACTIVATION.store(generation, Ordering::Release);
        ENABLED.store(true, Ordering::Release);

        std::thread::scope(|scope| {
            let server = scope.spawn(|| udp::serve(socket, &config));
            let mut request = b"KALE".to_vec();
            request.push(1);
            request.extend(99u64.to_be_bytes());
            request.extend(0u16.to_be_bytes());
            request.extend(2u16.to_be_bytes());
            request.extend([b'x'; 1024]);
            client.send(&request).unwrap();
            let mut response = [0; 32];
            let length = client.recv(&mut response).unwrap();
            assert_eq!(length, 17);
            assert_eq!(response[4], 2, "first upload chunk was acknowledged");

            revoke();
            assert!(server.join().unwrap().is_ok());
        });
        assert!(!enabled(), "revoked LAN worker must remain disabled");
    }

    #[test]
    fn udp_roundtrip_chunks_tokenless_save_and_rejects_completed_id_replay() {
        use std::net::UdpSocket;
        let _serial = TEST_GATE.lock().unwrap();
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        socket.set_nonblocking(true).unwrap();
        let address = socket.local_addr().unwrap();
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        client.connect(address).unwrap();
        client
            .set_read_timeout(Some(Duration::from_millis(20)))
            .unwrap();
        let config = Config {
            peer: Ipv4Addr::LOCALHOST,
            ..Config::default()
        };
        observe(true, true);
        ACTIVATION.store(GENERATION.load(Ordering::Acquire), Ordering::Release);
        ENABLED.store(true, Ordering::Release);
        let packet = |kind: u8, id: u64, index: u16, total: u16, body: &[u8]| {
            let mut bytes = b"KALE".to_vec();
            bytes.push(kind);
            bytes.extend(id.to_be_bytes());
            bytes.extend(index.to_be_bytes());
            bytes.extend(total.to_be_bytes());
            bytes.extend(body);
            bytes
        };
        std::thread::scope(|scope| {
            let server = scope.spawn(|| udp::serve(socket, &config));
            let body = vec![b'x'; 1500];
            let mut wire = format!(
                "PUT /sequences HTTP/1.1\r\nX-Kirby-Revision: abc\r\nContent-Length: {}\r\n\r\n",
                body.len()
            )
            .into_bytes();
            wire.extend(body);
            let chunks: Vec<_> = wire.chunks(1024).collect();
            let mut response = [0u8; 1041];
            client.send(&packet(1, 99, 0, 2, b"short")).unwrap();
            assert!(client.recv(&mut response).is_err());
            for id in [1, 2] {
                for (index, chunk) in chunks.iter().enumerate() {
                    observe(true, true);
                    let request = packet(1, id, index as u16, chunks.len() as u16, chunk);
                    client.send(&request).unwrap();
                    let n = client.recv(&mut response).unwrap();
                    assert_eq!(n, 17);
                    assert_eq!(response[4], 2);
                    // Repeated upload chunks are acknowledged, not appended or executed twice.
                    client.send(&request).unwrap();
                    assert_eq!(client.recv(&mut response).unwrap(), 17);
                }
                client
                    .send(&packet(1, 77, 0, 1, b"GET /status HTTP/1.1\r\n\r\n"))
                    .unwrap();
                assert!(
                    client.recv(&mut response).is_err(),
                    "unfinished reply cannot be evicted"
                );
                observe(true, true);
                client.send(&packet(3, id, 0, 0, &[])).unwrap();
                let n = client.recv(&mut response).unwrap();
                assert_eq!(response[4], 4);
                assert!(std::str::from_utf8(&response[17..n])
                    .unwrap()
                    .contains("\"saved\":true"));
                client.send(&packet(5, id, 1, 0, &[])).unwrap();
                assert_eq!(client.recv(&mut response).unwrap(), 17);
                assert_eq!(response[4], 6);
            }
            client
                .send(&packet(1, 1, 0, chunks.len() as u16, chunks[0]))
                .unwrap();
            assert!(client.recv(&mut response).is_err());
            revoke();
            server.join().unwrap().unwrap();
        });
    }

    #[test]
    fn paired_request_reads_logs_and_bad_token_never_reads_sequences() {
        let _serial = TEST_GATE.lock().unwrap();
        for (token, expected) in [
            ("0123456789abcdef0123456789abcdef", "200 OK"),
            ("incorrect", "403 Forbidden"),
        ] {
            observe(true, true);
            let generation = GENERATION.load(Ordering::Acquire);
            ACTIVATION.store(generation, Ordering::Release);
            ENABLED.store(true, Ordering::Release);
            let request = kirby_assist_core::diagnostic_http::read_request(
                &mut io::Cursor::new(format!(
                    "GET /logs?after=0 HTTP/1.1\r\nX-Kirby-Token: {token}\r\n\r\n"
                )),
                || true,
            )
            .unwrap();
            let config = Config {
                token: "0123456789abcdef0123456789abcdef".into(),
                ..Config::default()
            };
            let mut response = Vec::new();
            dispatch(&mut response, &config, request, Instant::now(), generation).unwrap();
            assert!(String::from_utf8(response).unwrap().contains(expected));
            revoke();
        }
    }

    #[test]
    fn paired_editor_flow_reports_admission_revision_and_save_result() {
        let _serial = TEST_GATE.lock().unwrap();
        let token = "0123456789abcdef0123456789abcdef";
        let source = b"[yoshi.roll.grab]\nactions=grab\n";
        let revision = kirby_assist_core::diagnostic_http::revision(source);
        for (method, route, editing, body, expected, detail) in [
            (
                "GET",
                "/status",
                false,
                &b""[..],
                "200 OK",
                "\"editing\":false",
            ),
            (
                "GET",
                "/sequences",
                false,
                &b""[..],
                "200 OK",
                revision.as_str(),
            ),
            (
                "PUT",
                "/sequences",
                false,
                &source[..],
                "403 Forbidden",
                "Open the Training editor",
            ),
            (
                "PUT",
                "/sequences",
                true,
                &source[..],
                "200 OK",
                "\"saved\":true",
            ),
        ] {
            let mut wire = format!(
                "{method} {route} HTTP/1.1\r\nX-Kirby-Token: {token}\r\nX-Kirby-Revision: {revision}\r\nContent-Length: {}\r\n\r\n",
                body.len()
            ).into_bytes();
            wire.extend_from_slice(body);
            observe(true, editing);
            ACTIVATION.store(GENERATION.load(Ordering::Acquire), Ordering::Release);
            ENABLED.store(true, Ordering::Release);
            let config = Config {
                local: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 4242),
                peer: Ipv4Addr::LOCALHOST,
                token: token.into(),
            };
            let request = kirby_assist_core::diagnostic_http::read_request(
                &mut io::Cursor::new(wire),
                || true,
            )
            .unwrap();
            let mut response = Vec::new();
            dispatch(
                &mut response,
                &config,
                request,
                Instant::now(),
                ACTIVATION.load(Ordering::Acquire),
            )
            .unwrap();
            let response = String::from_utf8(response).unwrap();
            assert!(response.contains(expected), "{response}");
            assert!(response.contains(detail), "{response}");
            revoke();
        }
    }

    #[test]
    fn setup_rejects_unpaired_public_duplicate_and_short_secrets() {
        let good = "switch_ip=192.168.1.10\ncomputer_ip=192.168.1.20\nport=4242\ntoken=0123456789abcdef0123456789abcdef\n";
        assert!(Config::parse(good).is_ok());
        for invalid in [
            good.replace("192.168.1.20", "8.8.8.8"),
            good.replace("192.168.1.20", "192.168.1.10"),
            format!("{good}port=4243\n"),
            good.replace("0123456789abcdef0123456789abcdef", "short"),
        ] {
            assert!(Config::parse(&invalid).is_err());
        }
    }
}
