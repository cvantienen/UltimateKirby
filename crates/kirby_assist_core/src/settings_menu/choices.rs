//! Stable choice indices: menu drafts never move another recipe's edits.
use super::*;
use crate::sequence_config::Sequence;

pub struct Choice<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub sequence: Sequence,
    pub allow_new: bool,
}
impl MenuSelection {
    /// Build a fresh session from the owned saved catalog. Only differences
    /// from startup need owned records; unchanged file choices remain borrowed.
    /// Never calls the startup static-reference publisher (which would leak).
    pub fn load_saved_recipes(
        &mut self,
        presets: &MenuPresets,
        saved: &crate::sequence_config::Catalog,
        kind: i32,
    ) {
        self.editor = None;
        self.catalog = None;
        let mut records = catalog::Catalog::default();
        for slot in PresetSlot::ALL {
            let options = presets.list(slot).options;
            let entries: Vec<_> = saved
                .entries()
                .iter()
                .filter(|e| e.fighter_kind == kind && e.slot == slot && e.sequence.mode.is_none())
                .collect();
            for (index, old) in options.iter().enumerate() {
                if !entries.iter().any(|e| e.choice == old.id) {
                    records.drafts.insert(
                        (slot as usize, index),
                        catalog::Draft {
                            id: old.id.into(),
                            name: old.name.into(),
                            sequence: Sequence::default(),
                            new: false,
                            deleted: true,
                            dirty: false,
                        },
                    );
                }
            }
            let mut next = options.len() + slot.templates();
            let mut first = None;
            let mut selected = None;
            for entry in entries {
                let old_index = options.iter().position(|o| o.id == entry.choice);
                let index = old_index.unwrap_or_else(|| {
                    let i = next;
                    next += 1;
                    i
                });
                let name = entry
                    .name
                    .clone()
                    .unwrap_or_else(|| entry.choice.replace('_', " "));
                if !old_index.is_some_and(|i| {
                    options[i].name == name && *options[i].sequence == entry.sequence
                }) {
                    records.drafts.insert(
                        (slot as usize, index),
                        catalog::Draft {
                            id: entry.choice.clone(),
                            name,
                            sequence: entry.sequence,
                            new: false,
                            deleted: false,
                            dirty: false,
                        },
                    );
                }
                if !entry.sequence.reaction.active() {
                    first.get_or_insert(index);
                    if entry.default {
                        selected = Some(index);
                    }
                }
            }
            self.choices[slot as usize] = selected.unwrap_or_else(|| {
                if slot.is_button() {
                    slot.default_index(options.len())
                } else {
                    first.unwrap_or(options.len())
                }
            });
        }
        if !records.drafts.is_empty() {
            self.catalog = Some(Box::new(records));
        }
        self.recipe_revision = self.recipe_revision.wrapping_add(1);
        self.revision = self.revision.wrapping_add(1);
    }

