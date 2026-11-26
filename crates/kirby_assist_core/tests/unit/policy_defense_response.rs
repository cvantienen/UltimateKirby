use super::*;
use crate::{
    native_input::MAPPER_ZR,
    native_states::situation::{AIR, GROUND},
};

fn context(situation: i32) -> Context {
    Context {
        object: 1,
        situation,
        allowed: true,
        guard_ready: true,
        air_dodge_ready: true,
        ..Context::default()
    }
}
fn hit(in_frames: f32) -> Option<Contact> {
    Some(Contact {
        source: 2,
        in_frames,
        projectile: false,
    })
}

#[test]
fn ground_reserves_controls_and_releases_at_contact_exit() {
    let mut defense = Defense::new();
    let c = context(GROUND);
    let out = defense.sample(0, true, c, hit(5.0), 4.0, 1.0);
    assert_eq!(out.requested, MAPPER_ZR);
    assert_eq!(out.owned, crate::ledge_trap::SEQUENCE_BUTTONS);
    assert_eq!(out.stick, Some([0.0; 2]));
    for contact in [hit(5.1), hit(f32::NAN), hit(-1.0), None] {
        assert_eq!(defense.sample(1, true, c, contact, 4.0, 1.0).requested, 0);
    }
    assert_eq!(
        defense.sample(2, false, c, hit(0.0), 4.0, 1.0),
        ControllerOutput::NONE
    );
    assert_eq!(
        defense.sample(
            3,
            true,
            Context {
                allowed: false,
                ..c
            },
            hit(0.0),
            4.0,
            1.0
        ),
        ControllerOutput::NONE
    );
    assert_eq!(
        defense
            .sample(
                4,
                true,
                Context {
                    guard_ready: false,
                    ..c
                },
                hit(0.0),
                4.0,
                1.0
            )
            .requested,
        0
    );
}

#[test]
fn air_receipt_stops_repeats_until_native_readiness_returns() {
    let mut defense = Defense::new();
    let c = context(AIR);
    assert_eq!(
        defense.sample(0, true, c, hit(0.0), 4.0, 1.0).requested,
        MAPPER_ZR
    );
    assert!(defense.air_pending());
    defense.mapped_air_dodge();
    assert!(defense.air_pending());
    assert_eq!(defense.sample(1, true, c, hit(0.0), 4.0, 1.0).requested, 0);
    assert_eq!(
        defense
            .sample(
                2,
                true,
                Context {
                    air_dodging: true,
                    ..c
                },
                hit(0.0),
                4.0,
                1.0
            )
            .requested,
        0
    );
    assert_eq!(
        defense
            .sample(
                3,
                true,
                Context {
                    air_dodge_ready: false,
                    ..c
                },
                hit(0.0),
                4.0,
                1.0
            )
            .requested,
        0
    );
    assert_eq!(
        defense.sample(4, true, c, hit(0.0), 4.0, 1.0).requested,
        MAPPER_ZR
    );
}

#[test]
fn pending_delivery_is_bounded_and_release_discards_it() {
    let mut defense = Defense::new();
    let c = context(AIR);
    assert_eq!(
        defense.sample(0, true, c, hit(0.0), 4.0, 1.0).requested,
        MAPPER_ZR
    );
    assert_eq!(defense.sample(10, true, c, hit(0.0), 4.0, 1.0).requested, 0);
    assert_eq!(
        defense.sample(11, true, c, hit(0.0), 4.0, 1.0).requested,
        MAPPER_ZR
    );
    defense.sample(12, false, c, hit(0.0), 4.0, 1.0);
    defense.mapped_air_dodge(); // A stale receipt cannot arm the new hold.
    assert_eq!(
        defense.sample(13, true, c, hit(0.0), 4.0, 1.0).requested,
        MAPPER_ZR
    );
    let stopped = Context { stopped: true, ..c };
    assert_eq!(
        defense
            .sample(14, true, stopped, hit(0.0), 4.0, 1.0)
            .requested,
        0
    );
    assert_eq!(defense.sample(15, true, c, None, 4.0, 1.0).requested, 0);
    assert!(!defense.air_pending());
}

