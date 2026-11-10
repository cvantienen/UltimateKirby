//! The roster-wide fighter statistics and frame-data catalog.
//!
//! `Ultimate_stats.json` is embedded and parsed once. There is no network
//! access, generated Rust mirror, role-specific fallback, or guessed data.
use serde::Deserialize;
use std::{collections::BTreeMap, sync::OnceLock};

/// Technical: Defines the `DATA_FILE` constant used by this module.
///
/// Plain English: Provides the DATA FILE constant value.
const DATA_FILE: &str = include_str!("../../../../data/Ultimate_stats.json");
/// Technical: Defines the `DATA_SCHEMA_VERSION` constant used by this module.
///
/// Plain English: Provides the DATA SCHEMA VERSION constant value.
const DATA_SCHEMA_VERSION: u32 = 1;
/// Technical: Defines the `EXPECTED_FIGHTER_COUNT` constant used by this module.
///
/// Plain English: Provides the EXPECTED FIGHTER COUNT constant value.
const EXPECTED_FIGHTER_COUNT: usize = 90;
/// Technical: Sets the upper bound for valid move frame.
///
/// Plain English: Holds the MAX VALID MOVE FRAME value.
const MAX_VALID_MOVE_FRAME: u16 = 120;

/// Technical: Defines the `STATUS_DASH_ATTACK` constant used by this module.
///
/// Plain English: Provides the STATUS DASH ATTACK constant value.
const STATUS_DASH_ATTACK: i32 = crate::native_states::status::ATTACK_DASH;
/// Technical: Defines the `STATUS_FORWARD_TILT` constant used by this module.
///
/// Plain English: Provides the STATUS FORWARD TILT constant value.
const STATUS_FORWARD_TILT: i32 = crate::native_states::status::ATTACK_S3;
/// Technical: Defines the `STATUS_UP_TILT` constant used by this module.
///
/// Plain English: Provides the STATUS UP TILT constant value.
const STATUS_UP_TILT: i32 = crate::native_states::status::ATTACK_HI3;
/// Technical: Defines the `STATUS_DOWN_TILT` constant used by this module.
///
/// Plain English: Provides the STATUS DOWN TILT constant value.
const STATUS_DOWN_TILT: i32 = crate::native_states::status::ATTACK_LW3;
/// Technical: Defines the `STATUS_FORWARD_SMASH` constant used by this module.
///
/// Plain English: Provides the STATUS FORWARD SMASH constant value.
const STATUS_FORWARD_SMASH: i32 = crate::native_states::status::ATTACK_S4;
/// Technical: Defines the `STATUS_DOWN_SMASH` constant used by this module.
///
/// Plain English: Provides the STATUS DOWN SMASH constant value.
const STATUS_DOWN_SMASH: i32 = crate::native_states::status::ATTACK_LW4;
/// Technical: Defines the `STATUS_UP_SMASH` constant used by this module.
///
/// Plain English: Provides the STATUS UP SMASH constant value.
const STATUS_UP_SMASH: i32 = crate::native_states::status::ATTACK_HI4;
/// Technical: Defines the `STATUS_GRAB` constant used by this module.
///
/// Plain English: Provides the STATUS GRAB constant value.
const STATUS_GRAB: i32 = crate::native_states::CATCH;
/// Technical: Defines the `STATUS_DASH_GRAB` constant used by this module.
///
/// Plain English: Provides the STATUS DASH GRAB constant value.
const STATUS_DASH_GRAB: i32 = crate::native_states::CATCH_DASH;
/// Technical: Defines the `STATUS_PIVOT_GRAB` constant used by this module.
///
/// Plain English: Provides the STATUS PIVOT GRAB constant value.
const STATUS_PIVOT_GRAB: i32 = crate::native_states::CATCH_TURN;
/// Technical: Defines the `GRAB_STATUSES` constant used by this module.
///
/// Plain English: Provides the GRAB STATUSES constant value.
const GRAB_STATUSES: [i32; 3] = [STATUS_GRAB, STATUS_DASH_GRAB, STATUS_PIVOT_GRAB];

