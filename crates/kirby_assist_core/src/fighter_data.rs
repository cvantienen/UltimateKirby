//! Compiled reference facts. Never fall back to Kirby for an unknown opponent.
//! Raw compound fields remain descriptive; only validated numbers drive policy.
mod generated;
use std::sync::OnceLock;
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrabReference {
    pub first: u8,
    pub last: u8,
    pub reach: f32,
}

type GrabFacts = (i32, [Option<GrabReference>; 3]);
const GRAB_KINDS: usize = generated::REFERENCES.len();
static GRAB_FACTS: OnceLock<[GrabFacts; GRAB_KINDS]> = OnceLock::new();

/// Startup-only preparation of immutable reference values. Live observations
/// still supply current status, motion, facing, positions and readiness.
pub fn prepare_grab_references() {
    GRAB_FACTS.get_or_init(|| {
        std::array::from_fn(|index| {
            let reference = &all()[index];
            (
                reference.kind,
                [0x37, 0x39, 0x3b].map(|status| ordinary_grab_uncached(reference.kind, status)),
            )
        })
    });
}

/// Validated ordinary standing/dash/pivot grab facts for every compiled
/// opponent, including fighters without a ledge-response profile.
pub fn ordinary_grab(kind: i32, status: i32) -> Option<GrabReference> {
    let slot = match status {
        0x37 => 0,
        0x39 => 1,
        0x3b => 2,
        _ => return None,
    };
    if let Some(facts) = GRAB_FACTS.get() {
        return facts
            .binary_search_by_key(&kind, |(kind, _)| *kind)
            .ok()
            .and_then(|index| facts[index].1[slot]);
    }
    // Standalone core callers retain the original behavior without initialization.
    ordinary_grab_uncached(kind, status)
}
fn ordinary_grab_uncached(kind: i32, status: i32) -> Option<GrabReference> {
    let name = match status {
        0x37 => "Grab",
        0x39 => "Dash Grab",
        0x3b => "Pivot Grab",
        _ => return None,
    };
    let reference = profile(kind)?;
    let mut window = None;
    for move_ in reference
        .moves
        .iter()
        .filter(|move_| move_.name == name || status == 0x37 && move_.name.ends_with(" Grabs"))
    {
        let candidate = (
            move_.active.iter().map(|span| span.first).min()?,
            move_.active.iter().map(|span| span.last).max()?,
        );
        if window.is_some_and(|known| known != candidate) {
            return None;
        }
        window = Some(candidate);
    }
    let (first, last) = window?;
    if first == 0 || last < first || last > 120 {
        return None;
    }
    // The shared source omits this stat on the three Pokemon Trainer forms and
    // Olimar; use their independently compiled, exact profile values only.
    let reach = if let Some(raw) = reference.stat("grabs.grabrange") {
        let number = raw
            .split(|c: char| !(c.is_ascii_digit() || matches!(c, '.' | '-')))
            .find(|part| !part.is_empty())?;
        number.parse().ok()?
    } else {
        match kind {
            36 => {
                crate::fighter_profiles::squirtle::LEDGE_DATA
                    .grab
                    .standing_range
            }
            37 => {
                crate::fighter_profiles::ivysaur::LEDGE_DATA
                    .grab
                    .standing_range
            }
            38 => {
                crate::fighter_profiles::charizard::LEDGE_DATA
                    .grab
                    .standing_range
            }
            43 => {
                crate::fighter_profiles::olimar::LEDGE_DATA
                    .grab
                    .standing_range
            }
            _ => return None,
        }
    };
    if !reach.is_finite() || !(1.0..=60.0).contains(&reach) {
        return None;
    }
    Some(GrabReference {
        first: first as u8,
        last: last as u8,
        reach,
    })
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

/// Return one unambiguous named move; duplicate source rows require a compiled
/// custom recipe rather than silently selecting the first variant.
pub fn named_move(kind: i32, name: &str) -> Option<&'static MoveData> {
    let mut matches = profile(kind)?
        .moves
        .iter()
        .filter(|move_| move_.name == name);
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
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

#[cfg(test)]
mod grab_reference_cache_tests {
    use super::*;
    #[test]
    fn prepared_references_match_uncached_data_for_every_kind_and_status() {
        let mut expected = Vec::new();
        for reference in all() {
            for status in [0x37, 0x39, 0x3b, 0x1f] {
                expected.push((
                    reference.kind,
                    status,
                    ordinary_grab_uncached(reference.kind, status),
                ));
            }
        }
        prepare_grab_references();
        for (kind, status, expected) in expected {
            assert_eq!(ordinary_grab(kind, status), expected);
        }
        for kind in [-1, 0x77, i32::MAX] {
            assert_eq!(
                ordinary_grab(kind, 0x37),
                ordinary_grab_uncached(kind, 0x37)
            );
        }
    }
}
