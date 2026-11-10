//! Parse and validate the startup recipe catalog.
//!
//! This code runs during startup or an explicit save, never in the frame loop.
use super::*;
use crate::{
    button_bindings::{Activation, Builtin, Mode, ModeBinding},
    fighter_tuning::Tuning,
    reaction_rules::Rule,
};
use std::collections::BTreeMap;

mod actions;
use actions::parse_actions;
pub use actions::validate_facing_blocks;

/// Technical: Defines the available `SectionHeader` cases.
///
/// Plain English: Lists the available SectionHeader choices.
#[derive(Clone, Copy)]
enum SectionHeader<'a> {
    /// Technical: Represents the `UniversalSettings` case of `SectionHeader`.
    ///
    /// Plain English: Represents the UniversalSettings option in SectionHeader.
    UniversalSettings,
    /// Technical: Represents the `Template` case of `SectionHeader`.
    ///
    /// Plain English: Represents the Template option in SectionHeader.
    Template { id: &'a str, countable: bool },
    /// Technical: Represents the `RetiredButtons` case of `SectionHeader`.
    ///
    /// Plain English: Represents the RetiredButtons option in SectionHeader.
    RetiredButtons,
    /// Technical: Represents the `Settings` case of `SectionHeader`.
    ///
    /// Plain English: Represents the Settings option in SectionHeader.
    Settings,
    /// Technical: Represents the `Entry` case of `SectionHeader`.
    ///
    /// Plain English: Represents the Entry option in SectionHeader.
    Entry,
}

/// Technical: Classifies the section header.
///
/// Plain English: Classifies the section header.
fn classify_section_header(line: &str) -> Option<SectionHeader<'_>> {
    if !line.starts_with('[') || !line.ends_with(']') {
        return None;
    }
    if line == "[universal.settings]" {
        return Some(SectionHeader::UniversalSettings);
    }
    if let Some(id) = line
        .strip_prefix("[template.")
        .and_then(|value| value.strip_suffix(']'))
    {
        return Some(SectionHeader::Template {
            id,
            countable: !retired_button_header(line) && !settings_header(line),
        });
    }
    if retired_button_header(line) {
        return Some(SectionHeader::RetiredButtons);
    }
    if settings_header(line) {
        return Some(SectionHeader::Settings);
    }
    Some(SectionHeader::Entry)
}

/// A line without its `#` comment. Every reader and writer of the file uses
/// this one rule, so they agree on what a header or key line is.
/// Plain English: A line without its # comment. Every reader and writer of the file uses this one rule, so they agree on what a header or key line is.
pub(super) fn strip_comment(line: &str) -> &str {
    line.split('#').next().unwrap_or("").trim()
}

/// Section headers that will hold a 32-action recipe.
/// Plain English: Section headers that will hold a 32-action recipe.
fn count_sections(source: &str) -> Result<usize, String> {
    let count = source
        .lines()
        .filter(|line| {
            matches!(
                classify_section_header(strip_comment(line)),
                Some(
                    SectionHeader::Template {
                        countable: true,
                        ..
                    } | SectionHeader::Entry
                )
            )
        })
        .take(MAX_ENTRIES + 1)
        .count();
    if count > MAX_ENTRIES {
        return Err(format!(
            "at most {MAX_ENTRIES} sequence sections are allowed"
        ));
    }
    Ok(count)
}

/// What the `key = value` lines under the current header mean.
/// Plain English: What the key = value lines under the current header mean.
#[derive(Clone, Copy)]
enum Body {
    /// Recipe keys for `Parser::current`; before any header there is none.
    /// Plain English: Recipe keys for Parser::current; before any header there is none.
    Recipe,
    /// Technical: Represents the `Retired` case of `Body`.
    ///
    /// Plain English: Represents the Retired option in Body.
    Retired,
    /// Technical: Represents the `Universal` case of `Body`.
    ///
    /// Plain English: Represents the Universal option in Body.
    Universal,
    /// Technical: Represents the `Tuning` case of `Body`.
    ///
    /// Plain English: Represents the Tuning option in Body.
    Tuning(
        /// Technical: Carries the numeric tuning value in `Body`.
        ///
        /// Plain English: Holds the numeric tuning value for this `Body` choice.
        i32,
    ),
}

