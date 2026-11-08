use kirby_assist_core::{
    action_owner::{ActionOwnership, Intent, Owner},
    fighter_tuning::{Tuning, TILT_TURN_DOWN, TILT_TURN_FORWARD, TILT_TURN_UP},
    manual_controls::ManualContext,
    native_input::{
        ButtonTrace, MAPPER_A, MAPPER_B, MAPPER_RIGHT_STICK_LEFT, MAPPER_RIGHT_STICK_RIGHT,
    },
    native_states::{situation, status},
    posture::{Posture, TiltSettings, TiltSideAliases},
    sequence_config::{persist, Catalog},
    settings::AssistSettings,
};

fn context() -> ManualContext {
    ManualContext {
        status: status::WAIT,
        situation: situation::GROUND,
        motion: 1,
        frame: 0.0,
        rate: 1.0,
        stopped: false,
        facing: 1.0,
    }
}

#[test]
fn defaults_and_settings_round_trip_for_each_fighter() {
    for kind in 0..kirby_assist_core::fighter_kinds::PLAYABLE_KIND_COUNT {
        let defaults = Tuning::default().tilt_turn(kind);
        assert_eq!(defaults.down, matches!(kind, 5 | 6));
        assert!(!defaults.up && !defaults.forward);
        let values = Tuning::values(AssistSettings {
            fighter_kind: kind,
            ..Default::default()
        });
        assert_eq!(values[TILT_TURN_DOWN] == 1.0, defaults.down);
        for i in [TILT_TURN_DOWN, TILT_TURN_FORWARD, TILT_TURN_UP] {
            assert!(Tuning::visible(kind, i));
            assert!(Tuning::is_bool(i));
        }
    }
    let source = "[bowser.settings]\ntilt_turn_down = true\ntilt_turn_forward = true\ntilt_turn_up = false\n";
    let bowser = kirby_assist_core::fighter_kinds::BOWSER;
    let tuning = Catalog::parse(source).unwrap().tuning[&bowser];
    assert_eq!(
        tuning.tilt_turn(bowser),
        TiltSettings {
            down: true,
            forward: true,
            up: false
        }
    );
    let saved = persist::update_tuning(source, bowser, tuning).unwrap();
    assert_eq!(Catalog::parse(&saved).unwrap().tuning[&bowser], tuning);
    assert!(Tuning::default().set("tilt_turn_up", "1").is_err());
}

#[test]
fn selected_c_stick_tilts_aim_without_creating_attack_buttons() {
    for facing in [-1.0, 1.0] {
        let c = ManualContext {
            facing,
            ..context()
        };
        for (settings, right, expected_right) in [
            (
                TiltSettings {
                    up: true,
                    ..Default::default()
                },
                [0.0, 1.0],
                [-facing * 0.8, 1.0],
            ),
            (
                TiltSettings {
                    forward: true,
                    ..Default::default()
                },
                [facing, 0.0],
                [-facing, 0.0],
            ),
        ] {
            let mut posture = Posture::default();
            let out = posture.sample_configured(
                true,
                false,
                true,
                settings,
                0,
                [0.0; 2],
                right,
                Some(-facing),
                c,
            );
            assert_eq!(out.stick, Some([-facing * 0.8, 0.0]));
            assert_eq!((out.requested, out.owned), (0, 0));
            assert_eq!(
                Posture::configured_c_stick(settings, 0, [0.0; 2], right, Some(-facing), c)
                    .unwrap()
                    .1,
                expected_right
            );
            assert_eq!(
                posture
                    .sample_configured(
                        true,
                        false,
                        false,
                        settings,
                        0,
                        [0.0; 2],
                        right,
                        Some(-facing),
                        c
                    )
                    .stick,
                None
            );
            assert_eq!(
                posture
                    .sample_configured(
                        true,
                        false,
                        true,
                        TiltSettings::default(),
                        0,
                        [0.0; 2],
                        right,
                        Some(-facing),
                        c
                    )
                    .stick,
                None
            );
            assert_eq!(
                posture
                    .sample_configured(
                        false,
                        false,
                        true,
                        settings,
                        0,
                        [0.0; 2],
                        right,
                        Some(-facing),
                        c
                    )
                    .stick,
                None
            );
        }
    }
}

