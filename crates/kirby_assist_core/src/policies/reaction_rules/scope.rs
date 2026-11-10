//! Cheap admission/observation gates. No scans, native writes or recovery waits.

/// Controllable grounded movement, ordinary landing, or native ready shield.
/// A cancel flag alone never admits a new punish during an attack or dodge.
///
///  Controllable grounded movement, ordinary landing, or native ready shield. A cancel flag alone never admits a new punish during an attack or dodge.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn automatic_start_status(status: i32) -> bool {
    matches!(status,crate::native_states::status::WAIT..=crate::native_states::status::TURN_RUN_BRAKE|crate::native_states::status::SQUAT..=crate::native_states::status::LANDING_LIGHT|crate::native_states::GUARD_ON|crate::native_states::GUARD|crate::native_states::status::OTTOTTO|crate::native_states::status::OTTOTTO_WAIT)
}
/// Determines whether the `automatic_start_allowed` condition holds.
///
/// Checks whether automatic start allowed.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// - `situation` (`i32`): Represents the situation; the function uses it as input to the operation described above.
/// - `stopped` (`bool`): Represents the stopped; the function uses it as input to the operation described above.
/// - `linked` (`bool`): Represents the linked; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn automatic_start_allowed(status: i32, situation: i32, stopped: bool, linked: bool) -> bool {
    situation == crate::native_states::situation::GROUND
        && !stopped
        && !linked
        && automatic_start_status(status)
}
/// Grab attempts may be defended/punished. Pull/hold/pummel/throw and victims
/// already in capture are not ordinary automatic-punish targets.
///
///  Grab attempts may be defended/punished. Pull/hold/pummel/throw and victims already in capture are not ordinary automatic-punish targets.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn automatic_target_available(status: i32) -> bool {
    status >= crate::native_states::status::WAIT
        && !crate::native_states::captured_or_scripted(status)
        && !matches!(status,crate::native_states::status::CATCH_PULL|crate::native_states::status::CATCH_DASH_PULL|crate::native_states::status::CATCH_WAIT..=crate::native_states::status::THROW|crate::native_states::status::CLIFF_CATCH_MOVE..=crate::native_states::status::CLIFF_JUMP3|crate::native_states::status::DEAD|crate::native_states::status::REBIRTH)
}
/// An existing authored sequence may keep reading its locked target through
/// ledge invulnerability; it cannot acquire or retarget an attack there.
///
///  An existing authored sequence may keep reading its locked target through ledge invulnerability; it cannot acquire or retarget an attack there.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn automatic_follow_target_available(status: i32) -> bool {
    automatic_target_available(status)
        || (crate::native_states::status::CLIFF_CATCH_MOVE
            ..=crate::native_states::status::CLIFF_JUMP3)
            .contains(&status)
}
/// Determines whether the `automatic_up_b_status` condition holds.
///
/// Checks whether automatic up b status.
/// ### Arguments
/// - `kind` (`i32`): Represents the kind or category being checked; the function uses it as input to the operation described above.
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// - `_previous` (`i32`): Represents the previous; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn automatic_up_b_status(kind: i32, status: i32, _previous: i32) -> bool {
    crate::native_states::up_special_status(kind, status)
}

/// Passive receipt observation is grounded and cannot admit an attack. A dodge
/// cancel receipt belongs only to the already assisted own spot dodge.
///
///  Passive receipt observation is grounded and cannot admit an attack. A dodge cancel receipt belongs only to the already assisted own spot dodge.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// - `situation` (`i32`): Represents the situation; the function uses it as input to the operation described above.
/// - `assisted_dodge` (`bool`): Represents the assisted dodge; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn automatic_up_b_receipt_status(status: i32, situation: i32, assisted_dodge: bool) -> bool {
    situation == crate::native_states::situation::GROUND
        && (automatic_start_status(status)
            || matches!(
                status,
                // Guard states: off means the character is not actively guarding. 
                // Guard damage indicates the character has been hit while guarding.
                crate::native_states::GUARD_OFF | crate::native_states::GUARD_DAMAGE
            )
            // Check for assisted dodge condition. 
            || assisted_dodge && status == crate::native_states::ESCAPE)
}

/// Defines the `AutomaticObservationScope` data structure.
///
/// Holds the data for AutomaticObservationScope.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AutomaticObservationScope {
    /// Stores the `acquire` value in `AutomaticObservationScope` as `bool`.
    ///
    ///
///  Holds the acquire value used by AutomaticObservationScope.
    pub acquire: bool,
    /// Stores the `follow` value in `AutomaticObservationScope` as `bool`.
    ///
    ///
///  Holds the follow value used by AutomaticObservationScope.
    pub follow: bool,
}
impl AutomaticObservationScope {
    /// Constructs a new value with the type’s initial state.
    ///
    ///
    ///  Creates a new value.
    /// ### Arguments
    /// - `suspended` (`bool`): Represents the suspended; the function uses it as input to the operation described above.
    /// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
    /// - `situation` (`i32`): Represents the situation; the function uses it as input to the operation described above.
    /// - `stopped` (`bool`): Represents the stopped; the function uses it as input to the operation described above.
    /// - `linked` (`bool`): Represents the linked; the function uses it as input to the operation described above.
    /// - `assisted_dodge` (`bool`): Represents the assisted dodge; the function uses it as input to the operation described above.
    /// - `committed_sequence` (`bool`): Represents the committed sequence; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns a `Self` value representing the result of the operation.
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
    /// Returns the `with_up_b_receipt` result.
    ///
    ///
    ///  Gets the with up b receipt result.
    /// ### Arguments
    /// - `self` (`self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `suspended` (`bool`): Represents the suspended; the function uses it as input to the operation described above.
    /// - `linked` (`bool`): Represents the linked; the function uses it as input to the operation described above.
    /// - `receipt` (`bool`): Represents the receipt; the function uses it as input to the operation described above.
    /// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
    /// - `situation` (`i32`): Represents the situation; the function uses it as input to the operation described above.
    /// - `assisted_dodge` (`bool`): Represents the assisted dodge; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns a `Self` value representing the result of the operation.
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
    /// Observes the targets.
    ///
    ///
    ///  Observes the targets.
    /// ### Arguments
    /// - `self` (`self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn observe_targets(self) -> bool {
        self.acquire || self.follow
    }
}

/// Determines whether the `automatic_fighter_target_available` condition holds.
///
/// Checks whether automatic fighter target available.
/// ### Arguments
/// - `kind` (`i32`): Represents the kind or category being checked; the function uses it as input to the operation described above.
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn automatic_fighter_target_available(kind: i32, status: i32) -> bool {
    automatic_target_available(status) && !crate::native_states::fighter_noncombat(kind, status)
}
