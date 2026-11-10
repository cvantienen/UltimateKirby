//! Event-triggered, file-authored defensive responses. No native calls or scans.
use crate::{
    gameplay::{ControllerOutput, GameplayContext, Opponent},
    ledge_trap::InputTimeline,
    move_input::{finite_positions, UpBFlow, UpBVerdict},
    reaction_rules::{automatic_plan, Resolved, Trigger},
    sequence_config::Sequence,
    settings::AssistSettings,
};

/// Defines the `Response` data structure.
///
/// Holds the data for Response.
#[derive(Default)]
pub struct Response {
    /// Stores the `plan` value in `Response` as `Option<InputTimeline>`.
    ///
    ///
    ///  Optionally stores the plan for Response; it may be absent.
    plan: Option<InputTimeline>,
    /// Stores the `target` value in `Response` as `Option<u32>`.
    ///
    ///
    ///  Optionally stores the target for Response; it may be absent.
    target: Option<u32>,
    /// Stores the `up_b` value in `Response` as `UpBFlow`.
    ///
    ///
    ///  Holds the up b value used by Response.
    up_b: UpBFlow,
    /// Stores the `configured` value in `Response` as `bool`.
    ///
    ///
    ///  Holds the configured value used by Response.
    configured: bool,
    /// Stores the `grab_sent` value in `Response` as `bool`.
    ///
    ///
    ///  Holds the grab sent value used by Response.
    grab_sent: bool,
    /// Stores the `grab_settling` value in `Response` as `bool`.
    ///
    ///
    ///  Holds the grab settling value used by Response.
    grab_settling: bool,
    /// Stores the `grab_delivery` value in `Response` as `crate::move_input::DeliveryBudget`.
    ///
    ///
    ///  Holds the grab delivery value used by Response.
    grab_delivery: crate::move_input::DeliveryBudget,
}
impl Response {
    /// Constructs a new value with the type’s initial state.
    ///
    ///
    ///  Creates a new value.
    pub const fn new() -> Self {
        Self {
            plan: None,
            target: None,
            up_b: UpBFlow::new(),
            configured: false,
            grab_sent: false,
            grab_settling: false,
            grab_delivery: crate::move_input::DeliveryBudget::new(),
        }
    }
    /// Returns the selected target observation.
    ///
    ///
    ///  Gets the chosen target.
    pub fn target(&self) -> Option<u32> {
        self.target
    }
    /// Determines whether ground.
    ///
    ///
    ///  Checks whether ground.
    pub fn needs_ground(&self) -> bool {
        self.plan.as_ref().is_some_and(InputTimeline::needs_ground)
    }
    /// Determines whether the `attack_hold` condition holds.
    ///
    ///
    ///  Checks whether attack hold.
    pub fn attack_hold(&self) -> bool {
        self.plan.as_ref().is_some_and(InputTimeline::attack_hold)
    }
    /// Returns an optional `right_stick` value.
    ///
    ///
    ///  Gets right stick if it is available.
    pub fn right_stick(&self) -> Option<[f32; 2]> {
        self.plan.as_ref().map(InputTimeline::right_stick)
    }
    /// Determines whether the `walking` condition holds.
    ///
    ///
    ///  Checks whether walking.
    pub fn walking(&self) -> bool {
        self.plan.as_ref().is_some_and(InputTimeline::walking)
    }
    /// Determines whether the `automatic_up_b` condition holds.
    ///
    ///
    ///  Checks whether automatic up b.
    pub fn automatic_up_b(&self) -> bool {
        self.up_b.armed() && self.plan.is_some()
    }
    /// Determines whether the `up_b_delivery_started` condition holds.
    ///
    ///
    ///  Checks whether up b delivery started.
    pub fn up_b_delivery_started(&self) -> bool {
        self.automatic_up_b() && self.up_b.started()
    }
    /// Updates the `attempted_up_special` state using the supplied inputs.
    ///
    ///
    ///  Updates attempted up special.
    pub fn attempted_up_special(&mut self, tick: i64, target: u32) {
        if self.automatic_up_b() && self.target == Some(target) {
            self.up_b.attempted(tick);
        }
    }
    /// Updates the `mapped_up_special` state using the supplied inputs.
    ///
    ///
    ///  Updates mapped up special.
    pub fn mapped_up_special(&mut self, tick: i64, target: u32) {
        if self.automatic_up_b() && self.target == Some(target) {
            self.up_b.mapped(tick);
        }
    }
    /// Determines whether the `configured_committed` condition holds.
    ///
    ///
    ///  Checks whether configured committed.
    pub fn configured_committed(&self) -> bool {
        self.configured && self.target.is_some()
    }
    /// Determines whether the `grab_delivery_pending` condition holds.
    ///
    ///
    ///  Checks whether grab delivery pending.
    pub fn grab_delivery_pending(&self) -> bool {
        self.grab_sent && self.target.is_some()
    }
    /// Updates the `mapped_grab` state using the supplied inputs.
    ///
    ///
    ///  Updates mapped grab.
    pub fn mapped_grab(&mut self, tick: i64, target: u32) {
        if self.configured && self.target == Some(target) {
            self.grab_sent = true;
            self.grab_delivery.mapped(tick);
        }
    }
    /// Determines whether the `up_b_receipt_pending` condition holds.
    ///
    ///
    ///  Checks whether up b receipt pending.
    pub fn up_b_receipt_pending(&self) -> bool {
        self.automatic_up_b() && self.up_b.receipt_pending()
    }
    /// Determines whether the `awaiting_up_b` condition holds.
    ///
    ///
    ///  Checks whether awaiting up b.
    pub fn awaiting_up_b(&self) -> bool {
        self.automatic_up_b() && self.up_b.awaiting()
    }
    /// Updates the `delivery_tick` state using the supplied inputs.
    ///
    ///
    ///  Updates delivery tick.
    pub fn delivery_tick(&mut self, tick: i64) {
        if self.grab_sent {
            self.grab_delivery.mapped(tick);
        }
        if self.automatic_up_b() {
            self.up_b.tick(tick);
        }
    }
    /// Stop playback without forgetting a controller Grab still in flight.
    ///
    ///  Stop playback without forgetting a controller Grab still in flight.
    pub fn suspend(&mut self) {
        if self.grab_sent {
            self.plan = None;
            self.grab_settling = true;
            self.up_b.disarm();
        } else {
            self.cancel();
        }
    }
    /// Cancels the pending operation and clears its state.
    ///
    ///
    ///  Stops the operation that was waiting.
    pub fn cancel(&mut self) {
        *self = Self::new();
    }
    /// Common admission for an explicitly configured automatic response: native
    /// control now, an available target and no grab already in progress.
    ///
    ///  Common admission for an explicitly configured automatic response: native control now, an available target and no grab already in progress.
    fn can_begin(c: GameplayContext, enemy: Opponent) -> bool {
        c.gameplay
            && c.situation == crate::native_states::situation::GROUND
            && !c.stopped
            && c.can_act
            && (crate::reaction_rules::automatic_start_status(c.status)
                || c.status == crate::native_states::ESCAPE)
            && crate::reaction_rules::automatic_target_available(enemy.status)
            && !crate::action_owner::native_grab_active(c.status)
    }
    /// Starts the up b.
    ///
    ///
    ///  Starts the up b.
    pub fn start_up_b(
        &mut self,
        mut sequence: Sequence,
        c: GameplayContext,
        enemy: Opponent,
    ) -> bool {
        if !Self::can_begin(c, enemy)
            || sequence.start_delay != 0
            || !crate::reaction_rules::starts_direct_up_b(&sequence)
        {
            return false;
        }
        self.cancel();
        sequence.while_held = false;
        sequence.ground_only = false;
        self.plan = Some(automatic_plan(sequence, enemy));
        self.target = Some(enemy.id);
        self.up_b.arm(c, true);
        self.configured = true;
        true
    }
    /// Commit an explicitly configured automatic response. Native Up-B keeps
    /// its confirmation/air/manual safeguards; other recipes use the same tape.
    ///
    ///  Commit an explicitly configured automatic response. Native Up-B keeps its confirmation/air/manual safeguards; other recipes use the same tape.
    pub fn start_sequence(
        &mut self,
        mut sequence: Sequence,
        c: GameplayContext,
        enemy: Opponent,
    ) -> bool {
        if crate::reaction_rules::starts_direct_up_b(&sequence) && sequence.start_delay == 0 {
            return self.start_up_b(sequence, c, enemy);
        }
        if !Self::can_begin(c, enemy) || crate::ledge_trap::sequence_interrupted(c.status) {
            return false;
        }
        self.cancel();
        sequence.while_held = false;
        self.plan = Some(automatic_plan(sequence, enemy));
        self.target = Some(enemy.id);
        self.configured = true;
        true
    }
    /// Determines whether the `start` condition holds.
    ///
    ///
    ///  Checks whether start.
    pub fn start(
        &mut self,
        rules: &Resolved,
        trigger: Trigger,
        c: GameplayContext,
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
    /// The locked target stays usable: same actor, measurable and either an
    /// ordinary target or, for an existing tape, a ledge-invulnerable one.
    ///
    ///  The locked target stays usable: same actor, measurable and either an ordinary target or, for an existing tape, a ledge-invulnerable one.
    fn follows(&self, e: Opponent, c: GameplayContext) -> bool {
        Some(e.id) == self.target
            && finite_positions(e.position, c.position)
            && (crate::reaction_rules::automatic_target_available(e.status)
                || self.plan.is_some()
                    && crate::reaction_rules::automatic_follow_target_available(e.status))
    }
    /// Updates the toggle state from the current button sample and match state.
    ///
    ///
    ///  Checks the current buttons and updates whether the toggle should activate.
    pub fn sample(
        &mut self,
        allowed: bool,
        settings: AssistSettings,
        mut c: GameplayContext,
        enemy: Option<Opponent>,
    ) -> ControllerOutput {
        if self.grab_sent {
            if !c.gameplay
                || c.situation != crate::native_states::situation::GROUND
                || crate::action_owner::native_grab_active(c.status)
                || crate::ledge_trap::sequence_interrupted(c.status)
                || self.grab_delivery.exhausted()
            {
                self.cancel();
                return ControllerOutput::default();
            }
            if self.grab_settling {
                return ControllerOutput::default();
            }
        }
        if !allowed {
            self.suspend();
            return ControllerOutput::default();
        }
        if self.configured && enemy.is_none_or(|e| !self.follows(e, c)) {
            self.suspend();
            return ControllerOutput::default();
        }
        if self.automatic_up_b() {
            match self.up_b.advance(c, settings.fighter_kind) {
                UpBVerdict::Cancel => {
                    self.cancel();
                    return ControllerOutput::default();
                }
                UpBVerdict::Wait => return ControllerOutput::default(),
                UpBVerdict::Confirm => {
                    if let (Some(plan), Some(enemy)) = (
                        self.plan.as_mut(),
                        enemy.filter(|e| Some(e.id) == self.target),
                    ) {
                        plan.confirm_initial_up_b(c, enemy);
                    }
                }
                UpBVerdict::Continue => {}
            }
        }
        let Some(plan) = self.plan.as_mut() else {
            return ControllerOutput::default();
        };
        let enemy = enemy.filter(|e| Some(e.id) == self.target);
        if !plan.inputs_only() && enemy.is_none() {
            self.cancel();
            return ControllerOutput::default();
        }
        let manual = self.up_b.manual_handback(c);
        if manual && plan.drifting() {
            self.cancel();
            return ControllerOutput::default();
        }
        c.held = true;
        let output = plan.sample(settings, c, enemy, None, 0.0);
        let output = if self.up_b.armed() || self.configured {
            crate::out_of_shield::direct_shield_input(c.status, output, plan.right_stick())
        } else {
            output
        };
        match self
            .up_b
            .after_sample(manual && plan.drifting(), output.requested)
        {
            UpBVerdict::Wait => return ControllerOutput::default(),
            UpBVerdict::Cancel => {
                self.cancel();
                return ControllerOutput::default();
            }
            UpBVerdict::Continue | UpBVerdict::Confirm => {}
        }
        if self.grab_sent && output.requested & crate::native_input::MAPPER_R == 0 {
            self.plan = None;
            self.grab_settling = true;
            return ControllerOutput::default();
        }
        if plan.finished() {
            self.cancel();
        }
        output
    }
}

/// Shared event/range selection; recipes are copied only when an event matches.
///
///  Shared event/range selection; recipes are copied only when an event matches.
pub fn select(
    rules: &Resolved,
    trigger: Trigger,
    c: GameplayContext,
    enemy: Opponent,
) -> Option<Sequence> {
    if !trigger.defensive() || c.stopped || !c.gameplay
        // Retained text token is inert for compatibility with older files.
        || trigger == Trigger::LedgeRegrab
        || !finite_positions(c.position, enemy.position)
        || !c.facing.is_finite() || c.facing.abs() < 0.5
    {
        return None;
    }
    rules.rules.iter().flatten().find_map(|candidate| {
        let r = candidate.sequence.reaction;
        if !r.trigger.event_matches(trigger)
            || r.trigger.needs_vulnerable() && !enemy.vulnerable
            || r.rejects(c, enemy.position).is_some()
        {
            return None;
        }
        let mut sequence = candidate.sequence;
        sequence.while_held = false;
        Some(sequence)
    })
}
