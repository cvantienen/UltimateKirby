//! Keep a released manual jump from racing a new grounded automatic input
//! while the native fighter has not acknowledged the queued jump yet.
use crate::{gameplay::GameplayContext, ledge_trap::NativeAttempt};
#[derive(Default)]
pub struct ManualJumpFence {
    owner: Option<(u32, i32)>,
    held: bool,
    pending: bool,
    clock: NativeAttempt,
}
impl ManualJumpFence {
    pub const fn new() -> Self {
        Self {
            owner: None,
            held: false,
            pending: false,
            clock: NativeAttempt::new(),
        }
    }
    pub fn pending(&self) -> bool {
        self.pending
    }
    pub fn sample(&mut self, object: u32, kind: i32, held: bool, c: GameplayContext) -> bool {
        if self.owner != Some((object, kind)) {
            *self = Self::new();
            self.owner = Some((object, kind));
        }
        if !c.gameplay {
            *self = Self::new();
            return false;
        }
        if held && !self.held {
            self.pending = true;
            self.clock = NativeAttempt::new();
        }
        self.held = held;
        if self.pending {
            if c.status == crate::native_states::status::JUMP_SQUAT || c.situation == 2 {
                self.pending = false;
            } else {
                // A lag pause/duplicate mapper poll cannot spend this fence.
                // Invalid/skipped animation samples cannot prove jump expiry.
                self.clock.advance_pause(c, None, u16::MAX);
                if self.clock.elapsed() >= crate::move_input::ACCEPTANCE_TIMEOUT {
                    self.pending = false;
                }
            }
        }
        held || self.pending
    }
}
