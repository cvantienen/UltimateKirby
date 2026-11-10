//! Stable choice indices: menu drafts never move another recipe's edits.
use super::*;
use crate::button_bindings::Builtin;
use crate::fighter_profiles::presets::SavedChoice;
use crate::sequence_config::{Action, Sequence};

/// Technical: Defines the `Choice` data structure.
///
/// Plain English: Holds the data for Choice.
pub struct Choice<'a> {
    /// Technical: Stores the `id` value in `Choice` as `&'a str`.
    ///
    /// Plain English: Holds the id value used by Choice.
    pub id: &'a str,
    /// Technical: Stores the `name` value in `Choice` as `&'a str`.
    ///
    /// Plain English: Holds the name value used by Choice.
    pub name: &'a str,
    /// Technical: Stores the `sequence` value in `Choice` as `Sequence`.
    ///
    /// Plain English: Holds the sequence value used by Choice.
    pub sequence: Sequence,
    /// Technical: Stores the `allow_new` value in `Choice` as `bool`.
    ///
    /// Plain English: Holds the allow new value used by Choice.
    pub allow_new: bool,
}
/// Identity of a menu choice. Resolving it never copies a timeline, so the
/// per-poll binding paths and list scans can ask about many choices cheaply.
/// Plain English: Identity of a menu choice. Resolving it never copies a timeline, so the per-poll binding paths and list scans can ask about many choices cheaply.
pub(super) enum Source<'a> {
    /// Technical: Represents the `Draft` case of `Source`.
    ///
    /// Plain English: Represents the Draft option in Source.
    Draft(
        /// Technical: Carries the draft choice source in `Source`.
        ///
        /// Plain English: Holds the draft choice source for this `Source` choice.
        &'a catalog::Draft,
    ),
    /// Technical: Represents the `Saved` case of `Source`.
    ///
    /// Plain English: Represents the Saved option in Source.
    Saved(
        /// Technical: Carries the saved choice source in `Source`.
        ///
        /// Plain English: Holds the saved choice source for this `Source` choice.
        &'a SavedChoice,
    ),
    /// Technical: Represents the `Builtin` case of `Source`.
    ///
    /// Plain English: Represents the Builtin option in Source.
    Builtin(
        /// Technical: Carries the builtin choice source in `Source`.
        ///
        /// Plain English: Holds the builtin choice source for this `Source` choice.
        Builtin,
    ),
    /// Technical: Represents the `Custom` case of `Source`.
    ///
    /// Plain English: Represents the Custom option in Source.
    Custom,
}
impl<'a> Source<'a> {
    /// Technical: Returns the native identifier of the observed object.
    ///
    /// Plain English: Gets the object’s identifier.
    pub fn id(&self) -> &'a str {
        match self {
            Self::Draft(d) => &d.id,
            Self::Saved(c) => c.id,
            Self::Builtin(b) => b.choice_id(),
            Self::Custom => "menu_custom",
        }
    }
    /// Technical: Returns the display name associated with the value.
    ///
    /// Plain English: Gets the name shown for this value.
    pub fn name(&self) -> &'a str {
        match self {
            Self::Draft(d) => &d.name,
            Self::Saved(c) => c.name,
            Self::Builtin(b) => b.label(),
            Self::Custom => "Custom (6 slots)",
        }
    }
    /// Technical: Determines whether the `allow_new` condition holds.
    ///
    /// Plain English: Checks whether allow new.
    pub fn allow_new(&self) -> bool {
        match self {
            Self::Draft(d) => d.new,
            Self::Saved(_) => false,
            Self::Builtin(_) | Self::Custom => true,
        }
    }
    /// Technical: Returns an optional `builtin` value.
    ///
    /// Plain English: Gets builtin if it is available.
    pub fn builtin(&self) -> Option<Builtin> {
        match self {
            Self::Draft(d) => d.sequence.builtin,
            Self::Saved(c) => c.sequence.builtin,
            Self::Builtin(b) => Some(*b),
            Self::Custom => None,
        }
    }
    /// Technical: Returns the `sequence` result.
    ///
    /// Plain English: Gets the sequence result.
    pub fn sequence(&self) -> Sequence {
        match self {
            Self::Draft(d) => d.sequence,
            Self::Saved(c) => *c.sequence,
            Self::Builtin(b) => Sequence {
                builtin: Some(*b),
                ..Sequence::default()
            },
            Self::Custom => editor::RecipeEditor::custom(),
        }
    }
}
/// A choice without any executable step: nothing to run, test or save as a recipe.
/// Plain English: A choice without any executable step: nothing to run, test or save as a recipe.
pub(super) fn is_blank(sequence: &Sequence) -> bool {
    sequence.actions().iter().all(|a| *a == Action::Empty)
}
/// What the catalog needs from every reaction rule, accumulated in one scan.
/// Plain English: What the catalog needs from every reaction rule, accumulated in one scan.
#[derive(Default)]
struct RuleScan {
    /// Technical: Stores the `simplified` value in `RuleScan` as `bool`.
    ///
    /// Plain English: Holds the simplified value used by RuleScan.
    simplified: bool,
    /// Technical: Stores the `has_reactions` value in `RuleScan` as `bool`.
    ///
    /// Plain English: Holds the has reactions value used by RuleScan.
    has_reactions: bool,
    /// Technical: Stores the `auto_grab` value in `RuleScan` as `u8`.
    ///
    /// Plain English: Holds the auto grab value used by RuleScan.
    auto_grab: u8,
    /// Technical: Stores the `buttons` value in `RuleScan` as `u32`.
    ///
    /// Plain English: Holds the buttons value used by RuleScan.
    buttons: u32,
    /// Technical: Stores the `controls` value in `RuleScan` as `u8`.
    ///
    /// Plain English: Holds the controls value used by RuleScan.
    controls: u8,
}
impl RuleScan {
    /// Technical: Updates the `add` state using the supplied inputs.
    ///
    /// Plain English: Updates add.
    fn add(&mut self, rule: crate::reaction_rules::Rule) {
        use crate::reaction_rules::Trigger;
        if self.simplified && rule.trigger.simple().is_none() {
            return;
        }
        self.auto_grab |= match rule.trigger {
            Trigger::AutoGrab => 1,
            Trigger::AutoShieldGrab => 2,
            _ => 0,
        };
        self.has_reactions |= rule.active();
        if rule.active() && !rule.trigger.defensive() {
            self.buttons |= rule.control.button();
            self.controls |= rule.control.bit();
        }
    }
}
impl MenuSelection {
    /// Build a fresh session from the owned saved catalog. Only differences
    /// from startup need owned records; unchanged file choices remain borrowed.
    /// Never calls the startup static-reference publisher (which would leak).
    /// Plain English: Build a fresh session from the owned saved catalog. Only differences from startup need owned records; unchanged file choices remain borrowed. Never calls the startup static-reference publisher (which would leak).
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
                            deleted: true,
                            ..Default::default()
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
                            ..Default::default()
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
        self.bump_recipes();
        self.view_changed();
    }

    /// Resolve at most eight tagged recipes only after configuration revisions.
    /// Plain English: Resolve at most eight tagged recipes only after configuration revisions.
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
                    || is_blank(&sequence)
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
    /// Plain English: One bounded profile scan on creation/recipe revisions, never idle polls.
    pub(super) fn refresh_rule_bindings(&mut self, presets: &MenuPresets, held: u32) {
        let mut scan = RuleScan {
            simplified: self.simplified_reactions,
            ..RuleScan::default()
        };
        if self.catalog.is_none() && self.editor.is_none() {
            // Normal startup path: borrow only small rule metadata. No choice
            // Vec, strings, full timeline copies or catalog rebuilding.
            for choice in presets.lists.iter().flat_map(|list| list.options) {
                scan.add(choice.sequence.reaction);
            }
        } else {
            for slot in PresetSlot::ALL {
                for index in self.all_choices(presets, slot) {
                    let Some(sequence) = self.recipe_snapshot(presets, slot, index) else {
                        continue;
                    };
                    if crate::reaction_rules::validate_response(sequence).is_ok() {
                        scan.add(sequence.reaction);
                    }
                }
            }
        }
        self.has_reactions = scan.has_reactions;
        self.auto_grab_configured = scan.auto_grab;
        let newly_reserved = scan.buttons & !self.rule_button_mask;
        self.trigger_ready &= !newly_reserved;
        self.button_latch |= held & newly_reserved;
        self.rule_button_mask = scan.buttons;
        self.rule_toggles &= scan.controls;
    }
    /// Technical: Returns the `reaction_enabled_controls` result.
    ///
    /// Plain English: Gets the reaction enabled controls result.
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
    /// Technical: Returns an optional `draft` value.
    ///
    /// Plain English: Gets draft if it is available.
    pub(super) fn draft(&self, slot: PresetSlot, index: usize) -> Option<&catalog::Draft> {
        self.catalog.as_ref()?.drafts.get(&(slot as usize, index))
    }
    /// Technical: Returns mutable access to the catalog value.
    ///
    /// Plain English: Provides mutable access to catalog.
    pub(super) fn catalog_mut(&mut self) -> &mut catalog::Catalog {
        self.catalog.get_or_insert_with(Default::default)
    }
    /// Does a live recipe for this slot already provide the built-in action?
    /// Plain English: Does a live recipe for this slot already provide the built-in action?
    fn builtin_recorded(&self, slot: PresetSlot, builtin: Builtin, options: &[SavedChoice]) -> bool {
        options.iter().any(|c| c.sequence.builtin == Some(builtin))
            || self.catalog.as_ref().is_some_and(|c| {
                c.drafts.iter().any(|((s, _), d)| {
                    *s == slot as usize && !d.deleted && d.sequence.builtin == Some(builtin)
                })
            })
    }
    /// Technical: Returns an optional `source` value.
    ///
    /// Plain English: Gets source if it is available.
    pub(super) fn source<'a>(
        &'a self,
        presets: &'a MenuPresets,
        slot: PresetSlot,
        index: usize,
    ) -> Option<Source<'a>> {
        if let Some(d) = self.draft(slot, index) {
            return d.live().then_some(Source::Draft(d));
        }
        let options = presets.list(slot).options;
        if let Some(c) = options.get(index) {
            return c.sequence.mode.is_none().then_some(Source::Saved(c));
        }
        if slot.is_button() {
            let template = Builtin::ALL
                .into_iter()
                .find(|b| index == options.len() + b.template_offset());
            if let Some(builtin) = template {
                // A built-in template disappears once a recipe provides it,
                // unless it is the current selection.
                let hidden = self.choices[slot as usize] != index
                    && self.builtin_recorded(slot, builtin, options);
                return (!hidden).then_some(Source::Builtin(builtin));
            }
        }
        (index == options.len() + usize::from(slot.is_button())).then_some(Source::Custom)
    }
    /// Resolve only identity for the hot input path; do not copy 32 actions.
    /// Plain English: Resolve only identity for the hot input path; do not copy 32 actions.
    pub fn choice_labels<'a>(
        &'a self,
        presets: &'a MenuPresets,
        slot: PresetSlot,
        index: usize,
    ) -> Option<(&'a str, &'a str)> {
        self.source(presets, slot, index)
            .map(|source| (source.id(), source.name()))
    }

    /// Technical: Returns an optional `choice` value.
    ///
    /// Plain English: Gets choice if it is available.
    pub fn choice<'a>(
        &'a self,
        presets: &'a MenuPresets,
        slot: PresetSlot,
        index: usize,
    ) -> Option<Choice<'a>> {
        self.source(presets, slot, index).map(|source| Choice {
            id: source.id(),
            name: source.name(),
            sequence: source.sequence(),
            allow_new: source.allow_new(),
        })
    }
    /// Technical: Determines whether the `managing` condition holds.
    ///
    /// Plain English: Checks whether managing.
    pub fn managing(&self) -> bool {
        self.catalog.as_ref().is_some_and(|c| c.active())
    }
    /// One past the highest choice index of a slot, templates and added drafts included.
    /// Plain English: One past the highest choice index of a slot, templates and added drafts included.
    pub(super) fn choice_end(&self, presets: &MenuPresets, slot: PresetSlot) -> usize {
        let base = presets.list(slot).options.len() + slot.templates();
        let slot_key = slot as usize;
        self.catalog
            .as_ref()
            .and_then(|c| c.drafts.range((slot_key, 0)..(slot_key + 1, 0)).next_back())
            .map_or(base, |((_, index), _)| base.max(index + 1))
    }
    /// Technical: Returns the `all_choices` result.
    ///
    /// Plain English: Gets the all choices result.
    pub(super) fn all_choices(&self, presets: &MenuPresets, slot: PresetSlot) -> Vec<usize> {
        (0..self.choice_end(presets, slot))
            .filter(|i| self.source(presets, slot, *i).is_some())
            .collect()
    }
    /// Technical: Returns an optional `recipe_snapshot` value.
    ///
    /// Plain English: Gets recipe snapshot if it is available.
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
        self.source(presets, slot, index).map(|s| s.sequence())
    }
    /// Technical: Returns the `visible_choices` result.
    ///
    /// Plain English: Gets the visible choices result.
    pub(super) fn visible_choices(&self, presets: &MenuPresets, slot: PresetSlot) -> Vec<usize> {
        self.all_choices(presets, slot)
            .into_iter()
            .filter(|index| {
                self.recipe_snapshot(presets, slot, *index)
                    .is_some_and(|s| !s.reaction.active())
            })
            .collect()
    }
    /// Technical: Returns the `template_sources` result.
    ///
    /// Plain English: Gets the template sources result.
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
                let Some(source) = self.source(presets, slot, index) else {
                    continue;
                };
                if source.id() == Builtin::Normal.choice_id() {
                    continue;
                }
                let sequence = self
                    .editor
                    .as_ref()
                    .and_then(|e| e.compiled((slot as usize, index)))
                    .unwrap_or_else(|| source.sequence());
                if sequence.mode.is_none() && !is_blank(&sequence) {
                    sources.push(editor::CopySource {
                        slot,
                        index,
                        name: source.name().to_owned(),
                    });
                }
            }
        }
        sources
    }
    /// Technical: Determines whether the `quick_choice` condition holds.
    ///
    /// Plain English: Checks whether quick choice.
    fn quick_choice(&self, presets: &MenuPresets, slot: PresetSlot, index: usize) -> bool {
        self.source(presets, slot, index).is_some_and(|source| {
            source.builtin().is_some()
                || !source.allow_new()
                || self.draft(slot, index).is_some()
                || self
                    .editor
                    .as_ref()
                    .and_then(|e| e.compiled((slot as usize, index)))
                    .is_some_and(|s| s.len > 0)
        })
    }
    /// Choices the compact page cycles: the unedited blank template stays hidden.
    /// Plain English: Choices the compact page cycles: the unedited blank template stays hidden.
    fn quick_choices(&self, presets: &MenuPresets, slot: PresetSlot) -> Vec<usize> {
        let mut visible = self.visible_choices(presets, slot);
        visible.retain(|i| self.quick_choice(presets, slot, *i));
        visible
    }
    /// Technical: Returns the `choice_position` result.
    ///
    /// Plain English: Gets the choice position result.
    pub fn choice_position(&self, presets: &MenuPresets, slot: PresetSlot) -> (usize, usize) {
        let visible = if self.quick_menu {
            self.quick_choices(presets, slot)
        } else {
            self.visible_choices(presets, slot)
        };
        (
            visible
                .iter()
                .position(|i| *i == self.choices[slot as usize])
                .unwrap_or(visible.len()),
            visible.len(),
        )
    }
    /// Technical: Returns the `cycle_choice` result.
    ///
    /// Plain English: Gets the cycle choice result.
    pub(super) fn cycle_choice(
        &self,
        presets: &MenuPresets,
        slot: PresetSlot,
        forward: bool,
    ) -> usize {
        let visible = self.visible_choices(presets, slot);
        Self::cycle_visible_choices(&visible, self.choices[slot as usize], forward, 0)
    }
    /// Technical: Returns the `cycle_quick_choice` result.
    ///
    /// Plain English: Gets the cycle quick choice result.
    pub(super) fn cycle_quick_choice(
        &self,
        presets: &MenuPresets,
        slot: PresetSlot,
        forward: bool,
    ) -> usize {
        let visible = self.quick_choices(presets, slot);
        if visible.is_empty() {
            return self.choices[slot as usize];
        }
        let missing_index = if forward { visible.len() - 1 } else { 0 };
        Self::cycle_visible_choices(
            &visible,
            self.choices[slot as usize],
            forward,
            missing_index,
        )
    }

    /// Technical: Returns the `cycle_visible_choices` result.
    ///
    /// Plain English: Gets the cycle visible choices result.
    fn cycle_visible_choices(
        visible: &[usize],
        current: usize,
        forward: bool,
        missing_index: usize,
    ) -> usize {
        let index = visible
            .iter()
            .position(|choice| *choice == current)
            .unwrap_or(missing_index);
        visible[cycle(index, visible.len(), forward)]
    }
    /// Technical: Returns an optional `name_override` value.
    ///
    /// Plain English: Gets name override if it is available.
    pub fn name_override(&self, slot: PresetSlot, index: usize) -> Option<&str> {
        self.draft(slot, index).map(|d| d.name.as_str())
    }
    /// Editing a different recipe must not make a completed snapshot look stale.
    /// Plain English: Editing a different recipe must not make a completed snapshot look stale.
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
                self.source(presets, slot, index)?
                    .builtin()
                    .map(|builtin| builtin.label())
            }) == Some(name)
        } else {
            // No explicit rename preserves the existing file name. A promoted
            // clean baseline may now own that label without being a newer edit.
            !self.draft(slot, index).is_some_and(|draft| draft.dirty)
        };
        current == Some(sequence) && name_matches
    }
    /// Technical: Marks the recipe saved.
    ///
    /// Plain English: Marks the recipe saved.
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
    /// Technical: Updates the `complete_file_save` state using the supplied inputs.
    ///
    /// Plain English: Updates complete file save.
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
    /// Plain English: Promote the acknowledged worker snapshot, never a newer unsaved edit or the immutable startup recipe. Completion can arrive after leaving Training.
    pub fn complete_saved_snapshot(
        &mut self,
        presets: &MenuPresets,
        slot: PresetSlot,
        index: usize,
        deleted: bool,
        snapshot: Option<Sequence>,
        name: Option<&str>,
    ) {
        let Some(source) = self.source(presets, slot, index) else {
            return;
        };
        let original = catalog::Draft {
            id: source.id().into(),
            name: source.name().into(),
            sequence: source.sequence(),
            deleted,
            ..Default::default()
        };
        let draft = self
            .catalog_mut()
            .drafts
            .entry((slot as usize, index))
            .or_insert(original);
        draft.new = false;
        draft.deleted = deleted;
        if deleted {
            self.complete_deletion(presets, slot, index);
            return;
        }
        if let Some(sequence) = snapshot {
            draft.sequence = sequence;
        }
        if !draft.dirty {
            if let Some(name) = name {
                draft.name = name.into();
            }
        }
        // Binding/rule/settings caches must see the committed base even when
        // maintenance consumes completion before input.
        self.bump_recipes();
    }
    /// Technical: Updates the `complete_deletion` state using the supplied inputs.
    ///
    /// Plain English: Updates complete deletion.
    fn complete_deletion(&mut self, presets: &MenuPresets, slot: PresetSlot, index: usize) {
        let key = (slot as usize, index);
        // A Custom saved at the template's index becomes a blank reusable
        // template after deletion, not a hidden last row.
        if index == presets.list(slot).options.len() + usize::from(slot.is_button()) {
            self.catalog_mut().drafts.remove(&key);
            if let Some(editor) = self.editor.as_mut() {
                editor.discard(key);
            }
        }
        let was_selected = self.choices[slot as usize] == index;
        if was_selected {
            let visible = self.visible_choices(presets, slot);
            self.choices[slot as usize] = visible
                .iter()
                .copied()
                .find(|i| *i > index)
                .or_else(|| visible.last().copied())
                .unwrap_or(presets.list(slot).options.len());
        }
        self.bump_recipes();
        if was_selected && self.row.slot() == Some(slot) {
            self.recipe_editing = false;
            self.mode_rule_list = self.mode_editor_origin.is_some();
        }
        self.preview_slot = None;
    }
    /// Drop Training-only tools once per scene transition. Retain unfinished
    /// drafts (including invalid drafts) and any clean editor differing from its
    /// base; a late save acknowledgement will promote that base first.
    /// Plain English: Drop Training-only tools once per scene transition. Retain unfinished drafts (including invalid drafts) and any clean editor differing from its base; a late save acknowledgement will promote that base first.
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
                    .and_then(|slot| self.source(presets, *slot, key.1))
                    .map(|source| source.sequence())
                    != sequence
        });
        if !editor.is_empty() {
            self.editor = Some(editor);
        }
    }
    /// Technical: Updates the `manage_input` state using the supplied inputs.
    ///
    /// Plain English: Updates manage input.
    pub(super) fn manage_input(&mut self, presets: &MenuPresets, buttons: u32, pressed: u32) {
        let Some(slot) = self.row.slot() else {
            return;
        };
        let index = self.choices[slot as usize];
        let rename_seed = self
            .catalog
            .as_ref()
            .is_some_and(|c| matches!(c.mode, catalog::Mode::Actions(0)))
            && pressed & MAPPER_A != 0;
        let name = if rename_seed {
            self.source(presets, slot, index)
                .map(|s| s.name().to_owned())
                .unwrap_or_default()
        } else {
            String::new()
        };
        let assist = self.mode_editor_origin.is_some();
        let c = self.catalog_mut();
        c.assist = assist;
        let event = c.input(buttons, pressed, &name);
        if matches!(event, catalog::Event::None) {
            return;
        }
        let Some(source) = self.source(presets, slot, index) else {
            return;
        };
        let permanent = source.builtin().is_some();
        let draft = catalog::Draft {
            id: source.id().into(),
            name: source.name().into(),
            sequence: source.sequence(),
            new: source.allow_new(),
            ..Default::default()
        };
        if permanent && matches!(event, catalog::Event::Rename(_) | catalog::Event::Delete) {
            self.save_status =
                Some("Built-in actions are permanent; select a recipe to rename/delete".into());
            self.catalog_mut().close();
            return;
        }
        match event {
            catalog::Event::None => {}
            catalog::Event::MakeDefault if self.mode_editor_origin.is_some() => {
                self.save_status =
                    Some("Assist rules use priority; turn the mode ON in Assist modes".into());
                self.catalog_mut().close();
            }
            catalog::Event::MakeDefault => {
                self.default_requested = true;
                self.save_requested = true;
                self.save_status = Some("Save as startup default requested".into());
                self.changed();
            }
            catalog::Event::Rename(name) => {
                let renamed = self
                    .catalog_mut()
                    .drafts
                    .entry((slot as usize, index))
                    .or_insert(draft);
                renamed.name = name;
                renamed.dirty = true;
                self.bump_recipes();
                self.save_status = Some("Renamed in memory — Y: Save recipe".into());
                self.open_editor(presets, slot);
            }
            catalog::Event::Add(copy) => self.add_choice(presets, slot, index, &draft, copy),
            catalog::Event::Delete => {
                if draft.id == Source::Custom.id() && draft.new {
                    self.save_status =
                        Some("Custom is a template; choose a saved option to delete".into());
                } else {
                    self.delete_requested = true;
                    self.save_status = Some("Deleting option...".into());
                }
            }
        }
    }
    /// Why another option cannot be added to this row, if it cannot.
    /// Plain English: Why another option cannot be added to this row, if it cannot.
    fn add_refusal(&self, presets: &MenuPresets, slot: PresetSlot) -> Option<String> {
        if let Some((row, _)) = self.mode_editor_origin.as_ref() {
            if self.auto_mode_events(*row).is_some() {
                return Some("One recipe per event. Use + Add missing event; use Copy recipe inside the editor for templates".into());
            }
            if self.mode_entries(presets, *row).len() >= crate::reaction_rules::MAX_RULES_PER_MODE {
                return Some("Eight rules in this assist; delete one before adding another".into());
            }
        }
        let user_choices = self
            .visible_choices(presets, slot)
            .into_iter()
            .filter(|i| {
                self.source(presets, slot, *i)
                    .is_some_and(|s| s.builtin().is_none())
            })
            .count();
        if user_choices > slot.max_choices() {
            return Some(format!(
                "Maximum {} saved options per row; delete one first",
                slot.max_choices()
            ));
        }
        (self.choice_end(presets, slot) >= slot.max_choices() * 2)
            .then(|| "Save and restart before adding more options".into())
    }
    /// First unused `menu_added_N` id at or after `serial`, with the serial it used.
    /// Plain English: First unused menuaddedN id at or after serial, with the serial it used.
    fn fresh_choice_id(&self, presets: &MenuPresets, slot: PresetSlot, mut serial: usize) -> (String, usize) {
        let options = presets.list(slot).options;
        let slot_key = slot as usize;
        loop {
            let id = format!("menu_added_{serial}");
            let taken = options.iter().any(|c| c.id == id)
                || self.catalog.as_ref().is_some_and(|c| {
                    c.drafts
                        .range((slot_key, 0)..(slot_key + 1, 0))
                        .any(|(_, d)| d.id == id)
                });
            if !taken {
                return (id, serial);
            }
            serial += 1;
        }
    }
    /// Technical: Adds the choice.
    ///
    /// Plain English: Adds the choice.
    fn add_choice(
        &mut self,
        presets: &MenuPresets,
        slot: PresetSlot,
        index: usize,
        current: &catalog::Draft,
        copy: bool,
    ) {
        if let Some(refusal) = self.add_refusal(presets, slot) {
            self.save_status = Some(refusal);
            return;
        }
        let next = self.choice_end(presets, slot);
        let (id, serial) = self.fresh_choice_id(presets, slot, next);
        let mut sequence = if copy {
            self.editor
                .as_ref()
                .and_then(|e| e.compiled((slot as usize, index)))
                .unwrap_or(current.sequence)
        } else {
            editor::RecipeEditor::custom()
        };
        if sequence.builtin.is_some() {
            sequence = editor::RecipeEditor::custom();
        }
        if let Some((row, _)) = self.mode_editor_origin.as_ref().filter(|_| !copy) {
            sequence.reaction = crate::reaction_rules::Rule {
                trigger: Self::mode_trigger(*row).unwrap(),
                state: crate::reaction_rules::SelfState::Grounded,
                ..Default::default()
            };
        }
        let name = if copy {
            let base: String = current.name.chars().filter(|c| c.is_ascii()).take(55).collect();
            format!("{base} copy")
        } else {
            format!("New option {}", serial + 1)
        };
        self.catalog_mut().drafts.insert(
            (slot as usize, next),
            catalog::Draft {
                id,
                name,
                sequence,
                new: true,
                dirty: true,
                ..Default::default()
            },
        );
        self.choices[slot as usize] = next;
        self.bump_recipes();
        self.save_status = Some("New option in memory — configure actions, then Y: Save".into());
        self.open_editor(presets, slot);
    }
    /// Technical: Updates the `open_editor` state using the supplied inputs.
    ///
    /// Plain English: Updates open editor.
    pub(super) fn open_editor(&mut self, presets: &MenuPresets, slot: PresetSlot) {
        self.mode_rule_list = false;
        let index = self.choices[slot as usize];
        if let Some(source) = self.source(presets, slot, index).map(|s| s.sequence()) {
            if source.mode.is_none() && source.builtin.is_none() {
                self.editor
                    .get_or_insert_with(|| Box::new(editor::RecipeEditor::default()))
                    .ensure((slot as usize, index), source);
            }
            self.recipe_editing = true;
        }
    }
}
