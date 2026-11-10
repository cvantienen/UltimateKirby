//! Confirmed shield-contact responses. Inputs only; never force native states.
use crate::{
    gameplay::{ControllerOutput, Opponent},
    roll_read::Target as ObservedTarget,
};
mod automatic;
mod simple;
pub use automatic::{AutomaticConfig, AutomaticLimits};
mod shield_up_b;

/// Returns the `copied_target` result.
///
/// Gets the copied target result.
/// ### Arguments
/// - `target` (`&ObservedTarget`): Represents the target object or value being checked; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `Opponent` value.
fn copied_target(target: &ObservedTarget) -> Opponent {
    target.opponent(crate::native_states::situation::GROUND, None)
}

/// The locked target: live follow facts first, else the copied snapshot.
///
///  The locked target: live follow facts first, else the copied snapshot.
/// ### Arguments
/// - `id` (`Option<u32>`): Represents the identifier of an object or entry; the function uses it as input to the operation described above.
/// - `follow` (`Option<Opponent>`): Represents the follow; the function uses it as input to the operation described above.
/// - `targets` (`&[Option<ObservedTarget>; 8]`): Represents the targets; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Some(value)` when a value is available; otherwise, returns `None`.
pub(super) fn locked_enemy(
    id: Option<u32>,
    follow: Option<Opponent>,
    targets: &[Option<ObservedTarget>; 8],
) -> Option<Opponent> {
    follow.filter(|e| Some(e.id) == id).or_else(|| {
        targets
            .iter()
            .flatten()
            .find(|t| Some(t.id) == id)
            .map(copied_target)
    })
}

/// The horizontally nearest acceptable fighter; snapshot targets precede `follow`.
///
///  The horizontally nearest acceptable fighter; snapshot targets precede follow.
/// ### Arguments
/// - `origin` (`f32`): Represents the origin; the function uses it as input to the operation described above.
/// - `targets` (`&[Option<ObservedTarget>; 8]`): Represents the targets; the function uses it as input to the operation described above.
/// - `follow` (`Option<Opponent>`): Represents the follow; the function uses it as input to the operation described above.
/// - `accept` (`impl FnMut(Opponent) -> bool`): Represents the accept; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `Some(value)` when a value is available; otherwise, returns `None`.
pub(super) fn nearest_enemy(
    origin: f32,
    targets: &[Option<ObservedTarget>; 8],
    follow: Option<Opponent>,
    mut accept: impl FnMut(Opponent) -> bool,
) -> Option<Opponent> {
    targets
        .iter()
        .flatten()
        .map(copied_target)
        .chain(follow)
        .filter(|e| accept(*e))
        .min_by(|a, b| {
            (a.position[0] - origin)
                .abs()
                .total_cmp(&(b.position[0] - origin).abs())
        })
}

/// Determines whether the `target_within_range` condition holds.
///
/// Checks whether target within range.
/// ### Arguments
/// - `origin` (`[f32; 2]`): Represents the origin; the function uses it as input to the operation described above.
/// - `target` (`[f32; 2]`): Represents the target object or value being checked; the function uses it as input to the operation described above.
/// - `range` (`f32`): Represents the range; the function uses it as input to the operation described above.
/// - `height` (`f32`): Represents the height; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub(super) fn target_within_range(
    origin: [f32; 2],
    target: [f32; 2],
    range: f32,
    height: f32,
) -> bool {
    range.is_finite()
        && range > 0.
        && height.is_finite()
        && height >= 0.
        && origin
            .iter()
            .chain(target.iter())
            .all(|value| value.is_finite())
        && (target[0] - origin[0]).abs() <= range
        && (target[1] - origin[1]).abs() <= height
}

