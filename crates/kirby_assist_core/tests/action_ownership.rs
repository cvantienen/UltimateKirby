use kirby_assist_core::{
    action_owner::{ActionOwnership, Intent, Owner},
    combat_assist::{TripContext, TripFollowup, TripTarget},
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
fn committed_actions_keep_control_through_release_frames_and_passive_competitors() {
    for owner in [
        Owner::Ledge,
        Owner::RollRead,
        Owner::JabLock,
        Owner::Finisher,
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
fn safety_interrupts_roll_but_trip_waits_and_scope_cancels() {
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
            &[intent(Owner::Trip, true), intent(Owner::Finisher, true)]
        ),
        Some(Owner::Trip)
    );
    assert_eq!(
        control.select(
            true,
            &[intent(Owner::Trip, true), intent(Owner::Safety, false)]
        ),
        Some(Owner::Safety)
    );
    assert_eq!(control.select(false, &[intent(Owner::Trip, true)]), None);
    assert!(control.allows(Owner::Ledge));
}
#[test]
fn explicit_manual_tools_yield_to_roll_reads_and_confirmed_followups() {
    for manual in [Owner::Finisher, Owner::Trump, Owner::Slide, Owner::Shortcut] {
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
            status: 0x88,
            previous_status: 0x86,
        });
        let base = TripContext {
            object: 1,
            kirby: true,
            status: 0x2c,
            motion: 1,
            frame: 10.0,
            grounded: true,
            facing,
            stopped: false,
            targets,
        };
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
                TurboOutput {
                    consume_dpad: false,
                    stick: out.stick,
                },
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
fn timed_punishes_preempt_every_lower_mode_and_protect_their_release_frames() {
    let lower = [
        Owner::Trip,
        Owner::JabLock,
        Owner::Finisher,
        Owner::Fair,
        Owner::Trump,
        Owner::Slide,
        Owner::Shortcut,
        Owner::Defense,
        Owner::Turbo,
        Owner::Posture,
    ];
    for punish in [Owner::Ledge, Owner::RollRead] {
        for other in lower {
            for committed in [false, true] {
                let mut control = ActionOwnership::new();
                control.select(true, &[intent(other, committed)]);
                assert!(control.allows(punish));
                assert_eq!(
                    control.select(true, &[intent(other, committed), intent(punish, true)]),
                    Some(punish)
                );
                assert!(!control.allows(other));
                for _ in 0..8 {
                    assert_eq!(
                        control.select(true, &[intent(other, committed), intent(punish, true)]),
                        Some(punish)
                    );
                }
                assert_eq!(
                    control.select(true, &[intent(other, committed)]),
                    Some(other)
                );
            }
        }
    }
}

#[test]
fn priority_reads_preempt_ledge_but_do_not_interrupt_each_others_committed_inputs() {
    for (current, next, interrupt) in [
        (Owner::Ledge, Owner::RollRead, true),
        (Owner::Ledge, Owner::OptionRead, true),
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
            Some(Owner::Safety)
        );
    }
}

#[test]
fn posture_is_last_even_while_other_modes_are_only_preparing() {
    for other in [
        Owner::Safety,
        Owner::OptionRead,
        Owner::Trip,
        Owner::JabLock,
        Owner::Finisher,
        Owner::Fair,
        Owner::Trump,
        Owner::Slide,
        Owner::Shortcut,
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