/// Keys of a recipe section. Each may appear once per section.
/// Plain English: Keys of a recipe section. Each may appear once per section.
#[derive(Clone, Copy)]
enum RecipeKey {
    /// Technical: Represents the `Name` case of `RecipeKey`.
    ///
    /// Plain English: Represents the Name option in RecipeKey.
    Name,
    /// Technical: Represents the `Builtin` case of `RecipeKey`.
    ///
    /// Plain English: Represents the Builtin option in RecipeKey.
    Builtin,
    /// Technical: Represents the `Mode` case of `RecipeKey`.
    ///
    /// Plain English: Represents the Mode option in RecipeKey.
    Mode,
    /// Technical: Represents the `Activation` case of `RecipeKey`.
    ///
    /// Plain English: Represents the Activation option in RecipeKey.
    Activation,
    /// Technical: Represents the `GroundOnly` case of `RecipeKey`.
    ///
    /// Plain English: Represents the GroundOnly option in RecipeKey.
    GroundOnly,
    /// Technical: Represents the `WhileHeld` case of `RecipeKey`.
    ///
    /// Plain English: Represents the WhileHeld option in RecipeKey.
    WhileHeld,
    /// Technical: Represents the `Reaction` case of `RecipeKey`.
    ///
    /// Plain English: Represents the Reaction option in RecipeKey.
    Reaction,
    /// Technical: Represents the `Actions` case of `RecipeKey`.
    ///
    /// Plain English: Represents the Actions option in RecipeKey.
    Actions,
    /// Technical: Represents the `Default` case of `RecipeKey`.
    ///
    /// Plain English: Represents the Default option in RecipeKey.
    Default,
    /// Technical: Represents the `StartDelay` case of `RecipeKey`.
    ///
    /// Plain English: Represents the StartDelay option in RecipeKey.
    StartDelay,
    /// Technical: Represents the `Timeout` case of `RecipeKey`.
    ///
    /// Plain English: Represents the Timeout option in RecipeKey.
    Timeout,
    /// Technical: Represents the `SpatialTimeout` case of `RecipeKey`.
    ///
    /// Plain English: Represents the SpatialTimeout option in RecipeKey.
    SpatialTimeout,
}

impl RecipeKey {
    /// Technical: Parses the supplied source into the module’s validated representation.
    ///
    /// Plain English: Reads the source and checks that it is valid.
    fn parse(key: &str) -> Option<Self> {
        Some(match key {
            "name" | "label" => Self::Name,
            "builtin" => Self::Builtin,
            "mode" => Self::Mode,
            "activation" => Self::Activation,
            "ground_only" => Self::GroundOnly,
            "while_held" => Self::WhileHeld,
            "reaction" => Self::Reaction,
            "actions" => Self::Actions,
            "default" => Self::Default,
            "start_delay" => Self::StartDelay,
            "timeout" => Self::Timeout,
            "spatial_timeout" => Self::SpatialTimeout,
            _ => return None,
        })
    }

    /// Technical: Returns the display name associated with the value.
    ///
    /// Plain English: Gets the name shown for this value.
    fn name(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Builtin => "builtin",
            Self::Mode => "mode",
            Self::Activation => "activation",
            Self::GroundOnly => "ground_only",
            Self::WhileHeld => "while_held",
            Self::Reaction => "reaction",
            Self::Actions => "actions",
            Self::Default => "default",
            Self::StartDelay => "start_delay",
            Self::Timeout => "timeout",
            Self::SpatialTimeout => "spatial_timeout",
        }
    }
}

/// Technical: Constructs the entry.
///
/// Plain English: Holds the new entry value.
fn new_entry(fighter_kind: i32, fighter: String, slot: PresetSlot, choice: String) -> Entry {
    Entry {
        fighter_kind,
        fighter,
        slot,
        choice,
        name: None,
        default: false,
        explicit_start: false,
        sequence: Sequence::default(),
    }
}

