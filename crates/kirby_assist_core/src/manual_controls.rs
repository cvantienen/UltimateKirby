//! Manual controller helpers, independent of native pointers and Skyline.
//! Left/right are absolute stick directions, not the fighter's facing direction.
//! The caller captures physical input first and restores native memory after
//! mapping; this module only proposes a trace and left-stick position.
//!
//! Reading order: `DpadReservation` consumes reserved physical triggers,
//! `TurboState` proposes its movement, then `ManualOverlay::apply_controls`
//! combines one chosen stick proposal with button ownership. A button mask
//! names several buttons at once; `owned` includes intentional releases.
use crate::native_input::{ButtonTrace, MapperButtonOverlay};

mod assist_handoff {
    //! Release-to-rearm for physical shield suppressed by an automatic punish.
    //! No timers, allocations, native queries or changes to requested assist inputs.
    use crate::native_input::ButtonTrace;
    use crate::{gameplay::GameplayContext, ledge_trap::NativeAttempt};

    /// Keep a released manual jump from racing a new grounded automatic input
    /// while the native fighter has not acknowledged the queued jump yet.
    ///
    ///  Keep a released manual jump from racing a new grounded automatic input while the native fighter has not acknowledged the queued jump yet.
    #[derive(Default)]
    pub struct ManualJumpFence {
        /// Stores the `owner` value in `ManualJumpFence` as `Option<(u32`.
        ///
        ///
        ///  Optionally stores the owner for ManualJumpFence; it may be absent.
        owner: Option<(u32, i32)>,
        /// Stores the `held` value in `ManualJumpFence` as `bool`.
        ///
        ///
        ///  Holds the held value used by ManualJumpFence.
        held: bool,
        /// Stores the `pending` value in `ManualJumpFence` as `bool`.
        ///
        ///
        ///  Holds the pending value used by ManualJumpFence.
        pending: bool,
        /// Stores the `clock` value in `ManualJumpFence` as `NativeAttempt`.
        ///
        ///
        ///  Holds the clock value used by ManualJumpFence.
        clock: NativeAttempt,
    }
    impl ManualJumpFence {
        /// Constructs a new value with the type’s initial state.
        ///
        ///
        ///  Creates a new value.
        /// ### Returns
        /// Returns a `Self` value representing the result of the operation.
        pub const fn new() -> Self {
            Self {
                owner: None,
                held: false,
                pending: false,
                clock: NativeAttempt::new(),
            }
        }
        /// Checks whether a pending operation is stored.
        ///
        ///
        ///  Checks whether something is waiting to be processed.
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns `true` when the documented condition holds; otherwise, returns `false`.
        pub fn pending(&self) -> bool {
            self.pending
        }
        /// Updates the toggle state from the current button sample and match state.
        ///
        ///
        ///  Checks the current buttons and updates whether the toggle should activate.
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// - `object` (`u32`): Represents the object; the function uses it as input to the operation described above.
        /// - `kind` (`i32`): Represents the kind or category being checked; the function uses it as input to the operation described above.
        /// - `held` (`bool`): Represents the held; the function uses it as input to the operation described above.
        /// - `c` (`GameplayContext`): Represents the c; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns `true` when the documented condition holds; otherwise, returns `false`.
        pub fn sample(&mut self, object: u32, kind: i32, held: bool, c: GameplayContext) -> bool {
            if self.owner != Some((object, kind)) {
                *self = Self::new();
                self.owner = Some((object, kind));
            }
            if !c.gameplay {
                *self = Self::new();
                return false;
            }
            if held && !self.held {
                self.pending = true;
                self.clock = NativeAttempt::new();
            }
            self.held = held;
            if self.pending {
                if c.status == crate::native_states::status::JUMP_SQUAT
                    || c.situation == crate::native_states::situation::AIR
                {
                    self.pending = false;
                } else {
                    // A lag pause/duplicate mapper poll cannot spend this fence.
                    // Invalid/skipped animation samples cannot prove jump expiry.
                    self.clock.advance_pause(c, None, u16::MAX);
                    if self.clock.elapsed() >= crate::move_input::ACCEPTANCE_TIMEOUT {
                        self.pending = false;
                    }
                }
            }
            held || self.pending
        }
    }