/// Technical: Defines the `Span` data structure.
///
/// Plain English: Holds the data for Span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    /// Technical: Stores the `first` value in `Span` as `u16`.
    ///
    /// Plain English: Holds the first value used by Span.
    pub first: u16,
    /// Technical: Stores the `last` value in `Span` as `u16`.
    ///
    /// Plain English: Holds the last value used by Span.
    pub last: u16,
}

/// Technical: Defines the `Stat` data structure.
///
/// Plain English: Holds the data for Stat.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stat {
    /// Technical: Stores the `key` value in `Stat` as `String`.
    ///
    /// Plain English: Holds the key value used by Stat.
    pub key: String,
    /// Technical: Stores the `value` value in `Stat` as `String`.
    ///
    /// Plain English: Holds the value value used by Stat.
    pub value: String,
}

/// Technical: Defines the `MoveData` data structure.
///
/// Plain English: Holds the data for MoveData.
#[derive(Clone, Debug)]
pub struct MoveData {
    /// Technical: Stores the `name` value in `MoveData` as `String`.
    ///
    /// Plain English: Holds the name value used by MoveData.
    pub name: String,
    /// Technical: Stores the `total` value in `MoveData` as `Option<u16>`.
    ///
    /// Plain English: Optionally stores the total for MoveData; it may be absent.
    pub total: Option<u16>,
    /// Technical: Stores the `landing_lag` value in `MoveData` as `Option<u16>`.
    ///
    /// Plain English: Optionally stores the landing lag for MoveData; it may be absent.
    pub landing_lag: Option<u16>,
    /// Technical: Stores the `active` value in `MoveData` as `Vec<Span>`.
    ///
    /// Plain English: Holds the active value used by MoveData.
    pub active: Vec<Span>,
    /// Technical: Stores the `motions` value in `MoveData` as `Vec<u64>`.
    ///
    /// Plain English: Holds the motions value used by MoveData.
    pub motions: Vec<u64>,
    /// Technical: Stores the `whiff_eligible` value in `MoveData` as `bool`.
    ///
    /// Plain English: Holds the whiff eligible value used by MoveData.
    pub whiff_eligible: bool,
    /// Technical: Stores the `raw` value in `MoveData` as `Vec<Stat>`.
    ///
    /// Plain English: Holds the raw value used by MoveData.
    pub raw: Vec<Stat>,
}

impl MoveData {
    /// Technical: Checks whether damage data.
    ///
    /// Plain English: Checks whether damage data.
    pub fn has_damage_data(&self) -> bool {
        self.raw
            .iter()
            .find(|stat| stat.key == "basedamage")
            .is_some_and(|stat| !stat.value.is_empty() && stat.value != "--")
    }

    /// Technical: Returns the most recently active frame.
    ///
    /// Plain English: Gets the last frame when the action was active.
    pub fn last_active(&self) -> Option<u16> {
        self.active.iter().map(|span| span.last).max()
    }

