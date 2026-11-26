//! Final controller delivery regressions shared by ledge/button/Auto react tapes.
use kirby_assist_core::{
    gameplay::{GameplayContext, Opponent},
    ledge_trap::{InputTimeline, SEQUENCE_BUTTONS},
    manual_controls::{ManualOverlay, TurboOutput, LEFT_STICK_DIRECTIONS},
    native_input::*,
    sequence_config::Catalog,
    settings::AssistSettings,
};
#[test]
fn counter_literal_delivery_survives_raw_mapping_loss_in_both_layouts() {
    use kirby_assist_core::{auto_counter::{Counter, Config, Context, Threat},
        fighter_kinds, native_states::{status, situation}, timing::AnimationSample};
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        let mut counter = Counter::new();
        let c = Context { kind: fighter_kinds::LUCINA, situation: situation::AIR,
            can_special: true, stopped: false, animation: AnimationSample {
                object: 1, status: status::FALL, kind: 1, frame: 1., auxiliary: 0. } };
        let config = Config::for_fighter(fighter_kinds::LUCINA, Default::default()).unwrap();
        let out = counter.sample(true, Some(config), c, Some(Threat {
            id: 7, motion: 9, status: status::ATTACK, frame: 3., in_frames: 1. }));
        assert_eq!(out.requested, MAPPER_B);
        let raw = ManualOverlay::default().apply_reaction_controls(ButtonTrace::default(),
            [1., 0.], TurboOutput { stick: out.stick }, 0,
            layout.physical_buttons(out.requested), layout.physical_owned_buttons(out.owned), true);
        assert_eq!(raw.stick, [0., -1.]);
        let mapped = mapped_literal_buttons(MAPPED_ATTACK | MAPPED_GUARD, out.requested);
        assert_eq!(mapped & MAPPED_SPECIAL, MAPPED_SPECIAL);
        assert_eq!(mapped & (MAPPED_ATTACK | MAPPED_GUARD), 0);
        counter.mapped();
        assert!(counter.awaiting_receipt());
    }
}

#[test]
fn spacing_fair_delivery_preserves_direction_in_both_control_layouts() {
    use kirby_assist_core::{auto_spacing::{Spacing, Config, Attack, AttackConfig},
        native_states::{situation, status}};
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for side in [-1., 1.] {
            let mut spacing = Spacing::default();
            let c = GameplayContext { gameplay: true, can_control: true, can_air_attack: true,
                situation: situation::AIR, status: status::FALL, facing: side, ..Default::default() };
            let config = Config { aerial: AttackConfig { attack: Attack::Forward, startup: 6.,
                active_frames: 3., reach: 20., height: 12., commitment: 30. }, ..Default::default() };
            let out = spacing.sample(1, true, config, c, Some(Opponent {
                position: [side * 10., 0.], vulnerable: true, ..Default::default()
            }));
            let physical = layout.physical_buttons(MAPPER_A | MAPPER_B | SEQUENCE_JUMP_BUTTONS);
            let raw = ManualOverlay::default().apply_reaction_controls(
                ButtonTrace { held: physical, previous: physical, ..Default::default() },
                [-side, 0.], TurboOutput { stick: out.controller.stick, ..Default::default() },
                0, layout.physical_buttons(out.controller.requested),
                layout.physical_owned_buttons(out.controller.owned), true,
            );
            assert_eq!(raw.stick, [side, 0.]);
            assert_ne!(raw.buttons.held & layout.physical_buttons(MAPPER_A), 0);
            assert_eq!(raw.buttons.held & MAPPER_RIGHT_STICK_DIRECTIONS, 0);
            let mapped = mapped_spacing_attack(MAPPED_JUMP | MAPPED_SPECIAL | 0x80, out.attack, false);
            assert_ne!(mapped & MAPPED_ATTACK, 0);
            assert_ne!(mapped & MAPPED_JUMP, 0);
            assert_eq!(mapped & (MAPPED_SPECIAL | 0x80), 0);
            assert_eq!(mapped_stick_axis(raw.stick[0]), (side * 127.) as i8);
        }
    }
}

#[test]
fn spacing_final_tilts_preserve_direction_instead_of_becoming_jabs() {
    use kirby_assist_core::{auto_spacing::{Attack, AttackConfig, Config, Spacing},
        native_states::{situation, status}};
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for side in [-1., 1.] {
            for (attack, direction) in [(Attack::Forward, [side * 0.6, 0.]),
                (Attack::Up, [0., 0.6]), (Attack::Down, [0., -0.6]), (Attack::Neutral, [0.; 2])] {
                let c = GameplayContext { gameplay: true, can_control: true, can_act: true,
                    situation: situation::GROUND, status: status::WAIT, facing: side,
                    walk_stick: Some(0.69), safe_ground: [true; 2], ..Default::default() };
                let cfg = Config { ground_speed: 0., minimum_drift: 0., ground: AttackConfig {
                    attack, startup: 8., active_frames: 5., reach: 20., height: 9., commitment: 33.,
                }, ..Default::default() };
                let mut spacing = Spacing::default();
                for tick in 1..=3 {
                    let out = spacing.sample(tick, true, cfg, c, Some(Opponent {
                        position: [side * 18., 0.], vulnerable: true, ..Default::default()
                    }));
                    assert_eq!(out.right_stick, [0.; 2]);
                    assert_eq!(out.controller.requested, if tick == 2 { 0 } else { MAPPER_A });
                    assert_eq!(out.controller.stick, Some(if tick == 2 { [0.; 2] } else { direction }));
                    let human = layout.physical_buttons(MAPPER_A | MAPPER_B | SEQUENCE_JUMP_BUTTONS
                        | MAPPER_LEFT_STICK_DIRECTIONS | MAPPER_RIGHT_STICK_DIRECTIONS);
                    let raw = ManualOverlay::default().apply_reaction_controls(
                        ButtonTrace { held: human, previous: human, ..Default::default() },
                        [-side, -1.], TurboOutput { stick: out.controller.stick, ..Default::default() },
                        MAPPER_LEFT_STICK_HORIZONTAL, layout.physical_buttons(out.controller.requested),
                        layout.physical_owned_buttons(out.controller.owned), true,
                    );
                    assert_eq!(raw.buttons.held & (MAPPER_LEFT_STICK_HORIZONTAL | MAPPER_RIGHT_STICK_DIRECTIONS), 0);
                    assert_eq!(raw.buttons.held & MAPPER_LEFT_STICK_UP,
                        if attack == Attack::Up && tick != 2 { MAPPER_LEFT_STICK_UP } else { 0 });
                    assert_eq!(raw.buttons.held & MAPPER_LEFT_STICK_DOWN,
                        if attack == Attack::Down && tick != 2 { MAPPER_LEFT_STICK_DOWN } else { 0 });
                    let (buttons, left, right) = mapped_spacing_input(
                        MAPPED_JUMP | MAPPED_SPECIAL | MAPPED_GUARD | 0x80,
                        out.attack, raw.stick, out.right_stick,
                    );
                    assert_eq!(buttons & MAPPED_ATTACK_HOLD_BITS,
                        if tick == 2 { 0 } else { MAPPED_ATTACK_HOLD_BITS });
                    assert_eq!(buttons & MAPPED_JUMP, MAPPED_JUMP);
                    assert_eq!(buttons & (MAPPED_SPECIAL | MAPPED_GUARD | 0x80), 0);
                    assert_eq!(left, if tick == 2 { [0.; 2] } else { direction });
                    assert_eq!(right, [0.; 2]);
                    if out.attack { spacing.mapped(tick); }
                }
            }
        }
    }
    for direction in [[0., 0.], [-1., 0.], [1., 0.], [0., 1.], [0., -1.]] {
        let (buttons, left, right) = mapped_spacing_input(MAPPED_JUMP, true, direction, [0.; 2]);
        assert_eq!(left, direction);
        assert_eq!(right, [0.; 2]);
        assert_eq!(buttons, MAPPED_JUMP | MAPPED_ATTACK_HOLD_BITS);
    }
}

