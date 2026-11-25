//! Held universal defense and event-triggered defensive responses. No native calls or scans.
use crate::{
    gameplay::{ControllerOutput, GameplayContext, Opponent},
    ledge_trap::InputTimeline,
    move_input::{finite_positions, UpBFlow, UpBVerdict},
    reaction_rules::{automatic_plan, Resolved, Trigger},
    sequence_config::Sequence,
    settings::AssistSettings,
};

/// Native eligibility is read afresh; this policy never changes fighter state.
#[derive(Clone, Copy, Debug, Default)]
pub struct Context {
    pub object: u32,
    pub situation: i32,
    pub allowed: bool,
    pub stopped: bool,
    pub guard_ready: bool,
    pub air_dodge_ready: bool,
    pub air_dodging: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contact {
    pub source: u32,
    pub in_frames: f32,
    pub projectile: bool,
}

pub fn native_allowed(kind: i32, status: i32, situation: i32, captured: bool, allowed: bool) -> bool {
    allowed && !captured
        && matches!(situation, crate::native_states::situation::GROUND | crate::native_states::situation::AIR)
        && !crate::action_owner::native_grab_active(status)
        && !crate::native_states::fighter_sequence_interrupted(kind, status)
}

/// Shared bounded lookahead for the native scan and held policy.
pub fn defense_horizon(lead_frames: f32, input_delay: f32) -> Option<f32> {
    if !(0.0..=30.0).contains(&lead_frames) || !(0.0..=120.0).contains(&input_delay) {
        return None;
    }
    Some((lead_frames + input_delay).min(40.0))
}

/// Scale a root-relative body estimate without treating size changes as immunity.
pub fn scaled_body(
    mut body: crate::auto_counter::Config,
    scale: f32,
) -> Option<crate::auto_counter::Config> {
    if !(0.01..=10.0).contains(&scale) {
        return None;
    }
    body.half_width *= scale;
    body.height *= scale;
    body.half_depth *= scale;
    [body.half_width, body.height, body.half_depth]
        .into_iter()
        .all(|value| value.is_finite() && value > 0.0)
        .then_some(body)
}

#[derive(Default)]
pub struct Defense {
    object: Option<u32>,
    pending_since: Option<i64>,
    delivered: bool,
    air_used: bool,
    unavailable_seen: bool,
}
impl Defense {
    pub const fn new() -> Self {
        Self { object: None, pending_since: None, delivered: false,
            air_used: false, unavailable_seen: false }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn air_pending(&self) -> bool {
        self.pending_since.is_some()
    }

    /// Call only after the selected air-Guard packet actually reaches the mapper.
    /// The mapper must clear its cached requested bits after acknowledging it.
    pub fn mapped_air_dodge(&mut self) {
        if self.pending_since.is_some() {
            self.delivered = true;
        }
    }

    pub fn sample(
        &mut self,
        tick: i64,
        held: bool,
        c: Context,
        contact: Option<Contact>,
        lead_frames: f32,
        input_delay: f32,
    ) -> ControllerOutput {
        use crate::native_states::situation::{AIR, GROUND};
        if !held || !c.allowed || !matches!(c.situation, GROUND | AIR) {
            self.reset();
            return ControllerOutput::NONE;
        }
        if self.object != Some(c.object) {
            self.reset();
            self.object = Some(c.object);
        }
        let mut output = ControllerOutput {
            owned: crate::ledge_trap::SEQUENCE_BUTTONS,
            stick: Some([0.0; 2]),
            ..ControllerOutput::NONE
        };
        let horizon = defense_horizon(lead_frames, input_delay);
        let imminent = contact.zip(horizon).is_some_and(|(hit, horizon)| {
            hit.in_frames.is_finite() && (0.0..=horizon).contains(&hit.in_frames)
        });
        if c.situation == GROUND {
            self.pending_since = None;
            self.delivered = false;
            self.air_used = false;
            self.unavailable_seen = false;
            if imminent && c.guard_ready {
                output.requested = crate::native_input::MAPPER_ZR;
            }
            return output;
        }
        if c.air_dodging {
            self.pending_since = None;
            self.delivered = false;
            self.air_used = true;
            self.unavailable_seen = true;
            return output;
        }
        if !c.air_dodge_ready {
            self.pending_since = None;
            if self.delivered {
                self.air_used = true;
            }
            self.delivered = false;
            if self.air_used {
                self.unavailable_seen = true;
            }
            return output;
        }
        if self.unavailable_seen {
            self.air_used = false;
            self.unavailable_seen = false;
        }
        if !imminent {
            self.pending_since = None;
            self.delivered = false;
            return output;
        }
        if c.stopped || self.air_used {
            return output;
        }
        if self.delivered {
            let start = self.pending_since.unwrap_or(tick);
            let wait = 2 + input_delay.ceil() as i64;
            if tick >= start && tick.saturating_sub(start) < wait {
                return output;
            }
            // Delivery is not native acceptance. Release between attempts, then
            // retry a still-incoming hit without latching the air dodge as used.
            self.pending_since = None;
            self.delivered = false;
            return output;
        }
        let start = *self.pending_since.get_or_insert(tick);
        let budget = 8 + input_delay.ceil() as i64;
        if tick < start || tick.saturating_sub(start) > budget {
            self.pending_since = None;
            return output;
        }
        output.requested = crate::native_input::MAPPER_ZR;
        output
    }
}

#[cfg(test)]
mod held_defense_tests {
    use super::*;
    use crate::{native_input::MAPPER_ZR, native_states::situation::{AIR, GROUND}};