    /// Technical: Returns the observed recovery duration.
    ///
    /// Plain English: Gets how long recovery lasts.
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

/// Technical: Defines the `RecoveryObservation` data structure.
///
/// Plain English: Holds the data for RecoveryObservation.
#[derive(Clone, Copy, Debug)]
pub struct RecoveryObservation {
    /// Technical: Stores the `motion` value in `RecoveryObservation` as `u64`.
    ///
    /// Plain English: Holds the motion value used by RecoveryObservation.
    pub motion: u64,
    /// Technical: Stores the `status` value in `RecoveryObservation` as `i32`.
    ///
    /// Plain English: Holds the status value used by RecoveryObservation.
    pub status: i32,
    /// Technical: Stores the `grounded` value in `RecoveryObservation` as `bool`.
    ///
    /// Plain English: Holds the grounded value used by RecoveryObservation.
    pub grounded: bool,
    /// Technical: Stores the `frame` value in `RecoveryObservation` as `f32`.
    ///
    /// Plain English: Holds the frame value used by RecoveryObservation.
    pub frame: f32,
    /// Technical: Stores the `rate` value in `RecoveryObservation` as `f32`.
    ///
    /// Plain English: Holds the rate value used by RecoveryObservation.
    pub rate: f32,
    /// Technical: Stores the `end` value in `RecoveryObservation` as `f32`.
    ///
    /// Plain English: Holds the end value used by RecoveryObservation.
    pub end: f32,
    /// Technical: Stores the `cancel` value in `RecoveryObservation` as `f32`.
    ///
    /// Plain English: Holds the cancel value used by RecoveryObservation.
    pub cancel: f32,
    /// Technical: Stores the `can_cancel` value in `RecoveryObservation` as `bool`.
    ///
    /// Plain English: Holds the can cancel value used by RecoveryObservation.
    pub can_cancel: bool,
    /// Technical: Stores the `active_attack` value in `RecoveryObservation` as `bool`.
    ///
    /// Plain English: Holds the active attack value used by RecoveryObservation.
    pub active_attack: bool,
    /// Technical: Stores the `registered_contact` value in `RecoveryObservation` as `bool`.
    ///
    /// Plain English: Holds the registered contact value used by RecoveryObservation.
    pub registered_contact: bool,
    /// Technical: Stores the `vulnerable` value in `RecoveryObservation` as `bool`.
    ///
    /// Plain English: Holds the vulnerable value used by RecoveryObservation.
    pub vulnerable: bool,
}

/// Technical: Defines the `FighterData` data structure.
///
/// Plain English: Holds the data for FighterData.
#[derive(Clone, Debug)]
pub struct FighterData {
    /// Technical: Stores the `kind` value in `FighterData` as `i32`.
    ///
    /// Plain English: Holds the kind value used by FighterData.
    pub kind: i32,
    /// Technical: Stores the `name` value in `FighterData` as `String`.
    ///
    /// Plain English: Holds the name value used by FighterData.
    pub name: String,
    /// Technical: Stores the `stats` value in `FighterData` as `Vec<Stat>`.
    ///
    /// Plain English: Holds the stats value used by FighterData.
    pub stats: Vec<Stat>,
    /// Technical: Stores the `moves` value in `FighterData` as `Vec<MoveData>`.
    ///
    /// Plain English: Holds the moves value used by FighterData.
    pub moves: Vec<MoveData>,
}

impl FighterData {
    /// Technical: Resolves the configured move associated with the observed motion.
    ///
    /// Plain English: Finds the move that matches this animation.
    pub fn move_for_motion(&self, motion: u64) -> Option<&MoveData> {
        unique(
            self.moves
                .iter()
                .filter(|move_data| move_data.motions.contains(&motion)),
        )
    }

    /// Technical: Returns the fighter statistic selected by the key.
    ///
    /// Plain English: Gets the fighter statistic.
    pub fn stat(&self, key: &str) -> Option<&str> {
        self.stats
            .iter()
            .find(|stat| stat.key == key)
            .map(|stat| stat.value.as_str())
    }

    /// Technical: Returns the numeric fighter statistic selected by the key.
    ///
    /// Plain English: Gets the number for this fighter statistic.
    pub fn numeric_stat(&self, key: &str) -> Option<f32> {
        self.stat(key)?.parse().ok()
    }
}

/// Technical: Defines the `GrabReference` data structure.
///
/// Plain English: Holds the data for GrabReference.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrabReference {
    /// Technical: Stores the `first` value in `GrabReference` as `u8`.
    ///
    /// Plain English: Holds the first value used by GrabReference.
    pub first: u8,
    /// Technical: Stores the `last` value in `GrabReference` as `u8`.
    ///
    /// Plain English: Holds the last value used by GrabReference.
    pub last: u8,
    /// Technical: Stores the `reach` value in `GrabReference` as `Option<f32>`.
    ///
    /// Plain English: Optionally stores the reach for GrabReference; it may be absent.
    pub reach: Option<f32>,
}