    /// Defines the `AssistGuardHandoff` data structure.
    ///
    ///
    ///  Holds the data for AssistGuardHandoff.
    #[derive(Default)]
    pub struct AssistGuardHandoff {
        /// Stores the `blocked` value in `AssistGuardHandoff` as `u32`.
        ///
        ///
        ///  Holds the blocked value used by AssistGuardHandoff.
        blocked: u32,
        /// Stores the `previous` value in `AssistGuardHandoff` as `Option<u32>`.
        ///
        ///
        ///  Optionally stores the previous for AssistGuardHandoff; it may be absent.
        previous: Option<u32>,
        /// Stores the `protected` value in `AssistGuardHandoff` as `u32`.
        ///
        ///
        ///  Holds the protected value used by AssistGuardHandoff.
        protected: u32,
    }
    impl AssistGuardHandoff {
        /// Constructs a new value with the type’s initial state.
        ///
        ///
        ///  Creates a new value.
        /// ### Returns
        /// Returns a `Self` value representing the result of the operation.
        pub const fn new() -> Self {
            Self {
                blocked: 0,
                previous: None,
                protected: 0,
            }
        }
        /// Updates the toggle state from the current button sample and match state.
        ///
        ///
        ///  Checks the current buttons and updates whether the toggle should activate.
        /// ### Arguments
        /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
        /// - `active` (`bool`): Represents the active; the function uses it as input to the operation described above.
        /// - `human` (`ButtonTrace`): Represents the human; the function uses it as input to the operation described above.
        /// - `guards` (`u32`): Represents the guards; the function uses it as input to the operation described above.
        /// - `owned` (`u32`): Represents the owned; the function uses it as input to the operation described above.
        /// - `requested` (`u32`): Represents the requested; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns the resulting `u32` value.
        pub fn sample(
            &mut self,
            active: bool,
            human: ButtonTrace,
            guards: u32,
            owned: u32,
            requested: u32,
        ) -> u32 {
            // A real release immediately rearms the physical button. A fresh press
            // is an intentional manual command, not an old hold to resurrect.
            let pressed = self.previous.map_or(0, |old| human.held & !old);
            self.previous = Some(human.held);
            self.blocked &= human.held & guards & !pressed;
            if !active {
                self.blocked |= self.protected & human.held & guards & !pressed;
            }
            // Only an actually SUPPRESSED human guard needs release-to-rearm.
            // Direct shield grab/Up-B explicitly keeps guard; quarantining that
            // unchanged hold after a failed request drops a perfectly valid shield.
            self.protected = if active {
                human.held & guards & owned & !requested & !pressed
            } else {
                0
            };
            self.blocked &= !(requested & guards);
            if active {
                self.blocked |= human.held & guards & owned & !requested & !pressed;
            }
            self.blocked
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::native_input::{MAPPER_ZL, MAPPER_ZR};
        /// Checks whether the required input is currently held.
        ///
        ///
        ///  Checks whether the button is still held.
        /// ### Arguments
        /// - `mask` (`u32`): Represents the mask; the function uses it as input to the operation described above.
        /// ### Returns
        /// Returns the resulting `ButtonTrace` value.
        fn held(mask: u32) -> ButtonTrace {
            ButtonTrace {
                held: mask,
                previous: mask,
                ..Default::default()
            }
        }
        /// Regression test for `old_shield_hold_does_not_return_as_a_roll_or_air_dodge`.
        ///
        ///
        ///  Checks that old shield hold does not return as a roll or air dodge.
        #[test]
        fn old_shield_hold_does_not_return_as_a_roll_or_air_dodge() {
            let mut h = AssistGuardHandoff::new();
            let g = MAPPER_ZR | MAPPER_ZL;
            assert_eq!(h.sample(true, held(g), g, g, 0), g);
            for _ in 0..8 {
                assert_eq!(h.sample(false, held(g), g, 0, 0), g);
            }
            assert_eq!(h.sample(false, held(MAPPER_ZR), g, 0, 0), MAPPER_ZR);
            assert_eq!(h.sample(false, ButtonTrace::default(), g, 0, 0), 0);
            assert_eq!(
                h.sample(
                    false,
                    ButtonTrace {
                        held: g,
                        pressed: g,
                        ..Default::default()
                    },
                    g,
                    0,
                    0
                ),
                0
            );
        }
        /// Regression test for `requested_defense_and_fresh_manual_shield_are_not_suppressed`.
        ///
        ///
        ///  Checks that requested defense and fresh manual shield are not suppressed.
        #[test]
        fn requested_defense_and_fresh_manual_shield_are_not_suppressed() {
            let mut h = AssistGuardHandoff::new();
            let g = MAPPER_ZR;
            assert_eq!(h.sample(true, held(g), g, g, g), 0);
            h.sample(false, ButtonTrace::default(), g, 0, 0);
            assert_eq!(
                h.sample(
                    true,
                    ButtonTrace {
                        held: g,
                        pressed: g,
                        ..Default::default()
                    },
                    g,
                    g,
                    0
                ),
                0
            );
            assert_eq!(h.sample(false, held(g), g, 0, 0), 0);
        }
    }
}
pub use assist_handoff::{AssistGuardHandoff, ManualJumpFence};
/// Defines the `TURBO_TRIGGER` constant used by this module.
///
/// Provides the TURBO TRIGGER constant value.
pub const TURBO_TRIGGER: u32 = 1 << 16; // Canonical turbo role; physical source comes from button bindings.
/// Defines the `DPAD_UP` constant used by this module.
///
/// Provides the DPAD UP constant value.
pub const DPAD_UP: u32 = crate::native_input::MAPPER_DPAD_UP;
/// Defines the `DPAD_RIGHT` constant used by this module.
///
/// Provides the DPAD RIGHT constant value.
pub const DPAD_RIGHT: u32 = crate::native_input::MAPPER_DPAD_RIGHT;
/// Defines the `DPAD_LEFT` constant used by this module.
///
/// Provides the DPAD LEFT constant value.
pub const DPAD_LEFT: u32 = crate::native_input::MAPPER_DPAD_LEFT;
/// Defines the `DPAD_SIDES` constant used by this module.
///
/// Provides the DPAD SIDES constant value.
pub const DPAD_SIDES: u32 = DPAD_LEFT | DPAD_RIGHT;
/// Defines the `LEFT_STICK_DIRECTIONS` constant used by this module.
///
/// Provides the LEFT STICK DIRECTIONS constant value.
pub const LEFT_STICK_DIRECTIONS: u32 = crate::native_input::MAPPER_LEFT_STICK_DIRECTIONS;
/// Defines the `RIGHT_STICK_DIRECTIONS` constant used by this module.
///
/// Provides the RIGHT STICK DIRECTIONS constant value.
pub const RIGHT_STICK_DIRECTIONS: u32 = crate::native_input::MAPPER_RIGHT_STICK_DIRECTIONS;
/// Defines the `STICK_CLICKS` constant used by this module.
///
/// Provides the STICK CLICKS constant value.
pub const STICK_CLICKS: u32 = crate::native_input::MAPPER_STICK_CLICKS;

/// Defines the `ManualContext` data structure.
///
/// Holds the data for ManualContext.
#[derive(Clone, Copy, Debug)]
pub struct ManualContext {
    /// Stores the `status` value in `ManualContext` as `i32`.
    ///
    ///
    ///  Holds the status value used by ManualContext.
    pub status: i32,
    /// Stores the `situation` value in `ManualContext` as `i32`.
    ///
    ///
    ///  Holds the situation value used by ManualContext.
    pub situation: i32,
    /// Stores the `motion` value in `ManualContext` as `u64`.
    ///
    ///
    ///  Holds the motion value used by ManualContext.
    pub motion: u64,
    /// Stores the `frame` value in `ManualContext` as `f32`.
    ///
    ///
    ///  Holds the frame value used by ManualContext.
    pub frame: f32,
    /// Stores the `rate` value in `ManualContext` as `f32`.
    ///
    ///
    ///  Holds the rate value used by ManualContext.
    pub rate: f32,
    /// Stores the `stopped` value in `ManualContext` as `bool`.
    ///
    ///
    ///  Holds the stopped value used by ManualContext.
    pub stopped: bool,
    /// Stores the `facing` value in `ManualContext` as `f32`.
    ///
    ///
    ///  Holds the facing value used by ManualContext.
    pub facing: f32,
}

/// Preserve fine physical input and crouching while capping horizontal motion
/// for manual ledge spacing. Attack recipes bypass this cap explicitly.
///
///  Preserve fine physical input and crouching while capping horizontal motion for manual ledge spacing. Attack recipes bypass this cap explicitly.
/// ### Arguments
/// - `physical` (`[f32; 2]`): Represents the physical; the function uses it as input to the operation described above.
/// - `maximum` (`f32`): Represents the maximum; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `[f32; 2]` value.
pub fn ledge_walk_stick(physical: [f32; 2], maximum: f32) -> [f32; 2] {
    let base = if maximum.is_finite() {
        maximum.clamp(0.1, 0.45)
    } else {
        0.35
    };
    // The existing setting remains the baseline for old config files. Use a
    // slightly lower cap normally, with a fixed fast-walk request near full
    // horizontal. The overlay also clears digital stick-flick aliases.
    let full_horizontal = physical[0].is_finite()
        && physical[1].is_finite()
        && physical[0].abs() >= 0.95
        && physical[1].abs() <= 0.15;
    let cap = if full_horizontal {
        0.5 // Full analog deflection can dash even with digital flick aliases cleared.
    } else {
        (base - 0.02).clamp(0.1, 0.45)
    };
    let x = if physical[0].is_finite() {
        physical[0].clamp(-cap, cap)
    } else {
        0.0
    };
    let y = if physical[1].is_finite() {
        physical[1].clamp(-1.0, 1.0)
    } else {
        0.0
    };
    [x, y]
}

/// Release only outward manual walking input at a known ledge gap.
/// This uses copied motion, not geometry searches or native position writes.
///
///  Release only outward manual walking input at a known ledge gap. This uses copied motion, not geometry searches or native position writes.
/// ### Arguments
/// - `stick` (`[f32; 2]`): Represents the stick; the function uses it as input to the operation described above.
/// - `position` (`[f32; 2]`): Represents an object position; the function uses it as input to the operation described above.
/// - `velocity` (`[f32; 2]`): Represents an object velocity or movement speed; the function uses it as input to the operation described above.
/// - `spot` (`crate::ledge_position::LedgeSpot`): Represents the spot; the function uses it as input to the operation described above.
/// - `stand_distance` (`f32`): Represents the stand distance; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `[f32; 2]` value.
pub fn stop_walk_at_ledge(
    mut stick: [f32; 2],
    position: [f32; 2],
    velocity: [f32; 2],
    spot: crate::ledge_position::LedgeSpot,
    stand_distance: f32,
) -> [f32; 2] {
    if !position
        .iter()
        .chain(velocity.iter())
        .chain(spot.edge.iter())
        .all(|v| v.is_finite())
        || !spot.inward.is_finite()
        || spot.inward.abs() != 1.0
        || !stand_distance.is_finite()
        || stand_distance < 1.0
        || (position[1] - spot.edge[1]).abs() > 6.0
        || !stick[0].is_finite()
        || stick[0] * spot.inward >= 0.0
    {
        return stick;
    }
    let clearance = (position[0] - spot.edge[0]) * spot.inward;
    let closing_speed = (-velocity[0] * spot.inward).max(0.0);
    if clearance - closing_speed * 3.0 <= stand_distance + 0.1 {
        stick[0] = 0.0;
    }
    stick
}

/// A diagonal press can include Up even when the intended trigger is Left/Right.
/// Quarantine all D-pad bits involved in a macro press until their release, so
/// a diagonal or its release cannot turn into a different taunt.
///
///  A diagonal press can include Up even when the intended trigger is Left/Right. Quarantine all D-pad bits involved in a macro press until their release, so a diagonal or its release cannot turn into a different taunt.
#[derive(Default)]
pub struct DpadReservation {
    /// Stores the `quarantined` value in `DpadReservation` as `u32`.
    ///
    ///
    ///  Holds the quarantined value used by DpadReservation.
    quarantined: u32,
}
impl DpadReservation {
    /// Updates the toggle state from the current button sample and match state.
    ///
    ///
    ///  Checks the current buttons and updates whether the toggle should activate.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `enabled` (`u32`): Represents whether the feature or option should be enabled; the function uses it as input to the operation described above.
    /// - `physical` (`u32`): Represents the physical; the function uses it as input to the operation described above.
    /// - `gameplay` (`bool`): Represents the gameplay; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `u32` value.
    pub fn sample(&mut self, enabled: u32, physical: u32, gameplay: bool) -> u32 {
        if !gameplay || enabled == 0 {
            *self = Self::default();
            return 0;
        }
        if physical & enabled != 0 {
            self.quarantined |= physical & crate::native_input::MAPPER_DPAD_BUTTONS;
        }
        // Include the old quarantine BEFORE dropping released bits: this poll
        // must swallow the release edge too. Enabled triggers stay reserved even
        // while the fighter is too busy to execute the corresponding macro.
        let consumed = enabled | self.quarantined;
        self.quarantined &= physical;
        consumed
    }
}

/// Defines the `TurboSettings` data structure.
///
/// Holds the data for TurboSettings.
#[derive(Clone, Copy, Debug)]
pub struct TurboSettings {
    /// Normalized horizontal stick magnitude during crouch and creep phases.
    ///
    ///  Normalized horizontal stick magnitude during crouch and creep phases.
    pub walk_strength: f32,
}
impl Default for TurboSettings {
    /// Returns the default value for this type.
    ///
    ///
    ///  Returns the default value for this type.
    /// ### Returns
    /// Returns a `Self` value representing the result of the operation.
    fn default() -> Self {
        Self {
            walk_strength: 0.45,
        }
    }
}

/// Grounded idle/walk/run/turn/crouch states qualify. Do not change game
/// statuses to force the macro through attacks, grabs, hitstop, or aerial states.
///
///  Grounded idle/walk/run/turn/crouch states qualify. Do not change game statuses to force the macro through attacks, grabs, hitstop, or aerial states.
/// ### Arguments
/// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
/// - `situation` (`i32`): Represents the situation; the function uses it as input to the operation described above.
/// - `stopped` (`bool`): Represents the stopped; the function uses it as input to the operation described above.
/// ### Returns
/// Returns `true` when the documented condition holds; otherwise, returns `false`.
pub fn turbo_movement_allowed(status: i32, situation: i32, stopped: bool) -> bool {
    situation == crate::native_states::situation::GROUND
        && !stopped
        && ((crate::native_states::status::WAIT..=crate::native_states::status::TURN_RUN_BRAKE)
            .contains(&status)
            || (crate::native_states::status::SQUAT..=crate::native_states::status::SQUAT_RV)
                .contains(&status))
}

/// Defines the `TurboOutput` data structure.
///
/// Holds the data for TurboOutput.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TurboOutput {
    /// Stores the `stick` value in `TurboOutput` as `Option<[f32; 2]>`.
    ///
    ///
    ///  Optionally stores the stick for TurboOutput; it may be absent.
    pub stick: Option<[f32; 2]>,
}

