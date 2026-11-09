//! Cached menu text shared by the native renderer and host rendering tests.
//! Builds only on content changes; lightweight contains option rows only.
use kirby_assist_core::{
    fighter_profiles::{spacing, FighterProfile},
    settings_menu::{MenuRow, MenuSelection},
};
use std::ffi::CString;
mod text_cache {
    //! Explicit owned C-string copies for cached UI. Never reuse a Vec<String>
    //! allocation as Vec<CString> or transfer the source strings' byte allocations.
    use std::{borrow::Cow, ffi::CString};

    pub(super) fn copy_text(text: &str) -> Result<Option<CString>, &'static str> {
        if text.as_bytes().contains(&0) {
            return Ok(None);
        }
        let size = text.len().checked_add(1).ok_or("Menu text is too large")?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(size)
            .map_err(|_| "Not enough memory for menu text")?;
        bytes.extend_from_slice(text.as_bytes());
        bytes.push(0);
        // Safe constructor, including empty strings: every retained entry owns at
        // least the NUL byte. Exact capacity avoids a shrink/reallocation step.
        CString::from_vec_with_nul(bytes)
            .map(Some)
            .map_err(|_| "Invalid menu text")
    }
    pub(super) fn copy_lines(
        lines: &[String],
        show_hints: bool,
        strip_hints: fn(String) -> Option<String>,
    ) -> Result<Vec<CString>, &'static str> {
        let mut cached = Vec::new();
        cached
            .try_reserve_exact(lines.len())
            .map_err(|_| "Not enough memory for menu rows")?;
        for source in lines {
            let text = if show_hints {
                Cow::Borrowed(source.as_str())
            } else {
                let Some(filtered) = strip_hints(source.clone()) else {
                    continue;
                };
                Cow::Owned(filtered)
            };
            if let Some(line) = copy_text(&text)? {
                cached.push(line);
            }
        }
        Ok(cached)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        fn strip(s: String) -> Option<String> {
            (!s.starts_with("Y:")).then_some(s)
        }
        #[test]
        fn copies_empty_unicode_error_and_filtered_rows_without_aliasing_sources() {
            let lines = vec![
                "".into(),
                "Save failed: SD unavailable (514)".into(),
                "Saving…".into(),
                "Y: Save".into(),
                "invalid\0text".into(),
            ];
            let result = copy_lines(&lines, false, strip).unwrap();
            assert_eq!(result.len(), 3);
            assert_eq!(result[0].as_bytes_with_nul(), b"\0");
            assert_eq!(result[1].to_str().unwrap(), lines[1]);
            assert_eq!(result[2].to_str().unwrap(), "Saving…");
            for (copy, source) in result.iter().zip(&lines) {
                assert_ne!(copy.as_ptr().cast::<u8>(), source.as_ptr());
            }
            drop(lines);
            assert_eq!(
                result[1].to_str().unwrap(),
                "Save failed: SD unavailable (514)"
            );
        }
        #[test]
        fn repeated_save_status_cache_replacement_does_not_reuse_source_storage() {
            let mut view = Some(copy_lines(&["Before save".into()], true, strip).unwrap());
            for i in 0..2000 {
                let source = vec![
                    format!("Field {i}"),
                    String::new(),
                    "Saved to sequences.txt (backup created)".into(),
                    "Save failed: temporary file exists".into(),
                ];
                let next = copy_lines(&source, true, strip).unwrap();
                let retired = view.replace(next);
                drop(source);
                drop(retired);
                for line in view.as_ref().unwrap() {
                    assert_eq!(line.as_bytes_with_nul().last(), Some(&0));
                }
            }
        }
    }
}

