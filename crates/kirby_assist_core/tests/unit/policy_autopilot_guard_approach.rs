use super::*;
/// Verified fair startup warns, while unknown motions remain unpredicted.
#[test]
fn verified_fair_startup_is_warned_and_unknown_motion_is_not_invented() {
    let m = crate::timing::motion_hash("attack_air_f");
    assert!(referenced_imminent_attack(5, m, 12.0, 1.0));
    assert!(!referenced_imminent_attack(5, m, 1.0, 1.0));
    assert!(!referenced_imminent_attack(5, 0, 12.0, 1.0));
    assert!(!referenced_imminent_attack(5, m, f32::NAN, 1.0));
}
/// Extended and descending hitboxes are detected even with distant roots.
#[test]
fn extended_and_descending_hitboxes_are_seen_without_close_roots() {
    let long = Hitbox {
        start: [8.0, 10.0, 0.0],
        end: [35.0, 10.0, 0.0],
        radius: 3.0,
        velocity: [-1.0, 0.0],
    };
    assert!(active_hitbox_threat([0.0; 3], [0.0; 2], long));
    let falling = Hitbox {
        start: [0.0, 40.0, 0.0],
        end: [0.0, 40.0, 0.0],
        radius: 3.0,
        velocity: [0.0, -5.0],
    };
    assert!(active_hitbox_threat([0.0; 3], [0.0; 2], falling));
    assert!(!active_hitbox_threat(
        [0.0; 3],
        [0.0; 2],
        Hitbox {
            velocity: [0.0, 5.0],
            ..falling
        }
    ));
    assert!(!active_hitbox_threat(
        [0.0; 3],
        [0.0; 2],
        Hitbox {
            start: [0.0, 10.0, 40.0],
            end: [0.0, 10.0, 40.0],
            ..long
        }
    ));
}
