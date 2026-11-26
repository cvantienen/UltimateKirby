use super::*;
/// Regression test for `all_auto_recipes_share_a_list_and_replace_duplicate_grab_tuning_rows`.
#[test]
fn all_auto_recipes_share_a_list_and_replace_duplicate_grab_tuning_rows() {
    let mut text = String::new();
    for t in Trigger::AUTO_EVENTS {
        text.push_str(&format!(
            "[bowser.roll.{}]\nstart_delay=0\nreaction={}:35:24\nactions={}\n",
            t.token(),
            t.token(),
            if t == Trigger::AutoShieldGrab {
                "input:r:0:0:4"
            } else {
                "up_b:4"
            }
        ));
    }
    let c = crate::sequence_config::Catalog::parse(&text).unwrap();
    let mut profiles = crate::fighter_profiles::FighterProfiles::default();
    profiles.add_sequence_choices(&c).unwrap();
    let p = profiles.profile(15).unwrap().menu_options;
    let mut m = MenuSelection::new(&p);
    m.simplified_reactions = true;
    m.y_fighter_kind = 15;
    m.show_modes();
    m.input(0, &p);
    assert_eq!(m.mode_entries(&p, MenuRow::RollRules).len(), 6);
    assert_eq!(m.mode_entries(&p, MenuRow::OutOfShield).len(), 2);
    assert_eq!(m.mode_entries(&p, MenuRow::GrabDodge).len(), 1);
    assert!(!m.tuning_visible(crate::fighter_tuning::AUTO_GRAB_CLOSE));
    assert!(!m.tuning_visible(crate::fighter_tuning::AUTO_GRAB_FAR));
    let r = m.resolve_reactions(&p).unwrap();
    assert_eq!(r.len, 6);
    for (i, t) in Trigger::AUTO_EVENTS.iter().enumerate() {
        assert_eq!(r.simple_sequence(i).unwrap().reaction.trigger, *t);
    }
    for row in [MenuRow::RollRules, MenuRow::OutOfShield, MenuRow::GrabDodge] {
        m.restore_mode_choices();
        m.show_modes();
        m.row = row;
        m.input(0, &p);
        m.input(MENU_STICK_CLICK, &p);
        assert!(m.mode_rule_list && !m.tuning_page);
        let n = m.mode_entries(&p, row).len();
        m.mode_rule_cursor = n;
        m.add_mode_rule(&p, row);
        assert_eq!(
            m.mode_entries(&p, row).len(),
            n,
            "cannot add a duplicate event"
        );
    }
}
/// Regression test for `add_missing_auto_event_stays_inactive_until_actions_are_configured`.
#[test]
fn add_missing_auto_event_stays_inactive_until_actions_are_configured() {
    let profiles = crate::fighter_profiles::FighterProfiles::default();
    let p = profiles.profile(15).unwrap().menu_options;
    let mut m = MenuSelection::new(&p);
    m.simplified_reactions = true;
    m.y_fighter_kind = 15;
    m.open_mode_list(&p, MenuRow::RollRules);
    m.auto_add_event = 4;
    m.add_mode_rule(&p, MenuRow::RollRules);
    let r = m.resolve_reactions(&p).unwrap();
    assert!(r.simple_sequence(4).is_none());
    let slot = m.row.slot().unwrap();
    let s = m
        .recipe_snapshot(&p, slot, m.choices[slot as usize])
        .unwrap();
    assert_eq!(s.reaction.trigger, Trigger::AutoGrabDodged);
    assert_eq!(s.len, 0);
    assert!(m.recipe_editing && !m.mode_rule_list);
}
/// Regression test for `native_simple_menu_keeps_legacy_events_and_opens_configured_grab_lists`.
#[test]
fn native_simple_menu_keeps_legacy_events_and_opens_configured_grab_lists() {
    let mut text =
        "[bowser.roll.old]\nstart_delay=0\nreaction=roll:grounded:0:80:30:any:1\nactions=grab:4\n"
            .to_owned();
    for event in ["auto_roll", "auto_spot_dodge", "auto_air_dodge"] {
        text.push_str(&format!(
            "[bowser.roll.{event}]\nstart_delay=0\nreaction={event}:80:30\nactions=up_b:4\n"
        ));
    }
    let catalog = crate::sequence_config::Catalog::parse(&text).unwrap();
    let mut profiles = crate::fighter_profiles::FighterProfiles::default();
    profiles.add_sequence_choices(&catalog).unwrap();
    let p = profiles.profile(15).unwrap().menu_options;
    let mut m = MenuSelection::new(&p);
    m.simplified_reactions = true;
    m.y_fighter_kind = 15;
    m.show_modes();
    m.input(0, &p);
    assert_eq!(m.resolve_reactions(&p).unwrap().len, 3);
    assert!(!m.row_visible(MenuRow::ShieldPressure));
    assert!(!m.row_visible(MenuRow::SpotPunish));
    m.row = MenuRow::RollRules;
    m.input(MENU_STICK_CLICK, &p);
    assert!(m.mode_rule_list);
    assert_eq!(m.mode_entries(&p, MenuRow::RollRules).len(), 3);
    m.restore_mode_choices();
    m.show_modes();
    m.row = MenuRow::OutOfShield;
    m.input(0, &p);
    m.input(MENU_STICK_CLICK, &p);
    assert!(!m.tuning_page);
    assert!(m.mode_rule_list);
    assert_eq!(m.mode_entries(&p, MenuRow::OutOfShield).len(), 0);
}
