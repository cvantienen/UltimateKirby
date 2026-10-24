//! Optional grounded stick helpers. Attack buttons always remain physical.
use crate::{
    ledge_trap::LedgeOutput,
    manual_controls::{ShortcutContext, LEFT_STICK_DIRECTIONS},
    native_input::{MAPPER_A, MAPPER_L},
};
/// Physical A or down on a C-stick configured as Tilt Attack. The latter
/// remains a physical input; this helper never creates a C-stick attack.
pub const DTILT_INPUTS: u32 = MAPPER_A | (1 << 26);
#[derive(Default)]
pub struct Posture;
impl Posture {
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
            && (!c.stopped || c.status == 0x2c)
            && stick[1] < -0.5
            && buttons & !(DTILT_INPUTS | MAPPER_L | LEFT_STICK_DIRECTIONS) == 0
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
