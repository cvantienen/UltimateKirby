//! Cached menu text shared by the native renderer and host rendering tests.
//! Technical: Caches formatted menu text and exposes shared rendering snapshots.
//! Plain English: This module prepares menu labels so the renderer can display them.
//! Builds only on content changes; lightweight contains option rows only.
use kirby_assist_core::{
    fighter_profiles::{presets::PresetSlot, FighterProfile},
    settings_menu::{MenuRow, MenuSelection, RecipePanel},
};
use std::{
    ffi::{CStr, CString},
    sync::atomic::{AtomicU64, Ordering},
};

/// Menu-independent, allocation-free publication of the selected L mode.
pub(super) struct BattleHud(AtomicU64);
impl BattleHud {
    pub(super) const fn new() -> Self {
        Self(AtomicU64::new(0))
    }

    pub(super) fn publish(&self, selection: &MenuSelection, player_index: u32) {
        let mode = if selection.l_spacing_selected() { 2 } else { 1 };
        self.0
            .store((u64::from(player_index) << 2) | mode, Ordering::Release);
    }

    pub(super) fn clear(&self) {
        self.0.store(0, Ordering::Release);
    }

    pub(super) fn current(&self) -> Option<(u32, &'static CStr, (f32, f32, f32))> {
        let state = self.0.load(Ordering::Acquire);
        let (label, rgb) = match state & 3 {
            1 => (c"Ledge", (0.45, 0.80, 1.0)),
            2 => (c"Spacing", (1.0, 0.73, 0.42)),
            _ => return None,
        };
        Some(((state >> 2) as u32, label, rgb))
    }
}

#[cfg_attr(test, path = "../../../tests/unit/live/settings_menu/view")]
mod text_cache {
    //! Explicit owned C-string copies for cached UI. Never reuse a Vec<String>
    //! allocation as Vec<CString> or transfer the source strings' byte allocations.
    //! Technical: Copies display strings into separately owned, valid native C strings.
    //! Plain English: Makes safe text copies for the menu renderer.
    use std::ffi::CString;

    /// Copies valid UTF-8 bytes into an independently owned NUL-terminated CString.
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
    #[path = "text_cache_tests.rs"]
    mod tests;
}

/// Fully owned, revision-tagged presentation data consumed by the draw callback.
pub(super) struct Snapshot {
    /// General menu revision represented by this presentation.
    pub(super) revision: u64,
    /// Lifecycle generation used to reject snapshots from prior sessions.
    pub(super) generation: u64,
    /// Content revision used to identify changes to visible display strings.
    pub(super) content_revision: u64,
    /// Recipe revision represented by this presentation.
    pub(super) recipe_revision: u64,
    /// Effective auto-mash value represented by the rows.
    pub(super) auto_mash: bool,
    /// Effective DI/SDI choices, including inherited startup defaults.
    pub(super) damage_sticks: [bool; 2],
    /// Whether this snapshot represents the quick-menu layout.
    pub(super) quick: bool,
    /// Whether this snapshot omits editor-heavy presentation content.
    pub(super) lightweight: bool,
    /// Whether button-choice rows are part of the layout.
    pub(super) buttons: bool,
    /// Whether mode-choice rows are part of the layout.
    pub(super) modes: bool,
    /// Whether the Training category strip has focus.
    pub(super) training_tab_focus: bool,
    /// Whether the Training categories have replaced the legacy all-rows view.
    pub(super) training_categories_active: bool,
    /// Selected Training category index.
    pub(super) training_category: u8,
    /// Whether shoulder category navigation is available on this page.
    pub(super) category_controls: bool,
    /// Native fighter kind used to build the rows.
    pub(super) fighter: i32,
    /// Currently selected navigation row.
    pub(super) row: MenuRow,
    /// Cached selectable row labels and values.
    pub(super) values: Vec<SavedChoice>,
    /// Optional cached navigation heading.
    pub(super) navigation: Option<CString>,
    /// Compact lightweight page and selected mode without control hints.
    pub(super) lightweight_label: Option<CString>,
    /// Cached category labels; drawing never allocates native strings.
    pub(super) categories: Vec<CString>,
    /// One bounded-list context label per possible cursor position.
    pub(super) row_context: Vec<CString>,
    /// Cached panel header lines.
    pub(super) header: Vec<CString>,
    /// Cached editor or recipe body lines.
    pub(super) body: Vec<CString>,
    /// Cached panel footer lines.
    pub(super) footer: Vec<CString>,
    /// Optional focus index for editor controls.
    pub(super) focus: Option<usize>,
    /// Selected option index in the current preset slot.
    pub(super) choice: usize,
    /// Editor state discriminant used by the renderer.
    pub(super) mode: u8,
}
/// Cached row identity, enabled-state hint, and owned display text.
pub(super) struct SavedChoice {
    /// Core menu row represented by this cached label.
    pub(super) row: MenuRow,
    /// Toggle value when the row is a simple ON/OFF control.
    pub(super) enabled: Option<bool>,
    // Complete single-column lines are cached on edits, never formatted in draw.
    /// Owned NUL-terminated full row text for native rendering.
    pub(super) line: CString,
}
impl Snapshot {
    /// Category colors stay distinct from save, error, and enabled-state colors.
    pub(super) fn accent_rgb(&self) -> (f32, f32, f32) {
        let category = if self.training_categories_active {
            self.training_category
        } else if self.modes {
            2
        } else if self.buttons {
            1
        } else {
            0
        };
        Self::category_rgb(category)
    }

