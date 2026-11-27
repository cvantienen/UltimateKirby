//! Explicitly activated, inbound-only diagnostic transport. No controller or game APIs.
use std::{
    io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

const CONFIG_PATH: &str = "sd:/ultimate/kirby_assist/lan_editor.txt";
const LEASE: Duration = Duration::from_millis(100);
static ENABLED: AtomicBool = AtomicBool::new(false);
static STATUS: Mutex<Option<String>> = Mutex::new(None);
static RELOAD: AtomicBool = AtomicBool::new(false);
static OBSERVATION: Mutex<Option<(Instant, bool)>> = Mutex::new(None);
static WORKER: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);

pub(crate) fn observe(training: bool, editing: bool) {
    if !training {
        revoke();
        return;
    }
    if let Ok(mut observation) = OBSERVATION.try_lock() {
        *observation = Some((Instant::now(), editing));
    }
}

pub(crate) fn revoke() {
    ENABLED.store(false, Ordering::Release);
    if let Ok(mut observation) = OBSERVATION.try_lock() {
        *observation = None;
    }
}

pub(crate) fn enabled() -> bool {
    ENABLED.load(Ordering::Acquire)
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
        observation.is_some_and(|(seen, open)| seen.elapsed() < LEASE && (!editing || open))
    })
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
        crate::logging::write("LAN: disabled explicitly");
        return Ok(());
    }
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
    set_status("STARTING — reading lan_editor.txt and opening network listener");
    ENABLED.store(true, Ordering::Release);
    match std::thread::Builder::new()
        .name("kirby-diagnostic-http".into())
        .stack_size(2 * 1024 * 1024)
        .spawn(run)
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

impl Config {
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
            token: token.ok_or("Missing token (32..128 URL-safe characters)")?,
        })
    }
}

fn private_ip(text: &str) -> Result<Ipv4Addr, String> {
    text.parse::<Ipv4Addr>()
        .ok()
        .filter(|ip| ip.is_private())
        .ok_or_else(|| "Use a numeric RFC1918 private IPv4 address".into())
}

fn run() {
    finish(serve());
}

fn finish(result: Result<(), String>) {
    let was_enabled = enabled();
    revoke();
    match result {
        Ok(()) => {
            if was_enabled {
                set_status("Training observation expired; reopen editor and activate again");
            }
            crate::logging::write("LAN: listener closed; explicit activation required");
        }
        Err(error) => {
            set_status(error.clone());
            crate::logging::write_owned(format!(
                "LAN: stopped: {error}; explicit activation required"
            ));
        }
    }
}

fn load_config(path: &std::path::Path) -> Result<Config, String> {
    use std::io::Read;
    let mut source = String::new();
    std::fs::File::open(path)
        .map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                "Setup required: create /ultimate/kirby_assist/lan_editor.txt on SD (switch_ip, computer_ip, port, token)".to_owned()
            } else {
                "Cannot open lan_editor.txt on SD; check file access".to_owned()
            }
        })?
        .take(2049)
        .read_to_string(&mut source)
        .map_err(|_| "Cannot read LAN setup as UTF-8")?;
    if source.len() > 2048 {
        return Err("LAN setup exceeds 2048 bytes".into());
    }
    Config::parse(&source)
}

fn serve() -> Result<(), String> {
    let config = load_config(std::path::Path::new(CONFIG_PATH))?;
    if !enabled() || !fresh(true) {
        return Ok(());
    }
    // The installed Switch std backend owns socket/NIFM initialization. It may
    // wait on the OS; never run this on a game callback or finalize shared NIFM.
    let listener = TcpListener::bind(config.local)
        .map_err(|e| format!("Cannot bind configured Switch address: {e}"))?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    listener_ready();
    crate::logging::write_owned(format!(
        "LAN: listening on {}; paired inbound HTTP only; no route-isolation claim",
        config.local
    ));
    while enabled() && fresh(false) {
        match listener.accept() {
            Ok((mut stream, SocketAddr::V4(peer))) if *peer.ip() == config.peer => {
                stream
                    .set_read_timeout(Some(Duration::from_millis(200)))
                    .map_err(|e| e.to_string())?;
                stream
                    .set_write_timeout(Some(Duration::from_millis(200)))
                    .map_err(|e| e.to_string())?;
                if let Err(error) = handle(&mut stream, &config) {
                    crate::logging::write_owned(format!("LAN: request failed: {error}"));
                    return Err("Transport failure".into());
                }
            }
            Ok(_) => {} // Unpaired peers are closed without parsing or replying.
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(10))
            }
            Err(error) => return Err(format!("Accept failed: {error}")),
        }
    }
    Ok(())
}

