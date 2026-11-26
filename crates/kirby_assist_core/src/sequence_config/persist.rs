//! Explicit-save serialization and surgical catalog edits; never called by execution.
use super::*;

/// Returns a byte-preserving header migration for an explicit save or text editor.
/// Conflicts fail atomically; this helper never accesses or writes a file.
pub fn migrate_mode_swap(source: &str) -> Result<String, String> {
    let catalog = Catalog::parse(source)?;
    let legacy: Vec<_> = catalog
        .entries()
        .iter()
        .filter(|entry| {
            entry.slot == PresetSlot::DpadUp
                && entry.sequence.builtin == Some(crate::button_bindings::Builtin::LModeSwap)
        })
        .map(|entry| (entry.fighter.clone(), entry.choice.clone()))
        .collect();
    let mut migrated = catalog;
    migrated.migrate_mode_swap()?;
    if legacy.is_empty() {
        return Ok(source.to_owned());
    }
    let mut output = String::with_capacity(source.len());
    for line in source.split_inclusive('\n') {
        let header = parser::strip_comment(line);
        let replacement = header
            .strip_prefix('[')
            .and_then(|h| h.strip_suffix(']'))
            .and_then(|h| {
                let parts: Vec<_> = h.split('.').collect();
                (parts.len() == 3
                    && parse_slot(parts[1]) == Some(PresetSlot::DpadUp)
                    && legacy.iter().any(|(fighter, choice)| {
                        *fighter == words_id(parts[0]) && *choice == words_id(parts[2])
                    }))
                .then(|| format!("[{}.r3.{}]", parts[0], parts[2]))
            });
        if let Some(replacement) = replacement {
            let start = line.find('[').unwrap();
            let end = start + header.len();
            output.push_str(&line[..start]);
            output.push_str(&replacement);
            output.push_str(&line[end..]);
        } else {
            output.push_str(line);
        }
    }
    Catalog::parse(&output)?.validate(&FighterProfiles::default())?;
    Ok(output)
}
/// Returns the `button_names` result.
/// Gets the button names result.
fn button_names(buttons: u32) -> Result<String, String> {
    let known = BUTTON_TOKENS
        .iter()
        .fold(0, |mask, (button, _)| mask | button);
    if buttons & !known != 0 {
        return Err("This input contains unsupported button bits".into());
    }

    let names = BUTTON_TOKENS
        .iter()
        .filter(|(button, _)| buttons & button != 0)
        .map(|(_, name)| *name)
        .collect::<Vec<_>>()
        .join("+");
    Ok(if names.is_empty() {
        "none".into()
    } else {
        names
    })
}

/// Text of `buttons`/`stick` chords that hold for `frames`; zero frames means
/// "until the player takes over", which needs at least one button.
fn input_action(
    buttons: u32,
    stick: [f32; 2],
    frames: u8,
    relative: bool,
) -> Result<String, String> {
    let names = button_names(buttons)?;
    if frames > 0 {
        let name = if relative { "facing_input" } else { "input" };
        return Ok(format!("{name}:{names}:{}:{}:{frames}", stick[0], stick[1]));
    }
    if buttons == 0 {
        return Err("Until-manual hold requires a button".into());
    }
    let name = if relative {
        "facing_hold_until_manual"
    } else {
        "hold_until_manual"
    };
    Ok(format!("{name}:{names}:{}:{}", stick[0], stick[1]))
}