    pub(super) fn category_rgb(category: u8) -> (f32, f32, f32) {
        match category {
            1 => (0.75, 0.65, 1.0),
            2 => (1.0, 0.73, 0.42),
            3 => (0.96, 0.56, 0.69),
            4 => (0.53, 0.82, 0.62),
            _ => (0.45, 0.80, 1.0),
        }
    }

    /// A centered, bounded option window shared by both renderers.
    pub(super) fn row_window(&self) -> (usize, usize, usize) {
        let focus = self
            .values
            .iter()
            .position(|v| v.row == self.row)
            .unwrap_or(0);
        let start = focus
            .saturating_sub(3)
            .min(self.values.len().saturating_sub(7));
        (start, (start + 7).min(self.values.len()), focus)
    }

    /// Tests whether content/layout inputs are reusable apart from cursor and revision.
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
            && self.damage_sticks == selection.damage_sticks()
            && self.buttons == selection.button_menu
            && self.modes == selection.modes_menu
            && self.lightweight_label.is_some() == selection.lightweight
            && self.quick == (selection.quick_menu && !selection.tuning_page)
            && self.values.iter().any(|v| v.row == selection.row)
    }
    /// Updates only cursor/revision fields when the cached content remains reusable.
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
    pub(super) fn matches_layout(&self, lightweight: bool) -> bool {
        self.lightweight == lightweight
    }
}

/// Converts a boolean toggle to its label, selected index, and option count.
fn toggle_value(enabled: bool) -> (String, usize, usize) {
    (
        (if enabled { "ON" } else { "OFF" }).to_owned(),
        usize::from(enabled),
        2,
    )
}

#[cfg(test)]
/// Regressions for cached menu row labels and choice positions.
#[path = "../../../tests/unit/live/settings_menu/view/row_tests.rs"]
mod row_tests;