#[test]
fn configured_tilts_yield_to_other_inputs_and_invalid_observations() {
    let settings = TiltSettings {
        down: false,
        forward: true,
        up: true,
    };
    for (buttons, left, right, target, c) in [
        (MAPPER_A, [0.0, 1.0], [0.0, 0.0], Some(-1.0), context()),
        (MAPPER_B, [0.0, 0.0], [1.0, 0.0], Some(-1.0), context()),
        (0, [0.5, 0.0], [1.0, 0.0], Some(-1.0), context()),
        (0, [0.0, 0.0], [f32::NAN, 1.0], Some(-1.0), context()),
        (0, [0.0, 0.0], [0.0, 0.0], Some(-1.0), context()),
        (
            MAPPER_RIGHT_STICK_LEFT,
            [0.0, 0.0],
            [0.0, 0.0],
            Some(-1.0),
            context(),
        ),
        (0, [0.0, 0.0], [1.0, 0.0], None, context()),
        (
            0,
            [0.0, 0.0],
            [1.0, 0.0],
            Some(-1.0),
            ManualContext {
                situation: situation::AIR,
                ..context()
            },
        ),
        (
            0,
            [0.0, 0.0],
            [1.0, 0.0],
            Some(-1.0),
            ManualContext {
                stopped: true,
                ..context()
            },
        ),
        (
            0,
            [0.0, 0.0],
            [1.0, 0.0],
            Some(-1.0),
            ManualContext {
                status: status::DAMAGE,
                ..context()
            },
        ),
    ] {
        assert_eq!(
            Posture::configured_c_stick(settings, buttons, left, right, target, c),
            None
        );
    }
    for status in [status::ATTACK_HI3, status::ATTACK_S3] {
        let c = ManualContext {
            status,
            stopped: true,
            ..context()
        };
        assert!(
            Posture::configured_c_stick(settings, 0, [0.0; 2], [1.0, 0.0], Some(-1.0), c).is_some()
        );
        assert!(Posture::needs_configured_target(
            status,
            situation::GROUND,
            true
        ));
    }
}

#[test]
fn side_aliases_keep_physical_press_and_release_without_extra_attack_edges() {
    let mut aliases = TiltSideAliases::default();
    let physical = ButtonTrace {
        held: MAPPER_RIGHT_STICK_RIGHT,
        pressed: MAPPER_RIGHT_STICK_RIGHT,
        ..Default::default()
    };
    let first = aliases.apply(physical, Some(-1.0));
    assert_eq!(first.held, MAPPER_RIGHT_STICK_LEFT);
    assert_eq!(first.pressed, MAPPER_RIGHT_STICK_LEFT);
    let held = ButtonTrace {
        held: MAPPER_RIGHT_STICK_RIGHT,
        previous: MAPPER_RIGHT_STICK_RIGHT,
        ..Default::default()
    };
    let changed = aliases.apply(held, Some(1.0));
    assert_eq!(changed.previous, MAPPER_RIGHT_STICK_LEFT);
    assert_eq!(changed.held, MAPPER_RIGHT_STICK_RIGHT);
    assert_eq!(changed.pressed, 0);
    let release = ButtonTrace {
        previous: MAPPER_RIGHT_STICK_RIGHT,
        released: MAPPER_RIGHT_STICK_RIGHT,
        ..Default::default()
    };
    assert_eq!(
        aliases.apply(release, None).released,
        MAPPER_RIGHT_STICK_RIGHT
    );
    let mut aliases = TiltSideAliases::default();
    aliases.apply(physical, Some(-1.0));
    let released = aliases.apply(release, None);
    assert_eq!(
        (released.previous, released.released),
        (MAPPER_RIGHT_STICK_LEFT, MAPPER_RIGHT_STICK_LEFT)
    );
    assert_eq!(released.pressed, 0);
}

#[test]
fn down_setting_preserves_legacy_aim_and_can_be_disabled() {
    let mut posture = Posture::default();
    let down = TiltSettings {
        down: true,
        ..Default::default()
    };
    let enabled = posture.sample_configured(
        true,
        false,
        true,
        down,
        MAPPER_A,
        [0.0, -1.0],
        [0.0; 2],
        Some(-1.0),
        context(),
    );
    assert_eq!(enabled.stick, Some([-0.8, -1.0]));
    assert_eq!(enabled.requested, 0);
    let disabled = posture.sample_configured(
        true,
        false,
        true,
        TiltSettings::default(),
        MAPPER_A,
        [0.0, -1.0],
        [0.0; 2],
        Some(-1.0),
        context(),
    );
    assert_eq!(disabled.stick, None);
}

#[test]
fn turnaround_remains_passive_against_committed_actions_and_safety() {
    for owner in [
        Owner::Ledge,
        Owner::RollRead,
        Owner::Trip,
        Owner::Fair,
        Owner::Safety,
    ] {
        let mut ownership = ActionOwnership::new();
        let intents = [
            Intent {
                owner: Owner::Posture,
                present: true,
                committed: false,
            },
            Intent {
                owner,
                present: true,
                committed: owner != Owner::Safety,
            },
        ];
        assert_eq!(ownership.select(true, &intents), Some(owner));
        assert_eq!(ownership.select(false, &intents), None);
    }
}
