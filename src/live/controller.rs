//! The native controller-memory layout and four-word button-history helpers.
//!
//! `repr(C)`, padding, field order and offset checks describe Smash's ABI (the
//! memory contract with the game). Treat them differently from ordinary Rust
//! structs: moving this code does not permit changing that layout.
//! The mapper hook saves these fields, temporarily overlays them, and restores
//! them after native mapping. This file does not decide which tool wins.
use kirby_assist_core::native_input::ButtonTrace;

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
    // Rest of the native axes/gyro/repeat prefix remains untouched.
    pub(super) sticks_gyro_repeat: [u8; 0x60],
    pub(super) just_down: u32,
    pub(super) just_release: u32,
    pub(super) autorepeat: [u32; 3],
    pub(super) style: u32,
    pub(super) controller_id: u32,
    pub(super) colors_and_flags: [u8; 0x17],
    pub(super) connected: u8,
    pub(super) trailing_flags: [u8; 8],
    pub(super) npad: u32,
}
// Compile-time checks catch mistakes in OUR declared layout. They cannot prove
// that an unknown/new game executable uses that same layout.
const _: () = {
    assert!(std::mem::offset_of!(Controller, current_buttons) == 8);
    assert!(std::mem::offset_of!(Controller, previous_buttons) == 0xc);
    assert!(std::mem::offset_of!(Controller, left_stick_x) == 0x10);
    assert!(std::mem::offset_of!(Controller, left_stick_y) == 0x14);
    assert!(std::mem::offset_of!(Controller, right_stick_x) == 0x20);
    assert!(std::mem::offset_of!(Controller, right_stick_y) == 0x24);
    assert!(std::mem::offset_of!(Controller, autorepeat) == 0x90);
    assert!(std::mem::offset_of!(Controller, just_down) == 0x88);
    assert!(std::mem::offset_of!(Controller, just_release) == 0x8c);
    assert!(std::mem::offset_of!(Controller, style) == 0x9c);
    assert!(std::mem::offset_of!(Controller, connected) == 0xbb);
    assert!(std::mem::offset_of!(Controller, npad) == 0xc4);
};

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
    assert!(std::mem::size_of::<MappedInputs>() == 8);
    assert!(std::mem::offset_of!(MappedInputs, buttons) == 0);
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
