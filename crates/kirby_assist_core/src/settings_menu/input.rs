//! One mapper sample in, the buttons the menu consumes out. The state machine
//! is split by input mode; each phase either handles the sample and returns its
//! consumed-button mask or declines with `None` so the next phase can run.
use super::*;

/// Buttons the recipe editor and its tool pickers own while open.
///
///  Buttons the recipe editor and its tool pickers own while open.
const EDITOR_BUTTONS: u32 = MAPPER_A | MAPPER_B | MAPPER_ZL | MAPPER_ZR | MAPPER_R;

/// One controller sample shared by every input phase.
///
///  One controller sample shared by every input phase.
#[derive(Clone, Copy)]
struct Sample {
    /// Stores the `buttons` value in `Sample` as `u32`.
    ///
    ///
///  Holds the buttons value used by Sample.
    buttons: u32,
    /// Stores the `pressed` value in `Sample` as `u32`.
    ///
    ///
///  Holds the pressed value used by Sample.
    pressed: u32,
    /// Directions this page owns; none on a lightweight page without the L modifier.
    ///
///  Directions this page owns; none on a lightweight page without the L modifier.
    directions: u32,
    /// Owned directions currently held.
    ///
///  Owned directions currently held.
    pad: u32,
    /// Direction quarantine as it was before this sample.
    ///
///  Direction quarantine as it was before this sample.
    quarantined: u32,
    /// Stores the `toggled` value in `Sample` as `bool`.
    ///
    ///
///  Holds the toggled value used by Sample.
    toggled: bool,
    /// Stores the `editing` value in `Sample` as `bool`.
    ///
    ///
///  Holds the editing value used by Sample.
    editing: bool,
    /// Stores the `allow_editing` value in `Sample` as `bool`.
    ///
    ///
///  Holds the allow editing value used by Sample.
    allow_editing: bool,
    /// Stores the `light_navigation` value in `Sample` as `bool`.
    ///
    ///
///  Holds the light navigation value used by Sample.
    light_navigation: bool,
}
impl Sample {
    /// Every phase hides the owned directions plus Minus and R3 from gameplay.
    ///
///  Every phase hides the owned directions plus Minus and R3 from gameplay.
    fn consume(&self, extra: u32) -> u32 {
        self.directions | MAPPER_MINUS | MENU_STICK_CLICK | extra
    }
    /// Exactly one owned direction, newly pressed.
    ///
///  Exactly one owned direction, newly pressed.
    fn pad_pressed(&self) -> bool {
        self.pad.count_ones() == 1 && self.pressed & self.pad != 0
    }
}

impl MenuSelection {
    /// Minus opens ledge home or closes. R3 edits; page rows navigate directly.
    /// In Training, visible D-pad belongs to navigation. The lightweight menu
    /// requires the held ledge modifier; otherwise custom D-pad inputs work.
    ///
///  Minus opens ledge home or closes. R3 edits; page rows navigate directly. In Training, visible D-pad belongs to navigation. The lightweight menu requires the held ledge modifier; otherwise custom D-pad inputs work.
    pub fn input(&mut self, buttons: u32, presets: &MenuPresets) -> u32 {
        self.input_with_editing(buttons, presets, true)
    }
    /// Runtime scene permission; saved recipes and mode/choice controls remain usable.
    ///
///  Runtime scene permission; saved recipes and mode/choice controls remain usable.
    pub fn input_with_editing(
        &mut self,
        buttons: u32,
        presets: &MenuPresets,
        allow_editing: bool,
    ) -> u32 {
        self.capture_binding_input(buttons, presets);
        let native = self.native_buttons(buttons);
        let ledge_input = native & MAPPER_L != 0;
        let ledge_pressed = ledge_input && !self.binding_ledge_previous.unwrap_or(ledge_input);
        self.binding_ledge_previous = Some(ledge_input);
        // Menu navigation uses its original controls. Hidden gameplay routes
        // only the selected native helpers; custom L/X/L3 cannot double-activate.
        let buttons = if self.menu_hidden && buttons & MAPPER_MINUS == 0 {
            native
        } else {
            buttons
        };
        let cancel_editor = self.sync_scene(presets, allow_editing);
        let mut s = self.begin_sample(presets, buttons, allow_editing, cancel_editor, ledge_pressed);
        if s.toggled {
            self.toggle_menu();
        }
        s.editing = !self.menu_hidden;
        if let Some(mask) = self.lightweight_page_switch(&s) {
            return mask;
        }
        if let Some(mask) = self.mode_shortcuts(presets, &s) {
            return mask;
        }
        if s.editing || s.toggled {
            self.pad_quarantine |= s.pad;
        }
        if let Some(mask) = self.page_shortcuts(&s) {
            return mask;
        }
        if let Some(mask) = self.tuning_input(&s) {
            return mask;
        }
        self.arm_recipe_open(&s);
        let bound = self.process_bindings(
            presets,
            s.buttons,
            s.pressed,
            (!s.editing || self.lightweight && !s.light_navigation) && !s.toggled,
            s.quarantined,
        );
        if s.editing && self.quick_menu {
            return if self.lightweight {
                self.lightweight_page_input(presets, &s, bound)
            } else {
                self.quick_page_input(presets, &s, bound)
            };
        }
        if let Some(mask) = self.editor_input(presets, &s) {
            return mask;
        }
        self.page_input(presets, &s, bound)
    }

