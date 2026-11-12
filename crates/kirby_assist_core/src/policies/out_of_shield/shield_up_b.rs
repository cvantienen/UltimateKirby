//! Immediate shield-grab fallback. Fixed-size facts, no search beyond eight fighters.
use crate::{
    gameplay::{ControllerOutput, GameplayContext, Opponent},
    ledge_trap::{NativeAttempt, SEQUENCE_BUTTONS},
    native_input::{MAPPER_B, MAPPER_ZR},
    roll_read::Target,
};

/// Runtime Smash data carried by `Timing`.
#[derive(Clone, Copy)]
pub(super) struct Timing {
/// Runtime delivery value used by Timing..
    pub delivery: u8,
/// Runtime timeout value used by Timing..
    pub timeout: u16,
}
impl Timing {
    /// Returns the `legacy` result.
    pub fn legacy() -> Self {
        Self {
            delivery: 4,
            timeout: 120,
        }
    }
}
/// Runtime Smash data carried by `ShieldUpB`.
#[derive(Default)]
pub(super) struct ShieldUpB {
/// selected opponent/projectile target.
    target: Option<u32>,
/// Runtime clock value used by ShieldUpB..
    clock: NativeAttempt,
/// Runtime mapped value used by ShieldUpB..
    mapped: u8,
/// Runtime last mapped value used by ShieldUpB..
    last_mapped: Option<i64>,
/// Runtime receipt value used by ShieldUpB..
    receipt: crate::move_input::UpBReceipt,
/// Runtime waiting value used by ShieldUpB..
    waiting: bool,
}
impl ShieldUpB {
    /// Constructs a new value with the type’s initial state.
    pub const fn new() -> Self {
        Self {
            target: None,
            clock: NativeAttempt::new(),
            mapped: 0,
            last_mapped: None,
            receipt: crate::move_input::UpBReceipt::new(),
            waiting: false,
        }
    }
    /// Returns the selected target observation.
    pub fn target(&self) -> Option<u32> {
        self.target
    }
    /// Checks whether the feature is active.
    pub fn active(&self) -> bool {
        self.target.is_some()
    }
    /// Cancels the pending operation and clears its state.
    pub fn cancel(&mut self) {
        *self = Self::new();
    }
    /// Updates the `attempted` state using the supplied inputs.
    pub fn attempted(&mut self, tick: i64, target: u32) {
        if self.target == Some(target) && self.last_mapped != Some(tick) {
            self.mapped = self.mapped.saturating_add(1);
            self.last_mapped = Some(tick);
        }
    }
    /// Checks whether the requested input was acknowledged by the mapper.
    pub fn mapped(&mut self, tick: i64, target: u32) {
        self.attempted(tick, target);
        if self.target == Some(target) {
            self.receipt.mapped(tick);
        }
    }
    /// Determines whether the `delivery_started` condition holds.
    pub fn delivery_started(&self) -> bool {
        self.active() && self.receipt.sent()
    }
    /// Determines whether the `receipt_pending` condition holds.
    pub fn receipt_pending(&self) -> bool {
        self.active() && self.receipt.sent()
    }
    /// Determines whether the `awaiting` condition holds.
    pub fn awaiting(&self) -> bool {
        self.receipt_pending() && self.waiting
    }
    /// Updates the `delivery_tick` state using the supplied inputs.
    pub fn delivery_tick(&mut self, tick: i64) {
        if self.receipt_pending() {
            self.receipt.tick(tick);
        }
    }
    /// Updates the toggle state from the current button sample and match state.
    pub fn sample(
        &mut self,
        allowed: bool,
        c: super::Context,
        lc: GameplayContext,
        targets: [Option<Target>; 8],
        follow: Option<Opponent>,
        range: f32,
        height: f32,
        timing: Timing,
    ) -> ControllerOutput {
        let ready = c.grounded
            && !c.stopped
            && !lc.stopped
            && matches!(
                c.status,
                crate::native_states::GUARD_ON | crate::native_states::GUARD
            )
            && c.shield_ready
            && c.can_up_special;
        if !allowed || !lc.gameplay || !self.receipt_pending() && !ready {
            self.cancel();
            return ControllerOutput::default();
        }
        if self.receipt_pending()
            && (!crate::reaction_rules::automatic_up_b_receipt_status(
                c.status,
                lc.situation,
                false,
            ) || self.receipt.expired())
        {
            self.cancel();
            return ControllerOutput::default();
        }
        let valid = |e: Opponent| {
            e.vulnerable
                && super::automatic_shield_target_status(e.status)
                && super::target_within_range(c.position, e.position, range, height)
        };
        let enemy = if let Some(id) = self.target {
            super::locked_enemy(Some(id), follow, &targets).filter(|e| {
                if self.receipt_pending() {
                    e.position
                        .iter()
                        .chain(c.position.iter())
                        .all(|v| v.is_finite())
                } else {
                    valid(*e)
                }
            })
        } else {
            super::nearest_enemy(c.position[0], &targets, follow, valid)
        };
        let Some(enemy) = enemy else {
            self.cancel();
            return ControllerOutput::default();
        };
        if self.receipt_pending()
            && (self.waiting
                || !ready
                || !valid(enemy)
                || self.mapped >= timing.delivery
                || self.clock.elapsed() >= f32::from(timing.delivery))
        {
            self.waiting = true;
            return ControllerOutput::default();
        };
        if self.target.is_none() {
            self.target = Some(enemy.id);
        }
        self.clock.advance_pause(lc, Some(enemy), timing.timeout);
        if self.mapped >= timing.delivery
            || self.clock.expired
            || self.clock.elapsed() >= f32::from(timing.delivery)
        {
            if self.receipt_pending() && !self.clock.expired {
                self.waiting = true;
                return ControllerOutput::default();
            }
            self.cancel();
            return ControllerOutput::default();
        }
        ControllerOutput {
            owned: SEQUENCE_BUTTONS,
            requested: MAPPER_B | MAPPER_ZR,
            stick: Some([0., 1.]),
            ..Default::default()
        }
    }
}
