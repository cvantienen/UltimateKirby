//! Configured defensive responses: native events gate a shared input timeline.
use kirby_assist_core::{
    action_owner::{ActionOwnership, Intent, Owner},
    defense_response::Response,
    fighter_profiles::presets::PresetSlot,
    fighter_tuning::Tuning,
    gameplay::{GameplayContext, MotionTiming, Opponent},
    native_input::*,
    out_of_shield::{Context, OutOfShield, Target},
    reaction_rules::{Candidate, Engine, Resolved, Trigger},
    sequence_config::{persist, Catalog},
    settings::AssistSettings,
    spot_dodge,
};
fn rules(trigger: &str, max: f32, side: &str, actions: &str) -> Resolved {
    let source=format!("[bowser.x.response]\nstart_delay=0\ndefault=false\nreaction={trigger}:any:0:{max}:20:{side}:1\nactions={actions}\n");
    let c = Catalog::parse(&source).unwrap();
    let mut r = Resolved::default();
    r.add(Candidate {
        slot: PresetSlot::X as usize,
        index: 0,
        sequence: c.entries()[0].sequence,
    })
    .unwrap();
    r
}
fn context(status: i32, frame: f32) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        status,
        situation: kirby_assist_core::native_states::situation::GROUND,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.,
        facing: 1.,
        position: [0.; 2],
        safe_ground: [true; 2],
        ..Default::default()
    }
}
fn native(c: GameplayContext, ready: bool) -> Context {
    Context {
        object: 1,
        kind: kirby_assist_core::fighter_kinds::BOWSER,
        status: c.status,
        motion: c.motion_kind,
        frame: c.motion_frame,
        rate: c.motion_rate,
        stopped: c.stopped,
        grounded: c.situation == kirby_assist_core::native_states::situation::GROUND,
        position: c.position,
        facing: c.facing,
        shield_ready: ready,
        can_grab: false,
        can_jump: false,
        can_up_special: false,
        threatened: false,
    }
}
fn enemy(x: f32, frame: f32) -> Opponent {
    Opponent {
        id: 2,
        status: kirby_assist_core::native_states::status::ATTACK,
        position: [x, 0.],
        motion: Some(MotionTiming {
            kind: 2,
            frame,
            rate: 1.,
            remaining: None,
        }),
        ..Default::default()
    }
}
fn sample(
    o: &mut OutOfShield,
    r: &Resolved,
    c: GameplayContext,
    ready: bool,
    x: f32,
) -> kirby_assist_core::gameplay::ControllerOutput {
    o.sample_configured(
        true,
        Some(r),
        AssistSettings::default(),
        native(c, ready),
        c,
        Some(Target {
            id: 2,
            position: [x, 0.],
            velocity: [0.; 2],
            grounded: true,
            dangerous: false,
            recovery: None,
        }),
        Some(enemy(x, c.motion_frame)),
    )
}
#[test]
fn universal_response_requires_confirmed_hit_then_runs_configured_up_b_once() {
    let r = rules("shield_hit", 12., "any", "up_b:4");
    let mut o = OutOfShield::new();
    assert_eq!(
        sample(
            &mut o,
            &r,
            context(kirby_assist_core::native_states::GUARD, 0.),
            true,
            6.
        )
        .owned,
        0
    );
    assert_eq!(
        sample(
            &mut o,
            &r,
            context(kirby_assist_core::native_states::GUARD_DAMAGE, 1.),
            false,
            6.
        )
        .owned,
        0
    );
    let c = context(kirby_assist_core::native_states::GUARD, 2.);
    assert_eq!(
        sample(&mut o, &r, c, true, 6.).requested & MAPPER_B,
        MAPPER_B
    );
    assert_eq!(
        sample(&mut o, &r, c, true, 6.).requested & MAPPER_B,
        MAPPER_B
    ); // duplicate pose
    for frame in 3..=5 {
        assert_ne!(
            sample(&mut o, &r, context(0x1dd, frame as f32), true, 6.).requested & MAPPER_B,
            0
        );
    }
    assert_eq!(
        sample(&mut o, &r, context(0x1dd, 6.), true, 6.).requested,
        0
    );
    assert_eq!(
        sample(
            &mut o,
            &r,
            context(kirby_assist_core::native_states::GUARD, 7.),
            true,
            6.
        )
        .owned,
        0
    );
}
#[test]
fn stun_hitstop_and_zero_rate_guard_do_not_launch_early_or_inject_extra_shield() {
    let r = rules("shield_hit", 12., "any", "up_b:4");
    let mut o = OutOfShield::new();
    for frame in 1..=20 {
        let mut c = context(kirby_assist_core::native_states::GUARD_DAMAGE, frame as f32);
        c.stopped = frame < 10;
        assert_eq!(sample(&mut o, &r, c, false, 6.).owned, 0);
    }
    let mut c = context(kirby_assist_core::native_states::GUARD, 21.);
    c.motion_rate = 0.;
    assert_ne!(sample(&mut o, &r, c, true, 6.).requested & MAPPER_B, 0);
}
#[test]
fn range_facing_and_height_are_recipe_conditions_and_misses_release_control() {
    let r = rules("shield_hit", 8., "front", "grab:4");
    for x in [-5., 9.] {
        let mut o = OutOfShield::new();
        sample(
            &mut o,
            &r,
            context(kirby_assist_core::native_states::GUARD_DAMAGE, 1.),
            false,
            x,
        );
        assert_eq!(
            sample(
                &mut o,
                &r,
                context(kirby_assist_core::native_states::GUARD, 2.),
                true,
                x
            )
            .owned,
            0
        );
        assert_eq!(
            sample(
                &mut o,
                &r,
                context(kirby_assist_core::native_states::GUARD, 3.),
                true,
                4.
            )
            .owned,
            0
        );
    }
    let mut response = Response::default();
    let mut e = enemy(4., 1.);
    e.position[1] = 21.;
    assert!(!response.start(&r, Trigger::ShieldHit, context(0, 1.), e));
}
#[test]
fn disabled_lost_attacker_and_ledge_preemption_cancel_pending_or_running_sequence() {
    let r = rules("shield_hit", 12., "any", "up_b:4");
    let mut o = OutOfShield::new();
    sample(
        &mut o,
        &r,
        context(kirby_assist_core::native_states::GUARD_DAMAGE, 1.),
        false,
        6.,
    );
    let c = context(kirby_assist_core::native_states::GUARD, 2.);
    assert_eq!(
        o.sample_configured(
            true,
            Some(&r),
            AssistSettings::default(),
            native(c, true),
            c,
            None,
            None
        )
        .owned,
        0
    );
    sample(
        &mut o,
        &r,
        context(kirby_assist_core::native_states::GUARD, 3.),
        true,
        6.,
    );
    sample(
        &mut o,
        &r,
        context(kirby_assist_core::native_states::GUARD_DAMAGE, 4.),
        false,
        6.,
    );
    assert_ne!(
        sample(
            &mut o,
            &r,
            context(kirby_assist_core::native_states::GUARD, 5.),
            true,
            6.
        )
        .owned,
        0
    );
    let mut owner = ActionOwnership::default();
    let intent = |who| Intent {
        owner: who,
        present: true,
        committed: true,
    };
    assert_eq!(
        owner.select(true, &[intent(Owner::Ledge), intent(Owner::OutOfShield)]),
        Some(Owner::Ledge)
    );
    assert_eq!(
        o.sample_configured(
            false,
            Some(&r),
            AssistSettings::default(),
            native(c, true),
            c,
            None,
            None
        )
        .owned,
        0
    );
    assert_eq!(o.target(), None);
}
#[test]
fn shared_sequence_supports_delays_holds_and_fresh_target_facing() {
    let r = rules("shield_hit", 20., "any", "wait:2, face:opponent, hold:a:3");
    let mut response = Response::default();
    assert!(response.start(&r, Trigger::ShieldHit, context(0, 0.), enemy(-5., 0.)));
    for frame in 0..2 {
        assert_eq!(
            response
                .sample(
                    true,
                    AssistSettings::default(),
                    context(0, frame as f32),
                    Some(enemy(-5., frame as f32))
                )
                .requested,
            0
        );
    }
    let out = response.sample(
        true,
        AssistSettings::default(),
        context(0, 2.),
        Some(enemy(-5., 2.)),
    );
    assert_eq!(out.requested & MAPPER_A, 0);
    assert!(out.stick.unwrap()[0] < 0.);
    let mut turned = context(0, 3.);
    turned.facing = -1.;
    assert_eq!(
        response
            .sample(
                true,
                AssistSettings::default(),
                turned,
                Some(enemy(-5., 3.))
            )
            .requested
            & MAPPER_A,
        MAPPER_A
    );
    assert!(response.attack_hold());
    response.sample(
        true,
        AssistSettings::default(),
        context(kirby_assist_core::native_states::THROWN, 4.),
        Some(enemy(-5., 4.)),
    );
    assert_eq!(response.target(), None);
}
fn grab() -> spot_dodge::Target {
    spot_dodge::Target {
        id: 2,
        status: kirby_assist_core::native_states::CATCH,
        motion: 7,
        frame: 4.,
        rate: 1.,
        position: [4., 0.],
        velocity: [0.; 2],
        facing: -1.,
        first: 7,
        last: 9,
        reach: 10.,
    }
}
fn dodge_context(status: i32, frame: f32) -> spot_dodge::Context {
    spot_dodge::Context {
        object: 1,
        status,
        situation: kirby_assist_core::native_states::situation::GROUND,
        motion: 1,
        frame,
        rate: 1.,
        stopped: false,
        can_spot_dodge: true,
        position: [0.; 2],
        velocity: [0.; 2],
    }
}
#[test]
fn ordinary_grab_defense_only_starts_when_standing_or_shielding() {
    for status in [
        kirby_assist_core::native_states::status::WAIT,
        kirby_assist_core::native_states::status::WALK,
        kirby_assist_core::native_states::status::DASH,
        kirby_assist_core::native_states::status::RUN,
        kirby_assist_core::native_states::status::TURN_DASH,
        kirby_assist_core::native_states::status::TURN_RUN,
        kirby_assist_core::native_states::GUARD_ON,
        kirby_assist_core::native_states::GUARD,
    ] {
        let mut s = spot_dodge::SpotDodge::default();
        let mut targets = [None; 8];
        targets[0] = Some(grab());
        let out = s.sample(true, dodge_context(status, 1.), targets);
        if matches!(
            status,
            kirby_assist_core::native_states::status::WAIT
                | kirby_assist_core::native_states::GUARD_ON
                | kirby_assist_core::native_states::GUARD
        ) {
            assert_eq!(out.requested & MAPPER_ZR, MAPPER_ZR);
            assert_eq!(out.stick, Some([0., -1.]));
        } else {
            assert_eq!(out.owned, 0);
        }
    }
}
#[test]
fn punish_requires_an_accepted_dodge_and_waits_until_actionable_then_runs_once() {
    let r = rules("grab_dodged", 12., "any", "up_b:4");
    let mut s = spot_dodge::SpotDodge::default();
    let mut targets = [None; 8];
    targets[0] = Some(grab());
    s.sample(true, dodge_context(0, 1.), targets);
    assert_eq!(
        s.punish_configured(
            true,
            true,
            Some(&r),
            AssistSettings::default(),
            context(0, 1.),
            Some(enemy(4., 1.))
        )
        .owned,
        0
    );
    s.sample(true, dodge_context(spot_dodge::ESCAPE, 1.), targets);
    assert_eq!(
        s.punish_configured(
            true,
            false,
            Some(&r),
            AssistSettings::default(),
            context(spot_dodge::ESCAPE, 1.),
            Some(enemy(4., 1.))
        )
        .owned,
        0
    );
    assert_ne!(
        s.punish_configured(
            true,
            true,
            Some(&r),
            AssistSettings::default(),
            context(0, 30.),
            Some(enemy(4., 30.))
        )
        .requested
            & MAPPER_B,
        0
    );
    s.punish_configured(
        true,
        true,
        Some(&r),
        AssistSettings::default(),
        context(0, 34.),
        Some(enemy(4., 34.)),
    );
    assert_eq!(
        s.punish_configured(
            true,
            true,
            Some(&r),
            AssistSettings::default(),
            context(0, 35.),
            Some(enemy(4., 35.))
        )
        .owned,
        0
    );
}
#[test]
fn forecast_is_bounded_and_fighter_settings_round_trip_new_defaults_as_booleans() {
    assert!(spot_dodge::contact_with_lookahead(dodge_context(0, 1.), grab(), 2.).is_none());
    assert!(spot_dodge::contact_with_lookahead(dodge_context(0, 1.), grab(), 6.).is_some());
    for value in [f32::NAN, 0., 9.] {
        assert!(spot_dodge::contact_with_lookahead(dodge_context(0, 1.), grab(), value).is_none());
    }
    let mut t = Tuning::default();
    t.set("grab_dodge_default", "true").unwrap();
    t.set("out_of_shield_default", "false").unwrap();
    t.set("grab_dodge_lookahead", "3.5").unwrap();
    let output = persist::update_tuning("", 15, t).unwrap();
    assert!(output.contains("grab_dodge_lookahead = 3.5"));
    assert_eq!(Catalog::parse(&output).unwrap().tuning[&15], t);
    assert!(t.grab_dodge_default(false));
    assert!(!t.oos_default());
    assert_eq!(t.dodge_lookahead(), 3.5);
}
#[test]
fn defensive_triggers_are_not_roll_rules_or_button_toggles() {
    let r = rules("shield_hit", 12., "any", "up_b:4");
    assert_eq!(r.controls, 0);
    assert!(r.has_defense(Trigger::ShieldHit));
    assert!(kirby_assist_core::reaction_rules::Rule::parse(
        "shield_hit:any:0:12:20:any:1:toggle_x"
    )
    .is_err());
    let mut engine = Engine::default();
    assert_eq!(
        engine
            .sample(
                true,
                &r,
                AssistSettings::default(),
                context(0, 1.),
                [None; 8],
                None
            )
            .owned,
        0
    );
    for kind in [0, 5, 6, 15, 0x5d] {
        let mut m = kirby_assist_core::settings_menu::MenuSelection::new(&Default::default());
        m.y_fighter_kind = kind;
        m.modes_menu = true;
        assert!(m.out_of_shield && m.grab_dodge);
        assert!(m.row_visible(kirby_assist_core::settings_menu::MenuRow::OutOfShield));
    }
}

