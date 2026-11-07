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

#[test]
fn shared_ganon_release_requires_grounded_capture_predecessor() {
    assert!(grounded_ganon_release(CATCHED_AIR_END_GANON, CATCHED_GANON));
    for previous in [0xbf, 0xc0, 0xc1, 0x49, 0] {
        assert!(!grounded_ganon_release(CATCHED_AIR_END_GANON, previous));
    }
    assert!(!grounded_ganon_release(0xbf, CATCHED_GANON));
}

#[test]
fn late_damage_stop_has_bounded_lead_and_rejects_unknown_timers() {
    for remaining in 1..=5 {
        assert!(late_damage_stop_prediction(remaining, 0.0));
    }
    for remaining in [0, 6, 120, u64::MAX] {
        assert!(!late_damage_stop_prediction(remaining, 0.0));
    }
    assert!(!late_damage_stop_prediction(2, f32::NAN));
}
