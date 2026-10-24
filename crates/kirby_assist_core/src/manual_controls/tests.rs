//! Existing regression cases for `manual_controls`. Kept outside the runtime code.
use super::*;

#[test]
fn stick_only_override_works_without_dpad_or_button_ownership_and_restores_input() {
    use crate::native_input::MAPPER_A;
    let mut overlay = ManualOverlay::default();
    let human = ButtonTrace {
        held: MAPPER_A | (1 << 22),
        previous: MAPPER_A | (1 << 22),
        ..ButtonTrace::default()
    };
    let physical = [0.1, -0.8];
    let mash = TurboOutput {
        consume_dpad: false,
        stick: Some([1.0, 0.0]),
    };
    let frame = overlay.apply_controls(human, physical, mash, 0, 0, 0);
    assert_eq!(frame.stick, [1.0, 0.0]);
    assert_eq!(frame.buttons.held & LEFT_STICK_DIRECTIONS, 1 << 21);
    assert_eq!(frame.buttons.held & MAPPER_A, MAPPER_A);
    assert_ne!(frame.clear_autorepeat & LEFT_STICK_DIRECTIONS, 0);
    let restored = overlay.apply_controls(human, physical, TurboOutput::default(), 0, 0, 0);
    assert_eq!(restored.stick, physical);
    assert_eq!(restored.buttons.held, human.held);
}
#[test]
fn left_right_repeat_and_direction_switch_starts_with_crouch() {
    let settings = TurboSettings {
        enabled: true,
        // This test exercises one-update phases at this explicit strength,
        // independently of the player's editable default tuning.
        phase_frames: 1,
        walk_strength: 0.25,
    };
    let mut state = TurboState::default();
    for tick in 0..6 {
        let output = state.sample(settings, tick, DPAD_LEFT, true, true);
        assert_eq!(
            output.stick,
            Some(if tick % 2 == 0 {
                [0.0, -1.0]
            } else {
                [-0.25, 0.0]
            })
        );
        assert_eq!(state.sample(settings, tick, DPAD_LEFT, true, true), output);
    }
    assert_eq!(
        state.sample(settings, 6, DPAD_RIGHT, true, true).stick,
        Some([0.0, -1.0])
    );
    assert_eq!(
        state.sample(settings, 7, DPAD_RIGHT, true, true).stick,
        Some([0.25, 0.0])
    );
}
#[test]
fn release_conflict_pause_and_unsupported_movement_cancel_immediately() {
    let settings = TurboSettings {
        enabled: true,
        ..TurboSettings::default()
    };
    let mut state = TurboState::default();
    for (buttons, gameplay, allowed) in [
        (0, true, true),
        (DPAD_SIDES, true, true),
        (DPAD_LEFT, true, false),
        (DPAD_LEFT, false, true),
    ] {
        state.sample(settings, 1, DPAD_LEFT, true, true);
        let output = state.sample(settings, 1, buttons, gameplay, allowed);
        assert!(output.stick.is_none());
        assert_eq!(output.consume_dpad, gameplay);
    }
    assert!(!turbo_movement_allowed(0, 2, false));
    assert!(!turbo_movement_allowed(0x49, 0, false));
    assert!(!turbo_movement_allowed(0, 0, true));
    for status in [0, 1, 2, 6, 0x11, 0x12, 0x13, 0x14, 0x15] {
        assert!(turbo_movement_allowed(status, 0, false));
    }
}
#[test]
fn suppresses_all_taunt_edges_preserves_attack_and_restores_physical_stick() {
    let attack = 1 << 5;
    let human = ButtonTrace {
        held: DPAD_LEFT | attack,
        previous: DPAD_LEFT,
        pressed: DPAD_LEFT | attack,
        released: DPAD_RIGHT,
    };
    let mut overlay = ManualOverlay::default();
    let crouch = overlay.apply(
        human,
        [0.4, 0.7],
        TurboOutput {
            consume_dpad: true,
            stick: Some([0.0, -1.0]),
        },
    );
    assert_eq!(crouch.buttons.held & DPAD_SIDES, 0);
    assert_eq!(crouch.buttons.previous & DPAD_SIDES, 0);
    assert_eq!(crouch.buttons.pressed & DPAD_SIDES, 0);
    assert_eq!(crouch.buttons.released & DPAD_SIDES, 0);
    assert_ne!(crouch.buttons.held & attack, 0);
    assert_ne!(crouch.buttons.pressed & (1 << 22), 0);
    let release = overlay.apply(
        ButtonTrace {
            held: attack,
            ..ButtonTrace::default()
        },
        [0.4, 0.7],
        TurboOutput {
            consume_dpad: true,
            stick: None,
        },
    );
    assert_eq!(release.stick, [0.4, 0.7]);
    assert_ne!(release.buttons.released & (1 << 22), 0);
    // Ownership hand-back may reconstruct one final edge; the following
    // sample must then be exactly the fresh physical trace.
    assert_eq!(
        overlay
            .apply(human, [0.4, 0.7], TurboOutput::default())
            .buttons
            .held,
        human.held
    );
    assert_eq!(
        overlay
            .apply(human, [0.4, 0.7], TurboOutput::default())
            .buttons,
        human
    );
}
#[test]
fn left_stick_trigger_and_attack_release_share_one_consistent_history() {
    use crate::ledge_trap::{DPAD_DOWN, LEDGE_TRIGGER};
    use crate::native_input::MAPPER_A;
    let human = ButtonTrace {
        held: LEDGE_TRIGGER | DPAD_DOWN,
        previous: LEDGE_TRIGGER | DPAD_DOWN,
        pressed: LEDGE_TRIGGER,
        released: LEDGE_TRIGGER,
    };
    let mut overlay = ManualOverlay::default();
    let press = overlay.apply_controls(
        human,
        [0.1, 0.2],
        TurboOutput::default(),
        LEDGE_TRIGGER,
        MAPPER_A,
        MAPPER_A,
    );
    assert_eq!(press.buttons.held, MAPPER_A | DPAD_DOWN);
    assert_eq!(press.buttons.pressed, MAPPER_A);
    assert_eq!(press.buttons.previous & LEDGE_TRIGGER, 0);
    assert_eq!(press.buttons.released & LEDGE_TRIGGER, 0);
    assert_ne!(press.buttons.held & DPAD_DOWN, 0);
    let release = overlay.apply(ButtonTrace::default(), [0.1, 0.2], TurboOutput::default());
    assert_eq!(release.buttons.released, MAPPER_A);
    assert_eq!(release.stick, [0.1, 0.2]);
}
#[test]
fn reserved_taunts_are_removed_from_mapped_result_without_affecting_other_actions() {
    let attack_and_up = 0x1 | 0x400;
    let all = attack_and_up | 0x7000;
    assert_eq!(
        suppress_reserved_taunts(all, DPAD_SIDES),
        attack_and_up | 0x1000
    );
    assert_eq!(
        suppress_reserved_taunts(all, crate::ledge_trap::DPAD_DOWN),
        attack_and_up | 0x6000
    );
    assert_eq!(
        suppress_reserved_taunts(all, DPAD_SIDES | crate::ledge_trap::DPAD_DOWN),
        attack_and_up
    );
    assert_eq!(suppress_reserved_taunts(all, 0), all);
}
#[test]
fn first_reservation_clears_stale_dpad_history_and_takeover_releases_human_attack() {
    use crate::ledge_trap::{DPAD_DOWN, LEDGE_BUTTONS};
    use crate::native_input::MAPPER_A;
    let mut overlay = ManualOverlay::default();
    let human = ButtonTrace {
        held: DPAD_DOWN | DPAD_LEFT | MAPPER_A,
        previous: DPAD_DOWN | DPAD_LEFT | MAPPER_A,
        pressed: DPAD_DOWN,
        released: DPAD_LEFT,
    };
    overlay.apply(human, [0.0, 0.0], TurboOutput::default());
    let setup = overlay.apply_controls(
        human,
        [0.0, 0.0],
        TurboOutput::default(),
        DPAD_SIDES | DPAD_DOWN,
        0,
        LEDGE_BUTTONS,
    );
    for word in [
        setup.buttons.held,
        setup.buttons.previous,
        setup.buttons.pressed,
        setup.buttons.released,
    ] {
        assert_eq!(word & (DPAD_SIDES | DPAD_DOWN), 0);
    }
    assert_eq!(setup.buttons.held & MAPPER_A, 0);
    let press = overlay.apply_controls(
        human,
        [0.0, 0.0],
        TurboOutput::default(),
        DPAD_SIDES | DPAD_DOWN,
        MAPPER_A,
        LEDGE_BUTTONS,
    );
    assert_eq!(press.buttons.pressed & MAPPER_A, MAPPER_A);
}
#[test]
fn diagonal_up_is_quarantined_until_its_release_without_disabling_normal_up_taunt() {
    let mut reservation = DpadReservation::default();
    let diagonal = reservation.sample(DPAD_SIDES, DPAD_LEFT | DPAD_UP, true);
    assert_ne!(diagonal & DPAD_UP, 0);
    assert_eq!(suppress_reserved_taunts(0x7400 | 1, diagonal), 0x1000 | 1);
    assert_ne!(reservation.sample(DPAD_SIDES, DPAD_UP, true) & DPAD_UP, 0);
    assert_ne!(reservation.sample(DPAD_SIDES, 0, true) & DPAD_UP, 0);
    assert_eq!(reservation.sample(DPAD_SIDES, 0, true) & DPAD_UP, 0);
    assert_eq!(reservation.sample(DPAD_SIDES, DPAD_UP, true) & DPAD_UP, 0);
    assert_eq!(reservation.sample(DPAD_SIDES, DPAD_LEFT, false), 0);
}
#[test]
fn turbo_starts_down_on_first_poll_and_in_grounded_running_states() {
    let settings = TurboSettings {
        enabled: true,
        ..TurboSettings::default()
    };
    for status in 0..=9 {
        assert!(turbo_movement_allowed(status, 0, false));
        let mut state = TurboState::default();
        assert_eq!(
            state.sample(settings, 12345, DPAD_RIGHT, true, true).stick,
            Some([0.0, -1.0])
        );
    }
}

