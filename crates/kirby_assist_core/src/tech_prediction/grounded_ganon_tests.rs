//! Existing regression cases for `tech_prediction`. Kept outside the runtime code.
use super::*;

#[test]
fn grounded_capture_uses_captor_throw_clock_instead_of_tumble_or_velocity() {
    assert_eq!(
        tech_gate(CATCHED_GANON, 0, false, false),
        TechGate::OutsideTumble
    );
    for rate in [0.5, 1.0, 2.0] {
        assert_eq!(
            grounded_ganon_contact_in(CATCHED_GANON, 0x18, 0x1e2, 31.0 - 4.0 * rate, rate),
            Some(4.0)
        );
    }
    assert_eq!(
        grounded_ganon_contact_in(CATCHED_GANON, 0x18, 0x1e2, 31.0, 1.0),
        Some(0.0)
    );
}

#[test]
fn aerial_slams_other_captors_and_invalid_or_passed_clocks_do_not_predict() {
    for status in [0xbf, 0xc0, 0xc1, 0x42, 0x49] {
        assert_eq!(
            grounded_ganon_contact_in(status, 0x18, 0x1e2, 27.0, 1.0),
            None
        );
    }
    assert_eq!(
        grounded_ganon_contact_in(CATCHED_GANON, 6, 0x1e2, 27.0, 1.0),
        None
    );
    assert_eq!(
        grounded_ganon_contact_in(CATCHED_GANON, 0x18, 0x1e3, 27.0, 1.0),
        None
    );
    for (frame, rate) in [
        (32.0, 1.0),
        (-1.0, 1.0),
        (f32::NAN, 1.0),
        (27.0, 0.0),
        (27.0, f32::INFINITY),
    ] {
        assert_eq!(
            grounded_ganon_contact_in(CATCHED_GANON, 0x18, 0x1e2, frame, rate),
            None
        );
    }
}
