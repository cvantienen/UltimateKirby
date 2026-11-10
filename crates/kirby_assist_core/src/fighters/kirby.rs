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
    /// Defines the `FAIR` constant used by this module.
    ///
    ///
    ///  Provides the FAIR constant value.
    const FAIR: u64 = crate::timing::MOTION_ATTACK_AIR_F;
    /// Defines the `UP_ANGLE` constant used by this module.
    ///
    ///
    ///  Provides the UP ANGLE constant value.
    const UP_ANGLE: f32 = 0.5;
    /// Defines the `FAST_FALL_FRAMES` constant used by this module.
    ///
    ///
    ///  Provides the FAST FALL FRAMES constant value.
    const FAST_FALL_FRAMES: f32 = 16.0;
    /// Sets the upper bound for latch frames.
    ///
    ///
    ///  Holds the MAX LATCH FRAMES value.
    const MAX_LATCH_FRAMES: f32 = 180.0;
    /// Fair frames whose hit counts as the first hit; contact from the next frame on is a later hit.
    ///
    ///  Fair frames whose hit counts as the first hit; contact from the next frame on is a later hit.
    const FIRST_HIT_FRAMES: Range<f32> = 10.0..12.0;

    /// Defines the `FairContext` data structure.
    ///
    ///
    ///  Holds the data for FairContext.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct FairContext {
        /// Stores the `object` value in `FairContext` as `u32`.
        ///
        ///
        ///  Holds the object value used by FairContext.
        pub object: u32,
        /// Stores the `kirby` value in `FairContext` as `bool`.
        ///
        ///
        ///  Holds the kirby value used by FairContext.
        pub kirby: bool,
        /// Stores the `status` value in `FairContext` as `i32`.
        ///
        ///
        ///  Holds the status value used by FairContext.
        pub status: i32,
        /// Stores the `motion` value in `FairContext` as `u64`.
        ///
        ///
        ///  Holds the motion value used by FairContext.
        pub motion: u64,
        /// Stores the `frame` value in `FairContext` as `f32`.
        ///
        ///
        ///  Holds the frame value used by FairContext.
        pub frame: f32,
        /// Stores the `rate` value in `FairContext` as `f32`.
        ///
        ///
        ///  Holds the rate value used by FairContext.
        pub rate: f32,
        /// Stores the `grounded` value in `FairContext` as `bool`.
        ///
        ///
        ///  Holds the grounded value used by FairContext.
        pub grounded: bool,
        /// Stores the `stopped` value in `FairContext` as `bool`.
        ///
        ///
        ///  Holds the stopped value used by FairContext.
        pub stopped: bool,
        /// Stores the `facing` value in `FairContext` as `f32`.
        ///
        ///
        ///  Holds the facing value used by FairContext.
        pub facing: f32,
        /// AttackModule::is_infliction(HIT), not the persistent status flag.
        ///
        ///  AttackModule::isinfliction(HIT), not the persistent status flag.
        pub hit_now: bool,
    }
    /// Defines the `FairFollowup` data structure.
    ///
    ///
    ///  Holds the data for FairFollowup.
    #[derive(Default)]
    pub struct FairFollowup {
        /// Stores the `last` value in `FairFollowup` as `Option<FairContext>`.
        ///
        ///
        ///  Optionally stores the last for FairFollowup; it may be absent.
        last: Option<FairContext>,
        /// Stores the `output` value in `FairFollowup` as `ControllerOutput`.
        ///
        ///
        ///  Holds the output value used by FairFollowup.
        output: ControllerOutput,
        /// Stores the `queued` value in `FairFollowup` as `bool`.
        ///
        ///
        ///  Holds the queued value used by FairFollowup.
        queued: bool,
        /// Stores the `spent` value in `FairFollowup` as `bool`.
        ///
        ///
        ///  Holds the spent value used by FairFollowup.
        spent: bool,
        /// Stores the `age` value in `FairFollowup` as `f32`.
        ///
        ///
        ///  Holds the age value used by FairFollowup.
        age: f32,
        /// Stores the `facing` value in `FairFollowup` as `f32`.
        ///
        ///
        ///  Holds the facing value used by FairFollowup.
        facing: f32,
        /// Stores the `down_held` value in `FairFollowup` as `bool`.
        ///
        ///
        ///  Holds the down held value used by FairFollowup.
        down_held: bool,
    }
    impl FairFollowup {
        /// Checks whether the feature is active.
        ///
        ///
        ///  Checks whether the feature is on.
        pub fn active(&self) -> bool {
            self.queued
        }
        /// Cancels the pending.
        ///
        ///
        ///  Cancels the pending.
        pub fn cancel_pending(&mut self) {
            self.finish();
            self.output = ControllerOutput::default();
        }
        /// Updates the toggle state from the current button sample and match state.
        ///
        ///
        ///  Checks the current buttons and updates whether the toggle should activate.
        pub fn sample(&mut self, enabled: bool, ctx: FairContext) -> ControllerOutput {
            self.sample_with_velocity(enabled, ctx, 0.0)
        }
        /// Samples the with velocity.
        ///
        ///
        ///  Samples the with velocity.
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
        ///
        ///  Rearm on a new fair, drop the latch on a later hit, and latch the first hit.
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
        /// Advances the age.
        ///
        ///
        ///  Advances the age.
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
        ///
        ///
        ///  Gets the input waiting to be delivered.
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
        ///
        ///
        ///  Finishes the current operation.
        fn finish(&mut self) {
            self.queued = false;
            self.down_held = false;
            self.spent = true;
        }
    }
    impl FairContext {
        /// Determines whether the `usable` condition holds.
        ///
        ///
        ///  Checks whether usable.
        fn usable(&self) -> bool {
            self.kirby
                && self.frame.is_finite()
                && self.frame >= 0.0
                && self.rate.is_finite()
                && self.rate > 0.0
                && valid_facing(self.facing)
        }
        /// Checks whether fair.
        ///
        ///
        ///  Checks whether fair.
        fn is_fair(&self) -> bool {
            self.status == status::ATTACK_AIR && self.motion == FAIR
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        /// Regression test for `later_contact_cancels_first_hit_followup_even_during_hitstop`.
        ///
        ///
        ///  Checks that later contact cancels first hit followup even during hitstop.
        #[test]
        fn later_contact_cancels_first_hit_followup_even_during_hitstop() {
            let mut tool = FairFollowup::default();
            tool.sample(true, fair(10.0));
            let later = FairContext {
                stopped: true,
                ..fair(17.0)
            };
            assert_eq!(tool.sample(true, later).requested, 0);
            assert!(!tool.queued);
            let landing = FairContext {
                grounded: true,
                status: crate::native_states::status::LANDING_ATTACK_AIR,
                motion: 4,
                frame: 0.0,
                hit_now: false,
                ..fair(10.0)
            };
            assert_eq!(tool.sample(true, landing).requested, 0);
        }
        /// Returns the `fair` result.
        ///
        ///
        ///  Gets the fair result.
        fn fair(frame: f32) -> FairContext {
            FairContext {
                object: 1,
                kirby: true,
                status: crate::native_states::status::ATTACK_AIR,
                motion: FAIR,
                frame,
                rate: 1.0,
                grounded: false,
                stopped: false,
                facing: 1.0,
                hit_now: true,
            }
        }
        /// Regression test for `manual_first_hit_survives_late_fair_skipped_frames_and_falling_until_grounded_smash`.
        ///
        ///
        ///  Checks that manual first hit survives late fair skipped frames and falling until grounded smash.
        #[test]
        fn manual_first_hit_survives_late_fair_skipped_frames_and_falling_until_grounded_smash() {
            for facing in [-1.0, 1.0] {
                let mut tool = FairFollowup::default();
                tool.sample(
                    true,
                    FairContext {
                        facing,
                        ..fair(10.0)
                    },
                );
                for frame in [11.0, 13.0, 15.0, 17.0] {
                    let c = FairContext {
                        facing,
                        frame,
                        hit_now: false,
                        ..fair(10.0)
                    };
                    assert_eq!(
                        tool.sample_with_velocity(true, c, -1.0).stick,
                        Some([0.0, -1.0])
                    );
                    assert!(tool.queued);
                }
                for frame in [0.0, 2.0, 0.0] {
                    let c = FairContext {
                        status: crate::native_states::status::FALL,
                        motion: 3,
                        frame,
                        facing,
                        hit_now: false,
                        ..fair(10.0)
                    };
                    assert_eq!(tool.sample_with_velocity(true, c, -1.0).requested, 0);
                    assert!(tool.queued);
                }
                let landing = FairContext {
                    grounded: true,
                    status: crate::native_states::status::LANDING_ATTACK_AIR,
                    motion: 4,
                    frame: 0.0,
                    facing,
                    hit_now: false,
                    ..fair(10.0)
                };
                assert_eq!(tool.sample(true, landing).requested, MAPPER_A);
                assert_eq!(tool.output.stick, Some([facing, UP_ANGLE]));
                for _ in 0..50 {
                    assert_eq!(tool.sample(true, landing).requested, MAPPER_A);
                }
                for frame in 1..12 {
                    let out = tool.sample(
                        true,
                        FairContext {
                            frame: frame as f32,
                            ..landing
                        },
                    );
                    assert_eq!(out.requested, if frame % 2 == 0 { MAPPER_A } else { 0 });
                }
                let accepted = tool.sample(
                    true,
                    FairContext {
                        status: crate::native_states::status::ATTACK_S4_START,
                        motion: 5,
                        ..landing
                    },
                );
                assert_eq!(accepted.requested, 0);
                assert_eq!(accepted.stick, Some([facing, UP_ANGLE]));
            }
        }
        /// Regression test for `first_hit_with_mixed_landing_flag_is_latched_without_sending_a_second_aerial`.
        ///
        ///
        ///  Checks that first hit with mixed landing flag is latched without sending a second aerial.
        #[test]
        fn first_hit_with_mixed_landing_flag_is_latched_without_sending_a_second_aerial() {
            let mut tool = FairFollowup::default();
            let contact = FairContext {
                grounded: true,
                ..fair(11.0)
            };
            assert_eq!(tool.sample(true, contact).requested, 0);
            assert!(tool.queued);
            let landing = FairContext {
                status: crate::native_states::status::LANDING_ATTACK_AIR,
                motion: 2,
                frame: 0.0,
                hit_now: false,
                ..contact
            };
            assert_eq!(tool.sample(true, landing).requested, MAPPER_A);
        }
        /// Regression test for `fresh_first_hit_reported_on_immediate_landing_arms_but_later_hit_does_not`.
        ///
        ///
        ///  Checks that fresh first hit reported on immediate landing arms but later hit does not.
        #[test]
        fn fresh_first_hit_reported_on_immediate_landing_arms_but_later_hit_does_not() {
            for previous_frame in [11.0, 17.0] {
                let mut tool = FairFollowup::default();
                tool.sample(
                    true,
                    FairContext {
                        hit_now: false,
                        ..fair(previous_frame)
                    },
                );
                let landing = FairContext {
                    status: crate::native_states::status::LANDING_ATTACK_AIR,
                    motion: crate::timing::motion_hash("landing_air_f"),
                    frame: 0.0,
                    grounded: true,
                    ..fair(10.0)
                };
                tool.sample(true, landing);
                let out = tool.sample(
                    true,
                    FairContext {
                        frame: 1.0,
                        hit_now: false,
                        ..landing
                    },
                );
                assert_eq!(
                    out.requested,
                    if previous_frame == 11.0 { MAPPER_A } else { 0 }
                );
            }
        }
        /// Regression test for `landing_hit_flag_changes_cannot_create_another_smash_input_frame`.
        ///
        ///
        ///  Checks that landing hit flag changes cannot create another smash input frame.
        #[test]
        fn landing_hit_flag_changes_cannot_create_another_smash_input_frame() {
            let mut tool = FairFollowup::default();
            tool.sample(true, fair(10.0));
            let landing = FairContext {
                grounded: true,
                status: crate::native_states::status::LANDING_ATTACK_AIR,
                motion: 2,
                frame: 0.0,
                hit_now: false,
                ..fair(10.0)
            };
            let press = tool.sample(true, landing);
            assert_eq!(press.requested, MAPPER_A);
            for hit_now in [true, false, true] {
                assert_eq!(tool.sample(true, FairContext { hit_now, ..landing }), press);
            }
            assert_eq!(
                tool.sample(
                    true,
                    FairContext {
                        frame: 1.0,
                        ..landing
                    }
                )
                .requested,
                0
            );
        }
        /// Regression test for `first_hit_waits_for_landing_then_buffers_upward_smash_without_charging`.
        ///
        ///
        ///  Checks that first hit waits for landing then buffers upward smash without charging.
        #[test]
        fn first_hit_waits_for_landing_then_buffers_upward_smash_without_charging() {
            for facing in [-1.0, 1.0] {
                let mut tool = FairFollowup::default();
                let hit = FairContext {
                    facing,
                    stopped: true,
                    ..fair(10.0)
                };
                let armed = tool.sample(true, hit);
                assert_ne!(armed.owned, 0);
                assert_eq!(armed.requested, 0);
                assert_eq!(tool.sample(true, hit), armed);
                assert_eq!(
                    tool.sample(
                        true,
                        FairContext {
                            stopped: false,
                            hit_now: false,
                            ..hit
                        }
                    )
                    .requested,
                    0
                );
                let landing = FairContext {
                    grounded: true,
                    status: crate::native_states::status::LANDING_ATTACK_AIR,
                    motion: 2,
                    frame: 0.0,
                    hit_now: false,
                    stopped: false,
                    ..hit
                };
                let out = tool.sample(true, landing);
                assert_eq!(out.requested, MAPPER_A);
                assert_eq!(out.stick, Some([facing, UP_ANGLE]));
                assert_eq!(
                    tool.sample(
                        true,
                        FairContext {
                            frame: 1.0,
                            ..landing
                        }
                    )
                    .requested,
                    0
                );
                assert_eq!(
                    tool.sample(
                        true,
                        FairContext {
                            frame: 2.0,
                            ..landing
                        }
                    )
                    .requested,
                    MAPPER_A
                );
                let startup = FairContext {
                    status: crate::native_states::status::ATTACK_S4_START,
                    motion: 3,
                    frame: 0.0,
                    ..landing
                };
                let accepted = tool.sample(true, startup);
                assert_eq!(accepted.requested, 0);
                assert_eq!(accepted.stick, Some([facing, UP_ANGLE]));
                let attack = FairContext {
                    status: crate::native_states::status::ATTACK_S4,
                    motion: 4,
                    frame: 0.0,
                    ..landing
                };
                assert_eq!(tool.sample(true, attack).requested, 0);
                assert_eq!(
                    tool.sample(
                        true,
                        FairContext {
                            frame: 1.0,
                            ..attack
                        }
                    )
                    .owned,
                    0,
                    "acceptance must not leave a permanent release takeover"
                );
            }
        }
        /// Regression test for `shield_stale_or_later_hits_and_other_moves_cannot_arm`.
        ///
        ///
        ///  Checks that shield stale or later hits and other moves cannot arm.
        #[test]
        fn shield_stale_or_later_hits_and_other_moves_cannot_arm() {
            for c in [
                FairContext {
                    hit_now: false,
                    ..fair(10.0)
                },
                fair(17.0),
                fair(25.0),
                FairContext {
                    kirby: false,
                    ..fair(10.0)
                },
                FairContext {
                    motion: 0xc3a4e2597,
                    ..fair(10.0)
                },
            ] {
                assert_eq!(
                    FairFollowup::default().sample(true, c),
                    ControllerOutput::default()
                );
            }
        }
        /// Regression test for `damage_still_cancels_the_confirm`.
        ///
        ///
        ///  Checks that damage still cancels the confirm.
        #[test]
        fn damage_still_cancels_the_confirm() {
            for changed in [FairContext {
                status: crate::native_states::status::DAMAGE_FLY,
                motion: 5,
                frame: 0.0,
                ..fair(10.0)
            }] {
                let mut tool = FairFollowup::default();
                tool.sample(true, fair(10.0));
                assert_eq!(tool.sample(true, changed).requested, 0);
                assert!(!tool.queued);
                let ground = FairContext {
                    grounded: true,
                    status: crate::native_states::status::WAIT,
                    motion: 6,
                    frame: 1.0,
                    hit_now: false,
                    ..changed
                };
                assert_eq!(tool.sample(true, ground).owned, 0);
            }
        }
    }
}
mod trip_followup {
    //! Kirby tilt-trip recognition and buffered forward-smash inputs.
    use super::{hold, press_a, valid_facing};
    use crate::gameplay::ControllerOutput;
    use crate::native_input::MAPPER_A;
    use crate::native_states::status;

