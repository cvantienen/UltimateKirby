//! Text-file catalog and bounded controller-input recipes.
//!
//! A section identifies a fighter, menu slot and stable choice ID. Its ordered
//! [`Action`] values describe proposed buttons/sticks or observation checks, not
//! forced moves, hitboxes or animation changes. [`Sequence::default`] supplies
//! storage defaults; a text recipe still requires explicit `start_delay` and
//! nonempty `actions`. [`Catalog::parse`] checks grammar; [`Catalog::validate`]
//! additionally checks fighter/menu limits, facing blocks and reaction constraints.
//!
//! Parsing and persistence may allocate at startup or explicit save. Execution
//! receives a fixed-size recipe rather than parsing text in the frame path.
//! The private parser and editor helpers remain implementation details. The
//! separate [`sequence`] samplers are experimental building blocks, not the
//! live SD-file executor.
use crate::fighter_profiles::{presets::PresetSlot, FighterProfiles};
pub mod layout {
    //! Bounded, save-time presentation only. Never used by input/frame processing.
    use super::MAX_FILE_BYTES;
    /// `BORDER` is a configuration value used by the Smash runtime.
    const BORDER: &str =
        "# ========================================================================";

    /// Builds a catalog suitable for persistence from the current catalog state.
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
    pub fn organize(source: &str) -> Result<String, String> {
        if source.len() as u64 > MAX_FILE_BYTES {
            return Err("sequence file exceeds 512 KiB".into());
        }
        let lines: Vec<_> = source.lines().collect();
        let mut clean = String::new();
        for (i, line) in lines.iter().enumerate() {
            if *line == BORDER
                || (line.starts_with("# ---- ") && line.ends_with(" ----"))
                || (line.starts_with('#')
                    && i > 0
                    && i + 1 < lines.len()
                    && lines[i - 1] == BORDER
                    && lines[i + 1] == BORDER)
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
            if fighter != "universal"
                && fighter != "template"
                && !fighters.iter().any(|f| f == fighter)
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
    /// Returns an optional `assist_group` value.
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
            "auto_roll" | "auto_spot_dodge" | "auto_air_dodge" | "auto_shield_grab"
            | "auto_grab_dodged" | "auto_shield_backup" => {
                Some((6, "AUTO REACT"))
            }
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
    /// Returns the ordering rank used to sort the value.
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
        include!("../tests/unit/sequence_layout.rs");
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

    /// Runtime Smash data carried by `PackedSequence`.
    #[derive(Debug)]
    struct PackedSequence {
        /// Runtime actions value used by PackedSequence..
        actions: Box<[Action]>,
        /// Runtime start delay value used by PackedSequence..
        start_delay: u16,
        /// Runtime timeout value used by PackedSequence..
        timeout: u16,
        /// Runtime spatial timeout value used by PackedSequence..
        spatial_timeout: u16,
        /// Runtime mode value used by PackedSequence..
        mode: Option<ModeBinding>,
        /// Runtime builtin value used by PackedSequence..
        builtin: Option<Builtin>,
        /// Runtime while held value used by PackedSequence..
        while_held: bool,
        /// Runtime ground only value used by PackedSequence..
        ground_only: bool,
        /// Runtime reaction value used by PackedSequence..
        auto_tilt: Option<crate::posture::TiltSettings>,
        reaction: Rule,
    }
    impl PackedSequence {
        /// Encodes the value into its compact packed representation.
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
                auto_tilt: sequence.auto_tilt,
                reaction: sequence.reaction,
            }
        }
        /// Expands the compact representation into its component values.
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
                auto_tilt: self.auto_tilt,
                reaction: self.reaction,
                ..Sequence::default()
            };
            sequence.actions[..self.actions.len()].copy_from_slice(&self.actions);
            sequence
        }
    }
    /// Runtime Smash data carried by `PackedEntry`.
    #[derive(Debug)]
    struct PackedEntry {
        /// Runtime slot value used by PackedEntry..
        slot: PresetSlot,
        /// Runtime choice value used by PackedEntry..
        choice: String,
        /// player-facing move/fighter name.
        name: Option<String>,
        /// Runtime default value used by PackedEntry..
        default: bool,
        /// Runtime explicit start value used by PackedEntry..
        explicit_start: bool,
        /// Runtime sequence value used by PackedEntry..
        sequence: PackedSequence,
    }
    /// Runtime Smash data carried by `Fighter`.
    #[derive(Debug, Default)]
    struct Fighter {
        /// player-facing move/fighter name.
        name: String,
        /// Runtime entries value used by Fighter..
        entries: Vec<PackedEntry>,
    }
    /// Parsed once at startup/on the explicit save worker. Dormant fighters retain
    /// only their actual action payloads, not MAX_ACTIONS-sized live tapes/menus.
    /// Lookup is indexed by native fighter identity; it never falls back to Kirby.
    #[derive(Debug, Default)]
    pub struct SequenceLibrary {
        /// Runtime fighters value used by SequenceLibrary..
        fighters: BTreeMap<i32, Fighter>,
        /// Runtime entries value used by SequenceLibrary..
        entries: usize,
        /// Runtime tuning value used by SequenceLibrary..
        pub tuning: BTreeMap<i32, Tuning>,
        /// Runtime templates value used by SequenceLibrary..
        pub templates: Vec<Template>,
    }
    impl SequenceLibrary {
        /// Constructs a value from the catalog.
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
        /// Returns the number of stored elements.
        pub fn len(&self) -> usize {
            self.entries
        }
        /// Checks whether empty.
        pub fn is_empty(&self) -> bool {
            self.entries == 0
        }
        /// Returns the `fighter_entries` result.
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
}
mod parser;
pub mod persist;
pub use library::SequenceLibrary;
pub use parser::validate_facing_blocks;
use parser::{parse_slot, slot_name, valid_id, words_id};
/// Maximum number of actions in a sequence.
pub const MAX_ACTIONS: usize = 32;
/// Maximum size of a sequence file in bytes.
pub const MAX_FILE_BYTES: u64 = 512 * 1024;
/// Maximum number of frames in a sequence.
pub const MAX_SEQUENCE_FRAME_COUNT: u16 = 3600;
// Admission bound only: parser allocation uses the actual header count, and
// without removing any saved choices; the 512 KiB file limit remains unchanged.
/// Maximum number of entries in the sequence library.
pub const MAX_ENTRIES: usize = 2432;

