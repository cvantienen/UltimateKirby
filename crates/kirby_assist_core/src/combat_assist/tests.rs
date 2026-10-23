//! Existing regression cases for `combat_assist`. Kept outside the runtime code.
use super::*;
fn ctx(status: i32, target_status: i32) -> TripContext {
    TripContext {
        object: 1,
        kirby: true,
        status,
        motion: 1,
        frame: 1.0,
        grounded: true,
        facing: 1.0,
        stopped: false,
        targets: [
            Some(TripTarget {
                id: 2,
                status: target_status,
            }),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
    }
}
#[test]
fn trip_immediately_owns_controls_and_buffers_smash_without_readiness_or_target_rechecks() {
    let mut state = TripFollowup::default();
    let trip = TripContext {
        stopped: true,
        ..ctx(0x2c, 0x86)
    };
    let prepare = state.sample(true, trip);
    assert_eq!(prepare.requested, 0);
    assert_eq!(prepare.owned, TRIP_BUTTONS);
    assert_eq!(prepare.stick, Some([0.0, 0.0]));
    assert!(state.watching());
    assert_eq!(state.sample(true, trip), prepare);
    // A latched trip is not cancelled by missing targets or their recovery.
    // There is no can_smash input: requests start while tilt lag remains.
    for frame in 2..=31 {
        let out = state.sample(
            true,
            TripContext {
                frame: frame as f32,
                stopped: false,
                targets: [None; 8],
                ..trip
            },
        );
        assert_eq!(out.owned, TRIP_BUTTONS);
        assert_eq!(out.requested, if frame % 2 == 0 { MAPPER_A } else { 0 });
        assert_eq!(
            out.stick,
            Some(if frame % 2 == 0 {
                [1.0, 0.0]
            } else {
                [0.0, 0.0]
            })
        );
    }
    // A transition to idle still receives a forward-smash request at once.
    let ready = TripContext {
        status: 0,
        frame: 0.0,
        targets: [None; 8],
        ..trip
    };
    let ready = TripContext {
        stopped: false,
        ..ready
    };
    assert_eq!(state.sample(true, ready).requested, MAPPER_A);
    let accepted = TripContext {
        status: 0x2d,
        frame: 1.0,
        ..ready
    };
    let released = state.sample(true, accepted);
    assert_eq!(released.requested, 0);
    assert_eq!(released.stick, Some([0.0, 0.0]));
    assert!(!state.watching());
    assert_eq!(
        state
            .sample(
                true,
                TripContext {
                    frame: 2.0,
                    ..accepted
                }
            )
            .owned,
        0
    );
}
#[test]
fn either_tilt_trip_flicks_in_existing_facing_and_new_tilt_rearms() {
    for status in [0x2a, 0x2c] {
        for facing in [-1.0, 1.0] {
            for trip_status in [0x86, 0x87, 0x88] {
                let mut state = TripFollowup::default();
                let trip = TripContext {
                    facing,
                    ..ctx(status, trip_status)
                };
                assert_eq!(state.sample(true, trip).requested, 0);
                let attack = state.sample(true, TripContext { frame: 2.0, ..trip });
                assert_eq!(attack.requested, MAPPER_A);
                assert_eq!(attack.stick, Some([facing, 0.0]));
                state.sample(
                    true,
                    TripContext {
                        status: 0x2d,
                        ..trip
                    },
                );
                state.sample(true, TripContext { status: 0, ..trip });
                state.sample(true, trip);
                assert!(state.watching());
                assert_eq!(
                    state
                        .sample(true, TripContext { frame: 2.0, ..trip })
                        .requested,
                    MAPPER_A
                );
            }
        }
    }
}
#[test]
fn trip_takeover_suppresses_physical_actions_stick_and_taunts_through_release_gaps() {
    use crate::manual_controls::{ManualOverlay, TurboOutput, LEFT_STICK_DIRECTIONS};
    use crate::native_input::ButtonTrace;
    let mut state = TripFollowup::default();
    let mut overlay = ManualOverlay::default();
    let human = ButtonTrace {
        held: TRIP_BUTTONS | LEFT_STICK_DIRECTIONS,
        ..ButtonTrace::default()
    };
    for frame in 1..=5 {
        let out = state.sample(
            true,
            TripContext {
                frame: frame as f32,
                ..ctx(0x2c, 0x86)
            },
        );
        let mapped = overlay.apply_controls(
            human,
            [-1.0, -1.0],
            TurboOutput {
                consume_dpad: false,
                stick: out.stick,
            },
            0,
            out.requested,
            out.owned,
        );
        assert_eq!(mapped.buttons.held & TRIP_BUTTONS, out.requested);
        assert_eq!(mapped.stick, out.stick.unwrap());
        assert_eq!(mapped.buttons.pressed & MAPPER_A, out.requested);
    }
    let off = state.sample(false, ctx(0x2c, 0x86));
    let restored = overlay.apply_controls(
        human,
        [-1.0, -1.0],
        TurboOutput {
            consume_dpad: false,
            stick: off.stick,
        },
        0,
        off.requested,
        off.owned,
    );
    assert_eq!(
        restored.buttons.held & TRIP_BUTTONS,
        human.held & TRIP_BUTTONS
    );
    assert_eq!(restored.stick, [-1.0, -1.0]);
}
#[test]
fn no_trip_no_tilt_and_non_kirby_are_inert_and_interruptions_cancel() {
    for sample in [
        ctx(0x2c, 0),
        ctx(0, 0x86),
        TripContext {
            kirby: false,
            ..ctx(0x2c, 0x86)
        },
    ] {
        assert_eq!(
            TripFollowup::default().sample(true, sample),
            LedgeOutput::default()
        );
    }
    for terminal in [
        TripContext {
            grounded: false,
            ..ctx(0xb, 0x86)
        },
        ctx(0x47, 0x86),
    ] {
        let mut state = TripFollowup::default();
        state.sample(true, ctx(0x2c, 0x86));
        assert_eq!(state.sample(true, terminal).requested, 0);
        assert!(!state.watching());
        assert_eq!(
            state
                .sample(
                    true,
                    TripContext {
                        frame: 2.0,
                        ..terminal
                    }
                )
                .owned,
            0
        );
    }
    let mut state = TripFollowup::default();
    state.sample(true, ctx(0x2c, 0x86));
    assert_eq!(state.sample(false, ctx(0x2c, 0x86)).owned, 0);
    assert!(!state.watching());
    state.sample(true, ctx(0x2c, 0x86));
    assert_eq!(
        state
            .sample(
                true,
                TripContext {
                    object: 3,
                    ..ctx(0, 0)
                }
            )
            .owned,
        0
    );
    assert!(!state.watching());
}
#[test]
fn sdi_flicks_follow_native_hitstop_and_hands_back_on_new_manual_input() {
    let mut sdi = SdiState::default();
    let ctx = SdiContext {
        object: 1,
        damage_stop: true,
        remaining: 8,
        outward: -1.0,
        input_delay: 0.0,
    };
    for _ in 0..30 {
        assert_eq!(sdi.sample(true, ctx, [0.0; 2]), Some([-1.0, 0.0]));
    }
    assert_eq!(
        sdi.sample(
            true,
            SdiContext {
                remaining: 7,
                ..ctx
            },
            [0.0; 2]
        ),
        Some([0.0; 2])
    );
    assert_eq!(
        sdi.sample(
            true,
            SdiContext {
                remaining: 6,
                ..ctx
            },
            [0.0, 0.7]
        ),
        None
    );
    assert_eq!(
        sdi.sample(
            true,
            SdiContext {
                damage_stop: false,
                ..ctx
            },
            [0.3, 0.4]
        ),
        None
    );
    assert_eq!(sdi.sample(true, ctx, [0.0; 2]), Some([-1.0, 0.0]));
    assert_eq!(sdi.sample(false, ctx, [0.0; 2]), None);
}
#[test]
fn sdi_uses_shared_stick_overlay_without_changing_buttons_and_restores_physical_input() {
    use crate::manual_controls::{ManualOverlay, TurboOutput};
    use crate::native_input::{ButtonTrace, MAPPER_ZR};
    let mut overlay = ManualOverlay::default();
    let mut sdi = SdiState::default();
    let human = ButtonTrace {
        held: MAPPER_ZR,
        previous: MAPPER_ZR,
        ..ButtonTrace::default()
    };
    let context = SdiContext {
        object: 1,
        damage_stop: true,
        remaining: 8,
        outward: -1.0,
        input_delay: 0.0,
    };
    let stick = sdi.sample(true, context, [0.0; 2]);
    let mapped = overlay.apply_controls(
        human,
        [0.0; 2],
        TurboOutput {
            consume_dpad: false,
            stick,
        },
        0,
        0,
        0,
    );
    assert_eq!(mapped.stick, [-1.0, 0.0]);
    assert_eq!(mapped.buttons.held & MAPPER_ZR, MAPPER_ZR);
    let physical = [0.3, -0.4];
    let stick = sdi.sample(
        true,
        SdiContext {
            damage_stop: false,
            ..context
        },
        physical,
    );
    let restored = overlay.apply_controls(
        human,
        physical,
        TurboOutput {
            consume_dpad: false,
            stick,
        },
        0,
        0,
        0,
    );
    assert_eq!(restored.stick, physical);
    assert_eq!(restored.buttons.held & MAPPER_ZR, MAPPER_ZR);
}
#[test]
fn sdi_stops_early_enough_for_known_delayed_inputs_to_clear_hitstop() {
    let mut sdi = SdiState::default();
    let ctx = SdiContext {
        object: 1,
        damage_stop: true,
        remaining: 8,
        outward: -1.0,
        input_delay: 5.0,
    };
    assert_eq!(sdi.sample(true, ctx, [0.0; 2]), Some([-1.0, 0.0]));
    assert_eq!(
        sdi.sample(
            true,
            SdiContext {
                remaining: 5,
                ..ctx
            },
            [0.0; 2]
        ),
        None
    );
}

#[test]
fn sdi_ignores_stick_carried_into_hit_and_allows_release_then_reengage() {
    let mut sdi = SdiState::default();
    let ctx = SdiContext {
        object: 1,
        damage_stop: true,
        remaining: 8,
        input_delay: 0.0,
        outward: -1.0,
    };
    // Still holding toward the enemy from before damage: auto goes away.
    assert_eq!(sdi.sample(true, ctx, [1.0, 0.0]), Some([-1.0, 0.0]));
    assert_eq!(sdi.sample(true, ctx, [1.0, 0.0]), Some([-1.0, 0.0]));
    assert_eq!(
        sdi.sample(
            true,
            SdiContext {
                remaining: 7,
                ..ctx
            },
            [0.0; 2]
        ),
        Some([0.0; 2])
    );
    // A fresh movement takes full control, including the choice to go inward.
    assert_eq!(
        sdi.sample(
            true,
            SdiContext {
                remaining: 6,
                ..ctx
            },
            [1.0, 0.0]
        ),
        None
    );
    assert_eq!(
        sdi.sample(
            true,
            SdiContext {
                remaining: 5,
                ..ctx
            },
            [0.0; 2]
        ),
        None
    );
    assert_eq!(
        sdi.sample(
            true,
            SdiContext {
                damage_stop: false,
                ..ctx
            },
            [0.0; 2]
        ),
        None
    );
    assert_eq!(sdi.sample(true, ctx, [0.0; 2]), Some([-1.0, 0.0]));
}

#[test]
fn sdi_does_not_guess_a_direction_when_attacker_is_unknown() {
    let mut sdi = SdiState::default();
    let ctx = SdiContext {
        object: 1,
        damage_stop: true,
        remaining: 8,
        input_delay: 0.0,
        outward: 0.0,
    };
    assert_eq!(sdi.sample(true, ctx, [0.0; 2]), None);
    assert_eq!(
        sdi.sample(
            true,
            SdiContext {
                remaining: 7,
                ..ctx
            },
            [0.8, 0.0]
        ),
        None
    );
    assert!(sdi.manual_control);
}
