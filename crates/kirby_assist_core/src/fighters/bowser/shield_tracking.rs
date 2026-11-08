//! Bowser's supported opponent-directed Up-B drift in shield backup recipes.
use crate::sequence_config::{Action, Direction};

pub(crate) fn tracking(kind: i32, actions: &[Action]) -> Option<(u8, f32, Direction)> {
    if kind != crate::fighter_kinds::BOWSER || actions.len() != 2 {
        return None;
    }
    let Action::Drift {
        direction,
        strength,
        frames,
    } = actions[1]
    else {
        return None;
    };
    matches!(
        direction,
        Direction::Opponent | Direction::AwayFromOpponent | Direction::Facing | Direction::Backward
    )
    .then_some((frames, strength, direction))
}

pub(crate) fn legacy_tracking_frames(kind: i32) -> u8 {
    if kind == crate::fighter_kinds::BOWSER {
        69
    } else {
        0
    }
}
