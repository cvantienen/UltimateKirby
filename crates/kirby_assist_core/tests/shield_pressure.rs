use kirby_assist_core::{
    fighter_profiles::{presets::PresetSlot, FighterProfiles},
    gameplay::{GameplayContext, MotionTiming, Opponent},
    settings_menu::{MenuRow, MenuSelection},
    ledge_trap::LedgeSettings,
    native_input::*,
    reaction_rules::{Candidate, Resolved, Trigger},
    roll_read::Target,
    sequence_config::{persist, Catalog},
    shield_pressure::Pressure,
};
fn rules(text: &str) -> Resolved {
    let c = Catalog::parse(text).unwrap();
    let mut r = Resolved::default();
    for (i, e) in c.entries().iter().enumerate() {
        r.add(Candidate {
            slot: PresetSlot::Attack as usize,
            index: i,
            sequence: e.sequence,
        })
        .unwrap();
    }
    r
}
fn ctx(f: f32) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        can_act: true,
        motion_kind: 1,
        motion_frame: f,
        motion_rate: 1.,
        facing: 1.,
        safe_ground: [true; 2],
        ..Default::default()
    }
}
fn enemy(status: i32, x: f32, f: f32) -> Target {
    Target {
        id: 2,
        status,
        motion: status as u64 + 10,
        frame: f,
        rate: 1.,
        position: [x, 0.],
        velocity_x: 0.,
        vulnerable_in: Some(0.),
        actionable_in: Some(12.),
    }
}
fn roster(t: Target) -> [Option<Target>; 8] {
    [Some(t), None, None, None, None, None, None, None]
}
fn sample(
    p: &mut Pressure,
    r: &Resolved,
    f: f32,
    t: Target,
) -> kirby_assist_core::gameplay::ControllerOutput {
    p.sample(
        true,
        true,
        1,
        Some(r),
        LedgeSettings::default(),
        ctx(f),
        roster(t),
        Some(Opponent {
            id: t.id,
            status: t.status,
            position: t.position,
            motion: Some(MotionTiming {
                kind: t.motion,
                frame: t.frame,
                rate: t.rate,
                remaining: None,
            }),
            ..Default::default()
        }),
    )
}
const HOLD:&str="[bowser.attack.shield_grab]\nstart_delay=0\nreaction=enemy_shield:grounded:0:10:12:front:1\nactions=hold:r:1\n";
const ROLL:&str="[bowser.attack.shield_smash]\nstart_delay=0\nreaction=shield_roll_cross_vulnerable:grounded:0:20:12:any:1\nactions=down_smash:1\n";
#[test]
fn nearby_shield_grab_fires_once_per_shield_visit_and_never_for_an_ordinary_roll() {
    let r = rules(HOLD);
    let mut p = Pressure::default();
    assert_eq!(
        sample(
            &mut p,
            &r,
            0.,
            enemy(kirby_assist_core::native_states::GUARD_ON, 6., 0.)
        )
        .requested
            & MAPPER_R,
        MAPPER_R
    );
    sample(
        &mut p,
        &r,
        1.,
        enemy(kirby_assist_core::native_states::GUARD, 6., 0.),
    );
    assert_eq!(
        sample(
            &mut p,
            &r,
            2.,
            enemy(kirby_assist_core::native_states::GUARD, 6., 0.)
        )
        .requested,
        0
    );
    sample(
        &mut p,
        &r,
        3.,
        enemy(kirby_assist_core::native_states::ESCAPE_F, 6., 1.),
    );
    assert!(!p.claims_roll(2)); // No shield-roll recipe: generic Roll remains available.
    assert_eq!(
        sample(
            &mut p,
            &r,
            4.,
            enemy(kirby_assist_core::native_states::GUARD_ON, 6., 0.)
        )
        .requested
            & MAPPER_R,
        MAPPER_R
    );
}
#[test]
fn crossing_requires_shield_origin_and_vulnerability_and_claims_only_that_roll() {
    let r = rules(ROLL);
    let mut p = Pressure::default();
    assert_eq!(
        sample(
            &mut p,
            &r,
            0.,
            enemy(kirby_assist_core::native_states::ESCAPE_F, 5., 1.)
        )
        .owned,
        0
    );
    assert!(!p.claims_roll(2));
    sample(
        &mut p,
        &r,
        1.,
        enemy(kirby_assist_core::native_states::GUARD, 5., 1.),
    );
    sample(
        &mut p,
        &r,
        2.,
        enemy(kirby_assist_core::native_states::GUARD_OFF, 5., 0.),
    ); // Shield-drop phase retains provenance.
    let t = Target {
        vulnerable_in: Some(4.),
        ..enemy(kirby_assist_core::native_states::ESCAPE_F, -5., 1.)
    };
    assert_eq!(sample(&mut p, &r, 3., t).requested, 0);
    assert!(p.claims_roll(2));
    let t = Target { frame: 2., ..t };
    assert_eq!(sample(&mut p, &r, 4., t).requested, 0);
    let t = Target {
        frame: 3.,
        vulnerable_in: Some(0.),
        ..t
    };
    assert_ne!(sample(&mut p, &r, 5., t).requested & MAPPER_A, 0);
    sample(&mut p, &r, 6., Target { frame: 4., ..t });
    assert_eq!(
        sample(&mut p, &r, 7., Target { frame: 5., ..t }).requested,
        0
    );
}
#[test]
fn range_priority_side_native_readiness_and_disabled_scope_are_respected() {
    let r=rules(&format!("{HOLD}[bowser.attack.shield_far]\nstart_delay=0\nreaction=enemy_shield:grounded:10:25:12:front:2\nactions=up_b:1\n"));
    let mut p = Pressure::default();
    let t = enemy(kirby_assist_core::native_states::GUARD, 16., 1.);
    let out = p.sample(
        true,
        true,
        1,
        Some(&r),
        LedgeSettings::default(),
        GameplayContext {
            can_act: false,
            ..ctx(0.)
        },
        roster(t),
        None,
    );
    assert_eq!(out.owned, 0);
    assert_ne!(sample(&mut p, &r, 1., t).requested & MAPPER_B, 0);
    assert_eq!(
        p.sample(
            false,
            true,
            1,
            Some(&r),
            LedgeSettings::default(),
            ctx(2.),
            roster(t),
            None
        )
        .owned,
        0
    );
    assert!(!p.committed());
    let mut p = Pressure::default();
    assert_eq!(
        sample(
            &mut p,
            &r,
            1.,
            enemy(kirby_assist_core::native_states::GUARD, -6., 1.)
        )
        .owned,
        0
    );
    assert_eq!(
        sample(
            &mut p,
            &r,
            2.,
            enemy(kirby_assist_core::native_states::GUARD, 30., 2.)
        )
        .owned,
        0
    );
    assert_eq!(
        p.sample(
            true,
            false,
            1,
            Some(&r),
            LedgeSettings::default(),
            ctx(3.),
            roster(t),
            None
        )
        .owned,
        0
    );
}
#[test]
fn pressure_rules_save_separately_and_roundtrip_without_becoming_ledge_choices() {
    let mut profiles = FighterProfiles::default();
    let c = Catalog::parse(HOLD).unwrap();
    profiles.add_sequence_choices(&c).unwrap();
    let presets = profiles.profile(15).unwrap().menu_options;
    let mut menu = MenuSelection::new(&presets);
    menu.y_fighter_kind = 15;
    menu.show_modes();
    menu.row = MenuRow::ShieldPressure;
    menu.input(0, &presets);
    menu.input(kirby_assist_core::settings_menu::MENU_STICK_CLICK, &presets);
    assert!(menu.assist_rule_page());
    assert!(menu
        .recipe_panel(&presets)
        .header
        .join(" ")
        .contains("Shield pressure"));
    let entry = &c.entries()[0];
    let saved =
        persist::update(HOLD, 15, entry.slot, &entry.choice, entry.sequence, false).unwrap();
    let parsed = Catalog::parse(&saved).unwrap();
    assert_eq!(
        parsed.entries()[0].sequence.reaction.trigger,
        Trigger::EnemyShield
    );
    assert!(kirby_assist_core::sequence_config::layout::organize(&saved)
        .unwrap()
        .contains("ASSIST RULES: SHIELD PRESSURE"));
}
#[test]
fn roll_away_direction_is_observed_and_does_not_flip_after_a_roll_crosses() {
    let r=rules("[bowser.attack.away]\nstart_delay=0\nreaction=shield_roll_away:grounded:10:30:12:any:1\nactions=up_b:1\n");
    let mut p = Pressure::default();
    sample(
        &mut p,
        &r,
        0.,
        enemy(kirby_assist_core::native_states::GUARD, 16., 0.),
    );
    assert_eq!(
        sample(
            &mut p,
            &r,
            1.,
            enemy(kirby_assist_core::native_states::ESCAPE_F, 16., 0.)
        )
        .owned,
        0
    ); // No guessed direction at zero speed.
    let away = Target {
        velocity_x: 2.,
        ..enemy(kirby_assist_core::native_states::ESCAPE_F, 18., 1.)
    };
    assert_ne!(sample(&mut p, &r, 2., away).requested & MAPPER_B, 0);
    let mut p = Pressure::default();
    sample(
        &mut p,
        &r,
        0.,
        enemy(kirby_assist_core::native_states::GUARD, 16., 0.),
    );
    assert_eq!(
        sample(
            &mut p,
            &r,
            1.,
            Target {
                velocity_x: -2.,
                ..enemy(kirby_assist_core::native_states::ESCAPE_F, 14., 1.)
            }
        )
        .owned,
        0
    );
    assert_eq!(
        sample(
            &mut p,
            &r,
            2.,
            Target {
                velocity_x: -2.,
                ..enemy(kirby_assist_core::native_states::ESCAPE_F, -14., 2.)
            }
        )
        .owned,
        0
    );
}
#[test]
fn hit_on_our_shield_cancels_pressure_without_replaying_the_old_action() {
    let r = rules(HOLD);
    let mut p = Pressure::default();
    assert_ne!(
        sample(
            &mut p,
            &r,
            0.,
            enemy(kirby_assist_core::native_states::GUARD, 6., 0.)
        )
        .owned,
        0
    );
    let blocked = GameplayContext {
        status: kirby_assist_core::native_states::GUARD_DAMAGE,
        stopped: true,
        ..ctx(1.)
    };
    assert_eq!(
        p.sample(
            true,
            true,
            1,
            Some(&r),
            LedgeSettings::default(),
            blocked,
            roster(enemy(kirby_assist_core::native_states::GUARD, 6., 1.)),
            None
        )
        .owned,
        0
    );
    assert!(!p.committed());
    assert_eq!(
        sample(
            &mut p,
            &r,
            2.,
            enemy(kirby_assist_core::native_states::GUARD, 6., 2.)
        )
        .owned,
        0
    );
}

