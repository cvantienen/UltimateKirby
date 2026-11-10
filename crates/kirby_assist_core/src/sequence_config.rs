//! Data model for named SD-card ledge choices and input recipes.
//!
//! Parsing may allocate, but live execution copies one fixed-size recipe into
//! `AssistSettings`; no strings, files, maps or allocations are touched in the
//! frame path. File parsing and validation live in the child `parser` module.
use crate::fighter_profiles::{presets::PresetSlot, FighterProfiles};
pub mod layout {
    //! Bounded, save-time presentation only. Never used by input/frame processing.
    use super::MAX_FILE_BYTES;
    /// Technical: Defines the `BORDER` constant used by this module.
    ///
    /// Plain English: Provides the BORDER constant value.
    const BORDER: &str = "# ========================================================================";

    /// Technical: Builds a catalog suitable for persistence from the current catalog state.
    ///
    /// Plain English: Prepares the catalog for saving.
    pub fn for_save(source: &str, updated: String) -> Result<String, String> {
        if source
            .lines()
            .any(|line| line.trim() == "[universal.settings]")
        {
            organize(&updated)
        } else {
            Ok(updated)
        }
    }

    /// Keep property bytes and per-category choice order; only move whole sections.
    /// Plain English: Keep property bytes and per-category choice order; only move whole sections.
    pub fn organize(source: &str) -> Result<String, String> {
        let lines: Vec<_> = source.lines().collect();
        let mut clean = String::new();
        for (i, line) in lines.iter().enumerate() {
            if *line == BORDER
                || (line.starts_with("# ---- ") && line.ends_with(" ----"))
                || (i > 0 && i + 1 < lines.len() && lines[i - 1] == BORDER && lines[i + 1] == BORDER)
            {
                continue;
            }
            clean.push_str(line);
            clean.push('\n');
        }
        let mut preamble = String::new();
        let mut sections: Vec<(String, String)> = Vec::new();
        for line in clean.lines() {
            let header = line.split('#').next().unwrap_or("").trim();
            if header.starts_with('[') && header.ends_with(']') {
                sections.push((header[1..header.len() - 1].to_owned(), String::new()));
            }
            if let Some((_, body)) = sections.last_mut() {
                body.push_str(line);
                body.push('\n');
            } else {
                preamble.push_str(line);
                preamble.push('\n');
            }
        }
        let mut fighters = Vec::new();
        for (header, _) in &sections {
            let fighter = header.split('.').next().unwrap_or("");
            if fighter != "universal" && fighter != "template" && !fighters.iter().any(|f| f == fighter)
            {
                fighters.push(fighter.to_owned());
            }
        }
        let mut output = String::new();
        if !preamble.trim().is_empty() {
            output.push_str(preamble.trim());
            output.push_str("\n\n");
        }
        for (header, body) in &sections {
            if header.starts_with("universal.") || header.starts_with("template.") {
                output.push_str("# ---- UNIVERSAL SETTINGS ----\n");
                output.push_str(body.trim());
                output.push_str("\n\n");
            }
        }
        for fighter in fighters {
            let title = format!("# {:^70}", fighter.replace('_', " ").to_uppercase());
            output.push_str(&format!("{BORDER}\n{}\n{BORDER}\n\n", title.trim_end()));
            let mut blocks: Vec<_> = sections
                .iter()
                .filter(|(h, _)| h.split('.').next() == Some(fighter.as_str()))
                .collect();
            // Stable sort preserves each category's menu order and implicit default.
            blocks.sort_by_key(|(h, body)| {
                assist_group(body).map_or(rank(h.split('.').nth(1).unwrap_or("")), |(n, _)| 100 + n)
            });
            let mut previous = String::new();
            for (header, body) in blocks {
                let slot = header.split('.').nth(1).unwrap_or("");
                let mode = assist_group(body);
                let group = if slot == "settings" {
                    "FIGHTER SETTINGS".to_owned()
                } else if let Some((_, name)) = mode {
                    format!("ASSIST RULES: {name}")
                } else if ["neutral", "jump", "roll", "attack", "drop"].contains(&slot) {
                    format!("LEDGE TRAPS: {}", slot.to_uppercase())
                } else {
                    format!("BUTTON SEQUENCES: {}", slot.to_uppercase())
                };
                if group != previous {
                    output.push_str(&format!("# ---- {group} ----\n"));
                    previous = group;
                }
                if slot != "settings" {
                    let name = body
                        .lines()
                        .find_map(|l| {
                            l.trim()
                                .split_once('=')
                                .filter(|(k, _)| k.trim() == "name")
                                .map(|(_, v)| v.trim())
                        })
                        .unwrap_or(header);
                    output.push_str(&format!("# ---- {name} | {header} ----\n"));
                }
                output.push_str(body.trim());
                output.push_str("\n\n");
            }
        }
        if output.len() as u64 > MAX_FILE_BYTES {
            // Presentation must not reject an otherwise valid explicit save. A
            // dense roster can fit on disk while generated name/ID banners do not.
            // Keep the edited source verbatim rather than discard authored comments
            // or increase the parser/file limits merely for decorative formatting.
            if source.len() as u64 <= MAX_FILE_BYTES {
                return Ok(source.to_owned());
            }
            return Err("sequence file exceeds 512 KiB".into());
        }
        Ok(output)
    }
    /// Technical: Returns an optional `assist_group` value.
    ///
    /// Plain English: Gets assist group if it is available.
    fn assist_group(body: &str) -> Option<(usize, &'static str)> {
        let token = body.lines().find_map(|line| {
            let (key, value) = line.split('#').next()?.split_once('=')?;
            (key.trim() == "reaction").then(|| value.trim().split(':').next().unwrap_or(""))
        })?;
        match token {
            "roll" | "roll_cross" | "roll_vulnerable" | "roll_cross_vulnerable" => {
                Some((0, "ROLL REACTIONS"))
            }
            "shield_hit" | "shield_hit_vulnerable" => Some((1, "OUT OF SHIELD")),
            "grab_dodged" | "grab_dodged_vulnerable" => Some((2, "GRAB DODGE + PUNISH")),
            "ledge_regrab" => Some((3, "VULNERABLE LEDGE / REGRAB")),
            "auto_roll" | "auto_spot_dodge" | "auto_air_dodge" | "auto_grab" | "auto_shield_grab"
            | "auto_grab_dodged" | "auto_shield_backup" => Some((6, "AUTO REACT")),
            "spot_dodge" | "spot_dodge_any" => Some((4, "SPOT DODGE PUNISH")),
            "shield_pressure_vulnerable"
            | "enemy_shield"
            | "shield_roll"
            | "shield_roll_cross"
            | "shield_roll_vulnerable"
            | "shield_roll_cross_vulnerable"
            | "shield_roll_away"
            | "shield_roll_away_vulnerable" => Some((5, "SHIELD PRESSURE")),
            _ => None,
        }
    }
    /// Technical: Returns the ordering rank used to sort the value.
    ///
    /// Plain English: Returns where this value belongs in the ordering.
    fn rank(slot: &str) -> usize {
        [
            "settings",
            "neutral",
            "jump",
            "roll",
            "attack",
            "drop",
            "x",
            "y",
            "dpad_up",
            "dpad_down",
            "dpad_left",
            "dpad_right",
        ]
        .iter()
        .position(|s| *s == slot)
        .unwrap_or(99)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        /// Technical: Regression test for `stable_grouping_and_repeat_saves`.
        ///
        /// Plain English: Checks that stable grouping and repeat saves.
        #[test]
        fn stable_grouping_and_repeat_saves() {
            let source = "[universal.settings]\nenabled = true\n[kirby.roll.a]\nname = First\nactions = grab\n[yoshi.neutral.a]\nactions = grab\n[kirby.settings]\nwalk_strength = 0.4\n[kirby.roll.b]\nname = Second\nactions = grab\n";
            let result = organize(source).unwrap();
            assert_eq!(organize(&result).unwrap(), result);
            assert!(result.find("[kirby.settings]").unwrap() < result.find("[kirby.roll.a]").unwrap());
            assert!(result.find("[kirby.roll.a]").unwrap() < result.find("[kirby.roll.b]").unwrap());
            assert!(result.find("[kirby.roll.b]").unwrap() < result.find("[yoshi.neutral.a]").unwrap());
            assert!(result.starts_with("# ---- UNIVERSAL SETTINGS ----"));
        }

        /// Technical: Regression test for `decorative_headers_never_reject_a_valid_near_limit_source`.
        ///
        /// Plain English: Checks that decorative headers never reject a valid near limit source.
        #[test]
        fn decorative_headers_never_reject_a_valid_near_limit_source() {
            let mut source = String::from("[universal.settings]\nenabled=true\n");
            let mut count = 0;
            'fighters: for fighter in ["mario", "luigi", "kirby", "yoshi", "bowser"] {
                for slot in [
                    "neutral",
                    "jump",
                    "roll",
                    "attack",
                    "drop",
                    "x",
                    "y",
                    "dpad_up",
                    "dpad_down",
                    "dpad_left",
                    "dpad_right",
                    "l3",
                    "r3",
                ] {
                    for i in 0..32 {
                        source.push_str(&format!("[{fighter}.{slot}.option_{i}]\nname = A saved option\nstart_delay=0\nactions=up_b:1\n"));
                        count += 1;
                        if count == 2040 {
                            break 'fighters;
                        }
                    }
                }
            }
            let remaining = MAX_FILE_BYTES as usize - source.len();
            source.push('#');
            source.extend(std::iter::repeat('x').take(remaining - 2));
            source.push('\n');
            assert_eq!(source.len() as u64, MAX_FILE_BYTES);
            crate::sequence_config::persist::validate_output(&source).unwrap();
            assert_eq!(organize(&source).unwrap(), source);
            assert_eq!(for_save(&source, source.clone()).unwrap(), source);
            source.push('x');
            assert!(organize(&source).is_err());
        }
    }
}
mod library {
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

