use super::*;
use crate::{fighter_profiles::FighterProfiles, native_input::*, sequence_config::Catalog};
fn configured() -> MenuPresets {
    let catalog=Catalog::parse("[bowser.roll.manual]\ndefault=true\nstart_delay=0\nactions=grab\n[bowser.roll.auto]\nstart_delay=0\nreaction=auto_roll:40:12\nactions=up_b:4\n").unwrap();
    let mut profiles = FighterProfiles::default();
    profiles.add_sequence_choices(&catalog).unwrap();
    profiles.profile(15).unwrap().menu_options
}
fn tap(m: &mut MenuSelection, p: &MenuPresets, button: u32) {
    m.input(0, p);
    m.input(button, p);
}
#[test]
fn first_saved_edit_survives_editor_release_reopen_and_next_session_in_every_slot() {
    for slot in PresetSlot::ALL {
        let slot_name = ["neutral", "jump", "roll", "attack", "drop", "x", "y", "dpad_up", "dpad_down", "dpad_left", "dpad_right", "l3", "r3"][slot as usize];
        let source = format!("[bowser.{slot_name}.first]\nstart_delay=2\nactions=shield:4,wait:2,grab\n");
        let mut profiles = FighterProfiles::default();
        profiles.add_sequence_choices(&Catalog::parse(&source).unwrap()).unwrap();
        let p = profiles.profile(15).unwrap().menu_options;
        let mut m = MenuSelection::new(&p);
        let key = (slot as usize, 0);
        m.choices[slot as usize] = 0;
        let original = m.choice(&p, slot, 0).unwrap().sequence;
        let mut e = editor::RecipeEditor::default();
        e.ensure(key, original);
        // Find and change Start delay through the actual editor field handler.
        let mut changed = None;
        for cursor in 0..100 {
            e.cursor = cursor;
            e.adjust(key, true);
            if e.compiled(key).is_some_and(|s| s.start_delay == 3) {
                changed = e.compiled(key); break;
            }
            e.discard(key); e.ensure(key, original);
        }
        let saved = changed.expect("start-delay field");
        m.editor = Some(Box::new(e));
        m.mark_recipe_saved(slot, 0);
        m.complete_saved_snapshot(&p, slot, 0, false, Some(saved), None);
        m.release_training_tools(&p);
        assert!(m.editor.is_none(), "saved editor retained for {slot:?}");
        assert_eq!(m.choice(&p, slot, 0).unwrap().sequence, saved);
        let mut reopened = editor::RecipeEditor::default();
        reopened.ensure(key, m.choice(&p, slot, 0).unwrap().sequence);
        assert_eq!(reopened.compiled(key), Some(saved));
        let text = crate::sequence_config::persist::update(&source, 15, slot, "first", saved, false).unwrap();
        let mut next = MenuSelection::new(&p);
        next.load_saved_recipes(&p, &Catalog::parse(&text).unwrap(), 15);
        assert_eq!(next.recipe_snapshot(&p, slot, 0), Some(saved));
    }
}

