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