    /// Technical: Defines the `PackedSequence` data structure.
    ///
    /// Plain English: Holds the data for PackedSequence.
    #[derive(Debug)]
    struct PackedSequence {
        /// Technical: Stores the `actions` value in `PackedSequence` as `Box<[Action]>`.
        ///
        /// Plain English: Holds the actions value used by PackedSequence.
        actions: Box<[Action]>,
        /// Technical: Stores the `start_delay` value in `PackedSequence` as `u16`.
        ///
        /// Plain English: Holds the start delay value used by PackedSequence.
        start_delay: u16,
        /// Technical: Stores the `timeout` value in `PackedSequence` as `u16`.
        ///
        /// Plain English: Holds the timeout value used by PackedSequence.
        timeout: u16,
        /// Technical: Stores the `spatial_timeout` value in `PackedSequence` as `u16`.
        ///
        /// Plain English: Holds the spatial timeout value used by PackedSequence.
        spatial_timeout: u16,
        /// Technical: Stores the `mode` value in `PackedSequence` as `Option<ModeBinding>`.
        ///
        /// Plain English: Optionally stores the mode for PackedSequence; it may be absent.
        mode: Option<ModeBinding>,
        /// Technical: Stores the `builtin` value in `PackedSequence` as `Option<Builtin>`.
        ///
        /// Plain English: Optionally stores the builtin for PackedSequence; it may be absent.
        builtin: Option<Builtin>,
        /// Technical: Stores the `while_held` value in `PackedSequence` as `bool`.
        ///
        /// Plain English: Holds the while held value used by PackedSequence.
        while_held: bool,
        /// Technical: Stores the `ground_only` value in `PackedSequence` as `bool`.
        ///
        /// Plain English: Holds the ground only value used by PackedSequence.
        ground_only: bool,
        /// Technical: Stores the `reaction` value in `PackedSequence` as `Rule`.
        ///
        /// Plain English: Holds the reaction value used by PackedSequence.
        reaction: Rule,
    }
    impl PackedSequence {
        /// Technical: Encodes the value into its compact packed representation.
        ///
        /// Plain English: Packs the value into a smaller representation.
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
        /// Technical: Expands the compact representation into its component values.
        ///
        /// Plain English: Unpacks the value into its parts.
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
    /// Technical: Defines the `PackedEntry` data structure.
    ///
    /// Plain English: Holds the data for PackedEntry.
    #[derive(Debug)]
    struct PackedEntry {
        /// Technical: Stores the `slot` value in `PackedEntry` as `PresetSlot`.
        ///
        /// Plain English: Holds the slot value used by PackedEntry.
        slot: PresetSlot,
        /// Technical: Stores the `choice` value in `PackedEntry` as `String`.
        ///
        /// Plain English: Holds the choice value used by PackedEntry.
        choice: String,
        /// Technical: Stores the `name` value in `PackedEntry` as `Option<String>`.
        ///
        /// Plain English: Optionally stores the name for PackedEntry; it may be absent.
        name: Option<String>,
        /// Technical: Stores the `default` value in `PackedEntry` as `bool`.
        ///
        /// Plain English: Returns the default value for this type.
        default: bool,
        /// Technical: Stores the `explicit_start` value in `PackedEntry` as `bool`.
        ///
        /// Plain English: Holds the explicit start value used by PackedEntry.
        explicit_start: bool,
        /// Technical: Stores the `sequence` value in `PackedEntry` as `PackedSequence`.
        ///
        /// Plain English: Holds the sequence value used by PackedEntry.
        sequence: PackedSequence,
    }
    /// Technical: Defines the `Fighter` data structure.
    ///
    /// Plain English: Holds the data for Fighter.
    #[derive(Debug, Default)]
    struct Fighter {
        /// Technical: Stores the `name` value in `Fighter` as `String`.
        ///
        /// Plain English: Holds the name value used by Fighter.
        name: String,
        /// Technical: Stores the `entries` value in `Fighter` as `Vec<PackedEntry>`.
        ///
        /// Plain English: Holds the entries value used by Fighter.
        entries: Vec<PackedEntry>,
    }
    /// Parsed once at startup/on the explicit save worker. Dormant fighters retain
    /// only their actual action payloads, not MAX_ACTIONS-sized live tapes/menus.
    /// Lookup is indexed by native fighter identity; it never falls back to Kirby.
    /// Plain English: Parsed once at startup/on the explicit save worker. Dormant fighters retain only their actual action payloads, not MAXACTIONS-sized live tapes/menus. Lookup is indexed by native fighter identity; it never falls back to Kirby.
    #[derive(Debug, Default)]
    pub struct SequenceLibrary {
        /// Technical: Stores the `fighters` value in `SequenceLibrary` as `BTreeMap<i32`.
        ///
        /// Plain English: Holds the fighters value used by SequenceLibrary.
        fighters: BTreeMap<i32, Fighter>,
        /// Technical: Stores the `entries` value in `SequenceLibrary` as `usize`.
        ///
        /// Plain English: Holds the entries value used by SequenceLibrary.
        entries: usize,
        /// Technical: Stores the `tuning` value in `SequenceLibrary` as `BTreeMap<i32`.
        ///
        /// Plain English: Holds the tuning value used by SequenceLibrary.
        pub tuning: BTreeMap<i32, Tuning>,
        /// Technical: Stores the `templates` value in `SequenceLibrary` as `Vec<Template>`.
        ///
        /// Plain English: Holds the templates value used by SequenceLibrary.
        pub templates: Vec<Template>,
    }
    impl SequenceLibrary {
        /// Technical: Constructs a value from the catalog.
        ///
        /// Plain English: Constructs a value from the catalog.
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
        /// Technical: Returns the number of stored elements.
        ///
        /// Plain English: Counts the stored items.
        pub fn len(&self) -> usize {
            self.entries
        }
        /// Technical: Checks whether empty.
        ///
        /// Plain English: Checks whether empty.
        pub fn is_empty(&self) -> bool {
            self.entries == 0
        }
        /// Technical: Returns the `fighter_entries` result.
        ///
        /// Plain English: Gets the fighter entries result.
        pub fn fighter_entries(&self, kind: i32) -> usize {
            self.fighters.get(&kind).map_or(0, |f| f.entries.len())
        }
        /// Creation/change-of-fighter only. Generic templates are shared separately
        /// by the identity registry; no other fighter is expanded or scanned here.
        /// Plain English: Creation/change-of-fighter only. Generic templates are shared separately by the identity registry; no other fighter is expanded or scanned here.
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
        /// Plain English: Useful for startup diagnostics/host checks, never needed by frame policy.
        pub fn action_slots(&self) -> usize {
            self.fighters
                .values()
                .flat_map(|f| &f.entries)
                .map(|e| e.sequence.actions.len())
                .sum()
        }
    }
}
mod parser;
pub mod persist;
pub use library::SequenceLibrary;
pub use parser::validate_facing_blocks;
use parser::{parse_slot, slot_name, valid_id, words_id};
/// Maximum number of actions in a sequence.
/// Plain English: Maximum number of actions in a sequence.
pub const MAX_ACTIONS: usize = 32;
/// Maximum size of a sequence file in bytes.
/// Plain English: Maximum size of a sequence file in bytes.
pub const MAX_FILE_BYTES: u64 = 512 * 1024;
/// Maximum number of frames in a sequence.
/// Plain English: Maximum number of frames in a sequence.
pub const MAX_SEQUENCE_FRAME_COUNT: u16 = 3600;
// Admission bound only: parser allocation uses the actual header count, and
// without removing any saved choices; the 512 KiB file limit remains unchanged.
/// Maximum number of entries in the sequence library.
/// Plain English: Maximum number of entries in the sequence library.
pub const MAX_ENTRIES: usize = 2112;

/// Specifies the possible directions relative to the fighter or the stage.
/// Plain English: Specifies the possible directions relative to the fighter or the stage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Direction {
    #[default]
    /// Technical: Represents the `Opponent` case of `Direction`.
    ///
    /// Plain English: Represents the Opponent option in Direction.
    Opponent,
    /// Technical: Represents the `AwayFromOpponent` case of `Direction`.
    ///
    /// Plain English: Represents the AwayFromOpponent option in Direction.
    AwayFromOpponent,
    /// Technical: Represents the `Ledge` case of `Direction`.
    ///
    /// Plain English: Represents the Ledge option in Direction.
    Ledge,
    /// Technical: Represents the `Inward` case of `Direction`.
    ///
    /// Plain English: Represents the Inward option in Direction.
    Inward,
    /// Technical: Represents the `Outward` case of `Direction`.
    ///
    /// Plain English: Represents the Outward option in Direction.
    Outward,
    /// Technical: Represents the `Facing` case of `Direction`.
    ///
    /// Plain English: Represents the Facing option in Direction.
    Facing,
    /// Technical: Represents the `Backward` case of `Direction`.
    ///
    /// Plain English: Represents the Backward option in Direction.
    Backward,
    /// Technical: Represents the `SavedLedge` case of `Direction`.
    ///
    /// Plain English: Represents the SavedLedge option in Direction.
    SavedLedge,
    /// Technical: Represents the `AwayFromLedge` case of `Direction`.
    ///
    /// Plain English: Represents the AwayFromLedge option in Direction.
    AwayFromLedge,
}