#[test]
fn adaptive_spacing_drift_survives_manual_overlay_without_weakening_fair_selection() {
    use kirby_assist_core::{auto_spacing::{Spacing, Config, Attack, AttackConfig},
        native_states::{situation, status}};
    let c = GameplayContext { gameplay: true, can_control: true, situation: situation::AIR,
        status: status::FALL, facing: 1., ..Default::default() };
    let cfg = Config { minimum_distance: 15., maximum_drift: 0.6, minimum_drift: 0.1,
        braking_distance: 10., ..Default::default() };
    let target = Some(Opponent { position: [20., 0.], vulnerable: true, ..Default::default() });
    let out = Spacing::default().sample(1, true, cfg, c, target);
    assert_eq!(out.controller.stick, None);
    for (input, expected) in [(1., 1.), (-1., -1.)] {
        let stick = kirby_assist_core::auto_spacing::manual_movement([input, 0.], out.movement_caps.unwrap());
        let raw = ManualOverlay::default().apply_reaction_controls(
            ButtonTrace::default(), [input, 0.], TurboOutput { stick: Some(stick), ..Default::default() },
            0, out.controller.requested, out.controller.owned, true,
        );
        assert!((raw.stick[0] - expected).abs() < 0.0001);
        assert_eq!(mapped_stick_axis(raw.stick[0]), mapped_stick_axis(expected));
    }
    let attack = Spacing::default().sample(2, true, Config {
        aerial: AttackConfig { attack: Attack::Forward, startup: 6., active_frames: 3.,
            reach: 20., height: 12., commitment: 30. }, ..cfg
    }, GameplayContext { can_air_attack: true, ..c }, target);
    assert!(attack.attack);
    assert_eq!(attack.controller.stick, Some([1., 0.]));
}

#[test]
fn spacing_manual_approach_preserves_direction_with_safe_movement_caps() {
    use kirby_assist_core::{auto_spacing::{Config, Spacing}, native_states::{situation, status}};
    for side in [-1., 1.] {
        for speed in [0., 0.5, 1.] {
            let c = GameplayContext { gameplay: true, can_control: true, can_act: true,
                situation: situation::GROUND, status: status::WAIT, facing: -side,
                walk_stick: Some(88. / 127.), safe_ground: [true; 2], ..Default::default() };
            let out = Spacing::default().sample(1, true,
                Config { ground_speed: speed, ..Default::default() }, c,
                Some(Opponent { position: [side * 40., 0.], ..Default::default() }));
            let physical = MAPPER_A | MAPPER_B | MAPPER_X | MAPPER_LEFT_STICK_HORIZONTAL;
            let raw = ManualOverlay::default().apply_reaction_controls(
                ButtonTrace { held: physical, previous: physical, ..Default::default() },
                [side, 0.], TurboOutput {
                    stick: Some(kirby_assist_core::auto_spacing::manual_movement(
                        [side, 0.], out.movement_caps.unwrap())), ..Default::default()
                },
                MAPPER_LEFT_STICK_HORIZONTAL, out.controller.requested, out.controller.owned, true,
            );
            assert_eq!(out.controller.stick, None);
            let (buttons, left, right) = mapped_spacing_input(
                MAPPED_JUMP | MAPPED_ATTACK | MAPPED_SPECIAL, out.attack, raw.stick, out.right_stick);
            assert_eq!(left, [side * speed.min(88. / 127.), 0.]);
            assert_eq!(right, [0.; 2]);
            assert_eq!(buttons, MAPPED_JUMP);
            assert_eq!(raw.buttons.held & MAPPER_LEFT_STICK_HORIZONTAL, 0);
            assert_eq!(raw.buttons.held & (MAPPER_A | MAPPER_B), MAPPER_A | MAPPER_B);
            assert_eq!(raw.buttons.held & MAPPER_X, MAPPER_X);
        }
    }
}

#[test]
fn spacing_manual_walk_keeps_buttons_and_direction_but_removes_dash_flicks() {
    let cap = fastest_walk_stick(0.2, 0.8).unwrap();
    for side in [-1., 1.] {
        let physical = MAPPER_A | MAPPER_X | MAPPER_LEFT_STICK_HORIZONTAL;
        let raw = ManualOverlay::default().apply_reaction_controls(
            ButtonTrace { held: physical, previous: physical, ..Default::default() },
            [side, 0.], TurboOutput {
                stick: Some(kirby_assist_core::auto_spacing::manual_walk([side, 0.], cap)),
                ..Default::default()
            }, MAPPER_LEFT_STICK_HORIZONTAL, 0, 0, true,
        );
        assert_eq!(raw.stick, [side * cap, 0.]);
        assert_eq!(raw.buttons.held & MAPPER_LEFT_STICK_HORIZONTAL, 0);
        assert_eq!(raw.buttons.held & (MAPPER_A | MAPPER_X), MAPPER_A | MAPPER_X);
        assert!((mapped_stick_axis(raw.stick[0]) as f32 / 120.).abs() < 0.8);
    }
}