/// Returns the `action` result.
/// Gets the action result.
fn action(a: Action) -> Result<String, String> {
    Ok(match a {
        Action::Empty => return Err("Remove empty execution steps before saving".into()),
        Action::IfFacingLedge { away } => (if away {
            "if_back_to_ledge"
        } else {
            "if_facing_ledge"
        })
        .into(),
        Action::IfFacingOpponent { away } => (if away {
            "if_back_to_opponent"
        } else {
            "if_facing_opponent"
        })
        .into(),
        Action::ElseFacing => "else_facing".into(),
        Action::EndFacing => "end_facing".into(),
        Action::LedgePosition {
            offset,
            strength,
            tolerance,
            brake,
            frames,
        } => format!("run_to_ledge:{offset}:{strength}:{tolerance}:{brake}:{frames}"),
        Action::Check {
            condition,
            value,
            wait,
        } if condition.has_distance() => {
            format!("check:{}:{value}:{wait}", condition.token())
        }
        Action::Check {
            condition, wait, ..
        } => format!("check:{}:{wait}", condition.token()),
        Action::Controller {
            buttons,
            stick,
            cstick,
            frames,
            reference,
            cstick_reference,
        } => {
            if frames == 0 {
                return Err("Controller input requires a positive hold duration".into());
            }
            format!(
                "controller_input:{}:{}:{}:{}:{}:{frames}:{}:{}",
                button_names(buttons)?,
                stick[0],
                stick[1],
                cstick[0],
                cstick[1],
                reference.token(),
                cstick_reference.token()
            )
        }
        Action::OpponentInput {
            buttons,
            stick,
            frames,
        } => {
            if frames == 0 {
                return Err("Opponent-relative input requires a positive duration".into());
            }
            format!(
                "opponent_input:{}:{}:{}:{frames}",
                button_names(buttons)?,
                stick[0],
                stick[1]
            )
        }
        Action::Input {
            buttons,
            stick,
            frames,
            relative,
        } => input_action(buttons, stick, frames, relative)?,
        Action::Tap {
            buttons,
            stick,
            hold,
            gap,
            frames,
        } => {
            let names = button_names(buttons)?;
            if buttons == 0 || hold == 0 || gap == 0 || frames == 0 {
                return Err("Repeated taps require a button and positive hold, gap and duration".into());
            }
            format!(
                "tap:{names}:{}:{}:{hold}:{gap}:{frames}",
                stick[0], stick[1]
            )
        }
        Action::CStick { stick, frames } => format!("cstick:{}:{}:{frames}", stick[0], stick[1]),
        Action::Wait(n) => format!("wait:{n}"),
        Action::Face(d) => format!("face:{}", d.token()),
        Action::Chase {
            gap,
            strength,
            frames,
        } => format!("chase_if_far:{gap}:{strength}:{frames}"),
        Action::Approach {
            airborne,
            strength,
            frames,
            gap,
        } => {
            let name = if airborne {
                "drift_to_opponent"
            } else {
                "walk_to_opponent"
            };
            format!("{name}:{strength}:{frames}:{gap}")
        }
        Action::Walk {
            direction,
            units,
            strength,
            min_gap,
        } => {
            let mut text = format!("walk:{}:{units}:{strength}", direction.token());
            if let Some(gap) = min_gap {
                text.push_str(&format!(":{gap}"));
            }
            text
        }
        Action::Run { direction, units } => format!("run:{}:{units}", direction.token()),
        Action::Drift {
            direction,
            strength,
            frames,
        } => format!("drift:{}:{strength}:{frames}", direction.token()),
        Action::Range(RangeCheck::Ignore) => "range:ignore".into(),
        Action::Range(RangeCheck::FighterGrabRange) => "range:fighter_range".into(),
        Action::Range(RangeCheck::FighterGrabRangeInstant) => "range:fighter_range_instant".into(),
        Action::Range(RangeCheck::Verify(distance)) => format!("range:{distance}"),
        Action::ShieldUntilManual => "shield_until_manual".into(),
        Action::ShieldUntilBlock(n) => format!("shield_until_block:{n}"),
    })
}

