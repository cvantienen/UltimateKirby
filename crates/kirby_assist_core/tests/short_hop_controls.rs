use kirby_assist_core::{
    manual_controls::{ManualOverlay, ShortcutContext, Shortcuts, TurboOutput},
    native_input::{ButtonTrace, MAPPER_X, MAPPER_ZL},
};

#[test]
fn horizontal_short_hop_never_injects_shield_and_releases_on_squat() {
    let ctx = ShortcutContext {
        status: 0, situation: 0, motion: 1, frame: 0., rate: 1.,
        stopped: false, facing: 1., can_short_hop: true,
        input_delay: 0., can_neutral_special: false,
    };
    for horizontal in [-1., 1.] {
        let mut shortcut = Shortcuts::default();
        shortcut.sample(0, true, Some(ctx));
        let hop = shortcut.sample(MAPPER_X, true, Some(ctx));
        assert_eq!(hop.requested, MAPPER_X);
        assert_eq!(hop.requested & MAPPER_ZL, 0);
        let applied = ManualOverlay::default().apply_controls(
            ButtonTrace { held: MAPPER_X, pressed: MAPPER_X, ..ButtonTrace::default() },
            [horizontal, 0.], TurboOutput::default(), shortcut.consumed_buttons(),
            hop.requested, hop.owned,
        );
        assert_eq!(applied.stick, [horizontal, 0.]);
        assert_eq!(applied.buttons.held & MAPPER_ZL, 0);
        let release = shortcut.sample(MAPPER_X, true, Some(ShortcutContext {
            status: 0xa, motion: 2, ..ctx
        }));
        assert_eq!(release.requested, 0);
        assert_eq!(release.owned, MAPPER_X);
    }
}