    /// Resolve the scene's layout (Training editor vs lightweight selector) on
    /// the first sample after entry or a scene change. Returns whether leaving
    /// Training cancels tools that were still open.
    ///
///  Resolve the scene's layout (Training editor vs lightweight selector) on the first sample after entry or a scene change. Returns whether leaving Training cancels tools that were still open.
    fn sync_scene(&mut self, presets: &MenuPresets, allow_editing: bool) -> bool {
        // Battle entry can reuse a selector prepared during the intro/results.
        // Resolve its default from the actual scene, even if layout did not change.
        let starting_match = std::mem::take(&mut self.match_visibility_pending);
        if starting_match {
            self.menu_hidden = allow_editing;
            self.lightweight_initialized = !allow_editing;
        }
        let cancel_editor = !allow_editing && !self.lightweight && self.training_tools_open();
        let layout_changed = self.lightweight != !allow_editing;
        if layout_changed {
            self.change_layout(presets, allow_editing);
        } else if starting_match && self.lightweight {
            self.open_page(MenuRow::LedgePage);
        }
        cancel_editor
    }
    /// Determines whether the `training_tools_open` condition holds.
    ///
    ///
///  Checks whether training tools open.
    fn training_tools_open(&self) -> bool {
        self.preview_slot.is_some()
            || self.held_binding.is_some()
            || self.recipe_editing
            || self.tuning_page
            || self.mode_editor_origin.is_some()
            || self.managing()
            || self.editor.as_ref().is_some_and(|e| e.command_active())
    }
    /// Updates the `change_layout` state using the supplied inputs.
    ///
    ///
///  Updates change layout.
    fn change_layout(&mut self, presets: &MenuPresets, allow_editing: bool) {
        self.lightweight = !allow_editing;
        if self.lightweight {
            if !self.lightweight_initialized {
                self.menu_hidden = false;
                self.lightweight_initialized = true;
            }
            let restored = self.restore_mode_choices().is_some();
            self.mode_rule_list = false;
            self.mode_rule_cursor = 0;
            self.preview_slot = None;
            self.held_binding = None;
            self.recipe_editing = false;
            self.tuning_page = false;
            self.hold.reset();
            self.release_training_tools(presets);
            self.close_catalog();
            if restored {
                self.refresh_native_bindings(presets);
            }
        }
        if !self.menu_hidden {
            self.open_page(MenuRow::LedgePage);
        }
        self.view_changed();
    }
    /// Per-sample bookkeeping: clear one-shot outputs, take the button edges and
    /// split the directions this page owns from the ones gameplay keeps.
    ///
///  Per-sample bookkeeping: clear one-shot outputs, take the button edges and split the directions this page owns from the ones gameplay keeps.
    fn begin_sample(
        &mut self,
        presets: &MenuPresets,
        buttons: u32,
        allow_editing: bool,
        cancel_editor: bool,
        ledge_pressed: bool,
    ) -> Sample {
        // Cancellation belongs to one mapper sample, not a later observer.
        self.cancel_pending = cancel_editor;
        self.toggle_mash = false;
        self.clear_requests();
        self.button_latch &= self.binding_layout.binding_ownership(self.binding_physical);
        let previous = self.previous.unwrap_or(buttons);
        let pressed = buttons & !previous;
        self.previous = Some(buttons);
        if self.rule_binding_stamp != Some(self.recipe_revision) {
            self.refresh_rule_bindings(presets, buttons);
            self.rule_binding_stamp = Some(self.recipe_revision);
        }
        let light_navigation = self.lightweight && self.lightweight_navigation();
        let directions = if self.lightweight && !light_navigation {
            0
        } else {
            MAPPER_DPAD_BUTTONS
        };
        let quarantined = self.pad_quarantine;
        self.pad_quarantine &= self.binding_layout.binding_ownership(self.binding_physical);
        // With autopilot already ON, L is only the manual walking modifier;
        // it must not reset a running response or reacquire the same ledge.
        if ledge_pressed && !self.ledge_mode() && !self.binding_starts_blocked {
            self.cancel_pending = true; // Held-L activation takes over when toggle is OFF.
        }
        Sample {
            buttons,
            pressed,
            directions,
            pad: buttons & directions,
            quarantined,
            toggled: menu_toggle_pressed(buttons, pressed),
            editing: false,
            allow_editing,
            light_navigation,
        }
    }
    /// Minus opens the ledge page, or closes the menu and everything it owns.
    ///
///  Minus opens the ledge page, or closes the menu and everything it owns.
    fn toggle_menu(&mut self) {
        if self.menu_hidden {
            self.open_page(MenuRow::LedgePage);
            return;
        }
        self.restore_mode_choices();
        self.menu_hidden = true;
        self.tuning_page = false;
        self.recipe_editing = false;
        self.modes_menu = false;
        self.button_menu = false;
        self.close_tools();
        if self.lightweight {
            self.view_changed();
        } else {
            self.changed();
        }
    }
    /// Lightweight R3 switches only between ledge and buttons/assists.
    /// It never allocates an editor or opens list-management tools.
    ///
///  Lightweight R3 switches only between ledge and buttons/assists. It never allocates an editor or opens list-management tools.
    fn lightweight_page_switch(&mut self, s: &Sample) -> Option<u32> {
        if !(self.lightweight
            && s.editing
            && !s.toggled
            && s.pressed & MENU_STICK_CLICK != 0
            && s.pad == 0)
        {
            return None;
        }
        self.open_page(if self.button_menu {
            MenuRow::LedgePage
        } else {
            MenuRow::ButtonsPage
        });
        self.pad_quarantine |= s.buttons & bindings::BUTTON_MASK;
        Some(s.consume(MAPPER_L | (self.pad_quarantine & MAPPER_DPAD_BUTTONS)))
    }
    /// Assist rule lists and the R+Left/Right rule switch inside a rule editor.
    ///
///  Assist rule lists and the R+Left/Right rule switch inside a rule editor.
    fn mode_shortcuts(&mut self, presets: &MenuPresets, s: &Sample) -> Option<u32> {
        if !(s.editing && s.allow_editing && !s.toggled) {
            return None;
        }
        if self.mode_list_input(presets, s.buttons, s.pressed) {
            return Some(s.consume(s.buttons | MENU_L3));
        }
        if s.pressed & (MENU_STICK_CLICK | MENU_L3) != 0
            && self.modes_menu
            && !self.tuning_page
            && Self::mode_trigger(self.row).is_some()
        {
            self.open_mode_list(presets, self.row);
            self.changed();
            return Some(s.consume(MAPPER_L));
        }
        if self.recipe_editing
            && !self.editor.as_ref().is_some_and(|e| e.command_active())
            && s.buttons & MAPPER_R != 0
            && s.pressed & DPAD_SIDES != 0
        {
            let (row, _) = self.mode_editor_origin?;
            self.open_mode_response(presets, row, Some(s.pressed & DPAD_RIGHT != 0));
            self.changed();
            return Some(s.consume(MAPPER_L | MAPPER_R));
        }
        None
    }
    /// A, R3 or a side press on a page link opens that page.
    ///
///  A, R3 or a side press on a page link opens that page.
    fn page_shortcuts(&mut self, s: &Sample) -> Option<u32> {
        if !s.editing || s.toggled || self.tuning_page {
            return None;
        }
        if Self::page_link(self.row) {
            let confirm = !self.lightweight
                && !self.recipe_editing
                && s.pad == 0
                && s.pressed & (MAPPER_A | MENU_STICK_CLICK) != 0;
            let side = s.pad & DPAD_SIDES != 0 && s.pad_pressed();
            if confirm || side {
                self.open_page(self.row);
                self.pad_quarantine |= s.pad | (s.buttons & (MAPPER_A | MENU_STICK_CLICK));
                return Some(s.consume(MAPPER_X | MAPPER_L | MAPPER_A));
            }
        }
        if s.allow_editing
            && self.row == MenuRow::FighterSettings
            && s.pressed & MENU_STICK_CLICK != 0
        {
            self.open_page(MenuRow::FighterSettings);
            return Some(s.consume(MAPPER_L | MAPPER_X | MAPPER_Y));
        }
        None
    }
    /// Returns an optional `tuning_input` value.
    ///
    ///
///  Gets tuning input if it is available.
    fn tuning_input(&mut self, s: &Sample) -> Option<u32> {
        if !(s.editing && s.allow_editing && self.tuning_page) {
            return None;
        }
        self.tuning_row = self.tuning_visible_index(self.tuning_row);
        self.trigger_ready &= !(MAPPER_L | bindings::BUTTON_MASK);
        if s.pressed & MENU_STICK_CLICK != 0 {
            self.open_page(MenuRow::LedgePage);
        } else if s.pad_pressed() {
            if s.pad & (DPAD_UP | DPAD_DOWN) != 0 {
                self.tuning_row = self.next_tuning_row(s.pad == DPAD_DOWN);
            } else {
                self.adjust_tuning(s.pad == DPAD_RIGHT);
            }
            self.changed();
        } else if s.pressed & MAPPER_X != 0 {
            self.tuning.0[self.tuning_row] = None;
            self.recipes_edited("Default restored — Y: Save");
            self.changed();
        } else if s.pressed & MAPPER_Y != 0 {
            self.tuning_save_requested = s.buttons == MAPPER_Y;
            if !self.tuning_save_requested {
                self.save_status = Some("Release other buttons, then press Y to save".into());
            }
            self.changed();
        }
        Some(s.consume(MAPPER_L | MAPPER_X | MAPPER_Y))
    }
    /// Next visible fighter-setting row, wrapping; stays put when none is visible.
    ///
///  Next visible fighter-setting row, wrapping; stays put when none is visible.
    fn next_tuning_row(&self, forward: bool) -> usize {
        let mut row = self.tuning_row;
        for _ in 0..crate::fighter_tuning::COUNT {
            row = cycle(row, crate::fighter_tuning::COUNT, forward);
            if self.tuning_visible(row) {
                return row;
            }
        }
        self.tuning_row
    }
    /// Updates the `adjust_tuning` state using the supplied inputs.
    ///
    ///
///  Updates adjust tuning.
    fn adjust_tuning(&mut self, increase: bool) {
        let i = self.tuning_row;
        let (lo, hi, step) = crate::fighter_tuning::Tuning::bounds(i);
        let value = self.tuning.0[i].unwrap_or(self.tuning_values[i]);
        self.tuning.0[i] = Some((value + if increase { step } else { -step }).clamp(lo, hi));
        self.recipes_edited("Unsaved fighter settings — Y: Save");
    }
    /// R3 on a compact row opens its recipe editor instead of the compact page.
    ///
///  R3 on a compact row opens its recipe editor instead of the compact page.
    fn arm_recipe_open(&mut self, s: &Sample) {
        if s.editing
            && s.allow_editing
            && !self.recipe_editing
            && !self.managing()
            && self.quick_menu
            && !s.toggled
            && self.row.slot().is_some()
            && s.pad == 0
            && s.pressed & MENU_STICK_CLICK != 0
            && s.buttons & (MAPPER_X | MAPPER_Y | MAPPER_PLUS | MAPPER_B | MAPPER_A) == 0
        {
            self.quick_menu = false;
        }
    }

