//! Parse and validate the startup recipe catalog.
//!
//! This code runs during startup or an explicit save, never in the frame loop.
use super::*;

mod actions;
use actions::parse_actions;
pub use actions::validate_facing_blocks;

impl Catalog {
    pub fn parse(source: &str) -> Result<Self, String> {
        if source.len() as u64 > MAX_FILE_BYTES {
            return Err("sequence file exceeds 512 KiB".into());
        }
        // Entry embeds all 32 actions. Geometric Vec growth temporarily kept
        // both the old and new roster buffers alive during saves. Count headers
        // first, then reserve once and report allocation failure instead of
        // invoking the aborting OOM handler for the largest save allocation.
        let count = source
            .lines()
            .filter(|line| {
                let line = line.split('#').next().unwrap_or("").trim();
                line.starts_with('[')
                    && line.ends_with(']')
                    && !retired_button_header(line)
                    && !settings_header(line)
            })
            .take(MAX_ENTRIES + 1)
            .count();
        if count > MAX_ENTRIES {
            return Err(format!(
                "at most {MAX_ENTRIES} sequence sections are allowed"
            ));
        }
        let mut entries: Vec<Entry> = Vec::new();
        entries.try_reserve_exact(count).map_err(|_| {
            "Not enough memory to validate sequences; retry outside battle".to_owned()
        })?;
        let identities = FighterProfiles::default();
        let mut current: Option<(Entry, usize)> = None;
        let mut numeric_keys = Vec::new();
        let mut retired_buttons = false;
        let mut universal = None::<String>;
        let mut universal_section = false;
        let mut universal_keys = Vec::new();
        let mut tuning_kind = None;
        let mut tuning = std::collections::BTreeMap::new();
        for (index, raw) in source.lines().enumerate() {
            let line_number = index + 1;
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                finish_entry(&mut current, &mut entries)?;
                tuning_kind = None;
                retired_buttons = false;
                universal_section = line == "[universal.settings]";
                if universal_section {
                    if universal.is_some() {
                        return Err(format!("line {line_number}: duplicate universal settings"));
                    }
                    universal = Some(String::new());
                    continue;
                }
                if let Some(id) = line
                    .strip_prefix("[template.")
                    .and_then(|v| v.strip_suffix(']'))
                {
                    if !valid_id(id)
                        || entries
                            .iter()
                            .any(|e| e.fighter_kind == -1 && e.choice == id)
                    {
                        return Err(format!(
                            "line {line_number}: invalid or duplicate template id {id}"
                        ));
                    }
                    numeric_keys.clear();
                    current = Some((
                        Entry {
                            fighter_kind: -1,
                            fighter: "template".into(),
                            slot: PresetSlot::X,
                            choice: id.into(),
                            name: None,
                            default: false,
                            explicit_start: false,
                            sequence: Sequence::default(),
                        },
                        line_number,
                    ));
                    continue;
                }
                retired_buttons = retired_button_header(line);
                if retired_buttons {
                    continue;
                }
                let header = &line[1..line.len() - 1];
                let mut parts = header.split('.');
                let fighter = words_id(parts.next().unwrap_or("").trim());
                let slot = words_id(parts.next().unwrap_or("").trim());
                let choice = words_id(parts.next().unwrap_or("").trim());
                if fighter.is_empty()
                    || slot.is_empty()
                    || (choice.is_empty() && slot != "settings")
                    || parts.next().is_some()
                {
                    return Err(format!(
                        "line {line_number}: section must be [fighter.situation.choice_id]"
                    ));
                }
                let fighter_kind = crate::fighter_data::all()
                    .iter()
                    .find(|entry| entry.name.eq_ignore_ascii_case(&fighter))
                    .map(|entry| entry.kind)
                    .or_else(|| {
                        (0..crate::fighter_kinds::PLAYABLE_KIND_COUNT).find(|kind| {
                            identities
                                .profile(*kind)
                                .is_some_and(|p| p.fighter_kind == *kind && p.name == fighter)
                        })
                    })
                    .ok_or_else(|| format!("line {line_number}: unknown fighter {fighter}"))?;
                if slot == "settings" && choice.is_empty() {
                    if tuning
                        .insert(fighter_kind, crate::fighter_tuning::Tuning::default())
                        .is_some()
                    {
                        return Err(format!("line {line_number}: duplicate fighter settings"));
                    }
                    tuning_kind = Some(fighter_kind);
                    continue;
                }
                let slot = parse_slot(&slot)
                    .ok_or_else(|| format!("line {line_number}: unknown situation {slot}"))?;
                if !valid_id(&choice) {
                    return Err(format!("line {line_number}: invalid choice id {choice}"));
                }
                if entries.iter().any(|entry| {
                    entry.fighter_kind == fighter_kind
                        && entry.slot == slot
                        && entry.choice == choice
                }) {
                    return Err(format!("line {line_number}: duplicate section {header}"));
                }
                numeric_keys.clear();
                current = Some((
                    Entry {
                        fighter_kind,
                        fighter,
                        slot,
                        choice,
                        name: None,
                        default: false,
                        explicit_start: false,
                        sequence: Sequence::default(),
                    },
                    line_number,
                ));
                continue;
            }
            if retired_buttons {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(format!("line {line_number}: expected key = value"));
            };
            if universal_section {
                let key = key.trim().to_ascii_lowercase();
                if universal_keys.contains(&key) {
                    return Err(format!("line {line_number}: duplicate universal key {key}"));
                }
                universal
                    .as_mut()
                    .unwrap()
                    .push_str(&format!("{key} = {}\n", value.trim()));
                universal_keys.push(key);
                continue;
            }
            if let Some(kind) = tuning_kind {
                tuning
                    .get_mut(&kind)
                    .unwrap()
                    .set(key.trim(), value.trim())
                    .map_err(|e| format!("line {line_number}: {e}"))?;
                continue;
            }
            let Some((entry, _)) = current.as_mut() else {
                return Err(format!(
                    "line {line_number}: name/actions must follow a section"
                ));
            };
            match key.trim().to_ascii_lowercase().as_str() {
                "name" | "label" => {
                    if entry.name.is_some() {
                        return Err(format!("line {line_number}: duplicate name"));
                    }
                    let value = value.trim();
                    let value = if value.starts_with('"') || value.ends_with('"') {
                        value
                            .strip_prefix('"')
                            .and_then(|v| v.strip_suffix('"'))
                            .ok_or_else(|| {
                                format!("line {line_number}: name needs matching quotes")
                            })?
                    } else {
                        value
                    };
                    if value.trim().is_empty()
                        || value.len() > 64
                        || value.chars().any(char::is_control)
                    {
                        return Err(format!("line {line_number}: name must be 1–64 bytes without control characters"));
                    }
                    entry.name = Some(value.trim().to_owned());
                }
                "builtin" => {
                    if entry.sequence.builtin.is_some() { return Err(format!("line {line_number}: duplicate builtin")); }
                    entry.sequence.builtin = Some(crate::button_bindings::Builtin::parse(value.trim())
                        .ok_or_else(||format!("line {line_number}: unknown built-in button action"))?);
                }
                "mode" => {
                    if entry.sequence.mode.is_some() {return Err(format!("line {line_number}: duplicate mode"));}
                    let mode=crate::button_bindings::Mode::parse(&words_id(value)).ok_or_else(||format!("line {line_number}: unknown/unimplemented mode {}",value.trim()))?;
                    entry.sequence.mode=Some(crate::button_bindings::ModeBinding {mode,activation:crate::button_bindings::Activation::Hold});
                }
                "activation" => {
                    if numeric_keys.iter().any(|k|k=="activation") {return Err(format!("line {line_number}: duplicate activation"));}
                    let Some(binding)=entry.sequence.mode.as_mut() else {return Err(format!("line {line_number}: activation must follow mode"));};
                    binding.activation=match value.trim() {"hold"=>crate::button_bindings::Activation::Hold,"toggle"=>crate::button_bindings::Activation::Toggle,_=>return Err(format!("line {line_number}: activation must be hold or toggle"))};
                    numeric_keys.push("activation".into());
                }
                "ground_only" => {
                    if numeric_keys.iter().any(|k|k=="ground_only") { return Err(format!("line {line_number}: duplicate ground_only")); }
                    entry.sequence.ground_only=match value.trim() {"true"=>true,"false"=>false,_=>return Err(format!("line {line_number}: ground_only must be true or false"))};
                    numeric_keys.push("ground_only".into());
                }
                "while_held" => {
                    if numeric_keys.iter().any(|k| k=="while_held") { return Err(format!("line {line_number}: duplicate while_held")); }
                    entry.sequence.while_held = match value.trim() { "true" => true, "false" => false, _ => return Err(format!("line {line_number}: while_held must be true or false")) };
                    numeric_keys.push("while_held".into());
                }
                "reaction" => {
                    if numeric_keys.iter().any(|k| k=="reaction") {return Err(format!("line {line_number}: duplicate reaction"));}
                    entry.sequence.reaction=crate::reaction_rules::Rule::parse(value.trim()).map_err(|e|format!("line {line_number}: {e}"))?;
                    numeric_keys.push("reaction".into());
                }
                "actions" => {
                    if entry.sequence.len != 0 {
                        return Err(format!("line {line_number}: duplicate actions"));
                    }
                    let parsed = parse_actions(value.trim(), line_number)?;
                    entry.sequence.actions = parsed.actions;
                    entry.sequence.len = parsed.len;
                }
                "default" => {
                    if numeric_keys.iter().any(|key| key == "default") {
                        return Err(format!("line {line_number}: duplicate default"));
                    }
                    entry.default = match value.trim() { "true" => true, "false" => false, _ => return Err(format!("line {line_number}: default must be true or false")) };
                    numeric_keys.push("default".to_owned());
                }
                "start_delay" | "timeout" | "spatial_timeout" => {
                    let key = key.trim().to_ascii_lowercase();
                    if numeric_keys.contains(&key) {
                        return Err(format!("line {line_number}: duplicate {key}"));
                    }
                    let frames: u16 = value.trim().parse()
                        .map_err(|_| format!("line {line_number}: {key} must be an integer frame count"))?;
                    let exceeds_frame_limit = frames > super::MAX_SEQUENCE_FRAME_COUNT;
                    let missing_required_frame = key != "start_delay" && frames == 0;
                    if exceeds_frame_limit || missing_required_frame {
                        return Err(format!(
                            "line {line_number}: {key} must be {}..{}",
                            if key == "start_delay" { 0 } else { 1 },
                            super::MAX_SEQUENCE_FRAME_COUNT,
                        ));
                    }
                    match key.as_str() {
                        "start_delay" => { entry.sequence.start_delay = frames; entry.explicit_start = true; },
                        "timeout" => entry.sequence.timeout = frames,
                        _ => entry.sequence.spatial_timeout = frames,
                    }
                    numeric_keys.push(key);
                }
                _ => {
                    return Err(format!(
                        "line {line_number}: supported keys are name, actions, builtin, default, start_delay, timeout, spatial_timeout, while_held, ground_only and reaction"
                    ))
                }
            }
        }
        finish_entry(&mut current, &mut entries)?;
        if let Some(settings) = universal.as_deref() {
            validate_universal_settings(settings)?;
        }
        let mut templates = Vec::new();
        entries.retain(|e| {
            if e.fighter_kind != -1 {
                return true;
            }
            templates.push(Template {
                id: e.choice.clone(),
                name: e.name.clone().unwrap_or_else(|| e.choice.replace('_', " ")),
                sequence: e.sequence,
            });
            false
        });
        if templates.len() > MAX_TEMPLATES {
            return Err(format!(
                "at most {MAX_TEMPLATES} generic templates are allowed"
            ));
        }
        Ok(Self {
            entries,
            tuning,
            universal,
            templates,
        })
    }

    pub fn validate(&self, profiles: &FighterProfiles) -> Result<(), String> {
        // Count each row once. The old nested full-catalog scan allocated a
        // temporary row Vec for every section, repeatedly during a single save.
        let mut rows = std::collections::BTreeMap::<(i32, usize), (usize, usize)>::new();
        let mut simple_events = std::collections::BTreeSet::new();
        let mut rule_counts = std::collections::BTreeMap::<(i32, usize), usize>::new();
        for entry in &self.entries {
            validate_facing_blocks(entry.sequence.actions())?;
            crate::reaction_rules::validate_response(entry.sequence)?;
            if let Some(event) = entry.sequence.reaction.trigger.simple() {
                if !simple_events.insert((entry.fighter_kind, event)) {
                    return Err(
                        "Only one auto-react sequence per event per fighter is allowed".into(),
                    );
                }
            }

            if entry.sequence.reaction.active() {
                let count = rule_counts
                    .entry((entry.fighter_kind, entry.sequence.reaction.trigger.family()))
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
            row.0 += usize::from(!entry.sequence.reaction.active());
            row.1 += usize::from(entry.default && !entry.sequence.reaction.active());
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

    pub fn find(&self, fighter_kind: i32, slot: PresetSlot, choice: &str) -> Option<Sequence> {
        self.entries
            .iter()
            .find(|entry| {
                entry.fighter_kind == fighter_kind && entry.slot == slot && entry.choice == choice
            })
            .map(|entry| entry.sequence)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
    /// Startup-only filtering, before menus are frozen and recipes can execute.
    pub fn for_build(mut self, diagnostic: bool) -> Self {
        if !diagnostic {
            self.entries.retain(|entry| {
                !entry.choice.starts_with("test")
                    && !entry
                        .name
                        .as_deref()
                        .is_some_and(|name| name.to_ascii_lowercase().contains("[test]"))
            });
        }
        self
    }
}

// Return only the validation result here. Expanded button settings must not
// reserve config-sized Result/map temporaries in every catalog parser frame,
// including small files that do not contain universal settings at all.
#[inline(never)]
fn validate_universal_settings(settings: &str) -> Result<(), String> {
    crate::live_input::LiveConfig::parse(settings)
        .map(|_| ())
        .map_err(|e| format!("universal.settings: {e}"))
}

// Policy sections contain no 32-action Entry; do not reserve recipe storage for them.
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

fn finish_entry(
    current: &mut Option<(Entry, usize)>,
    entries: &mut Vec<Entry>,
) -> Result<(), String> {
    if let Some((entry, line)) = current.take() {
        if entry.fighter_kind == -1
            && (entry.default
                || entry.sequence.mode.is_some()
                || entry.sequence.builtin.is_some()
                || entry.sequence.reaction.active()
                || entry.sequence.while_held)
        {
            return Err(format!("line {line}: templates contain actions/timings only, without defaults, modes, builtins, reactions or held activation"));
        }
        if let Some(binding) = entry.sequence.mode {
            if !entry.slot.is_button() || !binding.mode.supports(entry.fighter_kind) {
                return Err(format!(
                    "line {line}: mode is unavailable for this fighter/button row"
                ));
            }
            if entry.sequence.len != 0 || entry.sequence.start_delay != 0 {
                return Err(format!(
                    "line {line}: mode bindings cannot contain actions or start delays"
                ));
            }
        }
        if entry.sequence.builtin.is_some()
            && (!entry.slot.is_button()
                || entry.sequence.mode.is_some()
                || entry.sequence.len != 0
                || entry.sequence.start_delay != 0
                || entry.sequence.reaction.active()
                || entry.sequence.while_held
                || entry.sequence.ground_only)
        {
            return Err(format!("line {line}: builtin requires a button row and cannot contain actions, conditions, delays, loops or modes"));
        }
        if !entry.explicit_start
            && entry.sequence.mode.is_none()
            && entry.sequence.builtin.is_none()
        {
            return Err(format!(
                "line {line}: section requires start_delay = FRAMES (use 0 for immediate input)"
            ));
        }
        if entry.sequence.len == 0
            && entry.sequence.mode.is_none()
            && entry.sequence.builtin.is_none()
        {
            return Err(format!("line {line}: section is missing actions"));
        }
        if entries.len() >= MAX_ENTRIES {
            return Err(format!(
                "at most {MAX_ENTRIES} sequence sections are allowed"
            ));
        }
        entries.push(entry);
    }
    Ok(())
}

pub(super) fn words_id(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
        .replace('-', "_")
}

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

pub(super) fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 48
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
