//! Stable fixtures for the current three-event system, independent of user recipes.
use kirby_assist_core::{
    fighter_profiles::{presets::PresetSlot, FighterProfiles},
    reaction_rules::{Candidate, Resolved},
    sequence_config::{persist, Catalog},
};
fn source() -> String {
    let mut s=String::from("[bowser.neutral.keep]\nstart_delay=7\ndefault=true\nactions=down_smash:4\n[bowser.y.keep]\nstart_delay=0\ndefault=true\nactions=grab:4\n");
    for event in ["auto_roll", "auto_spot_dodge", "auto_air_dodge"] {
        s.push_str(&format!("[bowser.roll.{event}]\nstart_delay=0\nreaction={event}:30:10\nactions=up_b:4,drift:opponent:1:30\n"));
    }
    s
}
#[test]
fn bowser_current_assists_resolve_one_sequence_per_event_in_both_builds() {
    for diagnostic in [false, true] {
        let cat = Catalog::parse(&source()).unwrap().for_build(diagnostic);
        let mut p = FighterProfiles::default();
        p.add_sequence_choices(&cat).unwrap();
        p.validate().unwrap();
        let mut r = Resolved::default();
        for e in cat
            .entries()
            .iter()
            .filter(|e| e.sequence.reaction.active())
        {
            r.add(Candidate {
                slot: e.slot as usize,
                index: 0,
                sequence: e.sequence,
            })
            .unwrap();
        }
        assert_eq!(r.len, 3);
        for event in 0..3 {
            assert!(r.simple_sequence(event).is_some());
        }
        let menu = p.profile(15).unwrap().menu_options;
        assert_eq!(
            menu.list(PresetSlot::Neutral).options[menu.list(PresetSlot::Neutral).selected].id,
            "keep"
        );
        assert_eq!(
            menu.list(PresetSlot::Y).options[menu.list(PresetSlot::Y).selected].id,
            "keep"
        );
    }
}
#[test]
fn saving_auto_react_does_not_rewrite_manual_ledge_or_button_recipes() {
    let s = source();
    let c = Catalog::parse(&s).unwrap();
    let seq = c.find(15, PresetSlot::Roll, "auto_roll").unwrap();
    let saved = persist::update_named(
        &s,
        15,
        PresetSlot::Roll,
        "auto_roll",
        seq,
        false,
        Some("Roll response"),
    )
    .unwrap();
    let after = Catalog::parse(&saved).unwrap();
    for slot in [PresetSlot::Neutral, PresetSlot::Y] {
        assert_eq!(after.find(15, slot, "keep"), c.find(15, slot, "keep"));
    }
}
