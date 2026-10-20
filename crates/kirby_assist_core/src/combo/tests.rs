use super::*;
use crate::ledge_trap::MotionTiming;
use crate::native_input::MAPPER_R;
fn settings() -> ComboSettings {
    ComboSettings {
        enabled: true,
        ..ComboSettings::default()
    }
}
fn context() -> ComboContext {
    ComboContext {
        own: LedgeContext {
            held: true,
            gameplay: true,
            can_act: true,
            can_control: true,
            stopped: false,
            status: 0,
            situation: 0,
            motion_frame: 0.0,
            motion_kind: 1,
            position: [0.0, 0.0],
            velocity: [0.0, 0.0],
            facing: 1.0,
            previous_status: 0,
            can_grab: true,
            can_tilt: true,
            can_dash_attack: true,
            can_jump: true,
            can_air_attack: true,
            air_physics: None,
            short_hop_speed: None,
            shield_stun_frames: None,
            shield_attacker: None,
            hit_confirmed: false,
            safe_ground: [true, true],
        },
        own_id: 1,
        motion_rate: 1.0,
        can_jump: true,
        can_attack_air: false,
        capture_verified: false,
        throw_target: None,
        target: Some(ComboTarget {
            fighter: Opponent {
                id: 2,
                status: 0,
                previous_status: 0,
                situation: 0,
                position: [5.0, 0.0],
                vulnerable: true,
                velocity: [0.0, 0.0],
                cliff_position: None,
                vulnerable_in: None,
                motion: Some(MotionTiming {
                    kind: 1,
                    frame: 0.0,
                    rate: 1.0,
                    remaining: Some(60.0),
                }),
                attack_active: None,
                air_physics: None,
                actionable_in: None,
            },
            kind: 1,
            percent: 0.0,
            actionable: true,
            stopped: false,
        }),
    }
}
fn step(state: &mut ComboAttempt, ctx: ComboContext) -> LedgeOutput {
    state.sample(settings(), true, MAPPER_R, ctx)
}
fn captured(ctx: ComboContext) -> ComboContext {
    let mut c = ctx;
    c.own.status = 0x3c;
    c.own.motion_kind = 2;
    c.own.can_act = false;
    c.capture_verified = true;
    let t = c.target.as_mut().unwrap();
    t.actionable = false;
    t.fighter.status = 0x42;
    c
}
fn throw_setup(state: &mut ComboAttempt, initial: ComboContext) -> ComboContext {
    assert_eq!(step(state, initial).requested, MAPPER_R);
    let capture = captured(initial);
    assert_eq!(step(state, capture).requested, 0);
    let mut ready = capture;
    ready.own.motion_frame = 1.0;
    assert_eq!(step(state, ready).stick, Some([initial.own.facing, 0.0]));
    let mut thrown = ready;
    thrown.own.status = THROW;
    thrown.own.situation = 2;
    thrown.own.motion_kind = FORWARD_THROW_MOTION;
    thrown.own.motion_frame = 1.0;
    thrown.capture_verified = false;
    thrown.throw_target = Some(2);
    thrown.target.as_mut().unwrap().fighter.status = 0x46;
    step(state, thrown);
    thrown
}
fn throw_to_fair(state: &mut ComboAttempt, mut ctx: ComboContext) -> ComboContext {
    for frame in 2..=59 {
        ctx.own.motion_frame = frame as f32;
        if frame >= 45 {
            ctx.target.as_mut().unwrap().percent = 5.0;
        }
        if frame == 59 {
            ctx.can_attack_air = true;
        }
        let out = step(state, ctx);
        assert_eq!(
            out.requested,
            if frame == 59 { MAPPER_A } else { 0 },
            "frame={frame}, action={}",
            state.action_name()
        );
    }
    ctx
}
#[test]
fn full_observed_route_grabs_confirms_throw_and_all_three_fair_hits_on_both_sides() {
    for side in [-1.0, 1.0] {
        for kind in 0..86 {
            // No per-character fixed frame/weight assumption.
            let mut state = ComboAttempt::default();
            let mut initial = context();
            initial.own.facing = side;
            let t = initial.target.as_mut().unwrap();
            t.fighter.position[0] *= side;
            t.kind = kind;
            let thrown = throw_setup(&mut state, initial);
            let mut fair = throw_to_fair(&mut state, thrown);
            fair.own.status = ATTACK_AIR;
            fair.own.motion_kind = FAIR_MOTION;
            let mut percent = 5.0;
            for frame in 1..=25 {
                fair.own.motion_frame = frame as f32;
                fair.own.hit_confirmed = [10, 17, 25].contains(&frame);
                fair.own.stopped = fair.own.hit_confirmed;
                if fair.own.hit_confirmed {
                    percent += 3.0;
                }
                let target = fair.target.as_mut().unwrap();
                target.percent = percent;
                target.fighter.status = 0x49;
                let out = step(&mut state, fair);
                assert_eq!(out.requested, 0);
            }
            assert_eq!(state.action_name(), "combo_complete");
            assert!(!state.active());
            assert_eq!(
                step(&mut state, initial).owned,
                0,
                "held trigger must not loop"
            );
        }
    }
}
#[test]
fn target_actionability_between_throw_and_fair_aborts_without_a_late_aerial() {
    let mut state = ComboAttempt::default();
    let mut thrown = throw_setup(&mut state, context());
    thrown.own.motion_frame = 2.0;
    thrown.target.as_mut().unwrap().actionable = true;
    assert_eq!(step(&mut state, thrown).owned, 0);
    assert_eq!(state.action_name(), "combo_target_actionable_gap");
    assert_eq!(step(&mut state, context()).requested, 0);
}
#[test]
fn release_damage_target_loss_and_settings_changes_beat_duplicate_cache() {
    let ctx = context();
    for bad in [
        ComboContext {
            own: LedgeContext {
                status: 0x49,
                ..ctx.own
            },
            ..ctx
        },
        ComboContext {
            target: None,
            ..ctx
        },
        ComboContext {
            target: Some(ComboTarget {
                fighter: Opponent {
                    id: 3,
                    ..ctx.target.unwrap().fighter
                },
                ..ctx.target.unwrap()
            }),
            ..ctx
        },
    ] {
        let mut state = ComboAttempt::default();
        assert_eq!(step(&mut state, ctx).requested, MAPPER_R);
        assert_eq!(step(&mut state, bad).owned, 0);
    }
    let mut state = ComboAttempt::default();
    step(&mut state, ctx);
    assert_eq!(state.sample(settings(), false, MAPPER_R, ctx).owned, 0);
    assert_eq!(step(&mut state, ctx).requested, MAPPER_R);
    assert_eq!(
        state
            .sample(
                ComboSettings {
                    fair_range: 13.0,
                    ..settings()
                },
                true,
                MAPPER_R,
                ctx
            )
            .owned,
        0
    );
}
#[test]
fn capture_identity_and_forward_throw_motion_are_required() {
    let mut state = ComboAttempt::default();
    step(&mut state, context());
    let mut ctx = captured(context());
    ctx.capture_verified = false;
    step(&mut state, ctx);
    assert_eq!(state.action_name(), "combo_grab");
    ctx.capture_verified = true;
    ctx.own.motion_frame = 1.0;
    step(&mut state, ctx);
    ctx.own.motion_frame = 2.0;
    step(&mut state, ctx);
    ctx.own.status = THROW;
    ctx.own.motion_kind = FORWARD_THROW_MOTION;
    ctx.own.motion_frame = 0.0;
    ctx.throw_target = Some(3);
    assert_eq!(step(&mut state, ctx).owned, 0);
    assert_eq!(state.action_name(), "combo_wrong_throw");
}
#[test]
fn duplicate_native_polls_do_not_repeat_or_skip_a_step_and_stop_releases_buttons() {
    let mut state = ComboAttempt::default();
    let ctx = context();
    let press = step(&mut state, ctx);
    for _ in 0..50 {
        assert_eq!(step(&mut state, ctx), press);
    }
    let stopped = ComboContext {
        own: LedgeContext {
            stopped: true,
            ..ctx.own
        },
        ..ctx
    };
    let frozen = step(&mut state, stopped);
    assert_eq!(frozen.requested, 0);
    assert_ne!(frozen.owned, 0);
    assert_eq!(state.age, 0.0);
}
#[test]
fn range_percent_invalid_data_and_missing_readiness_never_start() {
    let mut ctx = context();
    ctx.target.as_mut().unwrap().percent = 0.6;
    assert_eq!(step(&mut ComboAttempt::default(), ctx).owned, 0);
    ctx = context();
    ctx.target.as_mut().unwrap().fighter.position[0] = 6.1;
    assert_eq!(step(&mut ComboAttempt::default(), ctx).owned, 0);
    ctx = context();
    ctx.own.motion_kind = 0;
    assert_eq!(step(&mut ComboAttempt::default(), ctx).owned, 0);
    ctx = context();
    ctx.own.can_grab = false;
    assert_eq!(step(&mut ComboAttempt::default(), ctx).owned, 0);
}
#[test]
fn optional_dair_waits_for_takeoff_hit_and_landing_recovery_before_grab() {
    let config = ComboSettings {
        starter: ComboStarter::DownAir,
        ..settings()
    };
    let mut state = ComboAttempt::default();
    let ctx = context();
    assert_eq!(
        state.sample(config, true, MAPPER_R, ctx).requested,
        MAPPER_X
    );
    let mut air = ctx;
    air.own.status = 0xa;
    air.own.can_act = false;
    air.own.motion_kind = 2;
    assert_eq!(state.sample(config, true, MAPPER_R, air).requested, 0);
    air.own.status = 0xb;
    air.own.situation = 2;
    air.own.motion_kind = 3;
    air.can_attack_air = true;
    assert_eq!(
        state.sample(config, true, MAPPER_R, air).requested,
        MAPPER_A
    );
    air.own.status = ATTACK_AIR;
    air.own.motion_kind = DAIR_MOTION;
    for frame in 1..=18 {
        air.own.motion_frame = frame as f32;
        if frame == 18 {
            air.own.hit_confirmed = true;
            let t = air.target.as_mut().unwrap();
            t.percent = 1.3;
            t.actionable = false;
            t.fighter.status = 0x49;
        }
        state.sample(config, true, MAPPER_R, air);
    }
    let mut landing = air;
    landing.own.status = 0x18;
    landing.own.situation = 0;
    landing.own.motion_kind = 4;
    landing.own.motion_frame = 0.0;
    assert_eq!(state.sample(config, true, MAPPER_R, landing).requested, 0);
    for frame in 1..=16 {
        landing.own.motion_frame = frame as f32;
        assert_eq!(state.sample(config, true, MAPPER_R, landing).requested, 0);
    }
    landing.own.status = 0;
    landing.own.can_act = true;
    landing.own.motion_kind = 1;
    landing.own.motion_frame = 0.0;
    assert_eq!(
        state.sample(config, true, MAPPER_R, landing).requested,
        MAPPER_R
    );
}
#[test]
fn one_fair_hit_or_a_different_aerial_cannot_finish_the_route() {
    let mut state = ComboAttempt::default();
    let thrown = throw_setup(&mut state, context());
    let mut fair = throw_to_fair(&mut state, thrown);
    fair.own.status = ATTACK_AIR;
    fair.own.motion_kind = DAIR_MOTION;
    fair.own.motion_frame = 1.0;
    assert_eq!(step(&mut state, fair).owned, 0);
    assert_eq!(state.action_name(), "combo_wrong_aerial");
}

