use kirby_assist_core::{
    autopilot_guard::{self, duck_geometry::Hitbox},
    fighter_tuning::*,
    roll_read::{self, Choice},
    sequence_config::{persist, Catalog},
    settings::AssistSettings,
    spot_dodge::{self, Context, Target},
    tech_prediction::{self, Body},
};
#[test]
fn defaults_preserve_geometry_and_per_fighter_references() {
    let t = Tuning::default();
    let body = t.tech_body();
    let rays = tech_prediction::tech_probes_with_body(0., 0., 1., -1., 3., body);
    let old = tech_prediction::tech_probes(0., 0., 1., -1., 3.);
    for (a, b) in rays.iter().zip(old) {
        assert_eq!(a.is_some(), b.is_some());
        if let (Some(a), Some(b)) = (a, b) {
            assert_eq!(a.origin, b.origin);
            assert_eq!(a.delta, b.delta);
            assert_eq!(a.passable, b.passable);
        }
    }
    assert_eq!(rays[0].unwrap().origin, [-2.5, 1.]);
    assert_eq!(rays[4].unwrap().origin, [0., 3.5]);
    let e = t.shield();
    assert_eq!(
        [
            e.warning_x,
            e.warning_y,
            e.half_width,
            e.height,
            e.half_depth,
            e.forecast,
            e.scan_x,
            e.scan_y
        ],
        [24., 28., 6., 18., 6., 5., 96., 96.]
    );
    for kind in [
        kirby_assist_core::fighter_kinds::YOSHI,
        kirby_assist_core::fighter_kinds::KIRBY,
    ] {
        for choice in [Choice::DownSmash, Choice::Grab] {
            let Some(base) = roll_read::config_for(kind, choice) else {
                assert_eq!((kind, choice), (5, Choice::Grab));
                continue;
            };
            let actual = t.legacy_roll(base, choice);
            assert_eq!(
                [actual.reach, actual.height, actual.watch_distance],
                [base.reach, base.height, base.watch_distance]
            );
            assert_eq!(
                (actual.first, actual.last, actual.active_windows),
                (base.first, base.last, base.active_windows)
            );
        }
    }
}
#[test]
fn body_and_warning_overrides_change_predictions_without_altering_attack_geometry() {
    let mut t = Tuning::default();
    t.set("shield_body_half_width", "10").unwrap();
    t.set("shield_warning_x", "12").unwrap();
    let hit = Hitbox {
        start: [8., 5., 0.],
        end: [8., 5., 0.],
        radius: 1.,
        velocity: [0.; 2],
    };
    assert!(!autopilot_guard::active_hitbox_threat(
        [0.; 3], [0.; 2], hit
    ));
    assert!(autopilot_guard::active_hitbox_threat_with(
        [0.; 3],
        [0.; 2],
        hit,
        t.shield()
    ));
    assert!(autopilot_guard::approaching_active_attack(
        [0.; 2],
        [0.; 2],
        [20., 0.],
        [0.; 2]
    ));
    assert!(!autopilot_guard::approaching_active_attack_with(
        [0.; 2],
        [0.; 2],
        [20., 0.],
        [0.; 2],
        t.shield()
    ));
    assert_eq!(hit.radius, 1.);
    assert_eq!(hit.start, [8., 5., 0.]);
}
#[test]
fn configured_tech_offsets_apply_to_curved_and_linear_probes_without_more_rays() {
    let mut t = Tuning::default();
    t.set("tech_half_width", "10").unwrap();
    t.set("tech_floor_origin", "3").unwrap();
    t.set("tech_wall_high", "20").unwrap();
    let rays = tech_prediction::tech_probes_with_body(0., 0., 1., -1., 3., t.tech_body());
    assert_eq!(rays[0].unwrap().origin, [-10., 3.]);
    assert_eq!(rays[5].unwrap().origin, [0., 20.]);
    assert_eq!(
        rays.iter().flatten().count(),
        tech_prediction::tech_probes(0., 0., 1., -1., 3.)
            .iter()
            .flatten()
            .count()
    );
    let mut configured = Vec::new();
    let mut baseline = Vec::new();
    assert!(tech_prediction::first_contact_with_body(
        0.,
        0.,
        0.,
        -1.,
        3.,
        -0.1,
        t.tech_body(),
        |r| {
            configured.push(r);
            false
        }
    )
    .is_none());
    tech_prediction::first_contact_with_body(0., 0., 0., -1., 3., -0.1, Body::default(), |r| {
        baseline.push(r);
        false
    });
    assert_eq!(configured.len(), baseline.len());
    assert_eq!(configured[0].origin, [-10., 3.]);
}
fn own() -> Context {
    Context {
        object: 1,
        status: kirby_assist_core::native_states::status::WAIT,
        situation: kirby_assist_core::native_states::situation::GROUND,
        motion: 1,
        frame: 1.,
        rate: 1.,
        stopped: false,
        can_spot_dodge: true,
        position: [0.; 2],
        velocity: [0.; 2],
    }
}
fn enemy() -> Target {
    Target {
        id: 2,
        status: kirby_assist_core::native_states::CATCH,
        motion: 2,
        frame: 1.,
        rate: 1.,
        position: [9., 0.],
        velocity: [0.; 2],
        facing: -1.,
        first: 1,
        last: 8,
        reach: 6.,
    }
}
#[test]
fn grab_padding_and_height_are_independent_of_the_attackers_reach() {
    let c = own();
    let mut e = enemy();
    assert!(spot_dodge::contact(c, e).is_none());
    assert!(spot_dodge::contact_with_shape(c, e, 6., 4., 8.).is_some());
    e.position[1] = 10.;
    assert!(spot_dodge::contact_with_shape(c, e, 6., 4., 8.).is_none());
    assert!(spot_dodge::contact_with_shape(c, e, 6., 4., 12.).is_some());
    assert_eq!(e.reach, 6.);
}
#[test]
fn saved_values_round_trip_per_fighter_and_leave_recipe_inputs_unchanged() {
    let mut t = Tuning::default();
    let base = AssistSettings {
        fighter_kind: kirby_assist_core::fighter_kinds::YOSHI,
        ..Default::default()
    };
    let defaults = Tuning::values(base);
    for (i, key) in KEYS.iter().enumerate() {
        let text = if Tuning::is_bool(i) {
            (defaults[i] == 1.).to_string()
        } else {
            defaults[i].to_string()
        };
        t.set(key, &text).unwrap();
    }
    let source = "[yoshi.roll.grab]\nstart_delay=0\nactions=grab:4\n";
    let updated = persist::update_tuning(source, 5, t).unwrap();
    assert!(updated.starts_with(source));
    let c = Catalog::parse(&updated).unwrap();
    assert_eq!(c.tuning[&5], t);
    assert!(c.tuning.get(&6).is_none());
    let mut custom = Tuning::default();
    custom.set("grab_range", "30").unwrap();
    custom.set("jab_lock_reach", "10").unwrap();
    let old = kirby_assist_core::jab_lock::config(5).unwrap();
    let new = custom.jab_lock(5).unwrap();
    assert_eq!(new.jab_reach, 10.);
    assert_eq!(old.grab_reach, None);
    assert_eq!(new.grab_reach, Some(30.));
    assert_eq!(
        (new.jab_start, new.smash_start, new.grab_start),
        (old.jab_start, old.smash_start, old.grab_start)
    );
    assert_eq!(
        custom
            .legacy_roll(
                roll_read::config_for(6, Choice::Grab).unwrap(),
                Choice::Grab
            )
            .reach,
        30.
    );
    assert_eq!(
        custom.apply(base).sequence_overrides,
        base.sequence_overrides
    );
}
#[test]
fn zero_shield_tail_still_shields_an_actual_threat_and_specialist_rows_are_filtered() {
    let mut hold = autopilot_guard::GuardHold::default();
    let c = autopilot_guard::GuardClock {
        status: kirby_assist_core::native_states::status::WAIT,
        motion: 1,
        frame: 1.,
        rate: 1.,
        stopped: false,
    };
    assert!(hold.sample_with_duration(true, true, c, 0.));
    assert!(!hold.sample_with_duration(true, false, c, 0.));
    assert!(Tuning::visible(5, JAB_LOCK_REACH));
    assert!(!Tuning::visible(6, JAB_LOCK_REACH));
    assert!(Tuning::visible(6, CROUCH_RANGE));
    assert!(!Tuning::visible(15, CROUCH_RANGE));
    assert!(Tuning::visible(15, SHIELD_SCAN_X));
    for key in [
        "shield_body_half_width",
        "tech_floor_origin",
        "jab_lock_reach",
    ] {
        assert!(Tuning::default().set(key, "NaN").is_err());
    }
}

