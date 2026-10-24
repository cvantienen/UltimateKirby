//! Compiled reference facts. Never fall back to Kirby for an unknown opponent.
//! Raw compound fields remain descriptive; only validated numbers drive policy.
mod generated;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub first: u16,
    pub last: u16,
}
#[derive(Clone, Copy, Debug)]
pub struct Stat {
    pub key: &'static str,
    pub value: &'static str,
}
#[derive(Clone, Copy, Debug)]
pub struct MoveData {
    pub name: &'static str,
    pub total: Option<u16>,
    pub landing_lag: Option<u16>,
    pub active: &'static [Span],
    pub motions: &'static [u64],
    /// Explicit conservative subset of uncomplicated grounded attacks only.
    pub whiff_eligible: bool,
    pub raw: &'static [Stat],
}
impl MoveData {
    pub fn last_active(&self) -> Option<u16> {
        self.active.iter().map(|x| x.last).max()
    }
    pub fn recovery(&self, o: RecoveryObservation) -> Option<f32> {
        let expected = match self.name {
            "Dash Attack" => 0x29,
            "Forward Tilt" => 0x2a,
            "Up Tilt" => 0x2b,
            "Down Tilt" => 0x2c,
            "Forward Smash" => 0x2f,
            "Down Smash" => 0x32,
            "Up Smash" => 0x35,
            _ => return None,
        };
        if !self.whiff_eligible
            || !self.motions.contains(&o.motion)
            || o.status != expected
            || !o.grounded
            || o.can_cancel
            || o.active_attack
            || o.registered_contact
            || !o.vulnerable
            || !o.frame.is_finite()
            || o.frame <= f32::from(self.last_active()?)
            || !o.end.is_finite()
            || (o.end - f32::from(self.total?)).abs() > 2.0
        {
            return None;
        }
        crate::ledge_trap::recovery_frames(
            o.frame,
            o.end.min(f32::from(self.total?)),
            o.cancel,
            o.rate,
        )
        .filter(|r| *r > 0.0)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct RecoveryObservation {
    pub motion: u64,
    pub status: i32,
    pub grounded: bool,
    pub frame: f32,
    pub rate: f32,
    pub end: f32,
    pub cancel: f32,
    pub can_cancel: bool,
    pub active_attack: bool,
    pub registered_contact: bool,
    pub vulnerable: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct Physics {
    pub weight: Option<f32>,
    pub walk_speed: Option<f32>,
    pub initial_dash: Option<f32>,
    pub run_speed: Option<f32>,
    pub air_speed: Option<f32>,
    pub gravity: Option<f32>,
    pub fall_speed: Option<f32>,
    pub fast_fall_speed: Option<f32>,
    pub full_hop_height: Option<f32>,
    pub short_hop_height: Option<f32>,
    pub air_accel_base: Option<f32>,
    pub air_accel_add: Option<f32>,
    pub air_accel_max: Option<f32>,
}
#[derive(Clone, Copy, Debug)]
pub struct Reference {
    pub kind: i32,
    pub name: &'static str,
    pub source: &'static str,
    pub physics: Physics,
    pub stats: &'static [Stat],
    pub moves: &'static [MoveData],
}
impl Reference {
    pub fn move_for_motion(&self, motion: u64) -> Option<&'static MoveData> {
        let mut found = self.moves.iter().filter(|m| m.motions.contains(&motion));
        let first = found.next()?;
        // A source page with several identically named variants needs a manual
        // mapping, not a guess based on its first/fastest row.
        found.next().is_none().then_some(first)
    }
    pub fn stat(&self, key: &str) -> Option<&'static str> {
        self.stats.iter().find(|s| s.key == key).map(|s| s.value)
    }
}
pub fn all() -> &'static [Reference] {
    generated::REFERENCES
}
pub fn profile(kind: i32) -> Option<&'static Reference> {
    let data = all();
    data.binary_search_by_key(&kind, |r| r.kind)
        .ok()
        .map(|i| &data[i])
}
/// Eight roster bindings, copied IDs/static references only; no native pointers.
#[derive(Clone, Copy, Default)]
pub struct OpponentProfiles {
    slots: [Option<(u32, i32, &'static Reference)>; 8],
}
impl OpponentProfiles {
    pub const fn new() -> Self {
        Self { slots: [None; 8] }
    }
    pub fn bind(&mut self, slot: usize, id: u32, kind: i32) -> Option<&'static Reference> {
        let entry = self.slots.get_mut(slot)?;
        if let Some((old_id, old_kind, reference)) = *entry {
            if old_id == id && old_kind == kind {
                return Some(reference);
            }
        }
        *entry = profile(kind).map(|r| (id, kind, r));
        entry.map(|(_, _, r)| r)
    }
    pub fn clear(&mut self, slot: usize) {
        if let Some(s) = self.slots.get_mut(slot) {
            *s = None;
        }
    }
}
