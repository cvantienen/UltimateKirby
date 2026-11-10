//! Fighter-specific sequences and explicitly assigned reaction toggle groups.
use super::*;
use crate::button_bindings::{Builtin, Mode};
use crate::manual_controls::TURBO_TRIGGER;
use crate::native_input::{ControlLayout, MAPPER_STICK_CLICKS, SEQUENCE_JUMP_BUTTONS};
use crate::sequence_config::{Action, Sequence};

/// Defines the `BUTTON_MASK` constant used by this module.
///
/// Provides the BUTTON MASK constant value.
pub const BUTTON_MASK: u32 = MAPPER_X | MAPPER_Y | MAPPER_DPAD_BUTTONS | MAPPER_STICK_CLICKS;

/// A literal one-frame jump input. Damage/tech safety routes it as input
/// instead of replacing the safety owner with a tape.
///
///  A literal one-frame jump input. Damage/tech safety routes it as input instead of replacing the safety owner with a tape.
/// ### Arguments
/// - `s` (`&Sequence`): Represents the s; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
fn is_single_jump_tape(s: &Sequence) -> bool {
    s.start_delay == 0
        && !s.ground_only
        && !s.while_held
        && !s.reaction.active()
        && matches!(
            s.actions(),
            [Action::Input { buttons, stick, frames: 1, .. }]
                if *buttons != 0
                    && *buttons & !SEQUENCE_JUMP_BUTTONS == 0
                    && *stick == [0.; 2]
        )
}

