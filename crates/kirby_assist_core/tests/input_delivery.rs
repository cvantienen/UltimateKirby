//! Final controller delivery regressions shared by ledge/button/Auto react tapes.
use kirby_assist_core::{
    ledge_trap::{InputTimeline, LedgeContext, LedgeSettings, Opponent, SEQUENCE_BUTTONS},
    manual_controls::{
        ManualOverlay, ShortcutContext, Shortcuts, TurboOutput, LEFT_STICK_DIRECTIONS,
    },
    native_input::*,
    sequence_config::Catalog,
};
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
fn context(frame: f32) -> LedgeContext {
    LedgeContext {
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
        let out = t.sample(LedgeSettings::default(), context(0.), None, None, 0.);
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
    let a = t.sample(LedgeSettings::default(), context(0.), None, None, 0.);
    assert_eq!(a.stick, Some([1., 0.]));
    let b = t.sample(LedgeSettings::default(), context(1.), None, None, 0.);
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
    let c = t.sample(LedgeSettings::default(), context(2.), None, None, 0.);
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
fn native_jump_pulse_survives_consumption_of_its_remapped_physical_source() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for source in [
            MAPPER_L,
            MAPPER_R,
            MAPPER_ZL,
            MAPPER_ZR,
            MAPPER_X,
            MAPPER_Y,
            1,
            2,
            4,
            8,
            1 << 16,
            1 << 17,
        ] {
            let mut hop = Shortcuts::default();
            let out = hop.sample(
                MAPPER_X,
                true,
                Some(ShortcutContext {
                    status: 0,
                    situation: 0,
                    motion: 1,
                    frame: 10.,
                    rate: 1.,
                    stopped: false,
                    facing: 1.,
                    can_short_hop: true,
                    input_delay: 0.,
                    can_neutral_special: false,
                }),
            );
            let physical = source | physical_button_aliases(source);
            let requested = layout.physical_buttons(out.requested);
            let raw = ManualOverlay::default().apply_controls(
                ButtonTrace {
                    held: physical,
                    pressed: physical,
                    ..Default::default()
                },
                [0.; 2],
                TurboOutput::default(),
                layout.physical_buttons(layout.binding_ownership(source)),
                requested,
                layout.physical_owned_buttons(out.owned),
            );
            assert_eq!(
                raw.buttons.held & requested,
                requested,
                "{layout:?} {source:#x}"
            );
            assert_eq!(raw.buttons.pressed & requested, requested);
            assert_eq!(raw.buttons.held & (MAPPER_DIGITAL_L | MAPPER_DIGITAL_R), 0);
            assert_eq!(mapped_builtin_jump(0, Some(true), true), 0x44);
        }
    }
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
        t.sample(LedgeSettings::default(), context(0.), None, None, 0.)
            .stick,
        Some([1., 0.])
    );
    for frame in [2., 3., 4., 5.] {
        let out = t.sample(LedgeSettings::default(), context(frame), None, None, 0.);
        assert_eq!(
            out.requested, MAPPER_B,
            "Up-B must retain four complete input frames"
        );
        assert_eq!(out.stick, Some([0., 1.]));
    }
    let out = t.sample(LedgeSettings::default(), context(6.), None, None, 0.);
    assert_eq!(out.requested, 0);
    assert_eq!(out.stick, Some([1., 0.]));
    assert!(!t.finished());
    // Ledge/button tapes keep strict gap handling unless explicitly opted in.
    let mut strict = tape("drift:opponent:1:1, up_b:4");
    strict.sample(LedgeSettings::default(), context(0.), None, None, 0.);
    strict.sample(LedgeSettings::default(), context(2.), None, None, 0.);
    assert!(strict.finished());
}

#[test]
fn a_saved_y_dash_grab_uses_native_r_even_when_r_has_a_custom_binding() {
    use kirby_assist_core::{
        button_bindings::Builtin,
        fighter_profiles::{presets::PresetSlot, FighterProfiles},
        ledge_menu::MenuSelection,
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
            let dash = t.sample(LedgeSettings::default(), context(0.), Some(e), None, 0.);
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
            let grab = t.sample(LedgeSettings::default(), context(1.), Some(e), None, 0.);
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
        assert!(ledge.allows(Owner::Safety));
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
        let proposed = timeline.sample(LedgeSettings::default(), context(0.), None, None, 0.);
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