/// Retained stationary gate for incoming-grab avoidance and legacy reactions.
///
///  Retained stationary gate for incoming-grab avoidance and legacy reactions.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn stationary_grab_state(status: i32) -> bool {
    matches!(
        status,
        crate::native_states::status::WAIT
            | crate::native_states::GUARD_ON
            | crate::native_states::GUARD
    )
}
/// Ground locomotion, crouch and ordinary landing may grab as soon as native
/// transitions permit it. Attack recovery, shield drop/stun and air stay out.
///
///  Ground locomotion, crouch and ordinary landing may grab as soon as native transitions permit it. Attack recovery, shield drop/stun and air stay out.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn ground_grab_state(status: i32) -> bool {
    matches!(
        status,
        crate::native_states::status::WAIT..=crate::native_states::status::TURN_RUN_BRAKE | crate::native_states::status::SQUAT..=crate::native_states::status::LANDING_LIGHT | crate::native_states::GUARD_ON | crate::native_states::GUARD
    )
}
/// Existing idle assistance retains its one delivered approach pulse. Already
/// moving/landing actors send Grab immediately, without first easing the stick.
///
///  Existing idle assistance retains its one delivered approach pulse. Already moving/landing actors send Grab immediately, without first easing the stick.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// - `velocity_x` (`f32`): Represents the velocity x; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn automatic_grab_needs_approach(status: i32, velocity_x: f32) -> bool {
    status == crate::native_states::status::WAIT
        && velocity_x.is_finite()
        && velocity_x.abs() <= 0.01
}
/// Held guard permits direct shield grab. Ground movement can use standing or
/// dash-catch readiness; no pivot input or native status is forced.
///
///  Held guard permits direct shield grab. Ground movement can use standing or dash-catch readiness; no pivot input or native status is forced.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// - `enabled` (`impl FnMut(i32) -> bool`): Represents whether the feature or option should be enabled; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn automatic_grab_ready(status: i32, enabled: impl FnMut(i32) -> bool) -> bool {
    automatic_grab_ready_with_shield(
        status,
        matches!(
            status,
            crate::native_states::GUARD_ON | crate::native_states::GUARD
        ),
        enabled,
    )
}
/// A fresh physical shield press counts before the native status catches up;
/// its direct grab must never borrow dash-catch readiness from locomotion.
///
///  A fresh physical shield press counts before the native status catches up; its direct grab must never borrow dash-catch readiness from locomotion.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// - `shielded` (`bool`): Represents the shielded; the function uses it as input to the operation described above.
/// - `enabled` (`impl FnMut(i32) -> bool`): Represents whether the feature or option should be enabled; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn automatic_grab_ready_with_shield(
    status: i32,
    shielded: bool,
    mut enabled: impl FnMut(i32) -> bool,
) -> bool {
    ground_grab_state(status)
        && (status == crate::native_states::GUARD
            || enabled(crate::native_states::transition_terms::CONT_CATCH)
            || !shielded
                && !matches!(
                    status,
                    crate::native_states::GUARD_ON | crate::native_states::GUARD
                )
                && enabled(crate::native_states::transition_terms::CONT_CATCH_DASH))
}
/// Unshielded automatic grabs require observed enemy shield startup/hold/damage.
/// Native vulnerability, capture links, front side and range are separate gates.
///
///  Unshielded automatic grabs require observed enemy shield startup/hold/damage. Native vulnerability, capture links, front side and range are separate gates.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn automatic_grab_target_status(status: i32) -> bool {
    matches!(
        status,
        crate::native_states::GUARD_ON
            | crate::native_states::GUARD
            | crate::native_states::GUARD_DAMAGE
    )
}
/// Shield grabs may punish ordinary hittable opponents without enemy shield.
/// Downed/tripped bodies cannot be ordinary-grab targets; shield Up-B may hit
/// them instead. Current vulnerability/capture links and reach are checked too.
///
///  Shield grabs may punish ordinary hittable opponents without enemy shield. Downed/tripped bodies cannot be ordinary-grab targets; shield Up-B may hit them instead. Current vulnerability/capture links and reach are checked too.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// - `shielded` (`bool`): Represents the shielded; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn automatic_grab_target_status_with_shield(status: i32, shielded: bool) -> bool {
    if shielded {
        automatic_shield_target_status(status)
            && !crate::native_states::ordinary_grab_immune(status)
    } else {
        automatic_grab_target_status(status)
    }
}
/// Up-B may hit a vulnerable downed opponent that cannot be grabbed. Keep
/// captures, ledge visits and noncombat states outside the ordinary OOS policy.
///
///  Up-B may hit a vulnerable downed opponent that cannot be grabbed. Keep captures, ledge visits and noncombat states outside the ordinary OOS policy.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn automatic_shield_target_status(status: i32) -> bool {
    crate::reaction_rules::automatic_target_available(status)
}

