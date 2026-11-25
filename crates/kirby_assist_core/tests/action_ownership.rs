use kirby_assist_core::{
    action_owner::{ActionOwnership, Intent, Owner},
    kirby::{TripContext, TripFollowup, TripTarget},
    manual_controls::{ManualOverlay, TurboOutput},
    native_input::{ButtonTrace, MAPPER_A},
};
fn intent(owner: Owner, committed: bool) -> Intent {
    Intent {
        owner,
        present: true,
        committed,
    }
}

#[test]
fn held_spacing_reserves_modes_even_without_an_attack_packet() {
    let mut control = ActionOwnership::new();
    control.select(true, &[intent(Owner::Counter, true)]);
    control.release(Owner::Counter);
    control.set_automatic_suspended(true);
    let spacing = intent(Owner::Spacing, true);
    assert_eq!(control.select(true, &[spacing, intent(Owner::Counter, true)]), Some(Owner::Spacing));
    for owner in [Owner::Counter, Owner::Reflect, Owner::RollRead, Owner::Ledge,
        Owner::Turbo, Owner::Posture, Owner::Trump] {
        assert!(!control.allows(owner));
    }
    assert!(control.allows(Owner::Safety));
    control.release(Owner::Spacing);
    control.set_automatic_suspended(false);
    assert!(!control.allows(Owner::Counter));
    control.set_defensive_hold(false, true);
    assert!(control.allows(Owner::Counter));
}

#[test]
fn counter_requires_hold_then_preempts_sequences_and_keeps_release_reservation() {
    for owner in [
        Owner::Ledge,
        Owner::RollRead,
        Owner::OutOfShield,
        Owner::Reflect,
    ] {
        let mut control = ActionOwnership::new();
        control.select(true, &[intent(owner, true)]);
        assert!(!control.allows(Owner::Counter));
        assert_eq!(
            control.select(true, &[intent(owner, true), intent(Owner::Counter, true)]),
            Some(owner)
        );
        assert_eq!(control.set_defensive_hold(false, true), Some(owner));
        assert_eq!(control.select(true, &[intent(owner, true), intent(Owner::Counter, true)]),
            Some(Owner::Counter));
    }
    let mut control = ActionOwnership::new();
    control.set_defensive_hold(false, true);
    control.select(true, &[intent(Owner::Counter, true)]);
    assert_eq!(
        control.select(
            true,
            &[intent(Owner::Counter, true), intent(Owner::RollRead, true)]
        ),
        Some(Owner::Counter)
    );
    control.set_defensive_hold(false, false);
    assert_eq!(
        control.select(true, &[intent(Owner::RollRead, true)]),
        Some(Owner::RollRead)
    );
}
#[test]
fn acquired_ledge_visit_reserves_new_work_without_replacing_committed_reads() {
    for read in [Owner::OptionRead, Owner::RollRead] {
        let mut control = ActionOwnership::new();
        control.select(true, &[intent(read, true)]);
        control.set_ledge_priority(true);
        assert!(control.allows(read));
        assert!(!control.allows(Owner::Ledge));
        assert_eq!(
            control.select(true, &[intent(read, true), intent(Owner::Ledge, true)]),
            Some(read)
        );
        assert_eq!(
            control.select(true, &[intent(Owner::Ledge, true)]),
            Some(Owner::Ledge)
        );
        for _ in 0..8 {
            assert_eq!(
                control.select(true, &[intent(read, true), intent(Owner::Ledge, true)]),
                Some(Owner::Ledge)
            );
        }
        assert_eq!(control.select(false, &[intent(Owner::Ledge, true)]), None);
        assert!(!control.ledge_priority());
        assert!(control.allows(read));
    }
}

