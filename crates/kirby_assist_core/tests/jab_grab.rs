use kirby_assist_core::{
    jab_grab::{JabGrab, TRIGGER},
    manual_controls::{ManualOverlay, ShortcutContext, TurboOutput},
    native_input::{ButtonTrace, MAPPER_A, MAPPER_B, MAPPER_R},
    timing::motion_hash,
};
fn context(status: i32, frame: f32) -> ShortcutContext {
    ShortcutContext {
        status,
        situation: 0,
        motion: motion_hash(if status == 0x27 { "attack_11" } else { "wait" }),
        frame,
        rate: 1.0,
        stopped: false,
        facing: 1.0,
        can_short_hop: false,
        input_delay: 0.0,
        can_neutral_special: false,
    }
}
fn arm(j: &mut JabGrab) -> kirby_assist_core::ledge_trap::LedgeOutput {
    j.sample(true, 0, [0.0; 2], Some(context(0, 0.0)), true);
    j.sample(true, TRIGGER, [0.0; 2], Some(context(0, 1.0)), true)
}
#[test]
fn a_alone_taps_holds_and_rapid_jabs_are_entirely_physical() {
    let mut j = JabGrab::default();
    for (frame, b) in [
        (0, 0),
        (1, MAPPER_A),
        (2, MAPPER_A),
        (3, 0),
        (4, MAPPER_A),
        (5, 0),
    ] {
        assert_eq!(
            j.sample(true, b, [0.0; 2], Some(context(0, frame as f32)), true)
                .owned,
            0
        );
    }
    for frame in 0..40 {
        assert_eq!(
            j.sample(
                true,
                MAPPER_A,
                [0.0; 2],
                Some(context(0x27, frame as f32)),
                true
            )
            .owned,
            0
        );
    }
    assert!(!j.active());
    j.cancel_pending();
    assert_eq!(
        j.sample(true, MAPPER_A, [0.0; 2], Some(context(0, 1.0)), true)
            .owned,
        0
    );
    j.sample(false, MAPPER_A, [0.0; 2], Some(context(0, 2.0)), true);
    assert_eq!(
        j.sample(true, MAPPER_A, [0.0; 2], Some(context(0, 3.0)), true)
            .owned,
        0
    );
}
#[test]
fn chord_tap_commits_single_jab_then_waits_for_recovery_and_native_grab() {
    let mut j = JabGrab::default();
    assert_eq!(arm(&mut j).requested, MAPPER_A);
    // Releasing both buttons must not cancel an explicit chord attempt.
    for frame in 0..18 {
        let out = j.sample(true, 0, [0.0; 2], Some(context(0x27, frame as f32)), true);
        assert_eq!(
            out.requested, 0,
            "never feed Grab/Attack into ongoing jab, even if catch term says true"
        );
    }
    assert!(j.active());
    assert_eq!(
        j.sample(true, 0, [0.0; 2], Some(context(0, 0.0)), false)
            .requested,
        0
    );
    assert_eq!(
        j.sample(true, 0, [0.0; 2], Some(context(1, 0.0)), true)
            .requested,
        MAPPER_R
    );
    assert_eq!(
        j.sample(true, 0, [0.0; 2], Some(context(0x37, 0.0)), false)
            .owned,
        0
    );
    assert!(!j.active());
}
#[test]
fn actual_overlay_consumes_b_so_a_plus_b_smash_never_receives_the_chord() {
    let mut j = JabGrab::default();
    let mut overlay = ManualOverlay::default();
    let out = arm(&mut j);
    let mapped = overlay.apply_controls(
        ButtonTrace {
            held: TRIGGER,
            pressed: TRIGGER,
            ..Default::default()
        },
        [0.0; 2],
        TurboOutput {
            consume_dpad: false,
            stick: out.stick,
        },
        0,
        out.requested,
        out.owned,
    );
    assert_eq!(mapped.buttons.held & TRIGGER, MAPPER_A);
    assert_eq!(mapped.buttons.pressed & MAPPER_B, 0);
    let out = j.sample(
        true,
        TRIGGER,
        [0.0; 2],
        Some(ShortcutContext {
            stopped: true,
            ..context(0x27, 0.0)
        }),
        true,
    );
    let mapped = overlay.apply_controls(
        ButtonTrace {
            held: TRIGGER,
            previous: TRIGGER,
            ..Default::default()
        },
        [0.0; 2],
        TurboOutput {
            consume_dpad: false,
            stick: out.stick,
        },
        0,
        out.requested,
        out.owned,
    );
    assert_eq!(mapped.buttons.held & TRIGGER, 0);
    assert_ne!(mapped.buttons.released & MAPPER_A, 0);
}
#[test]
fn crouch_and_dash_are_neutralized_before_jab_so_it_is_not_a_tilt_or_dash_attack() {
    for status in [0x12, 3] {
        let mut j = JabGrab::default();
        j.sample(true, 0, [0.0; 2], Some(context(status, 0.0)), true);
        let prepare = j.sample(true, TRIGGER, [0.0; 2], Some(context(status, 1.0)), true);
        assert_eq!(prepare.requested, 0);
        assert_eq!(prepare.stick, Some([0.0; 2]));
        assert_eq!(
            j.sample(true, 0, [0.0; 2], Some(context(0, 0.0)), true)
                .requested,
            MAPPER_A
        );
    }
}
#[test]
fn delayed_startup_duplicate_polls_and_hitstop_preserve_sequence() {
    let mut j = JabGrab::default();
    arm(&mut j);
    let c = context(0, 1.0);
    for _ in 0..100 {
        j.sample(true, 0, [0.0; 2], Some(c), true);
        assert!(j.active());
    }
    for frame in 2..14 {
        j.sample(true, 0, [0.0; 2], Some(context(0, frame as f32)), true);
        assert!(j.active());
    }
    let c = ShortcutContext {
        stopped: true,
        ..context(0x27, 0.0)
    };
    for _ in 0..100 {
        assert_eq!(j.sample(true, 0, [0.0; 2], Some(c), true).requested, 0);
    }
    let walk = j.sample(true, 0, [0.0; 2], Some(context(0, 0.0)), false);
    assert_eq!(walk.requested, 0);
    assert_eq!(walk.stick, Some([0.3, 0.0]));
    assert_eq!(
        j.sample(true, 0, [0.0; 2], Some(context(1, 0.0)), false)
            .requested,
        MAPPER_R
    );
}

