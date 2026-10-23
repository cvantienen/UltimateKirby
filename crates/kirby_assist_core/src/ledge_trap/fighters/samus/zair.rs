//! One jump, then one airborne Grab request. Pairing Grab with grounded jump
//! can select a ground grab instead; wait for observed airborne state first.
use crate::fighter_profiles::samus::ZAIR_ACTIVE;
use crate::ledge_trap::{
    attempt::NativeAttempt, LedgeContext, LedgeOutput, GUARD, GUARD_DAMAGE, GUARD_ON, JUMP_SQUAT,
    LEDGE_BUTTONS,
};
use crate::native_input::{MAPPER_R, MAPPER_X, MAPPER_ZR};

// Pinned lua_const.rs: AIR_LASSO, not ATTACK_AIR (0x36).
const AIR_LASSO: i32 = 0xf8;
pub(crate) struct Zair {
    jump: NativeAttempt,
    grab: NativeAttempt,
    jump_seen: bool,
    grab_requested: bool,
    lasso_seen: bool,
    finished: bool,
    output: LedgeOutput,
}
impl Zair {
    pub fn new() -> Self {
        Self {
            jump: NativeAttempt::default(),
            grab: NativeAttempt::default(),
            jump_seen: false,
            grab_requested: false,
            lasso_seen: false,
            finished: false,
            output: LedgeOutput::default(),
        }
    }
    pub fn finished(&self) -> bool {
        self.finished
    }
    pub fn owns_jump_state(&self, status: i32) -> bool {
        !self.finished
            && matches!(status, JUMP_SQUAT | 0xb | 0xe | 0xf | AIR_LASSO | 0xf9..=0xfd | 0x16..=0x18)
    }
    pub fn sample(&mut self, c: LedgeContext) -> LedgeOutput {
        let supported = if c.situation == 0 {
            matches!(c.status, 0..=JUMP_SQUAT | 0x11..=0x18 | GUARD_ON | GUARD | GUARD_DAMAGE)
        } else {
            c.situation == 2 && matches!(c.status, 0xb | 0xe | 0xf | AIR_LASSO)
        };
        if self.finished || !supported || !c.can_control && !c.stopped {
            return self.finish();
        }
        if self.jump_seen && c.situation == 0 && c.status != JUMP_SQUAT {
            return self.finish();
        }
        self.jump_seen |= c.status == JUMP_SQUAT || c.situation == 2;
        let lasso = c.status == AIR_LASSO;
        self.lasso_seen |= lasso;
        if lasso {
            self.grab.advance(c, true, 60);
            if self.grab.expired || c.motion_frame > f32::from(ZAIR_ACTIVE.last) {
                return self.finish();
            }
            // Release R after acceptance, preserve neutral drift/no fast fall
            // through both native active phases. Never extend their duration.
            self.output = controls(0);
            return self.output;
        }
        if self.lasso_seen {
            return self.finish();
        }
        if c.status == JUMP_SQUAT {
            self.output = controls(0); // Release X; do not ground-grab in squat.
            return self.output;
        }
        if c.situation == 2 {
            let fresh = self.grab.advance(c, false, 30);
            if self.grab.expired {
                return self.finish();
            }
            if !fresh {
                return self.output;
            }
            let previous = self.output.requested;
            self.output = controls(0);
            // Like the buffered aerial fix, acceptance ends the hold, not an
            // assumed two-frame delivery delay. No extra jump is ever sent.
            if self.grab_requested || self.grab.issue(c, previous, MAPPER_R) {
                self.grab_requested = true;
                self.output = controls(MAPPER_R);
            }
            return self.output;
        }
        if c.status == GUARD_DAMAGE {
            self.output = controls(MAPPER_ZR);
            return self.output;
        }
        let fresh = self.jump.advance(c, false, 30);
        if self.jump.expired {
            return self.finish();
        }
        if !fresh {
            return self.output;
        }
        let previous = self.output.requested;
        self.output = controls(0);
        if c.can_jump
            && (c.can_act || matches!(c.status, GUARD_ON | GUARD))
            && (self.jump.holding_special(c) || self.jump.issue(c, previous, MAPPER_X))
        {
            self.output = controls(MAPPER_X);
        }
        self.output
    }
    fn finish(&mut self) -> LedgeOutput {
        self.finished = true;
        self.output = LedgeOutput::default();
        self.output
    }
}
fn controls(requested: u32) -> LedgeOutput {
    LedgeOutput {
        consume_trigger: true,
        owned: LEDGE_BUTTONS,
        stick: Some([0.0; 2]),
        requested,
    }
}