/// Line-by-line catalog reader. `current` is the recipe being filled in; it is
/// validated and moved into `entries` when the next header (or EOF) arrives.
/// Plain English: Line-by-line catalog reader. current is the recipe being filled in; it is validated and moved into entries when the next header (or EOF) arrives.
struct Parser {
    /// Technical: Stores the `entries` value in `Parser` as `Vec<Entry>`.
    ///
    /// Plain English: Holds the entries value used by Parser.
    entries: Vec<Entry>,
    /// Technical: Stores the `current` value in `Parser` as `Option<(Entry`.
    ///
    /// Plain English: Optionally stores the current for Parser; it may be absent.
    current: Option<(Entry, usize)>,
    /// `RecipeKey` bits already used by `current`.
    /// Plain English: RecipeKey bits already used by current.
    seen: u16,
    /// Technical: Stores the `body` value in `Parser` as `Body`.
    ///
    /// Plain English: Holds the body value used by Parser.
    body: Body,
    /// Technical: Stores the `universal` value in `Parser` as `Option<String>`.
    ///
    /// Plain English: Optionally stores the universal for Parser; it may be absent.
    universal: Option<String>,
    /// Technical: Stores the `universal_keys` value in `Parser` as `Vec<String>`.
    ///
    /// Plain English: Holds the universal keys value used by Parser.
    universal_keys: Vec<String>,
    /// Technical: Stores the `tuning` value in `Parser` as `BTreeMap<i32`.
    ///
    /// Plain English: Holds the tuning value used by Parser.
    tuning: BTreeMap<i32, Tuning>,
    /// Built on first use: most fighters resolve through `fighter_data`.
    /// Plain English: Built on first use: most fighters resolve through fighterdata.
    identities: Option<FighterProfiles>,
}

impl Parser {
    /// Technical: Constructs a new value with the type’s initial state.
    ///
    /// Plain English: Creates a new value.
    fn new(sections: usize) -> Result<Self, String> {
        // Entry embeds all 32 actions. Geometric Vec growth temporarily kept
        // both the old and new roster buffers alive during saves. Count headers
        // first, then reserve once and report allocation failure instead of
        // invoking the aborting OOM handler for the largest save allocation.
        let mut entries = Vec::new();
        entries.try_reserve_exact(sections).map_err(|_| {
            "Not enough memory to validate sequences; retry outside battle".to_owned()
        })?;
        Ok(Self {
            entries,
            current: None,
            seen: 0,
            body: Body::Recipe,
            universal: None,
            universal_keys: Vec::new(),
            tuning: BTreeMap::new(),
            identities: None,
        })
    }

    /// Technical: Begins the entry.
    ///
    /// Plain English: Begins the entry.
    fn begin_entry(&mut self, entry: Entry, line_number: usize) {
        self.seen = 0;
        self.current = Some((entry, line_number));
    }

    /// Technical: Returns the `open_section` result.
    ///
    /// Plain English: Gets the open section result.
    fn open_section(
        &mut self,
        header: SectionHeader<'_>,
        line: &str,
        line_number: usize,
    ) -> Result<(), String> {
        finish_entry(&mut self.current, &mut self.entries)?;
        self.body = match header {
            SectionHeader::UniversalSettings => {
                if self.universal.is_some() {
                    return Err(format!("line {line_number}: duplicate universal settings"));
                }
                self.universal = Some(String::new());
                Body::Universal
            }
            SectionHeader::Template { id, .. } => {
                self.open_template(id, line_number)?;
                Body::Recipe
            }
            SectionHeader::RetiredButtons => Body::Retired,
            SectionHeader::Settings | SectionHeader::Entry => {
                self.open_fighter_section(line, line_number)?
            }
        };
        Ok(())
    }

    /// Technical: Returns the `open_template` result.
    ///
    /// Plain English: Gets the open template result.
    fn open_template(&mut self, id: &str, line_number: usize) -> Result<(), String> {
        if !valid_id(id)
            || self
                .entries
                .iter()
                .any(|e| e.fighter_kind == -1 && e.choice == id)
        {
            return Err(format!(
                "line {line_number}: invalid or duplicate template id {id}"
            ));
        }
        let entry = new_entry(-1, "template".into(), PresetSlot::X, id.into());
        self.begin_entry(entry, line_number);
        Ok(())
    }

