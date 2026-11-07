use kirby_assist_core::{
    ledge_trap::{InputTimeline, LedgeContext, LedgeSettings, Opponent, SEQUENCE_BUTTONS},
    manual_controls::{ManualOverlay, TurboOutput, LEFT_STICK_DIRECTIONS},
    native_input::*,
    sequence_config::Catalog,
};
fn context(status: i32, frame: f32) -> LedgeContext {
    LedgeContext {
        held: true,
        gameplay: true,
        status,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.,
        facing: 1.,
        ..Default::default()
    }
}
fn timeline() -> InputTimeline {
    let c=Catalog::parse("[bowser.roll.chase]\nstart_delay=0\nactions=drift:opponent:1:10,up_b:4,drift:opponent:1:65\n").unwrap();
    InputTimeline::new(
        c.entries()[0].sequence,
        Opponent {
            id: 2,
            position: [20., 0.],
            ..Default::default()
        },
    )
}
#[test]
fn reaction_chase_gets_a_fresh_full_stick_flick_despite_a_previously_held_direction() {
    for (x, bit) in [(1., 1 << 21), (-1., 1 << 23)] {
        let physical = ButtonTrace {
            held: bit,
            previous: bit,
            ..Default::default()
        };
        let mut overlay = ManualOverlay::default();
        overlay.apply_controls(physical, [x, 0.], TurboOutput::default(), 0, 0, 0);
        let command = TurboOutput {
            stick: Some([x, 0.]),
            ..Default::default()
        };
        let first = overlay.apply_reaction_controls(
            physical,
            [x, 0.],
            command,
            0,
            0,
            SEQUENCE_BUTTONS,
            true,
        );
        assert_eq!(first.buttons.pressed & bit, bit);
        assert_eq!(first.buttons.previous & bit, 0);
        assert_eq!(first.stick, [x, 0.]);
        assert_eq!(
            first.buttons.held & GAMEPLAY_BUTTONS,
            0,
            "no invented jump/guard/attack"
        );
        for _ in 0..8 {
            let next = overlay.apply_reaction_controls(
                physical,
                [x, 0.],
                command,
                0,
                0,
                SEQUENCE_BUTTONS,
                true,
            );
            assert_eq!(
                next.buttons.pressed & bit,
                0,
                "held chase must not flick every poll"
            );
            assert_eq!(next.stick, [x, 0.]);
        }
    }
}
#[test]
fn partial_drift_and_non_reaction_grab_or_ledge_paths_keep_their_existing_history() {
    let bit = 1 << 21;
    let physical = ButtonTrace {
        held: bit,
        previous: bit,
        ..Default::default()
    };
    for (reaction, strength, buttons) in [(true, 0.5, 0), (false, 1., 0), (false, 0., MAPPER_R)] {
        let mut overlay = ManualOverlay::default();
        overlay.apply_controls(physical, [1., 0.], TurboOutput::default(), 0, 0, 0);
        let out = overlay.apply_reaction_controls(
            physical,
            [1., 0.],
            TurboOutput {
                stick: Some([strength, 0.]),
                ..Default::default()
            },
            0,
            buttons,
            SEQUENCE_BUTTONS,
            reaction,
        );
        assert_eq!(out.buttons.pressed & bit, 0);
        if buttons != 0 {
            assert_eq!(out.buttons.held & MAPPER_R, MAPPER_R);
        }
    }
}
#[test]
fn shield_release_observation_gap_keeps_the_chase_and_its_entire_ten_frame_duration() {
    let mut t = timeline();
    t.allow_continuous_progress();
    for (status, frame) in [(0x1c, 0.), (0x1d, 0.), (0x1d, 2.)] {
        let out = t.sample(
            LedgeSettings::default(),
            context(status, frame),
            None,
            None,
            0.,
        );
        assert_eq!(out.requested, 0);
        assert_eq!(out.stick, Some([0.; 2]));
        assert!(!t.finished());
    }
    for frame in 0..10 {
        let out = t.sample(
            LedgeSettings::default(),
            context(0, frame as f32),
            None,
            None,
            0.,
        );
        assert_eq!(out.stick, Some([1., 0.]));
        assert_eq!(out.requested, 0, "chase frame {frame}");
    }
    let out = t.sample(LedgeSettings::default(), context(0, 10.), None, None, 0.);
    assert_eq!(out.requested, MAPPER_B);
    assert_eq!(out.stick, Some([0., 1.]));
}
#[test]
fn neutral_shield_release_still_suppresses_human_directions_before_chase_starts() {
    let physical = ButtonTrace {
        held: GAMEPLAY_BUTTONS | LEFT_STICK_DIRECTIONS,
        previous: GAMEPLAY_BUTTONS | LEFT_STICK_DIRECTIONS,
        ..Default::default()
    };
    let mut overlay = ManualOverlay::default();
    let mut t = timeline();
    t.allow_continuous_progress();
    let waiting = t.sample(LedgeSettings::default(), context(0x1c, 0.), None, None, 0.);
    let out = overlay.apply_reaction_controls(
        physical,
        [1., -1.],
        TurboOutput {
            stick: waiting.stick,
            ..Default::default()
        },
        0,
        waiting.requested,
        waiting.owned,
        true,
    );
    assert_eq!(out.stick, [0.; 2]);
    assert_eq!(
        out.buttons.held & (GAMEPLAY_BUTTONS | LEFT_STICK_DIRECTIONS),
        0
    );
    let chase = t.sample(LedgeSettings::default(), context(0, 0.), None, None, 0.);
    let out = overlay.apply_reaction_controls(
        physical,
        [1., -1.],
        TurboOutput {
            stick: chase.stick,
            ..Default::default()
        },
        0,
        chase.requested,
        chase.owned,
        true,
    );
    assert_eq!(out.stick, [1., 0.]);
    assert_eq!(out.buttons.pressed & (1 << 21), 1 << 21);
    assert_eq!(out.buttons.held & GAMEPLAY_BUTTONS, 0);
}
