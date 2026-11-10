//! Input-driven editor tools only; no game queries, files, workers or execution changes.
use super::*;
use crate::ledge_trap::DPAD_DOWN;
use crate::manual_controls::{DPAD_LEFT, DPAD_RIGHT, DPAD_UP};
use crate::settings_menu::RecipePanel;

/// Technical: Defines the available `Modal` cases.
///
/// Plain English: Lists the available Modal choices.
#[derive(Clone, Copy)]
pub(super) enum Modal {
    /// Technical: Represents the `Buttons` case of `Modal`.
    ///
    /// Plain English: Represents the Buttons option in Modal.
    Buttons {
        /// Technical: Stores the `slot` value in `Modal` as `usize`.
        ///
        /// Plain English: Holds the slot value used by Modal.
        slot: usize,
        /// Technical: Stores the `item` value in `Modal` as `usize`.
        ///
        /// Plain English: Holds the item value used by Modal.
        item: usize,
    },
    /// Technical: Represents the `Copy` case of `Modal`.
    ///
    /// Plain English: Represents the Copy option in Modal.
    Copy {
        /// Technical: Stores the `group` value in `Modal` as `usize`.
        ///
        /// Plain English: Holds the group value used by Modal.
        group: usize,
        /// Technical: Stores the `item` value in `Modal` as `usize`.
        ///
        /// Plain English: Holds the item value used by Modal.
        item: usize,
    },
    /// Technical: Represents the `Tools` case of `Modal`.
    ///
    /// Plain English: Represents the Tools option in Modal.
    Tools {
        /// Technical: Stores the `slot` value in `Modal` as `usize`.
        ///
        /// Plain English: Holds the slot value used by Modal.
        slot: usize,
        /// Technical: Stores the `item` value in `Modal` as `usize`.
        ///
        /// Plain English: Holds the item value used by Modal.
        item: usize,
    },
    /// Technical: Represents the `Picker` case of `Modal`.
    ///
    /// Plain English: Represents the Picker option in Modal.
    Picker {
        /// Technical: Stores the `slot` value in `Modal` as `usize`.
        ///
        /// Plain English: Holds the slot value used by Modal.
        slot: usize,
        /// Technical: Stores the `group` value in `Modal` as `usize`.
        ///
        /// Plain English: Holds the group value used by Modal.
        group: usize,
        /// Technical: Stores the `item` value in `Modal` as `usize`.
        ///
        /// Plain English: Holds the item value used by Modal.
        item: usize,
    },
}
/// Technical: Defines the `TOOLS` constant used by this module.
///
/// Plain English: Provides the TOOLS constant value.
const TOOLS: &[&str] = &[
    "Choose action",
    "Insert before",
    "Insert after",
    "Duplicate step",
    "Move up",
    "Move down",
    "Delete step",
    "Undo last edit",
    "Copy recipe / use template...",
    "Copy step",
    "Paste step before",
    "Paste step after",
    "Recipe options (rename/delete)",
];
// Positions in `TOOLS`.
/// Technical: Defines the `CHOOSE_ACTION` constant used by this module.
///
/// Plain English: Provides the CHOOSE ACTION constant value.
const CHOOSE_ACTION: usize = 0;
/// Technical: Defines the `INSERT_BEFORE` constant used by this module.
///
/// Plain English: Provides the INSERT BEFORE constant value.
const INSERT_BEFORE: usize = 1;
/// Technical: Defines the `INSERT_AFTER` constant used by this module.
///
/// Plain English: Provides the INSERT AFTER constant value.
const INSERT_AFTER: usize = 2;
/// Technical: Defines the `DUPLICATE` constant used by this module.
///
/// Plain English: Provides the DUPLICATE constant value.
const DUPLICATE: usize = 3;
/// Technical: Defines the `MOVE_UP` constant used by this module.
///
/// Plain English: Provides the MOVE UP constant value.
const MOVE_UP: usize = 4;
/// Technical: Defines the `MOVE_DOWN` constant used by this module.
///
/// Plain English: Provides the MOVE DOWN constant value.
const MOVE_DOWN: usize = 5;
/// Technical: Defines the `DELETE` constant used by this module.
///
/// Plain English: Provides the DELETE constant value.
const DELETE: usize = 6;
/// Technical: Defines the `UNDO` constant used by this module.
///
/// Plain English: Provides the UNDO constant value.
const UNDO: usize = 7;
/// Technical: Defines the `COPY_RECIPE` constant used by this module.
///
/// Plain English: Provides the COPY RECIPE constant value.
const COPY_RECIPE: usize = 8;
/// Technical: Defines the `COPY_STEP` constant used by this module.
///
/// Plain English: Provides the COPY STEP constant value.
const COPY_STEP: usize = 9;
/// Technical: Defines the `PASTE_BEFORE` constant used by this module.
///
/// Plain English: Provides the PASTE BEFORE constant value.
const PASTE_BEFORE: usize = 10;
/// Technical: Defines the `PASTE_AFTER` constant used by this module.
///
/// Plain English: Provides the PASTE AFTER constant value.
const PASTE_AFTER: usize = 11;
/// Technical: Defines the `RECIPE_OPTIONS` constant used by this module.
///
/// Plain English: Provides the RECIPE OPTIONS constant value.
const RECIPE_OPTIONS: usize = 12;
/// Technical: Defines the `NO_ROOM` constant used by this module.
///
/// Plain English: Provides the NO ROOM constant value.
const NO_ROOM: &str = "All 32 slots are occupied; remove a step first";
/// Technical: Defines the `GROUPS` constant used by this module.
///
/// Plain English: Provides the GROUPS constant value.
const GROUPS: &[&str] = &[
    "Attacks / aerials / tilts",
    "Jumps / jump attacks",
    "Specials",
    "Shield / dodge / grab",
    "Movement / facing",
    "Buttons / combinations",
    "Conditions / checks",
    "Repeated taps",
    "Saved inputs",
];
/// Technical: Defines the `SAVED_INPUTS_GROUP` constant used by this module.
///
/// Plain English: Provides the SAVED INPUTS GROUP constant value.
const SAVED_INPUTS_GROUP: usize = GROUPS.len() - 1;
/// Technical: Defines the `COPY_GROUPS` constant used by this module.
///
/// Plain English: Provides the COPY GROUPS constant value.
const COPY_GROUPS: &[&str] = &[
    "Neutral getup",
    "Ledge jump",
    "Ledge roll",
    "Getup attack",
    "Ledge drop",
    "X button",
    "Y button",
    "D-pad Up",
    "D-pad Down",
    "D-pad Left",
    "D-pad Right",
    "Generic templates",
];
/// Technical: Defines the `TEMPLATE_GROUP` constant used by this module.
///
/// Plain English: Provides the TEMPLATE GROUP constant value.
const TEMPLATE_GROUP: usize = COPY_GROUPS.len() - 1;
/// Technical: Defines the `DPAD` constant used by this module.
///
/// Plain English: Provides the DPAD constant value.
const DPAD: u32 = DPAD_UP | DPAD_DOWN | DPAD_LEFT | DPAD_RIGHT;
/// Picker category of a preset action.
/// Plain English: Picker category of a preset action.
fn group(action: Action) -> usize {
    match action {
        Action::Check { .. }
        | Action::Range(_)
        | Action::ShieldUntilBlock(_)
        | Action::IfFacingLedge { .. }
        | Action::IfFacingOpponent { .. }
        | Action::ElseFacing
        | Action::EndFacing => 6,
        Action::ShieldUntilManual => 3,
        Action::Tap { .. } => 7,
        Action::Controller { .. } => 5,
        Action::Face(_)
        | Action::Walk { .. }
        | Action::Run { .. }
        | Action::LedgePosition { .. }
        | Action::Approach { .. }
        | Action::Drift { .. } => 4,
        Action::Input { buttons, .. } | Action::OpponentInput { buttons, .. } => {
            if buttons & MAPPER_X != 0 {
                1
            } else if buttons == MAPPER_B {
                2
            } else if buttons & (MAPPER_R | MAPPER_ZR) != 0 {
                3
            } else if buttons == 0 {
                4
            } else if buttons == MAPPER_A {
                0
            } else {
                5
            }
        }
        Action::CStick { .. } => 0,
        _ => 5,
    }
}
/// Technical: Returns the `candidates` result.
///
/// Plain English: Gets the candidates result.
pub(super) fn candidates(r: &Recipe, category: usize) -> Vec<usize> {
    let ledge_free = |action: Action| !r.sequence.reaction.active() || !action.needs_ledge();
    if category == SAVED_INPUTS_GROUP {
        return (PRESETS.len()..PRESETS.len() + r.imported.len())
            .filter(|kind| ledge_free(r.action_for(*kind)))
            .collect();
    }
    PRESETS
        .iter()
        .enumerate()
        .filter_map(|(i, (_, action))| (group(*action) == category && ledge_free(*action)).then_some(i))
        .collect()
}
/// Technical: Returns the `changed_steps` result.
///
/// Plain English: Gets the changed steps result.
fn changed_steps(before: &Recipe, after: &Recipe) -> u32 {
    (0..usize::from(after.sequence.len)).fold(0, |mask, i| {
        if before.sequence.actions[i] != after.sequence.actions[i]
            || before.before[i] != after.before[i]
            || before.kinds[i] != after.kinds[i]
        {
            mask | (1u32 << i)
        } else {
            mask
        }
    })
}
/// Technical: Determines whether the `vertical` condition holds.
///
/// Plain English: Checks whether vertical.
fn vertical(pad: u32) -> bool {
    pad == DPAD_UP || pad == DPAD_DOWN
}
/// Move `item` through `count` entries on Up/Down, wrapping at both ends.
/// Plain English: Move item through count entries on Up/Down, wrapping at both ends.
fn step_item(item: usize, count: usize, nav: u32) -> usize {
    if nav & DPAD_DOWN != 0 {
        cycle(item, count, true)
    } else if nav & DPAD_UP != 0 {
        cycle(item, count, false)
    } else {
        item
    }
}
/// The next group in the given direction that has entries; stays put when none do.
/// Plain English: The next group in the given direction that has entries; stays put when none do.
fn next_group(
    group: usize,
    count: usize,
    forward: bool,
    has_entries: impl Fn(usize) -> bool,
) -> usize {
    let mut next = group;
    for _ in 0..count {
        next = cycle(next, count, forward);
        if has_entries(next) {
            return next;
        }
    }
    group
}
/// Replacing a move keeps its configured hold. Until-takeover (0 frames)
/// transfers only to another button input.
/// Plain English: Replacing a move keeps its configured hold. Until-takeover (0 frames) transfers only to another button input.
fn carry_hold(old: Action, new: &mut Action) {
    let held = match old {
        Action::Input { frames, .. }
        | Action::OpponentInput { frames, .. }
        | Action::Controller { frames, .. } => frames,
        Action::ShieldUntilManual => 0,
        _ => return,
    };
    match new {
        Action::Input {
            buttons, frames, ..
        } => *frames = if held == 0 && *buttons == 0 { 4 } else { held },
        Action::OpponentInput { frames, .. } | Action::Controller { frames, .. } => {
            *frames = if held == 0 { 4 } else { held }
        }
        _ => {}
    }
}
impl RecipeEditor {
    /// Technical: Updates the `close_commands` state using the supplied inputs.
    ///
    /// Plain English: Updates close commands.
    pub fn close_commands(&mut self) {
        self.modal = None;
        self.return_modal = None;
        self.manage_requested = false;
        self.command_held = 0;
        self.command_ticks = 0;
        self.clear_copy();
    }
    /// Technical: Clears the copy.
    ///
    /// Plain English: Clears the copy.
    pub fn clear_copy(&mut self) {
        self.copy_sources = Vec::new();
        self.copy_templates = Vec::new();
        self.template_apply = None;
        self.copy_requested = false;
        self.copy_apply = None;
    }
    /// Technical: Takes the manage request.
    ///
    /// Plain English: Takes the manage request.
    pub fn take_manage_request(&mut self) -> bool {
        std::mem::take(&mut self.manage_requested)
    }
    /// Technical: Copies the step.
    ///
    /// Plain English: Copies the step.
    fn copy_step(&mut self, key: (usize, usize), slot: usize) -> bool {
        let Some(r) = self.recipes.get(&key) else {
            return false;
        };
        let action = r.sequence.actions[slot];
        let delay = r.before[slot];
        if action == Action::Empty && delay == 0 {
            self.last_edit = Some("No action or delay to copy".into());
            return false;
        }
        self.step_clipboard = Some((action, delay));
        self.last_edit = Some("Step copied; paste before/after in any recipe".into());
        true
    }
    /// Technical: Determines whether the `paste_step` condition holds.
    ///
    /// Plain English: Checks whether paste step.
    fn paste_step(&mut self, key: (usize, usize), slot: usize, after: bool) -> bool {
        let Some((action, delay)) = self.step_clipboard else {
            self.last_edit = Some("Copy a step first".into());
            return false;
        };
        let Some(mut next) = self.recipes.get(&key).cloned() else {
            return false;
        };
        if next.sequence.reaction.active() && action.needs_ledge() {
            self.last_edit = Some("This step needs a ledge; choose a ledge recipe".into());
            return false;
        }
        let at = slot + usize::from(after);
        let kind = next.kind_for(action);
        let reuse_last = at < usize::from(next.sequence.len);
        if !next.insert_step(at, (action, delay, kind), reuse_last) {
            self.limited = true;
            self.last_edit = Some(NO_ROOM.into());
            return false;
        }
        self.commit_command(
            key,
            next,
            at,
            "Pasted step and delay; Undo restores the previous recipe",
        )
    }
    /// Technical: Takes the copy request.
    ///
    /// Plain English: Takes the copy request.
    pub fn take_copy_request(&mut self) -> bool {
        std::mem::take(&mut self.copy_requested)
    }
    /// Technical: Sets the copy sources.
    ///
    /// Plain English: Sets the copy sources.
    pub fn set_copy_sources(&mut self, sources: Vec<CopySource>) {
        self.copy_sources = sources;
        if let Some(Modal::Copy { group, item }) = self.modal.as_mut() {
            if *group != TEMPLATE_GROUP
                && !self.copy_sources.iter().any(|s| s.slot as usize == *group)
            {
                if let Some(first) = self.copy_sources.first() {
                    *group = first.slot as usize;
                    *item = 0;
                }
            }
        }
    }
    /// Technical: Sets the copy templates.
    ///
    /// Plain English: Sets the copy templates.
    pub fn set_copy_templates(&mut self, templates: &[crate::sequence_config::Template]) {
        self.copy_templates = templates.iter().map(|t| t.name.clone()).collect();
        if let Some(Modal::Copy { group, item }) = self.modal.as_mut() {
            if self.copy_sources.is_empty() && !self.copy_templates.is_empty() {
                *group = TEMPLATE_GROUP;
                *item = 0;
            }
        }
    }
    /// Technical: Copies the sources in.
    ///
    /// Plain English: Copies the sources in.
    fn copy_sources_in(&self, group: usize) -> impl Iterator<Item = &CopySource> {
        self.copy_sources
            .iter()
            .filter(move |s| s.slot as usize == group)
    }
    /// Technical: Copies the count.
    ///
    /// Plain English: Copies the count.
    fn copy_count(&self, group: usize) -> usize {
        if group == TEMPLATE_GROUP {
            self.copy_templates.len()
        } else {
            self.copy_sources_in(group).count()
        }
    }
    /// Technical: Takes the template apply.
    ///
    /// Plain English: Takes the template apply.
    pub fn take_template_apply(&mut self) -> Option<usize> {
        self.template_apply.take()
    }
    /// Technical: Applies the generic template.
    ///
    /// Plain English: Applies the generic template.
    pub fn apply_generic_template(
        &mut self,
        destination: (usize, usize),
        sequence: Sequence,
    ) -> bool {
        // This key cannot alias a fighter recipe or its unsaved draft.
        self.apply_template(destination, (usize::MAX, 0), sequence)
    }
    /// Technical: Takes the copy apply.
    ///
    /// Plain English: Takes the copy apply.
    pub fn take_copy_apply(
        &mut self,
    ) -> Option<(crate::fighter_profiles::presets::PresetSlot, usize)> {
        self.copy_apply.take()
    }
    /// Technical: Rejects the copy.
    ///
    /// Plain English: Rejects the copy.
    pub fn reject_copy(&mut self) {
        self.last_edit = Some("Source recipe is no longer available; nothing changed".into());
    }
    /// Technical: Applies the template.
    ///
    /// Plain English: Applies the template.
    pub fn apply_template(
        &mut self,
        destination: (usize, usize),
        source: (usize, usize),
        fallback: Sequence,
    ) -> bool {
        if destination == source {
            return false;
        }
        let Some(target) = self.recipes.get(&destination) else {
            return false;
        };
        let activation = target.sequence.while_held;
        let reaction = target.sequence.reaction;
        let mut next = self
            .recipes
            .get(&source)
            .cloned()
            .unwrap_or_else(|| Recipe::new(fallback));
        next.sequence.reaction = reaction;
        let Some(compiled) = next
            .compile()
            .filter(|s| s.len != 0 && s.mode.is_none())
        else {
            self.reject_copy();
            return false;
        };
        if let Err(error) = crate::reaction_rules::validate_response(compiled) {
            self.last_edit = Some(format!("Cannot copy into this recipe: {error}"));
            return false;
        }
        next.button_recipe = destination.0 >= 5;
        next.sequence.while_held = next.button_recipe && activation;
        if next.sequence == target.sequence && next.before == target.before {
            self.last_edit = Some("Destination already has these actions and timings".into());
            self.dismiss();
            self.clear_copy();
            return false;
        }
        let changed = self.commit_command(
            destination,
            next,
            0,
            "Copied actions and timings; source and destination name unchanged",
        );
        if changed {
            self.dismiss();
            self.clear_copy();
        }
        changed
    }

