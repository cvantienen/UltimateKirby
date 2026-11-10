//! Optional grounded stick helpers. Attack buttons always remain physical.
use crate::{
    gameplay::ControllerOutput,
    manual_controls::{ManualContext, LEFT_STICK_DIRECTIONS, RIGHT_STICK_DIRECTIONS},
    native_input::{
        ButtonTrace, MAPPER_A, MAPPER_L, MAPPER_RIGHT_STICK_DOWN, MAPPER_RIGHT_STICK_LEFT,
        MAPPER_RIGHT_STICK_RIGHT,
    },
};
/// Physical A or down on a C-stick configured as Tilt Attack. The latter
/// remains a physical input; this helper never creates a C-stick attack.
///
///  Physical A or down on a C-stick configured as Tilt Attack. The latter remains a physical input; this helper never creates a C-stick attack.
pub const DTILT_INPUTS: u32 = MAPPER_A | MAPPER_RIGHT_STICK_DOWN;
/// Defines the `AUTO_CROUCH_RANGE` constant used by this module.
///
/// Provides the AUTO CROUCH RANGE constant value.
pub const AUTO_CROUCH_RANGE: f32 = 45.0;
/// Defines the `AUTO_CROUCH_HEIGHT` constant used by this module.
///
/// Provides the AUTO CROUCH HEIGHT constant value.
pub const AUTO_CROUCH_HEIGHT: f32 = 20.0;

/// Defines the `TiltSettings` data structure.
///
/// Holds the data for TiltSettings.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TiltSettings {
    /// Stores the `down` value in `TiltSettings` as `bool`.
    ///
    ///
    ///  Holds the down value used by TiltSettings.
    pub down: bool,
    /// Stores the `forward` value in `TiltSettings` as `bool`.
    ///
    ///
    ///  Holds the forward value used by TiltSettings.
    pub forward: bool,
    /// Stores the `up` value in `TiltSettings` as `bool`.
    ///
    ///
    ///  Holds the up value used by TiltSettings.
    pub up: bool,
}
impl TiltSettings {
    /// Determines whether the `any` condition holds.
    ///
    ///
    ///  Checks whether any.
    /// ### Arguments
    /// - `self` (`self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn any(self) -> bool {
        self.down || self.forward || self.up
    }
}

/// Keep the digital side aliases consistent with an angled physical C-stick.
/// Preserve physical attack edges; a target change does not invent a press.
///
///  Keep the digital side aliases consistent with an angled physical C-stick. Preserve physical attack edges; a target change does not invent a press.
pub struct TiltSideAliases {
    /// Stores the `previous` value in `TiltSideAliases` as `Option<u32>`.
    ///
    ///
    ///  Optionally stores the previous for TiltSideAliases; it may be absent.
    previous: Option<u32>,
}
impl Default for TiltSideAliases {
    /// Returns the default value for this type.
    ///
    ///
    ///  Returns the default value for this type.
    /// ### Returns
    /// Returns a `Self` value representing the result of the operation.
    fn default() -> Self {
        Self::new()
    }
}
impl TiltSideAliases {
    /// Constructs a new value with the type’s initial state.
    ///
    ///
    ///  Creates a new value.
    /// ### Returns
    /// Returns a `Self` value representing the result of the operation.
    pub const fn new() -> Self {
        Self { previous: None }
    }

    /// Applies the supplied configuration or action.
    ///
    ///
    ///  Makes the supplied change.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `trace` (`ButtonTrace`): Represents the trace; the function uses it as input to the operation described above.
    /// - `direction` (`Option<f32>`): Represents the direction; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ButtonTrace` value.
    pub fn apply(&mut self, mut trace: ButtonTrace, direction: Option<f32>) -> ButtonTrace {
        let current = direction.map(|x| {
            if x < 0.0 {
                MAPPER_RIGHT_STICK_LEFT
            } else {
                MAPPER_RIGHT_STICK_RIGHT
            }
        });
        let previous = self.previous.or(current);
        let map = |bits: u32, side: Option<u32>| {
            let mask = MAPPER_RIGHT_STICK_LEFT | MAPPER_RIGHT_STICK_RIGHT;
            match side {
                Some(side) if bits & mask != 0 => (bits & !mask) | side,
                _ => bits,
            }
        };
        trace.held = map(trace.held, current);
        trace.previous = map(trace.previous, previous);
        trace.pressed = map(trace.pressed, current);
        trace.released = map(trace.released, previous);
        self.previous = current;
        trace
    }
}

