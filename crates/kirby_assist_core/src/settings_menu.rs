//! Shared in-memory menu state for manual recipes, assist switches and bindings.
//!
//! Minus opens ledge home or closes the current page. Actual Training scenes
//! permit editing/testing and explicit save requests; other battle scenes use
//! a three-page selector without editor or persistence controls. Navigation returns
//! a consumed-button mask so menu commands do not leak into gameplay.
//!
//! A selection, edit or mode toggle is session state, not an automatic disk
//! write. The live adapter consumes explicit save requests and owns the worker,
//! backups and catalog publication. Private editor/catalog modules remain private;
//! tests are included from `tests/unit` under their owning modules.
#[cfg(test)]
#[path = "../tests/unit/settings_menu_bindings.rs"]
mod binding_tests;
#[path = "settings_menu/bindings.rs"]
mod bindings;
#[cfg(test)]
#[path = "../tests/unit/settings_menu_buttons.rs"]
mod button_tests;
#[path = "settings_menu/catalog.rs"]
mod catalog;
#[path = "settings_menu/choices.rs"]
mod choices;
#[path = "settings_menu/editor.rs"]
mod editor;
#[path = "settings_menu/input.rs"]
mod input;
#[path = "settings_menu/mode_editor.rs"]
mod mode_editor;
#[path = "settings_menu/panel.rs"]
mod panel;
#[path = "settings_menu/reference.rs"]
mod reference;
use crate::fighter_profiles::presets::{MenuPresets, PresetSlot};
use crate::ledge_trap::DPAD_DOWN;
use crate::manual_controls::{DPAD_LEFT, DPAD_RIGHT, DPAD_SIDES, DPAD_UP};
use crate::native_input::{
    ControlLayout, MAPPER_A, MAPPER_B, MAPPER_DPAD_BUTTONS, MAPPER_L, MAPPER_MINUS, MAPPER_PLUS,
    MAPPER_R, MAPPER_X, MAPPER_Y, MAPPER_ZL, MAPPER_ZR,
};
use crate::settings::AssistSettings;
pub use choices::Choice;

/// `MENU_STICK_CLICK` is a configuration value used by the Smash runtime.
pub const MENU_STICK_CLICK: u32 = 1 << 17; // Physical right-stick click (R3).
/// `MENU_L3` is a configuration value used by the Smash runtime.
pub const MENU_L3: u32 = 1 << 16; // L3 is a menu control only while the menu is visible.
/// `MENU_TOGGLE_CHORD` is a configuration value used by the Smash runtime.
pub const MENU_TOGGLE_CHORD: u32 = MAPPER_MINUS;

/// Ticks a direction must be held before auto-repeat begins.
const MENU_REPEAT_INITIAL_DELAY_TICKS: u16 = 24;
/// Held ticks after which auto-repeat switches to the fast cadence.
const MENU_REPEAT_FAST_MODE_TICKS: u16 = 90;
/// Repeat interval, in ticks, once the fast cadence is active.
const MENU_REPEAT_FAST_INTERVAL_TICKS: u16 = 2;
/// Repeat interval, in ticks, before the fast cadence is reached.
const MENU_REPEAT_SLOW_INTERVAL_TICKS: u16 = 6;

/// Auto-repeat of one held direction: a delay, then a slow and later a fast cadence.
///
/// Shared by every menu surface so the editor, catalog and selector all repeat
/// at the same cadence.
#[derive(Default)]
pub(crate) struct HoldRepeat {
    /// Direction mask currently being held, or zero when released.
    direction: u32,
    /// Consecutive ticks the current direction has been held.
    ticks: u16,
}
impl HoldRepeat {
    /// Restores the state to its initial configuration.
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }
    /// Track the held direction; true when a repeat is due this sample.
    ///
    /// A zero or changed direction restarts the delay and never repeats.
    pub(crate) fn step(&mut self, direction: u32) -> bool {
        if direction == 0 || self.direction != direction {
            *self = Self {
                direction,
                ticks: 0,
            };
            return false;
        }
        self.ticks = self.ticks.saturating_add(1);
        let ticks = self.ticks;
        let interval = if ticks >= MENU_REPEAT_FAST_MODE_TICKS {
            MENU_REPEAT_FAST_INTERVAL_TICKS
        } else {
            MENU_REPEAT_SLOW_INTERVAL_TICKS
        };
        ticks >= MENU_REPEAT_INITIAL_DELAY_TICKS
            && (ticks - MENU_REPEAT_INITIAL_DELAY_TICKS).is_multiple_of(interval)
    }
}

/// Checks whether the monitor-button chord is present in the input sample.
/// Checks whether the monitor buttons are pressed together.
pub fn monitor_chord(buttons: u32) -> bool {
    let mask = MAPPER_L | DPAD_DOWN | MENU_STICK_CLICK;
    buttons & mask == mask
}
/// Checks whether the configured menu-toggle chord was newly pressed.
/// Checks whether the menu-toggle buttons were just pressed.
pub fn menu_toggle_pressed(_held: u32, pressed: u32) -> bool {
    pressed & MAPPER_MINUS != 0
}

/// Only the dedicated L trigger bypasses menu consumption for readiness.
/// Bound D-pad/X/Y inputs must not also reach legacy shortcut observers.
pub fn observation_buttons(physical: u32, consumed: u32) -> u32 {
    physical & !(consumed & !MAPPER_L)
}