/// Specifies the possible directions relative to the fighter or the stage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Direction {
    #[default]
    /// `Opponent` option in the `Direction` Smash policy state.
    Opponent,
    /// `AwayFromOpponent` option in the `Direction` Smash policy state.
    AwayFromOpponent,
    /// `Ledge` option in the `Direction` Smash policy state.
    Ledge,
    /// `Inward` option in the `Direction` Smash policy state.
    Inward,
    /// `Outward` option in the `Direction` Smash policy state.
    Outward,
    /// `Facing` option in the `Direction` Smash policy state.
    Facing,
    /// `Backward` option in the `Direction` Smash policy state.
    Backward,
    /// `SavedLedge` option in the `Direction` Smash policy state.
    SavedLedge,
    /// `AwayFromLedge` option in the `Direction` Smash policy state.
    AwayFromLedge,
}

impl Direction {
    /// Accepted spellings per direction. The first is canonical: it is what
    /// saves write, so every direction round-trips through the parser.
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
    pub fn token(self) -> &'static str {
        Self::TOKENS[self as usize].1[0]
    }

    /// Parses the supplied token into the corresponding value.
    pub fn from_token(token: &str) -> Option<Self> {
        Self::TOKENS
            .iter()
            .find(|(_, spellings)| spellings.contains(&token))
            .map(|&(direction, _)| direction)
    }

    /// Directions resolved against saved ledge geometry; onstage reaction
    /// recipes have none.
    pub fn needs_ledge(self) -> bool {
        matches!(
            self,
            Self::Ledge | Self::Inward | Self::Outward | Self::SavedLedge | Self::AwayFromLedge
        )
    }

    /// Returns the display label for the value.
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RangeCheck {
    /// `Ignore` option in the `RangeCheck` Smash policy state.
    Ignore,
    /// `Verify` option in the `RangeCheck` Smash policy state.
    Verify(
        /// Carries the verification threshold in `RangeCheck`.
        f32,
    ),
    /// `FighterGrabRange` option in the `RangeCheck` Smash policy state.
    FighterGrabRange,
    /// Check once on entry and cancel immediately when outside standing reach.
    FighterGrabRangeInstant,
}

