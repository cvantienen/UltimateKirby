//! Cross-fighter admission and revision-built observation requirements.
use kirby_assist_core::{
    fighter_profiles::FighterProfiles,
    gameplay::GameplayContext,
    ledge_trap::LedgeSettings,
    native_input::{MAPPER_A, MAPPER_B, MAPPER_ZR},
    reaction_rules::{sample_optional_simple, simple_readiness, Candidate, Engine, Resolved},
    roll_read::Target,
    sequence_config::Catalog,
};

fn rules(events: &[(&str, &str)]) -> Resolved {
    let mut result = Resolved::default();
    for (index, (event, actions)) in events.iter().enumerate() {
        let catalog = Catalog::parse(&format!(
            "[yoshi.roll.test]\nstart_delay=0\nreaction={event}:20:8\nactions={actions}\n",
        ))
        .unwrap();
        result
            .add(Candidate {
                slot: 2,
                index,
                sequence: catalog.entries()[0].sequence,
            })
            .unwrap();
    }
    result
}
fn context() -> GameplayContext {
    GameplayContext {
        gameplay: true,
        held: true,
        status: kirby_assist_core::native_states::status::WAIT,
        situation: 0,
        motion_kind: 1,
        motion_rate: 1.,
        facing: 1.,
        safe_ground: [true; 2],
        can_act: false,
        ..Default::default()
    }
}
fn targets(status: i32, x: f32, y: f32) -> [Option<Target>; 8] {
    let mut result = [None; 8];
    result[0] = Some(Target {
        id: 2,
        status,
        motion: 2,
        frame: 0.,
        rate: 1.,
        position: [x, y],
        velocity_x: 0.,
        vulnerable_in: None,
        actionable_in: None,
    });
    result
}

#[test]
fn custom_attacks_do_not_require_up_b_and_up_b_keeps_its_own_gate_for_every_fighter() {
    let attacks = rules(&[
        ("auto_roll", "down_smash:1"),
        ("auto_spot_dodge", "down_smash:1"),
        ("auto_air_dodge", "down_smash:1"),
    ]);
    let up_b = rules(&[("auto_roll", "up_b:4")]);
    let profiles = FighterProfiles::default();
    let mut checked = 0;
    for kind in 0..kirby_assist_core::fighter_kinds::PLAYABLE_KIND_COUNT {
        if profiles.profile(kind).is_none() {
            continue;
        }
        checked += 1;
        let settings = LedgeSettings {
            fighter_kind: kind,
            ..Default::default()
        };
        for status in [
            kirby_assist_core::native_states::ESCAPE_F,
            kirby_assist_core::native_states::ESCAPE,
            kirby_assist_core::native_states::status::ESCAPE_AIR,
        ] {
            let t = targets(status, 20., 8.);
            let readiness = simple_readiness(attacks.simple_starts(), context(), &t, || {
                panic!("standing attack needs no extra shield query")
            });
            let output = sample_optional_simple(
                &mut None,
                true,
                Some(&attacks),
                settings,
                context(),
                t,
                None,
                readiness,
            );
            assert_ne!(
                output.requested & MAPPER_A,
                0,
                "fighter {kind}, event {status:#x}"
            );
            assert_eq!(output.requested & MAPPER_B, 0);
        }
        let t = targets(kirby_assist_core::native_states::ESCAPE_F, 10., 0.);
        let readiness = simple_readiness(up_b.simple_starts(), context(), &t, || {
            panic!("Up-B must never use ordinary action readiness")
        });
        assert_eq!(readiness, [false; 3]);
        assert_eq!(
            sample_optional_simple(
                &mut None,
                true,
                Some(&up_b),
                settings,
                context(),
                t,
                None,
                readiness
            )
            .owned,
            0
        );
    }
    assert_eq!(checked, 79);
}