#[test]
fn capture_and_cancel_return_movement_without_pummel_or_special_from_original_chord() {
    let mut j = JabGrab::default();
    arm(&mut j);
    j.sample(true, TRIGGER, [0.0; 2], Some(context(0x27, 0.0)), false);
    j.sample(true, TRIGGER, [0.0; 2], Some(context(0, 0.0)), true);
    let caught = j.sample(true, TRIGGER, [0.0; 2], Some(context(0x37, 0.0)), false);
    assert_eq!(caught.owned, TRIGGER);
    assert!(caught.stick.is_none());
    assert_eq!(caught.requested, 0);
    let moving = j.sample(true, TRIGGER, [1.0, 0.0], Some(context(0x3c, 1.0)), false);
    assert!(moving.stick.is_none());
    assert_eq!(moving.owned, TRIGGER);
    assert_eq!(
        j.sample(true, 0, [0.0; 2], Some(context(0x3c, 2.0)), false)
            .owned,
        0
    );
    assert_eq!(
        j.sample(true, MAPPER_A, [0.0; 2], Some(context(0x3c, 3.0)), false)
            .owned,
        0
    );
    let mut j = JabGrab::default();
    arm(&mut j);
    j.sample(true, 0, [0.0, -1.0], Some(context(0, 2.0)), true);
    assert!(!j.active());
    assert_eq!(
        j.sample(true, 0, [0.0; 2], Some(context(0, 3.0)), true)
            .owned,
        0
    );
}

#[test]
fn native_recovery_starts_one_forward_input_then_grab_without_catch_gate() {
    use kirby_assist_core::jab_grab::Readiness;
    let mut j = JabGrab::default();
    arm(&mut j);
    for frame in 0..14 {
        let out = j.sample_with_readiness(
            true,
            0,
            [0.0; 2],
            Some(context(0x27, frame as f32)),
            Readiness {
                can_grab: true,
                can_walk: true,
                cancel_jab: false,
            },
        );
        assert_eq!(out.requested, 0, "catch term alone cannot cancel jab");
        assert_eq!(out.stick, Some([0.0; 2]));
    }
    let ready = Readiness {
        can_grab: true,
        can_walk: true,
        cancel_jab: true,
    };
    let out = j.sample_with_readiness(true, 0, [0.0; 2], Some(context(0x27, 14.0)), ready);
    assert_eq!(out.requested, 0);
    assert_eq!(out.stick, Some([0.3, 0.0]));
    for _ in 0..100 {
        assert_eq!(
            j.sample_with_readiness(true, 0, [0.0; 2], Some(context(0x27, 14.0)), ready),
            out
        );
    }
    let grab = j.sample_with_readiness(
        true,
        0,
        [0.0; 2],
        Some(context(1, 0.0)),
        Readiness::default(),
    );
    assert_eq!(grab.requested, MAPPER_R);
    assert_eq!(
        j.sample_with_readiness(
            true,
            0,
            [0.0; 2],
            Some(context(0x37, 0.0)),
            Readiness::default()
        )
        .owned,
        0
    );
}

