//! Live tech requests with mapper acknowledgment. Input-only: native collision
//! rules and lockout still decide whether the requested tech is possible.
//!
//! `sample` asks for a shield edge near predicted contact. `mapped_press` is
//! called only after the native mapper reports that edge; only then does our
//! cooldown begin. A prediction or a requested button alone is not delivery.
//! See src/live/tech.rs for native detection and tech_prediction.rs for geometry.
pub use crate::flame_choke::DEFAULT_GANON_PASSIVE_TRIGGER_FRAME;
use crate::tech_prediction::{is_tech_status, is_tumble_status};
pub const DEFAULT_PASSIVE_TRIGGER_FRAME: u16 = 12;
pub const MAX_INPUT_DELAY_FRAMES: f32 = 12.0;
pub const MAX_REACTION_REMAINING_FRAMES: f32 = 1200.0;
pub const MIN_TECH_COOLDOWN_FRAMES: u16 = 2;
const MAX_HITSTOP_OBSERVATION_FRAMES: u64 = 120;
const MAX_REACTION_CLOCK_ADVANCE_FRAMES: f32 = 4.0;
const STABLE_ANIMATION_RATE_EPSILON: f32 = 0.001;
const MAX_ANIMATION_CLOCK_ADVANCE_FRAMES: f32 = 4.0;
const MIN_NATIVE_TECH_WINDOW_FRAMES: u16 = 2;
const MAX_NATIVE_TECH_WINDOW_FRAMES: u16 = 60;
const TECH_WINDOW_SAFETY_MARGIN_FRAMES: u16 = 3;
const MIN_TECH_ARRIVAL_LEAD_FRAMES: u16 = 1;
const NEAR_CONTACT_EXTRA_MARGIN_FRAMES: f32 = 2.0;
const URGENT_CONTACT_LEAD_FRAMES: f32 = 1.25;
const MAX_GUARD_PREPARATION_ADVANCES: u8 = 3;
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
impl TechContext {
    /// A forecast contact only counts when it is finite and not already past.
    fn forecast_contact(&self) -> Option<f32> {
        self.contact_in.filter(|c| c.is_finite() && *c >= 0.)
    }
    fn input_lead(&self) -> f32 {
        self.input_delay.clamp(0., MAX_INPUT_DELAY_FRAMES)
    }
    /// Forecast contact time minus the clamped input delay.
    fn contact_lead(&self) -> Option<f32> {
        self.forecast_contact()
            .map(|contact| contact - self.input_lead())
    }
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
impl Delivery {
    /// Count a native clock change since the last receipt observation.
    fn observe(&mut self, ctx: TechContext, reaction: Option<f32>) {
        let previous = self.stamp;
        let changed = previous.status != ctx.status
            || previous.motion != ctx.motion
            || previous.frame.to_bits() != ctx.frame.to_bits()
            || previous.stopped != ctx.stopped
            || previous.remaining_stop != ctx.remaining_stop
            || self.reaction != reaction;
        self.advances = self.advances.saturating_add(u8::from(changed));
        self.stamp = ctx;
        self.reaction = reaction;
    }
}
impl TechInput {
    /// A fresh edge due by the first movement frame cannot wait behind launch
    /// DI. Release-only ownership still yields to DI, which releases Guard too.
    pub fn urgent_press(&self) -> bool {
        self.output.press
            && self.last.is_some_and(|c| {
                c.prepare_guard
                    && c.forecast_contact().is_some_and(|contact| {
                        self.delivery.is_some()
                            || contact - c.remaining_stop as f32 - c.input_lead()
                                <= URGENT_CONTACT_LEAD_FRAMES
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
    /// Begin the no-rapid-tech interval after a mapped or native shield edge.
    fn start_lockout(&mut self) {
        self.cooldown = f32::from(self.cooldown_setting.max(MIN_TECH_COOLDOWN_FRAMES));
    }
    /// Withdraw any outstanding proposal and publish `output` instead.
    fn hold(&mut self, output: TechOutput) -> TechOutput {
        self.waiting_for_mapper = false;
        self.delivery = None;
        self.output = output;
        output
    }
    /// Called only after a fresh mapped GUARD edge, never for a policy proposal.
    pub fn mapped_press(&mut self) {
        if self.waiting_for_mapper && self.output.press {
            self.start_lockout();
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
        if self.last.is_some_and(|ctx| is_tech_status(ctx.status)) {
            self.start_lockout();
            self.hold(TechOutput::default());
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
        self.sample_with_native_window(
            enabled,
            takeover,
            cooldown,
            ctx,
            reaction_remaining,
            DEFAULT_PASSIVE_TRIGGER_FRAME,
            DEFAULT_GANON_PASSIVE_TRIGGER_FRAME,
        )
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
        let eligible = is_tech_status(ctx.status);
        let consumed = native_guard == Some(true)
            && (self.delivery.is_some() || self.native_guard == Some(false))
            && eligible;
        self.native_guard = native_guard;
        if native_guard != Some(false) {
            self.delivery = None;
        }
        let reaction_elapsed = self.advance_reaction_clock(&ctx, reaction_remaining);
        // Even an unavailable animation clock can be followed by a real human
        // guard edge. Remember its configured lockout before early returns.
        self.cooldown_setting = cooldown;
        // Native success is authoritative even on a transition observation
        // whose animation clock is not available yet. Do not let the old
        // attempt's synthetic lockout survive into a later eligible impact.
        if matches!(
            ctx.status,
            crate::native_states::status::PASSIVE..=crate::native_states::status::PASSIVE_CEIL
        ) {
            self.cooldown = 0.0;
            self.last = Some(ctx);
            return self.hold(TechOutput::default());
        }
        let animation_known = crate::timing::clock_known(ctx.motion, ctx.frame, ctx.rate);
        let hitstop_clock_known =
            ctx.stopped && (1..=MAX_HITSTOP_OBSERVATION_FRAMES).contains(&ctx.remaining_stop);
        // Collision/air-escape eligibility is verified by the native adapter.
        // A frozen pose must not veto that safe imminent input merely because
        // no animation-derived elapsed time can be measured. It still cannot
        // advance a previous press's cooldown without an advancing clock.
        let native_safe = ctx.prepare_guard && eligible;
        let has_usable_clock = animation_known || hitstop_clock_known || native_safe;
        if !ctx.input_delay.is_finite() || !has_usable_clock {
            // A missing/frozen native clock withholds input, but must not erase
            // a cooldown belonging to an already mapped shield press.
            self.last = Some(ctx);
            if consumed {
                self.start_lockout();
            }
            return self.hold(TechOutput::default());
        }
        self.advance_cooldown(&ctx, reaction_elapsed, animation_known);
        self.last = Some(ctx);
        if consumed {
            // Native consumption can lag the mapper by a frame. Start the
            // no-rapid interval at the observed engine edge, not one frame early.
            self.start_lockout();
        }
        if !enabled {
            // Turning the feature off releases its proposal, not the native
            // lockout already consumed by a delivered shield edge. Continue
            // observing real progress so re-enabling cannot replay that edge.
            return self.hold(TechOutput::default());
        }
        // The command-grab slam needs a tighter arrival margin than ordinary
        // tumble. Re-evaluate contact rather than replaying an old prediction.
        let arrival_lead = native_arrival_lead(ctx.status, window, ganon_window);
        let due = ctx
            .contact_lead()
            .is_some_and(|lead| lead <= arrival_lead);
        if !due || !eligible {
            return self.hold(TechOutput {
                press: false,
                owns_guard: takeover
                    && ctx.prepare_guard
                    // Only a fresh, near contact can reserve inputs. Broad
                    // hitstop preparation would starve SDI for an entire hit.
                    && ctx.contact_lead().is_some_and(|lead| {
                        lead <= arrival_lead + NEAR_CONTACT_EXTRA_MARGIN_FRAMES
                    })
                    && eligible,
            });
        }
        if let Some(output) = self.continue_delivery(ctx, takeover) {
            return output;
        }
        // While the same collision remains imminent, keep physical shield
        // aliases released. Handing them back between samples could create
        // another mapped GUARD edge and restart the input lockout. Likewise an
        // unacknowledged proposal is not delivery: if guard became held
        // meanwhile, release it first instead of replaying a press forever
        // without producing the fresh mapped edge a tech needs.
        if self.cooldown > 0.0 || ctx.previous_guard {
            return self.hold(TechOutput {
                press: false,
                owns_guard: takeover,
            });
        }
        self.waiting_for_mapper = true;
        self.output = TechOutput {
            press: true,
            owns_guard: takeover,
        };
        self.output
    }

    /// Track native remaining hitstun, which advances even while the damage pose
    /// is frozen. Returns its plausible advance since the previous observation.
    fn advance_reaction_clock(&mut self, ctx: &TechContext, remaining: Option<f32>) -> Option<f32> {
        let remaining = remaining.filter(|v| {
            v.is_finite() && *v > 0. && *v <= MAX_REACTION_REMAINING_FRAMES && is_tumble_status(ctx.status)
        });
        let previous = std::mem::replace(&mut self.reaction_remaining, remaining);
        self.last
            .filter(|last| !last.stopped && !ctx.stopped && last.status == ctx.status)?;
        let elapsed = previous? - remaining?;
        (0.0..=MAX_REACTION_CLOCK_ADVANCE_FRAMES)
            .contains(&elapsed)
            .then_some(elapsed)
    }

    /// Spend the lockout by native hitstun progress, else by stable animation progress.
    fn advance_cooldown(
        &mut self,
        ctx: &TechContext,
        reaction_elapsed: Option<f32>,
        animation_known: bool,
    ) {
        let advance = match (reaction_elapsed, self.last) {
            (Some(elapsed), _) => elapsed,
            (None, Some(last)) if animation_known && !ctx.stopped && !last.stopped => {
                animation_advance(&last, ctx)
            }
            _ => return,
        };
        self.cooldown = (self.cooldown - advance).max(0.0);
    }

    /// Bounded game-input receipt of a mapped press: keep requesting while the
    /// native clock advances, abandon unsupported/unconsumed requests and retain
    /// the already mapped press's lockout.
    fn continue_delivery(&mut self, ctx: TechContext, takeover: bool) -> Option<TechOutput> {
        let mut delivery = self.delivery.take()?;
        delivery.observe(ctx, self.reaction_remaining);
        if !ctx.prepare_guard || delivery.advances >= MAX_GUARD_PREPARATION_ADVANCES {
            return None;
        }
        self.delivery = Some(delivery);
        self.output = TechOutput {
            press: true,
            owns_guard: takeover,
        };
        Some(self.output)
    }
}

/// Elapsed animation frames between two samples of one stable-rate motion.
fn animation_advance(last: &TechContext, ctx: &TechContext) -> f32 {
    let stable = ctx.status == last.status
        && ctx.motion == last.motion
        && ctx.frame >= last.frame
        && (ctx.rate - last.rate).abs() < STABLE_ANIMATION_RATE_EPSILON;
    if !stable {
        return 0.0;
    }
    crate::timing::motion_progress(
        // Preserve this cooldown's existing stable-rate policy.
        Some(crate::timing::MotionStamp::new(
            last.status,
            last.motion,
            last.frame,
            ctx.rate,
        )),
        crate::timing::MotionStamp::new(ctx.status, ctx.motion, ctx.frame, ctx.rate),
    )
    .elapsed(0.0, 1.0)
    .unwrap_or(0.0)
    .clamp(0.0, MAX_ANIMATION_CLOCK_ADVANCE_FRAMES)
}

/// The command-grab slam needs a tighter arrival margin than ordinary tumble.
fn native_arrival_lead(status: i32, window: u16, ganon_window: u16) -> f32 {
    f32::from(
        crate::flame_choke::passive_window(status, window, ganon_window)
            .clamp(MIN_NATIVE_TECH_WINDOW_FRAMES, MAX_NATIVE_TECH_WINDOW_FRAMES)
            .saturating_sub(TECH_WINDOW_SAFETY_MARGIN_FRAMES)
            .max(MIN_TECH_ARRIVAL_LEAD_FRAMES),
    )
}
#[cfg(test)]
#[path = "../../../tests/unit/tech_input.rs"]
mod tests;

#[cfg(test)]
#[path = "tech_input/ganon_and_release_regressions.rs"]
mod ganon_and_release_regressions;
