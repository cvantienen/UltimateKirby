use kirby_assist_core::{
    action_owner::{exclusive_output, safety_output, ActionOwnership, Intent, Owner},
    gameplay::{ControllerOutput, GameplayContext},
    grab_handoff::GrabHandoff,
    ledge_trap::SEQUENCE_BUTTONS,
    manual_controls::{ManualOverlay, TurboOutput},
    native_input::{
        self, ButtonTrace, ControlLayout, GAMEPLAY_BUTTONS, MAPPER_A, MAPPER_B, MAPPER_R, MAPPER_ZR,
    },
    tech_input::TechOutput,
};

const SEQUENCES: [Owner; 11] = [
    Owner::Safety,
    Owner::GrabDodge,
    Owner::OptionRead,
    Owner::Trip,
    Owner::JabLock,
    Owner::Fair,
    Owner::Trump,
    Owner::Ledge,
    Owner::RollRead,
    Owner::OutOfShield,
    Owner::Reflect,
];
fn intent(owner: Owner) -> Intent {
    Intent {
        owner,
        present: true,
        committed: true,
    }
}

#[test]
fn every_committed_owner_rejects_every_unrelated_sequence_even_after_reservation_changes() {
    for current in SEQUENCES {
        for contender in SEQUENCES {
            let mut owners = ActionOwnership::new();
            owners.select(true, &[intent(current)]);
            owners.set_ledge_priority(true);
            owners.set_automatic_suspended(true);
            assert!(owners.allows(current));
            let expected = if contender == Owner::Safety {
                contender
            } else {
                current
            };
            assert_eq!(
                owners.select(true, &[intent(current), intent(contender)]),
                Some(expected),
                "{current:?} vs {contender:?}"
            );
            assert_eq!(owners.select(false, &[intent(current)]), None);
            assert_eq!(owners.current_owner(), None);
            assert!(owners.allows(contender));
        }
    }
}

#[test]
fn all_sequence_packets_mask_manual_buttons_axes_aliases_and_triggers_through_neutral_gaps() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for owner in SEQUENCES {
            let mut overlay = ManualOverlay::default();
            let human = ButtonTrace {
                held: GAMEPLAY_BUTTONS,
                previous: GAMEPLAY_BUTTONS,
                pressed: GAMEPLAY_BUTTONS,
                released: GAMEPLAY_BUTTONS,
            };
            for requested in [MAPPER_A, 0, MAPPER_B, 0, MAPPER_R, 0, MAPPER_ZR, 0] {
                let packet = exclusive_output(
                    Some(owner),
                    true,
                    ControllerOutput {
                        requested,
                        owned: requested,
                        ..Default::default()
                    },
                );
                assert_eq!(packet.owned & SEQUENCE_BUTTONS, SEQUENCE_BUTTONS);
                let frame = overlay.apply_controls(
                    human,
                    [-1., 1.],
                    TurboOutput {
                        stick: packet.stick,
                        ..Default::default()
                    },
                    0,
                    layout.physical_buttons(packet.requested),
                    layout.physical_owned_buttons(packet.owned),
                );
                assert_eq!(
                    frame.buttons.held,
                    layout.physical_buttons(requested),
                    "{owner:?} {layout:?}"
                );
                assert_eq!(frame.stick, [0.; 2]);
                assert_eq!(frame.buttons.pressed & !frame.buttons.held, 0);
                assert_eq!(frame.buttons.released & frame.buttons.held, 0);
                assert_eq!(
                    native_input::owned_trigger_axes([1.; 2], frame.clear_autorepeat),
                    [0.; 2]
                );
                assert!(kirby_assist_core::move_input::blocks_physical_c_stick(
                    packet.owned
                ));
            }
            let frame = overlay.apply_controls(human, [-1., 1.], TurboOutput::default(), 0, 0, 0);
            assert_eq!(
                frame.buttons.held, human.held,
                "manual buttons restore on completion"
            );
            assert_eq!(frame.stick, [-1., 1.]);
            assert_eq!(
                native_input::owned_trigger_axes([1.; 2], frame.clear_autorepeat),
                [1.; 2]
            );
        }
    }
}