/// Keep guard for a direct OOS input, rather than requesting shield release and
/// the attack together. Movement/ordinary attacks keep the tape's neutral drop.
///
///  Keep guard for a direct OOS input, rather than requesting shield release and the attack together. Movement/ordinary attacks keep the tape's neutral drop.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// - `out` (`ControllerOutput`): Represents the out; the function uses it as input to the operation described above.
/// - `right` (`[f32; 2]`): Represents the right; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `ControllerOutput` value.
pub(crate) fn direct_shield_input(
    status: i32,
    mut out: ControllerOutput,
    right: [f32; 2],
) -> ControllerOutput {
    use crate::native_input::*;
    if matches!(
        status,
        crate::native_states::GUARD_ON | crate::native_states::GUARD
    ) && out.stick.is_some_and(|s| {
        out.requested & MAPPER_R != 0 && s[1] >= 0.
            || out.requested & SEQUENCE_JUMP_BUTTONS != 0 && s[1] >= 0.
            || out.requested & MAPPER_A != 0 && s[1] >= 0.95
            || out.requested & MAPPER_B != 0 && s[1] > 0.5
            || right[1] > 0.5
    }) {
        out.requested |= MAPPER_ZR;
    }
    out
}

/// Defines the `Target` data structure.
///
/// Holds the data for Target.
#[derive(Clone, Copy)]
pub struct Target {
    /// Stores the `id` value in `Target` as `u32`.
    ///
    ///
    ///  Holds the id value used by Target.
    pub id: u32,
    /// Stores the `position` value in `Target` as `[f32; 2]`.
    ///
    ///
    ///  Stores the position collection for Target.
    pub position: [f32; 2],
    /// Stores the `velocity` value in `Target` as `[f32; 2]`.
    ///
    ///
    ///  Stores the velocity collection for Target.
    pub velocity: [f32; 2],
    /// Stores the `grounded` value in `Target` as `bool`.
    ///
    ///
    ///  Holds the grounded value used by Target.
    pub grounded: bool,
    /// Stores the `dangerous` value in `Target` as `bool`.
    ///
    ///
    ///  Holds the dangerous value used by Target.
    pub dangerous: bool,
    /// Stores the `recovery` value in `Target` as `Option<f32>`.
    ///
    ///
    ///  Optionally stores the recovery for Target; it may be absent.
    pub recovery: Option<f32>,
}
/// Defines the `Context` data structure.
///
/// Holds the data for Context.
#[derive(Clone, Copy)]
pub struct Context {
    /// Stores the `object` value in `Context` as `u32`.
    ///
    ///
    ///  Holds the object value used by Context.
    pub object: u32,
    /// Stores the `kind` value in `Context` as `i32`.
    ///
    ///
    ///  Holds the kind value used by Context.
    pub kind: i32,
    /// Stores the `status` value in `Context` as `i32`.
    ///
    ///
    ///  Holds the status value used by Context.
    pub status: i32,
    /// Stores the `motion` value in `Context` as `u64`.
    ///
    ///
    ///  Holds the motion value used by Context.
    pub motion: u64,
    /// Stores the `frame` value in `Context` as `f32`.
    ///
    ///
    ///  Holds the frame value used by Context.
    pub frame: f32,
    /// Stores the `rate` value in `Context` as `f32`.
    ///
    ///
    ///  Holds the rate value used by Context.
    pub rate: f32,
    /// Stores the `stopped` value in `Context` as `bool`.
    ///
    ///
    ///  Holds the stopped value used by Context.
    pub stopped: bool,
    /// Stores the `grounded` value in `Context` as `bool`.
    ///
    ///
    ///  Holds the grounded value used by Context.
    pub grounded: bool,
    /// Stores the `position` value in `Context` as `[f32; 2]`.
    ///
    ///
    ///  Stores the position collection for Context.
    pub position: [f32; 2],
    /// Stores the `facing` value in `Context` as `f32`.
    ///
    ///
    ///  Holds the facing value used by Context.
    pub facing: f32,
    /// Stores the `shield_ready` value in `Context` as `bool`.
    ///
    ///
    ///  Holds the shield ready value used by Context.
    pub shield_ready: bool,
    /// Stores the `can_grab` value in `Context` as `bool`.
    ///
    ///
    ///  Holds the can grab value used by Context.
    pub can_grab: bool,
    /// Stores the `can_jump` value in `Context` as `bool`.
    ///
    ///
    ///  Holds the can jump value used by Context.
    pub can_jump: bool,
    /// Stores the `can_up_special` value in `Context` as `bool`.
    ///
    ///
    ///  Holds the can up special value used by Context.
    pub can_up_special: bool,
    /// Stores the `threatened` value in `Context` as `bool`.
    ///
    ///
    ///  Holds the threatened value used by Context.
    pub threatened: bool,
}
/// Defines the `OutOfShield` data structure.
///
/// Holds the data for OutOfShield.
#[derive(Default)]
pub struct OutOfShield {
    /// Stores the `object` value in `OutOfShield` as `Option<u32>`.
    ///
    ///
    ///  Optionally stores the object for OutOfShield; it may be absent.
    object: Option<u32>,
    /// Stores the `kind` value in `OutOfShield` as `Option<i32>`.
    ///
    ///
    ///  Optionally stores the kind for OutOfShield; it may be absent.
    kind: Option<i32>,
    /// Stores the `target` value in `OutOfShield` as `Option<u32>`.
    ///
    ///
    ///  Optionally stores the target for OutOfShield; it may be absent.
    target: Option<u32>,
    /// Stores the `guarded` value in `OutOfShield` as `bool`.
    ///
    ///
    ///  Holds the guarded value used by OutOfShield.
    guarded: bool,
    /// Stores the `response` value in `OutOfShield` as `crate::defense_response::Response`.
    ///
    ///
    ///  Holds the response value used by OutOfShield.
    response: crate::defense_response::Response,
    /// Stores the `simple` value in `OutOfShield` as `simple::Grab`.
    ///
    ///
    ///  Holds the simple value used by OutOfShield.
    simple: simple::Grab,
    /// Stores the `shield_up_b` value in `OutOfShield` as `shield_up_b::ShieldUpB`.
    ///
    ///
    ///  Holds the shield up b value used by OutOfShield.
    shield_up_b: shield_up_b::ShieldUpB,
    /// Stores the `configured_response` value in `OutOfShield` as `bool`.
    ///
    ///
    ///  Holds the configured response value used by OutOfShield.
    configured_response: bool,
    /// Stores the `configured_backup` value in `OutOfShield` as `bool`.
    ///
    ///
    ///  Holds the configured backup value used by OutOfShield.
    configured_backup: bool,
    /// Stores the `configured_shielded` value in `OutOfShield` as `bool`.
    ///
    ///
    ///  Holds the configured shielded value used by OutOfShield.
    configured_shielded: bool,
    /// Stores the `configured_grab_wait` value in `OutOfShield` as `bool`.
    ///
    ///
    ///  Holds the configured grab wait value used by OutOfShield.
    configured_grab_wait: bool,
    /// Stores the `configured_settling` value in `OutOfShield` as `bool`.
    ///
    ///
    ///  Holds the configured settling value used by OutOfShield.
    configured_settling: bool,
}

