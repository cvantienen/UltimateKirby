//! Existing regression cases for `native_input`. Kept outside the runtime code.
use super::*;

#[test]
fn player_one_follows_its_physical_controller_across_mapper_slots() {
    for mapper in 0..8 {
        assert_eq!(selected_controller_mapper(0, 0, mapper), Some(mapper));
        assert_eq!(selected_controller_mapper(0, 1, mapper), None);
    }
    assert_eq!(selected_controller_mapper(3, 3, 0), Some(0));
    assert_eq!(selected_controller_mapper(0, 0x20, 5), Some(5));
    assert_eq!(selected_controller_mapper(1, 0x20, 0), None);
    assert_eq!(selected_controller_mapper(0, 0x10, 0), None);
    for invalid in [-1, 8, i32::MAX] {
        assert_eq!(selected_controller_mapper(0, 0, invalid), None);
    }
    assert_eq!(selected_controller_mapper(8, 8, 0), None);
}

#[test]
fn missing_or_sentinel_controller_chain_does_not_dereference_invalid_addresses() {
    for address in [0usize, 1, 8, 0xff8, 0x1001] {
        assert_eq!(unsafe { controller_index(address as *const u8) }, None);
    }
    let mut accessor = [0usize; 10];
    for address in [0usize, 1, 8, 0xff8, 0x1001] {
        accessor[0x48 / 8] = address;
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
