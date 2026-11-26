use super::*;
use crate::{fighter_profiles::FighterProfiles, native_input::MAPPER_R};
/// Updates the `tap` state using the supplied inputs.
fn tap(m: &mut MenuSelection, p: &MenuPresets, b: u32) {
    m.input(0, p);
    m.input(b, p);
}
/// Regression test for `spot_punish_opens_its_own_list_and_saved_rule_remains_separate`.
#[test]
fn spot_punish_opens_its_own_list_and_saved_rule_remains_separate() {
    let p = MenuPresets::default();
    let mut m = MenuSelection::new(&p);
    m.show_modes();
    m.row = MenuRow::SpotPunish;
    tap(&mut m, &p, MENU_STICK_CLICK);
    assert!(m.mode_rule_list);
    tap(&mut m, &p, MAPPER_X);
    let slot = m.row.slot().unwrap();
    let i = m.choices[slot as usize];
    assert_eq!(
        m.recipe_snapshot(&p, slot, i).unwrap().reaction.trigger,
        Trigger::SpotDodge
    );
    assert!(!m.visible_choices(&p, slot).contains(&i));
    let e = m.editor.as_mut().unwrap();
    let key = (slot as usize, i);
    e.navigate(key, true);
    assert!(e.adjust(key, true));
    tap(&mut m, &p, MAPPER_Y);
    assert!(m.save_requested);
    let c = m.choice(&p, slot, i).unwrap();
    let seq = m.recipe_snapshot(&p, slot, i).unwrap();
    let saved = crate::sequence_config::persist::update_named(
        "",
        15,
        slot,
        c.id,
        seq,
        true,
        Some("Spot hop"),
    )
    .unwrap();
    let parsed = crate::sequence_config::Catalog::parse(&saved).unwrap();
    assert_eq!(
        parsed.entries()[0].sequence.reaction.trigger,
        Trigger::SpotDodge
    );
    assert_eq!(m.resolve_reactions(&p).unwrap().len, 1);
    assert!(!m.recipe_panel(&p).header.join(" ").contains("roster avg"));
}
/// Regression test for `each_assist_has_its_own_budget_and_legacy_storage_does_not_select_it_for_ledge`.
#[test]
fn each_assist_has_its_own_budget_and_legacy_storage_does_not_select_it_for_ledge() {
    let mut source = String::new();
    for trigger in ["roll", "shield_hit", "grab_dodged"] {
        for i in 0..8 {
            source.push_str(&format!("[bowser.roll.{trigger}_{i}]\nstart_delay=0\nreaction={trigger}:grounded:0:20:18:any:1\nactions=up_b:4\n"));
        }
    }
    let catalog = crate::sequence_config::Catalog::parse(&source).unwrap();
    let mut profiles = FighterProfiles::default();
    profiles.add_sequence_choices(&catalog).unwrap();
    let p = profiles.profile(15).unwrap().menu_options;
    let mut m = MenuSelection::new(&p);
    assert_eq!(m.resolve_reactions(&p).unwrap().len, 24);
    assert!(m.settings(&p, AssistSettings::default()).sequence_overrides[2].is_none());
    for row in [MenuRow::RollRules, MenuRow::OutOfShield, MenuRow::GrabDodge] {
        m.open_mode_list(&p, row);
        assert_eq!(m.mode_entries(&p, row).len(), 8);
        m.add_mode_rule(&p, row);
        assert!(m.save_status.as_ref().unwrap().contains("Eight rules"));
        m.restore_mode_choices();
    }
    source.push_str("[bowser.roll.too_many]\nstart_delay=0\nreaction=shield_hit:grounded:0:20:18:any:1\nactions=up_b:4\n");
    let bad = crate::sequence_config::Catalog::parse(&source).unwrap();
    assert!(bad.validate(&profiles).is_err());
}
/// Regression test for `rules_are_separate_sorted_and_idle_list_does_not_rebuild`.
#[test]
fn rules_are_separate_sorted_and_idle_list_does_not_rebuild() {
    let source="[bowser.roll.manual]\nstart_delay=0\ndefault=true\nactions=grab:4\n[bowser.attack.oos]\nstart_delay=0\nreaction=shield_hit:grounded:0:12:18:any:1\nactions=up_b:4\n[bowser.drop.regrab]\nstart_delay=0\nreaction=ledge_regrab:grounded:0:18:30:any:1\nactions=down_smash:4\n[bowser.roll.late]\nstart_delay=0\nreaction=roll:grounded:0:20:18:any:4\nactions=grab:4\n[bowser.attack.first]\nstart_delay=0\nreaction=roll_cross:grounded:0:12:18:any:1\nactions=up_b:4\n";
    let catalog = crate::sequence_config::Catalog::parse(source).unwrap();
    let mut profiles = FighterProfiles::default();
    profiles.add_sequence_choices(&catalog).unwrap();
    let p = profiles.profile(15).unwrap().menu_options;
    let mut m = MenuSelection::new(&p);
    for slot in PresetSlot::ALL {
        for i in m.visible_choices(&p, slot) {
            assert!(!m.recipe_snapshot(&p, slot, i).unwrap().reaction.active());
        }
    }
    assert_eq!(m.resolve_reactions(&p).unwrap().len, 4);
    assert!(m.settings(&p, AssistSettings::default()).sequence_overrides
        [PresetSlot::Attack as usize]
        .is_none());
    let original = m.choices;
    m.show_modes();
    m.row = MenuRow::RollRules;
    tap(&mut m, &p, MENU_L3);
    let panel = m.recipe_panel(&p);
    assert!(panel.body[0].starts_with("P1"));
    assert!(panel.body[1].starts_with("P4"));
    let revision = m.revision;
    for _ in 0..60 {
        m.input(0, &p);
    }
    assert_eq!(m.revision, revision);
    tap(&mut m, &p, crate::native_input::MAPPER_A);
    assert!(m.recipe_editing);
    assert!(m
        .recipe_snapshot(&p, PresetSlot::Attack, m.choices[3])
        .unwrap()
        .reaction
        .active());
    assert!(m.settings(&p, AssistSettings::default()).sequence_overrides[3].is_none());
    m.preview_slot = Some(PresetSlot::Attack);
    assert!(m.settings(&p, AssistSettings::default()).sequence_overrides[3].is_some());
    m.preview_slot = None;
    assert!(m.settings(&p, AssistSettings::default()).sequence_overrides[3].is_none());
    assert!(!m.recipe_panel(&p).header.join(" ").contains("roster avg"));
    tap(&mut m, &p, MENU_STICK_CLICK);
    assert!(m.mode_rule_list);
    tap(&mut m, &p, MENU_L3);
    assert_eq!(m.choices, original);
    assert!(m.modes_menu);
}
/// Constructs the oos rule and duplicate keep their family and never enter ledge choices.
#[test]
fn new_oos_rule_and_duplicate_keep_their_family_and_never_enter_ledge_choices() {
    let p = MenuPresets::default();
    let mut m = MenuSelection::new(&p);
    m.show_modes();
    m.row = MenuRow::OutOfShield;
    tap(&mut m, &p, MENU_L3);
    tap(&mut m, &p, MAPPER_X);
    let i = m.choices[PresetSlot::Attack as usize];
    assert_eq!(
        m.recipe_snapshot(&p, PresetSlot::Attack, i)
            .unwrap()
            .reaction
            .trigger,
        Trigger::ShieldHit
    );
    assert!(!m.visible_choices(&p, PresetSlot::Attack).contains(&i));
    let key = (PresetSlot::Attack as usize, i);
    let e = m.editor.as_mut().unwrap();
    e.navigate(key, true);
    assert!(e.adjust(key, true));
    assert!(m
        .recipe_snapshot(&p, PresetSlot::Attack, i)
        .unwrap()
        .actions()
        .iter()
        .any(|a| *a != crate::sequence_config::Action::Empty));
    assert!(!m
        .recipe_panel(&p)
        .footer
        .join(" ")
        .contains("R3: Back to modes"));
}
/// Regression test for `r3_edits_menu_responses_cycles_them_and_restores_manual_choices`.
#[test]
fn r3_edits_menu_responses_cycles_them_and_restores_manual_choices() {
    let text="[bowser.roll.manual]\nstart_delay=0\ndefault=true\nactions=grab:4\n[bowser.roll.first]\nstart_delay=0\nreaction=roll:grounded:0:20:18:any:1\nactions=down_smash:4\n[bowser.attack.second]\nstart_delay=0\nreaction=roll_cross:grounded:0:20:18:any:2\nactions=up_b:4\n";
    let mut profiles = FighterProfiles::default();
    profiles
        .add_sequence_choices(&crate::sequence_config::Catalog::parse(text).unwrap())
        .unwrap();
    let p = profiles.profile(15).unwrap().menu_options;
    let mut m = MenuSelection::new(&p);
    m.show();
    m.modes_menu = true;
    m.row = MenuRow::RollRules;
    let original = m.choices;
    tap(&mut m, &p, MENU_STICK_CLICK);
    assert!(m.mode_rule_list);
    tap(&mut m, &p, crate::native_input::MAPPER_A);
    assert!(m.recipe_editing);
    assert!(m.roll_rules);
    assert_eq!(m.row, MenuRow::Roll);
    tap(&mut m, &p, MAPPER_R | DPAD_RIGHT);
    assert_eq!(m.row, MenuRow::Attack);
    tap(&mut m, &p, MENU_STICK_CLICK);
    assert!(m.mode_rule_list);
    tap(&mut m, &p, MENU_L3);
    assert!(m.modes_menu);
    assert_eq!(m.row, MenuRow::RollRules);
    assert_eq!(m.choices, original);
}
/// Regression test for `empty_mode_creates_tagged_draft_and_minus_preserves_manual_defaults`.
#[test]
fn empty_mode_creates_tagged_draft_and_minus_preserves_manual_defaults() {
    let p = MenuPresets::default();
    let mut m = MenuSelection::new(&p);
    m.show();
    m.modes_menu = true;
    m.row = MenuRow::OutOfShield;
    let original = m.choices;
    tap(&mut m, &p, MENU_STICK_CLICK);
    assert!(m.mode_rule_list);
    tap(&mut m, &p, MAPPER_X);
    assert!(m.recipe_editing && m.out_of_shield);
    let slot = m.row.slot().unwrap();
    let c = m.choice(&p, slot, m.choices[slot as usize]).unwrap();
    assert!(c.allow_new);
    assert_eq!(c.sequence.reaction.trigger, Trigger::ShieldHit);
    tap(&mut m, &p, MAPPER_MINUS);
    assert!(m.menu_hidden);
    assert_eq!(m.choices, original);
}