#[test]
fn dash_pivot_holds_a_full_reversal_for_two_native_updates_only() {
    let mut pivot = DashPivotAssist::default();
    assert_eq!(pivot.sample(1, 3, 0, false, [1.0, 0.0]).stick, None);
    assert_eq!(
        pivot.sample(2, 3, 0, false, [-0.82, 0.1]).stick,
        Some([-1.0, 0.0])
    );
    // Repeated controller polls on one game update cannot consume the window.
    assert_eq!(
        pivot.sample(2, 3, 0, false, [-0.82, 0.1]).stick,
        Some([-1.0, 0.0])
    );
    assert_eq!(
        pivot.sample(3, 4, 0, false, [-0.82, 0.1]).stick,
        Some([-1.0, 0.0])
    );
    assert_eq!(pivot.sample(4, 4, 0, false, [-0.82, 0.1]).stick, None);
}

#[test]
fn dash_pivot_does_not_rewrite_walk_air_or_hitstop_input() {
    for (status, situation, stopped) in [(1, 0, false), (3, 1, false), (3, 0, true)] {
        let mut pivot = DashPivotAssist::default();
        pivot.sample(1, 3, 0, false, [1.0, 0.0]);
        assert_eq!(
            pivot
                .sample(2, status, situation, stopped, [-1.0, 0.0])
                .stick,
            None
        );
    }
}