#[test]
fn manual_policy_view_is_frozen_only_during_commitment_and_refreshes_on_handoff() {
    for owner in SEQUENCES {
        let mut owners = ActionOwnership::new();
        owners.observe_manual(0, [0.; 4]);
        owners.select(true, &[intent(owner)]);
        for button in [MAPPER_A, MAPPER_R, MAPPER_ZR] {
            owners.observe_manual(button, [-1., 1., 1., -1.]);
            assert_eq!(owners.policy_inputs().stick(), [0.; 2]);
            assert_eq!(owners.policy_inputs().action_epoch(), 0);
            assert!(
                owners.physical_inputs().action_epoch() > 0,
                "explicit until-manual steps retain real input"
            );
        }
        owners.release(owner);
        assert_eq!(owners.policy_inputs().stick(), [-1., 1.]);
        owners.select(true, &[intent(owner)]);
        assert_eq!(
            owners.policy_inputs().stick(),
            [-1., 1.],
            "new sequence captures current intent"
        );
        owners.select(true, &[]);
        assert_eq!(owners.current_owner(), None);
    }
}

#[test]
fn native_safety_selects_exactly_one_phase() {
    let tech = TechOutput {
        press: true,
        owns_guard: true,
    };
    let (packet, delivered) = safety_output(tech, Some([-1., 0.]), Some([0., 1.]), Some([1., 0.]));
    assert_eq!(packet.requested, 0);
    assert_eq!(packet.stick, Some([-1., 0.]));
    assert!(
        delivered.is_none(),
        "capture mash cannot receive a stale tech press"
    );
    let (packet, delivered) = safety_output(tech, None, Some([0., 1.]), Some([1., 0.]));
    assert_eq!(packet.requested, 0);
    assert_eq!(packet.stick, Some([0., 1.]));
    assert_eq!(delivered, None, "final DI must suppress tech guard");
    let (packet, _) = safety_output(
        TechOutput {
            press: false,
            owns_guard: true,
        },
        None,
        None,
        None,
    );
    let packet = exclusive_output(Some(Owner::Safety), true, packet);
    assert_eq!(packet.requested, 0);
    assert_eq!(
        packet.stick,
        Some([0.; 2]),
        "tech release never exposes manual drift/jump/attack"
    );
}

#[test]
fn neutral_native_receipts_retain_their_owner_without_allowing_a_late_special() {
    for owner in SEQUENCES.into_iter().filter(|o| *o != Owner::Safety) {
        let mut handoff = GrabHandoff::new();
        handoff.observe_native(
            true,
            Some(GameplayContext {
                gameplay: true,
                motion_kind: 1,
                motion_rate: 1.,
                ..Default::default()
            }),
        );
        handoff.mapped(0, Some(owner), true, true, true);
        assert!(handoff.permits(Some(owner), ControllerOutput::default(), None));
        assert!(!handoff.permits(
            Some(owner),
            ControllerOutput {
                requested: MAPPER_B,
                ..Default::default()
            },
            None
        ));
        let mut owners = ActionOwnership::new();
        owners.select(true, &[Intent::sequence_receipt(owner, 0, true)]);
        for _ in 0..10 {
            let winner = owners.select(
                true,
                &[
                    Intent::sequence_receipt(owner, 0, handoff.pending()),
                ],
            );
            assert_eq!(winner, Some(owner));
            let packet = exclusive_output(winner, true, ControllerOutput::default());
            assert_eq!(packet.stick, Some([0.; 2]));
            assert_eq!(packet.owned, SEQUENCE_BUTTONS);
        }
        handoff.observe_native(false, None);
        assert_eq!(owners.select(true, &[]), None);
    }
}

