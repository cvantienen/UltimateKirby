//! Parse and validate the startup recipe catalog.
//!
//! This code runs during startup or an explicit save, never in the frame loop.
use super::*;
use crate::{
    button_bindings::{Activation, Builtin, Mode, ModeBinding},
    fighter_tuning::Tuning,
    native_input::{MAPPER_A as A, MAPPER_B as B, MAPPER_R as R, MAPPER_X as X, MAPPER_ZR as ZR},
    reaction_rules::Rule,
};
use std::collections::BTreeMap;

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
    /// `Auto tilt directions` option in the `RecipeKey` Smash policy state.
    AutoTilt,
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
            "auto_tilt" | "auto_tilts" | "tilt_turn" => Self::AutoTilt,
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
            Self::AutoTilt => "auto_tilt",
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
            if self
                .tuning
                .insert(fighter_kind, Tuning::default())
                .is_some()
            {
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
                "line {line_number}: supported keys are name, actions, builtin, default, start_delay, timeout, spatial_timeout, while_held, ground_only, auto_tilt and reaction"
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
            let name = entry.name.take().unwrap_or_else(|| id.replace('_', " "));
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
            sequence.builtin =
                Some(Builtin::parse(value).ok_or_else(|| {
                    format!("line {line_number}: unknown built-in button action")
                })?);
        }
        RecipeKey::Mode => {
            let mode = Mode::parse(&words_id(value))
                .ok_or_else(|| format!("line {line_number}: unknown/unimplemented mode {value}"))?;
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
        RecipeKey::AutoTilt => {
            if !matches!(
                entry.fighter_kind,
                crate::fighter_kinds::KIRBY
                    | crate::fighter_kinds::YOSHI
                    | crate::fighter_kinds::LUCINA
                    | crate::fighter_kinds::LUIGI
                    | -1
            ) {
                return Err(format!(
                    "line {line_number}: auto_tilt compatibility key is supported only for Kirby, Yoshi, Lucina, and Luigi"
                ));
            }
            sequence.auto_tilt = crate::posture::TiltSettings::parse(value)
                .map_err(|e| format!("line {line_number}: {e}"))?;
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
    /// Parses one complete UTF-8 catalog, rejecting malformed sections/actions.
    ///
    /// Comments begin at the first `#`, including inside quoted names. Recipe
    /// names may be quoted, but action lists are single comma-separated lines.
    /// This allocates and belongs at startup or explicit save, not execution.
    /// Call `validate` as well to enforce cross-entry menu and event uniqueness.
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

    /// Checks parsed entries against fighter profiles and shared catalog limits.
    ///
    /// This rejects duplicate simple automatic events, excess rules/options,
    /// multiple manual startup defaults and invalid reaction/facing responses.
    /// It does not prove native move acceptance, timing, reach or a successful hit.
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
            let (choice_count, default_count) = rows
                .entry((entry.fighter_kind, entry.slot as usize))
                .or_default();
            if *choice_count == 0 {
                profiles
                    .profile(entry.fighter_kind)
                    .filter(|profile| profile.fighter_kind == entry.fighter_kind)
                    .ok_or_else(|| format!("{} has no compiled ledge profile", entry.fighter))?;
            }
            *choice_count += usize::from(!reaction.active());
            *default_count += usize::from(entry.default && !reaction.active());
            if *choice_count > entry.slot.max_choices() {
                return Err(format!(
                    "{}.{} exceeds {} menu options",
                    entry.fighter,
                    slot_name(entry.slot),
                    entry.slot.max_choices()
                ));
            }
            if *default_count > 1 {
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
        if sequence.len != 0
            || sequence.start_delay != 0
            || sequence.reaction.active()
            || sequence.while_held
            || sequence.ground_only
            || sequence.auto_tilt.is_some()
        {
            return Err(format!(
                "line {line}: mode bindings cannot contain actions, delays or recipe conditions"
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
            || sequence.ground_only
            || sequence.auto_tilt.is_some())
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

// --- Action-list token parsing into the fixed-size recipe representation ---

/// Appends one parsed action, rejecting a sequence that exceeds its fixed capacity.
fn push(sequence: &mut Sequence, action: Action) -> Result<(), String> {
    if usize::from(sequence.len) == MAX_ACTIONS {
        return Err(format!(
            "at most {MAX_ACTIONS} expanded actions are allowed"
        ));
    }
    sequence.actions[usize::from(sequence.len)] = action;
    sequence.len += 1;
    Ok(())
}

/// Parses and validates one comma-separated action list.
pub(super) fn parse_actions(value: &str, line: usize) -> Result<Sequence, String> {
    let mut result = Sequence::default();
    for token in value.split(',').map(str::trim) {
        if token.is_empty() {
            return Err(format!("line {line}: empty action"));
        }
        parse_action(token, &mut result).map_err(|e| format!("line {line}: {e}"))?;
    }
    validate_facing_blocks(result.actions()).map_err(|e| format!("line {line}: {e}"))?;
    Ok(result)
}

/// Flat facing blocks are deliberately bounded and cannot nest.
pub fn validate_facing_blocks(actions: &[Action]) -> Result<(), String> {
    let mut open = false;
    let mut otherwise = false;
    for action in actions {
        match action {
            Action::IfFacingLedge { .. } | Action::IfFacingOpponent { .. } => {
                if open {
                    return Err("Facing blocks cannot nest; add end_facing first".into());
                }
                open = true;
                otherwise = false;
            }
            Action::ElseFacing => {
                if !open || otherwise {
                    return Err(
                        "else_facing needs an open facing block and may appear only once".into(),
                    );
                }
                otherwise = true;
            }
            Action::EndFacing => {
                if !open {
                    return Err("end_facing has no matching facing condition".into());
                }
                open = false;
            }
            _ => {}
        }
    }
    if open {
        return Err("Facing block needs end_facing".into());
    }
    Ok(())
}

/// Builds one input action from validated button, stick, duration, and facing values.
fn input(buttons: u32, stick: [f32; 2], frames: u8, relative: bool) -> Action {
    Action::Input {
        buttons,
        stick,
        frames,
        relative,
    }
}

/// One colon-separated action token after alias normalization. Every accessor
/// reports failures against the author's original text.
struct Args<'a> {
    /// Runtime token value used by Args..
    token: &'a str,
    /// Runtime parts value used by Args..
    parts: Vec<&'a str>,
}

impl Args<'_> {
    /// Returns the number of stored elements.
    fn len(&self) -> usize {
        self.parts.len()
    }

    /// Returns the `invalid` result.
    fn invalid(&self) -> String {
        format!("invalid action {}", self.token)
    }

    /// Returns the `exact` result.
    fn exact(&self, count: usize) -> Result<(), String> {
        if self.len() == count {
            Ok(())
        } else {
            Err(self.invalid())
        }
    }

    /// Returns the `at_most` result.
    fn at_most(&self, count: usize) -> Result<(), String> {
        if self.len() <= count {
            Ok(())
        } else {
            Err(self.invalid())
        }
    }

    /// Returns the floating-point value when the fact contains one.
    fn float(&self, index: usize, min: f32, max: f32) -> Result<f32, String> {
        parse_float(self.parts[index], min, max, self.token)
    }

    /// Returns the `float_or` result.
    fn float_or(&self, index: usize, default: f32, min: f32, max: f32) -> Result<f32, String> {
        if index < self.len() {
            self.float(index, min, max)
        } else {
            Ok(default)
        }
    }

    /// Left- or right-stick pair at `index` and `index + 1`.
    fn stick(&self, index: usize) -> Result<[f32; 2], String> {
        Ok([
            self.float(index, -1.0, 1.0)?,
            self.float(index + 1, -1.0, 1.0)?,
        ])
    }

    /// Returns the `frames_or` result.
    fn frames_or(&self, index: usize, default: u8) -> Result<u8, String> {
        self.parts
            .get(index)
            .map_or(Ok(default), |value| parse_u8(value, self.token))
    }

    /// Input hold length: 1..255 native frames.
    fn duration(&self, index: usize, default: u8) -> Result<u8, String> {
        match self.frames_or(index, default)? {
            0 => Err(format!("{}: input duration must be 1..255", self.token)),
            frames => Ok(frames),
        }
    }

    /// Returns the `buttons` result.
    fn buttons(&self, index: usize) -> Result<u32, String> {
        buttons(self.parts[index])
    }

    /// Returns the `direction` result.
    fn direction(&self, index: usize) -> Result<Direction, String> {
        Direction::from_token(self.parts[index])
            .ok_or_else(|| format!("invalid direction in {}", self.token))
    }

    /// Returns the `reference` result.
    fn reference(&self, index: usize) -> Result<InputReference, String> {
        InputReference::parse(self.parts.get(index).copied().unwrap_or("screen"))
    }
}

/// Canonical single-letter names plus the long aliases accepted in `+` lists.
fn buttons(value: &str) -> Result<u32, String> {
    let mut mask = 0;
    for button in value.split('+') {
        mask |= match button_alias(button) {
            Some(bit) => bit,
            None => BUTTON_TOKENS
                .iter()
                .find(|(_, token)| *token == button)
                .map(|&(bit, _)| bit)
                .ok_or_else(|| format!("unknown button {button}"))?,
        };
    }
    Ok(mask)
}

/// Rewrite the spelled-out shortcuts that are not simple aliases of one token
/// into the canonical `name:arg` layout used by the dispatcher below.
fn normalize(token: &str) -> String {
    let normalized = token
        .split(':')
        .map(|v| {
            if v.trim().starts_with('-') {
                v.trim().to_ascii_lowercase()
            } else {
                words_id(v)
            }
        })
        .collect::<Vec<_>>()
        .join(":");
    if let Some(wait) = normalized.strip_prefix("wait_") {
        format!(
            "wait:{}",
            wait.strip_suffix("_frames")
                .or_else(|| wait.strip_suffix("_frame"))
                .unwrap_or(wait)
        )
    } else if let Some(direction) = normalized.strip_prefix("face_") {
        format!("face:{direction}")
    } else if normalized == "grab_no_range_check" {
        "grab:ignore".into()
    } else if normalized == "grab_range_verified" {
        "grab:fighter_range".into()
    } else {
        normalized
    }
}

/// Parses the action.
fn parse_action(token: &str, sequence: &mut Sequence) -> Result<(), String> {
    let normalized = normalize(token);
    let args = Args {
        token,
        parts: normalized.split(':').collect(),
    };
    let name = args.parts[0];
    let action = match name {
        "wait" | "release" => wait(&args)?,
        "check" | "require" => check(&args)?,
        "tap" => tap(&args)?,
        "press" | "hold" => press(&args)?,
        "controller_input" => controller_input(&args)?,
        "opponent_input" => opponent_input(&args)?,
        "input" | "facing_input" => {
            args.exact(5)?;
            input(
                args.buttons(1)?,
                args.stick(2)?,
                args.duration(4, 1)?,
                name == "facing_input",
            )
        }
        "cstick" => {
            args.exact(4)?;
            Action::CStick {
                stick: args.stick(1)?,
                frames: args.duration(3, 1)?,
            }
        }
        "stick" => {
            args.exact(4)?;
            input(0, args.stick(1)?, args.duration(3, 1)?, false)
        }
        "hold_until_manual" | "facing_hold_until_manual" => hold_until_manual(&args)?,
        "chase_if_far" => {
            args.exact(4)?;
            Action::Chase {
                gap: args.float(1, 0., 100.)?,
                strength: args.float(2, 0., 1.)?,
                frames: args.duration(3, 14)?,
            }
        }
        "shield_until_manual" => marker(&args, Action::ShieldUntilManual)?,
        "shield_until_block" => {
            args.exact(2)?;
            Action::ShieldUntilBlock(args.duration(1, 90)?)
        }
        "if_facing_ledge" | "if_back_to_ledge" => marker(
            &args,
            Action::IfFacingLedge {
                away: name == "if_back_to_ledge",
            },
        )?,
        "if_facing_opponent" | "if_back_to_opponent" => marker(
            &args,
            Action::IfFacingOpponent {
                away: name == "if_back_to_opponent",
            },
        )?,
        "else_facing" => marker(&args, Action::ElseFacing)?,
        "end_facing" => marker(&args, Action::EndFacing)?,
        "face" => {
            args.exact(2)?;
            Action::Face(args.direction(1)?)
        }
        "walk" => walk(&args)?,
        "run_to_ledge" => run_to_ledge(&args)?,
        "run" => {
            args.exact(3)?;
            Action::Run {
                direction: args.direction(1)?,
                units: args.float(2, 0.0, 100.0)?,
            }
        }
        "walk_to_opponent" | "drift_to_opponent" => approach(&args)?,
        "drift" | "drift_for" => {
            args.exact(4)?;
            Action::Drift {
                direction: args.direction(1)?,
                strength: args.float(2, 0.0, 1.0)?,
                frames: args.duration(3, 1)?,
            }
        }
        "range" => {
            args.exact(2)?;
            Action::Range(range_check(args.parts[1], token)?)
        }
        "grab" => return grab(&args, sequence),
        _ => return named_move(&args, sequence),
    };
    push(sequence, action)
}

/// Zero-argument markers such as `end_facing`.
fn marker(args: &Args, action: Action) -> Result<Action, String> {
    args.exact(1)?;
    Ok(action)
}

/// Returns the `wait` result.
/// Gets the wait result.
fn wait(args: &Args) -> Result<Action, String> {
    args.at_most(2)?;
    Ok(Action::Wait(args.frames_or(1, 1)?))
}

/// Returns the `check` result.
/// Gets the check result.
fn check(args: &Args) -> Result<Action, String> {
    let condition = args
        .parts
        .get(1)
        .and_then(|v| Condition::from_token(v))
        .ok_or_else(|| format!("{}: unknown check condition", args.token))?;
    let wait_index = if condition.has_distance() { 3 } else { 2 };
    if args.len() < wait_index || args.len() > wait_index + 1 {
        return Err(format!("invalid check {}", args.token));
    }
    let value = if condition.has_distance() {
        args.float(2, 0.0, 100.0)?
    } else {
        0.0
    };
    Ok(Action::Check {
        condition,
        value,
        wait: args.frames_or(wait_index, 0)?,
    })
}

/// Returns the `tap` result.
/// Gets the tap result.
fn tap(args: &Args) -> Result<Action, String> {
    args.exact(7)?;
    let frames = parse_frames(args.parts[6])
        .ok_or_else(|| format!("{}: total frames must be 1..3600", args.token))?;
    let buttons = args.buttons(1)?;
    if buttons == 0 {
        return Err("tap requires a button".into());
    }
    Ok(Action::Tap {
        buttons,
        stick: args.stick(2)?,
        hold: args.duration(4, 1)?,
        gap: args.duration(5, 1)?,
        frames,
    })
}

/// Returns the `press` result.
/// Gets the press result.
fn press(args: &Args) -> Result<Action, String> {
    if args.len() != 2 && args.len() != 3 {
        return Err("use press:BUTTONS[:FRAMES] or hold:BUTTONS:FRAMES".into());
    }
    Ok(input(
        args.buttons(1)?,
        [0.0; 2],
        args.duration(2, 1)?,
        false,
    ))
}

/// Returns the `controller_input` result.
/// Gets the controller input result.
fn controller_input(args: &Args) -> Result<Action, String> {
    if !(7..=9).contains(&args.len()) {
        return Err(
            "use controller_input:BUTTONS:LX:LY:RX:RY:FRAMES[:LEFT_REF[:RIGHT_REF]]".into(),
        );
    }
    Ok(Action::Controller {
        buttons: args.buttons(1)?,
        stick: args.stick(2)?,
        cstick: args.stick(4)?,
        frames: args.duration(6, 4)?,
        reference: args.reference(7)?,
        cstick_reference: args.reference(8)?,
    })
}

/// Returns the `opponent_input` result.
/// Gets the opponent input result.
fn opponent_input(args: &Args) -> Result<Action, String> {
    args.exact(5)?;
    Ok(Action::OpponentInput {
        buttons: args.buttons(1)?,
        stick: args.stick(2)?,
        frames: args.duration(4, 1)?,
    })
}

/// Returns the `hold_until_manual` result.
/// Gets the hold until manual result.
fn hold_until_manual(args: &Args) -> Result<Action, String> {
    args.exact(4)?;
    let held = args.buttons(1)?;
    if held == 0 {
        return Err("hold_until_manual requires a button".into());
    }
    Ok(input(
        held,
        args.stick(2)?,
        0,
        args.parts[0] == "facing_hold_until_manual",
    ))
}

/// Returns the `walk` result.
/// Gets the walk result.
fn walk(args: &Args) -> Result<Action, String> {
    if args.len() != 4 && args.len() != 5 {
        return Err("use walk:DIRECTION:UNITS:STRENGTH[:MIN_GAP]".into());
    }
    let direction = args.direction(1)?;
    let min_gap = if args.len() == 5 {
        if direction != Direction::Opponent {
            return Err("MIN_GAP requires walk:opponent".into());
        }
        Some(args.float(4, 0.0, 60.0)?)
    } else {
        None
    };
    Ok(Action::Walk {
        direction,
        units: args.float(2, 0.0, 100.0)?,
        strength: args.float(3, 0.1, 0.95)?,
        min_gap,
    })
}

/// Runs the to ledge.
fn run_to_ledge(args: &Args) -> Result<Action, String> {
    if !(3..=6).contains(&args.len()) {
        return Err(
            "use run_to_ledge:OFFSET:STRENGTH[:TOLERANCE[:BRAKE_DISTANCE[:MAX_FRAMES]]]".into(),
        );
    }
    Ok(Action::LedgePosition {
        offset: args.float(1, 0., 200.)?,
        strength: args.float(2, 0.3, 1.)?,
        tolerance: args.float_or(3, 1., 0., 20.)?,
        brake: args.float_or(4, 6., 0., 100.)?,
        frames: match args.parts.get(5) {
            Some(value) => parse_frames(value)
                .ok_or_else(|| format!("invalid movement timeout in {}", args.token))?,
            None => 180,
        },
    })
}

/// Returns the `approach` result.
/// Gets the approach result.
fn approach(args: &Args) -> Result<Action, String> {
    args.exact(4)?;
    let airborne = args.parts[0] == "drift_to_opponent";
    Ok(Action::Approach {
        airborne,
        strength: args.float(1, 0.1, if airborne { 1.0 } else { 0.95 })?,
        frames: args.duration(2, 1)?,
        gap: args.float(3, 0.0, 60.0)?,
    })
}

/// Returns the `range_check` result.
/// Gets the range check result.
fn range_check(value: &str, token: &str) -> Result<RangeCheck, String> {
    Ok(match value {
        "ignore" => RangeCheck::Ignore,
        "fighter_range" => RangeCheck::FighterGrabRange,
        "fighter_range_instant" => RangeCheck::FighterGrabRangeInstant,
        value => RangeCheck::Verify(parse_float(value, 0.0, 60.0, token)?),
    })
}

/// `grab[:RANGE]`: an optional range gate followed by one R press.
fn grab(args: &Args, sequence: &mut Sequence) -> Result<(), String> {
    args.at_most(2)?;
    let range = match args.parts.get(1).copied().unwrap_or("ignore") {
        "no_range_check" | "unverified" => "ignore",
        "range_verified" | "verified" => "fighter_range",
        value => value,
    };
    let range = range_check(range, args.token)?;
    if range != RangeCheck::Ignore {
        push(sequence, Action::Range(range))?;
    }
    push(sequence, input(R, [0.0; 2], 1, false))
}

/// Shortcut names (`jab`, `up_b`, `short_hop_nair`, ...) that expand to a
/// facing-relative input. Returns the buttons and unit stick direction.
fn named_input(name: &str) -> Option<(u32, [f32; 2])> {
    Some(match name {
        "short_hop" | "shorthop" | "jump" => (X, [0.0; 2]),
        "full_hop" | "fullhop" => (X, [0.0; 2]),
        "shield" => (ZR, [0.0; 2]),
        "jab" | "attack" => (A, [0.0; 2]),
        "forward_tilt" | "f_tilt" | "ftilt" => (A, [0.6, 0.0]),
        "up_tilt" | "u_tilt" | "utilt" => (A, [0.0, 0.6]),
        "down_tilt" | "d_tilt" | "dtilt" => (A, [0.0, -0.6]),
        "forward_smash" | "f_smash" | "fsmash" => (A, [1.0, 0.0]),
        "up_smash" | "u_smash" | "usmash" => (A, [0.0, 1.0]),
        "down_smash" | "d_smash" | "dsmash" => (A, [0.0, -1.0]),
        "nair" | "neutral_air" | "neutral_aerial" => (A, [0.0; 2]),
        "fair" | "forward_air" | "forward_aerial" => (A, [1.0, 0.0]),
        "bair" | "back_air" | "back_aerial" => (A, [-1.0, 0.0]),
        "uair" | "up_air" | "up_aerial" => (A, [0.0, 1.0]),
        "dair" | "down_air" | "down_aerial" => (A, [0.0, -1.0]),
        "neutral_special" | "neutral_b" | "special" => (B, [0.0; 2]),
        "side_special" | "side_b" => (B, [1.0, 0.0]),
        "up_special" | "up_b" | "final_cutter" => (B, [0.0, 1.0]),
        "down_special" | "down_b" | "stone" => (B, [0.0, -1.0]),
        "dash" => (0, [1.0, 0.0]),
        "crouch" | "fast_fall" => (0, [0.0, -1.0]),
        "roll_forward" => (ZR, [1.0, 0.0]),
        "roll_backward" => (ZR, [-1.0, 0.0]),
        "spot_dodge" => (ZR, [0.0, -1.0]),
        "air_dodge" => (ZR, [0.0; 2]),
        _ => return None,
    })
}

/// Only the documented aerial spellings may take the `short_hop_` prefix.
fn pairs_with_short_hop(name: &str) -> bool {
    matches!(
        name,
        "nair"
            | "fair"
            | "bair"
            | "uair"
            | "dair"
            | "neutral_air"
            | "forward_air"
            | "back_air"
            | "up_air"
            | "down_air"
    )
}

/// The catch-all action family: optional `turnaround_` / `short_hop_` prefixes
/// around a shortcut name, `:FRAMES` and `:STRENGTH` suffixes.
fn named_move(args: &Args, sequence: &mut Sequence) -> Result<(), String> {
    args.at_most(3)?;
    let mut name = args.parts[0];
    if let Some(rest) = name
        .strip_prefix("turnaround_")
        .or_else(|| name.strip_prefix("turn_around_"))
    {
        push(sequence, Action::Face(Direction::Backward))?;
        name = rest;
    }
    let (paired, name) = name
        .strip_prefix("short_hop_")
        .or_else(|| name.strip_prefix("shorthop_"))
        .map_or((false, name), |v| (true, v));
    if paired && !pairs_with_short_hop(name) {
        return Err(format!(
            "{}: short_hop_ requires an aerial name",
            args.token
        ));
    }
    if name == "dash_attack" || name == "dashattack" {
        push(sequence, input(0, [1.0, 0.0], args.duration(1, 1)?, true))?;
        return push(sequence, input(A, [1.0, 0.0], args.duration(2, 1)?, true));
    }
    if let Some(&(bit, _)) = BUTTON_TOKENS.iter().find(|(_, token)| *token == name) {
        args.at_most(2)?;
        return push(sequence, input(bit, [0.0; 2], args.duration(1, 1)?, false));
    }
    let (mask, stick) =
        named_input(name).ok_or_else(|| format!("unknown action {}", args.token))?;
    if matches!(name, "full_hop" | "fullhop") && args.len() < 2 {
        return Err("full_hop requires an explicit hold duration, e.g. full_hop:4".into());
    }
    let strength = args.float_or(2, 1.0, 0.0, 1.0)?;
    push(
        sequence,
        input(
            mask | if paired { X } else { 0 },
            [stick[0] * strength, stick[1] * strength],
            args.duration(1, 1)?,
            true,
        ),
    )
}

/// Long button spellings; the canonical letters come from `BUTTON_TOKENS`.
fn button_alias(name: &str) -> Option<u32> {
    Some(match name {
        "attack" => A,
        "special" => B,
        "jump" => X,
        "grab" => R,
        "shield" => ZR,
        "none" | "0" => 0,
        _ => return None,
    })
}

/// Parses the float.
fn parse_float(value: &str, min: f32, max: f32, token: &str) -> Result<f32, String> {
    let value = value
        .strip_suffix("_units")
        .or_else(|| value.strip_suffix("_unit"))
        .unwrap_or(value);
    let (raw, divisor) = value
        .strip_suffix('%')
        .or_else(|| value.strip_suffix("_percent"))
        .map_or((value, 1.0), |v| (v, 100.0));
    let value: f32 = raw
        .parse()
        .map_err(|_| format!("invalid number in {token}"))?;
    let value = value / divisor;
    if value.is_finite() && (min..=max).contains(&value) {
        Ok(value)
    } else {
        Err(format!("number out of range in {token}"))
    }
}

/// Parses the u 8.
/// Parses the u8.
fn parse_u8(value: &str, token: &str) -> Result<u8, String> {
    let value = value.strip_prefix("until_motion_frame_").unwrap_or(value);
    let value = value
        .strip_suffix("_frames")
        .or_else(|| value.strip_suffix("_frame"))
        .unwrap_or(value);
    value
        .parse()
        .map_err(|_| format!("invalid frame count in {token}"))
}

/// Whole-action frame budgets (taps, movement timeouts): 1..=3600.
fn parse_frames(value: &str) -> Option<u16> {
    value
        .parse::<u16>()
        .ok()
        .filter(|frames| (1..=MAX_SEQUENCE_FRAME_COUNT).contains(frames))
}

#[cfg(test)]
#[path = "../../tests/unit/sequence_parser.rs"]
mod tests;
