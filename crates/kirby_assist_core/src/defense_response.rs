//! Event-triggered, file-authored defensive responses. No native calls or scans.
use crate::{
    ledge_trap::{InputTimeline, LedgeContext, LedgeOutput, LedgeSettings, Opponent},
    reaction_rules::{Resolved, Side, Trigger},
};

#[derive(Default)]
pub struct Response {
    plan: Option<InputTimeline>,
    target: Option<u32>,
    automatic_up_b: bool,
    configured: bool,
    up_b_seen: bool,
    manual_epoch: u64,
    delivery: crate::move_input::DeliveryBudget,
    receipt: crate::move_input::UpBReceipt,
    waiting_up_b: bool,
    up_b_from_dodge: bool,
    grab_sent: bool,
    grab_settling: bool,
    grab_delivery: crate::move_input::DeliveryBudget,
}
impl Response {
    pub const fn new() -> Self {
        Self {
            plan: None,
            target: None,
            automatic_up_b: false,
            configured: false,
            up_b_seen: false,
            manual_epoch: 0,
            delivery: crate::move_input::DeliveryBudget::new(),
            receipt: crate::move_input::UpBReceipt::new(),
            waiting_up_b: false,
            up_b_from_dodge: false,
            grab_sent: false,
            grab_settling: false,
            grab_delivery: crate::move_input::DeliveryBudget::new(),
        }
    }
    pub fn target(&self) -> Option<u32> {
        self.target
    }
    pub fn needs_ground(&self) -> bool {
        self.plan.as_ref().is_some_and(InputTimeline::needs_ground)
    }
    pub fn attack_hold(&self) -> bool {
        self.plan.as_ref().is_some_and(InputTimeline::attack_hold)
    }
    pub fn right_stick(&self) -> Option<[f32; 2]> {
        self.plan.as_ref().map(InputTimeline::right_stick)
    }
    pub fn walking(&self) -> bool {
        self.plan.as_ref().is_some_and(InputTimeline::walking)
    }
    pub fn automatic_up_b(&self) -> bool {
        self.automatic_up_b && self.plan.is_some()
    }
    pub fn up_b_delivery_started(&self) -> bool {
        self.automatic_up_b() && (self.up_b_seen || self.receipt.sent())
    }
    pub fn attempted_up_special(&mut self, tick: i64, target: u32) {
        if self.automatic_up_b() && !self.up_b_seen && self.target == Some(target) {
            self.delivery.mapped(tick);
        }
    }
    pub fn mapped_up_special(&mut self, tick: i64, target: u32) {
        if self.automatic_up_b() && !self.up_b_seen && self.target == Some(target) {
            self.delivery.mapped(tick);
            self.receipt.mapped(tick);
        }
    }
    pub fn configured_committed(&self) -> bool {
        (self.configured || self.automatic_up_b) && self.target.is_some()
    }
    pub fn grab_delivery_pending(&self) -> bool {
        self.grab_sent && self.target.is_some()
    }
    pub fn mapped_grab(&mut self, tick: i64, target: u32) {
        if self.configured && self.target == Some(target) {
            self.grab_sent = true;
            self.grab_delivery.mapped(tick);
        }
    }
    pub fn up_b_receipt_pending(&self) -> bool {
        self.automatic_up_b() && !self.up_b_seen && self.receipt.sent()
    }
    pub fn awaiting_up_b(&self) -> bool {
        self.up_b_receipt_pending() && self.waiting_up_b
    }
    pub fn delivery_tick(&mut self, tick: i64) {
        if self.grab_sent {
            self.grab_delivery.mapped(tick);
        }
        if self.up_b_receipt_pending() {
            self.receipt.tick(tick);
        }
    }
    /// Stop playback without forgetting a controller Grab still in flight.
    pub fn suspend(&mut self) {
        if self.grab_sent {
            self.plan = None;
            self.grab_settling = true;
            self.automatic_up_b = false;
        } else {
            self.cancel();
        }
    }
    pub fn cancel(&mut self) {
        self.plan = None;
        self.target = None;
        self.automatic_up_b = false;
        self.configured = false;
        self.up_b_seen = false;
        self.receipt = Default::default();
        self.waiting_up_b = false;
        self.up_b_from_dodge = false;
        self.delivery = Default::default();
        self.grab_sent = false;
        self.grab_settling = false;
        self.grab_delivery = Default::default();
    }
    pub fn start_up_b(
        &mut self,
        mut sequence: crate::sequence_config::Sequence,
        c: LedgeContext,
        enemy: Opponent,
    ) -> bool {
        if !c.gameplay
            || c.situation != 0
            || c.stopped
            || !c.can_act
            || !(crate::reaction_rules::automatic_start_status(c.status) || c.status == 0x1f)
            || !crate::reaction_rules::automatic_target_available(enemy.status)
            || matches!(c.status, 0x37..=0x40)
            || sequence.start_delay != 0
            || !crate::reaction_rules::starts_direct_up_b(&sequence)
        {
            return false;
        }
        self.cancel();
        sequence.while_held = false;
        sequence.ground_only = false;
        let mut plan = InputTimeline::new(sequence, enemy);
        plan.allow_continuous_progress();
        plan.guard_automatic_ground_actions();
        self.plan = Some(plan);
        self.target = Some(enemy.id);
        self.automatic_up_b = true;
        self.configured = true;
        self.up_b_from_dodge = c.status == 0x1f;
        self.manual_epoch = c.automatic_up_b_epoch.unwrap_or(c.manual_input_epoch);
        true
    }
    /// Commit an explicitly configured automatic response. Native Up-B keeps
    /// its confirmation/air/manual safeguards; other recipes use the same tape.
    pub fn start_sequence(
        &mut self,
        mut sequence: crate::sequence_config::Sequence,
        c: LedgeContext,
        enemy: Opponent,
    ) -> bool {
        if crate::reaction_rules::starts_direct_up_b(&sequence) && sequence.start_delay == 0 {
            return self.start_up_b(sequence, c, enemy);
        }
        if !c.gameplay
            || c.situation != 0
            || c.stopped
            || !c.can_act
            || !(crate::reaction_rules::automatic_start_status(c.status) || c.status == 0x1f)
            || !crate::reaction_rules::automatic_target_available(enemy.status)
            || crate::action_owner::native_grab_active(c.status)
            || crate::ledge_trap::sequence_interrupted(c.status)
        {
            return false;
        }
        self.cancel();
        sequence.while_held = false;
        let mut plan = InputTimeline::new(sequence, enemy);
        plan.allow_continuous_progress();
        plan.guard_automatic_ground_actions();
        self.plan = Some(plan);
        self.target = Some(enemy.id);
        self.manual_epoch = c.manual_input_epoch;
        self.configured = true;
        true
    }
    pub fn start(
        &mut self,
        rules: &Resolved,
        trigger: Trigger,
        c: LedgeContext,
        enemy: Opponent,
    ) -> bool {
        let Some(sequence) = select(rules, trigger, c, enemy) else {
            return false;
        };
        self.cancel();
        let mut plan = InputTimeline::new(sequence, enemy);
        plan.guard_automatic_ground_actions();
        self.plan = Some(plan);
        self.target = Some(enemy.id);
        true
    }
    pub fn sample(
        &mut self,
        allowed: bool,
        settings: LedgeSettings,
        mut c: LedgeContext,
        enemy: Option<Opponent>,
    ) -> LedgeOutput {
        if self.grab_sent {
            if !c.gameplay
                || c.situation != 0
                || crate::action_owner::native_grab_active(c.status)
                || crate::ledge_trap::sequence_interrupted(c.status)
                || self.grab_delivery.exhausted()
            {
                self.cancel();
                return LedgeOutput::default();
            }
            if self.grab_settling {
                return LedgeOutput::default();
            }
        }
        if !allowed {
            self.suspend();
            return LedgeOutput::default();
        }
        let tracking = settings.fighter_kind == 15
            && self.automatic_up_b
            && (self.up_b_seen
                || crate::reaction_rules::native_up_b_started(settings.fighter_kind, c)
                || self.receipt.sent());
        if self.configured
            && enemy.is_none_or(|e| {
                Some(e.id) != self.target
                    || !e
                        .position
                        .iter()
                        .chain(c.position.iter())
                        .all(|v| v.is_finite())
                    || !(crate::reaction_rules::automatic_target_available(e.status)
                        || tracking
                            && crate::reaction_rules::automatic_tracking_target_available(e.status))
            })
        {
            self.suspend();
            return LedgeOutput::default();
        }
        if self.automatic_up_b && self.plan.is_some() {
            let native = crate::reaction_rules::native_up_b_started(settings.fighter_kind, c);
            if settings.fighter_kind == 15 && !self.up_b_seen && self.receipt.sent() {
                if native && self.waiting_up_b {
                    if let Some(enemy) = enemy.filter(|e| Some(e.id) == self.target) {
                        self.plan.as_mut().unwrap().confirm_initial_up_b(c, enemy);
                    }
                } else if !native {
                    if !crate::reaction_rules::automatic_up_b_receipt_status(
                        c.status,
                        c.situation,
                        self.up_b_from_dodge,
                    ) || self.receipt.expired()
                    {
                        self.cancel();
                        return LedgeOutput::default();
                    }
                    if self.waiting_up_b || c.stopped || !c.can_act || self.delivery.exhausted() {
                        self.waiting_up_b = true;
                        return LedgeOutput::default();
                    }
                }
            }
            self.up_b_seen |= native;
            let wrong = matches!(c.status,0xb..=0xf|0x20..=0x23|0x37..=0x40|0x1dc|0x1dd|0x1df);
            let wrong = wrong || c.status == 0x1f && (!c.can_act || self.up_b_seen);
            let ended = self.up_b_seen && matches!(c.status,0..=9|0x10|0x16..=0x1e);
            if wrong
                || ended
                || self.up_b_seen && !native
                || !self.up_b_seen
                    && (c.situation != 0
                        || c.stopped
                        || !c.can_act
                        || !(crate::reaction_rules::automatic_start_status(c.status)
                            || c.status == 0x1f)
                        || self.delivery.exhausted())
            {
                self.cancel();
                return LedgeOutput::default();
            }
        }
        let Some(plan) = self.plan.as_mut() else {
            return LedgeOutput::default();
        };
        let enemy = enemy.filter(|e| Some(e.id) == self.target);
        if !plan.inputs_only() && enemy.is_none() {
            self.cancel();
            return LedgeOutput::default();
        }
        let manual = settings.fighter_kind != 15
            && self.automatic_up_b
            && self.up_b_seen
            && c.automatic_up_b_epoch.unwrap_or(c.manual_input_epoch) != self.manual_epoch;
        if manual && plan.drifting() {
            self.cancel();
            return LedgeOutput::default();
        }
        c.held = true;
        let output = plan.sample(settings, c, enemy, None, 0.0);
        let output = if self.automatic_up_b || self.configured {
            crate::out_of_shield::direct_shield_input(c.status, output, plan.right_stick())
        } else {
            output
        };
        if self.automatic_up_b
            && (manual && plan.drifting()
                || !self.up_b_seen && output.requested & crate::native_input::MAPPER_B == 0)
        {
            if settings.fighter_kind == 15 && !self.up_b_seen && self.receipt.sent() {
                self.waiting_up_b = true;
                return LedgeOutput::default();
            }
            self.cancel();
            return LedgeOutput::default();
        }
        if self.grab_sent && output.requested & crate::native_input::MAPPER_R == 0 {
            self.plan = None;
            self.grab_settling = true;
            return LedgeOutput::default();
        }
        if plan.finished() {
            self.cancel();
        }
        output
    }
}

