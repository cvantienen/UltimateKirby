//! Saved ledge choices and Y binding: L always runs ledge autopilot.
//! R3 toggles the panel; Auto-mash is a saved menu row.
use crate::button_modes::YMode;
use crate::fighter_profiles::presets::{MenuPresets, PresetSlot};
use crate::ledge_trap::{LedgeSettings, DPAD_DOWN};
use crate::manual_controls::{DPAD_RIGHT, DPAD_SIDES, DPAD_UP, SHORT_HOP_TRIGGER};
use crate::native_input::{MAPPER_L, MAPPER_MINUS, MAPPER_Y};
pub const MENU_STICK_CLICK: u32 = 1 << 17; // Physical right-stick click (R3).
pub const TWO_FRAME_TRIGGER: u32 = 1 << 16; // Legacy constant; L3 is no longer the menu trigger.
pub const MENU_TOGGLE_CHORD: u32 = MENU_STICK_CLICK; // Compatibility name; R3 alone.
pub fn monitor_chord(buttons: u32) -> bool {
    let mask = MAPPER_L | DPAD_DOWN | MENU_STICK_CLICK;
    buttons & mask == mask
}
pub fn menu_toggle_pressed(held: u32, pressed: u32) -> bool {
    // Older Tesla/monitor setups use L+Down+R3. That overlay chord must not
    // also toggle our panel. An ordinary R3 click needs no modifier.
    pressed & MENU_STICK_CLICK != 0 && !monitor_chord(held)
}

