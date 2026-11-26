use super::*;
use crate::sequence_config::{persist, Catalog};
/// Startup-loaded per-fighter tuning overrides.
const SOURCE: &str = "# keep recipes\n[kirby.neutral.nair]\nstart_delay=0\nactions=nair\n[yoshi.neutral.grab]\nstart_delay=0\nactions=grab:fighter_range\n";
/// Regression test for `saved_overrides_are_per_fighter_and_preserve_recipes`.
#[test]
fn saved_overrides_are_per_fighter_and_preserve_recipes() {
    let mut tuning = Tuning::default();
    tuning.set("walk_strength", "0.28").unwrap();
    tuning.set("grab_range", "14.5").unwrap();
    tuning.set("use_range_limits", "true").unwrap();
    tuning.set("counter_aim_after_activation", "true").unwrap();
    let output = persist::update_tuning(SOURCE, 5, tuning).unwrap();
    assert!(output.starts_with(SOURCE));
    let catalog = Catalog::parse(&output).unwrap();
    assert_eq!(catalog.len(), 2);
    assert_eq!(catalog.tuning.get(&5), Some(&tuning));
    assert!(catalog.tuning.get(&6).is_none());
    let recipe = catalog
        .find(
            5,
            crate::fighter_profiles::presets::PresetSlot::Neutral,
            "grab",
        )
        .unwrap();
    let renamed = persist::update_named(
        &output,
        5,
        crate::fighter_profiles::presets::PresetSlot::Neutral,
        "grab",
        recipe,
        false,
        Some("Named grab"),
    )
    .unwrap();
    assert_eq!(Catalog::parse(&renamed).unwrap().tuning[&5], tuning);

    let cleared = persist::update_tuning(&output, 5, Tuning::default()).unwrap();
    assert!(cleared.starts_with(SOURCE));
    assert_eq!(cleared.matches("[yoshi.settings]").count(), 1);
    assert_eq!(
        Catalog::parse(&cleared).unwrap().tuning[&5],
        Tuning::default()
    );
}
/// Regression test for `invalid_settings_reject_before_loading`.
#[test]
fn invalid_settings_reject_before_loading() {
    for text in [
        "walk_strength=NaN",
        "walk_strength=0.9",
        "grab_range=-1",
        "unknown=4",
        "manual_spacing=1",
        "walk_strength=0.2\nwalk_strength=0.3",
    ] {
        assert!(
            Catalog::parse(&format!("[yoshi.settings]\n{text}")).is_err(),
            "{text}"
        );
    }
    assert!(Catalog::parse("[yoshi.settings]\n[yoshi.settings]\n").is_err());
}
#[test]
fn auto_spacing_overrides_roundtrip_independently() {
    use crate::auto_spacing::Attack;
    let mut tuning = Tuning::default();
    for (key, value) in [
        ("auto_spacing_minimum_distance", "14.5"),
        ("auto_spacing_engagement_range", "23.5"),
        ("auto_spacing_ground_speed", "1"),
        ("auto_spacing_maximum_drift", "0.6"),
        ("auto_spacing_minimum_drift", "0.15"),
        ("auto_spacing_ground_braking", "false"),
        ("auto_spacing_air_braking", "true"),
        ("auto_spacing_braking_distance", "3.5"),
        ("auto_spacing_ground_move", "3"),
        ("auto_spacing_ground_startup", "9"),
        ("auto_spacing_ground_active_frames", "2"),
        ("auto_spacing_ground_reach", "18.5"),
        ("auto_spacing_ground_height", "21"),
        ("auto_spacing_ground_commitment", "40"),
        ("auto_spacing_aerial_move", "5"),
        ("auto_spacing_aerial_startup", "7"),
        ("auto_spacing_aerial_active_frames", "4"),
        ("auto_spacing_aerial_reach", "22"),
        ("auto_spacing_aerial_height", "13.5"),
        ("auto_spacing_aerial_commitment", "31"),
    ] {
        tuning.set(key, value).unwrap();
    }
    let output = persist::update_tuning(SOURCE, crate::fighter_kinds::YOSHI, tuning).unwrap();
    let loaded = Catalog::parse(&output).unwrap().tuning[&crate::fighter_kinds::YOSHI];
    assert_eq!(loaded, tuning);
    let config = loaded.auto_spacing(crate::fighter_kinds::YOSHI);
    assert_eq!(config.minimum_distance, 14.5);
    assert_eq!(config.engagement_range, 23.5);
    assert_eq!(config.ground_speed, 1.);
    assert_eq!(config.maximum_drift, 0.6);
    assert_eq!(config.minimum_drift, 0.15);
    assert!(!config.ground_braking);
    assert!(config.air_braking);
    assert_eq!(config.braking_distance, 3.5);
    assert!(matches!(config.ground.attack, Attack::Up));
    assert!(matches!(config.aerial.attack, Attack::Back));
    assert_eq!(
        [
            config.ground.startup,
            config.ground.active_frames,
            config.ground.reach,
            config.ground.height,
            config.ground.commitment
        ],
        [9., 2., 18.5, 21., 40.]
    );
    assert_eq!(
        [
            config.aerial.startup,
            config.aerial.active_frames,
            config.aerial.reach,
            config.aerial.height,
            config.aerial.commitment
        ],
        [7., 4., 22., 13.5, 31.]
    );
    let mut back = Tuning::default();
    back.set("auto_spacing_ground_move", "5").unwrap();
    assert!(matches!(
        back.auto_spacing(crate::fighter_kinds::LUCINA)
            .ground
            .attack,
        Attack::Disabled
    ));
}
#[test]
fn auto_spacing_rejects_invalid_fields() {
    for text in [
        "auto_spacing_ground_move=1.5",
        "auto_spacing_aerial_move=6",
        "auto_spacing_ground_move=-1",
        "auto_spacing_minimum_distance=NaN",
        "auto_spacing_ground_startup=0",
        "auto_spacing_ground_active_frames=301",
        "auto_spacing_aerial_reach=-1",
        "auto_spacing_aerial_height=inf",
        "auto_spacing_aerial_commitment=0",
        "auto_spacing_ground_speed=1.01",
        "auto_spacing_maximum_drift=-0.1",
        "auto_spacing_minimum_drift=NaN",
        "auto_spacing_ground_braking=1",
        "auto_spacing_air_braking=maybe",
        "auto_spacing_braking_distance=-1",
        "auto_spacing_braking_distance=NaN",
        "auto_spacing_reactive_retreat_distance=34",
        "auto_spacing_minimum_distance=12\nauto_spacing_minimum_distance=14",
    ] {
        assert!(
            Catalog::parse(&format!("[yoshi.settings]\n{text}")).is_err(),
            "{text}"
        );
    }
}
#[test]
fn auto_spacing_saved_lucina_values_survive_new_defaults() {
    let catalog = Catalog::parse("[lucina.settings]\nauto_spacing_ground_move=2\nauto_spacing_aerial_move=2\nauto_spacing_minimum_distance=12\nauto_spacing_ground_speed=0\nauto_spacing_maximum_drift=0.45\n").unwrap();
    let tuning = catalog.tuning[&crate::fighter_kinds::LUCINA];
    let config = tuning.auto_spacing(crate::fighter_kinds::LUCINA);
    assert_eq!(config.ground.attack, crate::auto_spacing::Attack::Forward);
    assert_eq!(config.aerial.attack, crate::auto_spacing::Attack::Forward);
    assert_eq!(config.minimum_distance, 12.);
    assert_eq!(config.ground_speed, 0.);
    assert_eq!(config.maximum_drift, 0.45);
    assert_eq!(config.fighter_kind, crate::fighter_kinds::LUCINA);
    let source = persist::update_tuning("", crate::fighter_kinds::LUCINA, tuning).unwrap();
    assert!(!source.contains("reactive_retreat"));
    assert_eq!(
        Catalog::parse(&source).unwrap().tuning[&crate::fighter_kinds::LUCINA],
        tuning
    );
    assert!(Tuning::default().set("auto_defense_lead_frames", "10").is_ok());
}

