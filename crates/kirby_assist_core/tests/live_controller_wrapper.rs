//! Verify the production wrapper reader against the pinned controller offset.
#![allow(dead_code)]
use std::ffi::c_void;

use kirby_assist_core::native_input::{ButtonTrace, HANDHELD_NPAD_ID};

#[path = "../../../src/live/controller.rs"]
mod controller;

#[repr(C)]
struct Wrapper {
    prefix: [u8; 0x10],
    controller: *mut controller::Controller,
}

#[cfg(feature = "diagnostic-logging")]
#[test]
fn diagnostic_menu_consumes_only_reserved_input_and_telemetry_is_read_only() {
    let mut c: controller::Controller = unsafe { std::mem::zeroed() };
    c.current_buttons = 0b1111;
    c.previous_buttons = 0b1011;
    c.just_down = 0b1100;
    c.just_release = 0b0011;
    c.autorepeat = [0b1111, 42, 43];
    c.left_stick_x = 0.5;
    c.right_stick_y = f32::NAN;
    c.left_trigger = 0.25;
    let before = unsafe { controller::trace(&c) };
    let mut sample = unsafe { controller::telemetry_input(&c, 7, 2) };
    assert_eq!(unsafe { controller::trace(&c) }, before);
    assert_eq!(sample.values[0], Some(15));
    assert_eq!(sample.values[4], Some(u64::from(0.5f32.to_bits())));
    assert_eq!(sample.values[7], None);
    let saved = unsafe { controller::consume_diagnostic_menu(&mut c, 0b0101) }.unwrap();
    assert_eq!(c.current_buttons, 0b1010);
    assert_eq!(c.just_down, 0b1000);
    assert_eq!(c.autorepeat, [0b1010, 42, 43]);
    assert_eq!(c.left_stick_x, 0.5);
    assert_eq!(c.left_trigger, 0.25);
    controller::telemetry_mapped(&mut sample, &controller::MappedInputs {
        buttons: 0x1234, left_x: -127, left_y: 42, right_x: 0, right_y: 127,
    });
    assert_eq!(sample.values[11], Some(0x1234));
    assert_eq!(sample.values[12], Some(-127i64 as u64));
    assert_eq!(sample.values[0], Some(15));
    unsafe { saved.restore(); }
    assert_eq!(unsafe { controller::trace(&c) }, before);
    assert_eq!(c.autorepeat, [15, 42, 43]);
    assert!(c.right_stick_y.is_nan());
}

#[test]
fn reads_the_controller_pointer_without_modifying_the_wrapper() {
    let mut sentinel = 0u8;
    let expected = (&mut sentinel as *mut u8).cast::<controller::Controller>();
    let mut wrapper = Wrapper {
        prefix: [0xa5; 0x10],
        controller: expected,
    };
    let actual = unsafe {
        controller::controller_from_wrapper((&mut wrapper as *mut Wrapper).cast::<c_void>())
    };
    assert_eq!(actual, expected);
    assert_eq!(wrapper.prefix, [0xa5; 0x10]);
    assert_eq!(wrapper.controller, expected);
}

#[test]
fn preserves_a_null_controller_pointer() {
    let mut wrapper = Wrapper {
        prefix: [0; 0x10],
        controller: std::ptr::null_mut(),
    };
    assert!(unsafe {
        controller::controller_from_wrapper((&mut wrapper as *mut Wrapper).cast::<c_void>())
    }
    .is_null());
}

#[test]
fn selects_only_the_configured_controller_and_valid_mapper_slot() {
    let mut controller: controller::Controller = unsafe { std::mem::zeroed() };
    controller.npad = 3;
    let mut wrapper = Wrapper {
        prefix: [0; 0x10],
        controller: &mut controller,
    };
    let wrapper_ptr = (&mut wrapper as *mut Wrapper).cast::<c_void>();
    unsafe {
        assert_eq!(
            controller::selected_controller(wrapper_ptr, 5, 3),
            Some(wrapper.controller)
        );
        assert!(controller::selected_controller(wrapper_ptr, 5, 0).is_none());
        assert!(controller::selected_controller(wrapper_ptr, -1, 3).is_none());
        assert!(controller::selected_controller(wrapper_ptr, 8, 3).is_none());
        assert!(controller::selected_controller(std::ptr::null_mut(), 0, 0).is_none());
        controller.npad = HANDHELD_NPAD_ID;
        assert_eq!(controller.npad, HANDHELD_NPAD_ID);
        assert_eq!(
            controller::selected_controller(wrapper_ptr, 5, 0),
            Some(wrapper.controller)
        );
        wrapper.controller = std::ptr::null_mut();
        assert!(wrapper.controller.is_null());
        assert!(controller::selected_controller(wrapper_ptr, 0, 0).is_none());
    }
}

#[test]
fn restores_all_overlaid_controller_fields_without_touching_other_fields() {
    let mut controller: controller::Controller = unsafe { std::mem::zeroed() };
    controller.npad = 7;
    controller.autorepeat = [99, 42, 43];
    let physical = ButtonTrace {
        held: 1,
        previous: 2,
        pressed: 4,
        released: 8,
    };
    let saved = controller::ControllerRestore {
        controller: &mut controller,
        physical,
        stick: [0.25, -0.5],
        right_stick: [-0.75, 1.0],
        autorepeat: 16,
        triggers: [0.3, 0.6],
    };
    unsafe {
        saved.restore();
        assert_eq!(controller::trace(&controller), physical);
    }
    assert_eq!(
        [controller.left_stick_x, controller.left_stick_y],
        [0.25, -0.5]
    );
    assert_eq!(
        [controller.right_stick_x, controller.right_stick_y],
        [-0.75, 1.0]
    );
    assert_eq!(
        [controller.left_trigger, controller.right_trigger],
        [0.3, 0.6]
    );
    assert_eq!(controller.autorepeat, [16, 42, 43]);
    assert_eq!(controller.npad, 7);
}