    /// `[fighter.situation.choice_id]` or `[fighter.settings]`.
    /// Plain English: [fighter.situation.choiceid] or [fighter.settings].
    fn open_fighter_section(&mut self, line: &str, line_number: usize) -> Result<Body, String> {
        let header = &line[1..line.len() - 1];
        let mut parts = header.split('.');
        let fighter = words_id(parts.next().unwrap_or(""));
        let slot = words_id(parts.next().unwrap_or(""));
        let choice = words_id(parts.next().unwrap_or(""));
        if fighter.is_empty()
            || slot.is_empty()
            || (choice.is_empty() && slot != "settings")
            || parts.next().is_some()
        {
            return Err(format!(
                "line {line_number}: section must be [fighter.situation.choice_id]"
            ));
        }
        let fighter_kind = self
            .fighter_kind(&fighter)
            .ok_or_else(|| format!("line {line_number}: unknown fighter {fighter}"))?;
        if slot == "settings" && choice.is_empty() {
            if self.tuning.insert(fighter_kind, Tuning::default()).is_some() {
                return Err(format!("line {line_number}: duplicate fighter settings"));
            }
            return Ok(Body::Tuning(fighter_kind));
        }
        let slot = parse_slot(&slot)
            .ok_or_else(|| format!("line {line_number}: unknown situation {slot}"))?;
        if !valid_id(&choice) {
            return Err(format!("line {line_number}: invalid choice id {choice}"));
        }
        if self.entries.iter().any(|entry| {
            entry.fighter_kind == fighter_kind && entry.slot == slot && entry.choice == choice
        }) {
            return Err(format!("line {line_number}: duplicate section {header}"));
        }
        self.begin_entry(new_entry(fighter_kind, fighter, slot, choice), line_number);
        Ok(Body::Recipe)
    }

    /// Technical: Returns an optional `fighter_kind` value.
    ///
    /// Plain English: Gets fighter kind if it is available.
    fn fighter_kind(&mut self, fighter: &str) -> Option<i32> {
        crate::fighter_data::all()
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(fighter))
            .map(|entry| entry.kind)
            .or_else(|| {
                let identities = self.identities.get_or_insert_with(FighterProfiles::default);
                (0..crate::fighter_kinds::PLAYABLE_KIND_COUNT).find(|kind| {
                    identities
                        .profile(*kind)
                        .is_some_and(|p| p.fighter_kind == *kind && p.name == fighter)
                })
            })
    }

    /// Technical: Returns the `property` result.
    ///
    /// Plain English: Gets the property result.
    fn property(&mut self, line: &str, line_number: usize) -> Result<(), String> {
        if matches!(self.body, Body::Retired) {
            return Ok(());
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(format!("line {line_number}: expected key = value"));
        };
        match self.body {
            Body::Retired => Ok(()),
            Body::Universal => self.universal_property(key, value, line_number),
            Body::Tuning(kind) => self
                .tuning
                .get_mut(&kind)
                .expect("settings section registered by its header")
                .set(key.trim(), value.trim())
                .map_err(|e| format!("line {line_number}: {e}")),
            Body::Recipe => self.recipe_property(key, value.trim(), line_number),
        }
    }

    /// Technical: Returns the `universal_property` result.
    ///
    /// Plain English: Gets the universal property result.
    fn universal_property(
        &mut self,
        key: &str,
        value: &str,
        line_number: usize,
    ) -> Result<(), String> {
        let key = key.trim().to_ascii_lowercase();
        if self.universal_keys.contains(&key) {
            return Err(format!("line {line_number}: duplicate universal key {key}"));
        }
        self.universal
            .as_mut()
            .expect("universal body follows its header")
            .push_str(&format!("{key} = {}\n", value.trim()));
        self.universal_keys.push(key);
        Ok(())
    }

    /// Technical: Returns the `recipe_property` result.
    ///
    /// Plain English: Gets the recipe property result.
    fn recipe_property(
        &mut self,
        key: &str,
        value: &str,
        line_number: usize,
    ) -> Result<(), String> {
        let Some((entry, _)) = self.current.as_mut() else {
            return Err(format!(
                "line {line_number}: name/actions must follow a section"
            ));
        };
        let Some(key) = RecipeKey::parse(&key.trim().to_ascii_lowercase()) else {
            return Err(format!(
                "line {line_number}: supported keys are name, actions, builtin, default, start_delay, timeout, spatial_timeout, while_held, ground_only and reaction"
            ));
        };
        let bit = 1 << key as u16;
        if self.seen & bit != 0 {
            return Err(format!("line {line_number}: duplicate {}", key.name()));
        }
        self.seen |= bit;
        apply_recipe_key(entry, key, value, line_number)
    }

    /// Technical: Completes the active operation and records its final state.
    ///
    /// Plain English: Finishes the current operation.
    fn finish(mut self) -> Result<Catalog, String> {
        finish_entry(&mut self.current, &mut self.entries)?;
        self.identities = None;
        if let Some(settings) = self.universal.as_deref() {
            validate_universal_settings(settings)?;
        }
        let mut templates = Vec::new();
        self.entries.retain_mut(|entry| {
            if entry.fighter_kind != -1 {
                return true;
            }
            let id = std::mem::take(&mut entry.choice);
            let name = entry
                .name
                .take()
                .unwrap_or_else(|| id.replace('_', " "));
            templates.push(Template {
                id,
                name,
                sequence: entry.sequence,
            });
            false
        });
        if templates.len() > MAX_TEMPLATES {
            return Err(format!(
                "at most {MAX_TEMPLATES} generic templates are allowed"
            ));
        }
        Ok(Catalog {
            entries: self.entries,
            tuning: self.tuning,
            universal: self.universal,
            templates,
        })
    }
}

