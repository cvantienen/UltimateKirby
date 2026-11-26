//! Cached presentation text for the selected row; no execution state or native UI dependencies.
use super::*;
use crate::fighter_tuning::{self, Tuning};

impl MenuSelection {
    /// Returns the `recipe_panel` result.
    pub fn recipe_panel(&self, presets: &MenuPresets) -> RecipePanel {
        if self.mode_rule_list && !self.managing() {
            return self.mode_rules_panel(presets);
        }
        if self.tuning_page {
            return self.tuning_panel();
        }
        let Some(slot) = self.row.slot() else {
            let help = match self.row {
                MenuRow::AutoTech => "ON requests a shield input near an observed techable collision. OFF stops automatic tech requests. Missing collision data or game lockout can prevent a tech.",
                MenuRow::AutoMash => "Toggle automatic escape inputs for supported trapped states. This cannot shorten states where the game does not accept mash inputs.",
                MenuRow::IdleCrouch => "ON allows Kirby's idle crouch input when eligible. OFF leaves idle posture to you; manual movement still takes priority.",
                MenuRow::TiltTurn => "ON allows facing inputs for supported tilt attacks toward the observed target. OFF keeps your facing; unavailable observations cannot guide a turn.",
                MenuRow::SpotPunish => "ON enables configured reactions to opponent spot dodges. OFF disables them. A recipe still needs a matching observed event and valid range.",
                MenuRow::GrabDodge => "ON enables eligible responses to an observed incoming grab. OFF stops them. Detection and game action availability limit success.",
                MenuRow::AutoLedge => "ON enables selected ledge responses and spacing. OFF returns ledge control to you. Inputs do not alter ledge invulnerability or attack reach.",
                MenuRow::OutOfShield => "ON enables configured automatic grab responses. OFF stops those responses. The target must satisfy the configured observation and range checks.",
                MenuRow::ShieldPressure => "ON enables configured reactions to observed shield pressure. OFF disables them. Only matching recipes run; this does not guarantee a shield break.",
                MenuRow::RollRules => "ON enables authored automatic reactions. OFF disables them. First matching enabled rule runs; configure its event, range and inputs before use.",
                MenuRow::AutoReflect => "ON enables the supported fighter's reflect inputs for observed projectiles. OFF stops them. Missing projectile data or unavailable moves can prevent reflection.",
                MenuRow::AutoCounter => "Counter activation is assigned in Button recipes. Lucina only: hold the assigned button for detected-threat counter requests; release to stop.",
                MenuRow::AutoDi => "ON enables experimental launch-direction stick assistance. OFF keeps your launch input. This changes controller direction only and cannot guarantee survival.",
                MenuRow::AutoSdi => "ON enables experimental hitstop stick pulses. OFF keeps your input. Only eligible observed hitstop is used; this does not alter damage or knockback.",
                MenuRow::TestLatency => "Adjust the test delay label with Left/Right: increase or decrease the displayed frame count. This label does not add gameplay input latency.",
                MenuRow::Performance => "Toggle the performance display. It reports runtime work, not fighter speed; it does not improve or change gameplay timing.",
                MenuRow::FighterSettings => "Open this fighter's numeric settings. Left/Right adjusts, X resets, Y saves. Distances are world units; input timing is frames, not animation speed.",
                MenuRow::ButtonsPage => "Choose a physical button, then its normal action, built-in helper or input recipe. Custom assignments replace that button's normal action; saved bindings are preserved.",
                MenuRow::ModesPage => "Choose which assist modes are enabled, or edit their event recipes. Enabling a mode still requires its observations and safety conditions.",
                MenuRow::LedgePage => "Choose recipes for observed ledge options. Timing starts at recognition; these inputs cannot change the opponent's invulnerability or your hitboxes.",
                MenuRow::RollRead => "Legacy roll-read control is not selectable. Use Auto react event recipes instead.",
                _ => "Select a recipe to inspect its inputs, timing and conditions.",
            };
            return RecipePanel {
                header: vec![self.row.label().into(), format!("Help: {help}")],
                body: vec![help.into()],
                footer: vec!["Left/Right: adjust or toggle | R3: open | Minus: close".into()],
                focus: None,
            };
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
                    match builtin {
                        crate::button_bindings::Builtin::AutoSpacing => "Hold this button to space and request configured attacks; release to stop. Manual inputs and safety gates still apply.",
                        crate::button_bindings::Builtin::AutoSpacingToggle => "Press to enable spacing and configured attacks; press again to stop. Safety gates still apply; this does not extend attack reach.",
                        crate::button_bindings::Builtin::Counter => "Hold to request a counter against a detected threat; release to stop. Lucina only; unavailable or unobserved threats cannot be countered.",
                        crate::button_bindings::Builtin::CounterToggle => "Press to enable threat-based counter requests; press again to stop. Lucina only; no guaranteed counter against every attack.",
                        crate::button_bindings::Builtin::Normal => "Uses this button's normal game action; no standalone assist or recipe runs.",
                        _ => "Uses the existing input helper; no input recipe runs. The game still decides whether the action is available.",
                    }.into(),
                    "This assignment replaces the button's normal action. Assign it to another button to move its control.".into(),
                ],
                footer: vec!["Y: save as startup default | R3: back | B: manage".into()],
                focus: None,
            };
        }
        self.recipe_editor_panel(presets, slot, index)
    }
    /// Returns the `tuning_panel` result.
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
    fn tuning_line(&self, i: usize, selected: bool) -> String {
        let custom = self.tuning.0[i];
        let value = custom.unwrap_or(self.tuning_values[i]);
        let shown = if Tuning::is_bool(i) {
            if value == 1.0 {
                "ON".into()
            } else {
                "OFF".into()
            }
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
        } else if i < 14 || matches!(i, fighter_tuning::AUTO_DI_DEFAULT | fighter_tuning::AUTO_SDI_DEFAULT) {
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
    /// Returns the `catalog_panel` result.
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