/// Available `MenuRow` choices for the Smash assistance menu.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuRow {
    #[default]
    /// `Neutral` option in the `MenuRow` Smash policy state.
    Neutral,
    /// `Jump` option in the `MenuRow` Smash policy state.
    Jump,
    /// `Roll` option in the `MenuRow` Smash policy state.
    Roll,
    /// `Attack` option in the `MenuRow` Smash policy state.
    Attack,
    /// `Drop` option in the `MenuRow` Smash policy state.
    Drop,
    /// `XSequence` option in the `MenuRow` Smash policy state.
    XSequence,
    /// `YSequence` option in the `MenuRow` Smash policy state.
    YSequence,
    /// `DpadUp` option in the `MenuRow` Smash policy state.
    DpadUp,
    /// `DpadDown` option in the `MenuRow` Smash policy state.
    DpadDown,
    /// `DpadLeft` option in the `MenuRow` Smash policy state.
    DpadLeft,
    /// `DpadRight` option in the `MenuRow` Smash policy state.
    DpadRight,
    /// `RollRead` option in the `MenuRow` Smash policy state.
    RollRead,
    /// `AutoMash` option in the `MenuRow` Smash policy state.
    AutoMash,
    /// `IdleCrouch` option in the `MenuRow` Smash policy state.
    IdleCrouch,
    /// `TiltTurn` option in the `MenuRow` Smash policy state.
    TiltTurn,
    /// `SpotPunish` option in the `MenuRow` Smash policy state.
    SpotPunish,
    /// `GrabDodge` option in the `MenuRow` Smash policy state.
    GrabDodge,
    /// `AutoTech` option in the `MenuRow` Smash policy state.
    AutoTech,
    /// `Performance` option in the `MenuRow` Smash policy state.
    Performance,
    /// `AutoLedge` option in the `MenuRow` Smash policy state.
    AutoLedge,
    /// `OutOfShield` option in the `MenuRow` Smash policy state.
    OutOfShield,
    /// `TestLatency` option in the `MenuRow` Smash policy state.
    TestLatency = 26,
    /// `ButtonsPage` option in the `MenuRow` Smash policy state.
    ButtonsPage,
    /// `ModesPage` option in the `MenuRow` Smash policy state.
    ModesPage,
    /// `LedgePage` option in the `MenuRow` Smash policy state.
    LedgePage,
    /// `RollRules` option in the `MenuRow` Smash policy state.
    RollRules,
    /// `FighterSettings` option in the `MenuRow` Smash policy state.
    FighterSettings,
    /// `ShieldPressure` option in the `MenuRow` Smash policy state.
    ShieldPressure,
    /// `L3Sequence` option in the `MenuRow` Smash policy state.
    L3Sequence,
    /// `R3Sequence` option in the `MenuRow` Smash policy state.
    R3Sequence,
    /// `AutoReflect` option in the `MenuRow` Smash policy state.
    AutoReflect,
    /// Configured fighter counter.
    AutoCounter,
    /// Experimental launch DI.
    AutoDi,
    /// Experimental hitstop SDI.
    AutoSdi,
}
impl MenuRow {
    /// `ALL` is a configuration value used by the Smash runtime.
    pub const ALL: [Self; 34] = [
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
        Self::AutoCounter,
        Self::AutoDi,
        Self::AutoSdi,
    ];
    /// Returns the selected recipe slot.
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
            | Self::SpotPunish
            | Self::GrabDodge
            | Self::AutoTech
            | Self::Performance
            | Self::AutoLedge
            | Self::OutOfShield
            | Self::AutoReflect
            | Self::AutoCounter
            | Self::AutoDi
            | Self::AutoSdi
            | Self::TestLatency
            | Self::ButtonsPage
            | Self::ModesPage
            | Self::LedgePage
            | Self::RollRules
            | Self::FighterSettings
            | Self::ShieldPressure => None,
        }
    }
    /// Returns the display label for the value.
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
            Self::SpotPunish => "Spot punish",
            Self::GrabDodge => "Dodge incoming grabs",
            Self::AutoTech => "Auto-tech",
            Self::Performance => "Performance gauge",
            Self::AutoLedge => "Ledge trap mode",
            Self::OutOfShield => "Automatic grabs",
            Self::AutoReflect => "Auto-reflect",
            Self::AutoCounter => "Auto-counter",
            Self::AutoDi => "Auto DI (experimental)",
            Self::AutoSdi => "Auto SDI (experimental)",
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
    /// Runtime header value used by RecipePanel..
    pub header: Vec<String>,
    /// Runtime body value used by RecipePanel..
    pub body: Vec<String>,
    /// Runtime footer value used by RecipePanel..
    pub footer: Vec<String>,
    /// Runtime focus value used by RecipePanel..
    pub focus: Option<usize>,
}
impl RecipePanel {
    /// Converts the presentation model into display lines.
    pub fn into_lines(self) -> Vec<String> {
        self.header
            .into_iter()
            .chain(self.body)
            .chain(self.footer)
            .collect()
    }
}
/// Runtime Smash data carried by `MenuSelection`.
#[derive(Default)]
pub struct MenuSelection {
    /// Runtime binding input value used by MenuSelection..
    binding_input: Option<(u32, ControlLayout)>,
    /// Runtime binding layout value used by MenuSelection..
    binding_layout: ControlLayout,
    /// Runtime binding physical value used by MenuSelection..
    binding_physical: u32,
    /// Runtime binding pressed value used by MenuSelection..
    binding_pressed: u32,
    /// Runtime binding previous value used by MenuSelection..
    binding_previous: Option<u32>,
    /// Runtime binding ledge previous value used by MenuSelection..
    binding_ledge_previous: Option<bool>,
    /// Runtime binding ready value used by MenuSelection..
    binding_ready: u32,
    /// Runtime binding starts blocked value used by MenuSelection..
    binding_starts_blocked: bool,
    /// Runtime binding safety jump value used by MenuSelection..
    binding_safety_jump: bool,
    /// Runtime binding roles stamp value used by MenuSelection..
    binding_roles_stamp: Option<(ControlLayout, [usize; PresetSlot::COUNT], u64, u32, bool)>,
    #[cfg(test)]
    /// Runtime binding roles rebuilds value used by MenuSelection..
    binding_roles_rebuilds: usize,
    /// Runtime single jump sources value used by MenuSelection..
    single_jump_sources: u32,
    /// Runtime ledge assist sources value used by MenuSelection..
    ledge_assist_sources: u32,
    /// Runtime turbo sources value used by MenuSelection..
    turbo_sources: u32,
    /// Runtime trump sources value used by MenuSelection..
    trump_sources: u32,
    l_swap_sources: u32,
    /// False selects ledge trap; true selects spacing for the physical L hold.
    l_spacing_selected: bool,
    counter_sources: u32,
    counter_toggle_sources: u32,
    counter_toggled: bool,
    /// Runtime builtin consumed value used by MenuSelection..
    pub builtin_consumed: u32,
    /// Runtime binding sources value used by MenuSelection..
    binding_sources: u32,
    /// Runtime row value used by MenuSelection..
    pub row: MenuRow,
    /// Stable indices into five ledge lists and two manual-button lists.
    pub choices: [usize; PresetSlot::COUNT],
    /// Runtime roll rules value used by MenuSelection..
    pub roll_rules: bool,
    /// Native runtime uses the simplified event controller; legacy readers stay loadable.
    pub simplified_reactions: bool,
    /// Runtime shield pressure value used by MenuSelection..
    pub shield_pressure: bool,
    /// Runtime rule toggles value used by MenuSelection..
    pub rule_toggles: u8,
    /// Runtime rule button mask value used by MenuSelection..
    pub rule_button_mask: u32,
    /// Runtime rule binding stamp value used by MenuSelection..
    rule_binding_stamp: Option<u64>,
    /// Runtime rule feedback value used by MenuSelection..
    pub rule_feedback: crate::reaction_rules::Feedback,
    /// Runtime rule last priority value used by MenuSelection..
    pub rule_last_priority: Option<u8>,
    /// Runtime y fighter kind value used by MenuSelection..
    pub y_fighter_kind: i32,
    /// Runtime revision value used by MenuSelection..
    pub revision: u64,
    /// Displayed values/page changed; cursor-only navigation leaves this stable.
    pub content_revision: u64,
    /// Runtime recipe revision value used by MenuSelection..
    pub recipe_revision: u64,
    /// Runtime mode editor origin value used by MenuSelection..
    mode_editor_origin: Option<(MenuRow, [usize; PresetSlot::COUNT])>,
    /// Runtime mode rule list value used by MenuSelection..
    mode_rule_list: bool,
    /// Runtime mode rule cursor value used by MenuSelection..
    mode_rule_cursor: usize,
    /// Runtime auto add event value used by MenuSelection..
    auto_add_event: usize,
    /// Runtime shield-grab recipe configuration state used by tuning visibility.
    auto_grab_configured: u8,
    /// Runtime has reactions value used by MenuSelection..
    pub has_reactions: bool,
    /// Runtime recipe editing value used by MenuSelection..
    pub recipe_editing: bool,
    /// Runtime preview slot value used by MenuSelection..
    pub preview_slot: Option<PresetSlot>,
    // Suppress a bound physical button until release, even after its tape ends.
    /// Runtime button latch value used by MenuSelection..
    button_latch: u32,
    /// Runtime held binding value used by MenuSelection..
    held_binding: Option<PresetSlot>,
    /// Runtime save requested value used by MenuSelection..
    pub save_requested: bool,
    /// Runtime delete requested value used by MenuSelection..
    pub delete_requested: bool,
    /// Runtime default requested value used by MenuSelection..
    pub default_requested: bool,
    /// Match-only three-page selector; Training retains the full editor.
    pub lightweight: bool,
    /// Runtime lightweight initialized value used by MenuSelection..
    lightweight_initialized: bool,
    /// Runtime match visibility pending value used by MenuSelection..
    match_visibility_pending: bool,
    /// Runtime catalog value used by MenuSelection..
    catalog: Option<Box<catalog::Catalog>>,
    /// Runtime save status value used by MenuSelection..
    pub save_status: Option<String>,
    /// Runtime editor value used by MenuSelection..
    editor: Option<Box<editor::RecipeEditor>>,
    /// Runtime hold value used by MenuSelection..
    hold: HoldRepeat,
    /// Minus hides/shows the panel and never starts a gameplay tool.
    pub menu_hidden: bool,
    /// First Minus page: saved recipe selection and Training L3 test, without editing.
    pub quick_menu: bool,
    /// Third page groups existing non-sequence modes and toggles.
    pub modes_menu: bool,
    /// Selected Training category: ledge, buttons, reactions, assists, or fighter.
    pub training_category: u8,
    /// Category layout is explicitly active; legacy menu routes remain distinct.
    pub training_categories_active: bool,
    /// Runtime button menu value used by MenuSelection..
    pub button_menu: bool,
    /// Runtime auto ledge value used by MenuSelection..
    pub auto_ledge: bool,
    /// Runtime tuning value used by MenuSelection..
    pub tuning: crate::fighter_tuning::Tuning,
    /// Runtime tuning values value used by MenuSelection..
    pub tuning_values: crate::fighter_tuning::Values,
    /// Runtime tuning page value used by MenuSelection..
    pub tuning_page: bool,
    /// Selected group of fighter settings; independent of the main category tabs.
    pub tuning_group: usize,
    tuning_group_direction: u32,
    /// Runtime tuning row value used by MenuSelection..
    pub tuning_row: usize,
    /// Runtime tuning save requested value used by MenuSelection..
    pub tuning_save_requested: bool,
    pub lan_toggle_requested: bool,
    pub lan_enabled: bool,
    pub lan_status: Option<String>,
    pub telemetry_toggle_requested: bool,
    pub telemetry_enabled: bool,
    pub telemetry_status: Option<String>,
    pub saved_tuning: crate::fighter_tuning::Tuning,
    /// Training category tabs have focus instead of the selected page.
    pub training_tab_focus: bool,

