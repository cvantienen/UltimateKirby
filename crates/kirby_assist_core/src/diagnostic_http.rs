use std::collections::VecDeque;
use std::io::{Read, Write};

pub const MAX_HEADER_BYTES: usize = 4096;
pub const MAX_BODY_BYTES: usize = 512 * 1024;
pub const MAX_LOG_ENTRIES: usize = 64;
pub const MAX_LOG_TEXT_BYTES: usize = 48 * 1024;
pub const MAX_LOG_EVENT_BYTES: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Put,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Status,
    Sequences,
    Logs { after: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: Method,
    pub route: Route,
    pub token: String,
    pub revision: Option<String>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpError {
    Cancelled,
    Io,
    Truncated,
    HeaderTooLarge,
    BodyTooLarge,
    Malformed,
    UnsupportedMethod,
    UnsupportedRoute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Json,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub content_type: ContentType,
    pub revision: Option<String>,
    pub body: Vec<u8>,
}

fn read_some<R: Read + ?Sized, F: FnMut() -> bool>(
    reader: &mut R,
    bytes: &mut [u8],
    allowed: &mut F,
) -> Result<usize, HttpError> {
    loop {
        if !allowed() {
            return Err(HttpError::Cancelled);
        }
        match reader.read(bytes) {
            Ok(0) => return Err(HttpError::Truncated),
            Ok(n) => return Ok(n),
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return Err(HttpError::Io),
        }
    }
}

pub fn read_request<R: Read + ?Sized, F: FnMut() -> bool>(
    reader: &mut R,
    mut allowed: F,
) -> Result<Request, HttpError> {
    let mut header = Vec::with_capacity(MAX_HEADER_BYTES);
    let mut byte = [0];
    while !header.ends_with(b"\r\n\r\n") {
        if header.len() == MAX_HEADER_BYTES {
            return Err(HttpError::HeaderTooLarge);
        }
        read_some(reader, &mut byte, &mut allowed)?;
        header.push(byte[0]);
    }
    let header = std::str::from_utf8(&header).map_err(|_| HttpError::Malformed)?;
    let mut lines = header[..header.len() - 4].split("\r\n");
    let mut first = lines.next().ok_or(HttpError::Malformed)?.split(' ');
    let method = match first.next() {
        Some("GET") => Method::Get,
        Some("PUT") => Method::Put,
        Some(_) => return Err(HttpError::UnsupportedMethod),
        None => return Err(HttpError::Malformed),
    };
    let target = first.next().ok_or(HttpError::Malformed)?;
    if !matches!(first.next(), Some("HTTP/1.1" | "HTTP/1.0")) || first.next().is_some() {
        return Err(HttpError::Malformed);
    }
    let route = match (method, target) {
        (Method::Get, "/status") => Route::Status,
        (_, "/sequences") => Route::Sequences,
        (Method::Get, target) if target.starts_with("/logs?after=") => {
            let after = decimal(&target[12..]).ok_or(HttpError::Malformed)?;
            Route::Logs { after }
        }
        _ => return Err(HttpError::UnsupportedRoute),
    };
    let mut token = None;
    let mut revision = None;
    let mut length = None;
    for line in lines {
        let (name, value) = line.split_once(':').ok_or(HttpError::Malformed)?;
        if name.is_empty() || !name.bytes().all(header_name_byte) {
            return Err(HttpError::Malformed);
        }
        if !value.bytes().all(|b| b == b'\t' || (32..=126).contains(&b)) {
            return Err(HttpError::Malformed);
        }
        let value = value.trim_matches([' ', '\t']);
        if name.eq_ignore_ascii_case("transfer-encoding") {
            return Err(HttpError::Malformed);
        } else if name.eq_ignore_ascii_case("content-length") {
            if length.is_some() {
                return Err(HttpError::Malformed);
            }
            let parsed = decimal(value).ok_or(HttpError::Malformed)?;
            if parsed > MAX_BODY_BYTES as u64 {
                return Err(HttpError::BodyTooLarge);
            }
            length = Some(parsed as usize);
        } else if name.eq_ignore_ascii_case("x-kirby-token") {
            if token.is_some() || value.is_empty() {
                return Err(HttpError::Malformed);
            }
            token = Some(value.to_owned());
        } else if name.eq_ignore_ascii_case("x-kirby-revision") {
            if revision.is_some() || value.is_empty() {
                return Err(HttpError::Malformed);
            }
            revision = Some(value.to_owned());
        }
    }
    let length = match (method, length) {
        (Method::Put, None) => return Err(HttpError::Malformed),
        (Method::Get, Some(n)) if n != 0 => return Err(HttpError::Malformed),
        (_, n) => n.unwrap_or(0),
    };
    let token = token.ok_or(HttpError::Malformed)?;
    let mut body = vec![0; length];
    let mut offset = 0;
    while offset < length {
        offset += read_some(reader, &mut body[offset..], &mut allowed)?;
    }
    Ok(Request {
        method,
        route,
        token,
        revision,
        body,
    })
}

fn header_name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b)
}