/// Formats the selected preset display name and its one-based position.
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
                    v.name,
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
        MenuRow::AutoCounter => selection.auto_counter,
        MenuRow::OutOfShield => {
            selection.effective_mode(Mode::OutOfShield, selection.out_of_shield)
        }
        MenuRow::ShieldPressure => selection.shield_pressure,
        MenuRow::SpotPunish => selection.effective_mode(Mode::SpotPunish, selection.spot_punish),
        MenuRow::GrabDodge => selection.grab_dodge,
        MenuRow::Performance => selection.performance,
        MenuRow::AutoTech => selection.effective_mode(Mode::AutoTech, selection.auto_tech),
        MenuRow::AutoDi => selection.damage_sticks()[0],
        MenuRow::AutoSdi => selection.damage_sticks()[1],
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
fn navigation_text(
    profile: &FighterProfile,
    selection: &MenuSelection,
    values: &[SavedChoice],
) -> String {
    let list = if selection.training_categories_active
        && selection.training_category < MenuSelection::TRAINING_CATEGORIES.len() as u8
    {
        MenuSelection::TRAINING_CATEGORIES[selection.training_category as usize]
    } else if selection.button_menu {
        "BUTTON CONFIGS"
    } else {
        "LEDGE TRAPS"
    };
    let name = profile.name;
    if selection.assist_rule_page() {
        format!("{name} / ASSIST RULES")
    } else if selection.tuning_page {
        format!("{name} / FIGHTER SETTINGS")
    } else if selection.training_categories_active && selection.training_tab_focus {
        format!("{name}   /   {list}   /   CATEGORY")
    } else if selection.quick_menu {
        format!("{name}   /   {list}")
    } else if selection.training_categories_active
        && !selection.recipe_editing
        && !selection.managing()
    {
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
fn recipe_panel(profile: &FighterProfile, selection: &MenuSelection) -> RecipePanel {
    if selection.lightweight {
        RecipePanel::default()
    } else if selection.tuning_page || selection.training_tab_focus {
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
pub(super) fn build(
    profile: &FighterProfile,
    selection: &MenuSelection,
    auto_mash: bool,
    generation: u64,
) -> Result<Snapshot, &'static str> {
    let values = build_values(profile, selection, auto_mash)?;
    // Match pages use the compact label instead of Training's full card.
    let navigation = if selection.lightweight {
        None
    } else {
        let text = format!(
            "{}   /   L: {}",
            navigation_text(profile, selection, &values),
            if selection.l_spacing_selected() {
                "SPACING"
            } else {
                "LEDGE TRAP"
            }
        );
        Some(text_cache::copy_text(&text)?.ok_or("Invalid menu heading")?)
    };
    let lightweight_label = if selection.lightweight {
        let page = if selection.modes_menu {
            "ON/OFF SETTINGS"
        } else if selection.button_menu {
            "BUTTON BINDINGS"
        } else {
            "LEDGE TRAP"
        };
        let mode = if selection.l_spacing_selected() {
            "SPACING"
        } else {
            "LEDGE TRAP"
        };
        Some(
            text_cache::copy_text(&format!("{page}  |  L: {mode}"))?
                .ok_or("Invalid lightweight page label")?,
        )
    } else {
        None
    };
    let panel = recipe_panel(profile, selection);
    // Some help panels repeat the same paragraph in the heading and body.
    // Keep its body copy without hiding status or editor instructions.
    let header_lines: Vec<String> = panel
        .header
        .iter()
        .filter(|line| {
            !panel.body.iter().any(|body| {
                line.as_str() == body.as_str() || line.strip_prefix("Help: ") == Some(body.as_str())
            })
        })
        .cloned()
        .collect();
    let header = text_cache::copy_lines(&header_lines)?;
    let body = text_cache::copy_lines(&panel.body)?;
    let footer = text_cache::copy_lines(&panel.footer)?;
    let categories = if selection.training_categories_active {
        text_cache::copy_lines(
            &MenuSelection::TRAINING_CATEGORIES
                .iter()
                .map(|s| (*s).to_owned())
                .collect::<Vec<_>>(),
        )?
    } else {
        Vec::new()
    };
    let row_context = text_cache::copy_lines(
        &(0..values.len())
            .map(|focus| {
                let start = focus.saturating_sub(3).min(values.len().saturating_sub(7));
                let end = (start + 7).min(values.len());
                format!(
                    "OPTION {}/{}  |  {} {}-{} {}",
                    focus + 1,
                    values.len(),
                    if start > 0 { "^" } else { "" },
                    start + 1,
                    end,
                    if end < values.len() { "v" } else { "" }
                )
            })
            .collect::<Vec<_>>(),
    )?;
    Ok(Snapshot {
        revision: selection.revision,
        generation,
        content_revision: selection.content_revision,
        recipe_revision: selection.recipe_revision,
        auto_mash,
        damage_sticks: selection.damage_sticks(),
        quick: selection.quick_menu && !selection.tuning_page,
        lightweight: selection.lightweight,
        buttons: selection.button_menu,
        modes: selection.modes_menu,
        training_tab_focus: selection.training_tab_focus,
        training_categories_active: selection.training_categories_active,
        training_category: selection.training_category,
        category_controls: !selection.lightweight
            && !selection.managing()
            && (!selection.recipe_editing || selection.tuning_page)
            && !selection.assist_rule_page(),
        fighter: profile.fighter_kind,
        row: selection.row,
        values,
        navigation,
        lightweight_label,
        categories,
        row_context,
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
        } else if selection.tuning_page {
            3
        } else if selection.assist_rule_page() {
            4
        } else {
            u8::from(selection.recipe_editing)
        },
    })
}