    fn context(situation: i32) -> Context {
        Context { object: 1, situation, allowed: true, guard_ready: true,
            air_dodge_ready: true, ..Context::default() }
    }
    fn hit(in_frames: f32) -> Option<Contact> {
        Some(Contact { source: 2, in_frames, projectile: false })
    }

    #[test]
    fn ground_reserves_controls_and_releases_at_contact_exit() {
        let mut defense = Defense::new();
        let c = context(GROUND);
        let out = defense.sample(0, true, c, hit(5.0), 4.0, 1.0);
        assert_eq!(out.requested, MAPPER_ZR);
        assert_eq!(out.owned, crate::ledge_trap::SEQUENCE_BUTTONS);
        assert_eq!(out.stick, Some([0.0; 2]));
        for contact in [hit(5.1), hit(f32::NAN), hit(-1.0), None] {
            assert_eq!(defense.sample(1, true, c, contact, 4.0, 1.0).requested, 0);
        }
        assert_eq!(defense.sample(2, false, c, hit(0.0), 4.0, 1.0), ControllerOutput::NONE);
        assert_eq!(defense.sample(3, true, Context { allowed: false, ..c }, hit(0.0), 4.0, 1.0), ControllerOutput::NONE);
        assert_eq!(defense.sample(4, true, Context { guard_ready: false, ..c }, hit(0.0), 4.0, 1.0).requested, 0);
    }

    #[test]
    fn air_receipt_stops_repeats_until_native_readiness_returns() {
        let mut defense = Defense::new();
        let c = context(AIR);
        assert_eq!(defense.sample(0, true, c, hit(0.0), 4.0, 1.0).requested, MAPPER_ZR);
        assert!(defense.air_pending());
        defense.mapped_air_dodge();
        assert!(defense.air_pending());
        assert_eq!(defense.sample(1, true, c, hit(0.0), 4.0, 1.0).requested, 0);
        assert_eq!(defense.sample(2, true, Context { air_dodging: true, ..c }, hit(0.0), 4.0, 1.0).requested, 0);
        assert_eq!(defense.sample(3, true, Context { air_dodge_ready: false, ..c }, hit(0.0), 4.0, 1.0).requested, 0);
        assert_eq!(defense.sample(4, true, c, hit(0.0), 4.0, 1.0).requested, MAPPER_ZR);
    }

    #[test]
    fn pending_delivery_is_bounded_and_release_discards_it() {
        let mut defense = Defense::new();
        let c = context(AIR);
        assert_eq!(defense.sample(0, true, c, hit(0.0), 4.0, 1.0).requested, MAPPER_ZR);
        assert_eq!(defense.sample(10, true, c, hit(0.0), 4.0, 1.0).requested, 0);
        assert_eq!(defense.sample(11, true, c, hit(0.0), 4.0, 1.0).requested, MAPPER_ZR);
        defense.sample(12, false, c, hit(0.0), 4.0, 1.0);
        defense.mapped_air_dodge(); // A stale receipt cannot arm the new hold.
        assert_eq!(defense.sample(13, true, c, hit(0.0), 4.0, 1.0).requested, MAPPER_ZR);
        let stopped = Context { stopped: true, ..c };
        assert_eq!(defense.sample(14, true, stopped, hit(0.0), 4.0, 1.0).requested, 0);
        assert_eq!(defense.sample(15, true, c, None, 4.0, 1.0).requested, 0);
        assert!(!defense.air_pending());
    }

