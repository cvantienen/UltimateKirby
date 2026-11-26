use super::*;
/// Verifies shared templates remain shared and unrelated menus stay unexpanded.
#[test]
fn startup_keeps_shared_templates_but_never_expands_other_fighter_menus() {
    let catalog=Catalog::parse("[template.defend]\nstart_delay=0\nactions=shield:4\n[bowser.roll.spin]\nstart_delay=0\nreaction=auto_roll:30:12\nactions=up_b:4,drift:opponent:1:69\n[lucina.neutral.poke]\nstart_delay=23\nactions=forward_tilt:1\n").unwrap();
    let (_, profiles, library) = prepare_runtime(LiveConfig::default(), catalog).unwrap();
    let mut count = 0;
    for kind in 0..kirby_assist_core::fighter_kinds::PLAYABLE_KIND_COUNT {
        let Some(profile) = profiles.profile(kind) else {
            continue;
        };
        count += 1;
        assert!(profile
            .menu_options
            .lists
            .iter()
            .all(|list| list.options.is_empty()));
        assert_eq!(profile.menu_options.templates.len(), 1);
    }
    assert_eq!(count, 90);
    assert_eq!(library.fighter_entries(15), 1);
    assert_eq!(library.fighter_entries(22), 1);
    assert_eq!(library.for_fighter(5).len(), 0);
    assert!(std::ptr::eq(
        profiles
            .profile(15)
            .unwrap()
            .menu_options
            .templates
            .as_ptr(),
        profiles
            .profile(22)
            .unwrap()
            .menu_options
            .templates
            .as_ptr()
    ));
}
