//! Input-only damage-hitstop SDI. No status/physics writes.
//! Native detection and tool priority live in the Switch adapter.

// Preserve the original public paths for existing callers.
pub use crate::kirby::{FairContext, FairFollowup, TripContext, TripFollowup, TripTarget};

use super::{is_new_hit, stick_engaged, LAUNCH_DI_FRAMES, MAX_DAMAGE_STOP_DI_FRAMES};

/// Defines the `SdiContext` data structure.
///
/// Holds the data for SdiContext.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdiContext {
    /// Stores the `object` value in `SdiContext` as `u32`.
    ///
    ///
///  Holds the object value used by SdiContext.
    pub object: u32,
    /// Stores the `damage_stop` value in `SdiContext` as `bool`.
    ///
    ///
///  Holds the damage stop value used by SdiContext.
    pub damage_stop: bool,
    /// Stores the `remaining` value in `SdiContext` as `u64`.
    ///
    ///
///  Holds the remaining value used by SdiContext.
    pub remaining: u64,
    /// Stores the `input_delay` value in `SdiContext` as `f32`.
    ///
    ///
///  Holds the input delay value used by SdiContext.
    pub input_delay: f32,
    /// Away from the attributed attacker (or sole opponent); zero if unknown.
    ///
///  Away from the attributed attacker (or sole opponent); zero if unknown.
    pub outward: f32,
}
/// Defines the `SdiState` data structure.
///
/// Holds the data for SdiState.
#[derive(Default)]
pub struct SdiState {
    /// Stores the `last` value in `SdiState` as `Option<(u32`.
    ///
    ///
///  Optionally stores the last for SdiState; it may be absent.
    last: Option<(u32, u64)>,
    /// Stores the `last_flick` value in `SdiState` as `Option<u64>`.
    ///
    ///
///  Optionally stores the last flick for SdiState; it may be absent.
    last_flick: Option<u64>,
    /// Stores the `output` value in `SdiState` as `Option<[f32; 2]>`.
    ///
    ///
///  Optionally stores the output for SdiState; it may be absent.
    output: Option<[f32; 2]>,
    /// Stores the `manual_control` value in `SdiState` as `bool`.
    ///
    ///
///  Holds the manual control value used by SdiState.
    manual_control: bool,
}
impl SdiState {
    /// Re-arm a real controller flick when the previous game-visible stick was
    /// already held. Repeated mapper polls retain that neutral release until
    /// native hitstop advances; a same-side held sample is never a fresh SDI.
    ///
///  Re-arm a real controller flick when the previous game-visible stick was already held. Repeated mapper polls retain that neutral release until native hitstop advances; a same-side held sample is never a fresh SDI.
    pub fn sample_exclusive_with_previous(
        &mut self,
        enabled: bool,
        ctx: SdiContext,
        geometry: Option<([f32; 2], crate::auto_di::BlastZone)>,
        previous: [f32; 2],
    ) -> Option<[f32; 2]> {
        let new_hit = is_new_hit(self.last, ctx.object, ctx.remaining);
        if new_hit && stick_engaged(previous) {
            self.sample_exclusive(enabled, ctx)?;
            self.last_flick = None;
            self.output = Some([0.; 2]);
            return self.output;
        }
        self.sample_exclusive_with_geometry(enabled, ctx, geometry)
    }
    /// Neutral flick gaps are owned too; physical movement cannot cancel an
    /// already selected damage response or leak into its later DI phase.
    ///
///  Neutral flick gaps are owned too; physical movement cannot cancel an already selected damage response or leak into its later DI phase.
    pub fn sample_exclusive(&mut self, enabled: bool, ctx: SdiContext) -> Option<[f32; 2]> {
        self.manual_control = false;
        self.sample(enabled, ctx, [0.; 2])
    }
    /// Samples the exclusive with geometry.
    ///
    ///
///  Samples the exclusive with geometry.
    pub fn sample_exclusive_with_geometry(
        &mut self,
        enabled: bool,
        ctx: SdiContext,
        geometry: Option<([f32; 2], crate::auto_di::BlastZone)>,
    ) -> Option<[f32; 2]> {
        let output = self.sample_exclusive(enabled, ctx)?;
        if output == [0.; 2] {
            return Some(output);
        }
        let Some((position, blast)) = geometry.filter(|(p, b)| b.contains(*p)) else {
            return Some(output);
        };
        // Maximize separation while preserving room for an SDI step. Near an
        // edge, bend inward/upward instead of repeatedly SDIing into death.
        let mut best = [0.; 2];
        let mut score = f32::NEG_INFINITY;
        for i in 0..8 {
            let angle = i as f32 * std::f32::consts::FRAC_PI_4;
            let (y, x) = angle.sin_cos();
            let clearance = blast.margin([position[0] + x * 8., position[1] + y * 8.]);
            let candidate = x * ctx.outward.signum() + clearance.min(8.) * 0.25;
            if candidate > score {
                score = candidate;
                best = [x, y];
            }
        }
        Some(best)
    }
    /// Updates the toggle state from the current button sample and match state.
    ///
    ///
///  Checks the current buttons and updates whether the toggle should activate.
    pub fn sample(
        &mut self,
        enabled: bool,
        ctx: SdiContext,
        physical: [f32; 2],
    ) -> Option<[f32; 2]> {
        // None means leave the player's stick alone. Some([0, 0]) deliberately
        // owns a neutral sample between flicks; those outputs are different.
        if !enabled
            || !ctx.damage_stop
            || ctx.remaining <= LAUNCH_DI_FRAMES
            || ctx.remaining > MAX_DAMAGE_STOP_DI_FRAMES
            || !ctx.input_delay.is_finite()
            || ctx.remaining as f32
                <= ctx
                    .input_delay
                    .clamp(0.0, crate::tech_input::MAX_INPUT_DELAY_FRAMES)
                    .ceil()
            || !physical.iter().all(|axis| axis.is_finite())
        {
            *self = Self::default();
            return None;
        }
        if is_new_hit(self.last, ctx.object, ctx.remaining) {
            self.manual_control = false;
            self.last_flick = None;
            self.output = None;
        }
        self.manual_control |= stick_engaged(physical);
        let key = (ctx.object, ctx.remaining);
        if !ctx.outward.is_finite() || ctx.outward.abs() <= 0.5 {
            self.last = Some(key);
            self.last_flick = None;
            self.output = None;
            return None;
        }
        if self.last != Some(key) {
            // Ultimate's hit_stop_delay_flick_frame is four frames. Count the
            // native countdown, never mapper polls; missed observations cannot
            // compress the interval or manufacture additional flicks.
            let due = self
                .last_flick
                .is_none_or(|remaining| remaining.saturating_sub(ctx.remaining) >= 4);
            // A neutral observation must actually precede another same-side
            // flick, even if several native frames were skipped.
            let rearmed = self.output != Some([ctx.outward.signum(), 0.]);
            self.output = Some(if due && rearmed {
                self.last_flick = Some(ctx.remaining);
                [ctx.outward.signum(), 0.]
            } else {
                [0.; 2]
            });
            self.last = Some(key);
        }
        if self.manual_control {
            return None;
        }
        self.output
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/combat_assist.rs"]
mod tests;
