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
pub const MAPPER_MENU: u32 = (1 << 18) | (1 << 19);

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
    if accessor.is_null() {
        return None;
    }
    // Each read loads the pointer stored at an offset in the previous object.
    // add(offset) alone moves an address; read() follows the stored pointer.
    // A non-null pointer is still not proof of validity: native lifetime/version
    // checks remain the caller's responsibility, as documented in Safety above.
    let control = accessor.add(0x48).cast::<*const u8>().read();
    if control.is_null() {
        return None;
    }
    let first = control.add(0x118).cast::<*const u8>().read();
    if first.is_null() {
        return None;
    }
    let second = first.add(0x58).cast::<*const u8>().read();
    if second.is_null() {
        return None;
    }
    let input = second.add(8).cast::<*const u8>().read();
    if input.is_null() {
        return None;
    }
    let index = input.add(8).cast::<i32>().read();
    (0..8).contains(&index).then_some(index)
}

// Compiled only by cargo test on the host; these assertions are not live
// Switch logging or gameplay callbacks. Add regressions here for new behavior.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routing_follows_ownership_chain_instead_of_control_plus_eight() {
        let mut accessor = [0usize; 10];
        let mut control = [0usize; 36];
        let mut first = [0usize; 12];
        let mut second = [0usize; 2];
        let mut input = [0usize; 2];
        accessor[0x48 / 8] = control.as_mut_ptr() as usize;
        control[1] = 1234; // Old code misread this field as the index.
        control[0x118 / 8] = first.as_mut_ptr() as usize;
        first[0x58 / 8] = second.as_mut_ptr() as usize;
        second[1] = input.as_mut_ptr() as usize;
        input[1] = 3;
        unsafe {
            assert_eq!(controller_index(accessor.as_ptr().cast()), Some(3));
            input.as_mut_ptr().add(1).write(8);
            assert_eq!(controller_index(accessor.as_ptr().cast()), None);
            input.as_mut_ptr().add(1).write(0);
            assert_eq!(controller_index(accessor.as_ptr().cast()), Some(0));
            let accessor_ptr = accessor.as_ptr().cast();
            for (allocation, offset) in [
                (accessor.as_mut_ptr(), 0x48 / 8),
                (control.as_mut_ptr(), 0x118 / 8),
                (first.as_mut_ptr(), 0x58 / 8),
                (second.as_mut_ptr(), 1),
            ] {
                let field = allocation.add(offset);
                let saved = field.read();
                field.write(0);
                assert_eq!(controller_index(accessor_ptr), None);
                field.write(saved);
            }
            assert_eq!(controller_index(std::ptr::null()), None);
        }
    }

    #[test]
    fn mash_edges_alternate_with_physically_held_buttons_then_release() {
        let other = 1 << 8;
        let human = ButtonTrace {
            held: MAPPER_A | MAPPER_B | other,
            previous: MAPPER_A | MAPPER_B,
            pressed: other,
            released: 1 << 9,
        };
        let mut overlay = MapperButtonOverlay::default();
        let a = overlay.apply(human, HID_A, HID_A | HID_B);
        assert_eq!(a.held, MAPPER_A | other);
        let b = overlay.apply(human, HID_B, HID_A | HID_B);
        assert_eq!(b.held, MAPPER_B | other);
        assert_eq!(b.pressed, MAPPER_B | other);
        assert_eq!(b.released, MAPPER_A | (1 << 9));
        assert_eq!(b.previous, MAPPER_A);
        let restored = overlay.apply(human, 0, 0);
        assert_eq!(restored.held, human.held);
        assert_eq!(restored.pressed, MAPPER_A | other);
        assert_eq!(overlay.apply(human, 0, 0), human);
        overlay.reset();
        assert_eq!(overlay.apply(human, 0, 0), human);
    }

    #[test]
    fn unheld_zr_gets_a_press_and_release_without_touching_other_buttons() {
        let mut overlay = MapperButtonOverlay::default();
        let human = ButtonTrace {
            held: MAPPER_A,
            previous: MAPPER_A,
            ..ButtonTrace::default()
        };
        let pressed = overlay.apply(human, HID_ZR, HID_ZR);
        assert_eq!(pressed.held, MAPPER_A | MAPPER_ZR);
        assert_eq!(pressed.pressed, MAPPER_ZR);
        let released = overlay.apply(human, 0, 0);
        assert_eq!(released.held, MAPPER_A);
        assert_eq!(released.released, MAPPER_ZR);
    }
}
