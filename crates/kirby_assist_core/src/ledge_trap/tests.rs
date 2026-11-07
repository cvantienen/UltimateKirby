//! Host fixtures and regressions for file-only ledge recognition.
use super::*;
pub(super) fn context() -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        manual_input_epoch: 0,
        automatic_up_b_epoch: None,
        can_act: true,
        can_control: true,
        stopped: false,
        status: crate::native_states::status::WAIT,
        situation: 0,
        motion_frame: 0.0,
        motion_kind: 0,
        motion_rate: 1.0,
        position: [0.0, 0.0],
        velocity: [0.0, 0.0],
        ground_brake: Some(0.1),
        facing: 1.0,
        previous_status: crate::native_states::status::WAIT,
        can_grab: true,
        can_tilt: true,
        can_down_tilt: true,
        can_down_special: true,
        can_dash_attack: true,
        input_delay: 0.0,
        can_down_smash: true,
        can_jump: true,
        can_air_attack: true,
        air_physics: None,
        short_hop_speed: None,
        full_hop_speed: None,
        shield_stun_frames: None,
        shield_attacker: None,
        hit_confirmed: false,
        safe_ground: [true, true],
    }
}
pub(super) fn enemy(status: i32, vulnerable: bool) -> Opponent {
    Opponent {
        id: 2,
        status,
        previous_status: crate::native_states::status::WAIT,
        situation: 0,
        vulnerable,
        position: [5.0, 0.0],
        velocity: [0.0, 0.0],
        cliff_position: None,
        vulnerable_in: None,
        motion: None,
        attack_active: None,
        air_physics: None,
        actionable_in: None,
    }
}

fn input_settings() -> LedgeSettings {
    let catalog = crate::sequence_config::Catalog::parse("[kirby.neutral.n]\nstart_delay=0\nactions=a\n[kirby.jump.j]\nstart_delay=0\nactions=b\n[kirby.roll.r]\nstart_delay=0\nactions=x\n[kirby.attack.a]\nstart_delay=0\nactions=y\n[kirby.drop.d]\nstart_delay=0\nactions=r").unwrap();
    let mut profiles = crate::fighter_profiles::FighterProfiles::default();
    profiles.add_sequence_choices(&catalog).unwrap();
    let p = profiles.profile(6).unwrap();
    crate::ledge_menu::MenuSelection::new(&p.menu_options).settings(
        &p.menu_options,
        p.apply(LedgeSettings {
            enabled: true,
            ..LedgeSettings::default()
        }),
    )
}
fn frame(n: f32) -> GameplayContext {
    GameplayContext {
        motion_kind: crate::timing::motion_hash("wait"),
        motion_frame: n,
        ..context()
    }
}
#[test]
fn every_recognized_getup_selects_only_its_file_timeline() {
    for (status, situation, previous, expected) in [
        (CLIFF_CLIMB, 0, CLIFF_WAIT, MAPPER_A),
        (CLIFF_JUMP1, 2, CLIFF_WAIT, MAPPER_B),
        (CLIFF_ESCAPE, 0, CLIFF_WAIT, MAPPER_X),
        (CLIFF_ATTACK, 0, CLIFF_WAIT, MAPPER_Y),
        (0xe, 2, CLIFF_WAIT, MAPPER_R),
    ] {
        let mut trap = LedgeTrap::default();
        let output = trap.sample(
            input_settings(),
            0,
            frame(0.0),
            Some(Opponent {
                situation,
                previous_status: previous,
                ..enemy(status, false)
            }),
        );
        assert_eq!(output.requested, expected, "status {status:#x}");
    }
}
#[test]
fn hanging_and_ordinary_airborne_movement_do_not_start_an_option() {
    for target in [
        enemy(CLIFF_WAIT, false),
        Opponent {
            situation: 2,
            ..enemy(0xe, false)
        },
        enemy(0xb5, false),
    ] {
        let mut trap = LedgeTrap::default();
        assert_eq!(
            trap.sample(input_settings(), 0, frame(0.0), Some(target))
                .requested,
            0
        );
        assert!(!trap.file_timeline_active());
    }
}
#[test]
fn missing_selected_row_has_no_fallback_input_or_move_window() {
    let mut settings = input_settings();
    settings.sequence_overrides[0] = None;
    let mut trap = LedgeTrap::default();
    for n in 0..4 {
        assert_eq!(
            trap.sample(settings, n, frame(n as f32), Some(enemy(CLIFF_CLIMB, true)))
                .requested,
            0
        );
    }
    assert!(!trap.needs_response_observation());
    assert!(!trap.reserves_response(Some(enemy(CLIFF_CLIMB, true))));
}
#[test]
fn disabling_releasing_or_leaving_gameplay_resets_the_timeline() {
    for changed in [
        GameplayContext {
            held: false,
            ..frame(1.0)
        },
        GameplayContext {
            gameplay: false,
            ..frame(1.0)
        },
    ] {
        let mut trap = LedgeTrap::default();
        let settings = input_settings();
        assert_eq!(
            trap.sample(settings, 0, frame(0.0), Some(enemy(CLIFF_CLIMB, false)))
                .requested,
            MAPPER_A
        );
        assert_eq!(
            trap.sample(settings, 1, changed, None),
            ControllerOutput::default()
        );
        assert!(!trap.file_timeline_active());
        assert_eq!(trap.target(), None);
    }
    let mut trap = LedgeTrap::default();
    let mut settings = input_settings();
    trap.sample(settings, 0, frame(0.0), Some(enemy(CLIFF_CLIMB, false)));
    settings.enabled = false;
    assert_eq!(
        trap.sample(settings, 1, frame(1.0), None),
        ControllerOutput::default()
    );
}
#[test]
fn mapper_clock_reversal_cancels_and_completed_inputs_do_not_repeat() {
    let mut trap = LedgeTrap::default();
    let settings = input_settings();
    let first = trap.sample(settings, 10, frame(0.0), Some(enemy(CLIFF_CLIMB, false)));
    assert_eq!(first.requested, MAPPER_A);
    assert_eq!(trap.sample(settings, 10, frame(0.0), None), first);
    assert_eq!(trap.sample(settings, 11, frame(1.0), None).requested, 0);
    assert!(trap.manual_after_response());
    assert_eq!(
        trap.sample(settings, 12, frame(2.0), Some(enemy(CLIFF_CLIMB, false))),
        ControllerOutput::default()
    );
    assert_eq!(
        trap.sample(settings, 1, frame(0.0), None),
        ControllerOutput::default()
    );
    assert!(!trap.manual_after_response());
}

