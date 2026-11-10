//! Cached menu text shared by the native renderer and host rendering tests.
//! Technical: Caches formatted menu text and exposes shared rendering snapshots.
//! Plain English: This module prepares menu labels so the renderer can display them.
//! Builds only on content changes; lightweight contains option rows only.
use kirby_assist_core::{
    fighter_profiles::{presets::PresetSlot, spacing, FighterProfile},
    settings_menu::{MenuRow, MenuSelection, RecipePanel},
};
use std::ffi::CString;
mod text_cache {
    //! Explicit owned C-string copies for cached UI. Never reuse a Vec<String>
    //! allocation as Vec<CString> or transfer the source strings' byte allocations.
    //! Technical: Copies display strings into separately owned, valid native C strings.
    //! Plain English: Makes safe text copies for the menu renderer.
    use std::ffi::CString;

    /// Copies valid UTF-8 bytes into an independently owned NUL-terminated CString.
    ///
    ///  Makes a safe text copy for native menu display.
    /// ### Arguments
    /// - `text` (`&str`): Represents the text being parsed or displayed; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Ok(value)` on success or `Err(error)` if the operation fails.
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
    /// Reserves and copies each display line into separately owned C strings.
    ///
    ///  Prepares a list of menu lines that can safely outlive their source.
    /// ### Arguments
    /// - `lines` (`&[String]`): Represents the lines; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Ok(value)` on success or `Err(error)` if the operation fails.
    pub(super) fn copy_lines(lines: &[String]) -> Result<Vec<CString>, &'static str> {
        let mut cached = Vec::new();
        cached
            .try_reserve_exact(lines.len())
            .map_err(|_| "Not enough memory for menu rows")?;
        for source in lines {
            if let Some(line) = copy_text(source)? {
                cached.push(line);
            }
        }
        Ok(cached)
    }

    #[cfg(test)]
    /// Ownership and text-validity regressions for native display copies.
    ///
    ///  Tests that cached menu text is copied safely.
    mod tests {
        use super::*;
        #[test]
        /// Checks empty, Unicode, error, and embedded-NUL line handling and ownership.
        ///
        ///  Verifies the display copies valid text and skips invalid text.
        fn copies_empty_unicode_and_error_rows_without_aliasing_sources() {
            let lines = vec![
                "".into(),
                "Save failed: SD unavailable (514)".into(),
                "Saving…".into(),
                "invalid\0text".into(),
            ];
            let result = copy_lines(&lines).unwrap();
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
        /// Repeatedly replaces cached save-status lines without source-buffer reuse.
        ///
        ///  Checks repeated saves do not leave the menu using temporary text.
        fn repeated_save_status_cache_replacement_does_not_reuse_source_storage() {
            let mut view = Some(copy_lines(&["Before save".into()]).unwrap());
            for i in 0..2000 {
                let source = vec![
                    format!("Field {i}"),
                    String::new(),
                    "Saved to sequences.txt (backup created)".into(),
                    "Save failed: temporary file exists".into(),
                ];
                let next = copy_lines(&source).unwrap();
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

/// Fully owned, revision-tagged presentation data consumed by the draw callback.
///
///  A ready-to-display copy of the menu for one selection revision.
pub(super) struct Snapshot {
    /// General menu revision represented by this presentation.
    ///
    ///  Which menu update this snapshot displays.
    pub(super) revision: u64,
    /// Lifecycle generation used to reject snapshots from prior sessions.
    ///
    ///  Which menu session this snapshot belongs to.
    pub(super) generation: u64,
    /// Content revision used to identify changes to visible display strings.
    ///
    ///  Which version of the menu text this snapshot contains.
    pub(super) content_revision: u64,
    /// Recipe revision represented by this presentation.
    ///
    ///  Which recipe version the menu shows.
    pub(super) recipe_revision: u64,
    /// Effective auto-mash value represented by the rows.
    ///
    ///  Whether the displayed choices include auto-mash being on.
    pub(super) auto_mash: bool,
    /// Whether this snapshot represents the quick-menu layout.
    ///
    ///  Whether the quick menu is displayed.
    pub(super) quick: bool,
    /// Whether this snapshot omits editor-heavy presentation content.
    ///
    ///  Whether this is the compact menu.
    pub(super) lightweight: bool,
    /// Whether button-choice rows are part of the layout.
    ///
    ///  Whether button choices are shown.
    pub(super) buttons: bool,
    /// Whether mode-choice rows are part of the layout.
    ///
    ///  Whether mode choices are shown.
    pub(super) modes: bool,
    /// Native fighter kind used to build the rows.
    ///
    ///  Which fighter this display belongs to.
    pub(super) fighter: i32,
    /// Currently selected navigation row.
    ///
    ///  Which menu row is highlighted.
    pub(super) row: MenuRow,
    /// Cached selectable row labels and values.
    ///
    ///  The choices shown in the menu.
    pub(super) values: Vec<SavedChoice>,
    /// Optional cached navigation heading.
    ///
    ///  The menu title shown above the choices.
    pub(super) navigation: Option<CString>,
    /// Cached panel header lines.
    ///
    ///  Status lines shown near the top of the menu.
    pub(super) header: Vec<CString>,
    /// Cached editor or recipe body lines.
    ///
    ///  The main details shown in the menu.
    pub(super) body: Vec<CString>,
    /// Cached panel footer lines.
    ///
    ///  Hints shown at the bottom of the menu.
    pub(super) footer: Vec<CString>,
    /// Optional focus index for editor controls.
    ///
    ///  Which editor field should receive focus.
    pub(super) focus: Option<usize>,
    /// Selected option index in the current preset slot.
    ///
    ///  Which option is selected in the current slot.
    pub(super) choice: usize,
    /// Editor state discriminant used by the renderer.
    ///
    ///  Which recipe editing state the menu is in.
    pub(super) mode: u8,
}
/// Cached row identity, enabled-state hint, and owned display text.
///
///  One prepared menu choice line and its status.
pub(super) struct SavedChoice {
    /// Core menu row represented by this cached label.
    ///
    ///  Which menu choice this line belongs to.
    pub(super) row: MenuRow,
    /// Toggle value when the row is a simple ON/OFF control.
    ///
    ///  Whether a toggle row is currently on or off.
    pub(super) enabled: Option<bool>,
    // Complete single-column lines are cached on edits, never formatted in draw.
    /// Owned NUL-terminated full row text for native rendering.
    ///
    ///  The ready-made text shown for this choice.
    pub(super) line: CString,
}
impl Snapshot {
    /// Tests whether content/layout inputs are reusable apart from cursor and revision.
    ///
    ///  Checks if the existing display still matches the menu's choices.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `profile` (`&FighterProfile`): Represents the profile; the function uses it as input to the operation described above.
    /// - `selection` (`&MenuSelection`): Represents the selection; the function uses it as input to the operation described above.
    /// - `auto_mash` (`bool`): Represents the auto mash; the function uses it as input to the operation described above.
    /// - `generation` (`u64`): Represents the generation; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
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
    /// Updates only cursor/revision fields when the cached content remains reusable.
    ///
    ///  Moves the highlight without rebuilding the menu text.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `profile` (`&FighterProfile`): Represents the profile; the function uses it as input to the operation described above.
    /// - `selection` (`&MenuSelection`): Represents the selection; the function uses it as input to the operation described above.
    /// - `auto_mash` (`bool`): Represents the auto mash; the function uses it as input to the operation described above.
    /// - `generation` (`u64`): Represents the generation; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
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
    /// Checks layout, fighter, revision, and generation against a publication request.
    ///
    ///  Confirms this snapshot is the one the menu currently wants.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `lightweight` (`bool`): Represents the lightweight; the function uses it as input to the operation described above.
    /// - `fighter` (`i32`): Represents the fighter being checked or configured; the function uses it as input to the operation described above.
    /// - `revision` (`u64`): Represents the revision; the function uses it as input to the operation described above.
    /// - `generation` (`u64`): Represents the generation; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
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
    /// Compares only the requested lightweight/full layout flag.
    ///
    ///  Checks whether the menu is compact or full-sized as requested.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `lightweight` (`bool`): Represents the lightweight; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub(super) fn matches_layout(&self, lightweight: bool) -> bool {
        self.lightweight == lightweight
    }
}

/// Converts a boolean toggle to its label, selected index, and option count.
///
///  Prepares the ON/OFF text and selection position.
/// ### Arguments
/// - `enabled` (`bool`): Represents whether the feature or option should be enabled; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `(String, usize, usize)` value.
fn toggle_value(enabled: bool) -> (String, usize, usize) {
    (
        (if enabled { "ON" } else { "OFF" }).to_owned(),
        usize::from(enabled),
        2,
    )
}

#[cfg(test)]
/// Regressions for cached menu row labels and choice positions.
///
///  Tests that menu row text and positions are consistent.
mod row_tests {
    use super::*;

    #[test]
    /// Ensures toggle labels correspond to their selected indices.
    ///
    ///  Checks ON and OFF rows point at the right option.
    fn toggle_rows_keep_matching_labels_and_positions() {
        assert_eq!(toggle_value(false), ("OFF".to_owned(), 0, 2));
        assert_eq!(toggle_value(true), ("ON".to_owned(), 1, 2));
    }
}

/// Formats the selected preset display name and its one-based position.
///
///  Gets the name and position shown for a recipe slot.
/// ### Arguments
/// - `profile` (`&FighterProfile`): Represents the profile; the function uses it as input to the operation described above.
/// - `selection` (`&MenuSelection`): Represents the selection; the function uses it as input to the operation described above.
/// - `slot` (`PresetSlot`): Represents the configuration or storage slot being used; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `(String, usize, usize)` value.
fn slot_value(
    profile: &FighterProfile,
    selection: &MenuSelection,
    slot: PresetSlot,
) -> (String, usize, usize) {
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
}

/// Resolves one menu row into display text and choice position metadata.
///
///  Prepares the text and selection for a single menu row.
/// ### Arguments
/// - `profile` (`&FighterProfile`): Represents the profile; the function uses it as input to the operation described above.
/// - `selection` (`&MenuSelection`): Represents the selection; the function uses it as input to the operation described above.
/// - `row` (`MenuRow`): Represents the row; the function uses it as input to the operation described above.
/// - `auto_mash` (`bool`): Represents the auto mash; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Some(value)` when a value is available; otherwise, returns `None`.
fn row_value(
    profile: &FighterProfile,
    selection: &MenuSelection,
    row: MenuRow,
    auto_mash: bool,
) -> Option<(String, usize, usize)> {
    use kirby_assist_core::{button_bindings::Mode, fighter_kinds::KIRBY};
    if MenuSelection::page_link(row) {
        return Some(("Open >".to_owned(), 0, 0));
    }
    if let Some(slot) = row.slot() {
        return Some(slot_value(profile, selection, slot));
    }
    let toggle = match row {
        MenuRow::IdleCrouch if selection.y_fighter_kind == KIRBY => {
            selection.effective_mode(Mode::IdleCrouch, selection.idle_crouch)
        }
        MenuRow::TiltTurn => selection.effective_mode(Mode::TiltTurn, selection.tilt_turn),
        MenuRow::AutoReflect => selection.auto_reflect,
        MenuRow::OutOfShield => {
            selection.effective_mode(Mode::OutOfShield, selection.out_of_shield)
        }
        MenuRow::ShieldPressure => selection.shield_pressure,
        MenuRow::SpotPunish => selection.effective_mode(Mode::SpotPunish, selection.spot_punish),
        MenuRow::GrabDodge => selection.grab_dodge,
        MenuRow::Performance => selection.performance,
        MenuRow::AutoTech => selection.effective_mode(Mode::AutoTech, selection.auto_tech),
        MenuRow::AutoLedge => selection.ledge_mode(),
        MenuRow::RollRules => selection.roll_rules,
        MenuRow::AutoMash => auto_mash,
        MenuRow::TestLatency => {
            let label = match selection.test_latency {
                0 => "Unset (label only)".into(),
                n => format!("External +{}f (label only)", n - 1),
            };
            return Some((label, selection.test_latency as usize, 32));
        }
        _ => return None,
    };
    Some(toggle_value(toggle))
}

/// Rows shown in the single-column list, formatted once per rebuild.
/// Builds owned text records for visible rows, reserving their full capacity first.
///
///  Prepares all visible menu choices for display.
/// ### Arguments
/// - `profile` (`&FighterProfile`): Represents the profile; the function uses it as input to the operation described above.
/// - `selection` (`&MenuSelection`): Represents the selection; the function uses it as input to the operation described above.
/// - `auto_mash` (`bool`): Represents the auto mash; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Ok(value)` on success or `Err(error)` if the operation fails.
fn build_values(
    profile: &FighterProfile,
    selection: &MenuSelection,
    auto_mash: bool,
) -> Result<Vec<SavedChoice>, &'static str> {
    let rows: &[MenuRow] = if selection.assist_rule_page() {
        &[]
    } else {
        selection.display_rows()
    };
    let mut values = Vec::new();
    values
        .try_reserve_exact(rows.iter().filter(|r| selection.row_visible(**r)).count())
        .map_err(|_| "Not enough memory for menu rows")?;
    for &row in rows {
        if !selection.row_visible(row) {
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
    Ok(values)
}

/// Formats the heading and navigation position for the active page.
///
///  Creates the title showing the current fighter and menu page.
/// ### Arguments
/// - `profile` (`&FighterProfile`): Represents the profile; the function uses it as input to the operation described above.
/// - `selection` (`&MenuSelection`): Represents the selection; the function uses it as input to the operation described above.
/// - `values` (`&[SavedChoice]`): Represents the values; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting text.
fn navigation_text(
    profile: &FighterProfile,
    selection: &MenuSelection,
    values: &[SavedChoice],
) -> String {
    let list = if selection.button_menu {
        "BUTTON CONFIGS"
    } else {
        "LEDGE TRAPS"
    };
    let name = profile.name;
    if selection.assist_rule_page() {
        format!("{name} / ASSIST RULES")
    } else if selection.tuning_page {
        format!("{name} / FIGHTER SETTINGS")
    } else if selection.quick_menu {
        format!("{name}   /   {list}")
    } else if selection.modes_menu {
        format!("{name}   /   MODES")
    } else if selection.managing() {
        format!("{name}   /   MANAGE LIST")
    } else if selection.recipe_editing {
        format!("{name}   /   EDIT RECIPE")
    } else {
        let row_number = values
            .iter()
            .position(|v| v.row == selection.row)
            .map_or(0, |i| i + 1);
        format!("{name}   /   {list}   /   {row_number}/{}", values.len())
    }
}

/// Selects or constructs the panel content appropriate to the active editor page.
///
///  Chooses which status and help text belongs on this menu page.
/// ### Arguments
/// - `profile` (`&FighterProfile`): Represents the profile; the function uses it as input to the operation described above.
/// - `selection` (`&MenuSelection`): Represents the selection; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `RecipePanel` value.
fn recipe_panel(profile: &FighterProfile, selection: &MenuSelection) -> RecipePanel {
    if selection.lightweight {
        RecipePanel::default()
    } else if selection.tuning_page {
        selection.recipe_panel(&profile.menu_options)
    } else if selection.quick_menu {
        let mut header = Vec::new();
        if selection.preview_slot.is_some() {
            header.push("TEST RUNNING".into());
        }
        header.extend(selection.save_status.clone());
        RecipePanel {
            header,
            footer: vec![
                "Up/Down: row (hold to scroll) | Left/Right: change | R3: open/edit | L3: test | Minus: close".into()
            ],
            ..Default::default()
        }
    } else if selection.modes_menu {
        RecipePanel::default()
    } else {
        selection.recipe_panel(&profile.menu_options)
    }
}

/// Builds a revision-tagged snapshot, owning all text required by rendering.
///
///  Prepares a complete menu display from the current fighter choices.
/// ### Arguments
/// - `profile` (`&FighterProfile`): Represents the profile; the function uses it as input to the operation described above.
/// - `selection` (`&MenuSelection`): Represents the selection; the function uses it as input to the operation described above.
/// - `auto_mash` (`bool`): Represents the auto mash; the function uses it as input to the operation described above.
/// - `generation` (`u64`): Represents the generation; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Ok(value)` on success or `Err(error)` if the operation fails.
pub(super) fn build(
    profile: &FighterProfile,
    selection: &MenuSelection,
    auto_mash: bool,
    generation: u64,
) -> Result<Snapshot, &'static str> {
    let values = build_values(profile, selection, auto_mash)?;
    // Lightweight has no heading, editor body or controls footer at all.
    let navigation = if selection.lightweight {
        None
    } else {
        let text = navigation_text(profile, selection, &values);
        Some(text_cache::copy_text(&text)?.ok_or("Invalid menu heading")?)
    };
    let panel = recipe_panel(profile, selection);
    // Training keeps every original hint and status without text filtering.
    let header = text_cache::copy_lines(&panel.header)?;
    let body = text_cache::copy_lines(&panel.body)?;
    let footer = text_cache::copy_lines(&panel.footer)?;
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
        } else {
            u8::from(selection.recipe_editing)
        },
    })
}
