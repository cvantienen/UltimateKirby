use super::*;
use crate::{
    fighter_profiles::FighterProfiles,
    sequence_config::{persist, Catalog},
};
/// Regression test for `vulnerable_and_any_variants_share_the_right_menu_and_save_roundtrip`.
#[test]
fn vulnerable_and_any_variants_share_the_right_menu_and_save_roundtrip() {
    let source="[bowser.attack.oos]\nstart_delay=0\nreaction=shield_hit_vulnerable:grounded:0:22:12:front:1\nactions=input:r:0:0:4\n[bowser.attack.spot]\nstart_delay=0\nreaction=spot_dodge_any:grounded:0:30:12:any:2\nactions=up_b:4\n[bowser.attack.pressure]\nstart_delay=0\nreaction=shield_pressure_vulnerable:grounded:0:22:12:front:1\nactions=input:r:0:0:4\n";
    let c = Catalog::parse(source).unwrap();
    let mut profiles = FighterProfiles::default();
    profiles.add_sequence_choices(&c).unwrap();
    let p = profiles.profile(15).unwrap().menu_options;
    let m = MenuSelection::new(&p);
    for row in [
        MenuRow::OutOfShield,
        MenuRow::SpotPunish,
        MenuRow::ShieldPressure,
    ] {
        assert_eq!(m.mode_entries(&p, row).len(), 1);
    }
    assert_eq!(m.mode_entries(&p, MenuRow::RollRules).len(), 0);
    for e in c.entries() {
        let saved = persist::update(source, 15, e.slot, &e.choice, e.sequence, false).unwrap();
        let round = Catalog::parse(&saved).unwrap();
        assert_eq!(
            round
                .entries()
                .iter()
                .find(|r| r.choice == e.choice)
                .unwrap()
                .sequence
                .reaction,
            e.sequence.reaction
        );
    }
}