    /// Up/Down hold-repeat on a compact row; true when a repeat is due.
    ///
///  Up/Down hold-repeat on a compact row; true when a repeat is due.
    fn row_repeat(&mut self, pad: u32) -> bool {
        if pad.count_ones() == 1 && pad & (DPAD_UP | DPAD_DOWN) != 0 {
            self.hold.step(pad)
        } else {
            self.hold.reset();
            false
        }
    }
    /// The single direction a compact page acts on this sample (press or repeat).
    ///
///  The single direction a compact page acts on this sample (press or repeat).
    fn compact_direction(&mut self, s: &Sample) -> Option<u32> {
        if s.toggled {
            self.hold.reset();
            return None;
        }
        let repeat = self.row_repeat(s.pad);
        (s.pad.count_ones() == 1 && (s.pressed & s.pad != 0 || repeat)).then_some(s.pad)
    }
    /// The Match-only selector: no preview/editor/save controls; physical face
    /// buttons keep their normal behavior and navigation must not interrupt
    /// held-L ledge work.
    ///
///  The Match-only selector: no preview/editor/save controls; physical face buttons keep their normal behavior and navigation must not interrupt held-L ledge work.
    fn lightweight_page_input(&mut self, presets: &MenuPresets, s: &Sample, bound: u32) -> u32 {
        if s.light_navigation && self.preview_slot.is_some() {
            self.changed();
        }
        if let Some(pad) = self.compact_direction(s) {
            if pad & (DPAD_UP | DPAD_DOWN) != 0 {
                let before = self.row;
                self.navigate_row(pad == DPAD_DOWN);
                if self.row != before {
                    self.view_changed();
                }
            } else if let Some(slot) = self.row.slot() {
                let next = self.cycle_quick_choice(presets, slot, pad == DPAD_RIGHT);
                if next != self.choices[slot as usize] {
                    self.choices[slot as usize] = next;
                    self.changed();
                }
            } else {
                self.adjust_mode(pad == DPAD_RIGHT);
                self.changed();
            }
        }
        self.trigger_ready |= !s.buttons & (MAPPER_L | bindings::BUTTON_MASK);
        self.button_latch
            | bound
            | s.consume(0)
            | ((s.quarantined | self.pad_quarantine) & MAPPER_DPAD_BUTTONS)
    }
    /// The Training compact page: choices per row, L3 test, no editing.
    ///
///  The Training compact page: choices per row, L3 test, no editing.
    fn quick_page_input(&mut self, presets: &MenuPresets, s: &Sample, bound: u32) -> u32 {
        // Ordinary gameplay buttons are not compact-menu controls. A manual
        // action cancels a test so its owned inputs cannot swallow that press.
        let manual = MAPPER_L | MAPPER_Y | MAPPER_A | MAPPER_B | MAPPER_PLUS;
        if self.preview_slot.is_some() && s.pressed & manual != 0 && !self.binding_starts_blocked
        {
            self.changed();
        }
        if let Some(pad) = self.compact_direction(s) {
            if pad & (DPAD_UP | DPAD_DOWN) != 0 {
                self.row = self.page_row_after(pad == DPAD_DOWN);
            } else if let Some(slot) = self.row.slot() {
                self.choices[slot as usize] =
                    self.cycle_quick_choice(presets, slot, pad == DPAD_RIGHT);
            }
            self.changed();
        }
        if !s.toggled
            && self.binding_pressed & MENU_L3 != 0
            && s.pad == 0
            && s.buttons & (manual | MENU_STICK_CLICK) == 0
        {
            self.toggle_compact_test(presets);
        }
        // Compact selection must preserve held-L autopilot, including
        // resuming after navigation. Y remains separate from this page.
        self.trigger_ready |= !s.buttons & (MAPPER_L | bindings::BUTTON_MASK);
        self.trigger_ready &= !MAPPER_Y;
        self.pad_quarantine |= self
            .binding_layout
            .binding_ownership(self.binding_physical & MENU_L3);
        self.button_latch
            | bound
            | s.consume(MENU_L3)
            | ((s.quarantined | self.pad_quarantine) & EDITOR_BUTTONS)
    }
    /// Returns the `page_row_after` result.
    ///
    ///
///  Gets the page row after result.
    fn page_row_after(&self, forward: bool) -> MenuRow {
        let rows = self.page_rows();
        let index = rows.iter().position(|r| *r == self.row).unwrap_or(0);
        rows[cycle(index, rows.len(), forward)]
    }
    /// L3 starts, or stops, a test of the selected recipe without opening the editor.
    ///
///  L3 starts, or stops, a test of the selected recipe without opening the editor.
    fn toggle_compact_test(&mut self, presets: &MenuPresets) {
        let running = self.preview_slot.is_some();
        self.changed();
        if running {
            return;
        }
        if let Some(slot) = self.row.slot() {
            let selected = self.source(presets, slot, self.choices[slot as usize]);
            if selected.is_some_and(|source| source.builtin().is_none()) {
                self.preview_slot = Some(slot);
            }
        }
    }

