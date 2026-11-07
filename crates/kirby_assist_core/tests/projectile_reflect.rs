use kirby_assist_core::{
    action_owner::{ActionOwnership, Intent, Owner},
    autopilot_guard::duck_geometry::Hitbox,
    fighter_profiles::{presets::MenuPresets, FighterProfiles},
    fighter_tuning::{self, Tuning},
    ledge_menu::{MenuRow, MenuSelection},
    native_input::MAPPER_B,
    projectile_reflect::*,
    timing::AnimationSample,
};
fn values() -> Tuning {
    let mut t = Tuning::default();
    t.set("reflect_default", "true").unwrap();
    t
}
fn config() -> Config {
    Config::for_fighter(GAMEWATCH, values()).unwrap()
}
fn own(frame: f32, status: i32) -> Context {
    Context {
        kind: GAMEWATCH,
        situation: 0,
        facing: 1.,
        can_special: true,
        gauge: 0.,
        stopped: false,
        animation: AnimationSample {
            object: 1,
            status,
            kind: 1,
            frame,
            auxiliary: 0.,
        },
    }
}
fn threat(id: u32, frames: f32, direction: f32) -> Option<Threat> {
    Some(Threat {
        id,
        generation: 0,
        in_frames: frames,
        direction,
    })
}
fn hit(x: f32, y: f32, vx: f32, vy: f32) -> Hitbox {
    Hitbox {
        start: [x, y, 0.],
        end: [x, y, 0.],
        radius: 1.,
        velocity: [vx, vy],
    }
}

#[test]
fn only_explicit_supported_fighter_configuration_enables_projectile_defense() {
    let profiles = FighterProfiles::default();
    for kind in 0..kirby_assist_core::fighter_kinds::PLAYABLE_KIND_COUNT {
        if profiles.profile(kind).is_none() {
            continue;
        }
        assert!(Config::for_fighter(kind, Tuning::default()).is_none());
        assert_eq!(
            Config::for_fighter(kind, values()).is_some(),
            kind == GAMEWATCH
        );
        assert_eq!(
            Tuning::visible(kind, fighter_tuning::REFLECT_STARTUP),
            kind == GAMEWATCH
        );
        let mut menu = MenuSelection::new(&MenuPresets::default());
        menu.y_fighter_kind = kind;
        menu.show_modes();
        assert!(!menu.row_visible(MenuRow::AutoReflect));
        menu.tuning = values();
        assert_eq!(menu.row_visible(MenuRow::AutoReflect), kind == GAMEWATCH);
    }
    let mut bad = values();
    assert!(bad.set("reflect_startup", "5").is_err());
    assert!(bad.set("reflect_max_hold", "121").is_err());
}

#[test]
fn training_assist_toggle_operates_without_changing_other_punishes() {
    use kirby_assist_core::manual_controls::DPAD_RIGHT;
    let presets = MenuPresets::default();
    let mut menu = MenuSelection::new(&presets);
    menu.y_fighter_kind = GAMEWATCH;
    menu.tuning = values();
    menu.show_modes();
    menu.row = MenuRow::AutoReflect;
    let other = (
        menu.out_of_shield,
        menu.grab_dodge,
        menu.roll_rules,
        menu.auto_tech,
    );
    menu.input(0, &presets);
    menu.input(DPAD_RIGHT, &presets);
    assert!(menu.auto_reflect);
    menu.input(0, &presets);
    menu.input(DPAD_RIGHT, &presets);
    assert!(!menu.auto_reflect);
    assert_eq!(
        (
            menu.out_of_shield,
            menu.grab_dodge,
            menu.roll_rules,
            menu.auto_tech
        ),
        other
    );
}