#[test]
fn startup_and_edited_metadata_keep_defensive_recipes_without_reserving_buttons() {
    use kirby_assist_core::{fighter_profiles::FighterProfiles, settings_menu::MenuSelection};
    let text="[bowser.x.response]\nstart_delay=0\ndefault=false\nreaction=shield_hit:any:0:12:18:any:1\nactions=up_b:4\n";
    let mut profiles = FighterProfiles::default();
    profiles
        .add_sequence_choices(&Catalog::parse(text).unwrap())
        .unwrap();
    let p = profiles.profile(15).unwrap().menu_options;
    let mut m = MenuSelection::new(&p);
    m.input(0, &p);
    assert!(m.has_reactions);
    assert_eq!(m.reaction_enabled_controls() & !1, 0);
    let r = m.resolve_reactions(&p).unwrap();
    assert!(r.has_defense(Trigger::ShieldHit));
    assert_eq!(r.controls, 0);
    assert_eq!(
        m.choice(&p, PresetSlot::X, m.choices[PresetSlot::X as usize])
            .unwrap()
            .id,
        kirby_assist_core::button_bindings::Builtin::default_for(PresetSlot::X).choice_id()
    );
}

#[test]
fn native_actionable_dodge_cancel_can_start_without_waiting_for_idle() {
    let r = rules("grab_dodged", 12., "any", "up_b:4");
    let mut s = spot_dodge::SpotDodge::default();
    let mut targets = [None; 8];
    targets[0] = Some(grab());
    s.sample(true, dodge_context(0, 1.), targets);
    s.sample(true, dodge_context(spot_dodge::ESCAPE, 1.), targets);
    let c = context(spot_dodge::ESCAPE, 18.);
    assert_ne!(
        s.punish_configured(
            true,
            true,
            Some(&r),
            AssistSettings::default(),
            c,
            Some(enemy(4., 18.))
        )
        .requested
            & MAPPER_B,
        0
    );
}