/// Technical: Applies the recipe key.
///
/// Plain English: Applies the recipe key.
fn apply_recipe_key(
    entry: &mut Entry,
    key: RecipeKey,
    value: &str,
    line_number: usize,
) -> Result<(), String> {
    let sequence = &mut entry.sequence;
    match key {
        RecipeKey::Name => entry.name = Some(parse_name(value, line_number)?),
        RecipeKey::Builtin => {
            sequence.builtin = Some(
                Builtin::parse(value)
                    .ok_or_else(|| format!("line {line_number}: unknown built-in button action"))?,
            );
        }
        RecipeKey::Mode => {
            let mode = Mode::parse(&words_id(value)).ok_or_else(|| {
                format!("line {line_number}: unknown/unimplemented mode {value}")
            })?;
            sequence.mode = Some(ModeBinding {
                mode,
                activation: Activation::Hold,
            });
        }
        RecipeKey::Activation => {
            let Some(binding) = sequence.mode.as_mut() else {
                return Err(format!("line {line_number}: activation must follow mode"));
            };
            binding.activation = match value {
                "hold" => Activation::Hold,
                "toggle" => Activation::Toggle,
                _ => {
                    return Err(format!(
                        "line {line_number}: activation must be hold or toggle"
                    ))
                }
            };
        }
        RecipeKey::GroundOnly => sequence.ground_only = parse_flag(key, value, line_number)?,
        RecipeKey::WhileHeld => sequence.while_held = parse_flag(key, value, line_number)?,
        RecipeKey::Default => entry.default = parse_flag(key, value, line_number)?,
        RecipeKey::Reaction => {
            sequence.reaction =
                Rule::parse(value).map_err(|e| format!("line {line_number}: {e}"))?;
        }
        RecipeKey::Actions => {
            let parsed = parse_actions(value, line_number)?;
            sequence.actions = parsed.actions;
            sequence.len = parsed.len;
        }
        RecipeKey::StartDelay | RecipeKey::Timeout | RecipeKey::SpatialTimeout => {
            let frames = parse_frame_count(key, value, line_number)?;
            match key {
                RecipeKey::StartDelay => {
                    sequence.start_delay = frames;
                    entry.explicit_start = true;
                }
                RecipeKey::Timeout => sequence.timeout = frames,
                _ => sequence.spatial_timeout = frames,
            }
        }
    }
    Ok(())
}