impl MenuSelection {
    /// Determines whether the `effective_mode` condition holds.
    ///
    ///
    ///  Checks whether effective mode.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `_mode` (`Mode`): Represents the mode; the function uses it as input to the operation described above.
    /// - `base` (`bool`): Represents the base; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn effective_mode(&self, _mode: Mode, base: bool) -> bool {
        base
    }
    /// Determines whether the `ledge_mode` condition holds.
    ///
    ///
    ///  Checks whether ledge mode.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn ledge_mode(&self) -> bool {
        self.auto_ledge
    }
    /// Determines whether the `ledge_trump_held` condition holds.
    ///
    ///
    ///  Checks whether ledge trump held.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn ledge_trump_held(&self) -> bool {
        !self.cancel_pending
            && !self.binding_starts_blocked
            && self.menu_hidden
            && !self.recipe_editing
            && !self.managing()
            && !self.tuning_page
            && (self.binding_physical
                & self.trump_sources
                & !self
                    .binding_layout
                    .physical_buttons(self.button_latch | self.pad_quarantine))
                != 0
    }
    /// Returns the `process_bindings` result.
    ///
    ///
    ///  Gets the process bindings result.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `p` (`&MenuPresets`): Represents the p; the function uses it as input to the operation described above.
    /// - `buttons` (`u32`): Represents the controller buttons being sampled; the function uses it as input to the operation described above.
    /// - `pressed` (`u32`): Represents the pressed; the function uses it as input to the operation described above.
    /// - `available` (`bool`): Represents the available; the function uses it as input to the operation described above.
    /// - `quarantined` (`u32`): Represents the quarantined; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `u32` value.
    pub(super) fn process_bindings(
        &mut self,
        p: &MenuPresets,
        buttons: u32,
        pressed: u32,
        available: bool,
        quarantined: u32,
    ) -> u32 {
        let physical = self.binding_physical;
        let exclusive = MAPPER_X | MAPPER_Y | MAPPER_DPAD_BUTTONS | self.binding_sources;
        let hold_ended = self.held_binding.is_some_and(|slot| {
            !available
                || physical & slot.button() == 0
                || physical & (MAPPER_MINUS | MAPPER_PLUS) != 0
                || physical & self.ledge_assist_sources != 0
                || (physical & exclusive).count_ones() != 1
        });
        if hold_ended {
            self.changed();
        }
        self.toggle_rule_buttons(buttons, pressed, available, quarantined);
        let sequences = self.start_bound_sequences(p, available, quarantined, exclusive);
        if !available {
            self.binding_ready &= !(physical & BUTTON_MASK);
            return self.builtin_consumed;
        }
        self.rule_button_mask | sequences | self.builtin_consumed
    }
    /// Explicitly assigned rule buttons toggle their reaction group on a clean press.
    ///
    ///  Explicitly assigned rule buttons toggle their reaction group on a clean press.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `buttons` (`u32`): Represents the controller buttons being sampled; the function uses it as input to the operation described above.
    /// - `pressed` (`u32`): Represents the pressed; the function uses it as input to the operation described above.
    /// - `available` (`bool`): Represents the available; the function uses it as input to the operation described above.
    /// - `quarantined` (`u32`): Represents the quarantined; the function uses it as input to the operation described above.
    fn toggle_rule_buttons(&mut self, buttons: u32, pressed: u32, available: bool, quarantined: u32) {
        if self.rule_button_mask == 0 {
            return;
        }
        let armed = available
            && !self.binding_starts_blocked
            && buttons & (MAPPER_L | MAPPER_MINUS | MAPPER_PLUS) == 0
            && (buttons & BUTTON_MASK).count_ones() == 1;
        if !armed {
            return;
        }
        for control in crate::reaction_rules::Control::ALL.into_iter().skip(1) {
            let button = control.button();
            if self.rule_button_mask & button != 0
                && self.trigger_ready & button != 0
                && quarantined & button == 0
                && pressed & button != 0
            {
                self.changed();
                self.rule_toggles ^= control.bit();
                self.button_latch |= button;
            }
        }
    }
    /// Own every button bound to a literal recipe and start the one pressed
    /// cleanly. Returns the owned logical buttons.
    ///
    ///  Own every button bound to a literal recipe and start the one pressed cleanly. Returns the owned logical buttons.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `p` (`&MenuPresets`): Represents the p; the function uses it as input to the operation described above.
    /// - `available` (`bool`): Represents the available; the function uses it as input to the operation described above.
    /// - `quarantined` (`u32`): Represents the quarantined; the function uses it as input to the operation described above.
    /// - `exclusive` (`u32`): Represents the exclusive; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `u32` value.
    fn start_bound_sequences(
        &mut self,
        p: &MenuPresets,
        available: bool,
        quarantined: u32,
        exclusive: u32,
    ) -> u32 {
        let physical = self.binding_physical;
        let layout = self.binding_layout;
        let physical_quarantine = layout.physical_buttons(quarantined);
        let can_start = available
            && !self.binding_starts_blocked
            && physical & (MAPPER_MINUS | MAPPER_PLUS) == 0
            && physical & self.ledge_assist_sources == 0
            && (physical & exclusive).count_ones() == 1;
        let mut bound = 0;
        for slot in PresetSlot::BUTTONS {
            let button = slot.button();
            if self.binding_safety_jump && self.single_jump_sources & button != 0 {
                bound |= layout.binding_ownership(button);
                continue; // Safety routes this literal one-frame jump as input, not a tape.
            }
            if self.rule_button_mask & button != 0 {
                continue;
            }
            // Identity-only checks: never copy/format a full recipe per poll.
            if !matches!(self.selected_builtin(p, slot), Some(None)) {
                continue;
            }
            bound |= layout.binding_ownership(button);
            if can_start
                && self.binding_ready & button != 0
                && physical_quarantine & button == 0
                && physical & button != 0
                && self.binding_pressed & button != 0
            {
                self.start_binding(p, slot);
            }
        }
        bound
    }
    /// Starts the binding.
    ///
    ///
    ///  Starts the binding.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `p` (`&MenuPresets`): Represents the p; the function uses it as input to the operation described above.
    /// - `slot` (`PresetSlot`): Represents the configuration or storage slot being used; the function uses it as input to the operation described above.
    fn start_binding(&mut self, p: &MenuPresets, slot: PresetSlot) {
        self.changed();
        self.preview_slot = Some(slot);
        self.button_latch |= self.binding_layout.binding_ownership(slot.button());
        let key = (slot as usize, self.choices[slot as usize]);
        let hold = self
            .editor
            .as_ref()
            .and_then(|e| e.while_held(key))
            .unwrap_or_else(|| {
                self.source(p, slot, key.1)
                    .is_some_and(|s| s.sequence().while_held)
            });
        if hold {
            self.held_binding = Some(slot);
        }
    }
}