/// Replace only one recipe's actions/timings, preserving its label, default and
/// every other section. A new Custom recipe gets a separate, nondefault entry.
pub fn update(
    source: &str,
    kind: i32,
    slot: PresetSlot,
    choice: &str,
    sequence: Sequence,
    allow_new: bool,
) -> Result<String, String> {
    if usize::from(sequence.len) > super::MAX_ACTIONS {
        return Err("Recipe exceeds the action limit; no changes saved".into());
    }
    let catalog = Catalog::parse(source)?;
    if !valid_id(choice) {
        return Err("Invalid saved choice ID".into());
    }
    if sequence.len == 0 && sequence.mode.is_none() && sequence.builtin.is_none() {
        if !matches!(sequence.builtin, Some(crate::button_bindings::Builtin::Normal)) {
            return Err("Add an action or delay before saving".into());
        }
    }
    let existing = catalog
        .entries()
        .iter()
        .find(|e| e.fighter_kind == kind && e.slot == slot && e.choice == choice);
    if existing.is_none() && !allow_new {
        return Err("This section was removed from sequences.txt; reload before saving".into());
    }
    if existing.is_some_and(|entry| entry.sequence == sequence) {
        validate_catalog(&catalog)?;
        return Ok(source.to_owned());
    }
    let fighter = existing
        .or_else(|| catalog.entries().iter().find(|e| e.fighter_kind == kind))
        .map(|e| e.fighter.clone())
        .or_else(|| {
            FighterProfiles::default()
                .profile(kind)
                .map(|p| words_id(p.name))
        })
        .ok_or("Unknown fighter; cannot safely choose a header")?;
    // Only the small owned fighter identity is needed while rewriting. Release
    // the full input catalog before parsing/validating the output catalog.
    drop(catalog);
    let properties = recipe_properties(kind, slot, &sequence)?;
    let mut output = String::with_capacity(source.len() + properties.len());
    let mut inside = false;
    let mut replaced = false;
    for line in source.split_inclusive('\n') {
        let clean = line.split('#').next().unwrap_or("").trim();
        if clean.starts_with('[') && clean.ends_with(']') {
            inside = matches_section(clean, &fighter, slot, choice);
            output.push_str(line);
            if inside {
                if !line.ends_with('\n') {
                    output.push('\n');
                }
                output.push_str(&properties);
                replaced = true;
            }
            continue;
        }
        let key = clean
            .split_once('=')
            .map(|(key, _)| key.trim().to_ascii_lowercase());
        if inside
            && key.as_deref().is_some_and(|key| {
                matches!(
                    key,
                    "while_held"
                        | "ground_only"
                        | "auto_tilt"
                        | "auto_tilts"
                        | "tilt_turn"
                        | "reaction"
                        | "actions"
                        | "start_delay"
                        | "timeout"
                        | "spatial_timeout"
                        | "mode"
                        | "builtin"
                        | "activation"
                )
            })
        {
            continue;
        }
        output.push_str(line);
    }
    if !replaced {
        if !allow_new {
            return Err("Could not locate original section".into());
        }
        output.push_str(&format!(
            "\n[{fighter}.{}.{}]\n{properties}name = Custom (saved)\ndefault = false\n",
            slot_name(slot),
            choice
        ));
    }
    // Reject invalid values, oversized catalogs and expanded-menu overflow.
    let parsed = Catalog::parse(&output)?;
    validate_catalog(&parsed)?;
    let saved = parsed
        .entries()
        .iter()
        .find(|e| e.fighter_kind == kind && e.slot == slot && e.choice == choice)
        .ok_or("Saved recipe not found after validation")?;
    if saved.sequence.mode != sequence.mode
        || saved.sequence.builtin != sequence.builtin
        || saved.sequence.ground_only != sequence.ground_only
        || saved.sequence.reaction != sequence.reaction
        || saved.sequence.while_held != sequence.while_held
        || saved.sequence.actions() != sequence.actions()
        || saved.sequence.start_delay != sequence.start_delay
        || saved.sequence.timeout != sequence.timeout
        || saved.sequence.spatial_timeout != sequence.spatial_timeout
        || saved.sequence.auto_tilt != sequence.auto_tilt
    {
        return Err("Saved recipe did not round-trip exactly".into());
    }
    drop(parsed);
    super::layout::for_save(source, output)
}

fn recipe_properties(kind: i32, slot: PresetSlot, sequence: &Sequence) -> Result<String, String> {
    let mut properties = if let Some(builtin) = sequence.builtin {
        if matches!(builtin, crate::button_bindings::Builtin::Normal) {
            String::from("builtin = normal\n")
        } else {
        if !slot.is_button()
            || sequence.len != 0
            || sequence.mode.is_some()
            || sequence.start_delay != 0
            || sequence.reaction.active()
            || sequence.while_held
            || sequence.ground_only
            || sequence.auto_tilt.is_some()
        {
            return Err("Built-in helpers cannot include recipe actions or conditions".into());
        }
        format!("builtin = {}\n", builtin.token())
        }
    } else if let Some(binding) = sequence.mode {
        if !slot.is_button()
            || !binding.mode.supports(kind)
            || sequence.len != 0
            || sequence.start_delay != 0
            || sequence.reaction.active()
            || sequence.while_held
            || sequence.ground_only
            || sequence.auto_tilt.is_some()
        {
            return Err("Mode bindings cannot include recipe actions or conditions".into());
        }
        format!(
            "mode = {}\nactivation = {}\n",
            binding.mode.id(),
            binding.activation.id()
        )
    } else {
        let actions = sequence
            .actions()
            .iter()
            .map(|a| action(*a))
            .collect::<Result<Vec<_>, _>>()?
            .join(", ");
        let properties = format!(
            "start_delay = {}\ntimeout = {}\nspatial_timeout = {}\nactions = {actions}\n",
            sequence.start_delay, sequence.timeout, sequence.spatial_timeout
        );
        if sequence.while_held {
            format!("while_held = true\n{properties}")
        } else {
            properties
        }
    };
    if sequence.ground_only {
        properties.push_str("ground_only = true\n");
    }
    if let Some(tilts) = sequence.auto_tilt {
        let mut directions = String::new();
        if tilts.down {
            directions.push_str("down,");
        }
        if tilts.forward {
            directions.push_str("forward,");
        }
        if tilts.up {
            directions.push_str("up,");
        }
        directions.pop();
        properties.push_str(&format!("auto_tilt = {directions}\n"));
    }
    if sequence.reaction != crate::reaction_rules::Rule::default() {
        properties.push_str(&format!("reaction = {}\n", sequence.reaction.token()));
    }
    Ok(properties)
}

