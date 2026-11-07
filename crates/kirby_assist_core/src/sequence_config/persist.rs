//! Explicit-save serialization and surgical catalog edits; never called by execution.
use super::*;

fn direction(d: Direction) -> &'static str {
    match d {
        Direction::Opponent => "opponent",
        Direction::AwayFromOpponent => "away_from_opponent",
        Direction::Ledge => "ledge",
        Direction::Inward => "inward",
        Direction::Outward => "outward",
        Direction::Facing => "facing",
        Direction::Backward => "backward",
        Direction::SavedLedge => "saved_ledge",
        Direction::AwayFromLedge => "away_from_ledge",
    }
}

const INPUT_BUTTONS: [(u32, &str); 8] = [
    (crate::native_input::MAPPER_A, "a"),
    (crate::native_input::MAPPER_B, "b"),
    (crate::native_input::MAPPER_X, "x"),
    (crate::native_input::MAPPER_Y, "y"),
    (crate::native_input::MAPPER_L, "l"),
    (crate::native_input::MAPPER_R, "r"),
    (crate::native_input::MAPPER_ZL, "zl"),
    (crate::native_input::MAPPER_ZR, "zr"),
];

fn button_names(buttons: u32) -> Result<String, String> {
    let known = INPUT_BUTTONS
        .iter()
        .fold(0, |mask, (button, _)| mask | button);
    if buttons & !known != 0 {
        return Err("This input contains unsupported button bits".into());
    }

    let names = INPUT_BUTTONS
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

fn action(a: Action) -> Result<String, String> {
    Ok(match a {
        Action::IfFacingLedge { away } => {
            if away {
                "if_back_to_ledge".into()
            } else {
                "if_facing_ledge".into()
            }
        }
        Action::IfFacingOpponent { away } => {
            if away {
                "if_back_to_opponent".into()
            } else {
                "if_facing_opponent".into()
            }
        }
        Action::ElseFacing => "else_facing".into(),
        Action::LedgePosition {
            offset,
            strength,
            tolerance,
            brake,
            frames,
        } => format!("run_to_ledge:{offset}:{strength}:{tolerance}:{brake}:{frames}"),
        Action::EndFacing => "end_facing".into(),
        Action::Empty => return Err("Remove empty execution steps before saving".into()),
        Action::Check {
            condition,
            value,
            wait,
        } => {
            if condition.has_distance() {
                format!("check:{}:{value}:{wait}", condition.token())
            } else {
                format!("check:{}:{wait}", condition.token())
            }
        }
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
            let names = button_names(buttons)?;
            format!(
                "controller_input:{names}:{}:{}:{}:{}:{frames}:{}:{}",
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
            let names = button_names(buttons)?;
            format!("opponent_input:{names}:{}:{}:{frames}", stick[0], stick[1])
        }
        Action::Input {
            buttons,
            stick,
            frames,
            relative,
        } => {
            let names = button_names(buttons)?;
            if frames == 0 {
                if buttons == 0 {
                    return Err("Until-manual hold requires a button".into());
                }
                return Ok(format!(
                    "{}:{}:{}:{}",
                    if relative {
                        "facing_hold_until_manual"
                    } else {
                        "hold_until_manual"
                    },
                    names,
                    stick[0],
                    stick[1]
                ));
            }
            format!(
                "{}:{}:{}:{}:{}",
                if relative { "facing_input" } else { "input" },
                names,
                stick[0],
                stick[1],
                frames
            )
        }
        Action::Tap {
            buttons,
            stick,
            hold,
            gap,
            frames,
        } => {
            let names = button_names(buttons)?;
            if hold == 0 && buttons == 0 {
                return Err("Until-manual hold requires a button".into());
            }
            format!(
                "tap:{names}:{}:{}:{hold}:{gap}:{frames}",
                stick[0], stick[1]
            )
        }
        Action::CStick { stick, frames } => format!("cstick:{}:{}:{frames}", stick[0], stick[1]),
        Action::Wait(n) => format!("wait:{n}"),
        Action::Face(d) => format!("face:{}", direction(d)),
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
        } => format!(
            "{}:{strength}:{frames}:{gap}",
            if airborne {
                "drift_to_opponent"
            } else {
                "walk_to_opponent"
            }
        ),
        Action::Walk {
            direction: d,
            units,
            strength,
            min_gap,
        } => {
            let mut s = format!("walk:{}:{units}:{strength}", direction(d));
            if let Some(gap) = min_gap {
                s.push_str(&format!(":{gap}"));
            }
            s
        }
        Action::Run {
            direction: d,
            units,
        } => format!("run:{}:{units}", direction(d)),
        Action::Drift {
            direction: d,
            strength,
            frames,
        } => format!("drift:{}:{strength}:{frames}", direction(d)),
        Action::Range(r) => format!(
            "range:{}",
            match r {
                RangeCheck::Ignore => "ignore".into(),
                RangeCheck::FighterGrabRange => "fighter_range".into(),
                RangeCheck::FighterGrabRangeInstant => "fighter_range_instant".into(),
                RangeCheck::Verify(v) => v.to_string(),
            }
        ),
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
        return Err("Add an action or delay before saving".into());
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
    let mut properties = if let Some(builtin) = sequence.builtin {
        if !slot.is_button()
            || sequence.len != 0
            || sequence.mode.is_some()
            || sequence.start_delay != 0
            || sequence.reaction.active()
            || sequence.while_held
            || sequence.ground_only
        {
            return Err("Built-in helpers cannot include recipe actions or conditions".into());
        }
        format!("builtin = {}\n", builtin.token())
    } else if let Some(binding) = sequence.mode {
        if !slot.is_button() || sequence.len != 0 || sequence.start_delay != 0 {
            return Err("Mode bindings cannot include action timings".into());
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
    if sequence.reaction != crate::reaction_rules::Rule::default() {
        properties.push_str(&format!("reaction = {}\n", sequence.reaction.token()));
    }
    let mut output = String::with_capacity(source.len() + properties.len());
    let mut inside = false;
    let mut replaced = false;
    for line in source.split_inclusive('\n') {
        let clean = line.split('#').next().unwrap_or("").trim();
        if clean.starts_with('[') && clean.ends_with(']') {
            let parts = clean[1..clean.len() - 1]
                .split('.')
                .map(words_id)
                .collect::<Vec<_>>();
            inside = parts.len() == 3
                && parts[0] == fighter
                && parse_slot(&parts[1]) == Some(slot)
                && parts[2] == choice;
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
    if saved.sequence.ground_only != sequence.ground_only
        || saved.sequence.reaction != sequence.reaction
        || saved.sequence.while_held != sequence.while_held
        || saved.sequence.actions() != sequence.actions()
        || saved.sequence.start_delay != sequence.start_delay
        || saved.sequence.timeout != sequence.timeout
        || saved.sequence.spatial_timeout != sequence.spatial_timeout
    {
        return Err("Saved recipe did not round-trip exactly".into());
    }
    drop(parsed);
    super::layout::for_save(source, output)
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
pub fn validate_output(source: &str) -> Result<(), String> {
    let catalog = Catalog::parse(source)?;
    validate_catalog(&catalog)
}

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
mod tests {
    use super::*;
    const SOURCE: &str = "# keep\n[yoshi.roll.grab]\nname = Tongue grab\ndefault=true\nstart_delay=0\nactions=grab\n\n[kirby.roll.a]\nstart_delay=0\nactions=a\n";
    fn sequence(actions: &str) -> Sequence {
        Catalog::parse(&format!("[yoshi.roll.x]\nstart_delay=4\nactions={actions}"))
            .unwrap()
            .entries()[0]
            .sequence
    }

    #[test]
    fn while_held_survives_save_and_can_be_disabled_without_duplicate_keys() {
        let mut seq = sequence("tap:a:0:-0.6:1:11:120");
        seq.while_held = true;
        let text = update(
            "[yoshi.y.repeat]\nstart_delay=0\nactions=a\n",
            5,
            PresetSlot::Y,
            "repeat",
            seq,
            false,
        )
        .unwrap();
        assert!(
            Catalog::parse(&text).unwrap().entries()[0]
                .sequence
                .while_held
        );
        seq.while_held = false;
        let text = update(&text, 5, PresetSlot::Y, "repeat", seq, false).unwrap();
        assert!(!text.contains("while_held"));
        assert!(
            !Catalog::parse(&text).unwrap().entries()[0]
                .sequence
                .while_held
        );
    }
    #[test]
    fn taps_roundtrip_as_one_step_and_reject_zero_release_or_invalid_durations() {
        let original = sequence("tap:a+b:0:-0.6:1:11:1800");
        assert_eq!(original.len, 1);
        let text = update(SOURCE, 5, PresetSlot::Roll, "grab", original, false).unwrap();
        assert_eq!(
            Catalog::parse(&text).unwrap().entries()[0]
                .sequence
                .actions(),
            original.actions()
        );
        for bad in [
            "tap:none:0:0:1:1:60",
            "tap:a:0:0:0:1:60",
            "tap:a:0:0:1:0:60",
            "tap:a:0:0:1:1:0",
            "tap:a:0:0:1:1:3601",
            "tap:a:0:0:256:1:60",
            "tap:a:NaN:0:1:1:60",
        ] {
            assert!(
                Catalog::parse(&format!("[yoshi.roll.bad]\nstart_delay=0\nactions={bad}")).is_err(),
                "{bad}"
            );
        }
    }
    #[test]
    fn controller_and_opponent_inputs_roundtrip_without_rewriting_button_names() {
        let original = sequence(
            "controller_input:a+b:0.25:-0.5:0.75:-1:4:facing:opponent,opponent_input:b:1:0:8",
        );
        let text = update(SOURCE, 5, PresetSlot::Roll, "grab", original, false).unwrap();
        let saved = Catalog::parse(&text).unwrap().entries()[0].sequence;
        assert_eq!(saved.actions(), original.actions());
    }
    #[test]
    fn explicit_checks_serialize_and_validate() {
        let original = sequence(
            "check:enemy_within:12:4,check:enemy_shielding,check:self_grounded:255,hold:a:60",
        );
        let text = update(SOURCE, 5, PresetSlot::Roll, "grab", original, false).unwrap();
        let parsed = Catalog::parse(&text).unwrap();
        assert_eq!(parsed.entries()[0].sequence.actions(), original.actions());
        for bad in [
            "check:unknown",
            "check:enemy_within",
            "check:enemy_within:NaN",
            "check:enemy_shielding:256",
            "check:enemy_shielding:0:1",
        ] {
            assert!(
                Catalog::parse(&format!("[yoshi.roll.bad]\nstart_delay=0\nactions={bad}")).is_err(),
                "{bad}"
            );
        }
    }
    #[test]
    fn saving_a_ledge_recipe_preserves_ignored_retired_button_sections() {
        let archived = "[universal.x.old]\nname=Archived\nstart_delay=0\nactions=old_syntax\n";
        let source = format!("{archived}[kirby.neutral.keep]\nstart_delay=0\nactions=grab\n");
        let updated = update(
            &source,
            6,
            PresetSlot::Neutral,
            "keep",
            sequence("hold:a:4"),
            false,
        )
        .unwrap();
        assert!(updated.starts_with(archived));
        assert_eq!(Catalog::parse(&updated).unwrap().len(), 1);
    }
    #[test]
    fn diagnostic_rename_delete_and_add_preserve_other_sections_and_validate() {
        let seq = sequence("grab");
        let renamed = update_named(
            SOURCE,
            5,
            PresetSlot::Roll,
            "grab",
            seq,
            false,
            Some("Perfect roll grab"),
        )
        .unwrap();
        let c = Catalog::parse(&renamed).unwrap();
        assert_eq!(c.entries()[0].name.as_deref(), Some("Perfect roll grab"));
        assert!(c.entries()[0].default);
        assert!(renamed.ends_with("[kirby.roll.a]\nstart_delay=0\nactions=a\n"));
        let added = update_named(
            &renamed,
            5,
            PresetSlot::Roll,
            "menu_added_7",
            seq,
            true,
            Some("New setup"),
        )
        .unwrap();
        let removed = delete(&added, 5, PresetSlot::Roll, "grab", false).unwrap();
        let c = Catalog::parse(&removed).unwrap();
        assert_eq!(c.entries().len(), 2);
        assert!(c.entries().iter().any(|e| e.choice == "menu_added_7"));
        assert!(!c.entries().iter().any(|e| e.choice == "grab"));
        assert_eq!(
            delete(&removed, 5, PresetSlot::Roll, "grab", true).unwrap(),
            removed
        );
        for bad in ["", "bad#name", "bad\nname", "bad\"name"] {
            assert!(
                update_named(SOURCE, 5, PresetSlot::Roll, "grab", seq, false, Some(bad)).is_err()
            );
        }
    }
    #[test]
    fn saved_recipe_preserves_other_sections_label_and_default() {
        let s = sequence("walk:inward:4:0.5,wait:2,face:opponent,grab:fighter_range");
        let result = update(SOURCE, 5, PresetSlot::Roll, "grab", s, false).unwrap();
        assert!(result.starts_with("# keep\n"));
        assert!(result.contains("name = Tongue grab\ndefault=true"));
        assert!(result.ends_with("[kirby.roll.a]\nstart_delay=0\nactions=a\n"));
        assert_eq!(Catalog::parse(&result).unwrap().entries()[0].sequence, s);
    }
    #[test]
    fn saved_recipe_roundtrips_palette_actions_and_custom_updates_without_duplicates() {
        let s=sequence("facing_input:x+a:0.3:-0.5:4,input:none:0:0:2,cstick:1:0:4,walk_to_opponent:0.5:10:4,drift_to_opponent:0.5:10:4,run:ledge:4,drift:away:0.25:5,range:ignore,range:4,range:fighter_range,range:fighter_range_instant,shield_until_block:9");
        let first = update(SOURCE, 5, PresetSlot::Roll, "menu_custom", s, true).unwrap();
        let second = update(&first, 5, PresetSlot::Roll, "menu_custom", s, true).unwrap();
        assert_eq!(first, second);
        let catalog = Catalog::parse(&second).unwrap();
        assert_eq!(catalog.entries().len(), 3);
        assert_eq!(
            catalog
                .entries()
                .iter()
                .find(|e| e.choice == "menu_custom")
                .unwrap()
                .sequence,
            s
        );
    }
    #[test]
    fn saved_recipe_rejects_invalid_source_removed_sections_and_invalid_edits() {
        let s = sequence("grab");
        assert!(update("invalid", 5, PresetSlot::Roll, "grab", s, false).is_err());
        assert!(update(SOURCE, 5, PresetSlot::Roll, "missing", s, false).is_err());
        assert!(update(
            SOURCE,
            6,
            PresetSlot::Roll,
            "grab",
            Sequence::default(),
            false
        )
        .is_err());
        let mut bad = s;
        bad.start_delay = 3601;
        assert!(update(SOURCE, 5, PresetSlot::Roll, "grab", bad, false).is_err());
    }
}

#[cfg(test)]
mod default_selection_tests {
    use super::*;
    #[test]
    fn startup_default_is_unique_and_other_fighters_categories_and_actions_are_untouched() {
        let source = "# preserve\n[kirby.neutral.first]\nstart_delay=0\ndefault=true\nactions=grab\n[kirby.neutral.second]\nstart_delay=0\nname=Second\nactions=wait:2,jab\n[yoshi.neutral.first]\nstart_delay=0\ndefault=true\nactions=grab\n[kirby.x.first]\nstart_delay=0\ndefault=true\nactions=nair\n";
        let result = set_default(source, 6, PresetSlot::Neutral, "second").unwrap();
        let parsed = Catalog::parse(&result).unwrap();
        assert!(!parsed.entries()[0].default);
        assert!(parsed.entries()[1].default);
        assert_eq!(
            parsed.entries()[1].sequence,
            Catalog::parse(source).unwrap().entries()[1].sequence
        );
        assert_eq!(
            result.split("[yoshi.").nth(1),
            source.split("[yoshi.").nth(1)
        );
        assert!(set_default(source, 6, PresetSlot::Neutral, "missing").is_err());
        assert_eq!(
            set_default(&result, 6, PresetSlot::Neutral, "second").unwrap(),
            result
        );
    }
}

#[cfg(test)]
mod unchanged_save_tests {
    use super::*;
    #[test]
    fn same_sequence_and_name_preserve_original_bytes() {
        let original="[yoshi.neutral.test]\n# preserve formatting\nname=Grab\nstart_delay=0\nactions=grab:4\n";
        let c = Catalog::parse(original).unwrap();
        let seq = c.entries()[0].sequence;
        assert_eq!(
            update_named(
                original,
                5,
                PresetSlot::Neutral,
                "test",
                seq,
                false,
                Some("Grab")
            )
            .unwrap(),
            original
        );
        let renamed = update_named(
            original,
            5,
            PresetSlot::Neutral,
            "test",
            seq,
            false,
            Some("Other grab"),
        )
        .unwrap();
        assert_eq!(
            Catalog::parse(&renamed).unwrap().entries()[0]
                .name
                .as_deref(),
            Some("Other grab")
        );
    }
}

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
            result.push_str(&format!("{key} = {text}\n"));
        }
    }
    validate_output(&result)?;
    super::layout::for_save(source, result)
}
