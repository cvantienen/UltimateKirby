//! A missing fighter/event recipe is inert. Only ordinary grabs stay generic.
use kirby_assist_core::{
    fighter_profiles::FighterProfiles,
    gameplay::GameplayContext,
    settings_menu::MenuSelection,
    ledge_trap::LedgeSettings,
    native_input::*,
    out_of_shield::{AutomaticConfig, Context, OutOfShield},
    reaction_rules::{Engine, Resolved},
    roll_read::Target,
    sequence_config::Catalog,
};
fn own(kind: i32, status: i32) -> Context {
    Context {
        object: 1,
        kind,
        status,
        motion: 1,
        frame: 0.,
        rate: 1.,
        grounded: true,
        stopped: false,
        position: [0.; 2],
        facing: 1.,
        shield_ready: true,
        can_grab: true,
        can_jump: true,
        can_up_special: true,
        threatened: false,
    }
}
fn lc(c: Context) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        can_act: true,
        status: c.status,
        motion_kind: 1,
        motion_frame: c.frame,
        motion_rate: 1.,
        position: c.position,
        facing: 1.,
        safe_ground: [true; 2],
        ..Default::default()
    }
}
fn target(status: i32, x: f32) -> [Option<Target>; 8] {
    let mut t = [None; 8];
    t[0] = Some(Target {
        id: 2,
        status,
        motion: 2,
        frame: 0.,
        rate: 1.,
        position: [x, 0.],
        velocity_x: 0.,
        vulnerable_in: Some(0.),
        actionable_in: None,
    });
    t
}
fn settings(kind: i32) -> LedgeSettings {
    LedgeSettings {
        fighter_kind: kind,
        ..Default::default()
    }
}
#[test]
fn generic_templates_never_become_roll_dodge_or_grab_followups_and_fighter_grab_ranges_are_kept() {
    let saved = Catalog::parse(include_str!(
        "../../../deployment/ultimate/kirby_assist/assembled/sequences.txt"
    ))
    .unwrap();
    let templates_only =
        Catalog::parse("[template.spin]\nstart_delay=0\nactions=up_b:4\n").unwrap();
    for c in [saved, templates_only] {
        let mut profiles = FighterProfiles::default();
        profiles.add_sequence_choices(&c).unwrap();
        for kind in 0..kirby_assist_core::fighter_kinds::PLAYABLE_KIND_COUNT {
            let Some(p) = profiles.profile(kind) else {
                continue;
            };
            let mut m = MenuSelection::new(&p.menu_options);
            m.simplified_reactions = true;
            let rules = m.resolve_reactions(&p.menu_options).unwrap();
            for event in 0..7 {
                let authored = c.entries().iter().find(|entry| {
                    entry.fighter_kind == kind
                        && entry.sequence.reaction.trigger.simple() == Some(event)
                });
                assert_eq!(
                    rules.simple_sequence(event).copied(),
                    authored.map(|entry| entry.sequence),
                    "fighter {kind}, event {event}"
                );
            }
            for (event, status) in [
                (0, kirby_assist_core::native_states::ESCAPE_F),
                (1, kirby_assist_core::native_states::ESCAPE),
                (2, 0x22),
            ] {
                if rules.simple_sequence(event).is_none() {
                    assert_eq!(
                        Engine::default()
                            .sample_simple(
                                true,
                                &rules,
                                settings(kind),
                                lc(own(kind, 0)),
                                target(status, 5.),
                                None
                            )
                            .owned,
                        0
                    );
                }
            }
        }
    }
}
#[test]
fn unconfigured_fighters_never_get_up_b_from_generic_grab_backup_even_with_empty_rules() {
    let profiles = FighterProfiles::default();
    for kind in 0..kirby_assist_core::fighter_kinds::PLAYABLE_KIND_COUNT {
        let Some(p) = profiles.profile(kind) else {
            continue;
        };
        let (close, far) = kirby_assist_core::fighter_tuning::Tuning::default()
            .auto_grab_ranges(kind, p.ledge.grab.standing_range);
        for enemy in [
            0,
            kirby_assist_core::native_states::ESCAPE,
            kirby_assist_core::native_states::ESCAPE_F,
            kirby_assist_core::native_states::ESCAPE_AIR,
        ] {
            for x in [-close * 0.5, 0., close + 1.] {
                let mut o = OutOfShield::new();
                let c = own(kind, kirby_assist_core::native_states::GUARD);
                let out = o.sample_automatic(
                    true,
                    c,
                    lc(c),
                    target(enemy, x),
                    None,
                    true,
                    close,
                    far,
                    Some(8.),
                );
                assert_eq!(
                    out.requested & MAPPER_B,
                    0,
                    "fighter {kind}, enemy {enemy:#x}, x {x}"
                );
                assert!(!o.shield_up_b_pending());
            }
        }
        let mut o = OutOfShield::new();
        let c = own(kind, kirby_assist_core::native_states::GUARD);
        let grab = o
            .sample_automatic(
                true,
                c,
                lc(c),
                target(0, close * 0.5),
                None,
                true,
                close,
                far,
                Some(8.),
            )
            .requested
            & MAPPER_R;
        if close > 0.0 {
            assert_ne!(grab, 0, "front vulnerable shield grab, fighter {kind}");
        } else {
            assert_eq!(grab, 0, "missing reach disables grab, fighter {kind}");
        }
        let mut o = OutOfShield::new();
        let c = own(kind, 3);
        let grab = o
            .sample_automatic(
                true,
                c,
                lc(c),
                target(kirby_assist_core::native_states::GUARD, close * 0.5),
                None,
                false,
                close,
                far,
                Some(8.),
            )
            .requested
            & MAPPER_R;
        if close > 0.0 {
            assert_ne!(grab, 0, "moving grab vs shield, fighter {kind}");
        } else {
            assert_eq!(grab, 0, "missing reach disables grab, fighter {kind}");
        }
    }
    assert!(!Resolved::default().has_simple_events());
}
#[test]
fn yoshi_and_lucina_keep_saved_grab_limits_and_only_their_explicit_backup_metadata() {
    let file = Catalog::parse(include_str!(
        "../../../deployment/ultimate/kirby_assist/assembled/sequences.txt"
    ))
    .unwrap();
    let mut profiles = FighterProfiles::default();
    profiles.add_sequence_choices(&file).unwrap();
    let mut seen = Vec::new();
    for kind in [5, 22] {
        let p = profiles.profile(kind).unwrap();
        let mut m = MenuSelection::new(&p.menu_options);
        m.simplified_reactions = true;
        let r = m.resolve_reactions(&p.menu_options).unwrap();
        let g = r.simple_sequence(3).unwrap();
        let s = r.simple_sequence(4).unwrap();
        seen.push((g.reaction.max, s.reaction.max));
        let config = AutomaticConfig {
            grab: Some(g),
            shield_grab: Some(s),
            backup: r.simple_sequence(6),
        };
        // A saved explicit backup remains fighter configuration; global templates
        // must never supply or replace it. Some imported catalogs already contain one.
        let authored = file.entries().iter().find(|e| {
            e.fighter_kind == kind
                && e.sequence.reaction.trigger
                    == kirby_assist_core::reaction_rules::Trigger::AutoShieldBackup
        });
        assert_eq!(config.backup.copied(), authored.map(|e| e.sequence));
        let (close, far) = kirby_assist_core::fighter_tuning::Tuning::default()
            .auto_grab_ranges(kind, p.ledge.grab.standing_range);
        for (distance, want_grab) in [(s.reaction.max * 0.5, true), (s.reaction.max + 0.1, false)] {
            let c = own(kind, kirby_assist_core::native_states::GUARD);
            let out = OutOfShield::new().sample_automatic_configured(
                true,
                c,
                lc(c),
                target(0, distance),
                None,
                true,
                close,
                far,
                Some(8.),
                settings(kind),
                config,
            );
            assert_eq!(
                out.requested & MAPPER_R != 0,
                want_grab,
                "fighter {kind}, distance {distance}"
            );
            assert_eq!(out.requested & MAPPER_B, 0);
        }
    }
    assert_ne!(
        seen[0], seen[1],
        "fighters retain their independent configured ranges"
    );
}
#[test]
fn deleting_a_configured_backup_stops_its_active_spin_without_another_attack_or_grab() {
    let b=Catalog::parse("[bowser.roll.backup]\nstart_delay=0\nreaction=auto_shield_backup:25:24\nactions=up_b:4,drift:opponent:1:69\n").unwrap().entries()[0].sequence;
    let mut o = OutOfShield::new();
    let c = own(15, kirby_assist_core::native_states::GUARD);
    let ts = target(0, -5.);
    assert_ne!(
        o.sample_automatic_configured(
            true,
            c,
            lc(c),
            ts,
            None,
            true,
            25.,
            35.,
            Some(8.),
            settings(15),
            AutomaticConfig {
                backup: Some(&b),
                ..Default::default()
            }
        )
        .requested
            & MAPPER_B,
        0
    );
    let c = Context {
        status: kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_G,
        frame: 1.,
        ..c
    };
    let out = o.sample_automatic_configured(
        true,
        c,
        lc(c),
        ts,
        None,
        false,
        25.,
        35.,
        Some(8.),
        settings(15),
        AutomaticConfig::default(),
    );
    assert_eq!(out.owned, 0);
    assert!(!o.shield_up_b_pending());
}