    /// Technical: Determines whether the `command_active` condition holds.
    ///
    /// Plain English: Checks whether command active.
    pub fn command_active(&self) -> bool {
        self.modal.is_some()
    }
    /// Technical: Returns an optional `focused_field` value.
    ///
    /// Plain English: Gets focused field if it is available.
    fn focused_field(&self, key: (usize, usize)) -> Option<Field> {
        let r = self.recipes.get(&key)?;
        Some(focus(&fields(r), self.cursor))
    }
    /// Technical: Returns an optional `selected_slot` value.
    ///
    /// Plain English: Gets selected slot if it is available.
    fn selected_slot(&self, key: (usize, usize)) -> Option<usize> {
        Some(field_slot(self.focused_field(key)?).unwrap_or(0))
    }
    /// Open a tool, restarting the d-pad repeat timer.
    /// Plain English: Open a tool, restarting the d-pad repeat timer.
    fn open(&mut self, modal: Modal) {
        self.modal = Some(modal);
        self.command_held = 0;
        self.command_ticks = 0;
    }
    /// Close the open tool and its parent; copy lists and repeat state are left alone.
    /// Plain English: Close the open tool and its parent; copy lists and repeat state are left alone.
    fn dismiss(&mut self) {
        self.modal = None;
        self.return_modal = None;
    }
    /// Technical: Updates the `open_picker` state using the supplied inputs.
    ///
    /// Plain English: Updates open picker.
    fn open_picker(&mut self, key: (usize, usize), slot: usize) {
        let Some(r) = self.recipes.get(&key) else {
            return;
        };
        let kind = r.kinds[slot];
        let category = if kind >= PRESETS.len() {
            SAVED_INPUTS_GROUP
        } else {
            group(PRESETS[kind].1)
        };
        let item = candidates(r, category)
            .iter()
            .position(|k| *k == kind)
            .unwrap_or(0);
        self.return_modal = self.modal.filter(|m| matches!(m, Modal::Tools { .. }));
        self.open(Modal::Picker {
            slot,
            group: category,
            item,
        });
    }
    /// Technical: Returns an optional `field_index` value.
    ///
    /// Plain English: Gets field index if it is available.
    fn field_index(&self, key: (usize, usize), target: Field) -> Option<usize> {
        let r = self.recipes.get(&key)?;
        fields(r).iter().position(|f| *f == target)
    }
    /// Technical: Updates the `focus_slot` state using the supplied inputs.
    ///
    /// Plain English: Updates focus slot.
    fn focus_slot(&mut self, key: (usize, usize), slot: usize) {
        if self.recipes.contains_key(&key) {
            self.cursor = self.field_index(key, Field::Kind(slot)).unwrap_or(0);
        }
    }
    /// Technical: Determines whether the `commit_command` condition holds.
    ///
    /// Plain English: Checks whether commit command.
    fn commit_command(
        &mut self,
        key: (usize, usize),
        mut next: Recipe,
        focus: usize,
        message: &str,
    ) -> bool {
        if next.compile().is_none() {
            self.limited = true;
            self.last_edit = Some("Too many expanded steps; undo or remove a step first".into());
            return false;
        }
        let Some(old) = self.recipes.get_mut(&key) else {
            return false;
        };
        next.modified = true;
        next.edited |= changed_steps(old, &next);
        let previous = std::mem::replace(old, next);
        self.undo = Some(Box::new((key, previous)));
        self.limited = false;
        self.last_edit = Some(message.into());
        self.focus_slot(key, focus);
        true
    }
    /// Technical: Determines whether the `undo_edit` condition holds.
    ///
    /// Plain English: Checks whether undo edit.
    fn undo_edit(&mut self, key: (usize, usize)) -> bool {
        if !self.undo.as_ref().is_some_and(|u| u.0 == key) {
            self.last_edit = Some("No previous edit for this recipe".into());
            return false;
        }
        let Some(snapshot) = self.undo.take() else {
            return false;
        };
        let Some(current) = self.recipes.get_mut(&key) else {
            return false;
        };
        let mut previous = snapshot.1;
        // A previous snapshot may predate a successful Save; always mark Undo dirty.
        previous.modified = true;
        previous.edited |= changed_steps(current, &previous);
        *current = previous;
        self.cursor = self.cursor.min(fields(current).len() - 1);
        self.limited = false;
        self.last_edit = Some("Undid the previous edit".into());
        true
    }
    /// Technical: Determines whether the `structural` condition holds.
    ///
    /// Plain English: Checks whether structural.
    fn structural(&mut self, key: (usize, usize), slot: usize, item: usize) -> bool {
        if item == UNDO {
            return self.undo_edit(key);
        }
        let Some(mut next) = self.recipes.get(&key).cloned() else {
            return false;
        };
        let len = usize::from(next.sequence.len);
        let focus = match item {
            INSERT_BEFORE | INSERT_AFTER | DUPLICATE => {
                let at = if item == INSERT_BEFORE { slot } else { slot + 1 };
                let step = if item == DUPLICATE {
                    next.step(slot)
                } else {
                    (Action::Empty, 0, 0)
                };
                if !next.insert_step(at, step, slot + 1 < len) {
                    self.limited = true;
                    self.last_edit = Some(NO_ROOM.into());
                    return false;
                }
                at
            }
            MOVE_UP | MOVE_DOWN => {
                let other = if item == MOVE_UP {
                    slot.checked_sub(1)
                } else {
                    (slot + 1 < len).then_some(slot + 1)
                };
                let Some(other) = other else {
                    return false;
                };
                next.sequence.actions.swap(slot, other);
                next.before.swap(slot, other);
                next.kinds.swap(slot, other);
                other
            }
            DELETE => {
                next.remove_step(slot);
                slot.min(usize::from(next.sequence.len) - 1)
            }
            _ => return false,
        };
        self.commit_command(key, next, focus, TOOLS[item])
    }
    /// Technical: Determines whether the `command_repeat` condition holds.
    ///
    /// Plain English: Checks whether command repeat.
    fn command_repeat(&mut self, pad: u32) -> bool {
        if vertical(pad) {
            if self.command_held != pad {
                self.command_held = pad;
                self.command_ticks = 0;
                false
            } else {
                self.command_ticks = self.command_ticks.saturating_add(1);
                let t = self.command_ticks;
                t >= 24 && (t - 24) % if t >= 90 { 2 } else { 6 } == 0
            }
        } else {
            self.command_held = 0;
            self.command_ticks = 0;
            false
        }
    }
    /// Some = input owned by editor tools, (rebuild UI, recipe changed).
    /// Plain English: Some = input owned by editor tools, (rebuild UI, recipe changed).
    pub fn command_input(
        &mut self,
        key: (usize, usize),
        buttons: u32,
        pressed: u32,
    ) -> Option<(bool, bool)> {
        if pressed & crate::settings_menu::MENU_STICK_CLICK != 0
            || buttons == MAPPER_Y && pressed & MAPPER_Y != 0
            || buttons == MAPPER_X && pressed & MAPPER_X != 0
        {
            self.close_commands();
            return None;
        }
        match self.modal {
            None => self.idle_input(key, buttons, pressed),
            Some(modal) => self.modal_input(key, modal, buttons, pressed),
        }
    }
    /// Shortcuts while no tool is open. Runs every frame: only cheap checks until a press.
    /// Plain English: Shortcuts while no tool is open. Runs every frame: only cheap checks until a press.
    fn idle_input(&mut self, key: (usize, usize), buttons: u32, pressed: u32) -> Option<(bool, bool)> {
        let shoulders = MAPPER_ZL | MAPPER_ZR;
        if buttons & shoulders == shoulders && pressed & shoulders != 0 {
            let slot = self.selected_slot(key)?;
            self.open(Modal::Tools { slot, item: 0 });
            return Some((true, false));
        }
        if buttons & (MAPPER_R | shoulders) == 0 && pressed & (MAPPER_A | DPAD_LEFT | DPAD_RIGHT) != 0
        {
            match self.focused_field(key)? {
                Field::Buttons(slot) => {
                    self.open(Modal::Buttons { slot, item: 0 });
                    return Some((true, false));
                }
                Field::Kind(slot) if pressed & MAPPER_A != 0 => {
                    self.open_picker(key, slot);
                    return Some((true, false));
                }
                _ => {}
            }
        }
        if buttons & MAPPER_R != 0 {
            let pad = buttons & DPAD;
            let repeat = self.command_repeat(pad);
            if vertical(pad) && (pressed & pad != 0 || repeat) {
                let Some(slot) = field_slot(self.focused_field(key)?) else {
                    return Some((false, false));
                };
                let direction = if pad == DPAD_UP { MOVE_UP } else { MOVE_DOWN };
                let changed = self.structural(key, slot, direction);
                return Some((changed, changed));
            }
            return Some((false, false));
        }
        if pressed & (MAPPER_A | shoulders) == 0 {
            return None;
        }
        if pressed & MAPPER_A != 0 {
            let slot = self.selected_slot(key)?;
            self.open(Modal::Tools { slot, item: 0 });
        } else {
            self.jump_step(key, pressed & MAPPER_ZR != 0)?;
        }
        Some((true, false))
    }
    /// ZL/ZR: previous/next step; from a sequence-level row, the first/last step.
    /// Plain English: ZL/ZR: previous/next step; from a sequence-level row, the first/last step.
    fn jump_step(&mut self, key: (usize, usize), forward: bool) -> Option<()> {
        let r = self.recipes.get(&key)?;
        let len = usize::from(r.sequence.len);
        let next = match (field_slot(focus(&fields(r), self.cursor)), forward) {
            (None, true) => 0,
            (None, false) => len - 1,
            (Some(slot), true) => (slot + 1).min(len - 1),
            (Some(slot), false) => slot.saturating_sub(1),
        };
        self.focus_slot(key, next);
        Some(())
    }
    /// Technical: Returns an optional `modal_input` value.
    ///
    /// Plain English: Gets modal input if it is available.
    fn modal_input(
        &mut self,
        key: (usize, usize),
        modal: Modal,
        buttons: u32,
        pressed: u32,
    ) -> Option<(bool, bool)> {
        if pressed & MAPPER_B != 0 {
            self.modal = self.return_modal.take();
            self.clear_copy();
            self.command_held = 0;
            return Some((true, false));
        }
        let pad = buttons & DPAD;
        let repeat = self.command_repeat(pad);
        let confirm = pressed & MAPPER_A != 0;
        // The one d-pad direction pressed (or repeating) this frame.
        let nav = if !confirm && pad.count_ones() == 1 && (pressed & pad != 0 || repeat) {
            pad
        } else {
            0
        };
        if !confirm && nav == 0 {
            return Some((false, false));
        }
        match modal {
            Modal::Buttons { slot, item } => {
                let item = step_item(item, INPUT_BUTTONS.len(), nav);
                let toggle = confirm || nav & (DPAD_LEFT | DPAD_RIGHT) != 0;
                self.buttons_modal(key, slot, item, toggle)
            }
            Modal::Copy { group, item } => self.copy_modal(group, item, confirm, nav),
            Modal::Tools { slot, item } => self.tools_modal(key, slot, item, confirm, nav),
            Modal::Picker { slot, group, item } => {
                self.picker_modal(key, slot, group, item, confirm, nav)
            }
        }
    }
    /// Technical: Returns an optional `buttons_modal` value.
    ///
    /// Plain English: Gets buttons modal if it is available.
    fn buttons_modal(
        &mut self,
        key: (usize, usize),
        slot: usize,
        item: usize,
        toggle: bool,
    ) -> Option<(bool, bool)> {
        let mut changed = false;
        if toggle {
            let mut next = self.recipes.get(&key)?.clone();
            let action = &mut next.sequence.actions[slot];
            let Some(buttons) = buttons_mut(action) else {
                return Some((true, false));
            };
            *buttons ^= INPUT_BUTTONS[item].0;
            if *buttons == 0 {
                // Stick-only input cannot hold until manual takeover.
                if let Some((frames, _)) = frames_mut(action) {
                    if *frames == 0 {
                        *frames = 4;
                    }
                }
            }
            changed = self.commit_command(key, next, slot, "Changed buttons; kept axes and delay");
            if let Some(i) = self.field_index(key, Field::Buttons(slot)) {
                self.cursor = i;
            }
        }
        self.modal = Some(Modal::Buttons { slot, item });
        Some((true, changed))
    }
    /// Technical: Copies the modal.
    ///
    /// Plain English: Copies the modal.
    fn copy_modal(
        &mut self,
        group: usize,
        item: usize,
        confirm: bool,
        nav: u32,
    ) -> Option<(bool, bool)> {
        let count = self.copy_count(group);
        if confirm && count > 0 {
            let item = item.min(count - 1);
            if group == TEMPLATE_GROUP {
                self.template_apply = Some(item);
            } else {
                let source = self
                    .copy_sources_in(group)
                    .nth(item)
                    .map(|source| (source.slot, source.index));
                self.copy_apply = source;
            }
            return Some((true, false));
        }
        let (group, item) = if nav & (DPAD_LEFT | DPAD_RIGHT) != 0 {
            let forward = nav & DPAD_RIGHT != 0;
            let has_entries = |g| self.copy_count(g) > 0;
            (next_group(group, COPY_GROUPS.len(), forward, has_entries), 0)
        } else {
            (group, step_item(item, count, nav))
        };
        self.modal = Some(Modal::Copy { group, item });
        Some((true, false))
    }
    /// Technical: Returns an optional `tools_modal` value.
    ///
    /// Plain English: Gets tools modal if it is available.
    fn tools_modal(
        &mut self,
        key: (usize, usize),
        slot: usize,
        item: usize,
        confirm: bool,
        nav: u32,
    ) -> Option<(bool, bool)> {
        if !confirm {
            let item = step_item(item, TOOLS.len(), nav);
            self.modal = Some(Modal::Tools { slot, item });
            return Some((true, false));
        }
        let changed = match item {
            CHOOSE_ACTION => {
                self.open_picker(key, slot);
                false
            }
            COPY_STEP => {
                if self.copy_step(key, slot) {
                    self.close_commands();
                }
                false
            }
            PASTE_BEFORE | PASTE_AFTER => {
                let changed = self.paste_step(key, slot, item == PASTE_AFTER);
                if changed {
                    self.close_commands();
                }
                changed
            }
            RECIPE_OPTIONS => {
                self.close_commands();
                self.manage_requested = true;
                false
            }
            COPY_RECIPE => {
                self.clear_copy();
                self.return_modal = self.modal;
                self.copy_requested = true;
                self.open(Modal::Copy {
                    group: key.0,
                    item: 0,
                });
                false
            }
            _ => {
                let changed = self.structural(key, slot, item);
                if changed {
                    self.dismiss();
                }
                changed
            }
        };
        Some((true, changed))
    }
    /// Technical: Returns an optional `picker_modal` value.
    ///
    /// Plain English: Gets picker modal if it is available.
    fn picker_modal(
        &mut self,
        key: (usize, usize),
        slot: usize,
        group: usize,
        item: usize,
        confirm: bool,
        nav: u32,
    ) -> Option<(bool, bool)> {
        let r = self.recipes.get(&key)?;
        let available = candidates(r, group);
        if confirm && !available.is_empty() {
            let kind = available[item.min(available.len() - 1)];
            return self.apply_kind(key, slot, kind);
        }
        let (group, item) = if nav & (DPAD_LEFT | DPAD_RIGHT) != 0 {
            // Empty groups (notably Saved inputs) cost no extra category stop.
            let forward = nav & DPAD_RIGHT != 0;
            let has_entries = |g| !candidates(r, g).is_empty();
            (next_group(group, GROUPS.len(), forward, has_entries), 0)
        } else {
            (group, step_item(item, available.len(), nav))
        };
        self.modal = Some(Modal::Picker { slot, group, item });
        Some((true, false))
    }
    /// Technical: Applies the kind.
    ///
    /// Plain English: Applies the kind.
    fn apply_kind(&mut self, key: (usize, usize), slot: usize, kind: usize) -> Option<(bool, bool)> {
        let r = self.recipes.get(&key)?;
        if r.kinds[slot] == kind {
            self.dismiss();
            return Some((true, false));
        }
        let mut action = r.action_for(kind);
        if kind < PRESETS.len() && !matches!(action, Action::Input { buttons: MAPPER_X, .. }) {
            carry_hold(r.sequence.actions[slot], &mut action);
        }
        let mut next = r.clone();
        next.kinds[slot] = kind;
        next.sequence.actions[slot] = action;
        let changed = self.commit_command(key, next, slot, "Changed action; kept its delay");
        if changed {
            self.dismiss();
        }
        Some((true, changed))
    }
    /// Technical: Returns an optional `command_panel` value.
    ///
    /// Plain English: Gets command panel if it is available.
    pub fn command_panel(&self, key: (usize, usize)) -> Option<RecipePanel> {
        let modal = self.modal?;
        let r = self.recipes.get(&key)?;
        let mut panel = RecipePanel::default();
        match modal {
            Modal::Buttons { slot, item } => buttons_panel(&mut panel, r, slot, item),
            Modal::Copy { group, item } => self.copy_panel(&mut panel, group, item),
            Modal::Tools { slot, item } => tools_panel(&mut panel, r, slot, item),
            Modal::Picker { slot, group, item } => picker_panel(&mut panel, r, slot, group, item),
        }
        if let Some(message) = &self.last_edit {
            panel.footer.push(message.clone());
        }
        panel
            .footer
            .push("R3: Done | Y: Save current recipe".into());
        Some(panel)
    }
    /// Technical: Copies the panel.
    ///
    /// Plain English: Copies the panel.
    fn copy_panel(&self, panel: &mut RecipePanel, group: usize, item: usize) {
        panel.header.push("COPY RECIPE / GENERIC TEMPLATE".into());
        group_header(panel, COPY_GROUPS, group);
        if group == TEMPLATE_GROUP {
            for (i, name) in self.copy_templates.iter().enumerate() {
                numbered_row(panel, i, item, name);
            }
        } else {
            for (i, source) in self.copy_sources_in(group).enumerate() {
                numbered_row(panel, i, item, &source.name);
            }
        }
        if panel.body.is_empty() {
            panel
                .body
                .push("No configured source recipes in this category".into());
        } else {
            panel.focus = Some(item);
        }
        panel.footer.extend([
            "Left/Right: Category | Up/Down: Recipe".into(),
            "A: Replace current actions/timings | B: Cancel".into(),
            "Keeps destination name and button activation. Undo available.".into(),
        ]);
    }
}

