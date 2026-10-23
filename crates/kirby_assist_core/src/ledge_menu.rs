//! Saved ledge choices only: L runs ledge autopilot; Y is reserved for B-reverse.
//! L+R3 toggles the panel; Minus toggles mash. Hidden sides slide, Down trumps.
use crate::fighter_profiles::presets::{MenuPresets, PresetSlot};
use crate::ledge_trap::{LedgeSettings, DPAD_DOWN};
use crate::manual_controls::{DPAD_RIGHT, DPAD_SIDES, DPAD_UP, SHORT_HOP_TRIGGER};
use crate::native_input::{MAPPER_L, MAPPER_MINUS, MAPPER_Y};
pub const MENU_STICK_CLICK: u32 = 1 << 17; // Physical right-stick click (R3).
pub const TWO_FRAME_TRIGGER: u32 = 1 << 16; // Physical L3, Kirby only in live adapter.
const MENU_CHORD: u32 = MAPPER_L | MENU_STICK_CLICK;

/// Menu consumption applies to the game-facing overlay, but readiness observers
/// must still see physical assist triggers before any tool has started. Dropping
/// Y here prevents the first B-reverse press from ever opening its native reads.
pub fn observation_buttons(physical: u32, consumed: u32) -> u32 {
    let triggers = MAPPER_L | MAPPER_Y | SHORT_HOP_TRIGGER | DPAD_UP | TWO_FRAME_TRIGGER;
    physical & !(consumed & !triggers)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuRow {
    #[default]
    Neutral,
    Jump,
    Roll,
    Attack,
    Drop,
}
impl MenuRow {
    pub const ALL: [Self; 5] = [
        Self::Neutral,
        Self::Jump,
        Self::Roll,
        Self::Attack,
        Self::Drop,
    ];
    pub fn slot(self) -> Option<PresetSlot> {
        match self {
            Self::Neutral => Some(PresetSlot::Neutral),
            Self::Jump => Some(PresetSlot::Jump),
            Self::Roll => Some(PresetSlot::Roll),
            Self::Attack => Some(PresetSlot::Attack),
            Self::Drop => Some(PresetSlot::Drop),
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Neutral => "Neutral",
            Self::Jump => "Jump",
            Self::Roll => "Roll",
            Self::Attack => "Attack",
            Self::Drop => "Drop",
        }
    }
}
#[derive(Default)]
pub struct MenuSelection {
    pub row: MenuRow,
    /// Indices into the five compiled response lists, never raw settings.
    pub choices: [usize; 5],
    pub revision: u64,
    /// L+R3 hides/shows the panel and never starts a gameplay tool.
    pub menu_hidden: bool,
    /// One input-sample event. The adapter owns the shared mash toggle so it
    /// survives fighter/stock changes rather than being a fighter preset.
    pub toggle_mash: bool,
    previous: Option<u32>,
    // Menu-held directions must be released before becoming gameplay triggers.
    pad_quarantine: u32,
    // A held button must be released after reset/config changes before it can
    // control an assist. Unowned gameplay input remains physical.
    trigger_ready: u32,
    pub cancel_pending: bool,
}
impl MenuSelection {
    pub fn new(presets: &MenuPresets) -> Self {
        Self {
            choices: std::array::from_fn(|i| presets.lists[i].selected),
            ..Self::default()
        }
    }
    pub fn settings(&self, presets: &MenuPresets, base: LedgeSettings) -> LedgeSettings {
        // Preparation resolves only this compiled function's metadata/tuning.
        // Function indices, not function-pointer equality, are cached settings.
        let mut s = base;
        s.two_frame = false; // Saved responses always belong to L's ledge trap.
        s.two_frame_down_tilt = false;
        for slot in PresetSlot::ALL {
            if let Some(choice) = presets.list(slot).options.get(self.choices[slot as usize]) {
                s = (choice.prepare)(s);
                s.response_choices[slot as usize] = Some(self.choices[slot as usize]);
            }
        }
        s
    }
    pub fn ledge_held(&self, buttons: u32) -> bool {
        !self.cancel_pending
            && buttons & self.trigger_ready & MAPPER_L != 0
            && buttons
                & (DPAD_SIDES
                    | DPAD_UP
                    | DPAD_DOWN
                    | MAPPER_MINUS
                    | MENU_STICK_CLICK
                    | MAPPER_Y
                    | SHORT_HOP_TRIGGER
                    | TWO_FRAME_TRIGGER)
                == 0
    }
    pub fn two_frame_held(&self, buttons: u32) -> bool {
        !self.cancel_pending
            && buttons & TWO_FRAME_TRIGGER != 0
            && buttons
                & (DPAD_SIDES | DPAD_UP | DPAD_DOWN | MAPPER_MINUS | MAPPER_Y | MENU_STICK_CLICK)
                == 0
    }
    /// L+R3 toggles once per chord press. While visible, all D-pad directions belong
    /// to navigation, including when L is held. Closing preserves saved choices.
    pub fn input(&mut self, buttons: u32, presets: &MenuPresets) -> u32 {
        // Cancellation belongs to one mapper sample, not a later observer.
        self.cancel_pending = false;
        self.toggle_mash = false;
        let previous = self.previous.unwrap_or(buttons);
        let pressed = buttons & !previous;
        self.previous = Some(buttons);
        let directions = DPAD_SIDES | DPAD_UP | DPAD_DOWN;
        let pad = buttons & directions;
        let quarantined = self.pad_quarantine;
        self.pad_quarantine &= buttons;
        if pressed & (MAPPER_L | TWO_FRAME_TRIGGER) != 0 {
            self.cancel_pending = true; // Release an old proposal before takeover.
        }
        self.toggle_mash = pressed & MAPPER_MINUS != 0;
        let toggled = buttons & MENU_CHORD == MENU_CHORD && previous & MENU_CHORD != MENU_CHORD;
        if toggled {
            self.menu_hidden = !self.menu_hidden;
            self.pad_quarantine |= MENU_STICK_CLICK;
            self.changed(); // Release queued gameplay inputs at either boundary.
        }
        let editing = !self.menu_hidden;
        if editing || toggled {
            self.pad_quarantine |= pad;
        }
        if editing && !toggled && pad.count_ones() == 1 && pressed & pad != 0 {
            if pad == DPAD_UP || pad == DPAD_DOWN {
                self.row =
                    MenuRow::ALL[cycle(self.row as usize, MenuRow::ALL.len(), pad == DPAD_DOWN)];
            } else if let Some(slot) = self.row.slot() {
                self.choices[slot as usize] = cycle(
                    self.choices[slot as usize],
                    presets.list(slot).options.len(),
                    pad == DPAD_RIGHT,
                );
            }
            self.changed();
        }
        self.trigger_ready |= !buttons & (MAPPER_L | TWO_FRAME_TRIGGER);
        // Visible directions and their release edges cannot reach gameplay.
        // Hidden Left/Right/Down are reserved separately by the enabled tools.
        MAPPER_Y
            | TWO_FRAME_TRIGGER
            | MAPPER_L
            | MAPPER_MINUS
            | ((quarantined | self.pad_quarantine) & MENU_STICK_CLICK)
            | if editing {
                directions
            } else {
                quarantined | self.pad_quarantine
            }
    }
    fn changed(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        self.cancel_pending = true;
        // Saved-choice edits do not rebind or arm the independent Y shortcut.
    }
    pub fn reset_inputs(&mut self) {
        self.previous = None;
        self.pad_quarantine = DPAD_SIDES | DPAD_UP | DPAD_DOWN;
        self.cancel_actions();
    }
    /// Combat cancellation must not erase menu press history during entry,
    /// death or respawn: saved choices remain editable in those phases.
    pub fn cancel_actions(&mut self) {
        self.trigger_ready = 0;
        self.cancel_pending = true;
    }
    /// A new battle shows the saved choices; L+R3 hides them afterward.
    pub fn begin_match(&mut self) {
        self.menu_hidden = false;
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
mod tests;
