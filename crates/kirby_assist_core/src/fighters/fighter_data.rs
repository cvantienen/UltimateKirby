//! The roster-wide fighter statistics and frame-data catalog.
//!
//! `Ultimate_stats.json` is embedded and parsed once. There is no network
//! access, generated Rust mirror, role-specific fallback, or guessed data.
use serde::Deserialize;
use std::{collections::BTreeMap, sync::OnceLock};

/// Embedded roster statistics and frame-data JSON.
const DATA_FILE: &str = include_str!("../../../../data/Ultimate_stats.json");
/// Schema version required from the embedded fighter-data catalog.
const DATA_SCHEMA_VERSION: u32 = 1;
/// Roster entry count required for the embedded catalog.
const EXPECTED_FIGHTER_COUNT: usize = 90;
/// Largest move-frame value accepted from the embedded catalog.
const MAX_VALID_MOVE_FRAME: u16 = 120;

/// Native status used to recognize dash attack frame data.
const STATUS_DASH_ATTACK: i32 = crate::native_states::status::ATTACK_DASH;
/// Native status used to recognize forward tilt frame data.
const STATUS_FORWARD_TILT: i32 = crate::native_states::status::ATTACK_S3;
/// Native status used to recognize up tilt frame data.
const STATUS_UP_TILT: i32 = crate::native_states::status::ATTACK_HI3;
/// Native status used to recognize down tilt frame data.
const STATUS_DOWN_TILT: i32 = crate::native_states::status::ATTACK_LW3;
/// Native status used to recognize forward smash frame data.
const STATUS_FORWARD_SMASH: i32 = crate::native_states::status::ATTACK_S4;
/// Native status used to recognize down smash frame data.
const STATUS_DOWN_SMASH: i32 = crate::native_states::status::ATTACK_LW4;
/// Native status used to recognize up smash frame data.
const STATUS_UP_SMASH: i32 = crate::native_states::status::ATTACK_HI4;
/// Native status used to recognize grab frame data.
const STATUS_GRAB: i32 = crate::native_states::CATCH;
/// Native status used to recognize dash grab frame data.
const STATUS_DASH_GRAB: i32 = crate::native_states::CATCH_DASH;
/// Native status used to recognize pivot grab frame data.
const STATUS_PIVOT_GRAB: i32 = crate::native_states::CATCH_TURN;
/// Standing, dash, and pivot-grab statuses used for grab-data lookup.
const GRAB_STATUSES: [i32; 3] = [STATUS_GRAB, STATUS_DASH_GRAB, STATUS_PIVOT_GRAB];

/// Inclusive startup-to-last-active frame interval for one hitbox window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    /// First active frame in this hitbox window.
    pub first: u16,
    /// Last active frame in this hitbox window.
    pub last: u16,
}

/// One named fighter statistic exactly as loaded from the embedded data catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stat {
    /// Catalog key naming the fighter statistic.
    pub key: String,
    /// Raw catalog value for the fighter statistic.
    pub value: String,
}

/// Validated frame data and motion hashes for one fighter move.
#[derive(Clone, Debug)]
pub struct MoveData {
    /// Display name of the move.
    pub name: String,
    /// Total animation frames when the catalog supplies them.
    pub total: Option<u16>,
    /// Aerial landing lag in frames, when applicable.
    pub landing_lag: Option<u16>,
    /// All validated active-hitbox frame windows.
    pub active: Vec<Span>,
    /// Native motion hashes that identify this move.
    pub motions: Vec<u64>,
    /// Whether the move may produce a whiff-punish recovery estimate.
    pub whiff_eligible: bool,
    /// Unparsed statistics retained from the source catalog.
    pub raw: Vec<Stat>,
}

impl MoveData {
    /// Checks whether damage data.
    pub fn has_damage_data(&self) -> bool {
        self.raw
            .iter()
            .find(|stat| stat.key == "basedamage")
            .is_some_and(|stat| !stat.value.is_empty() && stat.value != "--")
    }

    /// Returns the most recently active frame.
    pub fn last_active(&self) -> Option<u16> {
        self.active.iter().map(|span| span.last).max()
    }

    /// Returns the observed recovery duration.
    pub fn recovery(&self, observation: RecoveryObservation) -> Option<f32> {
        let expected_status = match self.name.as_str() {
            "Dash Attack" => STATUS_DASH_ATTACK,
            "Forward Tilt" => STATUS_FORWARD_TILT,
            "Up Tilt" => STATUS_UP_TILT,
            "Down Tilt" => STATUS_DOWN_TILT,
            "Forward Smash" => STATUS_FORWARD_SMASH,
            "Down Smash" => STATUS_DOWN_SMASH,
            "Up Smash" => STATUS_UP_SMASH,
            _ => return None,
        };
        let total = self.total?;
        let last_active = self.last_active()?;

        if !self.whiff_eligible || !self.motions.contains(&observation.motion) {
            return None;
        }
        if observation.status != expected_status || !observation.grounded {
            return None;
        }
        if observation.can_cancel
            || observation.active_attack
            || observation.registered_contact
            || !observation.vulnerable
        {
            return None;
        }
        if !observation.frame.is_finite()
            || observation.frame <= f32::from(last_active)
            || !observation.end.is_finite()
            || (observation.end - f32::from(total)).abs() > 2.0
        {
            return None;
        }

        crate::gameplay::recovery_frames(
            observation.frame,
            observation.end.min(f32::from(total)),
            observation.cancel,
            observation.rate,
        )
        .filter(|remaining| *remaining > 0.0)
    }
}

