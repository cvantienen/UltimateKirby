//! One observation's lazy, fixed-size fighter facts. Never retain native pointers
//! or reuse these facts across callbacks. Readiness and collision queries remain
//! live; caching them could hide a transition immediately before an input.
use std::cell::Cell;

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
}
const FIELDS: usize = Field::EndFrame as usize + 1;
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
pub struct ObservationSnapshot {
    fighters: [Fighter; 8],
}
impl Default for ObservationSnapshot {
    fn default() -> Self {
        Self {
            fighters: std::array::from_fn(|_| Fighter {
                id: Cell::new(None),
                facts: std::array::from_fn(|_| Cell::new(None)),
            }),
        }
    }
}
impl ObservationSnapshot {
    pub fn read(&self, id: u32, field: Field, read: impl FnOnce() -> Fact) -> Fact {
        let entry = self
            .fighters
            .iter()
            .find(|e| e.id.get() == Some(id))
            .or_else(|| self.fighters.iter().find(|e| e.id.get().is_none()));
        let Some(entry) = entry else {
            return read();
        };
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
