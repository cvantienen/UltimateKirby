//! Front vulnerable-target selection: enemy shield required only when unshielded.
use crate::{
    gameplay::{ControllerOutput, GameplayContext, Opponent},
    ledge_trap::{NativeAttempt, SEQUENCE_BUTTONS},
    native_input::MAPPER_R,
    roll_read::Target,
};

/// Reach and mode of one automatic grab admission.
///
///  Reach and mode of one automatic grab admission.
#[derive(Clone, Copy)]
pub(super) struct Reach {
    /// Stores the `range` value in `Reach` as `f32`.
    ///
    ///
///  Holds the range value used by Reach.
    pub range: f32,
    /// Stores the `height` value in `Reach` as `f32`.
    ///
    ///
///  Holds the height value used by Reach.
    pub height: f32,
    /// Stores the `startup` value in `Reach` as `Option<f32>`.
    ///
    ///
///  Optionally stores the startup for Reach; it may be absent.
    pub startup: Option<f32>,
    /// An idle approach farther than this must enter native dash before R.
    ///
///  An idle approach farther than this must enter native dash before R.
    pub close: f32,
    /// Stores the `approach` value in `Reach` as `bool`.
    ///
    ///
///  Holds the approach value used by Reach.
    pub approach: bool,
    /// Stores the `shielded` value in `Reach` as `bool`.
    ///
    ///
///  Holds the shielded value used by Reach.
    pub shielded: bool,
}