    /// Recipe editor commands, its Back press and the list manager.
    ///
///  Recipe editor commands, its Back press and the list manager.
    fn editor_input(&mut self, presets: &MenuPresets, s: &Sample) -> Option<u32> {
        if !s.editing {
            return None;
        }
        // A modified Save press must explain why nothing was written. This
        // applies to picker/tools screens too, before they consume the input.
        if s.allow_editing
            && self.recipe_editing
            && !s.toggled
            && s.pressed & MAPPER_Y != 0
            && s.buttons != MAPPER_Y
        {
            self.changed();
            self.save_status = Some("Release other buttons, then press Y to save".into());
            return Some(s.consume(MAPPER_Y | MAPPER_L));
        }
        if self.recipe_editing && !s.toggled && !self.managing() {
            if let Some(mask) = self.editor_command_input(presets, s) {
                return Some(mask);
            }
            if s.pressed & MAPPER_B != 0 {
                self.show_quick();
                self.pad_quarantine |= MAPPER_B;
                return Some(s.consume(MAPPER_L | MAPPER_B));
            }
        }
        if !s.toggled
            && self.row.slot().is_some()
            && (self.managing() || s.pressed & MAPPER_B != 0)
        {
            return Some(self.manage_page_input(presets, s));
        }
        None
    }
    /// Editor-only commands: no allocations or label work on the idle path.
    ///
///  Editor-only commands: no allocations or label work on the idle path.
    fn editor_command_input(&mut self, presets: &MenuPresets, s: &Sample) -> Option<u32> {
        let slot = self.row.slot()?;
        let key = (slot as usize, self.choices[slot as usize]);
        let (ui_changed, mut recipe_changed) = self
            .editor
            .as_mut()
            .and_then(|e| e.command_input(key, s.buttons, s.pressed))?;
        recipe_changed |= self.service_editor_requests(presets, key);
        self.pad_quarantine |= s.buttons & (s.directions | MAPPER_X | MAPPER_Y | EDITOR_BUTTONS);
        self.trigger_ready &= !(MAPPER_L | bindings::BUTTON_MASK);
        self.hold.reset();
        if recipe_changed {
            self.recipes_edited("Unsaved changes — Y: Save");
        }
        if ui_changed {
            self.changed();
        }
        Some(s.consume(MAPPER_L | MAPPER_Y | MAPPER_X | EDITOR_BUTTONS))
    }
    /// Carry out what the editor asked of the menu while handling a command;
    /// true when a template rewrote the recipe.
    ///
///  Carry out what the editor asked of the menu while handling a command; true when a template rewrote the recipe.
    fn service_editor_requests(&mut self, presets: &MenuPresets, key: (usize, usize)) -> bool {
        let mut recipe_changed = false;
        if self.editor.as_mut().is_some_and(|e| e.take_copy_request()) {
            let sources = self.template_sources(presets, key);
            if let Some(editor) = self.editor.as_mut() {
                editor.set_copy_sources(sources);
                editor.set_copy_templates(presets.templates);
            }
        }
        if let Some((source_slot, source_index)) =
            self.editor.as_mut().and_then(|e| e.take_copy_apply())
        {
            let source = self
                .source(presets, source_slot, source_index)
                .map(|s| s.sequence());
            if let Some(editor) = self.editor.as_mut() {
                match source {
                    Some(source) => {
                        recipe_changed |= editor.apply_template(
                            key,
                            (source_slot as usize, source_index),
                            source,
                        );
                    }
                    None => editor.reject_copy(),
                }
            }
        }
        if let Some(index) = self.editor.as_mut().and_then(|e| e.take_template_apply()) {
            if let Some(editor) = self.editor.as_mut() {
                match presets.templates.get(index) {
                    Some(template) => {
                        recipe_changed |= editor.apply_generic_template(key, template.sequence);
                    }
                    None => editor.reject_copy(),
                }
            }
        }
        if self.editor.as_mut().is_some_and(|e| e.take_manage_request()) {
            self.manage_input(presets, MAPPER_B, MAPPER_B);
        }
        recipe_changed
    }
    /// Returns the `manage_page_input` result.
    ///
    ///
///  Gets the manage page input result.
    fn manage_page_input(&mut self, presets: &MenuPresets, s: &Sample) -> u32 {
        let before = self.catalog.as_ref().map_or(0, |c| c.revision);
        self.manage_input(presets, s.buttons, s.pressed);
        let changed =
            before != self.catalog.as_ref().map_or(0, |c| c.revision) || self.delete_requested;
        self.pad_quarantine |= s.pad | (s.buttons & (MAPPER_X | EDITOR_BUTTONS));
        self.trigger_ready &= !(MAPPER_L | MAPPER_Y);
        self.hold.reset();
        if changed {
            self.changed();
        }
        s.consume(MAPPER_L | MAPPER_Y | MAPPER_X | EDITOR_BUTTONS)
    }

