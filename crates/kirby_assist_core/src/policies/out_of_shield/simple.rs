//! Shielded front-target grab selection.
use crate::{
    gameplay::{ControllerOutput, GameplayContext, Opponent},
    ledge_trap::{NativeAttempt, SEQUENCE_BUTTONS},
    native_input::MAPPER_R,
    roll_read::Target,
};

/// Reach and mode of one automatic grab admission.
#[derive(Clone, Copy)]
pub(super) struct Reach {
    /// Runtime range value used by Reach..
    pub range: f32,
    /// Runtime height value used by Reach..
    pub height: f32,
    /// Runtime startup value used by Reach..
    pub startup: Option<f32>,
    /// Runtime shielded value used by Reach..
    pub shielded: bool,
}

/// Determines whether the `target_valid` condition holds.
/// Checks whether target valid.
pub(super) fn target_valid(
    c: super::Context,
    lc: GameplayContext,
    e: Opponent,
    reach: Reach,
) -> bool {
    let Reach {
        range,
        height,
        startup,
        shielded,
        ..
    } = reach;
    e.vulnerable
        && super::automatic_grab_target_status_with_shield(e.status, shielded)
        && c.facing.is_finite()
        && c.facing.abs() >= 0.5
        && super::target_within_range(c.position, e.position, range, height)
        && (e.position[0] - c.position[0]) * c.facing > 0.
        && {
            let relative = e.velocity[0] - lc.velocity[0];
            relative.is_finite()
                && (relative.abs() <= 0.001
                    || startup
                        .filter(|f| f.is_finite() && *f > 0. && *f <= 120.)
                        .is_some_and(|frames| {
                            let projected = e.position[0] - c.position[0] + relative * frames;
                            projected * c.facing > 0. && projected.abs() <= range
                        }))
        }
}
/// Runtime Smash data carried by `Grab`.
#[derive(Default)]
pub(super) struct Grab {
    /// selected opponent/projectile target.
    target: Option<u32>,
    /// Runtime clock value used by Grab..
    clock: NativeAttempt,
    /// Runtime mapped value used by Grab..
    mapped: u8,
    /// Runtime last mapped value used by Grab..
    last_mapped: Option<i64>,
    /// Runtime delivery polls value used by Grab..
    delivery_polls: u8,
    /// Runtime last delivery poll value used by Grab..
    last_delivery_poll: Option<i64>,
}
impl Grab {
    /// Constructs a new value with the type’s initial state.
    pub const fn new() -> Self {
        Self {
            target: None,
            clock: NativeAttempt::new(),
            mapped: 0,
            last_mapped: None,
            delivery_polls: 0,
            last_delivery_poll: None,
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
    /// Updates the `rebase_custom_delivery` state using the supplied inputs.
    pub fn rebase_custom_delivery(&mut self) {
        if !self.delivery_pending() {
            self.clock = NativeAttempt::default();
        }
    }
    /// Determines whether the `delivery_pending` condition holds.
    pub fn delivery_pending(&self) -> bool {
        self.active() && self.mapped > 0
    }
    /// Updates the `delivery_tick` state using the supplied inputs.
    pub fn delivery_tick(&mut self, tick: i64) {
        if self.active() && self.last_delivery_poll != Some(tick) {
            self.delivery_polls = self.delivery_polls.saturating_add(1);
            self.last_delivery_poll = Some(tick);
        }
    }
    /// Cancels an unanswered grab attempt.
    fn give_up(&mut self) {
        self.cancel();
    }
    /// Updates the `release_unless_delivered` state using the supplied inputs.
    fn release_unless_delivered(&mut self) {
        if !self.delivery_pending() {
            self.cancel();
        }
    }
    /// Updates the `mapped_grab` state using the supplied inputs.
    pub fn mapped_grab(&mut self, tick: i64, target: u32) {
        if self.target == Some(target) && self.last_mapped != Some(tick) {
            self.mapped = self.mapped.saturating_add(1);
            self.last_mapped = Some(tick);
        }
    }
    /// Returns the `output` result.
    fn output(buttons: u32) -> ControllerOutput {
        ControllerOutput {
            owned: SEQUENCE_BUTTONS,
            requested: buttons,
            stick: Some([0.; 2]),
            ..Default::default()
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
        reach: Reach,
    ) -> ControllerOutput {
        if self.active() && !self.delivery_pending() && self.delivery_polls >= 4 {
            self.give_up();
            return ControllerOutput::default();
        }
        // A mapped R may still be in the game's input queue. Once delivered,
        // release control instead of switching that unresolved request to Up+B.
        if self.delivery_pending() && self.settle_delivered(allowed, c, lc, follow) {
            return ControllerOutput::default();
        }
        if self.admission_lost(allowed, c, lc) {
            return ControllerOutput::default();
        }
        let enemy = if self.active() {
            let valid = super::locked_enemy(self.target, follow, &targets)
                .filter(|e| target_valid(c, lc, *e, reach));
            let Some(enemy) = valid else {
                self.release_unless_delivered();
                return ControllerOutput::default();
            };
            enemy
        } else {
            let Some(enemy) = super::nearest_enemy(c.position[0], &targets, follow, |e| {
                target_valid(c, lc, e, reach)
            }) else {
                return ControllerOutput::default();
            };
            self.acquire(enemy);
            enemy
        };
        if self.mapped >= 4 {
            self.cancel();
            return ControllerOutput::default();
        }
        self.clock.advance_pause(lc, Some(enemy), 120);
        if self.clock.expired || self.clock.elapsed() >= 4. {
            self.cancel();
            return ControllerOutput::default();
        }
        super::direct_shield_input(c.status, Self::output(MAPPER_R), [0.; 2])
    }
    /// True when this sample must emit nothing: the delivered Grab ended, expired
    /// or is still settling. Settling owns no buttons or movement; false means
    /// the request is still retryable within its bounded budget.
    fn settle_delivered(
        &mut self,
        allowed: bool,
        c: super::Context,
        lc: GameplayContext,
        follow: Option<Opponent>,
    ) -> bool {
        if matches!(
            c.status,
            crate::native_states::CATCH..=crate::native_states::THROW
        ) || !lc.gameplay
            || !c.grounded
            || lc.situation != crate::native_states::situation::GROUND
            || crate::ledge_trap::sequence_interrupted(c.status)
        {
            self.cancel();
            return true;
        }
        self.clock.advance_pause(lc, follow, 120);
        if self.delivery_polls >= 24 || self.clock.expired || self.clock.elapsed() >= 24. {
            self.give_up();
            return true;
        }
        !allowed
            || c.stopped
            || lc.stopped
            || c.status == crate::native_states::GUARD_DAMAGE
            || self.mapped >= 4
            || self.clock.elapsed() >= 4.
    }
    /// True when no grab may be sampled now. Native incapacitation, a renewed
    /// shield hit or catch startup ends the previous tap; never queue it
    /// through shieldstun or force a cancel of native recovery. A delivered R
    /// keeps its receipt state while the actor is merely not ready.
    fn admission_lost(
        &mut self,
        allowed: bool,
        c: super::Context,
        lc: GameplayContext,
    ) -> bool {
        if !allowed
            || !c.grounded
            || lc.situation != crate::native_states::situation::GROUND
            || !lc.gameplay
            || lc.stopped
            || self.active()
                && (crate::ledge_trap::sequence_interrupted(c.status)
                    || c.status == crate::native_states::GUARD_DAMAGE
                    // Native catch startup confirms delivery; release the input immediately.
                    || matches!(
                        c.status,
                        crate::native_states::CATCH..=crate::native_states::THROW
                    ))
        {
            self.cancel();
            return true;
        }
        let own_ready = super::ground_grab_state(c.status)
            && !c.stopped
            && c.shield_ready
            && (c.status == crate::native_states::GUARD || c.can_grab);
        if !own_ready {
            self.release_unless_delivered();
            return true;
        }
        false
    }
    /// Updates the `acquire` state using the supplied inputs.
    fn acquire(&mut self, e: Opponent) {
        self.target = Some(e.id);
        self.clock = NativeAttempt::default();
        self.mapped = 0;
        self.last_mapped = None;
    }
}