/// Determines whether the `target_valid` condition holds.
///
///
///  Checks whether target valid.
pub(super) fn target_valid(c: super::Context, lc: GameplayContext, e: Opponent, reach: Reach) -> bool {
    let Reach {
        range,
        height,
        startup,
        approach,
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
                && (approach
                    || relative.abs() <= 0.001
                    || startup
                        .filter(|f| f.is_finite() && *f > 0. && *f <= 120.)
                        .is_some_and(|frames| {
                            let projected = e.position[0] - c.position[0] + relative * frames;
                            projected * c.facing > 0. && projected.abs() <= range
                        }))
        }
}
/// Defines the `Grab` data structure.
///
///
///  Holds the data for Grab.
#[derive(Default)]
pub(super) struct Grab {
    /// Stores the `target` value in `Grab` as `Option<u32>`.
    ///
    ///
///  Optionally stores the target for Grab; it may be absent.
    target: Option<u32>,
    /// Stores the `clock` value in `Grab` as `NativeAttempt`.
    ///
    ///
///  Holds the clock value used by Grab.
    clock: NativeAttempt,
    /// Stores the `mapped` value in `Grab` as `u8`.
    ///
    ///
///  Holds the mapped value used by Grab.
    mapped: u8,
    /// Stores the `last_mapped` value in `Grab` as `Option<i64>`.
    ///
    ///
///  Optionally stores the last mapped for Grab; it may be absent.
    last_mapped: Option<i64>,
    /// Stores the `approach` value in `Grab` as `bool`.
    ///
    ///
///  Holds the approach value used by Grab.
    approach: bool,
    /// Stores the `dash_sent` value in `Grab` as `bool`.
    ///
    ///
///  Holds the dash sent value used by Grab.
    dash_sent: bool,
    /// Stores the `direction` value in `Grab` as `f32`.
    ///
    ///
///  Holds the direction value used by Grab.
    direction: f32,
    /// Stores the `await_dash` value in `Grab` as `bool`.
    ///
    ///
///  Holds the await dash value used by Grab.
    await_dash: bool,
    /// Stores the `assisted_dash` value in `Grab` as `bool`.
    ///
    ///
///  Holds the assisted dash value used by Grab.
    assisted_dash: bool,
    /// Stores the `approach_mapped` value in `Grab` as `u8`.
    ///
    ///
///  Holds the approach mapped value used by Grab.
    approach_mapped: u8,
    /// Stores the `delivery_polls` value in `Grab` as `u8`.
    ///
    ///
///  Holds the delivery polls value used by Grab.
    delivery_polls: u8,
    /// Stores the `last_delivery_poll` value in `Grab` as `Option<i64>`.
    ///
    ///
///  Optionally stores the last delivery poll for Grab; it may be absent.
    last_delivery_poll: Option<i64>,
    /// Stores the `approach_polls` value in `Grab` as `u8`.
    ///
    ///
///  Holds the approach polls value used by Grab.
    approach_polls: u8,
    /// Stores the `last_approach_poll` value in `Grab` as `Option<i64>`.
    ///
    ///
///  Optionally stores the last approach poll for Grab; it may be absent.
    last_approach_poll: Option<i64>,
    /// Stores the `failed_approach` value in `Grab` as `Option<(u32`.
    ///
    ///
///  Optionally stores the failed approach for Grab; it may be absent.
    failed_approach: Option<(u32, u64)>,
}
impl Grab {
    /// Constructs a new value with the type’s initial state.
    ///
    ///
///  Creates a new value.
    pub const fn new() -> Self {
        Self {
            target: None,
            clock: NativeAttempt::new(),
            mapped: 0,
            last_mapped: None,
            approach: false,
            dash_sent: false,
            direction: 0.,
            await_dash: false,
            assisted_dash: false,
            approach_mapped: 0,
            delivery_polls: 0,
            last_delivery_poll: None,
            approach_polls: 0,
            last_approach_poll: None,
            failed_approach: None,
        }
    }
    /// Returns the selected target observation.
    ///
    ///
///  Gets the chosen target.
    pub fn target(&self) -> Option<u32> {
        self.target
    }
    /// Checks whether the feature is active.
    ///
    ///
///  Checks whether the feature is on.
    pub fn active(&self) -> bool {
        self.target.is_some()
    }
    /// Cancels the pending operation and clears its state.
    ///
    ///
///  Stops the operation that was waiting.
    pub fn cancel(&mut self) {
        // Failure remains fenced until new manual intent or a positively
        // observed end of the enemy's shield. Missing observations do not rearm.
        *self = Self {
            failed_approach: self.failed_approach,
            ..Self::new()
        };
    }
    /// Updates the `rebase_custom_delivery` state using the supplied inputs.
    ///
    ///
///  Updates rebase custom delivery.
    pub fn rebase_custom_delivery(&mut self) {
        if !self.delivery_pending() {
            self.clock = NativeAttempt::default();
        }
    }
    /// Determines whether the `delivery_pending` condition holds.
    ///
    ///
///  Checks whether delivery pending.
    pub fn delivery_pending(&self) -> bool {
        self.active() && self.mapped > 0
    }
    /// Updates the `delivery_tick` state using the supplied inputs.
    ///
    ///
///  Updates delivery tick.
    pub fn delivery_tick(&mut self, tick: i64) {
        if self.approach_pending() && self.last_approach_poll != Some(tick) {
            self.approach_polls = self.approach_polls.saturating_add(1);
            self.last_approach_poll = Some(tick);
        }
        if self.active() && !self.approach_pending() && self.last_delivery_poll != Some(tick) {
            self.delivery_polls = self.delivery_polls.saturating_add(1);
            self.last_delivery_poll = Some(tick);
        }
    }
    /// Determines whether the `unshielded_grab_started` condition holds.
    ///
    ///
///  Checks whether unshielded grab started.
    pub fn unshielded_grab_started(&self) -> bool {
        self.active() && self.approach && self.dash_sent
    }
    /// Determines whether the `approach_pending` condition holds.
    ///
    ///
///  Checks whether approach pending.
    pub fn approach_pending(&self) -> bool {
        self.active() && self.approach && (!self.dash_sent || self.await_dash)
    }
    /// Determines whether the `approach_active` condition holds.
    ///
    ///
///  Checks whether approach active.
    pub fn approach_active(&self) -> bool {
        self.active() && self.approach
    }
    /// Determines whether the `turning_away` condition holds.
    ///
    ///
///  Checks whether turning away.
    pub fn turning_away(&self, physical: f32) -> bool {
        self.active()
            && self.approach
            && physical.is_finite()
            && physical.abs() >= 0.2
            && physical * self.direction < 0.
    }
    /// Observes the statuses.
    ///
    ///
///  Observes the statuses.
    pub fn observe_statuses<const N: usize>(&mut self, known: &[Option<(u32, i32)>; N]) {
        if self.failed_approach.is_some_and(|(id, _)| {
            known.iter().flatten().any(|(actor, status)| {
                *actor == id && !super::automatic_grab_target_status(*status)
            })
        }) {
            self.failed_approach = None;
        }
    }
    /// Updates the `fail_approach` state using the supplied inputs.
    ///
    ///
///  Updates fail approach.
    fn fail_approach(&mut self, epoch: u64) {
        let failed = self.target.map(|id| (id, epoch));
        self.cancel();
        self.failed_approach = failed;
    }
    /// An unanswered dash fences the target; an unanswered direct R just ends.
    ///
///  An unanswered dash fences the target; an unanswered direct R just ends.
    fn give_up(&mut self, epoch: u64) {
        if self.assisted_dash {
            self.fail_approach(epoch);
        } else {
            self.cancel();
        }
    }
    /// Updates the `release_unless_delivered` state using the supplied inputs.
    ///
    ///
///  Updates release unless delivered.
    fn release_unless_delivered(&mut self) {
        if !self.delivery_pending() {
            self.cancel();
        }
    }
    /// Updates the `mapped_approach` state using the supplied inputs.
    ///
    ///
///  Updates mapped approach.
    pub fn mapped_approach(&mut self, tick: i64, target: u32) {
        if self.target == Some(target)
            && self.approach
            && (!self.dash_sent || self.await_dash)
            && self.last_mapped != Some(tick)
        {
            if !self.dash_sent {
                self.clock = NativeAttempt::default();
            }
            self.dash_sent = true;
            self.approach_mapped = self.approach_mapped.saturating_add(1);
            self.last_mapped = Some(tick);
        }
    }
    /// Updates the `mapped_grab` state using the supplied inputs.
    ///
    ///
///  Updates mapped grab.
    pub fn mapped_grab(&mut self, tick: i64, target: u32) {
        if self.target == Some(target) && self.last_mapped != Some(tick) {
            self.mapped = self.mapped.saturating_add(1);
            self.last_mapped = Some(tick);
        }
    }
    /// Returns the `output` result.
    ///
    ///
///  Gets the output result.
    fn output(buttons: u32) -> ControllerOutput {
        ControllerOutput {
            owned: SEQUENCE_BUTTONS,
            requested: buttons,
            stick: Some([0.; 2]),
            ..Default::default()
        }
    }
    /// Updates the toggle state from the current button sample and match state.
    ///
    ///
///  Checks the current buttons and updates whether the toggle should activate.
    pub fn sample(
        &mut self,
        allowed: bool,
        c: super::Context,
        lc: GameplayContext,
        targets: [Option<Target>; 8],
        follow: Option<Opponent>,
        reach: Reach,
    ) -> ControllerOutput {
        if self
            .failed_approach
            .is_some_and(|(_, epoch)| epoch != lc.manual_input_epoch)
        {
            self.failed_approach = None;
        }
        if self.active()
            && !self.delivery_pending()
            && !self.approach_pending()
            && self.delivery_polls >= 4
        {
            self.give_up(lc.manual_input_epoch);
            return ControllerOutput::default();
        }
        // A mapped R may still be in the game's input queue. Once delivered,
        // release control instead of switching that unresolved request to Up+B.
        if self.delivery_pending() && self.settle_delivered(allowed, c, lc, follow) {
            return ControllerOutput::default();
        }
        if self.admission_lost(allowed, c, lc, reach.approach) {
            return ControllerOutput::default();
        }
        self.observe_dash(c, lc);
        let enemy = if self.active() {
            let valid = super::locked_enemy(self.target, follow, &targets)
                .filter(|e| target_valid(c, lc, *e, reach));
            let Some(enemy) = valid else {
                self.release_unless_delivered();
                return ControllerOutput::default();
            };
            enemy
        } else {
            let fenced = self.failed_approach.is_some()
                && reach.approach
                && c.status == crate::native_states::status::WAIT;
            let Some(enemy) = super::nearest_enemy(c.position[0], &targets, follow, |e| {
                target_valid(c, lc, e, reach)
                    && !(fenced && (e.position[0] - c.position[0]).abs() > reach.close)
            }) else {
                return ControllerOutput::default();
            };
            self.acquire(enemy, c, reach);
            enemy
        };
        if let Some(output) = self.approach_output(c, lc, enemy) {
            return output;
        }
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
    ///
///  True when this sample must emit nothing: the delivered Grab ended, expired or is still settling. Settling owns no buttons or movement; false means the request is still retryable within its bounded budget.
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
            self.give_up(lc.manual_input_epoch);
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
    ///
///  True when no grab may be sampled now. Native incapacitation, a renewed shield hit or catch startup ends the previous tap; never queue it through shieldstun or force a cancel of native recovery. A delivered R keeps its receipt.
    fn admission_lost(
        &mut self,
        allowed: bool,
        c: super::Context,
        lc: GameplayContext,
        approach: bool,
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
        // A new shield press cancels the approach; it can never launch a dash
        // from shield. Ground locomotion can also begin a fresh direct Grab.
        let own_ready = super::ground_grab_state(c.status)
            && !c.stopped
            && (self.approach_active() || c.can_grab && c.shield_ready);
        if self.active() && self.approach != approach || !own_ready {
            self.release_unless_delivered();
            return true;
        }
        false
    }
    /// Promote an approach whose dash is already visible in native status.
    ///
///  Promote an approach whose dash is already visible in native status.
    fn observe_dash(&mut self, c: super::Context, lc: GameplayContext) {
        if !self.active() {
            return;
        }
        if self.await_dash
            && matches!(c.status, crate::native_states::status::WALK..=crate::native_states::status::TURN_RUN_BRAKE | crate::native_states::status::SQUAT..=crate::native_states::status::LANDING_LIGHT)
        {
            self.await_dash = false;
            self.dash_sent = true;
            // The player may have begun walking before our dash was mapped.
            // Keep that movement manual instead of boosting it into a run.
            if !matches!(
                c.status,
                crate::native_states::status::DASH | crate::native_states::status::RUN
            ) {
                self.assisted_dash = false;
            }
            self.clock = NativeAttempt::default();
        }
        if self.approach
            && !self.dash_sent
            && !self.await_dash
            && !super::automatic_grab_needs_approach(c.status, lc.velocity[0])
        {
            // Native movement already exists; do not replay an unnecessary
            // approach pulse or wait for its mapper acknowledgment.
            self.dash_sent = true;
            self.clock = NativeAttempt::default();
        }
    }
    /// Updates the `acquire` state using the supplied inputs.
    ///
    ///
///  Updates acquire.
    fn acquire(&mut self, e: Opponent, c: super::Context, reach: Reach) {
        self.target = Some(e.id);
        self.clock = NativeAttempt::default();
        self.mapped = 0;
        self.last_mapped = None;
        self.approach = reach.approach;
        // A far idle grab must enter native dash before pressing R.
        // A mapped stick packet (or residual shield-drop speed) is not proof.
        self.await_dash = reach.approach
            && c.status == crate::native_states::status::WAIT
            && reach.close.is_finite()
            && (e.position[0] - c.position[0]).abs() > reach.close;
        self.assisted_dash = self.await_dash;
        self.approach_mapped = 0;
        // Close standing grabs and already moving grabs need no synthetic
        // run packet. Only the out-of-close-range idle path starts a dash.
        self.dash_sent = !self.await_dash;
        self.direction = (e.position[0] - c.position[0]).signum();
        if self.direction == 0. {
            self.direction = c.facing.signum();
        }
    }
    /// The approach stick while the dash is unsent or unconfirmed; `None` once
    /// the approach is over and the Grab itself may be pressed.
    ///
///  The approach stick while the dash is unsent or unconfirmed; None once the approach is over and the Grab itself may be pressed.
    fn approach_output(
        &mut self,
        c: super::Context,
        lc: GameplayContext,
        enemy: Opponent,
    ) -> Option<ControllerOutput> {
        if !self.approach_pending() {
            return None;
        }
        if self.await_dash {
            self.clock.advance_pause(lc, Some(enemy), 120);
            if self.approach_mapped >= 4
                || self.approach_polls >= 4
                || self.clock.expired
                || self.clock.elapsed() >= 4.
            {
                self.fail_approach(lc.manual_input_epoch);
                return Some(ControllerOutput::default());
            }
        }
        if matches!(
            c.status,
            crate::native_states::GUARD_ON..=crate::native_states::GUARD_DAMAGE
        ) || !lc.safe_ground[usize::from(self.direction > 0.)]
        {
            self.cancel();
            return Some(ControllerOutput::default());
        }
        Some(ControllerOutput {
            stick: Some([self.direction, 0.]),
            ..Self::output(0)
        })
    }
}
