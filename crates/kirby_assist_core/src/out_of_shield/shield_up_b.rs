//! Immediate shield-grab fallback. Fixed-size facts, no search beyond eight fighters.
use crate::{
    gameplay::{ControllerOutput, GameplayContext, Opponent},
    ledge_trap::{NativeAttempt, SEQUENCE_BUTTONS},
    native_input::{MAPPER_B, MAPPER_ZR},
    roll_read::Target,
};

#[derive(Clone, Copy)]
pub(super) struct Timing {
    pub delivery: u8,
    pub tracking: u8,
    pub strength: f32,
    pub timeout: u16,
    pub direction: crate::sequence_config::Direction,
}
impl Timing {
    pub fn legacy(kind: i32) -> Self {
        Self {
            delivery: 4,
            tracking: if kind == crate::fighter_kinds::BOWSER {
                69
            } else {
                0
            },
            strength: 1.,
            timeout: 120,
            direction: crate::sequence_config::Direction::Opponent,
        }
    }
}
#[derive(Default)]
pub(super) struct ShieldUpB {
    target: Option<u32>,
    clock: NativeAttempt,
    mapped: u8,
    last_mapped: Option<i64>,
    tracking: bool,
    direction: f32,
    spent_before_tracking: f32,
    bowser: bool,
    receipt: crate::move_input::UpBReceipt,
    waiting: bool,
}
impl ShieldUpB {
    pub const fn new() -> Self {
        Self {
            target: None,
            clock: NativeAttempt::new(),
            mapped: 0,
            last_mapped: None,
            tracking: false,
            direction: 0.,
            spent_before_tracking: 0.,
            bowser: false,
            receipt: crate::move_input::UpBReceipt::new(),
            waiting: false,
        }
    }
    pub fn target(&self) -> Option<u32> {
        self.target
    }
    pub fn active(&self) -> bool {
        self.target.is_some()
    }
    pub fn cancel(&mut self) {
        *self = Self::new();
    }
    pub fn attempted(&mut self, tick: i64, target: u32) {
        if self.target == Some(target) && self.last_mapped != Some(tick) {
            self.mapped = self.mapped.saturating_add(1);
            self.last_mapped = Some(tick);
        }
    }
    pub fn mapped(&mut self, tick: i64, target: u32) {
        self.attempted(tick, target);
        if self.target == Some(target) && self.bowser && !self.tracking {
            self.receipt.mapped(tick);
        }
    }
    pub fn delivery_started(&self) -> bool {
        self.active() && (self.tracking || self.receipt.sent())
    }
    pub fn receipt_pending(&self) -> bool {
        self.active() && self.bowser && !self.tracking && self.receipt.sent()
    }
    pub fn awaiting(&self) -> bool {
        self.receipt_pending() && self.waiting
    }
    pub fn delivery_tick(&mut self, tick: i64) {
        if self.receipt_pending() {
            self.receipt.tick(tick);
        }
    }
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
        // Bowser only: after confirmed native startup, use the existing tracked
        // identity for up to 69 progress frames of full opponent-directed drift.
        // This never starts a special in air or carries drift past the spin.
        let bowser_spin = c.kind == crate::fighter_kinds::BOWSER
            && crate::native_states::up_special_status(c.kind, c.status);
        if self.active() && (self.tracking || bowser_spin) {
            if !allowed || !lc.gameplay || !bowser_spin {
                self.cancel();
                return ControllerOutput::default();
            }
            let enemy = follow.filter(|e| Some(e.id) == self.target).or_else(|| {
                targets
                    .iter()
                    .flatten()
                    .find(|t| Some(t.id) == self.target)
                    .map(|t| Opponent {
                        id: t.id,
                        status: t.status,
                        position: t.position,
                        ..Default::default()
                    })
            });
            let Some(enemy) = enemy.filter(|e| {
                e.position
                    .iter()
                    .chain(c.position.iter())
                    .all(|v| v.is_finite())
                    && crate::reaction_rules::automatic_tracking_target_available(e.status)
            }) else {
                self.cancel();
                return ControllerOutput::default();
            };
            let toward = || {
                let dx = enemy.position[0] - c.position[0];
                let x = if dx.abs() < 0.001 { 0. } else { dx.signum() };
                use crate::sequence_config::Direction;
                (match timing.direction {
                    Direction::AwayFromOpponent => -x,
                    Direction::Facing => c.facing.signum(),
                    Direction::Backward => -c.facing.signum(),
                    _ => x,
                }) * timing.strength
            };
            if !self.tracking {
                self.clock.advance_pause(lc, Some(enemy), timing.timeout);
                self.spent_before_tracking = self.clock.elapsed();
                if self.clock.expired {
                    self.cancel();
                    return ControllerOutput::default();
                }
                self.tracking = true;
                self.clock = NativeAttempt::default();
                self.direction = toward();
            }
            self.clock.advance_pause(lc, Some(enemy), timing.timeout);
            if self.clock.expired
                || self.clock.elapsed() >= f32::from(timing.tracking)
                || self.spent_before_tracking + self.clock.elapsed() >= f32::from(timing.timeout)
            {
                self.cancel();
                return ControllerOutput::default();
            }
            if !c.stopped && !lc.stopped {
                self.direction = toward();
            }
            return ControllerOutput {
                owned: SEQUENCE_BUTTONS,
                stick: Some([self.direction, 0.]),
                ..Default::default()
            };
        }
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
                && range.is_finite()
                && range > 0.
                && e.position
                    .iter()
                    .chain(c.position.iter())
                    .all(|v| v.is_finite())
                && (e.position[0] - c.position[0]).abs() <= range
                && height.is_finite()
                && height >= 0.
                && (e.position[1] - c.position[1]).abs() <= height
        };
        let copied = |t: &Target| Opponent {
            id: t.id,
            status: t.status,
            position: t.position,
            velocity: [t.velocity_x, 0.],
            vulnerable: t.vulnerable_in == Some(0.),
            motion: Some(crate::gameplay::MotionTiming {
                kind: t.motion,
                frame: t.frame,
                rate: t.rate,
                remaining: None,
            }),
            ..Default::default()
        };
        let enemy = if let Some(id) = self.target {
            follow
                .filter(|e| e.id == id)
                .or_else(|| targets.iter().flatten().find(|t| t.id == id).map(copied))
                .filter(|e| {
                    if self.receipt_pending() {
                        e.position
                            .iter()
                            .chain(c.position.iter())
                            .all(|v| v.is_finite())
                            && crate::reaction_rules::automatic_tracking_target_available(e.status)
                    } else {
                        valid(*e)
                    }
                })
        } else {
            targets
                .iter()
                .flatten()
                .map(copied)
                .chain(follow)
                .filter(|e| valid(*e))
                .min_by(|a, b| {
                    (a.position[0] - c.position[0])
                        .abs()
                        .total_cmp(&(b.position[0] - c.position[0]).abs())
                })
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
        }
        if self.target.is_none() {
            self.target = Some(enemy.id);
            self.bowser = c.kind == crate::fighter_kinds::BOWSER;
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