/// Technical: Parses the name.
///
/// Plain English: Parses the name.
fn parse_name(value: &str, line_number: usize) -> Result<String, String> {
    let value = if value.starts_with('"') || value.ends_with('"') {
        value
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .ok_or_else(|| format!("line {line_number}: name needs matching quotes"))?
    } else {
        value
    };
    if value.trim().is_empty() || value.len() > 64 || value.chars().any(char::is_control) {
        return Err(format!(
            "line {line_number}: name must be 1–64 bytes without control characters"
        ));
    }
    Ok(value.trim().to_owned())
}

/// Technical: Parses the flag.
///
/// Plain English: Parses the flag.
fn parse_flag(key: RecipeKey, value: &str, line_number: usize) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!(
            "line {line_number}: {} must be true or false",
            key.name()
        )),
    }
}

/// `start_delay` may be zero; the timeouts must be at least one frame.
/// Plain English: startdelay may be zero; the timeouts must be at least one frame.
fn parse_frame_count(key: RecipeKey, value: &str, line_number: usize) -> Result<u16, String> {
    let name = key.name();
    let frames: u16 = value
        .parse()
        .map_err(|_| format!("line {line_number}: {name} must be an integer frame count"))?;
    let minimum = u16::from(!matches!(key, RecipeKey::StartDelay));
    if frames > MAX_SEQUENCE_FRAME_COUNT || frames < minimum {
        return Err(format!(
            "line {line_number}: {name} must be {minimum}..{MAX_SEQUENCE_FRAME_COUNT}"
        ));
    }
    Ok(frames)
}

impl Catalog {
    /// Technical: Parses the supplied source into the module’s validated representation.
    ///
    /// Plain English: Reads the source and checks that it is valid.
    pub fn parse(source: &str) -> Result<Self, String> {
        if source.len() as u64 > MAX_FILE_BYTES {
            return Err("sequence file exceeds 512 KiB".into());
        }
        let mut parser = Parser::new(count_sections(source)?)?;
        for (index, raw) in source.lines().enumerate() {
            let line = strip_comment(raw);
            if line.is_empty() {
                continue;
            }
            match classify_section_header(line) {
                Some(header) => parser.open_section(header, line, index + 1)?,
                None => parser.property(line, index + 1)?,
            }
        }
        parser.finish()
    }

    /// Technical: Returns the `validate` result.
    ///
    /// Plain English: Gets the validate result.
    pub fn validate(&self, profiles: &FighterProfiles) -> Result<(), String> {
        // Count each row once. The old nested full-catalog scan allocated a
        // temporary row Vec for every section, repeatedly during a single save.
        let mut rows = BTreeMap::<(i32, usize), (usize, usize)>::new();
        let mut simple_events = std::collections::BTreeSet::new();
        let mut rule_counts = BTreeMap::<(i32, usize), usize>::new();
        for entry in &self.entries {
            validate_facing_blocks(entry.sequence.actions())?;
            crate::reaction_rules::validate_response(entry.sequence)?;
            let reaction = entry.sequence.reaction;
            if let Some(event) = reaction.trigger.simple() {
                if !simple_events.insert((entry.fighter_kind, event)) {
                    return Err(
                        "Only one auto-react sequence per event per fighter is allowed".into(),
                    );
                }
            }

            if reaction.active() {
                let count = rule_counts
                    .entry((entry.fighter_kind, reaction.trigger.family()))
                    .or_default();
                *count += 1;
                if *count > crate::reaction_rules::MAX_RULES_PER_MODE {
                    return Err(format!(
                        "{} exceeds 8 rules in one assist mode",
                        entry.fighter
                    ));
                }
            }
            let row = rows
                .entry((entry.fighter_kind, entry.slot as usize))
                .or_default();
            if row.0 == 0 {
                profiles
                    .profile(entry.fighter_kind)
                    .filter(|profile| profile.fighter_kind == entry.fighter_kind)
                    .ok_or_else(|| format!("{} has no compiled ledge profile", entry.fighter))?;
            }
            row.0 += usize::from(!reaction.active());
            row.1 += usize::from(entry.default && !reaction.active());
            if row.0 > entry.slot.max_choices() {
                return Err(format!(
                    "{}.{} exceeds {} menu options",
                    entry.fighter,
                    slot_name(entry.slot),
                    entry.slot.max_choices()
                ));
            }
            if row.1 > 1 {
                return Err(format!(
                    "{}.{} has multiple defaults",
                    entry.fighter,
                    slot_name(entry.slot)
                ));
            }
        }
        Ok(())
    }