impl Direction {
    /// Accepted spellings per direction. The first is canonical: it is what
    /// saves write, so every direction round-trips through the parser.
    /// Plain English: Accepted spellings per direction. The first is canonical: it is what saves write, so every direction round-trips through the parser.
    const TOKENS: [(Self, &'static [&'static str]); 9] = [
        (
            Self::Opponent,
            &[
                "opponent",
                "enemy",
                "toward",
                "towards",
                "toward_opponent",
                "towards_opponent",
                "toward_enemy",
                "towards_enemy",
            ],
        ),
        (
            Self::AwayFromOpponent,
            &["away_from_opponent", "away", "away_from_enemy"],
        ),
        (Self::Ledge, &["ledge"]),
        (Self::Inward, &["inward"]),
        (Self::Outward, &["outward"]),
        (Self::Facing, &["facing", "forward"]),
        (Self::Backward, &["backward", "back"]),
        (Self::SavedLedge, &["saved_ledge", "toward_saved_ledge"]),
        (
            Self::AwayFromLedge,
            &["away_from_ledge", "away_from_saved_ledge"],
        ),
    ];

    /// Canonical spelling, as written by saves.
    /// Plain English: Canonical spelling, as written by saves.
    pub fn token(self) -> &'static str {
        Self::TOKENS[self as usize].1[0]
    }

    /// Technical: Parses the supplied token into the corresponding value.
    ///
    /// Plain English: Turns the token into the value it names.
    pub fn from_token(token: &str) -> Option<Self> {
        Self::TOKENS
            .iter()
            .find(|(_, spellings)| spellings.contains(&token))
            .map(|&(direction, _)| direction)
    }

    /// Directions resolved against saved ledge geometry; onstage reaction
    /// recipes have none.
    /// Plain English: Directions resolved against saved ledge geometry; onstage reaction recipes have none.
    pub fn needs_ledge(self) -> bool {
        matches!(
            self,
            Self::Ledge | Self::Inward | Self::Outward | Self::SavedLedge | Self::AwayFromLedge
        )
    }

    /// Technical: Returns the display label for the value.
    ///
    /// Plain English: Gets the text shown for this value.
    pub fn label(self) -> &'static str {
        match self {
            Self::Opponent => "Opponent",
            Self::AwayFromOpponent => "Away from opponent",
            Self::Ledge => "Ledge (legacy)",
            Self::Inward => "Inward",
            Self::Outward => "Outward",
            Self::Facing => "Facing",
            Self::Backward => "Backward",
            Self::SavedLedge => "Toward saved ledge",
            Self::AwayFromLedge => "Away from saved ledge",
        }
    }
}
/// Specifies how to check the range for an action or condition.
/// Plain English: Specifies how to check the range for an action or condition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RangeCheck {
    /// Technical: Represents the `Ignore` case of `RangeCheck`.
    ///
    /// Plain English: Represents the Ignore option in RangeCheck.
    Ignore,
    /// Technical: Represents the `Verify` case of `RangeCheck`.
    ///
    /// Plain English: Represents the Verify option in RangeCheck.
    Verify(
        /// Technical: Carries the verification threshold in `RangeCheck`.
        ///
        /// Plain English: Holds the verification threshold for this `RangeCheck` choice.
        f32,
    ),
    /// Technical: Represents the `FighterGrabRange` case of `RangeCheck`.
    ///
    /// Plain English: Represents the FighterGrabRange option in RangeCheck.
    FighterGrabRange,
    /// Check once on entry and cancel immediately when outside standing reach.
    /// Plain English: Check once on entry and cancel immediately when outside standing reach.
    FighterGrabRangeInstant,
}

/// Represents a condition that must be satisfied for a sequence to be valid.
/// Plain English: Represents a condition that must be satisfied for a sequence to be valid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    /// Technical: Represents the `SelfGrounded` case of `Condition`.
    ///
    /// Plain English: Represents the SelfGrounded option in Condition.
    SelfGrounded,
    /// Technical: Represents the `SelfAirborne` case of `Condition`.
    ///
    /// Plain English: Represents the SelfAirborne option in Condition.
    SelfAirborne,
    /// Technical: Represents the `EnemyGrounded` case of `Condition`.
    ///
    /// Plain English: Represents the EnemyGrounded option in Condition.
    EnemyGrounded,
    /// Technical: Represents the `EnemyAirborne` case of `Condition`.
    ///
    /// Plain English: Represents the EnemyAirborne option in Condition.
    EnemyAirborne,
    /// Technical: Represents the `EnemyShielding` case of `Condition`.
    ///
    /// Plain English: Represents the EnemyShielding option in Condition.
    EnemyShielding,
    /// Technical: Represents the `EnemyNotShielding` case of `Condition`.
    ///
    /// Plain English: Represents the EnemyNotShielding option in Condition.
    EnemyNotShielding,
    /// Technical: Represents the `EnemyInFront` case of `Condition`.
    ///
    /// Plain English: Represents the EnemyInFront option in Condition.
    EnemyInFront,
    /// Technical: Represents the `EnemyBehind` case of `Condition`.
    ///
    /// Plain English: Represents the EnemyBehind option in Condition.
    EnemyBehind,
    /// Technical: Represents the `EnemyWithin` case of `Condition`.
    ///
    /// Plain English: Represents the EnemyWithin option in Condition.
    EnemyWithin,
    /// Technical: Represents the `EnemyOutside` case of `Condition`.
    ///
    /// Plain English: Represents the EnemyOutside option in Condition.
    EnemyOutside,
}
impl Condition {
    /// Technical: Defines the `ALL` constant used by this module.
    ///
    /// Plain English: Provides the ALL constant value.
    pub const ALL: [Self; 10] = [
        Self::SelfGrounded,
        Self::SelfAirborne,
        Self::EnemyGrounded,
        Self::EnemyAirborne,
        Self::EnemyShielding,
        Self::EnemyNotShielding,
        Self::EnemyInFront,
        Self::EnemyBehind,
        Self::EnemyWithin,
        Self::EnemyOutside,
    ];
    /// Technical: Returns the serialized token for the value.
    ///
    /// Plain English: Gets the text form used when saving this value.
    pub fn token(self) -> &'static str {
        match self {
            Self::SelfGrounded => "self_grounded",
            Self::SelfAirborne => "self_airborne",
            Self::EnemyGrounded => "enemy_grounded",
            Self::EnemyAirborne => "enemy_airborne",
            Self::EnemyShielding => "enemy_shielding",
            Self::EnemyNotShielding => "enemy_not_shielding",
            Self::EnemyInFront => "enemy_in_front",
            Self::EnemyBehind => "enemy_behind",
            Self::EnemyWithin => "enemy_within",
            Self::EnemyOutside => "enemy_outside",
        }
    }
    /// Technical: Parses the supplied token into the corresponding value.
    ///
    /// Plain English: Turns the token into the value it names.
    pub fn from_token(token: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.token() == token)
    }
    /// Technical: Determines whether the action requires an enemy target.
    ///
    /// Plain English: Checks whether this action needs an opponent.
    pub fn needs_enemy(self) -> bool {
        !matches!(self, Self::SelfGrounded | Self::SelfAirborne)
    }
    /// Technical: Checks whether distance.
    ///
    /// Plain English: Checks whether distance.
    pub fn has_distance(self) -> bool {
        matches!(self, Self::EnemyWithin | Self::EnemyOutside)
    }
}
/// Specifies the reference frame for input directions.
/// Plain English: Specifies the reference frame for input directions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputReference {
    #[default]
    /// Technical: Represents the `Screen` case of `InputReference`.
    ///
    /// Plain English: Represents the Screen option in InputReference.
    Screen,
    /// Technical: Represents the `Facing` case of `InputReference`.
    ///
    /// Plain English: Represents the Facing option in InputReference.
    Facing,
    /// Technical: Represents the `Opponent` case of `InputReference`.
    ///
    /// Plain English: Represents the Opponent option in InputReference.
    Opponent,
}
impl InputReference {
    /// Technical: Defines the `ALL` constant used by this module.
    ///
    /// Plain English: Provides the ALL constant value.
    pub const ALL: [Self; 3] = [Self::Screen, Self::Facing, Self::Opponent];
    /// Technical: Returns the serialized token for the value.
    ///
    /// Plain English: Gets the text form used when saving this value.
    pub fn token(self) -> &'static str {
        match self {
            Self::Screen => "screen",
            Self::Facing => "facing",
            Self::Opponent => "opponent",
        }
    }
    /// Technical: Parses the supplied source into the module’s validated representation.
    ///
    /// Plain English: Reads the source and checks that it is valid.
    fn parse(s: &str) -> Result<Self, String> {
        Self::ALL
            .into_iter()
            .find(|r| r.token() == s)
            .ok_or_else(|| "input reference must be screen, facing or opponent".into())
    }
}
/// Canonical button spellings, in the order saves list chords.
/// Plain English: Canonical button spellings, in the order saves list chords.
const BUTTON_TOKENS: [(u32, &str); 8] = [
    (crate::native_input::MAPPER_A, "a"),
    (crate::native_input::MAPPER_B, "b"),
    (crate::native_input::MAPPER_X, "x"),
    (crate::native_input::MAPPER_Y, "y"),
    (crate::native_input::MAPPER_L, "l"),
    (crate::native_input::MAPPER_R, "r"),
    (crate::native_input::MAPPER_ZL, "zl"),
    (crate::native_input::MAPPER_ZR, "zr"),
];

