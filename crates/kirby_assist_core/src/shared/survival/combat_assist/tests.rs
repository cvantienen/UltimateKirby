//! Regression cases for damage-hitstop SDI.
use super::*;
#[test]
fn sdi_flicks_follow_native_hitstop_and_hands_back_on_new_manual_input() {
    let mut sdi = SdiState::default();
    let ctx = SdiContext {
        object: 1,
        damage_stop: true,
        remaining: 8,
        outward: -1.0,
        input_delay: 0.0,
    };
    for _ in 0..30 {
        assert_eq!(sdi.sample(true, ctx, [0.0; 2]), Some([-1.0, 0.0]));
    }
    assert_eq!(
        sdi.sample(
            true,
            SdiContext {
                remaining: 7,
                ..ctx
            },
            [0.0; 2]
        ),
        Some([0.0; 2])
    );
    assert_eq!(
        sdi.sample(
            true,
            SdiContext {
                remaining: 6,
                ..ctx
            },
            [0.0, 0.7]
        ),
        None
    );
    assert_eq!(
        sdi.sample(
            true,
            SdiContext {
                damage_stop: false,
                ..ctx
            },
            [0.3, 0.4]
        ),
        None
    );
    assert_eq!(sdi.sample(true, ctx, [0.0; 2]), Some([-1.0, 0.0]));
    assert_eq!(sdi.sample(false, ctx, [0.0; 2]), None);
}
#[test]
fn sdi_uses_shared_stick_overlay_without_changing_buttons_and_restores_physical_input() {
    use crate::manual_controls::{ManualOverlay, TurboOutput};
    use crate::native_input::{ButtonTrace, MAPPER_ZR};
    let mut overlay = ManualOverlay::default();
    let mut sdi = SdiState::default();
    let human = ButtonTrace {
        held: MAPPER_ZR,
        previous: MAPPER_ZR,
        ..ButtonTrace::default()
    };
    let context = SdiContext {
        object: 1,
        damage_stop: true,
        remaining: 8,
        outward: -1.0,
        input_delay: 0.0,
    };
    let stick = sdi.sample(true, context, [0.0; 2]);
    let mapped = overlay.apply_controls(
        human,
        [0.0; 2],
        TurboOutput {
            consume_dpad: false,
            stick,
        },
        0,
        0,
        0,
    );
    assert_eq!(mapped.stick, [-1.0, 0.0]);
    assert_eq!(mapped.buttons.held & MAPPER_ZR, MAPPER_ZR);
    let physical = [0.3, -0.4];
    let stick = sdi.sample(
        true,
        SdiContext {
            damage_stop: false,
            ..context
        },
        physical,
    );
    let restored = overlay.apply_controls(
        human,
        physical,
        TurboOutput {
            consume_dpad: false,
            stick,
        },
        0,
        0,
        0,
    );
    assert_eq!(restored.stick, physical);
    assert_eq!(restored.buttons.held & MAPPER_ZR, MAPPER_ZR);
}
#[test]
fn sdi_stops_early_enough_for_known_delayed_inputs_to_clear_hitstop() {
    let mut sdi = SdiState::default();
    let ctx = SdiContext {
        object: 1,
        damage_stop: true,
        remaining: 8,
        outward: -1.0,
        input_delay: 5.0,
    };
    assert_eq!(sdi.sample(true, ctx, [0.0; 2]), Some([-1.0, 0.0]));
    assert_eq!(
        sdi.sample(
            true,
            SdiContext {
                remaining: 5,
                ..ctx
            },
            [0.0; 2]
        ),
        None
    );
}

#[test]
fn sdi_respects_preheld_stick_and_keeps_manual_control_for_the_hit() {
    let mut sdi = SdiState::default();
    let ctx = SdiContext {
        object: 1,
        damage_stop: true,
        remaining: 8,
        input_delay: 0.0,
        outward: -1.0,
    };
    // Even a pre-held direction is manual control, including inward SDI.
    assert_eq!(sdi.sample(true, ctx, [1.0, 0.0]), None);
    assert_eq!(sdi.sample(true, ctx, [1.0, 0.0]), None);
    assert_eq!(
        sdi.sample(
            true,
            SdiContext {
                remaining: 7,
                ..ctx
            },
            [0.0; 2]
        ),
        None
    );
    // A fresh movement takes full control, including the choice to go inward.
    assert_eq!(
        sdi.sample(
            true,
            SdiContext {
                remaining: 6,
                ..ctx
            },
            [1.0, 0.0]
        ),
        None
    );
    assert_eq!(
        sdi.sample(
            true,
            SdiContext {
                remaining: 5,
                ..ctx
            },
            [0.0; 2]
        ),
        None
    );
    assert_eq!(
        sdi.sample(
            true,
            SdiContext {
                damage_stop: false,
                ..ctx
            },
            [0.0; 2]
        ),
        None
    );
    assert_eq!(sdi.sample(true, ctx, [0.0; 2]), Some([-1.0, 0.0]));
}

#[test]
fn sdi_does_not_guess_a_direction_when_attacker_is_unknown() {
    let mut sdi = SdiState::default();
    let ctx = SdiContext {
        object: 1,
        damage_stop: true,
        remaining: 8,
        input_delay: 0.0,
        outward: 0.0,
    };
    assert_eq!(sdi.sample(true, ctx, [0.0; 2]), None);
    assert_eq!(
        sdi.sample(
            true,
            SdiContext {
                remaining: 7,
                ..ctx
            },
            [0.8, 0.0]
        ),
        None
    );
    assert!(sdi.manual_control);
}
