//! Live tech requests with mapper acknowledgment. Input-only: native collision
//! rules and lockout still decide whether the requested tech is possible.
//!
//! `sample` asks for a shield edge near predicted contact. `mapped_press` is
//! called only after the native mapper reports that edge; only then does our
//! cooldown begin. A prediction or a requested button alone is not delivery.
//! See src/live/tech.rs for native detection and tech_prediction.rs for geometry.
use crate::tech_prediction::CATCHED_GANON;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TechContext {
    pub object: u32,
    pub status: i32,
    pub motion: u64,
    pub frame: f32,
    pub rate: f32,
    pub stopped: bool,
    pub remaining_stop: u64,
    pub contact_in: Option<f32>,
    pub input_delay: f32,
    pub previous_guard: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TechOutput {
    /// Ask for a fresh shield press, rather than writing a tech status.
    pub press: bool,
    /// Allow replacing physical shield aliases, including a release sample.
    pub owns_guard: bool,
}
#[derive(Default)]
pub struct TechInput {
    last: Option<TechContext>,
    output: TechOutput,
    waiting_for_mapper: bool,
    cooldown: f32,
    cooldown_setting: u16,
}
impl TechInput {
    pub fn last_observation(&self) -> Option<TechContext> {
        self.last
    }
    pub fn cooldown_remaining(&self) -> f32 {
        self.cooldown
    }
    /// Called only after a fresh mapped GUARD edge, never for a policy proposal.
    pub fn mapped_press(&mut self) {
        if self.waiting_for_mapper && self.output.press {
            self.cooldown = f32::from(self.cooldown_setting.max(2));
            self.waiting_for_mapper = false;
            self.output = TechOutput {
                press: false,
                owns_guard: true,
            };
        }
    }
    /// A human shield press during tumble also consumes native input timing.
    /// Track it so automation does not immediately press again into its lockout.
    pub fn mapped_human_guard(&mut self) {
        if self
            .last
            .is_some_and(|ctx| (0x49..=0x4f).contains(&ctx.status) || ctx.status == CATCHED_GANON)
        {
            self.cooldown = f32::from(self.cooldown_setting.max(2));
            self.waiting_for_mapper = false;
            self.output = TechOutput::default();
        }
    }
    pub fn sample(
        &mut self,
        enabled: bool,
        takeover: bool,
        cooldown: u16,
        ctx: TechContext,
    ) -> TechOutput {
        if !enabled {
            *self = Self::default();
            return TechOutput::default();
        }
        if self.last.is_some_and(|last| last.object != ctx.object) {
            *self = Self::default();
        }
        let animation_known = ctx.frame.is_finite()
            && ctx.frame >= 0.0
            && ctx.rate.is_finite()
            && ctx.rate > 0.0
            && ctx.motion != 0;
        let hitstop_clock_known = ctx.stopped && (1..=120).contains(&ctx.remaining_stop);
        if !ctx.input_delay.is_finite() || !animation_known && !hitstop_clock_known {
            // A missing/frozen native clock withholds input, but must not erase
            // a cooldown belonging to an already mapped shield press.
            self.last = Some(ctx);
            self.waiting_for_mapper = false;
            self.output = TechOutput::default();
            return self.output;
        }
        if let Some(last) = self.last {
            if !ctx.stopped && !last.stopped {
                let advance = if ctx.status == last.status
                    && ctx.motion == last.motion
                    && ctx.frame >= last.frame
                    && (ctx.rate - last.rate).abs() < 0.001
                {
                    crate::timing::motion_progress(
                        // Preserve this cooldown's existing stable-rate policy.
                        Some(crate::timing::MotionStamp::new(
                            last.status,
                            last.motion,
                            last.frame,
                            ctx.rate,
                        )),
                        crate::timing::MotionStamp::new(
                            ctx.status, ctx.motion, ctx.frame, ctx.rate,
                        ),
                    )
                    .elapsed(0.0, 1.0)
                    .unwrap_or(1.0)
                    .clamp(0.0, 4.0)
                } else {
                    1.0
                };
                self.cooldown = (self.cooldown - advance).max(0.0);
            }
        }
        let accepted = matches!(ctx.status, 0x67..=0x6b);
        if accepted {
            // An observed successful tech ends our attempt; do not carry a
            // synthetic cooldown through that tech into the next collision.
            self.cooldown = 0.0;
            self.waiting_for_mapper = false;
        }
        self.last = Some(ctx);
        self.cooldown_setting = cooldown;
        // The command-grab slam needs a tighter arrival margin than ordinary
        // tumble. Re-evaluate contact rather than replaying an old prediction.
        let arrival_lead = if ctx.status == CATCHED_GANON {
            4.0
        } else {
            9.0
        };
        let due = ctx.contact_in.is_some_and(|contact| {
            contact.is_finite()
                && contact >= 0.0
                && contact - ctx.input_delay.clamp(0.0, 12.0) <= arrival_lead
        });
        if !due || accepted {
            self.waiting_for_mapper = false;
            self.output = TechOutput::default();
            return self.output;
        }
        if self.cooldown > 0.0 {
            self.waiting_for_mapper = false;
            // While the same collision remains imminent, keep physical shield
            // aliases released. Handing them back between samples could create
            // another mapped GUARD edge and restart the input lockout.
            self.output = TechOutput {
                press: false,
                owns_guard: takeover,
            };
            return self.output;
        }
        if self.waiting_for_mapper {
            return self.output;
        }
        if ctx.previous_guard {
            self.output = TechOutput {
                press: false,
                owns_guard: takeover,
            };
        } else {
            self.waiting_for_mapper = true;
            self.output = TechOutput {
                press: true,
                owns_guard: takeover,
            };
        }
        self.output
    }
}
#[cfg(test)]
mod tests;

#[cfg(test)]
mod ganon_and_release_regressions;