#[test]
fn swept_prediction_rejects_melee_outgoing_off_plane_and_invalid_geometry() {
    let cfg = config();
    let origin = [0.; 3];
    let velocity = [0.; 2];
    let p = predict(
        cfg,
        origin,
        velocity,
        1.,
        2,
        hit(41., 8., -5., 0.),
        true,
        false,
    )
    .unwrap();
    assert_eq!(p.in_frames, 7.);
    assert_eq!(p.direction, 1.);
    assert!(predict(
        cfg,
        origin,
        velocity,
        1.,
        2,
        hit(-41., 8., 5., 0.),
        true,
        false
    )
    .is_some());
    for (h, r, a) in [
        (hit(41., 8., -5., 0.), false, false),
        (hit(41., 8., 5., 0.), true, false),
        (hit(41., 40., -5., 0.), true, false),
        (hit(41., 8., f32::NAN, 0.), true, false),
    ] {
        assert!(predict(cfg, origin, velocity, 1., 2, h, r, a).is_none());
    }
    let mut off = hit(41., 8., -5., 0.);
    off.start[2] = 30.;
    off.end[2] = 30.;
    assert!(predict(cfg, origin, velocity, 1., 2, off, true, true).is_none());
    assert!(predict(
        cfg,
        origin,
        velocity,
        1.,
        2,
        hit(0., 52., 0., -5.),
        false,
        true
    )
    .is_some());
    // A player walking toward the shot closes the gap faster; no root-distance
    // threshold or coarse one-frame sampling may miss that contact.
    assert!(predict(
        cfg,
        origin,
        [2., 0.],
        1.,
        2,
        hit(55., 8., -5., 0.),
        true,
        false
    )
    .is_some());
    assert!(predict(
        cfg,
        origin,
        velocity,
        1.,
        2,
        hit(405., 8., -60., 0.),
        true,
        false
    )
    .is_some());
}

#[test]
fn first_legal_observation_emits_down_b_without_waiting_and_busy_states_emit_nothing() {
    for frames in [0., 5., 20., 100., f32::NAN] {
        let output =
            Reflector::new().sample(true, Some(config()), own(0., 3), threat(2, frames, 1.));
        assert_eq!(output.owned, 0);
        assert_eq!(output.stick, None);
    }
    for status in [
        kirby_assist_core::native_states::status::WAIT,
        kirby_assist_core::native_states::status::WALK,
        kirby_assist_core::native_states::status::DASH,
        kirby_assist_core::native_states::status::RUN,
        kirby_assist_core::native_states::status::RUN_BRAKE,
        kirby_assist_core::native_states::status::TURN_RUN_BRAKE,
        kirby_assist_core::native_states::status::SQUAT,
        kirby_assist_core::native_states::status::SQUAT_WAIT,
        kirby_assist_core::native_states::status::OTTOTTO,
    ] {
        let mut engine = Reflector::new();
        let output = engine.sample(true, Some(config()), own(0., status), threat(2, 7., 1.));
        assert_eq!(output.requested, MAPPER_B);
        assert_eq!(output.stick, Some([0., -1.]));
    }
    for status in [
        kirby_assist_core::native_states::status::GUARD_ON,
        kirby_assist_core::native_states::status::GUARD,
        kirby_assist_core::native_states::status::GUARD_OFF,
        kirby_assist_core::native_states::status::GUARD_DAMAGE,
        kirby_assist_core::native_states::status::ESCAPE,
        kirby_assist_core::native_states::status::ESCAPE_F,
        kirby_assist_core::native_states::status::ESCAPE_AIR,
        kirby_assist_core::native_states::status::ATTACK,
        kirby_assist_core::native_states::status::ATTACK_AIR,
        kirby_assist_core::native_states::status::CATCH,
        kirby_assist_core::native_states::status::CATCH_JUMP,
        kirby_assist_core::native_states::status::DAMAGE,
        kirby_assist_core::native_states::status::DOWN,
        kirby_assist_core::native_states::status::CLIFF_CATCH_MOVE,
        kirby_assist_core::native_states::status::DEAD,
        kirby_assist_core::native_states::status::SPECIAL_HI,
        kirby_assist_core::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_SHOOT,
    ] {
        let output =
            Reflector::new().sample(true, Some(config()), own(0., status), threat(2, 7., 1.));
        assert_eq!(output.owned, 0, "native status {status:#x}");
        assert_eq!(output.stick, None);
    }
    let mut c = own(0., 0);
    c.can_special = false;
    assert_eq!(
        Reflector::new()
            .sample(true, Some(config()), c, threat(2, 7., 1.))
            .owned,
        0
    );
}