/// Cleans up a deliberate full-stick reversal during a native dash/run. The
/// two-update hold is advanced by the shared game tick, never mapper polls, so
/// online input sampling cannot shorten the pivot window.
///
///  Cleans up a deliberate full-stick reversal during a native dash/run. The two-update hold is advanced by the shared game tick, never mapper polls, so online input sampling cannot shorten the pivot window.
#[derive(Default)]
pub struct DashPivotAssist {
    /// Stores the `last_direction` value in `DashPivotAssist` as `i8`.
    ///
    ///
    ///  Holds the last direction value used by DashPivotAssist.
    last_direction: i8,
    /// Stores the `active_direction` value in `DashPivotAssist` as `i8`.
    ///
    ///
    ///  Holds the active direction value used by DashPivotAssist.
    active_direction: i8,
    /// Stores the `dash_walk_direction` value in `DashPivotAssist` as `i8`.
    ///
    ///
    ///  Holds the dash walk direction value used by DashPivotAssist.
    dash_walk_direction: i8,
    /// Stores the `updates_left` value in `DashPivotAssist` as `u8`.
    ///
    ///
    ///  Holds the updates left value used by DashPivotAssist.
    updates_left: u8,
    /// Stores the `last_tick` value in `DashPivotAssist` as `Option<i64>`.
    ///
    ///
    ///  Optionally stores the last tick for DashPivotAssist; it may be absent.
    last_tick: Option<i64>,
    /// Stores the `last_output` value in `DashPivotAssist` as `TurboOutput`.
    ///
    ///
    ///  Holds the last output value used by DashPivotAssist.
    last_output: TurboOutput,
}
impl DashPivotAssist {
    /// A higher-priority tool must not leave movement queued for handback.
    ///
    ///  A higher-priority tool must not leave movement queued for handback.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    pub fn cancel_pending(&mut self) {
        *self = Self::default();
    }
    /// Updates the toggle state from the current button sample and match state.
    ///
    ///
    ///  Checks the current buttons and updates whether the toggle should activate.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `tick` (`i64`): Represents the tick; the function uses it as input to the operation described above.
    /// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
    /// - `situation` (`i32`): Represents the situation; the function uses it as input to the operation described above.
    /// - `stopped` (`bool`): Represents the stopped; the function uses it as input to the operation described above.
    /// - `physical_stick` (`[f32; 2]`): Represents the physical stick; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `TurboOutput` value.
    pub fn sample(
        &mut self,
        tick: i64,
        status: i32,
        situation: i32,
        stopped: bool,
        physical_stick: [f32; 2],
    ) -> TurboOutput {
        // Revalidate before the duplicate cache: landing/stop/damage and fresh
        // physical vertical input can change between mapper polls online.
        if situation != crate::native_states::situation::GROUND
            || stopped
            || !(crate::native_states::status::WAIT..=crate::native_states::status::TURN_RUN)
                .contains(&status)
            || !physical_stick.iter().all(|v| v.is_finite())
            || physical_stick[1].abs() > 0.3
            || physical_stick[0].abs() < 0.1
        {
            *self = Self::default();
            return TurboOutput::default();
        }
        if self
            .last_tick
            .is_some_and(|last| tick < last || tick.saturating_sub(last) > 1)
        {
            *self = Self::default();
        }
        if self.last_tick == Some(tick) {
            // A duplicate native tick can still contain newer physical input
            // online. Never replay an old reversal or walk cap against it.
            if self.last_output.stick.is_some_and(|stick| {
                stick[0] * physical_stick[0] < 0.
                    || (stick[0].abs() >= 0.8) != (physical_stick[0].abs() >= 0.8)
                    || stick[0].abs() >= 0.8
                        && !(crate::native_states::status::DASH
                            ..=crate::native_states::status::TURN_RUN)
                            .contains(&status)
            }) {
                self.cancel_pending();
                return TurboOutput::default();
            }
            return self.last_output;
        }
        let x = physical_stick[0];
        let direction = if x >= 0.8 {
            1
        } else if x <= -0.8 {
            -1
        } else {
            0
        };
        // Native common statuses 3..=8 cover dash/turn-dash/run/brake/turn-run.
        // Do not reshape ordinary walking, crouching, attacks, or aerial input.
        let movement_status = (crate::native_states::status::DASH
            ..=crate::native_states::status::TURN_RUN)
            .contains(&status);
        if !movement_status || direction == 0 {
            self.updates_left = 0;
            self.active_direction = 0;
        }
        if movement_status
            && direction != 0
            && self.last_direction != 0
            && direction == -self.last_direction
        {
            self.active_direction = direction;
            self.updates_left = 2;
        }
        // A full dash followed by a deliberate same-direction stick ease is a
        // dash-walk gesture. Keep the eased input at the fastest value below
        // the digital dash threshold; a continuously full stick still runs.
        if status == crate::native_states::status::DASH && direction != 0 {
            self.dash_walk_direction = direction;
        }
        let gentle_direction = if (0.1..0.8).contains(&x) {
            1
        } else if (-0.8..-0.1).contains(&x) {
            -1
        } else {
            0
        };
        let dash_walking = self.dash_walk_direction != 0
            && gentle_direction == self.dash_walk_direction
            && (crate::native_states::status::WAIT..=crate::native_states::status::TURN_RUN)
                .contains(&status);
        if x.abs() < 0.1
            || direction != 0 && direction != self.dash_walk_direction
            || gentle_direction != 0 && gentle_direction != self.dash_walk_direction
        {
            self.dash_walk_direction = 0;
        }
        if direction != 0 {
            self.last_direction = direction;
        }
        let output = if self.updates_left != 0 {
            self.updates_left -= 1;
            TurboOutput {
                stick: Some([f32::from(self.active_direction), 0.0]),
            }
        } else if dash_walking {
            TurboOutput {
                stick: Some([f32::from(self.dash_walk_direction) * 0.49, 0.0]),
            }
        } else {
            TurboOutput::default()
        };
        self.last_tick = Some(tick);
        self.last_output = output;
        output
    }
}

