//! Configuration parity and safety for file-authored automatic responses.
use kirby_assist_core::{
    fighter_profiles::FighterProfiles,
    gameplay::{GameplayContext, MotionTiming, Opponent},
    native_input::{MAPPER_A, MAPPER_B, MAPPER_R, MAPPER_ZR},
    out_of_shield::{AutomaticConfig, Context, OutOfShield},
    reaction_rules::{Candidate, Resolved, Trigger},
    roll_read::Target,
    sequence_config::{Catalog, Sequence},
    settings::AssistSettings,
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
fn lc(c: Context) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        can_act: true,
        status: c.status,
        motion_kind: c.motion,
        motion_frame: c.frame,
        motion_rate: c.rate,
        stopped: c.stopped,
        situation: if c.grounded {
            kirby_assist_core::native_states::situation::GROUND
        } else {
            kirby_assist_core::native_states::situation::AIR
        },
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
fn opponent(status: i32, x: f32, y: f32, frame: f32) -> Opponent {
    Opponent {
        id: 2,
        status,
        position: [x, y],
        vulnerable: true,
        motion: Some(MotionTiming {
            kind: 2,
            frame,
            rate: 1.,
            remaining: None,
        }),
        ..Default::default()
    }
}
fn settings(kind: i32) -> AssistSettings {
    AssistSettings {
        fighter_kind: kind,
        ..Default::default()
    }
}
fn config<'a>(_g: &'a Sequence, s: &'a Sequence, b: &'a Sequence) -> AutomaticConfig<'a> {
    AutomaticConfig {
        shield_grab: Some(&s),
        backup: Some(b),
    }
}

#[test]
fn all_current_events_roundtrip_and_reject_duplicate_recipes_in_any_slot() {
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
    assert_eq!(rules.len, 6);
}

#[test]
fn configured_shield_grabs_admit_immediately_and_keep_inert_delivery_receipts() {
    let profiles = FighterProfiles::default();
    for kind in 0..kirby_assist_core::fighter_kinds::PLAYABLE_KIND_COUNT {
        let Some(profile) = profiles.profile(kind) else {
            continue;
        };
        let close = profile.grab.grab.standing_range;
        let sequence = recipe("auto_shield_grab", close, 24., "input:r:0:0:4");
        for status in [
            kirby_assist_core::native_states::GUARD_ON,
            kirby_assist_core::native_states::GUARD,
        ] {
            let mut tool = OutOfShield::new();
            let c = own(kind, status, 0.);
            let cfg = AutomaticConfig {
                shield_grab: Some(&sequence),
                backup: None,
            };
            let target = targets(0, close * 0.5, 0.);
            let first = tool.sample_automatic_configured(
                true,
                c,
                lc(c),
                target,
                None,
                true,
                close,
                close * 100.,
                Some(12.),
                settings(kind),
                cfg,
            );
            if close <= 0. {
                assert_eq!(first, Default::default());
                continue;
            }
            assert_eq!(
                first.requested,
                MAPPER_R | MAPPER_ZR,
                "kind {kind} status {status:#x}"
            );
            assert_eq!(first.stick, Some([0.; 2]));
            tool.mapped_grab(0, 2);
            for frame in 1..6 {
                let next = own(kind, status, frame as f32);
                let out = tool.sample_automatic_configured(
                    true,
                    next,
                    lc(next),
                    target,
                    None,
                    true,
                    close,
                    close * 100.,
                    Some(12.),
                    settings(kind),
                    cfg,
                );
                assert_eq!(
                    out.requested, 0,
                    "delivered grab cannot queue another action"
                );
                assert!(tool.grab_delivery_pending());
                tool.delivery_tick(frame);
            }
            let caught = own(kind, kirby_assist_core::native_states::CATCH, 6.);
            assert_eq!(
                tool.sample_automatic_configured(
                    true,
                    caught,
                    lc(caught),
                    target,
                    None,
                    true,
                    close,
                    close * 100.,
                    Some(12.),
                    settings(kind),
                    cfg
                ),
                Default::default()
            );
            assert!(tool.target().is_none());
        }
    }
}

