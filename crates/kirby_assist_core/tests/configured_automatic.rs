//! Configuration parity and safety for file-authored automatic responses.
use kirby_assist_core::{
    action_owner::{ActionOwnership, Intent, Owner},
    fighter_profiles::FighterProfiles,
    ledge_trap::{LedgeContext, LedgeSettings, MotionTiming, Opponent},
    native_input::{MAPPER_A, MAPPER_B, MAPPER_R, MAPPER_ZR},
    out_of_shield::{AutomaticConfig, Context, OutOfShield},
    reaction_rules::{Candidate, Engine, Resolved, Trigger},
    roll_read::Target,
    sequence_config::{Catalog, Sequence},
    spot_dodge::{Context as DodgeContext, SpotDodge, Target as GrabTarget},
};
fn recipe(event: &str, x: f32, y: f32, actions: &str) -> Sequence {
    Catalog::parse(&format!("[bowser.roll.test]\nstart_delay=0\ntimeout=240\nreaction={event}:{x}:{y}\nactions={actions}\n")).unwrap().entries()[0].sequence
}
fn own(kind: i32, status: i32, frame: f32) -> Context {
    Context {
        object: 1,
        kind,
        status,
        motion: 1,
        frame,
        rate: 1.,
        stopped: false,
        grounded: true,
        position: [0.; 2],
        facing: 1.,
        shield_ready: true,
        can_grab: true,
        can_jump: false,
        can_up_special: true,
        threatened: false,
    }
}
fn lc(c: Context) -> LedgeContext {
    LedgeContext {
        held: true,
        gameplay: true,
        can_act: true,
        status: c.status,
        motion_kind: c.motion,
        motion_frame: c.frame,
        motion_rate: c.rate,
        stopped: c.stopped,
        situation: if c.grounded { 0 } else { 2 },
        position: c.position,
        facing: c.facing,
        safe_ground: [true; 2],
        ..Default::default()
    }
}
fn targets(status: i32, x: f32, y: f32) -> [Option<Target>; 8] {
    let mut out = [None; 8];
    out[0] = Some(Target {
        id: 2,
        status,
        motion: 2,
        frame: 0.,
        rate: 1.,
        position: [x, y],
        velocity_x: 0.,
        vulnerable_in: Some(0.),
        actionable_in: None,
    });
    out
}
fn settings(kind: i32) -> LedgeSettings {
    LedgeSettings {
        fighter_kind: kind,
        ..Default::default()
    }
}
fn config<'a>(g: &'a Sequence, s: &'a Sequence, b: &'a Sequence) -> AutomaticConfig<'a> {
    AutomaticConfig {
        grab: Some(g),
        shield_grab: Some(s),
        backup: Some(b),
    }
}

#[test]
fn all_seven_events_roundtrip_and_reject_duplicate_recipes_in_any_slot() {
    let mut rules = Resolved::default();
    for (i, t) in Trigger::AUTO_EVENTS.into_iter().enumerate() {
        let s = recipe(t.token(), 35., 12., "up_b:4");
        assert_eq!(s.reaction.trigger, t);
        assert_eq!(s.reaction.token(), format!("{}:35:12", t.token()));
        rules
            .add(Candidate {
                slot: 2,
                index: i,
                sequence: s,
            })
            .unwrap();
        assert_eq!(rules.simple_sequence(i).unwrap().reaction, s.reaction);
        assert!(rules
            .add(Candidate {
                slot: 3,
                index: 99,
                sequence: s
            })
            .is_err());
    }
    assert_eq!(rules.len, 7);
}