/// Defines the `TurboState` data structure.
///
/// Holds the data for TurboState.
#[derive(Default)]
pub struct TurboState {
    /// Stores the `direction` value in `TurboState` as `i8`.
    ///
    ///
    ///  Holds the direction value used by TurboState.
    direction: i8,
    /// Stores the `next_phase` value in `TurboState` as `u32`.
    ///
    ///
    ///  Holds the next phase value used by TurboState.
    next_phase: u32,
    /// Stores the `last_tick` value in `TurboState` as `Option<i64>`.
    ///
    ///
    ///  Optionally stores the last tick for TurboState; it may be absent.
    last_tick: Option<i64>,
    /// Stores the `last_output` value in `TurboState` as `TurboOutput`.
    ///
    ///
    ///  Holds the last output value used by TurboState.
    last_output: TurboOutput,
}
impl TurboState {
    /// A selected button binding runs one crouch update and one gentle
    /// facing-direction walk update per cycle. Duplicate input updates reuse
    /// their output; releasing the binding or losing movement eligibility stops.
    ///
    ///  A selected button binding runs one crouch update and one gentle facing-direction walk update per cycle. Duplicate input updates reuse their output; releasing the binding or losing movement eligibility stops.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `settings` (`TurboSettings`): Represents the settings; the function uses it as input to the operation described above.
    /// - `tick` (`i64`): Represents the tick; the function uses it as input to the operation described above.
    /// - `held` (`bool`): Represents the held; the function uses it as input to the operation described above.
    /// - `facing` (`f32`): Represents the facing; the function uses it as input to the operation described above.
    /// - `gameplay` (`bool`): Represents the gameplay; the function uses it as input to the operation described above.
    /// - `can_move` (`bool`): Represents the can move; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `TurboOutput` value.
    pub fn sample_teabag(
        &mut self,
        settings: TurboSettings,
        tick: i64,
        held: bool,
        facing: f32,
        gameplay: bool,
        can_move: bool,
    ) -> TurboOutput {
        let direction = if facing.is_finite() && facing.abs() >= 0.5 {
            if facing < 0.0 {
                -1
            } else {
                1
            }
        } else {
            0
        };
        if !held || !gameplay || !can_move || direction == 0 {
            *self = Self::default();
            return TurboOutput::default();
        }

        if self.direction != direction || self.last_tick.is_some_and(|last| tick < last) {
            *self = Self::default();
            self.direction = direction;
        }
        if self.last_tick == Some(tick) {
            return self.last_output;
        }
        // Start creeping on the first crouch phase rather than spending the
        // initial update planted. Native crouch/walk rules still apply.
        let crouching = self.next_phase == 0;
        // Never propagate NaN into native controller memory if a direct caller
        // bypasses the validated text parser.
        let strength = if settings.walk_strength.is_finite() {
            settings.walk_strength.clamp(0.05, 0.45)
        } else {
            0.25
        };
        let output = TurboOutput {
            stick: Some([
                f32::from(direction) * strength,
                if crouching { -1.0 } else { 0.0 },
            ]),
        };
        self.next_phase = (self.next_phase + 1) % 2;
        self.last_tick = Some(tick);
        self.last_output = output;
        output
    }
}