/// Technical: Defines the `DataFile` data structure.
///
/// Plain English: Holds the data for DataFile.
#[derive(Deserialize)]
struct DataFile {
    /// Technical: Stores the `schema_version` value in `DataFile` as `u32`.
    ///
    /// Plain English: Holds the schema version value used by DataFile.
    schema_version: u32,
    /// Technical: Stores the `fighters` value in `DataFile` as `Vec<FighterRecord>`.
    ///
    /// Plain English: Holds the fighters value used by DataFile.
    fighters: Vec<FighterRecord>,
}

/// Technical: Defines the `FighterRecord` data structure.
///
/// Plain English: Holds the data for FighterRecord.
#[derive(Deserialize)]
struct FighterRecord {
    /// Technical: Stores the `kind` value in `FighterRecord` as `i32`.
    ///
    /// Plain English: Holds the kind value used by FighterRecord.
    kind: i32,
    /// Technical: Stores the `name` value in `FighterRecord` as `String`.
    ///
    /// Plain English: Holds the name value used by FighterRecord.
    name: String,
    /// Technical: Stores the `stats` value in `FighterRecord` as `Vec<StatRecord>`.
    ///
    /// Plain English: Holds the stats value used by FighterRecord.
    stats: Vec<StatRecord>,
    /// Technical: Stores the `moves` value in `FighterRecord` as `Vec<MoveRecord>`.
    ///
    /// Plain English: Holds the moves value used by FighterRecord.
    moves: Vec<MoveRecord>,
}

/// Technical: Defines the `StatRecord` data structure.
///
/// Plain English: Holds the data for StatRecord.
#[derive(Deserialize)]
struct StatRecord {
    /// Technical: Stores the `key` value in `StatRecord` as `String`.
    ///
    /// Plain English: Holds the key value used by StatRecord.
    key: String,
    /// Technical: Stores the `value` value in `StatRecord` as `String`.
    ///
    /// Plain English: Holds the value value used by StatRecord.
    value: String,
}

/// Technical: Defines the `MoveRecord` data structure.
///
/// Plain English: Holds the data for MoveRecord.
#[derive(Deserialize)]
struct MoveRecord {
    /// Technical: Stores the `name` value in `MoveRecord` as `String`.
    ///
    /// Plain English: Holds the name value used by MoveRecord.
    name: String,
    /// Technical: Stores the `total` value in `MoveRecord` as `Option<u16>`.
    ///
    /// Plain English: Optionally stores the total for MoveRecord; it may be absent.
    total: Option<u16>,
    /// Technical: Stores the `landing_lag` value in `MoveRecord` as `Option<u16>`.
    ///
    /// Plain English: Optionally stores the landing lag for MoveRecord; it may be absent.
    landing_lag: Option<u16>,
    /// Technical: Stores the `active` value in `MoveRecord` as `Vec<[u16; 2]>`.
    ///
    /// Plain English: Holds the active value used by MoveRecord.
    active: Vec<[u16; 2]>,
    /// Technical: Stores the `motions` value in `MoveRecord` as `Vec<String>`.
    ///
    /// Plain English: Holds the motions value used by MoveRecord.
    motions: Vec<String>,
    /// Technical: Stores the `whiff_eligible` value in `MoveRecord` as `bool`.
    ///
    /// Plain English: Holds the whiff eligible value used by MoveRecord.
    whiff_eligible: bool,
    /// Technical: Stores the `raw` value in `MoveRecord` as `BTreeMap<String`.
    ///
    /// Plain English: Holds the raw value used by MoveRecord.
    raw: BTreeMap<String, String>,
}