#[test]
fn late_save_promotes_only_its_snapshot_and_preserves_newer_and_invalid_drafts() {
    let p = configured();
    let slot = PresetSlot::Roll;
    let key = (slot as usize, 0);
    let mut m = MenuSelection::new(&p);
    let mut saved = m.choice(&p, slot, 0).unwrap().sequence;
    saved.start_delay = 10;
    let mut newer = saved;
    newer.start_delay = 11;
    let mut e = editor::RecipeEditor::default();
    e.ensure(key, newer);
    m.editor = Some(Box::new(e));
    m.complete_saved_snapshot(&p, slot, 0, false, Some(saved), None);
    m.release_training_tools(&p);
    assert_eq!(m.choice(&p, slot, 0).unwrap().sequence, saved);
    assert_eq!(m.recipe_snapshot(&p, slot, 0), Some(newer));
    let invalid = crate::sequence_config::Sequence { len: 0, ..newer };
    let e = m.editor.as_mut().unwrap();
    e.discard(key); e.ensure(key, invalid);
    // Empty editor state is retained when it differs from the committed base.
    m.release_training_tools(&p);
    assert!(m.editor.is_some());
    assert_eq!(m.choice(&p, slot, 0).unwrap().sequence, saved);
}
#[test]
fn leaving_training_restores_manual_choices_and_closes_assist_editor_tools() {
    let p = configured();
    for tools in [false, true] {
        let mut m = MenuSelection::new(&p);
        m.simplified_reactions = true;
        m.y_fighter_kind = 15;
        let original = m.choices;
        m.show_modes();
        tap(&mut m, &p, MENU_STICK_CLICK);
        assert!(m.assist_rule_page());
        tap(&mut m, &p, MAPPER_A);
        assert!(m.recipe_editing);
        assert_ne!(m.choices, original);
        if tools {
            let key = (
                PresetSlot::Roll as usize,
                m.choices[PresetSlot::Roll as usize],
            );
            let e = m.editor.as_mut().unwrap();
            for cursor in 0..100 {
                e.cursor = cursor;
                e.command_input(key, MAPPER_ZL | MAPPER_ZR, MAPPER_ZL | MAPPER_ZR);
                if e.command_active() {
                    break;
                }
            }
            assert!(e.command_active());
        }
        m.preview_slot = Some(PresetSlot::Roll);
        m.input_with_editing(0, &p, false);
        assert_eq!(
            m.choices, original,
            "temporary assist editing must not become a manual choice"
        );
        assert!(!m.assist_rule_page());
        assert!(!m.mode_rule_list);
        assert!(!m.recipe_editing && !m.tuning_page && !m.managing());
        assert!(m.editor.is_none(), "unchanged editor/tools should be released for gameplay");
        assert!(m.preview_slot.is_none());
        assert!(m.cancel_pending);
        assert!(m.lightweight && m.quick_menu && !m.menu_hidden);
        assert_eq!(m.row, MenuRow::Neutral);
        let stamp = (m.revision, m.content_revision, m.recipe_revision);
        m.input_with_editing(0, &p, false);
        assert!(!m.cancel_pending);
        assert_eq!((m.revision, m.content_revision, m.recipe_revision), stamp);
    }
}
#[test]
fn training_assist_sequence_lists_back_returns_to_the_same_modes_row() {
    let p=configured();
    for row in [MenuRow::OutOfShield,MenuRow::GrabDodge] {
        let mut m=MenuSelection::new(&p);m.simplified_reactions=true;m.y_fighter_kind=15;
        m.show_modes();m.row=row;tap(&mut m,&p,MENU_STICK_CLICK);
        assert!(m.mode_rule_list && !m.tuning_page);
        tap(&mut m,&p,crate::native_input::MAPPER_B);
        assert!(!m.tuning_page && !m.recipe_editing && !m.mode_rule_list);
        assert!(m.modes_menu);assert_eq!(m.row,row);
    }
}

#[test]
fn lightweight_production_pages_navigate_all_registered_fighters_without_editor_or_retired_rows() {
    let profiles = FighterProfiles::default();
    let mut checked = 0;
    for kind in 0..0x5e {
        let Some(profile) = profiles.profile(kind) else {
            continue;
        };
        checked += 1;
        let p = &profile.menu_options;
        let mut m = MenuSelection::new(p);
        m.simplified_reactions = true;
        m.y_fighter_kind = kind;
        m.input_with_editing(0, p, false);
        for buttons_page in [false, true] {
            if buttons_page {
                m.input_with_editing(0, p, false);
                m.input_with_editing(MENU_STICK_CLICK, p, false);
            }
            let visible: Vec<_> = m
                .display_rows()
                .iter()
                .copied()
                .filter(|r| m.row_visible(*r))
                .collect();
            assert!(!visible.is_empty());
            for row in [
                MenuRow::RollRead,
                MenuRow::YMode,
                MenuRow::JabGrab,
                MenuRow::ShieldPressure,
                MenuRow::SpotPunish,
            ] {
                assert!(!m.row_visible(row));
            }
            let mut visited = std::collections::BTreeSet::new();
            for _ in 0..visible.len() {
                assert!(visible.contains(&m.row));
                visited.insert(m.row as usize);
                m.input_with_editing(0, p, false);
                m.input_with_editing(MAPPER_L | DPAD_DOWN, p, false);
            }
            assert_eq!(
                visited.len(),
                visible.len(),
                "kind {kind}, buttons={buttons_page}"
            );
            assert!(m.editor.is_none() && m.catalog.is_none());
            assert!(!m.save_requested && !m.delete_requested && !m.tuning_save_requested);
        }
    }
    assert_eq!(checked, 79);
}

