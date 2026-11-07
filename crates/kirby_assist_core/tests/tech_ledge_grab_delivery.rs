//! Actual ledge recipe, delayed native Grab, and tech/mapper handoff regressions.
use kirby_assist_core::{
    action_owner::{ActionOwnership, Intent, Owner},
    fighter_profiles::presets::PresetSlot,
    grab_handoff::GrabHandoff,
    ledge_trap::{
        InputTimeline, LedgeContext, LedgeOutput, LedgeSettings, MotionTiming, Opponent,
        SEQUENCE_BUTTONS,
    },
    manual_controls::{ManualOverlay, TurboOutput},
    native_input::*,
    sequence_config::Catalog,
    tech_input::{mapped_guard, TechContext, TechInput, TechOutput},
};
fn c(status: i32, frame: f32) -> LedgeContext {
    LedgeContext {
        held: true,
        gameplay: true,
        status,
        situation: 0,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.,
        facing: 1.,
        ..Default::default()
    }
}
fn enemy(x: f32, frame: f32) -> Opponent {
    Opponent {
        id: 2,
        status: 0x78,
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
fn packet(buttons: u32, y: f32) -> LedgeOutput {
    LedgeOutput {
        owned: SEQUENCE_BUTTONS,
        requested: buttons,
        stick: Some([1., y]),
        ..Default::default()
    }
}
fn tc(frame: f32, previous_guard: bool) -> TechContext {
    TechContext {
        object: 1,
        status: 0x49,
        motion: 1,
        frame,
        rate: 1.,
        stopped: false,
        remaining_stop: 0,
        contact_in: Some(2.),
        input_delay: 0.,
        previous_guard,
        prepare_guard: true,
    }
}
#[test]
fn exact_bowser_shield_getup_attack_grab_keeps_guard_through_range_then_grabs() {
    let cat = Catalog::parse(include_str!(
        "../../../deployment/ultimate/kirby_assist/assembled/sequences.txt"
    ))
    .unwrap();
    let seq = cat
        .entries()
        .iter()
        .find(|e| e.fighter_kind == 15 && e.slot == PresetSlot::Attack && e.choice == "shield_grab")
        .unwrap()
        .sequence;
    let mut t = InputTimeline::new(seq, enemy(30., 0.));
    let settings = LedgeSettings::default();
    let anchor = Some([50., 0.]);
    assert_eq!(
        t.sample(settings, c(0x1c, 0.), Some(enemy(30., 0.)), anchor, 1.)
            .requested,
        MAPPER_ZR
    );
    assert_eq!(
        t.sample(settings, c(0x1e, 1.), Some(enemy(30., 1.)), anchor, 1.)
            .requested,
        MAPPER_ZR
    );
    // The saved fighter-range check waits in shield instead of dropping it.
    assert_eq!(
        t.sample(settings, c(0x1c, 2.), Some(enemy(30., 2.)), anchor, 1.)
            .requested,
        MAPPER_ZR
    );
    let out = t.sample(settings, c(0x1c, 3.), Some(enemy(1., 3.)), anchor, 1.);
    assert_eq!(out.requested, MAPPER_R | MAPPER_ZR);
    assert_eq!(out.stick, Some([0.; 2]));
    assert_eq!(
        t.sample(settings, c(0x37, 4.), Some(enemy(1., 4.)), anchor, 1.)
            .owned,
        0
    );
    assert!(
        t.finished(),
        "native catch ends the recipe before its Side-B fallback"
    );
}
#[test]
fn all_direct_literal_grab_forms_preserve_shield_without_a_drop_frame() {
    for action in [
        "grab:ignore",
        "facing_input:r:0:0:4",
        "opponent_input:r:0:0:4",
        "controller_input:r:0:0:0:0:4",
    ] {
        let source = format!(
            "[bowser.attack.test]\nstart_delay=0\nactions=shield_until_block:90,{action}\n"
        );
        let cat = Catalog::parse(&source).unwrap();
        let mut t = InputTimeline::new(cat.entries()[0].sequence, enemy(5., 0.));
        t.sample(
            LedgeSettings::default(),
            c(0x1e, 0.),
            Some(enemy(5., 0.)),
            Some([50., 0.]),
            1.,
        );
        let out = t.sample(
            LedgeSettings::default(),
            c(0x1c, 1.),
            Some(enemy(5., 1.)),
            Some([50., 0.]),
            1.,
        );
        assert_eq!(out.requested, MAPPER_R | MAPPER_ZR, "{action}");
    }
}
#[test]
fn every_grab_owner_blocks_other_punishes_and_its_own_up_b_until_native_receipt() {
    for owner in [
        Owner::Ledge,
        Owner::RollRead,
        Owner::OptionRead,
        Owner::OutOfShield,
        Owner::Shortcut,
        Owner::Trump,
        Owner::Slide,
    ] {
        let mut h = GrabHandoff::default();
        h.observe(true, 0, 0);
        h.mapped(0, Some(owner), true, true, true);
        assert!(h.pending());
        assert!(h.permits(Some(owner), packet(MAPPER_R | MAPPER_ZR, 0.), Some([0.; 2])));
        assert!(!h.permits(Some(owner), packet(MAPPER_B, 1.), None));
        assert!(!h.permits(Some(owner), packet(0, 1.), None));
        assert!(!h.permits(Some(owner), packet(MAPPER_R, 0.), Some([0., 1.])));
        let mut intents = [
            Intent::input_sequence(owner, SEQUENCE_BUTTONS),
            Intent::input_sequence(Owner::Safety, 1),
        ];
        h.restrict(&mut intents);
        let mut arbitration = ActionOwnership::new();
        assert_eq!(arbitration.select(true, &intents), Some(Owner::Safety));
        for other in [
            Owner::Ledge,
            Owner::RollRead,
            Owner::OptionRead,
            Owner::OutOfShield,
            Owner::GrabDodge,
        ] {
            if other != owner {
                let mut intents = [Intent::input_sequence(other, SEQUENCE_BUTTONS)];
                h.restrict(&mut intents);
                assert!(!intents[0].present);
            }
        }
        h.observe(true, 0x37, 0);
        assert!(!h.pending());
    }
}
#[test]
fn grab_receipts_expire_without_poll_duplication_hangs_or_rearming_the_same_hold() {
    let mut h = GrabHandoff::default();
    h.mapped(0, Some(Owner::Ledge), true, true, true);
    h.observe(true, -1, 0); // A menu change temporarily invalidated copied facts.
    assert!(
        h.pending(),
        "missing fresh facts must not forget delivered R"
    );
    for _ in 0..100 {
        h.mapped(0, Some(Owner::Ledge), true, true, true);
        h.observe(true, 0, 0);
        assert!(h.pending());
    }
    for tick in 1..24 {
        h.mapped(tick, None, false, false, true);
    }
    h.observe(true, 0, 0);
    assert!(!h.pending());
    h.mapped(24, Some(Owner::RollRead), true, true, true);
    assert!(h.pending());
    h.observe(true, 0x49, 2);
    assert!(
        !h.pending(),
        "damage/tech never waits behind a Grab receipt"
    );
    let mut held = GrabHandoff::default();
    for tick in 0..30 {
        held.observe(true, 0, 0);
        held.mapped(tick, Some(Owner::Ledge), true, true, true);
    }
    assert!(
        !held.pending(),
        "continuous rejected Grab cannot restart the watchdog forever"
    );
}
#[test]
fn first_grab_packet_never_contains_synthetic_throw_directions_or_extra_attacks() {
    let out = GrabHandoff::grab_packet(packet(MAPPER_R | MAPPER_B | MAPPER_A | MAPPER_ZR, 1.));
    assert_eq!(out.requested, MAPPER_R | MAPPER_ZR);
    assert_eq!(out.stick, Some([1., 0.]));
    // Native manual throws are outside the receipt: no physical input is rewritten.
    let mut h = GrabHandoff::default();
    h.mapped(0, None, false, true, true);
    h.observe(true, 0x3e, 0);
    assert!(!h.pending());
    assert!(h.permits(None, LedgeOutput::default(), None));
}
#[test]
fn a_tech_preempts_every_committed_mode_and_overrides_stale_mapped_guard() {
    for old in [
        Owner::Ledge,
        Owner::OutOfShield,
        Owner::RollRead,
        Owner::OptionRead,
        Owner::GrabDodge,
        Owner::Shortcut,
    ] {
        let mut owners = ActionOwnership::new();
        owners.select(true, &[Intent::input_sequence(old, 1)]);
        owners.set_automatic_suspended(true);
        owners.set_ledge_priority(true);
        let mut tech = TechInput::default();
        let out = tech.sample(true, true, 40, tc(0., false));
        assert!(out.press);
        let winner = owners.select(
            true,
            &[
                Intent::input_sequence(old, 1),
                Intent {
                    owner: Owner::Safety,
                    present: out.press || out.owns_guard,
                    committed: false,
                },
            ],
        );
        assert_eq!(winner, Some(Owner::Safety));
        let release = mapped_guard(
            0x8,
            TechOutput {
                press: false,
                owns_guard: true,
            },
        );
        assert_eq!(release & 0x8, 0);
        // Even if native mapping drops the raw request, the selected tech packet is Guard.
        let mapped = mapped_guard(0, out);
        assert_ne!(mapped & 0x8, 0);
        tech.mapped_press();
        let after = tech.sample(true, true, 40, tc(1., true));
        assert!(!after.press);
        assert!(tech.cooldown_remaining() > 0.);
        assert_eq!(mapped_guard(mapped, after) & 0x8, 0);
    }
}
#[test]
fn fresh_tech_rearms_raw_guard_after_an_automatic_mode_filtered_its_native_guard() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        let guard = layout.physical_buttons(MAPPER_ZR);
        let mut overlay = ManualOverlay::default();
        let physical = ButtonTrace::default();
        let still = overlay.apply_reaction_controls(
            physical,
            [0.; 2],
            TurboOutput {
                consume_dpad: false,
                stick: None,
            },
            0,
            guard,
            guard,
            false,
        );
        assert_ne!(still.buttons.pressed & guard, 0);
        // Raw Guard stayed held; an automatic post-filter removed native Guard.
        let mut tech = TechInput::default();
        assert!(tech.sample(true, true, 40, tc(0., false)).press);
        overlay.rearm_tech_guard(guard, physical);
        let fresh = overlay.apply_reaction_controls(
            physical,
            [0.; 2],
            TurboOutput {
                consume_dpad: false,
                stick: None,
            },
            0,
            guard,
            guard,
            false,
        );
        assert_ne!(
            fresh.buttons.pressed & guard,
            0,
            "new tech must not inherit the old raw hold"
        );
    }
}