#[test]
fn full_dash_then_same_direction_ease_becomes_max_walk_without_blocking_run() {
    let mut movement = DashPivotAssist::default();
    assert_eq!(movement.sample(1, 3, 0, false, [1.0, 0.0]).stick, None);
    assert_eq!(
        movement.sample(2, 3, 0, false, [0.3, 0.0]).stick,
        Some([0.49, 0.0])
    );
    assert_eq!(
        movement.sample(3, 1, 0, false, [0.3, 0.0]).stick,
        Some([0.49, 0.0])
    );
    assert_eq!(movement.sample(4, 1, 0, false, [0.0, 0.0]).stick, None);

    let mut running = DashPivotAssist::default();
    assert_eq!(running.sample(1, 3, 0, false, [1.0, 0.0]).stick, None);
    assert_eq!(running.sample(2, 6, 0, false, [1.0, 0.0]).stick, None);
}
#[test]
fn phase_length_and_reentry_do_not_keep_old_direction_or_clock() {
    let settings = TurboSettings {
        enabled: true,
        phase_frames: 2,
        walk_strength: 0.3,
    };
    let mut state = TurboState::default();
    for tick in 0..4 {
        assert_eq!(
            state.sample(settings, tick, DPAD_RIGHT, true, true).stick,
            Some(if tick < 2 { [0.0, -1.0] } else { [0.3, 0.0] })
        );
    }
    state.sample(settings, 4, 0, true, true);
    assert_eq!(
        state.sample(settings, 5, DPAD_RIGHT, true, true).stick,
        Some([0.0, -1.0])
    );
    assert_eq!(
        state.sample(settings, 0, DPAD_RIGHT, true, true).stick,
        Some([0.0, -1.0])
    );
}
#[test]
fn manual_ledge_walk_caps_both_sides_preserves_fine_input_and_vertical_control() {
    for (input, expected) in [
        ([1.0, 0.0], [0.5, 0.0]),
        ([-1.0, -1.0], [-(0.35 - 0.02), -1.0]),
        ([0.12, 0.7], [0.12, 0.7]),
        ([0.0, 0.0], [0.0, 0.0]),
    ] {
        assert_eq!(ledge_walk_stick(input, 0.35), expected);
    }
    assert_eq!(
        ledge_walk_stick([f32::NAN, f32::INFINITY], f32::NAN),
        [0.0, 0.0]
    );
    assert_eq!(ledge_walk_stick([1.0, 0.0], 1.0), [0.5, 0.0]);
}
#[test]
fn walk_cap_clears_full_stick_direction_bits_and_returns_physical_input_on_exit() {
    use crate::native_input::MAPPER_A;
    for direction in [-1.0, 1.0] {
        let mut overlay = ManualOverlay::default();
        let human = ButtonTrace {
            held: MAPPER_A | LEFT_STICK_DIRECTIONS,
            previous: MAPPER_A | LEFT_STICK_DIRECTIONS,
            ..ButtonTrace::default()
        };
        let physical = [direction, 0.0];
        let output = TurboOutput {
            consume_dpad: false,
            stick: Some(ledge_walk_stick(physical, 0.35)),
        };
        let frame = overlay.apply_controls(human, physical, output, (1 << 21) | (1 << 23), 0, 0);
        assert_eq!(frame.stick, [direction * 0.5, 0.0]);
        assert_eq!(frame.buttons.held & LEFT_STICK_DIRECTIONS, 0);
        assert_eq!(frame.buttons.held & MAPPER_A, MAPPER_A);
        assert_ne!(frame.clear_autorepeat & LEFT_STICK_DIRECTIONS, 0);
        let restored = overlay.apply_controls(human, physical, TurboOutput::default(), 0, 0, 0);
        assert_eq!(restored.stick, physical);
        assert_eq!(restored.buttons.held, human.held);
    }
}
#[test]
fn held_l3_turbo_creeps_in_facing_immediately_and_stops_on_release() {
    let settings = TurboSettings {
        enabled: true,
        walk_strength: 0.35,
        phase_frames: 1,
    };
    for facing in [-1.0, 1.0] {
        let mut state = TurboState::default();
        let first = state.sample_facing(settings, 0, true, facing, true, true);
        assert_eq!(first.stick, Some([facing * 0.35, -1.0]));
        assert!(!first.consume_dpad);
        assert_eq!(
            state
                .sample_facing(settings, 1, true, facing, true, true)
                .stick,
            Some([facing * 0.35, 0.0])
        );
        assert_eq!(
            state.sample_facing(settings, 1, false, facing, true, true),
            TurboOutput::default()
        );
        assert_eq!(
            state
                .sample_facing(settings, 2, true, -facing, true, true)
                .stick,
            Some([-facing * 0.35, -1.0])
        );
        assert_eq!(
            state
                .sample_facing(settings, 3, true, facing, true, false)
                .stick,
            None
        );
    }
}
#[test]
fn l_assist_and_l3_turbo_have_independent_consumed_bindings() {
    use crate::ledge_trap::LEDGE_TRIGGER;
    use crate::native_input::{MAPPER_L, MAPPER_X};
    assert_eq!(LEDGE_TRIGGER, MAPPER_L);
    assert_eq!(LEDGE_TRIGGER & TURBO_TRIGGER, 0);
    for held in [LEDGE_TRIGGER, TURBO_TRIGGER, LEDGE_TRIGGER | TURBO_TRIGGER] {
        let mut overlay = ManualOverlay::default();
        let human = ButtonTrace {
            held,
            previous: held,
            ..ButtonTrace::default()
        };
        let frame = overlay.apply_controls(
            human,
            [0.2, 0.0],
            TurboOutput::default(),
            LEDGE_TRIGGER | TURBO_TRIGGER,
            0,
            0,
        );
        assert_eq!(frame.buttons.held & (MAPPER_L | TURBO_TRIGGER), 0);
        assert_eq!(frame.buttons.pressed & MAPPER_L, 0);
        assert_eq!(frame.stick, [0.2, 0.0]);
        let jump = overlay.apply_controls(
            human,
            [0.0, 0.0],
            TurboOutput::default(),
            LEDGE_TRIGGER | TURBO_TRIGGER,
            MAPPER_X,
            MAPPER_X,
        );
        assert_eq!(jump.buttons.pressed & MAPPER_X, MAPPER_X);
    }
}

