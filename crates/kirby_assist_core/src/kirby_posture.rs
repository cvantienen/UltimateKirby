//! Optional grounded stick helpers. Attack buttons always remain physical.
use crate::{
    ledge_trap::LedgeOutput,
    manual_controls::{ShortcutContext, LEFT_STICK_DIRECTIONS, RIGHT_STICK_DIRECTIONS},
    native_input::{ButtonTrace, MAPPER_A, MAPPER_L, MAPPER_RIGHT_STICK_DOWN},
};
/// Physical A or down on a C-stick configured as Tilt Attack. The latter
/// remains a physical input; this helper never creates a C-stick attack.
pub const DTILT_INPUTS: u32 = MAPPER_A | MAPPER_RIGHT_STICK_DOWN;
pub const AUTO_CROUCH_RANGE: f32 = 45.0;
pub const AUTO_CROUCH_HEIGHT: f32 = 20.0;
#[derive(Default)]
pub struct Posture;
impl Posture {
    pub fn opponent_near(offset: Option<[f32; 2]>) -> bool {
        Self::opponent_near_with(offset,AUTO_CROUCH_RANGE,AUTO_CROUCH_HEIGHT)
    }
    pub fn opponent_near_with(offset:Option<[f32;2]>,range:f32,height:f32)->bool {
        offset.is_some_and(|[x, y]| {
            x.is_finite()
                && y.is_finite()
                && x.abs() <= range
                && y.abs() <= height
        })
    }
    /// Require clean physical neutral, including button release/history.
    /// This is a handoff check, not a timer or a delayed crouch recipe.
    pub fn idle_buttons_clear(buttons: ButtonTrace) -> bool {
        buttons.held | buttons.previous | buttons.pressed | buttons.released == 0
    }
    /// Facing preparation cannot affect aerials, other attacks or frozen
    /// locomotion. Down tilt is the exception: its hitstop can buffer a reversal.
    pub fn needs_target(status: i32, situation: i32, stopped: bool) -> bool {
        situation == 0 && (matches!(status, 0..=9 | 0x11..=0x15) && !stopped || status == 0x2c)
    }
    pub fn down_tilt_angle(buttons: u32, stick: [f32; 2]) -> bool {
        Self::down_tilt_intent(buttons, stick, [0.0; 2])
    }
    /// Read the real pre-map C-stick axes too: digital aliases may not exist
    /// yet on the first flick. Dominant Down avoids converting side/up tilts.
    pub fn down_c_stick(buttons: u32, right: [f32; 2]) -> bool {
        if right.iter().all(|v| v.is_finite()) && right.iter().any(|v| v.abs() > 0.2) {
            right[1] < -0.5 && right[1].abs() > right[0].abs()
        } else {
            buttons & MAPPER_RIGHT_STICK_DOWN != 0
        }
    }
    pub fn down_tilt_intent(buttons: u32, left: [f32; 2], right: [f32; 2]) -> bool {
        left[1] < -0.5 || Self::down_c_stick(buttons, right)
    }
    /// Only the winning grounded facing helper may apply this. Down remains
    /// the dominant C-stick axis; no attack button or fresh edge is generated.
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
    // No queued attack or captured facing: every input uses the latest target.
    pub fn cancel_pending(&mut self) {}
    pub fn sample(
        &mut self,
        enabled: bool,
        crouch: bool,
        turn: bool,
        buttons: u32,
        stick: [f32; 2],
        target: Option<f32>,
        c: ShortcutContext,
    ) -> LedgeOutput {
        self.sample_controls(enabled, crouch, turn, buttons, stick, [0.0; 2], target, c)
    }
    pub fn sample_controls(
        &mut self,
        enabled: bool,
        crouch: bool,
        turn: bool,
        buttons: u32,
        stick: [f32; 2],
        right: [f32; 2],
        target: Option<f32>,
        c: ShortcutContext,
    ) -> LedgeOutput {
        let c_down = Self::down_c_stick(buttons, right);
        let locomotion = matches!(c.status, 0..=9 | 0x11..=0x15);
        if !enabled || c.situation != 0
            || !(locomotion || c.status == 0x2c) // Current down tilt may buffer the next.
            || !stick.iter().all(|v| v.is_finite())
            || !c.facing.is_finite() || c.facing.abs() < 0.5
        {
            return LedgeOutput::default();
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
                return LedgeOutput {
                    // ManualOverlay owns stick axes itself. Do not mark this
                    // as walking: that suppresses the horizontal reversal flick.
                    stick: Some([x, -1.0]),
                    ..LedgeOutput::default()
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
            return LedgeOutput {
                stick: Some([0.0, -1.0]),
                ..LedgeOutput::default()
            };
        }
        LedgeOutput::default()
    }
}