/// Copied native facts used to verify that a grounded attack is in punishable recovery.
#[derive(Clone, Copy, Debug)]
pub struct RecoveryObservation {
    /// Current native motion hash.
    pub motion: u64,
    /// Current native fighter status.
    pub status: i32,
    /// Whether the attacker is currently grounded.
    pub grounded: bool,
    /// Current motion frame.
    pub frame: f32,
    /// Current motion playback rate.
    pub rate: f32,
    /// Observed motion end frame.
    pub end: f32,
    /// Observed cancel-enable frame.
    pub cancel: f32,
    /// Whether the attacker can already cancel the move.
    pub can_cancel: bool,
    /// Whether an attack hitbox is still active.
    pub active_attack: bool,
    /// Whether the move has hit or been blocked.
    pub registered_contact: bool,
    /// Whether the attacker can currently be hit.
    pub vulnerable: bool,
}

/// Validated statistics and move data for one roster fighter.
#[derive(Clone, Debug)]
pub struct FighterData {
    /// Native fighter-kind ID.
    pub kind: i32,
    /// Roster display name.
    pub name: String,
    /// Validated fighter-wide statistics.
    pub stats: Vec<Stat>,
    /// Validated move frame data.
    pub moves: Vec<MoveData>,
}

impl FighterData {
    /// Resolves the configured move associated with the observed motion.
    pub fn move_for_motion(&self, motion: u64) -> Option<&MoveData> {
        unique(
            self.moves
                .iter()
                .filter(|move_data| move_data.motions.contains(&motion)),
        )
    }

    /// Returns the fighter statistic selected by the key.
    pub fn stat(&self, key: &str) -> Option<&str> {
        self.stats
            .iter()
            .find(|stat| stat.key == key)
            .map(|stat| stat.value.as_str())
    }

    /// Returns the numeric fighter statistic selected by the key.
    pub fn numeric_stat(&self, key: &str) -> Option<f32> {
        self.stat(key)?.parse().ok()
    }
}

/// Startup, active-window, and optional reach data for one grab type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrabReference {
    /// First frame on which the grab box is active.
    pub first: u8,
    /// Last frame on which the grab box is active.
    pub last: u8,
    /// Optional root-relative grab reach from the data catalog.
    pub reach: Option<f32>,
}

/// Top-level shape of the embedded fighter-data JSON file.
#[derive(Deserialize)]
struct DataFile {
    /// Schema version declared by the embedded fighter-data file.
    schema_version: u32,
    /// Serialized roster entries from the embedded fighter-data file.
    fighters: Vec<FighterRecord>,
}

/// Serialized fighter entry read from the embedded data catalog.
#[derive(Deserialize)]
struct FighterRecord {
    /// Native fighter-kind ID from the serialized record.
    kind: i32,
    /// Roster name from the serialized record.
    name: String,
    /// Serialized fighter-wide statistics.
    stats: Vec<StatRecord>,
    /// Serialized move frame data.
    moves: Vec<MoveRecord>,
}

/// Serialized key/value fighter statistic from the embedded data catalog.
#[derive(Deserialize)]
struct StatRecord {
    /// Serialized statistic name.
    key: String,
    /// Serialized statistic value.
    value: String,
}

/// Serialized move frame-data entry from the embedded data catalog.
#[derive(Deserialize)]
struct MoveRecord {
    /// Serialized move name.
    name: String,
    /// Serialized total animation frames.
    total: Option<u16>,
    /// Serialized aerial landing lag.
    landing_lag: Option<u16>,
    /// Serialized active-frame intervals.
    active: Vec<[u16; 2]>,
    /// Serialized native motion names or hashes.
    motions: Vec<String>,
    /// Serialized permission to estimate whiff recovery.
    whiff_eligible: bool,
    /// Additional move statistics retained by key.
    raw: BTreeMap<String, String>,
}

/// Native grab status paired with standing, dash, and pivot reference data.
type GrabFacts = (i32, [Option<GrabReference>; 3]);
/// Lazily parsed and validated roster fighter data.
static FIGHTERS: OnceLock<Result<Vec<FighterData>, String>> = OnceLock::new();
/// Lazily derived grab startup, active-window, and reach data.
static GRAB_FACTS: OnceLock<Vec<GrabFacts>> = OnceLock::new();

/// Parses and validates the embedded roster-wide fighter-data catalog.
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

    // Validate each record while converting it into the smaller runtime types.
    let fighters = file
        .fighters
        .into_iter()
        .map(parse_fighter)
        .collect::<Result<Vec<_>, _>>()?;

    if !fighters.windows(2).all(|pair| pair[0].kind < pair[1].kind) {
        return Err("Ultimate_stats.json fighter records must be ordered by unique kind".into());
    }
    Ok(fighters)
}

