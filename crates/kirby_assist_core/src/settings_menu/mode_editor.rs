//! Separate Training assist rule lists; legacy saved identities remain stable.
use super::*;
use crate::reaction_rules::{Rule, SelfState, Trigger};
impl MenuSelection {
    /// Returns an optional `mode_trigger` value.
    pub(super) fn mode_trigger(row: MenuRow) -> Option<Trigger> {
        match row {
            MenuRow::ShieldPressure => Some(Trigger::EnemyShield),
            MenuRow::SpotPunish => Some(Trigger::SpotDodge),
            MenuRow::RollRules => Some(Trigger::Roll),
            MenuRow::GrabDodge => Some(Trigger::GrabDodged),
            MenuRow::OutOfShield => Some(Trigger::ShieldHit),
            _ => None,
        }
    }
    /// Returns an optional `auto_mode_events` value.
    pub(super) fn auto_mode_events(&self, row: MenuRow) -> Option<&'static [Trigger]> {
        if !self.simplified_reactions {
            return None;
        }
        match row {
            MenuRow::RollRules => Some(&Trigger::AUTO_EVENTS),
            MenuRow::GrabDodge => Some(&[Trigger::AutoGrabDodged]),
            MenuRow::OutOfShield => Some(&[
                Trigger::AutoShieldGrab,
                Trigger::AutoShieldBackup,
            ]),
            _ => None,
        }
    }
    /// Determines whether the `assist_rule_page` condition holds.
    pub fn assist_rule_page(&self) -> bool {
        self.mode_editor_origin.is_some()
    }
    /// Returns an optional `restore_mode_choices` value.
    pub(super) fn restore_mode_choices(&mut self) -> Option<MenuRow> {
        let (row, choices) = self.mode_editor_origin.take()?;
        self.mode_rule_list = false;
        self.mode_rule_cursor = 0;
        self.choices = choices;
        self.row = row;
        Some(row)
    }
    /// Returns the `mode_entries` result.
    pub(super) fn mode_entries(
        &self,
        presets: &MenuPresets,
        row: MenuRow,
    ) -> Vec<(u8, PresetSlot, usize)> {
        let events = self.auto_mode_events(row);
        let Some(trigger) = events
            .and_then(|events| events.first().copied())
            .or_else(|| Self::mode_trigger(row))
        else {
            return vec![];
        };
        let mut result = Vec::new();
        for slot in PresetSlot::ALL {
            for index in self.all_choices(presets, slot) {
                let Some(sequence) = self.recipe_snapshot(presets, slot, index) else {
                    continue;
                };
                let r = sequence.reaction;
                if r.active()
                    && events.map_or(r.trigger.family() == trigger.family(), |events| {
                        events.contains(&r.trigger)
                    })
                {
                    result.push((r.priority, slot, index));
                }
            }
        }
        if events.is_some() {
            result.sort_by_key(|(_, s, i)| {
                self.recipe_snapshot(presets, *s, *i)
                    .and_then(|s| s.reaction.trigger.simple())
                    .unwrap_or(7)
            });
        } else {
            result.sort_by_key(|v| v.0);
        }
        result
    }
    fn missing_auto_events(
        &self,
        presets: &MenuPresets,
        events: &[Trigger],
        entries: &[(u8, PresetSlot, usize)],
    ) -> Vec<Trigger> {
        events
            .iter()
            .copied()
            .filter(|trigger| {
                !entries.iter().any(|(_, slot, index)| {
                    self.recipe_snapshot(presets, *slot, *index)
                        .is_some_and(|sequence| sequence.reaction.trigger == *trigger)
                })
            })
            .collect()
    }

    /// Updates the `open_mode_list` state using the supplied inputs.
    pub(super) fn open_mode_list(&mut self, _presets: &MenuPresets, row: MenuRow) {
        if Self::mode_trigger(row).is_none() {
            return;
        }
        if self.mode_editor_origin.is_none() {
            self.mode_editor_origin = Some((row, self.choices));
        }
        self.mode_rule_list = true;
        self.mode_rule_cursor = 0;
        self.auto_add_event = 0;
        self.recipe_editing = false;
        self.modes_menu = false;
        self.quick_menu = false;
        self.button_menu = false;
        self.menu_hidden = false;
        self.tuning_page = false;
    }
    /// Chooses the mode rule.
    fn choose_mode_rule(&mut self, presets: &MenuPresets, slot: PresetSlot, index: usize) {
        self.row = MenuRow::ALL
            .into_iter()
            .find(|r| r.slot() == Some(slot))
            .unwrap();
        self.choices[slot as usize] = index;
        self.mode_rule_list = false;
        self.modes_menu = false;
        self.quick_menu = false;
        self.button_menu = false;
        self.open_editor(presets, slot);
    }
    /// Adds the mode rule.
    fn add_mode_rule(&mut self, presets: &MenuPresets, row: MenuRow) {
        let Some(mut trigger) = Self::mode_trigger(row) else {
            return;
        };
        if let Some(events) = self.auto_mode_events(row) {
            let entries = self.mode_entries(presets, row);
            let missing = self.missing_auto_events(presets, events, &entries);
            let missing = missing
                .get(self.auto_add_event % missing.len().max(1))
                .copied();
            let Some(event) = missing else {
                self.save_status =
                    Some("One recipe per event: edit the existing Auto react entry".into());
                return;
            };
            trigger = event;
        }
        if self.mode_entries(presets, row).len() >= crate::reaction_rules::MAX_RULES_PER_MODE {
            self.save_status =
                Some("Eight rules in this assist; delete one before adding another".into());
            return;
        }
        let slot = match trigger {
            Trigger::ShieldHit | Trigger::SpotDodge | Trigger::EnemyShield => PresetSlot::Attack,
            Trigger::LedgeRegrab => PresetSlot::Drop,
            _ => PresetSlot::Roll,
        };
        let base = presets.list(slot).options.len() + slot.templates();
        let c = self.catalog.get_or_insert_with(Default::default);
        let next = c
            .drafts
            .keys()
            .filter(|(s, _)| *s == slot as usize)
            .map(|(_, i)| i + 1)
            .max()
            .unwrap_or(base)
            .max(base);
        let mut serial = next;
        let id = loop {
            let id = format!("assist_{}_{serial}", trigger.token());
            if !presets.list(slot).options.iter().any(|v| v.id == id)
                && !c
                    .drafts
                    .iter()
                    .any(|((s, _), d)| *s == slot as usize && d.id == id)
            {
                break id;
            }
            serial += 1;
        };
        let mut sequence = editor::RecipeEditor::custom();
        sequence.reaction = Rule {
            trigger,
            state: SelfState::Grounded,
            ..Rule::default()
        };
        if let Some(event) = trigger.simple() {
            sequence.reaction.max = if event == 2 { 40. } else { 30. };
            sequence.reaction.height = if event == 2 { 20. } else { 10. };
            sequence.ground_only = false;
            if event >= 3 {
                let grab = trigger == Trigger::AutoShieldGrab;
                sequence.reaction.max = match trigger {
                    Trigger::AutoShieldGrab | Trigger::AutoShieldBackup => self
                        .tuning
                        .auto_grab_range(self.y_fighter_kind, self.tuning_values[1]),
                    _ => 30.,
                };
                sequence.reaction.height = if grab || trigger == Trigger::AutoShieldBackup {
                    24.
                } else {
                    10.
                };
                if grab {
                    sequence.len = 1;
                    sequence.actions[0] = crate::sequence_config::Action::Input {
                        buttons: crate::native_input::MAPPER_R,
                        stick: [0.; 2],
                        frames: 4,
                        relative: false,
                    };
                }
            }
        }
        c.drafts.insert(
            (slot as usize, next),
            catalog::Draft {
                id,
                name: if trigger.simple().is_some() {
                    format!("{} sequence", trigger.auto_name())
                } else {
                    format!("{} rule", row.label())
                },
                sequence,
                new: true,
                deleted: false,
                dirty: true,
            },
        );
        self.recipe_revision = self.recipe_revision.wrapping_add(1);
        self.save_status =
            Some("New assist rule: configure conditions/actions, then Y to save".into());
        self.choose_mode_rule(presets, slot, next);
    }
    /// Updates the `open_mode_response` state using the supplied inputs.
    pub(super) fn open_mode_response(
        &mut self,
        presets: &MenuPresets,
        row: MenuRow,
        cycle: Option<bool>,
    ) {
        let entries = self.mode_entries(presets, row);
        if entries.is_empty() {
            self.add_mode_rule(presets, row);
            return;
        }
        let current = entries
            .iter()
            .position(|(_, s, i)| self.row.slot() == Some(*s) && self.choices[*s as usize] == *i)
            .unwrap_or(0);
        let index = cycle.map_or(0, |forward| super::cycle(current, entries.len(), forward));
        self.mode_rule_cursor = index;
        let (_, slot, index) = entries[index];
        self.choose_mode_rule(presets, slot, index);
    }
    /// Updates the `mode_cursor_to_current` state using the supplied inputs.
    pub(super) fn mode_cursor_to_current(&mut self, presets: &MenuPresets) {
        if let Some((row, _)) = self.mode_editor_origin.as_ref() {
            let entries = self.mode_entries(presets, *row);
            if let Some(i) = entries.iter().position(|(_, s, i)| {
                self.row.slot() == Some(*s) && self.choices[*s as usize] == *i
            }) {
                self.mode_rule_cursor = i;
            }
        }
    }
    /// Returns the `mode_rules_panel` result.
    pub(super) fn mode_rules_panel(&self, presets: &MenuPresets) -> RecipePanel {
        let Some((row, _)) = self.mode_editor_origin.as_ref() else {
            return RecipePanel::default();
        };
        let entries = self.mode_entries(presets, *row);
        let simple = self.auto_mode_events(*row).is_some();
        let mut panel = RecipePanel {
            header: vec![
                format!("{} / SEQUENCES", row.label()),
                if simple {
                    "One recipe per event. Commit once; no rule priorities.".into()
                } else {
                    "Priority 1 first; first matching rule runs".into()
                },
            ],
            ..Default::default()
        };
        for (priority, slot, index) in &entries {
            let choice = self.choice(presets, *slot, *index).unwrap();
            let r = self
                .recipe_snapshot(presets, *slot, *index)
                .unwrap()
                .reaction;
            if simple {
                panel.body.push(format!(
                    "{} | {} | Max X {} | Max Y {}",
                    choice.name,
                    r.trigger.token(),
                    r.max,
                    r.height
                ));
                continue;
            }
            panel.body.push(format!(
                "P{priority}  {} | {} | X {}-{} | Y {} | {}",
                choice.name,
                r.trigger.token(),
                r.min,
                r.max,
                r.height,
                r.side.token()
            ));
        }
        if let Some(events) = self.auto_mode_events(*row) {
            let missing = self.missing_auto_events(presets, events, &entries);
            panel.body.push(
                missing
                    .get(self.auto_add_event % missing.len().max(1))
                    .map_or("All event sequences configured".into(), |t| {
                        format!("+ Add {} (Left/Right: choose)", t.auto_name())
                    }),
            );
        } else {
            panel.body.push("+ Add rule".into());
        }
        panel.focus = Some(self.mode_rule_cursor.min(entries.len()));
        panel
            .footer
            .push("Up/Down: rule | A/R3: edit | X: add | Y: manage | B/L3: back".into());
        panel.footer.push(if simple {
            "Roll/spot/air: Auto react toggle; grabs/backup: Automatic grabs; grab-dodge follow-up: Grab avoidance".into()
        } else {
            "Rules run from this assist's ON/OFF setting; separate from ledge choices".into()
        });
        if let Some(status) = &self.save_status {
            panel.footer.push(status.clone());
        }
        panel
    }
    /// Determines whether the `mode_list_input` condition holds.
    pub(super) fn mode_list_input(
        &mut self,
        presets: &MenuPresets,
        buttons: u32,
        pressed: u32,
    ) -> bool {
        use crate::native_input::{MAPPER_A as A, MAPPER_B as B};
        let Some((row, _)) = self.mode_editor_origin.as_ref() else {
            return false;
        };
        let row = *row;
        if self.recipe_editing
            && !self.managing()
            && !self.editor.as_ref().is_some_and(|e| e.command_active())
            && pressed & MENU_L3 != 0
        {
            self.mode_cursor_to_current(presets);
            self.show_quick();
            return true;
        }
        if !self.mode_rule_list || self.managing() {
            return false;
        }
        if pressed
            & (A | B
                | MAPPER_X
                | MAPPER_Y
                | MENU_STICK_CLICK
                | MENU_L3
                | DPAD_UP
                | DPAD_DOWN
                | DPAD_LEFT
                | DPAD_RIGHT)
            == 0
        {
            return true;
        }
        let entries = self.mode_entries(presets, row);
        self.mode_rule_cursor = self.mode_rule_cursor.min(entries.len());
        if self.auto_mode_events(row).is_some()
            && self.mode_rule_cursor == entries.len()
            && pressed & (DPAD_LEFT | DPAD_RIGHT) != 0
        {
            let missing = self
                .auto_mode_events(row)
                .unwrap()
                .len()
                .saturating_sub(entries.len());
            if missing > 0 {
                self.auto_add_event = cycle(
                    self.auto_add_event % missing,
                    missing,
                    pressed & DPAD_RIGHT != 0,
                );
            }
        } else if pressed & (B | MENU_L3) != 0 {
            self.restore_mode_choices();
            self.modes_menu = true;
            self.row = row;
        } else if pressed & DPAD_DOWN != 0 {
            self.mode_rule_cursor = (self.mode_rule_cursor + 1) % (entries.len() + 1);
        } else if pressed & DPAD_UP != 0 {
            self.mode_rule_cursor = (self.mode_rule_cursor + entries.len()) % (entries.len() + 1);
        } else if pressed & MAPPER_X != 0
            || pressed & (A | MENU_STICK_CLICK) != 0 && self.mode_rule_cursor == entries.len()
        {
            self.add_mode_rule(presets, row);
        } else if pressed & (A | MENU_STICK_CLICK | MAPPER_Y) != 0 {
            if let Some((_, slot, index)) = entries.get(self.mode_rule_cursor).copied() {
                self.choose_mode_rule(presets, slot, index);
                if pressed & MAPPER_Y != 0 {
                    self.mode_rule_list = true;
                    self.recipe_editing = false;
                    self.manage_input(presets, B, B);
                }
            }
        }
        self.pad_quarantine |= buttons & (DPAD_SIDES | DPAD_UP | DPAD_DOWN | bindings::BUTTON_MASK);
        self.changed();
        true
    }
}

#[cfg(test)]
#[path = "../../tests/unit/settings_menu_mode_editor.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/unit/settings_menu_mode_editor_event_family.rs"]
mod event_family_tests;

#[cfg(test)]
#[path = "../../tests/unit/settings_menu_mode_editor_simple_menu.rs"]
mod simple_menu_tests;

#[cfg(test)]
#[path = "../../tests/unit/settings_menu_mode_editor_explicit_draft.rs"]
mod explicit_draft_tests;
