//! Lazy bounded drafts and controller-operated list management.
use super::*;
use crate::sequence_config::Sequence;
use std::collections::BTreeMap;

/// `UTF8_CONTINUATION_MASK` is a configuration value used by the Smash runtime.
const UTF8_CONTINUATION_MASK: u8 = 0xc0;
/// `UTF8_CONTINUATION_PREFIX` is a configuration value used by the Smash runtime.
const UTF8_CONTINUATION_PREFIX: u8 = 0x80;

/// Runtime Smash data carried by `Draft`.
#[derive(Clone, Default)]
pub(super) struct Draft {
    /// battle-object ID.
    pub id: String,
    /// player-facing move/fighter name.
    pub name: String,
    /// Runtime sequence value used by Draft..
    pub sequence: Sequence,
    /// Runtime new value used by Draft..
    pub new: bool,
    /// Runtime deleted value used by Draft..
    pub deleted: bool,
    /// Runtime dirty value used by Draft..
    pub dirty: bool,
}
impl Draft {
    /// A deleted draft or a legacy mode binding is not a selectable recipe.
    pub fn live(&self) -> bool {
        !self.deleted && self.sequence.mode.is_none()
    }
}
/// Available `Mode` choices for the Smash assistance menu.
#[derive(Default)]
pub(super) enum Mode {
    #[default]
    /// `Closed` option in the `Mode` Smash policy state.
    Closed,
    /// `Actions` option in the `Mode` Smash policy state.
    Actions(
        /// Carries the number of actions in the selected mode in `Mode`.
        usize,
    ),
    /// `Name` option in the `Mode` Smash policy state.
    Name {
        /// Runtime text value used by Mode..
        text: Vec<u8>,
        /// Runtime cursor value used by Mode..
        cursor: usize,
        /// Active rename picker.
        picker: NamePicker,
    },
    /// `Delete` option in the `Mode` Smash policy state.
    Delete,
}
/// Rename picker available from the sequence list manager.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum NamePicker {
    /// Edit one character at a time from the alphabet.
    #[default]
    Alphabet,
    /// Replace the name with a predefined move term.
    Terms,
}
/// Runtime Smash data carried by `Catalog`.
#[derive(Default)]
pub(super) struct Catalog {
    /// Runtime drafts value used by Catalog..
    pub drafts: BTreeMap<(usize, usize), Draft>,
    /// Runtime mode value used by Catalog..
    pub mode: Mode,
    /// Runtime revision value used by Catalog..
    pub revision: u64,
    /// Runtime assist value used by Catalog..
    pub assist: bool,
    /// Direction currently held while renaming, or zero when released.
    held: u32,
    /// Consecutive ticks the current rename direction has been held.
    ticks: u16,
}
/// Available `Event` choices for the Smash assistance menu.
pub(super) enum Event {
    /// `None` option in the `Event` Smash policy state.
    None,
    /// `Rename` option in the `Event` Smash policy state.
    Rename(
        /// Carries the replacement name in `Event`.
        String,
    ),
    /// `Add` option in the `Event` Smash policy state.
    Add(
        /// Carries whether to add the entry in `Event`.
        bool,
    ),
    /// `Delete` option in the `Event` Smash policy state.
    Delete,
    /// `MakeDefault` option in the `Event` Smash policy state.
    MakeDefault,
}
// Cursors are UTF-8 byte boundaries; saved recipe names allow 64 bytes.
/// Returns the next the character.
fn next_character(text: &[u8], cursor: usize) -> usize {
    let mut end = cursor + 1;
    while end < text.len() && text[end] & UTF8_CONTINUATION_MASK == UTF8_CONTINUATION_PREFIX {
        end += 1;
    }
    end
}
/// Returns the `previous_character` result.
/// Gets the previous character result.
fn previous_character(text: &[u8], cursor: usize) -> usize {
    let mut start = if cursor == 0 { text.len() } else { cursor } - 1;
    while start > 0 && text[start] & UTF8_CONTINUATION_MASK == UTF8_CONTINUATION_PREFIX {
        start -= 1;
    }
    start
}
fn rename_seed(name: &str) -> Vec<u8> {
    let mut value = name.chars().filter(|c| !c.is_control()).collect::<String>();
    if value.len() > 64 {
        let mut end = 64;
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        value.truncate(end);
    }
    if value.is_empty() {
        vec![b'A']
    } else {
        value.into_bytes()
    }
}

