//! D-pad short-hop -> earliest-airborne downward-angled dodge attempt.
//! The live caller withholds D-pad triggers while the menu is visible.
//! Tap for one attempt; hold to repeat once native grounded jump becomes ready.
//!
//! `Phase` is a small state machine: remember the requested jump, wait for the
//! native airborne state, request the angled dodge, then wait for landing.
//! Inputs request transitions; observing the game confirms them. Poll-count
//! timeouts bound abandoned attempts and do not replace animation readiness.
use crate::ledge_trap::{LedgeOutput, LEDGE_BUTTONS};
use crate::manual_controls::{DPAD_LEFT, DPAD_RIGHT, DPAD_SIDES};
use crate::native_input::{MAPPER_X, MAPPER_ZR};

// Requested raw controller values; on-device slide distance is not established.
// Restore the shallow downward vector used before the vertical-angle change.
// Retain the same vector with ZR until native dodge acceptance.
pub const SLIDE_STICK_X: f32 = 0.9588;
pub const SLIDE_STICK_Y: f32 = -0.2839;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlideContext {
    pub gameplay: bool,
    pub stopped: bool,
    pub status: i32,
    pub situation: i32,
    pub motion_frame: f32,
    pub can_jump: bool,
    pub can_air_dodge: bool,
    pub safe_ground: [bool; 2],
    pub input_delay: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Phase {
    #[default]
    Idle,
    Queued,
    Takeoff,
    Dodge,
    Landing,
}

#[derive(Default)]
pub struct SlideState {
    phase: Phase,
    direction: i8,
    previous_direction: i8,
    started: i64,
    last_sample: Option<(i64, u32, SlideContext)>,
    last_output: LedgeOutput,
}
impl SlideState {
    /// Keep sampling an in-flight tool after its physical trigger is released.
    pub fn active(&self) -> bool {
        self.phase != Phase::Idle
    }

    /// Preserve only an already-owned hop/dodge through native stop. Never
    /// issue a new press or hold Jump while no native clock is available.
    pub fn frozen_output(&self, ctx: SlideContext) -> LedgeOutput {
        if !ctx.gameplay
            || !ctx.stopped
            || !ctx.motion_frame.is_finite()
            || !matches!(ctx.situation, 0 | 2)
        {
            return LedgeOutput::default();
        }
        match self.phase {
            Phase::Takeoff if matches!(ctx.status, 0..=0xb | 0x11..=0x15) => owned(),
            Phase::Dodge if matches!(ctx.status, 0xa | 0xb) => self.last_output,
            _ => LedgeOutput::default(),
        }
    }

    pub fn sample(
        &mut self,
        enabled: bool,
        tick: i64,
        physical: u32,
        ctx: SlideContext,
    ) -> LedgeOutput {
        if !enabled || !ctx.gameplay || !ctx.motion_frame.is_finite() {
            *self = Self::default();
            return LedgeOutput::default();
        }
        if self.last_sample.is_some_and(|last| tick < last.0) {
            *self = Self::default();
        }
        let key = (tick, physical, ctx);
        if self.last_sample == Some(key) {
            return self.last_output;
        }
        let direction = match physical & DPAD_SIDES {
            DPAD_LEFT => -1,
            DPAD_RIGHT => 1,
            _ => 0,
        };
        let pressed = direction != 0 && direction != self.previous_direction;
        self.previous_direction = direction;
        // Damage/captures/ledge states cancel. A grounded attack/recovery can
        // queue a tap, but receives no override until native jump readiness.
        let supported_status =
            matches!(ctx.status, 0..=0x46) || ctx.situation == 0 && ctx.status >= 0x1dc;
        let can_queue = ctx.situation == 0 || matches!(ctx.status, 0x22..=0x23);

        if pressed && can_queue && supported_status && physical & DPAD_SIDES != DPAD_SIDES {
            self.phase = Phase::Queued;
            self.direction = direction;
            self.started = tick;
        }
        let mut out = LedgeOutput::default();
        if physical & DPAD_SIDES == DPAD_SIDES || !supported_status {
            self.phase = Phase::Idle;
        } else if !ctx.stopped {
            if self.phase == Phase::Queued && self.jump_ready(ctx, self.direction) {
                self.phase = Phase::Takeoff;
                self.started = tick;
                out = owned();
                out.requested = MAPPER_X;
            } else {
                match self.phase {
                    Phase::Idle => {}
                    Phase::Queued => {
                        // A released tap is retained for up to 60 advancing native
                        // observations. Waiting never owns movement or buttons.
                        if direction != 0 && direction == self.direction {
                            self.started = tick; // Held requests survive long native recovery.
                        } else if tick.saturating_sub(self.started) > 60 {
                            self.phase = Phase::Idle;
                        }
                    }
                    Phase::Takeoff => {
                        if tick.saturating_sub(self.started) > 24 {
                            self.phase = Phase::Idle;
                        } else if (ctx.status == 0xa
                            && ctx.situation == 0
                            && ctx.motion_frame >= 2.0)
                            || ctx.status == 0xb
                                && matches!(ctx.situation, 0 | 2)
                                && (0.0..=6.0).contains(&ctx.motion_frame)
                        {
                            // Native squat/jump proves the hop was accepted.
                            // Buffer Guard before another observation round trip;
                            // the engine still decides when air dodge becomes legal.
                            out = owned();
                            out.stick =
                                Some([f32::from(self.direction) * SLIDE_STICK_X, SLIDE_STICK_Y]);
                            out.requested = MAPPER_ZR;
                            self.phase = Phase::Dodge;
                            self.started = tick;
                        } else if ctx.situation == 2 {
                            self.phase = Phase::Idle; // Never begin a late/high dodge.
                        } else {
                            out = owned(); // Release X during jump squat.
                        }
                    }
                    Phase::Dodge => {
                        if matches!(ctx.status, 0x22..=0x23)
                            || ctx.situation == 0 && !matches!(ctx.status, 0xa | 0xb)
                        {
                            self.phase = Phase::Landing;
                            out = owned(); // Release ZR, retain slide momentum.
                        } else if tick.saturating_sub(self.started)
                            > 8 + ctx.input_delay.ceil().clamp(0.0, 12.0) as i64
                        {
                            self.phase = Phase::Idle;
                        } else {
                            out = owned();
                            // Keep shield and the angle until native acceptance;
                            // a delayed command must not lose its follow-up.
                            out.requested = MAPPER_ZR;
                            out.stick =
                                Some([f32::from(self.direction) * SLIDE_STICK_X, SLIDE_STICK_Y]);
                        }
                    }
                    Phase::Landing => {
                        out = owned();
                        if tick.saturating_sub(self.started) > 120 {
                            self.phase = Phase::Idle;
                            out = LedgeOutput::default();
                        }
                    }
                }
            }
        }
        if self.phase == Phase::Landing && !ctx.stopped && ctx.situation == 0 && ctx.can_jump {
            self.phase = Phase::Idle;
            out = LedgeOutput::default();
            if self.jump_ready(ctx, direction) {
                self.direction = direction;
                self.phase = Phase::Takeoff;
                self.started = tick;
                out = owned();
                out.requested = MAPPER_X;
            }
        }
        if ctx.stopped && supported_status && physical & DPAD_SIDES != DPAD_SIDES {
            out = self.frozen_output(ctx);
        }
        self.last_sample = Some(key);
        self.last_output = out;
        out
    }

    fn jump_ready(&self, ctx: SlideContext, direction: i8) -> bool {
        direction != 0
            && ctx.situation == 0
            && ctx.can_jump
            && ctx.safe_ground[usize::from(direction > 0)]
    }
}
fn owned() -> LedgeOutput {
    LedgeOutput {
        stick: Some([0.0, 0.0]),
        owned: LEDGE_BUTTONS,
        ..LedgeOutput::default()
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod online_tests;