#[test]
fn grounded_throw_recovery_short_hops_before_forward_air_instead_of_grounded_attack() {
    let mut state = ComboAttempt::default();
    let mut ctx = throw_setup(&mut state, context());
    for frame in 2..=58 {
        ctx.own.motion_frame = frame as f32;
        if frame >= 45 {
            ctx.target.as_mut().unwrap().percent = 5.0;
        }
        step(&mut state, ctx);
    }
    ctx.own.status = 0;
    ctx.own.situation = 0;
    ctx.own.motion_kind = 1;
    ctx.own.motion_frame = 0.0;
    ctx.own.can_act = true;
    assert_eq!(step(&mut state, ctx).requested, MAPPER_X);
    ctx.own.status = 0xa;
    ctx.own.motion_kind = 2;
    ctx.own.can_act = false;
    assert_eq!(step(&mut state, ctx).requested, 0);
    ctx.own.status = 0xb;
    ctx.own.motion_kind = 3;
    ctx.own.situation = 2;
    ctx.can_attack_air = true;
    assert_eq!(step(&mut state, ctx).requested, MAPPER_A);
}

#[test]
fn missing_fair_hit_two_never_reports_completion_and_a_stale_hit_flag_is_not_a_hit() {
    let mut state = ComboAttempt::default();
    let thrown = throw_setup(&mut state, context());
    let mut ctx = throw_to_fair(&mut state, thrown);
    ctx.own.status = ATTACK_AIR;
    ctx.own.motion_kind = FAIR_MOTION;
    let mut percent = 5.0;
    for frame in 1..=28 {
        ctx.own.motion_frame = frame as f32;
        ctx.own.hit_confirmed = true;
        if [10, 25].contains(&frame) {
            percent += 3.0;
        }
        ctx.target.as_mut().unwrap().percent = percent;
        ctx.target.as_mut().unwrap().fighter.status = 0x49;
        step(&mut state, ctx);
    }
    assert_eq!(state.fair_hits, [true, false, true]);
    assert_eq!(state.action_name(), "combo_fair_hits_unconfirmed");
}

