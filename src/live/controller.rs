//! The native controller-memory layout and four-word button-history helpers.
//!
//! `repr(C)`, padding, field order and offset checks describe Smash's ABI (the
//! memory contract with the game). Treat them differently from ordinary Rust
//! structs: moving this code does not permit changing that layout.
//! The mapper hook saves these fields, temporarily overlays them, and restores
//! them after native mapping. This file does not decide which tool wins.
use kirby_assist_core::native_input::ButtonTrace;
use std::ffi::c_void;

const CONTROLLER_POINTER_OFFSET_BYTES: usize = 0x10;
const CONTROLLER_GYRO_REPEAT_BYTES: usize = 0x58;
const CONTROLLER_COLOR_FLAGS_BYTES: usize = 0x17;
const CONTROLLER_TRAILING_FLAGS_BYTES: usize = 8;
const CONTROLLER_AUTOREPEAT_WORDS: usize = 3;
const CONTROLLER_CURRENT_BUTTONS_OFFSET_BYTES: usize = 8;
const CONTROLLER_PREVIOUS_BUTTONS_OFFSET_BYTES: usize = 0xc;
const CONTROLLER_LEFT_STICK_X_OFFSET_BYTES: usize = 0x10;
const CONTROLLER_LEFT_STICK_Y_OFFSET_BYTES: usize = 0x14;
const CONTROLLER_LEFT_TRIGGER_OFFSET_BYTES: usize = 0x18;
const CONTROLLER_RIGHT_STICK_X_OFFSET_BYTES: usize = 0x20;
const CONTROLLER_RIGHT_STICK_Y_OFFSET_BYTES: usize = 0x24;
const CONTROLLER_RIGHT_TRIGGER_OFFSET_BYTES: usize = 0x28;
const CONTROLLER_JUST_DOWN_OFFSET_BYTES: usize = 0x88;
const CONTROLLER_JUST_RELEASE_OFFSET_BYTES: usize = 0x8c;
const CONTROLLER_AUTOREPEAT_OFFSET_BYTES: usize = 0x90;
const CONTROLLER_STYLE_OFFSET_BYTES: usize = 0x9c;
const CONTROLLER_CONNECTED_OFFSET_BYTES: usize = 0xbb;
const CONTROLLER_NPAD_OFFSET_BYTES: usize = 0xc4;
const MAPPED_INPUTS_SIZE_BYTES: usize = 8;
const MAPPED_INPUTS_BUTTONS_OFFSET_BYTES: usize = 0;