/// Result consumed by the native mapper; tech/mash button ownership is composed
/// first. The shared stick overlay accepts analog mash as well as D-pad tools,
/// owning stick/attack/shield fields only while the active policy requests them.
///
///  Result consumed by the native mapper; tech/mash button ownership is composed first. The shared stick overlay accepts analog mash as well as D-pad tools, owning stick/attack/shield fields only while the active policy requests them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ManualFrame {
    /// Stores the `buttons` value in `ManualFrame` as `ButtonTrace`.
    ///
    ///
    ///  Holds the buttons value used by ManualFrame.
    pub buttons: ButtonTrace,
    /// Stores the `stick` value in `ManualFrame` as `[f32; 2]`.
    ///
    ///
    ///  Stores the stick collection for ManualFrame.
    pub stick: [f32; 2],
    /// Stores the `clear_autorepeat` value in `ManualFrame` as `u32`.
    ///
    ///
    ///  Holds the clear autorepeat value used by ManualFrame.
    pub clear_autorepeat: u32,
}
/// Defines the `ManualOverlay` data structure.
///
/// Holds the data for ManualOverlay.
#[derive(Default)]
pub struct ManualOverlay {
    /// Stores the `buttons` value in `ManualOverlay` as `MapperButtonOverlay`.
    ///
    ///
    ///  Holds the buttons value used by ManualOverlay.
    buttons: MapperButtonOverlay,
    /// Stores the `had_stick_override` value in `ManualOverlay` as `bool`.
    ///
    ///
    ///  Holds the had stick override value used by ManualOverlay.
    had_stick_override: bool,
    /// Stores the `reaction_owned` value in `ManualOverlay` as `bool`.
    ///
    ///
    ///  Holds the reaction owned value used by ManualOverlay.
    reaction_owned: bool,
}
impl ManualOverlay {
    /// Native Guard can have been removed after raw mapping by an earlier mode.
    /// A scheduler-confirmed new tech edge must not inherit that raw held bit.
    ///
    ///  Native Guard can have been removed after raw mapping by an earlier mode. A scheduler-confirmed new tech edge must not inherit that raw held bit.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `mask` (`u32`): Represents the mask; the function uses it as input to the operation described above.
    /// - `human` (`ButtonTrace`): Represents the human; the function uses it as input to the operation described above.
    pub fn rearm_tech_guard(&mut self, mask: u32, human: ButtonTrace) {
        self.buttons.rearm_requested(mask, human.previous);
    }

