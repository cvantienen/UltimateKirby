//! Small revision-built observation metadata; no per-poll tape copies/scans.
use super::*;
use crate::move_input::{finite_positions, UpBVerdict};

/// Runtime Smash data carried by `SimpleStart`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimpleStart {
    /// Runtime distance value used by SimpleStart..
    pub distance: f32,
    /// Runtime height value used by SimpleStart..
    pub height: f32,
    /// Runtime up special value used by SimpleStart..
    pub up_special: bool,
}

/// Determines whether ground.
/// Checks whether ground.
pub(super) fn needs_ground(sequence: &Sequence) -> bool {
    sequence.actions().iter().any(|action| {
        matches!(
            action,
            Action::Chase { .. }
                | Action::Walk { .. }
                | Action::Run { .. }
                | Action::Approach {
                    airborne: false,
                    ..
                }
        )
    })
}

impl Resolved {
    /// Determines whether the `automatic_needs_up_special` condition holds.
    pub fn automatic_needs_up_special(&self, enabled_events: u8) -> bool {
        self.automatic_up_special & enabled_events != 0
    }
    /// Returns an optional `simple_starts` value.
    pub fn simple_starts(&self) -> [Option<SimpleStart>; 3] {
        self.simple_starts
    }

    /// Automatic grabs, their backup and grab-dodge punishment have independent
    /// switches. Their movement checks must not depend on Auto-react's toggle.
    pub fn automatic_ground_watch(&self, enabled_events: u8) -> Option<(f32, f32)> {
        if enabled_events == 0 {
            return None;
        }
        let mut result: Option<(f32, f32)> = None;
        for (index, watch) in self.automatic_ground.iter().enumerate() {
            if enabled_events & (1 << index) == 0 {
                continue;
            }
            if let Some((distance, height)) = watch {
                let previous = result.unwrap_or((0., 0.));
                result = Some((previous.0.max(*distance), previous.1.max(*height)));
            }
        }
        result
    }
}

/// Native Up-B availability remains specific to Up-B recipes. Ordinary custom
/// responses use grounded control readiness, with a lazy shield-readiness query
/// only for an in-range candidate. All native queries happen before STATE locks.
pub fn simple_readiness(
    starts: [Option<SimpleStart>; 3],
    c: GameplayContext,
    targets: &[Option<Target>; 8],
    mut shield_ready: impl FnMut() -> bool,
) -> [bool; 3] {
    let mut result = [false; 3];
    if !c.gameplay
        || !automatic_start_allowed(c.status, c.situation, c.stopped, false)
        || starts.iter().all(Option::is_none)
    {
        return result;
    }
    let mut shield = None;
    for target in targets.iter().flatten() {
        let Some(event) = event(target.status) else {
            continue;
        };
        let Some(start) = starts[event] else {
            continue;
        };
        if result[event]
            || !target.frame.is_finite()
            || target.frame < 0.
            || !target.rate.is_finite()
            || target.rate < 0.
            || !target
                .position
                .iter()
                .chain(c.position.iter())
                .all(|v| v.is_finite())
            || (target.position[0] - c.position[0]).abs() > start.distance
            || (target.position[1] - c.position[1]).abs() > start.height
        {
            continue;
        }
        result[event] = if start.up_special {
            c.can_act
        } else if !crate::out_of_shield::stationary_grab_state(c.status) {
            false
        } else if c.status == crate::native_states::status::WAIT || c.can_act {
            true
        } else {
            *shield.get_or_insert_with(&mut shield_ready)
        };
    }
    result
}

/// No engine is created for an unconfigured fighter/event group. Existing
/// engines are borrowed in place instead of taken and rebuilt every poll.
pub fn sample_optional_simple(
    engine: &mut Option<Box<Engine>>,
    allowed: bool,
    rules: Option<&Resolved>,
    settings: AssistSettings,
    c: GameplayContext,
    targets: [Option<Target>; 8],
    follow: Option<Opponent>,
    readiness: [bool; 3],
) -> ControllerOutput {
    let Some(rules) = rules.filter(|rules| rules.has_simple_events()) else {
        *engine = None;
        return ControllerOutput::default();
    };
    if engine.is_none() && (!allowed || !readiness.into_iter().any(|ready| ready)) {
        return ControllerOutput::default();
    }
    engine
        .get_or_insert_with(Box::<Engine>::default)
        .sample_simple_ready(allowed, rules, settings, c, targets, follow, readiness)
}

/// One committed response, with no vulnerability, priority, or crossing
/// decisions. These helpers share the simple-response readiness metadata above.
fn event(status: i32) -> Option<usize> {
    match status {
        crate::native_states::ESCAPE_F | crate::native_states::ESCAPE_B => Some(0),
        crate::native_states::ESCAPE => Some(1),
        crate::native_states::ESCAPE_AIR | crate::native_states::ESCAPE_AIR_SLIDE => Some(2),
        _ => None,
    }
}