pub(super) struct Snapshot {
    pub(super) revision: u64,
    pub(super) generation: u64,
    pub(super) content_revision: u64,
    pub(super) recipe_revision: u64,
    pub(super) auto_mash: bool,
    pub(super) quick: bool,
    pub(super) lightweight: bool,
    pub(super) buttons: bool,
    pub(super) modes: bool,
    pub(super) fighter: i32,
    pub(super) row: MenuRow,
    pub(super) values: Vec<SavedChoice>,
    pub(super) navigation: Option<CString>,
    pub(super) header: Vec<CString>,
    pub(super) body: Vec<CString>,
    pub(super) footer: Vec<CString>,
    pub(super) focus: Option<usize>,
    pub(super) choice: usize,
    pub(super) mode: u8,
}
pub(super) struct SavedChoice {
    pub(super) row: MenuRow,
    pub(super) enabled: Option<bool>,
    // Complete single-column lines are cached on edits, never formatted in draw.
    pub(super) line: CString,
}
impl Snapshot {
    pub(super) fn reusable(
        &self,
        profile: &FighterProfile,
        selection: &MenuSelection,
        auto_mash: bool,
        generation: u64,
    ) -> bool {
        self.lightweight
            && selection.lightweight
            && self.generation == generation
            && self.fighter == profile.fighter_kind
            && self.content_revision == selection.content_revision
            && self.recipe_revision == selection.recipe_revision
            && self.auto_mash == auto_mash
            && self.buttons == selection.button_menu
            && self.modes == selection.modes_menu
            && self.quick == (selection.quick_menu && !selection.tuning_page)
            && self.values.iter().any(|v| v.row == selection.row)
    }
    pub(super) fn refresh_cursor(
        &mut self,
        profile: &FighterProfile,
        selection: &MenuSelection,
        auto_mash: bool,
        generation: u64,
    ) -> bool {
        if !self.reusable(profile, selection, auto_mash, generation) {
            return false;
        }
        self.row = selection.row;
        self.revision = selection.revision;
        true
    }
    pub(super) fn matches_request(
        &self,
        lightweight: bool,
        fighter: i32,
        revision: u64,
        generation: u64,
    ) -> bool {
        self.matches_layout(lightweight)
            && self.fighter == fighter
            && self.revision == revision
            && self.generation == generation
    }
    pub(super) fn matches_layout(&self, lightweight: bool) -> bool {
        self.lightweight == lightweight
    }
}

fn toggle_value(enabled: bool) -> (String, usize, usize) {
    (
        (if enabled { "ON" } else { "OFF" }).to_owned(),
        usize::from(enabled),
        2,
    )
}

#[cfg(test)]
mod row_tests {
    use super::*;

    #[test]
    fn toggle_rows_keep_matching_labels_and_positions() {
        assert_eq!(toggle_value(false), ("OFF".to_owned(), 0, 2));
        assert_eq!(toggle_value(true), ("ON".to_owned(), 1, 2));
    }
}

fn row_value(
    profile: &FighterProfile,
    selection: &MenuSelection,
    row: MenuRow,
    auto_mash: bool,
) -> Option<(String, usize, usize)> {
    Some(if MenuSelection::page_link(row) {
        ("Open >".to_owned(), 0, 0)
    } else if let Some(slot) = row.slot() {
        let raw_index = selection.choices[slot as usize];
        let (index, count) = selection.choice_position(&profile.menu_options, slot);
        let name = selection
            .choice(&profile.menu_options, slot, raw_index)
            .map_or_else(
                || "Unavailable".into(),
                |v| {
                    format!(
                        "{}{}",
                        spacing::menu_name(v.name),
                        if selection.recipe_modified(slot) {
                            " *"
                        } else {
                            ""
                        }
                    )
                },
            );
        let name = if slot.is_button() && selection.rule_button_mask & slot.button() != 0 {
            format!("Toggle rules / saved: {name}")
        } else {
            name
        };
        (name, index, count)
    } else if row == MenuRow::IdleCrouch
        && selection.y_fighter_kind == kirby_assist_core::fighter_kinds::KIRBY
    {
        toggle_value(selection.effective_mode(
            kirby_assist_core::button_bindings::Mode::IdleCrouch,
            selection.idle_crouch,
        ))
    } else if row == MenuRow::TiltTurn {
        toggle_value(selection.effective_mode(
            kirby_assist_core::button_bindings::Mode::TiltTurn,
            selection.tilt_turn,
        ))
    } else if row == MenuRow::AutoReflect {
        toggle_value(selection.auto_reflect)
    } else if row == MenuRow::OutOfShield {
        let enabled = selection.effective_mode(
            kirby_assist_core::button_bindings::Mode::OutOfShield,
            selection.out_of_shield,
        );
        toggle_value(enabled)
    } else if matches!(
        row,
        MenuRow::SpotPunish | MenuRow::GrabDodge | MenuRow::ShieldPressure
    ) {
        let enabled = if row == MenuRow::ShieldPressure {
            selection.shield_pressure
        } else if row == MenuRow::SpotPunish {
            selection.effective_mode(
                kirby_assist_core::button_bindings::Mode::SpotPunish,
                selection.spot_punish,
            )
        } else {
            selection.grab_dodge
        };
        toggle_value(enabled)
    } else if row == MenuRow::TestLatency {
        (
            if selection.test_latency == 0 {
                "Unset (label only)".into()
            } else {
                format!("External +{}f (label only)", selection.test_latency - 1)
            },
            selection.test_latency as usize,
            32,
        )
    } else if row == MenuRow::Performance {
        toggle_value(selection.performance)
    } else if row == MenuRow::AutoTech {
        toggle_value(selection.effective_mode(
            kirby_assist_core::button_bindings::Mode::AutoTech,
            selection.auto_tech,
        ))
    } else if row == MenuRow::AutoLedge {
        toggle_value(selection.ledge_mode())
    } else if row == MenuRow::RollRules {
        toggle_value(selection.roll_rules)
    } else if row == MenuRow::AutoMash {
        toggle_value(auto_mash)
    } else {
        return None;
    })
}

