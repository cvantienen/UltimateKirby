//! Shared settings menu state for fighter recipes, feature modes and input bindings.
//! Minus toggles the panel; ledge choices and other settings are saved per fighter.
const MENU_REPEAT_INITIAL_DELAY_TICKS: u16 = 24;
const MENU_REPEAT_FAST_MODE_TICKS: u16 = 90;
const MENU_REPEAT_FAST_INTERVAL_TICKS: u16 = 2;
const MENU_REPEAT_SLOW_INTERVAL_TICKS: u16 = 6;

fn menu_repeat_due(ticks: u16) -> bool {
    ticks >= MENU_REPEAT_INITIAL_DELAY_TICKS
        && (ticks - MENU_REPEAT_INITIAL_DELAY_TICKS)
            % if ticks >= MENU_REPEAT_FAST_MODE_TICKS {
                MENU_REPEAT_FAST_INTERVAL_TICKS
            } else {
                MENU_REPEAT_SLOW_INTERVAL_TICKS
            }
            == 0
}

#[cfg(test)]
#[path = "../tests/unit/settings_menu/binding_tests.rs"]
mod binding_tests;
#[cfg(test)]
#[path = "../tests/unit/settings_menu/button_tests.rs"]
mod button_tests;
#[path = "settings_menu/choices.rs"]
mod choices;
#[path = "settings_menu/editor.rs"]
mod editor;
#[path = "settings_menu/mode_editor.rs"]
mod mode_editor;
#[path = "settings_menu/reference.rs"]
mod reference;
use crate::manual_controls::DPAD_LEFT;
pub use choices::Choice;
#[path = "settings_menu/bindings.rs"]
mod bindings;
#[path = "settings_menu/catalog.rs"]
mod catalog;
use crate::fighter_profiles::presets::{MenuPresets, PresetSlot};
use crate::ledge_trap::DPAD_DOWN;
use crate::manual_controls::{DPAD_RIGHT, DPAD_SIDES, DPAD_UP};
use crate::native_input::{MAPPER_DPAD_BUTTONS, MAPPER_L, MAPPER_MINUS, MAPPER_X, MAPPER_Y};
use crate::settings::AssistSettings;
pub const MENU_STICK_CLICK: u32 = 1 << 17; // Physical right-stick click (R3).
pub const MENU_L3: u32 = 1 << 16; // L3 is a menu control only while the menu is visible.
pub const MENU_TOGGLE_CHORD: u32 = MAPPER_MINUS;
pub fn monitor_chord(buttons: u32) -> bool {
    let mask = MAPPER_L | DPAD_DOWN | MENU_STICK_CLICK;
    buttons & mask == mask
}
pub fn menu_toggle_pressed(_held: u32, pressed: u32) -> bool {
    pressed & MAPPER_MINUS != 0
}