#[test]
fn leaving_training_rule_list_does_not_allocate_an_editor_or_leave_hidden_rule_rows() {
    let p = configured();
    let mut m = MenuSelection::new(&p);
    m.simplified_reactions = true;
    m.y_fighter_kind = 15;
    m.show_modes();
    tap(&mut m, &p, MENU_STICK_CLICK);
    assert!(m.mode_rule_list);
    assert!(m.editor.is_none());
    m.input_with_editing(0, &p, false);
    assert!(!m.mode_rule_list && !m.assist_rule_page());
    assert!(m.editor.is_none());
    assert_eq!(m.row, MenuRow::Neutral);
    let count = m
        .display_rows()
        .iter()
        .filter(|r| m.row_visible(**r))
        .count();
    assert_eq!(count, 5);
}

// Exercise real controller routing rather than calling editor operations directly.
fn audit_editor(p: &MenuPresets) -> MenuSelection {
    let mut m=MenuSelection::new(p);m.show();m.row=MenuRow::Roll;tap(&mut m,p,MENU_STICK_CLICK);
    assert!(m.recipe_editing && !m.quick_menu);m
}
fn audit_tools(m:&mut MenuSelection,p:&MenuPresets) {
    tap(m,p,MAPPER_ZL|MAPPER_ZR);assert!(m.editor.as_ref().unwrap().command_active());
}
fn audit_manage(m:&mut MenuSelection,p:&MenuPresets) {
    audit_tools(m,p);tap(m,p,DPAD_UP);tap(m,p,MAPPER_A);assert!(m.managing());
}
#[test]
fn editor_controller_back_done_save_preview_and_manager_are_consistent() {
    let p=configured();let mut m=audit_editor(&p);
    audit_tools(&mut m,&p);tap(&mut m,&p,MAPPER_A); // action picker
    tap(&mut m,&p,MAPPER_B);assert!(m.editor.as_ref().unwrap().command_active()); // tools
    tap(&mut m,&p,MENU_STICK_CLICK);assert!(m.quick_menu && !m.recipe_editing);
    assert!(!m.editor.as_ref().unwrap().command_active());
    tap(&mut m,&p,MENU_STICK_CLICK);audit_tools(&mut m,&p);tap(&mut m,&p,MAPPER_Y);
    assert!(m.save_requested && m.recipe_editing);assert!(!m.editor.as_ref().unwrap().command_active());
    m.save_requested=false;audit_tools(&mut m,&p);tap(&mut m,&p,MAPPER_X);
    assert!(m.preview_slot.is_some());assert!(!m.editor.as_ref().unwrap().command_active());
    audit_manage(&mut m,&p);tap(&mut m,&p,MAPPER_B);
    assert!(!m.managing() && m.recipe_editing);
    tap(&mut m,&p,MAPPER_B);assert!(m.quick_menu && !m.recipe_editing);
}
#[test]
fn tools_recipe_options_support_rename_duplicate_add_and_delete_through_controller_input() {
    let p=configured();let mut m=audit_editor(&p);let slot=m.row.slot().unwrap();
    let old=m.choices[slot as usize];let original=m.choice(&p,slot,old).unwrap().sequence;
    audit_manage(&mut m,&p);tap(&mut m,&p,MAPPER_A); // rename
    tap(&mut m,&p,DPAD_DOWN);tap(&mut m,&p,MAPPER_Y);
    assert!(m.name_override(slot,old).is_some());assert!(!m.save_requested);
    audit_manage(&mut m,&p);for _ in 0..2 {tap(&mut m,&p,DPAD_DOWN);}tap(&mut m,&p,MAPPER_A);
    let duplicate=m.choices[slot as usize];assert_ne!(old,duplicate);
    assert_eq!(m.recipe_snapshot(&p,slot,duplicate),Some(original));
    audit_manage(&mut m,&p);for _ in 0..3 {tap(&mut m,&p,DPAD_DOWN);}tap(&mut m,&p,MAPPER_A);
    tap(&mut m,&p,MAPPER_A);assert!(m.delete_requested);
    m.complete_file_save(&p,slot,duplicate,true);assert!(m.choice(&p,slot,duplicate).is_none());
    assert_eq!(m.recipe_snapshot(&p,slot,old),Some(original));
    tap(&mut m,&p,MENU_STICK_CLICK); // edit currently selected recipe again
    assert!(m.recipe_editing);audit_manage(&mut m,&p);tap(&mut m,&p,DPAD_DOWN);tap(&mut m,&p,MAPPER_A);
    let blank=m.choices[slot as usize];assert_ne!(blank,old);
    assert!(m.choice(&p,slot,blank).unwrap().allow_new);
    assert!(m.recipe_snapshot(&p,slot,blank).unwrap().actions().iter().all(|a|*a==crate::sequence_config::Action::Empty));
}
#[test]
fn incomplete_editor_rule_cannot_save_or_test_and_only_suspends_its_own_reaction() {
    use crate::{reaction_rules::Trigger,sequence_config::{Action,Sequence}};
    let c=Catalog::parse("[bowser.roll.first]\nreaction=auto_roll:40:12\nstart_delay=0\nactions=up_b:4\n[bowser.attack.second]\nreaction=auto_spot_dodge:35:12\nstart_delay=0\nactions=up_b:4\n").unwrap();
    let mut profiles=FighterProfiles::default();profiles.add_sequence_choices(&c).unwrap();let p=profiles.profile(15).unwrap().menu_options;
    let mut m=MenuSelection::new(&p);m.show();m.row=MenuRow::Roll;m.choices[PresetSlot::Roll as usize]=0;
    tap(&mut m,&p,MENU_STICK_CLICK);let key=(PresetSlot::Roll as usize,0);
    let source=m.recipe_snapshot(&p,PresetSlot::Roll,0).unwrap();assert_eq!(source.reaction.trigger,Trigger::AutoRoll);
    let mut invalid=Sequence{len:1,..source};invalid.actions[0]=Action::IfFacingOpponent{away:false};
    let e=m.editor.as_mut().unwrap();e.discard(key);e.ensure(key,invalid);
    let rules=m.resolve_reactions(&p).unwrap();assert!(rules.simple_sequence(0).is_none());assert!(rules.simple_sequence(1).is_some());
    audit_tools(&mut m,&p);tap(&mut m,&p,MAPPER_Y);assert!(!m.save_requested);assert!(m.save_status.as_ref().unwrap().starts_with("Cannot save:"));
    audit_tools(&mut m,&p);tap(&mut m,&p,MAPPER_X);assert!(m.preview_slot.is_none());assert!(m.save_status.as_ref().unwrap().starts_with("Cannot test:"));
    assert_eq!(m.editor.as_ref().unwrap().compiled(key),Some(invalid));
}
#[test]
fn editor_idle_and_ignored_diagonals_do_not_rebuild_text_or_change_recipes() {
    let p=configured();let mut m=audit_editor(&p);audit_tools(&mut m,&p);m.input(0,&p);
    let revision=(m.revision,m.content_revision,m.recipe_revision);
    for _ in 0..120 {m.input(0,&p);}
    m.input(DPAD_UP|DPAD_RIGHT,&p);
    assert_eq!((m.revision,m.content_revision,m.recipe_revision),revision);
}
#[test]
fn lightweight_held_row_scroll_reuses_content_and_never_allocates_an_editor() {
    let p=configured();let mut m=MenuSelection::new(&p);m.input_with_editing(0,&p,false);
    m.input_with_editing(0,&p,false);m.input_with_editing(MAPPER_L | DPAD_DOWN,&p,false);let first=m.row;
    let content=m.content_revision;for _ in 0..24 {m.input_with_editing(MAPPER_L | DPAD_DOWN,&p,false);}
    assert_ne!(m.row,first);assert_eq!(m.content_revision,content);assert!(m.editor.is_none());
    m.input_with_editing(0,&p,false);m.row=MenuRow::Neutral;
    m.input_with_editing(MAPPER_L | DPAD_RIGHT,&p,false);let choices=m.choices;
    for _ in 0..120 {m.input_with_editing(MAPPER_L | DPAD_RIGHT,&p,false);}
    assert_eq!(m.choices,choices);
    m.input_with_editing(0,&p,false);
    assert_eq!(m.input_with_editing(MAPPER_A|MAPPER_B,&p,false)&(MAPPER_A|MAPPER_B),0);
    assert!(m.editor.is_none() && !m.managing() && !m.save_requested);
}
#[test]
fn training_page_links_open_with_confirm_without_changing_recipes() {
    let p=configured();let mut m=MenuSelection::new(&p);m.show();let choices=m.choices;
    m.row=MenuRow::ButtonsPage;tap(&mut m,&p,MAPPER_A);assert!(m.button_menu && m.quick_menu);
    m.row=MenuRow::LedgePage;tap(&mut m,&p,MAPPER_A);assert!(!m.button_menu && m.quick_menu);
    assert_eq!(m.choices,choices);assert!(m.editor.is_none());
}