/// Select exactly one startup default in this fighter/category. Other sections
/// remain byte-for-byte unchanged. Call after update_named to allow new recipes.
pub fn set_default(
    source: &str,
    kind: i32,
    slot: PresetSlot,
    choice: &str,
) -> Result<String, String> {
    let catalog = Catalog::parse(source)?;
    let target = catalog
        .entries()
        .iter()
        .find(|e| e.fighter_kind == kind && e.slot == slot && e.choice == choice)
        .ok_or("Save a recipe before selecting its startup default")?;
    if target.sequence.reaction.active() {
        return Err("Assist rules use their mode ON/OFF control and priority, not a ledge/button startup default".into());
    }
    let fighter = &target.fighter;
    let row: Vec<_> = catalog
        .entries()
        .iter()
        .filter(|e| e.fighter_kind == kind && e.slot == slot && !e.sequence.reaction.active())
        .collect();
    let mut output = String::with_capacity(source.len() + 64);
    let mut selected_row = false;
    for line in source.split_inclusive('\n') {
        let clean = line.split('#').next().unwrap_or("").trim();
        if clean.starts_with('[') && clean.ends_with(']') {
            selected_row = row.iter().any(|e| {
                e.fighter_kind == kind
                    && e.slot == slot
                    && matches_section(clean, fighter, slot, &e.choice)
            });
            output.push_str(line);
            if selected_row {
                if !line.ends_with('\n') {
                    output.push('\n');
                }
                output.push_str(if matches_section(clean, fighter, slot, choice) {
                    "default = true\n"
                } else {
                    "default = false\n"
                });
            }
        } else if !(selected_row
            && clean
                .split_once('=')
                .is_some_and(|(k, _)| words_id(k) == "default"))
        {
            output.push_str(line);
        }
    }
    // The row borrows source entries. Release both before the second complete
    // parse, so default saves use the same bounded peak as ordinary saves.
    drop(row);
    drop(catalog);
    validate_output(&output)?;
    super::layout::for_save(source, output)
}

/// Persist an explicit selected binding without enabling any other row.
fn matches_section(clean: &str, fighter: &str, slot: PresetSlot, choice: &str) -> bool {
    if !clean.starts_with('[') || !clean.ends_with(']') {
        return false;
    }
    let parts = clean[1..clean.len() - 1]
        .split('.')
        .map(words_id)
        .collect::<Vec<_>>();
    parts.len() == 3
        && parts[0] == fighter
        && parse_slot(&parts[1]) == Some(slot)
        && parts[2] == choice
}
/// Parses and validates replacement text without publishing runtime menus.
///
/// This checks the diagnostic superset against default fighter profiles. It
/// returns validation only: callers still own disk backup, atomic replacement
/// and read-back verification. No gameplay state or file is changed here.
pub fn validate_output(source: &str) -> Result<(), String> {
    let catalog = Catalog::parse(source)?;
    validate_catalog(&catalog)
}

