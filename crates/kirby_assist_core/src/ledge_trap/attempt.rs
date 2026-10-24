//! Shared request bookkeeping for response variants. No allocations, pad-poll
//! frame counters, native pointers or assumptions that a requested action ran.
use super::{LedgeContext, MAX_ATTEMPTS};

#[derive(Default)]
pub(super) struct NativeAttempt {
    previous: Option<(i32, u64, f32, f32)>,
    age: f32,
    last_issue: Option<f32>,
    requests: u8,
    pub accepted: bool,
    pub expired: bool,
}
impl NativeAttempt {
    pub fn elapsed(&self) -> f32 {
        self.age
    }
    /// Keep a special request paired with its direction through a short native
    /// delivery window. Hitstop/duplicate polls do not spend the held duration.
    pub fn holding_special(&self, ctx: LedgeContext) -> bool {
        !self.expired
            && !self.accepted
            && self
                .last_issue
                .is_some_and(|last| self.age - last < ctx.input_delay.clamp(0.0, 12.0).ceil() + 2.0)
    }
    /// Return true only on real, non-stopped native progress (or initialization).
    /// Large/malformed gaps end the attempt rather than executing a late step.
    pub fn advance(&mut self, ctx: LedgeContext, started: bool, timeout: u16) -> bool {
        self.accepted |= self.requests != 0 && started;
        if ctx.stopped {
            return false;
        }
        if ctx.motion_kind == 0
            || !ctx.motion_frame.is_finite()
            || ctx.motion_frame < 0.0
            || !ctx.motion_rate.is_finite()
            || ctx.motion_rate <= 0.0
        {
            self.expired = true;
            return true;
        }
        let clock = (
            ctx.status,
            ctx.motion_kind,
            ctx.motion_frame,
            ctx.motion_rate,
        );
        let delta = crate::timing::motion_elapsed(
            self.previous,
            (
                ctx.status,
                ctx.motion_kind,
                ctx.motion_frame,
                ctx.motion_rate,
            ),
            0.0,
        );
        if !delta.is_finite() || !(0.0..=8.0).contains(&delta) {
            self.expired = true;
            return true;
        }
        let fresh = self.previous.is_none() || delta > 0.0;
        self.previous = Some(clock);
        self.age += delta;
        self.expired |= self.age > f32::from(timeout);
        fresh
    }
    pub fn issue(&mut self, ctx: LedgeContext, previous_buttons: u32, button: u32) -> bool {
        self.issue_after(ctx, previous_buttons, button, 8.0)
    }
    /// Single-move delivery holds for delay+2 frames, then allows two release
    /// frames before retrying. Duplicate mapper polls cannot spend this gap.
    pub fn issue_move(&mut self, ctx: LedgeContext, previous_buttons: u32, button: u32) -> bool {
        self.issue_after(
            ctx,
            previous_buttons,
            button,
            ctx.input_delay.clamp(0.0, 12.0).ceil() + 4.0,
        )
    }
    fn issue_after(
        &mut self,
        ctx: LedgeContext,
        previous_buttons: u32,
        button: u32,
        spacing: f32,
    ) -> bool {
        if !ctx.input_delay.is_finite() || !(0.0..=120.0).contains(&ctx.input_delay) {
            self.expired = true;
            return false;
        }
        if self.accepted || self.expired || previous_buttons & button != 0 {
            return false;
        }
        if self.last_issue.is_some_and(|last| {
            self.age - last < (ctx.input_delay.clamp(0.0, 12.0) + 2.0).max(spacing)
        }) {
            return false;
        }
        if self.requests >= MAX_ATTEMPTS {
            self.expired = true;
            return false;
        }
        self.requests += 1;
        self.last_issue = Some(self.age);
        true
    }
}

/// Delivery begins only when a grab is requested, never during its approach.
/// Walking/running animation loops are normal; they must not permanently disable
/// grabs. This clock owns one short delivery window, not a whole ledge visit.
pub(super) struct GrabDelivery {
    previous: (i32, u64, f32, f32),
    age: f32,
    duration: f32,
    buttons: u32,
}
impl GrabDelivery {
    pub fn new(ctx: LedgeContext, buttons: u32) -> Self {
        Self {
            previous: (
                ctx.status,
                ctx.motion_kind,
                ctx.motion_frame,
                ctx.motion_rate,
            ),
            age: 0.0,
            duration: if ctx.input_delay.is_finite() {
                ctx.input_delay.clamp(0.0, 12.0).ceil() + 2.0
            } else {
                2.0
            },
            buttons,
        }
    }
    pub fn sample(&mut self, ctx: LedgeContext) -> Option<u32> {
        if ctx.motion_kind == 0
            || !ctx.motion_frame.is_finite()
            || ctx.motion_frame < 0.0
            || !ctx.motion_rate.is_finite()
            || ctx.motion_rate <= 0.0
        {
            return None;
        }
        let delta = crate::timing::motion_elapsed(
            Some(self.previous),
            (
                ctx.status,
                ctx.motion_kind,
                ctx.motion_frame,
                ctx.motion_rate,
            ),
            0.0,
        );
        if !delta.is_finite() || !(0.0..=8.0).contains(&delta) {
            return None;
        }
        self.previous = (
            ctx.status,
            ctx.motion_kind,
            ctx.motion_frame,
            ctx.motion_rate,
        );
        self.age += delta;
        (self.age < self.duration).then_some(self.buttons)
    }
}
