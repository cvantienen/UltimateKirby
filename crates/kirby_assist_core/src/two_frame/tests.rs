//! Copied-observation regressions. These verify scheduling/ownership, not game
//! collision or a universal recovery vulnerability window.
use super::*;
fn settings() -> LedgeSettings {
    LedgeSettings {
        enabled: true,
        two_frame: true,
        ..LedgeSettings::default()
    }
}
fn spot() -> Option<LedgeSpot> {
    Some(LedgeSpot {
        edge: [0.0, 0.0],
        inward: 1.0,
    })
}
fn own(frame: f32) -> LedgeContext {
    LedgeContext {
        held: true,
        gameplay: true,
        can_act: true,
        can_control: true,
        stopped: false,
        status: 0,
        previous_status: 0,
        situation: 0,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.0,
        position: [8.0, 0.0],
        velocity: [0.0; 2],
        ground_brake: None,
        facing: -1.0,
        can_grab: true,
        can_tilt: true,
        can_down_tilt: true,
        can_down_special: true,
        can_dash_attack: true,
        can_down_smash: true,
        input_delay: 0.0,
        can_jump: true,
        can_air_attack: true,
        air_physics: None,
        short_hop_speed: None,
        full_hop_speed: None,
        shield_stun_frames: None,
        shield_attacker: None,
        hit_confirmed: false,
        safe_ground: [true; 2],
    }
}
fn enemy(y: f32) -> Option<RecoveryTarget> {
    Some(RecoveryTarget {
        id: 2,
        status: 0x1de,
        previous_status: 0xe,
        situation: 2,
        position: [-3.0, y],
        velocity: [0.0, 1.0],
        stopped: false,
        vulnerable: true,
    })
}
#[test]
fn release_then_paired_down_a_then_release_on_real_smash_start() {
    let mut tool = TwoFrame::default();
    assert_eq!(
        tool.sample(settings(), own(1.0), spot(), enemy(-15.0))
            .owned,
        0
    );
    assert_eq!(
        tool.sample(settings(), own(2.0), spot(), enemy(-14.0)),
        neutral()
    );
    // Duplicate mapper observations cannot spend the release frame.
    for _ in 0..20 {
        assert_eq!(
            tool.sample(settings(), own(2.0), spot(), enemy(-14.0)),
            neutral()
        );
    }
    assert_eq!(
        tool.sample(settings(), own(3.0), spot(), enemy(-13.0)),
        press()
    );
    assert_eq!(
        tool.sample(settings(), own(3.0), spot(), enemy(-13.0)),
        press()
    );
    // Acceptance, rather than an arbitrary held-input duration, releases A.
    let accepted = LedgeContext {
        status: 0x30,
        motion_kind: 3,
        motion_frame: 0.0,
        can_act: false,
        ..own(4.0)
    };
    assert_eq!(
        tool.sample(settings(), accepted, spot(), enemy(-12.0)),
        neutral()
    );
    for i in 1..5 {
        assert_eq!(
            tool.sample(
                settings(),
                LedgeContext {
                    motion_frame: i as f32,
                    ..accepted
                },
                spot(),
                enemy(-12.0)
            )
            .owned,
            0
        );
    }
}
#[test]
fn delivery_delay_moves_the_request_earlier() {
    let mut tool = TwoFrame::default();
    let ctx = LedgeContext {
        input_delay: 2.0,
        ..own(1.0)
    };
    assert_eq!(
        tool.sample(settings(), ctx, spot(), enemy(-16.0)),
        neutral()
    ); // arrival 10
    assert_eq!(
        tool.sample(
            settings(),
            LedgeContext {
                motion_frame: 2.0,
                ..ctx
            },
            spot(),
            enemy(-15.0)
        ),
        press()
    ); // arrival 9
}
#[test]
fn ignores_unknown_geometry_protected_and_late_approaches() {
    for (ctx, edge, target) in [
        (own(1.0), None, enemy(-14.0)),
        (
            own(1.0),
            spot(),
            Some(RecoveryTarget {
                vulnerable: false,
                ..enemy(-14.0).unwrap()
            }),
        ),
        (
            own(1.0),
            spot(),
            Some(RecoveryTarget {
                velocity: [0.0, -1.0],
                ..enemy(-14.0).unwrap()
            }),
        ),
        (own(1.0), spot(), enemy(-8.0)),
        (
            LedgeContext {
                can_down_smash: false,
                ..own(1.0)
            },
            spot(),
            enemy(-14.0),
        ),
        (
            LedgeContext {
                input_delay: f32::NAN,
                ..own(1.0)
            },
            spot(),
            enemy(-14.0),
        ),
    ] {
        assert_eq!(
            TwoFrame::default()
                .sample(settings(), ctx, edge, target)
                .owned,
            0
        );
    }
}
#[test]
fn pending_request_expires_without_repeating_and_release_cancels_immediately() {
    let mut tool = TwoFrame::default();
    tool.sample(settings(), own(1.0), spot(), enemy(-14.0));
    tool.sample(settings(), own(2.0), spot(), enemy(-13.0));
    for i in 3..15 {
        tool.sample(settings(), own(i as f32), spot(), enemy(-13.0));
    }
    assert_eq!(
        tool.sample(settings(), own(15.0), spot(), enemy(-13.0))
            .owned,
        0
    );
    assert_eq!(tool.phase, Phase::Done);
    let released = LedgeContext {
        held: false,
        ..own(16.0)
    };
    assert_eq!(
        tool.sample(settings(), released, spot(), enemy(-13.0))
            .owned,
        0
    );
    assert_eq!(tool.target(), None);
}
#[test]
fn skipped_or_rewound_pending_frames_cannot_emit_a_late_smash() {
    for next in [0.0, 8.0, f32::NAN] {
        let mut tool = TwoFrame::default();
        tool.sample(settings(), own(1.0), spot(), enemy(-14.0));
        assert_eq!(
            tool.sample(settings(), own(next), spot(), enemy(-13.0))
                .owned,
            0
        );
    }
}
#[test]
fn mirrored_ledge_has_the_same_arrival_forecast() {
    let right = LedgeSpot {
        edge: [0.0, 0.0],
        inward: -1.0,
    };
    let left = spot().unwrap();
    let target = enemy(-14.0).unwrap();
    assert_eq!(approach_in(left, target), Some(8.0));
    assert_eq!(
        approach_in(
            right,
            RecoveryTarget {
                position: [3.0, -14.0],
                ..target
            }
        ),
        Some(8.0)
    );
}

