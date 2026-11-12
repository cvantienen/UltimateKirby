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

/// Available `SectionHeader` choices for the Smash assistance menu.
#[derive(Clone, Copy)]
enum SectionHeader<'a> {
/// `UniversalSettings` option in the `SectionHeader` Smash policy state.
    UniversalSettings,
/// `Template` option in the `SectionHeader` Smash policy state.
    Template { id: &'a str, countable: bool },
/// `RetiredButtons` option in the `SectionHeader` Smash policy state.
    RetiredButtons,
/// `Settings` option in the `SectionHeader` Smash policy state.
    Settings,
/// `Entry` option in the `SectionHeader` Smash policy state.
    Entry,
}

/// Classifies the section header.
/// Classifies the section header.
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
pub(super) fn strip_comment(line: &str) -> &str {
    line.split('#').next().unwrap_or("").trim()
}

/// Section headers that will hold a 32-action recipe.
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
#[derive(Clone, Copy)]
enum Body {
    /// Recipe keys for `Parser::current`; before any header there is none.
    Recipe,
/// `Retired` option in the `Body` Smash policy state.
    Retired,
/// `Universal` option in the `Body` Smash policy state.
    Universal,
/// `Tuning` option in the `Body` Smash policy state.
    Tuning(
        /// Carries the numeric tuning value in `Body`.
        i32,
    ),
}

/// Keys of a recipe section. Each may appear once per section.
#[derive(Clone, Copy)]
enum RecipeKey {
/// `Name` option in the `RecipeKey` Smash policy state.
    Name,
/// `Builtin` option in the `RecipeKey` Smash policy state.
    Builtin,
/// `Mode` option in the `RecipeKey` Smash policy state.
    Mode,
/// `Activation` option in the `RecipeKey` Smash policy state.
    Activation,
/// `GroundOnly` option in the `RecipeKey` Smash policy state.
    GroundOnly,
/// `WhileHeld` option in the `RecipeKey` Smash policy state.
    WhileHeld,
/// `Reaction` option in the `RecipeKey` Smash policy state.
    Reaction,
/// `Actions` option in the `RecipeKey` Smash policy state.
    Actions,
/// `Default` option in the `RecipeKey` Smash policy state.
    Default,
/// `StartDelay` option in the `RecipeKey` Smash policy state.
    StartDelay,
/// `Timeout` option in the `RecipeKey` Smash policy state.
    Timeout,
/// `SpatialTimeout` option in the `RecipeKey` Smash policy state.
    SpatialTimeout,
}

impl RecipeKey {
    /// Parses the supplied source into the module’s validated representation.
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

    /// Returns the display name associated with the value.
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

/// Constructs the entry.
/// Holds the new entry value.
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
struct Parser {
/// Runtime entries value used by Parser..
    entries: Vec<Entry>,
/// Runtime current value used by Parser..
    current: Option<(Entry, usize)>,
    /// `RecipeKey` bits already used by `current`.
    seen: u16,
/// Runtime body value used by Parser..
    body: Body,
/// Runtime universal value used by Parser..
    universal: Option<String>,
/// Runtime universal keys value used by Parser..
    universal_keys: Vec<String>,
/// Runtime tuning value used by Parser..
    tuning: BTreeMap<i32, Tuning>,
    /// Built on first use: most fighters resolve through `fighter_data`.
    identities: Option<FighterProfiles>,
}

impl Parser {
    /// Constructs a new value with the type’s initial state.
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

    /// Begins the entry.
    fn begin_entry(&mut self, entry: Entry, line_number: usize) {
        self.seen = 0;
        self.current = Some((entry, line_number));
    }

    /// Returns the `open_section` result.
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

    /// Returns the `open_template` result.
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

    /// Returns an optional `fighter_kind` value.
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

    /// Returns the `property` result.
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

    /// Returns the `universal_property` result.
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

    /// Returns the `recipe_property` result.
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

    /// Completes the active operation and records its final state.
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

/// Applies the recipe key.
/// Applies the recipe key.
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

/// Parses the name.
/// Parses the name.
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

/// Parses the flag.
/// Parses the flag.
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
    /// Parses the supplied source into the module’s validated representation.
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

    /// Returns the `validate` result.
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

    /// Returns an optional `find` value.
    pub fn find(&self, fighter_kind: i32, slot: PresetSlot, choice: &str) -> Option<Sequence> {
        self.entries
            .iter()
            .find(|entry| {
                entry.fighter_kind == fighter_kind && entry.slot == slot && entry.choice == choice
            })
            .map(|entry| entry.sequence)
    }

    /// Returns the number of stored elements.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Checks whether empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the `entries` result.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
    /// Startup-only filtering, before menus are frozen and recipes can execute.
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
/// Validates the universal settings.
/// Validates the universal settings.
#[inline(never)]
fn validate_universal_settings(settings: &str) -> Result<(), String> {
    crate::live_input::LiveConfig::parse(settings)
        .map(|_| ())
        .map_err(|e| format!("universal.settings: {e}"))
}

// Policy sections contain no 32-action Entry; do not reserve recipe storage for them.
/// Determines whether the `settings_header` condition holds.
/// Checks whether settings header.
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

/// Finishes the entry.
/// Finishes the entry.
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

/// Returns the `words_id` result.
/// Gets the words id result.
pub(super) fn words_id(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
        .replace('-', "_")
}

/// Parses the slot.
/// Parses the slot.
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

/// Returns the `slot_name` result.
/// Gets the slot name result.
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

/// Determines whether the `valid_id` condition holds.
/// Checks whether valid id.
pub(super) fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 48
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