    /// Defines the `TripTarget` data structure.
    ///
    ///
    ///  Holds the data for TripTarget.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct TripTarget {
        /// Stores the `id` value in `TripTarget` as `u32`.
        ///
        ///
        ///  Holds the id value used by TripTarget.
        pub id: u32,
        /// Stores the `status` value in `TripTarget` as `i32`.
        ///
        ///
        ///  Holds the status value used by TripTarget.
        pub status: i32,
        /// Stores the `previous_status` value in `TripTarget` as `i32`.
        ///
        ///
        ///  Holds the previous status value used by TripTarget.
        pub previous_status: i32,
    }
    /// Determines whether the `trip_tilt` condition holds.
    ///
    ///
    ///  Checks whether trip tilt.
    fn trip_tilt(status: i32) -> bool {
        matches!(status, status::ATTACK_S3 | status::ATTACK_LW3)
    }
    /// Determines whether the `tripping` condition holds.
    ///
    ///
    ///  Checks whether tripping.
    fn tripping(status: i32) -> bool {
        matches!(status, status::SLIP..=status::SLIP_WAIT)
    }

    /// Defines the `TripContext` data structure.
    ///
    ///
    ///  Holds the data for TripContext.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct TripContext {
        /// Stores the `object` value in `TripContext` as `u32`.
        ///
        ///
        ///  Holds the object value used by TripContext.
        pub object: u32,
        /// Stores the `kirby` value in `TripContext` as `bool`.
        ///
        ///
        ///  Holds the kirby value used by TripContext.
        pub kirby: bool,
        /// Stores the `status` value in `TripContext` as `i32`.
        ///
        ///
        ///  Holds the status value used by TripContext.
        pub status: i32,
        /// Stores the `motion` value in `TripContext` as `u64`.
        ///
        ///
        ///  Holds the motion value used by TripContext.
        pub motion: u64,
        /// Stores the `frame` value in `TripContext` as `f32`.
        ///
        ///
        ///  Holds the frame value used by TripContext.
        pub frame: f32,
        /// Stores the `grounded` value in `TripContext` as `bool`.
        ///
        ///
        ///  Holds the grounded value used by TripContext.
        pub grounded: bool,
        /// Stores the `facing` value in `TripContext` as `f32`.
        ///
        ///
        ///  Holds the facing value used by TripContext.
        pub facing: f32,
        /// Stores the `stopped` value in `TripContext` as `bool`.
        ///
        ///
        ///  Holds the stopped value used by TripContext.
        pub stopped: bool,
        /// Stores the `targets` value in `TripContext` as `[Option<TripTarget>; 8]`.
        ///
        ///
        ///  Stores the targets collection for TripContext.
        pub targets: [Option<TripTarget>; 8],
    }
    /// Defines the `TripFollowup` data structure.
    ///
    ///
    ///  Holds the data for TripFollowup.
    #[derive(Default)]
    pub struct TripFollowup {
        /// Stores the `last` value in `TripFollowup` as `Option<TripContext>`.
        ///
        ///
        ///  Optionally stores the last for TripFollowup; it may be absent.
        last: Option<TripContext>,
        /// Stores the `output` value in `TripFollowup` as `ControllerOutput`.
        ///
        ///
        ///  Holds the output value used by TripFollowup.
        output: ControllerOutput,
        /// Stores the `queued` value in `TripFollowup` as `bool`.
        ///
        ///
        ///  Holds the queued value used by TripFollowup.
        queued: bool,
        /// Stores the `spent` value in `TripFollowup` as `bool`.
        ///
        ///
        ///  Holds the spent value used by TripFollowup.
        spent: bool,
        /// Stores the `facing` value in `TripFollowup` as `f32`.
        ///
        ///
        ///  Holds the facing value used by TripFollowup.
        facing: f32,
    }
    impl TripFollowup {
        /// Cancels the pending.
        ///
        ///
        ///  Cancels the pending.
        pub fn cancel_pending(&mut self) {
            self.finish();
            self.output = ControllerOutput::default();
        }
        /// Determines whether the `watching` condition holds.
        ///
        ///
        ///  Checks whether watching.
        pub fn watching(&self) -> bool {
            self.queued
        }
        /// Updates the toggle state from the current button sample and match state.
        ///
        ///
        ///  Checks the current buttons and updates whether the toggle should activate.
        pub fn sample(&mut self, enabled: bool, ctx: TripContext) -> ControllerOutput {
            // Invalid/changed identity resets the tool. An identical observation
            // reuses output, so repeated mapper polls do not manufacture new pulses.
            if !enabled
                || !ctx.kirby
                || !ctx.frame.is_finite()
                || ctx.frame < 0.0
                || !valid_facing(ctx.facing)
            {
                *self = Self::default();
                return ControllerOutput::default();
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
            let recognized = self.recognize(ctx);
            let mut out = ControllerOutput::default();
            let was_queued = self.queued;
            if self.queued {
                if ctx.tilt_ended() {
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
        /// Rearm on a new tilt, then latch a current trip or grounded re-hit
        /// damage whose immediate previous state was a trip. No new-trip
        /// transition or hit flag required.
        ///
        ///  Rearm on a new tilt, then latch a current trip or grounded re-hit damage whose immediate previous state was a trip. No new-trip transition or hit flag required.
        fn recognize(&mut self, ctx: TripContext) -> bool {
            let new_tilt = trip_tilt(ctx.status)
                && self.last.is_none_or(|last| {
                    last.status != ctx.status || ctx.frame < last.frame || ctx.motion != last.motion
                });
            if new_tilt && !self.queued {
                self.spent = false;
            }
            let recognized = !self.queued
                && !self.spent
                && trip_tilt(ctx.status)
                && ctx.targets.iter().flatten().any(|enemy| {
                    tripping(enemy.status)
                        || tripping(enemy.previous_status)
                            && matches!(enemy.status, status::DAMAGE | status::DOWN_DAMAGE)
                });
            if recognized {
                self.queued = true;
                self.facing = ctx.facing.signum();
            }
            recognized
        }
        /// Returns the `tilt_lag_output` result.
        ///
        ///
        ///  Gets the tilt lag output result.
        fn tilt_lag_output(&self, ctx: TripContext, recognized: bool) -> ControllerOutput {
            // Own every competing gameplay button and both sticks from
            // recognition onward, including tilt lag and release gaps. First
            // release the physical tilt/A. Then keep sending fresh directional+A
            // flicks into the game's normal buffer until smash startup is
            // observed. No readiness gate or six-pulse cutoff.
            if !recognized && !ctx.stopped && self.output.requested & MAPPER_A == 0 {
                press_a(crate::move_input::smash_stick(
                    crate::move_input::Smash::Forward,
                    self.facing,
                ))
            } else {
                hold([0.0, 0.0])
            }
        }
        /// Completes the active operation and records its final state.
        ///
        ///
        ///  Finishes the current operation.
        fn finish(&mut self) {
            self.queued = false;
            self.spent = true;
        }
    }
    impl TripContext {
        /// Smash startup was accepted, or Kirby left the grounded tilt/idle states.
        ///
        ///  Smash startup was accepted, or Kirby left the grounded tilt/idle states.
        fn tilt_ended(&self) -> bool {
            let accepted = matches!(self.status, status::ATTACK_S4_START..=status::ATTACK_S4);
            let supported = self.grounded
                && matches!(self.status, status::WAIT..=status::TURN_RUN_BRAKE | status::SQUAT..=status::SQUAT_RV | status::ATTACK_S3 | status::ATTACK_LW3);
            accepted || !supported
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::kirby::FOLLOWUP_BUTTONS;
        /// Returns the `ctx` result.
        ///
        ///
        ///  Gets the ctx result.
        fn ctx(status: i32, target_status: i32) -> TripContext {
            TripContext {
                object: 1,
                kirby: true,
                status,
                motion: 1,
                frame: 1.0,
                grounded: true,
                facing: 1.0,
                stopped: false,
                targets: [
                    Some(TripTarget {
                        id: 2,
                        status: target_status,
                        previous_status: crate::native_states::status::WAIT,
                    }),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ],
            }
        }
        /// Regression test for `trip_immediately_owns_controls_and_buffers_smash_without_readiness_or_target_rechecks`.
        ///
        ///
        ///  Checks that trip immediately owns controls and buffers smash without readiness or target rechecks.
        #[test]
        fn trip_immediately_owns_controls_and_buffers_smash_without_readiness_or_target_rechecks() {
            let mut state = TripFollowup::default();
            let trip = TripContext {
                stopped: true,
                ..ctx(
                    crate::native_states::status::ATTACK_LW3,
                    crate::native_states::status::SLIP,
                )
            };
            let prepare = state.sample(true, trip);
            assert_eq!(prepare.requested, 0);
            assert_eq!(prepare.owned, FOLLOWUP_BUTTONS);
            assert_eq!(prepare.stick, Some([0.0, 0.0]));
            assert!(state.watching());
            assert_eq!(state.sample(true, trip), prepare);
            // A latched trip is not cancelled by missing targets or their recovery.
            // There is no can_smash input: requests start while tilt lag remains.
            for frame in 2..=31 {
                let out = state.sample(
                    true,
                    TripContext {
                        frame: frame as f32,
                        stopped: false,
                        targets: [None; 8],
                        ..trip
                    },
                );
                assert_eq!(out.owned, FOLLOWUP_BUTTONS);
                assert_eq!(out.requested, if frame % 2 == 0 { MAPPER_A } else { 0 });
                assert_eq!(
                    out.stick,
                    Some(if frame % 2 == 0 {
                        [1.0, 0.0]
                    } else {
                        [0.0, 0.0]
                    })
                );
            }
            // A transition to idle still receives a forward-smash request at once.
            let ready = TripContext {
                status: crate::native_states::status::WAIT,
                frame: 0.0,
                targets: [None; 8],
                ..trip
            };
            let ready = TripContext {
                stopped: false,
                ..ready
            };
            assert_eq!(state.sample(true, ready).requested, MAPPER_A);
            let accepted = TripContext {
                status: crate::native_states::status::ATTACK_S4_START,
                frame: 1.0,
                ..ready
            };
            let released = state.sample(true, accepted);
            assert_eq!(released.requested, 0);
            assert_eq!(released.stick, Some([0.0, 0.0]));
            assert!(!state.watching());
            assert_eq!(
                state
                    .sample(
                        true,
                        TripContext {
                            frame: 2.0,
                            ..accepted
                        }
                    )
                    .owned,
                0
            );
        }
        /// Regression test for `either_tilt_trip_flicks_in_existing_facing_and_new_tilt_rearms`.
        ///
        ///
        ///  Checks that either tilt trip flicks in existing facing and new tilt rearms.
        #[test]
        fn either_tilt_trip_flicks_in_existing_facing_and_new_tilt_rearms() {
            for status in [
                crate::native_states::status::ATTACK_S3,
                crate::native_states::status::ATTACK_LW3,
            ] {
                for facing in [-1.0, 1.0] {
                    for trip_status in [
                        crate::native_states::status::SLIP,
                        crate::native_states::status::SLIP_DAMAGE,
                        crate::native_states::status::SLIP_WAIT,
                    ] {
                        let mut state = TripFollowup::default();
                        let trip = TripContext {
                            facing,
                            ..ctx(status, trip_status)
                        };
                        assert_eq!(state.sample(true, trip).requested, 0);
                        let attack = state.sample(true, TripContext { frame: 2.0, ..trip });
                        assert_eq!(attack.requested, MAPPER_A);
                        assert_eq!(attack.stick, Some([facing, 0.]));
                        state.sample(
                            true,
                            TripContext {
                                status: crate::native_states::status::ATTACK_S4_START,
                                ..trip
                            },
                        );
                        state.sample(
                            true,
                            TripContext {
                                status: crate::native_states::status::WAIT,
                                ..trip
                            },
                        );
                        state.sample(true, trip);
                        assert!(state.watching());
                        assert_eq!(
                            state
                                .sample(true, TripContext { frame: 2.0, ..trip })
                                .requested,
                            MAPPER_A
                        );
                    }
                }
            }
        }
        /// Regression test for `trip_takeover_suppresses_physical_actions_stick_and_taunts_through_release_gaps`.
        ///
        ///
        ///  Checks that trip takeover suppresses physical actions stick and taunts through release gaps.
        #[test]
        fn trip_takeover_suppresses_physical_actions_stick_and_taunts_through_release_gaps() {
            use crate::manual_controls::{ManualOverlay, TurboOutput, LEFT_STICK_DIRECTIONS};
            use crate::native_input::ButtonTrace;
            let mut state = TripFollowup::default();
            let mut overlay = ManualOverlay::default();
            let human = ButtonTrace {
                held: FOLLOWUP_BUTTONS | LEFT_STICK_DIRECTIONS,
                ..ButtonTrace::default()
            };
            for frame in 1..=5 {
                let out = state.sample(
                    true,
                    TripContext {
                        frame: frame as f32,
                        ..ctx(
                            crate::native_states::status::ATTACK_LW3,
                            crate::native_states::status::SLIP,
                        )
                    },
                );
                let mapped = overlay.apply_controls(
                    human,
                    [-1.0, -1.0],
                    TurboOutput { stick: out.stick },
                    0,
                    out.requested,
                    out.owned,
                );
                assert_eq!(mapped.buttons.held & FOLLOWUP_BUTTONS, out.requested);
                assert_eq!(mapped.stick, out.stick.unwrap());
                assert_eq!(mapped.buttons.pressed & MAPPER_A, out.requested);
            }
            let off = state.sample(
                false,
                ctx(
                    crate::native_states::status::ATTACK_LW3,
                    crate::native_states::status::SLIP,
                ),
            );
            let restored = overlay.apply_controls(
                human,
                [-1.0, -1.0],
                TurboOutput { stick: off.stick },
                0,
                off.requested,
                off.owned,
            );
            assert_eq!(
                restored.buttons.held & FOLLOWUP_BUTTONS,
                human.held & FOLLOWUP_BUTTONS
            );
            assert_eq!(restored.stick, [-1.0, -1.0]);
        }
        /// Regression test for `no_trip_no_tilt_and_non_kirby_are_inert_and_interruptions_cancel`.
        ///
        ///
        ///  Checks that no trip no tilt and non kirby are inert and interruptions cancel.
        #[test]
        fn no_trip_no_tilt_and_non_kirby_are_inert_and_interruptions_cancel() {
            for sample in [
                ctx(
                    crate::native_states::status::ATTACK_LW3,
                    crate::native_states::status::WAIT,
                ),
                ctx(
                    crate::native_states::status::WAIT,
                    crate::native_states::status::SLIP,
                ),
                TripContext {
                    kirby: false,
                    ..ctx(
                        crate::native_states::status::ATTACK_LW3,
                        crate::native_states::status::SLIP,
                    )
                },
                // An unreadable facing must not guess "forward = right".
                TripContext {
                    facing: 0.0,
                    ..ctx(
                        crate::native_states::status::ATTACK_LW3,
                        crate::native_states::status::SLIP,
                    )
                },
                TripContext {
                    facing: f32::NAN,
                    ..ctx(
                        crate::native_states::status::ATTACK_LW3,
                        crate::native_states::status::SLIP,
                    )
                },
            ] {
                let mut state = TripFollowup::default();
                assert_eq!(state.sample(true, sample), ControllerOutput::default());
                assert!(!state.watching());
            }
            for terminal in [
                TripContext {
                    grounded: false,
                    ..ctx(
                        crate::native_states::status::JUMP,
                        crate::native_states::status::SLIP,
                    )
                },
                ctx(
                    crate::native_states::status::DAMAGE,
                    crate::native_states::status::SLIP,
                ),
            ] {
                let mut state = TripFollowup::default();
                state.sample(
                    true,
                    ctx(
                        crate::native_states::status::ATTACK_LW3,
                        crate::native_states::status::SLIP,
                    ),
                );
                assert_eq!(state.sample(true, terminal).requested, 0);
                assert!(!state.watching());
                assert_eq!(
                    state
                        .sample(
                            true,
                            TripContext {
                                frame: 2.0,
                                ..terminal
                            }
                        )
                        .owned,
                    0
                );
            }
            let mut state = TripFollowup::default();
            state.sample(
                true,
                ctx(
                    crate::native_states::status::ATTACK_LW3,
                    crate::native_states::status::SLIP,
                ),
            );
            assert_eq!(
                state
                    .sample(
                        false,
                        ctx(
                            crate::native_states::status::ATTACK_LW3,
                            crate::native_states::status::SLIP,
                        )
                    )
                    .owned,
                0
            );
            assert!(!state.watching());
            state.sample(
                true,
                ctx(
                    crate::native_states::status::ATTACK_LW3,
                    crate::native_states::status::SLIP,
                ),
            );
            assert_eq!(
                state
                    .sample(
                        true,
                        TripContext {
                            object: 3,
                            ..ctx(
                                crate::native_states::status::WAIT,
                                crate::native_states::status::WAIT,
                            )
                        }
                    )
                    .owned,
                0
            );
            assert!(!state.watching());
        }
    }
}

pub use fair_followup::{FairContext, FairFollowup};
pub use trip_followup::{TripContext, TripFollowup, TripTarget};

// Follow-ups also suppress taunts; Plus/Minus remain available.
/// Defines the `FOLLOWUP_BUTTONS` constant used by this module.
///
/// Provides the FOLLOWUP BUTTONS constant value.
const FOLLOWUP_BUTTONS: u32 =
    crate::native_input::ASSIST_BUTTONS | crate::native_input::MAPPER_DPAD_BUTTONS;

use super::valid_facing;

/// Own every competing gameplay button and the left stick, holding `stick`.
///
///  Own every competing gameplay button and the left stick, holding stick.
fn hold(stick: [f32; 2]) -> ControllerOutput {
    ControllerOutput {
        owned: FOLLOWUP_BUTTONS,
        stick: Some(stick),
        ..ControllerOutput::default()
    }
}

/// `hold`, plus one fresh A press.
///
///  hold, plus one fresh A press.
fn press_a(stick: [f32; 2]) -> ControllerOutput {
    ControllerOutput {
        requested: MAPPER_A,
        ..hold(stick)
    }
}