#[test]
fn canonical_grab_recipes_preserve_generic_grab_outputs_for_every_fighter() {
    let profiles = FighterProfiles::default();
    for kind in 0..0x5e {
        let Some(p) = profiles.profile(kind) else {
            continue;
        };
        let (close, far) = kirby_assist_core::fighter_tuning::Tuning::default()
            .auto_grab_ranges(kind, p.ledge.grab.standing_range);
        let g = recipe("auto_grab", far, 24., "input:r:0:0:4");
        let s = recipe("auto_shield_grab", close, 24., "input:r:0:0:4");
        for status in [0, 1, 3, 4, 0x1b, 0x1c] {
            for enemy in [0, 0x1c, 0x20] {
                for x in [-far, -1., 1., close, close + 0.1, far, far + 0.1] {
                    for y in [0., 24., 24.1] {
                        let mut old = OutOfShield::new();
                        let mut new = OutOfShield::new();
                        let shielded = matches!(status, 0x1b | 0x1c);
                        let ts = targets(enemy, x, y);
                        for tick in 0..6 {
                            let c = own(kind, status, tick as f32);
                            let a = old.sample_automatic(
                                true,
                                c,
                                lc(c),
                                ts,
                                None,
                                shielded,
                                close,
                                far,
                                Some(12.),
                            );
                            let b = new.sample_automatic_configured(
                                true,
                                c,
                                lc(c),
                                ts,
                                None,
                                shielded,
                                close,
                                far,
                                Some(12.),
                                settings(kind),
                                AutomaticConfig {grab:Some(&g),shield_grab:Some(&s),backup:None},
                            );
                            assert_eq!(a,b,"kind {kind} own {status:#x} enemy {enemy:#x} x {x} y {y} tick {tick}");
                            assert_eq!(old.target(), new.target());
                            assert_eq!(old.approach_pending(), new.approach_pending());
                            assert_eq!(old.shield_up_b_pending(), new.shield_up_b_pending());
                            assert_eq!(
                                old.with_grab_movement(a, [0.7, 0.]),
                                new.with_grab_movement(b, [0.7, 0.])
                            );
                            for tool in [&mut old, &mut new] {
                                if a.requested & MAPPER_R != 0 {
                                    tool.mapped_grab(tick, 2);
                                } else if a.stick.is_some_and(|s| s[0] != 0.) {
                                    tool.mapped_approach(tick, 2);
                                }
                                if a.requested & MAPPER_B != 0 {
                                    tool.mapped_up_special(tick, 2);
                                }
                                tool.delivery_tick(tick);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn an_edited_grab_recipe_replaces_the_final_input_without_an_extra_grab_or_up_b() {
    let g = recipe("auto_grab", 35., 24., "input:a:0:0:2");
    let mut o = OutOfShield::new();
    let c = own(15, 3, 0.);
    let first = o.sample_automatic_configured(
        true,
        c,
        lc(c),
        targets(0x1c, 10., 0.),
        None,
        false,
        20.,
        35.,
        Some(8.),
        settings(15),
        AutomaticConfig {
            grab: Some(&g),
            ..Default::default()
        },
    );
    assert_eq!(first.requested, MAPPER_A);
    assert!(o.configured_committed());
    assert!(!o.automatic_grab_pending());
    let mut owner = ActionOwnership::new();
    owner.select(
        true,
        &[Intent::input_sequence(Owner::OutOfShield, first.owned)],
    );
    let mut rules = Resolved::default();
    rules
        .add(Candidate {
            slot: 2,
            index: 0,
            sequence: recipe("auto_roll", 40., 12., "up_b:4"),
        })
        .unwrap();
    assert!(!Engine::default().preempt_automatic_grab(
        true,
        &rules,
        lc(c),
        &targets(0x20, 10., 0.),
        None,
        &mut o,
        &mut owner
    ));
    let caught = own(15, 0x37, 1.);
    let out = o.sample_automatic_configured(
        true,
        caught,
        lc(caught),
        targets(0x41, 10., 0.),
        None,
        false,
        20.,
        35.,
        Some(8.),
        settings(15),
        AutomaticConfig {
            grab: Some(&g),
            ..Default::default()
        },
    );
    assert_eq!(out.owned, 0);
    assert!(o.target().is_none());
}

#[test]
fn edited_grab_inputs_cannot_fall_through_to_a_throw_producing_follow_up() {
    let s = recipe("auto_shield_grab", 20., 24., "input:r:0:0:1,up_b:4");
    let mut o = OutOfShield::new();
    let cfg = AutomaticConfig {
        shield_grab: Some(&s),
        ..Default::default()
    };
    let c = own(15, 0x1c, 0.);
    let out = o.sample_automatic_configured(
        true,
        c,
        lc(c),
        targets(0x1c, 10., 0.),
        None,
        true,
        20.,
        35.,
        Some(12.),
        settings(15),
        cfg,
    );
    assert_eq!(out.requested, MAPPER_R | MAPPER_ZR);
    o.mapped_grab(0, 2);
    o.delivery_tick(0);
    for tick in 1..24 {
        let c = own(15, 0x1c, tick as f32);
        let out = o.sample_automatic_configured(
            true,
            c,
            lc(c),
            targets(0x1c, 10., 0.),
            None,
            true,
            20.,
            35.,
            Some(12.),
            settings(15),
            cfg,
        );
        assert_eq!(
            out.owned, 0,
            "no queued R retry/B after the configured grab step; tick {tick}"
        );
        assert!(o.grab_delivery_pending());
        o.delivery_tick(tick);
    }
    let c = own(15, 0x37, 24.);
    assert_eq!(
        o.sample_automatic_configured(
            true,
            c,
            lc(c),
            targets(0x41, 10., 0.),
            None,
            true,
            20.,
            35.,
            Some(12.),
            settings(15),
            cfg
        )
        .owned,
        0
    );
    assert!(o.target().is_none());
}

#[test]
fn configurable_grab_and_backup_heights_keep_front_shield_and_vulnerability_gates() {
    let g = recipe("auto_grab", 35., 2., "input:r:0:0:4");
    let s = recipe("auto_shield_grab", 10., 4., "input:r:0:0:4");
    let b = recipe("auto_shield_backup", 20., 3., "up_b:4,drift:opponent:1:69");
    for (shield, status, x, y, expected) in [
        (false, 0x1c, 10., 2., MAPPER_R),
        (false, 0x1c, 10., 2.01, 0),
        (false, 0, 10., 0., 0),
        (false, 0x1c, -10., 0., 0),
        (true, 0x1c, 10., 4., MAPPER_R | MAPPER_ZR),
        (true, 0x1c, 10., 4.01, 0),
        (true, 0, -10., 3., MAPPER_B | MAPPER_ZR),
        (true, 0, -10., 3.01, 0),
    ] {
        let mut o = OutOfShield::new();
        let c = own(15, if shield { 0x1c } else { 3 }, 0.);
        let out = o.sample_automatic_configured(
            true,
            c,
            lc(c),
            targets(status, x, y),
            None,
            shield,
            22.,
            35.,
            Some(8.),
            settings(15),
            config(&g, &s, &b),
        );
        assert_eq!(
            out.requested, expected,
            "shield {shield} enemy {status} x {x} y {y}"
        );
    }
}

fn dodged() -> SpotDodge {
    let mut d = SpotDodge::default();
    let c = DodgeContext {
        object: 1,
        status: 0,
        situation: 0,
        motion: 1,
        frame: 0.,
        rate: 1.,
        stopped: false,
        can_spot_dodge: true,
        position: [0.; 2],
        velocity: [0.; 2],
    };
    let mut ts = [None; 8];
    ts[0] = Some(GrabTarget {
        id: 2,
        status: 0x37,
        motion: 2,
        frame: 2.,
        rate: 1.,
        position: [10., 0.],
        velocity: [0.; 2],
        facing: -1.,
        first: 6,
        last: 7,
        reach: 12.,
    });
    assert_eq!(d.sample(true, c, ts).requested, MAPPER_ZR);
    assert_eq!(
        d.sample(true, DodgeContext { status: 0x1f, ..c }, ts).owned,
        0
    );
    d
}
fn follow(frame: f32, x: f32) -> Opponent {
    Opponent {
        id: 2,
        status: 0x37,
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
#[test]
fn grab_dodged_recipe_uses_its_own_actions_and_range_after_native_confirmation() {
    let s = recipe("auto_grab_dodged", 7., 2., "input:a:0:0:2");
    let mut too_far = dodged();
    assert_eq!(
        too_far
            .punish_automatic(
                true,
                Some(&s),
                settings(15),
                lc(own(15, 0, 0.)),
                Some(follow(0., 10.))
            )
            .owned,
        0
    );
    let mut d = dodged();
    let c = lc(own(15, 0, 0.));
    let out = d.punish_automatic(true, Some(&s), settings(15), c, Some(follow(0., 5.)));
    assert_eq!(out.requested, MAPPER_A);
    assert!(d.punish_committed());
    assert!(!d.direct_up_b());
    assert_eq!(
        d.punish_automatic(false, Some(&s), settings(15), c, Some(follow(0., 5.)))
            .owned,
        0
    );
    assert!(!d.punish_committed());
}

#[test]
fn configured_grab_dodge_grab_keeps_a_zero_input_receipt_until_native_catch() {
    let s = recipe("auto_grab_dodged", 20., 12., "input:r:0:0:1,up_b:4");
    let mut d = dodged();
    let c = lc(own(15, 0, 0.));
    assert_eq!(
        d.punish_automatic(true, Some(&s), settings(15), c, Some(follow(0., 10.)))
            .requested,
        MAPPER_R
    );
    d.mapped_grab(0, 2);
    d.delivery_tick(0);
    assert_eq!(
        d.punish_automatic(
            true,
            Some(&s),
            settings(15),
            lc(own(15, 0, 1.)),
            Some(follow(1., 10.))
        )
        .owned,
        0
    );
    assert!(d.grab_delivery_pending());
    assert!(d.punish_committed());
    assert_eq!(
        d.punish_automatic(
            true,
            Some(&s),
            settings(15),
            lc(own(15, 0x37, 2.)),
            Some(follow(2., 10.))
        )
        .owned,
        0
    );
    assert!(!d.grab_delivery_pending());
    assert!(!d.punish_committed());
}

#[test]
fn supplied_catalog_exposes_native_grab_defaults_for_all_fighters_without_enabling_new_rolls() {
    let c = Catalog::parse(include_str!(
        "../../../deployment/ultimate/kirby_assist/assembled/sequences.txt"
    ))
    .unwrap();
    let p = FighterProfiles::default();
    c.validate(&p).unwrap();
    for kind in 0..0x5e {
        // New editable-only Aegis profiles intentionally have no auto recipes.
        // Their generic native grab admission does not require a text recipe.
        if matches!(kind, 90 | 91) {
            assert!(c.entries().iter().filter(|e| e.fighter_kind == kind)
                .all(|e| !matches!(e.sequence.reaction.trigger,
                    Trigger::AutoGrab | Trigger::AutoShieldGrab | Trigger::AutoShieldBackup)));
            continue;
        }
        if p.profile(kind).is_some() {
            for trigger in [Trigger::AutoGrab,Trigger::AutoShieldGrab] {
                assert_eq!(c.entries().iter().filter(|e|e.fighter_kind==kind && e.sequence.reaction.trigger==trigger).count(),1,"kind {kind} {trigger:?}");
                assert!(c.entries().iter().filter(|e|e.fighter_kind==kind && e.sequence.reaction.trigger==Trigger::AutoShieldBackup).count()<=1,"at most one explicitly authored backup; generic templates are inert");
            }
        }
    }
}

#[test]
fn editing_backup_drift_keeps_native_startup_handoff_and_uses_the_new_duration_and_strength() {
    let b = recipe(
        "auto_shield_backup",
        20.,
        24.,
        "up_b:3,drift:opponent:0.5:5",
    );
    let cfg = AutomaticConfig {
        backup: Some(&b),
        ..Default::default()
    };
    let mut o = OutOfShield::new();
    let c = own(15, 0x1c, 0.);
    assert_eq!(
        o.sample_automatic_configured(
            true,
            c,
            lc(c),
            targets(0, -10., 0.),
            None,
            true,
            20.,
            35.,
            Some(12.),
            settings(15),
            cfg
        )
        .requested,
        MAPPER_B | MAPPER_ZR
    );
    for frame in 1..6 {
        let c = Context {
            status: 0x1de,
            frame: frame as f32,
            ..c
        };
        let x = if frame % 2 == 0 { 10. } else { -10. };
        let out = o.sample_automatic_configured(
            true,
            c,
            lc(c),
            targets(0, x, 0.),
            None,
            true,
            20.,
            35.,
            Some(12.),
            settings(15),
            cfg,
        );
        assert_eq!(out.requested, 0, "native startup releases B immediately");
        assert_eq!(out.stick, Some([x.signum() * 0.5, 0.]), "frame {frame}");
    }
    let c = Context {
        status: 0x1de,
        frame: 6.,
        ..c
    };
    assert_eq!(
        o.sample_automatic_configured(
            true,
            c,
            lc(c),
            targets(0, 10., 0.),
            None,
            true,
            20.,
            35.,
            Some(12.),
            settings(15),
            cfg
        )
        .owned,
        0
    );
}

#[test]
fn editing_only_backup_timeout_still_bounds_the_native_tracking_path() {
    let mut b = recipe("auto_shield_backup", 20., 24., "up_b:4,drift:opponent:1:69");
    b.timeout = 30;
    let cfg = AutomaticConfig {
        backup: Some(&b),
        ..Default::default()
    };
    let mut o = OutOfShield::new();
    for frame in 0..30 {
        let c = own(15, if frame == 0 { 0x1c } else { 0x1de }, frame as f32);
        assert_ne!(
            o.sample_automatic_configured(
                true,
                c,
                lc(c),
                targets(0, -10., 0.),
                None,
                true,
                20.,
                35.,
                Some(12.),
                settings(15),
                cfg
            )
            .owned,
            0
        );
    }
    let c = own(15, 0x1de, 30.);
    assert_eq!(
        o.sample_automatic_configured(
            true,
            c,
            lc(c),
            targets(0, -10., 0.),
            None,
            true,
            20.,
            35.,
            Some(12.),
            settings(15),
            cfg
        )
        .owned,
        0
    );
}

#[test]
fn confirmed_dodge_followup_keeps_ownership_through_recovery_and_ignores_manual_churn() {
    use kirby_assist_core::action_owner::{exclusive_output, ActionOwnership, Intent, Owner};
    let s = recipe("auto_grab_dodged", 20., 12., "input:a:0:0:2");
    let mut d = dodged();
    let mut owners = ActionOwnership::new();
    owners.observe_manual(0, [0.; 4]);
    owners.select(true, &[Intent::input_sequence(Owner::GrabDodge, 1)]);
    assert!(!owners.handoff(Owner::RollRead, Owner::OptionRead));
    assert!(owners.handoff(Owner::GrabDodge, Owner::OptionRead));
    owners.set_ledge_priority(true);
    owners.set_automatic_suspended(true);
    for frame in 0..8 {
        owners.observe_manual(if frame % 2 == 0 { MAPPER_A } else { MAPPER_R }, [1., -1., 1., -1.]);
        let mut c = lc(own(15, 0x1f, frame as f32));
        c.can_act = false;
        c.manual_input_epoch = owners.physical_inputs().epoch();
        let out = d.punish_automatic(true, Some(&s), settings(15), c, Some(follow(frame as f32, 5.)));
        assert_eq!(out.owned, 0);
        assert!(d.punish_pending());
        let winner = owners.select(true, &[
            Intent::sequence_receipt(Owner::OptionRead, out.owned, d.punish_pending()),
            Intent::input_sequence(Owner::Ledge, 1),
        ]);
        assert_eq!(winner, Some(Owner::OptionRead));
        let packet = exclusive_output(winner, true, out);
        assert_eq!(packet.stick, Some([0.; 2]));
        assert_eq!(packet.requested, 0);
        assert_eq!(owners.policy_inputs().epoch(), 0);
    }
    let out = d.punish_automatic(true, Some(&s), settings(15), lc(own(15, 0, 8.)), Some(follow(8., 5.)));
    assert_eq!(out.requested, MAPPER_A, "the configured punish starts on native readiness");
    assert!(d.punish_committed());
    d.punish_automatic(false, Some(&s), settings(15), lc(own(15, 0, 9.)), Some(follow(9., 5.)));
    assert!(!d.punish_pending());
    owners.release(Owner::OptionRead);
    assert!(owners.allows(Owner::Ledge));
}
