//! Data model for named SD-card ledge choices and input recipes.
//!
//! Parsing may allocate, but live execution copies one fixed-size recipe into
//! `AssistSettings`; no strings, files, maps or allocations are touched in the
//! frame path. File parsing and validation live in the child `parser` module.
use crate::fighter_profiles::{presets::PresetSlot, FighterProfiles};
pub mod layout {
    //! Bounded, save-time presentation only. Never used by input/frame processing.
    use super::MAX_FILE_BYTES;
    const BORDER: &str = "# ========================================================================";

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
pub const MAX_ENTRIES: usize = 2112;

/// Specifies the possible directions relative to the fighter or the stage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Direction {
    #[default]
    Opponent,
    AwayFromOpponent,
    Ledge,
    Inward,
    Outward,
    Facing,
    Backward,
    SavedLedge,
    AwayFromLedge,
}

impl Direction {
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
    Ignore,
    Verify(f32),
    FighterGrabRange,
    /// Check once on entry and cancel immediately when outside standing reach.
    FighterGrabRangeInstant,
}

/// Represents a condition that must be satisfied for a sequence to be valid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    SelfGrounded,
    SelfAirborne,
    EnemyGrounded,
    EnemyAirborne,
    EnemyShielding,
    EnemyNotShielding,
    EnemyInFront,
    EnemyBehind,
    EnemyWithin,
    EnemyOutside,
}
impl Condition {
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
    pub fn from_token(token: &str) -> Option<Self> {
        [
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
        ]
        .into_iter()
        .find(|c| c.token() == token)
    }
    pub fn needs_enemy(self) -> bool {
        !matches!(self, Self::SelfGrounded | Self::SelfAirborne)
    }
    pub fn has_distance(self) -> bool {
        matches!(self, Self::EnemyWithin | Self::EnemyOutside)
    }
}
/// Specifies the reference frame for input directions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputReference {
    #[default]
    Screen,
    Facing,
    Opponent,
}
impl InputReference {
    pub const ALL: [Self; 3] = [Self::Screen, Self::Facing, Self::Opponent];
    pub fn token(self) -> &'static str {
        match self {
            Self::Screen => "screen",
            Self::Facing => "facing",
            Self::Opponent => "opponent",
        }
    }
    fn parse(s: &str) -> Result<Self, String> {
        Self::ALL
            .into_iter()
            .find(|r| r.token() == s)
            .ok_or_else(|| "input reference must be screen, facing or opponent".into())
    }
}
/// Represents a single action that can be performed by a fighter.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Action {
    #[default]
    Empty,
    /// Literal canonical controller input, with no action-readiness gate.
    Input {
        buttons: u32,
        stick: [f32; 2],
        frames: u8,
        relative: bool,
    },
    /// Literal input with horizontal axis toward (+) / away from (-) the target.
    /// Direction is captured at step entry; requires a fresh observed target.
    OpponentInput {
        buttons: u32,
        stick: [f32; 2],
        frames: u8,
    },
    /// Simultaneous gameplay buttons and both sticks; no readiness/move writes.
    Controller {
        buttons: u32,
        stick: [f32; 2],
        cstick: [f32; 2],
        frames: u8,
        reference: InputReference,
        cstick_reference: InputReference,
    },
    /// Repeated literal pulses; release stick/buttons between presses.
    Tap {
        buttons: u32,
        stick: [f32; 2],
        hold: u8,
        gap: u8,
        frames: u16,
    },
    CStick {
        stick: [f32; 2],
        frames: u8,
    },
    /// Skip when already near; otherwise chase for exactly frames before continuing.
    Chase {
        gap: f32,
        strength: f32,
        frames: u8,
    },
    Wait(u8),
    Face(Direction),
    IfFacingLedge {
        away: bool,
    },
    /// Instant current opponent-side branch; no turn or input is performed.
    IfFacingOpponent {
        away: bool,
    },
    ElseFacing,
    EndFacing,
    /// Input-only positioning at a locked ledge's inward offset.
    LedgePosition {
        offset: f32,
        strength: f32,
        tolerance: f32,
        brake: f32,
        frames: u16,
    },
    Approach {
        airborne: bool,
        strength: f32,
        frames: u8,
        gap: f32,
    },
    Walk {
        direction: Direction,
        units: f32,
        strength: f32,
        min_gap: Option<f32>,
    },
    Run {
        direction: Direction,
        units: f32,
    },
    /// Timed horizontal stick; resolves its spatial direction each update.
    Drift {
        direction: Direction,
        strength: f32,
        frames: u8,
    },
    Range(RangeCheck),
    ShieldUntilBlock(u8),
    /// Grounded shield until a fresh physical input takes control.
    ShieldUntilManual,
    /// Continue if true, else wait up to `wait` native frames, then end. Zero is immediate.
    Check {
        condition: Condition,
        value: f32,
        wait: u8,
    },
}