/// Technical: Defines `GrabFacts` as a type alias.
///
/// Plain English: Holds the GrabFacts value.
type GrabFacts = (i32, [Option<GrabReference>; 3]);
/// Technical: Defines the `FIGHTERS` constant used by this module.
///
/// Plain English: Provides the FIGHTERS constant value.
static FIGHTERS: OnceLock<Result<Vec<FighterData>, String>> = OnceLock::new();
/// Technical: Defines the `GRAB_FACTS` constant used by this module.
///
/// Plain English: Provides the GRAB FACTS constant value.
static GRAB_FACTS: OnceLock<Vec<GrabFacts>> = OnceLock::new();

/// Technical: Loads the fighters.
///
/// Plain English: Loads the fighters.
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

/// Technical: Parses the fighter.
///
/// Plain English: Parses the fighter.
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

/// Technical: Parses the move.
///
/// Plain English: Parses the move.
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
/// Plain English: Validate and initialize the embedded roster before installing live hooks.
pub fn initialize() -> Result<(), &'static str> {
    try_all().map(|_| ())
}

/// Technical: Attempts each eligible candidate until one is accepted.
///
/// Plain English: Tries the available options in order.
fn try_all() -> Result<&'static [FighterData], &'static str> {
    FIGHTERS
        .get_or_init(load_fighters)
        .as_deref()
        .map_err(String::as_str)
}

/// All fighter records, independent of player/opponent role.
/// Plain English: All fighter records, independent of player/opponent role.
pub fn all() -> &'static [FighterData] {
    try_all().unwrap_or_else(|error| panic!("invalid embedded fighter data: {error}"))
}

/// Exact native fighter-kind lookup. Unknown kinds never borrow another row.
/// Plain English: Exact native fighter-kind lookup. Unknown kinds never borrow another row.
pub fn fighter(kind: i32) -> Option<&'static FighterData> {
    let fighters = all();
    fighters
        .binary_search_by_key(&kind, |fighter| fighter.kind)
        .ok()
        .map(|index| &fighters[index])
}

/// Prepare the compact native-kind grab lookup once before live callbacks.
/// Plain English: Prepare the compact native-kind grab lookup once before live callbacks.
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
/// Plain English: Exact standing, dash, or pivot grab facts for any fighter-data row.
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

/// Technical: Reads ordinary-grab data without reusing a cached result.
///
/// Plain English: Gets fresh ordinary-grab data.
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
/// Plain English: Return one unambiguous named move; variants are never guessed.
pub fn named_move(kind: i32, name: &str) -> Option<&'static MoveData> {
    unique(
        fighter(kind)?
            .moves
            .iter()
            .filter(|move_data| move_data.name == name),
    )
}

/// The only item of `matches`; zero or several candidates are ambiguous.
/// Plain English: The only item of matches; zero or several candidates are ambiguous.
fn unique<'a, T>(mut matches: impl Iterator<Item = &'a T>) -> Option<&'a T> {
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}

/// Small cache for fighter objects observed in controller/player slots.
/// Plain English: Small cache for fighter objects observed in controller/player slots.
#[derive(Clone, Copy, Default)]
pub struct FighterDataCache {
    /// Technical: Stores the `slots` value in `FighterDataCache` as `[Option<(u32`.
    ///
    /// Plain English: Stores the slots collection for FighterDataCache.
    slots: [Option<(u32, i32, &'static FighterData)>; 8],
}

impl FighterDataCache {
    /// Technical: Constructs a new value with the type’s initial state.
    ///
    /// Plain English: Creates a new value.
    pub const fn new() -> Self {
        Self { slots: [None; 8] }
    }

    /// Technical: Resolves the configured binding for the requested action.
    ///
    /// Plain English: Finds the button assigned to this action.
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

    /// Technical: Clears the remembered fighter observation.
    ///
    /// Plain English: Forgets the last fighter observation.
    pub fn clear(&mut self, slot: usize) {
        if let Some(entry) = self.slots.get_mut(slot) {
            *entry = None;
        }
    }
}

#[cfg(test)]
mod grab_reference_cache_tests {
    use super::*;

    /// Technical: Regression test for `prepared_references_match_uncached_data_for_every_kind_and_status`.
    ///
    /// Plain English: Checks that prepared references match uncached data for every kind and status.
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