/// Validates the catalog.
fn validate_catalog(catalog: &Catalog) -> Result<(), String> {
    // add_sequence_choices is a startup publisher: it deliberately leaks frozen
    // menu references for the game's lifetime. Saves must NEVER call it.
    // Check the full diagnostic superset without publishing anything. Silent
    // only removes test entries, so cannot increase row sizes/default counts.
    let profiles = FighterProfiles::default();
    profiles.validate()?;
    catalog.validate(&profiles)?;
    Ok(())
}
/// Updates the named.
pub fn update_named(
    source: &str,
    kind: i32,
    slot: PresetSlot,
    choice: &str,
    sequence: Sequence,
    allow_new: bool,
    name: Option<&str>,
) -> Result<String, String> {
    let updated = update(source, kind, slot, choice, sequence, allow_new)?;
    let Some(name) = name else {
        return Ok(updated);
    };
    let name = name.trim();
    if name.is_empty()
        || name.len() > 64
        || name
            .chars()
            .any(|c| c.is_control() || matches!(c, '#' | '"'))
    {
        return Err("Name must be 1–64 bytes without quotes, # or control characters".into());
    }
    let catalog = Catalog::parse(&updated)?;
    let entry = catalog
        .entries()
        .iter()
        .find(|e| e.fighter_kind == kind && e.slot == slot && e.choice == choice)
        .ok_or("Cannot find renamed recipe")?;
    let current_name = entry
        .name
        .clone()
        .unwrap_or_else(|| entry.choice.replace('_', " "));
    if current_name == name {
        return Ok(updated);
    }
    let mut inside = false;
    let mut output = String::with_capacity(updated.len() + name.len() + 8);
    for line in updated.split_inclusive('\n') {
        let clean = line.split('#').next().unwrap_or("").trim();
        if clean.starts_with('[') && clean.ends_with(']') {
            inside = matches_section(clean, &entry.fighter, slot, choice);
            output.push_str(line);
            if inside {
                output.push_str(&format!("name = {name}\n"));
            }
        } else if !(inside
            && clean.split_once('=').is_some_and(|(k, _)| {
                matches!(k.trim().to_ascii_lowercase().as_str(), "name" | "label")
            }))
        {
            output.push_str(line);
        }
    }
    drop(catalog);
    validate_output(&output)?;
    super::layout::for_save(source, output)
}
/// Returns the `delete` result.
/// Gets the delete result.
pub fn delete(
    source: &str,
    kind: i32,
    slot: PresetSlot,
    choice: &str,
    allow_missing: bool,
) -> Result<String, String> {
    let catalog = Catalog::parse(source)?;
    let Some(entry) = catalog
        .entries()
        .iter()
        .find(|e| e.fighter_kind == kind && e.slot == slot && e.choice == choice)
    else {
        if allow_missing {
            validate_catalog(&catalog)?;
            return Ok(source.into());
        }
        return Err("Section already removed; reload before deleting".into());
    };
    let mut inside = false;
    let mut output = String::with_capacity(source.len());
    for line in source.split_inclusive('\n') {
        let clean = line.split('#').next().unwrap_or("").trim();
        if clean.starts_with('[') && clean.ends_with(']') {
            inside = matches_section(clean, &entry.fighter, slot, choice);
        }
        if !inside || clean.is_empty() {
            output.push_str(line);
        }
    }
    drop(catalog);
    validate_output(&output)?;
    super::layout::for_save(source, output)
}

#[cfg(test)]
#[path = "../../tests/unit/sequence_persist.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/unit/sequence_default_selection.rs"]
mod default_selection_tests;

#[cfg(test)]
#[path = "../../tests/unit/sequence_unchanged_save.rs"]
mod unchanged_save_tests;

/// Replace only one fighter's policy section, preserving recipes verbatim.
pub fn update_tuning(
    source: &str,
    kind: i32,
    tuning: crate::fighter_tuning::Tuning,
) -> Result<String, String> {
    validate_output(source)?;
    let profiles = FighterProfiles::default();
    let name = profiles.profile(kind).ok_or("Unknown fighter")?.name;
    let mut result = String::with_capacity(source.len() + 512);
    let mut skip = false;
    for raw in source.split_inclusive('\n') {
        let line = raw.split('#').next().unwrap_or("").trim();
        if let Some(header) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            let mut parts = header.split('.');
            skip = parts.next().is_some_and(|p| words_id(p) == name)
                && parts.next().is_some_and(|p| words_id(p) == "settings")
                && parts.next().is_none();
        }
        if !skip {
            result.push_str(raw);
        }
    }
    if !result.ends_with('\n') {
        result.push('\n');
    }
    result.push_str(&format!("\n[{name}.settings]\n"));
    for (i, (key, value)) in crate::fighter_tuning::KEYS.iter().zip(tuning.0).enumerate() {
        if let Some(value) = value {
            if crate::fighter_tuning::Tuning::is_bool(i) && value != 0.0 && value != 1.0 {
                return Err(format!("{key}: expected a boolean setting"));
            }
            let text = if crate::fighter_tuning::Tuning::is_bool(i) {
                (value == 1.0).to_string()
            } else {
                value.to_string()
            };
            result.push_str(&format!("# {}\n{key} = {text}\n", crate::fighter_tuning::HINTS[i]));
        }
    }
    validate_output(&result)?;
    super::layout::for_save(source, result)
}
