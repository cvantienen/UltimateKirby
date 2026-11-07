//! The roster-wide fighter statistics and frame-data catalog.
//!
//! `Ultimate_stats.json` is embedded and parsed once. There is no network
//! access, generated Rust mirror, role-specific fallback, or guessed data.
use serde::Deserialize;
use std::{collections::BTreeMap, sync::OnceLock};

const DATA_FILE: &str = include_str!("../../../data/Ultimate_stats.json");
const DATA_SCHEMA_VERSION: u32 = 1;
const EXPECTED_FIGHTER_COUNT: usize = 90;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub first: u16,
    pub last: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stat {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct MoveData {
    pub name: String,
    pub total: Option<u16>,
    pub landing_lag: Option<u16>,
    pub active: Vec<Span>,
    pub motions: Vec<u64>,
    pub whiff_eligible: bool,
    pub raw: Vec<Stat>,
}

impl MoveData {
    pub fn has_damage_data(&self) -> bool {
        self.raw
            .iter()
            .find(|stat| stat.key == "basedamage")
            .is_some_and(|stat| !stat.value.is_empty() && stat.value != "--")
    }

    pub fn last_active(&self) -> Option<u16> {
        self.active.iter().map(|span| span.last).max()
    }

    pub fn recovery(&self, observation: RecoveryObservation) -> Option<f32> {
        let expected_status = match self.name.as_str() {
            "Dash Attack" => 0x29,
            "Forward Tilt" => 0x2a,
            "Up Tilt" => 0x2b,
            "Down Tilt" => 0x2c,
            "Forward Smash" => 0x2f,
            "Down Smash" => 0x32,
            "Up Smash" => 0x35,
            _ => return None,
        };
        let total = self.total?;
        if !self.whiff_eligible
            || !self.motions.contains(&observation.motion)
            || observation.status != expected_status
            || !observation.grounded
            || observation.can_cancel
            || observation.active_attack
            || observation.registered_contact
            || !observation.vulnerable
            || !observation.frame.is_finite()
            || observation.frame <= f32::from(self.last_active()?)
            || !observation.end.is_finite()
            || (observation.end - f32::from(total)).abs() > 2.0
        {
            return None;
        }
        crate::ledge_trap::recovery_frames(
            observation.frame,
            observation.end.min(f32::from(total)),
            observation.cancel,
            observation.rate,
        )
        .filter(|remaining| *remaining > 0.0)
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

#[derive(Clone, Debug)]
pub struct FighterData {
    pub kind: i32,
    pub name: String,
    pub stats: Vec<Stat>,
    pub moves: Vec<MoveData>,
}

impl FighterData {
    pub fn move_for_motion(&self, motion: u64) -> Option<&MoveData> {
        let mut matches = self
            .moves
            .iter()
            .filter(|move_data| move_data.motions.contains(&motion));
        let first = matches.next()?;
        matches.next().is_none().then_some(first)
    }

    pub fn stat(&self, key: &str) -> Option<&str> {
        self.stats
            .iter()
            .find(|stat| stat.key == key)
            .map(|stat| stat.value.as_str())
    }

    pub fn numeric_stat(&self, key: &str) -> Option<f32> {
        self.stat(key)?.parse().ok()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrabReference {
    pub first: u8,
    pub last: u8,
    pub reach: Option<f32>,
}

#[derive(Deserialize)]
struct DataFile {
    schema_version: u32,
    fighters: Vec<FighterRecord>,
}

#[derive(Deserialize)]
struct FighterRecord {
    kind: i32,
    name: String,
    stats: Vec<StatRecord>,
    moves: Vec<MoveRecord>,
}

#[derive(Deserialize)]
struct StatRecord {
    key: String,
    value: String,
}

#[derive(Deserialize)]
struct MoveRecord {
    name: String,
    total: Option<u16>,
    landing_lag: Option<u16>,
    active: Vec<[u16; 2]>,
    motions: Vec<String>,
    whiff_eligible: bool,
    raw: BTreeMap<String, String>,
}

type GrabFacts = (i32, [Option<GrabReference>; 3]);
static FIGHTERS: OnceLock<Result<Vec<FighterData>, String>> = OnceLock::new();
static GRAB_FACTS: OnceLock<Vec<GrabFacts>> = OnceLock::new();

fn load_fighters() -> Result<Vec<FighterData>, String> {
    let file: DataFile = serde_json::from_str(DATA_FILE)
        .map_err(|error| format!("cannot parse embedded Ultimate_stats.json: {error}"))?;
    if file.schema_version != DATA_SCHEMA_VERSION {
        return Err(format!(
            "unsupported Ultimate_stats.json schema version {}",
            file.schema_version
        ));
    }
    if file.fighters.len() != EXPECTED_FIGHTER_COUNT {
        return Err(format!(
            "Ultimate_stats.json has {} fighter records; expected {EXPECTED_FIGHTER_COUNT}",
            file.fighters.len()
        ));
    }

    let mut fighters = Vec::with_capacity(file.fighters.len());
    for record in file.fighters {
        if record.name.is_empty() || record.stats.is_empty() || record.moves.is_empty() {
            return Err(format!(
                "fighter record {} ({}) is missing identity, stats, or frame data",
                record.kind, record.name
            ));
        }
        let mut stats = Vec::with_capacity(record.stats.len());
        for stat in record.stats {
            if stat.key.is_empty() {
                return Err(format!(
                    "fighter {} has a statistic with an empty key",
                    record.name
                ));
            }
            stats.push(Stat {
                key: stat.key,
                value: stat.value,
            });
        }
        let mut moves = Vec::with_capacity(record.moves.len());
        for move_record in record.moves {
            let active = move_record
                .active
                .into_iter()
                .map(|[first, last]| Span { first, last })
                .collect::<Vec<_>>();
            if active
                .iter()
                .any(|span| span.first == 0 || span.first > span.last)
            {
                return Err(format!(
                    "fighter {} has an invalid active-frame span for {}",
                    record.name, move_record.name
                ));
            }
            let motions = move_record
                .motions
                .iter()
                .map(|name| crate::timing::motion_hash(name))
                .collect();
            let raw = move_record
                .raw
                .into_iter()
                .map(|(key, value)| Stat { key, value })
                .collect();
            moves.push(MoveData {
                name: move_record.name,
                total: move_record.total,
                landing_lag: move_record.landing_lag,
                active,
                motions,
                whiff_eligible: move_record.whiff_eligible,
                raw,
            });
        }
        fighters.push(FighterData {
            kind: record.kind,
            name: record.name,
            stats,
            moves,
        });
    }
    if !fighters.windows(2).all(|pair| pair[0].kind < pair[1].kind) {
        return Err("Ultimate_stats.json fighter records must be ordered by unique kind".into());
    }
    Ok(fighters)
}

/// Validate and initialize the embedded roster before installing live hooks.
pub fn initialize() -> Result<(), &'static str> {
    try_all().map(|_| ())
}

fn try_all() -> Result<&'static [FighterData], &'static str> {
    match FIGHTERS.get_or_init(load_fighters) {
        Ok(fighters) => Ok(fighters),
        Err(error) => Err(error.as_str()),
    }
}

/// All fighter records, independent of player/opponent role.
pub fn all() -> &'static [FighterData] {
    try_all().unwrap_or_else(|error| panic!("invalid embedded fighter data: {error}"))
}

/// Exact native fighter-kind lookup. Unknown kinds never borrow another row.
pub fn fighter(kind: i32) -> Option<&'static FighterData> {
    let fighters = all();
    fighters
        .binary_search_by_key(&kind, |fighter| fighter.kind)
        .ok()
        .map(|index| &fighters[index])
}

/// Prepare the compact native-kind grab lookup once before live callbacks.
pub fn prepare_grab_references() -> Result<(), &'static str> {
    let fighters = try_all()?;
    GRAB_FACTS.get_or_init(|| {
        fighters
            .iter()
            .map(|fighter| {
                (
                    fighter.kind,
                    [0x37, 0x39, 0x3b].map(|status| ordinary_grab_uncached(fighter.kind, status)),
                )
            })
            .collect()
    });
    Ok(())
}

/// Exact standing, dash, or pivot grab facts for any fighter-data row.
pub fn ordinary_grab(kind: i32, status: i32) -> Option<GrabReference> {
    let slot = match status {
        0x37 => 0,
        0x39 => 1,
        0x3b => 2,
        _ => return None,
    };
    if let Some(facts) = GRAB_FACTS.get() {
        return facts
            .binary_search_by_key(&kind, |(fighter_kind, _)| *fighter_kind)
            .ok()
            .and_then(|index| facts[index].1[slot]);
    }
    ordinary_grab_uncached(kind, status)
}

fn ordinary_grab_uncached(kind: i32, status: i32) -> Option<GrabReference> {
    let move_name = match status {
        0x37 => "Grab",
        0x39 => "Dash Grab",
        0x3b => "Pivot Grab",
        _ => return None,
    };
    let fighter = fighter(kind)?;
    let mut active_window = None;
    for move_data in fighter.moves.iter().filter(|move_data| {
        move_data.name == move_name || status == 0x37 && move_data.name.ends_with(" Grabs")
    }) {
        let candidate = (
            move_data.active.iter().map(|span| span.first).min()?,
            move_data.active.iter().map(|span| span.last).max()?,
        );
        if active_window.is_some_and(|known| known != candidate) {
            return None;
        }
        active_window = Some(candidate);
    }
    let (first, last) = active_window?;
    if first == 0 || last < first || last > 120 {
        return None;
    }
    let reach = fighter
        .numeric_stat("grabs.grabrange")
        .filter(|reach| reach.is_finite() && (1.0..=60.0).contains(reach));
    Some(GrabReference {
        first: first as u8,
        last: last as u8,
        reach,
    })
}

/// Return one unambiguous named move; variants are never guessed.
pub fn named_move(kind: i32, name: &str) -> Option<&'static MoveData> {
    let mut matches = fighter(kind)?
        .moves
        .iter()
        .filter(|move_data| move_data.name == name);
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}