#[test]
fn menu_preview_runs_selected_timeline_once_without_getup_recognition() {
    let mut trap = LedgeTrap::default();
    trap.request_preview(PresetSlot::Jump);
    assert!(trap.needs_response_observation());
    // Ordinary idle opponent, outside the regular ledge tracking box.
    let opponent = Opponent {
        position: [100.0, 0.0],
        ..enemy(0, true)
    };
    let out = trap.sample(input_settings(), 0, frame(0.0), Some(opponent));
    assert_eq!(out.requested, MAPPER_B);
    let duplicate = trap.sample(input_settings(), 0, frame(0.0), Some(opponent));
    assert_eq!(duplicate, out);
    trap.sample(input_settings(), 1, frame(1.0), Some(opponent));
    assert!(trap.manual_after_response());
    assert!(!trap.preview_active());
    assert_eq!(
        trap.sample(input_settings(), 2, frame(2.0), Some(opponent))
            .requested,
        0
    );
}
#[test]
fn menu_preview_cancels_without_target_or_gameplay_and_on_damage() {
    for (ctx, opponent) in [
        (frame(0.0), None),
        (
            GameplayContext {
                gameplay: false,
                ..frame(0.0)
            },
            Some(enemy(0, true)),
        ),
        (
            GameplayContext {
                held: false,
                ..frame(0.0)
            },
            Some(enemy(0, true)),
        ),
        (
            GameplayContext {
                status: crate::native_states::status::DAMAGE,
                ..frame(0.0)
            },
            Some(enemy(0, true)),
        ),
    ] {
        let mut trap = LedgeTrap::default();
        trap.request_preview(PresetSlot::Neutral);
        assert_eq!(trap.sample(input_settings(), 0, ctx, opponent).requested, 0);
        assert!(!trap.preview_active());
    }
}

#[test]
fn sequence_walk_input_preserves_sub_eighty_percent_strength_in_both_facing_directions() {
    use crate::manual_controls::{ManualOverlay, TurboOutput, LEFT_STICK_DIRECTIONS};
    use crate::native_input::{mapped_stick_axis, ButtonTrace};
    use crate::sequence_config::Catalog;
    for facing in [-1.0, 1.0] {
        for strength in [0.3, 0.4, 0.5, 0.6, 0.79, 0.8] {
            let catalog = Catalog::parse(&format!(
                "[yoshi.neutral.walk]\nstart_delay=0\nactions=walk:facing:8:{strength}, grab"
            ))
            .unwrap();
            let mut settings = input_settings();
            settings.sequence_overrides[0] = Some(catalog.entries()[0].sequence);
            let mut trap = LedgeTrap::default();
            trap.request_preview(PresetSlot::Neutral);
            let ctx = GameplayContext {
                facing,
                ..frame(0.0)
            };
            let output = trap.sample(settings, 0, ctx, Some(enemy(0, true)));
            let stick = trap.requested_walk_stick().unwrap();
            assert_eq!(stick, [facing * strength, 0.0]);
            let mut overlay = ManualOverlay::default();
            let applied = overlay.apply_controls(
                ButtonTrace::default(),
                [0.0; 2],
                TurboOutput {
                    stick: output.stick,
                    consume_dpad: false,
                },
                (1 << 21) | (1 << 23),
                0,
                output.owned,
            );
            assert_eq!(applied.stick, stick);
            assert_eq!(applied.buttons.held & LEFT_STICK_DIRECTIONS, 0);
            assert_eq!(
                mapped_stick_axis(applied.stick[0]),
                (facing * strength * 127.0) as i8
            );
            assert_ne!(mapped_stick_axis(applied.stick[0]), 0);
            // Completion releases the walking override before delivering Grab.
            let next = GameplayContext {
                position: [facing * 8.0, 0.0],
                facing,
                ..frame(1.0)
            };
            assert_eq!(
                trap.sample(settings, 1, next, Some(enemy(0, true)))
                    .requested,
                MAPPER_R
            );
            assert!(trap.requested_walk_stick().is_none());
        }
    }
}