#[test]
fn shield_readiness_is_lazy_shared_and_never_queried_for_invalid_or_unconfigured_events() {
    let rules = rules(&[("auto_roll", "down_smash:1"), ("auto_spot_dodge", "a:1")]);
    let c = GameplayContext {
        status: kirby_assist_core::native_states::GUARD,
        ..context()
    };
    let mut t = targets(kirby_assist_core::native_states::ESCAPE_F, 10., 0.);
    t[1] = targets(kirby_assist_core::native_states::ESCAPE, 5., 0.)[0];
    let mut calls = 0;
    assert_eq!(
        simple_readiness(rules.simple_starts(), c, &t, || {
            calls += 1;
            true
        }),
        [true, true, false]
    );
    assert_eq!(calls, 1);
    assert_eq!(
        simple_readiness(rules.simple_starts(), c, &t, || false),
        [false; 3]
    );
    for t in [
        targets(kirby_assist_core::native_states::ESCAPE_F, 20.1, 0.),
        targets(kirby_assist_core::native_states::ESCAPE_F, 0., 8.1),
        targets(kirby_assist_core::native_states::ESCAPE_AIR, 5., 0.),
    ] {
        assert_eq!(
            simple_readiness(rules.simple_starts(), c, &t, || panic!(
                "out-of-range/unconfigured events perform no native readiness queries"
            )),
            [false; 3]
        );
    }
    for c in [
        GameplayContext { stopped: true, ..c },
        GameplayContext { situation: 2, ..c },
        GameplayContext {
            gameplay: false,
            ..c
        },
        GameplayContext {
            status: kirby_assist_core::native_states::status::DAMAGE_FLY_REFLECT_LR,
            ..c
        },
        GameplayContext {
            status: kirby_assist_core::native_states::status::CATCH_WAIT,
            ..c
        },
        GameplayContext {
            status: kirby_assist_core::native_states::status::DASH,
            ..c
        },
    ] {
        assert_eq!(
            simple_readiness(rules.simple_starts(), c, &t, || panic!(
                "invalid state or locomotion must not start a new ordinary punish"
            )),
            [false; 3]
        );
    }
}

#[test]
fn grab_backup_and_grab_dodge_floor_requirements_are_independent_of_auto_react() {
    for (index, event) in [
        (0, "auto_roll"),
        (1, "auto_spot_dodge"),
        (2, "auto_air_dodge"),
        (3, "auto_grab"),
        (4, "auto_shield_grab"),
        (5, "auto_grab_dodged"),
        (6, "auto_shield_backup"),
    ] {
        for actions in [
            "walk:opponent:4:50%,a:1",
            "run:opponent:4,a:1",
            "chase_if_far:4:1:3,a:1",
        ] {
            let mut rules = rules(&[(event, actions)]);
            rules.set_enabled_controls(0);
            assert_eq!(rules.automatic_ground_watch(0), None);
            assert!(!rules.automatic_needs_up_special(0x7f));
            assert_eq!(
                rules.automatic_ground_watch(1 << index),
                Some((20., 8.)),
                "{event}: {actions}"
            );
            assert_eq!(rules.automatic_ground_watch(0x7f ^ (1 << index)), None);
        }
    }
    let rules = rules(&[
        ("auto_roll", "up_b:4,drift:opponent:1:69"),
        ("auto_grab", "input:r:0:0:4"),
    ]);
    assert_eq!(
        rules.automatic_ground_watch(0x7f),
        None,
        "literal grab and spin drift need no floor probes"
    );
    assert!(rules.automatic_needs_up_special(1));
    assert!(!rules.automatic_needs_up_special(0x7e));
}

#[test]
fn unconfigured_fighters_do_not_construct_or_retain_a_roll_engine_during_grab_observations() {
    let grabs = rules(&[
        ("auto_grab", "input:r:0:0:4"),
        ("auto_shield_grab", "input:r:0:0:4"),
    ]);
    let mut engine = None;
    for _ in 0..600 {
        assert_eq!(
            sample_optional_simple(
                &mut engine,
                true,
                Some(&grabs),
                Default::default(),
                context(),
                targets(kirby_assist_core::native_states::GUARD, 5., 0.),
                None,
                [false; 3]
            )
            .owned,
            0
        );
        assert!(engine.is_none());
    }
    engine = Some(Box::<Engine>::default());
    sample_optional_simple(
        &mut engine,
        false,
        None,
        Default::default(),
        context(),
        [None; 8],
        None,
        [false; 3],
    );
    assert!(
        engine.is_none(),
        "removing a fighter's configuration drops its reaction engine"
    );
}

