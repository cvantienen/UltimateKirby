//! Kirby fast-fall fair hit 1 -> landing -> upward-angled forward smash.
//! A fresh hurtbox hit arms the follow-up. Shield/stale hit flags do not.
use super::TRIP_BUTTONS;
use crate::ledge_trap::LedgeOutput;
use crate::native_input::MAPPER_A;
const FAIR: u64 = 0xc3495ada5; // Hash40("attack_air_f")
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
    output: LedgeOutput,
    queued: bool,
    spent: bool,
    age: f32,
    facing: f32,
}
impl FairFollowup {
    pub fn sample(&mut self, enabled: bool, ctx: FairContext) -> LedgeOutput {
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
        let was_queued = self.queued;
        let fair = ctx.status == 0x36 && ctx.motion == FAIR;
        let new_fair = fair
            && self.last.is_none_or(|last| {
                last.status != 0x36 || last.motion != FAIR || ctx.frame < last.frame
            });
        if new_fair && !self.queued {
            self.spent = false;
        }
        let recognized = !self.queued
            && !self.spent
            && fair
            && !ctx.grounded
            && (10.0..12.0).contains(&ctx.frame)
            && ctx.hit_now;
        if recognized {
            self.queued = true;
            self.age = 0.0;
            self.facing = ctx.facing.signum();
        } else if self.queued && !ctx.stopped {
            let delta = self.last.map_or(0.0, |last| {
                if last.status == ctx.status && last.motion == ctx.motion {
                    (ctx.frame - last.frame) / last.rate
                } else {
                    1.0
                }
            });
            if !delta.is_finite() || !(0.0..=8.0).contains(&delta) {
                self.finish();
            } else {
                self.age += delta;
            }
        }
        let mut out = LedgeOutput::default();
        if self.queued {
            // If fair reaches hit 2 before landing, this is no longer the first-
            // hit landing route. Damage, grab, dodge and special cancel it too.
            let supported = if ctx.grounded {
                matches!(ctx.status, 0..=9 | 0x11..=0x18 | 0x2d..=0x2f)
            } else {
                fair && ctx.frame < 17.0
            };
            if !supported || self.age > 30.0 || matches!(ctx.status, 0x2e | 0x2f) {
                self.finish();
            } else {
                out.owned = TRIP_BUTTONS;
                out.stick = Some([0.0, 0.0]);
                if ctx.status == 0x2d {
                    // Release A to avoid charge, but preserve the angle through
                    // startup until native forward-smash attack selection.
                    out.stick = Some([self.facing, UP_ANGLE]);
                } else if ctx.grounded
                    && !recognized
                    && !ctx.stopped
                    && self.output.requested & MAPPER_A == 0
                {
                    // Ordinary smash-buffer input during landing lag; no range,
                    // DI, percent or opponent-state gates after confirmation.
                    out.requested = MAPPER_A;
                    out.stick = Some([self.facing, UP_ANGLE]);
                }
            }
        }
        // One explicit release when acceptance/interruption ends ownership.
        if out.owned == 0 && was_queued {
            out.owned = TRIP_BUTTONS;
            out.stick = Some([0.0, 0.0]);
        }
        self.last = Some(ctx);
        self.output = out;
        out
    }
    fn finish(&mut self) {
        self.queued = false;
        self.spent = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fair(frame: f32) -> FairContext {
        FairContext {
            object: 1,
            kirby: true,
            status: 0x36,
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
                status: 0x18,
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
                status: 0x2d,
                motion: 3,
                frame: 0.0,
                ..landing
            };
            let accepted = tool.sample(true, startup);
            assert_eq!(accepted.requested, 0);
            assert_eq!(accepted.stick, Some([facing, UP_ANGLE]));
            let attack = FairContext {
                status: 0x2f,
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
                LedgeOutput::default()
            );
        }
    }
    #[test]
    fn reaching_second_fair_hit_or_damage_cancels_the_confirm() {
        for changed in [
            fair(17.0),
            FairContext {
                status: 0x49,
                motion: 5,
                frame: 0.0,
                ..fair(10.0)
            },
        ] {
            let mut tool = FairFollowup::default();
            tool.sample(true, fair(10.0));
            assert_eq!(tool.sample(true, changed).requested, 0);
            assert!(!tool.queued);
            let ground = FairContext {
                grounded: true,
                status: 0,
                motion: 6,
                frame: 1.0,
                hit_now: false,
                ..changed
            };
            assert_eq!(tool.sample(true, ground).owned, 0);
        }
    }
}
