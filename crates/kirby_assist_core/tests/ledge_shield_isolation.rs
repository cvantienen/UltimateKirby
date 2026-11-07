//! Ledge ownership, literal shield timing, and first-legal-frame shield grabs.
use kirby_assist_core::{
    action_owner::{ActionOwnership, Intent, Owner},
    fighter_profiles::presets::PresetSlot,
    gameplay::{ControllerOutput, GameplayContext, MotionTiming, Opponent},
    grab_handoff::GrabHandoff,
    ledge_trap::{InputTimeline, LedgeSettings, SEQUENCE_BUTTONS},
    native_input::{MAPPER_B, MAPPER_R, MAPPER_ZR},
    out_of_shield::{AutomaticConfig, Context, OutOfShield},
    roll_read::Target,
    sequence_config::{Catalog, Sequence},
};
fn c(status: i32, frame: f32) -> GameplayContext {
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
fn own(context: GameplayContext) -> Context {
    Context {
        object: 1,
        kind: kirby_assist_core::fighter_kinds::BOWSER,
        status: context.status,
        motion: context.motion_kind,
        frame: context.motion_frame,
        rate: context.motion_rate,
        stopped: context.stopped,
        grounded: context.situation == 0,
        position: context.position,
        facing: context.facing,
        shield_ready: true,
        can_grab: true,
        can_jump: true,
        can_up_special: true,
        threatened: false,
    }
}
fn targets(status: i32, x: f32, frame: f32) -> [Option<Target>; 8] {
    let mut ts = [None; 8];
    ts[0] = Some(Target {
        id: 2,
        status,
        motion: 2,
        frame,
        rate: 1.,
        position: [x, 0.],
        velocity_x: 0.,
        vulnerable_in: Some(0.),
        actionable_in: None,
    });
    ts
}
fn enemy(status: i32, x: f32, frame: f32) -> Opponent {
    Opponent {
        id: 2,
        status,
        position: [x, 0.],
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
fn settings() -> LedgeSettings {
    LedgeSettings {
        fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
        ..Default::default()
    }
}
fn recipe(event: &str, actions: &str) -> Sequence {
    Catalog::parse(&format!(
        "[bowser.roll.test]\nstart_delay=0\nreaction={event}:35:24\nactions={actions}\n"
    ))
    .unwrap()
    .entries()[0]
        .sequence
}
fn sample(
    tool: &mut OutOfShield,
    context: GameplayContext,
    ts: [Option<Target>; 8],
    shielded: bool,
    grab: &Sequence,
    shield: &Sequence,
) -> ControllerOutput {
    tool.sample_automatic_configured(
        true,
        own(context),
        context,
        ts,
        None,
        shielded,
        22.,
        35.,
        Some(8.),
        settings(),
        AutomaticConfig {
            grab: Some(grab),
            shield_grab: Some(shield),
            backup: None,
        },
    )
}
fn shield() -> ControllerOutput {
    ControllerOutput {
        requested: MAPPER_ZR,
        owned: SEQUENCE_BUTTONS,
        stick: Some([0.; 2]),
        ..Default::default()
    }
}
#[test]
fn saved_bowser_chase_first_grab_uses_immediate_native_grab_instead_of_69_frame_run() {
    let catalog = Catalog::parse(include_str!(
        "../../../deployment/ultimate/kirby_assist/fighters/bowser.txt"
    ))
    .unwrap();
    let grab = &catalog.find(15, PresetSlot::Roll, "auto_grab").unwrap();
    let shield = &catalog
        .find(15, PresetSlot::Roll, "auto_shield_grab")
        .unwrap();
    let mut tool = OutOfShield::new();
    let out = sample(
        &mut tool,
        c(0, 0.),
        targets(kirby_assist_core::native_states::GUARD, 10., 0.),
        false,
        grab,
        shield,
    );
    assert_eq!(out.requested, MAPPER_R);
    assert_eq!(out.stick, Some([0.; 2]));
    tool.mapped_grab(0, 2);
    for frame in 1..70 {
        let out = sample(
            &mut tool,
            c(kirby_assist_core::native_states::CATCH, frame as f32),
            targets(
                kirby_assist_core::native_states::CAPTURE_PULLED,
                10.,
                frame as f32,
            ),
            false,
            grab,
            shield,
        );
        assert_eq!(
            out,
            ControllerOutput::default(),
            "no retained drift after native Catch"
        );
    }
    let mut tool = OutOfShield::new();
    let out = sample(
        &mut tool,
        c(0, 0.),
        targets(kirby_assist_core::native_states::GUARD, 30., 0.),
        false,
        grab,
        shield,
    );
    assert_eq!(out.requested, 0);
    assert_eq!(out.stick, Some([1., 0.]));
    tool.mapped_approach(0, 2);
    let out = sample(
        &mut tool,
        c(3, 1.),
        targets(kirby_assist_core::native_states::GUARD, 30., 1.),
        false,
        grab,
        shield,
    );
    assert_eq!(
        out.requested, MAPPER_R,
        "native dash needs no 69-frame chase"
    );
}
#[test]
fn a_fresh_shield_grab_is_sent_on_its_first_legal_observation_without_dropping_guard() {
    let grab = recipe("auto_grab", "input:r:0:0:4");
    let shield = recipe("auto_shield_grab", "input:r:0:0:4");
    for status in [
        kirby_assist_core::native_states::GUARD_ON,
        kirby_assist_core::native_states::GUARD,
    ] {
        for enemy_status in [0, kirby_assist_core::native_states::GUARD, 0x27, 0x57] {
            let mut tool = OutOfShield::new();
            let out = sample(
                &mut tool,
                c(status, 0.),
                targets(enemy_status, 10., 0.),
                true,
                &grab,
                &shield,
            );
            assert_eq!(out.requested, MAPPER_R | MAPPER_ZR);
            assert_eq!(out.stick, Some([0.; 2]));
            assert!(!tool.approach_pending());
        }
    }
}
#[test]
fn shield_entry_replaces_an_undelivered_custom_grab_plan_on_the_same_observation() {
    let grab = recipe("auto_grab", "wait:10,input:r:0:0:4");
    let shield = recipe("auto_shield_grab", "input:r:0:0:4");
    let mut tool = OutOfShield::new();
    assert_eq!(
        sample(
            &mut tool,
            c(0, 0.),
            targets(kirby_assist_core::native_states::GUARD, 10., 0.),
            false,
            &grab,
            &shield
        )
        .requested,
        0
    );
    let out = sample(
        &mut tool,
        c(kirby_assist_core::native_states::GUARD, 1.),
        targets(0, 10., 1.),
        true,
        &grab,
        &shield,
    );
    assert_eq!(out.requested, MAPPER_R | MAPPER_ZR);
    assert_eq!(out.stick, Some([0.; 2]));
}
#[test]
fn an_undelivered_custom_grab_cannot_chase_a_lost_unshielded_or_rear_target() {
    let grab = recipe("auto_grab", "wait:10,input:r:0:0:4");
    let shield = recipe("auto_shield_grab", "input:r:0:0:4");
    for (status, x) in [
        (0, 10.),
        (kirby_assist_core::native_states::GUARD, -10.),
        (kirby_assist_core::native_states::GUARD, 36.),
    ] {
        let mut tool = OutOfShield::new();
        sample(
            &mut tool,
            c(0, 0.),
            targets(kirby_assist_core::native_states::GUARD, 10., 0.),
            false,
            &grab,
            &shield,
        );
        assert_eq!(
            sample(
                &mut tool,
                c(0, 1.),
                targets(status, x, 1.),
                false,
                &grab,
                &shield
            ),
            ControllerOutput::default()
        );
        assert!(!tool.configured_committed());
    }
}
#[test]
fn ledge_takeover_waits_for_prior_receipts_and_never_replaces_a_queued_grab_with_special() {
    for previous in [
        None,
        Some(Owner::OutOfShield),
        Some(Owner::OptionRead),
        Some(Owner::RollRead),
    ] {
        let mut h = GrabHandoff::new();
        h.observe_native(true, Some(c(kirby_assist_core::native_states::GUARD, 0.)));
        h.mapped(0, previous, true, true, true);
        let mut ownership = ActionOwnership::new();
        if let Some(owner) = previous {
            ownership.select(true, &[Intent::input_sequence(owner, 1)]);
        }
        // Explicit cancellation precedes a requested ledge takeover. Passive
        // admission suspension alone cannot replace the existing sequence.
        if let Some(owner) = previous {
            ownership.release(owner);
        }
        ownership.set_automatic_suspended(true);
        assert!(!h.reserve_ledge_inputs());
        assert!(h.pending());
        assert!(!h.permits(
            Some(Owner::Ledge),
            ControllerOutput {
                requested: MAPPER_B,
                ..shield()
            },
            None
        ));
        h.observe_native(true, Some(c(0, 0.)));
        for frame in 1..=24 {
            h.observe_native(true, Some(c(0, frame as f32)));
        }
        assert!(!h.pending());
        assert!(h.reserve_ledge_inputs());
        let mut intents = [
            Intent::input_sequence(Owner::Ledge, SEQUENCE_BUTTONS),
            Intent::input_sequence(Owner::OutOfShield, SEQUENCE_BUTTONS),
        ];
        h.restrict(&mut intents);
        assert_eq!(ownership.select(true, &intents), Some(Owner::Ledge));
        assert!(h.permits(Some(Owner::Ledge), shield(), None));
        assert!(h.permits(
            Some(Owner::Ledge),
            ControllerOutput {
                requested: MAPPER_B,
                stick: Some([0., 1.]),
                ..shield()
            },
            None
        ));
    }
}
#[test]
fn a_ledge_tapes_own_grab_receipt_preserves_both_authored_four_frame_shield_holds() {
    let sequence = recipe("auto_grab", "input:r:0:0:4,shield:4,shield:4");
    let mut tape = InputTimeline::new(sequence, enemy(0x77, 10., 0.));
    let mut h = GrabHandoff::new();
    for tick in 0..12 {
        let context = c(kirby_assist_core::native_states::GUARD, tick as f32);
        h.observe_native(true, Some(context));
        h.reserve_ledge_inputs();
        let out = tape.sample(
            settings(),
            context,
            Some(enemy(0x77, 10., tick as f32)),
            None,
            0.,
        );
        assert_eq!(
            out.requested,
            if tick < 4 {
                MAPPER_R | MAPPER_ZR
            } else {
                MAPPER_ZR
            },
            "tick {tick}"
        );
        assert!(
            h.permits(Some(Owner::Ledge), out, None),
            "no shield step may be cancelled"
        );
        h.mapped(tick, Some(Owner::Ledge), tick < 4, tick < 4, true);
    }
    assert!(h.pending());
    assert!(!h.permits(
        Some(Owner::Ledge),
        ControllerOutput {
            requested: MAPPER_B,
            stick: Some([0., 1.]),
            ..shield()
        },
        None
    ));
}
#[test]
fn a_queued_grab_never_blocks_passive_defensive_shield() {
    let mut h = GrabHandoff::new();
    h.mapped(0, Some(Owner::OutOfShield), true, true, true);
    let mut intents = [
        Intent::input_sequence(Owner::Defense, SEQUENCE_BUTTONS),
        Intent::input_sequence(Owner::RollRead, SEQUENCE_BUTTONS),
    ];
    h.restrict(&mut intents);
    assert!(intents[0].present);
    assert!(!intents[1].present);
    assert!(h.permits(Some(Owner::Defense), shield(), None));
    for bad in [
        ControllerOutput {
            requested: MAPPER_ZR | MAPPER_B,
            ..shield()
        },
        ControllerOutput {
            stick: Some([1., 0.]),
            ..shield()
        },
    ] {
        assert!(!h.permits(Some(Owner::Defense), bad, None));
    }
}
#[test]
fn static_shield_receipts_expire_from_already_observed_enemy_progress_without_an_extra_grab_frame()
{
    let context = c(kirby_assist_core::native_states::GUARD, 0.);
    let mut h = GrabHandoff::new();
    h.observe_native(true, Some(context));
    h.mapped(0, Some(Owner::OutOfShield), true, true, true);
    for _ in 0..1000 {
        h.observe_native(true, Some(context));
        h.observe_enemy_progress(context, Some(enemy(0x27, 10., 0.)));
        assert!(h.pending());
    }
    for frame in 1..24 {
        h.observe_enemy_progress(context, Some(enemy(0x27, 10., frame as f32)));
        assert!(h.pending());
    }
    h.observe_enemy_progress(context, Some(enemy(0x27, 10., 24.)));
    assert!(!h.pending());
    let grab = recipe("auto_grab", "input:r:0:0:4");
    let shield = recipe("auto_shield_grab", "input:r:0:0:4");
    let mut tool = OutOfShield::new();
    let out = sample(
        &mut tool,
        context,
        targets(0x27, 10., 24.),
        true,
        &grab,
        &shield,
    );
    assert_eq!(out.requested, MAPPER_R | MAPPER_ZR);
    assert!(h.permits(Some(Owner::OutOfShield), out, None));
}
#[test]
fn stopped_or_missing_clock_samples_cannot_be_charged_to_a_queued_grab() {
    let context = c(kirby_assist_core::native_states::GUARD, 0.);
    let mut h = GrabHandoff::new();
    h.observe_native(true, Some(context));
    h.mapped(0, None, true, true, true);
    h.observe_enemy_progress(context, Some(enemy(0, 10., 0.)));
    for frame in 1..23 {
        h.observe_enemy_progress(context, Some(enemy(0, 10., frame as f32)));
    }
    h.observe_enemy_progress(
        GameplayContext {
            stopped: true,
            ..context
        },
        Some(enemy(0, 10., 25.)),
    );
    h.observe_enemy_progress(context, Some(enemy(0, 10., 30.)));
    assert!(h.pending());
    h.observe_enemy_progress(context, None);
    h.observe_enemy_progress(context, Some(enemy(0, 10., 32.)));
    assert!(h.pending());
}

#[test]
fn a_new_ledge_grab_requires_the_prior_receipt_to_end_and_a_new_input_edge() {
    let mut h = GrabHandoff::new();
    h.observe_native(true, Some(c(kirby_assist_core::native_states::GUARD, 0.)));
    h.mapped(0, Some(Owner::OutOfShield), true, true, true);
    assert!(!h.reserve_ledge_inputs());
    h.observe_native(true, Some(c(kirby_assist_core::native_states::CATCH, 0.)));
    h.observe_native(true, Some(c(kirby_assist_core::native_states::GUARD, 0.)));
    h.mapped(1, Some(Owner::Ledge), true, true, true);
    assert!(
        !h.pending(),
        "a held tail cannot create a second grab receipt"
    );
    h.mapped(2, None, false, false, false);
    h.mapped(3, Some(Owner::Ledge), true, true, true);
    assert!(h.pending());
    assert!(h.reserve_ledge_inputs());
    assert!(h.pending(), "the new ledge tape keeps its own receipt");
    assert!(h.permits(Some(Owner::Ledge), shield(), None));
    assert!(!h.permits(
        Some(Owner::Ledge),
        ControllerOutput {
            requested: MAPPER_B,
            stick: Some([0., 1.]),
            ..shield()
        },
        None
    ));
}

#[test]
fn manual_retreat_ends_an_undelivered_unshielded_custom_grab_but_not_a_shield_tilt_grab() {
    let grab = recipe("auto_grab", "wait:10,input:r:0:0:4");
    let shield = recipe("auto_shield_grab", "input:r:0:0:4");
    let mut tool = OutOfShield::new();
    let out = sample(
        &mut tool,
        c(0, 0.),
        targets(kirby_assist_core::native_states::GUARD, 10., 0.),
        false,
        &grab,
        &shield,
    );
    assert_ne!(out.owned, 0);
    assert_eq!(
        tool.with_grab_movement(out, [-0.7, 0.]),
        ControllerOutput::default()
    );
    assert!(!tool.configured_committed());
    let mut tool = OutOfShield::new();
    let out = sample(
        &mut tool,
        c(kirby_assist_core::native_states::GUARD, 0.),
        targets(0, 10., 0.),
        true,
        &grab,
        &shield,
    );
    let out = tool.with_grab_movement(out, [-0.4, 0.]);
    assert_eq!(out.requested, MAPPER_R | MAPPER_ZR);
    assert_eq!(out.stick, Some([0.; 2]));
}
