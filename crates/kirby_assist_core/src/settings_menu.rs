//! Shared settings menu state for fighter recipes, feature modes and input bindings.
//! Minus toggles the panel; ledge choices and other settings are saved per fighter.
#[cfg(test)]
#[path = "../tests/unit/settings_menu_bindings.rs"]
mod binding_tests;
#[cfg(test)]
#[path = "../tests/unit/settings_menu_buttons.rs"]
mod button_tests;
#[path = "settings_menu/bindings.rs"]
mod bindings;
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

/// Technical: Defines the `MENU_STICK_CLICK` constant used by this module.
///
/// Plain English: Provides the MENU STICK CLICK constant value.
pub const MENU_STICK_CLICK: u32 = 1 << 17; // Physical right-stick click (R3).
/// Technical: Defines the `MENU_L3` constant used by this module.
///
/// Plain English: Provides the MENU L3 constant value.
pub const MENU_L3: u32 = 1 << 16; // L3 is a menu control only while the menu is visible.
/// Technical: Defines the `MENU_TOGGLE_CHORD` constant used by this module.
///
/// Plain English: Provides the MENU TOGGLE CHORD constant value.
pub const MENU_TOGGLE_CHORD: u32 = MAPPER_MINUS;

/// Technical: Defines the `MENU_REPEAT_INITIAL_DELAY_TICKS` constant used by this module.
///
/// Plain English: Provides the MENU REPEAT INITIAL DELAY TICKS constant value.
const MENU_REPEAT_INITIAL_DELAY_TICKS: u16 = 24;
/// Technical: Defines the `MENU_REPEAT_FAST_MODE_TICKS` constant used by this module.
///
/// Plain English: Provides the MENU REPEAT FAST MODE TICKS constant value.
const MENU_REPEAT_FAST_MODE_TICKS: u16 = 90;
/// Technical: Defines the `MENU_REPEAT_FAST_INTERVAL_TICKS` constant used by this module.
///
/// Plain English: Provides the MENU REPEAT FAST INTERVAL TICKS constant value.
const MENU_REPEAT_FAST_INTERVAL_TICKS: u16 = 2;
/// Technical: Defines the `MENU_REPEAT_SLOW_INTERVAL_TICKS` constant used by this module.
///
/// Plain English: Provides the MENU REPEAT SLOW INTERVAL TICKS constant value.
const MENU_REPEAT_SLOW_INTERVAL_TICKS: u16 = 6;

