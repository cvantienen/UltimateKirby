//! Admission, shield retention and native-delivery safety regressions.
use kirby_assist_core::{
    defense_response::Response,
    gameplay::{GameplayContext, MotionTiming, Opponent},
    ledge_trap::LedgeSettings,
    native_input::{ButtonTrace, MAPPER_B, MAPPER_R, MAPPER_ZR},
    reaction_rules::{Candidate, Engine, Resolved},
    roll_read::Target as ReactionTarget,
    sequence_config::{Catalog, Sequence},
};
fn own(frame: f32, status: i32) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        can_act: true,
        status,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.,
        facing: 1.,
        safe_ground: [true; 2],
        ..Default::default()
    }
}
fn enemy(frame: f32, status: i32) -> Opponent {
    Opponent {
        id: 2,
        status,
        position: [10., 0.],
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
fn recipe() -> Sequence {
    Catalog::parse("[bowser.roll.auto_spot_dodge]\nstart_delay=0\ntimeout=180\nreaction=auto_spot_dodge:35:12\nactions=up_b:4,drift:opponent:1:69\n")
        .unwrap().entries()[0].sequence
}
fn settings() -> LedgeSettings {
    LedgeSettings {
        fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
        ..Default::default()
    }
}
fn targets(status: i32) -> [Option<ReactionTarget>; 8] {
    let mut out = [None; 8];
    out[0] = Some(ReactionTarget {
        id: 2,
        status,
        motion: 2,
        frame: 0.,
        rate: 1.,
        position: [10., 0.],
        velocity_x: 0.,
        vulnerable_in: Some(0.),
        actionable_in: None,
    });
    out
}

#[test]
fn busy_native_states_never_acquire_even_with_a_cancel_flag() {
    use kirby_assist_core::reaction_rules::AutomaticObservationScope as Scope;
    for status in [
        kirby_assist_core::native_states::status::GUARD_OFF,
        kirby_assist_core::native_states::status::GUARD_DAMAGE,
        kirby_assist_core::native_states::status::ESCAPE,
        kirby_assist_core::native_states::status::ESCAPE_F,
        kirby_assist_core::native_states::status::ESCAPE_B,
        kirby_assist_core::native_states::status::ESCAPE_AIR,
        kirby_assist_core::native_states::status::ESCAPE_AIR_SLIDE,
        kirby_assist_core::native_states::status::ATTACK,
        kirby_assist_core::native_states::status::ATTACK_AIR,
        kirby_assist_core::native_states::status::CATCH,
        kirby_assist_core::native_states::status::CATCH_CUT,
        kirby_assist_core::native_states::status::THROW,
        kirby_assist_core::native_states::status::DAMAGE,
        kirby_assist_core::native_states::status::DOWN,
        kirby_assist_core::native_states::status::DOWN_WAIT_CONTINUE,
        kirby_assist_core::native_states::status::CLIFF_CATCH_MOVE,
        kirby_assist_core::native_states::status::DEAD,
        kirby_assist_core::native_states::status::SPECIAL_HI,
    ] {
        let scope = Scope::new(
            false,
            status,
            kirby_assist_core::native_states::status::WAIT,
            false,
            false,
            false,
            false,
            false,
            false,
        );
        assert!(!scope.observe_targets(), "busy native status {status:#x}");
        let mut engine = Engine::default();
        let mut rules = Resolved::default();
        rules
            .add(Candidate {
                slot: 0,
                index: 0,
                sequence: recipe(),
            })
            .unwrap();
        let mut ts = targets(kirby_assist_core::native_states::ESCAPE);
        ts[0].as_mut().unwrap().frame = 1.;
        assert_eq!(
            engine
                .sample_simple(
                    true,
                    &rules,
                    settings(),
                    own(0., status),
                    ts,
                    Some(enemy(0., kirby_assist_core::native_states::ESCAPE))
                )
                .owned,
            0
        );
        assert!(!engine.committed());
        assert_ne!(
            engine
                .sample_simple(
                    true,
                    &rules,
                    settings(),
                    own(1., 0),
                    ts,
                    Some(enemy(1., kirby_assist_core::native_states::ESCAPE))
                )
                .requested
                & MAPPER_B,
            0,
            "rejected event was not consumed"
        );
    }
    for status in [
        kirby_assist_core::native_states::status::WAIT,
        kirby_assist_core::native_states::status::WALK,
        kirby_assist_core::native_states::status::DASH,
        kirby_assist_core::native_states::status::RUN,
        kirby_assist_core::native_states::status::RUN_BRAKE,
        kirby_assist_core::native_states::status::TURN_RUN_BRAKE,
        kirby_assist_core::native_states::status::SQUAT,
        kirby_assist_core::native_states::status::SQUAT_WAIT,
        kirby_assist_core::native_states::status::LANDING_LIGHT,
        kirby_assist_core::native_states::GUARD_ON,
        kirby_assist_core::native_states::GUARD,
        kirby_assist_core::native_states::status::OTTOTTO,
        kirby_assist_core::native_states::status::OTTOTTO_WAIT,
    ] {
        assert!(
            Scope::new(
                false,
                status,
                kirby_assist_core::native_states::status::WAIT,
                false,
                false,
                false,
                false,
                false,
                false
            )
            .acquire
        );
        assert!(
            !Scope::new(
                false,
                status,
                kirby_assist_core::native_states::status::WALK_BRAKE,
                false,
                false,
                false,
                false,
                false,
                false
            )
            .acquire
        );
        assert!(
            !Scope::new(
                false,
                status,
                kirby_assist_core::native_states::status::WAIT,
                true,
                false,
                false,
                false,
                false,
                false
            )
            .acquire
        );
    }
}
#[test]
fn ledge_suspension_and_capture_links_disable_all_search_and_follow() {
    use kirby_assist_core::reaction_rules::AutomaticObservationScope as Scope;
    for (status, air) in [
        (0, 0),
        (kirby_assist_core::native_states::GUARD, 0),
        (kirby_assist_core::native_states::ESCAPE, 0),
        (0x1e1, 2),
    ] {
        assert!(
            !Scope::new(true, status, air, false, false, true, true, true, true).observe_targets()
        );
        assert!(
            !Scope::new(false, status, air, false, true, true, true, true, true).observe_targets()
        );
    }
    let spin = Scope::new(false, 0x1e1, 2, false, false, true, true, false, false);
    assert!(!spin.acquire && spin.follow);
    let attack = Scope::new(false, 0x27, 0, false, false, false, false, false, true);
    assert!(
        !attack.acquire && attack.follow,
        "configured attack only follows its existing target"
    );
    for status in [
        kirby_assist_core::native_states::ESCAPE_F,
        kirby_assist_core::native_states::CATCH,
        kirby_assist_core::native_states::status::CATCH_CUT,
        kirby_assist_core::native_states::THROW,
        kirby_assist_core::native_states::status::DAMAGE,
        kirby_assist_core::native_states::status::DEAD,
    ] {
        assert!(!Scope::new(
            false,
            status,
            kirby_assist_core::native_states::status::WAIT,
            false,
            false,
            false,
            false,
            false,
            true
        )
        .observe_targets());
    }
    let dodge = Scope::new(false, 0x1f, 0, false, false, false, false, true, false);
    assert!(
        !dodge.acquire && dodge.follow,
        "only confirmed own dodge may reach its saved punish"
    );
}
#[test]
fn failed_shielded_requests_leave_the_physically_held_shield_intact() {
    use kirby_assist_core::manual_controls::AssistGuardHandoff;
    let trace = ButtonTrace {
        held: MAPPER_ZR,
        previous: MAPPER_ZR,
        ..Default::default()
    };
    for request in [MAPPER_R | MAPPER_ZR, MAPPER_B | MAPPER_ZR] {
        let mut handoff = AssistGuardHandoff::new();
        for _ in 0..4 {
            assert_eq!(handoff.sample(true, trace, MAPPER_ZR, u32::MAX, request), 0);
        }
        for _ in 0..8 {
            assert_eq!(handoff.sample(false, trace, MAPPER_ZR, 0, 0), 0);
        }
        // Once a confirmed spin suppresses guard, keep the old air-dodge protection.
        assert_eq!(
            handoff.sample(true, trace, MAPPER_ZR, u32::MAX, 0),
            MAPPER_ZR
        );
        assert_eq!(handoff.sample(false, trace, MAPPER_ZR, 0, 0), MAPPER_ZR);
        assert_eq!(
            handoff.sample(false, ButtonTrace::default(), MAPPER_ZR, 0, 0),
            0
        );
    }
}
#[test]
fn manual_grab_preempts_up_b_before_native_catch_in_both_layouts() {
    use kirby_assist_core::native_input::{automatic_escape, ControlLayout};
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        let human_grab = layout.physical_buttons(MAPPER_R);
        assert!(automatic_escape(
            layout, human_grab, 0, true, true, MAPPER_B
        ));
        assert!(automatic_escape(
            layout, human_grab, human_grab, false, true, 0
        ));
        assert!(
            !automatic_escape(layout, 0, 0, true, true, MAPPER_R),
            "synthetic grab is not human input"
        );
    }
}
#[test]
fn unconfirmed_up_b_never_enters_even_zero_horizontal_drift_or_neutral_release() {
    for actions in ["up_b:4,drift:opponent:1:69", "up_b:4"] {
        let seq=Catalog::parse(&format!("[bowser.roll.test]\nstart_delay=0\ntimeout=180\nreaction=auto_spot_dodge:35:12\nactions={actions}\n")).unwrap().entries()[0].sequence;
        let target = Opponent {
            position: [0., 0.],
            ..enemy(0., kirby_assist_core::native_states::ESCAPE)
        };
        let mut response = Response::new();
        assert!(response.start_up_b(
            seq,
            own(0., kirby_assist_core::native_states::GUARD),
            target
        ));
        let mut rules = Resolved::default();
        rules
            .add(Candidate {
                slot: 0,
                index: 0,
                sequence: seq,
            })
            .unwrap();
        let mut engine = Engine::default();
        let mut ts = targets(kirby_assist_core::native_states::ESCAPE);
        ts[0].as_mut().unwrap().position = [0.; 2];
        for n in 0..4 {
            let c = own(n as f32, kirby_assist_core::native_states::GUARD);
            assert_eq!(
                response.sample(true, settings(), c, Some(target)).requested,
                MAPPER_B | MAPPER_ZR
            );
            assert_eq!(
                engine
                    .sample_simple(true, &rules, settings(), c, ts, Some(target))
                    .requested,
                MAPPER_B | MAPPER_ZR
            );
        }
        assert_eq!(
            response
                .sample(
                    true,
                    settings(),
                    own(4., kirby_assist_core::native_states::GUARD),
                    Some(target)
                )
                .owned,
            0
        );
        assert_eq!(
            engine
                .sample_simple(
                    true,
                    &rules,
                    settings(),
                    own(4., kirby_assist_core::native_states::GUARD),
                    ts,
                    Some(target)
                )
                .owned,
            0
        );
        assert!(!response.configured_committed());
        assert!(!engine.committed());
    }
}
#[test]
fn unconfirmed_up_b_releases_on_shield_damage_stop_or_attack_recovery() {
    for (status, stop) in [
        (kirby_assist_core::native_states::GUARD_DAMAGE, false),
        (0x27, false),
        (kirby_assist_core::native_states::GUARD, true),
    ] {
        let mut response = Response::new();
        assert!(response.start_up_b(
            recipe(),
            own(0., kirby_assist_core::native_states::GUARD),
            enemy(0., kirby_assist_core::native_states::ESCAPE)
        ));
        assert_ne!(
            response
                .sample(
                    true,
                    settings(),
                    own(0., kirby_assist_core::native_states::GUARD),
                    Some(enemy(0., kirby_assist_core::native_states::ESCAPE))
                )
                .requested,
            0
        );
        assert_eq!(
            response
                .sample(
                    true,
                    settings(),
                    GameplayContext {
                        stopped: stop,
                        ..own(1., status)
                    },
                    Some(enemy(1., kirby_assist_core::native_states::ESCAPE))
                )
                .owned,
            0
        );
        assert!(!response.configured_committed());
    }
}
#[test]
fn captured_or_throwing_targets_cancel_punishes_but_grab_attempts_remain_defendable() {
    use kirby_assist_core::reaction_rules::automatic_target_available;
    for status in [
        kirby_assist_core::native_states::CATCH_PULL,
        kirby_assist_core::native_states::CATCH_DASH_PULL,
        kirby_assist_core::native_states::status::CATCH_WAIT,
        kirby_assist_core::native_states::status::CATCH_CUT,
        kirby_assist_core::native_states::THROW,
        kirby_assist_core::native_states::CAPTURE_PULLED,
        kirby_assist_core::native_states::THROWN,
        kirby_assist_core::native_states::status::DEAD,
        kirby_assist_core::native_states::status::REBIRTH,
    ] {
        assert!(!automatic_target_available(status));
        let mut response = Response::new();
        assert!(response.start_up_b(
            recipe(),
            own(0., 0),
            enemy(0., kirby_assist_core::native_states::ESCAPE)
        ));
        assert_eq!(
            response
                .sample(true, settings(), own(0., 0), Some(enemy(0., status)))
                .owned,
            0
        );
        assert!(!response.configured_committed());
    }
    for status in [
        kirby_assist_core::native_states::status::WAIT,
        kirby_assist_core::native_states::GUARD,
        kirby_assist_core::native_states::status::ATTACK,
        kirby_assist_core::native_states::status::ATTACK_AIR,
        kirby_assist_core::native_states::CATCH,
        kirby_assist_core::native_states::CATCH_DASH,
        kirby_assist_core::native_states::CATCH_TURN,
    ] {
        assert!(automatic_target_available(status));
    }
}
#[test]
fn suspension_keeps_only_a_bounded_inert_grab_receipt() {
    let seq=Catalog::parse("[bowser.roll.test]\nstart_delay=0\ntimeout=180\nreaction=auto_grab:35:24\nactions=input:r:0:0:4,up_b:4\n").unwrap().entries()[0].sequence;
    let mut response = Response::new();
    let target = enemy(0., kirby_assist_core::native_states::GUARD);
    assert!(response.start_sequence(seq, own(0., 0), target));
    assert_eq!(
        response
            .sample(true, settings(), own(0., 0), Some(target))
            .requested,
        MAPPER_R
    );
    response.mapped_grab(0, 2);
    response.suspend();
    for tick in 1..25 {
        response.delivery_tick(tick);
        let out = response.sample(false, settings(), own(tick as f32, 0), None);
        assert_eq!(out.owned, 0);
        assert_eq!(out.requested, 0);
        assert_eq!(out.stick, None);
    }
    assert!(!response.grab_delivery_pending());
    assert!(!response.configured_committed());
}

#[test]
fn ledge_suspension_cancels_native_grab_inputs_but_keeps_receipt_until_catch_or_expiry() {
    use kirby_assist_core::out_of_shield::{Context, OutOfShield};
    for caught in [false, true] {
        let mut tool = OutOfShield::new();
        let mut context = Context {
            object: 1,
            kind: kirby_assist_core::fighter_kinds::BOWSER,
            status: kirby_assist_core::native_states::GUARD,
            motion: 1,
            frame: 0.,
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
        };
        let out = tool.sample_automatic(
            true,
            context,
            own(0., kirby_assist_core::native_states::GUARD),
            targets(kirby_assist_core::native_states::GUARD),
            None,
            true,
            22.,
            35.,
            Some(12.),
        );
        assert_eq!(out.requested, MAPPER_R | MAPPER_ZR);
        tool.mapped_grab(0, 2);
        assert!(tool.grab_delivery_pending());
        for tick in 1..25 {
            // The actual mapper performs this suspension on every held-ledge poll.
            tool.suspend_automatic();
            tool.delivery_tick(tick);
            if caught {
                context.status = kirby_assist_core::native_states::CATCH;
            }
            context.frame = tick as f32;
            let out = tool.sample_automatic(
                false,
                context,
                own(tick as f32, context.status),
                [None; 8],
                None,
                true,
                22.,
                35.,
                Some(12.),
            );
            assert_eq!(out.owned, 0);
            assert_eq!(out.requested, 0);
            assert_eq!(out.stick, None);
            if caught {
                assert!(!tool.grab_delivery_pending());
            }
        }
        assert!(!tool.grab_delivery_pending());
        assert!(tool.target().is_none());
    }
}