/// Technical: Updates the `group_header` state using the supplied inputs.
///
/// Plain English: Updates group header.
fn group_header(panel: &mut RecipePanel, names: &[&str], group: usize) {
    panel.header.push(format!(
        "< {} >  {}/{}",
        names[group],
        group + 1,
        names.len()
    ));
}
/// Technical: Updates the `numbered_row` state using the supplied inputs.
///
/// Plain English: Updates numbered row.
fn numbered_row(panel: &mut RecipePanel, i: usize, item: usize, text: impl std::fmt::Display) {
    let marker = if i == item { "> " } else { "" };
    panel.body.push(format!("{marker}{:02}  {text}", i + 1));
}
/// Technical: Updates the `buttons_panel` state using the supplied inputs.
///
/// Plain English: Updates buttons panel.
fn buttons_panel(panel: &mut RecipePanel, r: &Recipe, slot: usize, item: usize) {
    panel
        .header
        .push(format!("STEP {:02} | BUTTON COMBINATION", slot + 1));
    let mask = input_buttons(r.sequence.actions[slot]).unwrap_or(0);
    for (i, (bit, name)) in INPUT_BUTTONS.iter().enumerate() {
        panel.body.push(format!(
            "{}[{}] {name}",
            if i == item { "> " } else { "  " },
            if mask & bit != 0 { "x" } else { " " }
        ));
    }
    panel.focus = Some(item);
    panel.footer.extend([
        "Up/Down: button | A/Left/Right: toggle | B: back".into(),
        "Any combination; none = sticks only. Uses normal configured mappings.".into(),
    ]);
}
/// Technical: Updates the `tools_panel` state using the supplied inputs.
///
/// Plain English: Updates tools panel.
fn tools_panel(panel: &mut RecipePanel, r: &Recipe, slot: usize, item: usize) {
    panel
        .header
        .push(format!("STEP {:02} | {}", slot + 1, action_name(r, slot)));
    panel.header.push("STEP TOOLS".into());
    for (i, name) in TOOLS.iter().enumerate() {
        panel
            .body
            .push(format!("{}{name}", if i == item { "> " } else { "  " }));
    }
    panel.focus = Some(item);
    panel
        .footer
        .push("A: Choose | B: Back | Up/Down: Tool".into());
}
/// Technical: Updates the `picker_panel` state using the supplied inputs.
///
/// Plain English: Updates picker panel.
fn picker_panel(panel: &mut RecipePanel, r: &Recipe, slot: usize, group: usize, item: usize) {
    panel
        .header
        .push(format!("CHOOSE ACTION | STEP {:02}", slot + 1));
    group_header(panel, GROUPS, group);
    for (i, kind) in candidates(r, group).into_iter().enumerate() {
        match PRESETS.get(kind) {
            Some((name, _)) => numbered_row(panel, i, item, name),
            None => numbered_row(
                panel,
                i,
                item,
                format_args!("Saved: {}", describe_action(r.action_for(kind))),
            ),
        }
    }
    if panel.body.is_empty() {
        panel.body.push("No imported inputs in this recipe".into());
    } else {
        panel.focus = Some(item);
    }
    panel
        .footer
        .push("Left/Right: Group | Up/Down: Action | A: Apply | B: Cancel".into());
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Technical: Defines the `KEY` constant used by this module.
    ///
    /// Plain English: Provides the KEY constant value.
    const KEY: (usize, usize) = (5, 0);
    /// Technical: Returns the `editor` result.
    ///
    /// Plain English: Gets the editor result.
    fn editor() -> RecipeEditor {
        let source=crate::sequence_config::Catalog::parse("[kirby.x.test]\nstart_delay=2\nwhile_held=true\nactions=hold:a:7,wait:3,hold:b:9,wait:2,hold:r:4").unwrap().entries()[0].sequence;
        let mut e = RecipeEditor::default();
        e.ensure(KEY, source);
        e
    }
    /// Technical: Returns an optional `tap` value.
    ///
    /// Plain English: Gets tap if it is available.
    fn tap(e: &mut RecipeEditor, button: u32) -> Option<(bool, bool)> {
        e.command_input(KEY, 0, 0);
        e.command_input(KEY, button, button)
    }
    /// Technical: Regression test for `delete_middle_preserves_following_parameters_and_undo_restores_exact_sequence`.
    ///
    /// Plain English: Checks that delete middle preserves following parameters and undo restores exact sequence.
    #[test]
    fn delete_middle_preserves_following_parameters_and_undo_restores_exact_sequence() {
        let mut e = editor();
        let before = e.compiled(KEY).unwrap();
        assert!(e.structural(KEY, 1, 6));
        let after = e.compiled(KEY).unwrap();
        assert_eq!(
            after.actions(),
            &[
                before.actions()[0],
                before.actions()[3],
                before.actions()[4]
            ]
        );
        assert!(after.while_held);
        assert_eq!(after.start_delay, 2);
        assert!(e.undo_edit(KEY));
        assert_eq!(e.compiled(KEY).unwrap(), before);
    }
    /// Technical: Regression test for `moves_duplicate_and_inserts_carry_delays_kinds_and_values`.
    ///
    /// Plain English: Checks that moves duplicate and inserts carry delays kinds and values.
    #[test]
    fn moves_duplicate_and_inserts_carry_delays_kinds_and_values() {
        let mut e = editor();
        let before = e.recipes[&KEY].clone();
        assert!(e.structural(KEY, 1, 4));
        let moved = &e.recipes[&KEY];
        assert_eq!(moved.sequence.actions[0], before.sequence.actions[1]);
        assert_eq!(moved.before[0], 3);
        assert_eq!(moved.kinds[0], before.kinds[1]);
        assert!(e.structural(KEY, 0, 3));
        let r = &e.recipes[&KEY];
        assert_eq!(r.sequence.actions[0], r.sequence.actions[1]);
        assert_eq!(r.before[1], 3);
        assert!(e.structural(KEY, 1, 1));
        let r = &e.recipes[&KEY];
        assert_eq!(r.sequence.actions[1], Action::Empty);
        assert_eq!(r.before[1], 0);
        assert_eq!(r.before[2], 3);
    }
    /// Technical: Regression test for `full_capacity_or_expanded_limit_refuses_without_losing_original`.
    ///
    /// Plain English: Checks that full capacity or expanded limit refuses without losing original.
    #[test]
    fn full_capacity_or_expanded_limit_refuses_without_losing_original() {
        let mut e = editor();
        let r = e.recipes.get_mut(&KEY).unwrap();
        r.sequence.len = 32;
        r.sequence.actions.fill(key(MAPPER_A, 0., 0., 4));
        r.before.fill(0);
        let before = e.compiled(KEY).unwrap();
        assert!(!e.structural(KEY, 1, 3));
        assert_eq!(e.compiled(KEY).unwrap(), before);
        let r = e.recipes.get_mut(&KEY).unwrap();
        r.sequence.len = 16;
        r.before.fill(0);
        for i in 0..16 {
            r.before[i] = 1;
        }
        let before = e.compiled(KEY).unwrap();
        assert!(!e.structural(KEY, 1, 3));
        assert_eq!(e.compiled(KEY).unwrap(), before);
    }
    /// Technical: Regression test for `picker_is_complete_and_cancel_or_apply_same_action_preserves_hold_and_delay`.
    ///
    /// Plain English: Checks that picker is complete and cancel or apply same action preserves hold and delay.
    #[test]
    fn picker_is_complete_and_cancel_or_apply_same_action_preserves_hold_and_delay() {
        let mut e = editor();
        e.focus_slot(KEY, 1);
        let before = e.compiled(KEY).unwrap();
        assert_eq!(tap(&mut e, MAPPER_A), Some((true, false)));
        assert!(matches!(e.modal, Some(Modal::Picker { slot: 1, .. })));
        assert_eq!(
            e.command_input(KEY, 0, 0),
            Some((false, false)),
            "idle modal does not request a rebuild"
        );
        tap(&mut e, MAPPER_A);
        assert_eq!(
            e.compiled(KEY).unwrap(),
            before,
            "choosing current action must not restore preset timing"
        );
        let all: Vec<_> = (0..8)
            .flat_map(|g| candidates(&e.recipes[&KEY], g))
            .collect();
        for kind in 0..PRESETS.len() {
            assert!(all.contains(&kind));
        }
        tap(&mut e, MAPPER_A);
        tap(&mut e, DPAD_RIGHT);
        tap(&mut e, MAPPER_B);
        assert_eq!(e.compiled(KEY).unwrap(), before);
    }
    /// Technical: Regression test for `custom_saved_inputs_remain_available_after_reorder_and_replace`.
    ///
    /// Plain English: Checks that custom saved inputs remain available after reorder and replace.
    #[test]
    fn custom_saved_inputs_remain_available_after_reorder_and_replace() {
        let mut e = editor();
        let mut source = e.compiled(KEY).unwrap();
        source.actions[0] = key(MAPPER_A | MAPPER_R, 0.37, -0.21, 13);
        e = RecipeEditor::default();
        e.ensure(KEY, source);
        assert!(candidates(&e.recipes[&KEY], 8).len() >= 1);
        assert!(e.structural(KEY, 0, 5));
        let r = &e.recipes[&KEY];
        let saved = candidates(r, 8)[0];
        assert_eq!(r.imported[saved - PRESETS.len()], source.actions[0]);
    }
    /// Technical: Regression test for `step_shortcuts_skip_settings_and_numeric_edits_are_undoable`.
    ///
    /// Plain English: Checks that step shortcuts skip settings and numeric edits are undoable.
    #[test]
    fn step_shortcuts_skip_settings_and_numeric_edits_are_undoable() {
        let mut e = editor();
        e.focus_slot(KEY, 0);
        tap(&mut e, MAPPER_ZR);
        assert_eq!(e.selected_slot(KEY), Some(1));
        tap(&mut e, MAPPER_ZL);
        assert_eq!(e.selected_slot(KEY), Some(0));
        e.cursor = 0;
        let old = e.compiled(KEY).unwrap();
        assert!(e.adjust(KEY, true));
        assert!(e.undo_edit(KEY));
        assert_eq!(e.compiled(KEY).unwrap(), old);
    }
    /// Technical: Regression test for `shoulder_reorder_is_direct_and_repeats_only_on_navigation_updates`.
    ///
    /// Plain English: Checks that shoulder reorder is direct and repeats only on navigation updates.
    #[test]
    fn shoulder_reorder_is_direct_and_repeats_only_on_navigation_updates() {
        let mut e = editor();
        assert_eq!(
            e.command_input(KEY, MAPPER_R | DPAD_DOWN, DPAD_DOWN),
            Some((false, false)),
            "sequence-level settings must not reorder a step"
        );
        e.focus_slot(KEY, 1);
        let before = e.recipes[&KEY].clone();
        assert_eq!(
            e.command_input(KEY, MAPPER_R, MAPPER_R),
            Some((false, false))
        );
        assert_eq!(
            e.command_input(KEY, MAPPER_R | DPAD_UP, DPAD_UP),
            Some((true, true))
        );
        assert_eq!(e.selected_slot(KEY), Some(0));
        assert_eq!(
            e.recipes[&KEY].sequence.actions[0],
            before.sequence.actions[1]
        );
        assert_eq!(e.recipes[&KEY].before[0], 3);
        for _ in 0..12 {
            assert_eq!(
                e.command_input(KEY, MAPPER_R | DPAD_UP, 0),
                Some((false, false))
            );
        }
        assert!(e.undo_edit(KEY));
        assert_eq!(e.compiled(KEY).unwrap(), before.compile().unwrap());
    }
    /// Technical: Regression test for `insertion_and_deleting_final_step_preserve_other_steps_and_duration`.
    ///
    /// Plain English: Checks that insertion and deleting final step preserve other steps and duration.
    #[test]
    fn insertion_and_deleting_final_step_preserve_other_steps_and_duration() {
        let mut e = editor();
        let before = e.compiled(KEY).unwrap();
        assert!(e.structural(KEY, 1, 2));
        assert_eq!(
            e.compiled(KEY).unwrap(),
            before,
            "an empty inserted slot adds no time"
        );
        assert_eq!(e.recipes[&KEY].before[3], 2);
        let len = usize::from(e.recipes[&KEY].sequence.len);
        assert!(e.structural(KEY, len - 1, 6));
        assert!(e.undo_edit(KEY));
        assert_eq!(e.compiled(KEY).unwrap(), before);
    }
    /// Technical: Regression test for `action_edits_and_undo_only_mark_affected_steps`.
    ///
    /// Plain English: Checks that action edits and undo only mark affected steps.
    #[test]
    fn action_edits_and_undo_only_mark_affected_steps() {
        let mut e = editor();
        let mut next = e.recipes[&KEY].clone();
        next.sequence.actions[0] = key(MAPPER_A, 0., 0., 14);
        assert!(e.commit_command(KEY, next, 0, "Changed hold"));
        assert_eq!(e.recipes[&KEY].edited, 1);
        assert!(e.undo_edit(KEY));
        assert_eq!(e.recipes[&KEY].edited, 1);
    }
    /// Technical: Copies the template uses unsaved source timings and preserves destination activation.
    ///
    /// Plain English: Checks that copy template uses unsaved source timings and preserves destination activation.
    #[test]
    fn copy_template_uses_unsaved_source_timings_and_preserves_destination_activation() {
        let mut e = editor();
        let target_before = e.compiled(KEY).unwrap();
        let source_key = (0, 0);
        let mut source = target_before;
        source.start_delay = 9;
        source.timeout = 900;
        source.spatial_timeout = 70;
        source.while_held = false;
        source.actions[0] = key(MAPPER_A, 0., -1., 17);
        e.ensure(source_key, source);
        // An in-memory edit must take precedence over file fallback.
        e.recipes.get_mut(&source_key).unwrap().before[1] = 13;
        let source_before = e.compiled(source_key).unwrap();
        assert!(e.apply_template(KEY, source_key, target_before));
        let mut expected = source_before;
        expected.while_held = true;
        assert_eq!(e.compiled(KEY).unwrap(), expected);
        assert_eq!(e.compiled(source_key).unwrap(), source_before);
        assert!(e.undo_edit(KEY));
        assert_eq!(e.compiled(KEY).unwrap(), target_before);
        assert!(!e.apply_template(KEY, KEY, source));
    }
    /// Technical: Regression test for `button_to_ledge_copy_cannot_enable_looping_and_releases_source_list`.
    ///
    /// Plain English: Checks that button to ledge copy cannot enable looping and releases source list.
    #[test]
    fn button_to_ledge_copy_cannot_enable_looping_and_releases_source_list() {
        let mut e = editor();
        let source = e.compiled(KEY).unwrap();
        let destination = (1, 0);
        e.ensure(destination, RecipeEditor::custom());
        e.copy_sources.push(CopySource {
            slot: crate::fighter_profiles::presets::PresetSlot::X,
            index: 0,
            name: "Source".into(),
        });
        assert!(e.apply_template(destination, KEY, source));
        let target = e.compiled(destination).unwrap();
        assert_eq!(target.actions(), source.actions());
        assert!(!target.while_held);
        assert!(!e.recipes[&destination].button_recipe);
        assert_eq!(e.copy_sources.capacity(), 0);
        assert!(e.modal.is_none());
    }
    /// Technical: Regression test for `cancelling_copy_drops_temporary_names_without_changing_recipes`.
    ///
    /// Plain English: Checks that cancelling copy drops temporary names without changing recipes.
    #[test]
    fn cancelling_copy_drops_temporary_names_without_changing_recipes() {
        let mut e = editor();
        let before = e.compiled(KEY).unwrap();
        e.modal = Some(Modal::Copy { group: 5, item: 0 });
        e.copy_sources.push(CopySource {
            slot: crate::fighter_profiles::presets::PresetSlot::X,
            index: 1,
            name: "Saved source".into(),
        });
        assert_eq!(e.command_input(KEY, 0, 0), Some((false, false)));
        assert_eq!(tap(&mut e, MAPPER_B), Some((true, false)));
        assert_eq!(e.copy_sources.capacity(), 0);
        assert_eq!(e.compiled(KEY).unwrap(), before);
    }
}