#[test]
fn combo_parser_requires_opt_in_valid_ranges_and_known_starters() {
    use crate::live_input::LiveConfig;
    assert!(!LiveConfig::default().combo.enabled);
    let supplied = LiveConfig::parse(include_str!(
        "../../../../deployment/ultimate/kirby_assist/config.txt"
    ))
    .unwrap();
    assert!(!supplied.combo.enabled);
    assert_eq!(supplied.combo.starter, ComboStarter::Grab);
    for input in [
        "combo_starter = hammer",
        "combo_max_start_percent = NaN",
        "combo_max_start_percent = 51",
        "combo_grab_range = 0",
        "combo_fair_range = inf",
    ] {
        assert!(LiveConfig::parse(input).is_err(), "{input}");
    }
    assert_eq!(
        LiveConfig::parse("combo_starter = down_air")
            .unwrap()
            .combo
            .starter,
        ComboStarter::DownAir
    );
}

#[test]
fn a_grab_whiff_and_rejected_input_do_not_loop_while_trigger_remains_held() {
    let mut state = ComboAttempt::default();
    let ctx = context();
    step(&mut state, ctx);
    let mut grabbing = ctx;
    grabbing.own.status = 0x37;
    grabbing.own.motion_kind = 2;
    grabbing.own.can_act = false;
    step(&mut state, grabbing);
    assert_eq!(step(&mut state, ctx).owned, 0);
    assert_eq!(state.action_name(), "combo_grab_whiff");
    for _ in 0..20 {
        assert_eq!(step(&mut state, ctx).requested, 0);
    }
    state.sample(settings(), false, MAPPER_R, ctx);
    assert_eq!(step(&mut state, ctx).requested, MAPPER_R);
    let mut rejected = ctx;
    for frame in 1..=16 {
        rejected.own.motion_frame = frame as f32;
        step(&mut state, rejected);
    }
    assert_eq!(state.action_name(), "combo_grab_rejected");
}

#[test]
fn actionability_between_fair_hits_cancels_instead_of_counting_a_later_hit() {
    let mut state = ComboAttempt::default();
    let thrown = throw_setup(&mut state, context());
    let mut fair = throw_to_fair(&mut state, thrown);
    fair.own.status = ATTACK_AIR;
    fair.own.motion_kind = FAIR_MOTION;
    for frame in 1..=10 {
        fair.own.motion_frame = frame as f32;
        if frame == 10 {
            fair.own.hit_confirmed = true;
            fair.target.as_mut().unwrap().percent = 9.0;
        }
        fair.target.as_mut().unwrap().fighter.status = 0x49;
        step(&mut state, fair);
    }
    fair.own.motion_frame = 11.0;
    fair.target.as_mut().unwrap().actionable = true;
    assert_eq!(step(&mut state, fair).owned, 0);
    assert_eq!(state.action_name(), "combo_target_actionable_gap");
}
