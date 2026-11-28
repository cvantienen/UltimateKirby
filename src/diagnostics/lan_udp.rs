//! Bounded UDP stop-and-wait transport for gated editor requests.
use super::{authorized, network::socket_result, Config, ACTIVATION};
use std::sync::atomic::Ordering;
use std::{
    io,
    net::{SocketAddr, UdpSocket},
    time::{Duration, Instant},
};

const CHUNK: usize = 1024;
const HEADER: usize = 17;
const MAX_CHUNKS: usize = 520;
const MAX_REQUEST: usize = kirby_assist_core::diagnostic_http::MAX_BODY_BYTES
    + kirby_assist_core::diagnostic_http::MAX_HEADER_BYTES;
const LIFETIME: Duration = Duration::from_secs(12);

struct Packet<'a> {
    kind: u8,
    id: u64,
    index: usize,
    total: usize,
    payload: &'a [u8],
}
impl<'a> Packet<'a> {
    fn parse(bytes: &'a [u8]) -> Option<Self> {
        if bytes.len() < HEADER || bytes.len() > HEADER + CHUNK || &bytes[..4] != b"KALE" {
            return None;
        }
        let packet = Self {
            kind: bytes[4],
            id: u64::from_be_bytes(bytes[5..13].try_into().ok()?),
            index: u16::from_be_bytes(bytes[13..15].try_into().ok()?) as usize,
            total: u16::from_be_bytes(bytes[15..17].try_into().ok()?) as usize,
            payload: &bytes[HEADER..],
        };
        (packet.id != 0).then_some(packet)
    }
}

struct Transfer {
    peer: SocketAddr,
    id: u64,
    total: usize,
    received: usize,
    request: Vec<u8>,
    response: Option<Vec<u8>>,
    response_finished: bool,
    response_sent: usize,
    started: Instant,
}
impl Transfer {
    fn matches(&self, peer: SocketAddr, packet: &Packet<'_>) -> bool {
        self.peer == peer && self.id == packet.id
    }

    fn upload(&mut self, packet: &Packet<'_>) -> bool {
        if packet.total != self.total
            || packet.index >= self.total
            || packet.payload.is_empty()
            || (packet.index + 1 < packet.total && packet.payload.len() != CHUNK)
        {
            return false;
        }
        if packet.index < self.received {
            let start = packet.index * CHUNK;
            let end = if packet.index + 1 == self.total {
                self.request.len()
            } else {
                start + CHUNK
            };
            return self.request.get(start..end) == Some(packet.payload);
        }
        if packet.index != self.received || self.request.len() + packet.payload.len() > MAX_REQUEST
        {
            return false;
        }
        self.request.extend_from_slice(packet.payload);
        self.received += 1;
        true
    }
}