    /// Full-page controls: R3 editor toggle, direction navigation, test and save.
    ///
///  Full-page controls: R3 editor toggle, direction navigation, test and save.
    fn page_input(&mut self, presets: &MenuPresets, s: &Sample, bound: u32) -> u32 {
        if s.toggled || s.pressed & MENU_STICK_CLICK != 0 {
            self.hold.reset();
        }
        let cancels_test = MAPPER_L | MAPPER_PLUS | if s.editing { MAPPER_Y } else { 0 };
        if self.preview_slot.is_some() && s.pressed & cancels_test != 0 {
            self.changed();
        }
        if s.editing && s.allow_editing && !s.toggled && s.pressed & MENU_STICK_CLICK != 0 {
            self.toggle_recipe_editor(presets);
        }
        if s.editing && self.recipe_editing && s.pressed & MAPPER_Y != 0 {
            if let Some(mask) = self.save_builtin_default(presets, s) {
                return mask;
            }
        }
        let repeat = self.recipe_hold_repeat(s);
        let recipe_direction = s.editing
            && self.recipe_editing
            && !s.toggled
            && s.pad.count_ones() == 1
            && (s.pressed & s.pad != 0 || repeat);
        if recipe_direction {
            self.recipe_direction(s, repeat);
        } else if s.editing && !s.toggled && s.pad_pressed() {
            self.page_direction(presets, s);
        }
        if let Some(mask) = self.test_press(presets, s) {
            return mask;
        }
        if let Some(mask) = self.save_press(presets, s) {
            return mask;
        }
        self.page_mask(s, bound)
    }
    /// Toggles the recipe editor.
    ///
    ///
///  Toggles the recipe editor.
    fn toggle_recipe_editor(&mut self, presets: &MenuPresets) {
        if self.recipe_editing {
            self.mode_cursor_to_current(presets);
            self.show_quick();
        } else if let Some(slot) = self.row.slot() {
            self.open_editor(presets, slot);
            self.trigger_ready &= !(MAPPER_L | MAPPER_Y);
            self.changed();
        }
    }
    /// Y on a built-in action saves it as the startup default; there is no recipe to write.
    ///
///  Y on a built-in action saves it as the startup default; there is no recipe to write.
    fn save_builtin_default(&mut self, presets: &MenuPresets, s: &Sample) -> Option<u32> {
        let slot = self.row.slot()?;
        let selected = self.source(presets, slot, self.choices[slot as usize]);
        if !selected.is_some_and(|source| source.builtin().is_some()) {
            return None;
        }
        self.save_requested = s.buttons == MAPPER_Y;
        self.default_requested = self.save_requested;
        self.changed();
        if !self.save_requested {
            self.save_status = Some("Release other buttons, then press Y to save".into());
        }
        Some(s.consume(MAPPER_Y))
    }
    /// Determines whether the `recipe_hold_repeat` condition holds.
    ///
    ///
///  Checks whether recipe hold repeat.
    fn recipe_hold_repeat(&mut self, s: &Sample) -> bool {
        if s.editing
            && self.recipe_editing
            && !s.toggled
            && s.pad.count_ones() == 1
            && s.pressed & MENU_STICK_CLICK == 0
        {
            self.hold.step(s.pad)
        } else {
            self.hold.reset();
            false
        }
    }
    /// Left/Right adjust the focused editor field, Up/Down move between fields.
    ///
///  Left/Right adjust the focused editor field, Up/Down move between fields.
    fn recipe_direction(&mut self, s: &Sample, repeat: bool) {
        let Some(slot) = self.row.slot() else {
            return;
        };
        let key = (slot as usize, self.choices[slot as usize]);
        let mut edited = false;
        if let Some(editor) = self.editor.as_mut() {
            if s.pad & DPAD_SIDES != 0 {
                edited = (!repeat || editor.repeatable(key))
                    && editor.adjust(key, s.pad == DPAD_RIGHT);
            } else {
                editor.navigate(key, s.pad == DPAD_DOWN);
            }
        }
        if edited {
            self.recipes_edited("Unsaved changes — Y: Save");
        }
        self.trigger_ready &= !(MAPPER_L | MAPPER_Y);
        self.changed();
    }
    /// Outside the editor one axis moves between rows and the other changes the
    /// row's value; assist pages swap the axes.
    ///
///  Outside the editor one axis moves between rows and the other changes the row's value; assist pages swap the axes.
    fn page_direction(&mut self, presets: &MenuPresets, s: &Sample) {
        let vertical = s.pad & (DPAD_UP | DPAD_DOWN) != 0;
        let positive = s.pad & (DPAD_DOWN | DPAD_RIGHT) != 0;
        if vertical == self.modes_menu {
            self.navigate_row(positive);
        } else if let Some(slot) = self.row.slot() {
            self.choices[slot as usize] = self.cycle_choice(presets, slot, positive);
        } else {
            self.adjust_mode(positive);
        }
        self.changed();
    }
    /// X starts or stops a test of the selected recipe.
    ///
///  X starts or stops a test of the selected recipe.
    fn test_press(&mut self, presets: &MenuPresets, s: &Sample) -> Option<u32> {
        if !(s.editing
            && !s.toggled
            && s.pressed & MAPPER_X != 0
            && s.buttons & (MAPPER_L | MAPPER_Y | MAPPER_PLUS) == 0
            && s.pad == 0
            && s.pressed & MENU_STICK_CLICK == 0)
        {
            return None;
        }
        let slot = self.row.slot()?;
        let running = self.preview_slot.is_some();
        if !running {
            if let Some(error) = self.recipe_input_error(presets) {
                self.changed();
                self.save_status = Some(format!("Cannot test: {error}"));
                return Some(s.consume(MAPPER_X | MAPPER_L));
            }
        }
        self.changed();
        if !running {
            self.preview_slot = Some(slot);
        }
        self.trigger_ready &= !(MAPPER_L | MAPPER_Y);
        None
    }
    /// A clean Y press requests a save when the recipe validates.
    ///
///  A clean Y press requests a save when the recipe validates.
    fn save_press(&mut self, presets: &MenuPresets, s: &Sample) -> Option<u32> {
        if !(s.editing
            && self.recipe_editing
            && !s.toggled
            && s.pressed & MAPPER_Y != 0
            && s.buttons & (MAPPER_L | MAPPER_X | MAPPER_PLUS | MENU_STICK_CLICK) == 0
            && s.pad == 0)
        {
            return None;
        }
        self.changed();
        if let Some(error) = self.recipe_input_error(presets) {
            self.save_status = Some(format!("Cannot save: {error}"));
            return Some(s.consume(MAPPER_Y | MAPPER_L));
        }
        self.save_requested = self.row.slot().is_some();
        self.trigger_ready &= !(MAPPER_L | MAPPER_Y);
        None
    }
    /// Buttons the page consumes after all phases declined the sample.
    ///
///  Buttons the page consumes after all phases declined the sample.
    fn page_mask(&mut self, s: &Sample, bound: u32) -> u32 {
        if s.editing && self.row.slot().is_some() {
            self.pad_quarantine |= s.buttons & MAPPER_X;
        }
        // The editor's B is handled by its own phase above.
        let editor_buttons = if s.editing && self.recipe_editing {
            EDITOR_BUTTONS & !MAPPER_B
        } else {
            0
        };
        self.pad_quarantine |= s.buttons & editor_buttons;
        self.trigger_ready |= !s.buttons & (MAPPER_L | bindings::BUTTON_MASK);
        // Visible directions and their release edges cannot reach gameplay.
        // Hidden Left/Right/Down are reserved separately by the enabled tools.
        let held = s.quarantined | self.pad_quarantine;
        let mut mask = bound
            | editor_buttons
            | self.button_latch
            | MAPPER_MINUS
            | (held & (MAPPER_X | MAPPER_A | MAPPER_B));
        if s.editing {
            mask |= s.directions | MAPPER_Y | MAPPER_L | MENU_STICK_CLICK;
            if self.row.slot().is_some() {
                mask |= MAPPER_X;
            }
        } else {
            mask |= held;
        }
        mask
    }
}
