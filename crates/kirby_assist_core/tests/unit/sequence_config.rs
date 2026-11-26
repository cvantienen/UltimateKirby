use super::*;
use crate::native_input::*;
/// Constructs a parsed input action from the supplied arguments.
fn input(buttons: u32, stick: [f32; 2], frames: u8, relative: bool) -> Action {
    Action::Input {
        buttons,
        stick,
        frames,
        relative,
    }
}

/// Parses the supplied source into the module’s validated representation.
fn parse(actions: &str) -> Sequence {
    Catalog::parse(&format!(
        "[kirby.drop.custom]\nstart_delay=0\nactions = {actions}"
    ))
    .unwrap()
    .entries()[0]
        .sequence
}
/// Regression test for sequence-declared auto-tilt directions.
#[test]
fn sequence_auto_tilt_parses_and_round_trips() {
    let sequence = Catalog::parse(
        "[lucina.neutral.custom]\nstart_delay=0\nactions=down_tilt\nauto_tilt=down,side,up",
    )
    .unwrap()
    .entries()[0]
        .sequence;
    assert_eq!(
        sequence.auto_tilt,
        Some(crate::posture::TiltSettings {
            down: true,
            forward: true,
            up: true
        })
    );
}

/// Regression test for `direction_spellings_are_unique_and_the_canonical_one_round_trips`.
#[test]
fn direction_spellings_are_unique_and_the_canonical_one_round_trips() {
    for (index, (direction, spellings)) in Direction::TOKENS.iter().enumerate() {
        assert_eq!(*direction as usize, index, "TOKENS must follow enum order");
        assert_eq!(direction.token(), spellings[0]);
        for spelling in *spellings {
            assert_eq!(Direction::from_token(spelling), Some(*direction));
        }
    }
    let mut all: Vec<_> = Direction::TOKENS
        .iter()
        .flat_map(|(_, spellings)| spellings.iter())
        .collect();
    let count = all.len();
    all.sort();
    all.dedup();
    assert_eq!(all.len(), count, "a spelling maps to two directions");
    assert_eq!(Direction::from_token("sideways"), None);
}

/// Regression test for `only_saved_ledge_geometry_directions_need_a_ledge`.
#[test]
fn only_saved_ledge_geometry_directions_need_a_ledge() {
    for (direction, _) in Direction::TOKENS {
        let needs = !matches!(
            direction,
            Direction::Opponent
                | Direction::AwayFromOpponent
                | Direction::Facing
                | Direction::Backward
        );
        for action in [
            Action::Face(direction),
            Action::Walk {
                direction,
                units: 1.0,
                strength: 0.5,
                min_gap: None,
            },
            Action::Run {
                direction,
                units: 1.0,
            },
            Action::Drift {
                direction,
                strength: 0.5,
                frames: 1,
            },
        ] {
            assert_eq!(action.needs_ledge(), needs, "{action:?}");
        }
    }
    assert!(Action::IfFacingLedge { away: true }.needs_ledge());
    assert!(!Action::IfFacingOpponent { away: true }.needs_ledge());
    assert!(!Action::Wait(1).needs_ledge());
}