/// Represents a single action that can be performed by a fighter.
/// Plain English: Represents a single action that can be performed by a fighter.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Action {
    #[default]
    /// Technical: Represents the `Empty` case of `Action`.
    ///
    /// Plain English: Represents the Empty option in Action.
    Empty,
    /// Literal canonical controller input, with no action-readiness gate.
    /// Plain English: Literal canonical controller input, with no action-readiness gate.
    Input {
        /// Technical: Stores the `buttons` value in `Action` as `u32`.
        ///
        /// Plain English: Holds the buttons value used by Action.
        buttons: u32,
        /// Technical: Stores the `stick` value in `Action` as `[f32; 2]`.
        ///
        /// Plain English: Stores the stick collection for Action.
        stick: [f32; 2],
        /// Technical: Stores the `frames` value in `Action` as `u8`.
        ///
        /// Plain English: Holds the frames value used by Action.
        frames: u8,
        /// Technical: Stores the `relative` value in `Action` as `bool`.
        ///
        /// Plain English: Holds the relative value used by Action.
        relative: bool,
    },
    /// Literal input with horizontal axis toward (+) / away from (-) the target.
    /// Direction is captured at step entry; requires a fresh observed target.
    /// Plain English: Literal input with horizontal axis toward (+) / away from (-) the target. Direction is captured at step entry; requires a fresh observed target.
    OpponentInput {
        /// Technical: Stores the `buttons` value in `Action` as `u32`.
        ///
        /// Plain English: Holds the buttons value used by Action.
        buttons: u32,
        /// Technical: Stores the `stick` value in `Action` as `[f32; 2]`.
        ///
        /// Plain English: Stores the stick collection for Action.
        stick: [f32; 2],
        /// Technical: Stores the `frames` value in `Action` as `u8`.
        ///
        /// Plain English: Holds the frames value used by Action.
        frames: u8,
    },
    /// Simultaneous gameplay buttons and both sticks; no readiness/move writes.
    /// Plain English: Simultaneous gameplay buttons and both sticks; no readiness/move writes.
    Controller {
        /// Technical: Stores the `buttons` value in `Action` as `u32`.
        ///
        /// Plain English: Holds the buttons value used by Action.
        buttons: u32,
        /// Technical: Stores the `stick` value in `Action` as `[f32; 2]`.
        ///
        /// Plain English: Stores the stick collection for Action.
        stick: [f32; 2],
        /// Technical: Stores the `cstick` value in `Action` as `[f32; 2]`.
        ///
        /// Plain English: Stores the cstick collection for Action.
        cstick: [f32; 2],
        /// Technical: Stores the `frames` value in `Action` as `u8`.
        ///
        /// Plain English: Holds the frames value used by Action.
        frames: u8,
        /// Technical: Stores the `reference` value in `Action` as `InputReference`.
        ///
        /// Plain English: Holds the reference value used by Action.
        reference: InputReference,
        /// Technical: Stores the `cstick_reference` value in `Action` as `InputReference`.
        ///
        /// Plain English: Holds the cstick reference value used by Action.
        cstick_reference: InputReference,
    },
    /// Repeated literal pulses; release stick/buttons between presses.
    /// Plain English: Repeated literal pulses; release stick/buttons between presses.
    Tap {
        /// Technical: Stores the `buttons` value in `Action` as `u32`.
        ///
        /// Plain English: Holds the buttons value used by Action.
        buttons: u32,
        /// Technical: Stores the `stick` value in `Action` as `[f32; 2]`.
        ///
        /// Plain English: Stores the stick collection for Action.
        stick: [f32; 2],
        /// Technical: Stores the `hold` value in `Action` as `u8`.
        ///
        /// Plain English: Holds the hold value used by Action.
        hold: u8,
        /// Technical: Stores the `gap` value in `Action` as `u8`.
        ///
        /// Plain English: Holds the gap value used by Action.
        gap: u8,
        /// Technical: Stores the `frames` value in `Action` as `u16`.
        ///
        /// Plain English: Holds the frames value used by Action.
        frames: u16,
    },
    /// Technical: Represents the `CStick` case of `Action`.
    ///
    /// Plain English: Represents the CStick option in Action.
    CStick {
        /// Technical: Stores the `stick` value in `Action` as `[f32; 2]`.
        ///
        /// Plain English: Stores the stick collection for Action.
        stick: [f32; 2],
        /// Technical: Stores the `frames` value in `Action` as `u8`.
        ///
        /// Plain English: Holds the frames value used by Action.
        frames: u8,
    },
    /// Skip when already near; otherwise chase for exactly frames before continuing.
    /// Plain English: Skip when already near; otherwise chase for exactly frames before continuing.
    Chase {
        /// Technical: Stores the `gap` value in `Action` as `f32`.
        ///
        /// Plain English: Holds the gap value used by Action.
        gap: f32,
        /// Technical: Stores the `strength` value in `Action` as `f32`.
        ///
        /// Plain English: Holds the strength value used by Action.
        strength: f32,
        /// Technical: Stores the `frames` value in `Action` as `u8`.
        ///
        /// Plain English: Holds the frames value used by Action.
        frames: u8,
    },
    /// Technical: Represents the `Wait` case of `Action`.
    ///
    /// Plain English: Represents the Wait option in Action.
    Wait(
        /// Technical: Carries the number of frames to wait in `Action`.
        ///
        /// Plain English: Holds the number of frames to wait for this `Action` choice.
        u8,
    ),
    /// Technical: Represents the `Face` case of `Action`.
    ///
    /// Plain English: Represents the Face option in Action.
    Face(
        /// Technical: Carries the facing direction in `Action`.
        ///
        /// Plain English: Holds the facing direction for this `Action` choice.
        Direction,
    ),
    /// Technical: Represents the `IfFacingLedge` case of `Action`.
    ///
    /// Plain English: Represents the IfFacingLedge option in Action.
    IfFacingLedge {
        /// Technical: Stores the `away` value in `Action` as `bool`.
        ///
        /// Plain English: Holds the away value used by Action.
        away: bool,
    },
    /// Instant current opponent-side branch; no turn or input is performed.
    /// Plain English: Instant current opponent-side branch; no turn or input is performed.
    IfFacingOpponent {
        /// Technical: Stores the `away` value in `Action` as `bool`.
        ///
        /// Plain English: Holds the away value used by Action.
        away: bool,
    },
    /// Technical: Represents the `ElseFacing` case of `Action`.
    ///
    /// Plain English: Represents the ElseFacing option in Action.
    ElseFacing,
    /// Technical: Represents the `EndFacing` case of `Action`.
    ///
    /// Plain English: Represents the EndFacing option in Action.
    EndFacing,
    /// Input-only positioning at a locked ledge's inward offset.
    /// Plain English: Input-only positioning at a locked ledge's inward offset.
    LedgePosition {
        /// Technical: Stores the `offset` value in `Action` as `f32`.
        ///
        /// Plain English: Holds the offset value used by Action.
        offset: f32,
        /// Technical: Stores the `strength` value in `Action` as `f32`.
        ///
        /// Plain English: Holds the strength value used by Action.
        strength: f32,
        /// Technical: Stores the `tolerance` value in `Action` as `f32`.
        ///
        /// Plain English: Holds the tolerance value used by Action.
        tolerance: f32,
        /// Technical: Stores the `brake` value in `Action` as `f32`.
        ///
        /// Plain English: Holds the brake value used by Action.
        brake: f32,
        /// Technical: Stores the `frames` value in `Action` as `u16`.
        ///
        /// Plain English: Holds the frames value used by Action.
        frames: u16,
    },
    /// Technical: Represents the `Approach` case of `Action`.
    ///
    /// Plain English: Represents the Approach option in Action.
    Approach {
        /// Technical: Stores the `airborne` value in `Action` as `bool`.
        ///
        /// Plain English: Holds the airborne value used by Action.
        airborne: bool,
        /// Technical: Stores the `strength` value in `Action` as `f32`.
        ///
        /// Plain English: Holds the strength value used by Action.
        strength: f32,
        /// Technical: Stores the `frames` value in `Action` as `u8`.
        ///
        /// Plain English: Holds the frames value used by Action.
        frames: u8,
        /// Technical: Stores the `gap` value in `Action` as `f32`.
        ///
        /// Plain English: Holds the gap value used by Action.
        gap: f32,
    },
    /// Technical: Represents the `Walk` case of `Action`.
    ///
    /// Plain English: Represents the Walk option in Action.
    Walk {
        /// Technical: Stores the `direction` value in `Action` as `Direction`.
        ///
        /// Plain English: Holds the direction value used by Action.
        direction: Direction,
        /// Technical: Stores the `units` value in `Action` as `f32`.
        ///
        /// Plain English: Holds the units value used by Action.
        units: f32,
        /// Technical: Stores the `strength` value in `Action` as `f32`.
        ///
        /// Plain English: Holds the strength value used by Action.
        strength: f32,
        /// Technical: Stores the `min_gap` value in `Action` as `Option<f32>`.
        ///
        /// Plain English: Optionally stores the min gap for Action; it may be absent.
        min_gap: Option<f32>,
    },
    /// Technical: Represents the `Run` case of `Action`.
    ///
    /// Plain English: Represents the Run option in Action.
    Run {
        /// Technical: Stores the `direction` value in `Action` as `Direction`.
        ///
        /// Plain English: Holds the direction value used by Action.
        direction: Direction,
        /// Technical: Stores the `units` value in `Action` as `f32`.
        ///
        /// Plain English: Holds the units value used by Action.
        units: f32,
    },
    /// Timed horizontal stick; resolves its spatial direction each update.
    /// Plain English: Timed horizontal stick; resolves its spatial direction each update.
    Drift {
        /// Technical: Stores the `direction` value in `Action` as `Direction`.
        ///
        /// Plain English: Holds the direction value used by Action.
        direction: Direction,
        /// Technical: Stores the `strength` value in `Action` as `f32`.
        ///
        /// Plain English: Holds the strength value used by Action.
        strength: f32,
        /// Technical: Stores the `frames` value in `Action` as `u8`.
        ///
        /// Plain English: Holds the frames value used by Action.
        frames: u8,
    },
    /// Technical: Represents the `Range` case of `Action`.
    ///
    /// Plain English: Represents the Range option in Action.
    Range(
        /// Technical: Carries the range condition in `Action`.
        ///
        /// Plain English: Holds the range condition for this `Action` choice.
        RangeCheck,
    ),
    /// Technical: Represents the `ShieldUntilBlock` case of `Action`.
    ///
    /// Plain English: Represents the ShieldUntilBlock option in Action.
    ShieldUntilBlock(
        /// Technical: Carries the maximum shield duration in `Action`.
        ///
        /// Plain English: Holds the maximum shield duration for this `Action` choice.
        u8,
    ),
    /// Grounded shield until a fresh physical input takes control.
    /// Plain English: Grounded shield until a fresh physical input takes control.
    ShieldUntilManual,
    /// Continue if true, else wait up to `wait` native frames, then end. Zero is immediate.
    /// Plain English: Continue if true, else wait up to wait native frames, then end. Zero is immediate.
    Check {
        /// Technical: Stores the `condition` value in `Action` as `Condition`.
        ///
        /// Plain English: Holds the condition value used by Action.
        condition: Condition,
        /// Technical: Stores the `value` value in `Action` as `f32`.
        ///
        /// Plain English: Holds the value value used by Action.
        value: f32,
        /// Technical: Stores the `wait` value in `Action` as `u8`.
        ///
        /// Plain English: Holds the wait value used by Action.
        wait: u8,
    },
}