/// Small cache for fighter objects observed in controller/player slots.
#[derive(Clone, Copy, Default)]
pub struct FighterDataCache {
    slots: [Option<(u32, i32, &'static FighterData)>; 8],
}

impl FighterDataCache {
    pub const fn new() -> Self {
        Self { slots: [None; 8] }
    }

    pub fn bind(&mut self, slot: usize, id: u32, kind: i32) -> Option<&'static FighterData> {
        let entry = self.slots.get_mut(slot)?;
        if let Some((old_id, old_kind, data)) = *entry {
            if old_id == id && old_kind == kind {
                return Some(data);
            }
        }
        *entry = fighter(kind).map(|data| (id, kind, data));
        entry.map(|(_, _, data)| data)
    }

    pub fn clear(&mut self, slot: usize) {
        if let Some(entry) = self.slots.get_mut(slot) {
            *entry = None;
        }
    }
}

#[cfg(test)]
mod grab_reference_cache_tests {
    use super::*;

    #[test]
    fn prepared_references_match_uncached_data_for_every_kind_and_status() {
        let expected = all()
            .iter()
            .flat_map(|fighter| {
                [0x37, 0x39, 0x3b, 0x1f].map(|status| {
                    (
                        fighter.kind,
                        status,
                        ordinary_grab_uncached(fighter.kind, status),
                    )
                })
            })
            .collect::<Vec<_>>();
        prepare_grab_references().unwrap();
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
