//! Opponent-directed tracking after Bowser's shield Up-B starts natively.
use super::{
    ControllerOutput, GameplayContext, NativeAttempt, Opponent, ShieldUpB, Target, Timing,
    SEQUENCE_BUTTONS,
};

impl ShieldUpB {
    pub(super) fn sample_spin(
        &mut self,
        allowed: bool,
        c: super::super::Context,
        lc: GameplayContext,
        targets: [Option<Target>; 8],
        follow: Option<Opponent>,
        timing: Timing,
    ) -> Option<ControllerOutput> {
        let bowser_spin = c.kind == crate::fighter_kinds::BOWSER
            && crate::native_states::up_special_status(c.kind, c.status);
        if !self.active() || !(self.tracking || bowser_spin) {
            return None;
        }
        if !allowed || !lc.gameplay || !bowser_spin {
            self.cancel();
            return Some(ControllerOutput::default());
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
            return Some(ControllerOutput::default());
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
                return Some(ControllerOutput::default());
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
            return Some(ControllerOutput::default());
        }
        if !c.stopped && !lc.stopped {
            self.direction = toward();
        }
        Some(ControllerOutput {
            owned: SEQUENCE_BUTTONS,
            stick: Some([self.direction, 0.]),
            ..Default::default()
        })
    }
}