// Pinned Controller prefix: overlay button words, stick floats, and affected
// repeat flags as requested by the active tool. Restore them after mapping.
#[repr(C)]
// Padding fields are intentional: they preserve offsets of later native fields.
// Do not remove padding or reorder fields as a normal Rust refactor would.
pub(super) struct Controller {
    pub(super) vtable: usize,
    pub(super) current_buttons: u32,
    pub(super) previous_buttons: u32,
    pub(super) left_stick_x: f32,
    pub(super) left_stick_y: f32,
    pub(super) left_trigger: f32,
    pub(super) left_padding: u32,
    pub(super) right_stick_x: f32,
    pub(super) right_stick_y: f32,
    pub(super) right_trigger: f32,
    pub(super) right_padding: u32,
    // Rest of the native gyro/repeat prefix remains untouched.
    pub(super) sticks_gyro_repeat: [u8; CONTROLLER_GYRO_REPEAT_BYTES],
    pub(super) just_down: u32,
    pub(super) just_release: u32,
    pub(super) autorepeat: [u32; CONTROLLER_AUTOREPEAT_WORDS],
    pub(super) style: u32,
    pub(super) controller_id: u32,
    pub(super) colors_and_flags: [u8; CONTROLLER_COLOR_FLAGS_BYTES],
    pub(super) connected: u8,
    pub(super) trailing_flags: [u8; CONTROLLER_TRAILING_FLAGS_BYTES],
    pub(super) npad: u32,
}
// Compile-time checks catch mistakes in OUR declared layout. They cannot prove
// that an unknown/new game executable uses that same layout.
const _: () = {
    assert!(
        std::mem::offset_of!(Controller, current_buttons)
            == CONTROLLER_CURRENT_BUTTONS_OFFSET_BYTES
    );
    assert!(
        std::mem::offset_of!(Controller, previous_buttons)
            == CONTROLLER_PREVIOUS_BUTTONS_OFFSET_BYTES
    );
    assert!(std::mem::offset_of!(Controller, left_stick_x) == CONTROLLER_LEFT_STICK_X_OFFSET_BYTES);
    assert!(std::mem::offset_of!(Controller, left_stick_y) == CONTROLLER_LEFT_STICK_Y_OFFSET_BYTES);
    assert!(std::mem::offset_of!(Controller, left_trigger) == CONTROLLER_LEFT_TRIGGER_OFFSET_BYTES);
    assert!(
        std::mem::offset_of!(Controller, right_stick_x) == CONTROLLER_RIGHT_STICK_X_OFFSET_BYTES
    );
    assert!(
        std::mem::offset_of!(Controller, right_stick_y) == CONTROLLER_RIGHT_STICK_Y_OFFSET_BYTES
    );
    assert!(
        std::mem::offset_of!(Controller, right_trigger) == CONTROLLER_RIGHT_TRIGGER_OFFSET_BYTES
    );
    assert!(std::mem::offset_of!(Controller, just_down) == CONTROLLER_JUST_DOWN_OFFSET_BYTES);
    assert!(std::mem::offset_of!(Controller, just_release) == CONTROLLER_JUST_RELEASE_OFFSET_BYTES);
    assert!(std::mem::offset_of!(Controller, autorepeat) == CONTROLLER_AUTOREPEAT_OFFSET_BYTES);
    assert!(std::mem::offset_of!(Controller, style) == CONTROLLER_STYLE_OFFSET_BYTES);
    assert!(std::mem::offset_of!(Controller, connected) == CONTROLLER_CONNECTED_OFFSET_BYTES);
    assert!(std::mem::offset_of!(Controller, npad) == CONTROLLER_NPAD_OFFSET_BYTES);
};

/// Read the native wrapper's controller pointer. The caller checks that the
/// wrapper is non-null and valid for this pinned layout before calling.
pub(super) unsafe fn controller_from_wrapper(wrapper: *mut c_void) -> *mut Controller {
    wrapper
        .cast::<u8>()
        .add(CONTROLLER_POINTER_OFFSET_BYTES)
        .cast::<*mut Controller>()
        .read()
}

// Pinned MappedInputs result (common/input.rs). The post-map filter clears
// reserved taunts which may otherwise survive native repeat/history handling.
#[repr(C)]
pub(super) struct MappedInputs {
    pub(super) buttons: u32,
    pub(super) left_x: i8,
    pub(super) left_y: i8,
    pub(super) right_x: i8,
    pub(super) right_y: i8,
}
const _: () = {
    assert!(std::mem::size_of::<MappedInputs>() == MAPPED_INPUTS_SIZE_BYTES);
    assert!(std::mem::offset_of!(MappedInputs, buttons) == MAPPED_INPUTS_BUTTONS_OFFSET_BYTES);
};

// Copy the four physical button words before changing any of them.
pub(super) unsafe fn trace(controller: *const Controller) -> ButtonTrace {
    ButtonTrace {
        held: (*controller).current_buttons,
        previous: (*controller).previous_buttons,
        pressed: (*controller).just_down,
        released: (*controller).just_release,
    }
}
// Write a complete, consistent button history; changing only held would leave
// new-press/release queries inconsistent with what the mapper sees.
pub(super) unsafe fn write_trace(controller: *mut Controller, buttons: ButtonTrace) {
    (*controller).current_buttons = buttons.held;
    (*controller).previous_buttons = buttons.previous;
    (*controller).just_down = buttons.pressed;
    (*controller).just_release = buttons.released;
}