pub(super) fn send(
    socket: &UdpSocket,
    peer: SocketAddr,
    kind: u8,
    id: u64,
    index: usize,
    total: usize,
    payload: &[u8],
    generation: u64,
) -> io::Result<()> {
    if payload.len() > CHUNK || index > u16::MAX as usize || total > MAX_CHUNKS || id == 0 {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    let mut bytes = [0u8; HEADER + CHUNK];
    bytes[..4].copy_from_slice(b"KALE");
    bytes[4] = kind;
    bytes[5..13].copy_from_slice(&id.to_be_bytes());
    bytes[13..15].copy_from_slice(&(index as u16).to_be_bytes());
    bytes[15..17].copy_from_slice(&(total as u16).to_be_bytes());
    bytes[HEADER..HEADER + payload.len()].copy_from_slice(payload);
    if !authorized(generation) {
        return Err(io::ErrorKind::PermissionDenied.into());
    }
    match socket_result(socket.send_to(&bytes[..HEADER + payload.len()], peer)) {
        Ok(sent) if sent == HEADER + payload.len() => Ok(()),
        Ok(_) => Err(io::ErrorKind::WriteZero.into()),
        Err(error) => Err(error),
    }
}

fn execute(request: &[u8], config: &Config, generation: u64) -> Vec<u8> {
    let started = Instant::now();
    let allowed = || authorized(generation) && started.elapsed() < Duration::from_secs(2);
    let mut input = io::Cursor::new(request);
    let parsed = kirby_assist_core::diagnostic_http::read_request(&mut input, allowed);
    let mut output = Vec::new();
    match parsed {
        Ok(request) if input.position() as usize == input.get_ref().len() => {
            let _ = super::dispatch(&mut output, config, request, started, generation);
        }
        _ => {
            let _ = super::respond(
                &mut output,
                400,
                b"{\"error\":\"Invalid editor request\"}",
                None,
                allowed,
            );
        }
    }
    output
}

pub(super) fn serve(socket: UdpSocket, config: &Config) -> Result<(), String> {
    let mut active: Option<Transfer> = None;
    // Never forget completed IDs during an activation: delayed uploads must not resave.
    let mut completed = std::collections::HashSet::<(SocketAddr, u64)>::new();
    let mut bytes = [0u8; HEADER + CHUNK + 1];
    let generation = ACTIVATION.load(Ordering::Acquire);
    while authorized(generation) {
        if active
            .as_ref()
            .is_some_and(|t| t.started.elapsed() >= LIFETIME)
        {
            active = None;
        }
        if !authorized(generation) {
            break;
        }
        let (length, peer) = match socket_result(socket.recv_from(&mut bytes)) {
            Ok(value) => value,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(1));
                continue;
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) if !authorized(generation) => break,
            Err(error) => return Err(format!("UDP editor receive: {error}")),
        };
        if !authorized(generation) {
            break;
        }
        let SocketAddr::V4(address) = peer else {
            continue;
        };
        if !config.permits(*address.ip()) {
            continue;
        }
        let Some(packet) = Packet::parse(&bytes[..length]) else {
            continue;
        };
        if packet.kind == 1 {
            if packet.total == 0 || packet.total > MAX_CHUNKS || packet.index >= packet.total {
                continue;
            }
            let matching = active.as_ref().is_some_and(|t| t.matches(peer, &packet));
            if !matching {
                if completed.contains(&(peer, packet.id))
                    || packet.index != 0
                    || packet.payload.is_empty()
                    || (packet.total > 1 && packet.payload.len() != CHUNK)
                    || active.as_ref().is_some_and(|t| !t.response_finished)
                {
                    continue;
                }
                if completed.len() >= 4096 {
                    return Err("UDP replay protection full; reactivate Training LAN".into());
                }
                active = Some(Transfer {
                    peer,
                    id: packet.id,
                    total: packet.total,
                    received: 0,
                    request: Vec::new(),
                    response: None,
                    response_finished: false,
                    response_sent: 0,
                    started: Instant::now(),
                });
            }
            let transfer = active.as_mut().unwrap();
            if !transfer.upload(&packet) {
                continue;
            }
            let _ = send(
                &socket,
                peer,
                2,
                packet.id,
                packet.index,
                packet.total,
                &[],
                generation,
            );
            if transfer.received == transfer.total && transfer.response.is_none() {
                completed.insert((peer, packet.id));
                transfer.response = Some(execute(&transfer.request, config, generation));
            }
        } else if packet.kind == 5 && packet.total == 0 && packet.payload.is_empty() {
            if let Some(transfer) = active.as_mut().filter(|t| t.matches(peer, &packet)) {
                if transfer.response.as_ref().is_some_and(|response| {
                    transfer.response_sent == response.len().div_ceil(CHUNK)
                        && packet.index == transfer.response_sent
                }) {
                    transfer.response_finished = true;
                    let _ = send(
                        &socket,
                        peer,
                        6,
                        packet.id,
                        packet.index,
                        0,
                        &[],
                        generation,
                    );
                }
            }
        } else if packet.kind == 3 && packet.total == 0 && packet.payload.is_empty() {
            if let Some(transfer) = active.as_mut().filter(|t| t.matches(peer, &packet)) {
                if let Some(response) = &transfer.response {
                    let total = response.len().div_ceil(CHUNK);
                    if packet.index < total && packet.index <= transfer.response_sent {
                        let start = packet.index * CHUNK;
                        let end = (start + CHUNK).min(response.len());
                        let sent = send(
                            &socket,
                            peer,
                            4,
                            packet.id,
                            packet.index,
                            total,
                            &response[start..end],
                            generation,
                        );
                        if sent.is_ok() {
                            transfer.response_sent = transfer.response_sent.max(packet.index + 1);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet(kind: u8, id: u64, index: u16, total: u16, payload: &[u8]) -> Vec<u8> {
        let mut bytes = b"KALE".to_vec();
        bytes.push(kind);
        bytes.extend(id.to_be_bytes());
        bytes.extend(index.to_be_bytes());
        bytes.extend(total.to_be_bytes());
        bytes.extend(payload);
        bytes
    }

    #[test]
    fn outgoing_oversize_is_rejected_without_panicking() {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        assert_eq!(
            send(
                &socket,
                socket.local_addr().unwrap(),
                4,
                1,
                0,
                1,
                &[0; CHUNK + 1],
                0
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn upload_rejects_gaps_conflicting_retries_and_oversize() {
        let mut transfer = Transfer {
            peer: "127.0.0.1:4242".parse().unwrap(),
            id: 1,
            total: 2,
            received: 0,
            request: Vec::new(),
            response: None,
            response_finished: false,
            response_sent: 0,
            started: Instant::now(),
        };
        let gap = packet(1, 1, 1, 2, b"end");
        assert!(!transfer.upload(&Packet::parse(&gap).unwrap()));
        let first = packet(1, 1, 0, 2, &[b'a'; CHUNK]);
        assert!(transfer.upload(&Packet::parse(&first).unwrap()));
        assert!(transfer.upload(&Packet::parse(&first).unwrap()));
        assert_eq!(transfer.received, 1);
        let changed = packet(1, 1, 0, 2, &[b'b'; CHUNK]);
        assert!(!transfer.upload(&Packet::parse(&changed).unwrap()));
        assert!(transfer.upload(&Packet::parse(&gap).unwrap()));
        assert_eq!(transfer.request.len(), CHUNK + 3);
        let shortened = packet(1, 1, 1, 2, b"en");
        assert!(!transfer.upload(&Packet::parse(&shortened).unwrap()));
        assert!(Packet::parse(&packet(1, 0, 0, 1, b"x")).is_none());
        assert!(Packet::parse(&packet(1, 1, 0, 1, &[0; CHUNK + 1])).is_none());
    }
}
