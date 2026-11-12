//! The native controller-memory layout and four-word button-history helpers.
//!
//! `repr(C)`, padding, field order and offset checks describe Smash's ABI (the
//! memory contract with the game). Treat them differently from ordinary Rust
//! structs: moving this code does not permit changing that layout.
//! The mapper hook saves these fields, temporarily overlays them, and restores
//! them after native mapping. This file does not decide which tool wins.
//! Technical: Models the pinned controller prefix and copies/restores input data
//! around the native mapping call.
//! Plain English: It temporarily applies assistance while keeping the player's
//! actual controller state intact.
use kirby_assist_core::native_input::{selected_controller_mapper, ButtonTrace, MAPPER_SLOT_COUNT};
use std::ffi::c_void;

/// Byte offset of the controller pointer inside the mapper wrapper.
const CONTROLLER_POINTER_OFFSET_BYTES: usize = 0x10;
/// Size of the opaque native gyro/repeat prefix.
const CONTROLLER_GYRO_REPEAT_BYTES: usize = 0x58;
/// Size of the native controller color/flag byte group.
const CONTROLLER_COLOR_FLAGS_BYTES: usize = 0x17;
/// Number of trailing native flag bytes before the NPad ID.
const CONTROLLER_TRAILING_FLAGS_BYTES: usize = 8;
/// Number of auto-repeat words represented by the native prefix.
const CONTROLLER_AUTOREPEAT_WORDS: usize = 3;
/// Expected byte offset of the current held-button word.
const CONTROLLER_CURRENT_BUTTONS_OFFSET_BYTES: usize = 8;
/// Expected byte offset of the previous held-button word.
const CONTROLLER_PREVIOUS_BUTTONS_OFFSET_BYTES: usize = 0xc;
/// Expected byte offset of the left-stick X value.
const CONTROLLER_LEFT_STICK_X_OFFSET_BYTES: usize = 0x10;
/// Expected byte offset of the left-stick Y value.
const CONTROLLER_LEFT_STICK_Y_OFFSET_BYTES: usize = 0x14;
/// Expected byte offset of the left-trigger value.
const CONTROLLER_LEFT_TRIGGER_OFFSET_BYTES: usize = 0x18;
/// Expected byte offset of the right-stick X value.
const CONTROLLER_RIGHT_STICK_X_OFFSET_BYTES: usize = 0x20;
/// Expected byte offset of the right-stick Y value.
const CONTROLLER_RIGHT_STICK_Y_OFFSET_BYTES: usize = 0x24;
/// Expected byte offset of the right-trigger value.
const CONTROLLER_RIGHT_TRIGGER_OFFSET_BYTES: usize = 0x28;
/// Expected byte offset of the native just-pressed mask.
const CONTROLLER_JUST_DOWN_OFFSET_BYTES: usize = 0x88;
/// Expected byte offset of the native just-released mask.
const CONTROLLER_JUST_RELEASE_OFFSET_BYTES: usize = 0x8c;
/// Expected byte offset of native auto-repeat history.
const CONTROLLER_AUTOREPEAT_OFFSET_BYTES: usize = 0x90;
/// Expected byte offset of the native controller-style field.
const CONTROLLER_STYLE_OFFSET_BYTES: usize = 0x9c;
/// Expected byte offset of the native connection flag.
const CONTROLLER_CONNECTED_OFFSET_BYTES: usize = 0xbb;
/// Expected byte offset of the native NPad identifier.
const CONTROLLER_NPAD_OFFSET_BYTES: usize = 0xc4;
/// Expected native result size for `MappedInputs`.
const MAPPED_INPUTS_SIZE_BYTES: usize = 8;
/// Expected byte offset of mapped button bits.
const MAPPED_INPUTS_BUTTONS_OFFSET_BYTES: usize = 0;