#[test]
fn partial_spacing_walk_suppresses_generated_direction_flicks() {
    let owned = MAPPER_LEFT_STICK_DIRECTIONS;
    for side in [-1.0, 1.0] {
        let mut overlay = ManualOverlay::default();
        let raw = overlay.apply_reaction_controls(
            ButtonTrace { held: MAPPER_A | MAPPER_LEFT_STICK_HORIZONTAL,
                ..Default::default() },
            [side, 0.0], TurboOutput { stick: Some([side * 0.6, 0.0]) },
            sequence_flick_consumption(owned, true), 0, owned, true,
        );
        assert_eq!(raw.stick, [side * 0.6, 0.0]);
        assert_eq!(raw.buttons.held & MAPPER_LEFT_STICK_HORIZONTAL, 0);
        assert_eq!(raw.buttons.pressed & MAPPER_LEFT_STICK_HORIZONTAL, 0);
        assert_ne!(raw.buttons.held & MAPPER_A, 0);
    }
}

#[test]
fn spacing_retreat_keeps_full_axis_and_dash_flick_while_manual_walk_is_capped() {
    use kirby_assist_core::{auto_spacing::{Config, Spacing}, gameplay::{GameplayContext, Opponent},
        native_states::{situation, status}};
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for side in [-1., 1.] {
            let c = GameplayContext {
                gameplay: true, can_control: true, can_act: true,
                situation: situation::GROUND, status: status::WAIT,
                facing: side, safe_ground: [true; 2], walk_stick: Some(0.6),
                ..Default::default()
            };
            let out = Spacing::default().sample_threat(1, true, Config::default(), c,
                Some(Opponent { id: 2, position: [side * 25., 0.], ..Default::default() }), true);
            let selected = out.controller;
            let walking = !selected.stick.is_some_and(|stick| stick[0].abs() >= 0.8);
            let raw = ManualOverlay::default().apply_reaction_controls(
                ButtonTrace::default(), [side, 0.], TurboOutput { stick: selected.stick },
                sequence_flick_consumption(selected.owned, walking),
                layout.physical_buttons(selected.requested),
                layout.physical_owned_buttons(selected.owned), true,
            );
            assert_eq!(raw.stick, [-side, 0.]);
            let direction = if side > 0. { MAPPER_LEFT_STICK_LEFT } else { MAPPER_LEFT_STICK_RIGHT };
            assert_ne!(raw.buttons.pressed & direction, 0);
            assert_eq!(mapped_stick_axis(raw.stick[0]), (-side * 127.) as i8);
            assert_eq!(kirby_assist_core::auto_spacing::manual_movement([side, 0.],
                out.movement_caps.unwrap()), [side * 0.49, 0.]);
        }
    }
}

#[test]
fn spacing_final_walk_cap_blocks_full_strength_reversals_after_mapping() {
    for dash_threshold in [0.5, 0.66, 0.8, 1.] {
        let cap = fastest_walk_stick(0.2, dash_threshold).unwrap();
        for input in [127, -127, 127, -127, 0, 12, -12] {
            let axis = kirby_assist_core::auto_spacing::mapped_walk_axis(input, cap);
            assert!((f32::from(axis) / 120.).abs() < dash_threshold);
            assert_eq!(axis.signum(), input.signum());
            if input.abs() <= 12 { assert_eq!(axis, input); }
        }
    }
    for cap in [0., f32::NAN, -1.] {
        assert_eq!(kirby_assist_core::auto_spacing::mapped_walk_axis(127, cap), 0);
        assert_eq!(kirby_assist_core::auto_spacing::mapped_walk_axis(-127, cap), 0);
    }
}

