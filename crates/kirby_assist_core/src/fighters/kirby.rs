//! Kirby-specific input-only follow-ups.
use crate::gameplay::ControllerOutput;
use crate::native_input::MAPPER_A;

mod fair_followup {
    //! Kirby fast-fall fair hit 1 -> landing -> upward-angled forward smash.
    //! A fresh hurtbox hit arms the follow-up. Shield/stale hit flags do not.
    use super::{hold, press_a, valid_facing};
    use crate::gameplay::ControllerOutput;
    use crate::native_input::MAPPER_A;
    use crate::native_states::{jump_or_fall, status};
    use std::ops::Range;
    /// Native motion hash for Kirby's forward aerial.
    const FAIR: u64 = crate::timing::MOTION_ATTACK_AIR_F;
    /// Upward stick angle used for the forward-smash follow-up.
    const UP_ANGLE: f32 = 0.5;
    /// Native-frame budget for starting the first-hit fair fast-fall.
    const FAST_FALL_FRAMES: f32 = 16.0;
    /// Maximum native age of an armed first-hit fair follow-up.
    const MAX_LATCH_FRAMES: f32 = 180.0;
    /// Fair frames before the second hit begins. The live observer can sample
    /// a confirmed first hit slightly after its exact active frame, so keep the
    /// first-hit boundary tolerant while still rejecting the later hit.
    const FIRST_HIT_FRAMES: Range<f32> = 0.0..17.0;

