//! Existing regression cases for `slide`. Kept outside the runtime code.
use super::*;
pub(super) fn ground() -> SlideContext {
    SlideContext {
        gameplay: true,
        stopped: false,
        status: 0,
        situation: 0,
        motion_frame: 0.0,
        can_jump: true,
        can_air_dodge: false,
        safe_ground: [true; 2],
        input_delay: 0.0,
    }
}
fn air(frame: f32) -> SlideContext {
    SlideContext {
        status: 0xb,
        situation: 2,
        motion_frame: frame,
        can_jump: false,
        can_air_dodge: true,
        ..ground()
    }
}
#[test]
fn tap_either_direction_short_hops_dodges_and_returns_control_after_landing() {
    for (button, direction) in [(DPAD_LEFT, -1.0), (DPAD_RIGHT, 1.0)] {
        let mut state = SlideState::default();
        let jump = state.sample(true, 0, button, ground());
        assert_eq!(jump.requested, MAPPER_X);
        assert_eq!(state.sample(true, 0, button, ground()), jump);
        let squat = SlideContext {
            status: 0xa,
            motion_frame: 1.0,
            can_jump: false,
            ..ground()
        };
        assert_eq!(state.sample(true, 1, 0, squat).requested, 0);
        let dodge = state.sample(true, 2, 0, air(1.0));
        assert_eq!(dodge.requested, MAPPER_ZR);
        assert_eq!(
            dodge.stick,
            Some([direction * SLIDE_STICK_X, SLIDE_STICK_Y])
        );
        assert_eq!(state.sample(true, 2, 0, air(1.0)), dodge);
        let escape = SlideContext {
            status: 0x23,
            motion_frame: 2.0,
            ..air(1.0)
        };
        assert_eq!(state.sample(true, 3, 0, escape).requested, 0);
        let lag = SlideContext {
            status: 0x19,
            can_jump: false,
            ..ground()
        };
        assert_eq!(state.sample(true, 4, 0, lag).requested, 0);
        assert_eq!(state.sample(true, 5, 0, ground()), LedgeOutput::default());
    }
}
#[test]
fn held_direction_repeats_only_when_landing_recovery_allows_jump() {
    let mut state = SlideState::default();
    state.sample(true, 0, DPAD_RIGHT, ground());
    state.sample(true, 1, DPAD_RIGHT, air(1.0));
    state.sample(
        true,
        2,
        DPAD_RIGHT,
        SlideContext {
            status: 0x23,
            ..air(2.0)
        },
    );
    for tick in 3..8 {
        assert_eq!(
            state
                .sample(
                    true,
                    tick,
                    DPAD_RIGHT,
                    SlideContext {
                        status: 0x19,
                        can_jump: false,
                        motion_frame: tick as f32,
                        ..ground()
                    }
                )
                .requested,
            0
        );
    }
    assert_eq!(
        state.sample(true, 8, DPAD_RIGHT, ground()).requested,
        MAPPER_X
    );
}
#[test]
fn a_missed_early_window_never_becomes_a_high_air_dodge() {
    for ctx in [
        air(7.0),
        SlideContext {
            can_air_dodge: false,
            ..air(7.0)
        },
    ] {
        let mut state = SlideState::default();
        state.sample(true, 0, DPAD_RIGHT, ground());
        assert_eq!(state.sample(true, 1, DPAD_RIGHT, ctx).requested, 0);
        assert_eq!(state.sample(true, 2, DPAD_RIGHT, air(8.0)).requested, 0);
    }
}
#[test]
fn unsafe_busy_conflicting_or_disabled_starts_do_not_jump() {
    for ctx in [
        SlideContext {
            safe_ground: [false; 2],
            ..ground()
        },
        SlideContext {
            can_jump: false,
            ..ground()
        },
        SlideContext {
            status: 0x47,
            ..ground()
        },
        air(1.0),
    ] {
        assert_eq!(
            SlideState::default().sample(true, 0, DPAD_RIGHT, ctx),
            LedgeOutput::default()
        );
    }
    assert_eq!(
        SlideState::default().sample(true, 0, DPAD_SIDES, ground()),
        LedgeOutput::default()
    );
    assert_eq!(
        SlideState::default().sample(false, 0, DPAD_RIGHT, ground()),
        LedgeOutput::default()
    );
}
#[test]
fn damage_scope_loss_and_clock_rewind_cancel_but_stop_freezes_without_dodging() {
    let mut state = SlideState::default();
    state.sample(true, 0, DPAD_RIGHT, ground());
    assert_eq!(
        state.sample(
            true,
            1,
            DPAD_RIGHT,
            SlideContext {
                stopped: true,
                ..air(1.0)
            }
        ),
        owned() // Keep ownership without pressing Jump or Guard during stop.
    );
    assert_eq!(
        state.sample(true, 2, DPAD_RIGHT, air(1.0)).requested,
        MAPPER_ZR
    );
    assert_eq!(
        state.sample(
            true,
            3,
            DPAD_RIGHT,
            SlideContext {
                status: 0x47,
                ..air(2.0)
            }
        ),
        LedgeOutput::default()
    );
    assert_eq!(
        state.sample(true, 4, DPAD_RIGHT, air(1.0)),
        LedgeOutput::default()
    );
    assert_eq!(
        state.sample(
            true,
            5,
            0,
            SlideContext {
                gameplay: false,
                ..ground()
            }
        ),
        LedgeOutput::default()
    );
    assert_eq!(
        state.sample(true, 6, DPAD_RIGHT, ground()).requested,
        MAPPER_X
    );
    assert_eq!(state.sample(true, 0, 0, ground()), LedgeOutput::default());
}
#[test]
fn directional_air_dodge_landing_can_loop_on_the_first_jump_ready_observation() {
    let mut state = SlideState::default();
    state.sample(true, 0, DPAD_LEFT, ground());
    state.sample(true, 1, DPAD_LEFT, air(1.0));
    // Skip intermediate dodge/landing samples, as can happen across hooks.
    assert_eq!(
        state.sample(true, 2, DPAD_LEFT, ground()).requested,
        MAPPER_X
    );
}