/// Auto-repeat of one held direction: a delay, then a slow and later a fast cadence.
/// Plain English: Auto-repeat of one held direction: a delay, then a slow and later a fast cadence.
#[derive(Default)]
struct HoldRepeat {
    /// Technical: Stores the `direction` value in `HoldRepeat` as `u32`.
    ///
    /// Plain English: Holds the direction value used by HoldRepeat.
    direction: u32,
    /// Technical: Stores the `ticks` value in `HoldRepeat` as `u16`.
    ///
    /// Plain English: Holds the ticks value used by HoldRepeat.
    ticks: u16,
}
impl HoldRepeat {
    /// Technical: Restores the state to its initial configuration.
    ///
    /// Plain English: Returns the state to its starting point.
    fn reset(&mut self) {
        *self = Self::default();
    }
    /// Track the held direction; true when a repeat is due this sample.
    /// Plain English: Track the held direction; true when a repeat is due this sample.
    fn step(&mut self, direction: u32) -> bool {
        if self.direction != direction {
            *self = Self { direction, ticks: 0 };
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
            && (ticks - MENU_REPEAT_INITIAL_DELAY_TICKS) % interval == 0
    }
}

/// Technical: Checks whether the monitor-button chord is present in the input sample.
///
/// Plain English: Checks whether the monitor buttons are pressed together.
pub fn monitor_chord(buttons: u32) -> bool {
    let mask = MAPPER_L | DPAD_DOWN | MENU_STICK_CLICK;
    buttons & mask == mask
}
/// Technical: Checks whether the configured menu-toggle chord was newly pressed.
///
/// Plain English: Checks whether the menu-toggle buttons were just pressed.
pub fn menu_toggle_pressed(_held: u32, pressed: u32) -> bool {
    pressed & MAPPER_MINUS != 0
}

/// Only the dedicated L trigger bypasses menu consumption for readiness.
/// Bound D-pad/X/Y inputs must not also reach legacy shortcut observers.
/// Plain English: Only the dedicated L trigger bypasses menu consumption for readiness. Bound D-pad/X/Y inputs must not also reach legacy shortcut observers.
pub fn observation_buttons(physical: u32, consumed: u32) -> u32 {
    physical & !(consumed & !MAPPER_L)
}

/// Technical: Defines the available `MenuRow` cases.
///
/// Plain English: Lists the available MenuRow choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuRow {
    #[default]
    /// Technical: Represents the `Neutral` case of `MenuRow`.
    ///
    /// Plain English: Represents the Neutral option in MenuRow.
    Neutral,
    /// Technical: Represents the `Jump` case of `MenuRow`.
    ///
    /// Plain English: Represents the Jump option in MenuRow.
    Jump,
    /// Technical: Represents the `Roll` case of `MenuRow`.
    ///
    /// Plain English: Represents the Roll option in MenuRow.
    Roll,
    /// Technical: Represents the `Attack` case of `MenuRow`.
    ///
    /// Plain English: Represents the Attack option in MenuRow.
    Attack,
    /// Technical: Represents the `Drop` case of `MenuRow`.
    ///
    /// Plain English: Represents the Drop option in MenuRow.
    Drop,
    /// Technical: Represents the `XSequence` case of `MenuRow`.
    ///
    /// Plain English: Represents the XSequence option in MenuRow.
    XSequence,
    /// Technical: Represents the `YSequence` case of `MenuRow`.
    ///
    /// Plain English: Represents the YSequence option in MenuRow.
    YSequence,
    /// Technical: Represents the `DpadUp` case of `MenuRow`.
    ///
    /// Plain English: Represents the DpadUp option in MenuRow.
    DpadUp,
    /// Technical: Represents the `DpadDown` case of `MenuRow`.
    ///
    /// Plain English: Represents the DpadDown option in MenuRow.
    DpadDown,
    /// Technical: Represents the `DpadLeft` case of `MenuRow`.
    ///
    /// Plain English: Represents the DpadLeft option in MenuRow.
    DpadLeft,
    /// Technical: Represents the `DpadRight` case of `MenuRow`.
    ///
    /// Plain English: Represents the DpadRight option in MenuRow.
    DpadRight,
    /// Technical: Represents the `RollRead` case of `MenuRow`.
    ///
    /// Plain English: Represents the RollRead option in MenuRow.
    RollRead,
    /// Technical: Represents the `AutoMash` case of `MenuRow`.
    ///
    /// Plain English: Represents the AutoMash option in MenuRow.
    AutoMash,
    /// Technical: Represents the `IdleCrouch` case of `MenuRow`.
    ///
    /// Plain English: Represents the IdleCrouch option in MenuRow.
    IdleCrouch,
    /// Technical: Represents the `TiltTurn` case of `MenuRow`.
    ///
    /// Plain English: Represents the TiltTurn option in MenuRow.
    TiltTurn,
    /// Technical: Represents the `SpotPunish` case of `MenuRow`.
    ///
    /// Plain English: Represents the SpotPunish option in MenuRow.
    SpotPunish,
    /// Technical: Represents the `GrabDodge` case of `MenuRow`.
    ///
    /// Plain English: Represents the GrabDodge option in MenuRow.
    GrabDodge,
    /// Technical: Represents the `AutoTech` case of `MenuRow`.
    ///
    /// Plain English: Represents the AutoTech option in MenuRow.
    AutoTech,
    /// Technical: Represents the `Performance` case of `MenuRow`.
    ///
    /// Plain English: Represents the Performance option in MenuRow.
    Performance,
    /// Technical: Represents the `AutoLedge` case of `MenuRow`.
    ///
    /// Plain English: Represents the AutoLedge option in MenuRow.
    AutoLedge,
    /// Technical: Represents the `OutOfShield` case of `MenuRow`.
    ///
    /// Plain English: Represents the OutOfShield option in MenuRow.
    OutOfShield,
    /// Technical: Represents the `TestLatency` case of `MenuRow`.
    ///
    /// Plain English: Represents the TestLatency option in MenuRow.
    TestLatency = 26,
    /// Technical: Represents the `ButtonsPage` case of `MenuRow`.
    ///
    /// Plain English: Represents the ButtonsPage option in MenuRow.
    ButtonsPage,
    /// Technical: Represents the `ModesPage` case of `MenuRow`.
    ///
    /// Plain English: Represents the ModesPage option in MenuRow.
    ModesPage,
    /// Technical: Represents the `LedgePage` case of `MenuRow`.
    ///
    /// Plain English: Represents the LedgePage option in MenuRow.
    LedgePage,
    /// Technical: Represents the `RollRules` case of `MenuRow`.
    ///
    /// Plain English: Represents the RollRules option in MenuRow.
    RollRules,
    /// Technical: Represents the `FighterSettings` case of `MenuRow`.
    ///
    /// Plain English: Represents the FighterSettings option in MenuRow.
    FighterSettings,
    /// Technical: Represents the `ShieldPressure` case of `MenuRow`.
    ///
    /// Plain English: Represents the ShieldPressure option in MenuRow.
    ShieldPressure,
    /// Technical: Represents the `L3Sequence` case of `MenuRow`.
    ///
    /// Plain English: Represents the L3Sequence option in MenuRow.
    L3Sequence,
    /// Technical: Represents the `R3Sequence` case of `MenuRow`.
    ///
    /// Plain English: Represents the R3Sequence option in MenuRow.
    R3Sequence,
    /// Technical: Represents the `AutoReflect` case of `MenuRow`.
    ///
    /// Plain English: Represents the AutoReflect option in MenuRow.
    AutoReflect,
}
impl MenuRow {
    /// Technical: Defines the `ALL` constant used by this module.
    ///
    /// Plain English: Provides the ALL constant value.
    pub const ALL: [Self; 31] = [
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
    ];
    /// Technical: Returns the selected recipe slot.
    ///
    /// Plain English: Gets the currently selected slot.
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
            | Self::TestLatency
            | Self::ButtonsPage
            | Self::ModesPage
            | Self::LedgePage
            | Self::RollRules
            | Self::FighterSettings
            | Self::ShieldPressure => None,
        }
    }
    /// Technical: Returns the display label for the value.
    ///
    /// Plain English: Gets the text shown for this value.
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
            Self::TestLatency => "Test delay label",
            Self::ButtonsPage => "Button recipes",
            Self::ModesPage => "Assist modes",
            Self::LedgePage => "Ledge traps",
        }
    }
}
/// Cached presentation data; no execution state or native UI dependencies.
/// Plain English: Cached presentation data; no execution state or native UI dependencies.
#[derive(Default)]
pub struct RecipePanel {
    /// Technical: Stores the `header` value in `RecipePanel` as `Vec<String>`.
    ///
    /// Plain English: Holds the header value used by RecipePanel.
    pub header: Vec<String>,
    /// Technical: Stores the `body` value in `RecipePanel` as `Vec<String>`.
    ///
    /// Plain English: Holds the body value used by RecipePanel.
    pub body: Vec<String>,
    /// Technical: Stores the `footer` value in `RecipePanel` as `Vec<String>`.
    ///
    /// Plain English: Holds the footer value used by RecipePanel.
    pub footer: Vec<String>,
    /// Technical: Stores the `focus` value in `RecipePanel` as `Option<usize>`.
    ///
    /// Plain English: Optionally stores the focus for RecipePanel; it may be absent.
    pub focus: Option<usize>,
}
impl RecipePanel {
    /// Technical: Converts the presentation model into display lines.
    ///
    /// Plain English: Turns the presentation into lines of text.
    pub fn into_lines(self) -> Vec<String> {
        self.header
            .into_iter()
            .chain(self.body)
            .chain(self.footer)
            .collect()
    }
}
/// Technical: Defines the `MenuSelection` data structure.
///
/// Plain English: Holds the data for MenuSelection.
#[derive(Default)]
pub struct MenuSelection {
    /// Technical: Stores the `binding_input` value in `MenuSelection` as `Option<(u32`.
    ///
    /// Plain English: Optionally stores the binding input for MenuSelection; it may be absent.
    binding_input: Option<(u32, ControlLayout)>,
    /// Technical: Stores the `binding_layout` value in `MenuSelection` as `ControlLayout`.
    ///
    /// Plain English: Holds the binding layout value used by MenuSelection.
    binding_layout: ControlLayout,
    /// Technical: Stores the `binding_physical` value in `MenuSelection` as `u32`.
    ///
    /// Plain English: Holds the binding physical value used by MenuSelection.
    binding_physical: u32,
    /// Technical: Stores the `binding_pressed` value in `MenuSelection` as `u32`.
    ///
    /// Plain English: Holds the binding pressed value used by MenuSelection.
    binding_pressed: u32,
    /// Technical: Stores the `binding_previous` value in `MenuSelection` as `Option<u32>`.
    ///
    /// Plain English: Optionally stores the binding previous for MenuSelection; it may be absent.
    binding_previous: Option<u32>,
    /// Technical: Stores the `binding_ledge_previous` value in `MenuSelection` as `Option<bool>`.
    ///
    /// Plain English: Optionally stores the binding ledge previous for MenuSelection; it may be absent.
    binding_ledge_previous: Option<bool>,
    /// Technical: Stores the `binding_ready` value in `MenuSelection` as `u32`.
    ///
    /// Plain English: Holds the binding ready value used by MenuSelection.
    binding_ready: u32,
    /// Technical: Stores the `binding_starts_blocked` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the binding starts blocked value used by MenuSelection.
    binding_starts_blocked: bool,
    /// Technical: Stores the `binding_safety_jump` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the binding safety jump value used by MenuSelection.
    binding_safety_jump: bool,
    /// Technical: Stores the `binding_roles_stamp` value in `MenuSelection` as `Option<(ControlLayout`.
    ///
    /// Plain English: Optionally stores the binding roles stamp for MenuSelection; it may be absent.
    binding_roles_stamp: Option<(ControlLayout, [usize; PresetSlot::COUNT], u64, u32, bool)>,
    #[cfg(test)]
    /// Technical: Stores the `binding_roles_rebuilds` value in `MenuSelection` as `usize`.
    ///
    /// Plain English: Holds the binding roles rebuilds value used by MenuSelection.
    binding_roles_rebuilds: usize,
    /// Technical: Stores the `single_jump_sources` value in `MenuSelection` as `u32`.
    ///
    /// Plain English: Holds the single jump sources value used by MenuSelection.
    single_jump_sources: u32,
    /// Technical: Stores the `ledge_assist_sources` value in `MenuSelection` as `u32`.
    ///
    /// Plain English: Holds the ledge assist sources value used by MenuSelection.
    ledge_assist_sources: u32,
    /// Technical: Stores the `turbo_sources` value in `MenuSelection` as `u32`.
    ///
    /// Plain English: Holds the turbo sources value used by MenuSelection.
    turbo_sources: u32,
    /// Technical: Stores the `trump_sources` value in `MenuSelection` as `u32`.
    ///
    /// Plain English: Holds the trump sources value used by MenuSelection.
    trump_sources: u32,
    /// Technical: Stores the `builtin_consumed` value in `MenuSelection` as `u32`.
    ///
    /// Plain English: Holds the builtin consumed value used by MenuSelection.
    pub builtin_consumed: u32,
    /// Technical: Stores the `binding_sources` value in `MenuSelection` as `u32`.
    ///
    /// Plain English: Holds the binding sources value used by MenuSelection.
    binding_sources: u32,
    /// Technical: Stores the `row` value in `MenuSelection` as `MenuRow`.
    ///
    /// Plain English: Holds the row value used by MenuSelection.
    pub row: MenuRow,
    /// Stable indices into five ledge lists and two manual-button lists.
    /// Plain English: Stable indices into five ledge lists and two manual-button lists.
    pub choices: [usize; PresetSlot::COUNT],
    /// Technical: Stores the `roll_rules` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the roll rules value used by MenuSelection.
    pub roll_rules: bool,
    /// Native runtime uses the simplified event controller; legacy readers stay loadable.
    /// Plain English: Native runtime uses the simplified event controller; legacy readers stay loadable.
    pub simplified_reactions: bool,
    /// Technical: Stores the `shield_pressure` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the shield pressure value used by MenuSelection.
    pub shield_pressure: bool,
    /// Technical: Stores the `rule_toggles` value in `MenuSelection` as `u8`.
    ///
    /// Plain English: Holds the rule toggles value used by MenuSelection.
    pub rule_toggles: u8,
    /// Technical: Stores the `rule_button_mask` value in `MenuSelection` as `u32`.
    ///
    /// Plain English: Holds the rule button mask value used by MenuSelection.
    pub rule_button_mask: u32,
    /// Technical: Stores the `rule_binding_stamp` value in `MenuSelection` as `Option<u64>`.
    ///
    /// Plain English: Optionally stores the rule binding stamp for MenuSelection; it may be absent.
    rule_binding_stamp: Option<u64>,
    /// Technical: Stores the `rule_feedback` value in `MenuSelection` as `crate::reaction_rules::Feedback`.
    ///
    /// Plain English: Holds the rule feedback value used by MenuSelection.
    pub rule_feedback: crate::reaction_rules::Feedback,
    /// Technical: Stores the `rule_last_priority` value in `MenuSelection` as `Option<u8>`.
    ///
    /// Plain English: Optionally stores the rule last priority for MenuSelection; it may be absent.
    pub rule_last_priority: Option<u8>,
    /// Technical: Stores the `y_fighter_kind` value in `MenuSelection` as `i32`.
    ///
    /// Plain English: Holds the y fighter kind value used by MenuSelection.
    pub y_fighter_kind: i32,
    /// Technical: Stores the `revision` value in `MenuSelection` as `u64`.
    ///
    /// Plain English: Holds the revision value used by MenuSelection.
    pub revision: u64,
    /// Displayed values/page changed; cursor-only navigation leaves this stable.
    /// Plain English: Displayed values/page changed; cursor-only navigation leaves this stable.
    pub content_revision: u64,
    /// Technical: Stores the `recipe_revision` value in `MenuSelection` as `u64`.
    ///
    /// Plain English: Holds the recipe revision value used by MenuSelection.
    pub recipe_revision: u64,
    /// Technical: Stores the `mode_editor_origin` value in `MenuSelection` as `Option<(MenuRow`.
    ///
    /// Plain English: Optionally stores the mode editor origin for MenuSelection; it may be absent.
    mode_editor_origin: Option<(MenuRow, [usize; PresetSlot::COUNT])>,
    /// Technical: Stores the `mode_rule_list` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the mode rule list value used by MenuSelection.
    mode_rule_list: bool,
    /// Technical: Stores the `mode_rule_cursor` value in `MenuSelection` as `usize`.
    ///
    /// Plain English: Holds the mode rule cursor value used by MenuSelection.
    mode_rule_cursor: usize,
    /// Technical: Stores the `auto_add_event` value in `MenuSelection` as `usize`.
    ///
    /// Plain English: Holds the auto add event value used by MenuSelection.
    auto_add_event: usize,
    /// Technical: Stores the `auto_grab_configured` value in `MenuSelection` as `u8`.
    ///
    /// Plain English: Holds the auto grab configured value used by MenuSelection.
    auto_grab_configured: u8,
    /// Technical: Stores the `has_reactions` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the has reactions value used by MenuSelection.
    pub has_reactions: bool,
    /// Technical: Stores the `recipe_editing` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the recipe editing value used by MenuSelection.
    pub recipe_editing: bool,
    /// Technical: Stores the `preview_slot` value in `MenuSelection` as `Option<PresetSlot>`.
    ///
    /// Plain English: Optionally stores the preview slot for MenuSelection; it may be absent.
    pub preview_slot: Option<PresetSlot>,
    // Suppress a bound physical button until release, even after its tape ends.
    /// Technical: Stores the `button_latch` value in `MenuSelection` as `u32`.
    ///
    /// Plain English: Holds the button latch value used by MenuSelection.
    button_latch: u32,
    /// Technical: Stores the `held_binding` value in `MenuSelection` as `Option<PresetSlot>`.
    ///
    /// Plain English: Optionally stores the held binding for MenuSelection; it may be absent.
    held_binding: Option<PresetSlot>,
    /// Technical: Stores the `save_requested` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the save requested value used by MenuSelection.
    pub save_requested: bool,
    /// Technical: Stores the `delete_requested` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the delete requested value used by MenuSelection.
    pub delete_requested: bool,
    /// Technical: Stores the `default_requested` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the default requested value used by MenuSelection.
    pub default_requested: bool,
    /// Match-only two-page selector; Training retains the full editor.
    /// Plain English: Match-only two-page selector; Training retains the full editor.
    pub lightweight: bool,
    /// Technical: Stores the `lightweight_initialized` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the lightweight initialized value used by MenuSelection.
    lightweight_initialized: bool,
    /// Technical: Stores the `match_visibility_pending` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the match visibility pending value used by MenuSelection.
    match_visibility_pending: bool,
    /// Technical: Stores the `catalog` value in `MenuSelection` as `Option<Box<catalog::Catalog>>`.
    ///
    /// Plain English: Optionally stores the catalog for MenuSelection; it may be absent.
    catalog: Option<Box<catalog::Catalog>>,
    /// Technical: Stores the `save_status` value in `MenuSelection` as `Option<String>`.
    ///
    /// Plain English: Optionally stores the save status for MenuSelection; it may be absent.
    pub save_status: Option<String>,
    /// Technical: Stores the `editor` value in `MenuSelection` as `Option<Box<editor::RecipeEditor>>`.
    ///
    /// Plain English: Optionally stores the editor for MenuSelection; it may be absent.
    editor: Option<Box<editor::RecipeEditor>>,
    /// Technical: Stores the `hold` value in `MenuSelection` as `HoldRepeat`.
    ///
    /// Plain English: Holds the hold value used by MenuSelection.
    hold: HoldRepeat,
    /// Minus hides/shows the panel and never starts a gameplay tool.
    /// Plain English: Minus hides/shows the panel and never starts a gameplay tool.
    pub menu_hidden: bool,
    /// First Minus page: saved recipe selection and Training L3 test, without editing.
    /// Plain English: First Minus page: saved recipe selection and Training L3 test, without editing.
    pub quick_menu: bool,
    /// Third page groups existing non-sequence modes and toggles.
    /// Plain English: Third page groups existing non-sequence modes and toggles.
    pub modes_menu: bool,
    /// Technical: Stores the `button_menu` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the button menu value used by MenuSelection.
    pub button_menu: bool,
    /// Technical: Stores the `auto_ledge` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the auto ledge value used by MenuSelection.
    pub auto_ledge: bool,
    /// Technical: Stores the `tuning` value in `MenuSelection` as `crate::fighter_tuning::Tuning`.
    ///
    /// Plain English: Holds the tuning value used by MenuSelection.
    pub tuning: crate::fighter_tuning::Tuning,
    /// Technical: Stores the `tuning_values` value in `MenuSelection` as `crate::fighter_tuning::Values`.
    ///
    /// Plain English: Holds the tuning values value used by MenuSelection.
    pub tuning_values: crate::fighter_tuning::Values,
    /// Technical: Stores the `tuning_page` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the tuning page value used by MenuSelection.
    pub tuning_page: bool,
    /// Technical: Stores the `tuning_row` value in `MenuSelection` as `usize`.
    ///
    /// Plain English: Holds the tuning row value used by MenuSelection.
    pub tuning_row: usize,
    /// Technical: Stores the `tuning_save_requested` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the tuning save requested value used by MenuSelection.
    pub tuning_save_requested: bool,