    /// Copied Kirby state used by the first-hit forward-air follow-up.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct FairContext {
        /// Kirby's native battle-object ID.
        pub object: u32,
        /// Whether the observed object is Kirby.
        pub kirby: bool,
        /// Kirby's current native status.
        pub status: i32,
        /// Kirby's current native motion hash.
        pub motion: u64,
        /// Kirby's current motion frame.
        pub frame: f32,
        /// Kirby's current motion playback rate.
        pub rate: f32,
        /// Whether Kirby is touching the ground.
        pub grounded: bool,
        /// Whether Kirby's native animation is stopped.
        pub stopped: bool,
        /// Kirby's left/right facing sign.
        pub facing: f32,
        /// AttackModule::is_infliction(HIT), not the persistent status flag.
        pub hit_now: bool,
    }
    /// State machine for Kirby's first-hit forward-air fast-fall into forward smash.
    #[derive(Default)]
    pub struct FairFollowup {
        /// Previous Kirby observation used to detect native progress.
        last: Option<FairContext>,
        /// Controller request retained across duplicate mapper polls.
        output: ControllerOutput,
        /// Whether first-hit fair has armed the landing follow-up.
        queued: bool,
        /// Whether the current fair has already consumed its follow-up.
        spent: bool,
        /// Native animation frames elapsed since the first-hit confirmation.
        age: f32,
        /// Facing captured when first-hit fair armed the follow-up.
        facing: f32,
        /// Whether fast-fall down input has begun.
        down_held: bool,
    }
    impl FairFollowup {
        /// Whether first-hit fair has armed Kirby's landing follow-up.
        pub fn active(&self) -> bool {
            self.queued
        }
        /// Cancels the armed fair follow-up and releases its controller output.
        pub fn cancel_pending(&mut self) {
            self.finish();
            self.output = ControllerOutput::default();
        }
        /// Advances the fair follow-up using Kirby's current native observation.
        pub fn sample(&mut self, enabled: bool, ctx: FairContext) -> ControllerOutput {
            self.sample_with_velocity(enabled, ctx, 0.0)
        }
        /// Advances the fair follow-up with Kirby's vertical speed for fast-fall confirmation.
        pub fn sample_with_velocity(
            &mut self,
            enabled: bool,
            ctx: FairContext,
            velocity_y: f32,
        ) -> ControllerOutput {
            if !enabled || !ctx.usable() {
                *self = Self::default();
                return self.output;
            }
            if self.last.is_some_and(|last| last.object != ctx.object) {
                *self = Self::default();
            }
            if self.last == Some(ctx) {
                return self.output;
            }
            let native_progress = !ctx.stopped
                && self.last.is_none_or(|last| {
                    ctx.status != last.status
                        || ctx.motion != last.motion
                        || ctx.frame != last.frame
                });
            let was_queued = self.queued;
            // Native aerial status can overlap a grounded situation flag on the
            // landing update. Do not discard its hit or issue another aerial A.
            let landed = ctx.grounded && !ctx.is_fair() && !jump_or_fall(ctx.status);
            let recognized = self.recognize(ctx, landed);
            if !recognized && self.queued && !ctx.stopped {
                self.advance_age(ctx);
            }
            let mut out = if self.queued {
                self.queued_output(ctx, landed, recognized, native_progress, velocity_y)
            } else {
                ControllerOutput::default()
            };
            // One explicit release when acceptance/interruption ends ownership.
            // Hand back physical movement immediately when an aerial times out.
            if out.owned == 0 && was_queued && landed {
                out = hold([0.0, 0.0]);
            }
            self.last = Some(ctx);
            self.output = out;
            out
        }
        /// Rearm on a new fair, drop the latch on a later hit, and latch the first hit.
        fn recognize(&mut self, ctx: FairContext, landed: bool) -> bool {
            let fair = ctx.is_fair();
            let new_fair = fair
                && self
                    .last
                    .is_none_or(|last| !last.is_fair() || ctx.frame < last.frame);
            if new_fair && !self.queued {
                self.spent = false;
                self.down_held = false;
            }
            // A later connecting hit invalidates the first-hit-only landing route.
            // Check before duplicate/stop handling: contact can arrive in hitstop.
            if self.queued && fair && ctx.frame >= FIRST_HIT_FRAMES.end && ctx.hit_now {
                self.finish();
            }
            let first_hit = fair && FIRST_HIT_FRAMES.contains(&ctx.frame);
            let landing_hit = landed
                && ctx.status == status::LANDING_ATTACK_AIR
                && ctx.motion == crate::timing::motion_hash("landing_air_f")
                && self
                    .last
                    .is_some_and(|last| last.is_fair() && FIRST_HIT_FRAMES.contains(&last.frame));
            let recognized =
                !self.queued && !self.spent && (first_hit || landing_hit) && ctx.hit_now;
            if recognized {
                self.queued = true;
                self.age = 0.0;
                self.facing = ctx.facing.signum();
            }
            recognized
        }
        /// Ages the armed fair follow-up using native animation progress.
        fn advance_age(&mut self, ctx: FairContext) {
            let mut delta = crate::timing::motion_elapsed(
                self.last
                    .map(|last| (last.status, last.motion, last.frame, last.rate)),
                (ctx.status, ctx.motion, ctx.frame, ctx.rate),
                0.0,
            );
            // A falling animation may loop before landing. This is a confirmed
            // hit latch, not an exact-frame aerial sequence; skipped advancing
            // snapshots must not erase the pending grounded smash either.
            if delta.is_nan()
                && matches!(ctx.status, status::FALL | status::FALL_AERIAL)
                && self.last.is_some_and(|last| {
                    ctx.status == last.status && ctx.motion == last.motion && ctx.frame < last.frame
                })
            {
                delta = 1.0;
            }
            if !delta.is_finite() || delta < 0.0 {
                self.finish();
            } else {
                self.age += delta;
            }
        }
        /// Returns the output currently queued for delivery.
        fn queued_output(
            &mut self,
            ctx: FairContext,
            landed: bool,
            recognized: bool,
            native_progress: bool,
            velocity_y: f32,
        ) -> ControllerOutput {
            // Later animation frames without contact retain hit 1. A later
            // connecting hit, damage, grab, dodge or special cancels the latch.
            let supported = if landed {
                matches!(ctx.status, status::WAIT..=status::TURN_RUN_BRAKE | status::SQUAT..=status::LANDING_ATTACK_AIR | status::ATTACK_S4_START..=status::ATTACK_S4)
            } else {
                ctx.is_fair() || jump_or_fall(ctx.status)
            };
            if !supported
                || self.age > MAX_LATCH_FRAMES
                || !landed && self.age >= FAST_FALL_FRAMES
                || matches!(ctx.status, status::ATTACK_S4_HOLD | status::ATTACK_S4)
            {
                self.finish();
                return ControllerOutput::default();
            }
            if ctx.status == status::ATTACK_S4_START {
                // Release A to avoid charge, but preserve the angle through
                // startup until native forward-smash attack selection.
                return hold([self.facing, UP_ANGLE]);
            }
            if landed && !recognized {
                if !native_progress {
                    return self.output;
                }
                if self.output.requested & MAPPER_A == 0 {
                    // Ordinary smash-buffer input during landing lag; no range,
                    // DI, percent or opponent-state gates after confirmation.
                    return press_a([self.facing, UP_ANGLE]);
                }
            }
            if !landed {
                self.down_held |= !ctx.stopped && velocity_y.is_finite() && velocity_y < -0.05;
                if self.down_held {
                    return hold([0.0, -1.0]);
                }
            }
            hold([0.0, 0.0])
        }
        /// Completes the active operation and records its final state.
        fn finish(&mut self) {
            self.queued = false;
            self.down_held = false;
            self.spent = true;
        }
    }
    impl FairContext {
        /// Whether this is a finite, forward-facing Kirby observation.
        fn usable(&self) -> bool {
            self.kirby
                && self.frame.is_finite()
                && self.frame >= 0.0
                && self.rate.is_finite()
                && self.rate > 0.0
                && valid_facing(self.facing)
        }
        /// Whether Kirby is currently performing forward aerial.
        fn is_fair(&self) -> bool {
            self.status == status::ATTACK_AIR && self.motion == FAIR
        }
    }

    #[cfg(test)]
    include!("../../tests/unit/fighter_kirby_fair_followup.rs");
}
mod trip_followup {
    //! Kirby tilt-trip recognition and buffered forward-smash inputs.
    use super::{hold, press_a, valid_facing};
    use crate::gameplay::ControllerOutput;
    use crate::native_input::MAPPER_A;
    use crate::native_states::status;