    /// Resolve at most eight tagged recipes only after configuration revisions.
    pub fn resolve_reactions(
        &self,
        presets: &MenuPresets,
    ) -> Result<crate::reaction_rules::Resolved, String> {
        let mut result = crate::reaction_rules::Resolved::default();
        for slot in PresetSlot::ALL {
            for index in self.all_choices(presets, slot) {
                // An incomplete draft suspends only its own rule. Never run a
                // stale saved action in its place or disable unrelated rules.
                let Some(sequence) = self.recipe_snapshot(presets, slot, index) else {
                    continue;
                };
                if crate::reaction_rules::validate_response(sequence).is_err()
                    || sequence
                        .actions()
                        .iter()
                        .all(|a| *a == crate::sequence_config::Action::Empty)
                {
                    continue;
                }
                if self.simplified_reactions && sequence.reaction.trigger.simple().is_none() {
                    continue;
                }
                result.add(crate::reaction_rules::Candidate {
                    slot: slot as usize,
                    index,
                    sequence,
                })?;
            }
        }
        Ok(result)
    }
    /// One bounded profile scan on creation/recipe revisions, never idle polls.
    pub(super) fn refresh_rule_bindings(&mut self, presets: &MenuPresets, held: u32) {
        self.has_reactions = false;
        self.auto_grab_configured = 0;
        let mut buttons = 0;
        let mut controls = 0;
        if self.catalog.is_none() && self.editor.is_none() {
            // Normal startup path: borrow only small rule metadata. No choice
            // Vec, strings, full timeline copies or catalog rebuilding.
            for list in &presets.lists {
                for choice in list.options {
                    let rule = choice.sequence.reaction;
                    if self.simplified_reactions && rule.trigger.simple().is_none() {
                        continue;
                    }
                    self.auto_grab_configured |= match rule.trigger {
                        crate::reaction_rules::Trigger::AutoGrab => 1,
                        crate::reaction_rules::Trigger::AutoShieldGrab => 2,
                        _ => 0,
                    };
                    self.has_reactions |= rule.active();
                    if rule.active() && !rule.trigger.defensive() {
                        buttons |= rule.control.button();
                        controls |= rule.control.bit();
                    }
                }
            }
        } else {
            for slot in PresetSlot::ALL {
                for index in self.all_choices(presets, slot) {
                    let Some(sequence) = self.recipe_snapshot(presets, slot, index) else {
                        continue;
                    };
                    if crate::reaction_rules::validate_response(sequence).is_err() {
                        continue;
                    }
                    if self.simplified_reactions && sequence.reaction.trigger.simple().is_none() {
                        continue;
                    }
                    self.auto_grab_configured |= match sequence.reaction.trigger {
                        crate::reaction_rules::Trigger::AutoGrab => 1,
                        crate::reaction_rules::Trigger::AutoShieldGrab => 2,
                        _ => 0,
                    };
                    self.has_reactions |= sequence.reaction.active();
                    if sequence.reaction.active() && !sequence.reaction.trigger.defensive() {
                        let control = sequence.reaction.control;
                        buttons |= control.button();
                        controls |= control.bit();
                    }
                }
            }
        }
        let newly_reserved = buttons & !self.rule_button_mask;
        self.trigger_ready &= !newly_reserved;
        self.button_latch |= held & newly_reserved;
        self.rule_button_mask = buttons;
        self.rule_toggles &= controls;
    }
    pub fn reaction_enabled_controls(&self) -> u8 {
        if self.tuning_page {
            return 0;
        }
        self.rule_toggles
            | if self.roll_rules {
                crate::reaction_rules::Control::Menu.bit()
            } else {
                0
            }
    }
    /// Formatted only during presentation revisions.
    pub fn reaction_control_status(&self) -> String {
        use crate::reaction_rules::Control;
        let mut labels = vec![format!(
            "Menu {}",
            if self.roll_rules { "ON" } else { "OFF" }
        )];
        for c in Control::ALL.into_iter().skip(1) {
            if self.rule_button_mask & c.button() != 0 {
                labels.push(format!(
                    "{} {}",
                    c.label().trim_start_matches("Toggle "),
                    if self.rule_toggles & c.bit() != 0 {
                        "ON"
                    } else {
                        "OFF"
                    }
                ));
            }
        }
        labels.join(" / ")
    }
    /// Resolve only identity for the hot input path; do not copy 32 actions.

