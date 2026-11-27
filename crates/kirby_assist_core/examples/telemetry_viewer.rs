use kirby_assist_core::telemetry::{decode, fields, Config, Sample, ValueKind, FIELD_COUNT};
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
    if first.next()? != "GET" {
        return None;
    }
    let route = first.next()?;
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
    if origin.is_some_and(|o| o != format!("http://{expected}")) {
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
        Some(_) => ("404 Not Found", "text/plain", b"Not found".to_vec()),
        None => ("403 Forbidden", "text/plain", b"Forbidden".to_vec()),
    };
    let headers = format!("HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nCross-Origin-Resource-Policy: same-origin\r\nContent-Security-Policy: default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'\r\n\r\n", body.len());
    stream.write_all(headers.as_bytes())?;
    stream.write_all(&body)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(1..=2).contains(&args.len()) {
        return Err("usage: telemetry_viewer <config path> [local HTTP port; default 8765]".into());
    }
    let config = Config::parse(&std::fs::read_to_string(&args[0])?)?;
    let port: u16 = args.get(1).map(|p| p.parse()).transpose()?.unwrap_or(8765);
    if port == 0 {
        return Err("HTTP port must be 1..65535".into());
    }
    let udp = UdpSocket::bind(config.computer)?;
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
    println!(
        "Trusted private LAN only: HMAC authenticates packets but does not encrypt telemetry."
    );
    let mut buffer = [0u8; 1201];
    loop {
        if http_worker.is_finished() {
            return Err("local HTTP worker stopped".into());
        }
        for _ in 0..256 {
            match udp.recv_from(&mut buffer) {
                Ok((n, source)) => {
                    if source != SocketAddr::V4(config.switch) {
                        state.lock().unwrap().wrong_source += 1;
                        continue;
                    }
                    match decode(&buffer[..n], config.token.as_bytes()) {
                        Ok(sample) => state.lock().unwrap().accept(sample, n, Instant::now()),
                        Err(_) => state.lock().unwrap().rejected += 1,
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e.into()),
            }
        }
        state.lock().unwrap().tick(Instant::now());
        thread::sleep(Duration::from_millis(2));
    }
}