#[test]
fn forward_input_lasts_one_native_frame_and_grab_brakes_immediately() {
    use kirby_assist_core::jab_grab::Readiness;
    for facing in [-1.0, 1.0] {
        let mut j = JabGrab::default();
        arm(&mut j);
        let mut c = context(0x27, 14.0);
        c.facing = facing;
        let ready = Readiness {
            can_grab: false,
            can_walk: true,
            cancel_jab: true,
        };
        let stopped = j.sample_with_readiness(
            true,
            0,
            [0.0; 2],
            Some(ShortcutContext { stopped: true, ..c }),
            ready,
        );
        assert_eq!(stopped.requested, 0);
        assert_eq!(stopped.stick, Some([0.0; 2]));
        let walk = j.sample_with_readiness(true, 0, [0.0; 2], Some(c), ready);
        assert_eq!(walk.requested, 0);
        assert_eq!(walk.stick, Some([facing * 0.3, 0.0]));
        for _ in 0..100 {
            assert_eq!(
                j.sample_with_readiness(true, 0, [0.0; 2], Some(c), ready),
                walk
            );
        }
        let brake = j.sample_with_readiness(
            true,
            0,
            [0.0; 2],
            Some(ShortcutContext { frame: 15.0, ..c }),
            ready,
        );
        assert_eq!(
            brake.stick,
            Some([0.0; 2]),
            "the forward input lasts only one native frame"
        );
        let grab = j.sample_with_readiness(
            true,
            0,
            [0.0; 2],
            Some(ShortcutContext {
                facing,
                ..context(1, 0.0)
            }),
            Readiness {
                can_grab: true,
                ..Readiness::default()
            },
        );
        assert_eq!(grab.requested, MAPPER_R);
        assert_eq!(grab.stick, Some([0.0; 2]));
    }
}

#[test]
fn grab_retries_survive_long_delay_and_stop_on_native_startup() {
    use kirby_assist_core::jab_grab::Readiness;
    let mut j = JabGrab::default();
    arm(&mut j);
    j.sample(true, 0, [0.0; 2], Some(context(0x27, 0.0)), false);
    let walk = j.sample(true, 0, [0.0; 2], Some(context(0, 0.0)), false);
    assert_eq!(walk.stick, Some([0.3, 0.0]));
    for frame in 0..120 {
        let c = context(1, frame as f32);
        let out = j.sample_with_readiness(true, 0, [0.0; 2], Some(c), Readiness::default());
        assert_eq!(out.requested, if frame % 2 == 0 { MAPPER_R } else { 0 });
        assert_eq!(out.stick, Some([0.0; 2]));
        assert!(j.active());
        for stopped in [false, true] {
            assert_eq!(
                j.sample_with_readiness(
                    true,
                    0,
                    [0.0; 2],
                    Some(ShortcutContext { stopped, ..c }),
                    Readiness::default()
                ),
                out
            );
        }
    }
    let accepted = j.sample(true, 0, [0.0; 2], Some(context(0x37, 0.0)), false);
    assert_eq!(accepted.requested, 0);
    assert!(!j.active());
}

#[test]
fn down_smash_choice_retries_fresh_stick_and_attack_edges_until_native_startup() {
    use kirby_assist_core::jab_grab::{Followup, Readiness};
    for startup in 0x30..=0x32 {
        let mut j = JabGrab::default();
        j.sample_with_followup(
            true,
            0,
            [0.0; 2],
            Some(context(0, 0.0)),
            Readiness::default(),
            Followup::DownSmash,
        );
        j.sample_with_followup(
            true,
            TRIGGER,
            [0.0; 2],
            Some(context(0, 1.0)),
            Readiness::default(),
            Followup::DownSmash,
        );
        j.sample(true, 0, [0.0; 2], Some(context(0x27, 0.0)), false);
        assert_eq!(
            j.sample(true, 0, [0.0; 2], Some(context(0, 0.0)), false)
                .stick,
            Some([0.3, 0.0])
        );
        for frame in 0..100 {
            // Changing the menu mid-attempt must not switch the latched move.
            let out = j.sample_with_followup(
                true,
                0,
                [0.0; 2],
                Some(context(1, frame as f32)),
                Readiness::default(),
                Followup::Grab,
            );
            assert_eq!(out.requested, if frame % 2 == 0 { MAPPER_A } else { 0 });
            assert_eq!(
                out.stick,
                Some(if frame % 2 == 0 {
                    [0.0, -1.0]
                } else {
                    [0.0; 2]
                })
            );
            assert_eq!(out.requested & (MAPPER_R | MAPPER_B), 0);
        }
        let out = j.sample(true, 0, [0.0; 2], Some(context(startup, 0.0)), false);
        assert_eq!(out.requested, 0);
        assert!(!j.active());
    }
}

#[test]
fn opening_jab_retries_rejected_inputs_then_releases_on_native_acceptance() {
    let mut j = JabGrab::default();
    arm(&mut j);
    assert_eq!(
        j.sample(true, 0, [0.0; 2], Some(context(0, 4.0)), true)
            .requested,
        0
    );
    assert_eq!(
        j.sample(true, 0, [0.0; 2], Some(context(0, 5.0)), true)
            .requested,
        MAPPER_A
    );
    let accepted = j.sample(true, 0, [0.0; 2], Some(context(0x27, 0.0)), false);
    assert_eq!(accepted.requested, 0);
    assert_eq!(
        j.sample(true, 0, [0.0; 2], Some(context(0x27, 4.0)), false)
            .requested,
        0
    );
}
