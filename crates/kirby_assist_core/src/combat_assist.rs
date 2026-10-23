//! Input-only trip follow-up and damage-hitstop SDI. No status/physics writes.
//!
//! Two independent tools share this file: `TripFollowup` remembers a recognized
//! trip and retries forward smash; `SdiState` chooses a stick flick during damage
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
        let new_tilt = trip_tilt(ctx.status)
            && self.last.is_none_or(|last| {
                last.status != ctx.status || ctx.frame < last.frame || ctx.motion != last.motion
            });
        if new_tilt && !self.queued {
            self.spent = false;
        }
        // Recognition is deliberately just tilt + opponent trip. No hit flag,
        // last-attacker attribution, trip age, spacing or protection checks.
        let recognized = !self.queued
            && !self.spent
            && trip_tilt(ctx.status)
            && ctx
                .targets
                .iter()
                .flatten()
                .any(|enemy| tripping(enemy.status));
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
            } else {
                // Own every competing gameplay button and both sticks from
                // recognition onward, including tilt lag and release gaps.
                out.owned = TRIP_BUTTONS;
                out.stick = Some([0.0, 0.0]);
                // First release the physical tilt/A. Then keep sending fresh
                // forward+A flicks into the game's normal buffer until smash
                // startup is observed. No readiness gate or six-pulse cutoff.
                if !recognized && !ctx.stopped && self.output.requested & MAPPER_A == 0 {
                    out.requested = MAPPER_A;
                    out.stick = Some([self.facing, 0.0]);
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
    outward_phase: bool,
    entry_stick: [f32; 2],
    manual_control: bool,
}
impl SdiState {
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
            || ctx.remaining == 0
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
            [0.0; 2]
        };
        if self.last.is_none_or(|(id, _)| id != ctx.object) {
            self.entry_stick = physical;
            self.manual_control = false;
        } else if physical[0].hypot(physical[1]) < 0.2 {
            // Releasing then re-engaging the same direction is also a new
            // deliberate input, even if it matches the pre-hit stick.
            self.entry_stick = [0.0; 2];
        } else if physical[0].hypot(physical[1]) >= 0.4
            && (physical[0] - self.entry_stick[0]).hypot(physical[1] - self.entry_stick[1]) >= 0.4
        {
            self.manual_control = true;
        }
        // A stick held from before the hit is not a deliberate SDI override.
        // A new movement hands the stick back fully until this hitstop ends.
        if self.manual_control {
            return None;
        }
        let key = (ctx.object, ctx.remaining);
        if self.last != Some(key) {
            // New/increased hitstop is another hit: start with an outward flick.
            self.outward_phase = if self
                .last
                .is_none_or(|(id, remaining)| id != ctx.object || ctx.remaining > remaining)
            {
                true
            } else {
                !self.outward_phase
            };
            self.last = Some(key);
        }
        if !ctx.outward.is_finite() || ctx.outward.abs() <= 0.5 {
            return None; // Unknown attacker direction: never guess into them.
        }
        if !self.outward_phase {
            return Some([0.0, 0.0]);
        }
        Some([ctx.outward.signum(), 0.0])
    }
}

#[cfg(test)]
mod tests;
