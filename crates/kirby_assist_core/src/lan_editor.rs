//! Fail-closed admission policy for a future Training-only LAN editor.
//!
//! This module performs no network I/O. Native integration must supply fresh,
//! read-only room and route observations and recheck admission before every I/O
//! operation. Private addressing alone is not proof of LAN isolation. Router
//! egress rules are still needed to enforce a network-level boundary.
use std::{net::Ipv4Addr, time::Duration};

const OBSERVATION_LIFETIME: Duration = Duration::from_millis(100);

/// A numeric IPv4 destination; hostnames, IPv6 and public addresses are rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Endpoint {
    address: Ipv4Addr,
    port: u16,
}

impl Endpoint {
    pub fn parse(address: &str, port: u16) -> Result<Self, &'static str> {
        let address = address
            .parse::<Ipv4Addr>()
            .map_err(|_| "Use a numeric private IPv4 address, not a hostname or URL")?;
        if !address.is_private() || port == 0 {
            return Err("A private IPv4 address and a nonzero port are required");
        }
        Ok(Self { address, port })
    }

    pub fn address(self) -> Ipv4Addr {
        self.address
    }

    pub fn port(self) -> u16 {
        self.port
    }
}

/// Platform-supplied facts, not permission to connect. Missing or unverified
/// route/interface facts must be represented as `None`, never inferred from IPs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DirectRoute {
    pub interface: u64,
    pub local_address: Ipv4Addr,
    pub prefix_length: u8,
    pub destination: Ipv4Addr,
    pub uses_gateway: bool,
    pub interface_is_lan: bool,
}

impl DirectRoute {
    fn admits(self, endpoint: Endpoint) -> bool {
        let private_prefix = match self.local_address.octets() {
            [10, ..] => 8,
            [172, second, ..] if (16..=31).contains(&second) => 12,
            [192, 168, ..] => 16,
            _ => return false,
        };
        if !self.interface_is_lan
            || self.uses_gateway
            || self.destination != endpoint.address
            || !(private_prefix..=30).contains(&self.prefix_length)
        {
            return false;
        }
        let mask = u32::MAX << (32 - self.prefix_length);
        let local = u32::from(self.local_address);
        let peer = u32::from(endpoint.address);
        let network = local & mask;
        let broadcast = network | !mask;
        peer & mask == network
            && peer != local
            && peer != network
            && peer != broadcast
            && local != network
            && local != broadcast
    }
}

/// Fresh facts from the native owner. Room identity must change on every entry,
/// including leaving and returning to Training with the same fighter/stage.
#[derive(Clone, Copy, Debug)]
pub struct Observation {
    pub observed_at: Duration,
    pub training_room: Option<u64>,
    pub training_menu_open: bool,
    pub route: Option<DirectRoute>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Admission {
    room: u64,
    endpoint: Endpoint,
    route: DirectRoute,
}

/// Identifies one explicit activation. It is not a transferable I/O permission;
/// the owning guard must validate it against fresh observations for every use.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ticket {
    generation: u64,
    admission: Admission,
}

/// Disabled at construction. Expiry, scene/route changes, manual disconnect and
/// transport failure revoke activation rather than scheduling a reconnect.
#[derive(Debug, Default)]
pub struct Guard {
    generation: u64,
    active: Option<Admission>,
    last_observed_at: Option<Duration>,
}

impl Guard {
    /// Call only for a deliberate native Training-menu activation action.
    /// The transport remains responsible for pairing and bounded I/O.
    pub fn activate(
        &mut self,
        endpoint: Endpoint,
        observation: Observation,
        now: Duration,
    ) -> Result<Ticket, &'static str> {
        self.revoke();
        if !observation.training_menu_open {
            return Err("Activate LAN editing explicitly inside the Training menu");
        }
        let admission = Self::admission(endpoint, observation, now)
            .ok_or("Fresh Training-room and directly attached LAN route facts are required")?;
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or("LAN editor activation counter exhausted")?;
        self.active = Some(admission);
        self.last_observed_at = Some(observation.observed_at);
        Ok(Ticket {
            generation: self.generation,
            admission,
        })
    }

    /// Recheck immediately before any connect/read/send. A failed check closes
    /// the logical session permanently; a new menu activation is required.
    pub fn allows_io(&mut self, ticket: Ticket, observation: Observation, now: Duration) -> bool {
        let current = Self::admission(ticket.admission.endpoint, observation, now);
        let valid = self.active == Some(ticket.admission)
            && self.generation == ticket.generation
            && current == self.active
            && self
                .last_observed_at
                .is_some_and(|last| observation.observed_at >= last);
        if valid {
            self.last_observed_at = Some(observation.observed_at);
        } else {
            self.revoke();
        }
        valid
    }

    /// First performs the I/O gate; editing additionally requires the Training
    /// menu to be open. Pause-only editing is unsupported until pause is verified.
    pub fn allows_edit(&mut self, ticket: Ticket, observation: Observation, now: Duration) -> bool {
        self.allows_io(ticket, observation, now) && observation.training_menu_open
    }

    /// Also call on native Training exit, disconnect, timeout or transport error.
    /// This is logical revocation, not a way to recall OS-queued packets.
    pub fn revoke(&mut self) {
        self.active = None;
        self.last_observed_at = None;
    }

    fn admission(endpoint: Endpoint, observation: Observation, now: Duration) -> Option<Admission> {
        let age = now.checked_sub(observation.observed_at)?;
        if age >= OBSERVATION_LIFETIME {
            return None;
        }
        let room = observation.training_room?;
        let route = observation.route?;
        route.admits(endpoint).then_some(Admission {
            room,
            endpoint,
            route,
        })
    }
}

#[cfg(test)]
#[path = "../tests/unit/lan_editor.rs"]
mod tests;