    /// Applies the supplied configuration or action.
    ///
    ///
    ///  Makes the supplied change.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `human` (`ButtonTrace`): Represents the human; the function uses it as input to the operation described above.
    /// - `physical_stick` (`[f32; 2]`): Represents the physical stick; the function uses it as input to the operation described above.
    /// - `output` (`TurboOutput`): Represents the output data being produced; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ManualFrame` value.
    pub fn apply(
        &mut self,
        human: ButtonTrace,
        physical_stick: [f32; 2],
        output: TurboOutput,
    ) -> ManualFrame {
        self.apply_controls(human, physical_stick, output, 0, 0, 0)
    }

    /// Compose an additional hold control through the SAME button history.
    /// Separate chained histories could fabricate edges when ownership changes.
    ///
    ///  Compose an additional hold control through the SAME button history. Separate chained histories could fabricate edges when ownership changes.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `human` (`ButtonTrace`): Represents the human; the function uses it as input to the operation described above.
    /// - `physical_stick` (`[f32; 2]`): Represents the physical stick; the function uses it as input to the operation described above.
    /// - `output` (`TurboOutput`): Represents the output data being produced; the function uses it as input to the operation described above.
    /// - `extra_consumed` (`u32`): Represents the extra consumed; the function uses it as input to the operation described above.
    /// - `requested_buttons` (`u32`): Represents the requested buttons; the function uses it as input to the operation described above.
    /// - `owned_buttons` (`u32`): Represents the owned buttons; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ManualFrame` value.
    pub fn apply_controls(
        &mut self,
        human: ButtonTrace,
        physical_stick: [f32; 2],
        output: TurboOutput,
        extra_consumed: u32,
        requested_buttons: u32,
        owned_buttons: u32,
    ) -> ManualFrame {
        self.apply_reaction_controls(
            human,
            physical_stick,
            output,
            extra_consumed,
            requested_buttons,
            owned_buttons,
            false,
        )
    }
    /// Only Auto react opts into a fresh first horizontal flick. Automatic
    /// grabs, ledge tapes, native safety and other helpers keep their history.
    ///
    ///  Only Auto react opts into a fresh first horizontal flick. Automatic grabs, ledge tapes, native safety and other helpers keep their history.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `human` (`ButtonTrace`): Represents the human; the function uses it as input to the operation described above.
    /// - `physical_stick` (`[f32; 2]`): Represents the physical stick; the function uses it as input to the operation described above.
    /// - `output` (`TurboOutput`): Represents the output data being produced; the function uses it as input to the operation described above.
    /// - `extra_consumed` (`u32`): Represents the extra consumed; the function uses it as input to the operation described above.
    /// - `requested_buttons` (`u32`): Represents the requested buttons; the function uses it as input to the operation described above.
    /// - `owned_buttons` (`u32`): Represents the owned buttons; the function uses it as input to the operation described above.
    /// - `reaction_owned` (`bool`): Represents the reaction owned; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ManualFrame` value.
    pub fn apply_reaction_controls(
        &mut self,
        human: ButtonTrace,
        physical_stick: [f32; 2],
        output: TurboOutput,
        extra_consumed: u32,
        requested_buttons: u32,
        owned_buttons: u32,
        reaction_owned: bool,
    ) -> ManualFrame {
        let entering_reaction = reaction_owned && !self.reaction_owned;
        self.reaction_owned = reaction_owned;
        if output.stick.is_none()
            && extra_consumed == 0
            && owned_buttons == 0
            && requested_buttons == 0
        {
            // One final history update releases any synthetic attack/shield
            // before returning ownership to fresh physical input.
            let buttons = self.buttons.apply_raw(human, 0, 0);
            let clear_autorepeat = if self.had_stick_override {
                LEFT_STICK_DIRECTIONS
            } else {
                0
            };
            self.had_stick_override = false;
            return ManualFrame {
                buttons,
                stick: physical_stick,
                clear_autorepeat,
            };
        }
        // The physical trigger was read before this call. Remove the D-pad
        // from all four words, including its history, so no taunt edge leaks.
        let consumed = crate::native_input::physical_button_aliases(extra_consumed);
        let owned_buttons = crate::native_input::physical_button_aliases(owned_buttons);
        let human = ButtonTrace {
            held: human.held & !consumed,
            previous: human.previous & !consumed,
            pressed: human.pressed & !consumed,
            released: human.released & !consumed,
        };
        // Raw direction bits accompany the normalized stick floats. Without
        // owning those bits, a physically held direction could disagree with
        // the automated stick value during the original mapper's button queries.
        let mut owned = consumed | owned_buttons;
        let mut requested = requested_buttons;
        if let Some([x, y]) = output.stick {
            owned |= LEFT_STICK_DIRECTIONS;
            // Analog values remain authoritative. These coarse direction bits
            // keep full crouch input consistent without marking gentle walking
            // as a fully deflected digital stick direction.
            if x >= 0.5 && consumed & (1 << 21) == 0 {
                requested |= 1 << 21;
            }
            if x <= -0.5 && consumed & (1 << 23) == 0 {
                requested |= 1 << 23;
            }
            if y >= 0.5 {
                requested |= 1 << 20;
            }
            if y <= -0.5 {
                requested |= 1 << 22;
            }
        }
        let clear_autorepeat = owned
            | if self.had_stick_override {
                LEFT_STICK_DIRECTIONS
            } else {
                0
            };
        self.had_stick_override = output.stick.is_some();
        if entering_reaction
            && output
                .stick
                .is_some_and(|s| s[0].abs() >= 0.95 && s[1].abs() <= 0.3)
        {
            self.buttons
                .rearm_requested(requested & ((1 << 21) | (1 << 23)), human.previous);
        }
        ManualFrame {
            // Menu controls can share buttons with a configured recipe. Physical
            // menu presses were filtered above; retain explicitly owned macro
            // press/release history rather than erasing the generated input.
            buttons: remove_consumed(
                self.buttons.apply_raw(human, requested, owned),
                consumed & !owned_buttons,
            ),
            stick: output.stick.unwrap_or(physical_stick),
            clear_autorepeat,
        }
    }
}

