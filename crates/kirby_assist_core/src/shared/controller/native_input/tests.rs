//! Existing regression cases for `native_input`. Kept outside the runtime code.
use super::*;

#[test]
fn brother_layout_translates_actions_without_turning_grab_or_special_into_triggers() {
    use crate::manual_controls::TURBO_TRIGGER;
    let layout = ControlLayout::Brother;
    assert_eq!(layout.policy_buttons(MAPPER_A), MAPPER_A); // physical A = Attack
    assert_eq!(layout.policy_buttons(MAPPER_X), MAPPER_R); // physical X = Grab
    assert_eq!(layout.policy_buttons(MAPPER_L), MAPPER_B); // physical L = Special
    assert_eq!(layout.policy_buttons(MAPPER_ZL), MAPPER_ZR); // physical ZL = Shield
    assert_eq!(layout.policy_buttons(MAPPER_R), 0); // native Jump, not X shortcut
    assert_eq!(layout.policy_buttons(MAPPER_ZR), 0); // native Jump, not X shortcut
    assert_eq!(layout.policy_buttons(TURBO_TRIGGER), MAPPER_L); // L3 = ledge assist
    assert_eq!(layout.physical_buttons(MAPPER_R), MAPPER_X);
    assert_eq!(layout.physical_buttons(MAPPER_B), MAPPER_L);
    assert_eq!(layout.physical_buttons(MAPPER_ZR), MAPPER_ZL);
    assert_eq!(
        layout.physical_buttons(MAPPER_X | MAPPER_ZL),
        MAPPER_R | MAPPER_ZR
    );
    assert_eq!(layout.physical_buttons(MAPPER_L), TURBO_TRIGGER);
    assert_eq!(layout.physical_buttons(MAPPER_A), MAPPER_A);
    assert_eq!(
        layout.physical_buttons(crate::move_input::SHORT_HOP_ATTACK),
        MAPPER_R | MAPPER_A,
        "Brother short-hop Nair must remain Jump+neutral Attack"
    );
    let directions_and_menu = MAPPER_DPAD_BUTTONS | MAPPER_PLUS | MAPPER_MINUS | MAPPER_Y;
    assert_eq!(
        layout.policy_buttons(directions_and_menu),
        directions_and_menu
    );
    assert_eq!(
        layout.physical_buttons(directions_and_menu),
        directions_and_menu
    );
}

#[test]
fn player_one_follows_its_physical_controller_across_mapper_slots() {
    for mapper in 0..MAPPER_SLOT_COUNT {
        assert_eq!(selected_controller_mapper(0, 0, mapper), Some(mapper));
        assert_eq!(selected_controller_mapper(0, 1, mapper), None);
    }
    assert_eq!(selected_controller_mapper(3, 3, 0), Some(0));
    assert_eq!(selected_controller_mapper(0, HANDHELD_NPAD_ID, 5), Some(5));
    assert_eq!(selected_controller_mapper(1, HANDHELD_NPAD_ID, 0), None);
    assert_eq!(selected_controller_mapper(0, 0x10, 0), None);
    for invalid in [-1, 8, i32::MAX] {
        assert_eq!(selected_controller_mapper(0, 0, invalid), None);
    }
    assert_eq!(selected_controller_mapper(8, 8, 0), None);
}

#[test]
fn missing_or_sentinel_controller_chain_does_not_dereference_invalid_addresses() {
    let pointer_size = std::mem::size_of::<usize>();
    for address in [
        0usize,
        1,
        pointer_size,
        MINIMUM_NATIVE_POINTER_ADDRESS - pointer_size,
        MINIMUM_NATIVE_POINTER_ADDRESS + 1,
    ] {
        assert_eq!(unsafe { controller_index(address as *const u8) }, None);
    }
    let mut accessor = [0usize; 10];
    for address in [
        0usize,
        1,
        pointer_size,
        MINIMUM_NATIVE_POINTER_ADDRESS - pointer_size,
        MINIMUM_NATIVE_POINTER_ADDRESS + 1,
    ] {
        accessor[MODULE_ACCESSOR_CONTROL_MODULE_POINTER_OFFSET_BYTES / pointer_size] = address;
        assert_eq!(unsafe { controller_index(accessor.as_ptr().cast()) }, None);
    }
}