pub(super) fn build(
    profile: &FighterProfile,
    selection: &MenuSelection,
    auto_mash: bool,
    generation: u64,
) -> Result<Snapshot, &'static str> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(
            selection
                .display_rows()
                .iter()
                .filter(|row| selection.row_visible(**row))
                .count(),
        )
        .map_err(|_| "Not enough memory for menu rows")?;
    for &row in selection.display_rows() {
        if selection.assist_rule_page() || !selection.row_visible(row) {
            continue;
        }
        let Some((name, index, count)) = row_value(profile, selection, row, auto_mash) else {
            continue;
        };
        let position = if index < count { index + 1 } else { 0 };
        let cursor = if !selection.lightweight
            && (selection.quick_menu || selection.modes_menu)
            && row == selection.row
        {
            "> "
        } else {
            ""
        };
        let position_label = if selection.modes_menu
            || count == 0
            || selection.lightweight && row.slot().is_some_and(|s| !s.is_button())
        {
            String::new()
        } else {
            format!("  ({position}/{count})")
        };
        let line =
            text_cache::copy_text(&format!("{cursor}{}: {name}{position_label}", row.label()))?
                .ok_or("Invalid menu row text")?;
        let enabled = match name.as_str() {
            "ON" => Some(true),
            "OFF" => Some(false),
            _ => None,
        };
        values.push(SavedChoice { row, line, enabled });
    }
    let row_number = values
        .iter()
        .position(|v| v.row == selection.row)
        .map_or(0, |i| i + 1);
    let navigation = if selection.lightweight {
        None
    } else {
        let navigation_text = if selection.assist_rule_page() {
            format!("{} / ASSIST RULES", profile.name)
        } else if selection.tuning_page {
            format!("{} / FIGHTER SETTINGS", profile.name)
        } else if selection.quick_menu {
            format!(
                "{}   /   {}",
                profile.name,
                if selection.button_menu {
                    "BUTTON CONFIGS"
                } else {
                    "LEDGE TRAPS"
                }
            )
        } else if selection.modes_menu {
            format!("{}   /   MODES", profile.name)
        } else if selection.managing() {
            format!("{}   /   MANAGE LIST", profile.name)
        } else if selection.recipe_editing {
            format!("{}   /   EDIT RECIPE", profile.name)
        } else {
            format!(
                "{}   /   {}   /   {row_number}/{}",
                profile.name,
                if selection.button_menu {
                    "BUTTON CONFIGS"
                } else {
                    "LEDGE TRAPS"
                },
                values.len()
            )
        };
        Some(text_cache::copy_text(&navigation_text)?.ok_or("Invalid menu heading")?)
    };
    let panel = if selection.lightweight {
        kirby_assist_core::settings_menu::RecipePanel::default()
    } else if selection.tuning_page {
        selection.recipe_panel(&profile.menu_options)
    } else if selection.quick_menu {
        kirby_assist_core::settings_menu::RecipePanel {
            header: {
                let mut lines=Vec::new();
                if selection.preview_slot.is_some() {lines.push("TEST RUNNING".into());}
                if let Some(status)=&selection.save_status {lines.push(status.clone());}
                lines
            },
            footer:vec![
                "Up/Down: row (hold to scroll) | Left/Right: change | R3: open/edit | L3: test | Minus: close".into()
            ],
            ..Default::default()
        }
    } else if selection.modes_menu {
        kirby_assist_core::settings_menu::RecipePanel::default()
    } else {
        selection.recipe_panel(&profile.menu_options)
    };
    // Lightweight has no heading, editor body or controls footer at all.
    // Training keeps every original hint and status without text filtering.
    let header = text_cache::copy_lines(&panel.header, true, Some)?;
    let body = text_cache::copy_lines(&panel.body, true, Some)?;
    let footer = text_cache::copy_lines(&panel.footer, true, Some)?;
    Ok(Snapshot {
        revision: selection.revision,
        generation,
        content_revision: selection.content_revision,
        recipe_revision: selection.recipe_revision,
        auto_mash,
        quick: selection.quick_menu && !selection.tuning_page,
        lightweight: selection.lightweight,
        buttons: selection.button_menu,
        modes: selection.modes_menu,
        fighter: profile.fighter_kind,
        row: selection.row,
        values,
        navigation,
        header,
        body,
        footer,
        focus: panel.focus,
        choice: selection
            .row
            .slot()
            .map_or(0, |s| selection.choices[s as usize]),
        mode: if selection.managing() {
            2
        } else if selection.recipe_editing {
            1
        } else {
            0
        },
    })
}