fn decimal(value: &str) -> Option<u64> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

fn write_all<W: Write + ?Sized, F: FnMut() -> bool>(
    writer: &mut W,
    mut bytes: &[u8],
    allowed: &mut F,
) -> Result<(), HttpError> {
    while !bytes.is_empty() {
        if !allowed() {
            return Err(HttpError::Cancelled);
        }
        match writer.write(bytes) {
            Ok(0) => return Err(HttpError::Io),
            Ok(n) => bytes = &bytes[n..],
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return Err(HttpError::Io),
        }
    }
    Ok(())
}

pub fn write_response<W: Write + ?Sized, F: FnMut() -> bool>(
    writer: &mut W,
    response: &Response,
    mut allowed: F,
) -> Result<(), HttpError> {
    if response.body.len() > MAX_BODY_BYTES {
        return Err(HttpError::BodyTooLarge);
    }
    let reason = match response.status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        409 => "Conflict",
        413 => "Payload Too Large",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => return Err(HttpError::Malformed),
    };
    let content_type = match response.content_type {
        ContentType::Json => "application/json; charset=utf-8",
        ContentType::Text => "text/plain; charset=utf-8",
    };
    if response
        .revision
        .as_ref()
        .is_some_and(|value| value.len() > MAX_HEADER_BYTES - 256)
    {
        return Err(HttpError::HeaderTooLarge);
    }
    let mut header = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
        response.status,
        reason,
        content_type,
        response.body.len()
    );
    if let Some(revision) = &response.revision {
        if revision.is_empty() || !revision.bytes().all(|b| (33..=126).contains(&b)) {
            return Err(HttpError::Malformed);
        }
        header.push_str("X-Kirby-Revision: ");
        header.push_str(revision);
        header.push_str("\r\n");
    }
    header.push_str("\r\n");
    if header.len() > MAX_HEADER_BYTES {
        return Err(HttpError::HeaderTooLarge);
    }
    write_all(writer, header.as_bytes(), &mut allowed)?;
    write_all(writer, &response.body, &mut allowed)
}

pub fn constant_time_eq(left: &str, right: &str) -> bool {
    let mut diff = left.len() ^ right.len();
    for i in 0..left.len().max(right.len()) {
        diff |= (left.as_bytes().get(i).copied().unwrap_or(0)
            ^ right.as_bytes().get(i).copied().unwrap_or(0)) as usize;
    }
    diff == 0
}

/// A deterministic optimistic-concurrency token, not an authentication hash.
pub fn revision(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

pub fn json_string(text: &str) -> String {
    let mut json = String::with_capacity(text.len() + 2);
    json.push('"');
    for ch in text.chars() {
        match ch {
            '"' => json.push_str("\\\""),
            '\\' => json.push_str("\\\\"),
            '\n' => json.push_str("\\n"),
            '\r' => json.push_str("\\r"),
            '\t' => json.push_str("\\t"),
            ch if ch.is_control() => {
                for unit in ch.encode_utf16(&mut [0; 2]) {
                    json.push_str(&format!("\\u{unit:04x}"));
                }
            }
            ch => json.push(ch),
        }
    }
    json.push('"');
    json
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub id: u64,
    pub text: String,
}

#[derive(Debug, Default)]
pub struct LogBuffer {
    entries: VecDeque<LogEntry>,
    text_bytes: usize,
    wire_bytes: usize,
    latest: u64,
    dropped: u64,
}

impl LogBuffer {
    pub fn push(&mut self, text: &str) -> u64 {
        let mut end = text.len().min(MAX_LOG_EVENT_BYTES);
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        let text = text[..end].to_owned();
        self.latest = self.latest.saturating_add(1);
        let wire_bytes = json_string(&text).len() + 48;
        while self.entries.len() >= MAX_LOG_ENTRIES
            || self.text_bytes + text.len() > MAX_LOG_TEXT_BYTES
            || self.wire_bytes + wire_bytes > 60 * 1024
        {
            if let Some(entry) = self.entries.pop_front() {
                self.text_bytes -= entry.text.len();
                self.wire_bytes -= json_string(&entry.text).len() + 48;
                self.dropped = self.dropped.saturating_add(1);
            }
        }
        self.text_bytes += text.len();
        self.wire_bytes += wire_bytes;
        self.entries.push_back(LogEntry {
            id: self.latest,
            text,
        });
        self.latest
    }

    pub fn logs_json(&self, after: u64) -> String {
        let mut json = format!(
            "{{\"next\":{},\"dropped\":{},\"entries\":[",
            self.latest, self.dropped
        );
        let mut first = true;
        for entry in self
            .entries
            .iter()
            .filter(|e| after > self.latest || e.id > after)
        {
            let encoded = format!(
                "{{\"id\":{},\"text\":{}}}",
                entry.id,
                json_string(&entry.text)
            );
            if !first {
                json.push(',');
            }
            first = false;
            json.push_str(&encoded);
        }
        json.push_str("]}");
        json
    }
}