#[cfg(test)]
mod reaction_copy_tests {
    use super::*;
    /// Technical: Defines the `KEY` constant used by this module.
    ///
    /// Plain English: Provides the KEY constant value.
    const KEY: (usize, usize) = (5, 0);
    /// Technical: Returns the `editor` result.
    ///
    /// Plain English: Gets the editor result.
    fn editor() -> RecipeEditor {
        let source = crate::sequence_config::Catalog::parse(
            "[kirby.x.test]\nstart_delay=2\nactions=hold:a:7\n",
        )
        .unwrap()
        .entries()[0]
            .sequence;
        let mut e = RecipeEditor::default();
        e.ensure(KEY, source);
        e
    }
    /// Technical: Returns an optional `tap` value.
    ///
    /// Plain English: Gets tap if it is available.
    fn tap(e: &mut RecipeEditor, b: u32) -> Option<(bool, bool)> {
        e.command_input(KEY, 0, 0);
        e.command_input(KEY, b, b)
    }
    /// Technical: Regression test for `copying_inputs_keeps_destination_reaction_conditions`.
    ///
    /// Plain English: Checks that copying inputs keeps destination reaction conditions.
    #[test]
    fn copying_inputs_keeps_destination_reaction_conditions() {
        let mut source = RecipeEditor::custom();
        source.len = 1;
        source.actions[0] = PRESETS[3].1;
        source.reaction = crate::reaction_rules::Rule::parse("roll:any:0:20:12:any:1").unwrap();
        let mut target = source;
        target.actions[0] = PRESETS[16].1;
        target.reaction = crate::reaction_rules::Rule::default();
        for (destination, from) in [(target, source), (source, target)] {
            let mut e = RecipeEditor::default();
            e.ensure((0, 0), destination);
            e.ensure((5, 0), from);
            assert!(e.apply_template((0, 0), (5, 0), from));
            let result = e.compiled((0, 0)).unwrap();
            assert_eq!(result.reaction, destination.reaction);
            assert_eq!(result.actions(), from.actions());
        }
    }