#[test]
fn tech_release_clears_guard_hold_and_catch_aliases_without_changing_attack_or_jump() {
    let native = MAPPED_GUARD | MAPPED_GUARD_HOLD | MAPPED_CATCH | 0x201 | 0x4;
    let released = mapped_guard(
        native,
        TechOutput {
            press: false,
            owns_guard: true,
        },
    );
    assert_eq!(released, 0x201 | 0x4);
    let press = mapped_guard(
        native,
        TechOutput {
            press: true,
            owns_guard: true,
        },
    );
    assert_eq!(press, 0x201 | 0x4 | MAPPED_GUARD);
    assert_eq!(mapped_guard(native, TechOutput::default()), native);
    assert_eq!(
        mapped_automatic_actions(MAPPED_GUARD | MAPPED_GUARD_HOLD, MAPPER_B),
        0
    );
}

#[test]
fn delayed_native_grab_cannot_reach_a_ledge_side_b_or_up_b_after_the_r_pulse() {
    for next in ["up_b:4", "side_b:4"] {
        let cat = Catalog::parse(&format!(
            "[bowser.attack.test]\nstart_delay=0\nactions=input:r:0:0:4,{next}\n"
        ))
        .unwrap();
        let mut tape = Some(InputTimeline::new(cat.entries()[0].sequence, enemy(5., 0.)));
        let mut receipt = GrabHandoff::default();
        for tick in 0..9 {
            let status = if tick >= 6 {
                0
            } else if tick >= 5 {
                0x1d
            } else {
                0x1c
            };
            let out = tape.as_mut().unwrap().sample(
                LedgeSettings::default(),
                c(status, tick as f32),
                Some(enemy(5., tick as f32)),
                None,
                0.,
            );
            let out = GrabHandoff::grab_packet(out);
            if tick < 4 {
                assert!(receipt.permits(Some(Owner::Ledge), out, Some([0.; 2])));
                assert_eq!(out.requested, MAPPER_R | MAPPER_ZR);
            } else if out.requested == 0 {
                assert!(
                    receipt.permits(Some(Owner::Ledge), out, Some([0.; 2])),
                    "neutral receipt waits remain exclusively owned"
                );
            } else {
                assert_ne!(out.requested & MAPPER_B, 0);
                assert!(!receipt.permits(Some(Owner::Ledge), out, Some([0.; 2])));
                tape = None;
                break;
            }
            receipt.mapped(tick, Some(Owner::Ledge), tick < 4, tick < 4, true);
        }
        for tick in 9..33 {
            receipt.mapped(tick, None, false, false, true);
            receipt.observe(true, 0x1c, 0);
        }
        assert!(!receipt.pending());
        assert!(tape.is_none(), "aborted follow-up cannot resume later");
    }
}
#[test]
fn explicit_ledge_shield_rearms_a_guard_quarantined_by_an_earlier_automatic_mode() {
    use kirby_assist_core::manual_controls::AssistGuardHandoff;
    let human = ButtonTrace {
        held: MAPPER_ZR,
        previous: MAPPER_ZR,
        ..Default::default()
    };
    let mut h = AssistGuardHandoff::new();
    assert_eq!(h.sample(true, human, MAPPER_ZR, MAPPER_ZR, 0), MAPPER_ZR);
    assert_eq!(h.sample(false, human, MAPPER_ZR, MAPPER_ZR, MAPPER_ZR), 0);
    assert_eq!(h.sample(false, human, MAPPER_ZR, 0, 0), 0);
}

#[test]
fn grab_handoff_can_initialize_a_const_native_state() {
    struct NativeState {
        handoff: GrabHandoff,
    }
    impl NativeState {
        const fn new() -> Self {
            Self {
                handoff: GrabHandoff::new(),
            }
        }
    }
    const INITIAL: NativeState = NativeState::new();
    assert!(!INITIAL.handoff.pending());
}