#[test]
fn walk_to_opponent_preserves_low_strength_until_its_explicit_frame_limit() {
    use crate::native_input::mapped_stick_axis;
    use crate::sequence_config::Catalog;
    for direction in [-1.0, 1.0] {
        for strength in [0.3, 0.4, 0.5, 0.6, 0.79, 0.8] {
            let catalog = Catalog::parse(&format!("[yoshi.neutral.walk]\nstart_delay=0\nactions=walk_to_opponent:{strength}:2:0, grab")).unwrap();
            let mut settings = input_settings();
            settings.sequence_overrides[0] = Some(catalog.entries()[0].sequence);
            let mut trap = LedgeTrap::default();
            trap.request_preview(PresetSlot::Neutral);
            let opponent = Opponent {
                position: [direction * 5.0, 0.0],
                ..enemy(0, true)
            };
            for tick in 0..2 {
                let ctx = GameplayContext {
                    facing: direction,
                    ..frame(tick as f32)
                };
                trap.sample(settings, tick, ctx, Some(opponent));
                let stick = trap.requested_walk_stick().unwrap();
                assert_eq!(stick, [direction * strength, 0.0]);
                assert_eq!(
                    mapped_stick_axis(stick[0]),
                    (direction * strength * 127.0) as i8
                );
            }
            let out = trap.sample(
                settings,
                2,
                GameplayContext {
                    facing: direction,
                    ..frame(2.0)
                },
                Some(opponent),
            );
            assert_eq!(out.requested, MAPPER_R);
            assert!(trap.requested_walk_stick().is_none());
        }
    }
}

#[test]
fn button_slots_execute_authored_delays_and_holds_without_a_ledge_or_move_readiness() {
    let c = crate::sequence_config::Catalog::parse("[kirby.x.route]\ndefault=true\nstart_delay=2\nactions=hold:a:3\n[kirby.y.route]\ndefault=true\nstart_delay=0\nactions=hold:b:2\n").unwrap();
    let mut profiles = crate::fighter_profiles::FighterProfiles::default();
    profiles.add_sequence_choices(&c).unwrap();
    let p = profiles.profile(6).unwrap();
    let s = crate::ledge_menu::MenuSelection::new(&p.menu_options).settings(
        &p.menu_options,
        LedgeSettings {
            enabled: true,
            ..Default::default()
        },
    );
    for (slot, expected) in [
        (
            PresetSlot::X,
            [
                0,
                0,
                crate::native_input::MAPPER_A,
                crate::native_input::MAPPER_A,
                crate::native_input::MAPPER_A,
                0,
            ],
        ),
        (
            PresetSlot::Y,
            [
                crate::native_input::MAPPER_B,
                crate::native_input::MAPPER_B,
                0,
                0,
                0,
                0,
            ],
        ),
    ] {
        let mut trap = LedgeTrap::default();
        trap.request_preview(slot);
        for (n, buttons) in expected.into_iter().enumerate() {
            let ctx = GameplayContext {
                can_act: false,
                can_control: false,
                situation: 2,
                motion_frame: n as f32,
                motion_kind: 123,
                ..context()
            };
            assert_eq!(
                trap.sample(s, n as i64, ctx, Some(enemy(0, true)))
                    .requested,
                buttons
            );
        }
    }
}

#[test]
fn ness_can_create_first_neutral_recipe_without_other_ledge_entries() {
    use crate::{
        fighter_profiles::FighterProfiles,
        ledge_menu::{MenuSelection, MENU_STICK_CLICK},
        native_input::*,
    };
    let profiles = FighterProfiles::default();
    let profile = profiles.profile(10).unwrap();
    let p = &profile.menu_options;
    let base = profile.apply(LedgeSettings {
        enabled: true,
        ..LedgeSettings::default()
    });
    let mut menu = MenuSelection::new(p);
    menu.show();
    for button in [
        0,
        MENU_STICK_CLICK,
        0,
        DPAD_DOWN,
        0,
        crate::manual_controls::DPAD_RIGHT,
        0,
    ] {
        menu.input(button, p);
    }
    let settings = menu.settings(p, base);
    assert!(settings.enabled);
    assert!(settings.sequence_overrides[0].is_some());
    assert!(settings.sequence_overrides[1..5]
        .iter()
        .all(Option::is_none));
    let mut trap = LedgeTrap::default();
    trap.sample(settings, 0, frame(0.), Some(enemy(CLIFF_WAIT, false)));
    assert_eq!(
        trap.sample(settings, 1, frame(1.), Some(enemy(CLIFF_CLIMB, true)))
            .requested,
        MAPPER_X
    );
    assert!(
        !profile
            .apply(LedgeSettings {
                enabled: false,
                ..base
            })
            .enabled
    );
}
