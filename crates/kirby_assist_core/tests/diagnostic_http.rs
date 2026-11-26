#![cfg(feature = "diagnostic-logging")]

use kirby_assist_core::diagnostic_http::*;
use std::io::{self, Cursor, Read, Write};

fn parse(bytes: &[u8]) -> Result<Request, HttpError> {
    read_request(&mut Cursor::new(bytes), || true)
}

#[test]
fn routes_and_binary_body_are_preserved() {
    let request = parse(b"PUT /sequences HTTP/1.1\r\nX-Kirby-Token: secret\r\nX-Kirby-Revision: abc\r\nContent-Length: 3\r\n\r\n\xff\x00x").unwrap();
    assert_eq!(request.method, Method::Put);
    assert_eq!(request.route, Route::Sequences);
    assert_eq!(request.token, "secret");
    assert_eq!(request.revision.as_deref(), Some("abc"));
    assert_eq!(request.body, [255, 0, b'x']);
    for (path, route) in [
        ("/status", Route::Status),
        ("/sequences", Route::Sequences),
        (
            "/logs?after=18446744073709551615",
            Route::Logs { after: u64::MAX },
        ),
    ] {
        let wire = format!("GET {path} HTTP/1.1\r\nx-kirby-token: secret\r\n\r\n");
        assert_eq!(parse(wire.as_bytes()).unwrap().route, route);
    }
}

#[test]
fn rejects_ambiguous_headers_and_request_framing() {
    for headers in [
        "X-Kirby-Token: a\r\nx-kirby-token: b\r\nContent-Length: 0",
        "X-Kirby-Token: a\r\nX-Kirby-Revision: a\r\nx-kirby-revision: a\r\nContent-Length: 0",
        "X-Kirby-Token: a\r\nContent-Length: 0\r\ncontent-length: 0",
        "X-Kirby-Token: a\r\nTransfer-Encoding: chunked\r\nContent-Length: 0",
        "X-Kirby-Token: a",
        "X-Kirby-Token: a\r\nContent-Length: +1",
        "X-Kirby-Token: a\r\nContent-Length: -1",
        "X-Kirby-Token: a\r\nContent-Length: 18446744073709551616",
        "X-Kirby-Token: a\r\n folded: x\r\nContent-Length: 0",
        "X-Kirby-Token: a\r\nInvalid\r\nContent-Length: 0",
        "Content-Length: 0",
    ] {
        let wire = format!("PUT /sequences HTTP/1.1\r\n{headers}\r\n\r\n");
        assert_eq!(
            parse(wire.as_bytes()),
            Err(HttpError::Malformed),
            "{headers}"
        );
    }
    assert_eq!(
        parse(b"GET /status HTTP/1.1\r\nX-Kirby-Token: a\r\nContent-Length: 1\r\n\r\nx"),
        Err(HttpError::Malformed)
    );
}

#[test]
fn rejects_unsupported_methods_and_paths() {
    for path in [
        "/",
        "/logs",
        "/logs?after=-1",
        "/logs?after=1&x=2",
        "/logs?after=",
        "/status/",
        "/%73tatus",
        "/../status",
        "http://host/status",
    ] {
        let wire = format!("GET {path} HTTP/1.1\r\nX-Kirby-Token: a\r\n\r\n");
        assert!(parse(wire.as_bytes()).is_err(), "{path}");
    }
    assert_eq!(
        parse(b"POST /sequences HTTP/1.1\r\n\r\n"),
        Err(HttpError::UnsupportedMethod)
    );
    assert_eq!(
        parse(b"PUT /status HTTP/1.1\r\n\r\n"),
        Err(HttpError::UnsupportedRoute)
    );
    assert_eq!(
        parse(b"GET  /status HTTP/1.1\r\n\r\n"),
        Err(HttpError::Malformed)
    );
}

#[test]
fn bounded_headers_and_bodies_and_truncation() {
    assert_eq!(
        parse(&vec![b'x'; MAX_HEADER_BYTES + 1]),
        Err(HttpError::HeaderTooLarge)
    );
    let oversized = format!(
        "PUT /sequences HTTP/1.1\r\nX-Kirby-Token: a\r\nContent-Length: {}\r\n\r\n",
        MAX_BODY_BYTES + 1
    );
    assert_eq!(parse(oversized.as_bytes()), Err(HttpError::BodyTooLarge));
    assert_eq!(
        parse(b"GET /status HTTP/1.1\r\n"),
        Err(HttpError::Truncated)
    );
    assert_eq!(
        parse(b"PUT /sequences HTTP/1.1\r\nX-Kirby-Token: a\r\nContent-Length: 3\r\n\r\nx"),
        Err(HttpError::Truncated)
    );
    let prefix = format!(
        "PUT /sequences HTTP/1.1\r\nX-Kirby-Token: a\r\nContent-Length: {MAX_BODY_BYTES}\r\n\r\n"
    );
    let mut wire = prefix.into_bytes();
    wire.resize(wire.len() + MAX_BODY_BYTES, 0);
    assert_eq!(parse(&wire).unwrap().body.len(), MAX_BODY_BYTES);
}

struct SingleByteIo {
    input: Cursor<Vec<u8>>,
    output: Vec<u8>,
    reads: usize,
    writes: usize,
}
impl Read for SingleByteIo {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reads += 1;
        self.input.read(&mut buf[..1])
    }
}
impl Write for SingleByteIo {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writes += 1;
        self.output.push(buf[0]);
        Ok(1)
    }
    fn flush(&mut self) -> io::Result<()> {
        panic!("must not flush without a gate")
    }
}
fn io(input: &[u8]) -> SingleByteIo {
    SingleByteIo {
        input: Cursor::new(input.to_vec()),
        output: Vec::new(),
        reads: 0,
        writes: 0,
    }
}
fn response() -> Response {
    Response {
        status: 200,
        content_type: ContentType::Json,
        revision: Some("123abc".into()),
        body: b"{}".to_vec(),
    }
}