#[test]
fn filtered_settings_navigation_and_focus_match_the_visible_rows() {
    use kirby_assist_core::{
        fighter_profiles::presets::MenuPresets, ledge_trap::DPAD_DOWN, settings_menu::MenuRow,
        settings_menu::MenuSelection,
    };
    let p = MenuPresets::default();
    let mut m = MenuSelection::new(&p);
    m.y_fighter_kind = 15;
    m.menu_hidden = false;
    m.recipe_editing = true;
    m.tuning_page = true;
    m.row = MenuRow::FighterSettings;
    m.tuning_values = Tuning::values(AssistSettings {
        fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
        ..Default::default()
    });
    m.tuning_row = SHIELD_SCAN_Y;
    let panel = m.recipe_panel(&p);
    assert!(panel.body[panel.focus.unwrap()].contains("scan Y"));
    m.tuning_row = AUTO_GRAB_FAR;
    let panel = m.recipe_panel(&p);
    assert!(panel.focus.unwrap() < panel.body.len());
    assert!(panel.body.iter().any(|s| s.contains("approach range")));
    m.tuning_row = usize::MAX;
    let panel = m.recipe_panel(&p);
    assert!(
        panel.focus.unwrap() < panel.body.len(),
        "retired/invalid focus cannot address past the visible rows"
    );
    m.tuning_row = PUNISH_HEIGHT;
    m.input(0, &p);
    assert_eq!(m.tuning_row, SHIELD_SCAN_X);
    m.input(DPAD_DOWN, &p);
    assert_eq!(m.tuning_row, SHIELD_SCAN_Y);
    m.tuning_row = usize::MAX;
    m.input(0, &p);
    assert!(Tuning::visible(15, m.tuning_row));
}