    /// Technical: Stores the `out_of_shield` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the out of shield value used by MenuSelection.
    pub out_of_shield: bool,
    /// One input-sample event. The adapter owns the shared mash toggle so it
    /// survives fighter/stock changes rather than being a fighter preset.
    /// Plain English: One input-sample event. The adapter owns the shared mash toggle so it survives fighter/stock changes rather than being a fighter preset.
    pub toggle_mash: bool,
    /// Technical: Stores the `idle_crouch` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the idle crouch value used by MenuSelection.
    pub idle_crouch: bool,
    /// Technical: Stores the `tilt_turn` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the tilt turn value used by MenuSelection.
    pub tilt_turn: bool,
    /// Technical: Stores the `spot_punish` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the spot punish value used by MenuSelection.
    pub spot_punish: bool,
    /// Technical: Stores the `grab_dodge` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the grab dodge value used by MenuSelection.
    pub grab_dodge: bool,
    /// Technical: Stores the `auto_tech` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the auto tech value used by MenuSelection.
    pub auto_tech: bool,
    /// Technical: Stores the `auto_reflect` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the auto reflect value used by MenuSelection.
    pub auto_reflect: bool,
    /// Technical: Stores the `performance` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the performance value used by MenuSelection.
    pub performance: bool,
    /// Diagnostic annotation only: 0=unset, 1..=31 label external delay 0..=30f.
    /// Plain English: Diagnostic annotation only: 0=unset, 1..=31 label external delay 0..=30f.
    pub test_latency: u8,
    /// Technical: Stores the `previous` value in `MenuSelection` as `Option<u32>`.
    ///
    /// Plain English: Optionally stores the previous for MenuSelection; it may be absent.
    previous: Option<u32>,
    // Menu-held directions must be released before becoming gameplay triggers.
    /// Technical: Stores the `pad_quarantine` value in `MenuSelection` as `u32`.
    ///
    /// Plain English: Holds the pad quarantine value used by MenuSelection.
    pad_quarantine: u32,
    // A held button must be released after reset/config changes before it can
    // control an assist. Unowned gameplay input remains physical.
    /// Technical: Stores the `trigger_ready` value in `MenuSelection` as `u32`.
    ///
    /// Plain English: Holds the trigger ready value used by MenuSelection.
    trigger_ready: u32,
    /// Technical: Stores the `cancel_pending` value in `MenuSelection` as `bool`.
    ///
    /// Plain English: Holds the cancel pending value used by MenuSelection.
    pub cancel_pending: bool,
}
impl MenuSelection {
    /// Estimated owned payload, not allocator bookkeeping or game memory.
    /// Diagnostic adapter calls only after revisions, never scans on idle polls.
    /// Plain English: Estimated owned payload, not allocator bookkeeping or game memory. Diagnostic adapter calls only after revisions, never scans on idle polls.
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
    /// Technical: Defines the `QUICK_ROWS` constant used by this module.
    ///
    /// Plain English: Provides the QUICK ROWS constant value.
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
    /// Technical: Defines the `LEDGE_ROWS` constant used by this module.
    ///
    /// Plain English: Provides the LEDGE ROWS constant value.
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
    /// Technical: Defines the `BUTTON_ROWS` constant used by this module.
    ///
    /// Plain English: Provides the BUTTON ROWS constant value.
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
    /// Technical: Defines the `ASSIST_ROWS` constant used by this module.
    ///
    /// Plain English: Provides the ASSIST ROWS constant value.
    pub const ASSIST_ROWS: [MenuRow; 18] = [
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
        MenuRow::RollRead,
        MenuRow::Performance,
        MenuRow::TestLatency,
        MenuRow::LedgePage,
        MenuRow::ButtonsPage,
        MenuRow::FighterSettings,
        MenuRow::ModesPage,
    ];
    /// The button recipes followed by the assist rows.
    /// Plain English: The button recipes followed by the assist rows.
    const LIGHTWEIGHT_ROWS: [MenuRow; PresetSlot::BUTTONS.len() + Self::ASSIST_ROWS.len()] = {
        let mut rows = [MenuRow::Neutral; PresetSlot::BUTTONS.len() + Self::ASSIST_ROWS.len()];
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
    // The fixed ledge modifier owns lightweight navigation. Custom D-pad
    // bindings (including a D-pad assigned to ledge assist) remain gameplay.
    /// Technical: Processes navigation input for the lightweight settings view.
    ///
    /// Plain English: Handles button navigation in the compact settings view.
    fn lightweight_navigation(&self) -> bool {
        self.binding_physical & self.binding_layout.physical_buttons(MAPPER_L) != 0
    }
    /// Technical: Moves the selection among rows in the current page.
    ///
    /// Plain English: Moves the highlight to another row.
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
    /// Technical: Builds the rows visible in the current settings page.
    ///
    /// Plain English: Lists the rows to show on the page.
    pub fn display_rows(&self) -> &'static [MenuRow] {
        if self.modes_menu {
            &Self::ASSIST_ROWS
        } else if self.lightweight && self.button_menu {
            &Self::LIGHTWEIGHT_ROWS
        } else {
            &MenuRow::ALL
        }
    }
    /// Technical: Returns the row range belonging to the requested page.
    ///
    /// Plain English: Gets the rows for one page.
    fn page_rows(&self) -> &'static [MenuRow] {
        if self.button_menu {
            &Self::BUTTON_ROWS
        } else {
            &Self::LEDGE_ROWS
        }
    }
    /// Technical: Builds the navigation link for a settings page.
    ///
    /// Plain English: Creates the button link to a page.
    pub fn page_link(row: MenuRow) -> bool {
        matches!(
            row,
            MenuRow::LedgePage
                | MenuRow::ButtonsPage
                | MenuRow::ModesPage
                | MenuRow::FighterSettings
        )
    }
    /// Technical: Opens the requested settings page.
    ///
    /// Plain English: Shows the selected page.
    fn open_page(&mut self, row: MenuRow) {
        self.hold.reset();
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
        self.close_catalog();
        if self.lightweight && self.preview_slot.is_none() && self.held_binding.is_none() {
            self.content_changed();
        } else {
            self.changed();
        }
    }
    /// Technical: Checks whether the row is visible in the current viewport.
    ///
    /// Plain English: Checks whether the row can be seen.
    pub fn row_visible(&self, row: MenuRow) -> bool {
        if self.tuning_page {
            return false;
        }
        if Self::page_link(row) {
            return !self.lightweight && self.page_link_visible(row);
        }
        self.row_on_page(row) && self.row_available(row)
    }
    /// Technical: Checks whether the page link is visible in the current viewport.
    ///
    /// Plain English: Checks whether the page link can be seen.
    fn page_link_visible(&self, row: MenuRow) -> bool {
        match row {
            MenuRow::FighterSettings => true,
            MenuRow::LedgePage => self.button_menu || self.modes_menu,
            MenuRow::ButtonsPage => !self.button_menu,
            MenuRow::ModesPage => !self.modes_menu,
            _ => false,
        }
    }
    /// Technical: Checks whether a row belongs to the requested page.
    ///
    /// Plain English: Checks whether the row is on this page.
    fn row_on_page(&self, row: MenuRow) -> bool {
        let slot = row.slot();
        if self.lightweight && self.button_menu {
            slot.is_none_or(|s| s.is_button())
        } else if self.modes_menu {
            slot.is_none()
        } else {
            slot.is_some_and(|s| s.is_button() == self.button_menu)
        }
    }
    /// Rows hidden by the build, the fighter or the reaction mode.
    /// Plain English: Rows hidden by the build, the fighter or the reaction mode.
    fn row_available(&self, row: MenuRow) -> bool {
        !(!cfg!(feature = "diagnostic-logging")
            && matches!(row, MenuRow::Performance | MenuRow::TestLatency))
            && row != MenuRow::RollRead
            && !(self.simplified_reactions
                && matches!(row, MenuRow::ShieldPressure | MenuRow::SpotPunish))
            && !(row == MenuRow::IdleCrouch && self.y_fighter_kind != crate::fighter_kinds::KIRBY)
            && !(row == MenuRow::AutoReflect
                && crate::projectile_reflect::Config::for_fighter(self.y_fighter_kind, self.tuning)
                    .is_none())
    }
    /// Technical: Constructs a new value with the type’s initial state.
    ///
    /// Plain English: Creates a new value.
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
    /// The on/off setting a mode row toggles; Auto-mash is an event, not a flag.
    /// Plain English: The on/off setting a mode row toggles; Auto-mash is an event, not a flag.
    fn mode_flag(&mut self, row: MenuRow) -> Option<&mut bool> {
        Some(match row {
            MenuRow::ShieldPressure => &mut self.shield_pressure,
            MenuRow::RollRules => &mut self.roll_rules,
            MenuRow::AutoTech => &mut self.auto_tech,
            MenuRow::AutoReflect => &mut self.auto_reflect,
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
    /// Technical: Applies the selected adjustment mode to the value.
    ///
    /// Plain English: Changes the value using the selected adjustment mode.
    fn adjust_mode(&mut self, forward: bool) {
        match self.row {
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
    /// Plain English: Ledge autopilot stays OFF unless separately enabled by the user.
    pub fn sequence_enabled(&self, base: bool) -> bool {
        base || self.preview_slot.is_some() || self.ledge_mode()
    }
    /// Technical: Returns the current settings view.
    ///
    /// Plain English: Gets the settings being edited.
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
    /// Technical: Checks whether the tuning section is visible.
    ///
    /// Plain English: Checks whether the tuning options are shown.
    fn tuning_visible(&self, i: usize) -> bool {
        crate::fighter_tuning::Tuning::visible(self.y_fighter_kind, i)
            && !(i == crate::fighter_tuning::AUTO_GRAB_CLOSE && self.auto_grab_configured & 2 != 0)
            && !(i == crate::fighter_tuning::AUTO_GRAB_FAR && self.auto_grab_configured & 1 != 0)
    }
    /// The first visible fighter-setting row at or after `row`, wrapping to the first one.
    /// Plain English: The first visible fighter-setting row at or after row, wrapping to the first one.
    fn tuning_visible_index(&self, row: usize) -> usize {
        (row..crate::fighter_tuning::COUNT)
            .find(|i| self.tuning_visible(*i))
            .or_else(|| (0..crate::fighter_tuning::COUNT).find(|i| self.tuning_visible(*i)))
            .unwrap_or(0)
    }
    /// Technical: Returns the current recipe-input validation error, if any.
    ///
    /// Plain English: Gets the recipe error message if one exists.
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
    /// Technical: Checks whether the recipe differs from its saved value.
    ///
    /// Plain English: Checks whether the recipe has unsaved changes.
    pub fn recipe_modified(&self, slot: PresetSlot) -> bool {
        let index = self.choices[slot as usize];
        self.editor
            .as_ref()
            .is_some_and(|e| e.edited((slot as usize, index)))
            || self.draft(slot, index).is_some_and(|d| d.dirty)
    }
    /// Technical: Checks whether ledge assistance is currently held.
    ///
    /// Plain English: Checks whether the ledge-assist input is held.
    pub fn ledge_held(&self, buttons: u32) -> bool {
        // Training navigation is still a UI command even when that physical
        // direction is assigned to a gameplay helper and filtered from policy.
        if !self.menu_hidden && !self.lightweight && buttons & MAPPER_DPAD_BUTTONS != 0 {
            return false;
        }
        let buttons = self.native_buttons(buttons);
        let blocked = if self.lightweight {
            // Minus is always a selector control; visible-menu D-pad is too.
            (if self.menu_hidden { MAPPER_DPAD_BUTTONS } else { 0 }) | MAPPER_Y
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
    /// Plain English: A toggled ledge watch stays reserved even while manual input temporarily pauses its movement. Safety tech/mash and ledge shield are separate.
    pub fn reserves_automatic_modes(&self) -> bool {
        self.ledge_mode()
            || self.preview_slot.is_some()
            || self.native_buttons(self.binding_layout.policy_buttons(self.binding_physical))
                & MAPPER_L
                != 0
    }
    /// Technical: Checks whether autopilot assistance is currently held.
    ///
    /// Plain English: Checks whether the autopilot input is held.
    pub fn autopilot_held(&self, buttons: u32) -> bool {
        self.ledge_mode()
            && !self.recipe_editing
            && !self.managing()
            && !self.cancel_pending
            && (buttons & MAPPER_DPAD_BUTTONS == 0
                || self.lightweight && !self.menu_hidden && self.lightweight_navigation())
    }
    /// Telemetry is not selector content; updating it must not rebuild menu text.
    /// Plain English: Telemetry is not selector content; updating it must not rebuild menu text.
    pub fn record_rule_feedback(
        &mut self,
        feedback: crate::reaction_rules::Feedback,
        last: Option<u8>,
    ) {
        self.rule_feedback = feedback;
        self.rule_last_priority = last;
    }

    /// Cursor/page moved; displayed values did not.
    /// Plain English: Cursor/page moved; displayed values did not.
    fn view_changed(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
    /// Displayed values or the page changed.
    /// Plain English: Displayed values or the page changed.
    fn content_changed(&mut self) {
        self.content_revision = self.content_revision.wrapping_add(1);
        self.view_changed();
    }
    /// A menu change cancels any running test or held binding.
    /// Plain English: A menu change cancels any running test or held binding.
    fn changed(&mut self) {
        self.content_changed();
        self.held_binding = None;
        self.preview_slot = None;
        self.cancel_pending = true;
        // Binding changes require a Y release before the new mode can run.
    }
    /// Recipe data changed: caches keyed on `recipe_revision` must rebuild.
    /// Plain English: Recipe data changed: caches keyed on reciperevision must rebuild.
    fn bump_recipes(&mut self) {
        self.recipe_revision = self.recipe_revision.wrapping_add(1);
    }
    /// Technical: Checks whether any recipe edits are pending.
    ///
    /// Plain English: Checks whether recipes have unsaved changes.
    fn recipes_edited(&mut self, status: &str) {
        self.bump_recipes();
        self.save_status = Some(status.into());
    }
    /// Technical: Closes the recipe catalog and returns to the previous view.
    ///
    /// Plain English: Closes the recipe list.
    fn close_catalog(&mut self) {
        if let Some(catalog) = self.catalog.as_mut() {
            catalog.close();
        }
    }
    /// Technical: Closes the active editor command state.
    ///
    /// Plain English: Leaves the editor command view.
    fn close_editor_commands(&mut self) {
        if let Some(editor) = self.editor.as_mut() {
            editor.close_commands();
        }
    }
    /// Technical: Closes the active tools view.
    ///
    /// Plain English: Leaves the tools view.
    fn close_tools(&mut self) {
        self.close_editor_commands();
        self.close_catalog();
    }
    /// One-shot save/delete/default requests live for a single input sample.
    /// Plain English: One-shot save/delete/default requests live for a single input sample.
    fn clear_requests(&mut self) {
        self.save_requested = false;
        self.delete_requested = false;
        self.default_requested = false;
        self.tuning_save_requested = false;
    }

    /// Technical: Resets the inputs.
    ///
    /// Plain English: Resets the inputs.
    pub fn reset_inputs(&mut self) {
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
    /// Plain English: Observe real releases while another tool owns input. Do not navigate, toggle mash, consume buttons or run an assist during passthrough. Remember held directions too, so resuming cannot synthesize a menu tap.
    pub fn observe_passthrough(&mut self, buttons: u32) {
        self.observe_physical_passthrough(
            self.binding_layout.physical_buttons(buttons),
            self.binding_layout,
        );
    }
    /// Technical: Observes the physical passthrough.
    ///
    /// Plain English: Observes the physical passthrough.
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
    /// Plain English: Keep real edge history while a visible editor waits for its save worker. Do not accumulate invisible edits or replay a held Save on completion. A Training worker must not keep its editor/layout alive in another mode or skip.
    pub fn waits_for_training_save(&self, training: bool, busy: bool) -> bool {
        training && busy && !self.lightweight && !self.match_visibility_pending && !self.menu_hidden
    }
    /// Technical: Waits for the pending save operation to complete.
    ///
    /// Plain English: Waits until saving is done.
    pub fn wait_for_save(&mut self, buttons: u32) -> u32 {
        self.observe_passthrough(buttons);
        self.cancel_pending = false;
        self.toggle_mash = false;
        self.clear_requests();
        self.button_latch &= buttons;
        buttons & !MAPPER_PLUS
    }
    /// A momentary Training save/load may resume held-L practice, never Y.
    /// Plain English: A momentary Training save/load may resume held-L practice, never Y.
    pub fn resume_training_ledge_hold(&mut self) {
        self.trigger_ready |= MAPPER_L;
    }
    /// Combat cancellation must not erase menu press history during entry,
    /// death or respawn: saved choices remain editable in those phases.
    /// Plain English: Combat cancellation must not erase menu press history during entry, death or respawn: saved choices remain editable in those phases.
    pub fn cancel_actions(&mut self) {
        self.held_binding = None;
        self.trigger_ready = 0;
        self.cancel_pending = true;
    }
    /// Technical: Builds the requested display representation.
    ///
    /// Plain English: Creates the view to show.
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
    /// Technical: Builds the display for the available assistance modes.
    ///
    /// Plain English: Shows the assistance mode choices.
    pub fn show_modes(&mut self) {
        self.show();
        self.modes_menu = true;
        self.recipe_editing = false;
        self.row = MenuRow::RollRules;
    }
    /// Technical: Builds the quick-settings display.
    ///
    /// Plain English: Shows the quick settings.
    pub fn show_quick(&mut self) {
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
        self.recipe_editing = false;
        if !Self::QUICK_ROWS.contains(&self.row) {
            self.row = MenuRow::Neutral;
        }
        self.changed();
    }
    /// Resolve the new battle's visibility on its first scene-aware input.
    /// Lightweight starts visible; Training starts hidden.
    /// Plain English: Resolve the new battle's visibility on its first scene-aware input. Lightweight starts visible; Training starts hidden.
    pub fn begin_match(&mut self) {
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
/// Technical: Advances the selected choice to the next value.
///
/// Plain English: Moves to the next choice.
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
#[path = "../tests/unit/settings_menu.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/unit/settings_menu_maintenance.rs"]
mod maintenance_tests;
