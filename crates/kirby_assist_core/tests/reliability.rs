use kirby_assist_core::live_input::{LiveSettings, RawAutoControls, HID_ZR};
use kirby_assist_core::manual_controls::{ManualOverlay, TurboOutput};
use kirby_assist_core::native_input::{ButtonTrace, MAPPER_A, MAPPER_ZR};
use kirby_assist_core::{
    arbitrate, ButtonSet, ConfirmedSequence, ConfirmedStep, ControllerFrame, InputPermissions,
    InputRecipe, SequenceObservation, SequenceState,
};

fn observation(frame: u64) -> SequenceObservation {
    SequenceObservation {
        frame,
        target: 2,
        status: 0,
        can_act: true,
        hit_confirmed: false,
        stopped: false,
        interrupted: false,
    }
}
fn steps() -> [ConfirmedStep; 2] {
    [
        ConfirmedStep {
            recipe: InputRecipe::buttons(ButtonSet::ATTACK),
            accepted_status: 0x2c,
            require_hit: true,
            timeout_frames: 10,
        },
        ConfirmedStep {
            recipe: InputRecipe::buttons(ButtonSet::JUMP),
            accepted_status: 0xb,
            require_hit: false,
            timeout_frames: 5,
        },
    ]
}

#[test]
fn combo_requires_native_hit_and_readiness_then_releases_between_steps() {
    let steps = steps();
    let mut combo = ConfirmedSequence::new(&steps, 2);
    let first = combo.sample(observation(100));
    assert_eq!(first, steps[0].recipe);
    for _ in 0..20 {
        assert_eq!(combo.sample(observation(100)), first);
    }
    let whiff = SequenceObservation {
        status: 0x2c,
        can_act: false,
        ..observation(101)
    };
    assert_eq!(combo.sample(whiff), InputRecipe::NONE);
    assert_eq!(combo.state(), SequenceState::WaitingForConfirmation);
    let hit = SequenceObservation {
        frame: 102,
        hit_confirmed: true,
        ..whiff
    };
    assert_eq!(combo.sample(hit), InputRecipe::NONE);
    assert_eq!(combo.state(), SequenceState::Ready);
    assert_eq!(
        combo.sample(SequenceObservation { frame: 103, ..hit }),
        InputRecipe::NONE
    );
    assert_eq!(combo.sample(observation(104)), steps[1].recipe);
    assert_eq!(
        combo.sample(SequenceObservation {
            status: 0xb,
            ..observation(105)
        }),
        InputRecipe::NONE
    );
    assert_eq!(combo.state(), SequenceState::Complete);
    assert_eq!(combo.sample(observation(106)), InputRecipe::NONE);
}

#[test]
fn combo_cancels_on_whiff_timeout_damage_target_change_clock_rewind_and_expired_readiness() {
    let steps = steps();
    for invalid in [
        SequenceObservation {
            interrupted: true,
            ..observation(101)
        },
        SequenceObservation {
            target: 3,
            ..observation(100)
        },
        observation(99),
        observation(110),
        SequenceObservation {
            status: 0x2c,
            hit_confirmed: true,
            ..observation(111)
        },
    ] {
        let mut combo = ConfirmedSequence::new(&steps, 2);
        combo.sample(observation(100));
        assert_eq!(combo.sample(invalid), InputRecipe::NONE);
        assert_eq!(combo.state(), SequenceState::Cancelled);
    }
    let mut combo = ConfirmedSequence::new(&steps, 2);
    combo.sample(SequenceObservation {
        can_act: false,
        ..observation(0)
    });
    combo.sample(SequenceObservation {
        can_act: false,
        ..observation(10)
    });
    assert_eq!(combo.state(), SequenceState::Cancelled);
}

#[test]
fn combo_hitstop_does_not_consume_confirmation_timeout_or_repeat_an_input() {
    let steps = steps();
    let mut combo = ConfirmedSequence::new(&steps, 2);
    combo.sample(observation(0));
    for frame in 1..100 {
        assert_eq!(
            combo.sample(SequenceObservation {
                stopped: true,
                ..observation(frame)
            }),
            InputRecipe::NONE
        );
    }
    combo.sample(observation(100));
    assert_eq!(combo.state(), SequenceState::WaitingForConfirmation);
    combo.sample(SequenceObservation {
        status: 0x2c,
        hit_confirmed: true,
        ..observation(101)
    });
    assert_eq!(combo.state(), SequenceState::Ready);
}

