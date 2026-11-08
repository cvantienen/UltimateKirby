//! Fighter-specific sequences and explicitly assigned reaction toggle groups.
use super::*;
use crate::button_bindings::Mode;
pub const BUTTON_MASK: u32 = MAPPER_X | MAPPER_Y | DPAD_UP | DPAD_DOWN | DPAD_SIDES | (3 << 16);
impl MenuSelection {
    pub fn effective_mode(&self, _mode: Mode, base: bool) -> bool {
        base
    }
    pub fn ledge_mode(&self) -> bool {
        self.auto_ledge
    }
    pub fn ledge_trump_held(&self) -> bool {
        !self.cancel_pending
            && !self.binding_starts_blocked
            && self.menu_hidden
            && !self.recipe_editing
            && !self.managing()
            && !self.tuning_page
            && (self.binding_physical
                & self.builtin_sources[3]
                & !self
                    .binding_layout
                    .physical_buttons(self.button_latch | self.pad_quarantine))
                != 0
    }
    pub(super) fn process_bindings(
        &mut self,
        p: &MenuPresets,
        buttons: u32,
        pressed: u32,
        available: bool,
        quarantined: u32,
    ) -> u32 {
        let physical = self.binding_physical;
        let binding_pressed = self.binding_pressed;
        let physical_quarantine = self.binding_layout.physical_buttons(quarantined);
        let exclusive = (MAPPER_X | MAPPER_Y | MAPPER_DPAD_BUTTONS) | self.binding_sources;
        if self.held_binding.is_some_and(|slot| {
            !available
                || physical & slot.button() == 0
                || physical & (MAPPER_MINUS | crate::native_input::MAPPER_PLUS) != 0
                || physical & self.builtin_sources[1] != 0
                || (physical & exclusive).count_ones() != 1
        }) {
            self.changed();
        }
        let mut bound = self.rule_button_mask;
        if self.rule_button_mask != 0 {
            for control in crate::reaction_rules::Control::ALL.into_iter().skip(1) {
                let button = control.button();
                if self.rule_button_mask & button == 0 {
                    continue;
                }
                let eligible = available
                    && !self.binding_starts_blocked
                    && buttons & (MAPPER_L | MAPPER_MINUS | crate::native_input::MAPPER_PLUS) == 0
                    && (buttons & BUTTON_MASK).count_ones() == 1
                    && self.trigger_ready & button != 0
                    && quarantined & button == 0;
                if eligible && pressed & button != 0 && buttons & button != 0 {
                    self.changed();
                    self.rule_toggles ^= control.bit();
                    self.button_latch |= button;
                }
            }
        }
        for slot in PresetSlot::BUTTONS {
            let button = slot.button();
            if self.binding_safety_jump && self.single_jump_sources & button != 0 {
                bound |= self.binding_layout.binding_ownership(button);
                continue; // Safety routes this literal one-frame jump as input, not a tape.
            }
            if self.rule_button_mask & button != 0 {
                continue;
            }
            let list = p.list(slot);
            let index = self.choices[slot as usize];
            let Some(kind) = self.selected_builtin(p, slot) else {
                continue;
            };
            if kind.is_some() {
                continue;
            }
            if index == list.options.len() {
                continue;
            } // Normal button.
              // Identity-only checks: never copy/format a full recipe per poll.
            if list
                .options
                .get(index)
                .is_some_and(|c| c.sequence.mode.is_some())
            {
                continue;
            }
            if index > list.options.len() + 1 {
                let valid_draft = self
                    .catalog
                    .as_ref()
                    .and_then(|c| c.drafts.get(&(slot as usize, index)))
                    .is_some_and(|d| !d.deleted && d.sequence.mode.is_none());
                if !valid_draft {
                    continue;
                }
            }
            bound |= self.binding_layout.binding_ownership(button);
            let eligible = available
                && !self.binding_starts_blocked
                && physical & (MAPPER_MINUS | crate::native_input::MAPPER_PLUS) == 0
                && physical & self.builtin_sources[1] == 0
                && (physical & exclusive).count_ones() == 1
                && self.binding_ready & button != 0
                && physical_quarantine & button == 0;
            if eligible && physical & button != 0 && binding_pressed & button != 0 {
                self.changed();
                self.preview_slot = Some(slot);
                self.button_latch |= self.binding_layout.binding_ownership(button);
                let key = (slot as usize, index);
                let hold = self
                    .editor
                    .as_ref()
                    .and_then(|e| e.while_held(key))
                    .unwrap_or_else(|| {
                        self.choice(p, slot, index)
                            .is_some_and(|c| c.sequence.while_held)
                    });
                if hold {
                    self.held_binding = Some(slot);
                }
            }
        }
        if !available {
            self.binding_ready &= !(physical & BUTTON_MASK);
        }
        if available {
            bound | self.builtin_consumed
        } else {
            self.builtin_consumed
        }
    }
}