#[test]
fn routing_follows_ownership_chain_instead_of_control_plus_eight() {
    let mut accessor = [0usize; 10];
    let mut control = [0usize; 36];
    let mut first = [0usize; 12];
    let mut second = [0usize; 2];
    let mut input = [0usize; 2];
    let pointer_size = std::mem::size_of::<usize>();
    accessor[MODULE_ACCESSOR_CONTROL_MODULE_POINTER_OFFSET_BYTES / pointer_size] =
        control.as_mut_ptr() as usize;
    control[CONTROLLER_RECORD_INDEX_POINTER_OFFSET_BYTES / pointer_size] = 1234;
    control[CONTROL_MODULE_INPUT_CONTEXT_POINTER_OFFSET_BYTES / pointer_size] =
        first.as_mut_ptr() as usize;
    first[INPUT_CONTEXT_CONTROLLER_RECORD_POINTER_OFFSET_BYTES / pointer_size] =
        second.as_mut_ptr() as usize;
    second[INPUT_INDEX_FIELD_OFFSET_BYTES / pointer_size] = input.as_mut_ptr() as usize;
    input[INPUT_INDEX_FIELD_OFFSET_BYTES / pointer_size] = 3;
    unsafe {
        assert_eq!(controller_index(accessor.as_ptr().cast()), Some(3));
        input
            .as_mut_ptr()
            .add(INPUT_INDEX_FIELD_OFFSET_BYTES / pointer_size)
            .write(MAPPER_SLOT_COUNT as usize);
        assert_eq!(controller_index(accessor.as_ptr().cast()), None);
        input
            .as_mut_ptr()
            .add(INPUT_INDEX_FIELD_OFFSET_BYTES / pointer_size)
            .write(0);
        assert_eq!(controller_index(accessor.as_ptr().cast()), Some(0));
        let accessor_ptr = accessor.as_ptr().cast();
        for (allocation, offset) in [
            (
                accessor.as_mut_ptr(),
                MODULE_ACCESSOR_CONTROL_MODULE_POINTER_OFFSET_BYTES / pointer_size,
            ),
            (
                control.as_mut_ptr(),
                CONTROL_MODULE_INPUT_CONTEXT_POINTER_OFFSET_BYTES / pointer_size,
            ),
            (
                first.as_mut_ptr(),
                INPUT_CONTEXT_CONTROLLER_RECORD_POINTER_OFFSET_BYTES / pointer_size,
            ),
            (
                second.as_mut_ptr(),
                INPUT_INDEX_FIELD_OFFSET_BYTES / pointer_size,
            ),
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

#[test]
fn cstick_flick_has_a_fresh_direction_edge_and_clean_release() {
    use crate::manual_controls::RIGHT_STICK_DIRECTIONS;
    for direction in [MAPPER_RIGHT_STICK_LEFT, MAPPER_RIGHT_STICK_RIGHT] {
        let mut overlay = MapperButtonOverlay::default();
        let flick = overlay.apply_raw(ButtonTrace::default(), direction, RIGHT_STICK_DIRECTIONS);
        assert_eq!(flick.held, direction);
        assert_eq!(flick.pressed, direction);
        let jump = overlay.apply_raw(
            ButtonTrace::default(),
            MAPPER_X | MAPPER_ZL,
            RIGHT_STICK_DIRECTIONS | MAPPER_X | MAPPER_ZL,
        );
        assert_eq!(jump.held & RIGHT_STICK_DIRECTIONS, 0);
        assert_eq!(jump.released & RIGHT_STICK_DIRECTIONS, direction);
        assert_eq!(jump.pressed & (MAPPER_X | MAPPER_ZL), MAPPER_X | MAPPER_ZL);
    }
}

#[test]
fn mapped_a_hold_survives_filtered_frames_without_touching_other_buttons() {
    let unrelated = MAPPED_SPECIAL | MAPPED_JUMP | MAPPED_GUARD | MAPPED_TAUNT_DOWN;
    let mut previous = 0;
    let mut press_edges = 0;
    for frame in 0..=60 {
        let requested = frame < 60;
        // Model a mapper that reports the trigger only on the first poll.
        let native = unrelated
            | if frame == 0 {
                MAPPED_ATTACK_HOLD_BITS
            } else {
                0
            };
        let mapped = mapped_attack_hold(native, requested);
        assert_eq!(
            mapped & MAPPED_ATTACK_HOLD_BITS,
            if requested {
                MAPPED_ATTACK_HOLD_BITS
            } else {
                0
            }
        );
        assert_eq!(mapped & !MAPPED_ATTACK_HOLD_BITS, unrelated);
        if mapped & MAPPED_ATTACK != 0 && previous & MAPPED_ATTACK == 0 {
            press_edges += 1;
        }
        previous = mapped;
    }
    assert_eq!(press_edges, 1);
    assert_eq!(mapped_attack_hold(0xffff, false), 0xffff);
}

#[test]
fn full_gameplay_ownership_covers_unused_raw_buttons_in_both_layouts() {
    for layout in [ControlLayout::Default, ControlLayout::Brother] {
        let owned = layout.physical_owned_buttons(GAMEPLAY_BUTTONS);
        assert_eq!(owned & GAMEPLAY_BUTTONS, GAMEPLAY_BUTTONS);
        assert_eq!(
            owned & (MAPPER_MENU | crate::manual_controls::LEFT_STICK_DIRECTIONS),
            0
        );
    }
    assert_eq!(
        ControlLayout::Brother.physical_owned_buttons(MAPPER_ZR),
        MAPPER_ZL
    );
}
