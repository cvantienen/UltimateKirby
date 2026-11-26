use super::*;
/// Constructs the non grab events start blank for bowser yoshi and lucina.
#[test]
fn new_non_grab_events_start_blank_for_bowser_yoshi_and_lucina() {
    let profiles = crate::fighter_profiles::FighterProfiles::default();
    for kind in [
        crate::fighter_kinds::YOSHI,
        crate::fighter_kinds::BOWSER,
        crate::fighter_kinds::WARIO,
    ] {
        let p = profiles.profile(kind).unwrap().menu_options;
        for event in 0..6 {
            let mut m = MenuSelection::new(&p);
            m.simplified_reactions = true;
            m.y_fighter_kind = kind;
            m.tuning_values[1] = profiles.profile(kind).unwrap().grab.grab.standing_range;
            m.open_mode_list(&p, MenuRow::RollRules);
            m.auto_add_event = event;
            m.add_mode_rule(&p, MenuRow::RollRules);
            let rules = m.resolve_reactions(&p).unwrap();
            let grab = event == 3;
            assert_eq!(
                rules.simple_sequence(event).is_some(),
                grab,
                "kind {kind}, event {event}"
            );
            let slot = m.row.slot().unwrap();
            let draft = m
                .recipe_snapshot(&p, slot, m.choices[slot as usize])
                .unwrap();
            if grab {
                assert!(matches!(
                    draft.actions(),
                    [crate::sequence_config::Action::Input {
                        buttons: crate::native_input::MAPPER_R,
                        ..
                    }]
                ));
            } else {
                assert_eq!(draft.len, 0);
                assert!(m.recipe_input_error(&p).is_some());
            }
        }
    }
}
