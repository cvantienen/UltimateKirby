//! Release-to-rearm for physical shield suppressed by an automatic punish.
//! No timers, allocations, native queries or changes to requested assist inputs.
use crate::native_input::ButtonTrace;
use crate::{gameplay::GameplayContext, ledge_trap::NativeAttempt};

/// Keep a released manual jump from racing a new grounded automatic input
/// while the native fighter has not acknowledged the queued jump yet.
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
            if c.status == crate::native_states::status::JUMP_SQUAT
                || c.situation == crate::native_states::situation::AIR
            {
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

#[derive(Default)]
pub struct AssistGuardHandoff {
    blocked: u32,
    previous: Option<u32>,
    protected: u32,
}
impl AssistGuardHandoff {
    pub const fn new() -> Self {
        Self {
            blocked: 0,
            previous: None,
            protected: 0,
        }
    }
    pub fn sample(
        &mut self,
        active: bool,
        human: ButtonTrace,
        guards: u32,
        owned: u32,
        requested: u32,
    ) -> u32 {
        // A real release immediately rearms the physical button. A fresh press
        // is an intentional manual command, not an old hold to resurrect.
        let pressed = self.previous.map_or(0, |old| human.held & !old);
        self.previous = Some(human.held);
        self.blocked &= human.held & guards & !pressed;
        if !active {
            self.blocked |= self.protected & human.held & guards & !pressed;
        }
        // Only an actually SUPPRESSED human guard needs release-to-rearm.
        // Direct shield grab/Up-B explicitly keeps guard; quarantining that
        // unchanged hold after a failed request drops a perfectly valid shield.
        self.protected = if active {
            human.held & guards & owned & !requested & !pressed
        } else {
            0
        };
        self.blocked &= !(requested & guards);
        if active {
            self.blocked |= human.held & guards & owned & !requested & !pressed;
        }
        self.blocked
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_input::{MAPPER_ZL, MAPPER_ZR};
    fn held(mask: u32) -> ButtonTrace {
        ButtonTrace {
            held: mask,
            previous: mask,
            ..Default::default()
        }
    }
    #[test]
    fn old_shield_hold_does_not_return_as_a_roll_or_air_dodge() {
        let mut h = AssistGuardHandoff::new();
        let g = MAPPER_ZR | MAPPER_ZL;
        assert_eq!(h.sample(true, held(g), g, g, 0), g);
        for _ in 0..8 {
            assert_eq!(h.sample(false, held(g), g, 0, 0), g);
        }
        assert_eq!(h.sample(false, held(MAPPER_ZR), g, 0, 0), MAPPER_ZR);
        assert_eq!(h.sample(false, ButtonTrace::default(), g, 0, 0), 0);
        assert_eq!(
            h.sample(
                false,
                ButtonTrace {
                    held: g,
                    pressed: g,
                    ..Default::default()
                },
                g,
                0,
                0
            ),
            0
        );
    }
    #[test]
    fn requested_defense_and_fresh_manual_shield_are_not_suppressed() {
        let mut h = AssistGuardHandoff::new();
        let g = MAPPER_ZR;
        assert_eq!(h.sample(true, held(g), g, g, g), 0);
        h.sample(false, ButtonTrace::default(), g, 0, 0);
        assert_eq!(
            h.sample(
                true,
                ButtonTrace {
                    held: g,
                    pressed: g,
                    ..Default::default()
                },
                g,
                g,
                0
            ),
            0
        );
        assert_eq!(h.sample(false, held(g), g, 0, 0), 0);
    }
}