/// Validates one serialized fighter and converts it to runtime frame data.
fn parse_fighter(record: FighterRecord) -> Result<FighterData, String> {
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

    let moves = record
        .moves
        .into_iter()
        .map(|move_record| parse_move(&record.name, move_record))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(FighterData {
        kind: record.kind,
        name: record.name,
        stats,
        moves,
    })
}

/// Validates one serialized move and converts its active frames and motion IDs.
fn parse_move(fighter_name: &str, record: MoveRecord) -> Result<MoveData, String> {
    let active = record
        .active
        .into_iter()
        .map(|[first, last]| Span { first, last })
        .collect::<Vec<_>>();
    if active
        .iter()
        .any(|span| span.first == 0 || span.first > span.last)
    {
        return Err(format!(
            "fighter {fighter_name} has an invalid active-frame span for {}",
            record.name
        ));
    }

    // Convert readable motion names to hashes once, not during live lookups.
    let motions = record
        .motions
        .iter()
        .map(|name| crate::timing::motion_hash(name))
        .collect();
    let raw = record
        .raw
        .into_iter()
        .map(|(key, value)| Stat { key, value })
        .collect();

    Ok(MoveData {
        name: record.name,
        total: record.total,
        landing_lag: record.landing_lag,
        active,
        motions,
        whiff_eligible: record.whiff_eligible,
        raw,
    })
}

/// Validate and initialize the embedded roster before installing live hooks.
pub fn initialize() -> Result<(), &'static str> {
    try_all().map(|_| ())
}

/// Attempts each eligible candidate until one is accepted.
/// Tries the available options in order.
fn try_all() -> Result<&'static [FighterData], &'static str> {
    FIGHTERS
        .get_or_init(load_fighters)
        .as_deref()
        .map_err(String::as_str)
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
                    GRAB_STATUSES.map(|status| ordinary_grab_uncached(fighter.kind, status)),
                )
            })
            .collect()
    });
    Ok(())
}

/// Exact standing, dash, or pivot grab facts for any fighter-data row.
pub fn ordinary_grab(kind: i32, status: i32) -> Option<GrabReference> {
    let slot = GRAB_STATUSES.iter().position(|&known| known == status)?;
    if let Some(facts) = GRAB_FACTS.get() {
        return facts
            .binary_search_by_key(&kind, |(fighter_kind, _)| *fighter_kind)
            .ok()
            .and_then(|index| facts[index].1[slot]);
    }
    ordinary_grab_uncached(kind, status)
}

/// Reads and validates ordinary-grab data before the shared cache exists.
fn ordinary_grab_uncached(kind: i32, status: i32) -> Option<GrabReference> {
    let move_name = match status {
        STATUS_GRAB => "Grab",
        STATUS_DASH_GRAB => "Dash Grab",
        STATUS_PIVOT_GRAB => "Pivot Grab",
        _ => return None,
    };
    let fighter = fighter(kind)?;
    let mut active_window = None;
    for move_data in fighter.moves.iter().filter(|move_data| {
        move_data.name == move_name || (status == STATUS_GRAB && move_data.name.ends_with(" Grabs"))
    }) {
        // Some fighters have more than one grab variant. Conflicting timings
        // are unknown, so do not select one variant arbitrarily.
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
    if first == 0 || last < first || last > MAX_VALID_MOVE_FRAME {
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
    unique(
        fighter(kind)?
            .moves
            .iter()
            .filter(|move_data| move_data.name == name),
    )
}

/// The only item of `matches`; zero or several candidates are ambiguous.
fn unique<'a, T>(mut matches: impl Iterator<Item = &'a T>) -> Option<&'a T> {
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}

/// Small cache for fighter objects observed in controller/player slots.
#[derive(Clone, Copy, Default)]
pub struct FighterDataCache {
    /// Cached battle-object IDs paired with their resolved fighter data.
    slots: [Option<(u32, i32, &'static FighterData)>; 8],
}

impl FighterDataCache {
    /// Creates an empty fighter-data cache for the eight controller slots.
    pub const fn new() -> Self {
        Self { slots: [None; 8] }
    }

    /// Resolves the configured binding for the requested action.
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

    /// Forgets the fighter identity and data cached for one controller slot.
    pub fn clear(&mut self, slot: usize) {
        if let Some(entry) = self.slots.get_mut(slot) {
            *entry = None;
        }
    }
}

#[cfg(test)]
mod grab_reference_cache_tests {
    use super::*;

    /// Regression test for `prepared_references_match_uncached_data_for_every_kind_and_status`.
    #[test]
    fn prepared_references_match_uncached_data_for_every_kind_and_status() {
        let expected = all()
            .iter()
            .flat_map(|fighter| {
                [
                    STATUS_GRAB,
                    STATUS_DASH_GRAB,
                    STATUS_PIVOT_GRAB,
                    crate::native_states::ESCAPE,
                ]
                .map(|status| {
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
                ordinary_grab(kind, crate::native_states::CATCH),
                ordinary_grab_uncached(kind, crate::native_states::CATCH)
            );
        }
    }
}