impl Action {
    /// Onstage reaction recipes have no saved ledge geometry.
    /// Plain English: Onstage reaction recipes have no saved ledge geometry.
    pub fn needs_ledge(self) -> bool {
        match self {
            Self::IfFacingLedge { .. } | Self::LedgePosition { .. } => true,
            Self::Face(direction)
            | Self::Walk { direction, .. }
            | Self::Run { direction, .. }
            | Self::Drift { direction, .. } => direction.needs_ledge(),
            _ => false,
        }
    }
}
/// A sequence of actions that can be executed by a fighter.
/// Plain English: A sequence of actions that can be executed by a fighter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sequence {
    /// Technical: Stores the `actions` value in `Sequence` as `[Action; MAX_ACTIONS]`.
    ///
    /// Plain English: Stores the actions collection for Sequence.
    pub actions: [Action; MAX_ACTIONS],
    /// Technical: Stores the `len` value in `Sequence` as `u8`.
    ///
    /// Plain English: Holds the len value used by Sequence.
    pub len: u8,
    /// Technical: Stores the `start_delay` value in `Sequence` as `u16`.
    ///
    /// Plain English: Holds the start delay value used by Sequence.
    pub start_delay: u16,
    /// Technical: Stores the `timeout` value in `Sequence` as `u16`.
    ///
    /// Plain English: Holds the timeout value used by Sequence.
    pub timeout: u16,
    /// Technical: Stores the `spatial_timeout` value in `Sequence` as `u16`.
    ///
    /// Plain English: Holds the spatial timeout value used by Sequence.
    pub spatial_timeout: u16,
    /// Technical: Stores the `mode` value in `Sequence` as `Option<crate::button_bindings::ModeBinding>`.
    ///
    /// Plain English: Optionally stores the mode for Sequence; it may be absent.
    pub mode: Option<crate::button_bindings::ModeBinding>,
    /// Technical: Stores the `builtin` value in `Sequence` as `Option<crate::button_bindings::Builtin>`.
    ///
    /// Plain English: Optionally stores the builtin for Sequence; it may be absent.
    pub builtin: Option<crate::button_bindings::Builtin>,
    /// Technical: Stores the `while_held` value in `Sequence` as `bool`.
    ///
    /// Plain English: Holds the while held value used by Sequence.
    pub while_held: bool,
    /// Opt-in: stop this tape if the fighter becomes airborne.
    /// Plain English: Opt-in: stop this tape if the fighter becomes airborne.
    pub ground_only: bool,
    /// Technical: Stores the `reaction` value in `Sequence` as `crate::reaction_rules::Rule`.
    ///
    /// Plain English: Holds the reaction value used by Sequence.
    pub reaction: crate::reaction_rules::Rule,
}
impl Default for Sequence {
    /// Technical: Returns the default value for this type.
    ///
    /// Plain English: Returns the default value for this type.
    fn default() -> Self {
        Self {
            actions: [Action::Empty; MAX_ACTIONS],
            len: 0,
            start_delay: 0,
            timeout: MAX_SEQUENCE_FRAME_COUNT,
            spatial_timeout: 60,
            mode: None,
            builtin: None,
            while_held: false,
            ground_only: false,
            reaction: crate::reaction_rules::Rule::default(),
        }
    }
}
impl Sequence {
    /// Technical: Returns the actions represented by the parsed input.
    ///
    /// Plain English: Gets the actions in the input.
    pub fn actions(&self) -> &[Action] {
        &self.actions[..usize::from(self.len)]
    }
}
/// Represents a single entry in the catalog, corresponding to a specific fighter and preset slot.
/// Plain English: Represents a single entry in the catalog, corresponding to a specific fighter and preset slot.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    /// Technical: Stores the `fighter_kind` value in `Entry` as `i32`.
    ///
    /// Plain English: Holds the fighter kind value used by Entry.
    pub fighter_kind: i32,
    /// Technical: Stores the `fighter` value in `Entry` as `String`.
    ///
    /// Plain English: Holds the fighter value used by Entry.
    pub fighter: String,
    /// Technical: Stores the `slot` value in `Entry` as `PresetSlot`.
    ///
    /// Plain English: Holds the slot value used by Entry.
    pub slot: PresetSlot,
    /// Technical: Stores the `choice` value in `Entry` as `String`.
    ///
    /// Plain English: Holds the choice value used by Entry.
    pub choice: String,
    /// Technical: Stores the `name` value in `Entry` as `Option<String>`.
    ///
    /// Plain English: Optionally stores the name for Entry; it may be absent.
    pub name: Option<String>,
    /// Technical: Stores the `default` value in `Entry` as `bool`.
    ///
    /// Plain English: Returns the default value for this type.
    pub default: bool,
    /// Technical: Stores the `explicit_start` value in `Entry` as `bool`.
    ///
    /// Plain English: Holds the explicit start value used by Entry.
    explicit_start: bool,
    /// Technical: Stores the `sequence` value in `Entry` as `Sequence`.
    ///
    /// Plain English: Holds the sequence value used by Entry.
    pub sequence: Sequence,
}

/// Inert, fighter-independent Training editor starter; never a live response.
/// Plain English: Inert, fighter-independent Training editor starter; never a live response.
#[derive(Clone, Debug, PartialEq)]
pub struct Template {
    /// Technical: Stores the `id` value in `Template` as `String`.
    ///
    /// Plain English: Holds the id value used by Template.
    pub id: String,
    /// Technical: Stores the `name` value in `Template` as `String`.
    ///
    /// Plain English: Holds the name value used by Template.
    pub name: String,
    /// Technical: Stores the `sequence` value in `Template` as `Sequence`.
    ///
    /// Plain English: Holds the sequence value used by Template.
    pub sequence: Sequence,
}
/// Technical: Sets the upper bound for templates.
///
/// Plain English: Holds the MAX TEMPLATES value.
pub const MAX_TEMPLATES: usize = 64;