/// Menu consumption applies to the game-facing overlay, but readiness observers
/// must still see physical assist triggers before any tool has started. Dropping
/// Y here prevents the held Y mode from opening its native reads.
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
    RollRead,
    YMode,
    AutoMash,
    IdleCrouch,
    TiltTurn,
    JabGrab,
    JabLock,
    SpotPunish,
    GrabDodge,
}
impl MenuRow {
    pub const ALL: [Self; 14] = [
        Self::Neutral,
        Self::Jump,
        Self::Roll,
        Self::Attack,
        Self::Drop,
        Self::RollRead,
        Self::YMode,
        Self::AutoMash,
        Self::IdleCrouch,
        Self::TiltTurn,
        Self::JabGrab,
        Self::JabLock,
        Self::SpotPunish,
        Self::GrabDodge,
    ];
    pub fn slot(self) -> Option<PresetSlot> {
        match self {
            Self::Neutral => Some(PresetSlot::Neutral),
            Self::Jump => Some(PresetSlot::Jump),
            Self::Roll => Some(PresetSlot::Roll),
            Self::Attack => Some(PresetSlot::Attack),
            Self::Drop => Some(PresetSlot::Drop),
            Self::RollRead
            | Self::YMode
            | Self::AutoMash
            | Self::IdleCrouch
            | Self::TiltTurn
            | Self::JabGrab
            | Self::JabLock
            | Self::SpotPunish
            | Self::GrabDodge => None,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Neutral => "Neutral",
            Self::Jump => "Jump",
            Self::Roll => "Roll",
            Self::Attack => "Attack",
            Self::Drop => "Drop",
            Self::RollRead => "Roll read",
            Self::YMode => "Finisher",
            Self::AutoMash => "Auto-mash",
            Self::IdleCrouch => "Idle crouch",
            Self::TiltTurn => "Dtilt face",
            Self::JabGrab => "A+B followup",
            Self::JabLock => "Jab lock",
            Self::SpotPunish => "Spot punish",
            Self::GrabDodge => "Grab dodge+punish",
        }
    }
}
#[derive(Default)]
pub struct MenuSelection {
    pub row: MenuRow,
    /// Indices into the five compiled response lists, never raw settings.
    pub choices: [usize; 5],
    pub roll_read: crate::roll_read::Choice,
    pub roll_read_available: bool,
    pub y_mode: YMode,
    pub auto_duck_available: bool,
    pub y_fighter_kind: i32,
    pub revision: u64,
    /// R3 hides/shows the panel and never starts a gameplay tool.
    pub menu_hidden: bool,
    /// One input-sample event. The adapter owns the shared mash toggle so it
    /// survives fighter/stock changes rather than being a fighter preset.
    pub toggle_mash: bool,
    pub idle_crouch: bool,
    pub tilt_turn: bool,
    pub jab_grab: crate::jab_grab::Followup,
    pub jab_lock: bool,
    pub spot_punish: bool,
    pub grab_dodge: bool,
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
            roll_read: crate::roll_read::Choice::DownSmash,
            y_fighter_kind: 6,
            idle_crouch: true,
            tilt_turn: true,
            jab_grab: crate::jab_grab::Followup::Grab,
            spot_punish: true,
            grab_dodge: true,
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
                & (DPAD_SIDES | DPAD_UP | DPAD_DOWN | MAPPER_MINUS | MAPPER_Y | SHORT_HOP_TRIGGER)
                == 0
    }
    pub fn auto_duck_held(&self, buttons: u32) -> bool {
        let _ = buttons;
        false // Y is now the selected finisher; auto-duck has no binding.
    }
    pub fn finisher_held(&self, buttons: u32) -> bool {
        self.auto_duck_available
            && !self.cancel_pending
            && buttons & self.trigger_ready & MAPPER_Y != 0
            && buttons & !(MAPPER_Y | MAPPER_L | crate::manual_controls::LEFT_STICK_DIRECTIONS) == 0
    }
    /// R3 toggles once per press. While visible, all D-pad directions belong
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
        if pressed & MAPPER_L != 0 {
            self.cancel_pending = true; // Release an old proposal before takeover.
        }
        let toggled = menu_toggle_pressed(buttons, pressed);
        if toggled {
            self.menu_hidden = !self.menu_hidden;
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
                while (self.row == MenuRow::JabLock && self.y_fighter_kind != 5)
                    || (self.row == MenuRow::RollRead && !self.roll_read_available)
                    || (self.row == MenuRow::YMode && !self.auto_duck_available)
                    || (matches!(
                        self.row,
                        MenuRow::IdleCrouch | MenuRow::TiltTurn | MenuRow::JabGrab
                    ) && self.y_fighter_kind != 6)
                {
                    self.row = MenuRow::ALL
                        [cycle(self.row as usize, MenuRow::ALL.len(), pad == DPAD_DOWN)];
                }
            } else if self.row == MenuRow::RollRead && self.roll_read_available {
                self.roll_read = self.roll_read.cycle(pad == DPAD_RIGHT);
            } else if self.row == MenuRow::YMode && self.auto_duck_available {
                self.y_mode = self
                    .y_mode
                    .cycle_for_kind(pad == DPAD_RIGHT, self.y_fighter_kind);
                // Changing a binding while Y is held must not start its new action.
                self.trigger_ready &= !MAPPER_Y;
            } else if self.row == MenuRow::AutoMash {
                self.toggle_mash = true;
            } else if self.row == MenuRow::IdleCrouch {
                self.idle_crouch = !self.idle_crouch;
            } else if self.row == MenuRow::TiltTurn {
                self.tilt_turn = !self.tilt_turn;
            } else if self.row == MenuRow::JabLock {
                self.jab_lock = !self.jab_lock;
            } else if self.row == MenuRow::SpotPunish {
                self.spot_punish = !self.spot_punish;
            } else if self.row == MenuRow::GrabDodge {
                self.grab_dodge = !self.grab_dodge;
            } else if self.row == MenuRow::JabGrab {
                self.jab_grab = self.jab_grab.cycle();
            } else if let Some(slot) = self.row.slot() {
                self.choices[slot as usize] = cycle(
                    self.choices[slot as usize],
                    presets.list(slot).options.len(),
                    pad == DPAD_RIGHT,
                );
            }
            self.changed();
        }
        self.trigger_ready |= !buttons & (MAPPER_L | MAPPER_Y);
        // Visible directions and their release edges cannot reach gameplay.
        // Hidden Left/Right/Down are reserved separately by the enabled tools.
        MAPPER_Y
            | MAPPER_L
            | MENU_STICK_CLICK
            | if editing {
                directions
            } else {
                quarantined | self.pad_quarantine
            }
    }
    fn changed(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        self.cancel_pending = true;
        // Binding changes require a Y release before the new mode can run.
    }
    pub fn reset_inputs(&mut self) {
        self.previous = None;
        self.pad_quarantine = DPAD_SIDES | DPAD_UP | DPAD_DOWN;
        self.cancel_actions();
    }
    /// Observe real releases while another tool owns input. Do not navigate,
    /// toggle mash, consume buttons or run an assist during passthrough.
    /// Remember held directions too, so resuming cannot synthesize a menu tap.
    pub fn observe_passthrough(&mut self, buttons: u32) {
        self.previous = Some(buttons);
        self.pad_quarantine &= buttons;
        self.trigger_ready |= !buttons & (MAPPER_L | MAPPER_Y);
    }
    /// A momentary Training save/load may resume held-L practice, never Y.
    pub fn resume_training_ledge_hold(&mut self) {
        self.trigger_ready |= MAPPER_L;
    }
    /// Combat cancellation must not erase menu press history during entry,
    /// death or respawn: saved choices remain editable in those phases.
    pub fn cancel_actions(&mut self) {
        self.trigger_ready = 0;
        self.cancel_pending = true;
    }
    pub fn show(&mut self) {
        self.menu_hidden = false;
        self.changed();
    }
    /// A new battle shows the saved choices; R3 hides them afterward.
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
#[path = "ledge_menu/tests.rs"]
mod tests;