/// Represents a condition that must be satisfied for a sequence to be valid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    /// `SelfGrounded` option in the `Condition` Smash policy state.
    SelfGrounded,
    /// `SelfAirborne` option in the `Condition` Smash policy state.
    SelfAirborne,
    /// `EnemyGrounded` option in the `Condition` Smash policy state.
    EnemyGrounded,
    /// `EnemyAirborne` option in the `Condition` Smash policy state.
    EnemyAirborne,
    /// `EnemyShielding` option in the `Condition` Smash policy state.
    EnemyShielding,
    /// `EnemyNotShielding` option in the `Condition` Smash policy state.
    EnemyNotShielding,
    /// `EnemyInFront` option in the `Condition` Smash policy state.
    EnemyInFront,
    /// `EnemyBehind` option in the `Condition` Smash policy state.
    EnemyBehind,
    /// `EnemyWithin` option in the `Condition` Smash policy state.
    EnemyWithin,
    /// `EnemyOutside` option in the `Condition` Smash policy state.
    EnemyOutside,
}
impl Condition {
    /// `ALL` is a configuration value used by the Smash runtime.
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
    /// Returns the serialized token for the value.
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
    /// Parses the supplied token into the corresponding value.
    pub fn from_token(token: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.token() == token)
    }
    /// Determines whether the action requires an enemy target.
    pub fn needs_enemy(self) -> bool {
        !matches!(self, Self::SelfGrounded | Self::SelfAirborne)
    }
    /// Checks whether distance.
    pub fn has_distance(self) -> bool {
        matches!(self, Self::EnemyWithin | Self::EnemyOutside)
    }
}
/// Specifies the reference frame for input directions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputReference {
    #[default]
    /// `Screen` option in the `InputReference` Smash policy state.
    Screen,
    /// `Facing` option in the `InputReference` Smash policy state.
    Facing,
    /// `Opponent` option in the `InputReference` Smash policy state.
    Opponent,
}
impl InputReference {
    /// `ALL` is a configuration value used by the Smash runtime.
    pub const ALL: [Self; 3] = [Self::Screen, Self::Facing, Self::Opponent];
    /// Returns the serialized token for the value.
    pub fn token(self) -> &'static str {
        match self {
            Self::Screen => "screen",
            Self::Facing => "facing",
            Self::Opponent => "opponent",
        }
    }
    /// Parses the supplied source into the module’s validated representation.
    fn parse(s: &str) -> Result<Self, String> {
        Self::ALL
            .into_iter()
            .find(|r| r.token() == s)
            .ok_or_else(|| "input reference must be screen, facing or opponent".into())
    }
}
/// Canonical button spellings, in the order saves list chords.
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
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Action {
    #[default]
    /// `Empty` option in the `Action` Smash policy state.
    Empty,
    /// Literal canonical controller input, with no action-readiness gate.
    Input {
        /// Runtime buttons value used by Action..
        buttons: u32,
        /// Runtime stick value used by Action..
        stick: [f32; 2],
        /// Runtime frames value used by Action..
        frames: u8,
        /// Runtime relative value used by Action..
        relative: bool,
    },
    /// Literal input with horizontal axis toward (+) / away from (-) the target.
    /// Direction is captured at step entry; requires a fresh observed target.
    OpponentInput {
        /// Runtime buttons value used by Action..
        buttons: u32,
        /// Runtime stick value used by Action..
        stick: [f32; 2],
        /// Runtime frames value used by Action..
        frames: u8,
    },
    /// Simultaneous gameplay buttons and both sticks; no readiness/move writes.
    Controller {
        /// Runtime buttons value used by Action..
        buttons: u32,
        /// Runtime stick value used by Action..
        stick: [f32; 2],
        /// Runtime cstick value used by Action..
        cstick: [f32; 2],
        /// Runtime frames value used by Action..
        frames: u8,
        /// Runtime reference value used by Action..
        reference: InputReference,
        /// Runtime cstick reference value used by Action..
        cstick_reference: InputReference,
    },
    /// Repeated literal pulses; release stick/buttons between presses.
    Tap {
        /// Runtime buttons value used by Action..
        buttons: u32,
        /// Runtime stick value used by Action..
        stick: [f32; 2],
        /// Runtime hold value used by Action..
        hold: u8,
        /// Runtime gap value used by Action..
        gap: u8,
        /// Runtime frames value used by Action..
        frames: u16,
    },
    /// `CStick` option in the `Action` Smash policy state.
    CStick {
        /// Runtime stick value used by Action..
        stick: [f32; 2],
        /// Runtime frames value used by Action..
        frames: u8,
    },
    /// Skip when already near; otherwise chase for exactly frames before continuing.
    Chase {
        /// Runtime gap value used by Action..
        gap: f32,
        /// Runtime strength value used by Action..
        strength: f32,
        /// Runtime frames value used by Action..
        frames: u8,
    },
    /// `Wait` option in the `Action` Smash policy state.
    Wait(
        /// Carries the number of frames to wait in `Action`.
        u8,
    ),
    /// `Face` option in the `Action` Smash policy state.
    Face(
        /// Carries the facing direction in `Action`.
        Direction,
    ),
    /// `IfFacingLedge` option in the `Action` Smash policy state.
    IfFacingLedge {
        /// Runtime away value used by Action..
        away: bool,
    },
    /// Instant current opponent-side branch; no turn or input is performed.
    IfFacingOpponent {
        /// Runtime away value used by Action..
        away: bool,
    },
    /// `ElseFacing` option in the `Action` Smash policy state.
    ElseFacing,
    /// `EndFacing` option in the `Action` Smash policy state.
    EndFacing,
    /// Input-only positioning at a locked ledge's inward offset.
    LedgePosition {
        /// Runtime offset value used by Action..
        offset: f32,
        /// Runtime strength value used by Action..
        strength: f32,
        /// Runtime tolerance value used by Action..
        tolerance: f32,
        /// Runtime brake value used by Action..
        brake: f32,
        /// Runtime frames value used by Action..
        frames: u16,
    },
    /// `Approach` option in the `Action` Smash policy state.
    Approach {
        /// Runtime airborne value used by Action..
        airborne: bool,
        /// Runtime strength value used by Action..
        strength: f32,
        /// Runtime frames value used by Action..
        frames: u8,
        /// Runtime gap value used by Action..
        gap: f32,
    },
    /// `Walk` option in the `Action` Smash policy state.
    Walk {
        /// Runtime direction value used by Action..
        direction: Direction,
        /// Runtime units value used by Action..
        units: f32,
        /// Runtime strength value used by Action..
        strength: f32,
        /// Runtime min gap value used by Action..
        min_gap: Option<f32>,
    },
    /// `Run` option in the `Action` Smash policy state.
    Run {
        /// Runtime direction value used by Action..
        direction: Direction,
        /// Runtime units value used by Action..
        units: f32,
    },
    /// Timed horizontal stick; resolves its spatial direction each update.
    Drift {
        /// Runtime direction value used by Action..
        direction: Direction,
        /// Runtime strength value used by Action..
        strength: f32,
        /// Runtime frames value used by Action..
        frames: u8,
    },
    /// `Range` option in the `Action` Smash policy state.
    Range(
        /// Carries the range condition in `Action`.
        RangeCheck,
    ),
    /// `ShieldUntilBlock` option in the `Action` Smash policy state.
    ShieldUntilBlock(
        /// Carries the maximum shield duration in `Action`.
        u8,
    ),
    /// Grounded shield until a fresh physical input takes control.
    ShieldUntilManual,
    /// Continue if true, else wait up to `wait` native frames, then end. Zero is immediate.
    Check {
        /// Runtime condition value used by Action..
        condition: Condition,
        /// Runtime value value used by Action..
        value: f32,
        /// Runtime wait value used by Action..
        wait: u8,
    },
}

