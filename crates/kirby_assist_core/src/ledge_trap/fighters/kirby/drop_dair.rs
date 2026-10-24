//! One committed short-hop down-air attempt. No target chasing, range checks,
//! extra airborne jumps, automatic recovery or game-state writes.
use crate::fighter_profiles::kirby::{DROP_DAIR_FIRST_ACTIVE, DROP_DAIR_SELECT_DRIFT};
use crate::ledge_trap::{
    attempt::NativeAttempt, LedgeContext, LedgeOutput, ATTACK_AIR, GUARD, GUARD_DAMAGE, GUARD_ON,
    JUMP_SQUAT, LEDGE_BUTTONS,
};
use crate::native_input::{MAPPER_A, MAPPER_X};

// Hash40("attack_air_lw"). A different aerial must never be treated as dair.
const DOWN_AIR_MOTION: u64 = 0xd40042152;

pub(crate) struct DropDair {
    direction: f32,
    attempt: NativeAttempt,
    jump_seen: bool,
    dair_seen: bool,
    finished: bool,
    output: LedgeOutput,
}
impl DropDair {
    pub fn new(direction: f32) -> Self {
        Self {
            direction: direction.signum(),
            attempt: NativeAttempt::default(),
            jump_seen: false,
            dair_seen: false,
            finished: false,
            output: LedgeOutput::default(),
        }
    }
    pub fn finished(&self) -> bool {
        self.finished
    }
    pub fn owns_jump_state(&self, status: i32) -> bool {
        !self.finished
            && matches!(
                status,
                JUMP_SQUAT | 0xb | 0xe | 0xf | 0x16..=0x18 | ATTACK_AIR
            )
    }
    pub fn sample(&mut self, ctx: LedgeContext) -> LedgeOutput {
        let supported = if ctx.situation == 0 {
            matches!(ctx.status, 0..=JUMP_SQUAT | 0x11..=0x18 | GUARD_ON | GUARD | GUARD_DAMAGE)
        } else {
            ctx.situation == 2 && matches!(ctx.status, 0xb | 0xe | 0xf | ATTACK_AIR)
        };
        if self.finished
            || !supported
            || !ctx.can_control && !ctx.stopped
            || !self.direction.is_finite()
            || self.direction.abs() != 1.0
        {
            return self.finish();
        }
        // Landing/cancelled jump ends the one-shot attempt, including a whiff.
        if self.jump_seen && ctx.situation == 0 && ctx.status != JUMP_SQUAT {
            return self.finish();
        }
        self.jump_seen |= ctx.status == JUMP_SQUAT || ctx.situation == 2;
        let dair = ctx.status == ATTACK_AIR && ctx.motion_kind == DOWN_AIR_MOTION;
        self.dair_seen |= dair;
        let fresh = self
            .attempt
            .advance(ctx, dair, if self.dair_seen { 60 } else { 30 });
        if self.attempt.expired {
            return self.finish();
        }
        if ctx.status == ATTACK_AIR {
            // Release A/down once the real aerial starts: retain full outward
            // drift through startup, without adding a fast-fall flick or a hop.
            if !dair || ctx.motion_frame >= DROP_DAIR_FIRST_ACTIVE {
                return self.finish();
            }
            self.output = self.controls([self.direction, 0.0], 0);
            return self.output;
        }
        if self.dair_seen {
            return self.finish();
        }
        if ctx.status == JUMP_SQUAT || ctx.situation == 2 {
            // Like the existing nair fix, release X in squat but keep attack and
            // direction paired until native attack acceptance. No air-jump retry.
            self.output = self.controls([self.direction * DROP_DAIR_SELECT_DRIFT, -1.0], MAPPER_A);
            return self.output;
        }
        if !fresh {
            return self.output; // Hitstop and duplicate polls cannot spend time.
        }
        let previous_buttons = self.output.requested;
        self.output = self.controls([0.0, 0.0], 0);
        if ctx.can_act && ctx.can_jump && self.attempt.issue(ctx, previous_buttons, MAPPER_X) {
            // Jump first so the initial A cannot be interpreted as neutral air.
            // Like Yoshi's jump-then-directional-attack recipe, release Jump
            // and buffer down+A as soon as native squat/takeoff is observed.
            self.output = self.controls([0.0, 0.0], MAPPER_X);
        }
        self.output
    }
    fn controls(&self, stick: [f32; 2], requested: u32) -> LedgeOutput {
        LedgeOutput {
            consume_trigger: true,
            owned: LEDGE_BUTTONS,
            stick: Some(stick),
            requested,
        }
    }
    fn finish(&mut self) -> LedgeOutput {
        self.finished = true;
        self.output = LedgeOutput::default();
        self.output
    }
}
