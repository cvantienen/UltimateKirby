//! Lazy bounded drafts and controller-operated list management.
use super::*;
use crate::sequence_config::Sequence;
use std::collections::BTreeMap;

const UTF8_CONTINUATION_MASK: u8 = 0xc0;
const UTF8_CONTINUATION_PREFIX: u8 = 0x80;

#[derive(Clone)]
pub(super) struct Draft {
    pub id: String,
    pub name: String,
    pub sequence: Sequence,
    pub new: bool,
    pub deleted: bool,
    pub dirty: bool,
}
#[derive(Default)]
pub(super) enum Mode {
    #[default]
    Closed,
    Actions(usize),
    Name {
        text: Vec<u8>,
        cursor: usize,
    },
    Delete,
}
#[derive(Default)]
pub(super) struct Catalog {
    pub drafts: BTreeMap<(usize, usize), Draft>,
    pub mode: Mode,
    pub revision: u64,
    pub assist: bool,
    held: u32,
    ticks: u16,
}
pub(super) enum Event {
    None,
    Rename(String),
    Add(bool),
    Delete,
    MakeDefault,
}
// Cursors are UTF-8 byte boundaries; saved recipe names allow 64 bytes.
fn next_character(text: &[u8], cursor: usize) -> usize {
    let mut end = cursor + 1;
    while end < text.len() && text[end] & UTF8_CONTINUATION_MASK == UTF8_CONTINUATION_PREFIX {
        end += 1;
    }
    end
}
fn previous_character(text: &[u8], cursor: usize) -> usize {
    let mut start = if cursor == 0 { text.len() } else { cursor } - 1;
    while start > 0 && text[start] & UTF8_CONTINUATION_MASK == UTF8_CONTINUATION_PREFIX {
        start -= 1;
    }
    start
}
impl Catalog {
    pub fn active(&self) -> bool {
        !matches!(self.mode, Mode::Closed)
    }
    pub fn close(&mut self) {
        self.mode = Mode::Closed;
        self.revision = self.revision.wrapping_add(1);
        self.held = 0;
        self.ticks = 0;
    }
    pub fn input(&mut self, buttons: u32, pressed: u32, name: &str) -> Event {
        use crate::native_input::{MAPPER_A as A, MAPPER_B as B, MAPPER_X as X, MAPPER_Y as Y};
        let pad = buttons & (DPAD_SIDES | DPAD_UP | DPAD_DOWN);
        let mut repeat = false;
        if matches!(self.mode, Mode::Name { .. }) && pad.count_ones() == 1 {
            if self.held == pad {
                self.ticks = self.ticks.saturating_add(1);
                repeat = self.ticks >= 24 && (self.ticks - 24) % 6 == 0;
            } else {
                self.held = pad;
                self.ticks = 0;
            }
        } else {
            self.held = 0;
            self.ticks = 0;
        }
        if pressed & (A | B | X | Y | DPAD_SIDES | DPAD_UP | DPAD_DOWN) != 0 || repeat {
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
                            let mut value =
                                name.chars().filter(|c| !c.is_control()).collect::<String>();
                            if value.len() > 64 {
                                let mut end = 64;
                                while !value.is_char_boundary(end) {
                                    end -= 1;
                                }
                                value.truncate(end);
                            }
                            let text = value.into_bytes();
                            self.mode = Mode::Name {
                                text: if text.is_empty() { vec![b'A'] } else { text },
                                cursor: 0,
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
            Mode::Name { text, cursor } => {
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
                } else if pressed & X == 0
                    && pad.count_ones() == 1
                    && (pressed & pad != 0 || repeat)
                {
                    if pad == DPAD_RIGHT {
                        *cursor = next_character(text, *cursor) % text.len();
                    } else if pad & DPAD_SIDES != 0 {
                        *cursor = previous_character(text, *cursor);
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
            Mode::Name { text, cursor } => vec![
                "RENAME OPTION".into(),
                format!(
                    "{}[{}]{}",
                    String::from_utf8_lossy(&text[..*cursor]),
                    String::from_utf8_lossy(&text[*cursor..next_character(text, *cursor)]),
                    String::from_utf8_lossy(&text[next_character(text, *cursor)..])
                ),
                "Left/Right: cursor | Up/Down: letter (hold)".into(),
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
const ALPHABET: &[u8] = b" ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_/().+%";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        fighter_profiles::FighterProfiles,
        native_input::{MAPPER_A as A, MAPPER_B as B, MAPPER_X, MAPPER_Y as Y},
        sequence_config::Catalog as FileCatalog,
    };
    fn presets() -> MenuPresets {
        let file=FileCatalog::parse("[yoshi.neutral.alpha]\nname=Alpha\nstart_delay=0\nactions=a\n[yoshi.neutral.beta]\nname=Beta\nstart_delay=0\nactions=b\n").unwrap();
        let mut profiles = FighterProfiles::default();
        profiles.add_sequence_choices(&file).unwrap();
        profiles.profile(5).unwrap().menu_options
    }
    fn tap(menu: &mut MenuSelection, p: &MenuPresets, button: u32) {
        menu.input(button, p);
        menu.input(0, p);
    }
    fn menu(p: &MenuPresets) -> MenuSelection {
        let mut m = MenuSelection::new(p);
        m.show();
        m.input(0, p);
        m
    }
    #[test]
    fn added_unsaved_recipes_can_be_switched_from_compact() {
        let p = presets();
        let mut m = menu(&p);
        m.row = MenuRow::Neutral;
        tap(&mut m, &p, MENU_STICK_CLICK);
        // Add using the same events as B -> Add -> A.
        let base_count = m.choice_position(&p, PresetSlot::Neutral).1;
        m.manage_input(
            &p,
            crate::native_input::MAPPER_B,
            crate::native_input::MAPPER_B,
        );
        m.catalog.as_mut().unwrap().mode = Mode::Actions(1);
        m.manage_input(
            &p,
            crate::native_input::MAPPER_A,
            crate::native_input::MAPPER_A,
        );
        let added = m.choices[PresetSlot::Neutral as usize];
        assert!(m.choice(&p, PresetSlot::Neutral, added).unwrap().allow_new);
        m.show_quick();
        let count = m.choice_position(&p, PresetSlot::Neutral).1;
        // The untouched blank template is hidden; the explicit new draft is visible.
        assert_eq!(count, base_count);
        let next = m.cycle_quick_choice(&p, PresetSlot::Neutral, true);
        assert_ne!(next, added);
        m.choices[PresetSlot::Neutral as usize] = next;
        assert_eq!(m.cycle_quick_choice(&p, PresetSlot::Neutral, false), added);
    }
    #[test]
    fn diagnostic_add_and_duplicate_keep_other_option_indices_and_edits_stable() {
        let p = presets();
        let mut m = menu(&p);
        tap(&mut m, &p, B);
        tap(&mut m, &p, DPAD_DOWN);
        m.input(A, &p);
        assert!(m.recipe_editing);
        assert!(!m.managing());
        assert_eq!(m.choices[0], 3);
        assert_ne!(m.input(A, &p) & A, 0); // held Select cannot become a gameplay attack
        m.input(0, &p);
        tap(&mut m, &p, DPAD_DOWN);
        tap(&mut m, &p, DPAD_RIGHT);
        let first_new = m.choices[0];
        let configured = m.settings(&p, AssistSettings::default()).sequence_overrides[0].unwrap();
        assert!(matches!(
            configured.actions(),
            [crate::sequence_config::Action::Input {
                buttons: MAPPER_X,
                ..
            }]
        ));
        m.manage_input(&p, B, B);
        tap(&mut m, &p, DPAD_DOWN);
        tap(&mut m, &p, DPAD_DOWN);
        tap(&mut m, &p, A);
        assert_eq!(m.choices[0], 4);
        assert_eq!(
            m.settings(&p, AssistSettings::default()).sequence_overrides[0]
                .unwrap()
                .actions(),
            configured.actions()
        );
        assert_ne!(
            m.choice(&p, PresetSlot::Neutral, first_new).unwrap().id,
            m.choice(&p, PresetSlot::Neutral, 4).unwrap().id
        );
        assert_eq!(m.choice(&p, PresetSlot::Neutral, 0).unwrap().id, "alpha");
        assert_eq!(m.choice(&p, PresetSlot::Neutral, 1).unwrap().id, "beta");
    }
    #[test]
    fn diagnostic_rename_changes_label_not_id_and_requires_explicit_file_save() {
        let p = presets();
        let mut m = menu(&p);
        tap(&mut m, &p, B);
        tap(&mut m, &p, A);
        tap(&mut m, &p, DPAD_DOWN);
        tap(&mut m, &p, Y);
        assert!(m.recipe_editing);
        assert_eq!(m.choice(&p, PresetSlot::Neutral, 0).unwrap().name, "Blpha");
        assert_eq!(m.choice(&p, PresetSlot::Neutral, 0).unwrap().id, "alpha");
        assert!(!m.save_requested);
        m.input(Y, &p);
        assert!(m.save_requested);
        assert!(!m.finisher_held(Y));
    }
    #[test]
    fn diagnostic_delete_needs_confirmation_and_only_hides_after_successful_file_write() {
        let p = presets();
        let mut m = menu(&p);
        tap(&mut m, &p, B);
        for _ in 0..3 {
            tap(&mut m, &p, DPAD_DOWN);
        }
        tap(&mut m, &p, A);
        assert!(!m.delete_requested);
        assert!(m.managing());
        m.input(A, &p);
        assert!(m.delete_requested);
        assert!(m.choice(&p, PresetSlot::Neutral, 0).is_some());
        m.complete_file_save(&p, PresetSlot::Neutral, 0, true);
        assert!(m.choice(&p, PresetSlot::Neutral, 0).is_none());
        assert_eq!(m.choice(&p, PresetSlot::Neutral, 1).unwrap().id, "beta");
        assert_ne!(m.choices[0], 0);
        assert_eq!(m.choice_position(&p, PresetSlot::Neutral).1, 2);
    }
    #[test]
    fn closing_or_match_reset_exits_manager_without_deleting_or_renaming() {
        let p = presets();
        let mut m = menu(&p);
        tap(&mut m, &p, B);
        assert!(m.managing());
        m.reset_inputs();
        assert!(!m.managing());
        assert!(!m.delete_requested);
        assert_eq!(m.choice(&p, PresetSlot::Neutral, 0).unwrap().name, "Alpha");
    }
    #[test]
    fn empty_ness_and_falcon_catalogs_can_create_edit_preview_and_queue_a_recipe() {
        use crate::{
            gameplay::{GameplayContext, Opponent},
            ledge_trap::LedgeTrap,
        };
        for kind in [10, 11] {
            let profiles = crate::fighter_profiles::FighterProfiles::default();
            let profile = profiles.profile(kind).unwrap();
            let presets = &profile.menu_options;
            assert!(presets.list(PresetSlot::Neutral).options.is_empty());
            let base = profile.apply(AssistSettings {
                enabled: true,
                ..AssistSettings::default()
            });
            assert!(base.enabled);
            let mut m = menu(presets);
            m.auto_ledge = false;
            tap(&mut m, presets, B);
            tap(&mut m, presets, DPAD_DOWN);
            tap(&mut m, presets, A); // add a distinct editable draft
            tap(&mut m, presets, DPAD_DOWN);
            tap(&mut m, presets, DPAD_RIGHT); // Empty -> Short hop
            let index = m.choices[0];
            let choice = m.choice(presets, PresetSlot::Neutral, index).unwrap();
            assert!(choice.allow_new);
            assert!(choice.name.starts_with("New option"));
            assert!(
                !m.ledge_mode(),
                "availability does not enable automatic mode"
            );
            tap(&mut m, presets, crate::native_input::MAPPER_X); // explicit Test
            assert_eq!(m.preview_slot, Some(PresetSlot::Neutral));
            let settings = m.settings(presets, base);
            assert!(
                settings.enabled,
                "explicit test cannot depend on existing file rows"
            );
            let mut trap = LedgeTrap::default();
            trap.request_preview(PresetSlot::Neutral);
            let out = trap.sample(
                settings,
                0,
                GameplayContext {
                    gameplay: true,
                    held: true,
                    facing: 1.,
                    motion_rate: 1.,
                    motion_kind: crate::timing::motion_hash("wait"),
                    ..GameplayContext::default()
                },
                Some(Opponent {
                    id: 2,
                    ..Opponent::default()
                }),
            );
            assert_ne!(out.requested & crate::native_input::MAPPER_X, 0);
            m.preview_slot = None;
            m.input(crate::native_input::MAPPER_Y, presets);
            assert!(m.save_requested);
            assert!(!m.sequence_enabled(false));
        }
    }

    #[test]
    fn saved_custom_can_be_renamed_saved_again_and_deleted_without_restart() {
        use crate::sequence_config::persist;
        let source = "[yoshi.neutral.alpha]\nname=Alpha\nstart_delay=0\nactions=a\n[yoshi.neutral.beta]\nname=Beta\nstart_delay=0\nactions=b\n";
        let p = presets();
        let mut m = menu(&p);
        let slot = PresetSlot::Neutral;
        let index = p.list(slot).options.len(); // built-in Custom template
        m.choices[slot as usize] = index;
        tap(&mut m, &p, MENU_STICK_CLICK);
        tap(&mut m, &p, DPAD_DOWN);
        tap(&mut m, &p, DPAD_RIGHT); // Short hop
        let sequence = m.settings(&p, AssistSettings::default()).sequence_overrides[0].unwrap();
        let added =
            persist::update_named(source, 5, slot, "menu_custom", sequence, true, None).unwrap();
        m.complete_file_save(&p, slot, index, false);
        assert!(!m.choice(&p, slot, index).unwrap().allow_new);
        m.manage_input(&p, B, B);
        tap(&mut m, &p, A); // rename
        tap(&mut m, &p, DPAD_DOWN); // change first letter
        tap(&mut m, &p, Y); // accept name, no automatic disk write
        assert!(!m.save_requested);
        let choice = m.choice(&p, slot, index).unwrap();
        assert_eq!(choice.id, "menu_custom");
        let renamed = persist::update_named(
            &added,
            5,
            slot,
            choice.id,
            sequence,
            choice.allow_new,
            m.name_override(slot, index),
        )
        .unwrap();
        let parsed = FileCatalog::parse(&renamed).unwrap();
        let entry = parsed
            .entries()
            .iter()
            .find(|e| e.choice == "menu_custom")
            .unwrap();
        assert_eq!(entry.sequence.actions(), sequence.actions());
        assert_eq!(entry.name.as_deref(), m.name_override(slot, index));
        m.complete_file_save(&p, slot, index, false);
        m.manage_input(&p, B, B);
        for _ in 0..3 {
            tap(&mut m, &p, DPAD_DOWN);
        }
        tap(&mut m, &p, A); // ask to delete
        m.input(A, &p);
        assert!(
            m.delete_requested,
            "saved Custom is a real recipe, not an undeletable template"
        );
        let deleted = persist::delete(&renamed, 5, slot, "menu_custom", false).unwrap();
        assert!(
            deleted.starts_with(source),
            "unrelated choices remain byte-for-byte intact"
        );
        assert!(deleted[source.len()..].trim().is_empty());
        m.complete_file_save(&p, slot, index, true);
        let blank = m.choice(&p, slot, index).unwrap();
        assert!(blank.allow_new);
        assert_eq!(blank.id, "menu_custom");
        assert!(m.settings(&p, AssistSettings::default()).sequence_overrides[0].is_none());
    }

    #[test]
    fn saving_one_recipe_is_not_stale_after_renaming_another() {
        let p = presets();
        let mut m = menu(&p);
        let slot = PresetSlot::Neutral;
        let sequence = m.choice(&p, slot, 0).unwrap().sequence;
        let revision = m.recipe_revision;
        m.choices[0] = 1;
        tap(&mut m, &p, B);
        tap(&mut m, &p, A);
        tap(&mut m, &p, DPAD_DOWN);
        tap(&mut m, &p, Y);
        assert!(m.recipe_revision > revision);
        assert!(m.saved_recipe_matches(&p, slot, 0, sequence, None));
        let mut older = sequence;
        older.start_delay += 1;
        assert!(!m.saved_recipe_matches(&p, slot, 0, older, None));
    }

    #[test]
    fn deleting_only_saved_custom_restores_an_empty_reusable_template() {
        let p = MenuPresets::EMPTY;
        let mut m = menu(&p);
        let slot = PresetSlot::Neutral;
        tap(&mut m, &p, MENU_STICK_CLICK);
        tap(&mut m, &p, DPAD_DOWN);
        tap(&mut m, &p, DPAD_RIGHT);
        assert!(m.settings(&p, AssistSettings::default()).sequence_overrides[0].is_some());
        m.complete_file_save(&p, slot, 0, false);
        assert!(!m.choice(&p, slot, 0).unwrap().allow_new);
        m.complete_file_save(&p, slot, 0, true);
        assert_eq!(m.choices[0], 0);
        assert!(m.choice(&p, slot, 0).unwrap().allow_new);
        assert!(m.settings(&p, AssistSettings::default()).sequence_overrides[0].is_none());
        tap(&mut m, &p, MENU_STICK_CLICK);
        assert!(m.recipe_editing);
    }
}

#[cfg(test)]
mod startup_default_tests {
    use super::*;
    #[test]
    fn list_manager_explicit_default_requests_a_save_without_changing_the_recipe() {
        use crate::native_input::*;
        let mut profiles = crate::fighter_profiles::FighterProfiles::default();
        profiles
            .add_sequence_choices(
                &crate::sequence_config::Catalog::parse(
                    "[kirby.neutral.first]\nstart_delay=0\nactions=grab\n",
                )
                .unwrap(),
            )
            .unwrap();
        let p = profiles.profile(6).unwrap().menu_options;
        let mut m = MenuSelection::new(&p);
        m.show_quick();
        m.input(0, &p);
        m.input(MENU_STICK_CLICK, &p);
        m.input(0, &p);
        m.manage_input(&p, MAPPER_B, MAPPER_B);
        // Up wraps Rename to Save as startup default.
        m.input(0, &p);
        m.input(DPAD_UP, &p);
        m.input(0, &p);
        m.input(MAPPER_A, &p);
        assert!(m.default_requested && m.save_requested && !m.delete_requested);
        assert_eq!(
            m.choice(&p, PresetSlot::Neutral, m.choices[0]).unwrap().id,
            "first"
        );
        m.input(0, &p);
        assert!(!m.default_requested && !m.save_requested);
    }
}

#[cfg(test)]
mod rename_regressions {
    use super::*;
    use crate::native_input::{MAPPER_A as A, MAPPER_X as X, MAPPER_Y as Y};
    fn rename(name: &str) -> Catalog {
        let mut c = Catalog {
            mode: Mode::Actions(0),
            ..Default::default()
        };
        c.input(A, A, name);
        c
    }
    fn accepted(c: &mut Catalog) -> String {
        match c.input(Y, Y, "") {
            Event::Rename(n) => n,
            _ => panic!("expected a valid name"),
        }
    }
    #[test]
    fn accepting_a_unicode_name_preserves_it_and_cursor_edits_whole_characters() {
        let name = "Shield → Up-B / café";
        assert_eq!(accepted(&mut rename(name)), name);
        let mut c = rename("A→é");
        c.input(DPAD_RIGHT, DPAD_RIGHT, "");
        assert!(c.lines()[1].contains("[→]"));
        c.input(A, A, "");
        assert_eq!(accepted(&mut c), "Aé");
        let mut c = rename("é");
        c.input(A, A, "");
        assert_eq!(accepted(&mut c), "é");
    }
    #[test]
    fn rename_inserts_at_cursor_obeys_byte_limit_and_ignores_held_direction_on_confirm() {
        let mut c = rename("AB");
        c.input(X, X, "");
        assert!(c.lines()[1].contains("A[ ]B"));
        c.input(DPAD_DOWN, DPAD_DOWN, "");
        let expected = match &c.mode {
            Mode::Name { text, .. } => String::from_utf8(text.clone()).unwrap(),
            _ => panic!(),
        };
        let Event::Rename(actual) = c.input(Y | DPAD_DOWN, Y, "") else {
            panic!()
        };
        assert_eq!(actual, expected);
        let name = "é".repeat(32);
        let mut c = rename(&name);
        c.input(X, X, "");
        assert_eq!(accepted(&mut c), name);
        let mut c = rename(&"é".repeat(33));
        assert_eq!(accepted(&mut c), name);
    }
}
