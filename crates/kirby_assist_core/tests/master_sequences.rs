use kirby_assist_core::{
    fighter_profiles::presets::PresetSlot,
    fighter_tuning::Tuning,
    sequence_config::{layout, persist, Catalog},
};
const SOURCE: &str = "[universal.settings]\nenabled = true\nledge_walk_strength = 0.35\n[kirby.neutral.first]\nname = Original\nstart_delay = 0\nactions = grab:4\n[yoshi.neutral.first]\nstart_delay = 0\nactions = grab:4\n";
#[test]
fn master_validates_settings_and_rejects_errors() {
    assert!(Catalog::parse(SOURCE)
        .unwrap()
        .universal
        .unwrap()
        .contains("ledge_walk_strength"));
    assert!(Catalog::parse(&SOURCE.replace("enabled = true", "enabled = invalid")).is_err());
    assert!(
        Catalog::parse(&SOURCE.replace("enabled = true", "enabled = true\nenabled = false"))
            .is_err()
    );
    assert!(Catalog::parse(&format!("{SOURCE}[universal.settings]\nenabled = true\n")).is_err());
}
#[test]
fn saves_create_rename_settings_delete_in_fighter_group() {
    let source = layout::organize(SOURCE).unwrap();
    let sequence = Catalog::parse(&source).unwrap().entries()[0].sequence;
    let added = persist::update_named(
        &source,
        6,
        PresetSlot::Roll,
        "new",
        sequence,
        true,
        Some("New recipe"),
    )
    .unwrap();
    assert!(added.find("[kirby.roll.new]").unwrap() < added.find("[yoshi.neutral.first]").unwrap());
    assert!(added.contains("# ---- New recipe | kirby.roll.new ----"));
    let mut t = Tuning::default();
    t.set("stand_distance", "12").unwrap();
    let tuned = persist::update_tuning(&added, 6, t).unwrap();
    assert!(tuned.find("[kirby.settings]").unwrap() < tuned.find("[kirby.neutral.first]").unwrap());
    let removed = persist::delete(&tuned, 6, PresetSlot::Roll, "new", false).unwrap();
    assert!(!removed.contains("New recipe"));
    assert_eq!(
        Catalog::parse(&removed).unwrap().universal,
        Catalog::parse(SOURCE).unwrap().universal
    );
    assert_eq!(layout::organize(&removed).unwrap(), removed);
}

#[test]
fn default_save_cannot_replace_ledge_defaults_with_an_assist_rule() {
    let source=format!("{SOURCE}[kirby.neutral.oos]\nstart_delay=0\nreaction=shield_hit:grounded:0:12:18:any:1\nactions=up_b:4\n");
    assert!(persist::set_default(&source, 6, PresetSlot::Neutral, "oos").is_err());
    let saved = persist::set_default(&source, 6, PresetSlot::Neutral, "first").unwrap();
    let catalog = Catalog::parse(&saved).unwrap();
    assert!(
        catalog
            .entries()
            .iter()
            .find(|e| e.choice == "first" && e.fighter_kind == 6)
            .unwrap()
            .default
    );
    assert!(
        !catalog
            .entries()
            .iter()
            .find(|e| e.choice == "oos")
            .unwrap()
            .default
    );
}
