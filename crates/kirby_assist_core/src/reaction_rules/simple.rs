//! One committed response, no vulnerability/priority/crossing decisions.
use super::*;
pub fn event(status: i32) -> Option<usize> {
    match status {
        crate::native_states::ESCAPE_F | crate::native_states::ESCAPE_B => Some(0),
        crate::native_states::ESCAPE => Some(1),
        crate::native_states::ESCAPE_AIR | crate::native_states::ESCAPE_AIR_SLIDE => Some(2),
        _ => None,
    }
}
pub(super) fn starts_up_b(sequence: &Sequence) -> bool {
    matches!(sequence.actions().first(),
        Some(crate::sequence_config::Action::Input{buttons,stick,..}
            |crate::sequence_config::Action::OpponentInput{buttons,stick,..}
            |crate::sequence_config::Action::Controller{buttons,stick,..})
            if *buttons==crate::native_input::MAPPER_B && stick[1]>0.5)
}
pub(super) fn up_b_status(kind: i32, c: GameplayContext) -> bool {
    automatic_up_b_status(kind, c.status, c.previous_status)
}
impl Engine {
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
        let mut nearest: Option<(f32, usize, &Sequence)> = None;
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
            if self.tracks.iter().flatten().any(|track| {
                track.id == t.id
                    && track.status == t.status
                    && track.motion == t.motion
                    && t.frame >= track.frame
                    && track.used
            }) {
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
                && nearest.as_ref().is_none_or(|(old, _, _)| d < *old)
            {
                nearest = Some((d, i, sequence));
            }
        }
        nearest.map(|(_, i, sequence)| (i, sequence))
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
        let tracking = settings.fighter_kind == crate::fighter_kinds::BOWSER
            && self.simple_up_b
            && (self.simple_up_b_seen
                || up_b_status(settings.fighter_kind, c)
                || self.simple_receipt.sent());
        if follow.is_some_and(|enemy| {
            !automatic_target_available(enemy.status)
                && !(tracking && automatic_tracking_target_available(enemy.status))
        }) {
            self.cancel();
            for track in self.tracks.iter_mut().flatten() {
                track.used = true;
            }
            return ControllerOutput::default();
        }
        if !c.gameplay {
            self.cancel();
            self.tracks = [None; crate::battle_objects::FIGHTER_CAPACITY];
            return ControllerOutput::default();
        }
        // Read current live events without queuing an unactable response. A
        // still-occurring roll can become usable after shieldstun/recovery.
        if !c.stopped {
            let old = self.tracks;
            self.tracks = std::array::from_fn(|i| {
                targets
                    .get(i)
                    .copied()
                    .flatten()
                    .filter(|t| {
                        event(t.status).is_some()
                            && t.frame.is_finite()
                            && t.frame >= 0.
                            && t.rate.is_finite()
                            && t.rate >= 0.
                            && t.position.iter().all(|v| v.is_finite())
                    })
                    .map(|t| {
                        let used = old.iter().flatten().any(|p| {
                            p.id == t.id
                                && p.status == t.status
                                && p.motion == t.motion
                                && t.frame >= p.frame
                                && p.used
                        });
                        Track {
                            id: t.id,
                            status: t.status,
                            motion: t.motion,
                            frame: t.frame,
                            dx: 0.,
                            crossed: false,
                            used,
                        }
                    })
            });
            // Fresh actors take their current slots first. Preserve missing
            // consumed identities in spare slots, rather than overwriting one
            // when a different fighter reuses its old roster position.
            for previous in old.into_iter().flatten().filter(|p| p.used) {
                if self.tracks.iter().flatten().any(|t| t.id == previous.id)
                    || targets
                        .iter()
                        .flatten()
                        .any(|t| t.id == previous.id && t.status != previous.status)
                {
                    continue;
                }
                if let Some(empty) = self.tracks.iter_mut().find(|t| t.is_none()) {
                    *empty = Some(previous);
                }
            }
        }
        if !allowed {
            self.cancel();
            for t in self.tracks.iter_mut().flatten() {
                t.used = true;
            }
            return ControllerOutput::default();
        }
        // Only a selected/cancelled event is consumed. There is no delayed
        // response: admission still requires a live event and readiness now.
        if self.plan.is_none() {
            if let Some((i, sequence)) = self.simple_candidate(rules, c, &targets, readiness) {
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
                self.simple_up_b = starts_up_b(&sequence);
                self.simple_up_b_seen = false;
                self.simple_manual_epoch = c.automatic_up_b_epoch.unwrap_or(c.manual_input_epoch);
                self.simple_delivery = Default::default();
                self.simple_receipt = Default::default();
                self.simple_waiting = false;
                if self.simple_up_b {
                    sequence.ground_only = false;
                }
                let enemy = Opponent {
                    id: t.id,
                    status: t.status,
                    // These three event families have known native situations.
                    // Keep first-frame conditional checks consistent with the
                    // subsequent read-only follow sample.
                    situation: if matches!(
                        t.status,
                        crate::native_states::ESCAPE_AIR | crate::native_states::ESCAPE_AIR_SLIDE
                    ) {
                        2
                    } else {
                        0
                    },
                    position: t.position,
                    velocity: [t.velocity_x, 0.],
                    motion: Some(crate::gameplay::MotionTiming {
                        kind: t.motion,
                        frame: t.frame,
                        rate: t.rate,
                        remaining: None,
                    }),
                    ..Default::default()
                };
                self.target = Some(t.id);
                let mut plan = InputTimeline::new(sequence, enemy);
                plan.allow_continuous_progress();
                plan.guard_automatic_ground_actions();
                self.plan = Some(plan);
                follow = Some(enemy);
            }
        }
        if self.simple_up_b && self.plan.is_some() {
            let native = up_b_status(settings.fighter_kind, c);
            if settings.fighter_kind == crate::fighter_kinds::BOWSER
                && !self.simple_up_b_seen
                && self.simple_receipt.sent()
            {
                if follow.is_none_or(|e| {
                    Some(e.id) != self.target
                        || !e
                            .position
                            .iter()
                            .chain(c.position.iter())
                            .all(|v| v.is_finite())
                }) {
                    self.cancel();
                    return ControllerOutput::default();
                }
                if native && self.simple_waiting {
                    if let Some(enemy) = follow.filter(|e| Some(e.id) == self.target) {
                        self.plan.as_mut().unwrap().confirm_initial_up_b(c, enemy);
                    }
                } else if !native {
                    if !automatic_up_b_receipt_status(c.status, c.situation, false)
                        || self.simple_receipt.expired()
                    {
                        self.cancel();
                        return ControllerOutput::default();
                    }
                    if self.simple_waiting
                        || c.stopped
                        || !c.can_act
                        || self.simple_delivery.exhausted()
                    {
                        self.simple_waiting = true;
                        return ControllerOutput::default();
                    }
                }
            }
            self.simple_up_b_seen |= native;
            // A native roll/jump/wrong special is not successful Up-B delivery.
            // Confirmed Up-B drift also ends once ordinary control resumes.
            let wrong = matches!(
                c.status,
                crate::native_states::status::JUMP..=crate::native_states::status::FALL_AERIAL
                    | crate::native_states::ESCAPE
                        ..=crate::native_states::ESCAPE_AIR_SLIDE
                    | crate::native_states::CATCH..=crate::native_states::THROW
                    | crate::native_states::status::SPECIAL_N
                    | crate::native_states::status::SPECIAL_S
                    | crate::native_states::status::SPECIAL_LW
            );
            let ended = self.simple_up_b_seen
                && matches!(c.status,crate::native_states::status::WAIT..=crate::native_states::status::TURN_RUN_BRAKE|crate::native_states::status::FALL_SPECIAL|crate::native_states::status::LANDING..=crate::native_states::GUARD_DAMAGE);
            if wrong
                || ended
                || self.simple_up_b_seen && !native
                || !self.simple_up_b_seen
                    && (!c.can_act
                        || !automatic_start_allowed(c.status, c.situation, c.stopped, false)
                        || self.simple_delivery.exhausted())
            {
                self.cancel();
                return ControllerOutput::default();
            }
        }
        if let Some(plan) = self.plan.as_mut() {
            let enemy = follow.filter(|e| Some(e.id) == self.target);
            if enemy.is_none() {
                self.cancel();
                return ControllerOutput::default();
            }
            let manual_drift = settings.fighter_kind != crate::fighter_kinds::BOWSER
                && self.simple_up_b
                && self.simple_up_b_seen
                && c.automatic_up_b_epoch.unwrap_or(c.manual_input_epoch)
                    != self.simple_manual_epoch;
            if manual_drift && plan.drifting() {
                self.cancel();
                return ControllerOutput::default();
            }
            c.held = true;
            // Admission required immediate native Up-B readiness. Never reserve
            // or neutralize controls waiting for our own dodge/shield recovery.
            let output = crate::out_of_shield::direct_shield_input(
                c.status,
                plan.sample(settings, c, enemy, None, 0.),
                plan.right_stick(),
            );
            // Other fighters keep manual handback on the first drift observation.
            // Bowser retains the entire tail; never cut the critical B pulse.
            if manual_drift && plan.drifting() {
                self.cancel();
                return ControllerOutput::default();
            }
            // Never drift before native startup. A delivered Bowser packet may
            // keep only a bounded, input-free receipt; no B is retried.
            if self.simple_up_b
                && !self.simple_up_b_seen
                && output.requested & crate::native_input::MAPPER_B == 0
            {
                if settings.fighter_kind == crate::fighter_kinds::BOWSER
                    && self.simple_receipt.sent()
                {
                    self.simple_waiting = true;
                    return ControllerOutput::default();
                }
                self.cancel();
                return ControllerOutput::default();
            }
            if plan.finished() {
                self.simple_up_b = false;
                self.simple_up_b_seen = false;
                self.plan = None;
                self.target = None;
            }
            output
        } else {
            ControllerOutput::default()
        }
    }
}
