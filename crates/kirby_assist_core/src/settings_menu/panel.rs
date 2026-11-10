//! Cached presentation text for the selected row; no execution state or native UI dependencies.
use super::*;
use crate::fighter_tuning::{self, Tuning};

impl MenuSelection {
    /// Technical: Returns the `recipe_panel` result.
    ///
    /// Plain English: Gets the recipe panel result.
    pub fn recipe_panel(&self, presets: &MenuPresets) -> RecipePanel {
        if self.mode_rule_list && !self.managing() {
            return self.mode_rules_panel(presets);
        }
        if self.tuning_page {
            return self.tuning_panel();
        }
        let Some(slot) = self.row.slot() else {
            return RecipePanel::default();
        };
        let index = self.choices[slot as usize];
        if let Some(catalog) = self.catalog.as_ref().filter(|c| c.active()) {
            return self.catalog_panel(catalog);
        }
        let builtin = self
            .source(presets, slot, index)
            .and_then(|source| source.builtin());
        if let Some(builtin) = builtin {
            return RecipePanel {
                header: vec![builtin.label().into()],
                body: vec![
                    "Uses the existing native helper; no input recipe runs.".into(),
                    "Select this built-in on another button to move its activation.".into(),
                ],
                footer: vec!["Y: save as startup default | R3: back | B: manage".into()],
                focus: None,
            };
        }
        self.recipe_editor_panel(presets, slot, index)
    }
    /// Technical: Returns the `tuning_panel` result.
    ///
    /// Plain English: Gets the tuning panel result.
    fn tuning_panel(&self) -> RecipePanel {
        let row = self.tuning_visible_index(self.tuning_row);
        RecipePanel {
            header: vec![
                "FIGHTER SETTINGS / Training only".into(),
                fighter_tuning::HINTS[row].into(),
            ],
            body: (0..fighter_tuning::COUNT)
                .filter(|i| self.tuning_visible(*i))
                .map(|i| self.tuning_line(i, i == row))
                .collect(),
            footer: [
                Some("Up/Down: select | Left/Right: adjust".into()),
                Some("X: reset value | Y alone: save | R3: back | Minus: close".into()),
                self.save_status.clone(),
            ]
            .into_iter()
            .flatten()
            .collect(),
            focus: Some((0..row).filter(|i| self.tuning_visible(*i)).count()),
        }
    }
    /// One fighter-setting row: value plus where that value comes from.
    /// Plain English: One fighter-setting row: value plus where that value comes from.
    fn tuning_line(&self, i: usize, selected: bool) -> String {
        let custom = self.tuning.0[i];
        let value = custom.unwrap_or(self.tuning_values[i]);
        let shown = if Tuning::is_bool(i) {
            if value == 1.0 { "ON".into() } else { "OFF".into() }
        } else if i == 0 || i == 7 {
            format!("{:.0}%", value * 100.0)
        } else if Tuning::frames(i) {
            format!("{value:.1} frames")
        } else {
            format!("{value:.1} units")
        };
        let origin = if custom.is_some() {
            " [custom]"
        } else if i == 1 {
            " [fighter reference]"
        } else if i < 14 {
            " [default]"
        } else if Tuning::frames(i) {
            " [policy default]"
        } else {
            " [estimate]"
        };
        format!(
            "{} {}: {shown}{origin}",
            if selected { ">" } else { " " },
            fighter_tuning::LABELS[i]
        )
    }
    /// Technical: Returns the `catalog_panel` result.
    ///
    /// Plain English: Gets the catalog panel result.
    fn catalog_panel(&self, catalog: &catalog::Catalog) -> RecipePanel {
        let mut header = catalog.lines();
        let body = if header.is_empty() {
            vec![]
        } else {
            header.split_off(1)
        };
        let mut panel = RecipePanel {
            focus: body.iter().position(|l| l.starts_with('>')),
            header,
            body,
            footer: vec![],
        };
        if matches!(&catalog.mode, catalog::Mode::Actions(_)) {
            panel
                .footer
                .push("Up/Down: select | A: choose | B: back".into());
        }
        if let Some(status) = &self.save_status {
            panel.footer.push(status.clone());
        }
        panel
    }
    /// The recipe editor (or its read-only preview) with the page hints around it.
    /// Plain English: The recipe editor (or its read-only preview) with the page hints around it.
    fn recipe_editor_panel(
        &self,
        presets: &MenuPresets,
        slot: PresetSlot,
        index: usize,
    ) -> RecipePanel {
        let key = (slot as usize, index);
        let mut panel = if let Some(editor) = self.editor.as_ref().filter(|e| e.has(key)) {
            editor.panel(key, self.recipe_editing)
        } else {
            let sequence = self
                .source(presets, slot, index)
                .map(|s| s.sequence())
                .unwrap_or_else(editor::RecipeEditor::custom);
            editor::RecipeEditor::preview_panel(sequence)
        };
        if !self.editor.as_ref().is_some_and(|e| e.command_active()) {
            panel.footer.push(if self.recipe_editing {
                "B: Back | A / ZL+ZR: Step tools and recipe options".into()
            } else {
                "B: Manage list (add / rename / duplicate / delete)".into()
            });
            if self.recipe_editing {
                panel
                    .footer
                    .push("Up/Down: field | Left/Right: adjust | Y: Save".into());
            }
        }
        if let Some((row, _)) = self.mode_editor_origin.as_ref() {
            panel
                .header
                .insert(0, format!("MODE RESPONSE: {}", row.label()));
            panel
                .footer
                .push("R+Left/Right: Other rules | R3/L3: Back to rule list".into());
        }
        if let Some(status) = &self.save_status {
            panel.footer.push(status.clone());
        }
        if self.preview_slot.is_some() {
            panel
                .header
                .insert(0, "TEST RUNNING | X or a menu change cancels".into());
        }
        if self.recipe_editing && !self.lightweight {
            if let Some(editor) = self.editor.as_ref().filter(|e| e.reference_visible()) {
                let hint_slot = if self.mode_editor_origin.is_some() {
                    PresetSlot::X
                } else {
                    slot
                };
                panel.header.extend(reference::hints(
                    self.y_fighter_kind,
                    hint_slot,
                    editor.reference_label(key),
                ));
            }
        }
        panel
    }
}