    /// Technical: Returns an optional `find` value.
    ///
    /// Plain English: Gets find if it is available.
    pub fn find(&self, fighter_kind: i32, slot: PresetSlot, choice: &str) -> Option<Sequence> {
        self.entries
            .iter()
            .find(|entry| {
                entry.fighter_kind == fighter_kind && entry.slot == slot && entry.choice == choice
            })
            .map(|entry| entry.sequence)
    }

    /// Technical: Returns the number of stored elements.
    ///
    /// Plain English: Counts the stored items.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Technical: Checks whether empty.
    ///
    /// Plain English: Checks whether empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Technical: Returns the `entries` result.
    ///
    /// Plain English: Gets the entries result.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
    /// Startup-only filtering, before menus are frozen and recipes can execute.
    /// Plain English: Startup-only filtering, before menus are frozen and recipes can execute.
    pub fn for_build(mut self, diagnostic: bool) -> Self {
        if !diagnostic {
            self.entries.retain(|entry| {
                !entry.choice.starts_with("test")
                    && !entry.name.as_deref().is_some_and(|name| {
                        name.as_bytes()
                            .windows(b"[test]".len())
                            .any(|window| window.eq_ignore_ascii_case(b"[test]"))
                    })
            });
        }
        self
    }
}

// Return only the validation result here. Expanded button settings must not
// reserve config-sized Result/map temporaries in every catalog parser frame,
// including small files that do not contain universal settings at all.
/// Technical: Validates the universal settings.
///
/// Plain English: Validates the universal settings.
#[inline(never)]
fn validate_universal_settings(settings: &str) -> Result<(), String> {
    crate::live_input::LiveConfig::parse(settings)
        .map(|_| ())
        .map_err(|e| format!("universal.settings: {e}"))
}

// Policy sections contain no 32-action Entry; do not reserve recipe storage for them.
/// Technical: Determines whether the `settings_header` condition holds.
///
/// Plain English: Checks whether settings header.
fn settings_header(line: &str) -> bool {
    let Some(header) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) else {
        return false;
    };
    let mut parts = header.split('.');
    parts.next().is_some()
        && parts
            .next()
            .is_some_and(|s| s.trim().eq_ignore_ascii_case("settings"))
        && parts.next().is_none()
}

/// Old universal bindings remain inert; never migrate them into fighter lists.
/// Plain English: Old universal bindings remain inert; never migrate them into fighter lists.
fn retired_button_header(line: &str) -> bool {
    let Some(header) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) else {
        return false;
    };
    let mut parts = header.split('.');
    let fighter = parts.next().unwrap_or("").trim();
    let slot = parts.next().unwrap_or("").trim();
    // Compatibility only: retired trigger sections never enter the catalog,
    // menu, ownership masks or playback. Keep installed saved files loadable.
    if ["l", "r", "zl", "zr", "lz", "rz"]
        .iter()
        .any(|s| slot.eq_ignore_ascii_case(s))
    {
        return true;
    }
    fighter.eq_ignore_ascii_case("universal")
        && [
            "x", "y", "button_x", "button_y", "button-x", "button-y", "button x", "button y",
        ]
        .iter()
        .any(|s| slot.eq_ignore_ascii_case(s))
}