/// Regression test for `only_universal_bindings_are_ignored_and_fighter_bindings_stay_separate`.
#[test]
fn only_universal_bindings_are_ignored_and_fighter_bindings_stay_separate() {
    let source="[universal.x.old]\nold invalid text ignored\n[kirby.x.custom]\nstart_delay=0\nactions=short_hop\n[yoshi.x.custom]\nstart_delay=2\nactions=grab\n[kirby.y.custom]\nstart_delay=1\nactions=shield\n";
    let c = Catalog::parse(source).unwrap();
    assert_eq!(c.len(), 3);
    assert_ne!(c.entries()[0].fighter_kind, c.entries()[1].fighter_kind);
    assert_ne!(c.entries()[0].sequence, c.entries()[1].sequence);
    assert!(Catalog::parse("[kirby.x.bad]\nstart_delay=oops\nactions=grab").is_err());
}
/// Regression test for `file_facing_inputs_and_special_strength_are_explicit_and_bounded`.
#[test]
fn file_facing_inputs_and_special_strength_are_explicit_and_bounded() {
    let s = parse("facing_input:A:1:-0.5:1, side_special:2:0.5");
    assert_eq!(s.actions()[0], input(MAPPER_A, [1.0, -0.5], 1, true));
    assert_eq!(s.actions()[1], input(MAPPER_B, [0.5, 0.0], 2, true));
    assert!(Catalog::parse("[kirby.neutral.bad]\nstart_delay=0\nactions=full_hop").is_err());
    assert!(Catalog::parse("[kirby.neutral.bad]\nstart_delay=0\nactions=up_special:1:2").is_err());
}

/// Regression test for `timeline_accepts_ordered_multi_move_air_specials_and_explicit_durations`.
#[test]
fn timeline_accepts_ordered_multi_move_air_specials_and_explicit_durations() {
    let sequence = parse("Short Hop, wait:2, Up B:3, wait:6, Down Special, release, grab");
    assert_eq!(sequence.len, 7);
    assert_eq!(sequence.actions()[0], input(MAPPER_X, [0.0; 2], 1, true));
    assert_eq!(sequence.actions()[1], Action::Wait(2));
    assert_eq!(sequence.actions()[2], input(MAPPER_B, [0.0, 1.0], 3, true));
    assert_eq!(sequence.actions()[4], input(MAPPER_B, [0.0, -1.0], 1, true));
}
/// Regression test for `timeline_raw_inputs_preserve_signed_axes_and_button_chords`.
#[test]
fn timeline_raw_inputs_preserve_signed_axes_and_button_chords() {
    let sequence = parse("input:A+X:-1:0.5:3, cstick:0:-1:1, hold:shield:8, press:B, stick:1:0:4");
    assert_eq!(
        sequence.actions()[0],
        input(MAPPER_A | MAPPER_X, [-1.0, 0.5], 3, false)
    );
    assert_eq!(
        sequence.actions()[1],
        Action::CStick {
            stick: [0.0, -1.0],
            frames: 1
        }
    );
    assert_eq!(sequence.actions()[2], input(MAPPER_ZR, [0.0; 2], 8, false));
}
/// Regression test for `timeline_universal_actions_validate_for_every_configured_fighter_without_hitbox_data`.
#[test]
fn timeline_universal_actions_validate_for_every_configured_fighter_without_hitbox_data() {
    let profiles = FighterProfiles::default();
    let mut checked = 0;
    for kind in 0..crate::fighter_kinds::PLAYABLE_KIND_COUNT {
        if !profiles
            .profile(kind)
            .is_some_and(|p| p.fighter_kind == kind)
        {
            continue;
        }
        let mut catalog = Catalog::parse("[kirby.neutral.everything]\nstart_delay=0\nactions = nair, fair, bair, uair, dair, jab, ftilt, utilt, dtilt, fsmash, usmash, dsmash, neutral_special, side_special, up_special, down_special, dash_attack, shield, roll_forward, roll_backward, spot_dodge, air_dodge, grab").unwrap();
        catalog.entries[0].fighter_kind = kind;
        catalog.validate(&profiles).unwrap();
        checked += 1;
    }
    assert!(checked > 70);
}
/// Regression test for `timeline_numeric_keys_are_order_independent_and_duplicates_invalid`.
#[test]
fn timeline_numeric_keys_are_order_independent_and_duplicates_invalid() {
    let catalog = Catalog::parse("[kirby.roll.custom]\nstart_delay = 7\ntimeout = 200\nspatial_timeout = 15\nactions = dash_attack:3:2\nname = My attack").unwrap();
    let sequence = catalog.entries()[0].sequence;
    assert_eq!(
        (
            sequence.start_delay,
            sequence.timeout,
            sequence.spatial_timeout
        ),
        (7, 200, 15)
    );
    assert_eq!(sequence.actions()[0], input(0, [1.0, 0.0], 3, true));
    assert_eq!(sequence.actions()[1], input(MAPPER_A, [1.0, 0.0], 2, true));
    assert!(
        Catalog::parse("[kirby.roll.custom]\nstart_delay=1\nstart_delay=2\nactions=jab").is_err()
    );
}
/// Regression test for `timeline_file_menus_require_explicit_delays_and_have_no_code_fallback`.
#[test]
fn timeline_file_menus_require_explicit_delays_and_have_no_code_fallback() {
    assert!(Catalog::parse("[kirby.neutral.custom]\nactions=jab")
        .unwrap_err()
        .contains("requires start_delay"));
    assert!(Catalog::parse("[kirby.neutral.custom]\nstart_delay=0\nactions=code:nair").is_err());
    let catalog = Catalog::parse("[kirby.neutral.first]\nstart_delay=0\ndefault=true\nactions=jab\n[kirby.neutral.second]\nstart_delay=2\ndefault=true\nactions=up_b").unwrap();
    assert!(catalog
        .validate(&FighterProfiles::default())
        .unwrap_err()
        .contains("multiple defaults"));
}