#[test]
fn inactive_automation_and_passive_helpers_do_not_acquire_gameplay_inputs() {
    for owner in [
        None,
        Some(Owner::Defense),
        Some(Owner::Turbo),
        Some(Owner::Posture),
    ] {
        assert_eq!(
            exclusive_output(owner, true, ControllerOutput::default()),
            ControllerOutput::default()
        );
    }
    assert_eq!(
        exclusive_output(Some(Owner::Ledge), false, ControllerOutput::default()),
        ControllerOutput::default()
    );
}

#[test]
fn a_blocked_new_ledge_trigger_or_manual_attack_cannot_cancel_existing_playback() {
    use kirby_assist_core::{
        fighter_profiles::presets::{MenuPresets, PresetSlot},
        native_input::MAPPER_L,
        settings_menu::MenuSelection,
    };
    let presets = MenuPresets::default();
    let mut menu = MenuSelection::new(&presets);
    menu.input(0, &presets);
    menu.block_binding_starts(true);
    menu.input(MAPPER_L, &presets);
    assert!(
        !menu.cancel_pending,
        "a new assist request waits for the committed owner"
    );
    menu.input(0, &presets);
    menu.show_quick();
    menu.preview_slot = Some(PresetSlot::Neutral);
    menu.input(MAPPER_A, &presets);
    assert_eq!(menu.preview_slot, Some(PresetSlot::Neutral));
    assert!(!menu.cancel_pending);
}

#[test]
fn authored_timeline_survives_manual_churn_and_duplicate_polls_then_restores_manual_control() {
    use kirby_assist_core::{
        fighter_profiles::presets::PresetSlot, gameplay::Opponent, ledge_trap::LedgeTrap,
        sequence_config::Catalog, settings::AssistSettings,
    };
    let catalog = Catalog::parse(
        "[bowser.neutral.exclusive]\nstart_delay=0\nactions=wait:2,input:a:0:0:2,wait:2\n",
    )
    .unwrap();
    let sequence = catalog.entries()[0].sequence;
    let mut settings = AssistSettings {
        enabled: true,
        fighter_kind: kirby_assist_core::fighter_kinds::BOWSER,
        ..Default::default()
    };
    settings.sequence_overrides[PresetSlot::Neutral as usize] = Some(sequence);
    let enemy = Opponent {
        id: 2,
        position: [10., 0.],
        ..Default::default()
    };
    let mut tape = LedgeTrap::default();
    let mut baseline = LedgeTrap::default();
    tape.request_preview(PresetSlot::Neutral);
    baseline.request_preview(PresetSlot::Neutral);
    let mut owners = ActionOwnership::new();
    let mut overlay = ManualOverlay::default();
    owners.observe_manual(0, [0.; 4]);
    let mut finished = false;
    for frame in 0..12 {
        let physical = ButtonTrace {
            held: [MAPPER_B, MAPPER_R, MAPPER_ZR][frame % 3],
            ..Default::default()
        };
        owners.observe_manual(physical.held, [1., -1., -1., 1.]);
        let context = GameplayContext {
            held: true,
            gameplay: true,
            can_act: true,
            motion_kind: 1,
            motion_frame: frame as f32,
            motion_rate: 1.,
            facing: 1.,
            manual_input_epoch: owners.physical_inputs().epoch(),
            ..Default::default()
        };
        let out = tape.sample(settings, frame as i64, context, Some(enemy));
        assert_eq!(
            out,
            baseline.sample(
                settings,
                frame as i64,
                GameplayContext {
                    manual_input_epoch: 0,
                    ..context
                },
                Some(enemy),
            )
        );
        for _ in 0..4 {
            // The live observer samples once per input update; duplicate
            // mapper queries deliver its cached packet through the same owner.

            let winner = owners.select(true, &[Intent::input_sequence(Owner::Ledge, out.owned)]);
            let packet = exclusive_output(winner, owners.current_owner().is_some(), out);
            let mapped = overlay.apply_controls(
                physical,
                [1., -1.],
                TurboOutput {
                    stick: packet.stick,
                    ..Default::default()
                },
                0,
                packet.requested,
                packet.owned,
            );
            if out.owned != 0 {
                assert_eq!(winner, Some(Owner::Ledge));
                assert_eq!(mapped.buttons.held, out.requested);
                assert_eq!(mapped.stick, [0.; 2]);
            } else {
                assert_eq!(winner, None);
                assert_eq!(mapped.buttons.held, physical.held);
                assert_eq!(mapped.stick, [1., -1.]);
                finished = true;
            }
        }
    }
    assert!(finished);
}