const HTML: &str = r#"<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>Training telemetry</title>
<style>body{font:14px system-ui;background:#15191d;color:#ddd;margin:24px}table{border-collapse:collapse;width:100%}td,th{text-align:left;padding:6px;border-bottom:1px solid #444}input,select{padding:8px;margin:5px;background:#222;color:#eee}.changed{background:#594821}.stale{opacity:.45}code{font-family:monospace}pre{white-space:pre-wrap}</style>
<h1>Read-only Training telemetry</h1><p>Trusted private LAN only: authenticated, not encrypted. No control channel. Missing values are unavailable, not zero. Values become stale after 250ms and expire after 1s.</p>
<pre id="status">Waiting for stream</pre><label>Object / group <select id="picker"></select></label><label>Search <input id="search" placeholder="Field, type, source"></label><label><input id="browse" type="checkbox">Browse schema</label>
<p>Coverage: fighter roster max16; observed attacking weapon/items max32; attack slots0..7. Common verified fields, not every engine variable. Inputs distinguish physical pre-mapping from mapped results after the existing finishing overlay.</p>
<label>Numeric history <select id="graph"></select></label><p>Graph uses approximate numeric display values; exact identifiers remain raw hex below.</p><canvas id="chart" width="760" height="160"></canvas>
<table><thead><tr><th>Field</th><th>Value</th><th>Exact raw u64</th><th>Kind / source</th></tr></thead><tbody id="rows"></tbody></table>
<script>
'use strict';const $=id=>document.getElementById(id);let schemas=null,state=null,previous=new Map(),highlights=new Map(),session=null,renderKey=null,lastFetch=performance.now();
const key=o=>o.object_id+':'+o.group;const schemasFor=g=>g===0?schemas.object:g===9?schemas.input:schemas.attack;
let history=[],historyKey=null,historySequence=null;
function chart(o,stale){const numeric=schemasFor(o?o.group:0).filter(f=>['Float','Signed','Unsigned'].includes(f.kind)&&!f.source.startsWith('unavailable'));const selector=$('graph');const wanted=selector.value;selector.replaceChildren();for(const f of numeric){const option=document.createElement('option');option.value=String(f.index);option.textContent=f.name;selector.append(option)}if(numeric.some(f=>String(f.index)===wanted))selector.value=wanted;const selected=(o?key(o):'none')+':'+selector.value;if(selected!==historyKey){history=[];historySequence=null;historyKey=selected}const value=o?o.values[Number(selector.value)]:null;if(o&&!stale&&o.sequence!==historySequence&&value){const n=Number(value.display);if(Number.isFinite(n)){history.push(n);if(history.length>120)history.shift()}historySequence=o.sequence}const canvas=$('chart'),ctx=canvas.getContext('2d');ctx.clearRect(0,0,canvas.width,canvas.height);ctx.fillStyle='#ddd';if(!history.length){ctx.fillText('Select a numeric field; waiting for fresh samples',12,20);return}const min=Math.min(...history),max=Math.max(...history),range=max-min||1;ctx.fillText('Last '+history.length+' samples (max120), min '+min+' max '+max+(stale?' — stale':''),12,20);ctx.strokeStyle=stale?'#777':'#6db9ee';ctx.beginPath();history.forEach((n,i)=>{const x=12+i*(canvas.width-24)/119,y=canvas.height-12-(n-min)*(canvas.height-45)/range;i?ctx.lineTo(x,y):ctx.moveTo(x,y)});ctx.stroke()}

function render(){if(!state||!schemas)return;const o=state.objects.find(o=>key(o)===$('picker').value);const selected=o?key(o):'schema';if(renderKey!==selected){previous.clear();highlights.clear();renderKey=selected}const elapsed=performance.now()-lastFetch;const stale=!o||o.stale||o.age_ms+elapsed>=250;const list=schemasFor(o?o.group:0);chart(o,stale);$('rows').replaceChildren();const search=$('search').value.toLowerCase();
for(const f of list){if(![f.name,f.kind,f.source].join(' ').toLowerCase().includes(search))continue;const v=o?o.values[f.index]:null;const raw=v?v.raw:null;if(previous.has(f.index)&&previous.get(f.index)!==raw)highlights.set(f.index,performance.now()+500);previous.set(f.index,raw);const tr=document.createElement('tr');if(stale&&!$('browse').checked)tr.classList.add('stale');if((highlights.get(f.index)||0)>performance.now())tr.classList.add('changed');for(const text of [f.index+' '+f.name,$('browse').checked?'—':v?v.display:'unavailable',$('browse').checked?'—':v?v.raw:'—',f.kind+' / '+f.source]){const td=document.createElement('td');td.textContent=text;tr.append(td)}$('rows').append(tr)}
$('status').textContent=[elapsed>=250?'stale / receiver unavailable':state.status,'session '+(state.session||'—'),o?'object '+o.object_id+' category '+o.category+' kind '+o.kind+' group '+o.group+' sequence '+o.sequence+' sample_us '+o.sample_us+' age '+Math.round(o.age_ms+elapsed)+'ms':'No current object', 'packets '+state.packets+' bytes '+state.bytes+' gaps '+state.gaps+' reordered '+state.reordered+' rejected '+state.rejected+' wrong-source '+state.wrong_source+' sender-dropped '+state.sender_dropped+' session changes '+state.session_changes, state.packets_per_second.toFixed(1)+' packets/s '+state.bytes_per_second.toFixed(0)+' bytes/s',state.session_history_full?'Session history full: restart viewer to admit another session':''].join('\n')}
async function poll(){try{if(!schemas)schemas=await(await fetch('/schema',{cache:'no-store'})).json();const response=await fetch('/state',{cache:'no-store',signal:AbortSignal.timeout(500)});if(!response.ok)throw Error('HTTP');const next=await response.json();if(next.session!==session){previous.clear();highlights.clear();history=[];historyKey=null;historySequence=null;session=next.session}state=next;lastFetch=performance.now();const old=$('picker').value;$('picker').replaceChildren();for(const o of state.objects){const option=document.createElement('option');option.value=key(o);option.textContent=o.object_id+' / '+(o.group===0?'object state':o.group===9?'input':'attack slot '+(o.group-1))+(o.stale?' (stale)':'');$('picker').append(option)}if(state.objects.some(o=>key(o)===old))$('picker').value=old;render()}catch(e){render();if(!state)$('status').textContent='Receiver unavailable'}setTimeout(poll,75)}
$('search').oninput=render;$('picker').onchange=render;$('browse').onchange=render;$('graph').onchange=render;setInterval(render,100);poll();
</script>"#;

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