/// Whether a sequence begins with an immediate upward special input.
pub(super) fn starts_up_b(sequence: &Sequence) -> bool {
    matches!(sequence.actions().first(),
        Some(crate::sequence_config::Action::Input{buttons,stick,..}
            |crate::sequence_config::Action::OpponentInput{buttons,stick,..}
            |crate::sequence_config::Action::Controller{buttons,stick,..})
            if *buttons==crate::native_input::MAPPER_B && stick[1]>0.5)
}

impl Engine {
    /// Observes the simple events.
    pub(super) fn observe_simple_events(&mut self, targets: &[Option<Target>; 8]) {
        self.observe_events(targets, None, |t| event(t.status).is_some());
    }

    fn simple_candidate<'a>(
        &self,
        rules: &'a Resolved,
        c: GameplayContext,
        targets: &[Option<Target>; 8],
        readiness: [bool; 3],
    ) -> Option<(usize, &'a Sequence)> {
        let ready_now = automatic_start_allowed(c.status, c.situation, c.stopped, false);
        if !c.gameplay || !ready_now || !readiness.into_iter().any(|ready| ready) {
            return None;
        }
        for (i, t) in targets.iter().enumerate() {
            let Some(t) = t.filter(|t| {
                t.frame.is_finite()
                    && t.frame >= 0.
                    && t.rate.is_finite()
                    && t.rate >= 0.
                    && t.position.iter().all(|v| v.is_finite())
            }) else {
                continue;
            };
            if self
                .tracks
                .iter()
                .flatten()
                .any(|track| track.continues(&t) && track.used)
            {
                continue;
            }
            let Some(sequence) = event(t.status)
                .filter(|event| readiness[*event])
                .and_then(|event| rules.simple_sequence(event))
            else {
                continue;
            };
            let d = (t.position[0] - c.position[0]).abs();
            let r = sequence.reaction;
            // Moving admission is reserved for a direct Up-B input. A
            // configured grab/ordinary attack must not brake locomotion.
            let immediate_up_b = starts_up_b(sequence);
            if (crate::out_of_shield::stationary_grab_state(c.status) || immediate_up_b)
                && d.is_finite()
                && d <= r.max
                && (t.position[1] - c.position[1]).abs() <= r.height
            // The first observed eligible event owns the response. Do
            // not reorder simultaneous events by distance: that made a
            // later target steal a sequence from the event seen first.
            {
                return Some((i, sequence));
            }
        }
        None
    }

    /// Only a usable immediate Up-B may replace an unaccepted automatic grab.
    /// This does not consume an event, advance a tape or bypass native readiness.
    fn prefers_simple_up_b(
        &self,
        rules: &Resolved,
        c: GameplayContext,
        targets: &[Option<Target>; 8],
        follow: Option<Opponent>,
    ) -> bool {
        self.plan.is_none()
            && !follow.is_some_and(|e| {
                matches!(
                    e.status,
                    crate::native_states::status::CLIFF_CATCH_MOVE
                        ..=crate::native_states::status::CLIFF_WAIT
                )
            })
            && self
                .simple_candidate(rules, c, targets, [c.can_act; 3])
                .is_some_and(|(_, sequence)| sequence.start_delay == 0 && starts_up_b(sequence))
    }

    /// Yield an uncommitted grab or a committed approach before R is delivered.
    /// The caller clears cached grab output before sampling the reaction.
    pub fn preempt_automatic_grab(
        &self,
        allowed: bool,
        rules: &Resolved,
        c: GameplayContext,
        targets: &[Option<Target>; 8],
        follow: Option<Opponent>,
        grab: &mut crate::out_of_shield::OutOfShield,
        ownership: &mut crate::action_owner::ActionOwnership,
    ) -> bool {
        use crate::action_owner::Owner;
        let approach_owner =
            grab.approach_active() && ownership.current_owner() == Some(Owner::OutOfShield);
        let admitted = if approach_owner {
            ownership.allows_after_release(Owner::RollRead, Owner::OutOfShield)
        } else {
            ownership.allows(Owner::RollRead)
        };
        if !allowed
            || !grab.automatic_grab_pending()
            || grab.grab_delivery_pending()
            || !admitted
            || !self.prefers_simple_up_b(rules, c, targets, follow)
        {
            return false;
        }
        grab.cancel_pending();
        if approach_owner {
            ownership.release(Owner::OutOfShield);
        }
        true
    }

    /// Samples an automatic simple reaction.
    pub fn sample_simple(
        &mut self,
        allowed: bool,
        rules: &Resolved,
        settings: AssistSettings,
        c: GameplayContext,
        targets: [Option<Target>; 8],
        follow: Option<Opponent>,
    ) -> ControllerOutput {
        self.sample_simple_ready(allowed, rules, settings, c, targets, follow, [c.can_act; 3])
    }

    /// Drop any tape and treat every remembered event as answered.
    fn consume_and_cancel(&mut self) -> ControllerOutput {
        self.cancel();
        for track in self.tracks.iter_mut().flatten() {
            track.used = true;
        }
        ControllerOutput::default()
    }

    /// Samples a simple reaction using the supplied native readiness snapshot.
    pub fn sample_simple_ready(
        &mut self,
        allowed: bool,
        rules: &Resolved,
        settings: AssistSettings,
        mut c: GameplayContext,
        targets: [Option<Target>; 8],
        mut follow: Option<Opponent>,
        readiness: [bool; 3],
    ) -> ControllerOutput {
        if follow.is_some_and(|enemy| {
            !automatic_target_available(enemy.status)
                && !(self.plan.is_some() && automatic_follow_target_available(enemy.status))
        }) {
            return self.consume_and_cancel();
        }
        if !c.gameplay {
            self.cancel();
            self.tracks = [None; crate::battle_objects::FIGHTER_CAPACITY];
            return ControllerOutput::default();
        }
        // A live event can become usable after shieldstun or recovery.
        if !c.stopped {
            self.observe_simple_events(&targets);
        }
        if !allowed {
            return self.consume_and_cancel();
        }
        if self.plan.is_none() {
            if let Some(enemy) = self.admit_simple(rules, c, &targets, readiness) {
                follow = Some(enemy);
            }
        }
        if self.plan.is_none() {
            return ControllerOutput::default();
        }
        if self.up_b.armed() && !self.advance_up_b(settings, c, follow) {
            return ControllerOutput::default();
        }
        let enemy = follow.filter(|e| Some(e.id) == self.target);
        let Some(plan) = self.plan.as_mut().filter(|_| enemy.is_some()) else {
            self.cancel();
            return ControllerOutput::default();
        };
        c.held = true;
        // Admission required immediate native Up-B readiness. Never reserve
        // or neutralize controls waiting for our own dodge/shield recovery.
        let sampled = plan.sample(settings, c, enemy, None, 0.);
        let output =
            crate::out_of_shield::direct_shield_input(c.status, sampled, plan.right_stick());
        match self.up_b.after_sample(false, output.requested) {
            UpBVerdict::Wait => return ControllerOutput::default(),
            UpBVerdict::Cancel => {
                self.cancel();
                return ControllerOutput::default();
            }
            UpBVerdict::Continue | UpBVerdict::Confirm => {}
        }
        if self.up_b.armed()
            && plan.drifting()
            && c.manual_input_epoch != self.start_epoch
        {
            self.cancel();
            return ControllerOutput::default();
        }
        if plan.finished() {
            self.up_b.disarm();
            self.plan = None;
            self.target = None;
        }
        output
    }

    /// Commit the nearest ready, unconsumed event.
    fn admit_simple(
        &mut self,
        rules: &Resolved,
        c: GameplayContext,
        targets: &[Option<Target>; 8],
        readiness: [bool; 3],
    ) -> Option<Opponent> {
        let (i, sequence) = self.simple_candidate(rules, c, targets, readiness)?;
        // Copy the winning tape once, not once for every candidate.
        let mut sequence = *sequence;
        let t = targets[i].unwrap();
        if let Some(track) = self
            .tracks
            .iter_mut()
            .flatten()
            .find(|track| track.id == t.id)
        {
            track.used = true;
        }
        sequence.while_held = false;
        // Grounded is an admission condition, not a reason to release
        // ownership halfway through a special move.
        if starts_up_b(&sequence) {
            sequence.ground_only = false;
            self.up_b.arm(c, false);
        } else {
            self.up_b.disarm();
        }
        // These three event families have known native situations. Keep
        // first-frame conditional checks consistent with the subsequent
        // read-only follow sample.
        let situation = if matches!(
            t.status,
            crate::native_states::ESCAPE_AIR | crate::native_states::ESCAPE_AIR_SLIDE
        ) {
            crate::native_states::situation::AIR
        } else {
            crate::native_states::situation::GROUND
        };
        let enemy = t.opponent(situation, None);
        self.target = Some(t.id);
        self.start_epoch = c.manual_input_epoch;
        self.plan = Some(automatic_plan(sequence, enemy));
        Some(enemy)
    }

    /// Native confirmation and lifecycle of an automatic direct Up-B.
    fn advance_up_b(
        &mut self,
        settings: AssistSettings,
        c: GameplayContext,
        follow: Option<Opponent>,
    ) -> bool {
        let tracked = follow.filter(|e| Some(e.id) == self.target);
        if self.up_b.receipt_pending()
            && !tracked.is_some_and(|e| finite_positions(e.position, c.position))
        {
            self.cancel();
            return false;
        }
        match self.up_b.advance(c, settings.fighter_kind) {
            UpBVerdict::Cancel => {
                self.cancel();
                false
            }
            UpBVerdict::Wait => false,
            UpBVerdict::Confirm => {
                if let (Some(plan), Some(enemy)) = (self.plan.as_mut(), tracked) {
                    plan.confirm_initial_up_b(c, enemy);
                }
                true
            }
            UpBVerdict::Continue => true,
        }
    }
}
