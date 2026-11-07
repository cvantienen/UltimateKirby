//! Literal ledge attack packets and passive guard expiry under a static pose.
use kirby_assist_core::{
    autopilot_guard::{GuardClock, GuardHold},
    ledge_trap::{InputTimeline, LedgeContext, LedgeSettings, MotionTiming, Opponent},
    manual_controls::{ManualOverlay, TurboOutput},
    native_input::*,
    observation::{Fact, Field, ObservationSnapshot},
    sequence_config::Catalog,
};
fn own(frame: f32) -> LedgeContext {
    LedgeContext {
        held: true,
        gameplay: true,
        status: 0,
        motion_kind: 1,
        motion_frame: frame,
        motion_rate: 1.,
        facing: 1.,
        ..Default::default()
    }
}
fn enemy(frame: f32) -> Opponent {
    Opponent {
        id: 2,
        status: 0x78,
        position: [10., 0.],
        motion: Some(MotionTiming {
            kind: 2,
            frame,
            rate: 1.,
            remaining: None,
        }),
        ..Default::default()
    }
}
fn tape(actions: &str) -> InputTimeline {
    let c = Catalog::parse(&format!(
        "[bowser.neutral.test]\nstart_delay=0\nactions={actions}\n"
    ))
    .unwrap();
    InputTimeline::new(c.entries()[0].sequence, enemy(0.))
}
fn gc(frame: f32) -> GuardClock {
    GuardClock {
        status: 0x1c,
        motion: 1,
        frame,
        rate: 1.,
        stopped: false,
    }
}
fn source(frame: f32) -> Option<(u32, GuardClock)> {
    Some((
        2,
        GuardClock {
            status: 0x27,
            motion: 2,
            ..gc(frame)
        },
    ))
}
#[test]
fn all_smash_axes_keep_attack_when_native_mapping_only_preserves_movement() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        for (action, stick) in [
            ("forward_smash:1", [1., 0.]),
            ("up_smash:1", [0., 1.]),
            ("down_smash:1", [0., -1.]),
            ("opponent_input:a:1:0:5", [1., 0.]),
        ] {
            let mut t = tape(action);
            let out = t.sample(LedgeSettings::default(), own(0.), Some(enemy(0.)), None, 0.);
            assert_eq!(out.stick, Some(stick));
            assert_eq!(out.requested, MAPPER_A);
            let raw = ManualOverlay::default().apply_controls(
                ButtonTrace::default(),
                [0.; 2],
                TurboOutput {
                    stick: out.stick,
                    ..Default::default()
                },
                0,
                layout.physical_buttons(out.requested),
                layout.physical_owned_buttons(out.owned),
            );
            assert_ne!(raw.buttons.held & layout.physical_buttons(MAPPER_A), 0);
            // Native mapper returned movement and unrelated bits, but lost A.
            let mapped = mapped_literal_buttons(0x800, out.requested);
            assert_eq!(mapped, 0xa01, "{layout:?} {action}");
            assert_eq!(
                [mapped_stick_axis(stick[0]), mapped_stick_axis(stick[1])],
                [
                    mapped_stick_axis(raw.stick[0]),
                    mapped_stick_axis(raw.stick[1])
                ]
            );
        }
    }
}
#[test]
fn charged_smash_preserves_every_requested_packet_and_its_release_without_turbo_edges() {
    let mut t = tape("opponent_input:a:1:0:5,wait:2");
    let mut raw = ManualOverlay::default();
    for frame in 0..7 {
        let out = t.sample(
            LedgeSettings::default(),
            own(frame as f32),
            Some(enemy(frame as f32)),
            None,
            0.,
        );
        let packet = raw.apply_controls(
            ButtonTrace::default(),
            [0.; 2],
            TurboOutput {
                stick: out.stick,
                ..Default::default()
            },
            0,
            out.requested,
            out.owned,
        );
        let mapped = mapped_literal_buttons(0, out.requested);
        assert_eq!(mapped & 0x201, if frame < 5 { 0x201 } else { 0 });
        assert_eq!(
            packet.buttons.pressed & MAPPER_A,
            if frame == 0 { MAPPER_A } else { 0 }
        );
        assert_eq!(
            packet.buttons.released & MAPPER_A,
            if frame == 5 { MAPPER_A } else { 0 }
        );
    }
}
#[test]
fn literal_oos_requests_keep_grab_special_and_guard_without_inventing_attack_or_jump() {
    for (requested, expected) in [
        (MAPPER_R | MAPPER_ZR, MAPPED_CATCH | MAPPED_GUARD),
        (MAPPER_B | MAPPER_ZR, MAPPED_SPECIAL | MAPPED_GUARD),
        (MAPPER_ZR, MAPPED_GUARD),
        (MAPPER_A | MAPPER_ZR, 0x201 | MAPPED_GUARD),
    ] {
        assert_eq!(mapped_literal_buttons(0x800, requested), 0x800 | expected);
    }
    assert_eq!(
        mapped_literal_buttons(0x800, 0),
        0x800,
        "drift/waits must not acquire buttons"
    );
    assert_eq!(
        mapped_literal_buttons(0x44, MAPPER_R),
        0x44 | MAPPED_CATCH,
        "unowned native bits stay unchanged"
    );
}
#[test]
fn up_smash_uses_native_shield_cancel_while_side_smash_waits_for_legal_shield_drop() {
    let mut up = tape("up_smash:1");
    let mut side = tape("forward_smash:1");
    let shield = LedgeContext {
        status: 0x1c,
        ..own(0.)
    };
    let out = up.sample(LedgeSettings::default(), shield, Some(enemy(0.)), None, 0.);
    assert_eq!(out.requested, MAPPER_A | MAPPER_ZR);
    assert_eq!(
        mapped_literal_buttons(0, out.requested),
        0x201 | MAPPED_GUARD
    );
    let out = side.sample(LedgeSettings::default(), shield, Some(enemy(0.)), None, 0.);
    assert_eq!(out.requested, 0);
    assert_eq!(out.stick, Some([0.; 2]));
    let out = side.sample(LedgeSettings::default(), own(1.), Some(enemy(1.)), None, 0.);
    assert_eq!(out.requested, MAPPER_A);
    assert_eq!(out.stick, Some([1., 0.]));
}
#[test]
fn static_guard_uses_enemy_frames_for_the_configured_four_frame_release_tail() {
    let mut g = GuardHold::default();
    let own = gc(0.);
    assert!(g.sample_with_progress(true, true, own, 4., source(0.)));
    for frame in 1..4 {
        assert!(g.sample_with_progress(true, false, own, 4., source(frame as f32)));
        for _ in 0..20 {
            assert!(g.sample_with_progress(true, false, own, 4., source(frame as f32)));
        }
    }
    assert!(!g.sample_with_progress(true, false, own, 4., source(4.)));
    assert!(!g.sample_with_progress(true, false, own, 4., source(5.)));
}
#[test]
fn missing_or_passive_clock_cannot_latch_shield_after_the_threat_ends() {
    for source in [
        None,
        Some((
            2,
            GuardClock {
                status: 0x1c,
                ..gc(0.)
            },
        )),
        Some((
            2,
            GuardClock {
                status: 0x77,
                ..gc(0.)
            },
        )),
        Some((2, GuardClock { rate: 0., ..gc(0.) })),
    ] {
        let mut g = GuardHold::default();
        assert!(g.sample_with_progress(true, true, gc(0.), 4., source));
        assert!(!g.sample_with_progress(true, false, gc(0.), 4., source));
    }
}
#[test]
fn shieldstun_and_hitstop_keep_defense_but_do_not_spend_or_bridge_enemy_clock_gaps() {
    let mut g = GuardHold::default();
    assert!(g.sample_with_progress(true, true, gc(0.), 4., source(0.)));
    for _ in 0..50 {
        assert!(g.sample_with_progress(
            true,
            false,
            GuardClock {
                stopped: true,
                ..gc(0.)
            },
            4.,
            source(8.)
        ));
    }
    assert!(g.sample_with_progress(
        true,
        false,
        GuardClock {
            status: 0x1e,
            ..gc(0.)
        },
        4.,
        None
    ));
    assert!(g.sample_with_progress(true, false, gc(0.), 4., source(12.)));
    for frame in 13..15 {
        assert!(g.sample_with_progress(true, false, gc(0.), 4., source(frame as f32)));
    }
    assert!(!g.sample_with_progress(true, false, gc(0.), 4., source(15.)));
}
#[test]
fn own_and_enemy_progress_are_never_added_together_and_explicit_takeover_clears_guard_now() {
    let mut g = GuardHold::default();
    assert!(g.sample_with_progress(true, true, gc(0.), 4., source(0.)));
    for frame in 1..4 {
        assert!(g.sample_with_progress(true, false, gc(frame as f32), 4., source(frame as f32)));
    }
    assert!(!g.sample_with_progress(true, false, gc(4.), 4., source(4.)));
    assert!(g.sample_with_progress(true, true, gc(5.), 4., source(5.)));
    assert!(
        !g.sample_with_progress(false, true, gc(5.), 4., source(5.)),
        "authored ledge tape owns this exact input"
    );
}
#[test]
fn an_authored_shield_hold_keeps_its_duration_independently_of_passive_defense() {
    let mut t = tape("shield:4,up_smash:1");
    for frame in 0..4 {
        assert_eq!(
            t.sample(
                LedgeSettings::default(),
                own(frame as f32),
                Some(enemy(frame as f32)),
                None,
                0.
            )
            .requested,
            MAPPER_ZR
        );
    }
    let out = t.sample(LedgeSettings::default(), own(4.), Some(enemy(4.)), None, 0.);
    assert_eq!(out.requested, MAPPER_A);
}
#[test]
fn cached_progress_clock_reads_only_existing_facts_and_ignores_own_or_static_guard_poses() {
    let facts = ObservationSnapshot::default();
    let mut calls = 0;
    for (id, status) in [(1, 0), (2, 0x1c), (3, 0x27)] {
        for (field, value) in [
            (Field::Status, Fact::Int(status)),
            (Field::Motion, Fact::Motion(42)),
            (Field::Frame, Fact::Float(3.)),
            (Field::Rate, Fact::Float(1.)),
        ] {
            facts.read(id, field, || {
                calls += 1;
                value
            });
        }
    }
    assert_eq!(facts.known_progress_clock(1), Some((3, 0x27, 42, 3., 1.)));
    assert_eq!(calls, 12);
    let empty = ObservationSnapshot::default();
    assert_eq!(empty.known_progress_clock(1), None);
}

