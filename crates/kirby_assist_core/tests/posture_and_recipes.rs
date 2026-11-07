use kirby_assist_core::{
    action_owner::{ActionOwnership, Intent, Owner},
    posture::Posture,
    manual_controls::ShortcutContext,
    native_input::{MAPPER_A, MAPPER_X},
};
fn context(frame: f32) -> ShortcutContext {
    ShortcutContext {
        status: kirby_assist_core::native_states::status::WAIT,
        situation: kirby_assist_core::native_states::situation::GROUND,
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
fn down_cstick_with_neutral_left_stick_aims_without_changing_attack_edges() {
    use kirby_assist_core::{
        manual_controls::{ManualOverlay, TurboOutput},
        native_input::ButtonTrace,
    };
    let down = 1 << 26;
    for facing in [-1.0, 1.0] {
        let mut p = Posture::default();
        let physical = ButtonTrace {
            held: down,
            pressed: down,
            ..Default::default()
        };
        let out = p.sample(
            true,
            false,
            true,
            down,
            [0.0; 2],
            Some(-facing),
            ShortcutContext {
                facing,
                ..context(1.0)
            },
        );
        assert_eq!(out.stick, Some([-facing * 0.8, -1.0]));
        assert_eq!(out.requested, 0);
        let mapped = ManualOverlay::default().apply_controls(
            physical,
            [0.0; 2],
            TurboOutput {
                consume_dpad: false,
                stick: out.stick,
            },
            0,
            out.requested,
            out.owned,
        );
        assert_eq!(mapped.buttons.held & down, down);
        assert_eq!(mapped.buttons.pressed & down, down);
        assert_eq!(mapped.buttons.held & MAPPER_A, 0);
        let facing_correct = p.sample(
            true,
            false,
            true,
            down,
            [0.0; 2],
            Some(facing),
            ShortcutContext {
                facing,
                ..context(2.0)
            },
        );
        assert_eq!(facing_correct.stick, Some([0.0, -1.0]));
        assert_eq!(
            p.sample(
                true,
                false,
                true,
                down,
                [0.0; 2],
                Some(-facing),
                ShortcutContext {
                    situation: kirby_assist_core::native_states::situation::AIR,
                    ..context(3.0)
                }
            )
            .stick,
            None
        );
        assert_eq!(
            p.sample(
                true,
                false,
                false,
                down,
                [0.0; 2],
                Some(-facing),
                context(4.0)
            )
            .stick,
            None
        );
    }
}

#[test]
fn crouch_yields_to_grab_edges_and_restores_neutral_without_a_down_tail() {
    use kirby_assist_core::{
        manual_controls::{ManualOverlay, TurboOutput, LEFT_STICK_DIRECTIONS},
        native_input::{ButtonTrace, MAPPER_R},
    };
    let mut posture = Posture::default();
    let mut overlay = ManualOverlay::default();
    let sequence = [
        ButtonTrace::default(),
        ButtonTrace {
            held: MAPPER_R,
            pressed: MAPPER_R,
            ..Default::default()
        },
        ButtonTrace {
            previous: MAPPER_R,
            released: MAPPER_R,
            ..Default::default()
        },
        ButtonTrace::default(),
    ];
    for (tick, physical) in sequence.into_iter().enumerate() {
        let p = posture.sample(
            true,
            Posture::idle_buttons_clear(physical),
            false,
            physical.held,
            [0.0; 2],
            None,
            context(tick as f32),
        );
        let mapped = overlay.apply_controls(
            physical,
            [0.0; 2],
            TurboOutput {
                consume_dpad: false,
                stick: p.stick,
            },
            0,
            p.requested,
            p.owned,
        );
        assert_eq!(mapped.buttons.held & MAPPER_R, physical.held & MAPPER_R);
        if tick == 0 || tick == 3 {
            assert_eq!(mapped.stick, [0.0, -1.0]);
            assert_eq!(mapped.buttons.held & (MAPPER_A | MAPPER_R), 0);
        } else {
            assert_eq!(mapped.stick, [0.0; 2]);
            assert_eq!(mapped.buttons.held & LEFT_STICK_DIRECTIONS, 0);
        }
    }
    // Once a native catch/hold/throw is visible, neutral physical input must
    // never become an automatic Down that can select a throw direction.
    for status in kirby_assist_core::native_states::CATCH..=kirby_assist_core::native_states::THROW
    {
        let out = posture.sample(
            true,
            true,
            false,
            0,
            [0.0; 2],
            None,
            ShortcutContext {
                status,
                ..context(10.0)
            },
        );
        assert_eq!(out.stick, None, "grab status={status:#x}");
        assert_eq!(out.requested, 0);
    }
}

#[test]
fn omitted_facing_scans_cannot_change_unusable_posture_states() {
    for status in [
        kirby_assist_core::native_states::status::WAIT,
        kirby_assist_core::native_states::status::DASH,
        kirby_assist_core::native_states::status::TURN_RUN_BRAKE,
        kirby_assist_core::native_states::status::SQUAT,
        kirby_assist_core::native_states::status::SQUAT_RV,
        kirby_assist_core::native_states::status::JUMP,
        kirby_assist_core::native_states::status::LANDING_ATTACK_AIR,
        kirby_assist_core::native_states::status::ATTACK_LW3,
        kirby_assist_core::native_states::status::ATTACK_LW4,
        kirby_assist_core::native_states::status::DAMAGE,
    ] {
        for situation in [kirby_assist_core::native_states::situation::GROUND, kirby_assist_core::native_states::situation::AIR] {
            for stopped in [false, true] {
                let c = ShortcutContext {
                    status,
                    situation,
                    stopped,
                    ..context(1.0)
                };
                let needed = Posture::needs_target(status, situation, stopped);
                let mut reference = Posture::default();
                let mut gated = Posture::default();
                assert_eq!(
                    gated.sample(
                        true,
                        true,
                        true,
                        MAPPER_A,
                        [0.0, -1.0],
                        needed.then_some(-1.0),
                        c
                    ),
                    reference.sample(true, true, true, MAPPER_A, [0.0, -1.0], Some(-1.0), c)
                );
            }
        }
    }
    assert!(Posture::needs_target(0x2c, 0, true));
    assert!(!Posture::needs_target(0, 2, false));
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
                    status: kirby_assist_core::native_states::status::ATTACK_LW3,
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
        kirby_assist_core::ledge_trap::ASSIST_BUTTONS
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
        status: kirby_assist_core::native_states::status::SQUAT_WAIT,
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
                status: kirby_assist_core::native_states::status::SQUAT_WAIT,
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
                situation: kirby_assist_core::native_states::situation::AIR,
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
                status: kirby_assist_core::native_states::status::ATTACK_LW3,
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

#[test]
fn smash_retry_rhythm_has_clean_release_frames_and_target_variation() {
    use kirby_assist_core::move_input::smash_retry_pressed;
    let even: Vec<_> = (0..12)
        .filter(|frame| smash_retry_pressed(*frame as f32, 2))
        .collect();
    let odd: Vec<_> = (0..12)
        .filter(|frame| smash_retry_pressed(*frame as f32, 3))
        .collect();
    assert_eq!(even, [2, 3, 5, 6, 7]);
    assert_eq!(odd, [2, 3, 4, 6, 7, 9]);
    assert_ne!(even, odd);
    assert!(!smash_retry_pressed(f32::NAN, 2));
    assert!(!smash_retry_pressed(-1.0, 2));
}

#[test]
fn idle_crouch_starts_early_in_range_and_stops_for_missing_or_distant_enemies() {
    let mut posture = Posture::default();
    for (offset, crouch) in [
        (Some([45.0, 0.0]), true),
        (Some([-45.0, 20.0]), true),
        (Some([0.0, 0.0]), true),
        (Some([45.1, 0.0]), false),
        (Some([0.0, 20.1]), false),
        (Some([f32::NAN, 0.0]), false),
        (None, false),
    ] {
        let output = posture.sample(
            true,
            Posture::opponent_near(offset),
            false,
            0,
            [0.0; 2],
            None,
            context(0.0),
        );
        assert_eq!(output.stick.is_some(), crouch);
    }
}

#[test]
fn raw_down_cstick_first_poll_from_standing_and_crouch_needs_no_digital_alias() {
    use kirby_assist_core::{
        manual_controls::{ManualOverlay, TurboOutput},
        native_input::ButtonTrace,
    };
    for status in [
        kirby_assist_core::native_states::status::WAIT,
        kirby_assist_core::native_states::status::WALK,
        kirby_assist_core::native_states::status::SQUAT,
        kirby_assist_core::native_states::status::SQUAT_WAIT,
        kirby_assist_core::native_states::status::ATTACK_LW3,
    ] {
        for facing in [-1.0, 1.0] {
            let c = ShortcutContext {
                status,
                facing,
                ..context(1.0)
            };
            let right = [0.15, -1.0];
            assert!(Posture::down_tilt_intent(0, [0.0; 2], right));
            let out = Posture::default().sample_controls(
                true,
                false,
                true,
                0,
                [0.0; 2],
                right,
                Some(-facing),
                c,
            );
            assert_eq!(out.stick, Some([-facing * 0.8, -1.0]));
            assert_eq!(out.requested, 0);
            assert_eq!(out.owned, 0);
            assert_eq!(
                Posture::aimed_c_stick(0, right, out.stick),
                Some([-facing * 0.8, -1.0])
            );
            let mapped = ManualOverlay::default().apply_controls(
                ButtonTrace::default(),
                [0.0; 2],
                TurboOutput {
                    consume_dpad: false,
                    stick: out.stick,
                },
                0,
                out.requested,
                out.owned,
            );
            assert_eq!(mapped.buttons.held & (MAPPER_A | (0xf << 24)), 0);
            assert_eq!(mapped.buttons.pressed & (MAPPER_A | (0xf << 24)), 0);
        }
    }
}

#[test]
fn raw_cstick_aim_releases_immediately_and_cannot_convert_other_tilts_or_aerials() {
    use kirby_assist_core::native_input::MAPPER_RIGHT_STICK_DOWN;
    let mut p = Posture::default();
    let out = p.sample_controls(
        true,
        false,
        true,
        0,
        [0.0; 2],
        [0.0, -1.0],
        Some(-1.0),
        context(1.0),
    );
    assert!(out.stick.is_some());
    let out = p.sample_controls(
        true,
        false,
        true,
        0,
        [0.0; 2],
        [0.0; 2],
        Some(-1.0),
        context(2.0),
    );
    assert_eq!(out.stick, None);
    assert_eq!(
        Posture::aimed_c_stick(MAPPER_RIGHT_STICK_DOWN, [0.0; 2], Some([-0.8, -1.0])),
        None
    );
    for right in [[1.0, 0.0], [0.0, 1.0], [-1.0, -0.4], [f32::NAN, 0.0]] {
        let out = p.sample_controls(
            true,
            false,
            true,
            0,
            [0.0, -1.0],
            right,
            Some(-1.0),
            context(3.0),
        );
        assert_eq!(out.stick, None, "{right:?}");
        assert_eq!(Posture::aimed_c_stick(0, right, out.stick), None);
    }
    for (enabled, turn, c) in [
        (false, true, context(4.0)),
        (true, false, context(4.0)),
        (
            true,
            true,
            ShortcutContext {
                situation: kirby_assist_core::native_states::situation::AIR,
                ..context(4.0)
            },
        ),
    ] {
        let out = p.sample_controls(
            enabled,
            false,
            turn,
            0,
            [0.0; 2],
            [0.0, -1.0],
            Some(-1.0),
            c,
        );
        assert_eq!(out.stick, None);
        assert_eq!(Posture::aimed_c_stick(0, [0.0, -1.0], out.stick), None);
    }
}