#[test]
fn shared_defense_walks_between_threats_without_consuming_attacks_or_air_drift() {
    use kirby_assist_core::{auto_spacing::manual_walk,
        defense_response::{Context, Contact, Defense}, native_states::situation::{GROUND, AIR}};
    let cap = fastest_walk_stick(0.2, 0.8).unwrap();
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for side in [-1., 1.] {
            for (situation, movement) in [
                (GROUND, manual_walk([side, 0.], cap)), (AIR, [side, 0.])
            ] {
                let mut defense = Defense::new();
                let c = Context { object: 1, situation, allowed: true, guard_ready: true,
                    air_dodge_ready: true, ..Default::default() };
                let mut overlay = ManualOverlay::default();
                for (tick, threat) in [(0, false), (1, true), (2, false)] {
                    let contact = threat.then_some(Contact { source: 2, in_frames: 0., projectile: false });
                    let out = kirby_assist_core::action_owner::exclusive_output(
                        Some(kirby_assist_core::action_owner::Owner::Defense), true,
                        defense.sample(tick, true, c, contact, 4., 0.));
                    let physical = layout.physical_buttons(MAPPER_A | MAPPER_B | MAPPER_X
                        | MAPPER_LEFT_STICK_HORIZONTAL | MAPPER_RIGHT_STICK_DIRECTIONS);
                    let walking = situation == GROUND && !threat;
                    let raw = overlay.apply_reaction_controls(
                        ButtonTrace { held: physical, previous: physical, ..Default::default() },
                        [side, 0.], TurboOutput { stick: out.stick.or(Some(movement)) },
                        layout.physical_buttons(if walking { MAPPER_LEFT_STICK_HORIZONTAL } else { 0 }),
                        layout.physical_buttons(out.requested), layout.physical_owned_buttons(out.owned), true);
                    if threat {
                        assert_eq!(raw.stick, [0.; 2]);
                        assert_ne!(raw.buttons.held & layout.physical_buttons(MAPPER_ZR), 0);
                    } else {
                        assert_eq!(raw.stick, movement);
                        let actions = layout.physical_buttons(MAPPER_A | MAPPER_B | MAPPER_X
                            | MAPPER_RIGHT_STICK_DIRECTIONS);
                        assert_eq!(raw.buttons.held & actions, actions);
                        assert_eq!(raw.buttons.held & layout.physical_buttons(MAPPER_ZR), 0);
                        if walking {
                            assert!((f32::from(mapped_stick_axis(raw.stick[0])) / 120.).abs() < 0.8);
                            assert_eq!(raw.buttons.pressed & MAPPER_LEFT_STICK_HORIZONTAL, 0);
                        } else {
                            assert_eq!(raw.stick, [side, 0.]);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn move_input_axis_packet_replaces_competing_physical_direction_bits() {
    for stick in [[0., 0.6], [0., -0.6], [0.6, 0.], [0.; 2]] {
        let out = kirby_assist_core::move_input::output(stick, MAPPER_A);
        let raw = MapperButtonOverlay::default().apply_raw(ButtonTrace {
            held: MAPPER_A | MAPPER_LEFT_STICK_DIRECTIONS | MAPPER_RIGHT_STICK_DIRECTIONS,
            ..Default::default()
        }, out.requested, out.owned);
        assert_eq!(raw.held, MAPPER_A);
        assert_eq!(out.stick, Some(stick));
    }
}

#[test]
fn spacing_forward_tilt_never_exceeds_walk_cap_or_falls_back_to_jab() {
    use kirby_assist_core::{auto_spacing::{Attack, AttackConfig, Config, Spacing},
        native_states::{situation, status}};
    for side in [-1., 1.] {
        for cap in [None, Some(0.), Some(0.4), Some(0.69)] {
            let c = GameplayContext { gameplay: true, can_control: true, can_act: true,
                situation: situation::GROUND, status: status::WAIT, facing: side,
                walk_stick: cap, safe_ground: [true; 2], ..Default::default() };
            let cfg = Config { ground: AttackConfig { attack: Attack::Forward,
                startup: 8., active_frames: 5., reach: 20., height: 9., commitment: 33.,
            }, ..Default::default() };
            let mut spacing = Spacing::default();
            for tick in 1..=3 {
                let out = spacing.sample(tick, true, cfg, c, Some(Opponent {
                    position: [side * 18., 0.], vulnerable: true, ..Default::default()
                }));
                let safe = cap.unwrap_or(0.);
                assert_eq!(out.attack, safe > 0. && tick != 2);
                assert_eq!(out.controller.requested, if out.attack { MAPPER_A } else { 0 });
                assert_eq!(out.right_stick, [0.; 2]);
                if out.attack {
                    assert_eq!(out.controller.stick, Some([side * safe.min(0.6), 0.]));
                    spacing.mapped(tick);
                } else if safe > 0. {
                    assert_eq!(out.controller.stick, Some([0.; 2]));
                }
            }
        }
    }
}

#[test]
fn lucina_spacing_down_tilt_faces_enemy_and_releases_cstick_between_retries() {
    use kirby_assist_core::{auto_spacing::{Attack, AttackConfig, Config, Spacing},
        fighter_kinds::LUCINA, native_states::{situation, status}};
    let cfg = Config { fighter_kind: LUCINA, ground: AttackConfig {
        attack: Attack::Down, startup: 7., active_frames: 2., reach: 20., height: 9., commitment: 25.,
    }, ..Default::default() };
    for side in [-1., 1.] {
        let c = GameplayContext { gameplay: true, can_control: true, can_act: true,
            situation: situation::GROUND, status: status::WAIT, facing: -side,
            walk_stick: Some(0.69), safe_ground: [true; 2], ..Default::default() };
        let target = Some(Opponent { position: [side * 18., 0.], vulnerable: true, ..Default::default() });
        let mut spacing = Spacing::default();
        let turn = spacing.sample_threat(0, true, cfg, c, target, true);
        assert!(!turn.attack);
        assert_eq!(turn.controller.stick, Some([side * 0.6, 0.]));
        for tick in 1..=3 {
            let out = spacing.sample_threat(tick, true, cfg,
                GameplayContext { facing: side, ..c }, target, true);
            assert_eq!(out.attack, tick != 2);
            assert_eq!(out.right_stick, if out.attack { [side * 0.2, -1.] } else { [0.; 2] });
            let (buttons, left, right) = mapped_spacing_input(MAPPED_JUMP, out.attack,
                out.controller.stick.unwrap(), out.right_stick);
            assert_eq!(left, if out.attack { [side * 0.2, -0.6] } else { [0.; 2] });
            assert_eq!(right, [0.; 2]);
            assert_eq!(buttons, MAPPED_JUMP | if out.attack { MAPPED_ATTACK_HOLD_BITS } else { 0 });
            if out.attack { spacing.mapped(tick); }
        }
    }
}

#[test]
fn spacing_brakes_native_dash_in_attack_range_without_accidental_dash_attack() {
    use kirby_assist_core::{auto_spacing::{Attack, AttackConfig, Config, Spacing},
        native_states::{situation, status}};
    let cfg = Config { ground: AttackConfig { attack: Attack::Forward,
        startup: 8., active_frames: 5., reach: 20., height: 9., commitment: 33.,
    }, ..Default::default() };
    for native in [status::DASH, status::TURN_DASH, status::RUN, status::TURN_RUN,
        status::RUN_BRAKE, status::TURN_RUN_BRAKE] {
        let c = GameplayContext { gameplay: true, can_control: true, can_act: true,
            situation: situation::GROUND, status: native, facing: 1.,
            walk_stick: Some(0.69), safe_ground: [true; 2], ..Default::default() };
        let enemy = Some(Opponent { position: [18., 0.], vulnerable: true, ..Default::default() });
        let mut spacing = Spacing::default();
        for target in [enemy, None] {
            let out = spacing.sample(1, true, cfg, c, target);
            assert_eq!(out.controller.stick, target.map(|_| [0.; 2]));
            assert_eq!(out.walk_cap, Some(0.49));
            assert_eq!(out.controller.requested, 0);
            assert_eq!(out.right_stick, [0.; 2]);
            assert!(!out.attack);
            spacing.mapped(1);
        }
        let out = spacing.sample(2, true, cfg, GameplayContext { status: status::WAIT, ..c }, enemy);
        assert!(out.attack);
        assert_eq!(out.controller.stick, Some([0.6, 0.]));
    }
}

#[test]
fn spacing_ground_retry_cancels_old_direction_when_target_crosses() {
    use kirby_assist_core::{auto_spacing::{Attack, AttackConfig, Config, Spacing},
        native_states::{situation, status}};
    let cfg = Config { ground: AttackConfig { attack: Attack::Forward,
        startup: 8., active_frames: 5., reach: 20., height: 9., commitment: 33.,
    }, ..Default::default() };
    let c = GameplayContext { gameplay: true, can_control: true, can_act: true,
        situation: situation::GROUND, status: status::WAIT, facing: 1.,
        walk_stick: Some(0.69), safe_ground: [true; 2], ..Default::default() };
    let mut spacing = Spacing::default();
    assert!(spacing.sample(1, true, cfg, c, Some(Opponent {
        position: [18., 0.], vulnerable: true, ..Default::default()
    })).attack);
    spacing.mapped(1);
    for tick in 2..=3 {
        let out = spacing.sample(tick, true, cfg, c, Some(Opponent {
            position: [-18., 0.], vulnerable: true, ..Default::default()
        }));
        assert!(!out.attack);
        assert_eq!(out.controller.requested, 0);
    }
}

#[test]
fn ended_guard_releases_only_its_request_and_never_overwrites_the_next_owner() {
    use kirby_assist_core::autopilot_guard::{GuardClock, GuardHold};
    let clock = GuardClock {
        status: kirby_assist_core::native_states::GUARD, motion: 1,
        frame: 0., rate: 0., stopped: false,
    };
    for manual in [0, MAPPER_ZR] {
        let mut guard = GuardHold::default();
        let mut overlay = ManualOverlay::default();
        let human = ButtonTrace { held: manual, previous: manual, ..Default::default() };
        assert!(guard.sample_with_progress(true, true, clock, 4., None));
        overlay.apply_controls(human, [0.; 2], TurboOutput::default(), 0, MAPPER_ZR, SEQUENCE_BUTTONS);
        assert!(!guard.sample_with_progress(true, false, clock, 4., None));
        let released = overlay.apply_controls(human, [0.; 2], TurboOutput::default(), 0, 0, 0);
        assert_eq!(released.buttons.held & MAPPER_ZR, manual);
        for next in [MAPPER_A, MAPPER_B, MAPPER_ZR] {
            assert!(!guard.sample_with_progress(false, true, clock, 4., None));
            let out = overlay.apply_controls(human, [0.; 2], TurboOutput::default(), 0, next, SEQUENCE_BUTTONS);
            assert_eq!(out.buttons.held & (MAPPER_A | MAPPER_B | MAPPER_ZR), next);
        }
    }
}

#[test]
fn counter_down_special_and_retry_release_survive_held_physical_inputs() {
    use kirby_assist_core::{auto_counter::{Config, Context, Counter, Threat},
        fighter_kinds::LUCINA, native_states::{status, situation}, timing::AnimationSample};
    let mut policy = Counter::new();
    let mut overlay = ManualOverlay::default();
    let config = Config { startup: 5., input_delay: 0., margin: 2.,
        half_width: 5., height: 16., half_depth: 5. };
    let mut ctx = Context { kind: LUCINA, situation: situation::GROUND,
        can_special: true, stopped: false, animation: AnimationSample {
            object: 1, status: status::RUN, kind: 1, frame: 1., auxiliary: 0.,
        }};
    let threat = Threat { id: 7, motion: 9, status: status::ATTACK,
        frame: 3., in_frames: 6. };
    for step in 0..4 {
        ctx.animation.frame = step as f32 + 1.;
        let selected = policy.sample(true, Some(config), ctx, Some(threat));
        if step == 1 { policy.poll_receipt(2); continue; }
        let raw = overlay.apply_reaction_controls(
            ButtonTrace { held: GAMEPLAY_BUTTONS | LEFT_STICK_DIRECTIONS,
                previous: GAMEPLAY_BUTTONS | LEFT_STICK_DIRECTIONS,
                ..Default::default() },
            [1., 1.], TurboOutput { stick: selected.stick, ..Default::default() },
            sequence_flick_consumption(selected.owned, false), selected.requested,
            selected.owned, true,
        );
        assert_eq!(raw.buttons.held & (GAMEPLAY_BUTTONS & !MAPPER_B), 0);
        if step == 2 {
            assert_eq!(raw.buttons.held & MAPPER_B, 0);
            policy.mapped_release();
        } else {
            assert_ne!(raw.buttons.held & MAPPER_B, 0);
            let stick = exclusive_sequence_stick(selected.owned, selected.stick).unwrap();
            assert_eq!(mapped_stick_axis(stick[0]), 0);
            assert_eq!(mapped_stick_axis(stick[1]), -127);
            policy.mapped();
            policy.poll_receipt(1);
        }
    }
}
#[test]
fn shared_air_defense_delivers_neutral_guard_once_then_releases_owned_inputs() {
    use kirby_assist_core::{action_owner::{exclusive_output, Owner},
        defense_response::{Defense, Context, Contact}, native_states::situation};
    let mut defense = Defense::new();
    let mut overlay = ManualOverlay::default();
    let c = Context { object: 1, situation: situation::AIR, allowed: true,
        air_dodge_ready: true, ..Default::default() };
    let contact = Some(Contact { source: 2, in_frames: 3., projectile: true });
    for tick in 0..3 {
        let held = tick != 2;
        let output = defense.sample(tick, held, c, contact, 4., 1.);
        let selected = exclusive_output(held.then_some(Owner::Defense), held, output);
        let raw = overlay.apply_reaction_controls(
            ButtonTrace { held: MAPPER_A | LEFT_STICK_DIRECTIONS,
                previous: MAPPER_A | LEFT_STICK_DIRECTIONS, ..Default::default() },
            [1., -1.], TurboOutput { stick: selected.stick, ..Default::default() },
            sequence_flick_consumption(selected.owned, false), selected.requested,
            selected.owned, true,
        );
        if tick == 0 {
            assert_eq!(raw.buttons.held & MAPPER_A, 0);
            assert_eq!(selected.stick, Some([0.; 2]));
            let mapped = mapped_literal_buttons(MAPPED_ATTACK | MAPPED_SPECIAL | MAPPED_GUARD,
                selected.requested);
            assert_ne!(mapped & MAPPED_GUARD, 0);
            assert_eq!(mapped & (MAPPED_ATTACK | MAPPED_SPECIAL), 0);
            defense.mapped_air_dodge();
        } else {
            assert_eq!(selected.owned, if held { SEQUENCE_GUARD_INPUTS } else { 0 });
            assert_eq!(selected.stick, None);
            assert_eq!(raw.stick, [1., -1.]);
            assert_eq!(raw.buttons.held & MAPPER_ZR, 0);
            assert_ne!(raw.buttons.held & MAPPER_A, 0);
        }
    }
}

#[test]
fn shared_defense_neutral_takeover_blocks_all_physical_actions_in_both_layouts() {
    const MAPPED_CSTICK_BUTTON: u32 = 0x80;
    use kirby_assist_core::{defense_response::{Defense, Context, Contact},
        action_owner::{exclusive_output, Owner}, native_states::situation};
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for where_ in [situation::GROUND, situation::AIR] {
            for stick in [[-1., -1.], [1., 1.], [0., -1.], [0., 0.]] {
                let c = Context { object: 1, situation: where_, allowed: true,
                    guard_ready: true, air_dodge_ready: true, ..Default::default() };
                let output = Defense::new().sample(0, true, c,
                    Some(Contact { source: 2, in_frames: 0., projectile: false }), 4., 0.);
                let selected = exclusive_output(Some(Owner::Defense), true, output);
                let held = layout.policy_buttons(GAMEPLAY_BUTTONS | MAPPER_RIGHT_STICK_DIRECTIONS);
                let raw = ManualOverlay::default().apply_reaction_controls(
                    ButtonTrace { held, previous: held, pressed: held, ..Default::default() },
                    stick, TurboOutput { stick: selected.stick, ..Default::default() },
                    0, selected.requested, selected.owned, true);
                assert_eq!(raw.stick, [0.; 2]);
                assert_eq!(raw.buttons.held & GAMEPLAY_BUTTONS, MAPPER_ZR);
                let mapped = mapped_literal_buttons(mapped_automatic_actions(
                    MAPPED_ATTACK_HOLD_BITS | MAPPED_SPECIAL | MAPPED_JUMP | MAPPED_JUMP_MINI
                        | MAPPED_CATCH | MAPPED_CSTICK_BUTTON | MAPPED_GUARD_HOLD,
                    selected.requested), selected.requested);
                let mapped = mapped_sequence_cstick(mapped, [0.; 2]);
                assert_eq!(mapped & (MAPPED_ATTACK_HOLD_BITS | MAPPED_SPECIAL | MAPPED_JUMP
                    | MAPPED_JUMP_MINI | MAPPED_CATCH | MAPPED_CSTICK_BUTTON), 0);
                assert_ne!(mapped & MAPPED_GUARD, 0);
            }
        }
    }
}

fn tape(actions: &str) -> InputTimeline {
    let catalog = Catalog::parse(&format!(
        "[bowser.roll.test]\nstart_delay=0\nactions={actions}\n"
    ))
    .unwrap();
    InputTimeline::new(
        catalog.entries()[0].sequence,
        Opponent {
            id: 2,
            position: [10., 0.],
            ..Default::default()
        },
    )
}
fn observed_target() -> Option<Opponent> {
    Some(Opponent {
        id: 2,
        position: [10., 0.],
        ..Default::default()
    })
}
fn context(frame: f32) -> GameplayContext {
    GameplayContext {
        held: true,
        gameplay: true,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.,
        facing: 1.,
        safe_ground: [true; 2],
        ..Default::default()
    }
}
#[test]
fn full_drift_keeps_the_dash_edge_and_exact_strength_while_walk_suppresses_it() {
    for actions in [
        "drift:opponent:1:1, up_b:4",
        "run:opponent:10",
        "chase_if_far:5:1:14",
        "walk:opponent:10:50%",
    ] {
        let mut t = tape(actions);
        let out = t.sample(
            AssistSettings::default(),
            context(0.),
            observed_target(),
            None,
            0.,
        );
        let consumption = sequence_flick_consumption(out.owned, t.walking());
        let raw = ManualOverlay::default().apply_controls(
            ButtonTrace {
                held: GAMEPLAY_BUTTONS | LEFT_STICK_DIRECTIONS,
                previous: GAMEPLAY_BUTTONS | LEFT_STICK_DIRECTIONS,
                ..Default::default()
            },
            [-1., -1.],
            TurboOutput {
                stick: out.stick,
                ..Default::default()
            },
            consumption,
            out.requested,
            out.owned,
        );
        let walking = actions.starts_with("walk:");
        assert_eq!(raw.buttons.held & (1 << 21) != 0, !walking, "{actions}");
        let exact = exclusive_sequence_stick(out.owned, out.stick).unwrap();
        assert_eq!(
            mapped_stick_axis(exact[0]),
            if walking { 63 } else { 127 },
            "{actions}"
        );
        assert_eq!(
            raw.buttons.held & (GAMEPLAY_BUTTONS & !out.requested),
            0,
            "physical attacks/guard must be suppressed"
        );
        assert_eq!(raw.stick, exact);
    }
}
#[test]
fn a_one_frame_dash_advances_directly_to_up_b_without_manual_guard_or_jump() {
    let mut t = tape("drift:opponent:1:1, up_b:4, drift:opponent:1:65");
    let a = t.sample(
        AssistSettings::default(),
        context(0.),
        observed_target(),
        None,
        0.,
    );
    assert_eq!(a.stick, Some([1., 0.]));
    let b = t.sample(
        AssistSettings::default(),
        context(1.),
        observed_target(),
        None,
        0.,
    );
    assert_eq!(b.requested, MAPPER_B);
    assert_eq!(b.stick, Some([0., 1.]));
    let raw = ManualOverlay::default().apply_controls(
        ButtonTrace {
            held: GAMEPLAY_BUTTONS,
            pressed: GAMEPLAY_BUTTONS,
            ..Default::default()
        },
        [-1., -1.],
        TurboOutput {
            stick: b.stick,
            ..Default::default()
        },
        0,
        b.requested,
        b.owned,
    );
    assert_eq!(raw.buttons.held & GAMEPLAY_BUTTONS, MAPPER_B);
    assert_eq!(
        mapped_sequence_cstick(0x2 | 0x800 | 0x80, [0.; 2]),
        0x2 | 0x800
    );
    assert_eq!(exclusive_sequence_stick(b.owned, b.stick), Some([0., 1.]));
    let c = t.sample(
        AssistSettings::default(),
        context(2.),
        observed_target(),
        None,
        0.,
    );
    assert_eq!(c.requested, MAPPER_B);
}
#[test]
fn trigger_click_aliases_and_axes_cannot_leak_through_full_or_partial_takeovers() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        let mut overlay = ManualOverlay::default();
        let raw = overlay.apply_controls(
            ButtonTrace {
                held: GAMEPLAY_BUTTONS,
                previous: GAMEPLAY_BUTTONS,
                pressed: GAMEPLAY_BUTTONS,
                ..Default::default()
            },
            [1., 1.],
            TurboOutput {
                stick: Some([1., 0.]),
                ..Default::default()
            },
            0,
            0,
            layout.physical_owned_buttons(SEQUENCE_BUTTONS),
        );
        assert_eq!(raw.buttons.held & GAMEPLAY_BUTTONS, 0);
        assert_eq!(raw.buttons.pressed & GAMEPLAY_BUTTONS, 0);
        assert_eq!(
            owned_trigger_axes([0.9, 0.8], raw.clear_autorepeat),
            [0., 0.]
        );
    }
    let raw = ManualOverlay::default().apply_controls(
        ButtonTrace {
            held: MAPPER_L | MAPPER_DIGITAL_L | MAPPER_R | MAPPER_DIGITAL_R,
            ..Default::default()
        },
        [0.; 2],
        TurboOutput::default(),
        MAPPER_L,
        0,
        0,
    );
    assert_eq!(raw.buttons.held, MAPPER_R | MAPPER_DIGITAL_R);
    assert_eq!(
        owned_trigger_axes([0.9, 0.8], raw.clear_autorepeat),
        [0., 0.8]
    );
    assert_eq!(owned_trigger_axes([0.9, 0.8], 0), [0.9, 0.8]);
    assert_eq!(exclusive_sequence_stick(MAPPER_X, Some([1., 0.])), None);
}
#[test]
fn explicit_shield_jump_and_cstick_commands_survive_the_final_exclusive_filter() {
    let all = 0x1ffff;
    assert_eq!(mapped_sequence_cstick(all, [1., 0.]), all);
    assert_eq!(mapped_sequence_cstick(0x10008, [0.; 2]), 0x10008);
    assert_eq!(mapped_sequence_cstick(0x8044, [0.; 2]), 0x8044);
    // A legitimate native grab result must retain its Guard/Attack components.
    assert_eq!(mapped_sequence_cstick(0x219, [0.; 2]), 0x219);
    // Recipe ZL is Jump; the native Jump result must survive without X/Y.
    assert_eq!(SEQUENCE_JUMP_BUTTONS & MAPPER_ZL, MAPPER_ZL);
    assert_eq!(SEQUENCE_GUARD_BUTTONS & MAPPER_ZL, 0);
}

#[test]
fn auto_react_rebases_a_crossed_one_frame_step_without_shortening_the_next_input() {
    let mut t = tape("drift:opponent:1:1, up_b:4, drift:opponent:1:65");
    t.allow_continuous_progress();
    assert_eq!(
        t.sample(
            AssistSettings::default(),
            context(0.),
            observed_target(),
            None,
            0.
        )
        .stick,
        Some([1., 0.])
    );
    for frame in [2., 3., 4., 5.] {
        let out = t.sample(
            AssistSettings::default(),
            context(frame),
            observed_target(),
            None,
            0.,
        );
        assert_eq!(
            out.requested, MAPPER_B,
            "Up-B must retain four complete input frames"
        );
        assert_eq!(out.stick, Some([0., 1.]));
    }
    let out = t.sample(
        AssistSettings::default(),
        context(6.),
        observed_target(),
        None,
        0.,
    );
    assert_eq!(out.requested, 0);
    assert_eq!(out.stick, Some([1., 0.]));
    assert!(!t.finished());
    // Ledge/button tapes keep strict gap handling unless explicitly opted in.
    let mut strict = tape("drift:opponent:1:1, up_b:4");
    strict.sample(
        AssistSettings::default(),
        context(0.),
        observed_target(),
        None,
        0.,
    );
    strict.sample(
        AssistSettings::default(),
        context(2.),
        observed_target(),
        None,
        0.,
    );
    assert!(strict.finished());
}

#[test]
fn a_saved_y_dash_grab_uses_native_r_even_when_r_has_a_custom_binding() {
    use kirby_assist_core::{
        button_bindings::Builtin,
        fighter_profiles::{presets::PresetSlot, FighterProfiles},
        settings_menu::MenuSelection,
    };
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for builtin in Builtin::ALL {
            let source=format!("[bowser.y.grab]\ndefault=true\nstart_delay=0\nactions=check:enemy_within:45:0,drift:opponent:1:1,opponent_input:r:0:0:1\n[bowser.r.rebound]\ndefault=true\nbuiltin={}\n",builtin.token());
            let cat = Catalog::parse(&source).unwrap();
            let mut profiles = FighterProfiles::default();
            profiles.add_sequence_choices(&cat).unwrap();
            let p = &profiles.profile(15).unwrap().menu_options;
            let mut menu = MenuSelection::new(p);
            menu.set_binding_input(0, layout);
            menu.input_with_editing(0, p, false);
            menu.set_binding_input(MAPPER_Y, layout);
            let consumed = menu.input_with_editing(layout.policy_buttons(MAPPER_Y), p, false);
            assert_eq!(menu.preview_slot, Some(PresetSlot::Y));
            let selected = menu
                .choice(p, PresetSlot::Y, menu.choices[PresetSlot::Y as usize])
                .unwrap()
                .sequence;
            let e = Opponent {
                id: 2,
                position: [10., 0.],
                ..Default::default()
            };
            let mut t = InputTimeline::new(selected, e);
            let dash = t.sample(AssistSettings::default(), context(0.), Some(e), None, 0.);
            assert_eq!(dash.stick, Some([1., 0.]));
            assert_eq!(dash.requested, 0);
            let mut overlay = ManualOverlay::default();
            let human = ButtonTrace {
                held: MAPPER_Y,
                pressed: MAPPER_Y,
                ..Default::default()
            };
            overlay.apply_controls(
                human,
                [0.; 2],
                TurboOutput {
                    stick: dash.stick,
                    ..Default::default()
                },
                layout.physical_buttons(consumed),
                layout.physical_buttons(dash.requested),
                layout.physical_owned_buttons(dash.owned),
            );
            let grab = t.sample(AssistSettings::default(), context(1.), Some(e), None, 0.);
            assert_eq!(grab.requested, MAPPER_R);
            let out = overlay.apply_controls(
                human,
                [0.; 2],
                TurboOutput {
                    stick: grab.stick,
                    ..Default::default()
                },
                layout.physical_buttons(consumed),
                layout.physical_buttons(grab.requested),
                layout.physical_owned_buttons(grab.owned),
            );
            let native_grab = layout.physical_buttons(MAPPER_R);
            assert_eq!(
                out.buttons.held & GAMEPLAY_BUTTONS,
                native_grab,
                "{layout:?} {builtin:?}"
            );
            assert_eq!(out.buttons.pressed & native_grab, native_grab);
            assert_eq!(
                menu.preview_slot,
                Some(PresetSlot::Y),
                "generated R never runs its physical binding"
            );
            assert_eq!(
                mapped_sequence_cstick(0x219, [0.; 2]),
                0x219,
                "native grab does not turn into a jab"
            );
        }
    }
}
#[test]
fn shield_handback_never_quarantines_the_canonical_zl_jump_button() {
    use kirby_assist_core::manual_controls::AssistGuardHandoff;
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        let guard = layout.physical_buttons(SEQUENCE_GUARD_BUTTONS);
        let jump = layout.physical_buttons(MAPPER_ZL);
        let human = ButtonTrace {
            held: guard | jump,
            previous: guard | jump,
            ..Default::default()
        };
        let mut handback = AssistGuardHandoff::new();
        assert_eq!(
            handback.sample(true, human, guard, GAMEPLAY_BUTTONS, 0),
            guard
        );
        let blocked = handback.sample(false, human, guard, 0, 0);
        assert_eq!(blocked, guard);
        assert_eq!(blocked & jump, 0);
    }
}

#[test]
fn admission_escape_does_not_cancel_a_committed_owner_and_explicit_cancellation_releases() {
    use kirby_assist_core::action_owner::{ActionOwnership, Intent, Owner};
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        let jumps = if layout == ControlLayout::Default {
            MAPPER_ZL
        } else {
            MAPPER_R | MAPPER_ZR
        };
        let shield = layout.physical_buttons(MAPPER_ZR);
        assert!(automatic_escape(layout, jumps, 0, true, true, MAPPER_B));
        assert!(automatic_escape(
            layout, shield, shield, true, false, MAPPER_B
        ));
        assert!(
            !automatic_escape(layout, shield, 0, true, true, MAPPER_B),
            "held shield still permits OOS reactions"
        );
        assert!(
            !automatic_escape(layout, shield, shield, false, true, 0),
            "ordinary shield does not disable automatic grabs"
        );
        assert!(
            !automatic_escape(layout, shield, shield, true, true, MAPPER_B),
            "fresh shield cannot cancel the grounded Up-B input"
        );
        assert!(
            automatic_escape(layout, shield, shield, true, true, 0),
            "fresh shield can leave follow-through"
        );
        let mut owner = ActionOwnership::default();
        owner.select(
            true,
            &[Intent::input_sequence(
                Owner::RollRead,
                kirby_assist_core::ledge_trap::SEQUENCE_BUTTONS,
            )],
        );
        owner.set_automatic_suspended(automatic_escape(layout, jumps, jumps, true, true, MAPPER_B));
        assert!(owner.allows(Owner::RollRead));
        assert!(!owner.allows(Owner::OutOfShield));
        owner.release(Owner::RollRead);
        assert!(
            !owner.allows(Owner::RollRead),
            "suspension still blocks new admission after explicit cancellation"
        );
        let mut overlay = ManualOverlay::default();
        overlay.apply_controls(
            ButtonTrace::default(),
            [0.; 2],
            TurboOutput::default(),
            0,
            layout.physical_buttons(MAPPER_B),
            GAMEPLAY_BUTTONS,
        );
        let out = overlay.apply_controls(
            ButtonTrace {
                held: jumps,
                pressed: jumps,
                ..Default::default()
            },
            [0.; 2],
            TurboOutput::default(),
            0,
            0,
            0,
        );
        assert_eq!(out.buttons.held & jumps, jumps);
        assert_eq!(out.buttons.held & layout.physical_buttons(MAPPER_B), 0);
        let mut ledge = ActionOwnership::default();
        ledge.select(
            true,
            &[Intent::input_sequence(
                Owner::Ledge,
                kirby_assist_core::ledge_trap::SEQUENCE_BUTTONS,
            )],
        );
        ledge.set_automatic_suspended(true);
        assert_eq!(ledge.current_owner(), Some(Owner::Ledge));
        assert!(
            ledge.allows(Owner::Safety),
            "native safety may preempt a committed ledge tape"
        );
        ledge.release(Owner::Ledge);
        assert!(
            ledge.allows(Owner::Safety),
            "suspension does not block safety after release"
        );
    }
}

