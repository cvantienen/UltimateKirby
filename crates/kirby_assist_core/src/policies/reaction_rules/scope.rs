//! Cheap admission/observation gates. No scans, native writes or recovery waits.

/// Controllable grounded movement, ordinary landing, or native ready shield.
/// A cancel flag alone never admits a new punish during an attack or dodge.
/// Plain English: Controllable grounded movement, ordinary landing, or native ready shield. A cancel flag alone never admits a new punish during an attack or dodge.
pub fn automatic_start_status(status: i32) -> bool {
    matches!(status,crate::native_states::status::WAIT..=crate::native_states::status::TURN_RUN_BRAKE|crate::native_states::status::SQUAT..=crate::native_states::status::LANDING_LIGHT|crate::native_states::GUARD_ON|crate::native_states::GUARD|crate::native_states::status::OTTOTTO|crate::native_states::status::OTTOTTO_WAIT)
}
/// Technical: Determines whether the `automatic_start_allowed` condition holds.
///
/// Plain English: Checks whether automatic start allowed.
pub fn automatic_start_allowed(status: i32, situation: i32, stopped: bool, linked: bool) -> bool {
    situation == crate::native_states::situation::GROUND
        && !stopped
        && !linked
        && automatic_start_status(status)
}
/// Grab attempts may be defended/punished. Pull/hold/pummel/throw and victims
/// already in capture are not ordinary automatic-punish targets.
/// Plain English: Grab attempts may be defended/punished. Pull/hold/pummel/throw and victims already in capture are not ordinary automatic-punish targets.
pub fn automatic_target_available(status: i32) -> bool {
    status >= crate::native_states::status::WAIT
        && !crate::native_states::captured_or_scripted(status)
        && !matches!(status,crate::native_states::status::CATCH_PULL|crate::native_states::status::CATCH_DASH_PULL|crate::native_states::status::CATCH_WAIT..=crate::native_states::status::THROW|crate::native_states::status::CLIFF_CATCH_MOVE..=crate::native_states::status::CLIFF_JUMP3|crate::native_states::status::DEAD|crate::native_states::status::REBIRTH)
}
/// An existing authored sequence may keep reading its locked target through
/// ledge invulnerability; it cannot acquire or retarget an attack there.
/// Plain English: An existing authored sequence may keep reading its locked target through ledge invulnerability; it cannot acquire or retarget an attack there.
pub fn automatic_follow_target_available(status: i32) -> bool {
    automatic_target_available(status)
        || (crate::native_states::status::CLIFF_CATCH_MOVE
            ..=crate::native_states::status::CLIFF_JUMP3)
            .contains(&status)
}
/// Technical: Determines whether the `automatic_up_b_status` condition holds.
///
/// Plain English: Checks whether automatic up b status.
pub fn automatic_up_b_status(kind: i32, status: i32, _previous: i32) -> bool {
    crate::native_states::up_special_status(kind, status)
}

/// Passive receipt observation is grounded and cannot admit an attack. A dodge
/// cancel receipt belongs only to the already assisted own spot dodge.
/// Plain English: Passive receipt observation is grounded and cannot admit an attack. A dodge cancel receipt belongs only to the already assisted own spot dodge.
pub fn automatic_up_b_receipt_status(status: i32, situation: i32, assisted_dodge: bool) -> bool {
    situation == crate::native_states::situation::GROUND
        && (automatic_start_status(status)
            || matches!(
                status,
                crate::native_states::GUARD_OFF | crate::native_states::GUARD_DAMAGE
            )
            || assisted_dodge && status == crate::native_states::ESCAPE)
}

/// Technical: Defines the `AutomaticObservationScope` data structure.
///
/// Plain English: Holds the data for AutomaticObservationScope.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AutomaticObservationScope {
    /// Technical: Stores the `acquire` value in `AutomaticObservationScope` as `bool`.
    ///
    /// Plain English: Holds the acquire value used by AutomaticObservationScope.
    pub acquire: bool,
    /// Technical: Stores the `follow` value in `AutomaticObservationScope` as `bool`.
    ///
    /// Plain English: Holds the follow value used by AutomaticObservationScope.
    pub follow: bool,
}
impl AutomaticObservationScope {
    /// Technical: Constructs a new value with the type’s initial state.
    ///
    /// Plain English: Creates a new value.
    pub fn new(
        suspended: bool,
        status: i32,
        situation: i32,
        stopped: bool,
        linked: bool,
        assisted_dodge: bool,
        committed_sequence: bool,
    ) -> Self {
        if suspended || linked {
            return Self::default();
        }
        Self {
            acquire: automatic_start_allowed(status, situation, stopped, false),
            follow: assisted_dodge
                && situation == crate::native_states::situation::GROUND
                && status == crate::native_states::ESCAPE
                || committed_sequence
                    && automatic_target_available(status)
                    && !crate::action_owner::native_grab_active(status)
                    && !crate::ledge_trap::sequence_interrupted(status)
                    && !matches!(
                        status,
                        crate::native_states::ESCAPE..=crate::native_states::ESCAPE_AIR_SLIDE
                    ),
        }
    }
    /// Technical: Returns the `with_up_b_receipt` result.
    ///
    /// Plain English: Gets the with up b receipt result.
    pub fn with_up_b_receipt(
        mut self,
        suspended: bool,
        linked: bool,
        receipt: bool,
        status: i32,
        situation: i32,
        assisted_dodge: bool,
    ) -> Self {
        if suspended || linked {
            return Self::default();
        }
        self.follow |= receipt && automatic_up_b_receipt_status(status, situation, assisted_dodge);
        self
    }
    /// Technical: Observes the targets.
    ///
    /// Plain English: Observes the targets.
    pub fn observe_targets(self) -> bool {
        self.acquire || self.follow
    }
}

/// Technical: Determines whether the `automatic_fighter_target_available` condition holds.
///
/// Plain English: Checks whether automatic fighter target available.
pub fn automatic_fighter_target_available(kind: i32, status: i32) -> bool {
    automatic_target_available(status) && !crate::native_states::fighter_noncombat(kind, status)
}