#[test]
fn committed_actions_keep_control_through_release_frames_and_passive_competitors() {
    for owner in [
        Owner::Ledge,
        Owner::RollRead,
        Owner::Fair,
        Owner::Fair,
        Owner::Trip,
    ] {
        let mut control = ActionOwnership::new();
        assert_eq!(control.select(true, &[intent(owner, true)]), Some(owner));
        for _ in 0..20 {
            assert_eq!(
                control.select(
                    true,
                    &[
                        intent(owner, true),
                        intent(Owner::Defense, false),
                        intent(Owner::Turbo, false)
                    ]
                ),
                Some(owner)
            );
        }
        assert_eq!(
            control.select(true, &[intent(Owner::Defense, false)]),
            Some(Owner::Defense),
            "completion hands back in the same sample"
        );
    }
}
#[test]
fn native_safety_preempts_committed_reactions_and_scope_still_cancels() {
    let mut control = ActionOwnership::new();
    control.select(true, &[intent(Owner::RollRead, true)]);
    assert!(!control.allows(Owner::Ledge));
    assert!(!control.allows(Owner::Defense));
    assert_eq!(
        control.select(
            true,
            &[intent(Owner::RollRead, true), intent(Owner::Trip, true)]
        ),
        Some(Owner::RollRead)
    );
    assert_eq!(
        control.select(
            true,
            &[intent(Owner::Trip, true), intent(Owner::Fair, true)]
        ),
        Some(Owner::Trip)
    );
    assert_eq!(
        control.select(
            true,
            &[intent(Owner::Trip, true), intent(Owner::Safety, false)]
        ),
        Some(Owner::Safety),
        "native safety must preempt an active auto reaction"
    );
    assert_eq!(control.select(false, &[intent(Owner::Trip, true)]), None);
    assert!(control.allows(Owner::Ledge));
}
#[test]
fn safety_preemption_respects_presence_and_releases_the_old_reservation() {
    for committed in [false, true] {
        let mut control = ActionOwnership::new();
        let tape = intent(Owner::RollRead, true);
        control.select(true, &[tape]);
        control.set_ledge_priority(true);
        control.set_automatic_suspended(true);
        assert!(control.allows(Owner::Safety));
        assert_eq!(
            control.select(
                true,
                &[
                    tape,
                    Intent {
                        owner: Owner::Safety,
                        present: false,
                        committed,
                    }
                ]
            ),
            Some(Owner::RollRead)
        );
        assert_eq!(
            control.select(true, &[tape, intent(Owner::Safety, committed)]),
            Some(Owner::Safety)
        );
        assert_eq!(control.current_owner(), committed.then_some(Owner::Safety));
        assert_eq!(control.select(true, &[tape]), None);
        assert_eq!(
            control.select(false, &[intent(Owner::Safety, committed)]),
            None
        );
    }
}

#[test]
fn explicit_manual_tools_yield_to_roll_reads_and_confirmed_followups() {
    for manual in [Owner::Trump] {
        let mut control = ActionOwnership::new();
        control.select(true, &[intent(Owner::RollRead, true)]);
        assert_eq!(
            control.select(true, &[intent(Owner::RollRead, true), intent(manual, true)]),
            Some(Owner::RollRead)
        );
        let mut control = ActionOwnership::new();
        control.select(true, &[intent(Owner::Trip, true)]);
        assert_eq!(
            control.select(true, &[intent(Owner::Trip, true), intent(manual, true)]),
            Some(Owner::Trip)
        );
    }
}
#[test]
fn trip_fsmash_stick_and_a_remain_paired_when_no_timed_punish_is_present() {
    for facing in [-1.0, 1.0] {
        let mut trip = TripFollowup::default();
        let mut control = ActionOwnership::new();
        let mut overlay = ManualOverlay::default();
        let mut targets = [None; 8];
        targets[0] = Some(TripTarget {
            id: 2,
            status: kirby_assist_core::native_states::status::SLIP_WAIT,
            previous_status: kirby_assist_core::native_states::status::SLIP,
        relative_position: [facing * 8.0, 0.0],
        grounded: true,
        hit_confirmed: true,
        });
        let base = TripContext {
            object: 1,
            kirby: true,
            status: kirby_assist_core::native_states::status::ATTACK_LW3,
            motion: 1,
            frame: 10.0,
            grounded: true,
            facing,
            stopped: false,
            targets,
        };
        let mut before = base;
        before.frame = 9.0;
        before.targets[0].as_mut().unwrap().status = kirby_assist_core::native_states::status::WAIT;
        before.targets[0].as_mut().unwrap().hit_confirmed = false;
        trip.sample(true, before);
        for step in 0..6 {
            let out = trip.sample(
                true,
                TripContext {
                    frame: 10.0 + step as f32,
                    ..base
                },
            );
            assert_eq!(
                control.select(
                    true,
                    &[intent(Owner::Trip, true), intent(Owner::Defense, false)]
                ),
                Some(Owner::Trip)
            );
            let raw = overlay.apply_controls(
                ButtonTrace::default(),
                [0.0, -1.0],
                TurboOutput { stick: out.stick },
                0,
                out.requested,
                out.owned,
            );
            assert_eq!(
                raw.buttons.held & MAPPER_A,
                if step % 2 == 1 { MAPPER_A } else { 0 }
            );
            if step % 2 == 1 {
                assert_eq!(raw.stick, [facing, 0.0]);
            }
        }
    }
}

#[test]
fn priority_chooses_new_work_but_never_interrupts_a_committed_sequence() {
    for punish in [Owner::Ledge, Owner::RollRead] {
        for other in [
            Owner::Trip,
            Owner::Fair,
            Owner::Fair,
            Owner::Trump,
            Owner::Defense,
            Owner::Turbo,
            Owner::Posture,
        ] {
            for committed in [false, true] {
                let mut control = ActionOwnership::new();
                control.select(true, &[intent(other, committed)]);
                let expected = if committed { other } else { punish };
                assert_eq!(
                    control.select(true, &[intent(other, committed), intent(punish, true)]),
                    Some(expected)
                );
            }
        }
    }
}