#[test]
fn slide_recipe_has_real_jump_and_shield_edges_with_the_shared_overlay() {
    use crate::manual_controls::{ManualOverlay, TurboOutput, LEFT_STICK_DIRECTIONS};
    use crate::native_input::ButtonTrace;
    let mut state = SlideState::default();
    let mut overlay = ManualOverlay::default();
    let human = ButtonTrace::default();
    let mut apply = |out: LedgeOutput| {
        overlay.apply_controls(
            human,
            [-1.0, 0.0],
            TurboOutput {
                consume_dpad: false,
                stick: out.stick,
            },
            DPAD_SIDES,
            out.requested,
            out.owned,
        )
    };
    let jump = apply(state.sample(true, 0, DPAD_RIGHT, ground()));
    assert_eq!(jump.buttons.pressed & MAPPER_X, MAPPER_X);
    let release = apply(state.sample(
        true,
        1,
        DPAD_RIGHT,
        SlideContext {
            status: 0xa,
            can_jump: false,
            motion_frame: 1.0,
            ..ground()
        },
    ));
    assert_eq!(release.buttons.released & MAPPER_X, MAPPER_X);
    let dodge = apply(state.sample(true, 2, DPAD_RIGHT, air(1.0)));
    assert_eq!(dodge.buttons.pressed & MAPPER_ZR, MAPPER_ZR);
    assert_eq!(dodge.stick, [SLIDE_STICK_X, SLIDE_STICK_Y]);
    assert_eq!(
        dodge.buttons.held & LEFT_STICK_DIRECTIONS,
        1 << 21 // Shallow analog Down stays below the digital Down threshold.
    );
    let release = apply(state.sample(
        true,
        3,
        0,
        SlideContext {
            status: 0x23,
            ..air(2.0)
        },
    ));
    assert_eq!(release.buttons.released & MAPPER_ZR, MAPPER_ZR);
}
#[test]
fn downward_angle_and_guard_are_preserved_until_native_dodge_acceptance() {
    for (button, direction) in [(DPAD_RIGHT, 1.0), (DPAD_LEFT, -1.0)] {
        let mut state = SlideState::default();
        state.sample(true, 0, button, ground());
        let request = state.sample(true, 1, button, air(1.0));
        let angle = [direction * 0.9588, -0.2839];
        assert_eq!(request.stick, Some(angle));
        assert_eq!(request.requested, MAPPER_ZR);
        let awaiting = state.sample(true, 2, button, air(2.0));
        assert_eq!(awaiting.stick, Some(angle));
        assert_eq!(awaiting.requested, MAPPER_ZR);
        let accepted = state.sample(
            true,
            3,
            button,
            SlideContext {
                status: 0x23,
                ..air(1.0)
            },
        );
        assert_eq!(accepted.stick, Some([0.0, 0.0]));
        assert_eq!(accepted.requested, 0);
    }
}
#[test]
fn moving_and_cancel_ready_grounded_actions_start_without_waiting_for_idle() {
    for status in [1, 2, 3, 4, 5, 6, 7, 8, 9, 0x23, 0x29, 0x2a] {
        let ctx = SlideContext { status, ..ground() };
        assert_eq!(
            SlideState::default()
                .sample(true, 0, DPAD_RIGHT, ctx)
                .requested,
            MAPPER_X
        );
    }
}