    pub fn choice_labels<'a>(
        &'a self,
        presets: &'a MenuPresets,
        slot: PresetSlot,
        index: usize,
    ) -> Option<(&'a str, &'a str)> {
        if let Some(d) = self
            .catalog
            .as_ref()
            .and_then(|c| c.drafts.get(&(slot as usize, index)))
        {
            return (!d.deleted && d.sequence.mode.is_none())
                .then_some((d.id.as_str(), d.name.as_str()));
        }
        if let Some(c) = presets.list(slot).options.get(index) {
            return c.sequence.mode.is_none().then_some((c.id, c.name));
        }
        self.choice(presets, slot, index).map(|c| (c.id, c.name))
    }

    pub fn choice<'a>(
        &'a self,
        presets: &'a MenuPresets,
        slot: PresetSlot,
        index: usize,
    ) -> Option<Choice<'a>> {
        if let Some(d) = self
            .catalog
            .as_ref()
            .and_then(|c| c.drafts.get(&(slot as usize, index)))
        {
            return (!d.deleted && d.sequence.mode.is_none()).then_some(Choice {
                id: &d.id,
                name: &d.name,
                sequence: d.sequence,
                allow_new: d.new,
            });
        }
        let options = presets.list(slot).options;
        if let Some(c) = options.get(index) {
            if c.sequence.mode.is_some() {
                return None;
            }
            return Some(Choice {
                id: c.id,
                name: c.name,
                sequence: *c.sequence,
                allow_new: false,
            });
        }
        if slot.is_button() && index == options.len() {
            if self.choices[slot as usize] != index
                && (options
                    .iter()
                    .any(|c| c.sequence.builtin == Some(crate::button_bindings::Builtin::Normal))
                    || self.catalog.as_ref().is_some_and(|c| {
                        c.drafts.iter().any(|((s, _), d)| {
                            *s == slot as usize
                                && !d.deleted
                                && d.sequence.builtin
                                    == Some(crate::button_bindings::Builtin::Normal)
                        })
                    }))
            {
                return None;
            }
            return Some(Choice {
                id: "__normal_button",
                name: "Normal button",
                sequence: Sequence {
                    builtin: Some(crate::button_bindings::Builtin::Normal),
                    ..Sequence::default()
                },
                allow_new: true,
            });
        }
        if slot.is_button() {
            if let Some(builtin) = crate::button_bindings::Builtin::ALL
                .into_iter()
                .skip(1)
                .find(|b| index == options.len() + b.template_offset())
            {
                if self.choices[slot as usize] != index
                    && (options.iter().any(|c| c.sequence.builtin == Some(builtin))
                        || self.catalog.as_ref().is_some_and(|c| {
                            c.drafts.iter().any(|((s, _), d)| {
                                *s == slot as usize
                                    && !d.deleted
                                    && d.sequence.builtin == Some(builtin)
                            })
                        }))
                {
                    return None;
                }
                return Some(Choice {
                    id: builtin.choice_id(),
                    name: builtin.label(),
                    sequence: Sequence {
                        builtin: Some(builtin),
                        ..Sequence::default()
                    },
                    allow_new: true,
                });
            }
        }
        (index == options.len() + usize::from(slot.is_button())).then(|| Choice {
            id: "menu_custom",
            name: "Custom (6 slots)",
            sequence: editor::RecipeEditor::custom(),
            allow_new: true,
        })
    }
    pub fn managing(&self) -> bool {
        self.catalog.as_ref().is_some_and(|c| c.active())
    }
    pub(super) fn all_choices(&self, presets: &MenuPresets, slot: PresetSlot) -> Vec<usize> {
        let end = presets.list(slot).options.len() + slot.templates();
        let end = self.catalog.as_ref().map_or(end, |c| {
            c.drafts
                .keys()
                .filter(|(s, _)| *s == slot as usize)
                .map(|(_, i)| i + 1)
                .max()
                .unwrap_or(end)
                .max(end)
        });
        (0..end)
            .filter(|i| self.choice(presets, slot, *i).is_some())
            .collect()
    }
    pub fn recipe_snapshot(
        &self,
        presets: &MenuPresets,
        slot: PresetSlot,
        index: usize,
    ) -> Option<Sequence> {
        let key = (slot as usize, index);
        if let Some(editor) = self.editor.as_ref().filter(|e| e.has(key)) {
            return editor.compiled(key); // Never replace an invalid draft with old saved data.
        }
        self.choice(presets, slot, index).map(|c| c.sequence)
    }
    pub(super) fn visible_choices(&self, presets: &MenuPresets, slot: PresetSlot) -> Vec<usize> {
        self.all_choices(presets, slot)
            .into_iter()
            .filter(|index| {
                self.recipe_snapshot(presets, slot, *index)
                    .is_some_and(|s| !s.reaction.active())
            })
            .collect()
    }
    pub(super) fn template_sources(
        &self,
        presets: &MenuPresets,
        destination: (usize, usize),
    ) -> Vec<editor::CopySource> {
        let mut sources = Vec::new();
        for slot in PresetSlot::ALL {
            for index in self.all_choices(presets, slot) {
                if (slot as usize, index) == destination {
                    continue;
                }
                let Some(choice) = self.choice(presets, slot, index) else {
                    continue;
                };
                if choice.id == "__normal_button" {
                    continue;
                }
                let sequence = self
                    .editor
                    .as_ref()
                    .and_then(|e| e.compiled((slot as usize, index)))
                    .unwrap_or(choice.sequence);
                if sequence.mode.is_none()
                    && sequence
                        .actions()
                        .iter()
                        .any(|a| *a != crate::sequence_config::Action::Empty)
                {
                    sources.push(editor::CopySource {
                        slot,
                        index,
                        name: choice.name.to_owned(),
                    });
                }
            }
        }
        sources
    }
    fn quick_choice(&self, presets: &MenuPresets, slot: PresetSlot, index: usize) -> bool {
        self.choice(presets, slot, index).is_some_and(|c| {
            c.sequence.builtin.is_some()
                || !c.allow_new
                || self
                    .catalog
                    .as_ref()
                    .is_some_and(|catalog| catalog.drafts.contains_key(&(slot as usize, index)))
                || self
                    .editor
                    .as_ref()
                    .and_then(|e| e.compiled((slot as usize, index)))
                    .is_some_and(|s| s.len > 0)
        })
    }
    pub fn choice_position(&self, presets: &MenuPresets, slot: PresetSlot) -> (usize, usize) {
        let mut visible = self.visible_choices(presets, slot);
        if self.quick_menu {
            visible.retain(|i| self.quick_choice(presets, slot, *i));
        }
        (
            visible
                .iter()
                .position(|i| *i == self.choices[slot as usize])
                .unwrap_or(visible.len()),
            visible.len(),
        )
    }
    pub(super) fn cycle_choice(
        &self,
        presets: &MenuPresets,
        slot: PresetSlot,
        forward: bool,
    ) -> usize {
        let visible = self.visible_choices(presets, slot);
        let i = visible
            .iter()
            .position(|i| *i == self.choices[slot as usize])
            .unwrap_or(0);
        visible[cycle(i, visible.len(), forward)]
    }
    pub(super) fn cycle_quick_choice(
        &self,
        presets: &MenuPresets,
        slot: PresetSlot,
        forward: bool,
    ) -> usize {
        let visible: Vec<_> = self
            .visible_choices(presets, slot)
            .into_iter()
            .filter(|i| self.quick_choice(presets, slot, *i))
            .collect();
        if visible.is_empty() {
            return self.choices[slot as usize];
        }
        let Some(index) = visible
            .iter()
            .position(|i| *i == self.choices[slot as usize])
        else {
            return if forward {
                visible[0]
            } else {
                visible[visible.len() - 1]
            };
        };
        visible[cycle(index, visible.len(), forward)]
    }
    pub fn name_override(&self, slot: PresetSlot, index: usize) -> Option<&str> {
        {
            return self
                .catalog
                .as_ref()?
                .drafts
                .get(&(slot as usize, index))
                .map(|d| d.name.as_str());
        }
    }
    /// Editing a different recipe must not make a completed snapshot look stale.
    pub fn saved_recipe_matches(
        &self,
        presets: &MenuPresets,
        slot: PresetSlot,
        index: usize,
        sequence: Sequence,
        name: Option<&str>,
    ) -> bool {
        let current = self.recipe_snapshot(presets, slot, index);
        let name_matches = if let Some(name) = name {
            self.name_override(slot, index).or_else(|| {
                let source = self.choice(presets, slot, index)?.sequence;
                source
                    .mode
                    .map(|mode| mode.label())
                    .or_else(|| source.builtin.map(|builtin| builtin.label()))
            }) == Some(name)
        } else {
            // No explicit rename preserves the existing file name. A promoted
            // clean baseline may now own that label without being a newer edit.
            !self
                .catalog
                .as_ref()
                .and_then(|c| c.drafts.get(&(slot as usize, index)))
                .is_some_and(|draft| draft.dirty)
        };
        current == Some(sequence) && name_matches
    }
    pub fn mark_recipe_saved(&mut self, slot: PresetSlot, index: usize) {
        if let Some(editor) = self.editor.as_mut() {
            editor.mark_saved((slot as usize, index));
        }
        if let Some(draft) = self
            .catalog
            .as_mut()
            .and_then(|c| c.drafts.get_mut(&(slot as usize, index)))
        {
            draft.dirty = false;
        }
    }
    pub fn complete_file_save(
        &mut self,
        presets: &MenuPresets,
        slot: PresetSlot,
        index: usize,
        deleted: bool,
    ) {
        let snapshot = self.recipe_snapshot(presets, slot, index);
        self.complete_saved_snapshot(presets, slot, index, deleted, snapshot, None);
    }
    /// Promote the acknowledged worker snapshot, never a newer unsaved edit or
    /// the immutable startup recipe. Completion can arrive after leaving Training.
    pub fn complete_saved_snapshot(
        &mut self,
        presets: &MenuPresets,
        slot: PresetSlot,
        index: usize,
        deleted: bool,
        snapshot: Option<Sequence>,
        name: Option<&str>,
    ) {
        {
            let original = self.choice(presets, slot, index).map(|c| catalog::Draft {
                id: c.id.into(),
                name: c.name.into(),
                sequence: c.sequence,
                new: false,
                deleted,
                dirty: false,
            });
            if let Some(d) = original {
                let c = self.catalog.get_or_insert_with(Default::default);
                let draft = c.drafts.entry((slot as usize, index)).or_insert(d);
                draft.new = false;
                draft.deleted = deleted;
                if !deleted {
                    if let Some(sequence) = snapshot {
                        draft.sequence = sequence;
                    }
                    if !draft.dirty {
                        if let Some(name) = name {
                            draft.name = name.into();
                        }
                    }
                    // Binding/rule/settings caches must see the committed base
                    // even when maintenance consumes completion before input.
                    self.recipe_revision = self.recipe_revision.wrapping_add(1);
                }
                if deleted {
                    // A Custom saved at the template's index becomes a blank
                    // reusable template after deletion, not a hidden last row.
                    let template = presets.list(slot).options.len() + usize::from(slot.is_button());
                    if index == template {
                        c.drafts.remove(&(slot as usize, index));
                        if let Some(editor) = self.editor.as_mut() {
                            editor.discard((slot as usize, index));
                        }
                    }
                    let editing_deleted =
                        self.row.slot() == Some(slot) && self.choices[slot as usize] == index;
                    if self.choices[slot as usize] == index {
                        let visible = self.visible_choices(presets, slot);
                        self.choices[slot as usize] = visible
                            .iter()
                            .copied()
                            .find(|i| *i > index)
                            .unwrap_or_else(|| {
                                visible
                                    .last()
                                    .copied()
                                    .unwrap_or(presets.list(slot).options.len())
                            });
                    }
                    self.recipe_revision = self.recipe_revision.wrapping_add(1);
                    if editing_deleted {
                        self.recipe_editing = false;
                        self.mode_rule_list = self.mode_editor_origin.is_some();
                    }
                    self.preview_slot = None;
                }
            }
        }
    }
    /// Drop Training-only tools once per scene transition. Retain unfinished
    /// drafts (including invalid drafts) and any clean editor differing from its
    /// base; a late save acknowledgement will promote that base first.
    pub fn release_training_tools(&mut self, presets: &MenuPresets) {
        let Some(mut editor) = self.editor.take() else {
            return;
        };
        editor.release_tools();
        editor.retain_recipes(|key, sequence, modified| {
            modified
                || sequence.is_none()
                || PresetSlot::ALL
                    .get(key.0)
                    .and_then(|slot| self.choice(presets, *slot, key.1))
                    .map(|choice| choice.sequence)
                    != sequence
        });
        if !editor.is_empty() {
            self.editor = Some(editor);
        }
    }
    pub(super) fn manage_input(&mut self, presets: &MenuPresets, buttons: u32, pressed: u32) {
        let Some(slot) = self.row.slot() else {
            return;
        };
        let index = self.choices[slot as usize];
        let need_name = self
            .catalog
            .as_ref()
            .is_some_and(|c| matches!(c.mode, catalog::Mode::Actions(0)))
            && pressed & crate::native_input::MAPPER_A != 0;
        let name = if need_name {
            self.choice(presets, slot, index)
                .map(|c| c.name.to_owned())
                .unwrap_or_default()
        } else {
            String::new()
        };
        let assist = self.mode_editor_origin.is_some();
        let c = self.catalog.get_or_insert_with(Default::default);
        c.assist = assist;
        let event = c.input(buttons, pressed, &name);
        if matches!(event, catalog::Event::None) {
            return;
        }
        let Some(choice) = self.choice(presets, slot, index) else {
            return;
        };
        if (choice.sequence.builtin.is_some() || choice.id.starts_with("__mode_"))
            && matches!(event, catalog::Event::Rename(_) | catalog::Event::Delete)
        {
            self.save_status =
                Some("Built-in actions are permanent; select a recipe to rename/delete".into());
            self.catalog.as_mut().unwrap().close();
            return;
        }
        let draft = catalog::Draft {
            id: choice.id.into(),
            name: choice.name.into(),
            sequence: choice.sequence,
            new: choice.allow_new,
            deleted: false,
            dirty: false,
        };
        match event {
            catalog::Event::None => {}
            catalog::Event::MakeDefault if self.mode_editor_origin.is_some() => {
                self.save_status =
                    Some("Assist rules use priority; turn the mode ON in Assist modes".into());
                self.catalog.as_mut().unwrap().close();
            }
            catalog::Event::MakeDefault => {
                self.default_requested = true;
                self.save_requested = true;
                self.save_status = Some("Save as startup default requested".into());
                self.changed();
            }
            catalog::Event::Rename(name) => {
                let c = self.catalog.as_mut().unwrap();
                let changed = c.drafts.entry((slot as usize, index)).or_insert(draft);
                changed.name = name;
                changed.dirty = true;
                self.recipe_revision = self.recipe_revision.wrapping_add(1);
                self.save_status = Some("Renamed in memory — Y: Save recipe".into());
                self.open_editor(presets, slot);
            }
            catalog::Event::Add(copy) => {
                if self
                    .mode_editor_origin
                    .as_ref()
                    .is_some_and(|(row, _)| self.auto_mode_events(*row).is_some())
                {
                    self.save_status=Some("One recipe per event. Use + Add missing event; use Copy recipe inside the editor for templates".into());
                    return;
                }

                if let Some((row, _)) = self.mode_editor_origin.as_ref() {
                    if self.mode_entries(presets, *row).len()
                        >= crate::reaction_rules::MAX_RULES_PER_MODE
                    {
                        self.save_status = Some(
                            "Eight rules in this assist; delete one before adding another".into(),
                        );
                        return;
                    }
                }
                let user_choices = self
                    .visible_choices(presets, slot)
                    .into_iter()
                    .filter(|i| {
                        self.choice(presets, slot, *i).is_some_and(|c| {
                            c.sequence.builtin.is_none() && !c.id.starts_with("__mode_")
                        })
                    })
                    .count();
                if user_choices > slot.max_choices() {
                    self.save_status = Some(format!(
                        "Maximum {} saved options per row; delete one first",
                        slot.max_choices()
                    ));
                    return;
                }
                let base = presets.list(slot).options.len() + slot.templates();
                let next = self
                    .catalog
                    .as_ref()
                    .unwrap()
                    .drafts
                    .keys()
                    .filter(|(s, _)| *s == slot as usize)
                    .map(|(_, i)| i + 1)
                    .max()
                    .unwrap_or(base)
                    .max(base);
                if next >= slot.max_choices() * 2 {
                    self.save_status = Some("Save and restart before adding more options".into());
                    return;
                }
                let mut serial = next;
                let id = loop {
                    let id = format!("menu_added_{serial}");
                    if !presets.list(slot).options.iter().any(|c| c.id == id)
                        && !self
                            .catalog
                            .as_ref()
                            .unwrap()
                            .drafts
                            .iter()
                            .any(|((s, _), d)| *s == slot as usize && d.id == id)
                    {
                        break id;
                    }
                    serial += 1;
                };
                let mut source = if copy {
                    self.editor
                        .as_ref()
                        .and_then(|e| e.compiled((slot as usize, index)))
                        .unwrap_or(draft.sequence)
                } else {
                    editor::RecipeEditor::custom()
                };
                if source.builtin.is_some() {
                    source = editor::RecipeEditor::custom();
                }
                if let Some((row, _)) = self.mode_editor_origin.as_ref() {
                    if !copy {
                        source.reaction = crate::reaction_rules::Rule {
                            trigger: Self::mode_trigger(*row).unwrap(),
                            state: crate::reaction_rules::SelfState::Grounded,
                            ..Default::default()
                        };
                    }
                }
                let name = if copy {
                    format!(
                        "{} copy",
                        draft
                            .name
                            .chars()
                            .filter(|c| c.is_ascii())
                            .take(55)
                            .collect::<String>()
                    )
                } else {
                    format!("New option {}", serial + 1)
                };
                self.catalog.as_mut().unwrap().drafts.insert(
                    (slot as usize, next),
                    catalog::Draft {
                        id,
                        name,
                        sequence: source,
                        new: true,
                        deleted: false,
                        dirty: true,
                    },
                );
                self.choices[slot as usize] = next;
                self.recipe_revision = self.recipe_revision.wrapping_add(1);
                self.save_status =
                    Some("New option in memory — configure actions, then Y: Save".into());
                self.open_editor(presets, slot);
            }
            catalog::Event::Delete => {
                if draft.id == "menu_custom" && draft.new {
                    self.save_status =
                        Some("Custom is a template; choose a saved option to delete".into());
                } else {
                    self.delete_requested = true;
                    self.save_status = Some("Deleting option...".into());
                }
            }
        }
    }
    pub(super) fn open_editor(&mut self, presets: &MenuPresets, slot: PresetSlot) {
        self.mode_rule_list = false;
        let index = self.choices[slot as usize];
        if let Some(source) = self.choice(presets, slot, index).map(|c| c.sequence) {
            if source.mode.is_none() && source.builtin.is_none() {
                self.editor
                    .get_or_insert_with(|| Box::new(editor::RecipeEditor::default()))
                    .ensure((slot as usize, index), source);
            }
            self.recipe_editing = true;
        }
    }
}