/// Technical: Defines the `Catalog` data structure.
///
/// Plain English: Holds the data for Catalog.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Catalog {
    /// Technical: Stores the `entries` value in `Catalog` as `Vec<Entry>`.
    ///
    /// Plain English: Holds the entries value used by Catalog.
    entries: Vec<Entry>,
    /// Technical: Stores the `templates` value in `Catalog` as `Vec<Template>`.
    ///
    /// Plain English: Holds the templates value used by Catalog.
    pub templates: Vec<Template>,
    /// Technical: Stores the `universal` value in `Catalog` as `Option<String>`.
    ///
    /// Plain English: Optionally stores the universal for Catalog; it may be absent.
    pub universal: Option<String>,
    /// Technical: Stores the `tuning` value in `Catalog` as `std::collections::BTreeMap<i32`.
    ///
    /// Plain English: Holds the tuning value used by Catalog.
    pub tuning: std::collections::BTreeMap<i32, crate::fighter_tuning::Tuning>,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_input::*;
    /// Technical: Constructs a parsed input action from the supplied arguments.
    ///
    /// Plain English: Builds the input action described by the arguments.
    fn input(buttons: u32, stick: [f32; 2], frames: u8, relative: bool) -> Action {
        Action::Input {
            buttons,
            stick,
            frames,
            relative,
        }
    }

    /// Technical: Parses the supplied source into the module’s validated representation.
    ///
    /// Plain English: Reads the source and checks that it is valid.
    fn parse(actions: &str) -> Sequence {
        Catalog::parse(&format!(
            "[kirby.drop.custom]\nstart_delay=0\nactions = {actions}"
        ))
        .unwrap()
        .entries()[0]
            .sequence
    }
    /// Technical: Regression test for `direction_spellings_are_unique_and_the_canonical_one_round_trips`.
    ///
    /// Plain English: Checks that direction spellings are unique and the canonical one round trips.
    #[test]
    fn direction_spellings_are_unique_and_the_canonical_one_round_trips() {
        for (index, (direction, spellings)) in Direction::TOKENS.iter().enumerate() {
            assert_eq!(*direction as usize, index, "TOKENS must follow enum order");
            assert_eq!(direction.token(), spellings[0]);
            for spelling in *spellings {
                assert_eq!(Direction::from_token(spelling), Some(*direction));
            }
        }
        let mut all: Vec<_> = Direction::TOKENS
            .iter()
            .flat_map(|(_, spellings)| spellings.iter())
            .collect();
        let count = all.len();
        all.sort();
        all.dedup();
        assert_eq!(all.len(), count, "a spelling maps to two directions");
        assert_eq!(Direction::from_token("sideways"), None);
    }

    /// Technical: Regression test for `only_saved_ledge_geometry_directions_need_a_ledge`.
    ///
    /// Plain English: Checks that only saved ledge geometry directions need a ledge.
    #[test]
    fn only_saved_ledge_geometry_directions_need_a_ledge() {
        for (direction, _) in Direction::TOKENS {
            let needs = !matches!(
                direction,
                Direction::Opponent
                    | Direction::AwayFromOpponent
                    | Direction::Facing
                    | Direction::Backward
            );
            for action in [
                Action::Face(direction),
                Action::Walk {
                    direction,
                    units: 1.0,
                    strength: 0.5,
                    min_gap: None,
                },
                Action::Run {
                    direction,
                    units: 1.0,
                },
                Action::Drift {
                    direction,
                    strength: 0.5,
                    frames: 1,
                },
            ] {
                assert_eq!(action.needs_ledge(), needs, "{action:?}");
            }
        }
        assert!(Action::IfFacingLedge { away: true }.needs_ledge());
        assert!(!Action::IfFacingOpponent { away: true }.needs_ledge());
        assert!(!Action::Wait(1).needs_ledge());
    }

    /// Technical: Regression test for `only_universal_bindings_are_ignored_and_fighter_bindings_stay_separate`.
    ///
    /// Plain English: Checks that only universal bindings are ignored and fighter bindings stay separate.
    #[test]
    fn only_universal_bindings_are_ignored_and_fighter_bindings_stay_separate() {
        let source="[universal.x.old]\nold invalid text ignored\n[kirby.x.custom]\nstart_delay=0\nactions=short_hop\n[yoshi.x.custom]\nstart_delay=2\nactions=grab\n[kirby.y.custom]\nstart_delay=1\nactions=shield\n";
        let c = Catalog::parse(source).unwrap();
        assert_eq!(c.len(), 3);
        assert_ne!(c.entries()[0].fighter_kind, c.entries()[1].fighter_kind);
        assert_ne!(c.entries()[0].sequence, c.entries()[1].sequence);
        assert!(Catalog::parse("[kirby.x.bad]\nstart_delay=oops\nactions=grab").is_err());
    }
    /// Technical: Regression test for `file_facing_inputs_and_special_strength_are_explicit_and_bounded`.
    ///
    /// Plain English: Checks that file facing inputs and special strength are explicit and bounded.
    #[test]
    fn file_facing_inputs_and_special_strength_are_explicit_and_bounded() {
        let s = parse("facing_input:A:1:-0.5:1, side_special:2:0.5");
        assert_eq!(s.actions()[0], input(MAPPER_A, [1.0, -0.5], 1, true));
        assert_eq!(s.actions()[1], input(MAPPER_B, [0.5, 0.0], 2, true));
        assert!(Catalog::parse("[kirby.neutral.bad]\nstart_delay=0\nactions=full_hop").is_err());
        assert!(
            Catalog::parse("[kirby.neutral.bad]\nstart_delay=0\nactions=up_special:1:2").is_err()
        );
    }

    /// Technical: Regression test for `timeline_accepts_ordered_multi_move_air_specials_and_explicit_durations`.
    ///
    /// Plain English: Checks that timeline accepts ordered multi move air specials and explicit durations.
    #[test]
    fn timeline_accepts_ordered_multi_move_air_specials_and_explicit_durations() {
        let sequence = parse("Short Hop, wait:2, Up B:3, wait:6, Down Special, release, grab");
        assert_eq!(sequence.len, 7);
        assert_eq!(sequence.actions()[0], input(MAPPER_X, [0.0; 2], 1, true));
        assert_eq!(sequence.actions()[1], Action::Wait(2));
        assert_eq!(sequence.actions()[2], input(MAPPER_B, [0.0, 1.0], 3, true));
        assert_eq!(sequence.actions()[4], input(MAPPER_B, [0.0, -1.0], 1, true));
    }
    /// Technical: Regression test for `timeline_raw_inputs_preserve_signed_axes_and_button_chords`.
    ///
    /// Plain English: Checks that timeline raw inputs preserve signed axes and button chords.
    #[test]
    fn timeline_raw_inputs_preserve_signed_axes_and_button_chords() {
        let sequence =
            parse("input:A+X:-1:0.5:3, cstick:0:-1:1, hold:shield:8, press:B, stick:1:0:4");
        assert_eq!(
            sequence.actions()[0],
            input(MAPPER_A | MAPPER_X, [-1.0, 0.5], 3, false)
        );
        assert_eq!(
            sequence.actions()[1],
            Action::CStick {
                stick: [0.0, -1.0],
                frames: 1
            }
        );
        assert_eq!(sequence.actions()[2], input(MAPPER_ZR, [0.0; 2], 8, false));
    }
    /// Technical: Regression test for `timeline_universal_actions_validate_for_every_configured_fighter_without_hitbox_data`.
    ///
    /// Plain English: Checks that timeline universal actions validate for every configured fighter without hitbox data.
    #[test]
    fn timeline_universal_actions_validate_for_every_configured_fighter_without_hitbox_data() {
        let profiles = FighterProfiles::default();
        let mut checked = 0;
        for kind in 0..crate::fighter_kinds::PLAYABLE_KIND_COUNT {
            if !profiles
                .profile(kind)
                .is_some_and(|p| p.fighter_kind == kind)
            {
                continue;
            }
            let mut catalog = Catalog::parse("[kirby.neutral.everything]\nstart_delay=0\nactions = nair, fair, bair, uair, dair, jab, ftilt, utilt, dtilt, fsmash, usmash, dsmash, neutral_special, side_special, up_special, down_special, dash_attack, shield, roll_forward, roll_backward, spot_dodge, air_dodge, grab").unwrap();
            catalog.entries[0].fighter_kind = kind;
            catalog.validate(&profiles).unwrap();
            checked += 1;
        }
        assert!(checked > 70);
    }
    /// Technical: Regression test for `timeline_numeric_keys_are_order_independent_and_duplicates_invalid`.
    ///
    /// Plain English: Checks that timeline numeric keys are order independent and duplicates invalid.
    #[test]
    fn timeline_numeric_keys_are_order_independent_and_duplicates_invalid() {
        let catalog = Catalog::parse("[kirby.roll.custom]\nstart_delay = 7\ntimeout = 200\nspatial_timeout = 15\nactions = dash_attack:3:2\nname = My attack").unwrap();
        let sequence = catalog.entries()[0].sequence;
        assert_eq!(
            (
                sequence.start_delay,
                sequence.timeout,
                sequence.spatial_timeout
            ),
            (7, 200, 15)
        );
        assert_eq!(sequence.actions()[0], input(0, [1.0, 0.0], 3, true));
        assert_eq!(sequence.actions()[1], input(MAPPER_A, [1.0, 0.0], 2, true));
        assert!(
            Catalog::parse("[kirby.roll.custom]\nstart_delay=1\nstart_delay=2\nactions=jab")
                .is_err()
        );
    }
    /// Technical: Regression test for `timeline_file_menus_require_explicit_delays_and_have_no_code_fallback`.
    ///
    /// Plain English: Checks that timeline file menus require explicit delays and have no code fallback.
    #[test]
    fn timeline_file_menus_require_explicit_delays_and_have_no_code_fallback() {
        assert!(Catalog::parse("[kirby.neutral.custom]\nactions=jab")
            .unwrap_err()
            .contains("requires start_delay"));
        assert!(
            Catalog::parse("[kirby.neutral.custom]\nstart_delay=0\nactions=code:nair").is_err()
        );
        let catalog = Catalog::parse("[kirby.neutral.first]\nstart_delay=0\ndefault=true\nactions=jab\n[kirby.neutral.second]\nstart_delay=2\ndefault=true\nactions=up_b").unwrap();
        assert!(catalog
            .validate(&FighterProfiles::default())
            .unwrap_err()
            .contains("multiple defaults"));
    }

    /// Technical: Regression test for `timeline_bounds_unknown_names_and_nonfinite_axes_are_rejected`.
    ///
    /// Plain English: Checks that timeline bounds unknown names and nonfinite axes are rejected.
    #[test]
    fn timeline_bounds_unknown_names_and_nonfinite_axes_are_rejected() {
        for actions in [
            "up_special:0",
            "input:A:NaN:0:1",
            "input:A:2:0:1",
            "hold:bad:2",
            "jab,,jab",
            "cstick:0:1:256",
            "unknown",
            "walk:away:5:0.5:2",
        ] {
            assert!(
                Catalog::parse(&format!(
                    "[kirby.neutral.custom]\nstart_delay=0\nactions={actions}"
                ))
                .is_err(),
                "{actions}"
            );
        }
        let long = vec!["jab"; MAX_ACTIONS + 1].join(",");
        assert!(Catalog::parse(&format!(
            "[kirby.neutral.custom]\nstart_delay=0\nactions={long}"
        ))
        .is_err());
        assert!(Catalog::parse(
            "[kirby.neutral.custom]\nstart_delay=0\nactions=jab\n[kirby.neutral.custom]\nstart_delay=0\nactions=up_special"
        )
        .is_err());
    }
    /// Technical: Regression test for `timeline_deployment_validates_and_filters_only_diagnostic_tests`.
    ///
    /// Plain English: Checks that timeline deployment validates and filters only diagnostic tests.
    #[test]
    fn timeline_deployment_validates_and_filters_only_diagnostic_tests() {
        let catalog = Catalog::parse(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../deployment/ultimate/kirby_assist/assembled/sequences.txt"
        )))
        .unwrap();
        for diagnostic in [false, true] {
            let mut profiles = FighterProfiles::default();
            profiles
                .add_sequence_choices(&catalog.clone().for_build(diagnostic))
                .unwrap();
        }
        assert!(catalog.clone().for_build(false).len() < catalog.for_build(true).len());
    }
}