#[test]
fn ordinary_roll_engine_does_not_steal_a_claimed_shield_roll() {
    use kirby_assist_core::reaction_rules::Engine;
    let r = rules(&format!("{ROLL}[bowser.roll.ordinary]\nstart_delay=0\nreaction=roll:grounded:0:20:12:any:1\nactions=up_b:1\n"));
    let mut p = Pressure::default();
    let mut generic = Engine::default();
    sample(
        &mut p,
        &r,
        0.,
        enemy(kirby_assist_core::native_states::GUARD, 5., 0.),
    );
    let shield_roll = Target {
        vulnerable_in: Some(4.),
        ..enemy(kirby_assist_core::native_states::ESCAPE_F, 4., 1.)
    };
    assert_eq!(sample(&mut p, &r, 1., shield_roll).owned, 0);
    assert!(p.claims_roll(2));
    let filtered = roster(shield_roll).map(|t| t.filter(|t| !p.claims_roll(t.id)));
    assert_eq!(
        generic
            .sample(true, &r, LedgeSettings::default(), ctx(1.), filtered, None)
            .owned,
        0
    );
    // A different fighter's ordinary roll is still available in the same frame.
    let ordinary = Target {
        id: 3,
        ..shield_roll
    };
    let filtered = roster(ordinary).map(|t| t.filter(|t| !p.claims_roll(t.id)));
    assert_ne!(
        generic
            .sample(true, &r, LedgeSettings::default(), ctx(2.), filtered, None)
            .requested
            & MAPPER_B,
        0
    );
}

#[test]
fn pressure_variants_share_one_eight_rule_budget_and_keep_menu_control() {
    let mut r = Resolved::default();
    let sequence = Catalog::parse(HOLD).unwrap().entries()[0].sequence;
    for i in 0..8 {
        let mut s = sequence;
        s.reaction.trigger = Trigger::SHIELD_EVENTS[i % Trigger::SHIELD_EVENTS.len()];
        s.reaction.priority = (i + 1) as u8;
        r.add(Candidate {
            slot: PresetSlot::Attack as usize,
            index: i,
            sequence: s,
        })
        .unwrap();
    }
    assert!(r
        .add(Candidate {
            slot: 0,
            index: 9,
            sequence
        })
        .is_err());
    assert!(Catalog::parse("[bowser.attack.bad]\nreaction=enemy_shield:grounded:0:10:12:front:1:toggle_x\nactions=hold:r:4\n").is_err());
}