    #[test]
    fn rejected_mapped_dodge_releases_and_retries_until_native_acceptance() {
        let mut defense = Defense::new();
        let c = context(AIR);
        for start in [0, 4, 8] {
            assert_eq!(defense.sample(start, true, c, hit(0.), 4., 1.).requested, MAPPER_ZR);
            defense.mapped_air_dodge();
            for tick in start + 1..=start + 3 {
                assert_eq!(defense.sample(tick, true, c, hit(0.), 4., 1.).requested, 0);
            }
        }
        assert_eq!(defense.sample(12, true, c, hit(0.), 4., 1.).requested, MAPPER_ZR);
        defense.mapped_air_dodge();
        assert_eq!(defense.sample(13, true, Context { air_dodging: true, ..c }, hit(0.), 4., 1.).requested, 0);
        assert_eq!(defense.sample(14, true, Context { air_dodge_ready: false, ..c }, hit(0.), 4., 1.).requested, 0);
        assert_eq!(defense.sample(15, true, c, hit(0.), 4., 1.).requested, MAPPER_ZR);
    }

    #[test]
    fn movement_and_attack_states_have_no_extra_defense_lockouts() {
        use crate::native_states::{status, situation};
        for (where_, states) in [
            (GROUND, &[status::WAIT, status::WALK, status::DASH, status::TURN_DASH,
                status::RUN, status::TURN_RUN, status::RUN_BRAKE, status::TURN_RUN_BRAKE,
                status::JUMP_SQUAT, status::SQUAT, status::SQUAT_WAIT, status::LANDING,
                status::LANDING_LIGHT, status::GUARD_OFF, status::ATTACK][..]),
            (AIR, &[status::JUMP, status::JUMP_AERIAL, status::FALL, status::FALL_AERIAL,
                status::ATTACK_AIR, status::ESCAPE_AIR, status::ESCAPE_AIR_SLIDE][..])
        ] {
            for &state in states {
                assert!(native_allowed(crate::fighter_kinds::KIRBY, state, where_, false, true), "{state}");
            }
        }
        for state in [status::DAMAGE, status::CAPTURE_PULLED, status::DEAD] {
            assert!(!native_allowed(crate::fighter_kinds::KIRBY, state, GROUND, false, true));
        }
        assert!(!native_allowed(crate::fighter_kinds::KIRBY, status::WAIT, situation::CLIFF, false, true));
        assert!(!native_allowed(crate::fighter_kinds::KIRBY, status::WAIT, GROUND, true, true));
    }

    #[test]
    fn lead_and_delivery_delay_share_bounded_prediction_window() {
        assert_eq!(defense_horizon(0.0, 0.0), Some(0.0));
        assert_eq!(defense_horizon(4.0, 3.0), Some(7.0));
        assert_eq!(defense_horizon(30.0, 120.0), Some(40.0));
        assert_eq!(defense_horizon(f32::NAN, 0.0), None);
        assert_eq!(defense_horizon(31.0, 0.0), None);
        assert_eq!(defense_horizon(4.0, -1.0), None);
    }

