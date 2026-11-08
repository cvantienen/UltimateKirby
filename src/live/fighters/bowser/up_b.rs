//! Bowser-specific observation admission for a committed Up-B.
use kirby_assist_core::{fighter_kinds::BOWSER, reaction_rules::automatic_up_b_status};

pub(super) fn spin_follow(
    kind: i32,
    committed: bool,
    receipt: bool,
    status: i32,
    previous: i32,
) -> bool {
    kind == BOWSER && committed && (receipt || automatic_up_b_status(kind, status, previous))
}

pub(super) fn input_epoch(kind: i32, epoch: u64) -> Option<u64> {
    (kind == BOWSER).then_some(epoch)
}

pub(super) fn retain_rules(kind: i32, direct_up_b: bool) -> bool {
    kind == BOWSER && direct_up_b
}

pub(super) fn reserve_receipt(kind: Option<i32>, waiting: bool) -> bool {
    kind == Some(BOWSER) && waiting
}