#[test]
fn up_b_mapper_filter_removes_unrequested_jump_guard_but_preserves_native_grab_and_jump() {
    let native = 0x1 | 0x2 | 0x4 | 0x8 | 0x40 | 0x2000;
    assert_eq!(
        mapped_automatic_actions(native, MAPPER_B),
        0x1 | 0x2 | 0x2000,
        "Up-B must not tap-jump or add shield"
    );
    assert_eq!(
        mapped_automatic_actions(native, 0),
        0x1 | 0x2 | 0x2000,
        "drift must not roll from stale Guard"
    );
    for jump in [MAPPER_X, MAPPER_Y, MAPPER_ZL] {
        assert_eq!(
            mapped_automatic_actions(native, jump) & (0x4 | 0x40),
            0x4 | 0x40
        );
    }
    for guard in [MAPPER_R, MAPPER_ZR] {
        assert_eq!(
            mapped_automatic_actions(native, guard) & (0x8 | 0x1),
            0x8 | 0x1,
            "grab retains Guard plus Attack"
        );
    }
    assert_eq!(
        mapped_automatic_actions(native, MAPPER_B | MAPPER_ZR) & 0x8,
        0x8,
        "direct OOS guard is explicit"
    );
}

#[test]
fn a_recent_but_previous_update_packet_cannot_replay_a_dash_grab_or_special() {
    use kirby_assist_core::observation::input_update_current;
    for actions in ["up_b:4", "side_b:4", "grab:4", "drift:opponent:1:4"] {
        let mut timeline = tape(actions);
        let proposed = timeline.sample(
            AssistSettings::default(),
            context(0.),
            observed_target(),
            None,
            0.,
        );
        let mut overlay = ManualOverlay::default();
        let physical = ButtonTrace::default();
        let first = overlay.apply_controls(
            physical,
            [-0.75, 0.],
            TurboOutput {
                stick: proposed.stick,
                ..Default::default()
            },
            0,
            proposed.requested,
            proposed.owned,
        );
        assert_ne!(first.stick, [-0.75, 0.]);
        // A blocked/missing refresh must release the old packet even inside its
        // former 35ms lifetime. Human movement is returned on this same update.
        let selected = if input_update_current(Some(8), 9) {
            proposed
        } else {
            Default::default()
        };
        let handback = overlay.apply_controls(
            physical,
            [-0.75, 0.],
            TurboOutput {
                stick: selected.stick,
                ..Default::default()
            },
            0,
            selected.requested,
            selected.owned,
        );
        assert_eq!(handback.buttons.held & GAMEPLAY_BUTTONS, 0, "{actions}");
        assert_eq!(handback.buttons.pressed & GAMEPLAY_BUTTONS, 0, "{actions}");
        assert_eq!(handback.stick, [-0.75, 0.], "{actions}");
        assert_eq!(
            handback.buttons.released & proposed.requested,
            proposed.requested
        );
        // Same-update duplicate reads may reuse the packet; older, future,
        // absent and wrapped counter values cannot be relabelled as current.
        assert!(input_update_current(Some(9), 9));
        assert!(!input_update_current(None, 9));
        assert!(!input_update_current(Some(10), 9));
        assert!(!input_update_current(Some(i64::MAX), i64::MIN));
    }
}

#[test]
fn guard_release_clears_hold_history_without_losing_attack_or_jump() {
    let buttons = MAPPED_GUARD | MAPPED_GUARD_HOLD | MAPPED_ATTACK_HOLD_BITS | MAPPED_JUMP;
    assert_eq!(mapped_guard_release(buttons), MAPPED_ATTACK_HOLD_BITS | MAPPED_JUMP);
}
