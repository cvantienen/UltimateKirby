//! Smash's raw controller buttons, before user-configured mapping.
//! Layout/accessor source: HDR a8f98eace1c6a7b45c88cb3f0ac1ef05426624dc,
//! dynamic/src/ext.rs (get_player_idx_from_boma and Controller).
//!
//! This is the final conversion layer: HID-named request bits become Smash's
//! internal button positions, then merge with a fresh physical ButtonTrace.
//! Game mapping remains responsible for turning physical buttons into actions.
//! To support another button, verify its native position, add a conversion, and
//! test held/pressed/released behavior plus preservation of unrelated inputs.
use crate::live_input::{HID_A, HID_B, HID_L, HID_R, HID_X, HID_Y, HID_ZL, HID_ZR};

pub const MAPPER_A: u32 = 1 << 5;
pub const MAPPER_B: u32 = 1 << 6;
pub const MAPPER_X: u32 = 1 << 4;
pub const MAPPER_Y: u32 = 1 << 7;
pub const MAPPER_L: u32 = 1 << 8;
pub const MAPPER_R: u32 = 1 << 9;
pub const MAPPER_ZL: u32 = 1 << 10;
pub const MAPPER_ZR: u32 = 1 << 11;
pub const MAPPER_PLUS: u32 = 1 << 18;
pub const MAPPER_MINUS: u32 = 1 << 19;
pub const MAPPER_MENU: u32 = MAPPER_PLUS | MAPPER_MINUS;
// Controller::ButtonBitfield r_right/r_left in the pinned HDR layout.
// A C-stick flick includes these digital aliases as well as its analog axis.
pub const MAPPER_RIGHT_STICK_RIGHT: u32 = 1 << 25;
pub const MAPPER_RIGHT_STICK_LEFT: u32 = 1 << 27;

/// Bind the configured physical controller to its current game mapper slot.
/// Controller identity and mapper slot are separate namespaces: controller 0
/// can occupy another mapper slot after entering a different battle lobby.
pub fn selected_controller_mapper(
    selected_player: u32,
    controller_npad: u32,
    mapper_index: i32,
) -> Option<i32> {
    // libnx hid.h: No1..No8 = 0..7; Handheld = 0x20. Handheld is the
    // console's local Player 1 input, rather than an additional battle slot.
    let local_player = if controller_npad == 0x20 {
        0
    } else {
        controller_npad
    };
    (selected_player < 8 && local_player == selected_player && (0..8).contains(&mapper_index))
        .then_some(mapper_index)
}

// Conversion is per button, not a cast: the two bitfields use different orders.
// Unsupported request bits naturally produce no native bits.
pub fn mapper_buttons(hid: u64) -> u32 {
    (u32::from(hid & HID_A != 0) * MAPPER_A)
        | (u32::from(hid & HID_B != 0) * MAPPER_B)
        | (u32::from(hid & HID_X != 0) * MAPPER_X)
        | (u32::from(hid & HID_Y != 0) * MAPPER_Y)
        | (u32::from(hid & HID_L != 0) * MAPPER_L)
        | (u32::from(hid & HID_R != 0) * MAPPER_R)
        | (u32::from(hid & HID_ZL != 0) * MAPPER_ZL)
        | (u32::from(hid & HID_ZR != 0) * MAPPER_ZR)
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
// All four words describe one view of the same input. `pressed` is an edge
// (off -> on), whereas `held` stays on for as long as a button remains pressed.
pub struct ButtonTrace {
    pub held: u32,
    pub previous: u32,
    pub pressed: u32,
    pub released: u32,
}

/// Changes only assist-owned bits and their edge history. The caller supplies
/// a fresh physical trace each mapping call and restores it after mapping.
#[derive(Default)]
pub struct MapperButtonOverlay {
    last_output: Option<u32>,
    last_affected: u32,
}
impl MapperButtonOverlay {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    // `human` must come from the freshly read physical structure, not our last
    // returned trace. The native hook saves/restores that structure around the
    // original mapping call; this helper only returns an ordinary Rust value.
    pub fn apply(&mut self, human: ButtonTrace, requested: u64, owned: u64) -> ButtonTrace {
        let requested = mapper_buttons(requested);
        let owned = mapper_buttons(owned);
        // Include the previous update's affected bits for one final release or
        // hand-back to human input when the scenario ends. Otherwise a synthetic
        // press could disappear without the corresponding release edge.
        self.apply_raw(human, requested, owned)
    }

    /// Already-converted native masks for manual D-pad/stick ownership. Callers
    /// must limit masks to their own controls; use apply() for HID-named requests.
    pub fn apply_raw(&mut self, human: ButtonTrace, requested: u32, owned: u32) -> ButtonTrace {
        // requested = bits to turn ON; owned = bits automation may replace,
        // including turning them OFF. Request=0 with owned=ZR releases ZR even
        // when the human holds it; request=0/owned=0 hands it back to the human.
        let affected = requested | owned | self.last_affected;
        // & !mask clears only owned bits, then | adds requested presses.
        // Buttons outside the mask are copied unchanged from the human trace.
        let held = (human.held & !owned) | requested;
        let previous =
            (human.previous & !affected) | (self.last_output.unwrap_or(human.previous) & affected);
        // Rebuild edges only for affected buttons; unrelated native edge words
        // remain untouched. last_output supplies the previous GAME-visible state.
        // For an affected bit: pressed = held now AND not held previously;
        // released = held previously AND not held now. A continuous held bit
        // has neither edge, which is why repeatedly writing "held" isn't turbo.
        let result = ButtonTrace {
            held,
            previous,
            pressed: (human.pressed & !affected) | (held & !previous & affected),
            released: (human.released & !affected) | (previous & !held & affected),
        };
        self.last_output = Some(held);
        self.last_affected = requested | owned;
        result
    }
}

/// Resolve the input-mapper index through the game's controller ownership chain.
/// A ControlModule's +8 field is NOT its controller index.
///
/// # Safety
/// `accessor` and every non-null pointer in this game-version-specific chain
/// must point to live, sufficiently large, aligned native allocations.
pub unsafe fn controller_index(accessor: *const u8) -> Option<i32> {
    if !native_pointer_address(accessor) {
        return None;
    }
    // Each read loads the pointer stored at an offset in the previous object.
    // add(offset) alone moves an address; read() follows the stored pointer.
    // A non-null pointer is still not proof of validity: native lifetime/version
    // checks remain the caller's responsibility, as documented in Safety above.
    let control = accessor.add(0x48).cast::<*const u8>().read();
    if !native_pointer_address(control) {
        return None;
    }
    let first = control.add(0x118).cast::<*const u8>().read();
    if !native_pointer_address(first) {
        return None;
    }
    let second = first.add(0x58).cast::<*const u8>().read();
    if !native_pointer_address(second) {
        return None;
    }
    let input = second.add(8).cast::<*const u8>().read();
    if !native_pointer_address(input) {
        return None;
    }
    let index = input.add(8).cast::<i32>().read();
    (0..8).contains(&index).then_some(index)
}

// These checks reject null, small sentinel and misaligned addresses. They do
// NOT prove mapped memory or object lifetime; callers must validate the live
// native callback identity/readiness before entering this chain.
fn native_pointer_address(pointer: *const u8) -> bool {
    let address = pointer as usize;
    address >= 0x1000 && address.is_multiple_of(std::mem::align_of::<usize>())
}

// Compiled only by cargo test on the host; these assertions are not live
// Switch logging or gameplay callbacks. Add regressions here for new behavior.
#[cfg(test)]
mod tests;