impl OutOfShield {
    /// Constructs a new value with the type’s initial state.
    ///
    ///
    ///  Creates a new value.
    /// ### Returns
    /// Returns a `Self` value representing the result of the operation.
    pub const fn new() -> Self {
        Self {
            object: None,
            kind: None,
            target: None,
            guarded: false,
            response: crate::defense_response::Response::new(),
            simple: simple::Grab::new(),
            shield_up_b: shield_up_b::ShieldUpB::new(),
            configured_response: false,
            configured_backup: false,
            configured_shielded: false,
            configured_grab_wait: false,
            configured_settling: false,
        }
    }

    /// Updates the `bind_actor` state using the supplied inputs.
    ///
    ///
    ///  Updates bind actor.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `c` (`Context`): Represents the c; the function uses it as input to the operation described above.
    fn bind_actor(&mut self, c: Context) {
        if self.object != Some(c.object) || self.kind != Some(c.kind) {
            *self = Self::default();
            self.object = Some(c.object);
            self.kind = Some(c.kind);
        }
    }

    /// Observes the statuses.
    ///
    ///
    ///  Observes the statuses.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `known` (`&[Option<(u32, i32)>; N]`): Represents the known; the function uses it as input to the operation described above.
    pub fn observe_statuses<const N: usize>(&mut self, known: &[Option<(u32, i32)>; N]) {
        self.simple.observe_statuses(known);
    }
    /// Returns the selected target observation.
    ///
    ///
    ///  Gets the chosen target.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
    pub fn target(&self) -> Option<u32> {
        self.simple
            .target()
            .or(self.shield_up_b.target())
            .or(self.target)
            .or(self.response.target())
    }
    /// Cancels the pending.
    ///
    ///
    ///  Cancels the pending.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    pub fn cancel_pending(&mut self) {
        self.response.cancel();
        self.configured_response = false;
        self.configured_backup = false;
        self.configured_shielded = false;
        self.configured_grab_wait = false;
        self.configured_settling = false;
        self.simple.cancel();
        self.shield_up_b.cancel();
        self.target = None;
    }
    /// Ledge/manual takeover releases inputs immediately. Delivered R retains
    /// only its bounded receipt so a competing script cannot turn it into Up-B.
    ///
    ///  Ledge/manual takeover releases inputs immediately. Delivered R retains only its bounded receipt so a competing script cannot turn it into Up-B.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    pub fn suspend_automatic(&mut self) {
        self.shield_up_b.cancel();
        self.response.suspend();
        if !self.simple.delivery_pending() {
            self.simple.cancel();
        }
        self.configured_response = self.response.grab_delivery_pending();
        self.configured_settling = self.simple.delivery_pending();
        self.target = None;
    }
    /// Acknowledge only an input packet actually passed to the native mapper.
    ///
    ///  Acknowledge only an input packet actually passed to the native mapper.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `tick` (`i64`): Represents the tick; the function uses it as input to the operation described above.
    /// - `target` (`u32`): Represents the target object or value being checked; the function uses it as input to the operation described above.
    pub fn mapped_grab(&mut self, tick: i64, target: u32) {
        self.simple.mapped_grab(tick, target);
        if self.configured_response {
            self.response.mapped_grab(tick, target);
        }
    }
    /// Determines whether the `configured_committed` condition holds.
    ///
    ///
    ///  Checks whether configured committed.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn configured_committed(&self) -> bool {
        self.configured_response
    }
    /// Determines whether ground.
    ///
    ///
    ///  Checks whether ground.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn needs_ground(&self) -> bool {
        self.configured_response && self.response.needs_ground()
    }
    /// Determines whether the `attack_hold` condition holds.
    ///
    ///
    ///  Checks whether attack hold.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn attack_hold(&self) -> bool {
        self.response.attack_hold()
    }
    /// Returns an optional `right_stick` value.
    ///
    ///
    ///  Gets right stick if it is available.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
    pub fn right_stick(&self) -> Option<[f32; 2]> {
        if self.configured_response {
            self.response.right_stick()
        } else if self.simple.active() || self.shield_up_b.active() {
            Some([0.; 2])
        } else {
            self.response.right_stick()
        }
    }
    /// Determines whether the `walking` condition holds.
    ///
    ///
    ///  Checks whether walking.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn walking(&self) -> bool {
        self.response.walking()
    }
    /// Samples the simple.
    ///
    ///
    ///  Samples the simple.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `allowed` (`bool`): Represents the allowed; the function uses it as input to the operation described above.
    /// - `c` (`Context`): Represents the c; the function uses it as input to the operation described above.
    /// - `context` (`crate::gameplay::GameplayContext`): Represents the current execution context; the function uses it as input to the operation described above.
    /// - `targets` (`[Option<crate::roll_read::Target>; 8]`): Represents the targets; the function uses it as input to the operation described above.
    /// - `attacker` (`Option<crate::gameplay::Opponent>`): Represents the attacker; the function uses it as input to the operation described above.
    /// - `close` (`f32`): Represents the close; the function uses it as input to the operation described above.
    /// - `_legacy_far` (`f32`): Represents the legacy far; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ControllerOutput` value.
    pub fn sample_simple(
        &mut self,
        allowed: bool,
        c: Context,
        context: crate::gameplay::GameplayContext,
        targets: [Option<crate::roll_read::Target>; 8],
        attacker: Option<crate::gameplay::Opponent>,
        close: f32,
        _legacy_far: f32,
    ) -> ControllerOutput {
        let startup = crate::fighter_data::ordinary_grab(c.kind, crate::native_states::CATCH)
            .map(|g| f32::from(g.first));
        self.sample_simple_timed(allowed, c, context, targets, attacker, close, startup)
    }
    /// The live adapter supplies this fighter's resolved standing/shield startup.
    /// No new delay: this only rejects projected misses before pressing Grab.
    ///
    ///  The live adapter supplies this fighter's resolved standing/shield startup. No new delay: this only rejects projected misses before pressing Grab.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `allowed` (`bool`): Represents the allowed; the function uses it as input to the operation described above.
    /// - `c` (`Context`): Represents the c; the function uses it as input to the operation described above.
    /// - `context` (`crate::gameplay::GameplayContext`): Represents the current execution context; the function uses it as input to the operation described above.
    /// - `targets` (`[Option<crate::roll_read::Target>; 8]`): Represents the targets; the function uses it as input to the operation described above.
    /// - `attacker` (`Option<crate::gameplay::Opponent>`): Represents the attacker; the function uses it as input to the operation described above.
    /// - `range` (`f32`): Represents the range; the function uses it as input to the operation described above.
    /// - `startup` (`Option<f32>`): Represents the startup; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ControllerOutput` value.
    pub fn sample_simple_timed(
        &mut self,
        allowed: bool,
        c: Context,
        context: crate::gameplay::GameplayContext,
        targets: [Option<crate::roll_read::Target>; 8],
        attacker: Option<crate::gameplay::Opponent>,
        range: f32,
        startup: Option<f32>,
    ) -> ControllerOutput {
        self.bind_actor(c);
        self.shield_up_b.cancel();
        self.simple.sample(
            allowed,
            c,
            context,
            targets,
            attacker,
            simple::Reach {
                range,
                height: 24.,
                startup,
                close: range,
                approach: false,
                shielded: matches!(
                    c.status,
                    crate::native_states::GUARD_ON..=crate::native_states::GUARD_DAMAGE
                ),
            },
        )
    }
    /// Front vulnerable-target grab (enemy shield required only when unshielded):
    /// direct neutral-axis R in shield, immediate
    /// R during ground movement/landing, or one idle approach pulse then R.
    ///
    ///  Front vulnerable-target grab (enemy shield required only when unshielded): direct neutral-axis R in shield, immediate R during ground movement/landing, or one idle approach pulse then R.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `allowed` (`bool`): Represents the allowed; the function uses it as input to the operation described above.
    /// - `c` (`Context`): Represents the c; the function uses it as input to the operation described above.
    /// - `context` (`crate::gameplay::GameplayContext`): Represents the current execution context; the function uses it as input to the operation described above.
    /// - `targets` (`[Option<crate::roll_read::Target>; 8]`): Represents the targets; the function uses it as input to the operation described above.
    /// - `follow` (`Option<crate::gameplay::Opponent>`): Represents the follow; the function uses it as input to the operation described above.
    /// - `shielded` (`bool`): Represents the shielded; the function uses it as input to the operation described above.
    /// - `close` (`f32`): Represents the close; the function uses it as input to the operation described above.
    /// - `approach` (`f32`): Represents the approach; the function uses it as input to the operation described above.
    /// - `startup` (`Option<f32>`): Represents the startup; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ControllerOutput` value.
    pub fn sample_automatic(
        &mut self,
        allowed: bool,
        c: Context,
        context: crate::gameplay::GameplayContext,
        targets: [Option<crate::roll_read::Target>; 8],
        follow: Option<crate::gameplay::Opponent>,
        shielded: bool,
        close: f32,
        approach: f32,
        startup: Option<f32>,
    ) -> ControllerOutput {
        self.sample_automatic_configured(
            allowed,
            c,
            context,
            targets,
            follow,
            shielded,
            close,
            approach,
            startup,
            crate::settings::AssistSettings {
                fighter_kind: c.kind,
                ..Default::default()
            },
            AutomaticConfig::default(),
        )
    }
    /// Neutralize the stick during unshielded Grab delivery while preserving
    /// manual turning-away cancellation. Shield grabs already use neutral axes.
    ///
    ///  Neutralize the stick during unshielded Grab delivery while preserving manual turning-away cancellation. Shield grabs already use neutral axes.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `output` (`ControllerOutput`): Represents the output data being produced; the function uses it as input to the operation described above.
    /// - `physical` (`[f32; 2]`): Represents the physical; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ControllerOutput` value.
    pub fn with_grab_movement(
        &mut self,
        mut output: ControllerOutput,
        physical: [f32; 2],
    ) -> ControllerOutput {
        if self.simple.turning_away(physical[0]) {
            self.suspend_automatic();
            return ControllerOutput::default();
        }
        if self.simple.unshielded_grab_started()
            && output.requested & crate::native_input::MAPPER_R != 0
        {
            output.stick = Some([0.0, 0.0]);
        }
        output
    }
    /// Determines whether the `unshielded_grab_started` condition holds.
    ///
    ///
    ///  Checks whether unshielded grab started.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn unshielded_grab_started(&self) -> bool {
        self.simple.unshielded_grab_started()
    }
    /// Determines whether the `shield_up_b_pending` condition holds.
    ///
    ///
    ///  Checks whether shield up b pending.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn shield_up_b_pending(&self) -> bool {
        self.shield_up_b.active() || self.configured_response && self.response.automatic_up_b()
    }
    /// Determines whether the `up_b_delivery_started` condition holds.
    ///
    ///
    ///  Checks whether up b delivery started.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn up_b_delivery_started(&self) -> bool {
        self.shield_up_b.delivery_started()
            || self.configured_response && self.response.up_b_delivery_started()
    }
    /// Determines whether the `up_b_receipt_pending` condition holds.
    ///
    ///
    ///  Checks whether up b receipt pending.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn up_b_receipt_pending(&self) -> bool {
        self.shield_up_b.receipt_pending()
            || self.configured_response && self.response.up_b_receipt_pending()
    }
    /// Determines whether the `awaiting_up_b` condition holds.
    ///
    ///
    ///  Checks whether awaiting up b.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn awaiting_up_b(&self) -> bool {
        self.shield_up_b.awaiting() || self.configured_response && self.response.awaiting_up_b()
    }
    /// Updates the `attempted_up_special` state using the supplied inputs.
    ///
    ///
    ///  Updates attempted up special.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `tick` (`i64`): Represents the tick; the function uses it as input to the operation described above.
    /// - `target` (`u32`): Represents the target object or value being checked; the function uses it as input to the operation described above.
    pub fn attempted_up_special(&mut self, tick: i64, target: u32) {
        if self.configured_response {
            self.response.attempted_up_special(tick, target);
        } else {
            self.shield_up_b.attempted(tick, target);
        }
    }
    /// Updates the `mapped_up_special` state using the supplied inputs.
    ///
    ///
    ///  Updates mapped up special.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `tick` (`i64`): Represents the tick; the function uses it as input to the operation described above.
    /// - `target` (`u32`): Represents the target object or value being checked; the function uses it as input to the operation described above.
    pub fn mapped_up_special(&mut self, tick: i64, target: u32) {
        if self.configured_response {
            self.response.mapped_up_special(tick, target);
        } else {
            self.shield_up_b.mapped(tick, target);
        }
    }
    /// Determines whether the `grab_delivery_pending` condition holds.
    ///
    ///
    ///  Checks whether grab delivery pending.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn grab_delivery_pending(&self) -> bool {
        self.simple.delivery_pending() || self.response.grab_delivery_pending()
    }
    /// Updates the `delivery_tick` state using the supplied inputs.
    ///
    ///
    ///  Updates delivery tick.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `tick` (`i64`): Represents the tick; the function uses it as input to the operation described above.
    pub fn delivery_tick(&mut self, tick: i64) {
        self.shield_up_b.delivery_tick(tick);
        self.simple.delivery_tick(tick);
        self.response.delivery_tick(tick);
    }
    /// Determines whether the `automatic_grab_pending` condition holds.
    ///
    ///
    ///  Checks whether automatic grab pending.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn automatic_grab_pending(&self) -> bool {
        self.simple.active() && !self.configured_response && !self.configured_settling
    }
    /// Determines whether the `approach_pending` condition holds.
    ///
    ///
    ///  Checks whether approach pending.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn approach_pending(&self) -> bool {
        self.simple.approach_pending()
    }
    /// Determines whether the `approach_active` condition holds.
    ///
    ///
    ///  Checks whether approach active.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn approach_active(&self) -> bool {
        self.simple.approach_active()
    }
    /// Updates the `mapped_approach` state using the supplied inputs.
    ///
    ///
    ///  Updates mapped approach.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `tick` (`i64`): Represents the tick; the function uses it as input to the operation described above.
    /// - `target` (`u32`): Represents the target object or value being checked; the function uses it as input to the operation described above.
    pub fn mapped_approach(&mut self, tick: i64, target: u32) {
        self.simple.mapped_approach(tick, target);
    }
    /// Samples the configured.
    ///
    ///
    ///  Samples the configured.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `enabled` (`bool`): Represents whether the feature or option should be enabled; the function uses it as input to the operation described above.
    /// - `rules` (`Option<&crate::reaction_rules::Resolved>`): Represents the rules; the function uses it as input to the operation described above.
    /// - `settings` (`crate::settings::AssistSettings`): Represents the settings; the function uses it as input to the operation described above.
    /// - `c` (`Context`): Represents the c; the function uses it as input to the operation described above.
    /// - `context` (`crate::gameplay::GameplayContext`): Represents the current execution context; the function uses it as input to the operation described above.
    /// - `target` (`Option<Target>`): Represents the target object or value being checked; the function uses it as input to the operation described above.
    /// - `follow` (`Option<crate::gameplay::Opponent>`): Represents the follow; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ControllerOutput` value.
    pub fn sample_configured(
        &mut self,
        enabled: bool,
        rules: Option<&crate::reaction_rules::Resolved>,
        settings: crate::settings::AssistSettings,
        c: Context,
        context: crate::gameplay::GameplayContext,
        target: Option<Target>,
        follow: Option<crate::gameplay::Opponent>,
    ) -> ControllerOutput {
        self.simple.cancel();
        self.shield_up_b.cancel();
        if !enabled || rules.is_none() {
            self.cancel_pending();
            self.guarded = c.status == crate::native_states::GUARD_DAMAGE;
            return ControllerOutput::default();
        }
        self.bind_actor(c);
        if let Some(previous_attacker) = self.response.target() {
            let renewed_hit = c.status == crate::native_states::GUARD_DAMAGE
                && (!self.guarded
                    || c.stopped
                    || !c.shield_ready
                    || target.is_some_and(|t| t.id != previous_attacker));
            if renewed_hit {
                // A new block invalidates the old response's timing/attacker.
                // Re-arm from this contact; never buffer an old Up-B through
                // renewed shieldstun and replay it against a different attacker.
                self.cancel_pending();
                self.guarded = false;
            } else {
                self.guarded = c.status == crate::native_states::GUARD_DAMAGE;
                return self.response.sample(enabled, settings, context, follow);
            }
        }
        let hit = c.status == crate::native_states::GUARD_DAMAGE;
        if hit
            && (!self.guarded || target.is_some_and(|t| self.target.is_some_and(|id| id != t.id)))
        {
            self.target = target.map(|t| t.id);
        }
        self.guarded = hit;
        if !c.grounded
            || !matches!(
                c.status,
                crate::native_states::GUARD_ON..=crate::native_states::GUARD_DAMAGE
            )
        {
            self.cancel_pending();
            return ControllerOutput::default();
        }
        let Some(t) = target.filter(|t| Some(t.id) == self.target) else {
            self.target = None;
            return ControllerOutput::default();
        };
        if c.stopped || !c.shield_ready {
            return ControllerOutput::default();
        }
        // One response per confirmed contact. No waiting in shield for a later
        // range/recovery window, and no hard-coded fighter move or startup.
        self.target = None;
        if let Some(enemy) = follow.filter(|e| e.id == t.id) {
            self.response.start(
                rules.unwrap(),
                crate::reaction_rules::Trigger::ShieldHit,
                context,
                enemy,
            );
        }
        self.response.sample(enabled, settings, context, follow)
    }
}