#[test]
fn auto_spacing_defaults_resolve_by_fighter_and_match_menu() {
    use crate::auto_spacing::Attack;
    for kind in [
        crate::fighter_kinds::LUCINA,
        crate::fighter_kinds::YOSHI,
        crate::fighter_kinds::KIRBY,
    ] {
        let config = Tuning::default().auto_spacing(kind);
        let menu = Tuning::values(AssistSettings {
            fighter_kind: kind,
            ..AssistSettings::default()
        });
        assert_eq!(config.minimum_distance, menu[AUTO_SPACING_MINIMUM_DISTANCE]);
        assert_eq!(config.maximum_drift, menu[AUTO_SPACING_MAXIMUM_DRIFT]);
        assert_eq!(config.ground_speed, 0.49);
        assert_eq!(config.minimum_distance, 15.);
        assert_eq!(config.engagement_range, 20.);
        assert!(config.ground_braking && config.air_braking);
        assert_eq!(config.braking_distance, 10.);
        assert_eq!(config.minimum_drift, 0.1);
        assert_eq!(config.maximum_drift, 1.);
        assert!(matches!(config.ground.attack, Attack::Disabled));
        assert!(matches!(config.aerial.attack, Attack::Disabled));
        assert_eq!(menu[AUTO_SPACING_GROUND_MOVE], 0.);
        assert_eq!(menu[AUTO_SPACING_AERIAL_MOVE], 0.);
    }
}
/// Regression test for `applied_values_change_decisions_and_inputs_only`.
#[test]
fn applied_values_change_decisions_and_inputs_only() {
    let base = AssistSettings::default();
    assert_eq!(
        Tuning::default().apply(base).walk_strength,
        base.walk_strength
    );
    let mut tuning = Tuning::default();
    tuning.set("walk_strength", "0.28").unwrap();
    tuning.set("grab_range", "14.5").unwrap();
    tuning.set("use_range_limits", "true").unwrap();
    let s = tuning.apply(base);
    assert_eq!(s.fighter.grab.standing_range, 14.5);
    assert!(!s.distance_agnostic);
    assert_eq!(
        s.fighter.grab.standing_startup,
        base.fighter.grab.standing_startup
    );
    assert_eq!(s.sequence_overrides, base.sequence_overrides);
    assert_eq!(
        crate::manual_controls::ledge_walk_stick([1.0, -0.6], s.walk_strength),
        [0.26, -0.6]
    );
    assert_eq!(
        crate::manual_controls::ledge_walk_stick([-1.0, 0.0], s.walk_strength),
        [-0.5, 0.0]
    );
    assert_eq!(
        crate::manual_controls::ledge_walk_stick([0.15, 0.0], s.walk_strength),
        [0.15, 0.0]
    );
}
