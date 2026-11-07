//! Startup-frozen file menu: labels and references to validated input timelines.
//! No compiled fighter choices, response functions or move timing defaults.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresetSlot {
    Neutral,
    Jump,
    Roll,
    Attack,
    Drop,
    X,
    Y,
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    L3,
    R3,
}
impl PresetSlot {
    pub const COUNT: usize = 13;
    pub const ALL: [Self; Self::COUNT] = [
        Self::Neutral,
        Self::Jump,
        Self::Roll,
        Self::Attack,
        Self::Drop,
        Self::X,
        Self::Y,
        Self::DpadUp,
        Self::DpadDown,
        Self::DpadLeft,
        Self::DpadRight,
        Self::L3,
        Self::R3,
    ];
    pub const BUTTONS: [Self; 8] = [
        Self::X,
        Self::Y,
        Self::DpadUp,
        Self::DpadDown,
        Self::DpadLeft,
        Self::DpadRight,
        Self::L3,
        Self::R3,
    ];
    pub fn button(self) -> u32 {
        use crate::{manual_controls::*, native_input::*};
        match self {
            Self::X => MAPPER_X,
            Self::Y => MAPPER_Y,
            Self::DpadUp => DPAD_UP,
            Self::DpadDown => crate::ledge_trap::DPAD_DOWN,
            Self::DpadLeft => DPAD_LEFT,
            Self::DpadRight => DPAD_RIGHT,
            Self::L3 => 1 << 16,
            Self::R3 => 1 << 17,
            _ => 0,
        }
    }
    pub fn is_button(self) -> bool {
        self as usize >= Self::X as usize
    }
    pub fn default_index(self, saved: usize) -> usize {
        saved
            + if self.is_button() {
                crate::button_bindings::Builtin::default_for(self).template_offset()
            } else {
                0
            }
    }
    pub fn max_choices(self) -> usize {
        32
    }
    pub fn templates(self) -> usize {
        if self.is_button() {
            5 // Normal + custom recipe + three native helpers.
        } else {
            1
        }
    }
}
/// A file-authored menu choice. There are no executable fighter handlers here.
#[derive(Clone, Copy, Debug)]
pub struct SavedChoice {
    pub id: &'static str,
    pub name: &'static str,
    pub sequence: &'static crate::sequence_config::Sequence,
}
impl SavedChoice {
    pub(crate) fn file(
        id: &'static str,
        name: &'static str,
        sequence: &'static crate::sequence_config::Sequence,
    ) -> Self {
        Self { id, name, sequence }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct SavedList {
    pub selected: usize,
    pub options: &'static [SavedChoice],
}
impl SavedList {
    pub const fn new(options: &'static [SavedChoice]) -> Self {
        Self {
            selected: 0,
            options,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct MenuPresets {
    pub lists: [SavedList; PresetSlot::COUNT],
    pub templates: &'static [crate::sequence_config::Template],
}
// Empty until startup loads the sole text catalog.
impl Default for MenuPresets {
    fn default() -> Self {
        Self::EMPTY
    }
}
impl MenuPresets {
    pub const EMPTY: Self = Self {
        lists: [SavedList::new(&[]); PresetSlot::COUNT],
        templates: &[],
    };
    pub fn list(&self, slot: PresetSlot) -> &SavedList {
        &self.lists[slot as usize]
    }
}