impl MenuSelection {
    /// Existing owners finish before another button can start a sequence.
    /// Still observe releases and menu controls; never queue a held button.
    pub fn block_binding_starts(&mut self, blocked: bool) {
        self.binding_starts_blocked = blocked;
        self.binding_safety_jump = false;
    }
    /// Ordinary jump inputs must remain bufferable during damage/tech safety.
    /// Other custom tapes remain blocked and cannot replace that safety owner.
    pub fn preserve_safety_jump(&mut self, enabled: bool) {
        self.binding_safety_jump = enabled;
    }
    /// Physical binding keys remain physical even with the alternate layout.
    pub fn set_binding_input(&mut self, physical: u32, layout: crate::native_input::ControlLayout) {
        self.binding_input = Some((physical, layout));
    }
    /// Actual manual gameplay roles after custom bindings. Automatic rule
    /// buttons are explicit commands to their own rule, not manual jump intent.
    pub fn manual_native_actions(&self, p: &MenuPresets) -> u32 {
        use crate::native_input::*;
        let mut actions = self
            .native_buttons(self.binding_layout.policy_buttons(self.binding_physical))
            & !self.rule_button_mask
            & (MAPPER_A | MAPPER_B | MAPPER_R | SEQUENCE_JUMP_BUTTONS);
        if self.binding_layout == ControlLayout::Default
            && self.binding_physical & MAPPER_X != 0
            && self.rule_button_mask & MAPPER_X == 0
            && self.selected_builtin(p, PresetSlot::X)
                == Some(Some(crate::button_bindings::Builtin::Normal))
        {
            actions |= MAPPER_X;
        }
        if self.binding_layout == ControlLayout::Brother
            && self.binding_physical & (MAPPER_R | MAPPER_ZR) != 0
        {
            actions |= MAPPER_X;
        }
        actions
    }
    fn selected_builtin(
        &self,
        p: &MenuPresets,
        slot: PresetSlot,
    ) -> Option<Option<crate::button_bindings::Builtin>> {
        let index = self.choices[slot as usize];
        if let Some(d) = self
            .catalog
            .as_ref()
            .and_then(|c| c.drafts.get(&(slot as usize, index)))
        {
            return (!d.deleted && d.sequence.mode.is_none()).then_some(d.sequence.builtin);
        }
        let options = p.list(slot).options;
        if let Some(c) = options.get(index) {
            return c.sequence.mode.is_none().then_some(c.sequence.builtin);
        }
        if index == options.len() + 1 {
            return Some(None);
        }
        crate::button_bindings::Builtin::ALL
            .into_iter()
            .find(|b| index == options.len() + b.template_offset())
            .map(Some)
    }
    pub(super) fn capture_binding_input(&mut self, buttons: u32, p: &MenuPresets) {
        let (physical, layout) = self
            .binding_input
            .take()
            .unwrap_or((buttons, crate::native_input::ControlLayout::Default));
        self.binding_layout = layout;
        self.binding_physical = physical;
        self.binding_pressed = physical & !self.binding_previous.unwrap_or(physical);
        self.binding_previous = Some(physical);
        self.binding_ready |= !physical & BUTTON_MASK;
        self.refresh_native_bindings(p);
        if self.binding_starts_blocked {
            self.binding_ready &= !physical;
            // trigger_ready also owns the LOGICAL L destination. Clearing it
            // from physical held keys cancels L's own committed ledge sequence.
            // Only legacy rule-button starts use this extra rearm mask.
            self.trigger_ready &= !(layout.binding_ownership(physical) & self.rule_button_mask);
            let continuing_hop = if self.binding_safety_jump {
                self.single_jump_sources
            } else {
                0
            };
            self.button_latch |= layout.binding_ownership(physical & BUTTON_MASK & !continuing_hop);
        }
        if self.binding_safety_jump {
            self.button_latch &= !layout
                .binding_ownership(physical & (self.builtin_sources[0] | self.single_jump_sources));
        }
    }
    pub fn refresh_native_bindings(&mut self, p: &MenuPresets) {
        let layout = self.binding_layout;
        let stamp = (
            layout,
            self.choices,
            self.recipe_revision,
            self.rule_button_mask,
            self.binding_safety_jump,
        );
        if self.binding_roles_stamp == Some(stamp) {
            return;
        }
        self.binding_roles_stamp = Some(stamp);
        #[cfg(test)]
        {
            self.binding_roles_rebuilds += 1;
        }
        self.builtin_sources = [0; 4];
        self.builtin_consumed = 0;
        self.binding_sources = 0;
        self.single_jump_sources = 0;
        // Fixed legacy L assistance is not a customizable binding or latch.
        if layout == crate::native_input::ControlLayout::Default {
            self.builtin_sources[1] = MAPPER_L;
            self.builtin_consumed = MAPPER_L;
        }
        for slot in PresetSlot::BUTTONS {
            if self.rule_button_mask & slot.button() != 0 {
                continue;
            }
            let Some(kind) = self.selected_builtin(p, slot) else {
                continue;
            };
            match kind {
                Some(crate::button_bindings::Builtin::Normal) => {}
                Some(builtin) => {
                    self.binding_sources |= slot.button();
                    self.builtin_consumed |= layout.binding_ownership(slot.button());
                    let index = match builtin {
                        crate::button_bindings::Builtin::LedgeAssist => 1,
                        crate::button_bindings::Builtin::TurboTeabag => 2,
                        crate::button_bindings::Builtin::LedgeTrump => 3,
                        crate::button_bindings::Builtin::Normal => continue,
                    };
                    self.builtin_sources[index] |= slot.button();
                }
                None => {
                    self.binding_sources |= slot.button();
                    // Only inspect literal jump tapes when recovering from damage;
                    // ordinary gameplay needs only the existing binding identity.
                    if self.binding_safety_jump {
                        let sequence = self
                            .catalog
                            .as_ref()
                            .and_then(|c| {
                                c.drafts.get(&(slot as usize, self.choices[slot as usize]))
                            })
                            .map(|d| &d.sequence)
                            .or_else(|| {
                                p.list(slot)
                                    .options
                                    .get(self.choices[slot as usize])
                                    .map(|c| c.sequence)
                            });
                        if sequence.is_some_and(|s|s.start_delay==0 && !s.ground_only && !s.while_held
                            && !s.reaction.active() && matches!(s.actions(),[crate::sequence_config::Action::Input{buttons,stick,frames:1,..}]
                                if *buttons!=0 && *buttons & !crate::native_input::SEQUENCE_JUMP_BUTTONS==0 && *stick==[0.;2])) {
                            self.single_jump_sources|=slot.button();
                            self.builtin_consumed|=layout.binding_ownership(slot.button());
                        }
                    }
                }
            }
        }
    }
    /// Canonical inputs for existing helpers only. Physical game inputs are
    /// untouched until the final overlay consumes the chosen binding source.
    pub fn native_buttons(&self, buttons: u32) -> u32 {
        use crate::manual_controls::TURBO_TRIGGER;
        let mut result = buttons
            & !(MAPPER_X
                | MAPPER_L
                | TURBO_TRIGGER
                | self.binding_layout.binding_ownership(self.binding_sources));
        result |= buttons & self.rule_button_mask;
        // The source always comes from the captured PHYSICAL trace. Using an
        // already-routed logical button here can remap it a second time.
        let held = self.binding_physical;
        let latched = self
            .binding_layout
            .physical_buttons(self.button_latch | self.pad_quarantine);
        let enabled = !self.recipe_editing && !self.managing() && !self.tuning_page;
        if enabled {
            let reserved = if self.menu_hidden {
                0
            } else {
                MENU_STICK_CLICK
                    | if self.lightweight {
                        if self.lightweight_navigation() {
                            MAPPER_DPAD_BUTTONS
                        } else {
                            0
                        }
                    } else {
                        MENU_L3 | MAPPER_DPAD_BUTTONS
                    }
            };
            for (index, dest) in [MAPPER_X, MAPPER_L, TURBO_TRIGGER].into_iter().enumerate() {
                let blocked = reserved | if index != 1 { latched } else { 0 };
                let sources = self.builtin_sources[index]
                    | if index == 0 && self.binding_safety_jump {
                        self.single_jump_sources
                    } else {
                        0
                    };
                if held & sources & !blocked != 0 {
                    result |= dest;
                }
            }
        }
        result
    }
}
