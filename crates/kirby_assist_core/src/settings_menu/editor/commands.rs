//! Input-driven editor tools only; no game queries, files, workers or execution changes.
use super::*;
use crate::ledge_trap::DPAD_DOWN;
use crate::manual_controls::{DPAD_LEFT, DPAD_RIGHT, DPAD_UP};
use crate::settings_menu::RecipePanel;

/// Available `Modal` choices for the Smash assistance menu.
#[derive(Clone, Copy)]
pub(super) enum Modal {
    /// `Buttons` option in the `Modal` Smash policy state.
    Buttons {
        /// Runtime slot value used by Modal..
        slot: usize,
        /// Runtime item value used by Modal..
        item: usize,
    },
    /// `Copy` option in the `Modal` Smash policy state.
    Copy {
        /// Runtime group value used by Modal..
        group: usize,
        /// Runtime item value used by Modal..
        item: usize,
    },
    /// `Tools` option in the `Modal` Smash policy state.
    Tools {
        /// Runtime slot value used by Modal..
        slot: usize,
        /// Runtime item value used by Modal..
        item: usize,
    },
    /// `Picker` option in the `Modal` Smash policy state.
    Picker {
        /// Runtime slot value used by Modal..
        slot: usize,
        /// Runtime group value used by Modal..
        group: usize,
        /// Runtime item value used by Modal..
        item: usize,
    },
}
/// `TOOLS` is a configuration value used by the Smash runtime.
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
/// `CHOOSE_ACTION` is a configuration value used by the Smash runtime.
const CHOOSE_ACTION: usize = 0;
/// `INSERT_BEFORE` is a configuration value used by the Smash runtime.
const INSERT_BEFORE: usize = 1;
/// `INSERT_AFTER` is a configuration value used by the Smash runtime.
const INSERT_AFTER: usize = 2;
/// `DUPLICATE` is a configuration value used by the Smash runtime.
const DUPLICATE: usize = 3;
/// `MOVE_UP` is a configuration value used by the Smash runtime.
const MOVE_UP: usize = 4;
/// `MOVE_DOWN` is a configuration value used by the Smash runtime.
const MOVE_DOWN: usize = 5;
/// `DELETE` is a configuration value used by the Smash runtime.
const DELETE: usize = 6;
/// `UNDO` is a configuration value used by the Smash runtime.
const UNDO: usize = 7;
/// `COPY_RECIPE` is a configuration value used by the Smash runtime.
const COPY_RECIPE: usize = 8;
/// `COPY_STEP` is a configuration value used by the Smash runtime.
const COPY_STEP: usize = 9;
/// `PASTE_BEFORE` is a configuration value used by the Smash runtime.
const PASTE_BEFORE: usize = 10;
/// `PASTE_AFTER` is a configuration value used by the Smash runtime.
const PASTE_AFTER: usize = 11;
/// `RECIPE_OPTIONS` is a configuration value used by the Smash runtime.
const RECIPE_OPTIONS: usize = 12;
/// `NO_ROOM` is a configuration value used by the Smash runtime.
const NO_ROOM: &str = "All 32 slots are occupied; remove a step first";
/// `GROUPS` is a configuration value used by the Smash runtime.
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
/// `SAVED_INPUTS_GROUP` is a configuration value used by the Smash runtime.
const SAVED_INPUTS_GROUP: usize = GROUPS.len() - 1;
/// `COPY_GROUPS` is a configuration value used by the Smash runtime.
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
    "L3 button",
    "R3 button",
    "Generic templates",
];
/// `TEMPLATE_GROUP` is a configuration value used by the Smash runtime.
const TEMPLATE_GROUP: usize = COPY_GROUPS.len() - 1;
/// `DPAD` is a configuration value used by the Smash runtime.
const DPAD: u32 = DPAD_UP | DPAD_DOWN | DPAD_LEFT | DPAD_RIGHT;
/// Picker category of a preset action.
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
/// Returns the `candidates` result.
/// Gets the candidates result.
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
        .filter_map(|(i, (_, action))| {
            (group(*action) == category && ledge_free(*action)).then_some(i)
        })
        .collect()
}
/// Returns the `changed_steps` result.
/// Gets the changed steps result.
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
/// Determines whether the `vertical` condition holds.
/// Checks whether vertical.
fn vertical(pad: u32) -> bool {
    pad == DPAD_UP || pad == DPAD_DOWN
}
/// Move `item` through `count` entries on Up/Down, wrapping at both ends.
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
/// transfers only to another shield button input.
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
        } => *frames = if held == 0 && *buttons != MAPPER_ZR { 4 } else { held },
        Action::OpponentInput { frames, .. } | Action::Controller { frames, .. } => {
            *frames = if held == 0 { 4 } else { held }
        }
        _ => {}
    }
}
impl RecipeEditor {
    /// Updates the `close_commands` state using the supplied inputs.
    pub fn close_commands(&mut self) {
        self.modal = None;
        self.return_modal = None;
        self.manage_requested = false;
        self.command_hold.reset();
        self.clear_copy();
    }
    /// Clears the copy.
    pub fn clear_copy(&mut self) {
        self.copy_sources = Vec::new();
        self.copy_templates = Vec::new();
        self.template_apply = None;
        self.copy_requested = false;
        self.copy_apply = None;
    }
    /// Takes the manage request.
    pub fn take_manage_request(&mut self) -> bool {
        std::mem::take(&mut self.manage_requested)
    }
    /// Copies the step.
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
    /// Determines whether the `paste_step` condition holds.
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
    /// Takes the copy request.
    pub fn take_copy_request(&mut self) -> bool {
        std::mem::take(&mut self.copy_requested)
    }
    /// Sets the copy sources.
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
    /// Sets the copy templates.
    pub fn set_copy_templates(&mut self, templates: &[crate::sequence_config::Template]) {
        self.copy_templates = templates.iter().map(|t| t.name.clone()).collect();
        if let Some(Modal::Copy { group, item }) = self.modal.as_mut() {
            if self.copy_sources.is_empty() && !self.copy_templates.is_empty() {
                *group = TEMPLATE_GROUP;
                *item = 0;
            }
        }
    }
    /// Copies the sources in.
    fn copy_sources_in(&self, group: usize) -> impl Iterator<Item = &CopySource> {
        self.copy_sources
            .iter()
            .filter(move |s| s.slot as usize == group)
    }
    /// Copies the count.
    fn copy_count(&self, group: usize) -> usize {
        if group == TEMPLATE_GROUP {
            self.copy_templates.len()
        } else {
            self.copy_sources_in(group).count()
        }
    }
    /// Takes the template apply.
    pub fn take_template_apply(&mut self) -> Option<usize> {
        self.template_apply.take()
    }
    /// Applies the generic template.
    pub fn apply_generic_template(
        &mut self,
        destination: (usize, usize),
        sequence: Sequence,
    ) -> bool {
        // This key cannot alias a fighter recipe or its unsaved draft.
        self.apply_template(destination, (usize::MAX, 0), sequence)
    }
    /// Takes the copy apply.
    pub fn take_copy_apply(
        &mut self,
    ) -> Option<(crate::fighter_profiles::presets::PresetSlot, usize)> {
        self.copy_apply.take()
    }
    /// Rejects the copy.
    pub fn reject_copy(&mut self) {
        self.last_edit = Some("Source recipe is no longer available; nothing changed".into());
    }
    /// Applies the template.
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
        let Some(compiled) = next.compile().filter(|s| s.len != 0 && s.mode.is_none()) else {
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

    /// Determines whether the `command_active` condition holds.
    pub fn command_active(&self) -> bool {
        self.modal.is_some()
    }
    /// Returns an optional `focused_field` value.
    fn focused_field(&self, key: (usize, usize)) -> Option<Field> {
        let r = self.recipes.get(&key)?;
        Some(focus(&fields(r), self.cursor))
    }
    /// Returns an optional `selected_slot` value.
    fn selected_slot(&self, key: (usize, usize)) -> Option<usize> {
        Some(field_slot(self.focused_field(key)?).unwrap_or(0))
    }
    /// Open a tool, restarting the d-pad repeat timer.
    fn open(&mut self, modal: Modal) {
        self.modal = Some(modal);
        self.command_hold.reset();
    }
    /// Close the open tool and its parent; copy lists and repeat state are left alone.
    fn dismiss(&mut self) {
        self.modal = None;
        self.return_modal = None;
    }
    /// Updates the `open_picker` state using the supplied inputs.
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
    /// Returns an optional `field_index` value.
    fn field_index(&self, key: (usize, usize), target: Field) -> Option<usize> {
        let r = self.recipes.get(&key)?;
        fields(r).iter().position(|f| *f == target)
    }
    /// Updates the `focus_slot` state using the supplied inputs.
    fn focus_slot(&mut self, key: (usize, usize), slot: usize) {
        if self.recipes.contains_key(&key) {
            self.cursor = self.field_index(key, Field::Kind(slot)).unwrap_or(0);
        }
    }
    /// Determines whether the `commit_command` condition holds.
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
    /// Determines whether the `undo_edit` condition holds.
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
    /// Determines whether the `structural` condition holds.
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
                let at = if item == INSERT_BEFORE {
                    slot
                } else {
                    slot + 1
                };
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
    /// True when the held vertical d-pad direction is due to auto-repeat.
    fn command_repeat(&mut self, pad: u32) -> bool {
        self.command_hold.step(if vertical(pad) { pad } else { 0 })
    }
    /// Some = input owned by editor tools, (rebuild UI, recipe changed).
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
    fn idle_input(
        &mut self,
        key: (usize, usize),
        buttons: u32,
        pressed: u32,
    ) -> Option<(bool, bool)> {
        let shoulders = MAPPER_ZL | MAPPER_ZR;
        if buttons & shoulders == shoulders && pressed & shoulders != 0 {
            let slot = self.selected_slot(key)?;
            self.open(Modal::Tools { slot, item: 0 });
            return Some((true, false));
        }
        if buttons & (MAPPER_R | shoulders) == 0
            && pressed & (MAPPER_A | DPAD_LEFT | DPAD_RIGHT) != 0
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
    /// Returns an optional `modal_input` value.
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
            self.command_hold.reset();
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
    /// Returns an optional `buttons_modal` value.
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
    /// Copies the modal.
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
            (
                next_group(group, COPY_GROUPS.len(), forward, has_entries),
                0,
            )
        } else {
            (group, step_item(item, count, nav))
        };
        self.modal = Some(Modal::Copy { group, item });
        Some((true, false))
    }
    /// Returns an optional `tools_modal` value.
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
    /// Returns an optional `picker_modal` value.
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
    /// Applies the kind.
    fn apply_kind(
        &mut self,
        key: (usize, usize),
        slot: usize,
        kind: usize,
    ) -> Option<(bool, bool)> {
        let r = self.recipes.get(&key)?;
        if r.kinds[slot] == kind {
            self.dismiss();
            return Some((true, false));
        }
        let mut action = r.action_for(kind);
        if kind < PRESETS.len()
            && !matches!(
                action,
                Action::Input {
                    buttons: MAPPER_X,
                    ..
                }
            )
        {
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
    /// Returns an optional `command_panel` value.
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
    /// Copies the panel.
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

/// Updates the `group_header` state using the supplied inputs.
/// Updates group header.
fn group_header(panel: &mut RecipePanel, names: &[&str], group: usize) {
    panel.header.push(format!(
        "< {} >  {}/{}",
        names[group],
        group + 1,
        names.len()
    ));
}
/// Updates the `numbered_row` state using the supplied inputs.
/// Updates numbered row.
fn numbered_row(panel: &mut RecipePanel, i: usize, item: usize, text: impl std::fmt::Display) {
    let marker = if i == item { "> " } else { "" };
    panel.body.push(format!("{marker}{:02}  {text}", i + 1));
}
/// Updates the `buttons_panel` state using the supplied inputs.
/// Updates buttons panel.
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
/// Updates the `tools_panel` state using the supplied inputs.
/// Updates tools panel.
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
/// Updates the `picker_panel` state using the supplied inputs.
/// Updates picker panel.
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
#[path = "../../../tests/unit/settings_menu_editor_commands.rs"]
mod tests;

#[cfg(test)]
#[path = "../../../tests/unit/settings_menu_editor_commands_reaction_copy.rs"]
mod reaction_copy_tests;

#[cfg(test)]
#[path = "../../../tests/unit/settings_menu_editor_commands_menu_audit.rs"]
mod menu_audit_tests;

#[cfg(test)]
#[path = "../../../tests/unit/settings_menu_editor_commands_generic_template_copy.rs"]
mod generic_template_copy_tests;