    /// Nearby opponent identity and status used by Kirby's trip follow-up.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct TripTarget {
        /// Opponent battle-object ID.
        pub id: u32,
        /// Opponent's current native status.
        pub status: i32,
        /// Opponent's immediately preceding native status.
        pub previous_status: i32,
        /// Opponent position relative to Kirby.
        pub relative_position: [f32; 2],
        /// Whether the opponent is grounded.
        pub grounded: bool,
        /// Current Kirby hit corroborated by the victim's recorded attacker entry.
        /// This is not a collision-level hit-object ID.
        pub hit_confirmed: bool,
    }
    impl TripTarget {
        fn nearby(self, facing: f32) -> bool {
            let [dx, dy] = self.relative_position;
            self.grounded && dx.is_finite() && dy.is_finite()
                && (0.0..=18.0).contains(&(dx * facing.signum())) && dy.abs() <= 8.0
        }
    }
    /// Whether Kirby is performing a tilt capable of feeding the trip follow-up.
    fn trip_tilt(status: i32) -> bool {
        matches!(status, status::ATTACK_S3 | status::ATTACK_LW3)
    }
    /// Whether an opponent is in one of Smash's trip states.
    fn tripping(status: i32) -> bool {
        matches!(status, status::SLIP..=status::SLIP_WAIT)
    }

    /// Copied Kirby and opponent state used to recognize a down-tilt trip.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct TripContext {
        /// Kirby's native battle-object ID.
        pub object: u32,
        /// Whether the observed object is Kirby.
        pub kirby: bool,
        /// Kirby's current native status.
        pub status: i32,
        /// Kirby's current native motion hash.
        pub motion: u64,
        /// Kirby's current motion frame.
        pub frame: f32,
        /// Whether Kirby is grounded.
        pub grounded: bool,
        /// Kirby's left/right facing sign.
        pub facing: f32,
        /// Whether Kirby's native animation is stopped.
        pub stopped: bool,
        /// Bounded snapshot of nearby opponents eligible for trip recognition.
        pub targets: [Option<TripTarget>; 8],
    }
    /// State machine for Kirby's confirmed down-tilt trip follow-up.
    #[derive(Default)]
    pub struct TripFollowup {
        /// Previous Kirby observation used to detect native progress.
        last: Option<TripContext>,
        /// Controller request retained across duplicate mapper polls.
        output: ControllerOutput,
        /// Whether a confirmed trip has armed the follow-up.
        queued: bool,
        /// Whether the current down tilt already consumed its follow-up.
        spent: bool,
        /// Facing captured when the trip armed the follow-up.
        facing: f32,
        target: Option<u32>,
        attempts: u8,
        progress: u8,
    }
    impl TripFollowup {
        /// Cancels the armed trip follow-up and releases its controller output.
        pub fn cancel_pending(&mut self) {
            self.finish();
            self.output = ControllerOutput::default();
        }
        /// Whether a confirmed trip is currently waiting for forward smash.
        pub fn watching(&self) -> bool {
            self.queued
        }
        /// Advances the trip follow-up from Kirby's current native observation.
        pub fn sample(&mut self, enabled: bool, ctx: TripContext) -> ControllerOutput {
            // Invalid/changed identity resets the tool. An identical observation
            // reuses output, so repeated mapper polls do not manufacture new pulses.
            if self.last.is_some_and(|last| last.object != ctx.object) {
                *self = Self::default();
            }
            if !enabled
                || !ctx.kirby
                || !ctx.frame.is_finite()
                || ctx.frame < 0.0
                || !valid_facing(ctx.facing)
            {
                self.finish();
                self.last = Some(ctx);
                self.output = ControllerOutput::default();
                return self.output;
            }
            if self.last == Some(ctx) {
                return self.output;
            }
            let native_progress = !ctx.stopped
                && self.last.is_none_or(|last| {
                    ctx.status != last.status
                        || ctx.motion != last.motion
                        || ctx.frame != last.frame
                });
            if self.spent && !self.queued && !native_progress {
                self.last = Some(ctx);
                return self.output;
            }
            let recognized = self.recognize(ctx);
            let mut out = ControllerOutput::default();
            let was_queued = self.queued;
            if self.queued {
                let target_valid = ctx.targets.iter().flatten().any(|target| {
                    Some(target.id) == self.target && target.nearby(self.facing)
                        && tripping(target.status)
                });
                if ctx.tilt_ended() || ctx.facing.signum() != self.facing || !target_valid {
                    self.finish();
                } else if !recognized && !native_progress {
                    // Changed target facts are evidence, not another input frame.
                    // Preserve the current press/release through duplicate polls.
                    self.last = Some(ctx);
                    return self.output;
                } else {
                    out = self.tilt_lag_output(ctx, recognized);
                }
            }
            // Release immediately at smash startup so the macro does not charge it.
            if out.owned == 0 && was_queued {
                out = hold([0.0, 0.0]);
            }
            self.last = Some(ctx);
            self.output = out;
            out
        }
        /// Requires an observed non-trip to trip transition during this tilt.
        fn recognize(&mut self, ctx: TripContext) -> bool {
            let new_tilt = trip_tilt(ctx.status)
                && self.last.is_none_or(|last| {
                    last.status != ctx.status || ctx.frame < last.frame || ctx.motion != last.motion
                });
            if new_tilt && !self.queued {
                self.spent = false;
            }
            if self.queued || self.spent || !ctx.grounded || !trip_tilt(ctx.status) {
                return false;
            }
            let target = ctx.targets.iter().flatten().find(|enemy| {
                enemy.hit_confirmed && enemy.nearby(ctx.facing) && tripping(enemy.status)
                    && self.last.is_some_and(|last| {
                        last.status == ctx.status && last.motion == ctx.motion
                            && last.targets.iter().flatten().any(|old| {
                                old.id == enemy.id && !tripping(old.status)
                            })
                    })
            });
            if let Some(target) = target {
                self.queued = true;
                self.target = Some(target.id);
                self.facing = ctx.facing.signum();
                self.attempts = 0;
                self.progress = 0;
                true
            } else {
                false
            }
        }
        /// Releases between at most six smash flicks, counting native progress only.
        fn tilt_lag_output(&mut self, ctx: TripContext, recognized: bool) -> ControllerOutput {
            if !recognized && !ctx.stopped {
                self.progress += 1;
                if self.progress >= 18 || (self.attempts >= 6 && self.output.requested & MAPPER_A == 0) {
                    self.finish();
                    return hold([0.0, 0.0]);
                }
                if self.output.requested & MAPPER_A == 0 {
                    self.attempts += 1;
                    return press_a(crate::move_input::smash_stick(
                        crate::move_input::Smash::Forward, self.facing,
                    ));
                }
            }
            hold([0.0, 0.0])
        }
        /// Completes the active operation and records its final state.
        fn finish(&mut self) {
            self.queued = false;
            self.spent = true;
        }
    }
    impl TripContext {
        /// Smash startup was accepted, or Kirby left the grounded tilt/idle states.
        fn tilt_ended(&self) -> bool {
            let accepted = matches!(self.status, status::ATTACK_S4_START..=status::ATTACK_S4);
            let supported = self.grounded
                && matches!(self.status, status::WAIT..=status::TURN_RUN_BRAKE | status::SQUAT..=status::SQUAT_RV | status::ATTACK_S3 | status::ATTACK_LW3);
            accepted || !supported
        }
    }


    #[cfg(test)]
    include!("../../tests/unit/fighter_kirby_trip_followup.rs");
}

pub use fair_followup::{FairContext, FairFollowup};
pub use trip_followup::{TripContext, TripFollowup, TripTarget};

// Follow-ups also suppress taunts; Plus/Minus remain available.
/// Controller buttons owned while a Kirby follow-up is active.
const FOLLOWUP_BUTTONS: u32 =
    crate::native_input::ASSIST_BUTTONS | crate::native_input::MAPPER_DPAD_BUTTONS;

use super::valid_facing;

/// Own every competing gameplay button and the left stick, holding `stick`.
fn hold(stick: [f32; 2]) -> ControllerOutput {
    ControllerOutput {
        owned: FOLLOWUP_BUTTONS,
        stick: Some(stick),
        ..ControllerOutput::default()
    }
}

/// `hold`, plus one fresh A press.
fn press_a(stick: [f32; 2]) -> ControllerOutput {
    ControllerOutput {
        requested: MAPPER_A,
        ..hold(stick)
    }
}