/// Apply consumption after history reconstruction too: old native trigger
/// history must never reappear as a synthetic taunt press/release.
///
///  Apply consumption after history reconstruction too: old native trigger history must never reappear as a synthetic taunt press/release.
/// ### Arguments
/// - `trace` (`ButtonTrace`): Represents the trace; the function uses it as input to the operation described above.
/// - `mask` (`u32`): Represents the mask; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `ButtonTrace` value.
fn remove_consumed(trace: ButtonTrace, mask: u32) -> ButtonTrace {
    ButtonTrace {
        held: trace.held & !mask,
        previous: trace.previous & !mask,
        pressed: trace.pressed & !mask,
        released: trace.released & !mask,
    }
}

/// Filter the mapper RESULT as well as raw inputs. Native mapping can retain
/// repeat/history state beyond the four raw words. Only reserved taunts are
/// removed; this neither adds attack commands nor changes unrelated buttons.
// These masks belong to the mapped ACTION format, not raw D-pad bit positions.
// A new trigger must handle both raw consumption and this final mapped filter
// if native taunt repeat/history can survive the raw overlay.
/// Physical bindings reserve taunts, but an explicit owned sequence request
/// must retain its ordinary D-pad meaning. Side-taunt filtering is a pair.
///
///  Filter the mapper RESULT as well as raw inputs. Native mapping can retain repeat/history state beyond the four raw words. Only reserved taunts are removed; this neither adds attack commands nor changes unrelated buttons.
/// ### Arguments
/// - `consumed` (`u32`): Represents the consumed; the function uses it as input to the operation described above.
/// - `requested` (`u32`): Represents the requested; the function uses it as input to the operation described above.
/// - `owned` (`u32`): Represents the owned; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `u32` value.
pub fn sequence_taunt_reservation(consumed: u32, requested: u32, owned: u32) -> u32 {
    let intentional = requested & owned & crate::native_input::MAPPER_DPAD_BUTTONS;
    let sides = if intentional & DPAD_SIDES != 0 {
        DPAD_SIDES
    } else {
        0
    };
    consumed & !(intentional | sides)
}
/// Determines whether reserved taunt inputs must be suppressed.
///
/// Checks whether the reserved taunt buttons should be ignored.
/// ### Arguments
/// - `mapped_buttons` (`u32`): Represents the mapped buttons; the function uses it as input to the operation described above.
/// - `consumed` (`u32`): Represents the consumed; the function uses it as input to the operation described above.
/// ### Returns
/// Returns the resulting `u32` value.
pub fn suppress_reserved_taunts(mapped_buttons: u32, consumed: u32) -> u32 {
    let sides = if consumed & DPAD_SIDES != 0 {
        crate::native_input::MAPPED_TAUNT_SIDES
    } else {
        0
    };
    let down = if consumed & crate::ledge_trap::DPAD_DOWN != 0 {
        crate::native_input::MAPPED_TAUNT_DOWN
    } else {
        0
    };
    let up = if consumed & DPAD_UP != 0 {
        crate::native_input::MAPPED_TAUNT_UP
    } else {
        0
    };
    mapped_buttons & !(sides | down | up)
}

#[cfg(test)]
#[path = "../tests/unit/manual_controls.rs"]
mod tests;