#[cfg(test)]
mod facing_block_tests {
    use super::*;
    /// Technical: Regression test for `markers_validate_and_save_roundtrip`.
    ///
    /// Plain English: Checks that markers validate and save roundtrip.
    #[test]
    fn markers_validate_and_save_roundtrip() {
        let source="[yoshi.neutral.test]\nstart_delay=0\nactions=if_facing_ledge, drift:saved_ledge:0.25:4, end_facing, if_back_to_ledge, walk:away_from_ledge:4:0.3, end_facing";
        let c = Catalog::parse(source).unwrap();
        let s = c.entries()[0].sequence;
        let saved = persist::update(source, 5, PresetSlot::Neutral, "test", s, false).unwrap();
        assert_eq!(Catalog::parse(&saved).unwrap().entries()[0].sequence, s);
        for actions in [
            "end_facing",
            "if_facing_ledge, a",
            "if_facing_ledge, if_back_to_ledge, a, end_facing, end_facing",
        ] {
            assert!(Catalog::parse(&format!(
                "[yoshi.neutral.test]\nstart_delay=0\nactions={actions}"
            ))
            .is_err());
        }
    }
}

#[cfg(test)]
mod generic_template_tests {
    use super::*;
    /// Technical: Regression test for `named_templates_are_inert_bounded_and_survive_organizing_and_saving`.
    ///
    /// Plain English: Checks that named templates are inert bounded and survive organizing and saving.
    #[test]
    fn named_templates_are_inert_bounded_and_survive_organizing_and_saving() {
        let source="[universal.settings]\nenabled=true\n[template.shield]\nname=Quick shield\nstart_delay=0\nactions=input:zr:0:0:4\n[kirby.neutral.test]\nstart_delay=0\nactions=jab:2\n";
        let c = Catalog::parse(source).unwrap();
        assert_eq!(c.len(), 1);
        assert_eq!(c.templates.len(), 1);
        assert_eq!(c.templates[0].id, "shield");
        assert_eq!(c.templates[0].name, "Quick shield");
        let organized = layout::organize(source).unwrap();
        let again = Catalog::parse(&organized).unwrap();
        assert_eq!(again.templates, c.templates);
        let updated = persist::update(
            source,
            6,
            PresetSlot::Neutral,
            "test",
            c.entries()[0].sequence,
            false,
        )
        .unwrap();
        assert_eq!(Catalog::parse(&updated).unwrap().templates, c.templates);
        for diagnostic in [false, true] {
            let c = c.clone().for_build(diagnostic);
            assert_eq!(c.templates.len(), 1);
            c.validate(&FighterProfiles::default()).unwrap();
        }
    }
    /// Technical: Regression test for `templates_cannot_activate_modes_reactions_or_defaults_and_reject_duplicate_ids`.
    ///
    /// Plain English: Checks that templates cannot activate modes reactions or defaults and reject duplicate ids.
    #[test]
    fn templates_cannot_activate_modes_reactions_or_defaults_and_reject_duplicate_ids() {
        for value in [
            "default=true",
            "while_held=true",
            "reaction=auto_roll:30:12",
            "mode=auto_tech",
            "builtin=normal",
        ] {
            assert!(
                Catalog::parse(&format!(
                    "[template.bad]\nstart_delay=0\nactions=jab\n{value}\n"
                ))
                .is_err(),
                "{value}"
            );
        }
        assert!(Catalog::parse(
            "[template.a]\nstart_delay=0\nactions=jab\n[template.a]\nstart_delay=0\nactions=grab\n"
        )
        .is_err());
        assert!(Catalog::parse("[template.a.b]\nstart_delay=0\nactions=jab\n").is_err());
        let text = (0..=MAX_TEMPLATES)
            .map(|i| format!("[template.test_{i}]\nstart_delay=0\nactions=jab\n"))
            .collect::<String>();
        assert!(Catalog::parse(&text)
            .unwrap_err()
            .contains("generic templates"));
    }
}
pub mod sequence {
    //! Frame-indexed input sequences for experimenting with custom combos.
    //! These describe proposed inputs, not guaranteed fighter actions.
    //! This sampler is used by examples. Recorded-slot playback is not connected
    //! to the live plugin, and adding steps here does not automatically run a combo.
    //! `InputSequence<'a>` borrows a slice; the steps must outlive the sequence.
    use crate::InputRecipe;

    /// Hold a recipe for this many simulation frames. An empty recipe is a wait.
    /// Plain English: Hold a recipe for this many simulation frames. An empty recipe is a wait.
    #[derive(Clone, Copy, Debug)]
    pub struct InputStep {
        /// Technical: Stores the `frames` value in `InputStep` as `u16`.
        ///
        /// Plain English: Holds the frames value used by InputStep.
        pub frames: u16,
        /// Technical: Stores the `recipe` value in `InputStep` as `InputRecipe`.
        ///
        /// Plain English: Holds the recipe value used by InputStep.
        pub recipe: InputRecipe,
    }

    impl InputStep {
        /// Technical: Returns the `hold` result.
        ///
        /// Plain English: Gets the hold result.
        pub const fn hold(frames: u16, recipe: InputRecipe) -> Self {
            Self { frames, recipe }
        }
    
        /// Technical: Returns the `wait` result.
        ///
        /// Plain English: Gets the wait result.
        pub const fn wait(frames: u16) -> Self {
            Self::hold(frames, InputRecipe::NONE)
        }
    }