#[test]
fn edited_grab_inputs_cannot_fall_through_to_a_throw_producing_follow_up() {
    let s = recipe("auto_shield_grab", 20., 24., "input:r:0:0:1,up_b:4");
    let mut o = OutOfShield::new();
    let cfg = AutomaticConfig {
        shield_grab: Some(&s),
        ..Default::default()
    };
    let c = own(15, kirby_assist_core::native_states::GUARD, 0.);
    let out = o.sample_automatic_configured(
        true,
        c,
        lc(c),
        targets(kirby_assist_core::native_states::GUARD, 10., 0.),
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
        let c = own(15, kirby_assist_core::native_states::GUARD, tick as f32);
        let out = o.sample_automatic_configured(
            true,
            c,
            lc(c),
            targets(kirby_assist_core::native_states::GUARD, 10., 0.),
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
    let c = own(15, kirby_assist_core::native_states::CATCH, 24.);
    assert_eq!(
        o.sample_automatic_configured(
            true,
            c,
            lc(c),
            targets(kirby_assist_core::native_states::CAPTURE_PULLED, 10., 0.),
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
    let g = recipe("auto_shield_grab", 35., 2., "input:r:0:0:4");
    let s = recipe("auto_shield_grab", 10., 4., "input:r:0:0:4");
    let b = recipe("auto_shield_backup", 20., 3., "up_b:4,drift:opponent:1:69");
    for (shield, status, x, y, expected) in [
        (false, 0x1c, 10., 2., 0),
        (false, 0x1c, 10., 2.01, 0),
        (false, 0, 10., 0., 0),
        (false, 0x1c, -10., 0., 0),
        (true, 0x1c, 10., 4., MAPPER_R | MAPPER_ZR),
        (true, 0x1c, 10., 4.01, 0),
        (true, 0, -10., 3., MAPPER_B | MAPPER_ZR),
        (true, 0, -10., 3.01, 0),
    ] {
        let mut o = OutOfShield::new();
        let c = own(
            15,
            if shield {
                kirby_assist_core::native_states::GUARD
            } else {
                3
            },
            0.,
        );
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
        status: kirby_assist_core::native_states::status::WAIT,
        situation: kirby_assist_core::native_states::situation::GROUND,
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
        status: kirby_assist_core::native_states::CATCH,
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
        d.sample(
            true,
            DodgeContext {
                status: kirby_assist_core::native_states::ESCAPE,
                ..c
            },
            ts
        )
        .owned,
        0
    );
    d
}
fn follow(frame: f32, x: f32) -> Opponent {
    Opponent {
        id: 2,
        status: kirby_assist_core::native_states::CATCH,
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
            lc(own(15, kirby_assist_core::native_states::CATCH, 2.)),
            Some(follow(2., 10.))
        )
        .owned,
        0
    );
    assert!(!d.grab_delivery_pending());
    assert!(!d.punish_committed());
}

#[test]
fn supplied_catalog_exposes_native_grab_modes_and_configured_entries() {
    let c = Catalog::parse(include_str!(
        "../../../deployment/ultimate/kirby_assist/assembled/sequences.txt"
    ))
    .unwrap();
    let p = FighterProfiles::default();
    c.validate(&p).unwrap();
    for kind in 0..kirby_assist_core::fighter_kinds::PLAYABLE_KIND_COUNT {
        // Min Min has a shield backup but no auto-grab recipes; editable-only
        // Aegis profiles have neither. Native admission needs no text recipe.
        if matches!(
            kind,
            kirby_assist_core::fighter_kinds::MIN_MIN
                | kirby_assist_core::fighter_kinds::PYRA
                | kirby_assist_core::fighter_kinds::MYTHRA
        ) {
            assert!(c
                .entries()
                .iter()
                .filter(|e| e.fighter_kind == kind)
                .all(
                    |e| !matches!(e.sequence.reaction.trigger, Trigger::AutoShieldGrab)
                        && (kind == kirby_assist_core::fighter_kinds::MIN_MIN
                            || e.sequence.reaction.trigger != Trigger::AutoShieldBackup)
                ));
            continue;
        }
        if p.profile(kind).is_some() {
            for trigger in [Trigger::AutoShieldGrab] {
                let count = c
                    .entries()
                    .iter()
                    .filter(|e| e.fighter_kind == kind && e.sequence.reaction.trigger == trigger)
                    .count();
                assert!(count <= 1, "kind {kind} {trigger:?}");
                assert!(
                    c.entries()
                        .iter()
                        .filter(|e| e.fighter_kind == kind
                            && e.sequence.reaction.trigger == Trigger::AutoShieldBackup)
                        .count()
                        <= 1,
                    "at most one explicitly authored backup; generic templates are inert"
                );
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
    let c = own(15, kirby_assist_core::native_states::GUARD, 0.);
    assert_eq!(
        o.sample_automatic_configured(
            true,
            c,
            lc(c),
            targets(0, -10., 0.),
            Some(opponent(0, -10., 0., 0.)),
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
    o.mapped_up_special(0, 2);
    for frame in 1..6 {
        let c = Context {
            status: kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_G,
            frame: frame as f32,
            ..c
        };
        let x = if frame % 2 == 0 { 10. } else { -10. };
        let out = o.sample_automatic_configured(
            true,
            c,
            lc(c),
            targets(0, x, 0.),
            Some(opponent(0, x, 0., frame as f32)),
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
    let mut owned = u32::MAX;
    for frame in 6..=12 {
        let c = Context {
            status: kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_G,
            frame: frame as f32,
            ..c
        };
        let out = o.sample_automatic_configured(
            true,
            c,
            lc(c),
            targets(0, 10., 0.),
            Some(opponent(0, 10., 0., frame as f32)),
            true,
            20.,
            35.,
            Some(12.),
            settings(15),
            cfg,
        );
        if frame == 6 {
            assert_eq!(
                out.stick,
                Some([0., 0.]),
                "configured drift ends after five frames"
            );
        }
        owned = out.owned;
        if owned == 0 {
            break;
        }
    }
    assert_eq!(owned, 0);
}

#[test]
fn editing_only_backup_timeout_still_bounds_the_authored_sequence_path() {
    let mut b = recipe("auto_shield_backup", 20., 24., "up_b:4,drift:opponent:1:69");
    b.timeout = 30;
    let cfg = AutomaticConfig {
        backup: Some(&b),
        ..Default::default()
    };
    let mut o = OutOfShield::new();
    for frame in 0..30 {
        let c = own(
            15,
            if frame == 0 {
                kirby_assist_core::native_states::GUARD
            } else {
                kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_G
            },
            frame as f32,
        );
        let out = o.sample_automatic_configured(
            true,
            c,
            lc(c),
            targets(0, -10., 0.),
            Some(opponent(0, -10., 0., frame as f32)),
            true,
            20.,
            35.,
            Some(12.),
            settings(15),
            cfg,
        );
        assert_ne!(out.owned, 0);
        if frame == 0 {
            o.mapped_up_special(0, 2);
        }
    }
    let mut owned = u32::MAX;
    for frame in 30..=35 {
        let c = own(
            15,
            kirby_assist_core::native_states::fighter_status::KOOPA_SPECIAL_HI_G,
            frame as f32,
        );
        owned = o
            .sample_automatic_configured(
                true,
                c,
                lc(c),
                targets(0, -10., 0.),
                Some(opponent(0, -10., 0., frame as f32)),
                true,
                20.,
                35.,
                Some(12.),
                settings(15),
                cfg,
            )
            .owned;
        if owned == 0 {
            break;
        }
    }
    assert_eq!(owned, 0);
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
        owners.observe_manual(
            if frame % 2 == 0 { MAPPER_A } else { MAPPER_R },
            [1., -1., 1., -1.],
        );
        let mut c = lc(own(
            15,
            kirby_assist_core::native_states::ESCAPE,
            frame as f32,
        ));
        c.can_act = false;
        c.manual_input_epoch = owners.physical_inputs().epoch();
        let out = d.punish_automatic(
            true,
            Some(&s),
            settings(15),
            c,
            Some(follow(frame as f32, 5.)),
        );
        assert_eq!(out.owned, 0);
        assert!(d.punish_pending());
        let winner = owners.select(
            true,
            &[
                Intent::sequence_receipt(Owner::OptionRead, out.owned, d.punish_pending()),
                Intent::input_sequence(Owner::Ledge, 1),
            ],
        );
        assert_eq!(winner, Some(Owner::OptionRead));
        let packet = exclusive_output(winner, true, out);
        assert_eq!(packet.stick, Some([0.; 2]));
        assert_eq!(packet.requested, 0);
        assert_eq!(owners.policy_inputs().epoch(), 0);
    }
    let out = d.punish_automatic(
        true,
        Some(&s),
        settings(15),
        lc(own(15, 0, 8.)),
        Some(follow(8., 5.)),
    );
    assert_eq!(
        out.requested, MAPPER_A,
        "the configured punish starts on native readiness"
    );
    assert!(d.punish_committed());
    d.punish_automatic(
        false,
        Some(&s),
        settings(15),
        lc(own(15, 0, 9.)),
        Some(follow(9., 5.)),
    );
    assert!(!d.punish_pending());
    owners.release(Owner::OptionRead);
    assert!(owners.allows(Owner::Ledge));
}
#[test]
fn configured_shield_recipes_cannot_enable_unshielded_native_automatic_inputs() {
    for actions in [
        "input:r:0.75:0.5:2",
        "input:a:0:0:2",
        "wait:10,input:r:0:0:4",
    ] {
        let authored = recipe("auto_shield_grab", 35., 24., actions);
        assert_eq!(authored.reaction.trigger, Trigger::AutoShieldGrab);
        for status in [0, 3, 4] {
            let mut tool = OutOfShield::new();
            let c = own(15, status, 0.);
            let out = tool.sample_automatic_configured(
                true,
                c,
                lc(c),
                targets(kirby_assist_core::native_states::GUARD, 10., 0.),
                None,
                false,
                20.,
                35.,
                Some(8.),
                settings(15),
                AutomaticConfig {
                    shield_grab: Some(&authored),
                    backup: None,
                },
            );
            assert_eq!(out, Default::default());
            assert!(!tool.configured_committed());
            assert!(tool.target().is_none());
        }
    }
}