impl Catalog {
    /// Returns whether the catalog is currently active (not closed).
    pub fn active(&self) -> bool {
        !matches!(self.mode, Mode::Closed)
    }
    /// Closes the catalog, resetting its mode and input state.
    pub fn close(&mut self) {
        self.mode = Mode::Closed;
        self.revision = self.revision.wrapping_add(1);
        self.held = 0;
        self.ticks = 0;
    }
    /// Processes input for the catalog, returning an event based on the current mode and input state.
    pub fn input(&mut self, buttons: u32, pressed: u32, name: &str) -> Event {
        use crate::native_input::{MAPPER_A as A, MAPPER_B as B, MAPPER_X as X, MAPPER_Y as Y};
        let pad = buttons & (DPAD_SIDES | DPAD_UP | DPAD_DOWN);
        let mut repeat = false;
        if matches!(self.mode, Mode::Name { .. }) && pad.count_ones() == 1 {
            if self.held == pad {
                self.ticks = self.ticks.saturating_add(1);
                // Renaming keeps the steady slow cadence; a fast mode would
                // overshoot single characters and predefined terms.
                repeat = self.ticks >= super::MENU_REPEAT_INITIAL_DELAY_TICKS
                    && (self.ticks - super::MENU_REPEAT_INITIAL_DELAY_TICKS)
                        .is_multiple_of(super::MENU_REPEAT_SLOW_INTERVAL_TICKS);
            } else {
                self.held = pad;
                self.ticks = 0;
            }
        } else {
            self.held = 0;
            self.ticks = 0;
        }
        if pressed & (A | B | X | Y | DPAD_SIDES | DPAD_UP | DPAD_DOWN) != 0
            || repeat
            || matches!(self.mode, Mode::Name { .. })
                && pressed & crate::native_input::MAPPER_ZL != 0
        {
            self.revision = self.revision.wrapping_add(1);
        }
        if pressed & B != 0 {
            self.mode = match self.mode {
                Mode::Closed => Mode::Actions(0),
                Mode::Actions(_) => Mode::Closed,
                _ => Mode::Actions(0),
            };
            return Event::None;
        }
        match &mut self.mode {
            Mode::Closed => {}
            Mode::Actions(selected) => {
                let count = if self.assist { 4 } else { 5 };
                *selected = (*selected).min(count - 1);
                if pressed & DPAD_DOWN != 0 {
                    *selected = (*selected + 1) % count;
                }
                if pressed & DPAD_UP != 0 {
                    *selected = (*selected + count - 1) % count;
                }
                if pressed & A != 0 {
                    match *selected {
                        0 => {
                            self.mode = Mode::Name {
                                text: rename_seed(name),
                                cursor: 0,
                                picker: NamePicker::Alphabet,
                            };
                        }
                        1 => {
                            self.close();
                            return Event::Add(false);
                        }
                        2 => {
                            self.close();
                            return Event::Add(true);
                        }
                        3 => self.mode = Mode::Delete,
                        _ => {
                            self.close();
                            return Event::MakeDefault;
                        }
                    }
                }
            }
            Mode::Name {
                text,
                cursor,
                picker,
            } => {
                // Confirm/erase/insert must not also apply a held direction.
                if pressed & Y != 0 {
                    let value = String::from_utf8_lossy(text).trim().to_owned();
                    if !value.is_empty() {
                        self.close();
                        return Event::Rename(value);
                    }
                    return Event::None;
                }
                if pressed & X != 0 && text.len() < 64 {
                    *cursor = next_character(text, *cursor);
                    text.insert(*cursor, b' ');
                } else if pressed & A != 0 {
                    let end = next_character(text, *cursor);
                    if *cursor != 0 || end != text.len() {
                        text.drain(*cursor..end);
                        if *cursor == text.len() {
                            *cursor = previous_character(text, *cursor);
                        }
                    }
                } else if pressed & crate::native_input::MAPPER_ZL != 0 {
                    *picker = match picker {
                        NamePicker::Alphabet => NamePicker::Terms,
                        NamePicker::Terms => NamePicker::Alphabet,
                    };
                } else if pressed & X == 0
                    && pad.count_ones() == 1
                    && (pressed & pad != 0 || repeat)
                {
                    if pad == DPAD_RIGHT {
                        *cursor = next_character(text, *cursor) % text.len();
                    } else if pad & DPAD_SIDES != 0 {
                        *cursor = previous_character(text, *cursor);
                    } else if pad != 0 && *picker == NamePicker::Terms {
                        let current = String::from_utf8_lossy(text);
                        let index = match NAME_TERMS.iter().position(|term| *term == current) {
                            Some(index) if pad == DPAD_DOWN => (index + 1) % NAME_TERMS.len(),
                            Some(index) => (index + NAME_TERMS.len() - 1) % NAME_TERMS.len(),
                            None if pad == DPAD_DOWN => 0,
                            None => NAME_TERMS.len() - 1,
                        };
                        text.clear();
                        text.extend_from_slice(NAME_TERMS[index].as_bytes());
                        *cursor = 0;
                    } else if pad != 0 {
                        let i = ALPHABET
                            .iter()
                            .position(|b| *b == text[*cursor])
                            .unwrap_or(0);
                        let value = ALPHABET[if pad == DPAD_DOWN {
                            (i + 1) % ALPHABET.len()
                        } else {
                            (i + ALPHABET.len() - 1) % ALPHABET.len()
                        }];
                        let end = next_character(text, *cursor);
                        text.splice(*cursor..end, [value]);
                    }
                }
            }
            Mode::Delete => {
                if pressed & A != 0 {
                    self.close();
                    return Event::Delete;
                }
            }
        }
        Event::None
    }
    /// Returns the `lines` result.
    pub fn lines(&self) -> Vec<String> {
        match &self.mode {
            Mode::Closed => vec![],
            Mode::Actions(selected) => {
                let mut lines = vec!["LIST MANAGER | A: Select   B: Back".into()];
                for (i, label) in [
                    "Rename option",
                    "Add blank option",
                    "Duplicate current option",
                    "Delete option...",
                    "Save as startup default",
                ]
                .iter()
                .take(if self.assist { 4 } else { 5 })
                .enumerate()
                {
                    lines.push(format!(
                        "{} {label}",
                        if i == *selected { ">" } else { " " }
                    ));
                }
                lines
            }
            Mode::Name {
                text,
                cursor,
                picker,
            } => vec![
                "RENAME OPTION".into(),
                format!(
                    "{}[{}]{}",
                    String::from_utf8_lossy(&text[..*cursor]),
                    String::from_utf8_lossy(&text[*cursor..next_character(text, *cursor)]),
                    String::from_utf8_lossy(&text[next_character(text, *cursor)..])
                ),
                match picker {
                    NamePicker::Alphabet => "Left/Right: cursor | Up/Down: letter (hold)",
                    NamePicker::Terms => "Up/Down: predefined term (hold)",
                }
                .into(),
                "ZL: switch alphabet / term picker".into(),
                "X: insert after cursor | A: erase character".into(),
                "Y: use name | B: cancel. Then Y: Save in editor.".into(),
            ],
            Mode::Delete => vec![
                "DELETE THIS OPTION FROM sequences.txt?".into(),
                "A: Confirm delete | B: Cancel".into(),
                "A backup is created before file replacement.".into(),
            ],
        }
    }
}
/// `ALPHABET` is a configuration value used by the Smash runtime.
const ALPHABET: &[u8] = b" ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
/// Predefined sequence-name terms offered by the rename term picker.
const NAME_TERMS: &[&str] = &[
    "F-TILT",
    "U-TILT",
    "D-TILT",
    "F-SMASH",
    "U-SMASH",
    "D-SMASH",
    "NEUTRAL-B",
    "SIDE-B",
    "UP-B",
    "DOWN-B",
    "N-AIR",
    "F-AIR",
    "B-AIR",
    "U-AIR",
    "D-AIR",
    "Z-AIR",
    "GRAB",
    "WALK",
    "RUN",
    "S-HOP",
    "F-HOP",
];

#[cfg(test)]
#[path = "../../tests/unit/settings_menu_catalog.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/unit/settings_menu_catalog_startup_default.rs"]
mod startup_default_tests;

#[cfg(test)]
#[path = "../../tests/unit/settings_menu_catalog_rename_regressions.rs"]
mod rename_regressions;