    /// Technical: Regression test for `fixed_picker_has_unique_actions_names_and_one_category_each`.
    ///
    /// Plain English: Checks that fixed picker has unique actions names and one category each.
    #[test]
    fn fixed_picker_has_unique_actions_names_and_one_category_each() {
        let e = editor();
        let all: Vec<_> = (0..8)
            .flat_map(|g| candidates(&e.recipes[&KEY], g))
            .collect();
        for (i, (name, action)) in PRESETS.iter().enumerate() {
            assert_eq!(all.iter().filter(|k| **k == i).count(), 1, "{name}");
            for (other, other_action) in &PRESETS[i + 1..] {
                assert_ne!(name, other);
                assert_ne!(action, other_action, "duplicate {name} / {other}");
            }
        }
        assert!(!DIRECTIONS.contains(&D::Ledge));
        let docs = include_str!("../../../../../docs/EDITOR_ACTION_CHEATSHEET.md");
        // Documentation for these new actions is explicitly deferred by the user.
        for (name, action) in PRESETS.iter().filter(|(_, a)| {
            !matches!(
                a,
                Action::Chase { .. }
                    | Action::IfFacingOpponent { .. }
                    | Action::ElseFacing
                    | Action::LedgePosition { .. }
            )
        }) {
            let _ = action;
            assert!(
                docs.contains(&format!("| {name} |")),
                "missing cheat-sheet entry {name}"
            );
        }
        assert!(
            std::mem::size_of::<Action>() <= 24,
            "keep controller input within existing action storage"
        );
    }
    /// Technical: Regression test for `button_panel_composes_all_gameplay_buttons_and_preserves_sticks_hold_and_delay`.
    ///
    /// Plain English: Checks that button panel composes all gameplay buttons and preserves sticks hold and delay.
    #[test]
    fn button_panel_composes_all_gameplay_buttons_and_preserves_sticks_hold_and_delay() {
        let mut source = Sequence::default();
        source.len = 1;
        source.start_delay = 7;
        source.actions[0] = Action::Controller {
            buttons: 0,
            stick: [0.4, -0.3],
            cstick: [-0.8, 0.6],
            frames: 17,
            reference: IR::Opponent,
            cstick_reference: IR::Facing,
        };
        let mut e = RecipeEditor::default();
        e.ensure(KEY, source);
        e.cursor = fields(&e.recipes[&KEY])
            .iter()
            .position(|f| matches!(f, Field::Buttons(0)))
            .unwrap();
        assert_eq!(tap(&mut e, MAPPER_A), Some((true, false)));
        for i in 0..INPUT_BUTTONS.len() {
            e.modal = Some(Modal::Buttons { slot: 0, item: i });
            assert_eq!(tap(&mut e, MAPPER_A), Some((true, true)));
        }
        let result = e.compiled(KEY).unwrap();
        assert_eq!(result.start_delay, 7);
        assert!(
            matches!(result.actions[0],Action::Controller{buttons,stick:[0.4,-0.3],cstick:[-0.8,0.6],frames:17,reference:IR::Opponent,cstick_reference:IR::Facing} if buttons==INPUT_BUTTONS.iter().fold(0,|m,(b,_)|m|b))
        );
        tap(&mut e, MAPPER_B);
        assert!(e.modal.is_none());
        assert!(e.undo_edit(KEY));
    }
    /// Technical: Regression test for `jump_choices_use_distinct_pulses_instead_of_inheriting_attack_hold`.
    ///
    /// Plain English: Checks that jump choices use distinct pulses instead of inheriting attack hold.
    #[test]
    fn jump_choices_use_distinct_pulses_instead_of_inheriting_attack_hold() {
        let mut e = editor();
        for (label, frames) in [("Short hop", 1), ("Full hop hold", 4)] {
            let kind = PRESETS.iter().position(|(name, _)| *name == label).unwrap();
            let group = group(PRESETS[kind].1);
            let item = candidates(&e.recipes[&KEY], group)
                .iter()
                .position(|k| *k == kind)
                .unwrap();
            e.modal = Some(Modal::Picker {
                slot: 0,
                group,
                item,
            });
            tap(&mut e, MAPPER_A);
            assert!(
                matches!(e.compiled(KEY).unwrap().actions[0],Action::Input{buttons:MAPPER_X,frames:f,..} if f==frames)
            );
        }
    }
    /// Technical: Regression test for `legacy_saved_actions_are_retained_without_reintroducing_fixed_duplicates`.
    ///
    /// Plain English: Checks that legacy saved actions are retained without reintroducing fixed duplicates.
    #[test]
    fn legacy_saved_actions_are_retained_without_reintroducing_fixed_duplicates() {
        let source = crate::sequence_config::Catalog::parse(
            "[kirby.x.t]\nstart_delay=0\nactions=range:ignore,shield_until_manual,range:ignore\n",
        )
        .unwrap()
        .entries()[0]
            .sequence;
        let mut e = RecipeEditor::default();
        e.ensure(KEY, source);
        assert_eq!(e.compiled(KEY).unwrap(), source);
        assert_eq!(candidates(&e.recipes[&KEY], 8).len(), 2);
        for (_, a) in PRESETS {
            assert!(!matches!(
                a,
                Action::Range(RangeCheck::Ignore) | Action::ShieldUntilManual
            ));
        }
    }
}