#[test]
fn kirby_down_tilt_uses_four_frame_startup_and_a_tilt_primer() {
    let settings = LedgeSettings {
        two_frame_down_tilt: true,
        ..settings()
    };
    let mut tool = TwoFrame::default();
    assert_eq!(settings.fighter.down_tilt.first, 4);
    let prep = tool.sample(settings, own(1.0), spot(), enemy(-11.0)); // arrival 5
    assert_eq!(prep.requested, 0);
    assert_eq!(prep.stick, Some([0.0, -0.6]));
    assert_eq!(tool.sample(settings, own(1.0), spot(), enemy(-11.0)), prep);
    let request = tool.sample(settings, own(2.0), spot(), enemy(-10.0)); // arrival 4
    assert_eq!(request.requested, MAPPER_A);
    assert_eq!(request.stick, Some([0.0, -0.6]));
    let accepted = LedgeContext {
        status: 0x2c,
        motion_kind: 2,
        motion_frame: 0.0,
        can_act: false,
        ..own(3.0)
    };
    assert_eq!(
        tool.sample(settings, accepted, spot(), enemy(-9.0))
            .requested,
        0
    );
    assert_eq!(tool.phase, Phase::Done);
    assert_eq!(
        tool.sample(
            settings,
            LedgeContext {
                held: false,
                ..own(4.0)
            },
            spot(),
            enemy(-8.0)
        )
        .owned,
        0
    );
}
#[test]
fn down_tilt_forecast_accounts_for_delay_and_needs_correct_facing_and_range() {
    let settings = LedgeSettings {
        two_frame_down_tilt: true,
        ..settings()
    };
    let mut tool = TwoFrame::default();
    let own = LedgeContext {
        input_delay: 2.0,
        ..own(1.0)
    };
    assert_eq!(
        tool.sample(settings, own, spot(), enemy(-13.0)).stick,
        Some([0.0, -0.6])
    );
    assert_eq!(
        tool.sample(
            settings,
            LedgeContext {
                motion_frame: 2.0,
                ..own
            },
            spot(),
            enemy(-12.0)
        )
        .requested,
        MAPPER_A
    );
    let wrong_way = LedgeContext { facing: 1.0, ..own };
    let turn = TwoFrame::default().sample(settings, wrong_way, spot(), enemy(-13.0));
    assert_eq!(turn.requested, 0);
    assert_eq!(turn.stick, Some([-0.3, 0.0]));
    assert_eq!(
        TwoFrame::default()
            .sample(
                settings,
                LedgeContext {
                    position: [14.0, 0.0],
                    ..own
                },
                spot(),
                enemy(-13.0)
            )
            .owned,
        0
    );
}
