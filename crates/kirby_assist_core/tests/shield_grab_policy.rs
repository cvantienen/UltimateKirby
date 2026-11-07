//! Distinct shielded/unshielded grab policies and exclusive shield fallback.
use kirby_assist_core::{
    action_owner::{ActionOwnership, Owner},
    gameplay::{ControllerOutput, GameplayContext},
    native_input::{MAPPER_B, MAPPER_R, MAPPER_X, MAPPER_ZR},
    out_of_shield::{AutomaticConfig, Context, OutOfShield},
    reaction_rules::{Candidate, Engine, Resolved},
    roll_read::Target,
    sequence_config::{Catalog, Sequence},
    settings::AssistSettings,
};
fn own(status: i32, frame: f32) -> Context {
    Context {
        object: 1,
        kind: kirby_assist_core::fighter_kinds::BOWSER,
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
        gameplay: true,
        held: true,
        can_act: true,
        status: c.status,
        situation: if c.grounded {
            kirby_assist_core::native_states::situation::GROUND
        } else {
            kirby_assist_core::native_states::situation::AIR
        },
        stopped: c.stopped,
        motion_kind: c.motion,
        motion_frame: c.frame,
        motion_rate: c.rate,
        position: c.position,
        facing: c.facing,
        safe_ground: [true; 2],
        ..Default::default()
    }
}
fn target(status: i32, x: f32, y: f32) -> [Option<Target>; 8] {
    let mut ts = [None; 8];
    ts[0] = Some(Target {
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
    ts
}
fn recipe(event: &str, x: f32, y: f32, actions: &str) -> Sequence {
    Catalog::parse(&format!("[bowser.roll.test]\nstart_delay=0\ntimeout=240\nreaction={event}:{x}:{y}\nactions={actions}\n")).unwrap().entries()[0].sequence
}
fn sample(
    tool: &mut OutOfShield,
    c: Context,
    ts: [Option<Target>; 8],
    shielded: bool,
    custom: bool,
) -> ControllerOutput {
    let grab = recipe("auto_grab", 35., 24., "input:r:0:0:4");
    let shield = recipe(
        "auto_shield_grab",
        10.,
        4.,
        if custom {
            "input:x:0:0:2"
        } else {
            "input:r:0:0:4"
        },
    );
    let backup = recipe("auto_shield_backup", 20., 12., "up_b:4,drift:opponent:1:69");
    let out = tool.sample_automatic_configured(
        true,
        c,
        lc(c),
        ts,
        None,
        shielded,
        10.,
        35.,
        Some(12.),
        AssistSettings {
            fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
            ..Default::default()
        },
        AutomaticConfig {
            grab: Some(&grab),
            shield_grab: Some(&shield),
            backup: Some(&backup),
        },
    );
    assert_ne!(
        out.requested & (MAPPER_R | MAPPER_B),
        MAPPER_R | MAPPER_B,
        "Grab and Special must never be emitted together"
    );
    out
}
#[test]
fn shielded_front_grabs_do_not_require_enemy_shield_and_use_their_own_recipe() {
    for enemy in [
        0, 1, 3, 4, 0x1b, 0x1c, 0x1e, 0x27, 0x2c, 0x36, 0x47, 0x57, 0x59,
    ] {
        for status in [
            kirby_assist_core::native_states::status::WAIT,
            kirby_assist_core::native_states::GUARD_ON,
            kirby_assist_core::native_states::GUARD,
        ] {
            for custom in [false, true] {
                let mut tool = OutOfShield::new();
                let out = sample(
                    &mut tool,
                    own(status, 0.),
                    target(enemy, 10., 4.),
                    true,
                    custom,
                );
                let expected = if custom {
                    MAPPER_X
                        | if status == kirby_assist_core::native_states::status::WAIT {
                            0
                        } else {
                            MAPPER_ZR
                        }
                } else {
                    MAPPER_R
                        | if status == kirby_assist_core::native_states::status::WAIT {
                            0
                        } else {
                            MAPPER_ZR
                        }
                };
                assert_eq!(
                    out.requested, expected,
                    "own {status:#x}, enemy {enemy:#x}, custom {custom}"
                );
                assert!(!tool.shield_up_b_pending());
                assert_eq!(tool.target(), Some(2));
                assert_eq!(out.stick, Some([0.; 2]));
            }
        }
    }
}
#[test]
fn unshielded_movement_only_grabs_enemy_shield_and_reacts_on_first_shield_sample() {
    for status in [
        kirby_assist_core::native_states::status::WALK,
        kirby_assist_core::native_states::status::DASH,
        kirby_assist_core::native_states::status::RUN,
        kirby_assist_core::native_states::status::RUN_BRAKE,
        kirby_assist_core::native_states::status::TURN,
        kirby_assist_core::native_states::status::TURN_DASH,
        kirby_assist_core::native_states::status::TURN_RUN,
        kirby_assist_core::native_states::status::TURN_RUN_BRAKE,
        kirby_assist_core::native_states::status::SQUAT,
        kirby_assist_core::native_states::status::SQUAT_WAIT,
        kirby_assist_core::native_states::status::SQUAT_F,
    ] {
        let mut tool = OutOfShield::new();
        for enemy in [0, 3, 0x27, 0x36, 0x47] {
            assert_eq!(
                sample(
                    &mut tool,
                    own(status, 0.),
                    target(enemy, 10., 0.),
                    false,
                    false
                )
                .owned,
                0
            );
            assert!(tool.target().is_none());
        }
        let out = sample(
            &mut tool,
            own(status, 1.),
            target(kirby_assist_core::native_states::GUARD_ON, 10., 0.),
            false,
            false,
        );
        assert_eq!(
            out.requested, MAPPER_R,
            "movement status {status:#x} must grab immediately"
        );
        assert!(!tool.shield_up_b_pending());
    }
}
#[test]
fn shield_backup_is_immediate_for_behind_ungrabbable_and_outside_grab_limits() {
    for (status, x, y, grab_ready, expected) in [
        (0, 10., 4., true, MAPPER_R | MAPPER_ZR),
        (0, -10., 0., true, MAPPER_B | MAPPER_ZR),
        (0, 0., 0., true, MAPPER_B | MAPPER_ZR),
        (0, 10.01, 0., true, MAPPER_B | MAPPER_ZR),
        (0, 10., 4.01, true, MAPPER_B | MAPPER_ZR),
        (0x36, 20., 12., true, MAPPER_B | MAPPER_ZR),
        (0, 20.01, 0., true, 0),
        (0, 15., 12.01, true, 0),
        (0, 5., 0., false, MAPPER_B | MAPPER_ZR),
        (0x53, 5., 0., true, MAPPER_B | MAPPER_ZR),
        (0x88, 5., 0., true, MAPPER_B | MAPPER_ZR),
    ] {
        let mut tool = OutOfShield::new();
        let out = sample(
            &mut tool,
            Context {
                can_grab: grab_ready,
                ..own(kirby_assist_core::native_states::GUARD, 0.)
            },
            target(status, x, y),
            true,
            false,
        );
        assert_eq!(out.requested, expected, "enemy {status:#x}, x {x}, y {y}");
        assert_eq!(out.owned != 0, expected != 0);
        assert_eq!(tool.shield_up_b_pending(), expected & MAPPER_B != 0);
    }
}
#[test]
fn illegal_native_states_and_captured_or_invulnerable_targets_never_take_controls() {
    for status in [
        kirby_assist_core::native_states::status::GUARD_OFF,
        kirby_assist_core::native_states::status::GUARD_DAMAGE,
        kirby_assist_core::native_states::status::ESCAPE,
        kirby_assist_core::native_states::status::ESCAPE_F,
        kirby_assist_core::native_states::status::ESCAPE_AIR,
        kirby_assist_core::native_states::status::ATTACK,
        kirby_assist_core::native_states::status::CATCH,
        kirby_assist_core::native_states::status::CATCH_CUT,
        kirby_assist_core::native_states::status::THROW,
        kirby_assist_core::native_states::status::CAPTURE_PULLED,
        kirby_assist_core::native_states::status::DAMAGE,
        kirby_assist_core::native_states::status::DOWN,
        kirby_assist_core::native_states::status::PASSIVE,
        kirby_assist_core::native_states::status::DEAD,
        kirby_assist_core::native_states::status::SPECIAL_HI,
    ] {
        let mut tool = OutOfShield::new();
        assert_eq!(
            sample(&mut tool, own(status, 0.), target(0, 5., 0.), true, false).owned,
            0,
            "own status {status:#x}"
        );
    }
    for status in [
        kirby_assist_core::native_states::CATCH_PULL,
        kirby_assist_core::native_states::status::CATCH_CUT,
        kirby_assist_core::native_states::THROW,
        kirby_assist_core::native_states::CAPTURE_PULLED,
        kirby_assist_core::native_states::THROWN,
        kirby_assist_core::native_states::status::CLIFF_WAIT,
        kirby_assist_core::native_states::status::DEAD,
        kirby_assist_core::native_states::status::REBIRTH,
    ] {
        assert_eq!(
            sample(
                &mut OutOfShield::new(),
                own(kirby_assist_core::native_states::GUARD, 0.),
                target(status, 5., 0.),
                true,
                false
            )
            .owned,
            0
        );
    }
    let mut ts = target(0x36, 5., 0.);
    ts[0].as_mut().unwrap().vulnerable_in = None;
    assert_eq!(
        sample(
            &mut OutOfShield::new(),
            own(kirby_assist_core::native_states::GUARD, 0.),
            ts,
            true,
            false
        )
        .owned,
        0
    );
    for case in 0..4 {
        let mut c = own(kirby_assist_core::native_states::GUARD, 0.);
        match case {
            0 => c.stopped = true,
            1 => c.grounded = false,
            2 => c.shield_ready = false,
            _ => {
                c.can_grab = false;
                c.can_up_special = false;
            }
        }
        assert_eq!(
            sample(&mut OutOfShield::new(), c, target(0, 5., 0.), true, false).owned,
            0
        );
    }
}
#[test]
fn delivered_nonshield_target_grab_cannot_turn_into_up_b_or_a_second_punish() {
    let mut tool = OutOfShield::new();
    assert_eq!(
        sample(
            &mut tool,
            own(kirby_assist_core::native_states::GUARD, 0.),
            target(0x36, 5., 0.),
            true,
            false
        )
        .requested,
        MAPPER_R | MAPPER_ZR
    );
    tool.mapped_grab(0, 2);
    let out = sample(
        &mut tool,
        own(kirby_assist_core::native_states::GUARD, 1.),
        target(0x36, -5., 0.),
        true,
        false,
    );
    assert_eq!(out.requested, 0);
    assert!(tool.grab_delivery_pending());
    assert!(!tool.shield_up_b_pending());
    let mut rules = Resolved::default();
    rules
        .add(Candidate {
            slot: 0,
            index: 0,
            sequence: recipe("auto_roll", 30., 12., "up_b:4"),
        })
        .unwrap();
    let engine = Engine::default();
    let mut owner = ActionOwnership::new();
    assert!(!engine.preempt_automatic_grab(
        true,
        &rules,
        lc(own(kirby_assist_core::native_states::GUARD, 1.)),
        &target(kirby_assist_core::native_states::ESCAPE_F, 5., 0.),
        None,
        &mut tool,
        &mut owner
    ));
    assert_eq!(
        sample(
            &mut tool,
            own(kirby_assist_core::native_states::CATCH, 2.),
            target(0x36, 5., 0.),
            true,
            false
        )
        .owned,
        0
    );
    assert!(!tool.grab_delivery_pending());
    assert!(!tool.shield_up_b_pending());
    assert!(owner.allows(Owner::OutOfShield));
}
#[test]
fn committed_shield_up_b_cannot_switch_to_a_front_grab_mid_pulse() {
    let mut tool = OutOfShield::new();
    assert_eq!(
        sample(
            &mut tool,
            own(kirby_assist_core::native_states::GUARD, 0.),
            target(0x36, -5., 0.),
            true,
            false
        )
        .requested,
        MAPPER_B | MAPPER_ZR
    );
    tool.mapped_up_special(0, 2);
    let out = sample(
        &mut tool,
        own(kirby_assist_core::native_states::GUARD, 1.),
        target(0x36, 5., 0.),
        true,
        false,
    );
    assert_eq!(out.requested, MAPPER_B | MAPPER_ZR);
    assert!(!tool.automatic_grab_pending());
}

#[test]
fn shield_grab_policy_tracks_facing_and_enemy_shield_release_without_switching_to_special() {
    for facing in [-1., 1.] {
        let mut tool = OutOfShield::new();
        let c = Context {
            facing,
            ..own(kirby_assist_core::native_states::GUARD, 0.)
        };
        assert_eq!(
            sample(
                &mut tool,
                c,
                target(kirby_assist_core::native_states::GUARD, 5. * facing, 0.),
                true,
                false
            )
            .requested,
            MAPPER_R | MAPPER_ZR
        );
        tool.mapped_grab(0, 2);
        let next = Context { frame: 1., ..c };
        assert_eq!(
            sample(&mut tool, next, target(0x36, 5. * facing, 0.), true, false).requested,
            MAPPER_R | MAPPER_ZR
        );
        assert!(tool.grab_delivery_pending());
        assert!(!tool.shield_up_b_pending());
        tool.mapped_grab(1, 2);
        let behind = Context { frame: 2., ..c };
        assert_eq!(
            sample(
                &mut tool,
                behind,
                target(0x36, -5. * facing, 0.),
                true,
                false
            )
            .requested,
            0
        );
        assert!(tool.grab_delivery_pending());
        assert!(!tool.shield_up_b_pending());
    }
}

#[test]
fn an_edited_shield_backup_uses_its_own_recipe_without_preliminary_grab_or_special() {
    let grab = recipe("auto_shield_grab", 10., 4., "input:r:0:0:4");
    let backup = recipe("auto_shield_backup", 20., 12., "input:x:0:0:2");
    for (status, x, y) in [(0, -5., 0.), (0x36, 15., 0.), (0x53, 5., 0.), (0, 5., 8.)] {
        let mut tool = OutOfShield::new();
        let c = own(kirby_assist_core::native_states::GUARD, 0.);
        let out = tool.sample_automatic_configured(
            true,
            c,
            lc(c),
            target(status, x, y),
            None,
            true,
            10.,
            35.,
            Some(12.),
            AssistSettings {
                fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
                ..Default::default()
            },
            AutomaticConfig {
                shield_grab: Some(&grab),
                backup: Some(&backup),
                ..Default::default()
            },
        );
        assert_eq!(out.requested, MAPPER_X | MAPPER_ZR);
        assert!(tool.configured_committed());
        assert!(!tool.grab_delivery_pending());
    }
}