/// Regression test for `timeline_bounds_unknown_names_and_nonfinite_axes_are_rejected`.
#[test]
fn timeline_bounds_unknown_names_and_nonfinite_axes_are_rejected() {
    for actions in [
        "up_special:0",
        "input:A:NaN:0:1",
        "input:A:2:0:1",
        "hold:bad:2",
        "jab,,jab",
        "cstick:0:1:256",
        "unknown",
        "walk:away:5:0.5:2",
    ] {
        assert!(
            Catalog::parse(&format!(
                "[kirby.neutral.custom]\nstart_delay=0\nactions={actions}"
            ))
            .is_err(),
            "{actions}"
        );
    }
    let long = vec!["jab"; MAX_ACTIONS + 1].join(",");
    assert!(Catalog::parse(&format!(
        "[kirby.neutral.custom]\nstart_delay=0\nactions={long}"
    ))
    .is_err());
    assert!(Catalog::parse(
        "[kirby.neutral.custom]\nstart_delay=0\nactions=jab\n[kirby.neutral.custom]\nstart_delay=0\nactions=up_special"
    )
    .is_err());
}
/// Regression test for `timeline_deployment_validates_and_filters_only_diagnostic_tests`.
#[test]
fn timeline_deployment_validates_and_filters_only_diagnostic_tests() {
    let catalog = Catalog::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../deployment/ultimate/kirby_assist/assembled/sequences.txt"
    )))
    .unwrap();
    let lucina = catalog.tuning.get(&crate::fighter_kinds::LUCINA).unwrap();
    assert_eq!(lucina.0[crate::fighter_tuning::COUNTER_DEFAULT], Some(1.0));
    assert!(
        crate::auto_counter::Config::for_fighter(crate::fighter_kinds::LUCINA, *lucina).is_some()
    );
    let gamewatch = catalog
        .tuning
        .get(&crate::fighter_kinds::MR_GAME_AND_WATCH)
        .unwrap();
    assert_eq!(
        gamewatch.0[crate::fighter_tuning::REFLECT_DEFAULT],
        Some(1.0)
    );
    assert!(crate::projectile_reflect::Config::for_fighter(
        crate::fighter_kinds::MR_GAME_AND_WATCH,
        *gamewatch
    )
    .is_some());
    for diagnostic in [false, true] {
        let mut profiles = FighterProfiles::default();
        profiles
            .add_sequence_choices(&catalog.clone().for_build(diagnostic))
            .unwrap();
    }
    assert!(catalog.clone().for_build(false).len() < catalog.for_build(true).len());
}
