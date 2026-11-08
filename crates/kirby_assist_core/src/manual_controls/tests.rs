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
fn turbo_movement_requires_grounded_active_movement() {
    assert!(!turbo_movement_allowed(0, 2, false));
    assert!(!turbo_movement_allowed(
        crate::native_states::status::DAMAGE_FLY,
        0,
        false
    ));
    assert!(!turbo_movement_allowed(0, 0, true));
    for status in [
        crate::native_states::status::WAIT,
        crate::native_states::status::WALK,
        crate::native_states::status::WALK_BRAKE,
        crate::native_states::status::TURN,
        crate::native_states::status::SQUAT,
        crate::native_states::status::SQUAT_WAIT,
        crate::native_states::status::SQUAT_F,
        crate::native_states::status::SQUAT_B,
        crate::native_states::status::SQUAT_RV,
    ] {
        assert!(turbo_movement_allowed(
            status,
            crate::native_states::status::WAIT,
            false
        ));
    }
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
    let attack_and_up = crate::native_input::MAPPED_ATTACK | crate::native_input::MAPPED_TAUNT_UP;
    let all = attack_and_up
        | crate::native_input::MAPPED_TAUNT_SIDES
        | crate::native_input::MAPPED_TAUNT_DOWN;
    assert_eq!(
        suppress_reserved_taunts(all, DPAD_SIDES),
        attack_and_up | crate::native_input::MAPPED_TAUNT_DOWN
    );
    assert_eq!(
        suppress_reserved_taunts(all, crate::ledge_trap::DPAD_DOWN),
        attack_and_up | crate::native_input::MAPPED_TAUNT_SIDES
    );
    assert_eq!(
        suppress_reserved_taunts(all, DPAD_SIDES | crate::ledge_trap::DPAD_DOWN),
        attack_and_up
    );
    assert_eq!(suppress_reserved_taunts(all, 0), all);
}
#[test]
fn first_reservation_clears_stale_dpad_history_and_takeover_releases_human_attack() {
    use crate::native_input::MAPPER_A;
    use crate::{ledge_trap::DPAD_DOWN, native_input::ASSIST_BUTTONS};
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
        ASSIST_BUTTONS,
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
        ASSIST_BUTTONS,
    );
    assert_eq!(press.buttons.pressed & MAPPER_A, MAPPER_A);
}
#[test]
fn diagonal_up_is_quarantined_until_its_release_without_disabling_normal_up_taunt() {
    let mut reservation = DpadReservation::default();
    let diagonal = reservation.sample(DPAD_SIDES, DPAD_LEFT | DPAD_UP, true);
    assert_ne!(diagonal & DPAD_UP, 0);
    assert_eq!(
        suppress_reserved_taunts(
            crate::native_input::MAPPED_TAUNT_SIDES
                | crate::native_input::MAPPED_TAUNT_UP
                | crate::native_input::MAPPED_TAUNT_DOWN
                | crate::native_input::MAPPER_DPAD_UP,
            diagonal
        ),
        crate::native_input::MAPPED_TAUNT_DOWN | crate::native_input::MAPPER_DPAD_UP
    );
    assert_ne!(reservation.sample(DPAD_SIDES, DPAD_UP, true) & DPAD_UP, 0);
    assert_ne!(reservation.sample(DPAD_SIDES, 0, true) & DPAD_UP, 0);
    assert_eq!(reservation.sample(DPAD_SIDES, 0, true) & DPAD_UP, 0);
    assert_eq!(reservation.sample(DPAD_SIDES, DPAD_UP, true) & DPAD_UP, 0);
    assert_eq!(reservation.sample(DPAD_SIDES, DPAD_LEFT, false), 0);
}
#[test]
fn turbo_starts_crouch_and_creep_on_first_eligible_input_update() {
    let settings = TurboSettings::default();
    for status in crate::native_states::status::WAIT..=crate::native_states::status::TURN_RUN_BRAKE
    {
        assert!(turbo_movement_allowed(
            status,
            crate::native_states::status::WAIT,
            false
        ));
        let mut state = TurboState::default();
        assert_eq!(
            state
                .sample_teabag(settings, 12345, true, 1.0, true, true)
                .stick,
            Some([0.45, -1.0])
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
    for (status, situation, stopped) in [
        (
            crate::native_states::status::WALK,
            crate::native_states::situation::GROUND,
            false,
        ),
        (
            crate::native_states::status::DASH,
            crate::native_states::situation::CLIFF,
            false,
        ),
        (
            crate::native_states::status::DASH,
            crate::native_states::situation::GROUND,
            true,
        ),
    ] {
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
fn pending_movement_cancels_before_same_tick_cache_on_interruption() {
    for (status, situation, stopped, stick) in [
        (
            crate::native_states::status::DASH,
            crate::native_states::situation::AIR,
            false,
            [-1.0, 0.0],
        ),
        (
            crate::native_states::status::ATTACK,
            crate::native_states::situation::GROUND,
            false,
            [-1.0, 0.0],
        ),
        (
            crate::native_states::status::DASH,
            crate::native_states::situation::GROUND,
            true,
            [-1.0, 0.0],
        ),
        (
            crate::native_states::status::DASH,
            crate::native_states::situation::GROUND,
            false,
            [-1.0, -1.0],
        ),
        (
            crate::native_states::status::DASH,
            crate::native_states::situation::GROUND,
            false,
            [0.0, 0.0],
        ),
        (
            crate::native_states::status::DASH,
            crate::native_states::situation::GROUND,
            false,
            [-1.0, f32::NAN],
        ),
    ] {
        let mut tool = DashPivotAssist::default();
        tool.sample(1, 3, 0, false, [1.0, 0.0]);
        assert!(tool.sample(2, 3, 0, false, [-1.0, 0.0]).stick.is_some());
        assert_eq!(
            tool.sample(2, status, situation, stopped, stick).stick,
            None
        );
        assert_eq!(tool.sample(3, 0, 0, false, [-0.3, 0.0]).stick, None);
    }
}

#[test]
fn movement_handoff_and_missing_ticks_cannot_replay_a_previous_pivot_or_walk() {
    for next_tick in [0, 5] {
        let mut tool = DashPivotAssist::default();
        tool.sample(1, 3, 0, false, [1.0, 0.0]);
        tool.sample(2, 3, 0, false, [-1.0, 0.0]);
        assert_eq!(tool.sample(next_tick, 3, 0, false, [-0.3, 0.0]).stick, None);
    }
    let mut tool = DashPivotAssist::default();
    tool.sample(1, 3, 0, false, [1.0, 0.0]);
    assert_eq!(
        tool.sample(2, 3, 0, false, [0.3, 0.0]).stick,
        Some([0.49, 0.0])
    );
    tool.cancel_pending();
    assert_eq!(tool.sample(3, 1, 0, false, [0.3, 0.0]).stick, None);
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
fn turbo_binding_creeps_in_facing_immediately_and_stops_on_release() {
    let settings = TurboSettings {
        walk_strength: 0.35,
    };
    for facing in [-1.0, 1.0] {
        let mut state = TurboState::default();
        let first = state.sample_teabag(settings, 0, true, facing, true, true);
        assert_eq!(first.stick, Some([facing * 0.35, -1.0]));
        assert_eq!(
            state
                .sample_teabag(settings, 1, true, facing, true, true)
                .stick,
            Some([facing * 0.35, 0.0])
        );
        assert_eq!(
            state.sample_teabag(settings, 1, false, facing, true, true),
            TurboOutput::default()
        );
        assert_eq!(
            state
                .sample_teabag(settings, 2, true, -facing, true, true)
                .stick,
            Some([-facing * 0.35, -1.0])
        );
        assert_eq!(
            state
                .sample_teabag(settings, 3, true, facing, true, false)
                .stick,
            None
        );
    }
}
#[test]
fn l_assist_and_turbo_binding_have_independent_consumed_bits() {
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
fn turbo_binding_restores_facing_walk_cancel_at_one_update_per_phase() {
    assert_eq!(TURBO_TRIGGER, 1 << 16);
    assert_eq!(TURBO_TRIGGER & crate::settings_menu::MENU_STICK_CLICK, 0);
    for facing in [-1.0, 1.0] {
        let mut tool = TurboState::default();
        let settings = TurboSettings {
            walk_strength: 0.35,
        };
        let crouch = tool.sample_teabag(settings, 0, true, facing, true, true);
        assert_eq!(crouch.stick, Some([facing * 0.35, -1.0]));
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

#[test]
fn menu_owned_buttons_block_human_presses_without_erasing_explicit_recipe_inputs() {
    use crate::native_input::{
        MAPPER_A, MAPPER_B, MAPPER_L, MAPPER_R, MAPPER_X, MAPPER_Y, MAPPER_ZL, MAPPER_ZR,
    };
    for button in [
        MAPPER_A, MAPPER_B, MAPPER_X, MAPPER_Y, MAPPER_L, MAPPER_R, MAPPER_ZL, MAPPER_ZR,
    ] {
        let human = ButtonTrace {
            held: button,
            pressed: button,
            ..ButtonTrace::default()
        };
        let mut overlay = ManualOverlay::default();
        assert_eq!(
            overlay
                .apply_controls(human, [0.0; 2], TurboOutput::default(), button, 0, 0)
                .buttons
                .held
                & button,
            0
        );
        let frame = overlay.apply_controls(
            human,
            [0.0; 2],
            TurboOutput::default(),
            button,
            button,
            button,
        );
        assert_eq!(frame.buttons.held & button, button);
        let released =
            overlay.apply_controls(human, [0.0; 2], TurboOutput::default(), button, 0, button);
        assert_eq!(released.buttons.held & button, 0);
    }
}

#[test]
fn held_walk_stop_preserves_retreat_and_only_applies_at_known_ledge_height() {
    use crate::ledge_position::LedgeSpot;
    for inward in [-1.0, 1.0] {
        let spot = LedgeSpot {
            edge: [0.0; 2],
            inward,
        };
        let toward = [-inward * 0.3, -0.4];
        assert_eq!(
            stop_walk_at_ledge(toward, [inward * 8.0, 0.0], [0.0; 2], spot, 8.0),
            [0.0, -0.4]
        );
        let retreat = [inward * 0.3, -0.4];
        assert_eq!(
            stop_walk_at_ledge(retreat, [inward * 4.0, 0.0], [0.0; 2], spot, 8.0),
            retreat
        );
        assert_eq!(
            stop_walk_at_ledge(toward, [inward * 12.0, 0.0], [0.0; 2], spot, 8.0),
            toward
        );
        assert_eq!(
            stop_walk_at_ledge(toward, [inward * 10.0, 0.0], [-inward, 0.0], spot, 8.0),
            [0.0, -0.4]
        );
        assert_eq!(
            stop_walk_at_ledge(toward, [inward * 8.0, 20.0], [0.0; 2], spot, 8.0),
            toward
        );
    }
}