#[cfg(test)]
mod menu_audit_tests {
    use super::*;
    /// Technical: Defines the `SRC` constant used by this module.
    ///
    /// Plain English: Provides the SRC constant value.
    const SRC: (usize, usize) = (5, 0);
    /// Technical: Defines the `DST` constant used by this module.
    ///
    /// Plain English: Provides the DST constant value.
    const DST: (usize, usize) = (0, 0);
    /// Technical: Returns the `recipe` result.
    ///
    /// Plain English: Gets the recipe result.
    fn recipe(a: &str) -> Sequence {
        crate::sequence_config::Catalog::parse(&format!(
            "[bowser.attack.test]\nstart_delay=0\nactions={a}\n"
        ))
        .unwrap()
        .entries()[0]
            .sequence
    }
    /// Technical: Returns an optional `tap` value.
    ///
    /// Plain English: Gets tap if it is available.
    fn tap(e: &mut RecipeEditor, key: (usize, usize), b: u32) -> Option<(bool, bool)> {
        e.command_input(key, 0, 0);
        e.command_input(key, b, b)
    }
    /// Technical: Regression test for `confirm_on_a_held_arrow_toggles_the_selected_button_without_moving_the_cursor`.
    ///
    /// Plain English: Checks that confirm on a held arrow toggles the selected button without moving the cursor.
    #[test]
    fn confirm_on_a_held_arrow_toggles_the_selected_button_without_moving_the_cursor() {
        let mut e = RecipeEditor::default();
        e.ensure(DST, recipe("input:a:0:0:4"));
        e.cursor = fields(&e.recipes[&DST])
            .iter()
            .position(|f| matches!(f, Field::Buttons(0)))
            .unwrap();
        tap(&mut e, DST, MAPPER_A);
        tap(&mut e, DST, DPAD_DOWN);
        let Some(Modal::Buttons { item, .. }) = e.modal else {
            panic!()
        };
        let expected =
            input_buttons(e.recipes[&DST].sequence.actions[0]).unwrap() ^ INPUT_BUTTONS[item].0;
        assert_eq!(
            e.command_input(DST, DPAD_DOWN | MAPPER_A, MAPPER_A),
            Some((true, true))
        );
        assert!(matches!(e.modal,Some(Modal::Buttons{item:i,..})if i==item));
        assert_eq!(
            input_buttons(e.recipes[&DST].sequence.actions[0]),
            Some(expected)
        );
    }
    /// Technical: Regression test for `step_clipboard_preserves_imported_axes_references_duration_and_delay_across_recipes`.
    ///
    /// Plain English: Checks that step clipboard preserves imported axes references duration and delay across recipes.
    #[test]
    fn step_clipboard_preserves_imported_axes_references_duration_and_delay_across_recipes() {
        let source = recipe("wait:7,controller_input:a+b:0.4:-0.2:-0.6:0.8:17:opponent:facing");
        let dest = recipe("controller_input:r:0:0:0:0:4:screen:screen");
        let mut e = RecipeEditor::default();
        e.ensure(SRC, source);
        assert!(e.copy_step(SRC, 0));
        e.ensure(DST, dest);
        assert!(e.step_clipboard.is_some());
        assert!(e.paste_step(DST, 0, true));
        let compiled = e.compiled(DST).unwrap();
        assert_eq!(&compiled.actions()[1..], source.actions());
        assert_eq!(compiled.actions()[0], dest.actions()[0]);
        assert_eq!(e.compiled(SRC), Some(source));
        assert!(e.undo_edit(DST));
        assert_eq!(e.compiled(DST), Some(dest));
    }
    /// Technical: Regression test for `paste_limits_and_empty_clipboard_cannot_overwrite_or_lose_the_existing_recipe`.
    ///
    /// Plain English: Checks that paste limits and empty clipboard cannot overwrite or lose the existing recipe.
    #[test]
    fn paste_limits_and_empty_clipboard_cannot_overwrite_or_lose_the_existing_recipe() {
        let mut e = RecipeEditor::default();
        let mut full = recipe("input:a:0:0:1");
        full.len = MAX_ACTIONS as u8;
        let action = full.actions[0];
        full.actions.fill(action);
        e.ensure(DST, full);
        assert!(!e.paste_step(DST, 0, true));
        assert_eq!(e.compiled(DST), Some(full));
        assert!(e.copy_step(DST, 0));
        assert!(!e.paste_step(DST, 0, false));
        assert_eq!(e.compiled(DST), Some(full));
        let mut expanded = recipe("input:a:0:0:1");
        expanded.len = (MAX_ACTIONS - 1) as u8;
        let action = expanded.actions[0];
        expanded.actions[..MAX_ACTIONS - 1].fill(action);
        e.ensure((1, 0), expanded);
        e.step_clipboard = Some((
            Action::Input {
                buttons: MAPPER_B,
                stick: [0.; 2],
                frames: 1,
                relative: false,
            },
            1,
        ));
        assert!(
            !e.paste_step((1, 0), 0, false),
            "pasted delay counts toward expanded step limit"
        );
        assert_eq!(e.compiled((1, 0)), Some(expanded));
    }
    /// Technical: Regression test for `back_returns_to_parent_tools_but_done_and_save_close_all_popups`.
    ///
    /// Plain English: Checks that back returns to parent tools but done and save close all popups.
    #[test]
    fn back_returns_to_parent_tools_but_done_and_save_close_all_popups() {
        let mut e = RecipeEditor::default();
        e.ensure(DST, recipe("input:a:0:0:4"));
        tap(&mut e, DST, MAPPER_ZL | MAPPER_ZR);
        tap(&mut e, DST, MAPPER_A);
        assert!(matches!(e.modal, Some(Modal::Picker { .. })));
        tap(&mut e, DST, MAPPER_B);
        assert!(matches!(e.modal, Some(Modal::Tools { item: 0, .. })));
        tap(&mut e, DST, MAPPER_A);
        assert_eq!(
            tap(&mut e, DST, crate::settings_menu::MENU_STICK_CLICK),
            None
        );
        assert!(e.modal.is_none());
        assert!(e.return_modal.is_none());
        tap(&mut e, DST, MAPPER_ZL | MAPPER_ZR);
        assert_eq!(tap(&mut e, DST, MAPPER_Y), None);
        assert!(e.modal.is_none());
    }
    /// Technical: Copies the category navigation skips empty groups in both directions.
    ///
    /// Plain English: Checks that copy category navigation skips empty groups in both directions.
    #[test]
    fn copy_category_navigation_skips_empty_groups_in_both_directions() {
        let mut e = RecipeEditor::default();
        e.ensure(DST, recipe("input:a:0:0:4"));
        e.modal = Some(Modal::Copy { group: 0, item: 0 });
        e.set_copy_sources(vec![
            CopySource {
                slot: super::super::super::PresetSlot::Roll,
                index: 0,
                name: "Roll".into(),
            },
            CopySource {
                slot: super::super::super::PresetSlot::Y,
                index: 0,
                name: "Y".into(),
            },
        ]);
        assert!(matches!(e.modal, Some(Modal::Copy { group: 2, .. })));
        tap(&mut e, DST, DPAD_RIGHT);
        assert!(matches!(e.modal, Some(Modal::Copy { group: 6, .. })));
        tap(&mut e, DST, DPAD_RIGHT);
        assert!(matches!(e.modal, Some(Modal::Copy { group: 2, .. })));
        tap(&mut e, DST, DPAD_LEFT);
        assert!(matches!(e.modal, Some(Modal::Copy { group: 6, .. })));
    }
    /// Technical: Regression test for `identical_template_is_a_clean_noop_and_incompatible_copy_keeps_the_destination`.
    ///
    /// Plain English: Checks that identical template is a clean noop and incompatible copy keeps the destination.
    #[test]
    fn identical_template_is_a_clean_noop_and_incompatible_copy_keeps_the_destination() {
        let original = recipe("input:a:0:0:4");
        let mut e = RecipeEditor::default();
        e.ensure(DST, original);
        e.ensure(SRC, original);
        e.modal = Some(Modal::Copy { group: 5, item: 0 });
        assert!(!e.apply_template(DST, SRC, original));
        assert!(!e.edited(DST));
        assert!(e.undo.is_none());
        assert!(e.modal.is_none());
        let mut response = original;
        response.reaction = crate::reaction_rules::Rule::parse("auto_roll:30:12").unwrap();
        e.ensure((2, 0), response);
        let ledge = recipe("run_to_ledge:12:1");
        e.ensure((3, 0), ledge);
        assert!(!e.apply_template((2, 0), (3, 0), ledge));
        assert_eq!(e.compiled((2, 0)), Some(response));
        assert!(e.copy_step((3, 0), 0));
        assert!(!e.paste_step((2, 0), 0, true));
    }
    /// Technical: Regression test for `onstage_picker_and_direction_adjustment_only_offer_actions_that_can_execute_there`.
    ///
    /// Plain English: Checks that onstage picker and direction adjustment only offer actions that can execute there.
    #[test]
    fn onstage_picker_and_direction_adjustment_only_offer_actions_that_can_execute_there() {
        let mut source = recipe("face:opponent");
        source.reaction = crate::reaction_rules::Rule::parse("auto_roll:30:12").unwrap();
        let mut e = RecipeEditor::default();
        e.ensure(DST, source);
        for group in 0..8 {
            for kind in candidates(&e.recipes[&DST], group) {
                assert!(!PRESETS[kind].1.needs_ledge());
            }
        }
        e.cursor = fields(&e.recipes[&DST])
            .iter()
            .position(|f| matches!(f, Field::Direction(0)))
            .unwrap();
        for _ in 0..16 {
            assert!(e.adjust(DST, true));
            assert!(!e.compiled(DST).unwrap().actions()[0].needs_ledge());
        }
    }
}