    /// A borrowed sequence with an explicit frame cursor supplied by its caller.
    /// Start at zero when the trigger fires. Cancel by stopping evaluation.
    /// Advance only once per simulation frame, never once per hook invocation.
    /// Plain English: A borrowed sequence with an explicit frame cursor supplied by its caller. Start at zero when the trigger fires. Cancel by stopping evaluation. Advance only once per simulation frame, never once per hook invocation.
    pub struct InputSequence<'a> {
        /// Technical: Stores the `steps` value in `InputSequence` as `&'a [InputStep]`.
        ///
        /// Plain English: Holds the steps value used by InputSequence.
        pub steps: &'a [InputStep],
    }

    impl InputSequence<'_> {
        /// Returns None when finished. Zero-length steps are skipped.
        /// Plain English: Returns None when finished. Zero-length steps are skipped.
        pub fn recipe_at(&self, mut frame: u32) -> Option<InputRecipe> {
            // Subtract whole step durations until `frame` lies inside a step.
            // This stateless lookup can be restarted by supplying frame zero again.
            // For hold(1), wait(2), hold(1), ages 0 / 1..2 / 3 select those
            // respective steps; age 4 returns None. Some(NONE) is a deliberate wait,
            // whereas None means there are no more steps at all.
            for step in self.steps {
                if frame < u32::from(step.frames) {
                    return Some(step.recipe);
                }
                frame -= u32::from(step.frames);
            }
            None
        }
    }

    /// A reactive combo step: send one input, then wait for the requested native
    /// action (and optionally its hit). A rejected input or whiff cannot advance it.
    /// Plain English: A reactive combo step: send one input, then wait for the requested native action (and optionally its hit). A rejected input or whiff cannot advance it.
    #[derive(Clone, Copy, Debug)]
    pub struct ConfirmedStep {
        /// Technical: Stores the `recipe` value in `ConfirmedStep` as `InputRecipe`.
        ///
        /// Plain English: Holds the recipe value used by ConfirmedStep.
        pub recipe: InputRecipe,
        /// Technical: Stores the `accepted_status` value in `ConfirmedStep` as `i32`.
        ///
        /// Plain English: Holds the accepted status value used by ConfirmedStep.
        pub accepted_status: i32,
        /// Technical: Stores the `require_hit` value in `ConfirmedStep` as `bool`.
        ///
        /// Plain English: Requires hit when ConfirmedStep is used.
        pub require_hit: bool,
        /// Technical: Stores the `timeout_frames` value in `ConfirmedStep` as `u16`.
        ///
        /// Plain English: Sets the timeout frames timing for ConfirmedStep.
        pub timeout_frames: u16,
    }

    /// Technical: Defines the `SequenceObservation` data structure.
    ///
    /// Plain English: Holds the data for SequenceObservation.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SequenceObservation {
        /// A real simulation-frame ID supplied by the adapter, never a hook count.
        /// Plain English: A real simulation-frame ID supplied by the adapter, never a hook count.
        pub frame: u64,
        /// Technical: Stores the `target` value in `SequenceObservation` as `u32`.
        ///
        /// Plain English: Holds the target value used by SequenceObservation.
        pub target: u32,
        /// Technical: Stores the `status` value in `SequenceObservation` as `i32`.
        ///
        /// Plain English: Holds the status value used by SequenceObservation.
        pub status: i32,
        /// Technical: Stores the `can_act` value in `SequenceObservation` as `bool`.
        ///
        /// Plain English: Holds the can act value used by SequenceObservation.
        pub can_act: bool,
        /// Technical: Stores the `hit_confirmed` value in `SequenceObservation` as `bool`.
        ///
        /// Plain English: Holds the hit confirmed value used by SequenceObservation.
        pub hit_confirmed: bool,
        /// Technical: Stores the `stopped` value in `SequenceObservation` as `bool`.
        ///
        /// Plain English: Holds the stopped value used by SequenceObservation.
        pub stopped: bool,
        /// Technical: Stores the `interrupted` value in `SequenceObservation` as `bool`.
        ///
        /// Plain English: Holds the interrupted value used by SequenceObservation.
        pub interrupted: bool,
    }

    /// Technical: Defines the available `SequenceState` cases.
    ///
    /// Plain English: Lists the available SequenceState choices.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum SequenceState {
        /// Technical: Represents the `Ready` case of `SequenceState`.
        ///
        /// Plain English: Represents the Ready option in SequenceState.
        Ready,
        /// Technical: Represents the `WaitingForConfirmation` case of `SequenceState`.
        ///
        /// Plain English: Represents the WaitingForConfirmation option in SequenceState.
        WaitingForConfirmation,
        /// Technical: Represents the `Complete` case of `SequenceState`.
        ///
        /// Plain English: Represents the Complete option in SequenceState.
        Complete,
        /// Technical: Represents the `Cancelled` case of `SequenceState`.
        ///
        /// Plain English: Represents the Cancelled option in SequenceState.
        Cancelled,
    }

    /// Building block for measured combos. Not wired to live playback slots: each
    /// combo still needs a verified trigger, simulation clock and move/hit evidence.
    /// Plain English: Building block for measured combos. Not wired to live playback slots: each combo still needs a verified trigger, simulation clock and move/hit evidence.
    pub struct ConfirmedSequence<'a> {
        /// Technical: Stores the `steps` value in `ConfirmedSequence` as `&'a [ConfirmedStep]`.
        ///
        /// Plain English: Holds the steps value used by ConfirmedSequence.
        steps: &'a [ConfirmedStep],
        /// Technical: Stores the `target` value in `ConfirmedSequence` as `u32`.
        ///
        /// Plain English: Holds the target value used by ConfirmedSequence.
        target: u32,
        /// Technical: Stores the `index` value in `ConfirmedSequence` as `usize`.
        ///
        /// Plain English: Holds the index value used by ConfirmedSequence.
        index: usize,
        /// Technical: Stores the `issued` value in `ConfirmedSequence` as `bool`.
        ///
        /// Plain English: Holds the issued value used by ConfirmedSequence.
        issued: bool,
        /// Technical: Stores the `age` value in `ConfirmedSequence` as `u64`.
        ///
        /// Plain English: Holds the age value used by ConfirmedSequence.
        age: u64,
        /// Technical: Stores the `last_frame` value in `ConfirmedSequence` as `Option<u64>`.
        ///
        /// Plain English: Optionally stores the last frame for ConfirmedSequence; it may be absent.
        last_frame: Option<u64>,
        /// Technical: Stores the `state` value in `ConfirmedSequence` as `SequenceState`.
        ///
        /// Plain English: Holds the state value used by ConfirmedSequence.
        state: SequenceState,
        /// Technical: Stores the `output` value in `ConfirmedSequence` as `InputRecipe`.
        ///
        /// Plain English: Holds the output value used by ConfirmedSequence.
        output: InputRecipe,
    }

    impl<'a> ConfirmedSequence<'a> {
        /// Technical: Constructs a new value with the type’s initial state.
        ///
        /// Plain English: Creates a new value.
        pub fn new(steps: &'a [ConfirmedStep], target: u32) -> Self {
            Self {
                steps,
                target,
                index: 0,
                issued: false,
                age: 0,
                last_frame: None,
                state: if steps.is_empty() {
                    SequenceState::Complete
                } else {
                    SequenceState::Ready
                },
                output: InputRecipe::NONE,
            }
        }

        /// Technical: Returns the current state value.
        ///
        /// Plain English: Gets the current state.
        pub fn state(&self) -> SequenceState {
            self.state
        }

        /// Technical: Updates the toggle state from the current button sample and match state.
        ///
        /// Plain English: Checks the current buttons and updates whether the toggle should activate.
        pub fn sample(&mut self, observation: SequenceObservation) -> InputRecipe {
            // Cancellation wins over the duplicate-frame cache, including damage
            // or target reassignment between callbacks in the same simulation frame.
            if observation.interrupted
                || observation.target != self.target
                || self.last_frame.is_some_and(|last| observation.frame < last)
            {
                self.state = SequenceState::Cancelled;
            }
            if matches!(
                self.state,
                SequenceState::Complete | SequenceState::Cancelled
            ) {
                return InputRecipe::NONE;
            }
            if observation.stopped {
                self.last_frame = Some(observation.frame);
                self.output = InputRecipe::NONE;
                return self.output;
            }
            if self.last_frame == Some(observation.frame) {
                return self.output;
            }
            // Backward frames already cancel above; duplicate frames already return.
            let elapsed =
                crate::timing::simulation_elapsed(self.last_frame, observation.frame).unwrap_or(0);
            self.last_frame = Some(observation.frame);
            self.output = InputRecipe::NONE;
            let step = self.steps[self.index];
            self.age = self.age.saturating_add(elapsed);
            if self.issued {
                if self.age > u64::from(step.timeout_frames.max(1)) {
                    self.state = SequenceState::Cancelled;
                    return self.output;
                }
                if observation.status == step.accepted_status
                    && (!step.require_hit || observation.hit_confirmed)
                {
                    self.index += 1;
                    self.issued = false;
                    self.age = 0;
                    self.state = if self.index == self.steps.len() {
                        SequenceState::Complete
                    } else {
                        SequenceState::Ready
                    };
                    // Separate confirmation and the next request by a release frame.
                    return self.output;
                }
                if self.age >= u64::from(step.timeout_frames.max(1)) {
                    self.state = SequenceState::Cancelled;
                }
            } else if self.age >= u64::from(step.timeout_frames.max(1)) {
                self.state = SequenceState::Cancelled;
            } else if observation.can_act && observation.status != step.accepted_status {
                self.output = step.recipe;
                self.issued = true;
                self.state = SequenceState::WaitingForConfirmation;
            }
            self.output
        }
    }

    // Compiled only by cargo test on the host; these assertions are not live
    // Switch logging or gameplay callbacks. Add regressions here for new behavior.
    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::ButtonSet;

        /// Technical: Regression test for `sequence_has_release_gaps_and_ends_without_stuck_inputs`.
        ///
        /// Plain English: Checks that sequence has release gaps and ends without stuck inputs.
        #[test]
        fn sequence_has_release_gaps_and_ends_without_stuck_inputs() {
            let attack = InputRecipe::buttons(ButtonSet::ATTACK);
            let steps = [
                InputStep::hold(1, attack),
                InputStep::wait(2),
                InputStep::hold(1, attack),
            ];
            let sequence = InputSequence { steps: &steps };
            assert_eq!(sequence.recipe_at(0), Some(attack));
            assert_eq!(sequence.recipe_at(1), Some(InputRecipe::NONE));
            assert_eq!(sequence.recipe_at(2), Some(InputRecipe::NONE));
            assert_eq!(sequence.recipe_at(3), Some(attack));
            assert_eq!(sequence.recipe_at(4), None);
            assert_eq!(sequence.recipe_at(u32::MAX), None);
        }
    }
}
