use kirby_assist_core::{
    action_owner::{exclusive_output, safety_output_with_deadline, Owner},
    native_input::{MAPPED_GUARD, MAPPER_ZR},
    tech_input::{mapped_guard, TechContext, TechInput, TechOutput},
};
fn context() -> TechContext {
    TechContext {
        object: 1,
        status: kirby_assist_core::native_states::status::DAMAGE_FLY,
        motion: 1,
        frame: 0.,
        rate: 1.,
        stopped: false,
        remaining_stop: 0,
        contact_in: Some(1.),
        input_delay: 0.,
        previous_guard: false,
        prepare_guard: true,
    }
}
fn sample(input: &mut TechInput, c: TechContext, native_guard: bool) -> TechOutput {
    input.sample_with_native_delivery(true, true, 28, c, None, 12, 7, native_guard)
}
#[test]
fn native_held_guard_must_release_even_when_the_last_mapped_packet_was_neutral() {
    let mut input = TechInput::default();
    let c = context();
    for _ in 0..12 {
        assert_eq!(
            sample(&mut input, c, true),
            TechOutput {
                press: false,
                owns_guard: true
            }
        );
    }
    assert!(sample(&mut input, c, false).press);
    assert_eq!(input.cooldown_remaining(), 0.);
}
#[test]
fn mapped_press_remains_owned_until_the_fighter_consumes_guard_then_di_receives_the_release() {
    let mut input = TechInput::default();
    let c = context();
    let first = sample(&mut input, c, false);
    assert!(first.press);
    input.mapped_press();
    for _ in 0..40 {
        let held = sample(&mut input, c, false);
        assert!(held.press && held.owns_guard);
        assert!(input.urgent_press());
        let (packet, ack) = safety_output_with_deadline(
            held,
            None,
            Some([-1., 0.]),
            Some([1., 0.]),
            input.urgent_press(),
        );
        let packet = exclusive_output(Some(Owner::Safety), true, packet);
        assert_eq!(packet.requested, MAPPER_ZR);
        assert_eq!(packet.stick, Some([0.; 2]));
        assert_eq!(mapped_guard(0, ack.unwrap()) & MAPPED_GUARD, MAPPED_GUARD);
        input.mapped_press(); // Repeated output is not a second fresh edge.
        assert_eq!(input.cooldown_remaining(), 28.);
    }
    let release = sample(&mut input, c, true);
    assert!(!release.press);
    assert!(!input.urgent_press());
    let (packet, ack) = safety_output_with_deadline(
        release,
        None,
        Some([-1., 0.]),
        Some([1., 0.]),
        input.urgent_press(),
    );
    assert_eq!(packet.requested, 0);
    assert_eq!(packet.stick, Some([-1., 0.]));
    assert_eq!(ack, None);
}
#[test]
fn an_unconsumed_mapped_press_expires_on_native_progress_instead_of_mapper_poll_count() {
    let mut input = TechInput::default();
    let c = context();
    assert!(sample(&mut input, c, false).press);
    input.mapped_press();
    for frame in 0..3 {
        for _ in 0..30 {
            assert!(
                sample(
                    &mut input,
                    TechContext {
                        frame: frame as f32,
                        ..c
                    },
                    false
                )
                .press
            );
        }
    }
    let expired = sample(&mut input, TechContext { frame: 3., ..c }, false);
    assert!(!expired.press);
    assert!(!input.urgent_press());
    assert_eq!(input.cooldown_remaining(), 25.);
}
#[test]
fn a_deadline_displaces_di_only_for_an_actual_tech_press_and_never_mixes_inputs() {
    let mut input = TechInput::default();
    let c = context();
    for (contact, urgent) in [(7., false), (3.25, true)] {
        let out = sample(
            &mut input,
            TechContext {
                stopped: true,
                remaining_stop: 2,
                contact_in: Some(contact),
                ..c
            },
            false,
        );
        assert!(out.press);
        assert_eq!(input.urgent_press(), urgent);
        let (packet, ack) =
            safety_output_with_deadline(out, None, Some([-1., 0.]), Some([1., 0.]), urgent);
        if urgent {
            assert_eq!(packet.requested, MAPPER_ZR);
            assert_eq!(packet.stick, None);
            assert_eq!(ack, Some(out));
        } else {
            assert_eq!(packet.requested, 0);
            assert_eq!(packet.stick, Some([-1., 0.]));
            assert_eq!(ack, None);
        }
    }
    let (packet, ack) = safety_output_with_deadline(
        TechOutput {
            press: false,
            owns_guard: true,
        },
        None,
        Some([-1., 0.]),
        Some([1., 0.]),
        true,
    );
    assert_eq!(packet.requested, 0);
    assert_eq!(packet.stick, Some([-1., 0.]));
    assert_eq!(ack, None);
}
#[test]
fn no_contact_recovery_disable_and_cancel_invalidate_the_delivery_receipt() {
    let c = context();
    for next in [
        TechContext {
            contact_in: None,
            ..c
        },
        TechContext {
            status: kirby_assist_core::native_states::status::FALL,
            ..c
        },
        TechContext {
            prepare_guard: false,
            contact_in: None,
            ..c
        },
    ] {
        let mut input = TechInput::default();
        assert!(sample(&mut input, c, false).press);
        input.mapped_press();
        let out = sample(&mut input, next, false);
        assert!(!out.press && !out.owns_guard);
        assert!(
            !sample(&mut input, c, false).press,
            "old request must not resume after a cancellation"
        );
    }
    let mut input = TechInput::default();
    sample(&mut input, c, false);
    input.mapped_press();
    input.cancel_request();
    assert!(!input.urgent_press());
    assert!(!sample(&mut input, c, false).press);
    assert_eq!(input.cooldown_remaining(), 28.);
    let mut input = TechInput::default();
    sample(&mut input, c, false);
    input.mapped_press();
    assert_eq!(
        input.sample_with_native_delivery(false, true, 28, c, None, 12, 7, false),
        TechOutput::default()
    );
    assert!(!sample(&mut input, c, false).press);
}
#[test]
fn confirmed_native_guard_prevents_replayed_edges_across_frozen_poses_and_hitlag_exit() {
    let mut input = TechInput::default();
    let c = TechContext {
        stopped: true,
        remaining_stop: 0,
        motion: 0,
        frame: f32::NAN,
        rate: 0.,
        ..context()
    };
    assert!(sample(&mut input, c, false).press);
    input.mapped_press();
    for _ in 0..30 {
        assert!(sample(&mut input, c, false).press);
    }
    assert!(!sample(&mut input, c, true).press);
    let flight = TechContext {
        motion: 2,
        frame: 0.,
        rate: 1.,
        stopped: false,
        ..context()
    };
    assert!(!sample(&mut input, flight, true).press);
    assert!(!sample(&mut input, flight, false).press);
}
#[test]
fn capture_mash_remains_exclusive_even_at_an_imminent_tech_deadline() {
    let (packet, ack) = safety_output_with_deadline(
        TechOutput {
            press: true,
            owns_guard: true,
        },
        Some([0., 1.]),
        Some([-1., 0.]),
        Some([1., 0.]),
        true,
    );
    assert_eq!(packet.requested, 0);
    assert_eq!(packet.stick, Some([0., 1.]));
    assert_eq!(ack, None);
}
#[test]
fn lockout_is_synchronized_to_native_consumption_even_after_a_delayed_mapper_edge() {
    let mut input = TechInput::default();
    let c = context();
    sample(&mut input, c, false);
    input.mapped_press();
    assert!(sample(&mut input, TechContext { frame: 1., ..c }, false).press);
    assert_eq!(input.cooldown_remaining(), 27.);
    assert!(!sample(&mut input, TechContext { frame: 2., ..c }, true).press);
    assert_eq!(
        input.cooldown_remaining(),
        28.,
        "native consumption starts the real input interval"
    );
    sample(&mut input, TechContext { frame: 3., ..c }, false);
    assert_eq!(input.cooldown_remaining(), 27.);
}