#[test]
fn new_grab_dodge_cancels_old_followup_instead_of_advancing_it_hidden() {
    let r = rules("grab_dodged", 12., "any", "hold:a:20");
    let mut s = spot_dodge::SpotDodge::default();
    let mut targets = [None; 8];
    targets[0] = Some(grab());
    s.sample(true, dodge_context(0, 1.), targets);
    s.sample(true, dodge_context(spot_dodge::ESCAPE, 1.), targets);
    assert_ne!(
        s.punish_configured(
            true,
            true,
            Some(&r),
            AssistSettings::default(),
            context(0, 30.),
            Some(enemy(4., 30.))
        )
        .requested
            & MAPPER_A,
        0
    );
    assert!(s.attack_hold());
    targets[0] = Some(spot_dodge::Target { id: 3, ..grab() });
    assert_ne!(
        s.sample(true, dodge_context(0, 31.), targets).requested & MAPPER_ZR,
        0
    );
    let old = s.punish_configured(
        true,
        true,
        Some(&r),
        AssistSettings::default(),
        context(0, 31.),
        Some(enemy(4., 31.)),
    );
    assert_eq!(old.owned, 0);
    assert!(!s.attack_hold());
    // If the new request is then preempted, the old hold cannot resume.
    s.cancel_pending();
    assert_eq!(
        s.punish_configured(
            true,
            true,
            Some(&r),
            AssistSettings::default(),
            context(0, 32.),
            Some(enemy(4., 32.))
        )
        .owned,
        0
    );
}