#[cfg(test)]
mod tests {
    use super::{copied_target, target_within_range};
    use crate::{gameplay::MotionTiming, roll_read::Target as ObservedTarget};

    /// Regression test for `target_range_is_inclusive_and_rejects_invalid_facts`.
    ///
    ///
    ///  Checks that target range is inclusive and rejects invalid facts.
    #[test]
    fn target_range_is_inclusive_and_rejects_invalid_facts() {
        assert!(target_within_range([0., 0.], [3., -2.], 3., 2.));
        assert!(!target_within_range([0., 0.], [3.01, 0.], 3., 2.));
        assert!(!target_within_range([0., 0.], [0., 0.], 0., 2.));
        assert!(!target_within_range([0., 0.], [0., 0.], 2., -1.));
        assert!(!target_within_range([f32::NAN, 0.], [0., 0.], 2., 2.));
    }

    /// Regression test for `copied_target_preserves_common_observation_facts`.
    ///
    ///
    ///  Checks that copied target preserves common observation facts.
    #[test]
    fn copied_target_preserves_common_observation_facts() {
        let opponent = copied_target(&ObservedTarget {
            id: 7,
            status: 12,
            motion: 34,
            frame: 5.5,
            rate: 1.25,
            position: [2., -3.],
            velocity_x: 0.5,
            vulnerable_in: Some(0.),
            actionable_in: Some(4.),
        });

        assert_eq!(opponent.id, 7);
        assert_eq!(opponent.status, 12);
        assert_eq!(opponent.position, [2., -3.]);
        assert_eq!(opponent.velocity, [0.5, 0.]);
        assert!(opponent.vulnerable);
        assert_eq!(
            opponent.motion,
            Some(MotionTiming {
                kind: 34,
                frame: 5.5,
                rate: 1.25,
                remaining: None,
            })
        );
        assert_eq!(opponent.actionable_in, None);
    }
}