#[test]
fn every_supported_fighter_uses_its_own_reference_grab_defaults_before_saved_overrides() {
    use kirby_assist_core::{fighter_data, fighter_profiles::FighterProfiles};
    let profiles = FighterProfiles::default();
    for reference in fighter_data::all() {
        let Some(profile) = profiles.profile(reference.kind) else {
            continue;
        };
        for status in [
            kirby_assist_core::native_states::CATCH,
            kirby_assist_core::native_states::CATCH_DASH,
            kirby_assist_core::native_states::CATCH_TURN,
        ] {
            let Some(g) = fighter_data::ordinary_grab(reference.kind, status) else {
                continue;
            };
            let actual = profile.grab.grab;
            match status {
                0x37 => {
                    assert_eq!(actual.standing_startup, g.first);
                    assert_eq!(actual.standing_range, g.reach.unwrap_or(0.0));
                    assert_eq!(actual.standing_active_frames, g.last - g.first + 1);
                }
                0x39 => assert_eq!(actual.dash_startup, g.first),
                _ => assert_eq!(actual.pivot_startup, g.first),
            }
        }
        if let Some(frame) = reference
            .stat("outofshield.grabpostshieldstun")
            .and_then(|s| s.parse::<u8>().ok())
            .filter(|f| (1..=120).contains(f))
        {
            assert_eq!(profile.grab.grab.shield_startup, frame);
        }
    }
    let mut t = Tuning::default();
    t.set("grab_range", "15").unwrap();
    let bowser = t.apply(
        profiles
            .profile(15)
            .unwrap()
            .apply(AssistSettings::default()),
    );
    assert_eq!(bowser.fighter.grab.standing_range, 15.0);
    assert_eq!(profiles.profile(1).unwrap().grab.grab.standing_range, 19.5);
}

#[test]
fn lightweight_automatic_grab_values_match_full_defaults_for_every_registered_fighter() {
    use kirby_assist_core::fighter_profiles::FighterProfiles;
    let profiles = FighterProfiles::default();
    for kind in 0..kirby_assist_core::fighter_kinds::PLAYABLE_KIND_COUNT {
        let Some(profile) = profiles.profile(kind) else {
            continue;
        };
        let base = profile.apply(AssistSettings::default());
        let values = Tuning::values(base);
        let default = Tuning::default();
        assert_eq!(
            default.auto_grab_ranges(kind, profile.grab.grab.standing_range),
            (values[AUTO_GRAB_CLOSE], values[AUTO_GRAB_FAR]),
            "fighter {kind}"
        );
        for (close, far) in [(None, Some(8.)), (Some(40.), None), (Some(9.), Some(35.))] {
            let mut t = Tuning::default();
            t.0[AUTO_GRAB_CLOSE] = close;
            t.0[AUTO_GRAB_FAR] = far;
            let old_close = t.value(AUTO_GRAB_CLOSE, values[AUTO_GRAB_CLOSE]);
            let old_far = t.value(AUTO_GRAB_FAR, values[AUTO_GRAB_FAR]).max(old_close);
            assert_eq!(
                t.auto_grab_ranges(kind, profile.grab.grab.standing_range),
                (old_close, old_far)
            );
        }
    }
}