#[test]
fn exclusive_sdi_hands_the_stick_to_di_without_a_manual_gap_then_releases_on_hitstop_exit() {
    use kirby_assist_core::{
        auto_di::{AutoDi, Context as DiContext, Settings},
        combat_assist::{SdiContext, SdiState},
    };
    let mut sdi = SdiState::default();
    let mut di = AutoDi::default();
    let mut owners = ActionOwnership::new();
    let mut overlay = ManualOverlay::default();
    for remaining in (1..=8).rev() {
        let s = sdi.sample_exclusive(
            true,
            SdiContext {
                object: 1,
                damage_stop: true,
                remaining,
                input_delay: 0.,
                outward: 1.,
            },
        );
        let d = di.sample_exclusive(
            Settings::default(),
            DiContext {
                object: 1,
                damage_stop: true,
                remaining,
                launch: [4., 1.],
                outward: 1.,
            },
        );
        assert_eq!(s.is_some(), remaining > 3);
        assert_eq!(d.is_some(), remaining <= 3);
        let stick = d.or(s).unwrap();
        let winner = owners.select(true, &[intent(Owner::Safety)]);
        let (packet, _) = safety_output(TechOutput::default(), None, Some(stick), None);
        let packet = exclusive_output(winner, true, packet);
        let mapped = overlay.apply_controls(
            ButtonTrace {
                held: GAMEPLAY_BUTTONS,
                ..Default::default()
            },
            [-1., 1.],
            TurboOutput {
                stick: packet.stick,
                ..Default::default()
            },
            0,
            packet.requested,
            packet.owned,
        );
        assert_eq!(mapped.stick, stick);
        assert_eq!(mapped.buttons.held & GAMEPLAY_BUTTONS, 0);
    }
    assert!(di
        .sample_exclusive(
            Settings::default(),
            DiContext {
                object: 1,
                damage_stop: false,
                remaining: 0,
                launch: [4., 1.],
                outward: 1.
            }
        )
        .is_none());
    assert_eq!(owners.select(true, &[]), None);
    let mapped = overlay.apply_controls(
        ButtonTrace {
            held: MAPPER_A,
            ..Default::default()
        },
        [-1., 1.],
        TurboOutput::default(),
        0,
        0,
        0,
    );
    assert_eq!(mapped.stick, [-1., 1.]);
    assert_eq!(mapped.buttons.held, MAPPER_A);
}

#[test]
fn exclusive_damage_control_does_not_inherit_a_previous_manual_veto() {
    use kirby_assist_core::{
        auto_di::{AutoDi, Context, Settings},
        combat_assist::{SdiContext, SdiState},
    };
    let c = Context {
        object: 1,
        damage_stop: true,
        remaining: 3,
        launch: [4., 1.],
        outward: 1.,
    };
    let mut di = AutoDi::default();
    assert!(di.sample(Settings::default(), c, [1., 0.]).is_none());
    assert!(di.manual_override());
    assert!(di.sample_exclusive(Settings::default(), c).is_some());
    assert!(!di.manual_override());
    let c = SdiContext {
        object: 1,
        damage_stop: true,
        remaining: 8,
        input_delay: 0.,
        outward: 1.,
    };
    let mut sdi = SdiState::default();
    assert!(sdi.sample(true, c, [1., 0.]).is_none());
    assert!(sdi.sample_exclusive(true, c).is_some());
}
