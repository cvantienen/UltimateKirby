//! Kirby fast-fall fair hit 1 -> landing -> upward-angled forward smash.
//! A fresh hurtbox hit arms the follow-up. Shield/stale hit flags do not.
use super::TRIP_BUTTONS;
use crate::gameplay::ControllerOutput;
use crate::native_input::MAPPER_A;
const FAIR: u64 = crate::timing::MOTION_ATTACK_AIR_F;
const UP_ANGLE: f32 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FairContext {
    pub object: u32,
    pub kirby: bool,
    pub status: i32,
    pub motion: u64,
    pub frame: f32,
    pub rate: f32,
    pub grounded: bool,
    pub stopped: bool,
    pub facing: f32,
    /// AttackModule::is_infliction(HIT), not the persistent status flag.
    pub hit_now: bool,
}
#[derive(Default)]
pub struct FairFollowup {
    last: Option<FairContext>,
    output: ControllerOutput,
    queued: bool,
    spent: bool,
    age: f32,
    facing: f32,
    down_held: bool,
}
impl FairFollowup {
    pub fn active(&self) -> bool {
        self.queued
    }
    pub fn cancel_pending(&mut self) {
        self.finish();
        self.output = ControllerOutput::default();
    }
    pub fn sample(&mut self, enabled: bool, ctx: FairContext) -> ControllerOutput {
        self.sample_with_velocity(enabled, ctx, 0.0)
    }
    pub fn sample_with_velocity(
        &mut self,
        enabled: bool,
        ctx: FairContext,
        velocity_y: f32,
    ) -> ControllerOutput {
        if !enabled
            || !ctx.kirby
            || !ctx.frame.is_finite()
            || ctx.frame < 0.0
            || !ctx.rate.is_finite()
            || ctx.rate <= 0.0
            || !ctx.facing.is_finite()
            || ctx.facing.abs() < 0.5
        {
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
                ctx.status != last.status || ctx.motion != last.motion || ctx.frame != last.frame
            });
        let was_queued = self.queued;
        let fair = ctx.status == crate::native_states::status::ATTACK_AIR && ctx.motion == FAIR;
        // Native aerial status can overlap a grounded situation flag on the
        // landing update. Do not discard its hit or issue another aerial A.
        let landed = ctx.grounded
            && !fair
            && !matches!(
                ctx.status,
                crate::native_states::status::JUMP
                    | crate::native_states::status::JUMP_AERIAL
                    | crate::native_states::status::FALL
                    | crate::native_states::status::FALL_AERIAL
            );
        let new_fair = fair
            && self.last.is_none_or(|last| {
                last.status != crate::native_states::status::ATTACK_AIR
                    || last.motion != FAIR
                    || ctx.frame < last.frame
            });
        if new_fair && !self.queued {
            self.spent = false;
            self.down_held = false;
        }
        let first_hit = fair && (10.0..12.0).contains(&ctx.frame);
        // A later connecting hit invalidates the first-hit-only landing route.
        // Check before duplicate/stop handling: contact can arrive in hitstop.
        if self.queued && fair && ctx.frame >= 12.0 && ctx.hit_now {
            self.finish();
        }
        let landing_hit = landed
            && ctx.status == crate::native_states::status::LANDING_ATTACK_AIR
            && ctx.motion == crate::timing::motion_hash("landing_air_f")
            && self.last.is_some_and(|last| {
                last.status == crate::native_states::status::ATTACK_AIR
                    && last.motion == FAIR
                    && (10.0..12.0).contains(&last.frame)
            });
        let recognized = !self.queued && !self.spent && (first_hit || landing_hit) && ctx.hit_now;
        if recognized {
            self.queued = true;
            self.age = 0.0;
            self.facing = ctx.facing.signum();
        } else if self.queued && !ctx.stopped {
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
                && matches!(
                    ctx.status,
                    crate::native_states::status::FALL | crate::native_states::status::FALL_AERIAL
                )
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
        let mut out = ControllerOutput::default();
        if self.queued {
            // Later animation frames without contact retain hit 1. A later
            // connecting hit, damage, grab, dodge or special cancels the latch.
            let supported = if landed {
                matches!(ctx.status, crate::native_states::status::WAIT..=crate::native_states::status::TURN_RUN_BRAKE | crate::native_states::status::SQUAT..=crate::native_states::status::LANDING_ATTACK_AIR | crate::native_states::status::ATTACK_S4_START..=crate::native_states::status::ATTACK_S4)
            } else {
                fair || matches!(
                    ctx.status,
                    crate::native_states::status::JUMP
                        | crate::native_states::status::JUMP_AERIAL
                        | crate::native_states::status::FALL
                        | crate::native_states::status::FALL_AERIAL
                )
            };
            if !supported
                || self.age > 180.0
                || !landed && self.age >= crate::finisher::FAST_FALL_FRAMES
                || matches!(
                    ctx.status,
                    crate::native_states::status::ATTACK_S4_HOLD
                        | crate::native_states::status::ATTACK_S4
                )
            {
                self.finish();
            } else {
                out.owned = TRIP_BUTTONS;
                out.stick = Some([0.0, 0.0]);
                if !landed {
                    self.down_held |= !ctx.stopped && velocity_y.is_finite() && velocity_y < -0.05;
                    if self.down_held {
                        out.stick = Some([0.0, -1.0]);
                    }
                }
                if ctx.status == crate::native_states::status::ATTACK_S4_START {
                    // Release A to avoid charge, but preserve the angle through
                    // startup until native forward-smash attack selection.
                    out.stick = Some([self.facing, UP_ANGLE]);
                } else if landed && !recognized && !native_progress {
                    out = self.output;
                } else if landed && !recognized && self.output.requested & MAPPER_A == 0 {
                    // Ordinary smash-buffer input during landing lag; no range,
                    // DI, percent or opponent-state gates after confirmation.
                    out.requested = MAPPER_A;
                    out.stick = Some([self.facing, UP_ANGLE]);
                }
            }
        }
        // One explicit release when acceptance/interruption ends ownership.
        if out.owned == 0 && was_queued {
            // Hand back physical movement immediately when an aerial times out.
            if landed {
                out.owned = TRIP_BUTTONS;
                out.stick = Some([0.0, 0.0]);
            }
        }
        self.last = Some(ctx);
        self.output = out;
        out
    }
    fn finish(&mut self) {
        self.queued = false;
        self.down_held = false;
        self.spent = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