#[test]
fn ordinary_shield_backup_does_not_emit_up_b_or_require_its_native_transition() {
    use kirby_assist_core::out_of_shield::{AutomaticConfig, Context, OutOfShield};
    let ordinary = rules(&[("auto_shield_backup", "up_smash:4")]);
    let up_b = rules(&[("auto_shield_backup", "up_b:4")]);
    let profiles = FighterProfiles::default();
    for kind in 0..kirby_assist_core::fighter_kinds::PLAYABLE_KIND_COUNT {
        if profiles.profile(kind).is_none() {
            continue;
        }
        let c = Context {
            object: 1,
            kind,
            status: kirby_assist_core::native_states::GUARD,
            motion: 1,
            frame: 0.,
            rate: 1.,
            grounded: true,
            stopped: false,
            position: [0.; 2],
            facing: 1.,
            shield_ready: true,
            can_grab: false,
            can_jump: true,
            can_up_special: false,
            threatened: false,
        };
        let lc = GameplayContext {
            status: kirby_assist_core::native_states::GUARD,
            ..context()
        };
        let mut t = targets(0, -5., 0.);
        t[0].as_mut().unwrap().vulnerable_in = Some(0.);
        for (rules, expected) in [(&ordinary, MAPPER_A | MAPPER_ZR), (&up_b, 0)] {
            let mut engine = OutOfShield::new();
            let output = engine.sample_automatic_configured(
                true,
                c,
                lc,
                t,
                None,
                true,
                10.,
                20.,
                None,
                LedgeSettings {
                    fighter_kind: kind,
                    ..Default::default()
                },
                AutomaticConfig {
                    backup: rules.simple_sequence(6),
                    ..Default::default()
                },
            );
            assert_eq!(output.requested, expected, "fighter {kind}");
            assert_eq!(
                output.requested & MAPPER_B,
                0,
                "no placeholder Up-B leaks to the mapper"
            );
            assert!(
                !engine.shield_up_b_pending(),
                "ordinary responses never acquire Bowser's spin tracking"
            );
        }
        for blocked in [
            Context {
                can_jump: false,
                ..c
            },
            Context { stopped: true, ..c },
            Context {
                grounded: false,
                ..c
            },
            Context {
                status: kirby_assist_core::native_states::status::CATCH_WAIT,
                ..c
            },
        ] {
            let output = OutOfShield::new().sample_automatic_configured(
                true,
                blocked,
                lc,
                t,
                None,
                true,
                10.,
                20.,
                None,
                LedgeSettings {
                    fighter_kind: kind,
                    ..Default::default()
                },
                AutomaticConfig {
                    backup: ordinary.simple_sequence(6),
                    ..Default::default()
                },
            );
            assert_eq!(
                output.owned, 0,
                "unready/captured fighters must not release shield or receive a punish input"
            );
        }
    }
}

#[test]
fn configured_boxed_engine_is_reused_between_polls() {
    let configured = rules(&[("auto_roll", "down_smash:1")]);
    let mut engine: Option<Box<Engine>> = None;
    sample_optional_simple(
        &mut engine,
        false,
        Some(&configured),
        Default::default(),
        context(),
        [None; 8],
        None,
        [false; 3],
    );
    assert!(
        engine.is_none(),
        "disabled polling must not allocate an engine"
    );
    for _ in 0..1000 {
        sample_optional_simple(
            &mut engine,
            true,
            Some(&configured),
            Default::default(),
            context(),
            [None; 8],
            None,
            [false; 3],
        );
        assert!(
            engine.is_none(),
            "idle configured polling must not allocate an engine"
        );
    }
    engine = Some(Box::<Engine>::default());
    let original = engine.as_deref().unwrap() as *const Engine;
    for _ in 0..10 {
        sample_optional_simple(
            &mut engine,
            false,
            Some(&configured),
            Default::default(),
            context(),
            [None; 8],
            None,
            [false; 3],
        );
        assert_eq!(engine.as_deref().unwrap() as *const Engine, original);
    }
}

#[test]
fn profile_fallback_cannot_relabel_another_fighters_selected_tapes() {
    use kirby_assist_core::{
        fighter_profiles::FighterProfiles, ledge_trap::LedgeSettings, sequence_config::Catalog,
    };
    let profiles = FighterProfiles::default();
    let recipe = Catalog::parse(
        "[bowser.roll.private]\nstart_delay=0\nactions=up_b:4,drift:opponent:1:69\n",
    )
    .unwrap()
    .entries()[0]
        .sequence;
    let mut bowser = profiles.ledge_settings(Some(15), LedgeSettings::default());
    bowser.enabled = true;
    bowser.response_choices[2] = Some(1);
    bowser.sequence_overrides[2] = Some(recipe);
    assert_eq!(
        profiles.ledge_settings(Some(15), bowser).sequence_overrides[2],
        Some(recipe)
    );
    for kind in [
        None,
        Some(-1),
        Some(22),
        Some(5),
        Some(28),
        Some(90),
        Some(91),
    ] {
        let incoming = profiles.ledge_settings(kind, bowser);
        assert!(incoming.response_choices.iter().all(Option::is_none));
        assert!(incoming.sequence_overrides.iter().all(Option::is_none));
        if profiles.profile(kind.unwrap_or(-1)).is_none() {
            assert!(!incoming.enabled && incoming.implementation.is_none());
        }
    }
}
