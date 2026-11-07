//! One observation's lazy, fixed-size fighter facts. Never retain native pointers
//! or reuse these facts across callbacks. Readiness queries remain live; the
//! separate floor cache only reuses an identical query inside this observation.
use std::cell::Cell;

/// Input proposals belong to exactly one selected-controller update. Wall-clock
/// freshness is only a secondary timeout; it cannot make an old packet current.
pub fn input_update_current(evaluated: Option<i64>, current: i64) -> bool {
    evaluated == Some(current)
}

/// A command callback may upgrade a mapper-only sample once, including when
/// no policy needs a roster. Discovery capability is distinct from roster data.
#[derive(Clone, Copy, Default)]
pub struct ObservationSetup {
    discovered: Option<i64>,
}
impl ObservationSetup {
    pub const fn new() -> Self {
        Self { discovered: None }
    }
    pub fn needed(&self, tick: i64, evaluated: bool, discover: bool) -> bool {
        !evaluated || (discover && self.discovered != Some(tick))
    }
    pub fn complete(&mut self, tick: i64, discover: bool) {
        if discover {
            self.discovered = Some(tick);
        }
    }
    pub fn clear(&mut self) {
        self.discovered = None;
    }
}

/// Exact floor-query reuse within ONE synchronous observation. Store copied
/// values, never a pointer, and discard at callback exit so moving platforms
/// and later frames always get new collision queries.
#[derive(Default)]
pub struct GroundSupportSnapshot {
    last: Cell<Option<(u32, [f32; 2], f32, [bool; 2])>>,
}
impl GroundSupportSnapshot {
    pub fn read(
        &self,
        object: u32,
        position: [f32; 2],
        vx: f32,
        query: impl FnOnce() -> [bool; 2],
    ) -> [bool; 2] {
        if let Some((id, at, speed, supported)) = self.last.get() {
            if id == object && at == position && speed == vx {
                return supported;
            }
        }
        let supported = query();
        self.last.set(Some((object, position, vx, supported)));
        supported
    }
}

#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Field {
    Status,
    Situation,
    Motion,
    Frame,
    Rate,
    X,
    Y,
    Vx,
    Vy,
    Facing,
    Team,
    Stop,
    PreviousStatus,
    EndFrame,
    CaptureLink,
    Kind,
}
const FIELDS: usize = Field::Kind as usize + 1;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Fact {
    Int(i32),
    Motion(u64),
    Float(f32),
    Bool(bool),
}
struct Fighter {
    id: Cell<Option<u32>>,
    facts: [Cell<Option<Fact>>; FIELDS],
}
/// At most eight fighter identities; an unexpected extra identity reads through.
/// Construction allocates no heap memory and does not read the game eagerly.
pub type ObservationSnapshot = ObservationSnapshotSized<8>;
pub struct ObservationSnapshotSized<const N: usize> {
    fighters: [Fighter; N],
    last: Cell<Option<usize>>,
}
impl<const N: usize> Default for ObservationSnapshotSized<N> {
    fn default() -> Self {
        Self {
            last: Cell::new(None),
            fighters: std::array::from_fn(|_| Fighter {
                id: Cell::new(None),
                facts: std::array::from_fn(|_| Cell::new(None)),
            }),
        }
    }
}
impl<const N: usize> ObservationSnapshotSized<N> {
    /// A copied motion clock from an already observed other fighter. No native
    /// calls or additional roster scan; passive frozen poses cannot time a tail.
    pub fn known_progress_clock(&self, own: u32) -> Option<(u32, i32, u64, f32, f32)> {
        self.fighters.iter().find_map(|f| {
            let id = f.id.get().filter(|id| *id != own)?;
            let Fact::Int(status) = f.facts[Field::Status as usize].get()? else {
                return None;
            };
            if matches!(status, 0x1b | 0x1c | 0x77 | 0x7e | 0x7f) {
                return None;
            }
            if f.facts[Field::Stop as usize].get() == Some(Fact::Bool(true)) {
                return None;
            }
            let Fact::Motion(motion) = f.facts[Field::Motion as usize].get()? else {
                return None;
            };
            let Fact::Float(frame) = f.facts[Field::Frame as usize].get()? else {
                return None;
            };
            let Fact::Float(rate) = f.facts[Field::Rate as usize].get()? else {
                return None;
            };
            (motion != 0 && frame.is_finite() && frame >= 0. && rate.is_finite() && rate > 0.)
                .then_some((id, status, motion, frame, rate))
        })
    }
    /// Already-read statuses only: no native calls, eager reads or allocation.
    /// Event consumers can distinguish a known end from a missing observation.
    pub fn known_statuses(&self) -> [Option<(u32, i32)>; N] {
        std::array::from_fn(|i| {
            let entry = &self.fighters[i];
            let id = entry.id.get()?;
            match entry.facts[Field::Status as usize].get()? {
                Fact::Int(status) => Some((id, status)),
                _ => None,
            }
        })
    }
    pub fn read(&self, id: u32, field: Field, read: impl FnOnce() -> Fact) -> Fact {
        // Most consumers read several fields from the same fighter in a row.
        // This cursor only speeds slot lookup; fact lifetime/overflow stay exact.
        let index = self
            .last
            .get()
            .filter(|i| self.fighters[*i].id.get() == Some(id))
            .or_else(|| self.fighters.iter().position(|e| e.id.get() == Some(id)))
            .or_else(|| self.fighters.iter().position(|e| e.id.get().is_none()));
        let Some(index) = index else {
            return read();
        };
        self.last.set(Some(index));
        let entry = &self.fighters[index];
        entry.id.set(Some(id));
        let fact = &entry.facts[field as usize];
        if let Some(value) = fact.get() {
            return value;
        }
        let value = read();
        fact.set(Some(value));
        value
    }
}

/// One synchronous observation's roster IDs. Empty/unavailable is cached too.
/// No native pointers, allocation, frame throttling, or cross-frame lifetime.
pub struct RosterSnapshot<const N: usize = 8> {
    value: Cell<Option<Option<[Option<u32>; N]>>>,
}
impl<const N: usize> Default for RosterSnapshot<N> {
    fn default() -> Self {
        Self {
            value: Cell::new(None),
        }
    }
}
impl<const N: usize> RosterSnapshot<N> {
    pub fn read(
        &self,
        query: impl FnOnce() -> Option<[Option<u32>; N]>,
    ) -> Option<[Option<u32>; N]> {
        if let Some(value) = self.value.get() {
            return value;
        }
        let value = query();
        self.value.set(Some(value));
        value
    }
}