#[test]
fn priority_reads_cannot_interrupt_committed_ledge_or_other_tapes() {
    for (current, next, interrupt) in [
        (Owner::Ledge, Owner::RollRead, false),
        (Owner::Ledge, Owner::OptionRead, false),
        (Owner::RollRead, Owner::OptionRead, false),
        (Owner::OptionRead, Owner::RollRead, false),
        (Owner::RollRead, Owner::Ledge, false),
    ] {
        let mut control = ActionOwnership::new();
        control.select(true, &[intent(current, true)]);
        assert_eq!(control.allows(next), interrupt);
        assert_eq!(
            control.select(true, &[intent(current, true), intent(next, true)]),
            Some(if interrupt { next } else { current })
        );
        assert_eq!(
            control.select(true, &[intent(current, true), intent(Owner::Safety, false)]),
            Some(Owner::Safety),
            "native safety must preempt committed controller sequences"
        );
    }
}
#[test]
fn posture_is_last_even_while_other_modes_are_only_preparing() {
    for other in [
        Owner::Safety,
        Owner::OptionRead,
        Owner::Trip,
        Owner::Fair,
        Owner::Fair,
        Owner::Trump,
        Owner::Ledge,
        Owner::RollRead,
        Owner::Defense,
        Owner::Turbo,
    ] {
        let mut control = ActionOwnership::new();
        assert_eq!(
            control.select(true, &[intent(Owner::Posture, false)]),
            Some(Owner::Posture)
        );
        assert!(
            control.allows(other),
            "crouching must not block proposal generation for {other:?}"
        );
        assert_eq!(
            control.select(true, &[intent(Owner::Posture, false), intent(other, false)]),
            Some(other)
        );
    }
}

#[test]
fn ledge_reservation_blocks_new_configured_actions_until_released() {
    for owner in [
        Owner::OptionRead,
        Owner::RollRead,
        Owner::OutOfShield,
        Owner::Trip,
        Owner::Fair,
        Owner::Fair,
        Owner::Trump,
    ] {
        let mut control = ActionOwnership::new();
        control.set_ledge_priority(true);
        assert!(!control.allows(owner));
        assert!(control.allows(Owner::Ledge));
        assert_eq!(
            control.select(true, &[intent(owner, true), intent(Owner::Ledge, true)]),
            Some(Owner::Ledge)
        );
        control.set_ledge_priority(false);
        assert!(
            !control.allows(owner),
            "reservation loss cannot end a still-running tape"
        );
        control.release(Owner::Ledge);
        assert!(control.allows(owner));
        assert_eq!(control.select(true, &[intent(owner, true)]), Some(owner));
    }
}

#[test]
fn incoming_grab_defense_waits_for_committed_work_and_yields_to_native_safety() {
    for current in [Owner::Trip] {
        let mut control = ActionOwnership::new();
        control.select(true, &[intent(current, true)]);
        assert!(!control.allows(Owner::GrabDodge));
        control.release(Owner::RollRead);
        assert_eq!(control.current_owner(), Some(current));
        control.release(current);
        assert_eq!(
            control.select(true, &[intent(Owner::GrabDodge, true)]),
            Some(Owner::GrabDodge)
        );
        assert!(!control.allows(Owner::OptionRead));
        assert_eq!(
            control.select(
                true,
                &[intent(Owner::GrabDodge, true), intent(Owner::Safety, true)]
            ),
            Some(Owner::Safety)
        );
    }
}

#[test]
fn committed_automatic_tapes_reject_competing_grab_and_punish_requests() {
    for current in [
        Owner::RollRead,
        Owner::OptionRead,
        Owner::OutOfShield,
        Owner::Ledge,
        Owner::Reflect,
        Owner::Counter,
        Owner::Trip,
        Owner::Fair,
    ] {
        let mut control = ActionOwnership::new();
        control.set_defensive_hold(false, current == Owner::Counter);
        control.select(true, &[intent(current, true)]);
        for other in [
            Owner::GrabDodge,
            Owner::RollRead,
            Owner::OptionRead,
            Owner::OutOfShield,
        ] {
            if other == current {
                continue;
            }
            assert!(!control.allows(other), "{other:?} interrupted {current:?}");
            assert_eq!(
                control.select(true, &[intent(current, true), intent(other, true)]),
                Some(current)
            );
        }
        assert_eq!(
            control.select(true, &[intent(current, true), intent(Owner::Safety, false)]),
            Some(Owner::Safety),
            "native safety must preempt committed controller sequences"
        );
    }
    let mut control = ActionOwnership::new();
    control.select(true, &[intent(Owner::OutOfShield, true)]);
    control.set_ledge_priority(true);
    assert_eq!(
        control.select(
            true,
            &[intent(Owner::OutOfShield, true), intent(Owner::Ledge, true)]
        ),
        Some(Owner::OutOfShield)
    );
}