#[test]
fn l3_turbo_restores_facing_walk_cancel_at_one_update_per_phase() {
    assert_eq!(TURBO_TRIGGER, 1 << 16);
    assert_eq!(TURBO_TRIGGER & crate::ledge_menu::MENU_STICK_CLICK, 0);
    assert_eq!(SHORT_HOP_TRIGGER, crate::native_input::MAPPER_X);
    for facing in [-1.0, 1.0] {
        let mut tool = TurboState::default();
        let settings = TurboSettings {
            enabled: true,
            phase_frames: 10,
            walk_strength: 0.35,
        };
        let crouch = tool.sample_teabag(settings, 0, true, facing, true, true);
        assert_eq!(crouch.stick, Some([facing * 0.35, -1.0]));
        assert!(!crouch.consume_dpad);
        assert_eq!(
            tool.sample_teabag(settings, 0, true, facing, true, true),
            crouch
        );
        assert_eq!(
            tool.sample_teabag(settings, 1, true, facing, true, true)
                .stick,
            Some([facing * 0.35, 0.0])
        );
        assert_eq!(
            tool.sample_teabag(settings, 2, true, facing, true, true)
                .stick,
            Some([facing * 0.35, -1.0])
        );
        assert_eq!(
            tool.sample_teabag(settings, 3, false, facing, true, true),
            TurboOutput::default()
        );
        assert_eq!(
            tool.sample_teabag(settings, 4, true, facing, true, false)
                .stick,
            None
        );
    }
}
