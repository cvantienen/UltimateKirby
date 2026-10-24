use kirby_assist_core::autopilot_guard::{
    duck::{geometry_threat, DefenseThreat, DuckContext, DuckDefense},
    duck_geometry::{contact, Contact, Hitbox, CROUCH_TOP},
    GuardClock,
};
fn shot(x: f32, y: f32, vx: f32) -> Hitbox {
    Hitbox {
        start: [x, y, 0.],
        end: [x, y, 0.],
        radius: 1.,
        velocity: [vx, 0.],
    }
}
#[test]
fn detects_high_projectiles_from_both_sides_and_fast_swept_contacts() {
    for side in [-1., 1.] {
        assert!(matches!(
            contact([0.; 3], [0.; 2], shot(side * 24., 8., -side * 4.)),
            Contact::Duck(_)
        ));
        // Crosses the whole body between frames; endpoint-only tests miss it.
        assert!(matches!(
            contact([0.; 3], [0.; 2], shot(side * 30., 8., -side * 60.)),
            Contact::Duck(_)
        ));
        assert_eq!(
            contact([0.; 3], [0.; 2], shot(side * 24., 8., side * 4.)),
            Contact::None
        );
    }
}
#[test]
fn low_attacks_and_large_or_descending_projectiles_choose_shield() {
    assert_eq!(
        contact([0.; 3], [0.; 2], shot(10., 3., -3.)),
        Contact::Shield
    );
    assert_eq!(
        contact(
            [0.; 3],
            [0.; 2],
            Hitbox {
                radius: 6.,
                ..shot(10., 9., -3.)
            }
        ),
        Contact::Shield
    );
    assert_eq!(
        contact(
            [0.; 3],
            [0.; 2],
            Hitbox {
                velocity: [-3., -2.],
                ..shot(10., 10., -3.)
            }
        ),
        Contact::Shield
    );
    assert_eq!(
        contact([0.; 3], [0.; 2], shot(0., CROUCH_TOP + 1., 0.)),
        Contact::Shield
    );
}
#[test]
fn extended_fighter_hitboxes_use_both_capsule_endpoints() {
    assert!(matches!(
        contact(
            [0.; 3],
            [0.; 2],
            Hitbox {
                start: [-20., 8., 0.],
                end: [5., 8., 0.],
                radius: 1.,
                velocity: [0.; 2],
            }
        ),
        Contact::Duck(_)
    ));
    assert_eq!(
        contact(
            [0.; 3],
            [0.; 2],
            Hitbox {
                start: [-20., 8., 0.],
                end: [5., 3., 0.],
                radius: 1.,
                velocity: [0.; 2],
            }
        ),
        Contact::Shield
    );
}
#[test]
fn ignores_invalid_far_above_or_depth_separated_geometry() {
    for hit in [
        shot(0., 50., 0.),
        shot(100., 8., 0.),
        shot(f32::NAN, 8., 0.),
        Hitbox {
            radius: -1.,
            ..shot(0., 8., 0.)
        },
        Hitbox {
            start: [0., 8., 50.],
            end: [0., 8., 50.],
            ..shot(0., 8., 0.)
        },
    ] {
        assert_eq!(contact([0.; 3], [0.; 2], hit), Contact::None);
    }
}
fn context() -> DuckContext {
    DuckContext {
        clock: GuardClock {
            status: 0,
            motion: 123,
            frame: 1.,
            rate: 1.,
            stopped: false,
        },
        can_crouch: true,
        crouched: false,
        can_shield: true,
    }
}
#[test]
fn any_high_hitbox_can_request_duck_without_falcon_name_or_eight_frame_lead() {
    let mut defense = DuckDefense::default();
    let mut threats = DefenseThreat::default();
    threats.add_duck(geometry_threat(0x10000001, 10., 2.));
    threats.add_duck(geometry_threat(2, 2., 1.)); // Simultaneous high melee hit.
    let down = defense.sample(true, context(), threats);
    assert_eq!(down.stick, Some([0., -1.]));
    assert_eq!(down.requested, 0);
    assert_eq!(defense.sample(false, context(), threats).owned, 0);
    assert_eq!(
        defense
            .sample(true, context(), DefenseThreat::default())
            .owned,
        0
    );
}
#[test]
fn low_competing_hit_and_native_readiness_take_priority() {
    use kirby_assist_core::native_input::MAPPER_ZR;
    let threats = DefenseThreat {
        shield: true,
        duck: Some(geometry_threat(1, 10., 1.)),
    };
    assert_eq!(
        DuckDefense::default()
            .sample(true, context(), threats)
            .requested,
        MAPPER_ZR
    );
    let high = DefenseThreat {
        shield: false,
        duck: Some(geometry_threat(1, 10., 1.)),
    };
    let unavailable = DuckContext {
        can_crouch: false,
        can_shield: false,
        ..context()
    };
    assert_eq!(
        DuckDefense::default().sample(true, unavailable, high).owned,
        0
    );
}
