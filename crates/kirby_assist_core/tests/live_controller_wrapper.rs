//! Verify the production wrapper reader against the pinned controller offset.
#![allow(dead_code)]
use std::ffi::c_void;

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