impl Action {
    /// Onstage reaction recipes have no saved ledge geometry.
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
/// Bounded input tape plus activation metadata.
///
/// Only the first `len` actions are meaningful; keep `len <= MAX_ACTIONS` when
/// constructing a value in Rust. Durations are native-progress input frames,
/// not attack startup or wall-clock milliseconds. A default value is empty and
/// is not, by itself, a valid executable text recipe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sequence {
    /// Fixed storage; trailing unused elements remain `Action::Empty`.
    pub actions: [Action; MAX_ACTIONS],
    /// Number of initialized actions, bounded by `MAX_ACTIONS`.
    pub len: u8,
    /// Neutral input frames before the first action; text range 0–3600.
    pub start_delay: u16,
    /// Whole-tape native-progress budget; text range 1–3600, default 3600.
    pub timeout: u16,
    /// Budget for supported observation-dependent steps; default 60 frames.
    pub spatial_timeout: u16,
    /// Runtime mode value used by Sequence..
    pub mode: Option<crate::button_bindings::ModeBinding>,
    /// Runtime builtin value used by Sequence..
    pub builtin: Option<crate::button_bindings::Builtin>,
    /// Runtime while held value used by Sequence..
    pub while_held: bool,
    /// Opt-in: stop this tape if the fighter becomes airborne.
    pub ground_only: bool,
    /// Retired recipe field accepted for compatibility; fighter tilt directions use `[fighter.settings]`.
    pub auto_tilt: Option<crate::posture::TiltSettings>,
    /// Runtime reaction value used by Sequence..
    pub reaction: crate::reaction_rules::Rule,
}
impl Default for Sequence {
    /// Returns the default value for this type.
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
            auto_tilt: None,
            reaction: crate::reaction_rules::Rule::default(),
        }
    }
}
impl Sequence {
    /// Returns the actions represented by the parsed input.
    pub fn actions(&self) -> &[Action] {
        &self.actions[..usize::from(self.len)]
    }
}
/// Represents a single entry in the catalog, corresponding to a specific fighter and preset slot.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    /// Runtime fighter kind value used by Entry..
    pub fighter_kind: i32,
    /// Runtime fighter value used by Entry..
    pub fighter: String,
    /// Runtime slot value used by Entry..
    pub slot: PresetSlot,
    /// Runtime choice value used by Entry..
    pub choice: String,
    /// player-facing move/fighter name.
    pub name: Option<String>,
    /// Runtime default value used by Entry..
    pub default: bool,
    /// Runtime explicit start value used by Entry..
    explicit_start: bool,
    /// Runtime sequence value used by Entry..
    pub sequence: Sequence,
}

