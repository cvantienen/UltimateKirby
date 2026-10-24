use kirby_assist_core::{
    action_owner::{ActionOwnership, Intent, Owner},
    kirby_posture::Posture,
    manual_controls::ShortcutContext,
    native_input::{MAPPER_A, MAPPER_X},
};
fn context(frame: f32) -> ShortcutContext {
    ShortcutContext {
        status: 0,
        situation: 0,
        motion: 1,
        frame,
        rate: 1.0,
        stopped: false,
        facing: 1.0,
        can_short_hop: true,
        input_delay: 0.0,
        can_neutral_special: false,
    }
}

#[test]
fn dtilt_keeps_angle_in_hitstop_and_preserves_physical_tilt_stick() {
    use kirby_assist_core::{
        manual_controls::{ManualOverlay, TurboOutput, LEFT_STICK_DIRECTIONS},
        move_input::blocks_physical_c_stick,
        native_input::ButtonTrace,
    };
    let down_cstick = 1 << 26;
    for facing in [-1.0, 1.0] {
        let mut p = Posture::default();
        let mut overlay = ManualOverlay::default();
        for buttons in [MAPPER_A, 0, down_cstick] {
            let out = p.sample(
                true,
                false,
                true,
                buttons,
                [0.0, -1.0],
                Some(-facing),
                ShortcutContext {
                    status: 0x2c,
                    stopped: true,
                    facing,
                    ..context(8.0)
                },
            );
            assert_eq!(out.stick, Some([-facing * 0.8, -1.0]));
            let mapped = overlay.apply_controls(
                ButtonTrace {
                    held: buttons,
                    pressed: buttons,
                    ..Default::default()
                },
                [0.0, -1.0],
                TurboOutput {
                    consume_dpad: false,
                    stick: out.stick,
                },
                0,
                out.requested,
                LEFT_STICK_DIRECTIONS,
            );
            assert_eq!(mapped.buttons.held & (MAPPER_A | down_cstick), buttons);
            assert!(!blocks_physical_c_stick(LEFT_STICK_DIRECTIONS));
        }
    }
    assert!(blocks_physical_c_stick(
        kirby_assist_core::ledge_trap::LEDGE_BUTTONS
    ));
}
#[test]
fn idle_crouch_holds_through_entry_wait_turn_rise_and_animation_wraps() {
    let mut p = Posture::default();
    for (status, frame) in [
        (0, 0.0),
        (0x11, 0.0),
        (0x11, 1.0),
        (0x12, 0.0),
        (0x12, 40.0),
        (0x12, 0.0),
        (0x12, 0.0),
        (0x13, 0.0),
        (0x14, 0.0),
        (0x15, 0.0),
        (0x12, 0.0),
    ] {
        let c = ShortcutContext {
            status,
            motion: status as u64 + 1,
            ..context(frame)
        };
        let out = p.sample(true, true, false, 0, [0.0; 2], None, c);
        assert_eq!(
            out.stick,
            Some([0.0, -1.0]),
            "status {status:#x}, frame {frame}"
        );
        assert_eq!(out.owned, 0);
    }
    let c = ShortcutContext {
        status: 0x12,
        ..context(1.0)
    };
    assert!(p
        .sample(true, true, false, MAPPER_X, [0.0; 2], None, c)
        .stick
        .is_none());
    assert_eq!(
        p.sample(true, true, false, 0, [0.5, 0.0], None, c).stick,
        None
    );
}

