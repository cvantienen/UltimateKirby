//! Live tech requests with mapper acknowledgment. Input-only: native collision
//! rules and lockout still decide whether the requested tech is possible.
//!
//! `sample` asks for a shield edge near predicted contact. `mapped_press` is
//! called only after the native mapper reports that edge; only then does our
//! cooldown begin. A prediction or a requested button alone is not delivery.
//! See src/live/tech.rs for native detection and tech_prediction.rs for geometry.
use crate::tech_prediction::{CATCHED_AIR_END_GANON, CATCHED_GANON};
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
    /// Native adapter verified that guard cannot become an ordinary air dodge.
    /// Prepare release before imminent contact, without pressing shield early.
    pub prepare_guard: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TechOutput {
    /// Ask for a fresh shield press, rather than writing a tech status.
    pub press: bool,
    /// Allow replacing physical shield aliases, including a release sample.
    pub owns_guard: bool,
}
impl TechOutput {
    /// Canonical Shield and Grab can both produce mapped Guard. Jump, attack,
    /// special and movement remain manual; preparation never owns a whole tape.
    pub fn owned_buttons(self) -> u32 {
        if self.owns_guard {
            crate::native_input::SEQUENCE_GUARD_INPUTS
        } else {
            0
        }
    }
}
/// Final simulated controller Guard, after the original mapper's binding and
/// flick filtering. Only the selected native-eligible safety request calls this.
pub fn mapped_guard(buttons: u32, output: TechOutput) -> u32 {
    use crate::native_input::{MAPPED_CATCH, MAPPED_GUARD, MAPPED_GUARD_HOLD};
    let buttons = if output.owns_guard {
        buttons & !(MAPPED_GUARD | MAPPED_GUARD_HOLD | MAPPED_CATCH)
    } else {
        buttons
    };
    if output.press {
        buttons | MAPPED_GUARD
    } else {
        buttons
    }
}
#[derive(Default)]
pub struct TechInput {
    last: Option<TechContext>,
    output: TechOutput,
    waiting_for_mapper: bool,
    cooldown: f32,
    cooldown_setting: u16,
    reaction_remaining: Option<f32>,
    native_guard: Option<bool>,
    delivery: Option<Delivery>,
}
#[derive(Clone, Copy)]
struct Delivery {
    stamp: TechContext,
    reaction: Option<f32>,
    advances: u8,
}
impl TechInput {
    /// A fresh edge due by the first movement frame cannot wait behind launch
    /// DI. Release-only ownership still yields to DI, which releases Guard too.
    pub fn urgent_press(&self) -> bool {
        self.output.press
            && self.last.is_some_and(|c| {
                c.prepare_guard
                    && c.contact_in.is_some_and(|contact| {
                        contact.is_finite()
                            && contact >= 0.
                            && (self.delivery.is_some()
                                || contact - c.remaining_stop as f32 - c.input_delay.clamp(0., 12.)
                                    <= 1.25)
                    })
            })
    }
    /// Cancel a request/receipt while preserving an already spent native lockout.
    pub fn cancel_request(&mut self) {
        self.waiting_for_mapper = false;
        self.delivery = None;
        self.output = TechOutput::default();
    }
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
            if self.native_guard.is_some() {
                self.delivery = self.last.map(|stamp| Delivery {
                    stamp,
                    reaction: self.reaction_remaining,
                    advances: 0,
                });
            }
            self.output = TechOutput {
                press: false,
                owns_guard: true,
            };
        }
    }
    /// A human shield press during tumble also consumes native input timing.
    /// Track it so automation does not immediately press again into its lockout.
    pub fn mapped_human_guard(&mut self) {
        if self.last.is_some_and(|ctx| {
            (0x49..=0x4f).contains(&ctx.status)
                || matches!(ctx.status, 0x6c | 0x6d)
                || matches!(ctx.status, CATCHED_GANON | CATCHED_AIR_END_GANON)
        }) {
            self.cooldown = f32::from(self.cooldown_setting.max(2));
            self.waiting_for_mapper = false;
            self.output = TechOutput::default();
            self.delivery = None;
        }
    }
    pub fn sample(
        &mut self,
        enabled: bool,
        takeover: bool,
        cooldown: u16,
        ctx: TechContext,
    ) -> TechOutput {
        self.sample_with_reaction_clock(enabled, takeover, cooldown, ctx, None)
    }
    /// Native remaining hitstun advances even when the damage pose is frozen.
    /// Use it once, instead of animation progress, with no mapper-poll clock.
    pub fn sample_with_reaction_clock(
        &mut self,
        enabled: bool,
        takeover: bool,
        cooldown: u16,
        ctx: TechContext,
        reaction_remaining: Option<f32>,
    ) -> TechOutput {
        self.sample_with_native_window(enabled, takeover, cooldown, ctx, reaction_remaining, 12, 7)
    }
    /// Native windows count the input frame too. Keep a three-frame margin for
    /// release, mapper delivery and collision refinement, without guessing a
    /// Melee-style twenty-frame window. Lockout begins only on acknowledgment.
    pub fn sample_with_native_window(
        &mut self,
        enabled: bool,
        takeover: bool,
        cooldown: u16,
        ctx: TechContext,
        reaction_remaining: Option<f32>,
        window: u16,
        ganon_window: u16,
    ) -> TechOutput {
        self.sample_native(
            enabled,
            takeover,
            cooldown,
            ctx,
            reaction_remaining,
            window,
            ganon_window,
            None,
        )
    }
    /// The mapped packet is not yet proof that the fighter consumed Guard.
    /// Observe ControlModule's game-visible Guard/GuardHold before releasing a
    /// mapped press, and before treating a held native guard as freshly armed.
    pub fn sample_with_native_delivery(
        &mut self,
        enabled: bool,
        takeover: bool,
        cooldown: u16,
        mut ctx: TechContext,
        reaction_remaining: Option<f32>,
        window: u16,
        ganon_window: u16,
        native_guard: bool,
    ) -> TechOutput {
        ctx.previous_guard |= native_guard;
        self.sample_native(
            enabled,
            takeover,
            cooldown,
            ctx,
            reaction_remaining,
            window,
            ganon_window,
            Some(native_guard),
        )
    }
    fn sample_native(
        &mut self,
        enabled: bool,
        takeover: bool,
        cooldown: u16,
        ctx: TechContext,
        reaction_remaining: Option<f32>,
        window: u16,
        ganon_window: u16,
        native_guard: Option<bool>,
    ) -> TechOutput {
        if self.last.is_some_and(|last| last.object != ctx.object) {
            *self = Self::default();
        }
        let consumed = native_guard == Some(true)
            && (self.delivery.is_some() || self.native_guard == Some(false))
            && crate::tech_prediction::tech_press_allowed(ctx.status, false);
        self.native_guard = native_guard;
        if native_guard != Some(false) {
            self.delivery = None;
        }
        let reaction_remaining = reaction_remaining.filter(|v| {
            v.is_finite() && *v > 0. && *v <= 1200. && (0x49..=0x4f).contains(&ctx.status)
        });
        let previous_reaction = std::mem::replace(&mut self.reaction_remaining, reaction_remaining);
        let reaction_elapsed = self
            .last
            .filter(|last| {
                !last.stopped
                    && !ctx.stopped
                    && last.object == ctx.object
                    && last.status == ctx.status
            })
            .and_then(|_| previous_reaction.zip(reaction_remaining))
            .and_then(|(previous, current)| {
                let elapsed = previous - current;
                (0.0..=4.0).contains(&elapsed).then_some(elapsed)
            });
        // Even an unavailable animation clock can be followed by a real human
        // guard edge. Remember its configured lockout before early returns.
        self.cooldown_setting = cooldown;
        // Native success is authoritative even on a transition observation
        // whose animation clock is not available yet. Do not let the old
        // attempt's synthetic lockout survive into a later eligible impact.
        if matches!(ctx.status, 0x67..=0x6b) {
            self.cooldown = 0.0;
            self.waiting_for_mapper = false;
            self.delivery = None;
            self.last = Some(ctx);
            self.output = TechOutput::default();
            return self.output;
        }
        let animation_known = ctx.frame.is_finite()
            && ctx.frame >= 0.0
            && ctx.rate.is_finite()
            && ctx.rate > 0.0
            && ctx.motion != 0;
        let hitstop_clock_known = ctx.stopped && (1..=120).contains(&ctx.remaining_stop);
        // Collision/air-escape eligibility is verified by the native adapter.
        // A frozen pose must not veto that safe imminent input merely because
        // no animation-derived elapsed time can be measured. It still cannot
        // advance a previous press's cooldown without an advancing clock.
        let native_safe =
            ctx.prepare_guard && crate::tech_prediction::tech_press_allowed(ctx.status, false);
        if !ctx.input_delay.is_finite() || !animation_known && !hitstop_clock_known && !native_safe
        {
            // A missing/frozen native clock withholds input, but must not erase
            // a cooldown belonging to an already mapped shield press.
            self.last = Some(ctx);
            if consumed {
                self.cooldown = f32::from(cooldown.max(2));
            }
            self.waiting_for_mapper = false;
            self.delivery = None;
            self.output = TechOutput::default();
            return self.output;
        }
        if let Some(elapsed) = reaction_elapsed {
            self.cooldown = (self.cooldown - elapsed).max(0.);
        } else if let Some(last) = self.last {
            if animation_known && !ctx.stopped && !last.stopped {
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
                    .unwrap_or(0.0)
                    .clamp(0.0, 4.0)
                } else {
                    0.0
                };
                self.cooldown = (self.cooldown - advance).max(0.0);
            }
        }
        self.last = Some(ctx);
        if consumed {
            // Native consumption can lag the mapper by a frame. Start the
            // no-rapid interval at the observed engine edge, not one frame early.
            self.cooldown = f32::from(cooldown.max(2));
        }
        if !enabled {
            // Turning the feature off releases its proposal, not the native
            // lockout already consumed by a delivered shield edge. Continue
            // observing real progress so re-enabling cannot replay that edge.
            self.waiting_for_mapper = false;
            self.delivery = None;
            self.output = TechOutput::default();
            return self.output;
        }
        // The command-grab slam needs a tighter arrival margin than ordinary
        // tumble. Re-evaluate contact rather than replaying an old prediction.
        let arrival_lead = if ctx.status == CATCHED_GANON {
            f32::from(ganon_window.clamp(2, 60).saturating_sub(3).max(1))
        } else {
            f32::from(window.clamp(2, 60).saturating_sub(3).max(1))
        };
        let due = ctx.contact_in.is_some_and(|contact| {
            contact.is_finite()
                && contact >= 0.0
                && contact - ctx.input_delay.clamp(0.0, 12.0) <= arrival_lead
        });
        if !due || !crate::tech_prediction::tech_press_allowed(ctx.status, false) {
            self.waiting_for_mapper = false;
            self.delivery = None;
            self.output = TechOutput {
                press: false,
                owns_guard: takeover
                    && ctx.prepare_guard
                    // Only a fresh, near contact can reserve inputs. Broad
                    // hitstop preparation would starve SDI for an entire hit.
                    && ctx.contact_in.is_some_and(|contact|
                        contact.is_finite() && contact>=0.
                            && contact-ctx.input_delay.clamp(0.,12.)<=arrival_lead+2.)
                    && crate::tech_prediction::tech_press_allowed(ctx.status, false),
            };
            return self.output;
        }
        if let Some(mut delivery) = self.delivery {
            let previous = delivery.stamp;
            let changed = previous.status != ctx.status
                || previous.motion != ctx.motion
                || previous.frame.to_bits() != ctx.frame.to_bits()
                || previous.stopped != ctx.stopped
                || previous.remaining_stop != ctx.remaining_stop
                || delivery.reaction != reaction_remaining;
            if changed {
                delivery.advances = delivery.advances.saturating_add(1);
            }
            delivery.stamp = ctx;
            delivery.reaction = reaction_remaining;
            if ctx.prepare_guard && delivery.advances < 3 {
                self.delivery = Some(delivery);
                self.output = TechOutput {
                    press: true,
                    owns_guard: takeover,
                };
                return self.output;
            }
            // Bounded game-input receipt: abandon unsupported/unconsumed
            // requests, retaining the already mapped press's lockout.
            self.delivery = None;
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
        if ctx.previous_guard {
            // An unacknowledged proposal is not delivery. If guard became
            // held meanwhile, release it first instead of replaying a press
            // forever without producing the fresh mapped edge a tech needs.
            self.waiting_for_mapper = false;
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