#[test]
fn renewed_shield_hit_cancels_old_up_b_and_arms_the_current_attacker() {
    for attacker_id in [2, 3] {
        let r = rules("shield_hit", 12., "any", "up_b:4");
        let mut o = OutOfShield::new();
        sample(
            &mut o,
            &r,
            context(kirby_assist_core::native_states::GUARD_DAMAGE, 1.),
            false,
            6.,
        );
        assert_ne!(
            sample(
                &mut o,
                &r,
                context(kirby_assist_core::native_states::GUARD, 2.),
                true,
                6.
            )
            .requested
                & MAPPER_B,
            0
        );
        let mut c = context(kirby_assist_core::native_states::GUARD_DAMAGE, 3.);
        c.stopped = true;
        let attacker = Target {
            id: attacker_id,
            position: [5., 0.],
            velocity: [0.; 2],
            grounded: true,
            dangerous: false,
            recovery: None,
        };
        let other = Opponent {
            id: attacker_id,
            ..enemy(5., 3.)
        };
        let interrupted = o.sample_configured(
            true,
            Some(&r),
            AssistSettings::default(),
            native(c, false),
            c,
            Some(attacker),
            Some(other),
        );
        assert_eq!(
            interrupted.owned, 0,
            "old Up-B must not continue during renewed shieldstun"
        );
        assert_eq!(o.target(), Some(attacker_id));
        c = context(kirby_assist_core::native_states::GUARD, 4.);
        let ready = o.sample_configured(
            true,
            Some(&r),
            AssistSettings::default(),
            native(c, true),
            c,
            Some(attacker),
            Some(Opponent {
                id: attacker_id,
                ..enemy(5., 4.)
            }),
        );
        assert_ne!(ready.requested & MAPPER_B, 0);
        assert_eq!(o.target(), Some(attacker_id));
    }
}