/// Inert, fighter-independent Training editor starter; never a live response.
#[derive(Clone, Debug, PartialEq)]
pub struct Template {
    /// Stable text identifier from `[template.ID]`, not a battle-object ID.
    pub id: String,
    /// Human-readable starter label shown in the generic template picker.
    pub name: String,
    /// Runtime sequence value used by Template..
    pub sequence: Sequence,
}
/// Sets the upper bound for templates.
/// Holds the MAX TEMPLATES value.
pub const MAX_TEMPLATES: usize = 64;

/// Runtime Smash data carried by `Catalog`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Catalog {
    /// Runtime entries value used by Catalog..
    entries: Vec<Entry>,
    /// Runtime templates value used by Catalog..
    pub templates: Vec<Template>,
    /// Runtime universal value used by Catalog..
    pub universal: Option<String>,
    /// Runtime tuning value used by Catalog..
    pub tuning: std::collections::BTreeMap<i32, crate::fighter_tuning::Tuning>,
}
impl Catalog {
    /// Moves legacy Up mode-swap choices at load time without writing the source file.
    /// Any occupied R3 slot is reported before changing any entry.
    pub fn migrate_mode_swap(&mut self) -> Result<(), String> {
        use crate::button_bindings::Builtin;
        let legacy = |entry: &Entry| {
            entry.slot == PresetSlot::DpadUp && entry.sequence.builtin == Some(Builtin::LModeSwap)
        };
        let mut conflicts = Vec::new();
        for entry in self.entries.iter().filter(|entry| legacy(entry)) {
            for assigned in self.entries.iter().filter(|assigned| {
                assigned.fighter_kind == entry.fighter_kind && assigned.slot == PresetSlot::R3
            }) {
                conflicts.push(format!(
                    "{}: Up '{}' [{}] -> R3 conflicts with '{}' [{}]",
                    entry.fighter,
                    entry.name.as_deref().unwrap_or(&entry.choice),
                    entry.choice,
                    assigned.name.as_deref().unwrap_or(&assigned.choice),
                    assigned.choice
                ));
            }
        }
        if !conflicts.is_empty() {
            return Err(format!(
                "Mode-swap migration needs confirmation; no assignments changed: {}",
                conflicts.join("; ")
            ));
        }
        for entry in &mut self.entries {
            if legacy(entry) {
                entry.slot = PresetSlot::R3;
            }
        }
        Ok(())
    }
}
#[cfg(test)]
#[path = "../tests/unit/sequence_config.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/unit/sequence_facing_blocks.rs"]
mod facing_block_tests;

