//! Front vulnerable-target selection: enemy shield required only when unshielded.
use crate::{
    gameplay::{ControllerOutput, GameplayContext, Opponent},
    ledge_trap::{NativeAttempt, SEQUENCE_BUTTONS},
    native_input::MAPPER_R,
    roll_read::Target,
};
pub(super) fn target_valid(
    c: super::Context,
    lc: GameplayContext,
    e: Opponent,
    range: f32,
    height: f32,
    startup: Option<f32>,
    approach: bool,
    shielded: bool,
) -> bool {
    e.vulnerable
        && super::automatic_grab_target_status_with_shield(e.status, shielded)
        && range.is_finite()
        && range > 0.
        && c.facing.is_finite()
        && c.facing.abs() >= 0.5
        && e.position
            .iter()
            .chain(c.position.iter())
            .all(|v| v.is_finite())
        && (e.position[0] - c.position[0]) * c.facing > 0.
        && (e.position[0] - c.position[0]).abs() <= range
        && height.is_finite()
        && height >= 0.
        && (e.position[1] - c.position[1]).abs() <= height
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
#[derive(Default)]
pub(super) struct Grab {
    target: Option<u32>,
    clock: NativeAttempt,
    mapped: u8,
    last_mapped: Option<i64>,
    approach: bool,
    dash_sent: bool,
    direction: f32,
    await_dash: bool,
    assisted_dash: bool,
    approach_mapped: u8,
    delivery_polls: u8,
    last_delivery_poll: Option<i64>,
    approach_polls: u8,
    last_approach_poll: Option<i64>,
    failed_approach: Option<(u32, u64)>,
}
impl Grab {
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
    pub fn target(&self) -> Option<u32> {
        self.target
    }
    pub fn active(&self) -> bool {
        self.target.is_some()
    }
    pub fn cancel(&mut self) {
        self.target = None;
        self.clock = NativeAttempt::default();
        self.mapped = 0;
        self.last_mapped = None;
        self.approach = false;
        self.dash_sent = false;
        self.direction = 0.;
        self.await_dash = false;
        self.assisted_dash = false;
        self.approach_mapped = 0;
        self.delivery_polls = 0;
        self.last_delivery_poll = None;
        self.approach_polls = 0;
        self.last_approach_poll = None;
        // Failure remains fenced until new manual intent or a positively
        // observed end of the enemy's shield. Missing observations do not rearm.
    }
    pub fn rebase_custom_delivery(&mut self) {
        if !self.delivery_pending() {
            self.clock = NativeAttempt::default();
        }
    }
    pub fn delivery_pending(&self) -> bool {
        self.active() && self.mapped > 0
    }
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
    pub fn preserve_movement(&self) -> bool {
        self.active() && self.approach && self.dash_sent
    }
    pub fn approach_pending(&self) -> bool {
        self.active() && self.approach && (!self.dash_sent || self.await_dash)
    }
    pub fn delivery_x(&self, physical: f32) -> f32 {
        if self.assisted_dash
            && self.mapped == 0
            && self.delivery_polls == 0
            && physical.abs() < 0.1
        {
            self.direction
        } else {
            physical
        }
    }
    pub fn turning_away(&self, physical: f32) -> bool {
        self.active()
            && self.approach
            && physical.is_finite()
            && physical.abs() >= 0.2
            && physical * self.direction < 0.
    }
    pub fn observe_statuses<const N: usize>(&mut self, known: &[Option<(u32, i32)>; N]) {
        if self.failed_approach.is_some_and(|(id, _)| {
            known.iter().flatten().any(|(actor, status)| {
                *actor == id && !super::automatic_grab_target_status(*status)
            })
        }) {
            self.failed_approach = None;
        }
    }
    fn fail_approach(&mut self, epoch: u64) {
        let failed = self.target.map(|id| (id, epoch));
        self.cancel();
        self.failed_approach = failed;
    }
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
    pub fn mapped_grab(&mut self, tick: i64, target: u32) {
        if self.target == Some(target) && self.last_mapped != Some(tick) {
            self.mapped = self.mapped.saturating_add(1);
            self.last_mapped = Some(tick);
        }
    }
    fn output(buttons: u32) -> ControllerOutput {
        ControllerOutput {
            owned: SEQUENCE_BUTTONS,
            requested: buttons,
            stick: Some([0.; 2]),
            ..Default::default()
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
        startup: Option<f32>,
        close: f32,
        approach: bool,
        shielded: bool,
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
            if self.assisted_dash {
                self.fail_approach(lc.manual_input_epoch);
            } else {
                self.cancel();
            }
            return ControllerOutput::default();
        }
        // A mapped R may still be in the game's input queue. Once delivered,
        // release control instead of switching that unresolved request to Up+B.
        if self.delivery_pending() {
            if matches!(
                c.status,
                crate::native_states::CATCH..=crate::native_states::THROW
            ) || !lc.gameplay
                || !c.grounded
                || lc.situation != 0
                || crate::ledge_trap::sequence_interrupted(c.status)
            {
                self.cancel();
                return ControllerOutput::default();
            }
            self.clock.advance_pause(lc, follow, 120);
            if self.delivery_polls >= 24 || self.clock.expired || self.clock.elapsed() >= 24. {
                if self.assisted_dash {
                    self.fail_approach(lc.manual_input_epoch);
                } else {
                    self.cancel();
                }
                return ControllerOutput::default();
            }
            if !allowed
                || c.stopped
                || lc.stopped
                || c.status == crate::native_states::GUARD_DAMAGE
                || self.mapped >= 4
                || self.clock.elapsed() >= 4.
            {
                return ControllerOutput::default(); // Settling owns no buttons or movement.
            }
        }
        if !allowed || !c.grounded || lc.situation != 0 || !lc.gameplay || lc.stopped {
            self.cancel();
            return ControllerOutput::default();
        }
        // A renewed shield hit or native incapacitation ends the previous tap;
        // never queue it through shieldstun or force a cancel of native recovery.
        if self.active()
            && (crate::ledge_trap::sequence_interrupted(c.status)
                || c.status == crate::native_states::GUARD_DAMAGE)
        {
            self.cancel();
            return ControllerOutput::default();
        }
        // Native catch startup confirms delivery; release the input immediately.
        if self.active()
            && matches!(
                c.status,
                crate::native_states::CATCH..=crate::native_states::THROW
            )
        {
            self.cancel();
            return ControllerOutput::default();
        }
        // A new shield press cancels the approach; it can never launch a dash
        // from shield. Ground locomotion can also begin a fresh direct Grab.
        if self.active() && self.approach != approach {
            if !self.delivery_pending() {
                self.cancel();
            }
            return ControllerOutput::default();
        }
        let own_ready = if self.active() && self.approach {
            super::ground_grab_state(c.status) && !c.stopped
        } else {
            !c.stopped && c.can_grab && c.shield_ready && super::ground_grab_state(c.status)
        };
        if !own_ready {
            if !self.delivery_pending() {
                self.cancel();
            }
            return ControllerOutput::default();
        }
        if self.active()
            && self.await_dash
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
        if self.active()
            && self.approach
            && !self.dash_sent
            && !self.await_dash
            && !super::automatic_grab_needs_approach(c.status, lc.velocity[0])
        {
            // Native movement already exists; do not replay an unnecessary
            // approach pulse or wait for its mapper acknowledgment.
            self.dash_sent = true;
            self.clock = NativeAttempt::default();
        }
        let valid =
            |e: Opponent| target_valid(c, lc, e, range, height, startup, approach, shielded);
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
        let enemy = if self.active() {
            follow.filter(|e| Some(e.id) == self.target).or_else(|| {
                targets
                    .iter()
                    .flatten()
                    .find(|t| Some(t.id) == self.target)
                    .map(copied)
            })
        } else {
            if c.stopped || !c.can_grab || !c.shield_ready {
                return ControllerOutput::default();
            }
            targets
                .iter()
                .flatten()
                .map(copied)
                .chain(follow)
                .filter(|e| {
                    valid(*e)
                        && !(self.failed_approach.is_some()
                            && approach
                            && c.status == crate::native_states::status::WAIT
                            && (e.position[0] - c.position[0]).abs() > close)
                })
                .min_by(|a, b| {
                    (a.position[0] - c.position[0])
                        .abs()
                        .total_cmp(&(b.position[0] - c.position[0]).abs())
                })
        };
        if self.active() && enemy.is_none_or(|e| !valid(e)) {
            if !self.delivery_pending() {
                self.cancel();
            }
            return ControllerOutput::default();
        }
        if !self.active() {
            let Some(e) = enemy else {
                return ControllerOutput::default();
            };
            self.target = Some(e.id);
            self.clock = NativeAttempt::default();
            self.mapped = 0;
            self.last_mapped = None;
            self.approach = approach;
            // A far idle grab must enter native dash before pressing R.
            // A mapped stick packet (or residual shield-drop speed) is not proof.
            self.await_dash = approach
                && c.status == crate::native_states::status::WAIT
                && close.is_finite()
                && (e.position[0] - c.position[0]).abs() > close;
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
        if self.approach && (!self.dash_sent || self.await_dash) {
            if self.await_dash {
                self.clock.advance_pause(lc, enemy, 120);
                if self.approach_mapped >= 4
                    || self.approach_polls >= 4
                    || self.clock.expired
                    || self.clock.elapsed() >= 4.
                {
                    self.fail_approach(lc.manual_input_epoch);
                    return ControllerOutput::default();
                }
            }
            if matches!(
                c.status,
                crate::native_states::GUARD_ON..=crate::native_states::GUARD_DAMAGE
            ) || !lc.safe_ground[usize::from(self.direction > 0.)]
            {
                self.cancel();
                return ControllerOutput::default();
            }
            return ControllerOutput {
                stick: Some([self.direction, 0.]),
                ..Self::output(0)
            };
        }
        if self.mapped >= 4 {
            self.cancel();
            return ControllerOutput::default();
        }
        self.clock.advance_pause(lc, enemy, 120);
        if self.clock.expired || self.clock.elapsed() >= 4. {
            self.cancel();
            return ControllerOutput::default();
        }
        super::direct_shield_input(c.status, Self::output(MAPPER_R), [0.; 2])
    }
}
