use super::*;
/// Regression test for `list_manager_explicit_default_requests_a_save_without_changing_the_recipe`.
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