#[cfg(test)]
#[path = "../tests/unit/sequence_templates.rs"]
mod generic_template_tests;
pub mod sequence {
    //! Frame-indexed input sequences for experimenting with custom combos.
    //! These describe proposed inputs, not guaranteed fighter actions.
    //! This sampler is used by examples. Recorded-slot playback is not connected
    //! to the live plugin, and adding steps here does not automatically run a combo.
    //! `InputSequence<'a>` borrows a slice; the steps must outlive the sequence.
    use crate::InputRecipe;

    /// Hold a recipe for this many simulation frames. An empty recipe is a wait.
    #[derive(Clone, Copy, Debug)]
    pub struct InputStep {
        /// Runtime frames value used by InputStep..
        pub frames: u16,
        /// Runtime recipe value used by InputStep..
        pub recipe: InputRecipe,
    }

    impl InputStep {
        /// Returns the `hold` result.
        pub const fn hold(frames: u16, recipe: InputRecipe) -> Self {
            Self { frames, recipe }
        }

        /// Returns the `wait` result.
        pub const fn wait(frames: u16) -> Self {
            Self::hold(frames, InputRecipe::NONE)
        }
    }

    /// A borrowed sequence with an explicit frame cursor supplied by its caller.
    /// Start at zero when the trigger fires. Cancel by stopping evaluation.
    /// Advance only once per simulation frame, never once per hook invocation.
    pub struct InputSequence<'a> {
        /// Runtime steps value used by InputSequence..
        pub steps: &'a [InputStep],
    }

    impl InputSequence<'_> {
        /// Returns None when finished. Zero-length steps are skipped.
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
    #[derive(Clone, Copy, Debug)]
    pub struct ConfirmedStep {
        /// Runtime recipe value used by ConfirmedStep..
        pub recipe: InputRecipe,
        /// Runtime accepted status value used by ConfirmedStep..
        pub accepted_status: i32,
        /// Runtime require hit value used by ConfirmedStep..
        pub require_hit: bool,
        /// Runtime timeout frames value used by ConfirmedStep..
        pub timeout_frames: u16,
    }

    /// Runtime Smash data carried by `SequenceObservation`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SequenceObservation {
        /// A real simulation-frame ID supplied by the adapter, never a hook count.
        pub frame: u64,
        /// selected opponent/projectile target.
        pub target: u32,
        /// native fighter status.
        pub status: i32,
        /// Runtime can act value used by SequenceObservation..
        pub can_act: bool,
        /// Runtime hit confirmed value used by SequenceObservation..
        pub hit_confirmed: bool,
        /// Runtime stopped value used by SequenceObservation..
        pub stopped: bool,
        /// Runtime interrupted value used by SequenceObservation..
        pub interrupted: bool,
    }

    /// Available `SequenceState` choices for the Smash assistance menu.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum SequenceState {
        /// `Ready` option in the `SequenceState` Smash policy state.
        Ready,
        /// `WaitingForConfirmation` option in the `SequenceState` Smash policy state.
        WaitingForConfirmation,
        /// `Complete` option in the `SequenceState` Smash policy state.
        Complete,
        /// `Cancelled` option in the `SequenceState` Smash policy state.
        Cancelled,
    }

    /// Building block for measured combos. Not wired to live playback slots: each
    /// combo still needs a verified trigger, simulation clock and move/hit evidence.
    pub struct ConfirmedSequence<'a> {
        /// Runtime steps value used by ConfirmedSequence..
        steps: &'a [ConfirmedStep],
        /// selected opponent/projectile target.
        target: u32,
        /// Runtime index value used by ConfirmedSequence..
        index: usize,
        /// Runtime issued value used by ConfirmedSequence..
        issued: bool,
        /// Runtime age value used by ConfirmedSequence..
        age: u64,
        /// Runtime last frame value used by ConfirmedSequence..
        last_frame: Option<u64>,
        /// Runtime state value used by ConfirmedSequence..
        state: SequenceState,
        /// controller input output.
        output: InputRecipe,
    }

    impl<'a> ConfirmedSequence<'a> {
        /// Constructs a new value with the type’s initial state.
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

        /// Returns the current state value.
        pub fn state(&self) -> SequenceState {
            self.state
        }

        /// Updates the toggle state from the current button sample and match state.
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
        include!("../tests/unit/input_sequence.rs");
    }
}