#[test]
fn invalid_draft_does_not_reserve_a_gameplay_toggle_from_its_saved_rule() {
    use crate::{reaction_rules::{Control,Trigger},sequence_config::Action};
    let p=configured();let mut m=MenuSelection::new(&p);let key=(PresetSlot::Roll as usize,1);
    let mut draft=m.choice(&p,PresetSlot::Roll,1).unwrap().sequence;
    draft.reaction.trigger=Trigger::Roll;draft.reaction.control=Control::ToggleDown;
    draft.actions[0]=Action::IfFacingOpponent{away:false};draft.len=1;
    m.editor.get_or_insert_with(Default::default).ensure(key,draft);
    m.refresh_rule_bindings(&p,0);assert_eq!(m.rule_button_mask&DPAD_DOWN,0);
}

#[test]
fn generic_template_is_named_selectable_undoable_and_never_changes_its_source() {
 let c=Catalog::parse("[template.short_shield]\nname=Quick shield\nstart_delay=2\nactions=input:zr:0:0:4\n[kirby.neutral.current]\nname=Keep my name\nstart_delay=7\nactions=jab:3\n").unwrap();
 let mut profiles=FighterProfiles::default();profiles.add_sequence_choices(&c).unwrap();
 let p=profiles.profile(6).unwrap().menu_options;
 assert_eq!(p.templates.as_ptr(),profiles.profile(5).unwrap().menu_options.templates.as_ptr());
 assert_eq!(p.list(PresetSlot::Neutral).options.len(),1,"templates are never executable options");
 let mut m=MenuSelection::new(&p);m.show();m.row=MenuRow::Neutral;tap(&mut m,&p,MENU_STICK_CLICK);
 let original=m.recipe_snapshot(&p,PresetSlot::Neutral,0).unwrap();
 audit_tools(&mut m,&p);for _ in 0..8 {tap(&mut m,&p,DPAD_DOWN);}tap(&mut m,&p,MAPPER_A);
 tap(&mut m,&p,DPAD_LEFT);let panel=m.recipe_panel(&p);
 assert!(panel.header.iter().any(|s|s.contains("Generic templates")));
 assert!(panel.body.iter().any(|s|s.contains("Quick shield")));
 tap(&mut m,&p,MAPPER_A);
 let copied=m.recipe_snapshot(&p,PresetSlot::Neutral,0).unwrap();assert_eq!(copied,c.templates[0].sequence);
 assert_eq!(m.choice(&p,PresetSlot::Neutral,0).unwrap().name,"Keep my name");
 assert_eq!(p.templates[0].sequence,c.templates[0].sequence);
 audit_tools(&mut m,&p);for _ in 0..7 {tap(&mut m,&p,DPAD_DOWN);}tap(&mut m,&p,MAPPER_A);
 assert_eq!(m.recipe_snapshot(&p,PresetSlot::Neutral,0),Some(original));
}