#[test]
fn rejected_mapped_dodge_releases_and_retries_until_native_acceptance() {
    let mut defense = Defense::new();
    let c = context(AIR);
    for start in [0, 4, 8] {
        assert_eq!(
            defense.sample(start, true, c, hit(0.), 4., 1.).requested,
            MAPPER_ZR
        );
        defense.mapped_air_dodge();
        for tick in start + 1..=start + 3 {
            assert_eq!(defense.sample(tick, true, c, hit(0.), 4., 1.).requested, 0);
        }
    }
    assert_eq!(
        defense.sample(12, true, c, hit(0.), 4., 1.).requested,
        MAPPER_ZR
    );
    defense.mapped_air_dodge();
    assert_eq!(
        defense
            .sample(
                13,
                true,
                Context {
                    air_dodging: true,
                    ..c
                },
                hit(0.),
                4.,
                1.
            )
            .requested,
        0
    );
    assert_eq!(
        defense
            .sample(
                14,
                true,
                Context {
                    air_dodge_ready: false,
                    ..c
                },
                hit(0.),
                4.,
                1.
            )
            .requested,
        0
    );
    assert_eq!(
        defense.sample(15, true, c, hit(0.), 4., 1.).requested,
        MAPPER_ZR
    );
}

#[test]
fn movement_and_attack_states_have_no_extra_defense_lockouts() {
    use crate::native_states::{situation, status};
    for (where_, states) in [
        (
            GROUND,
            &[
                status::WAIT,
                status::WALK,
                status::DASH,
                status::TURN_DASH,
                status::RUN,
                status::TURN_RUN,
                status::RUN_BRAKE,
                status::TURN_RUN_BRAKE,
                status::JUMP_SQUAT,
                status::SQUAT,
                status::SQUAT_WAIT,
                status::LANDING,
                status::LANDING_LIGHT,
                status::GUARD_OFF,
                status::ATTACK,
            ][..],
        ),
        (
            AIR,
            &[
                status::JUMP,
                status::JUMP_AERIAL,
                status::FALL,
                status::FALL_AERIAL,
                status::ATTACK_AIR,
                status::ESCAPE_AIR,
                status::ESCAPE_AIR_SLIDE,
            ][..],
        ),
    ] {
        for &state in states {
            assert!(
                native_allowed(crate::fighter_kinds::KIRBY, state, where_, false, true),
                "{state}"
            );
        }
    }
    for state in [status::DAMAGE, status::CAPTURE_PULLED, status::DEAD] {
        assert!(!native_allowed(
            crate::fighter_kinds::KIRBY,
            state,
            GROUND,
            false,
            true
        ));
    }
    assert!(!native_allowed(
        crate::fighter_kinds::KIRBY,
        status::WAIT,
        situation::CLIFF,
        false,
        true
    ));
    assert!(!native_allowed(
        crate::fighter_kinds::KIRBY,
        status::WAIT,
        GROUND,
        true,
        true
    ));
}

#[test]
fn lead_and_delivery_delay_share_bounded_prediction_window() {
    assert_eq!(defense_horizon(0.0, 0.0), Some(0.0));
    assert_eq!(defense_horizon(4.0, 3.0), Some(7.0));
    assert_eq!(defense_horizon(30.0, 120.0), Some(40.0));
    assert_eq!(defense_horizon(f32::NAN, 0.0), None);
    assert_eq!(defense_horizon(31.0, 0.0), None);
    assert_eq!(defense_horizon(4.0, -1.0), None);
}

#[test]
fn valid_scaled_bodies_predict_real_contacts_not_root_distance() {
    let body = crate::auto_counter::Config {
        startup: 1.0,
        input_delay: 3.0,
        margin: 0.0,
        half_width: 2.0,
        height: 8.0,
        half_depth: 2.0,
    };
    let attack = crate::autopilot_guard::Hitbox {
        start: [5.0, 4.0, 0.0],
        end: [5.0, 4.0, 0.0],
        radius: 1.0,
        velocity: [0.0; 2],
    };
    assert_eq!(
        crate::auto_counter::incoming_contact(body, [0.0; 3], [0.0; 2], attack, 4.0, None),
        None
    );
    let large = scaled_body(body, 2.0).unwrap();
    assert_eq!(
        crate::auto_counter::incoming_contact(large, [0.0; 3], [0.0; 2], attack, 4.0, None),
        Some(0.0)
    );
    let approaching = crate::autopilot_guard::Hitbox {
        velocity: [-1.0, 0.0],
        ..attack
    };
    assert_eq!(
        crate::auto_counter::incoming_contact(body, [0.0; 3], [0.0; 2], approaching, 4.0, None),
        Some(2.0)
    );
    assert!(scaled_body(body, f32::NAN).is_none());
    assert!(scaled_body(body, 0.0).is_none());
}