impl MenuSelection {
    /// Existing owners finish before another button can start a sequence.
    /// Still observe releases and menu controls; never queue a held button.
    ///
    ///  Existing owners finish before another button can start a sequence. Still observe releases and menu controls; never queue a held button.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `blocked` (`bool`): Represents the blocked; the function uses it as input to the operation described above.
    pub fn block_binding_starts(&mut self, blocked: bool) {
        self.binding_starts_blocked = blocked;
        self.binding_safety_jump = false;
    }
    /// Ordinary jump inputs must remain bufferable during damage/tech safety.
    /// Other custom tapes remain blocked and cannot replace that safety owner.
    ///
    ///  Ordinary jump inputs must remain bufferable during damage/tech safety. Other custom tapes remain blocked and cannot replace that safety owner.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `enabled` (`bool`): Represents whether the feature or option should be enabled; the function uses it as input to the operation described above.
    pub fn preserve_safety_jump(&mut self, enabled: bool) {
        self.binding_safety_jump = enabled;
    }
    /// Physical binding keys remain physical even with the alternate layout.
    ///
    ///  Physical binding keys remain physical even with the alternate layout.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `physical` (`u32`): Represents the physical; the function uses it as input to the operation described above.
    /// - `layout` (`ControlLayout`): Represents the layout; the function uses it as input to the operation described above.
    pub fn set_binding_input(&mut self, physical: u32, layout: ControlLayout) {
        self.binding_input = Some((physical, layout));
    }
    /// Actual manual gameplay roles after custom bindings. Automatic rule
    /// buttons are explicit commands to their own rule, not manual jump intent.
    ///
    ///  Actual manual gameplay roles after custom bindings. Automatic rule buttons are explicit commands to their own rule, not manual jump intent.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `p` (`&MenuPresets`): Represents the p; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `u32` value.
    pub fn manual_native_actions(&self, p: &MenuPresets) -> u32 {
        let mut actions = self
            .native_buttons(self.binding_layout.policy_buttons(self.binding_physical))
            & !self.rule_button_mask
            & (MAPPER_A | MAPPER_B | MAPPER_R | SEQUENCE_JUMP_BUTTONS);
        if self.binding_layout == ControlLayout::Default
            && self.binding_physical & MAPPER_X != 0
            && self.rule_button_mask & MAPPER_X == 0
            && self.selected_builtin(p, PresetSlot::X) == Some(Some(Builtin::Normal))
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
    /// `None` when the selected choice is unavailable, `Some(None)` for a recipe.
    ///
    ///  None when the selected choice is unavailable, Some(None) for a recipe.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `p` (`&MenuPresets`): Represents the p; the function uses it as input to the operation described above.
    /// - `slot` (`PresetSlot`): Represents the configuration or storage slot being used; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
    fn selected_builtin(
        &self,
        p: &MenuPresets,
        slot: PresetSlot,
    ) -> Option<Option<Builtin>> {
        self.source(p, slot, self.choices[slot as usize])
            .map(|source| source.builtin())
    }
    /// Updates the `capture_binding_input` state using the supplied inputs.
    ///
    ///
    ///  Updates capture binding input.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `buttons` (`u32`): Represents the controller buttons being sampled; the function uses it as input to the operation described above.
    /// - `p` (`&MenuPresets`): Represents the p; the function uses it as input to the operation described above.
    pub(super) fn capture_binding_input(&mut self, buttons: u32, p: &MenuPresets) {
        let (physical, layout) = self
            .binding_input
            .take()
            .unwrap_or((buttons, ControlLayout::Default));
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
            self.button_latch &= !layout.binding_ownership(physical & self.single_jump_sources);
        }
    }
    /// Updates the `refresh_native_bindings` state using the supplied inputs.
    ///
    ///
    ///  Updates refresh native bindings.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `p` (`&MenuPresets`): Represents the p; the function uses it as input to the operation described above.
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
        self.ledge_assist_sources = 0;
        self.turbo_sources = 0;
        self.trump_sources = 0;
        self.builtin_consumed = 0;
        self.binding_sources = 0;
        self.single_jump_sources = 0;
        // Fixed legacy L assistance is not a customizable binding or latch.
        if layout == ControlLayout::Default {
            self.ledge_assist_sources = MAPPER_L;
            self.builtin_consumed = MAPPER_L;
        }
        for slot in PresetSlot::BUTTONS {
            let button = slot.button();
            if self.rule_button_mask & button != 0 {
                continue;
            }
            let Some(kind) = self.selected_builtin(p, slot) else {
                continue;
            };
            let sources = match kind {
                Some(Builtin::Normal) => continue,
                Some(Builtin::LedgeAssist) => &mut self.ledge_assist_sources,
                Some(Builtin::TurboTeabag) => &mut self.turbo_sources,
                Some(Builtin::LedgeTrump) => &mut self.trump_sources,
                None => {
                    self.binding_sources |= button;
                    // Only inspect literal jump tapes when recovering from damage;
                    // ordinary gameplay needs only the existing binding identity.
                    if self.binding_safety_jump
                        && self
                            .source(p, slot, self.choices[slot as usize])
                            .is_some_and(|s| is_single_jump_tape(&s.sequence()))
                    {
                        self.single_jump_sources |= button;
                        self.builtin_consumed |= layout.binding_ownership(button);
                    }
                    continue;
                }
            };
            *sources |= button;
            self.binding_sources |= button;
            self.builtin_consumed |= layout.binding_ownership(button);
        }
    }
    /// Canonical inputs for existing helpers only. Physical game inputs are
    /// untouched until the final overlay consumes the chosen binding source.
    ///
    ///  Canonical inputs for existing helpers only. Physical game inputs are untouched until the final overlay consumes the chosen binding source.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `buttons` (`u32`): Represents the controller buttons being sampled; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `u32` value.
    pub fn native_buttons(&self, buttons: u32) -> u32 {
        let mut result = buttons
            & !(MAPPER_X
                | MAPPER_L
                | TURBO_TRIGGER
                | self.binding_layout.binding_ownership(self.binding_sources));
        result |= buttons & self.rule_button_mask;
        if self.recipe_editing || self.managing() || self.tuning_page {
            return result;
        }
        let reserved = if self.menu_hidden {
            0
        } else {
            MENU_STICK_CLICK
                | if !self.lightweight {
                    MENU_L3 | MAPPER_DPAD_BUTTONS
                } else if self.lightweight_navigation() {
                    MAPPER_DPAD_BUTTONS
                } else {
                    0
                }
        };
        let latched = self
            .binding_layout
            .physical_buttons(self.button_latch | self.pad_quarantine);
        let safety_jumps = if self.binding_safety_jump {
            self.single_jump_sources
        } else {
            0
        };
        // The source always comes from the captured PHYSICAL trace. Using an
        // already-routed logical button here can remap it a second time.
        for (dest, sources, blocked) in [
            (MAPPER_X, safety_jumps, reserved | latched),
            (MAPPER_L, self.ledge_assist_sources, reserved),
            (TURBO_TRIGGER, self.turbo_sources, reserved | latched),
        ] {
            if self.binding_physical & sources & !blocked != 0 {
                result |= dest;
            }
        }
        result
    }
}
