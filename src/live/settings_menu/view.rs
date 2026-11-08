//! Cached menu text shared by the native renderer and host rendering tests.
//! Builds only on content changes; lightweight contains option rows only.
use kirby_assist_core::{
    fighter_profiles::{spacing, FighterProfile},
    settings_menu::{MenuRow, MenuSelection},
};
use std::ffi::CString;
#[path = "text_cache.rs"]
mod text_cache;

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
        if selection.assist_rule_page() {
            continue;
        }
        if !selection.row_visible(row) {
            continue;
        }
        let (name, index, count) = if MenuSelection::page_link(row) {
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
            (
                if selection.effective_mode(
                    kirby_assist_core::button_bindings::Mode::IdleCrouch,
                    selection.idle_crouch,
                ) {
                    "ON"
                } else {
                    "OFF"
                }
                .to_owned(),
                usize::from(selection.effective_mode(
                    kirby_assist_core::button_bindings::Mode::IdleCrouch,
                    selection.idle_crouch,
                )),
                2,
            )
        } else if row == MenuRow::TiltTurn {
            (
                if selection.effective_mode(
                    kirby_assist_core::button_bindings::Mode::TiltTurn,
                    selection.tilt_turn,
                ) {
                    "ON"
                } else {
                    "OFF"
                }
                .to_owned(),
                usize::from(selection.effective_mode(
                    kirby_assist_core::button_bindings::Mode::TiltTurn,
                    selection.tilt_turn,
                )),
                2,
            )
        } else if row == MenuRow::AutoReflect {
            (
                (if selection.auto_reflect { "ON" } else { "OFF" }).to_owned(),
                usize::from(selection.auto_reflect),
                2,
            )
        } else if row == MenuRow::OutOfShield {
            let enabled = selection.effective_mode(
                kirby_assist_core::button_bindings::Mode::OutOfShield,
                selection.out_of_shield,
            );
            (
                (if enabled { "ON" } else { "OFF" }).to_owned(),
                usize::from(enabled),
                2,
            )
        } else if row == MenuRow::JabLock
            && selection.y_fighter_kind == kirby_assist_core::fighter_kinds::YOSHI
        {
            (
                if selection.effective_mode(
                    kirby_assist_core::button_bindings::Mode::JabLock,
                    selection.jab_lock,
                ) {
                    "ON"
                } else {
                    "OFF"
                }
                .to_owned(),
                usize::from(selection.effective_mode(
                    kirby_assist_core::button_bindings::Mode::JabLock,
                    selection.jab_lock,
                )),
                2,
            )
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
            (
                if enabled { "ON" } else { "OFF" }.to_owned(),
                usize::from(enabled),
                2,
            )
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
            let enabled = selection.performance;
            (
                if enabled { "ON" } else { "OFF" }.to_owned(),
                usize::from(enabled),
                2,
            )
        } else if row == MenuRow::AutoTech {
            (
                if selection.effective_mode(
                    kirby_assist_core::button_bindings::Mode::AutoTech,
                    selection.auto_tech,
                ) {
                    "ON"
                } else {
                    "OFF"
                }
                .to_owned(),
                usize::from(selection.effective_mode(
                    kirby_assist_core::button_bindings::Mode::AutoTech,
                    selection.auto_tech,
                )),
                2,
            )
        } else if row == MenuRow::AutoLedge {
            let enabled = selection.ledge_mode();
            (
                (if enabled { "ON" } else { "OFF" }).to_owned(),
                usize::from(enabled),
                2,
            )
        } else if row == MenuRow::RollRules {
            (
                (if selection.roll_rules { "ON" } else { "OFF" }).to_owned(),
                usize::from(selection.roll_rules),
                2,
            )
        } else if row == MenuRow::AutoMash {
            (
                if auto_mash { "ON" } else { "OFF" }.to_owned(),
                usize::from(auto_mash),
                2,
            )
        } else {
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