#[test]
fn literal_release_and_smash_clear_stale_shield_and_catch_history_in_the_same_packet() {
    let previous = MAPPED_GUARD | MAPPED_GUARD_HOLD | MAPPED_CATCH | 0x800;
    assert_eq!(mapped_literal_buttons(previous, 0), 0x800);
    assert_eq!(mapped_literal_buttons(previous, MAPPER_A), 0xa01);
    assert_eq!(mapped_literal_buttons(previous, MAPPER_B), 0x802);
    assert_eq!(
        mapped_literal_buttons(0, MAPPER_A | MAPPER_ZR),
        0x201 | MAPPED_GUARD
    );
    assert_eq!(
        mapped_literal_buttons(0, MAPPER_R | MAPPER_ZR),
        MAPPED_CATCH | MAPPED_GUARD
    );
}

#[test]
fn up_tilt_keeps_its_authored_strength_and_waits_for_shield_drop() {
    let mut t = tape("up_tilt:1");
    let out = t.sample(
        LedgeSettings::default(),
        LedgeContext {
            status: 0x1c,
            ..own(0.)
        },
        Some(enemy(0.)),
        None,
        0.,
    );
    assert_eq!(out.requested, 0);
    assert_eq!(out.stick, Some([0.; 2]));
    let out = t.sample(LedgeSettings::default(), own(1.), Some(enemy(1.)), None, 0.);
    assert_eq!(out.requested, MAPPER_A);
    assert_eq!(out.stick, Some([0., 0.6]));
    assert_eq!(
        mapped_literal_buttons(MAPPED_GUARD_HOLD, out.requested),
        0x201
    );
}

#[test]
fn shield_up_b_and_pure_shield_do_not_inherit_a_catch_alias_from_an_earlier_grab() {
    let old = MAPPED_GUARD | MAPPED_GUARD_HOLD | MAPPED_CATCH;
    assert_eq!(
        mapped_literal_buttons(old, MAPPER_B | MAPPER_ZR),
        MAPPED_GUARD | MAPPED_GUARD_HOLD | MAPPED_SPECIAL
    );
    assert_eq!(
        mapped_literal_buttons(old, MAPPER_ZR),
        MAPPED_GUARD | MAPPED_GUARD_HOLD
    );
    assert_eq!(mapped_literal_buttons(old, MAPPER_R | MAPPER_ZR), old);
    assert_ne!(
        mapped_literal_buttons(old, MAPPER_A | MAPPER_ZR) & MAPPED_CATCH,
        0,
        "explicit shield+attack combo keeps native grab semantics"
    );
}