fn handle(stream: &mut TcpStream, config: &Config) -> Result<(), String> {
    use kirby_assist_core::diagnostic_http::{self as http, Method, Route};
    let started = Instant::now();
    let allowed = || enabled() && fresh(false) && started.elapsed() < Duration::from_secs(2);
    let request = http::read_request(
        &mut std::io::BufReader::with_capacity(4096, &mut *stream),
        allowed,
    )
    .map_err(|e| format!("{e:?}"))?;
    if !http::constant_time_eq(&request.token, &config.token) {
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
                "{{\"protocol\":1,\"training\":true,\"editing\":{}}}",
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
            Err(error) => {
                crate::logging::write_owned(format!("LAN: read rejected: {error}"));
                respond(
                    stream,
                    409,
                    b"{\"error\":\"File unavailable or save busy; see live logs\"}",
                    None,
                    allowed,
                )
            }
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
                || enabled() && fresh(false) && started.elapsed() < Duration::from_secs(10);
            match crate::live::settings_menu::lan_write(
                &request.body,
                request.revision.as_deref().unwrap_or(""),
                || save_allowed() && fresh(true),
            ) {
                Ok(()) => {
                    crate::logging::write("LAN SAVE: canonical SD file verified and catalog reloaded; fighter menu refresh pending");
                    respond(
                        stream,
                        200,
                        b"{\"saved\":true,\"reloaded\":true}",
                        None,
                        save_allowed,
                    )
                }
                Err(error) => {
                    crate::logging::write_owned(format!("LAN SAVE: rejected: {error}"));
                    respond(
                        stream,
                        409,
                        b"{\"error\":\"Save rejected; reload file and inspect live logs\"}",
                        None,
                        save_allowed,
                    )
                }
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

fn respond(
    stream: &mut TcpStream,
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
    fn missing_setup_failure_stays_visible_without_starting_network() {
        let _serial = TEST_GATE.lock().unwrap();
        let directory = std::env::temp_dir().join(format!(
            "kirby-lan-setup-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let error = load_config(&directory.join("lan_editor.txt"))
            .err()
            .unwrap();
        observe(true, true);
        ENABLED.store(true, Ordering::Release);
        set_status("STARTING");
        finish(Err(error));
        assert!(!enabled());
        assert!(status().unwrap().contains("Setup required"));
        assert!(status().unwrap().contains("lan_editor.txt"));
        observe(true, true);
        assert!(status().unwrap().contains("Setup required"));
    }

    #[test]
    fn listener_status_changes_only_after_binding_and_failures_hide_secrets() {
        let _serial = TEST_GATE.lock().unwrap();
        observe(true, true);
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
        ENABLED.store(true, Ordering::Release);
        *OBSERVATION.lock().unwrap() = Some((Instant::now() - LEASE, true));
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
        ENABLED.store(true, Ordering::Release);
        observe(true, false);
        assert!(fresh(false));
        assert!(!fresh(true));
        *OBSERVATION.lock().unwrap() = Some((Instant::now() - LEASE, true));
        assert!(!fresh(false));
        observe(false, false);
        assert!(!enabled());
    }

    #[test]
    fn paired_request_reads_logs_and_bad_token_never_reads_sequences() {
        use std::io::{Read, Write};
        let _serial = TEST_GATE.lock().unwrap();
        for (token, expected) in [
            ("0123456789abcdef0123456789abcdef", "200 OK"),
            ("incorrect", "403 Forbidden"),
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            let client = std::thread::spawn(move || {
                let mut stream = TcpStream::connect(address).unwrap();
                stream
                    .write_all(
                        format!("GET /logs?after=0 HTTP/1.1\r\nX-Kirby-Token: {token}\r\n\r\n")
                            .as_bytes(),
                    )
                    .unwrap();
                let mut response = String::new();
                stream.read_to_string(&mut response).unwrap();
                response
            });
            let (mut stream, _) = listener.accept().unwrap();
            observe(true, true);
            ENABLED.store(true, Ordering::Release);
            let config = Config {
                local: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 4242),
                peer: Ipv4Addr::LOCALHOST,
                token: "0123456789abcdef0123456789abcdef".into(),
            };
            handle(&mut stream, &config).unwrap();
            drop(stream);
            assert!(client.join().unwrap().contains(expected));
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