#[test]
fn combo_does_not_accept_a_preexisting_animation_as_a_new_step() {
    let steps = steps();
    let mut combo = ConfirmedSequence::new(&steps, 2);
    assert_eq!(
        combo.sample(SequenceObservation {
            status: 0x2c,
            hit_confirmed: true,
            ..observation(0)
        }),
        InputRecipe::NONE
    );
    assert_eq!(combo.state(), SequenceState::Ready);
    assert_eq!(combo.sample(observation(1)), steps[0].recipe);
}

#[test]
fn one_native_history_releases_ledge_attack_and_creates_a_fresh_tech_edge() {
    let mut overlay = ManualOverlay::default();
    let human = ButtonTrace {
        held: MAPPER_A | MAPPER_ZR,
        previous: MAPPER_A | MAPPER_ZR,
        ..ButtonTrace::default()
    };
    let ledge = overlay.apply_controls(
        human,
        [0.0, 0.0],
        TurboOutput::default(),
        0,
        MAPPER_A,
        MAPPER_A | MAPPER_ZR,
    );
    assert_eq!(ledge.buttons.held, MAPPER_A);
    let tech = overlay.apply_controls(
        human,
        [0.0, 0.0],
        TurboOutput::default(),
        0,
        MAPPER_ZR,
        MAPPER_A | MAPPER_ZR,
    );
    assert_eq!(tech.buttons.pressed & MAPPER_ZR, MAPPER_ZR);
    assert_eq!(tech.buttons.released & MAPPER_A, MAPPER_A);
    assert_eq!(tech.buttons.previous & MAPPER_ZR, 0);
}

#[test]
fn raw_scheduler_cancellation_wins_even_on_an_identical_native_tick() {
    let mut controls = RawAutoControls::default();
    let settings = LiveSettings::default();
    controls.update(settings, 1, 1, true, false);
    assert!(controls.mash_stick().is_some());
    assert_eq!(controls.update(settings, 1, 1, false, false), 0);
    assert!(controls.mash_stick().is_none());
    assert_eq!(controls.update(settings, 2, 1, false, true), HID_ZR);
    assert_eq!(controls.update(settings, 2, 1, false, false), 0);
}

#[test]
fn arbitration_reports_only_buttons_actually_added() {
    let human = ControllerFrame {
        buttons: ButtonSet::ATTACK,
        ..ControllerFrame::default()
    };
    let result = arbitrate(
        human,
        InputRecipe::buttons(ButtonSet::ATTACK),
        InputPermissions::default(),
    );
    assert_eq!(result.added_buttons, ButtonSet::EMPTY);
    assert_eq!(result.frame, human);
}

#[test]
fn paused_scheduler_does_not_replay_a_cached_tech_press_when_the_clock_is_still_frozen() {
    let mut controls = RawAutoControls::default();
    let settings = LiveSettings::default();
    assert_eq!(controls.update(settings, 1, 1, false, true), HID_ZR);
    let remaining = controls.tech_cooldown_remaining();
    controls.pause_inputs();
    assert_eq!(controls.update(settings, 1, 1, false, true), 0);
    assert_eq!(controls.tech_cooldown_remaining(), remaining);
}

#[test]
fn additive_tech_request_is_applied_without_takeover_or_reserved_dpad() {
    let mut overlay = ManualOverlay::default();
    let human = ButtonTrace {
        held: MAPPER_A,
        previous: MAPPER_A,
        ..ButtonTrace::default()
    };
    let press = overlay.apply_controls(human, [0.2, 0.0], TurboOutput::default(), 0, MAPPER_ZR, 0);
    assert_eq!(press.buttons.held, MAPPER_A | MAPPER_ZR);
    assert_eq!(press.buttons.pressed, MAPPER_ZR);
    let release = overlay.apply_controls(human, [0.2, 0.0], TurboOutput::default(), 0, 0, 0);
    assert_eq!(release.buttons.released, MAPPER_ZR);
    assert_eq!(release.buttons.held, MAPPER_A);
}