/// Constant-size physical-input history for explicit sequence handback.
///
///  Constant-size physical-input history for explicit sequence handback.
#[derive(Clone, Copy, Default)]
pub struct ManualTakeover {
    /// Stores the `previous` value in `ManualTakeover` as `Option<(u32`.
    ///
    ///
    ///  Optionally stores the previous for ManualTakeover; it may be absent.
    previous: Option<(u32, [f32; 4])>,
    /// Stores the `epoch` value in `ManualTakeover` as `u64`.
    ///
    ///
    ///  Holds the epoch value used by ManualTakeover.
    epoch: u64,
    /// Stores the `action_epoch` value in `ManualTakeover` as `u64`.
    ///
    ///
    ///  Holds the action epoch value used by ManualTakeover.
    action_epoch: u64,
}
impl ManualTakeover {
    /// Constructs a new value with the type’s initial state.
    ///
    ///
    ///  Creates a new value.
    /// ### Returns
    /// Returns a `Self` value representing the result of the operation.
    pub const fn new() -> Self {
        Self {
            previous: None,
            epoch: 0,
            action_epoch: 0,
        }
    }
    /// Returns the current session epoch.
    ///
    ///
    ///  Gets the current session generation.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `u64` value.
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    /// Returns the requested stick direction.
    ///
    ///
    ///  Gets the chosen stick direction.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `[f32; 2]` value.
    pub fn stick(&self) -> [f32; 2] {
        self.previous
            .map_or([0.; 2], |(_, axes)| [axes[0], axes[1]])
    }
    /// Physical gameplay-button/C-stick edges, separate from movement intent.
    /// Committed automation reads the owner's frozen policy snapshot.
    ///
    ///  Physical gameplay-button/C-stick edges, separate from movement intent. Committed automation reads the owner's frozen policy snapshot.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `u64` value.
    pub fn action_epoch(&self) -> u64 {
        self.action_epoch
    }
    /// Checks whether shield input is currently held.
    ///
    ///
    ///  Checks whether shield is being held.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn shield_held(&self) -> bool {
        self.previous
            .is_some_and(|(buttons, _)| buttons & crate::native_input::MAPPER_ZR != 0)
    }
    /// Native WAIT can lag a fresh run input. Stationary assists must inspect
    /// the physical stick too; an existing shield still permits OOS actions.
    ///
    ///  Native WAIT can lag a fresh run input. Stationary assists must inspect the physical stick too; an existing shield still permits OOS actions.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn stationary_assist(&self, status: i32) -> bool {
        use crate::native_input::*;
        self.previous.is_some_and(|(buttons, axes)| {
            if !axes.iter().all(|v| v.is_finite())
                || buttons & (MAPPER_A | MAPPER_B | MAPPER_R) != 0
                || axes[2].abs() > 0.2
                || axes[3].abs() > 0.2
            {
                return false;
            }
            let cap = if matches!(
                status,
                crate::native_states::GUARD_ON | crate::native_states::GUARD
            ) && buttons & MAPPER_ZR != 0
            {
                0.7
            } else {
                0.2
            };
            axes[0].abs() <= cap && axes[1].abs() <= cap
        })
    }
    /// Grounded offense may use a held horizontal walk/run input. Native or
    /// freshly pressed shield keeps the stricter tilt/roll gate; manual attacks,
    /// C-stick, and strong vertical jump/drop intent keep player priority.
    ///
    ///  Grounded offense may use a held horizontal walk/run input. Native or freshly pressed shield keeps the stricter tilt/roll gate; manual attacks, C-stick, and strong vertical jump/drop intent keep player priority.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn grounded_grab_assist(&self, status: i32) -> bool {
        use crate::native_input::*;
        self.previous.is_some_and(|(buttons, axes)| {
            if buttons & MAPPER_ZR != 0
                || matches!(
                    status,
                    crate::native_states::GUARD_ON..=crate::native_states::GUARD_DAMAGE
                )
            {
                return self.stationary_assist(status);
            }
            axes.iter().all(|v| v.is_finite())
                && axes[1].abs() <= 0.7
                && axes[2].abs() <= 0.2
                && axes[3].abs() <= 0.2
                && buttons & (MAPPER_A | MAPPER_B | MAPPER_R) == 0
        })
    }
    /// Updates the toggle state from the current button sample and match state.
    ///
    ///
    ///  Checks the current buttons and updates whether the toggle should activate.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `buttons` (`u32`): Represents the controller buttons being sampled; the function uses it as input to the operation described above.
    /// - `sticks` (`[f32; 4]`): Represents the sticks; the function uses it as input to the operation described above.
    pub fn sample(&mut self, buttons: u32, sticks: [f32; 4]) {
        use crate::native_input::*;
        let mask = MAPPER_A | MAPPER_B | MAPPER_X | MAPPER_Y | MAPPER_R | MAPPER_ZR | MAPPER_ZL;
        if let Some((old_buttons, old)) = self.previous {
            let pressed = buttons & !old_buttons & mask != 0;
            let moved = sticks.iter().zip(old).any(|(new, old)| {
                new.is_finite()
                    && old.is_finite()
                    && ((new.abs() > 0.25) != (old.abs() > 0.25) || (*new - old).abs() > 0.2)
            });
            if pressed || moved {
                self.epoch = self.epoch.wrapping_add(1);
            }
            let cstick_moved = sticks[2..].iter().zip(&old[2..]).any(|(new, old)| {
                new.is_finite()
                    && old.is_finite()
                    && ((new.abs() > 0.25) != (old.abs() > 0.25) || (*new - *old).abs() > 0.2)
            });
            if pressed || cstick_moved {
                self.action_epoch = self.action_epoch.wrapping_add(1);
            }
        }
        self.previous = Some((buttons, sticks));
    }
}