/// Defines the `Posture` data structure.
///
/// Holds the data for Posture.
#[derive(Default)]
pub struct Posture;
impl Posture {
    /// Determines whether the `opponent_near` condition holds.
    ///
    ///
    ///  Checks whether opponent near.
    /// ### Arguments
    /// - `offset` (`Option<[f32; 2]>`): Represents the offset; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn opponent_near(offset: Option<[f32; 2]>) -> bool {
        Self::opponent_near_with(offset, AUTO_CROUCH_RANGE, AUTO_CROUCH_HEIGHT)
    }
    /// Determines whether the `opponent_near_with` condition holds.
    ///
    ///
    ///  Checks whether opponent near with.
    /// ### Arguments
    /// - `offset` (`Option<[f32; 2]>`): Represents the offset; the function uses it as input to the operation described above.
    /// - `range` (`f32`): Represents the range; the function uses it as input to the operation described above.
    /// - `height` (`f32`): Represents the height; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn opponent_near_with(offset: Option<[f32; 2]>, range: f32, height: f32) -> bool {
        offset.is_some_and(|[x, y]| {
            x.is_finite() && y.is_finite() && x.abs() <= range && y.abs() <= height
        })
    }
    /// Require clean physical neutral, including button release/history.
    /// This is a handoff check, not a timer or a delayed crouch recipe.
    ///
    ///  Require clean physical neutral, including button release/history. This is a handoff check, not a timer or a delayed crouch recipe.
    /// ### Arguments
    /// - `buttons` (`ButtonTrace`): Represents the controller buttons being sampled; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn idle_buttons_clear(buttons: ButtonTrace) -> bool {
        buttons.held | buttons.previous | buttons.pressed | buttons.released == 0
    }
    /// Facing preparation cannot affect aerials, other attacks or frozen
    /// locomotion. Down tilt is the exception: its hitstop can buffer a reversal.
    ///
    ///  Facing preparation cannot affect aerials, other attacks or frozen locomotion. Down tilt is the exception: its hitstop can buffer a reversal.
    /// ### Arguments
    /// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
    /// - `situation` (`i32`): Represents the situation; the function uses it as input to the operation described above.
    /// - `stopped` (`bool`): Represents the stopped; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn needs_target(status: i32, situation: i32, stopped: bool) -> bool {
        situation == crate::native_states::situation::GROUND
            && (matches!(status, crate::native_states::status::WAIT..=crate::native_states::status::TURN_RUN_BRAKE | crate::native_states::status::SQUAT..=crate::native_states::status::SQUAT_RV)
                && !stopped
                || status == crate::native_states::status::ATTACK_LW3)
    }
    /// Determines whether configured target.
    ///
    ///
    ///  Checks whether configured target.
    /// ### Arguments
    /// - `status` (`i32`): Represents the status being checked; the function uses it as input to the operation described above.
    /// - `situation` (`i32`): Represents the situation; the function uses it as input to the operation described above.
    /// - `stopped` (`bool`): Represents the stopped; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn needs_configured_target(status: i32, situation: i32, stopped: bool) -> bool {
        Self::needs_target(status, situation, stopped)
            || situation == crate::native_states::situation::GROUND
                && matches!(
                    status,
                    crate::native_states::status::ATTACK_S3
                        | crate::native_states::status::ATTACK_HI3
                )
    }
    /// Determines whether the `any_tilt_intent` condition holds.
    ///
    ///
    ///  Checks whether any tilt intent.
    /// ### Arguments
    /// - `buttons` (`u32`): Represents the controller buttons being sampled; the function uses it as input to the operation described above.
    /// - `left` (`[f32; 2]`): Represents the left; the function uses it as input to the operation described above.
    /// - `right` (`[f32; 2]`): Represents the right; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn any_tilt_intent(buttons: u32, left: [f32; 2], right: [f32; 2]) -> bool {
        Self::down_tilt_intent(buttons, left, right)
            || right.iter().all(|v| v.is_finite()) && (right[0].abs() > 0.5 || right[1] > 0.5)
    }
    /// Up and forward are explicitly opted in for a C-stick mapped to Tilt Attack.
    /// Require real analog input, not a stale digital direction alias.
    ///
    ///  Up and forward are explicitly opted in for a C-stick mapped to Tilt Attack. Require real analog input, not a stale digital direction alias.
    /// ### Arguments
    /// - `settings` (`TiltSettings`): Represents the settings; the function uses it as input to the operation described above.
    /// - `buttons` (`u32`): Represents the controller buttons being sampled; the function uses it as input to the operation described above.
    /// - `left` (`[f32; 2]`): Represents the left; the function uses it as input to the operation described above.
    /// - `right` (`[f32; 2]`): Represents the right; the function uses it as input to the operation described above.
    /// - `target` (`Option<f32>`): Represents the target object or value being checked; the function uses it as input to the operation described above.
    /// - `c` (`ManualContext`): Represents the c; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
    pub fn configured_c_stick(
        settings: TiltSettings,
        buttons: u32,
        left: [f32; 2],
        right: [f32; 2],
        target: Option<f32>,
        c: ManualContext,
    ) -> Option<([f32; 2], [f32; 2])> {
        if !Self::needs_configured_target(c.status, c.situation, c.stopped)
            || !left.iter().all(|v| v.is_finite() && v.abs() <= 0.2)
            || !right.iter().all(|v| v.is_finite())
            || !c.facing.is_finite()
            || c.facing.abs() < 0.5
            || buttons & !(MAPPER_L | RIGHT_STICK_DIRECTIONS | LEFT_STICK_DIRECTIONS) != 0
        {
            return None;
        }
        let toward = target.filter(|v| v.is_finite() && v.abs() > 0.5)?.signum();
        let x = if c.facing * toward < 0.0 {
            toward * 0.8
        } else {
            0.0
        };
        if settings.up && right[1] > 0.5 && right[1].abs() > right[0].abs() {
            Some(([x, 0.0], [x, 1.0]))
        } else if settings.forward && right[0].abs() > 0.5 && right[0].abs() > right[1].abs() {
            Some(([x, 0.0], [toward, 0.0]))
        } else {
            None
        }
    }
    /// Samples the configured.
    ///
    ///
    ///  Samples the configured.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `enabled` (`bool`): Represents whether the feature or option should be enabled; the function uses it as input to the operation described above.
    /// - `crouch` (`bool`): Represents the crouch; the function uses it as input to the operation described above.
    /// - `turn` (`bool`): Represents the turn; the function uses it as input to the operation described above.
    /// - `settings` (`TiltSettings`): Represents the settings; the function uses it as input to the operation described above.
    /// - `buttons` (`u32`): Represents the controller buttons being sampled; the function uses it as input to the operation described above.
    /// - `stick` (`[f32; 2]`): Represents the stick; the function uses it as input to the operation described above.
    /// - `right` (`[f32; 2]`): Represents the right; the function uses it as input to the operation described above.
    /// - `target` (`Option<f32>`): Represents the target object or value being checked; the function uses it as input to the operation described above.
    /// - `c` (`ManualContext`): Represents the c; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ControllerOutput` value.
    pub fn sample_configured(
        &mut self,
        enabled: bool,
        crouch: bool,
        turn: bool,
        settings: TiltSettings,
        buttons: u32,
        stick: [f32; 2],
        right: [f32; 2],
        target: Option<f32>,
        c: ManualContext,
    ) -> ControllerOutput {
        if enabled && turn {
            if let Some((left, _)) =
                Self::configured_c_stick(settings, buttons, stick, right, target, c)
            {
                return ControllerOutput {
                    stick: Some(left),
                    ..ControllerOutput::default()
                };
            }
        }
        self.sample_controls(
            enabled,
            crouch,
            turn && settings.down,
            buttons,
            stick,
            right,
            target,
            c,
        )
    }
    /// Read the real pre-map C-stick axes too: digital aliases may not exist
    /// yet on the first flick. Dominant Down avoids converting side/up tilts.
    ///
    ///  Read the real pre-map C-stick axes too: digital aliases may not exist yet on the first flick. Dominant Down avoids converting side/up tilts.
    /// ### Arguments
    /// - `buttons` (`u32`): Represents the controller buttons being sampled; the function uses it as input to the operation described above.
    /// - `right` (`[f32; 2]`): Represents the right; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn down_c_stick(buttons: u32, right: [f32; 2]) -> bool {
        if right.iter().all(|v| v.is_finite()) && right.iter().any(|v| v.abs() > 0.2) {
            right[1] < -0.5 && right[1].abs() > right[0].abs()
        } else {
            buttons & MAPPER_RIGHT_STICK_DOWN != 0
        }
    }
    /// Determines whether the `down_tilt_intent` condition holds.
    ///
    ///
    ///  Checks whether down tilt intent.
    /// ### Arguments
    /// - `buttons` (`u32`): Represents the controller buttons being sampled; the function uses it as input to the operation described above.
    /// - `left` (`[f32; 2]`): Represents the left; the function uses it as input to the operation described above.
    /// - `right` (`[f32; 2]`): Represents the right; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `true` when the documented condition holds; otherwise, returns `false`.
    pub fn down_tilt_intent(buttons: u32, left: [f32; 2], right: [f32; 2]) -> bool {
        left[1] < -0.5 || Self::down_c_stick(buttons, right)
    }
    /// Only the winning grounded facing helper may apply this. Down remains
    /// the dominant C-stick axis; no attack button or fresh edge is generated.
    ///
    ///  Only the winning grounded facing helper may apply this. Down remains the dominant C-stick axis; no attack button or fresh edge is generated.
    /// ### Arguments
    /// - `buttons` (`u32`): Represents the controller buttons being sampled; the function uses it as input to the operation described above.
    /// - `right` (`[f32; 2]`): Represents the right; the function uses it as input to the operation described above.
    /// - `prepared` (`Option<[f32; 2]>`): Represents the prepared; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns `Some(value)` when a value is available; otherwise, returns `None`.
    pub fn aimed_c_stick(
        buttons: u32,
        right: [f32; 2],
        prepared: Option<[f32; 2]>,
    ) -> Option<[f32; 2]> {
        // Do not turn a stale digital alias into a new physical stick stroke.
        (Self::down_c_stick(buttons, right)
            && right.iter().all(|v| v.is_finite())
            && right[1] < -0.5)
            .then_some(())?;
        prepared
            .filter(|s| s.iter().all(|v| v.is_finite()) && s[1] < -0.5)
            .map(|s| [s[0], -1.0])
    }
    /// Updates the toggle state from the current button sample and match state.
    ///
    ///
    ///  Checks the current buttons and updates whether the toggle should activate.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `enabled` (`bool`): Represents whether the feature or option should be enabled; the function uses it as input to the operation described above.
    /// - `crouch` (`bool`): Represents the crouch; the function uses it as input to the operation described above.
    /// - `turn` (`bool`): Represents the turn; the function uses it as input to the operation described above.
    /// - `buttons` (`u32`): Represents the controller buttons being sampled; the function uses it as input to the operation described above.
    /// - `stick` (`[f32; 2]`): Represents the stick; the function uses it as input to the operation described above.
    /// - `target` (`Option<f32>`): Represents the target object or value being checked; the function uses it as input to the operation described above.
    /// - `c` (`ManualContext`): Represents the c; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ControllerOutput` value.
    pub fn sample(
        &mut self,
        enabled: bool,
        crouch: bool,
        turn: bool,
        buttons: u32,
        stick: [f32; 2],
        target: Option<f32>,
        c: ManualContext,
    ) -> ControllerOutput {
        self.sample_controls(enabled, crouch, turn, buttons, stick, [0.0; 2], target, c)
    }
    /// Samples the controls.
    ///
    ///
    ///  Samples the controls.
    /// ### Arguments
    /// - `self` (`&self`): Represents the current instance being read or updated; the function uses it as input to the operation described above.
    /// - `enabled` (`bool`): Represents whether the feature or option should be enabled; the function uses it as input to the operation described above.
    /// - `crouch` (`bool`): Represents the crouch; the function uses it as input to the operation described above.
    /// - `turn` (`bool`): Represents the turn; the function uses it as input to the operation described above.
    /// - `buttons` (`u32`): Represents the controller buttons being sampled; the function uses it as input to the operation described above.
    /// - `stick` (`[f32; 2]`): Represents the stick; the function uses it as input to the operation described above.
    /// - `right` (`[f32; 2]`): Represents the right; the function uses it as input to the operation described above.
    /// - `target` (`Option<f32>`): Represents the target object or value being checked; the function uses it as input to the operation described above.
    /// - `c` (`ManualContext`): Represents the c; the function uses it as input to the operation described above.
    /// ### Returns
    /// Returns the resulting `ControllerOutput` value.
    pub fn sample_controls(
        &mut self,
        enabled: bool,
        crouch: bool,
        turn: bool,
        buttons: u32,
        stick: [f32; 2],
        right: [f32; 2],
        target: Option<f32>,
        c: ManualContext,
    ) -> ControllerOutput {
        let c_down = Self::down_c_stick(buttons, right);
        let locomotion = matches!(c.status, crate::native_states::status::WAIT..=crate::native_states::status::TURN_RUN_BRAKE | crate::native_states::status::SQUAT..=crate::native_states::status::SQUAT_RV);
        if !enabled || c.situation != crate::native_states::situation::GROUND
            || !(locomotion || c.status == crate::native_states::status::ATTACK_LW3) // Current down tilt may buffer the next.
            || !stick.iter().all(|v| v.is_finite())
            || !c.facing.is_finite() || c.facing.abs() < 0.5
        {
            return ControllerOutput::default();
        }
        // A held directional tilt may be buffered during its hitstop. Releasing
        // its angle there lets buffered A select the old facing. This is only
        // stick preparation; no attack edge or timer is generated while stopped.
        if turn
            && Self::needs_target(c.status, c.situation, c.stopped)
            && Self::down_tilt_intent(buttons, stick, right)
            && (c_down || right.iter().all(|v| v.is_finite() && v.abs() <= 0.2))
            && buttons
                & !(DTILT_INPUTS
                    | MAPPER_L
                    | LEFT_STICK_DIRECTIONS
                    | if c_down { RIGHT_STICK_DIRECTIONS } else { 0 })
                == 0
        {
            if let Some(toward) = target.filter(|v| v.is_finite() && v.abs() > 0.5) {
                // Keep Down fully held. Angle toward the enemy only when a
                // reversal is needed; never stand/walk first or manufacture A.
                // Prepare the angle BEFORE A is pressed, including while
                // crouching. Waiting for A can let native command buffering
                // select the old facing before the turnaround is established.
                // Re-evaluate during recovery so mashed inputs aim the next tilt.
                let x = if c.facing * toward < 0.0 {
                    toward.signum() * 0.8
                } else {
                    0.0
                };
                return ControllerOutput {
                    // ManualOverlay owns stick axes itself. Do not mark this
                    // as walking: that suppresses the horizontal reversal flick.
                    stick: Some([x, -1.0]),
                    ..ControllerOutput::default()
                };
            }
        }
        // Hold through crouch animation loops; no animation timer can release it.
        if crouch
            && !c.stopped
            && locomotion
            && buttons == 0
            && right.iter().all(|v| v.is_finite() && v.abs() <= 0.08)
            && stick[0].abs() <= 0.08
            && stick[1].abs() <= 0.08
        {
            return ControllerOutput {
                stick: Some([0.0, -1.0]),
                ..ControllerOutput::default()
            };
        }
        ControllerOutput::default()
    }
}