#[cfg(test)]
mod generic_template_copy_tests {
    use super::*;
    use crate::sequence_config::Catalog;
    /// Technical: Returns the `sequence` result.
    ///
    /// Plain English: Gets the sequence result.
    fn sequence(source: &str) -> Sequence {
        Catalog::parse(source).unwrap().entries()[0].sequence
    }
    /// Technical: Regression test for `generic_copy_keeps_destination_reaction_and_activation_and_frees_dialog_names`.
    ///
    /// Plain English: Checks that generic copy keeps destination reaction and activation and frees dialog names.
    #[test]
    fn generic_copy_keeps_destination_reaction_and_activation_and_frees_dialog_names() {
        let key = (5, 0);
        let original=sequence("[kirby.x.custom]\nwhile_held=true\nstart_delay=0\nreaction=auto_shield_grab:20:5\nactions=grab:4\n");
        let c =
            Catalog::parse("[template.fast_grab]\nname=Fast grab\nstart_delay=1\nactions=grab:2\n")
                .unwrap();
        let mut e = RecipeEditor::default();
        e.ensure(key, original);
        e.set_copy_templates(&c.templates);
        assert!(e.apply_generic_template(key, c.templates[0].sequence));
        let after = e.compiled(key).unwrap();
        assert_eq!(after.reaction, original.reaction);
        assert!(after.while_held);
        assert_eq!(after.start_delay, 1);
        assert_eq!(after.actions(), c.templates[0].sequence.actions());
        assert_eq!(e.copy_templates.capacity(), 0);
        assert_eq!(e.copy_sources.capacity(), 0);
    }
    /// Technical: Regression test for `a_ledge_template_cannot_replace_an_automatic_punish`.
    ///
    /// Plain English: Checks that a ledge template cannot replace an automatic punish.
    #[test]
    fn a_ledge_template_cannot_replace_an_automatic_punish() {
        let key = (2, 0);
        let original = sequence(
            "[bowser.roll.custom]\nstart_delay=0\nreaction=auto_roll:30:12\nactions=up_b:4\n",
        );
        let c = Catalog::parse(
            "[template.spacing]\nstart_delay=0\nactions=run_to_ledge:10:1:1:2:120\n",
        )
        .unwrap();
        let mut e = RecipeEditor::default();
        e.ensure(key, original);
        assert!(!e.apply_generic_template(key, c.templates[0].sequence));
        assert_eq!(e.compiled(key), Some(original));
    }
}
