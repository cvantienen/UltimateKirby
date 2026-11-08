//! Bowser Up-B ownership used for native tracking observations.
use crate::action_owner::Owner;

/// Fail closed for mixed owners; generic input exclusivity applies to all fighters.
pub fn bowser_up_b_owner(
    kind: Option<i32>,
    owner: Option<Owner>,
    roll: bool,
    option: bool,
    shield: bool,
) -> Option<Owner> {
    let active = match (kind, roll, option, shield) {
        (Some(15), true, false, false) => Owner::RollRead,
        (Some(15), false, true, false) => Owner::OptionRead,
        (Some(15), false, false, true) => Owner::OutOfShield,
        _ => return None,
    };
    (owner.is_none() || owner == Some(active)).then_some(active)
}