#[test]
fn held_l_suspends_idle_crouch_and_release_resumes_it_without_changing_toggle() {
    use kirby_assist_core::native_input::MAPPER_L;
    let mut p = Posture::default();
    assert_eq!(
        p.sample(true, true, false, 0, [0.0; 2], None, context(0.0))
            .stick,
        Some([0.0, -1.0])
    );
    assert!(p
        .sample(
            true,
            true,
            false,
            MAPPER_L,
            [0.0; 2],
            None,
            ShortcutContext {
                status: 0x12,
                ..context(1.0)
            }
        )
        .stick
        .is_none());
    assert_eq!(
        p.sample(true, true, false, 0, [0.0; 2], None, context(2.0))
            .stick,
        Some([0.0, -1.0])
    );
}
#[test]
fn idle_crouch_never_eats_buttons_movement_air_or_stop_and_roll_wins() {
    let mut p = Posture::default();
    let crouch = p.sample(true, true, false, 0, [0.0; 2], None, context(0.0));
    assert_eq!(crouch.stick, Some([0.0, -1.0]));
    assert_eq!(crouch.owned, 0);
    for (buttons, stick, c) in [
        (MAPPER_X, [0.0; 2], context(1.0)),
        (
            0,
            [0.0; 2],
            ShortcutContext {
                situation: 2,
                ..context(3.0)
            },
        ),
        (
            0,
            [0.0; 2],
            ShortcutContext {
                stopped: true,
                ..context(4.0)
            },
        ),
    ] {
        assert!(p
            .sample(true, true, false, buttons, stick, None, c)
            .stick
            .is_none());
    }
    let mut owners = ActionOwnership::new();
    assert_eq!(
        owners.select(
            true,
            &[
                Intent {
                    owner: Owner::Posture,
                    present: true,
                    committed: false
                },
                Intent {
                    owner: Owner::RollRead,
                    present: true,
                    committed: true
                }
            ]
        ),
        Some(Owner::RollRead)
    );
}
#[test]
fn dtilt_angles_down_toward_latest_target_and_preserves_mashed_attack_edges() {
    use kirby_assist_core::{
        manual_controls::{ManualOverlay, TurboOutput, LEFT_STICK_DIRECTIONS},
        native_input::ButtonTrace,
    };
    for facing in [-1.0, 1.0] {
        let mut p = Posture::default();
        let mut overlay = ManualOverlay::default();
        let mut previous = 0;
        for (status, buttons, target, expected_x) in [
            (0x12, 0, -facing, -facing * 0.8), // Aim before the first A press.
            (0x12, MAPPER_A, -facing, -facing * 0.8),
            (0x2c, 0, -facing, -facing * 0.8),
            (0x2c, MAPPER_A, facing, 0.0),
            (0x2c, 0, facing, 0.0),
            (0x12, MAPPER_A, -facing, -facing * 0.8),
        ] {
            let c = ShortcutContext {
                status,
                facing,
                ..context(0.0)
            };
            let out = p.sample(true, false, true, buttons, [0.0, -1.0], Some(target), c);
            assert_eq!(out.requested, 0, "never synthesize Attack");
            assert_eq!(out.owned & MAPPER_A, 0, "never suppress physical Attack");
            if buttons != 0 {
                assert_eq!(out.stick, Some([expected_x, -1.0]));
            }
            let raw = ButtonTrace {
                held: buttons,
                previous,
                pressed: buttons & !previous,
                released: previous & !buttons,
            };
            let mapped = overlay.apply_controls(
                raw,
                [0.0, -1.0],
                TurboOutput {
                    consume_dpad: false,
                    stick: out.stick,
                },
                if out.owned & LEFT_STICK_DIRECTIONS != 0 {
                    (1 << 21) | (1 << 23)
                } else {
                    0
                },
                out.requested,
                out.owned,
            );
            assert_eq!(mapped.buttons.held & MAPPER_A, raw.held & MAPPER_A);
            assert_eq!(mapped.buttons.pressed & MAPPER_A, raw.pressed & MAPPER_A);
            assert_eq!(mapped.buttons.released & MAPPER_A, raw.released & MAPPER_A);
            assert_eq!(mapped.stick, [expected_x, -1.0]);
            if expected_x != 0.0 {
                let horizontal = if expected_x > 0.0 { 1 << 21 } else { 1 << 23 };
                assert_ne!(
                    mapped.buttons.held & horizontal,
                    0,
                    "the native horizontal reversal alias must survive"
                );
            }
            previous = buttons;
        }
        let out = p.sample(
            true,
            false,
            true,
            MAPPER_A,
            [0.0, -1.0],
            Some(facing),
            ShortcutContext {
                status: 0x2c,
                facing,
                ..context(1.0)
            },
        );
        assert_eq!(
            out.stick,
            Some([0.0, -1.0]),
            "no reversal when already facing the target"
        );
        assert!(p
            .sample(true, false, true, MAPPER_A, [0.0, -1.0], None, context(0.0))
            .stick
            .is_none());
    }
}
#[test]
fn shared_recipes_select_relative_moves_and_only_fast_fall_on_descent() {
    use kirby_assist_core::move_input::*;
    for facing in [-1.0, 1.0] {
        assert_eq!(aerial_stick(Aerial::Back, facing), [-facing, 0.0]);
        assert_eq!(aerial_stick(Aerial::Forward, facing), [facing, 0.0]);
        assert_eq!(aerial_stick(Aerial::Neutral, facing), [0.0; 2]);
    }
    assert!(!aerial_ready(7, 0));
    assert!(aerial_ready(0xb, 2));
    assert!(fast_fall(1.0).is_none());
    assert!(fast_fall(f32::NAN).is_none());
    assert_eq!(fast_fall(-0.5), Some([0.0, -1.0]));
}
