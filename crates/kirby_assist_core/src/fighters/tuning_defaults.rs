//! Fighter-specific visibility and legacy display defaults.
use crate::fighter_tuning::{
    self as tuning, COUNT, CROUCH_HEIGHT, CROUCH_RANGE,
    PUNISH_DOWN_REACH, PUNISH_FORWARD_REACH, REFLECT_DEFAULT, REFLECT_HALF_DEPTH, ROLL_GRAB_WATCH,
};

pub(crate) fn visible(kind: i32, index: usize) -> bool {
    if index >= COUNT {
        return false;
    }
    if (REFLECT_DEFAULT..=REFLECT_HALF_DEPTH).contains(&index) {
        return crate::projectile_reflect::supports(kind);
    }
    if (PUNISH_DOWN_REACH..=ROLL_GRAB_WATCH).contains(&index) {
        return false;
    }
    if (CROUCH_RANGE..=CROUCH_HEIGHT).contains(&index) {
        return kind == crate::fighter_kinds::KIRBY;
    }
    true
}

pub(crate) fn tilt_turn_down(kind: i32) -> bool {
    matches!(
        kind,
        crate::fighter_kinds::KIRBY | crate::fighter_kinds::YOSHI
    )
}

pub(crate) fn legacy_ranges(kind: i32, values: &mut [f32; tuning::COUNT]) {
    values[PUNISH_DOWN_REACH] = if kind == crate::fighter_kinds::KIRBY {
        14.0
    } else if kind == crate::fighter_kinds::YOSHI {
        13.0
    } else {
        12.0
    };
    values[PUNISH_FORWARD_REACH] = if kind == crate::fighter_kinds::KIRBY {
        22.0
    } else {
        21.0
    };
    values[ROLL_GRAB_WATCH] = if kind == crate::fighter_kinds::KIRBY {
        32.0
    } else {
        36.0
    };
}