#[test]
fn a_tapped_slide_queues_through_recovery_without_owning_inputs_then_starts_when_legal() {
    let mut state = SlideState::default();
    let busy = SlideContext {
        status: 0x29,
        can_jump: false,
        ..ground()
    };
    assert_eq!(
        state.sample(true, 0, DPAD_RIGHT, busy),
        LedgeOutput::default()
    );
    assert_eq!(state.sample(true, 1, 0, busy), LedgeOutput::default());
    let ready = SlideContext {
        can_jump: true,
        motion_frame: 20.0,
        ..busy
    };
    assert_eq!(state.sample(true, 2, 0, ready).requested, MAPPER_X);
    assert_eq!(state.sample(true, 3, 0, air(1.0)).requested, MAPPER_ZR);
}

#[test]
fn queued_slide_uses_the_correct_side_floor_probe_and_remembers_a_hitstop_tap() {
    let mut state = SlideState::default();
    let stopped = SlideContext {
        stopped: true,
        ..ground()
    };
    assert_eq!(
        state.sample(true, 0, DPAD_RIGHT, stopped),
        LedgeOutput::default()
    );
    assert_eq!(state.sample(true, 1, 0, ground()).requested, MAPPER_X);
    for (button, safe) in [(DPAD_LEFT, [true, false]), (DPAD_RIGHT, [false, true])] {
        assert_eq!(
            SlideState::default()
                .sample(
                    true,
                    0,
                    button,
                    SlideContext {
                        safe_ground: safe,
                        ..ground()
                    }
                )
                .requested,
            MAPPER_X
        );
    }
}

#[test]
fn queued_slide_expires_or_cancels_on_damage_and_disable() {
    let busy = SlideContext {
        status: 0x29,
        can_jump: false,
        ..ground()
    };
    for cancel in [
        SlideContext {
            status: 0x47,
            ..busy
        },
        SlideContext {
            gameplay: false,
            ..busy
        },
    ] {
        let mut state = SlideState::default();
        state.sample(true, 0, DPAD_RIGHT, busy);
        state.sample(true, 1, 0, cancel);
        assert_eq!(state.sample(true, 2, 0, ground()), LedgeOutput::default());
    }
    let mut state = SlideState::default();
    state.sample(true, 0, DPAD_RIGHT, busy);
    state.sample(true, 61, 0, busy);
    assert_eq!(state.sample(true, 62, 0, ground()), LedgeOutput::default());
}
#[test]
fn tap_during_air_dodge_slide_is_retained_until_grounded_jump_readiness() {
    let mut state = SlideState::default();
    let sliding = SlideContext {
        status: 0x23,
        situation: 2,
        can_jump: false,
        ..ground()
    };
    assert_eq!(
        state.sample(true, 0, DPAD_LEFT, sliding),
        LedgeOutput::default()
    );
    assert_eq!(state.sample(true, 1, 0, sliding), LedgeOutput::default());
    let grounded = SlideContext {
        situation: 0,
        can_jump: true,
        ..sliding
    };
    assert_eq!(state.sample(true, 2, 0, grounded).requested, MAPPER_X);
}

#[test]
fn holding_slide_through_long_recovery_starts_at_first_legal_window() {
    let mut state = SlideState::default();
    let busy = SlideContext {
        status: 0x29,
        can_jump: false,
        ..ground()
    };
    state.sample(true, 0, DPAD_RIGHT, busy);
    for tick in 1..90 {
        assert_eq!(
            state.sample(true, tick, DPAD_RIGHT, busy),
            LedgeOutput::default()
        );
    }
    assert_eq!(
        state.sample(true, 90, DPAD_RIGHT, ground()).requested,
        MAPPER_X
    );
}

#[test]
fn plain_dpad_starts_slide_and_scope_loss_cancels_pending_air_dodge() {
    let mut state = SlideState::default();
    assert_eq!(
        state.sample(true, 0, DPAD_RIGHT, ground()).requested,
        MAPPER_X
    );
    let blocked = SlideContext {
        gameplay: false,
        ..air(1.0)
    };
    assert_eq!(state.sample(true, 1, 0, blocked), LedgeOutput::default());
    assert_eq!(state.sample(true, 2, 0, air(2.0)), LedgeOutput::default());
    assert_eq!(state.sample(true, 3, 0, ground()), LedgeOutput::default());
}

#[test]
fn scope_loss_while_dodge_is_queued_releases_guard_and_angle() {
    let mut state = SlideState::default();
    state.sample(true, 0, DPAD_LEFT, ground());
    let dodge = state.sample(true, 1, 0, air(1.0));
    assert_eq!(dodge.requested, MAPPER_ZR);
    let blocked = SlideContext {
        gameplay: false,
        ..air(2.0)
    };
    assert_eq!(state.sample(true, 2, 0, blocked), LedgeOutput::default());
}