/// Shared event/range selection; recipes are copied only when an event matches.
pub fn select(
    rules: &Resolved,
    trigger: Trigger,
    c: LedgeContext,
    enemy: Opponent,
) -> Option<crate::sequence_config::Sequence> {
    if !trigger.defensive() || c.stopped || !c.gameplay
        // Retained text token is inert for compatibility with older files.
        || trigger == Trigger::LedgeRegrab
        || !c.position.iter().chain(enemy.position.iter()).all(|v| v.is_finite())
        || !c.facing.is_finite() || c.facing.abs() < 0.5
    {
        return None;
    }
    for candidate in rules.rules.iter().flatten() {
        let r = candidate.sequence.reaction;
        let dx = enemy.position[0] - c.position[0];
        if !r.trigger.event_matches(trigger)
            || r.trigger.needs_vulnerable() && !enemy.vulnerable
            || !r.state.matches(c)
            || dx.abs() < r.min
            || dx.abs() > r.max
            || (enemy.position[1] - c.position[1]).abs() > r.height
            || matches!(r.side, Side::Front) && dx * c.facing <= 0.001
            || matches!(r.side, Side::Behind) && dx * c.facing >= -0.001
        {
            continue;
        }
        let mut sequence = candidate.sequence;
        sequence.while_held = false;
        return Some(sequence);
    }
    None
}