#[test]
fn ground_turn_is_confirmed_before_bucket_and_air_does_not_fake_a_turn() {
    let mut engine = Reflector::new();
    let turn = engine.sample(true, Some(config()), own(0., 0), threat(2, 9., -1.));
    assert_eq!(turn.requested, 0);
    assert_eq!(turn.stick, Some([-0.6, 0.]));
    let c = Context {
        facing: -1.,
        ..own(1., 0x12)
    };
    assert_eq!(
        engine
            .sample(true, Some(config()), c, threat(2, 8., -1.))
            .requested,
        MAPPER_B
    );
    let air = Context {
        situation: 2,
        ..own(0., 0xe)
    };
    assert_eq!(
        Reflector::new()
            .sample(true, Some(config()), air, threat(2, 7., 1.))
            .requested,
        MAPPER_B
    );
    assert_eq!(
        Reflector::new()
            .sample(true, Some(config()), air, threat(2, 9., -1.))
            .owned,
        0
    );
}

#[test]
fn full_bucket_catch_reflect_and_changed_native_move_never_select_an_oil_attack() {
    for next in [
        kirby_assist_core::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_CATCH,
        kirby_assist_core::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_REFLECT,
        kirby_assist_core::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_SHOOT,
        kirby_assist_core::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_END,
        kirby_assist_core::native_states::status::DAMAGE,
        kirby_assist_core::native_states::status::CATCH,
        kirby_assist_core::native_states::status::SPECIAL_HI,
    ] {
        let mut engine = Reflector::new();
        engine.sample(true, Some(config()), own(0., 0), threat(2, 7., 1.));
        engine.mapped(1);
        assert_eq!(
            engine
                .sample(true, Some(config()), own(1., next), threat(2, 6., 1.))
                .requested,
            0
        );
        assert!(!engine.active());
    }
    for gauge in [3., 4., -1., f32::NAN] {
        let mut engine = Reflector::new();
        let c = Context {
            gauge,
            ..own(0., 0)
        };
        assert_eq!(
            engine
                .sample(true, Some(config()), c, threat(2, 7., 1.))
                .owned,
            0
        );
        assert!(!engine.active());
    }
    // An unrelated manual bucket is not evidence that our unselected input ran.
    let mut engine = Reflector::new();
    engine.sample(true, Some(config()), own(0., 0), threat(2, 7., 1.));
    assert_eq!(
        engine
            .sample(
                true,
                Some(config()),
                own(
                    1.,
                    kirby_assist_core::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_WAIT,
                ),
                threat(2, 6., 1.),
            )
            .owned,
        0
    );
}

#[test]
fn duplicated_polls_hitstop_failed_delivery_and_threat_loss_have_bounded_handoffs() {
    let mut engine = Reflector::new();
    let first = engine.sample(true, Some(config()), own(0., 0), threat(2, 7., 1.));
    for _ in 0..100 {
        assert_eq!(
            engine.sample(true, Some(config()), own(0., 0), threat(2, 7., 1.)),
            first
        );
    }
    engine.mapped(1);
    let stopped = Context {
        stopped: true,
        ..own(1., 0)
    };
    assert_eq!(engine.sample(true, Some(config()), stopped, None).owned, 0);
    assert!(engine.active());
    assert_eq!(
        engine
            .sample(
                true,
                Some(config()),
                own(
                    1.,
                    kirby_assist_core::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_WAIT_START,
                ),
                threat(2, 6., 1.),
            )
            .requested,
        MAPPER_B
    );
    assert_eq!(
        engine
            .sample(
                true,
                Some(config()),
                own(
                    2.,
                    kirby_assist_core::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_WAIT,
                ),
                None,
            )
            .owned,
        0
    );
    assert!(!engine.active());
    assert_eq!(
        engine
            .sample(true, Some(config()), own(3., 0), threat(2, 7., 1.))
            .owned,
        0,
        "never requeue a delivered stale projectile"
    );
    assert_eq!(
        engine
            .sample(true, Some(config()), own(4., 0), threat(3, 7., 1.))
            .requested,
        MAPPER_B
    );
    engine.cancel();
    for tick in 0..24 {
        engine.sample(true, Some(config()), own(4., 0), threat(4, 7., 1.));
        engine.mapped(tick);
    }
    assert_eq!(
        engine
            .sample(true, Some(config()), own(4., 0), threat(4, 7., 1.))
            .owned,
        0,
        "frozen startup delivery has an independent bound"
    );
}

