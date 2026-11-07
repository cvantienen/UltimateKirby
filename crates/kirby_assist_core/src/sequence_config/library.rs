//! Compact dormant recipe definitions. Only the selected fighter is expanded
//! into owned live menus; no per-fighter static timeline allocations are leaked.
use super::{Action, Catalog, Entry, Sequence, Template};
use crate::{
    button_bindings::{Builtin, ModeBinding},
    fighter_profiles::presets::PresetSlot,
    fighter_tuning::Tuning,
    reaction_rules::Rule,
};
use std::collections::BTreeMap;

#[derive(Debug)]
struct PackedSequence {
    actions: Box<[Action]>,
    start_delay: u16,
    timeout: u16,
    spatial_timeout: u16,
    mode: Option<ModeBinding>,
    builtin: Option<Builtin>,
    while_held: bool,
    ground_only: bool,
    reaction: Rule,
}
impl PackedSequence {
    fn pack(sequence: Sequence) -> Self {
        Self {
            actions: sequence.actions().into(),
            start_delay: sequence.start_delay,
            timeout: sequence.timeout,
            spatial_timeout: sequence.spatial_timeout,
            mode: sequence.mode,
            builtin: sequence.builtin,
            while_held: sequence.while_held,
            ground_only: sequence.ground_only,
            reaction: sequence.reaction,
        }
    }
    fn expand(&self) -> Sequence {
        let mut sequence = Sequence {
            len: self.actions.len() as u8,
            start_delay: self.start_delay,
            timeout: self.timeout,
            spatial_timeout: self.spatial_timeout,
            mode: self.mode,
            builtin: self.builtin,
            while_held: self.while_held,
            ground_only: self.ground_only,
            reaction: self.reaction,
            ..Sequence::default()
        };
        sequence.actions[..self.actions.len()].copy_from_slice(&self.actions);
        sequence
    }
}
#[derive(Debug)]
struct PackedEntry {
    slot: PresetSlot,
    choice: String,
    name: Option<String>,
    default: bool,
    explicit_start: bool,
    sequence: PackedSequence,
}
#[derive(Debug, Default)]
struct Fighter {
    name: String,
    entries: Vec<PackedEntry>,
}
/// Parsed once at startup/on the explicit save worker. Dormant fighters retain
/// only their actual action payloads, not MAX_ACTIONS-sized live tapes/menus.
/// Lookup is indexed by native fighter identity; it never falls back to Kirby.
#[derive(Debug, Default)]
pub struct SequenceLibrary {
    fighters: BTreeMap<i32, Fighter>,
    entries: usize,
    pub tuning: BTreeMap<i32, Tuning>,
    pub templates: Vec<Template>,
}
impl SequenceLibrary {
    pub fn from_catalog(catalog: Catalog) -> Self {
        let mut library = Self {
            entries: catalog.entries.len(),
            tuning: catalog.tuning,
            templates: catalog.templates,
            ..Self::default()
        };
        for entry in catalog.entries {
            let fighter = library.fighters.entry(entry.fighter_kind).or_default();
            if fighter.name.is_empty() {
                fighter.name = entry.fighter;
            }
            fighter.entries.push(PackedEntry {
                slot: entry.slot,
                choice: entry.choice,
                name: entry.name,
                default: entry.default,
                explicit_start: entry.explicit_start,
                sequence: PackedSequence::pack(entry.sequence),
            });
        }
        // Do not retain geometric spare capacity from grouping the startup Vec.
        for fighter in library.fighters.values_mut() {
            fighter.entries.shrink_to_fit();
        }
        library
    }
    pub fn len(&self) -> usize {
        self.entries
    }
    pub fn is_empty(&self) -> bool {
        self.entries == 0
    }
    pub fn fighter_entries(&self, kind: i32) -> usize {
        self.fighters.get(&kind).map_or(0, |f| f.entries.len())
    }
    /// Creation/change-of-fighter only. Generic templates are shared separately
    /// by the identity registry; no other fighter is expanded or scanned here.
    pub fn for_fighter(&self, kind: i32) -> Catalog {
        let entries = self.fighters.get(&kind).map_or_else(Vec::new, |fighter| {
            fighter
                .entries
                .iter()
                .map(|entry| Entry {
                    fighter_kind: kind,
                    fighter: fighter.name.clone(),
                    slot: entry.slot,
                    choice: entry.choice.clone(),
                    name: entry.name.clone(),
                    default: entry.default,
                    explicit_start: entry.explicit_start,
                    sequence: entry.sequence.expand(),
                })
                .collect()
        });
        let mut tuning = BTreeMap::new();
        if let Some(value) = self.tuning.get(&kind) {
            tuning.insert(kind, *value);
        }
        Catalog {
            entries,
            tuning,
            ..Catalog::default()
        }
    }
    /// Useful for startup diagnostics/host checks, never needed by frame policy.
    pub fn action_slots(&self) -> usize {
        self.fighters
            .values()
            .flat_map(|f| &f.entries)
            .map(|e| e.sequence.actions.len())
            .sum()
    }
}