/// Pinned Controller prefix: overlay button words, stick floats, and affected
/// repeat flags as requested by the active tool. Restore them after mapping.
#[repr(C)]
/// Padding fields are intentional: they preserve offsets of later native fields.
/// Do not remove padding or reorder fields as a normal Rust refactor would.
/// Native controller memory prefix pinned to the game ABI.
pub(super) struct Controller {
    /// Native vtable pointer at byte offset zero.
    pub(super) vtable: usize,
    /// Current held-button bitfield.
    pub(super) current_buttons: u32,
    /// Previous poll's held-button bitfield.
    pub(super) previous_buttons: u32,
    /// Left-stick horizontal axis in the native controller layout.
    pub(super) left_stick_x: f32,
    /// Left-stick vertical axis in the native controller layout.
    pub(super) left_stick_y: f32,
    /// Left analog-trigger value.
    pub(super) left_trigger: f32,
    /// Padding word preserving the pinned right-stick offset.
    pub(super) left_padding: u32,
    /// Right-stick horizontal axis in the native controller layout.
    pub(super) right_stick_x: f32,
    /// Right-stick vertical axis in the native controller layout.
    pub(super) right_stick_y: f32,
    /// Right analog-trigger value.
    pub(super) right_trigger: f32,
    /// Padding word preserving the pinned native field offsets.
    pub(super) right_padding: u32,
    // Rest of the native gyro/repeat prefix remains untouched.
    /// Opaque native gyro/repeat bytes between analog inputs and edge history.
    pub(super) sticks_gyro_repeat: [u8; CONTROLLER_GYRO_REPEAT_BYTES],
    /// Buttons newly pressed in the current poll.
    pub(super) just_down: u32,
    /// Buttons released in the current poll.
    pub(super) just_release: u32,
    /// Native auto-repeat history words.
    pub(super) autorepeat: [u32; CONTROLLER_AUTOREPEAT_WORDS],
    /// Native controller style identifier.
    pub(super) style: u32,
    /// Native controller/player ID value.
    pub(super) controller_id: u32,
    /// Opaque native color and flag bytes.
    pub(super) colors_and_flags: [u8; CONTROLLER_COLOR_FLAGS_BYTES],
    /// Native connected-state byte.
    pub(super) connected: u8,
    /// Opaque trailing flag bytes before the NPad identifier.
    pub(super) trailing_flags: [u8; CONTROLLER_TRAILING_FLAGS_BYTES],
    /// Native NPad identifier used to validate the controller route.
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

/// Reads the native wrapper's controller pointer. The caller checks that the
/// wrapper is non-null and valid for this pinned layout before calling.
/// Loads the controller pointer stored at the wrapper's pinned offset.
/// # Safety
/// `wrapper` must point to a readable native wrapper using the documented layout.
pub(super) unsafe fn controller_from_wrapper(wrapper: *mut c_void) -> *mut Controller {
    wrapper
        .cast::<u8>()
        .add(CONTROLLER_POINTER_OFFSET_BYTES)
        .cast::<*mut Controller>()
        .read()
}

/// Selects the appropriate controller based on the wrapper, index, and player index.
/// Bounds-checks the mapper slot and validates the wrapper's controller
/// against the configured player-to-mapper route.
/// # Safety
/// `wrapper` must point to a readable native wrapper for the pinned game ABI.
pub(super) unsafe fn selected_controller(
    wrapper: *mut c_void,
    index: i32,
    player_index: u32,
) -> Option<*mut Controller> {
    if wrapper.is_null() || !(0..MAPPER_SLOT_COUNT).contains(&index) {
        return None;
    }
    let controller = controller_from_wrapper(wrapper);
    if controller.is_null()
        || selected_controller_mapper(player_index, (*controller).npad, index) != Some(index)
    {
        return None;
    }
    Some(controller)
}

/// Stores the state of a controller for later restoration, including button traces, stick positions, and trigger values.
/// Owns a snapshot of all input fields temporarily overwritten by mapping.
pub(super) struct ControllerRestore {
    /// Native controller pointer whose temporary overlay must be reverted.
    pub(super) controller: *mut Controller,
    /// Copied held/previous/pressed/released button history.
    pub(super) physical: ButtonTrace,
    /// Copied left-stick X/Y values.
    pub(super) stick: [f32; 2],
    /// Copied right-stick X/Y values.
    pub(super) right_stick: [f32; 2],
    /// First auto-repeat history word changed by the overlay.
    pub(super) autorepeat: u32,
    /// Copied left/right analog trigger values.
    pub(super) triggers: [f32; 2],
}

impl ControllerRestore {
    /// Restores the controller to the previously saved state.
    /// Writes the owned snapshot back to each overlaid native input field.
    /// # Safety
    /// The controller pointer must still refer to the same live native controller.
    pub(super) unsafe fn restore(self) {
        let controller = self.controller;
        write_trace(controller, self.physical);
        (*controller).left_stick_x = self.stick[0];
        (*controller).left_stick_y = self.stick[1];
        (*controller).right_stick_x = self.right_stick[0];
        (*controller).right_stick_y = self.right_stick[1];
        (*controller).autorepeat[0] = self.autorepeat;
        (*controller).left_trigger = self.triggers[0];
        (*controller).right_trigger = self.triggers[1];
    }
}

/// Pinned MappedInputs result (common/input.rs). The post-map filter clears
/// reserved taunts which may otherwise survive native repeat/history handling.
#[repr(C)]
/// Native mapper result layout copied from the pinned input ABI.
pub(super) struct MappedInputs {
    /// Mapped canonical button bitfield returned by the native mapper.
    pub(super) buttons: u32,
    /// Mapped left-stick horizontal axis.
    pub(super) left_x: i8,
    /// Mapped left-stick vertical axis.
    pub(super) left_y: i8,
    /// Mapped right-stick horizontal axis.
    pub(super) right_x: i8,
    /// Mapped right-stick vertical axis.
    pub(super) right_y: i8,
}
const _: () = {
    assert!(std::mem::size_of::<MappedInputs>() == MAPPED_INPUTS_SIZE_BYTES);
    assert!(std::mem::offset_of!(MappedInputs, buttons) == MAPPED_INPUTS_BUTTONS_OFFSET_BYTES);
};

/// Copies the four physical button words before changing any of them.
/// Reads the held, previous, pressed, and released native button masks.
/// # Safety
/// `controller` must point to a valid controller with the pinned layout.
pub(super) unsafe fn trace(controller: *const Controller) -> ButtonTrace {
    ButtonTrace {
        held: (*controller).current_buttons,
        previous: (*controller).previous_buttons,
        pressed: (*controller).just_down,
        released: (*controller).just_release,
    }
}
/// Writes a complete, consistent button history; changing only held would leave
/// new-press/release queries inconsistent with what the mapper sees.
/// Writes all four related button masks from one trace.
/// # Safety
/// `controller` must point to a writable controller with the pinned layout.
pub(super) unsafe fn write_trace(controller: *mut Controller, buttons: ButtonTrace) {
    (*controller).current_buttons = buttons.held;
    (*controller).previous_buttons = buttons.previous;
    (*controller).just_down = buttons.pressed;
    (*controller).just_release = buttons.released;
}