/// Only the dedicated L trigger bypasses menu consumption for readiness.
/// Bound D-pad/X/Y inputs must not also reach legacy shortcut observers.
pub fn observation_buttons(physical: u32, consumed: u32) -> u32 {
    physical & !(consumed & !MAPPER_L)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuRow {
    #[default]
    Neutral,
    Jump,
    Roll,
    Attack,
    Drop,
    XSequence,
    YSequence,
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    RollRead,
    AutoMash,
    IdleCrouch,
    TiltTurn,
    JabLock,
    SpotPunish,
    GrabDodge,
    AutoTech,
    Performance,
    AutoLedge,
    OutOfShield,
    TestLatency = 26,
    ButtonsPage,
    ModesPage,
    LedgePage,
    RollRules,
    FighterSettings,
    ShieldPressure,
    L3Sequence,
    R3Sequence,
    AutoReflect,
}
impl MenuRow {
    pub const ALL: [Self; 32] = [
        Self::Neutral,
        Self::Jump,
        Self::Roll,
        Self::Attack,
        Self::Drop,
        Self::XSequence,
        Self::YSequence,
        Self::DpadUp,
        Self::DpadDown,
        Self::DpadLeft,
        Self::DpadRight,
        Self::RollRead,
        Self::AutoMash,
        Self::IdleCrouch,
        Self::TiltTurn,
        Self::JabLock,
        Self::SpotPunish,
        Self::GrabDodge,
        Self::AutoTech,
        Self::Performance,
        Self::AutoLedge,
        Self::OutOfShield,
        Self::TestLatency,
        Self::ButtonsPage,
        Self::ModesPage,
        Self::LedgePage,
        Self::RollRules,
        Self::FighterSettings,
        Self::ShieldPressure,
        Self::L3Sequence,
        Self::R3Sequence,
        Self::AutoReflect,
    ];
    pub fn slot(self) -> Option<PresetSlot> {
        match self {
            Self::Neutral => Some(PresetSlot::Neutral),
            Self::Jump => Some(PresetSlot::Jump),
            Self::Roll => Some(PresetSlot::Roll),
            Self::Attack => Some(PresetSlot::Attack),
            Self::Drop => Some(PresetSlot::Drop),
            Self::XSequence => Some(PresetSlot::X),
            Self::YSequence => Some(PresetSlot::Y),
            Self::DpadUp => Some(PresetSlot::DpadUp),
            Self::DpadDown => Some(PresetSlot::DpadDown),
            Self::DpadLeft => Some(PresetSlot::DpadLeft),
            Self::DpadRight => Some(PresetSlot::DpadRight),
            Self::L3Sequence => Some(PresetSlot::L3),
            Self::R3Sequence => Some(PresetSlot::R3),
            Self::RollRead
            | Self::AutoMash
            | Self::IdleCrouch
            | Self::TiltTurn
            | Self::JabLock
            | Self::SpotPunish
            | Self::GrabDodge
            | Self::AutoTech
            | Self::Performance
            | Self::AutoLedge
            | Self::OutOfShield
            | Self::AutoReflect
            | Self::TestLatency
            | Self::ButtonsPage
            | Self::ModesPage
            | Self::LedgePage
            | Self::RollRules
            | Self::FighterSettings
            | Self::ShieldPressure => None,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Neutral => "Neutral",
            Self::Jump => "Jump",
            Self::Roll => "Roll",
            Self::Attack => "Attack",
            Self::Drop => "Drop",
            Self::XSequence => "X button",
            Self::YSequence => "Y button",
            Self::DpadUp => "D-pad Up",
            Self::DpadDown => "D-pad Down",
            Self::DpadLeft => "D-pad Left",
            Self::DpadRight => "D-pad Right",
            Self::L3Sequence => "L3 button",
            Self::R3Sequence => "R3 button",
            Self::RollRead => "Legacy roll read",
            Self::RollRules => "Auto react",
            Self::ShieldPressure => "Shield pressure",
            Self::FighterSettings => "Fighter settings",
            Self::AutoMash => "Auto-mash",
            Self::IdleCrouch => "Idle crouch",
            Self::TiltTurn => "Tilt face",
            Self::JabLock => "Jab lock",
            Self::SpotPunish => "Spot punish",
            Self::GrabDodge => "Dodge incoming grabs",
            Self::AutoTech => "Auto-tech",
            Self::Performance => "Performance gauge",
            Self::AutoLedge => "Ledge trap mode",
            Self::OutOfShield => "Automatic grabs",
            Self::AutoReflect => "Auto-reflect",
            Self::TestLatency => "Test delay label",
            Self::ButtonsPage => "Button recipes",
            Self::ModesPage => "Assist modes",
            Self::LedgePage => "Ledge traps",
        }
    }
}
/// Cached presentation data; no execution state or native UI dependencies.
#[derive(Default)]
pub struct RecipePanel {
    pub header: Vec<String>,
    pub body: Vec<String>,
    pub footer: Vec<String>,
    pub focus: Option<usize>,
}
impl RecipePanel {
    pub fn into_lines(self) -> Vec<String> {
        self.header
            .into_iter()
            .chain(self.body)
            .chain(self.footer)
            .collect()
    }
}
#[derive(Default)]
pub struct MenuSelection {
    binding_input: Option<(u32, crate::native_input::ControlLayout)>,
    binding_layout: crate::native_input::ControlLayout,
    binding_physical: u32,
    binding_pressed: u32,
    binding_previous: Option<u32>,
    binding_ledge_previous: Option<bool>,
    binding_ready: u32,
    binding_starts_blocked: bool,
    binding_safety_jump: bool,
    binding_roles_stamp: Option<(
        crate::native_input::ControlLayout,
        [usize; PresetSlot::COUNT],
        u64,
        u32,
        bool,
    )>,
    #[cfg(test)]
    binding_roles_rebuilds: usize,
    single_jump_sources: u32,
    builtin_sources: [u32; 4],
    pub builtin_consumed: u32,
    binding_sources: u32,
    pub row: MenuRow,
    /// Stable indices into five ledge lists and two manual-button lists.
    pub choices: [usize; crate::fighter_profiles::presets::PresetSlot::COUNT],
    pub roll_rules: bool,
    /// Native runtime uses the simplified event controller; legacy readers stay loadable.
    pub simplified_reactions: bool,
    pub shield_pressure: bool,
    pub rule_toggles: u8,
    pub rule_button_mask: u32,
    rule_binding_stamp: Option<u64>,
    pub rule_feedback: crate::reaction_rules::Feedback,
    pub rule_last_priority: Option<u8>,
    pub y_fighter_kind: i32,
    pub revision: u64,
    /// Displayed values/page changed; cursor-only navigation leaves this stable.
    pub content_revision: u64,
    pub recipe_revision: u64,
    mode_editor_origin: Option<(
        MenuRow,
        [usize; crate::fighter_profiles::presets::PresetSlot::COUNT],
    )>,
    mode_rule_list: bool,
    mode_rule_cursor: usize,
    auto_add_event: usize,
    auto_grab_configured: u8,
    pub has_reactions: bool,
    pub recipe_editing: bool,
    pub preview_slot: Option<PresetSlot>,
    // Suppress a bound physical button until release, even after its tape ends.
    button_latch: u32,
    held_binding: Option<PresetSlot>,
    pub save_requested: bool,
    pub delete_requested: bool,
    pub default_requested: bool,
    /// Match-only two-page selector; Training retains the full editor.
    pub lightweight: bool,
    lightweight_initialized: bool,
    match_visibility_pending: bool,
    catalog: Option<Box<catalog::Catalog>>,
    pub save_status: Option<String>,
    editor: Option<Box<editor::RecipeEditor>>,
    edit_hold_direction: u32,
    edit_hold_ticks: u16,
    /// Minus hides/shows the panel and never starts a gameplay tool.
    pub menu_hidden: bool,
    /// First Minus page: saved recipe selection and Training L3 test, without editing.
    pub quick_menu: bool,
    /// Third page groups existing non-sequence modes and toggles.
    pub modes_menu: bool,
    pub button_menu: bool,
    pub auto_ledge: bool,
    pub tuning: crate::fighter_tuning::Tuning,
    pub tuning_values: crate::fighter_tuning::Values,
    pub tuning_page: bool,
    pub tuning_row: usize,
    pub tuning_save_requested: bool,

    pub out_of_shield: bool,
    /// One input-sample event. The adapter owns the shared mash toggle so it
    /// survives fighter/stock changes rather than being a fighter preset.
    pub toggle_mash: bool,
    pub idle_crouch: bool,
    pub tilt_turn: bool,
    pub jab_lock: bool,
    pub spot_punish: bool,
    pub grab_dodge: bool,
    pub auto_tech: bool,
    pub auto_reflect: bool,
    pub performance: bool,
    /// Diagnostic annotation only: 0=unset, 1..=31 label external delay 0..=30f.
    pub test_latency: u8,
    previous: Option<u32>,
    // Menu-held directions must be released before becoming gameplay triggers.
    pad_quarantine: u32,
    // A held button must be released after reset/config changes before it can
    // control an assist. Unowned gameplay input remains physical.
    trigger_ready: u32,
    pub cancel_pending: bool,
}
impl MenuSelection {
    /// Estimated owned payload, not allocator bookkeeping or game memory.
    /// Diagnostic adapter calls only after revisions, never scans on idle polls.
    #[cfg(feature = "diagnostic-logging")]
    pub fn diagnostic_retained(&self) -> (usize, usize, usize) {
        let (recipes, editor_bytes) = self.editor.as_ref().map_or((0, 0), |e| e.retained());
        let (drafts, catalog_bytes) = self.catalog.as_ref().map_or((0, 0), |c| {
            let name_bytes = match &c.mode {
                catalog::Mode::Name { text, .. } => text.capacity(),
                _ => 0,
            };
            (
                c.drafts.len(),
                std::mem::size_of::<catalog::Catalog>()
                    + name_bytes
                    + c.drafts
                        .values()
                        .map(|d| {
                            std::mem::size_of::<((usize, usize), catalog::Draft)>()
                                + d.id.capacity()
                                + d.name.capacity()
                        })
                        .sum::<usize>(),
            )
        });
        (
            recipes,
            drafts,
            std::mem::size_of::<Self>()
                + editor_bytes
                + catalog_bytes
                + self.save_status.as_ref().map_or(0, String::capacity),
        )
    }
    pub const QUICK_ROWS: [MenuRow; crate::fighter_profiles::presets::PresetSlot::COUNT] = [
        MenuRow::Neutral,
        MenuRow::Jump,
        MenuRow::Roll,
        MenuRow::Attack,
        MenuRow::Drop,
        MenuRow::XSequence,
        MenuRow::YSequence,
        MenuRow::DpadUp,
        MenuRow::DpadDown,
        MenuRow::DpadLeft,
        MenuRow::DpadRight,
        MenuRow::L3Sequence,
        MenuRow::R3Sequence,
    ];
    pub const LEDGE_ROWS: [MenuRow; 8] = [
        MenuRow::Neutral,
        MenuRow::Jump,
        MenuRow::Roll,
        MenuRow::Attack,
        MenuRow::Drop,
        MenuRow::ButtonsPage,
        MenuRow::ModesPage,
        MenuRow::FighterSettings,
    ];
    pub const BUTTON_ROWS: [MenuRow; 10] = [
        MenuRow::XSequence,
        MenuRow::YSequence,
        MenuRow::DpadUp,
        MenuRow::DpadDown,
        MenuRow::DpadLeft,
        MenuRow::DpadRight,
        MenuRow::L3Sequence,
        MenuRow::R3Sequence,
        MenuRow::LedgePage,
        MenuRow::ModesPage,
    ];
    pub const ASSIST_ROWS: [MenuRow; 19] = [
        MenuRow::RollRules,
        MenuRow::ShieldPressure,
        MenuRow::SpotPunish,
        MenuRow::OutOfShield,
        MenuRow::GrabDodge,
        MenuRow::AutoReflect,
        MenuRow::AutoLedge,
        MenuRow::AutoTech,
        MenuRow::AutoMash,
        MenuRow::TiltTurn,
        MenuRow::IdleCrouch,
        MenuRow::JabLock,
        MenuRow::RollRead,
        MenuRow::Performance,
        MenuRow::TestLatency,
        MenuRow::LedgePage,
        MenuRow::ButtonsPage,
        MenuRow::FighterSettings,
        MenuRow::ModesPage,
    ];
    const LIGHTWEIGHT_ROWS: [MenuRow; 27] = {
        let mut rows = [MenuRow::Neutral; 27];
        let mut i = 0;
        while i < PresetSlot::BUTTONS.len() {
            rows[i] = Self::BUTTON_ROWS[i];
            i += 1;
        }
        i = 0;
        while i < Self::ASSIST_ROWS.len() {
            rows[PresetSlot::BUTTONS.len() + i] = Self::ASSIST_ROWS[i];
            i += 1;
        }
        rows
    };
    fn row_repeat(&mut self, pad: u32) -> bool {
        if pad.count_ones() != 1 || pad & (DPAD_UP | DPAD_DOWN) == 0 {
            self.edit_hold_direction = 0;
            self.edit_hold_ticks = 0;
            return false;
        }
        if self.edit_hold_direction != pad {
            self.edit_hold_direction = pad;
            self.edit_hold_ticks = 0;
            return false;
        }
        self.edit_hold_ticks = self.edit_hold_ticks.saturating_add(1);
        let t = self.edit_hold_ticks;
        menu_repeat_due(t)
    }
    // The fixed ledge modifier owns lightweight navigation. Custom D-pad
    // bindings (including a D-pad assigned to ledge assist) remain gameplay.
    fn lightweight_navigation(&self) -> bool {
        self.binding_physical & self.binding_layout.physical_buttons(MAPPER_L) != 0
    }
    fn navigate_row(&mut self, forward: bool) {
        let rows = self.display_rows();
        let mut index = rows.iter().position(|r| *r == self.row).unwrap_or(0);
        for _ in 0..rows.len() {
            index = cycle(index, rows.len(), forward);
            if self.row_visible(rows[index]) {
                self.row = rows[index];
                break;
            }
        }
    }
    pub fn display_rows(&self) -> &[MenuRow] {
        if self.modes_menu {
            &Self::ASSIST_ROWS
        } else if self.lightweight && self.button_menu {
            &Self::LIGHTWEIGHT_ROWS
        } else {
            &MenuRow::ALL
        }
    }
    fn page_rows(&self) -> &'static [MenuRow] {
        if self.button_menu {
            &Self::BUTTON_ROWS
        } else {
            &Self::LEDGE_ROWS
        }
    }
    pub fn page_link(row: MenuRow) -> bool {
        matches!(
            row,
            MenuRow::LedgePage
                | MenuRow::ButtonsPage
                | MenuRow::ModesPage
                | MenuRow::FighterSettings
        )
    }
    fn open_page(&mut self, row: MenuRow) {
        self.edit_hold_direction = 0;
        self.edit_hold_ticks = 0;
        self.menu_hidden = false;
        self.tuning_page = row == MenuRow::FighterSettings;
        self.recipe_editing = self.tuning_page;
        self.modes_menu = row == MenuRow::ModesPage;
        self.button_menu = row == MenuRow::ButtonsPage;
        self.quick_menu = !self.modes_menu;
        self.row = match row {
            MenuRow::ButtonsPage => MenuRow::XSequence,
            MenuRow::ModesPage => MenuRow::RollRules,
            MenuRow::FighterSettings => MenuRow::FighterSettings,
            _ => MenuRow::Neutral,
        };
        if let Some(c) = self.catalog.as_mut() {
            c.close();
        }
        if self.lightweight && self.preview_slot.is_none() && self.held_binding.is_none() {
            self.content_revision = self.content_revision.wrapping_add(1);
            self.view_changed();
        } else {
            self.changed();
        }
    }
    pub fn row_visible(&self, row: MenuRow) -> bool {
        if self.tuning_page {
            return false;
        }
        if Self::page_link(row) {
            if self.lightweight {
                return false;
            }
            return match row {
                MenuRow::FighterSettings => true,
                MenuRow::LedgePage => self.button_menu || self.modes_menu,
                MenuRow::ButtonsPage => !self.button_menu,
                MenuRow::ModesPage => !self.modes_menu,
                _ => false,
            };
        }

        (if self.lightweight && self.button_menu {
            row.slot().is_none() || row.slot().is_some_and(|s| s.is_button())
        } else if self.modes_menu {
            row.slot().is_none()
        } else {
            row.slot()
                .is_some_and(|s| s.is_button() == self.button_menu)
        }) && !(row == MenuRow::JabLock && self.y_fighter_kind != crate::fighter_kinds::YOSHI)
            && !(!cfg!(feature = "diagnostic-logging")
                && matches!(row, MenuRow::Performance | MenuRow::TestLatency))
            && row != MenuRow::RollRead
            && !(self.simplified_reactions
                && matches!(row, MenuRow::ShieldPressure | MenuRow::SpotPunish))
            && !(row == MenuRow::IdleCrouch && self.y_fighter_kind != crate::fighter_kinds::KIRBY)
            && !(row == MenuRow::AutoReflect
                && crate::projectile_reflect::Config::for_fighter(self.y_fighter_kind, self.tuning)
                    .is_none())
    }
    pub fn new(presets: &MenuPresets) -> Self {
        Self {
            choices: std::array::from_fn(|i| {
                let list = presets.lists[i];
                if list.options.is_empty() && list.selected == 0 {
                    PresetSlot::ALL[i].default_index(0)
                } else {
                    list.selected
                }
            }),
            roll_rules: true,
            shield_pressure: true,
            y_fighter_kind: crate::fighter_kinds::KIRBY,
            idle_crouch: true,
            tilt_turn: true,
            spot_punish: true,
            auto_tech: true,
            grab_dodge: true,
            out_of_shield: true,
            menu_hidden: true,
            ..Self::default()
        }
    }
    /// Explicit previews/bindings can run before a fighter has file recipes.
    fn adjust_mode(&mut self, forward: bool) {
        if self.row == MenuRow::ShieldPressure {
            self.shield_pressure = !self.shield_pressure;
        } else if self.row == MenuRow::RollRules {
            self.roll_rules = !self.roll_rules;
        } else if self.row == MenuRow::AutoMash {
            self.toggle_mash = true;
        } else if self.row == MenuRow::AutoTech {
            self.auto_tech = !self.auto_tech;
        } else if self.row == MenuRow::AutoReflect {
            self.auto_reflect = !self.auto_reflect;
        } else if self.row == MenuRow::AutoLedge {
            self.auto_ledge = !self.auto_ledge;
        } else if self.row == MenuRow::TestLatency {
            self.test_latency = cycle(self.test_latency as usize, 32, forward) as u8;
        } else if self.row == MenuRow::Performance {
            self.performance = !self.performance;
        } else if self.row == MenuRow::IdleCrouch {
            self.idle_crouch = !self.idle_crouch;
        } else if self.row == MenuRow::TiltTurn {
            self.tilt_turn = !self.tilt_turn;
        } else if self.row == MenuRow::OutOfShield {
            self.out_of_shield = !self.out_of_shield;
        } else if self.row == MenuRow::JabLock {
            self.jab_lock = !self.jab_lock;
        } else if self.row == MenuRow::SpotPunish {
            self.spot_punish = !self.spot_punish;
        } else if self.row == MenuRow::GrabDodge {
            self.grab_dodge = !self.grab_dodge;
        }
    }
    /// Ledge autopilot stays OFF unless separately enabled by the user.
    pub fn sequence_enabled(&self, base: bool) -> bool {
        base || self.preview_slot.is_some() || self.ledge_mode()
    }
    pub fn settings(&self, presets: &MenuPresets, base: AssistSettings) -> AssistSettings {
        // Cache selected file timelines. There is no fighter-function fallback.
        let mut s = self.tuning.apply(base);
        s.enabled = self.sequence_enabled(base.enabled);
        s.sequence_overrides = [None; crate::fighter_profiles::presets::PresetSlot::COUNT];
        s.response_choices = [None; crate::fighter_profiles::presets::PresetSlot::COUNT];
        for slot in PresetSlot::ALL {
            let index = self.choices[slot as usize];
            let source = self.choice(presets, slot, index).map(|c| c.sequence);
            if let Some(source) = source {
                if source.mode.is_some() {
                    continue;
                }
                let mut sequence = self
                    .editor
                    .as_ref()
                    .and_then(|e| e.compiled((slot as usize, index)))
                    .unwrap_or(source);
                if sequence.reaction.active() && self.preview_slot != Some(slot) {
                    continue;
                }
                if !slot.is_button() {
                    sequence.while_held = false;
                }
                if sequence.start_delay == 0
                    && sequence
                        .actions()
                        .iter()
                        .all(|a| *a == crate::sequence_config::Action::Empty)
                {
                    continue;
                }
                s.sequence_overrides[slot as usize] = Some(sequence);
                s.response_choices[slot as usize] = Some(index);
            }
        }
        s
    }
    pub fn recipe_lines(&self, presets: &MenuPresets) -> Vec<String> {
        self.recipe_panel(presets).into_lines()
    }
    fn tuning_visible(&self, i: usize) -> bool {
        crate::fighter_tuning::Tuning::visible(self.y_fighter_kind, i)
            && !(i == crate::fighter_tuning::AUTO_GRAB_CLOSE && self.auto_grab_configured & 2 != 0)
            && !(i == crate::fighter_tuning::AUTO_GRAB_FAR && self.auto_grab_configured & 1 != 0)
    }
    fn tuning_visible_index(&self, row: usize) -> usize {
        (row..crate::fighter_tuning::COUNT)
            .find(|i| self.tuning_visible(*i))
            .or_else(|| (0..crate::fighter_tuning::COUNT).find(|i| self.tuning_visible(*i)))
            .unwrap_or(0)
    }
    pub fn recipe_panel(&self, presets: &MenuPresets) -> RecipePanel {
        if self.mode_rule_list && !self.managing() {
            return self.mode_rules_panel(presets);
        }
        if self.tuning_page {
            let row = self.tuning_visible_index(self.tuning_row);
            return RecipePanel {
                header: vec![
                    "FIGHTER SETTINGS / Training only".into(),
                    crate::fighter_tuning::HINTS[row].into(),
                ],
                body: crate::fighter_tuning::LABELS
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| self.tuning_visible(*i))
                    .map(|(i, label)| {
                        let value = self.tuning.0[i].unwrap_or(self.tuning_values[i]);
                        let shown = if crate::fighter_tuning::Tuning::is_bool(i) {
                            if value == 1.0 {
                                "ON".into()
                            } else {
                                "OFF".into()
                            }
                        } else if i == 0 || i == 7 {
                            format!("{:.0}%", value * 100.0)
                        } else if crate::fighter_tuning::Tuning::frames(i) {
                            format!("{value:.1} frames")
                        } else {
                            format!("{value:.1} units")
                        };
                        format!(
                            "{} {label}: {shown}{}",
                            if i == row { ">" } else { " " },
                            if self.tuning.0[i].is_some() {
                                " [custom]"
                            } else if i == 1 {
                                " [fighter reference]"
                            } else if i >= 14 {
                                if crate::fighter_tuning::Tuning::frames(i) {
                                    " [policy default]"
                                } else {
                                    " [estimate]"
                                }
                            } else {
                                " [default]"
                            }
                        )
                    })
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
            };
        }
        let Some(slot) = self.row.slot() else {
            return RecipePanel::default();
        };
        let index = self.choices[slot as usize];
        let key = (slot as usize, index);
        if let Some(catalog) = self.catalog.as_ref().filter(|c| c.active()) {
            let mut lines = catalog.lines();
            let header = if lines.is_empty() {
                vec![]
            } else {
                vec![lines.remove(0)]
            };
            let focus = lines.iter().position(|l| l.starts_with('>'));
            let mut panel = RecipePanel {
                header,
                body: lines,
                footer: vec![],
                focus,
            };
            if matches!(&catalog.mode, catalog::Mode::Actions(_)) {
                panel
                    .footer
                    .push("Up/Down: select | A: choose | B: back".into());
            }
            if let Some(status) = &self.save_status {
                panel.footer.push(status.clone());
            }
            return panel;
        }
        if let Some(builtin) = self
            .choice(presets, slot, index)
            .and_then(|c| c.sequence.builtin)
        {
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
        let mut panel = if let Some(editor) = self.editor.as_ref().filter(|e| e.has(key)) {
            editor.panel(key, self.recipe_editing)
        } else {
            let sequence = self
                .choice(presets, slot, index)
                .map(|c| c.sequence)
                .unwrap_or_else(editor::RecipeEditor::custom);
            editor::RecipeEditor::preview_panel(sequence)
        };
        if !self.editor.as_ref().is_some_and(|e| e.command_active()) {
            panel.footer.push(if self.recipe_editing {
                "B: Back | A / ZL+ZR: Step tools and recipe options".into()
            } else {
                "B: Manage list (add / rename / duplicate / delete)".into()
            });
        }

        if self.recipe_editing && !self.editor.as_ref().is_some_and(|e| e.command_active()) {
            panel
                .footer
                .push("Up/Down: field | Left/Right: adjust | Y: Save".into());
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
        if self.recipe_editing
            && !self.lightweight
            && self.editor.as_ref().is_some_and(|e| e.reference_visible())
        {
            let label = self.editor.as_ref().and_then(|e| e.reference_label(key));
            let hint_slot = if self.mode_editor_origin.is_some() {
                PresetSlot::X
            } else {
                slot
            };
            panel
                .header
                .extend(reference::hints(self.y_fighter_kind, hint_slot, label));
        }
        panel
    }
    fn recipe_input_error(&self, presets: &MenuPresets) -> Option<String> {
        let slot = self.row.slot()?;
        let key = (slot as usize, self.choices[slot as usize]);
        let sequence = if let Some(editor) = self.editor.as_ref().filter(|e| e.has(key)) {
            match editor.compiled(key) {
                Some(s) => s,
                None => return Some("Too many expanded steps; remove a step or delay".into()),
            }
        } else {
            self.choice(presets, slot, key.1)?.sequence
        };
        if sequence.builtin.is_some() || sequence.mode.is_some() {
            return None;
        }
        if sequence.len == 0 {
            return Some("Add an action before testing or saving".into());
        }
        crate::reaction_rules::validate_response(sequence).err()
    }
    pub fn recipe_modified(&self, slot: PresetSlot) -> bool {
        let edited = self
            .editor
            .as_ref()
            .is_some_and(|e| e.edited((slot as usize, self.choices[slot as usize])));
        {
            return edited
                || self
                    .catalog
                    .as_ref()
                    .and_then(|c| c.drafts.get(&(slot as usize, self.choices[slot as usize])))
                    .is_some_and(|d| d.dirty);
        }
    }
    pub fn ledge_held(&self, buttons: u32) -> bool {
        // Training navigation is still a UI command even when that physical
        // direction is assigned to a gameplay helper and filtered from policy.
        if !self.menu_hidden
            && !self.lightweight
            && buttons & (DPAD_SIDES | DPAD_UP | DPAD_DOWN) != 0
        {
            return false;
        }
        let buttons = self.native_buttons(buttons);
        let directions = DPAD_SIDES | DPAD_UP | DPAD_DOWN;
        let blocked = if self.lightweight {
            // Minus is always a selector control; visible-menu D-pad is too.
            (if self.menu_hidden { directions } else { 0 }) | MAPPER_Y
        } else {
            directions | MAPPER_MINUS | MAPPER_Y
        };
        !self.cancel_pending
            && !self.managing()
            && !self.recipe_editing
            && buttons & self.trigger_ready & MAPPER_L != 0
            && buttons & blocked == 0
    }
    /// A toggled ledge watch stays reserved even while manual input temporarily
    /// pauses its movement. Safety tech/mash and ledge shield are separate.
    pub fn reserves_automatic_modes(&self) -> bool {
        self.ledge_mode()
            || self.preview_slot.is_some()
            || self.native_buttons(self.binding_layout.policy_buttons(self.binding_physical))
                & MAPPER_L
                != 0
    }
    pub fn autopilot_held(&self, buttons: u32) -> bool {
        self.ledge_mode()
            && !self.recipe_editing
            && !self.managing()
            && !self.cancel_pending
            && (buttons & MAPPER_DPAD_BUTTONS == 0
                || self.lightweight && !self.menu_hidden && self.lightweight_navigation())
    }

    pub fn auto_duck_held(&self, buttons: u32) -> bool {
        let _ = buttons;
        false // Auto-duck has no binding.
    }
    /// Minus opens ledge home or closes. R3 edits; page rows navigate directly.
    /// In Training, visible D-pad belongs to navigation. The lightweight menu
    /// requires the held ledge modifier; otherwise custom D-pad inputs work.
    pub fn input(&mut self, buttons: u32, presets: &MenuPresets) -> u32 {
        self.input_with_editing(buttons, presets, true)
    }
    /// Runtime scene permission; saved recipes and mode/choice controls remain usable.
    pub fn input_with_editing(
        &mut self,
        buttons: u32,
        presets: &MenuPresets,
        allow_editing: bool,
    ) -> u32 {
        self.capture_binding_input(buttons, presets);
        let ledge_input = self.native_buttons(buttons) & MAPPER_L != 0;
        let ledge_pressed = ledge_input && !self.binding_ledge_previous.unwrap_or(ledge_input);
        self.binding_ledge_previous = Some(ledge_input);
        // Menu navigation uses its original controls. Hidden gameplay routes
        // only the selected native helpers; custom L/X/L3 cannot double-activate.
        let buttons = if self.menu_hidden && buttons & MAPPER_MINUS == 0 {
            self.native_buttons(buttons)
        } else {
            buttons
        };
        // Battle entry can reuse a selector prepared during the intro/results.
        // Resolve its default from the actual scene, even if layout did not change.
        let starting_match = self.match_visibility_pending;
        if starting_match {
            self.match_visibility_pending = false;
            self.menu_hidden = allow_editing;
            self.lightweight_initialized = !allow_editing;
        }
        let leaving_training = !allow_editing && !self.lightweight;
        let cancel_editor = leaving_training
            && (self.preview_slot.is_some()
                || self.held_binding.is_some()
                || self.recipe_editing
                || self.tuning_page
                || self.mode_editor_origin.is_some()
                || self.managing()
                || self.editor.as_ref().is_some_and(|e| e.command_active()));
        let layout_changed = self.lightweight != !allow_editing;
        if layout_changed {
            self.lightweight = !allow_editing;
            if self.lightweight {
                if !self.lightweight_initialized {
                    self.menu_hidden = false;
                    self.lightweight_initialized = true;
                }
                let restored = self.restore_mode_choices().is_some();
                self.mode_rule_list = false;
                self.mode_rule_cursor = 0;
                self.preview_slot = None;
                self.held_binding = None;
                self.recipe_editing = false;
                self.tuning_page = false;
                self.edit_hold_direction = 0;
                self.edit_hold_ticks = 0;
                self.release_training_tools(presets);
                if let Some(catalog) = self.catalog.as_mut() {
                    catalog.close();
                }
                if restored {
                    self.refresh_native_bindings(presets);
                }
            }
            if !self.menu_hidden {
                self.open_page(MenuRow::LedgePage);
            }
            self.revision = self.revision.wrapping_add(1);
        }
        if starting_match && self.lightweight && !layout_changed {
            self.open_page(MenuRow::LedgePage);
        }
        // Cancellation belongs to one mapper sample, not a later observer.
        self.cancel_pending = cancel_editor;
        self.toggle_mash = false;
        self.save_requested = false;
        self.delete_requested = false;
        self.default_requested = false;
        self.tuning_save_requested = false;
        self.button_latch &= self.binding_layout.binding_ownership(self.binding_physical);
        let previous = self.previous.unwrap_or(buttons);
        let pressed = buttons & !previous;
        self.previous = Some(buttons);
        if self.rule_binding_stamp != Some(self.recipe_revision) {
            self.refresh_rule_bindings(presets, buttons);
            self.rule_binding_stamp = Some(self.recipe_revision);
        }
        let all_directions = DPAD_SIDES | DPAD_UP | DPAD_DOWN;
        let light_navigation = self.lightweight && self.lightweight_navigation();
        let directions = if self.lightweight && !light_navigation {
            0
        } else {
            all_directions
        };
        let pad = buttons & directions;
        let quarantined = self.pad_quarantine;
        self.pad_quarantine &= self.binding_layout.binding_ownership(self.binding_physical);
        if ledge_pressed && !self.ledge_mode() && !self.binding_starts_blocked {
            self.cancel_pending = true; // Held-L activation takes over when toggle is OFF.
        }
        // With autopilot already ON, L is only the manual walking modifier;
        // it must not reset a running response or reacquire the same ledge.
        let toggled = menu_toggle_pressed(buttons, pressed);
        if toggled {
            if self.menu_hidden {
                self.open_page(MenuRow::LedgePage);
            } else {
                self.restore_mode_choices();
                self.menu_hidden = true;
                self.tuning_page = false;
                if let Some(e) = self.editor.as_mut() {
                    e.close_commands();
                }
                self.recipe_editing = false;
                self.modes_menu = false;
                self.button_menu = false;
                if let Some(c) = self.catalog.as_mut() {
                    c.close();
                }
                if self.lightweight {
                    self.view_changed();
                } else {
                    self.changed();
                }
            }
        }
        // Lightweight R3 switches only between ledge and buttons/assists.
        // It never allocates an editor or opens list-management tools.
        if self.lightweight
            && !self.menu_hidden
            && !toggled
            && pressed & MENU_STICK_CLICK != 0
            && pad == 0
        {
            self.open_page(if self.button_menu {
                MenuRow::LedgePage
            } else {
                MenuRow::ButtonsPage
            });
            self.pad_quarantine |= buttons & bindings::BUTTON_MASK;
            return directions
                | MAPPER_MINUS
                | MENU_STICK_CLICK
                | MAPPER_L
                | (self.pad_quarantine & all_directions);
        }
        let editing = !self.menu_hidden;
        if editing && allow_editing && !toggled && self.mode_list_input(presets, buttons, pressed) {
            return buttons | directions | MAPPER_MINUS | MENU_STICK_CLICK | MENU_L3;
        }
        if editing && allow_editing && !toggled {
            if pressed & (MENU_STICK_CLICK | MENU_L3) != 0
                && self.modes_menu
                && !self.tuning_page
                && Self::mode_trigger(self.row).is_some()
            {
                self.open_mode_list(presets, self.row);
                self.changed();
                return directions | MAPPER_L | MAPPER_MINUS | MENU_STICK_CLICK;
            }
            if self.recipe_editing
                && !self.editor.as_ref().is_some_and(|e| e.command_active())
                && buttons & crate::native_input::MAPPER_R != 0
                && pressed & (DPAD_LEFT | DPAD_RIGHT) != 0
            {
                if let Some((row, _)) = self.mode_editor_origin.as_ref() {
                    let row = *row;
                    self.open_mode_response(presets, row, Some(pressed & DPAD_RIGHT != 0));
                    self.changed();
                    return directions
                        | MAPPER_L
                        | MAPPER_MINUS
                        | MENU_STICK_CLICK
                        | crate::native_input::MAPPER_R;
                }
            }
        }

        if editing || toggled {
            self.pad_quarantine |= pad;
        }
        if toggled {
            if let Some(c) = self.catalog.as_mut() {
                c.close();
            }
        }
        if editing
            && !toggled
            && !self.tuning_page
            && Self::page_link(self.row)
            && (!self.lightweight
                && !self.recipe_editing
                && pad == 0
                && pressed & (crate::native_input::MAPPER_A | MENU_STICK_CLICK) != 0
                || pad & DPAD_SIDES != 0 && pad.count_ones() == 1 && pressed & pad != 0)
        {
            self.open_page(self.row);
            self.pad_quarantine |=
                pad | (buttons & (crate::native_input::MAPPER_A | MENU_STICK_CLICK));
            return directions
                | MAPPER_MINUS
                | MAPPER_X
                | MAPPER_L
                | MENU_STICK_CLICK
                | crate::native_input::MAPPER_A;
        }
        if editing
            && allow_editing
            && !self.tuning_page
            && self.row == MenuRow::FighterSettings
            && pressed & MENU_STICK_CLICK != 0
        {
            self.open_page(MenuRow::FighterSettings);
            return directions | MAPPER_MINUS | MENU_STICK_CLICK | MAPPER_L | MAPPER_X | MAPPER_Y;
        }
        if editing && allow_editing && self.tuning_page {
            self.tuning_row = self.tuning_visible_index(self.tuning_row);
            self.trigger_ready &= !(MAPPER_L | bindings::BUTTON_MASK);
            if pressed & MENU_STICK_CLICK != 0 {
                let row = self.row;
                let assist_settings = self.simplified_reactions
                    && matches!(row, MenuRow::OutOfShield | MenuRow::GrabDodge);
                self.open_page(if assist_settings {
                    MenuRow::ModesPage
                } else {
                    MenuRow::LedgePage
                });
                if assist_settings {
                    self.row = row;
                }
            } else if pad.count_ones() == 1 && pressed & pad != 0 {
                if pad & (DPAD_UP | DPAD_DOWN) != 0 {
                    self.tuning_row = cycle(
                        self.tuning_row,
                        crate::fighter_tuning::COUNT,
                        pad == DPAD_DOWN,
                    );
                    while !self.tuning_visible(self.tuning_row) {
                        self.tuning_row = cycle(
                            self.tuning_row,
                            crate::fighter_tuning::COUNT,
                            pad == DPAD_DOWN,
                        );
                    }
                } else {
                    let i = self.tuning_row;
                    let (lo, hi, step) = crate::fighter_tuning::Tuning::bounds(i);
                    let value = self.tuning.0[i].unwrap_or(self.tuning_values[i]);
                    self.tuning.0[i] =
                        Some((value + if pad == DPAD_RIGHT { step } else { -step }).clamp(lo, hi));
                    self.recipe_revision = self.recipe_revision.wrapping_add(1);
                    self.save_status = Some("Unsaved fighter settings — Y: Save".into());
                }
                self.changed();
            } else if pressed & MAPPER_X != 0 {
                self.tuning.0[self.tuning_row] = None;
                self.recipe_revision = self.recipe_revision.wrapping_add(1);
                self.save_status = Some("Default restored — Y: Save".into());
                self.changed();
            } else if pressed & MAPPER_Y != 0 {
                self.tuning_save_requested = buttons == MAPPER_Y;
                if !self.tuning_save_requested {
                    self.save_status = Some("Release other buttons, then press Y to save".into());
                }
                self.changed();
            }
            return directions | MAPPER_MINUS | MENU_STICK_CLICK | MAPPER_L | MAPPER_X | MAPPER_Y;
        }
        let open_recipe = editing
            && allow_editing
            && !self.recipe_editing
            && !self.managing()
            && self.quick_menu
            && !toggled
            && self.row.slot().is_some()
            && pad == 0
            && pressed & MENU_STICK_CLICK != 0
            && buttons
                & (MAPPER_X
                    | MAPPER_Y
                    | crate::native_input::MAPPER_PLUS
                    | crate::native_input::MAPPER_B
                    | crate::native_input::MAPPER_A)
                == 0;
        if open_recipe {
            self.quick_menu = false;
        }
        let bound_buttons = self.process_bindings(
            presets,
            buttons,
            pressed,
            (!editing || self.lightweight && !light_navigation) && !toggled,
            quarantined,
        );
        if editing && self.quick_menu && self.lightweight {
            if light_navigation && self.preview_slot.is_some() {
                self.changed();
            }
            let repeat = if !toggled {
                self.row_repeat(pad)
            } else {
                self.edit_hold_direction = 0;
                self.edit_hold_ticks = 0;
                false
            };
            if !toggled && pad.count_ones() == 1 && (pressed & pad != 0 || repeat) {
                if pad & (DPAD_UP | DPAD_DOWN) != 0 {
                    let forward = pad == DPAD_DOWN;
                    let before = self.row;
                    self.navigate_row(forward);
                    if self.row != before {
                        self.view_changed();
                    }
                } else if let Some(slot) = self.row.slot() {
                    let next = self.cycle_quick_choice(presets, slot, pad == DPAD_RIGHT);
                    if next != self.choices[slot as usize] {
                        self.choices[slot as usize] = next;
                        self.changed();
                    }
                } else {
                    self.adjust_mode(pad == DPAD_RIGHT);
                    self.changed();
                }
            }
            // No preview/editor/save controls; physical face buttons keep their
            // normal behavior. Navigation must not interrupt held-L ledge work.
            self.trigger_ready |= !buttons & (MAPPER_L | bindings::BUTTON_MASK);
            return self.button_latch
                | directions
                | MAPPER_MINUS
                | MENU_STICK_CLICK
                | bound_buttons
                | ((quarantined | self.pad_quarantine) & all_directions);
        }
        if editing && self.quick_menu {
            // Ordinary gameplay buttons are not compact-menu controls. A manual
            // action cancels a test so its owned inputs cannot swallow that press.
            let manual = MAPPER_L
                | MAPPER_Y
                | crate::native_input::MAPPER_A
                | crate::native_input::MAPPER_B
                | crate::native_input::MAPPER_PLUS;
            if self.preview_slot.is_some() && pressed & manual != 0 && !self.binding_starts_blocked
            {
                self.changed();
            }
            let repeat = if !toggled {
                self.row_repeat(pad)
            } else {
                self.edit_hold_direction = 0;
                self.edit_hold_ticks = 0;
                false
            };
            if !toggled && pad.count_ones() == 1 && (pressed & pad != 0 || repeat) {
                if pad & (DPAD_UP | DPAD_DOWN) != 0 {
                    let index = self
                        .page_rows()
                        .iter()
                        .position(|r| *r == self.row)
                        .unwrap_or(0);
                    self.row =
                        self.page_rows()[cycle(index, self.page_rows().len(), pad == DPAD_DOWN)];
                } else if let Some(slot) = self.row.slot() {
                    self.choices[slot as usize] =
                        self.cycle_quick_choice(presets, slot, pad == DPAD_RIGHT);
                }
                self.changed();
            }
            if !toggled
                && self.binding_pressed & MENU_L3 != 0
                && pad == 0
                && buttons & (manual | MENU_STICK_CLICK) == 0
            {
                let running = self.preview_slot.is_some();
                self.changed();
                if !running {
                    if let Some(slot) = self.row.slot() {
                        if self
                            .choice(presets, slot, self.choices[slot as usize])
                            .is_some_and(|c| {
                                c.sequence.mode.is_none() && c.sequence.builtin.is_none()
                            })
                        {
                            self.preview_slot = Some(slot);
                        }
                    }
                }
            }
            // Compact selection must preserve held-L autopilot, including
            // resuming after navigation. Y remains separate from this page.
            self.trigger_ready |= !buttons & (MAPPER_L | bindings::BUTTON_MASK);
            self.trigger_ready &= !MAPPER_Y;
            self.pad_quarantine |= self
                .binding_layout
                .binding_ownership(self.binding_physical & MENU_L3);
            return self.button_latch
                | directions
                | MAPPER_MINUS
                | MENU_L3
                | MENU_STICK_CLICK
                | bound_buttons
                | ((quarantined | self.pad_quarantine)
                    & (crate::native_input::MAPPER_A
                        | crate::native_input::MAPPER_B
                        | crate::native_input::MAPPER_ZL
                        | crate::native_input::MAPPER_ZR
                        | crate::native_input::MAPPER_R));
        }
        // A modified Save press must explain why nothing was written. This
        // applies to picker/tools screens too, before they consume the input.
        if editing
            && allow_editing
            && self.recipe_editing
            && !toggled
            && pressed & MAPPER_Y != 0
            && buttons != MAPPER_Y
        {
            self.changed();
            self.save_status = Some("Release other buttons, then press Y to save".into());
            return directions | MAPPER_MINUS | MAPPER_Y | MENU_STICK_CLICK | MAPPER_L;
        }
        // Editor-only commands: no allocations or label work on the idle path.
        if editing && self.recipe_editing && !toggled && !self.managing() {
            if let Some(slot) = self.row.slot() {
                let key = (slot as usize, self.choices[slot as usize]);
                if let Some((ui_changed, mut recipe_changed)) = self
                    .editor
                    .as_mut()
                    .and_then(|e| e.command_input(key, buttons, pressed))
                {
                    if self.editor.as_mut().is_some_and(|e| e.take_copy_request()) {
                        let sources = self.template_sources(presets, key);
                        if let Some(editor) = self.editor.as_mut() {
                            editor.set_copy_sources(sources);
                            editor.set_copy_templates(presets.templates);
                        }
                    }
                    if let Some((source_slot, source_index)) =
                        self.editor.as_mut().and_then(|e| e.take_copy_apply())
                    {
                        let source = self
                            .choice(presets, source_slot, source_index)
                            .map(|c| c.sequence);
                        if let Some(editor) = self.editor.as_mut() {
                            if let Some(source) = source {
                                recipe_changed |= editor.apply_template(
                                    key,
                                    (source_slot as usize, source_index),
                                    source,
                                );
                            } else {
                                editor.reject_copy();
                            }
                        }
                    }
                    if let Some(index) = self.editor.as_mut().and_then(|e| e.take_template_apply())
                    {
                        if let Some(editor) = self.editor.as_mut() {
                            if let Some(template) = presets.templates.get(index) {
                                recipe_changed |=
                                    editor.apply_generic_template(key, template.sequence);
                            } else {
                                editor.reject_copy();
                            }
                        }
                    }
                    if self
                        .editor
                        .as_mut()
                        .is_some_and(|e| e.take_manage_request())
                    {
                        self.manage_input(
                            presets,
                            crate::native_input::MAPPER_B,
                            crate::native_input::MAPPER_B,
                        );
                    }
                    self.pad_quarantine |= buttons
                        & (directions
                            | crate::native_input::MAPPER_A
                            | crate::native_input::MAPPER_B
                            | MAPPER_X
                            | MAPPER_Y
                            | crate::native_input::MAPPER_ZL
                            | crate::native_input::MAPPER_ZR
                            | crate::native_input::MAPPER_R);
                    self.trigger_ready &= !(MAPPER_L | bindings::BUTTON_MASK);
                    self.edit_hold_direction = 0;
                    self.edit_hold_ticks = 0;
                    if recipe_changed {
                        self.recipe_revision = self.recipe_revision.wrapping_add(1);
                        self.save_status = Some("Unsaved changes — Y: Save".into());
                    }
                    if ui_changed {
                        self.changed();
                    }
                    return directions
                        | MAPPER_L
                        | MAPPER_Y
                        | MAPPER_MINUS
                        | MENU_STICK_CLICK
                        | MAPPER_X
                        | crate::native_input::MAPPER_A
                        | crate::native_input::MAPPER_B
                        | crate::native_input::MAPPER_ZL
                        | crate::native_input::MAPPER_ZR
                        | crate::native_input::MAPPER_R;
                }
            }
        }
        if editing
            && self.recipe_editing
            && !self.managing()
            && !toggled
            && pressed & crate::native_input::MAPPER_B != 0
        {
            self.show_quick();
            self.pad_quarantine |= crate::native_input::MAPPER_B;
            return directions
                | MAPPER_MINUS
                | MENU_STICK_CLICK
                | MAPPER_L
                | crate::native_input::MAPPER_B;
        }
        {
            if editing
                && !toggled
                && self.row.slot().is_some()
                && (self.managing() || pressed & crate::native_input::MAPPER_B != 0)
            {
                let was = self.catalog.as_ref().map_or(0, |c| c.revision);
                self.manage_input(presets, buttons, pressed);
                let changed =
                    was != self.catalog.as_ref().map_or(0, |c| c.revision) || self.delete_requested;
                self.pad_quarantine |= pad
                    | (buttons
                        & (crate::native_input::MAPPER_A
                            | crate::native_input::MAPPER_B
                            | MAPPER_X
                            | crate::native_input::MAPPER_ZL
                            | crate::native_input::MAPPER_ZR
                            | crate::native_input::MAPPER_R));
                self.trigger_ready &= !(MAPPER_L | MAPPER_Y);
                self.edit_hold_direction = 0;
                self.edit_hold_ticks = 0;
                if changed {
                    self.changed();
                }
                return MAPPER_L
                    | MAPPER_Y
                    | MAPPER_MINUS
                    | MENU_STICK_CLICK
                    | MAPPER_X
                    | crate::native_input::MAPPER_A
                    | crate::native_input::MAPPER_B
                    | crate::native_input::MAPPER_ZL
                    | crate::native_input::MAPPER_ZR
                    | crate::native_input::MAPPER_R
                    | directions;
            }
        }
        if toggled || pressed & MENU_STICK_CLICK != 0 {
            self.edit_hold_direction = 0;
            self.edit_hold_ticks = 0;
        }
        if self.preview_slot.is_some()
            && pressed
                & (MAPPER_L | crate::native_input::MAPPER_PLUS | if editing { MAPPER_Y } else { 0 })
                != 0
        {
            self.changed();
        }
        if editing && allow_editing && !toggled && (pressed & MENU_STICK_CLICK != 0 || open_recipe)
        {
            if self.recipe_editing {
                self.mode_cursor_to_current(presets);
                self.show_quick();
            } else if let Some(slot) = self.row.slot() {
                let index = self.choices[slot as usize];
                let source = self
                    .choice(presets, slot, index)
                    .map(|c| c.sequence)
                    .unwrap_or_else(editor::RecipeEditor::custom);
                if source.mode.is_none() && source.builtin.is_none() {
                    self.editor
                        .get_or_insert_with(|| Box::new(editor::RecipeEditor::default()))
                        .ensure((slot as usize, index), source);
                }
                self.recipe_editing = !self.recipe_editing;
                self.trigger_ready &= !(MAPPER_L | MAPPER_Y);
                self.changed();
            }
        }
        if editing && self.recipe_editing && pressed & MAPPER_Y != 0 {
            if self.row.slot().is_some_and(|slot| {
                self.choice(presets, slot, self.choices[slot as usize])
                    .is_some_and(|c| c.sequence.builtin.is_some())
            }) {
                self.save_requested = buttons == MAPPER_Y;
                self.default_requested = self.save_requested;
                self.changed();
                if !self.save_requested {
                    self.save_status = Some("Release other buttons, then press Y to save".into());
                }
                return directions | MAPPER_MINUS | MAPPER_Y | MENU_STICK_CLICK;
            }
        }
        let mut edit_repeat = false;
        if editing
            && self.recipe_editing
            && !toggled
            && pad.count_ones() == 1
            && pressed & MENU_STICK_CLICK == 0
        {
            if self.edit_hold_direction != pad {
                self.edit_hold_direction = pad;
                self.edit_hold_ticks = 0;
            } else {
                self.edit_hold_ticks = self.edit_hold_ticks.saturating_add(1);
                let ticks = self.edit_hold_ticks;
                edit_repeat = menu_repeat_due(ticks);
            }
        } else {
            self.edit_hold_direction = 0;
            self.edit_hold_ticks = 0;
        }
        if editing
            && self.recipe_editing
            && !toggled
            && pad.count_ones() == 1
            && (pressed & pad != 0 || edit_repeat)
        {
            if let Some(slot) = self.row.slot() {
                {
                    let key = (slot as usize, self.choices[slot as usize]);
                    if let Some(editor) = self.editor.as_mut() {
                        if pad & DPAD_SIDES != 0 {
                            if (!edit_repeat || editor.repeatable(key))
                                && editor.adjust(key, pad == DPAD_RIGHT)
                            {
                                self.recipe_revision = self.recipe_revision.wrapping_add(1);
                                self.save_status = Some("Unsaved changes — Y: Save".into());
                            }
                        } else {
                            editor.navigate(key, pad == DPAD_DOWN);
                        }
                    }
                }
                self.trigger_ready &= !(MAPPER_L | MAPPER_Y);
                self.changed();
            }
        } else if editing && !toggled && pad.count_ones() == 1 && pressed & pad != 0 {
            let navigating = if self.modes_menu {
                pad & (DPAD_UP | DPAD_DOWN) != 0
            } else {
                pad & DPAD_SIDES != 0
            };
            let forward = if self.modes_menu {
                pad == DPAD_DOWN
            } else {
                pad == DPAD_RIGHT
            };
            let next_value = if self.modes_menu {
                pad == DPAD_RIGHT
            } else {
                pad == DPAD_DOWN
            };
            if navigating {
                self.navigate_row(forward);
            } else if self.row.slot().is_none() {
                self.adjust_mode(next_value);
            } else if let Some(slot) = self.row.slot() {
                self.choices[slot as usize] = self.cycle_choice(presets, slot, next_value);
            }
            self.changed();
        }
        if editing
            && !toggled
            && pressed & MAPPER_X != 0
            && buttons & (MAPPER_L | MAPPER_Y | crate::native_input::MAPPER_PLUS) == 0
            && pad == 0
            && pressed & MENU_STICK_CLICK == 0
        {
            if let Some(slot) = self.row.slot() {
                let running = self.preview_slot.is_some();
                if !running {
                    if let Some(error) = self.recipe_input_error(presets) {
                        self.changed();
                        self.save_status = Some(format!("Cannot test: {error}"));
                        return directions | MAPPER_MINUS | MAPPER_X | MENU_STICK_CLICK | MAPPER_L;
                    }
                }
                self.changed();
                self.preview_slot = if running { None } else { Some(slot) };
                self.trigger_ready &= !(MAPPER_L | MAPPER_Y);
            }
        }
        if editing
            && self.recipe_editing
            && !toggled
            && pressed & MAPPER_Y != 0
            && buttons & (MAPPER_L | MAPPER_X | crate::native_input::MAPPER_PLUS | MENU_STICK_CLICK)
                == 0
            && pad == 0
        {
            self.changed();
            if let Some(error) = self.recipe_input_error(presets) {
                self.save_status = Some(format!("Cannot save: {error}"));
                return directions | MAPPER_MINUS | MAPPER_Y | MENU_STICK_CLICK | MAPPER_L;
            }
            self.save_requested = self.row.slot().is_some();
            self.trigger_ready &= !(MAPPER_L | MAPPER_Y);
        }
        if editing && self.row.slot().is_some() {
            self.pad_quarantine |= buttons & MAPPER_X;
        }
        let editor_buttons = if editing && self.recipe_editing {
            crate::native_input::MAPPER_A
                | crate::native_input::MAPPER_ZL
                | crate::native_input::MAPPER_ZR
                | crate::native_input::MAPPER_R
        } else {
            0
        };
        self.pad_quarantine |= buttons & editor_buttons;

        self.trigger_ready |= !buttons & (MAPPER_L | bindings::BUTTON_MASK);
        // Visible directions and their release edges cannot reach gameplay.
        // Hidden Left/Right/Down are reserved separately by the enabled tools.
        bound_buttons
            | editor_buttons
            | self.button_latch
            | (if editing { MAPPER_Y | MAPPER_L } else { 0 })
            | MAPPER_MINUS
            | ((quarantined | self.pad_quarantine)
                & (MAPPER_X | crate::native_input::MAPPER_A | crate::native_input::MAPPER_B))
            | if editing {
                MENU_STICK_CLICK
                    | if self.row.slot().is_some() {
                        MAPPER_X
                    } else {
                        0
                    }
            } else {
                0
            }
            | if editing {
                directions
            } else {
                quarantined | self.pad_quarantine
            }
    }
    /// Telemetry is not selector content; updating it must not rebuild menu text.
    pub fn record_rule_feedback(
        &mut self,
        feedback: crate::reaction_rules::Feedback,
        last: Option<u8>,
    ) {
        self.rule_feedback = feedback;
        self.rule_last_priority = last;
    }
    fn view_changed(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
    fn changed(&mut self) {
        self.content_revision = self.content_revision.wrapping_add(1);
        self.held_binding = None;
        self.preview_slot = None;
        self.revision = self.revision.wrapping_add(1);
        self.cancel_pending = true;
        // Binding changes require a Y release before the new mode can run.
    }
    pub fn reset_inputs(&mut self) {
        self.restore_mode_choices();
        self.tuning_page = false;
        self.tuning_save_requested = false;
        if let Some(e) = self.editor.as_mut() {
            e.close_commands();
        }
        if let Some(catalog) = self.catalog.as_mut() {
            catalog.close();
        }
        self.save_requested = false;
        self.delete_requested = false;
        self.preview_slot = None;
        self.held_binding = None;
        self.button_latch = 0;
        self.recipe_editing = false;
        self.edit_hold_direction = 0;
        self.edit_hold_ticks = 0;
        self.previous = None;
        self.binding_previous = None;
        self.binding_ledge_previous = None;
        self.binding_ready = 0;
        self.binding_input = None;
        self.binding_safety_jump = false;
        self.binding_starts_blocked = false;
        self.pad_quarantine = self
            .binding_layout
            .binding_ownership(DPAD_SIDES | DPAD_UP | DPAD_DOWN | MAPPER_X);
        self.cancel_actions();
    }
    /// Observe real releases while another tool owns input. Do not navigate,
    /// toggle mash, consume buttons or run an assist during passthrough.
    /// Remember held directions too, so resuming cannot synthesize a menu tap.
    pub fn observe_passthrough(&mut self, buttons: u32) {
        self.observe_physical_passthrough(
            self.binding_layout.physical_buttons(buttons),
            self.binding_layout,
        );
    }
    pub fn observe_physical_passthrough(
        &mut self,
        physical: u32,
        layout: crate::native_input::ControlLayout,
    ) {
        let buttons = layout.policy_buttons(physical);
        self.binding_layout = layout;
        self.binding_physical = physical;
        self.binding_previous = Some(physical);
        self.binding_ledge_previous = Some(self.native_buttons(buttons) & MAPPER_L != 0);
        self.binding_ready |= !physical & bindings::BUTTON_MASK;
        self.button_latch &= layout.binding_ownership(physical);
        self.held_binding = None;
        self.preview_slot = None;
        self.edit_hold_direction = 0;
        self.edit_hold_ticks = 0;
        self.previous = Some(buttons);
        self.pad_quarantine &= self.binding_layout.binding_ownership(self.binding_physical);
        self.trigger_ready |= !buttons & (MAPPER_L | bindings::BUTTON_MASK);
    }
    /// Keep real edge history while a visible editor waits for its save worker.
    /// Do not accumulate invisible edits or replay a held Save on completion.
    /// A Training worker must not keep its editor/layout alive in another mode
    /// or skip first-frame scene defaults at the next battle entry.
    pub fn waits_for_training_save(&self, training: bool, busy: bool) -> bool {
        training && busy && !self.lightweight && !self.match_visibility_pending && !self.menu_hidden
    }
    pub fn wait_for_save(&mut self, buttons: u32) -> u32 {
        self.observe_passthrough(buttons);
        self.cancel_pending = false;
        self.toggle_mash = false;
        self.save_requested = false;
        self.delete_requested = false;
        self.default_requested = false;
        self.tuning_save_requested = false;
        self.button_latch &= buttons;
        buttons & !crate::native_input::MAPPER_PLUS
    }
    /// A momentary Training save/load may resume held-L practice, never Y.
    pub fn resume_training_ledge_hold(&mut self) {
        self.trigger_ready |= MAPPER_L;
    }
    /// Combat cancellation must not erase menu press history during entry,
    /// death or respawn: saved choices remain editable in those phases.
    pub fn cancel_actions(&mut self) {
        self.held_binding = None;
        self.trigger_ready = 0;
        self.cancel_pending = true;
    }
    pub fn show(&mut self) {
        self.tuning_page = false;
        self.menu_hidden = false;
        self.quick_menu = false;
        self.modes_menu = false;
        if self.row.slot().is_none() {
            self.row = MenuRow::Neutral;
        }
        self.changed();
    }
    pub fn show_modes(&mut self) {
        self.show();
        self.modes_menu = true;
        self.recipe_editing = false;
        self.row = MenuRow::RollRules;
    }
    pub fn show_quick(&mut self) {
        if self.mode_editor_origin.is_some() && !self.mode_rule_list {
            self.recipe_editing = false;
            self.mode_rule_list = true;
            if let Some(e) = self.editor.as_mut() {
                e.close_commands();
            }
            if let Some(c) = self.catalog.as_mut() {
                c.close();
            }
            self.changed();
            return;
        }
        if let Some(row) = self.restore_mode_choices() {
            if let Some(e) = self.editor.as_mut() {
                e.close_commands();
            }
            if let Some(c) = self.catalog.as_mut() {
                c.close();
            }
            self.row = row;
            self.menu_hidden = false;
            self.modes_menu = true;
            self.quick_menu = false;
            self.recipe_editing = false;
            self.tuning_page = false;
            self.changed();
            return;
        }
        self.tuning_page = false;
        if let Some(e) = self.editor.as_mut() {
            e.close_commands();
        }
        self.button_menu = self.row.slot().is_some_and(|s| s.is_button());
        self.menu_hidden = false;
        self.quick_menu = true;
        self.modes_menu = false;
        self.recipe_editing = false;
        if !Self::QUICK_ROWS.contains(&self.row) {
            self.row = MenuRow::Neutral;
        }
        #[cfg(feature = "diagnostic-logging")]
        if let Some(catalog) = self.catalog.as_mut() {
            catalog.close();
        }
        self.changed();
    }
    /// Resolve the new battle's visibility on its first scene-aware input.
    /// Lightweight starts visible; Training starts hidden.
    pub fn begin_match(&mut self) {
        self.menu_hidden = true;
        self.match_visibility_pending = true;
        self.button_menu = false;
        self.quick_menu = false;
        self.modes_menu = false;
        self.row = MenuRow::Neutral;
        self.content_revision = self.content_revision.wrapping_add(1);
        self.revision = self.revision.wrapping_add(1);
        self.reset_inputs();
    }
}
fn cycle(index: usize, count: usize, forward: bool) -> usize {
    if count == 0 {
        return 0;
    }
    let index = index % count;
    if forward {
        (index + 1) % count
    } else {
        (index + count - 1) % count
    }
}

#[cfg(test)]
#[path = "../tests/unit/settings_menu/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/unit/settings_menu/maintenance_tests.rs"]
mod maintenance_tests;