#[test]
fn gate_cancels_header_body_and_partial_writes_before_io() {
    let wire = b"PUT /sequences HTTP/1.1\r\nX-Kirby-Token: a\r\nContent-Length: 3\r\n\r\nabc";
    for limit in [0, 3, wire.len() - 2] {
        let mut stream = io(wire);
        let mut calls = 0;
        assert_eq!(
            read_request(&mut stream, || {
                calls += 1;
                calls <= limit
            }),
            Err(HttpError::Cancelled)
        );
        assert_eq!(stream.reads, limit);
    }
    let mut stream = io(&[]);
    let mut calls = 0;
    assert_eq!(
        write_response(&mut stream, &response(), || {
            calls += 1;
            calls <= 12
        }),
        Err(HttpError::Cancelled)
    );
    assert_eq!(stream.writes, 12);
    let mut complete = io(&[]);
    write_response(&mut complete, &response(), || true).unwrap();
    let mut stream = io(&[]);
    let mut calls = 0;
    let limit = complete.output.len() - 1;
    assert_eq!(
        write_response(&mut stream, &response(), || {
            calls += 1;
            calls <= limit
        }),
        Err(HttpError::Cancelled)
    );
    assert_eq!(stream.writes, limit);
}

#[test]
fn response_is_framed_and_header_injection_rejected() {
    let mut out = Vec::new();
    write_response(&mut out, &response(), || true).unwrap();
    let wire = String::from_utf8(out).unwrap();
    assert!(wire.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(wire.contains("Content-Length: 2\r\nConnection: close\r\n"));
    assert!(wire.ends_with("X-Kirby-Revision: 123abc\r\n\r\n{}"));
    let mut bad = response();
    bad.revision = Some("abc\r\nInjected: yes".into());
    let mut out = Vec::new();
    assert_eq!(
        write_response(&mut out, &bad, || true),
        Err(HttpError::Malformed)
    );
    assert!(out.is_empty());
    bad.revision = Some("a".repeat(MAX_HEADER_BYTES));
    assert_eq!(
        write_response(&mut out, &bad, || true),
        Err(HttpError::HeaderTooLarge)
    );
    bad.revision = None;
    bad.body.resize(MAX_BODY_BYTES + 1, 0);
    assert_eq!(
        write_response(&mut out, &bad, || true),
        Err(HttpError::BodyTooLarge)
    );
    assert!(out.is_empty());
}

#[test]
fn token_comparison_and_revision() {
    assert!(constant_time_eq("secret", "secret"));
    for other in ["Secret", "secre", "secret\0", "xxxxxx", ""] {
        assert!(!constant_time_eq("secret", other));
    }
    assert_eq!(revision(b""), "cbf29ce484222325");
    assert_eq!(revision(b"hello"), "a430d84680aabd0b");
}

#[test]
fn log_unicode_escaping_truncation_and_restart_cursor() {
    let mut logs = LogBuffer::default();
    let text = "quote\" slash\\ newline\n tab\t null\0 del\u{7f} nel\u{85} emoji😀";
    logs.push(text);
    let json: serde_json::Value = serde_json::from_str(&logs.logs_json(0)).unwrap();
    assert_eq!(json["entries"][0]["text"], text);
    assert_eq!(json["next"], 1);
    assert!(logs.logs_json(1).contains("\"entries\":[]"));
    assert_eq!(logs.logs_json(99), logs.logs_json(0));
    let long = format!("{}😀", "a".repeat(MAX_LOG_EVENT_BYTES - 1));
    logs.push(&long);
    let json: serde_json::Value = serde_json::from_str(&logs.logs_json(1)).unwrap();
    assert_eq!(
        json["entries"][0]["text"].as_str().unwrap().len(),
        MAX_LOG_EVENT_BYTES - 1
    );
}

#[test]
fn log_retention_and_wire_size_account_for_escaping() {
    let mut logs = LogBuffer::default();
    for n in 1..=70 {
        assert_eq!(logs.push("x"), n);
    }
    let json: serde_json::Value = serde_json::from_str(&logs.logs_json(0)).unwrap();
    assert_eq!(json["dropped"], 6);
    assert_eq!(json["entries"].as_array().unwrap().len(), MAX_LOG_ENTRIES);
    assert_eq!(json["entries"][0]["id"], 7);
    for text in [
        "x".repeat(MAX_LOG_EVENT_BYTES),
        "\0".repeat(MAX_LOG_EVENT_BYTES),
    ] {
        let mut logs = LogBuffer::default();
        for _ in 0..64 {
            logs.push(&text);
        }
        let wire = logs.logs_json(0);
        assert!(wire.len() < 64 * 1024);
        let json: serde_json::Value = serde_json::from_str(&wire).unwrap();
        let entries = json["entries"].as_array().unwrap();
        let bytes: usize = entries
            .iter()
            .map(|e| e["text"].as_str().unwrap().len())
            .sum();
        assert!(bytes <= MAX_LOG_TEXT_BYTES);
        assert_eq!(entries.last().unwrap()["id"], json["next"]);
        assert_eq!(json["dropped"].as_u64().unwrap() + entries.len() as u64, 64);
    }
}