impl Action {
    /// Onstage reaction recipes have no saved ledge geometry.
    pub fn needs_ledge(self) -> bool {
        matches!(
            self,
            Self::IfFacingLedge { .. }
                | Self::LedgePosition { .. }
                | Self::Face(
                    Direction::Ledge
                        | Direction::Inward
                        | Direction::Outward
                        | Direction::SavedLedge
                        | Direction::AwayFromLedge
                )
                | Self::Walk {
                    direction: Direction::Ledge
                        | Direction::Inward
                        | Direction::Outward
                        | Direction::SavedLedge
                        | Direction::AwayFromLedge,
                    ..
                }
                | Self::Run {
                    direction: Direction::Ledge
                        | Direction::Inward
                        | Direction::Outward
                        | Direction::SavedLedge
                        | Direction::AwayFromLedge,
                    ..
                }
                | Self::Drift {
                    direction: Direction::Ledge
                        | Direction::Inward
                        | Direction::Outward
                        | Direction::SavedLedge
                        | Direction::AwayFromLedge,
                    ..
                }
        )
    }
}
/// A sequence of actions that can be executed by a fighter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sequence {
    pub actions: [Action; MAX_ACTIONS],
    pub len: u8,
    pub start_delay: u16,
    pub timeout: u16,
    pub spatial_timeout: u16,
    pub mode: Option<crate::button_bindings::ModeBinding>,
    pub builtin: Option<crate::button_bindings::Builtin>,
    pub while_held: bool,
    /// Opt-in: stop this tape if the fighter becomes airborne.
    pub ground_only: bool,
    pub reaction: crate::reaction_rules::Rule,
}
impl Default for Sequence {
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
    pub fn actions(&self) -> &[Action] {
        &self.actions[..usize::from(self.len)]
    }
}
/// Represents a single entry in the catalog, corresponding to a specific fighter and preset slot.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub fighter_kind: i32,
    pub fighter: String,
    pub slot: PresetSlot,
    pub choice: String,
    pub name: Option<String>,
    pub default: bool,
    explicit_start: bool,
    pub sequence: Sequence,
}

/// Inert, fighter-independent Training editor starter; never a live response.
#[derive(Clone, Debug, PartialEq)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub sequence: Sequence,
}
pub const MAX_TEMPLATES: usize = 64;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Catalog {
    entries: Vec<Entry>,
    pub templates: Vec<Template>,
    pub universal: Option<String>,
    pub tuning: std::collections::BTreeMap<i32, crate::fighter_tuning::Tuning>,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_input::*;
    fn input(buttons: u32, stick: [f32; 2], frames: u8, relative: bool) -> Action {
        Action::Input {
            buttons,
            stick,
            frames,
            relative,
        }
    }

    fn parse(actions: &str) -> Sequence {
        Catalog::parse(&format!(
            "[kirby.drop.custom]\nstart_delay=0\nactions = {actions}"
        ))
        .unwrap()
        .entries()[0]
            .sequence
    }
    #[test]
    fn only_universal_bindings_are_ignored_and_fighter_bindings_stay_separate() {
        let source="[universal.x.old]\nold invalid text ignored\n[kirby.x.custom]\nstart_delay=0\nactions=short_hop\n[yoshi.x.custom]\nstart_delay=2\nactions=grab\n[kirby.y.custom]\nstart_delay=1\nactions=shield\n";
        let c = Catalog::parse(source).unwrap();
        assert_eq!(c.len(), 3);
        assert_ne!(c.entries()[0].fighter_kind, c.entries()[1].fighter_kind);
        assert_ne!(c.entries()[0].sequence, c.entries()[1].sequence);
        assert!(Catalog::parse("[kirby.x.bad]\nstart_delay=oops\nactions=grab").is_err());
    }
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

    #[test]
    fn timeline_accepts_ordered_multi_move_air_specials_and_explicit_durations() {
        let sequence = parse("Short Hop, wait:2, Up B:3, wait:6, Down Special, release, grab");
        assert_eq!(sequence.len, 7);
        assert_eq!(sequence.actions()[0], input(MAPPER_X, [0.0; 2], 1, true));
        assert_eq!(sequence.actions()[1], Action::Wait(2));
        assert_eq!(sequence.actions()[2], input(MAPPER_B, [0.0, 1.0], 3, true));
        assert_eq!(sequence.actions()[4], input(MAPPER_B, [0.0, -1.0], 1, true));
    }
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
    #[derive(Clone, Copy, Debug)]
    pub struct InputStep {
        pub frames: u16,
        pub recipe: InputRecipe,
    }

    impl InputStep {
        pub const fn hold(frames: u16, recipe: InputRecipe) -> Self {
            Self { frames, recipe }
        }
    
        pub const fn wait(frames: u16) -> Self {
            Self::hold(frames, InputRecipe::NONE)
        }
    }

    /// A borrowed sequence with an explicit frame cursor supplied by its caller.
    /// Start at zero when the trigger fires. Cancel by stopping evaluation.
    /// Advance only once per simulation frame, never once per hook invocation.
    pub struct InputSequence<'a> {
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
        pub recipe: InputRecipe,
        pub accepted_status: i32,
        pub require_hit: bool,
        pub timeout_frames: u16,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SequenceObservation {
        /// A real simulation-frame ID supplied by the adapter, never a hook count.
        pub frame: u64,
        pub target: u32,
        pub status: i32,
        pub can_act: bool,
        pub hit_confirmed: bool,
        pub stopped: bool,
        pub interrupted: bool,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum SequenceState {
        Ready,
        WaitingForConfirmation,
        Complete,
        Cancelled,
    }

    /// Building block for measured combos. Not wired to live playback slots: each
    /// combo still needs a verified trigger, simulation clock and move/hit evidence.
    pub struct ConfirmedSequence<'a> {
        steps: &'a [ConfirmedStep],
        target: u32,
        index: usize,
        issued: bool,
        age: u64,
        last_frame: Option<u64>,
        state: SequenceState,
        output: InputRecipe,
    }

    impl<'a> ConfirmedSequence<'a> {
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

        pub fn state(&self) -> SequenceState {
            self.state
        }

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
