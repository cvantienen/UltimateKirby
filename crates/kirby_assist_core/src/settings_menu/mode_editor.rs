//! Separate Training assist rule lists; legacy saved identities remain stable.
use super::*;
use crate::reaction_rules::{Rule, SelfState, Trigger};
impl MenuSelection {
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
    pub(super) fn auto_mode_events(&self, row: MenuRow) -> Option<&'static [Trigger]> {
        if !self.simplified_reactions {
            return None;
        }
        match row {
            MenuRow::RollRules => Some(&Trigger::AUTO_EVENTS),
            MenuRow::GrabDodge => Some(&[Trigger::AutoGrabDodged]),
            MenuRow::OutOfShield => Some(&[
                Trigger::AutoGrab,
                Trigger::AutoShieldGrab,
                Trigger::AutoShieldBackup,
            ]),
            _ => None,
        }
    }
    pub fn assist_rule_page(&self) -> bool {
        self.mode_editor_origin.is_some()
    }
    pub(super) fn restore_mode_choices(&mut self) -> Option<MenuRow> {
        let (row, choices) = self.mode_editor_origin.take()?;
        self.mode_rule_list = false;
        self.mode_rule_cursor = 0;
        self.choices = choices;
        self.row = row;
        Some(row)
    }
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
    fn add_mode_rule(&mut self, presets: &MenuPresets, row: MenuRow) {
        let Some(mut trigger) = Self::mode_trigger(row) else {
            return;
        };
        if let Some(events) = self.auto_mode_events(row) {
            let entries = self.mode_entries(presets, row);
            let missing: Vec<_> = events
                .iter()
                .copied()
                .filter(|t| {
                    !entries.iter().any(|(_, slot, index)| {
                        self.recipe_snapshot(presets, *slot, *index)
                            .is_some_and(|s| s.reaction.trigger == *t)
                    })
                })
                .collect();
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
                let grab = matches!(trigger, Trigger::AutoGrab | Trigger::AutoShieldGrab);
                sequence.reaction.max = match trigger {
                    Trigger::AutoGrab => {
                        self.tuning
                            .auto_grab_ranges(self.y_fighter_kind, self.tuning_values[1])
                            .1
                    }
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
        let index = cycle.map_or(0, |up| {
            if up {
                (current + 1) % entries.len()
            } else {
                (current + entries.len() - 1) % entries.len()
            }
        });
        self.mode_rule_cursor = index;
        let (_, slot, index) = entries[index];
        self.choose_mode_rule(presets, slot, index);
    }
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
            let missing: Vec<_> = events
                .iter()
                .copied()
                .filter(|t| {
                    !entries.iter().any(|(_, s, i)| {
                        self.recipe_snapshot(presets, *s, *i)
                            .is_some_and(|s| s.reaction.trigger == *t)
                    })
                })
                .collect();
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
        panel.footer.push(if simple {"Roll/spot/air: Auto react toggle; grabs/backup: Automatic grabs; grab-dodge follow-up: Grab avoidance".into()}else{"Rules run from this assist's ON/OFF setting; separate from ledge choices".into()});
        if let Some(status) = &self.save_status {
            panel.footer.push(status.clone());
        }
        panel
    }
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
            && pressed & TWO_FRAME_TRIGGER != 0
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
                | TWO_FRAME_TRIGGER
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
        } else if pressed & (B | TWO_FRAME_TRIGGER) != 0 {
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
mod tests {
    use super::*;
    use crate::{fighter_profiles::FighterProfiles, native_input::MAPPER_R};
    fn tap(m: &mut MenuSelection, p: &MenuPresets, b: u32) {
        m.input(0, p);
        m.input(b, p);
    }
    #[test]
    fn spot_punish_opens_its_own_list_and_saved_rule_remains_separate() {
        let p = MenuPresets::default();
        let mut m = MenuSelection::new(&p);
        m.show_modes();
        m.row = MenuRow::SpotPunish;
        tap(&mut m, &p, MENU_STICK_CLICK);
        assert!(m.mode_rule_list);
        tap(&mut m, &p, MAPPER_X);
        let slot = m.row.slot().unwrap();
        let i = m.choices[slot as usize];
        assert_eq!(
            m.recipe_snapshot(&p, slot, i).unwrap().reaction.trigger,
            Trigger::SpotDodge
        );
        assert!(!m.visible_choices(&p, slot).contains(&i));
        let e = m.editor.as_mut().unwrap();
        let key = (slot as usize, i);
        e.navigate(key, true);
        assert!(e.adjust(key, true));
        tap(&mut m, &p, MAPPER_Y);
        assert!(m.save_requested);
        let c = m.choice(&p, slot, i).unwrap();
        let seq = m.recipe_snapshot(&p, slot, i).unwrap();
        let saved = crate::sequence_config::persist::update_named(
            "",
            15,
            slot,
            c.id,
            seq,
            true,
            Some("Spot hop"),
        )
        .unwrap();
        let parsed = crate::sequence_config::Catalog::parse(&saved).unwrap();
        assert_eq!(
            parsed.entries()[0].sequence.reaction.trigger,
            Trigger::SpotDodge
        );
        assert_eq!(m.resolve_reactions(&p).unwrap().len, 1);
        assert!(!m.recipe_panel(&p).header.join(" ").contains("roster avg"));
    }
    #[test]
    fn each_assist_has_its_own_budget_and_legacy_storage_does_not_select_it_for_ledge() {
        let mut source = String::new();
        for trigger in ["roll", "shield_hit", "grab_dodged"] {
            for i in 0..8 {
                source.push_str(&format!("[bowser.roll.{trigger}_{i}]\nstart_delay=0\nreaction={trigger}:grounded:0:20:18:any:1\nactions=up_b:4\n"));
            }
        }
        let catalog = crate::sequence_config::Catalog::parse(&source).unwrap();
        let mut profiles = FighterProfiles::default();
        profiles.add_sequence_choices(&catalog).unwrap();
        let p = profiles.profile(15).unwrap().menu_options;
        let mut m = MenuSelection::new(&p);
        assert_eq!(m.resolve_reactions(&p).unwrap().len, 24);
        assert!(m.settings(&p, LedgeSettings::default()).sequence_overrides[2].is_none());
        for row in [MenuRow::RollRules, MenuRow::OutOfShield, MenuRow::GrabDodge] {
            m.open_mode_list(&p, row);
            assert_eq!(m.mode_entries(&p, row).len(), 8);
            m.add_mode_rule(&p, row);
            assert!(m.save_status.as_ref().unwrap().contains("Eight rules"));
            m.restore_mode_choices();
        }
        source.push_str("[bowser.roll.too_many]\nstart_delay=0\nreaction=shield_hit:grounded:0:20:18:any:1\nactions=up_b:4\n");
        let bad = crate::sequence_config::Catalog::parse(&source).unwrap();
        assert!(bad.validate(&profiles).is_err());
    }
    #[test]
    fn rules_are_separate_sorted_and_idle_list_does_not_rebuild() {
        let source="[bowser.roll.manual]\nstart_delay=0\ndefault=true\nactions=grab:4\n[bowser.attack.oos]\nstart_delay=0\nreaction=shield_hit:grounded:0:12:18:any:1\nactions=up_b:4\n[bowser.drop.regrab]\nstart_delay=0\nreaction=ledge_regrab:grounded:0:18:30:any:1\nactions=down_smash:4\n[bowser.roll.late]\nstart_delay=0\nreaction=roll:grounded:0:20:18:any:4\nactions=grab:4\n[bowser.attack.first]\nstart_delay=0\nreaction=roll_cross:grounded:0:12:18:any:1\nactions=up_b:4\n";
        let catalog = crate::sequence_config::Catalog::parse(source).unwrap();
        let mut profiles = FighterProfiles::default();
        profiles.add_sequence_choices(&catalog).unwrap();
        let p = profiles.profile(15).unwrap().menu_options;
        let mut m = MenuSelection::new(&p);
        for slot in PresetSlot::ALL {
            for i in m.visible_choices(&p, slot) {
                assert!(!m.recipe_snapshot(&p, slot, i).unwrap().reaction.active());
            }
        }
        assert_eq!(m.resolve_reactions(&p).unwrap().len, 4);
        assert!(m.settings(&p, LedgeSettings::default()).sequence_overrides
            [PresetSlot::Attack as usize]
            .is_none());
        let original = m.choices;
        m.show_modes();
        m.row = MenuRow::RollRules;
        tap(&mut m, &p, TWO_FRAME_TRIGGER);
        let panel = m.recipe_panel(&p);
        assert!(panel.body[0].starts_with("P1"));
        assert!(panel.body[1].starts_with("P4"));
        let revision = m.revision;
        for _ in 0..60 {
            m.input(0, &p);
        }
        assert_eq!(m.revision, revision);
        tap(&mut m, &p, crate::native_input::MAPPER_A);
        assert!(m.recipe_editing);
        assert!(m
            .recipe_snapshot(&p, PresetSlot::Attack, m.choices[3])
            .unwrap()
            .reaction
            .active());
        assert!(m.settings(&p, LedgeSettings::default()).sequence_overrides[3].is_none());
        m.preview_slot = Some(PresetSlot::Attack);
        assert!(m.settings(&p, LedgeSettings::default()).sequence_overrides[3].is_some());
        m.preview_slot = None;
        assert!(m.settings(&p, LedgeSettings::default()).sequence_overrides[3].is_none());
        assert!(!m.recipe_panel(&p).header.join(" ").contains("roster avg"));
        tap(&mut m, &p, MENU_STICK_CLICK);
        assert!(m.mode_rule_list);
        tap(&mut m, &p, TWO_FRAME_TRIGGER);
        assert_eq!(m.choices, original);
        assert!(m.modes_menu);
    }
    #[test]
    fn new_oos_rule_and_duplicate_keep_their_family_and_never_enter_ledge_choices() {
        let p = MenuPresets::default();
        let mut m = MenuSelection::new(&p);
        m.show_modes();
        m.row = MenuRow::OutOfShield;
        tap(&mut m, &p, TWO_FRAME_TRIGGER);
        tap(&mut m, &p, MAPPER_X);
        let i = m.choices[PresetSlot::Attack as usize];
        assert_eq!(
            m.recipe_snapshot(&p, PresetSlot::Attack, i)
                .unwrap()
                .reaction
                .trigger,
            Trigger::ShieldHit
        );
        assert!(!m.visible_choices(&p, PresetSlot::Attack).contains(&i));
        let key = (PresetSlot::Attack as usize, i);
        let e = m.editor.as_mut().unwrap();
        e.navigate(key, true);
        assert!(e.adjust(key, true));
        assert!(m
            .recipe_snapshot(&p, PresetSlot::Attack, i)
            .unwrap()
            .actions()
            .iter()
            .any(|a| *a != crate::sequence_config::Action::Empty));
        assert!(!m
            .recipe_panel(&p)
            .footer
            .join(" ")
            .contains("R3: Back to modes"));
    }
    #[test]
    fn r3_edits_menu_responses_cycles_them_and_restores_manual_choices() {
        let text="[bowser.roll.manual]\nstart_delay=0\ndefault=true\nactions=grab:4\n[bowser.roll.first]\nstart_delay=0\nreaction=roll:grounded:0:20:18:any:1\nactions=down_smash:4\n[bowser.attack.second]\nstart_delay=0\nreaction=roll_cross:grounded:0:20:18:any:2\nactions=up_b:4\n";
        let mut profiles = FighterProfiles::default();
        profiles
            .add_sequence_choices(&crate::sequence_config::Catalog::parse(text).unwrap())
            .unwrap();
        let p = profiles.profile(15).unwrap().menu_options;
        let mut m = MenuSelection::new(&p);
        m.show();
        m.modes_menu = true;
        m.row = MenuRow::RollRules;
        let original = m.choices;
        tap(&mut m, &p, MENU_STICK_CLICK);
        assert!(m.mode_rule_list);
        tap(&mut m, &p, crate::native_input::MAPPER_A);
        assert!(m.recipe_editing);
        assert!(m.roll_rules);
        assert_eq!(m.row, MenuRow::Roll);
        tap(&mut m, &p, MAPPER_R | DPAD_RIGHT);
        assert_eq!(m.row, MenuRow::Attack);
        tap(&mut m, &p, MENU_STICK_CLICK);
        assert!(m.mode_rule_list);
        tap(&mut m, &p, TWO_FRAME_TRIGGER);
        assert!(m.modes_menu);
        assert_eq!(m.row, MenuRow::RollRules);
        assert_eq!(m.choices, original);
    }
    #[test]
    fn empty_mode_creates_tagged_draft_and_minus_preserves_manual_defaults() {
        let p = MenuPresets::default();
        let mut m = MenuSelection::new(&p);
        m.show();
        m.modes_menu = true;
        m.row = MenuRow::OutOfShield;
        let original = m.choices;
        tap(&mut m, &p, MENU_STICK_CLICK);
        assert!(m.mode_rule_list);
        tap(&mut m, &p, MAPPER_X);
        assert!(m.recipe_editing && m.out_of_shield);
        let slot = m.row.slot().unwrap();
        let c = m.choice(&p, slot, m.choices[slot as usize]).unwrap();
        assert!(c.allow_new);
        assert_eq!(c.sequence.reaction.trigger, Trigger::ShieldHit);
        tap(&mut m, &p, MAPPER_MINUS);
        assert!(m.menu_hidden);
        assert_eq!(m.choices, original);
    }
}

#[cfg(test)]
mod event_family_tests {
    use super::*;
    use crate::{
        fighter_profiles::FighterProfiles,
        sequence_config::{persist, Catalog},
    };
    #[test]
    fn vulnerable_and_any_variants_share_the_right_menu_and_save_roundtrip() {
        let source="[bowser.attack.oos]\nstart_delay=0\nreaction=shield_hit_vulnerable:grounded:0:22:12:front:1\nactions=input:r:0:0:4\n[bowser.attack.spot]\nstart_delay=0\nreaction=spot_dodge_any:grounded:0:30:12:any:2\nactions=up_b:4\n[bowser.attack.pressure]\nstart_delay=0\nreaction=shield_pressure_vulnerable:grounded:0:22:12:front:1\nactions=input:r:0:0:4\n";
        let c = Catalog::parse(source).unwrap();
        let mut profiles = FighterProfiles::default();
        profiles.add_sequence_choices(&c).unwrap();
        let p = profiles.profile(15).unwrap().menu_options;
        let m = MenuSelection::new(&p);
        for row in [
            MenuRow::OutOfShield,
            MenuRow::SpotPunish,
            MenuRow::ShieldPressure,
        ] {
            assert_eq!(m.mode_entries(&p, row).len(), 1);
        }
        assert_eq!(m.mode_entries(&p, MenuRow::RollRules).len(), 0);
        for e in c.entries() {
            let saved = persist::update(source, 15, e.slot, &e.choice, e.sequence, false).unwrap();
            let round = Catalog::parse(&saved).unwrap();
            assert_eq!(
                round
                    .entries()
                    .iter()
                    .find(|r| r.choice == e.choice)
                    .unwrap()
                    .sequence
                    .reaction,
                e.sequence.reaction
            );
        }
    }
}

#[cfg(test)]
mod simple_menu_tests {
    use super::*;
    #[test]
    fn all_auto_recipes_share_a_list_and_replace_duplicate_grab_tuning_rows() {
        let mut text = String::new();
        for t in Trigger::AUTO_EVENTS {
            text.push_str(&format!(
                "[bowser.roll.{}]\nstart_delay=0\nreaction={}:35:24\nactions={}\n",
                t.token(),
                t.token(),
                if matches!(t, Trigger::AutoGrab | Trigger::AutoShieldGrab) {
                    "input:r:0:0:4"
                } else {
                    "up_b:4"
                }
            ));
        }
        let c = crate::sequence_config::Catalog::parse(&text).unwrap();
        let mut profiles = crate::fighter_profiles::FighterProfiles::default();
        profiles.add_sequence_choices(&c).unwrap();
        let p = profiles.profile(15).unwrap().menu_options;
        let mut m = MenuSelection::new(&p);
        m.simplified_reactions = true;
        m.y_fighter_kind = 15;
        m.show_modes();
        m.input(0, &p);
        assert_eq!(m.mode_entries(&p, MenuRow::RollRules).len(), 7);
        assert_eq!(m.mode_entries(&p, MenuRow::OutOfShield).len(), 3);
        assert_eq!(m.mode_entries(&p, MenuRow::GrabDodge).len(), 1);
        assert!(!m.tuning_visible(crate::fighter_tuning::AUTO_GRAB_CLOSE));
        assert!(!m.tuning_visible(crate::fighter_tuning::AUTO_GRAB_FAR));
        let r = m.resolve_reactions(&p).unwrap();
        assert_eq!(r.len, 7);
        for (i, t) in Trigger::AUTO_EVENTS.iter().enumerate() {
            assert_eq!(r.simple_sequence(i).unwrap().reaction.trigger, *t);
        }
        for row in [MenuRow::RollRules, MenuRow::OutOfShield, MenuRow::GrabDodge] {
            m.restore_mode_choices();
            m.show_modes();
            m.row = row;
            m.input(0, &p);
            m.input(MENU_STICK_CLICK, &p);
            assert!(m.mode_rule_list && !m.tuning_page);
            let n = m.mode_entries(&p, row).len();
            m.mode_rule_cursor = n;
            m.add_mode_rule(&p, row);
            assert_eq!(
                m.mode_entries(&p, row).len(),
                n,
                "cannot add a duplicate event"
            );
        }
    }
    #[test]
    fn add_missing_auto_event_stays_inactive_until_actions_are_configured() {
        let profiles = crate::fighter_profiles::FighterProfiles::default();
        let p = profiles.profile(15).unwrap().menu_options;
        let mut m = MenuSelection::new(&p);
        m.simplified_reactions = true;
        m.y_fighter_kind = 15;
        m.open_mode_list(&p, MenuRow::RollRules);
        m.auto_add_event = 5;
        m.add_mode_rule(&p, MenuRow::RollRules);
        let r = m.resolve_reactions(&p).unwrap();
        assert!(r.simple_sequence(5).is_none());
        let slot = m.row.slot().unwrap();
        let s = m
            .recipe_snapshot(&p, slot, m.choices[slot as usize])
            .unwrap();
        assert_eq!(s.reaction.trigger, Trigger::AutoGrabDodged);
        assert_eq!(s.len, 0);
        assert!(m.recipe_editing && !m.mode_rule_list);
    }
    #[test]
    fn native_simple_menu_keeps_legacy_events_and_opens_configured_grab_lists() {
        let mut text="[bowser.roll.old]\nstart_delay=0\nreaction=roll:grounded:0:80:30:any:1\nactions=grab:4\n".to_owned();
        for event in ["auto_roll", "auto_spot_dodge", "auto_air_dodge"] {
            text.push_str(&format!(
                "[bowser.roll.{event}]\nstart_delay=0\nreaction={event}:80:30\nactions=up_b:4\n"
            ));
        }
        let catalog = crate::sequence_config::Catalog::parse(&text).unwrap();
        let mut profiles = crate::fighter_profiles::FighterProfiles::default();
        profiles.add_sequence_choices(&catalog).unwrap();
        let p = profiles.profile(15).unwrap().menu_options;
        let mut m = MenuSelection::new(&p);
        m.simplified_reactions = true;
        m.y_fighter_kind = 15;
        m.show_modes();
        m.input(0, &p);
        assert_eq!(m.resolve_reactions(&p).unwrap().len, 3);
        assert!(!m.row_visible(MenuRow::ShieldPressure));
        assert!(!m.row_visible(MenuRow::SpotPunish));
        m.row = MenuRow::RollRules;
        m.input(MENU_STICK_CLICK, &p);
        assert!(m.mode_rule_list);
        assert_eq!(m.mode_entries(&p, MenuRow::RollRules).len(), 3);
        m.restore_mode_choices();
        m.show_modes();
        m.row = MenuRow::OutOfShield;
        m.input(0, &p);
        m.input(MENU_STICK_CLICK, &p);
        assert!(!m.tuning_page);
        assert!(m.mode_rule_list);
        assert_eq!(m.mode_entries(&p, MenuRow::OutOfShield).len(), 0);
    }
}

#[cfg(test)]
mod explicit_draft_tests {
    use super::*;
    #[test]
    fn new_non_grab_events_start_blank_for_bowser_yoshi_and_lucina() {
        let profiles = crate::fighter_profiles::FighterProfiles::default();
        for kind in [
            crate::fighter_kinds::YOSHI,
            crate::fighter_kinds::BOWSER,
            crate::fighter_kinds::WARIO,
        ] {
            let p = profiles.profile(kind).unwrap().menu_options;
            for event in 0..7 {
                let mut m = MenuSelection::new(&p);
                m.simplified_reactions = true;
                m.y_fighter_kind = kind;
                m.tuning_values[1] = profiles.profile(kind).unwrap().ledge.grab.standing_range;
                m.open_mode_list(&p, MenuRow::RollRules);
                m.auto_add_event = event;
                m.add_mode_rule(&p, MenuRow::RollRules);
                let rules = m.resolve_reactions(&p).unwrap();
                let grab = matches!(event, 3 | 4);
                assert_eq!(
                    rules.simple_sequence(event).is_some(),
                    grab,
                    "kind {kind}, event {event}"
                );
                let slot = m.row.slot().unwrap();
                let draft = m
                    .recipe_snapshot(&p, slot, m.choices[slot as usize])
                    .unwrap();
                if grab {
                    assert!(matches!(
                        draft.actions(),
                        [crate::sequence_config::Action::Input {
                            buttons: crate::native_input::MAPPER_R,
                            ..
                        }]
                    ));
                } else {
                    assert_eq!(draft.len, 0);
                    assert!(m.recipe_input_error(&p).is_some());
                }
            }
        }
    }
}