/// Cross-key rules that can only be judged once the whole section is read.
/// Plain English: Cross-key rules that can only be judged once the whole section is read.
fn check_entry(entry: &Entry, line: usize) -> Result<(), String> {
    let sequence = &entry.sequence;
    if entry.fighter_kind == -1
        && (entry.default
            || sequence.mode.is_some()
            || sequence.builtin.is_some()
            || sequence.reaction.active()
            || sequence.while_held)
    {
        return Err(format!("line {line}: templates contain actions/timings only, without defaults, modes, builtins, reactions or held activation"));
    }
    if let Some(binding) = sequence.mode {
        if !entry.slot.is_button() || !binding.mode.supports(entry.fighter_kind) {
            return Err(format!(
                "line {line}: mode is unavailable for this fighter/button row"
            ));
        }
        if sequence.len != 0 || sequence.start_delay != 0 {
            return Err(format!(
                "line {line}: mode bindings cannot contain actions or start delays"
            ));
        }
    }
    if sequence.builtin.is_some()
        && (!entry.slot.is_button()
            || sequence.mode.is_some()
            || sequence.len != 0
            || sequence.start_delay != 0
            || sequence.reaction.active()
            || sequence.while_held
            || sequence.ground_only)
    {
        return Err(format!("line {line}: builtin requires a button row and cannot contain actions, conditions, delays, loops or modes"));
    }
    let plain_recipe = sequence.mode.is_none() && sequence.builtin.is_none();
    if !entry.explicit_start && plain_recipe {
        return Err(format!(
            "line {line}: section requires start_delay = FRAMES (use 0 for immediate input)"
        ));
    }
    if sequence.len == 0 && plain_recipe {
        return Err(format!("line {line}: section is missing actions"));
    }
    Ok(())
}

/// Technical: Finishes the entry.
///
/// Plain English: Finishes the entry.
fn finish_entry(
    current: &mut Option<(Entry, usize)>,
    entries: &mut Vec<Entry>,
) -> Result<(), String> {
    if let Some((entry, line)) = current.take() {
        check_entry(&entry, line)?;
        if entries.len() >= MAX_ENTRIES {
            return Err(format!(
                "at most {MAX_ENTRIES} sequence sections are allowed"
            ));
        }
        entries.push(entry);
    }
    Ok(())
}

/// Technical: Returns the `words_id` result.
///
/// Plain English: Gets the words id result.
pub(super) fn words_id(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
        .replace('-', "_")
}

/// Technical: Parses the slot.
///
/// Plain English: Parses the slot.
pub(super) fn parse_slot(value: &str) -> Option<PresetSlot> {
    Some(match value.to_ascii_lowercase().as_str() {
        "neutral" | "neutral_getup" => PresetSlot::Neutral,
        "jump" | "jump_getup" => PresetSlot::Jump,
        "roll" | "roll_getup" => PresetSlot::Roll,
        "attack" | "getup_attack" => PresetSlot::Attack,
        "drop" | "ledge_drop" => PresetSlot::Drop,
        "x" | "button_x" | "button-x" => PresetSlot::X,
        "y" | "button_y" | "button-y" => PresetSlot::Y,
        "dpad_up" => PresetSlot::DpadUp,
        "dpad_down" => PresetSlot::DpadDown,
        "dpad_left" => PresetSlot::DpadLeft,
        "dpad_right" => PresetSlot::DpadRight,
        "l3" => PresetSlot::L3,
        "r3" => PresetSlot::R3,
        _ => return None,
    })
}

/// Technical: Returns the `slot_name` result.
///
/// Plain English: Gets the slot name result.
pub(super) fn slot_name(slot: PresetSlot) -> &'static str {
    match slot {
        PresetSlot::Neutral => "neutral",
        PresetSlot::Jump => "jump",
        PresetSlot::Roll => "roll",
        PresetSlot::Attack => "attack",
        PresetSlot::Drop => "drop",
        PresetSlot::X => "x",
        PresetSlot::Y => "y",
        PresetSlot::DpadUp => "dpad_up",
        PresetSlot::DpadDown => "dpad_down",
        PresetSlot::DpadLeft => "dpad_left",
        PresetSlot::DpadRight => "dpad_right",
        PresetSlot::L3 => "l3",
        PresetSlot::R3 => "r3",
    }
}

/// Technical: Determines whether the `valid_id` condition holds.
///
/// Plain English: Checks whether valid id.
pub(super) fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 48
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
