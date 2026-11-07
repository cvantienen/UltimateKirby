//! Input-only trip follow-up and damage-hitstop SDI. No status/physics writes.
//!
//! Two independent tools share this file: `TripFollowup` remembers a recognized
//! trip and retries forward smash after either tilt; `SdiState` chooses a stick flick during damage
//! hitstop. Each `sample` receives copied facts and returns an input proposal.
//! Native detection and tool priority live in the Switch adapter, not here.
mod fair_followup;
use crate::ledge_trap::{LedgeOutput, LEDGE_BUTTONS};
use crate::native_input::MAPPER_A;
pub use fair_followup::{FairContext, FairFollowup};

// Trip takeover also suppresses taunts; Plus/Minus remain available.
const TRIP_BUTTONS: u32 = LEDGE_BUTTONS | 0xf;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TripTarget {
    pub id: u32,
    pub status: i32,
    pub previous_status: i32,
}
fn trip_tilt(status: i32) -> bool {
    matches!(status, 0x2a | 0x2c)
}
fn tripping(status: i32) -> bool {
    matches!(status, 0x86..=0x88)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TripContext {
    pub object: u32,
    pub kirby: bool,
    pub status: i32,
    pub motion: u64,
    pub frame: f32,
    pub grounded: bool,
    pub facing: f32,
    pub stopped: bool,
    pub targets: [Option<TripTarget>; 8],
}
#[derive(Default)]
pub struct TripFollowup {
    last: Option<TripContext>,
    output: LedgeOutput,
    queued: bool,
    spent: bool,
    facing: f32,
}
impl TripFollowup {
    pub fn cancel_pending(&mut self) {
        self.queued = false;
        self.spent = true;
        self.output = LedgeOutput::default();
    }
    pub fn watching(&self) -> bool {
        self.queued
    }
    pub fn sample(&mut self, enabled: bool, ctx: TripContext) -> LedgeOutput {
        // Invalid/changed identity resets the tool. An identical observation
        // reuses output, so repeated mapper polls do not manufacture new pulses.
        if !enabled || !ctx.kirby || !ctx.frame.is_finite() || ctx.frame < 0.0 {
            *self = Self::default();
            return LedgeOutput::default();
        }
        if self.last.is_some_and(|last| last.object != ctx.object) {
            *self = Self::default();
        }
        if self.last == Some(ctx) {
            return self.output;
        }
        let native_progress = !ctx.stopped
            && self.last.is_none_or(|last| {
                ctx.status != last.status || ctx.motion != last.motion || ctx.frame != last.frame
            });
        let new_tilt = trip_tilt(ctx.status)
            && self.last.is_none_or(|last| {
                last.status != ctx.status || ctx.frame < last.frame || ctx.motion != last.motion
            });
        if new_tilt && !self.queued {
            self.spent = false;
        }
        // Recognize current trips and grounded re-hit damage whose immediate
        // previous state was a trip. No new-trip transition or hit flag required.
        let recognized = !self.queued
            && !self.spent
            && trip_tilt(ctx.status)
            && ctx.targets.iter().flatten().any(|enemy| {
                tripping(enemy.status)
                    || tripping(enemy.previous_status) && matches!(enemy.status, 0x47 | 0x5a)
            });
        if recognized {
            self.queued = true;
            self.facing = if ctx.facing.is_finite() && ctx.facing < 0.0 {
                -1.0
            } else {
                1.0
            };
        }
        let mut out = LedgeOutput::default();
        let was_queued = self.queued;
        if self.queued {
            let accepted = matches!(ctx.status, 0x2d..=0x2f);
            let supported =
                ctx.grounded && matches!(ctx.status, 0..=0x9 | 0x11..=0x15 | 0x2a | 0x2c);
            if accepted || !supported {
                self.queued = false;
                self.spent = true;
            } else if !recognized && !native_progress {
                // Changed target facts are evidence, not another input frame.
                // Preserve the current press/release through duplicate polls.
                self.last = Some(ctx);
                return self.output;
            } else {
                // Own every competing gameplay button and both sticks from
                // recognition onward, including tilt lag and release gaps.
                out.owned = TRIP_BUTTONS;
                out.stick = Some([0.0, 0.0]);
                // First release the physical tilt/A. Then keep sending fresh
                // directional+A flicks into the game's normal buffer until smash
                // startup is observed. No readiness gate or six-pulse cutoff.
                if !recognized && !ctx.stopped && self.output.requested & MAPPER_A == 0 {
                    out.requested = MAPPER_A;
                    out.stick = Some(crate::move_input::smash_stick(
                        crate::move_input::Smash::Forward,
                        self.facing,
                    ));
                }
            }
        }
        // Release immediately at smash startup so the macro does not charge it.
        if out.owned == 0 && was_queued {
            out.owned = TRIP_BUTTONS;
            out.stick = Some([0.0, 0.0]);
        }
        self.last = Some(ctx);
        self.output = out;
        out
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdiContext {
    pub object: u32,
    pub damage_stop: bool,
    pub remaining: u64,
    pub input_delay: f32,
    /// Away from the attributed attacker (or sole opponent); zero if unknown.
    pub outward: f32,
}
#[derive(Default)]
pub struct SdiState {
    last: Option<(u32, u64)>,
    last_flick: Option<u64>,
    output: Option<[f32; 2]>,
    manual_control: bool,
}
impl SdiState {
    /// Re-arm a real controller flick when the previous game-visible stick was
    /// already held. Repeated mapper polls retain that neutral release until
    /// native hitstop advances; a same-side held sample is never a fresh SDI.
    pub fn sample_exclusive_with_previous(&mut self, enabled:bool, ctx:SdiContext,
        geometry:Option<([f32;2],crate::auto_di::BlastZone)>, previous:[f32;2]) -> Option<[f32;2]> {
        let new_hit=self.last.is_none_or(|(id,left)|id!=ctx.object || ctx.remaining>left);
        if new_hit && previous[0].hypot(previous[1])>=0.2 {
            self.sample_exclusive(enabled,ctx)?;
            self.last_flick=None;
            self.output=Some([0.;2]);
            return self.output;
        }
        self.sample_exclusive_with_geometry(enabled,ctx,geometry)
    }
    /// Neutral flick gaps are owned too; physical movement cannot cancel an
    /// already selected damage response or leak into its later DI phase.
    pub fn sample_exclusive(&mut self, enabled: bool, ctx: SdiContext) -> Option<[f32; 2]> {
        self.manual_control = false;
        self.sample(enabled, ctx, [0.; 2])
    }
    pub fn sample_exclusive_with_geometry(&mut self, enabled: bool, ctx: SdiContext,
        geometry: Option<([f32; 2], crate::auto_di::BlastZone)>) -> Option<[f32; 2]> {
        let output = self.sample_exclusive(enabled, ctx)?;
        if output == [0.; 2] {return Some(output);}
        let Some((position, blast)) = geometry.filter(|(p,b)| b.contains(*p)) else {
            return Some(output);
        };
        // Maximize separation while preserving room for an SDI step. Near an
        // edge, bend inward/upward instead of repeatedly SDIing into death.
        let mut best = [0.; 2];
        let mut score = f32::NEG_INFINITY;
        for i in 0..8 {
            let angle = i as f32 * std::f32::consts::FRAC_PI_4;
            let (y,x) = angle.sin_cos();
            let clearance = blast.margin([position[0]+x*8., position[1]+y*8.]);
            let candidate = x * ctx.outward.signum() + clearance.min(8.) * 0.25;
            if candidate > score {score=candidate;best=[x,y];}
        }
        Some(best)
    }
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
            || ctx.remaining <= 3 // Reserve final launch-input frames for DI/manual control.
            || ctx.remaining > 120
            || !ctx.input_delay.is_finite()
            || ctx.remaining as f32 <= ctx.input_delay.clamp(0.0, 12.0).ceil()
        {
            *self = Self::default();
            return None;
        }
        let physical = if physical.iter().all(|axis| axis.is_finite()) {
            [physical[0].clamp(-1.0, 1.0), physical[1].clamp(-1.0, 1.0)]
        } else {
            *self = Self::default();
            return None;
        };
        let new_hit = self
            .last
            .is_none_or(|(id, remaining)| id != ctx.object || ctx.remaining > remaining);
        if new_hit {
            self.manual_control = false;
            self.last_flick = None;
            self.output = None;
        }
        self.manual_control |= physical[0].hypot(physical[1]) >= 0.2;
        let key = (ctx.object, ctx.remaining);
        if !ctx.outward.is_finite() || ctx.outward.abs()<=0.5 {
            self.last=Some(key);
            self.last_flick=None;
            self.output=None;
            return None;
        }
        if self.last != Some(key) {
            // Ultimate's hit_stop_delay_flick_frame is four frames. Count the
            // native countdown, never mapper polls; missed observations cannot
            // compress the interval or manufacture additional flicks.
            let due = self.last_flick.is_none_or(|remaining|
                remaining.saturating_sub(ctx.remaining) >= 4);
            // A neutral observation must actually precede another same-side
            // flick, even if several native frames were skipped.
            let rearmed = self.output != Some([ctx.outward.signum(), 0.]);
            self.output = Some(if due && rearmed {
                self.last_flick = Some(ctx.remaining);
                [ctx.outward.signum(), 0.]
            } else { [0.;2] });
            self.last = Some(key);
        }
        if self.manual_control {
            return None;
        }
        self.output
    }
}

#[cfg(test)]
mod tests;