    /// Runtime out of shield value used by MenuSelection..
    pub out_of_shield: bool,
    /// One input-sample event. The adapter owns the shared mash toggle so it
    /// survives fighter/stock changes rather than being a fighter preset.
    pub toggle_mash: bool,
    /// Runtime idle crouch value used by MenuSelection..
    pub idle_crouch: bool,
    /// Runtime tilt turn value used by MenuSelection..
    pub tilt_turn: bool,
    /// Runtime spot punish value used by MenuSelection..
    pub spot_punish: bool,
    /// Runtime grab dodge value used by MenuSelection..
    pub grab_dodge: bool,
    /// Runtime auto tech value used by MenuSelection..
    pub auto_tech: bool,
    /// Runtime auto reflect value used by MenuSelection..
    pub auto_reflect: bool,
    /// Runtime auto-counter toggle.
    pub auto_counter: bool,
    /// Runtime performance value used by MenuSelection..
    pub performance: bool,
    /// Diagnostic annotation only: 0=unset, 1..=31 label external delay 0..=30f.
    pub test_latency: u8,
    /// Runtime previous value used by MenuSelection..
    previous: Option<u32>,
    // Menu-held directions must be released before becoming gameplay triggers.
    /// Runtime pad quarantine value used by MenuSelection..
    pad_quarantine: u32,
    // A held button must be released after reset/config changes before it can
    // control an assist. Unowned gameplay input remains physical.
    /// Runtime trigger ready value used by MenuSelection..
    trigger_ready: u32,
    /// Runtime cancel pending value used by MenuSelection..
    pub cancel_pending: bool,
}
impl MenuSelection {
    /// Estimated owned payload, not allocator bookkeeping or game memory.
    /// Diagnostic adapter calls only after revisions, never scans on idle polls.
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
    /// `QUICK_ROWS` is a configuration value used by the Smash runtime.
    pub const QUICK_ROWS: [MenuRow; PresetSlot::COUNT] = [
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
    /// `LEDGE_ROWS` is a configuration value used by the Smash runtime.
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
    /// `BUTTON_ROWS` is a configuration value used by the Smash runtime.
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
    /// Rows shown in Training's ledge category.
    pub const TRAINING_LEDGE_ROWS: [MenuRow; 5] = [
        MenuRow::Neutral,
        MenuRow::Jump,
        MenuRow::Roll,
        MenuRow::Attack,
        MenuRow::Drop,
    ];
    /// Rows shown in Training's button category.
    pub const TRAINING_BUTTON_ROWS: [MenuRow; 8] = [
        MenuRow::XSequence,
        MenuRow::YSequence,
        MenuRow::DpadUp,
        MenuRow::DpadDown,
        MenuRow::DpadLeft,
        MenuRow::DpadRight,
        MenuRow::L3Sequence,
        MenuRow::R3Sequence,
    ];
    /// Rows shown in Training's reaction category.
    pub const REACTION_ROWS: [MenuRow; 6] = [
        MenuRow::RollRules,
        MenuRow::ShieldPressure,
        MenuRow::SpotPunish,
        MenuRow::OutOfShield,
        MenuRow::GrabDodge,
        MenuRow::AutoCounter,
    ];
    /// Rows shown in Training's assist-toggle and advanced category.
    pub const TOGGLE_ROWS: [MenuRow; 11] = [
        MenuRow::AutoReflect,
        MenuRow::AutoLedge,
        MenuRow::AutoTech,
        MenuRow::AutoDi,
        MenuRow::AutoSdi,
        MenuRow::AutoMash,
        MenuRow::TiltTurn,
        MenuRow::IdleCrouch,
        MenuRow::RollRead,
        MenuRow::Performance,
        MenuRow::TestLatency,
    ];
    /// Category names shown by the Training menu's tab strip.
    pub const TRAINING_CATEGORIES: [&'static str; 5] =
        ["LEDGE", "BUTTONS", "REACTIONS", "ASSISTS", "FIGHTER"];
    /// Numeric settings available on the final Training category.
    pub const FIGHTER_ROWS: [MenuRow; 1] = [MenuRow::FighterSettings];
    /// `ASSIST_ROWS` is a configuration value used by the Smash runtime.
    pub const ASSIST_ROWS: [MenuRow; 21] = [
        MenuRow::RollRules,
        MenuRow::ShieldPressure,
        MenuRow::SpotPunish,
        MenuRow::OutOfShield,
        MenuRow::GrabDodge,
        MenuRow::AutoReflect,
        MenuRow::AutoCounter,
        MenuRow::AutoLedge,
        MenuRow::AutoTech,
        MenuRow::AutoDi,
        MenuRow::AutoSdi,
        MenuRow::AutoMash,
        MenuRow::TiltTurn,
        MenuRow::IdleCrouch,
        MenuRow::RollRead,
        MenuRow::Performance,
        MenuRow::TestLatency,
        MenuRow::LedgePage,
        MenuRow::ButtonsPage,
        MenuRow::FighterSettings,
        MenuRow::ModesPage,
    ];
    // The fixed ledge modifier owns lightweight navigation. Custom D-pad
    // bindings (including a D-pad assigned to ledge assist) remain gameplay.
    /// Processes navigation input for the lightweight settings view.
    fn lightweight_navigation(&self) -> bool {
        self.binding_physical & self.binding_layout.physical_buttons(MAPPER_L) != 0
    }
    /// Moves the selection among rows in the current page.
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
    /// Builds the rows visible in the current settings page.
    pub fn display_rows(&self) -> &'static [MenuRow] {
        if self.lightweight {
            if self.modes_menu {
                &Self::ASSIST_ROWS
            } else if self.button_menu {
                &Self::BUTTON_ROWS
            } else {
                &Self::LEDGE_ROWS
            }
        } else if self.training_categories_active && self.training_tab_focus {
            &[]
        } else if self.training_categories_active && self.training_category == 4 && self.tuning_page
        {
            &Self::FIGHTER_ROWS
        } else if !self.training_categories_active || self.training_category == u8::MAX {
            if self.modes_menu {
                &Self::ASSIST_ROWS
            } else {
                &MenuRow::ALL
            }
        } else if self.tuning_page {
            &Self::FIGHTER_ROWS
        } else if self.managing() || self.recipe_editing {
            &MenuRow::ALL
        } else {
            match self.training_category {
                1 => &Self::TRAINING_BUTTON_ROWS,
                2 => &Self::REACTION_ROWS,
                3 => &Self::TOGGLE_ROWS,
                _ => &Self::TRAINING_LEDGE_ROWS,
            }
        }
    }
    /// Selects one of Training's five navigation categories.
    pub fn set_training_category(&mut self, category: u8) {
        self.training_categories_active = true;
        self.training_category = category % Self::TRAINING_CATEGORIES.len() as u8;
        self.button_menu = self.training_category == 1;
        self.modes_menu = self.training_category == 2 || self.training_category == 3;
        self.quick_menu = false;
        self.recipe_editing = false;
        self.tuning_page = self.training_category == 4;
        self.training_tab_focus = false;
        self.hold.reset();
        self.row = match self.training_category {
            1 => MenuRow::XSequence,
            2 => MenuRow::RollRules,
            3 => MenuRow::AutoReflect,
            4 => MenuRow::FighterSettings,
            _ => MenuRow::Neutral,
        };
        if self.tuning_page {
            self.tuning_group = 0;
            self.tuning_row = self.tuning_visible_index(0);
        } else if let Some(row) = self
            .display_rows()
            .iter()
            .copied()
            .find(|row| self.row_visible(*row))
        {
            self.row = row;
        }
        self.changed();
    }
    /// Builds the navigation link for a settings page.
    pub fn page_link(row: MenuRow) -> bool {
        matches!(
            row,
            MenuRow::LedgePage
                | MenuRow::ButtonsPage
                | MenuRow::ModesPage
                | MenuRow::FighterSettings
        )
    }
    /// Opens the requested settings page.
    fn open_page(&mut self, row: MenuRow) {
        self.hold.reset();
        self.menu_hidden = false;
        self.tuning_page = row == MenuRow::FighterSettings;
        self.recipe_editing = self.tuning_page;
        self.modes_menu = row == MenuRow::ModesPage;
        self.button_menu = row == MenuRow::ButtonsPage;
        self.training_category = u8::MAX;
        self.training_categories_active = false;
        self.training_tab_focus = false;
        self.quick_menu = !self.modes_menu || self.lightweight;
        self.row = match row {
            MenuRow::ButtonsPage => MenuRow::XSequence,
            MenuRow::ModesPage => MenuRow::RollRules,
            MenuRow::FighterSettings => MenuRow::FighterSettings,
            _ => MenuRow::Neutral,
        };
        self.close_catalog();
        if self.lightweight && self.preview_slot.is_none() && self.held_binding.is_none() {
            self.content_changed();
        } else {
            self.changed();
        }
    }
    /// Checks whether the row is visible in the current viewport.
    pub fn row_visible(&self, row: MenuRow) -> bool {
        if self.tuning_page || self.training_tab_focus {
            return false;
        }
        if self.training_categories_active && !self.managing() && !self.recipe_editing {
            return self.display_rows().contains(&row) && self.row_available(row);
        }
        if Self::page_link(row) {
            return !self.lightweight && self.page_link_visible(row);
        }
        self.row_on_page(row) && self.row_available(row)
    }
    /// Checks whether the page link is visible in the current viewport.
    fn page_link_visible(&self, row: MenuRow) -> bool {
        match row {
            MenuRow::FighterSettings => true,
            MenuRow::LedgePage => self.button_menu || self.modes_menu,
            MenuRow::ButtonsPage => !self.button_menu,
            MenuRow::ModesPage => !self.modes_menu,
            _ => false,
        }
    }
    /// Checks whether a row belongs to the requested page.
    fn row_on_page(&self, row: MenuRow) -> bool {
        let slot = row.slot();
        if self.lightweight && self.button_menu {
            slot.is_some_and(|s| s.is_button())
        } else if self.modes_menu {
            slot.is_none()
        } else {
            slot.is_some_and(|s| s.is_button() == self.button_menu)
        }
    }
    /// Rows hidden by the build, the fighter or the reaction mode.
    fn row_available(&self, row: MenuRow) -> bool {
        match row {
            MenuRow::Performance | MenuRow::TestLatency => !self.lightweight,
            MenuRow::RollRead | MenuRow::AutoCounter => false,
            MenuRow::ShieldPressure | MenuRow::SpotPunish => !self.simplified_reactions,
            MenuRow::IdleCrouch => self.y_fighter_kind == crate::fighter_kinds::KIRBY,
            MenuRow::TiltTurn => {
                crate::button_bindings::Mode::TiltTurn.supports(self.y_fighter_kind)
            }
            MenuRow::AutoReflect => {
                crate::projectile_reflect::Config::for_fighter(self.y_fighter_kind, self.tuning)
                    .is_some()
            }
            _ => true,
        }
    }
    /// Constructs a new value with the type’s initial state.
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
            auto_tech: false,
            grab_dodge: true,
            out_of_shield: true,
            menu_hidden: true,
            training_category: u8::MAX,
            ..Self::default()
        }
    }
    /// The on/off setting a mode row toggles; Auto-mash is an event, not a flag.
    fn mode_flag(&mut self, row: MenuRow) -> Option<&mut bool> {
        Some(match row {
            MenuRow::ShieldPressure => &mut self.shield_pressure,
            MenuRow::RollRules => &mut self.roll_rules,
            MenuRow::AutoTech => &mut self.auto_tech,
            MenuRow::AutoReflect => &mut self.auto_reflect,
            MenuRow::AutoCounter => &mut self.auto_counter,
            MenuRow::AutoLedge => &mut self.auto_ledge,
            MenuRow::Performance => &mut self.performance,
            MenuRow::IdleCrouch => &mut self.idle_crouch,
            MenuRow::TiltTurn => &mut self.tilt_turn,
            MenuRow::OutOfShield => &mut self.out_of_shield,
            MenuRow::SpotPunish => &mut self.spot_punish,
            MenuRow::GrabDodge => &mut self.grab_dodge,
            _ => return None,
        })
    }
    /// Uses the same persisted preference for both menu rendering and live input.
    pub fn damage_sticks(&self) -> [bool; 2] {
        [
            crate::fighter_tuning::AUTO_DI_DEFAULT,
            crate::fighter_tuning::AUTO_SDI_DEFAULT,
        ]
        .map(|i| self.tuning.0[i].unwrap_or(self.tuning_values[i]) == 1.0)
    }
    /// Applies the selected adjustment mode to the value.
    fn adjust_mode(&mut self, forward: bool) {
        match self.row {
            MenuRow::AutoDi | MenuRow::AutoSdi => {
                let i = if self.row == MenuRow::AutoDi {
                    crate::fighter_tuning::AUTO_DI_DEFAULT
                } else {
                    crate::fighter_tuning::AUTO_SDI_DEFAULT
                };
                let enabled = self.tuning.0[i].unwrap_or(self.tuning_values[i]) == 1.0;
                self.tuning.0[i] = Some(f32::from(!enabled));
                self.recipes_edited("Unsaved fighter settings — Fighter settings / Y: Save");
            }
            MenuRow::AutoMash => self.toggle_mash = true,
            MenuRow::TestLatency => {
                self.test_latency = cycle(self.test_latency as usize, 32, forward) as u8;
            }
            row => {
                if let Some(flag) = self.mode_flag(row) {
                    *flag = !*flag;
                }
            }
        }
    }
    /// Ledge autopilot stays OFF unless separately enabled by the user.
    pub fn sequence_enabled(&self, base: bool) -> bool {
        base || self.preview_slot.is_some() || self.ledge_mode()
    }
    /// Returns the current settings view.
    pub fn settings(&self, presets: &MenuPresets, base: AssistSettings) -> AssistSettings {
        // Cache selected file timelines. There is no fighter-function fallback.
        let mut s = self.tuning.apply(base);
        s.enabled = self.sequence_enabled(base.enabled);
        s.sequence_overrides = [None; PresetSlot::COUNT];
        s.response_choices = [None; PresetSlot::COUNT];
        for slot in PresetSlot::ALL {
            let index = self.choices[slot as usize];
            let Some(source) = self.source(presets, slot, index) else {
                continue;
            };
            let mut sequence = self
                .editor
                .as_ref()
                .and_then(|e| e.compiled((slot as usize, index)))
                .unwrap_or_else(|| source.sequence());
            if sequence.reaction.active() && self.preview_slot != Some(slot) {
                continue;
            }
            if !slot.is_button() {
                sequence.while_held = false;
            }
            if sequence.start_delay == 0 && choices::is_blank(&sequence) {
                continue;
            }
            s.sequence_overrides[slot as usize] = Some(sequence);
            s.response_choices[slot as usize] = Some(index);
        }
        s
    }
    /// Checks whether the tuning section is visible.
    fn tuning_visible(&self, i: usize) -> bool {
        if i >= crate::fighter_tuning::COUNT {
            return i < crate::fighter_tuning::COUNT + 2 && !self.lightweight;
        }
        crate::fighter_tuning::Tuning::visible(self.y_fighter_kind, i)
            && i != crate::fighter_tuning::AUTO_GRAB_FAR
            && !(i == crate::fighter_tuning::AUTO_GRAB_CLOSE && self.auto_grab_configured != 0)
    }
    pub const TUNING_GROUPS: [&'static str; 12] = [
        "Movement & facing",
        "Ledge positioning",
        "Grabs & avoidance",
        "Shield defense",
        "Tech & damage escape",
        "Projectile reflection",
        "Counters",
        "Auto-spacing movement",
        "Auto-spacing ground attacks",
        "Auto-spacing aerial attacks",
        "Fighter-specific",
        "Developer tools",
    ];

    fn tuning_row_group(i: usize) -> usize {
        match i {
            0 | 8 | 51..=53 => 0,
            2..=7 | 9..=10 => 1,
            1 | 11..=13 | 21..=22 | 42..=43 => 2,
            14..=20 | 40..=41 => 3,
            23..=30 | 73..=74 => 4,
            44..=50 => 5,
            54..=59 | 82 => 6,
            60 | 75..=81 | 83..=84 => 7,
            61..=66 => 8,
            67..=72 => 9,
            31..=39 => 10,
            _ => 11,
        }
    }

    fn tuning_group_visible(&self, i: usize) -> bool {
        self.tuning_visible(i) && Self::tuning_row_group(i) == self.tuning_group
    }

    fn change_tuning_group(&mut self, forward: bool) {
        for _ in 0..Self::TUNING_GROUPS.len() {
            self.tuning_group = cycle(self.tuning_group, Self::TUNING_GROUPS.len(), forward);
            if (0..crate::fighter_tuning::COUNT + 2).any(|i| self.tuning_group_visible(i)) {
                break;
            }
        }
        self.tuning_row = self.tuning_visible_index(0);
        self.hold.reset();
        self.view_changed();
    }

    /// The first visible fighter-setting row at or after `row`, wrapping to the first one.
    fn tuning_visible_index(&self, row: usize) -> usize {
        (row..crate::fighter_tuning::COUNT + 2)
            .find(|i| self.tuning_group_visible(*i))
            .or_else(|| (0..crate::fighter_tuning::COUNT + 2).find(|i| self.tuning_group_visible(*i)))
            .unwrap_or(0)
    }
    /// Returns the current recipe-input validation error, if any.
    fn recipe_input_error(&self, presets: &MenuPresets) -> Option<String> {
        let slot = self.row.slot()?;
        let key = (slot as usize, self.choices[slot as usize]);
        let sequence = if let Some(editor) = self.editor.as_ref().filter(|e| e.has(key)) {
            match editor.compiled(key) {
                Some(s) => s,
                None => return Some("Too many expanded steps; remove a step or delay".into()),
            }
        } else {
            self.source(presets, slot, key.1)?.sequence()
        };
        if sequence.builtin.is_some() || sequence.mode.is_some() {
            return None;
        }
        if sequence.len == 0 {
            return Some("Add an action before testing or saving".into());
        }
        crate::reaction_rules::validate_response(sequence).err()
    }
    /// Checks whether the recipe differs from its saved value.
    pub fn recipe_modified(&self, slot: PresetSlot) -> bool {
        let index = self.choices[slot as usize];
        self.editor
            .as_ref()
            .is_some_and(|e| e.edited((slot as usize, index)))
            || self.draft(slot, index).is_some_and(|d| d.dirty)
    }
    pub fn lan_has_unsaved_edits(&self) -> bool {
        self.tuning != self.saved_tuning
            || self.catalog.as_ref()
                .is_some_and(|c| c.drafts.values().any(|d| d.dirty))
            || self.editor.as_ref().is_some_and(|e| e.has_unsaved_edits())
    }
    pub fn lan_editing_allowed(&self) -> bool {
        !self.menu_hidden
            && !self.lightweight
            && !self.lan_has_unsaved_edits()
            && !self.managing()
            && !self.save_requested
            && !self.tuning_save_requested
            && !self.delete_requested
            && !self.default_requested
    }
    /// Checks whether ledge assistance is currently held.
    pub fn ledge_held(&self, buttons: u32) -> bool {
        // Training navigation is still a UI command even when that physical
        // direction is assigned to a gameplay helper and filtered from policy.
        if !self.menu_hidden && !self.lightweight && buttons & MAPPER_DPAD_BUTTONS != 0 {
            return false;
        }
        let buttons = self.native_buttons(buttons);
        let blocked = if self.lightweight {
            // Minus is always a selector control; visible-menu D-pad is too.
            (if self.menu_hidden {
                MAPPER_DPAD_BUTTONS
            } else {
                0
            }) | MAPPER_Y
        } else {
            MAPPER_DPAD_BUTTONS | MAPPER_MINUS | MAPPER_Y
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
        self.defensive_owner().is_some()
            || self.auto_spacing_held()
            || self.ledge_mode()
            || self.preview_slot.is_some()
            || self.native_buttons(self.binding_layout.policy_buttons(self.binding_physical))
                & MAPPER_L
                != 0
    }
    /// Checks whether autopilot assistance is currently held.
    pub fn autopilot_held(&self, buttons: u32) -> bool {
        self.ledge_mode()
            && !self.recipe_editing
            && !self.managing()
            && !self.cancel_pending
            && (buttons & MAPPER_DPAD_BUTTONS == 0
                || self.lightweight && !self.menu_hidden && self.lightweight_navigation())
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

    /// Cursor/page moved; displayed values did not.
    fn view_changed(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
    /// Displayed values or the page changed.
    fn content_changed(&mut self) {
        self.content_revision = self.content_revision.wrapping_add(1);
        self.view_changed();
    }
    /// A menu change cancels any running test or held binding.
    fn changed(&mut self) {
        self.content_changed();
        self.held_binding = None;
        self.preview_slot = None;
        self.cancel_pending = true;
        // Binding changes require a Y release before the new mode can run.
    }
    /// Recipe data changed: caches keyed on `recipe_revision` must rebuild.
    fn bump_recipes(&mut self) {
        self.recipe_revision = self.recipe_revision.wrapping_add(1);
    }
    /// Checks whether any recipe edits are pending.
    fn recipes_edited(&mut self, status: &str) {
        self.bump_recipes();
        self.save_status = Some(status.into());
    }
    /// Closes the recipe catalog and returns to the previous view.
    fn close_catalog(&mut self) {
        if let Some(catalog) = self.catalog.as_mut() {
            catalog.close();
        }
    }
    /// Closes the active editor command state.
    fn close_editor_commands(&mut self) {
        if let Some(editor) = self.editor.as_mut() {
            editor.close_commands();
        }
    }
    /// Closes the active tools view.
    fn close_tools(&mut self) {
        self.close_editor_commands();
        self.close_catalog();
    }
    /// One-shot save/delete/default requests live for a single input sample.
    fn clear_requests(&mut self) {
        self.save_requested = false;
        self.delete_requested = false;
        self.default_requested = false;
        self.tuning_save_requested = false;
        self.lan_toggle_requested = false;
        self.telemetry_toggle_requested = false;
    }

    /// Resets the inputs.
    pub fn reset_inputs(&mut self) {
        self.l_spacing_selected = false;
        self.binding_roles_stamp = None;
        self.counter_toggled = false;
        self.restore_mode_choices();
        self.tuning_page = false;
        self.close_tools();
        self.clear_requests();
        self.preview_slot = None;
        self.held_binding = None;
        self.button_latch = 0;
        self.recipe_editing = false;
        self.hold.reset();
        self.previous = None;
        self.binding_previous = None;
        self.binding_ledge_previous = None;
        self.binding_ready = 0;
        self.binding_input = None;
        self.binding_safety_jump = false;
        self.binding_starts_blocked = false;
        self.pad_quarantine = self
            .binding_layout
            .binding_ownership(MAPPER_DPAD_BUTTONS | MAPPER_X);
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
    /// Observes the physical passthrough.
    pub fn observe_physical_passthrough(&mut self, physical: u32, layout: ControlLayout) {
        let buttons = layout.policy_buttons(physical);
        self.binding_layout = layout;
        self.binding_physical = physical;
        self.binding_previous = Some(physical);
        self.binding_ledge_previous = Some(self.native_buttons(buttons) & MAPPER_L != 0);
        self.binding_ready |= !physical & bindings::BUTTON_MASK;
        self.button_latch &= layout.binding_ownership(physical);
        self.held_binding = None;
        self.preview_slot = None;
        self.hold.reset();
        self.previous = Some(buttons);
        self.pad_quarantine &= layout.binding_ownership(physical);
        self.trigger_ready |= !buttons & (MAPPER_L | bindings::BUTTON_MASK);
    }
    /// Keep real edge history while a visible editor waits for its save worker.
    /// Do not accumulate invisible edits or replay a held Save on completion.
    /// A Training worker must not keep its editor/layout alive in another mode
    /// or skip first-frame scene defaults at the next battle entry.
    pub fn waits_for_training_save(&self, training: bool, busy: bool) -> bool {
        training && busy && !self.lightweight && !self.match_visibility_pending && !self.menu_hidden
    }
    /// Waits for the pending save operation to complete.
    pub fn wait_for_save(&mut self, buttons: u32) -> u32 {
        self.observe_passthrough(buttons);
        self.cancel_pending = false;
        self.toggle_mash = false;
        self.clear_requests();
        self.button_latch &= buttons;
        buttons & !MAPPER_PLUS
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
    /// Builds the requested display representation.
    pub fn show(&mut self) {
        self.training_category = u8::MAX;
        self.training_categories_active = false;
        self.training_tab_focus = false;
        self.tuning_page = false;
        self.menu_hidden = false;
        self.quick_menu = false;
        self.modes_menu = false;
        if self.row.slot().is_none() {
            self.row = MenuRow::Neutral;
        }
        self.changed();
    }
    /// Builds the display for the available assistance modes.
    pub fn show_modes(&mut self) {
        self.show();
        self.training_category = u8::MAX;
        self.training_categories_active = false;
        self.modes_menu = true;
        self.recipe_editing = false;
        self.row = MenuRow::RollRules;
    }
    /// Builds the quick-settings display.
    pub fn show_quick(&mut self) {
        if self.mode_editor_origin.is_some() {
            self.training_category = u8::MAX;
            self.training_categories_active = false;
        }
        self.training_tab_focus = false;
        if self.mode_editor_origin.is_some() && !self.mode_rule_list {
            self.recipe_editing = false;
            self.mode_rule_list = true;
            self.close_tools();
            self.changed();
            return;
        }
        if let Some(row) = self.restore_mode_choices() {
            self.close_tools();
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
        self.close_tools();
        self.button_menu = self.row.slot().is_some_and(|s| s.is_button());
        self.menu_hidden = false;
        self.quick_menu = true;
        self.modes_menu = false;
        self.training_tab_focus = false;
        self.recipe_editing = false;
        if !Self::QUICK_ROWS.contains(&self.row) {
            self.row = MenuRow::Neutral;
        }
        self.changed();
    }
    /// Resolve the new battle's visibility on its first scene-aware input.
    /// Lightweight starts visible; Training starts hidden.
    pub fn begin_match(&mut self) {
        self.training_category = u8::MAX;
        self.training_categories_active = false;
        self.training_tab_focus = false;
        self.menu_hidden = true;
        self.match_visibility_pending = true;
        self.button_menu = false;
        self.quick_menu = false;
        self.modes_menu = false;
        self.row = MenuRow::Neutral;
        self.content_changed();
        self.reset_inputs();
    }
}
/// Advances the selected choice to the next value.
/// Moves to the next choice.
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
#[path = "../tests/unit/settings_menu_auto_spacing.rs"]
mod auto_spacing_tests;

#[cfg(test)]
#[path = "../tests/unit/settings_menu.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/unit/settings_menu_lan.rs"]
mod lan_tests;

#[cfg(test)]
#[path = "../tests/unit/settings_menu_maintenance.rs"]
mod maintenance_tests;
