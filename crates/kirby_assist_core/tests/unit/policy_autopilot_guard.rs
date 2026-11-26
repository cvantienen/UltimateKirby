use super::*;
/// Approaching and crossing attacks warn; distant retreats do not.
#[test]
fn checks_incoming_and_crossing_attacks_without_reacting_to_distant_retreats() {
    assert!(approaching_active_attack(
        [0.; 2],
        [0.; 2],
        [20., 0.],
        [-2., 0.]
    ));
    assert!(approaching_active_attack(
        [0.; 2],
        [0.; 2],
        [50., 0.],
        [-50., 0.]
    ));
    assert!(!approaching_active_attack(
        [0.; 2],
        [0.; 2],
        [30., 0.],
        [2., 0.]
    ));
    assert!(!approaching_active_attack(
        [0.; 2],
        [0.; 2],
        [0., 50.],
        [0., 0.]
    ));
    assert!(!approaching_active_attack(
        [f32::NAN, 0.],
        [0.; 2],
        [0.; 2],
        [0.; 2]
    ));
}