    #[test]
    fn valid_scaled_bodies_predict_real_contacts_not_root_distance() {
        let body = crate::auto_counter::Config { startup: 1.0, input_delay: 3.0,
            margin: 0.0, half_width: 2.0, height: 8.0, half_depth: 2.0 };
        let attack = crate::autopilot_guard::Hitbox {
            start: [5.0, 4.0, 0.0], end: [5.0, 4.0, 0.0], radius: 1.0, velocity: [0.0; 2],
        };
        assert_eq!(crate::auto_counter::incoming_contact(body, [0.0; 3], [0.0; 2], attack, 4.0, None), None);
        let large = scaled_body(body, 2.0).unwrap();
        assert_eq!(crate::auto_counter::incoming_contact(large, [0.0; 3], [0.0; 2], attack, 4.0, None), Some(0.0));
        let approaching = crate::autopilot_guard::Hitbox { velocity: [-1.0, 0.0], ..attack };
        assert_eq!(crate::auto_counter::incoming_contact(body, [0.0; 3], [0.0; 2], approaching, 4.0, None), Some(2.0));
        assert!(scaled_body(body, f32::NAN).is_none());
        assert!(scaled_body(body, 0.0).is_none());
    }
}

/// Runtime Smash data carried by `Response`.
#[derive(Default)]
pub struct Response {
    /// Runtime plan value used by Response..
    plan: Option<InputTimeline>,
    /// selected opponent/projectile target.
    target: Option<u32>,
    /// Runtime up b value used by Response..
    up_b: UpBFlow,
    /// Runtime configured value used by Response..
    configured: bool,
    /// Runtime grab sent value used by Response..
    grab_sent: bool,
    /// Runtime grab settling value used by Response..
    grab_settling: bool,
    /// Runtime grab delivery value used by Response..
    grab_delivery: crate::move_input::DeliveryBudget,
}
impl Response {
    /// Constructs a new value with the type’s initial state.
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
    pub fn target(&self) -> Option<u32> {
        self.target
    }
    /// Determines whether ground.
    pub fn needs_ground(&self) -> bool {
        self.plan.as_ref().is_some_and(InputTimeline::needs_ground)
    }
    /// Determines whether the `attack_hold` condition holds.
    pub fn attack_hold(&self) -> bool {
        self.plan.as_ref().is_some_and(InputTimeline::attack_hold)
    }
    /// Returns an optional `right_stick` value.
    pub fn right_stick(&self) -> Option<[f32; 2]> {
        self.plan.as_ref().map(InputTimeline::right_stick)
    }
    /// Determines whether the `walking` condition holds.
    pub fn walking(&self) -> bool {
        self.plan.as_ref().is_some_and(InputTimeline::walking)
    }
    /// Determines whether the `automatic_up_b` condition holds.
    pub fn automatic_up_b(&self) -> bool {
        self.up_b.armed() && self.plan.is_some()
    }
    /// Determines whether the `up_b_delivery_started` condition holds.
    pub fn up_b_delivery_started(&self) -> bool {
        self.automatic_up_b() && self.up_b.started()
    }
    /// Updates the `attempted_up_special` state using the supplied inputs.
    pub fn attempted_up_special(&mut self, tick: i64, target: u32) {
        if self.automatic_up_b() && self.target == Some(target) {
            self.up_b.attempted(tick);
        }
    }
    /// Updates the `mapped_up_special` state using the supplied inputs.
    pub fn mapped_up_special(&mut self, tick: i64, target: u32) {
        if self.automatic_up_b() && self.target == Some(target) {
            self.up_b.mapped(tick);
        }
    }
    /// Determines whether the `configured_committed` condition holds.
    pub fn configured_committed(&self) -> bool {
        self.configured && self.target.is_some()
    }
    /// Determines whether the `grab_delivery_pending` condition holds.
    pub fn grab_delivery_pending(&self) -> bool {
        self.grab_sent && self.target.is_some()
    }
    /// Updates the `mapped_grab` state using the supplied inputs.
    pub fn mapped_grab(&mut self, tick: i64, target: u32) {
        if self.configured && self.target == Some(target) {
            self.grab_sent = true;
            self.grab_delivery.mapped(tick);
        }
    }
    /// Determines whether the `up_b_receipt_pending` condition holds.
    pub fn up_b_receipt_pending(&self) -> bool {
        self.automatic_up_b() && self.up_b.receipt_pending()
    }
    /// Determines whether the `awaiting_up_b` condition holds.
    pub fn awaiting_up_b(&self) -> bool {
        self.automatic_up_b() && self.up_b.awaiting()
    }
    /// Updates the `delivery_tick` state using the supplied inputs.
    pub fn delivery_tick(&mut self, tick: i64) {
        if self.grab_sent {
            self.grab_delivery.mapped(tick);
        }
        if self.automatic_up_b() {
            self.up_b.tick(tick);
        }
    }
    /// Stop playback without forgetting a controller Grab still in flight.
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
    pub fn cancel(&mut self) {
        *self = Self::new();
    }
    /// Common admission for an explicitly configured automatic response: native
    /// control now, an available target and no grab already in progress.
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
    fn follows(&self, e: Opponent, c: GameplayContext) -> bool {
        Some(e.id) == self.target
            && finite_positions(e.position, c.position)
            && (crate::reaction_rules::automatic_target_available(e.status)
                || self.plan.is_some()
                    && crate::reaction_rules::automatic_follow_target_available(e.status))
    }
    /// Updates the toggle state from the current button sample and match state.
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
        c.held = true;
        let output = plan.sample(settings, c, enemy, None, 0.0);
        let output = if self.up_b.armed() || self.configured {
            crate::out_of_shield::direct_shield_input(c.status, output, plan.right_stick())
        } else {
            output
        };
        match self.up_b.after_sample(false, output.requested) {
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