#[test]
fn confirmed_bucket_has_a_finite_native_hold_and_never_retries_after_a_cancel() {
    let mut engine = Reflector::new();
    engine.sample(true, Some(config()), own(0., 0), threat(2, 7., 1.));
    engine.mapped(1);
    for frame in 1..=45 {
        assert_eq!(
            engine
                .sample(
                    true,
                    Some(config()),
                    own(
                        frame as f32,
                        kirby_assist_core::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_WAIT,
                    ),
                    threat(2, 1., 1.)
                )
                .requested,
            MAPPER_B
        );
    }
    assert_eq!(
        engine
            .sample(
                true,
                Some(config()),
                own(
                    46.,
                    kirby_assist_core::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_WAIT,
                ),
                threat(2, 1., 1.),
            )
            .owned,
        0
    );
    assert!(!engine.active());
    assert_eq!(
        engine
            .sample(true, Some(config()), own(47., 0), threat(2, 7., 1.))
            .owned,
        0
    );
    assert_eq!(
        engine
            .sample(false, Some(config()), own(48., 0), threat(3, 7., 1.))
            .owned,
        0
    );
    assert_eq!(
        engine
            .sample(true, Some(config()), own(49., 0), threat(3, 7., 1.))
            .requested,
        MAPPER_B,
        "unselected observations do not consume a target"
    );
}

#[test]
fn delayed_native_receipt_does_not_hold_movement_or_allow_a_competing_grab() {
    let mut engine = Reflector::new();
    engine.sample(true, Some(config()), own(0., 0), threat(2, 7., 1.));
    engine.mapped(0);
    for frame in 2..12 {
        let output = engine.sample(true, Some(config()), own(frame as f32, 0), None);
        assert_eq!(output.owned, 0);
        assert_eq!(output.stick, None);
        assert!(engine.active());
    }
    let confirmed = engine.sample(
        true,
        Some(config()),
        own(
            12.,
            kirby_assist_core::native_states::fighter_status::GAMEWATCH_SPECIAL_LW_WAIT_START,
        ),
        threat(2, 1., 1.),
    );
    assert_eq!(confirmed.requested, MAPPER_B);
    assert!(engine.active());
    engine = Reflector::new();
    engine.sample(true, Some(config()), own(0., 0), threat(3, 7., 1.));
    engine.mapped(0);
    for tick in 0..23 {
        engine.poll_receipt(tick);
        engine.poll_receipt(tick);
        assert!(engine.active());
    }
    engine.poll_receipt(23);
    assert!(
        !engine.active(),
        "a frozen native receipt cannot reserve input forever"
    );
    assert_eq!(
        engine
            .sample(true, Some(config()), own(1., 0), threat(3, 7., 1.))
            .owned,
        0
    );
}

#[test]
fn reflector_and_punishes_cannot_own_input_together_and_ledge_off_safety_win() {
    let input = |owner| Intent {
        owner,
        present: true,
        committed: true,
    };
    for current in [
        Owner::OutOfShield,
        Owner::RollRead,
        Owner::OptionRead,
        Owner::GrabDodge,
        Owner::Trip,
        Owner::JabLock,
        Owner::Shortcut,
        Owner::Ledge,
        Owner::Safety,
    ] {
        let mut ownership = ActionOwnership::new();
        ownership.select(true, &[input(current)]);
        assert!(!ownership.allows(Owner::Reflect));
        assert_eq!(
            ownership.select(true, &[input(current), input(Owner::Reflect)]),
            Some(current)
        );
    }
    let mut ownership = ActionOwnership::new();
    ownership.select(true, &[input(Owner::Reflect)]);
    for contender in [
        Owner::OutOfShield,
        Owner::RollRead,
        Owner::OptionRead,
        Owner::GrabDodge,
    ] {
        assert_eq!(
            ownership.select(true, &[input(Owner::Reflect), input(contender)]),
            Some(Owner::Reflect)
        );
    }
    assert_eq!(
        ownership.select(true, &[input(Owner::Reflect), input(Owner::Safety)]),
        Some(Owner::Safety)
    );
    ownership = ActionOwnership::new();
    ownership.select(true, &[input(Owner::Reflect)]);
    ownership.set_automatic_suspended(true);
    assert!(ownership.allows(Owner::Reflect));
    ownership.set_automatic_suspended(false);
    ownership.set_ledge_priority(true);
    assert!(ownership.allows(Owner::Reflect));
    assert_eq!(ownership.select(false, &[input(Owner::Reflect)]), None);
    #[cfg(feature = "diagnostic-logging")]
    assert_eq!(
        kirby_assist_core::diagnostics::modes::bit(Owner::Reflect),
        1 << 16
    );
}
