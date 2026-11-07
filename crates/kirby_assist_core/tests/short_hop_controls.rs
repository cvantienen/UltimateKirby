use kirby_assist_core::{
    manual_controls::{ManualOverlay, ShortcutContext, Shortcuts, TurboOutput},
    native_input::{mapped_short_hop, ButtonTrace, MAPPER_X, MAPPER_ZL},
};

#[test]
fn horizontal_short_hop_never_injects_shield_and_releases_on_squat() {
    let ctx = ShortcutContext {
        status: kirby_assist_core::native_states::status::WAIT,
        situation: 0,
        motion: 1,
        frame: 0.,
        rate: 1.,
        stopped: false,
        facing: 1.,
        can_short_hop: true,
        input_delay: 0.,
        can_neutral_special: false,
    };
    for horizontal in [-1., 1.] {
        let mut shortcut = Shortcuts::default();
        shortcut.sample(0, true, Some(ctx));
        let hop = shortcut.sample(MAPPER_X, true, Some(ctx));
        assert_eq!(hop.requested, MAPPER_X);
        assert_eq!(hop.requested & MAPPER_ZL, 0);
        let applied = ManualOverlay::default().apply_controls(
            ButtonTrace {
                held: MAPPER_X,
                pressed: MAPPER_X,
                ..ButtonTrace::default()
            },
            [horizontal, 0.],
            TurboOutput::default(),
            shortcut.consumed_buttons(),
            hop.requested,
            hop.owned,
        );
        assert_eq!(applied.stick, [horizontal, 0.]);
        assert_eq!(applied.buttons.held & MAPPER_ZL, 0);
        // Native mapping reports Jump for X. The production post-map step
        // marks that jump as mini even if physical-release feedback is late.
        let mapped = mapped_short_hop(0x4, hop.requested & MAPPER_X != 0);
        assert_eq!(mapped & 0x44, 0x44);
        assert_eq!(mapped & (0x1 | 0x8), 0, "no attack or shield injected");
        let release = shortcut.sample(
            MAPPER_X,
            true,
            Some(ShortcutContext {
                status: kirby_assist_core::native_states::status::JUMP_SQUAT,
                motion: 2,
                ..ctx
            }),
        );
        assert_eq!(release.requested, 0);
        assert_eq!(release.owned, MAPPER_X);
    }
}

#[test]
fn held_x_stays_suppressed_after_shortcut_ownership_ends_and_other_jump_tools_still_work() {
    let mut shortcut = Shortcuts::default();
    let mut overlay = ManualOverlay::default();
    let grounded = ShortcutContext {
        status: kirby_assist_core::native_states::status::WAIT,
        situation: 0,
        motion: 1,
        frame: 0.0,
        rate: 1.0,
        stopped: false,
        facing: 1.0,
        can_short_hop: true,
        input_delay: 0.0,
        can_neutral_special: false,
    };
    let held = ButtonTrace {
        held: MAPPER_X,
        ..Default::default()
    };
    for context in [
        grounded,
        ShortcutContext {
            status: kirby_assist_core::native_states::status::JUMP_SQUAT,
            motion: 2,
            ..grounded
        },
        ShortcutContext {
            status: kirby_assist_core::native_states::status::JUMP,
            situation: 2,
            motion: 3,
            ..grounded
        },
        ShortcutContext {
            status: kirby_assist_core::native_states::status::JUMP,
            situation: 2,
            motion: 3,
            frame: 1.0,
            ..grounded
        },
        ShortcutContext {
            frame: 1.0,
            ..grounded
        },
    ] {
        let out = shortcut.sample(MAPPER_X, true, Some(context));
        let mapped = overlay.apply_controls(
            held,
            [0.0; 2],
            TurboOutput::default(),
            shortcut.consumed_buttons(),
            out.requested,
            out.owned,
        );
        assert_eq!(mapped.buttons.held & MAPPER_X, out.requested & MAPPER_X);
        let native_jump = if mapped.buttons.held & MAPPER_X != 0 {
            0x4
        } else {
            0
        };
        let final_buttons = mapped_short_hop(native_jump, out.requested & MAPPER_X != 0);
        assert_eq!(
            final_buttons & 0x44,
            if out.requested & MAPPER_X != 0 {
                0x44
            } else {
                0
            }
        );
    }
    // A higher-priority combo can still request its own jump while the old
    // physical hold is quarantined.
    let synthetic = overlay.apply_controls(
        held,
        [0.0; 2],
        TurboOutput::default(),
        shortcut.consumed_buttons(),
        MAPPER_X,
        MAPPER_X,
    );
    assert_eq!(synthetic.buttons.held & MAPPER_X, MAPPER_X);
    assert_eq!(
        mapped_short_hop(0x4, false),
        0x4,
        "other owners retain full hops"
    );
    assert_eq!(
        mapped_short_hop(0x8, true),
        0x8,
        "non-jump mappings remain physical"
    );
    shortcut.sample(0, true, Some(grounded));
    assert_eq!(shortcut.consumed_buttons(), 0);
}
