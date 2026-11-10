//! Ordered file inputs: one cursor, one native-progress clock, no move lookup.
//! Simple inputs never query action readiness, hitboxes, or acceptance. Only
//! explicitly spatial steps use already-copied opponent/ground observations.
use super::{
    attempt::NativeAttempt, finite2, AssistSettings, ControllerOutput, GameplayContext, Opponent,
    SEQUENCE_BUTTONS,
};
use crate::native_input::{
    MAPPER_A, MAPPER_B, MAPPER_R, MAPPER_RIGHT_STICK_DOWN, MAPPER_RIGHT_STICK_LEFT,
    MAPPER_RIGHT_STICK_RIGHT, MAPPER_RIGHT_STICK_UP, MAPPER_ZR, SEQUENCE_GUARD_BUTTONS,
    SEQUENCE_JUMP_BUTTONS,
};
use crate::native_states::{situation, GUARD, GUARD_DAMAGE, GUARD_OFF, GUARD_ON};
use crate::sequence_config::{
    Action, Condition, Direction, InputReference, RangeCheck, Sequence, MAX_ACTIONS,
};

pub struct Timeline {
    sequence: Sequence,
    cursor: u8,
    clock: NativeAttempt,
    step_start: f32,
    entered: bool,
    direction: f32,
    cstick_direction: f32,
    origin: f32,
    enemy: Opponent,
    done: bool,
    output: ControllerOutput,
    spatial_steps: u32,
    ground_steps: u32,
    right_stick: [f32; 2],
    blocked: bool,
    ledge_facing: Option<bool>,
    facing_captured: bool,
    manual_epoch: u64,
    shield_handoff: bool,
    continuous_progress: bool,
    grounded_automatic_actions: bool,
    automatic_ground_steps: u32,
    facing_jumps: [u8; MAX_ACTIONS],
    #[cfg(feature = "diagnostic-logging")]
    pub(super) end_reason: Option<&'static str>,
}

/// One observation shared by every step handler of a sample.
#[derive(Clone, Copy)]
struct Frame {
    ctx: GameplayContext,
    /// Last known state of the tracked opponent.
    enemy: Opponent,
    /// The tracked opponent as observed on this very poll, if usable.
    fresh: Option<Opponent>,
    anchor: Option<[f32; 2]>,
    inward: f32,
    grab_reach: f32,
    /// Native frames since the tape started (rebased by each `while_held` loop).
    now: f32,
    /// Native frames the current action has been running.
    elapsed: f32,
}
impl Frame {
    fn grounded(&self) -> bool {
        self.ctx.situation == situation::GROUND
    }
    fn enemy_dx(&self) -> f32 {
        self.enemy.position[0] - self.ctx.position[0]
    }
    fn safe_ground(&self, toward_positive: bool) -> bool {
        self.ctx.safe_ground[usize::from(toward_positive)]
    }
    fn direction(&self, d: Direction) -> f32 {
        direction(d, self.ctx, self.enemy, self.anchor, self.inward)
    }
}

/// What one action asks of the transition loop.
enum Step {
    /// Satisfied: begin the next action at this native time, in this sample.
    Advance(f32),
    /// Branch to a compiled destination at the current time.
    Goto(u8),
    Output(ControllerOutput),
    End(&'static str),
    /// End the tape and hand this very sample back to physical input.
    Release(&'static str),
}

impl Timeline {
    /// Only automatic tapes reject a pending Grab/Up-B after manual takeoff.
    /// Classify once at acquisition; the per-observation test is a bit mask.
    pub fn guard_automatic_ground_actions(&mut self) {
        self.grounded_automatic_actions = true;
        self.automatic_ground_steps = step_mask(self.sequence.actions(), |a| {
            literal(*a).is_some_and(|l| {
                l.buttons & MAPPER_R != 0 || l.buttons & MAPPER_B != 0 && l.stick[1] > 0.5
            })
        });
    }
    /// Auto react retains delivered holds across a short observation gap. At
    /// a crossed boundary, start the next input now with its full frame budget.
    /// Never burst unseen actions to catch up; ordinary ledge tapes stay strict.
    pub fn allow_continuous_progress(&mut self) {
        self.continuous_progress = true;
    }
    /// A late native receipt resumes AFTER the already delivered initial B.
    /// Keep the total timeout clock and grant the next authored step its full budget.
    pub fn confirm_initial_up_b(&mut self, ctx: GameplayContext, enemy: Opponent) {
        self.clock
            .advance_pause(ctx, Some(enemy), self.sequence.timeout);
        self.goto(1, self.clock.elapsed());
        self.done = false;
        self.output = ControllerOutput::default();
        self.right_stick = [0.; 2];
    }
    pub fn new(sequence: Sequence, enemy: Opponent) -> Self {
        let actions = sequence.actions();
        Self {
            sequence,
            cursor: 0,
            clock: NativeAttempt::default(),
            step_start: f32::from(sequence.start_delay),
            right_stick: [0.0; 2],
            blocked: false,
            ledge_facing: None,
            facing_captured: false,
            manual_epoch: 0,
            shield_handoff: false,
            continuous_progress: false,
            grounded_automatic_actions: false,
            automatic_ground_steps: 0,
            facing_jumps: facing_jumps(actions),
            #[cfg(feature = "diagnostic-logging")]
            end_reason: None,
            spatial_steps: step_mask(actions, is_spatial),
            ground_steps: step_mask(actions, moves_on_ground),
            entered: false,
            direction: 0.0,
            cstick_direction: 1.0,
            origin: 0.0,
            enemy,
            done: crate::sequence_config::validate_facing_blocks(actions).is_err(),
            output: ControllerOutput::default(),
        }
    }
    pub fn needs_ground(&self) -> bool {
        steps_remain(self.ground_steps, self.cursor)
    }
    pub fn finished(&self) -> bool {
        self.done
    }
    pub fn walking(&self) -> bool {
        !self.done
            && matches!(
                self.current(),
                Some(
                    Action::Walk { .. }
                        | Action::Approach {
                            airborne: false,
                            ..
                        }
                )
            )
    }
    /// Optional analog-only tail, eligible for explicit manual handback by the
    /// automatic reaction owner. Literal ledge/button playback stays unchanged.
    pub fn drifting(&self) -> bool {
        !self.done
            && self.entered
            && self.output.requested == 0
            && matches!(self.current(), Some(Action::Drift { .. }))
    }
    /// A literal neutral A hold, while this timeline still requests that input.
    pub fn attack_hold(&self) -> bool {
        !self.done
            && self.entered
            && self.output.requested & MAPPER_A != 0
            && self.current().and_then(literal).is_some_and(|l| {
                l.buttons == MAPPER_A && l.stick == [0.; 2] && l.cstick == [0.; 2] && l.hold > 1
            })
    }
    pub fn right_stick(&self) -> [f32; 2] {
        self.right_stick
    }
    pub fn inputs_only(&self) -> bool {
        !steps_remain(self.spatial_steps, self.cursor)
    }
    fn current(&self) -> Option<Action> {
        self.sequence
            .actions()
            .get(usize::from(self.cursor))
            .copied()
    }
    fn finish(&mut self, _reason: &'static str) -> ControllerOutput {
        #[cfg(feature = "diagnostic-logging")]
        {
            self.end_reason.get_or_insert(_reason);
        }
        self.done = true;
        self.right_stick = [0.0; 2];
        self.output = neutral();
        self.output
    }
    /// End the tape and return control now, rather than blanking one more
    /// packet of buffered human inputs.
    fn release(&mut self, reason: &'static str) -> ControllerOutput {
        self.finish(reason);
        self.output = ControllerOutput::default();
        self.output
    }
    fn goto(&mut self, cursor: u8, start: f32) {
        self.cursor = cursor;
        self.step_start = start;
        self.entered = false;
        self.blocked = false;
    }
    fn next(&mut self, start: f32) {
        self.goto(self.cursor + 1, start);
    }
    #[inline(always)]
    pub fn sample(
        &mut self,
        settings: AssistSettings,
        ctx: GameplayContext,
        enemy: Option<Opponent>,
        anchor: Option<[f32; 2]>,
        inward: f32,
    ) -> ControllerOutput {
        // A manual jump/takeoff must not leave an automatic Grab/Up-B queued.
        // Authored aerial attacks without Grab/Up-B stay literal.
        let takeoff = self.grounded_automatic_actions
            && (ctx.situation != situation::GROUND
                || ctx.status == crate::native_states::status::JUMP_SQUAT);
        let up_b =
            takeoff && crate::native_states::up_special_status(settings.fighter_kind, ctx.status);
        if takeoff && !up_b && steps_remain(self.automatic_ground_steps, self.cursor) {
            // Cancel before a preceding wait/neutral step can keep masking the manual jump.
            return self.release("automatic_ground_action_became_airborne");
        }
        let out = self.sample_inner(settings, ctx, enemy, anchor, inward);
        if takeoff {
            let grab = out.requested & MAPPER_R != 0;
            let special =
                out.requested & MAPPER_B != 0 && out.stick.is_some_and(|s| s[1] > 0.5) && !up_b;
            if grab || special {
                return self.release("automatic_ground_action_became_airborne");
            }
        }

        // Keep guard through direct native shield cancels, rather than release shield
        // in the same packet. Apply this only to the returned output: cached
        // literal controls must not retain an injected guard after a status
        // change/takeoff on a duplicate clock sample. No retry or extra frame.
        if ctx.situation == situation::GROUND {
            crate::out_of_shield::direct_shield_input(ctx.status, out, self.right_stick)
        } else {
            out
        }
    }
    fn sample_inner(
        &mut self,
        settings: AssistSettings,
        ctx: GameplayContext,
        enemy: Option<Opponent>,
        anchor: Option<[f32; 2]>,
        inward: f32,
    ) -> ControllerOutput {
        if let Some(out) = self.gate(settings.fighter_kind, ctx) {
            return out;
        }
        if !self.facing_captured {
            self.ledge_facing = facing_saved_ledge(ctx, anchor);
            self.facing_captured = true;
        }
        let before = self.clock.elapsed();
        self.clock.advance_pause(
            ctx,
            enemy.filter(|e| e.id == self.enemy.id),
            self.sequence.timeout,
        );
        let now = self.clock.elapsed();
        if let Some(reason) = self.clock_fault(before, now) {
            return self.finish(reason);
        }
        if ctx.stopped {
            return self.stopped_output(ctx, now);
        }
        // The first observation is frame zero. Later duplicate polls reuse the
        // output even if the mapper tick or opponent metadata changes.
        if self.entered && now == before && !self.current().is_some_and(is_predicate) {
            return self.output;
        }
        self.right_stick = [0.0; 2];
        let fresh = enemy.filter(|e| e.id == self.enemy.id && finite2(e.position));
        if let Some(enemy) = fresh {
            self.enemy = enemy;
        }
        if now < f32::from(self.sequence.start_delay) {
            return self.hold_neutral();
        }
        let frame = Frame {
            ctx,
            enemy: self.enemy,
            fresh,
            anchor,
            inward,
            grab_reach: settings.fighter.grab.standing_range,
            now,
            elapsed: 0.0,
        };
        self.advance(frame)
    }
    /// Scope and ownership checks that end the tape (or keep an until-manual
    /// hold) before this observation advances the clock.
    fn gate(&mut self, fighter_kind: i32, ctx: GameplayContext) -> Option<ControllerOutput> {
        // Death, ordinary capture and hitstun end this visit. Attack recovery,
        // jump squat, air states, shield and special phases do not gate inputs.
        if crate::action_owner::native_grab_active(ctx.status) {
            return Some(self.release("native_grab_handoff"));
        }
        // Damage/capture already ended this tape. Return control now.
        if crate::native_states::fighter_sequence_interrupted(fighter_kind, ctx.status) {
            return Some(self.release("scope_end_or_damage"));
        }
        if self.done || !ctx.gameplay || !ctx.held {
            return Some(self.finish("scope_end_or_damage"));
        }
        // Ordinary physical input is masked throughout the tape, not treated
        // as cancellation. Only explicit until-manual holds hand back on intent.
        if self.sequence.ground_only && ctx.situation != situation::GROUND {
            return Some(self.release("ground_only_airborne"));
        }
        // This explicit hold has no elapsed-frame deadline once entered. It
        // never injects an airborne dodge and fresh physical input ends the tape.
        if self.entered
            && matches!(
                self.current(),
                Some(Action::ShieldUntilManual | Action::Input { frames: 0, .. })
            )
        {
            let shield = matches!(self.current(), Some(Action::ShieldUntilManual))
                || self.output.requested & MAPPER_ZR != 0;
            if shield && ctx.situation != situation::GROUND
                || ctx.manual_input_epoch != self.manual_epoch
            {
                // Physical input owns this very sample.
                return Some(self.release("manual_takeover_or_airborne"));
            }
            return Some(self.output);
        }
        None
    }
    /// End the tape when the native clock is invalid, timed out, or jumped by
    /// more than one observation can account for.
    fn clock_fault(&mut self, before: f32, now: f32) -> Option<&'static str> {
        if self.clock.expired {
            return Some("clock_invalid_or_timeout");
        }
        (now - before > 1.25 && !self.gap_tolerated(before, now)).then_some("native_clock_gap")
    }
    /// Whether the current step can absorb a native-clock jump of more than one
    /// frame between observations.
    fn gap_tolerated(&mut self, before: f32, now: f32) -> bool {
        let current = self.current();
        let hold_end = |frames: u8, start: f32| start + f32::from(frames);
        // A gap entirely inside an already-held input needs no new edge: the
        // game-visible button stays down. Never catch up across its release,
        // start delay, or another action; missed taps still abort, never burst.
        let inside_hold = self.entered
            && matches!(
                current,
                Some(Action::Input { buttons, frames, .. } | Action::OpponentInput { buttons, frames, .. } | Action::Controller { buttons, frames, .. })
                    if buttons != 0 && now < hold_end(frames, self.step_start)
            );
        let inside_continuous = self.continuous_progress
            && self.entered
            && matches!(
                current,
                Some(Action::Drift { frames, .. } | Action::Wait(frames))
                    if now < hold_end(frames, self.step_start)
            );
        let resume_boundary = self.continuous_progress
            && self.entered
            && !self.clock.expired
            && match current {
                Some(
                    Action::Input { frames, .. }
                    | Action::OpponentInput { frames, .. }
                    | Action::Controller { frames, .. }
                    | Action::Drift { frames, .. }
                    | Action::Wait(frames),
                ) if frames > 0
                    && before < hold_end(frames, self.step_start)
                    && now >= hold_end(frames, self.step_start) =>
                {
                    // The previous hold was already delivered. End it on this
                    // observation, rather than spending/skipping the next tap.
                    self.step_start = now - f32::from(frames);
                    true
                }
                _ => false,
            };
        // Auto react's neutral shield-release wait has already been delivered.
        // A short observation gap cannot skip an action here: its configured
        // duration starts only after guard-off ends. Keep strict ledge timing.
        let waiting_handoff = self.continuous_progress && self.shield_handoff;
        // Waiting predicates have no scheduled input boundary to miss. Evaluate
        // current native facts after a short gap; a successful check starts the
        // next action NOW, with its entire configured duration intact.
        let waiting_predicate = self.entered && current.is_some_and(is_waiting_predicate);
        inside_hold || inside_continuous || resume_boundary || waiting_handoff || waiting_predicate
    }
    fn stopped_output(&mut self, ctx: GameplayContext, now: f32) -> ControllerOutput {
        if now >= f32::from(self.sequence.start_delay)
            && matches!(self.current(), Some(Action::ShieldUntilBlock(_)))
        {
            self.blocked |= ctx.status == GUARD_DAMAGE;
            self.output = guard_hold();
        }
        self.output
    }
    fn hold_neutral(&mut self) -> ControllerOutput {
        self.output = neutral();
        self.output
    }
    /// At most MAX_ACTIONS transitions, including zero-waits and already
    /// satisfied spatial conditions. No unbounded per-frame interpreter loop.
    fn advance(&mut self, mut f: Frame) -> ControllerOutput {
        for _ in 0..=MAX_ACTIONS {
            let Some(action) = self.current() else {
                if self.rewind_loop(&mut f) {
                    if f.now < self.step_start {
                        return self.hold_neutral();
                    }
                    continue;
                }
                return self.finish("completed");
            };
            if let Some(reason) = self.invalid_step(action, &f) {
                return self.finish(reason);
            }
            if !self.entered {
                // Satisfied spatial checks do not perform an input. Resolve them
                // before shield handoff so a no-op cannot pay shield-drop recovery.
                // (A near Chase branch must preserve direct Up-B out of shield.)
                if satisfied_on_entry(action, &f) {
                    self.next(f.now);
                    continue;
                }
                // Drop shield with neutral controls before ordinary attacks/movement.
                // The action's own duration starts only after native guard-off ends.
                // This shared tape path covers ledge/button recipes and all assists.
                if shield_handoff_needed(action, f.ctx.status, f.ctx.situation) {
                    self.shield_handoff = true;
                    return self.hold_neutral();
                }
                if self.shield_handoff {
                    self.shield_handoff = false;
                    self.step_start = f.now;
                }
                if let Some(reason) = self.enter(action, &f) {
                    return self.finish(reason);
                }
            }
            f.elapsed = f.now - self.step_start;
            match self.run_step(action, &f) {
                Step::Advance(start) => self.next(start),
                Step::Goto(cursor) => self.goto(cursor, f.now),
                Step::Output(out) => {
                    self.output = out;
                    return out;
                }
                Step::End(reason) => return self.finish(reason),
                Step::Release(reason) => return self.release(reason),
            }
        }
        self.finish("transition_limit")
    }
    /// A `while_held` tape repeats after its last action: rebase the native
    /// clock to the loop start so no extra frame is spent between repetitions.
    fn rewind_loop(&mut self, f: &mut Frame) -> bool {
        if !(self.sequence.while_held
            && self.step_start > 0.0
            && !self.sequence.actions().is_empty())
        {
            return false;
        }
        let duration = self.step_start;
        self.clock.consume(duration);
        f.now -= duration;
        self.step_start = f32::from(self.sequence.start_delay);
        self.cursor = 0;
        self.ledge_facing = facing_saved_ledge(f.ctx, f.anchor);
        self.entered = false;
        true
    }
    /// Parameter and geometry preconditions of an action about to run.
    fn invalid_step(&self, action: Action, f: &Frame) -> Option<&'static str> {
        if let Action::LedgePosition {
            offset,
            strength,
            tolerance,
            brake,
            frames,
        } = action
        {
            if !f.grounded() {
                return Some("ledge_position_requires_ground");
            }
            if !self.entered
                && (!(0. ..=200.).contains(&offset)
                    || !(0.3..=1.).contains(&strength)
                    || !(0. ..=20.).contains(&tolerance)
                    || !(0. ..=100.).contains(&brake)
                    || !(1..=3600).contains(&frames))
            {
                return Some("invalid_ledge_position");
            }
        }
        (needs_saved_ledge(action) && !f.anchor.is_some_and(finite2))
            .then_some("saved_ledge_unavailable")
    }
    /// Capture per-action references once, at the observation that starts it.
    fn enter(&mut self, action: Action, f: &Frame) -> Option<&'static str> {
        self.entered = true;
        self.manual_epoch = f.ctx.manual_input_epoch;
        self.origin = f.ctx.position[0];
        match action {
            Action::LedgePosition { offset, .. } => {
                let Some(goal) = ledge_goal(f.ctx, f.anchor, f.inward, offset) else {
                    return Some("ledge_position_unavailable");
                };
                self.origin = goal;
            }
            Action::Chase { gap, .. } => {
                self.origin = if f.enemy_dx().abs() > gap { 1. } else { 0. };
            }
            _ => {}
        }
        self.direction = match action {
            Action::Controller {
                reference,
                cstick_reference,
                ..
            } => {
                let axis = |r| match r {
                    InputReference::Screen => 1.,
                    InputReference::Facing => f.ctx.facing.signum(),
                    InputReference::Opponent => f.direction(Direction::Opponent),
                };
                self.cstick_direction = axis(cstick_reference);
                axis(reference)
            }
            Action::OpponentInput { .. } => f.direction(Direction::Opponent),
            Action::Input { relative: true, .. } | Action::Tap { .. } => f.ctx.facing.signum(),
            // Grounded backward stick can turn the fighter around.
            // Keep this action's initial direction rather than chasing
            // that turnaround and alternating left/right each update.
            Action::Drift {
                direction: Direction::Backward,
                ..
            } => -f.ctx.facing.signum(),
            Action::Face(d)
            | Action::Walk { direction: d, .. }
            | Action::Run { direction: d, .. } => f.direction(d),
            _ => 0.0,
        };
        None
    }
    fn run_step(&mut self, action: Action, f: &Frame) -> Step {
        match action {
            Action::Empty | Action::EndFacing => Step::Advance(f.now),
            Action::ElseFacing => self
                .facing_jump()
                .map_or(Step::End("invalid_facing_block"), Step::Goto),
            Action::IfFacingLedge { away } => self.branch(
                away,
                self.ledge_facing,
                "saved_ledge_facing_unavailable",
                f.now,
            ),
            Action::IfFacingOpponent { away } => self.branch(
                away,
                f.fresh
                    .and_then(|enemy| facing_toward(f.ctx, enemy.position[0])),
                "facing_opponent_unavailable",
                f.now,
            ),
            Action::Check {
                condition,
                value,
                wait,
            } => {
                if condition_met(condition, f.ctx, f.fresh, value) {
                    Step::Advance(f.now)
                } else if f.elapsed >= f32::from(wait) {
                    Step::End("condition_failed_or_timeout")
                } else {
                    Step::Output(neutral())
                }
            }
            Action::ShieldUntilManual if !f.grounded() => Step::Release("shield_requires_ground"),
            Action::ShieldUntilManual => Step::Output(guard_hold()),
            Action::ShieldUntilBlock(frames) => {
                self.blocked |= f.ctx.status == GUARD_DAMAGE;
                if self.blocked && matches!(f.ctx.status, GUARD_ON | GUARD) {
                    Step::Advance(f.now)
                } else if f.elapsed >= f32::from(frames) {
                    Step::End("shield_wait_timeout")
                } else {
                    Step::Output(guard_hold())
                }
            }
            Action::Input {
                buttons,
                stick,
                frames,
                relative,
            } => self.step_input(buttons, stick, frames, relative, f),
            Action::OpponentInput {
                buttons,
                stick,
                frames,
            } => {
                if f.fresh.is_none() || frames == 0 {
                    return Step::End("opponent_input_target_unavailable");
                }
                self.step_input(buttons, stick, frames, true, f)
            }
            Action::Tap {
                buttons,
                stick,
                hold,
                gap,
                frames,
            } => self.step_tap(buttons, stick, hold, gap, frames, f),
            Action::Controller {
                buttons,
                stick,
                cstick,
                frames,
                reference,
                cstick_reference,
            } => {
                let opponent_relative = |r| matches!(r, InputReference::Opponent);
                if (opponent_relative(reference) || opponent_relative(cstick_reference))
                    && f.fresh.is_none()
                {
                    return Step::End("controller_target_unavailable");
                }
                if frames == 0 {
                    return Step::End("invalid_controller_duration");
                }
                self.step_controller(buttons, stick, cstick, frames, f)
            }
            Action::CStick { stick, frames } => {
                if let Some(end) = self.hold_ended(f, frames) {
                    return end;
                }
                self.right_stick = stick;
                Step::Output(controls([0.0; 2], cstick_buttons(stick)))
            }
            Action::Wait(frames) => self
                .hold_ended(f, frames)
                .unwrap_or(Step::Output(neutral())),
            Action::Drift {
                direction: d,
                strength,
                frames,
            } => {
                if let Some(end) = self.hold_ended(f, frames) {
                    return end;
                }
                let dir = if d == Direction::Backward {
                    self.direction
                } else {
                    f.direction(d)
                };
                Step::Output(controls([dir * strength, 0.0], 0))
            }
            Action::Face(d) => self.step_face(d, f),
            Action::Walk {
                units,
                strength,
                min_gap,
                ..
            } => self.step_walk(units, strength, min_gap, f),
            Action::Run { units, .. } => self.step_run(units, f),
            Action::LedgePosition {
                strength,
                tolerance,
                brake,
                frames,
                ..
            } => self.step_ledge_position(strength, tolerance, brake, frames, f),
            Action::Chase {
                strength, frames, ..
            } => self.step_chase(strength, frames, f),
            Action::Approach {
                airborne,
                strength,
                frames,
                gap,
            } => Self::step_approach(airborne, strength, frames, gap, f),
            Action::Range(range) => self.step_range(range, f),
        }
    }
    fn step_tap(
        &self,
        buttons: u32,
        mut stick: [f32; 2],
        hold: u8,
        gap: u8,
        frames: u16,
        f: &Frame,
    ) -> Step {
        if let Some(end) = self.hold_ended(f, frames) {
            return end;
        }
        let period = (u16::from(hold) + u16::from(gap)).max(1);
        let phase = f.elapsed.floor() as u16 % period;
        Step::Output(if phase < u16::from(hold) {
            stick[0] *= self.direction;
            shield_grab_input(f.ctx.status, controls(stick, buttons), [0.; 2])
        } else {
            neutral()
        })
    }
    fn step_controller(
        &mut self,
        buttons: u32,
        mut stick: [f32; 2],
        mut cstick: [f32; 2],
        frames: u8,
        f: &Frame,
    ) -> Step {
        if let Some(end) = self.hold_ended(f, frames) {
            return end;
        }
        stick[0] *= self.direction;
        cstick[0] *= self.cstick_direction;
        self.right_stick = cstick;
        Step::Output(shield_grab_input(
            f.ctx.status,
            controls(stick, buttons | cstick_buttons(cstick)),
            cstick,
        ))
    }
    fn step_run(&self, units: f32, f: &Frame) -> Step {
        if self.direction == 0.0 || (f.ctx.position[0] - self.origin) * self.direction >= units {
            Step::Advance(f.now)
        } else if self.walk_blocked(f) {
            Step::End("walk_timeout")
        } else {
            Step::Output(controls([self.direction, 0.0], 0))
        }
    }
    fn step_chase(&self, strength: f32, frames: u8, f: &Frame) -> Step {
        if self.origin == 0. {
            return Step::Advance(f.now);
        }
        if let Some(end) = self.hold_ended(f, frames) {
            return end;
        }
        let dx = f.enemy_dx();
        if !f.grounded() || !f.safe_ground(dx > 0.) {
            return Step::End("unsafe_chase");
        }
        Step::Output(controls([dx.signum() * strength, 0.], 0))
    }
    fn step_approach(airborne: bool, strength: f32, frames: u8, gap: f32, f: &Frame) -> Step {
        let dx = f.enemy_dx();
        if dx.abs() <= gap || f.elapsed >= f32::from(frames) {
            return Step::Advance(f.now);
        }
        // A file may place an air approach immediately after Jump;
        // wait through squat, spending the explicitly configured time.
        if airborne && f.ctx.situation != situation::AIR {
            return Step::Output(neutral());
        }
        if !airborne && (!f.grounded() || !f.safe_ground(dx > 0.0)) {
            return Step::End("unsafe_approach");
        }
        Step::Output(controls([dx.signum() * strength, 0.0], 0))
    }
    /// End of a plain timed hold: the next action starts exactly on the
    /// boundary, not at the (possibly later) observation.
    fn hold_ended(&self, f: &Frame, frames: impl Into<f32>) -> Option<Step> {
        let frames = frames.into();
        (f.elapsed >= frames).then(|| Step::Advance(self.step_start + frames))
    }
    fn facing_jump(&self) -> Option<u8> {
        Some(self.facing_jumps[usize::from(self.cursor)]).filter(|jump| *jump != u8::MAX)
    }
    fn branch(&self, away: bool, toward: Option<bool>, missing: &'static str, now: f32) -> Step {
        let Some(jump) = self.facing_jump() else {
            return Step::End("invalid_facing_block");
        };
        match toward {
            None => Step::End(missing),
            Some(toward) if toward != away => Step::Advance(now),
            Some(_) => Step::Goto(jump),
        }
    }
    fn step_input(
        &self,
        buttons: u32,
        mut stick: [f32; 2],
        frames: u8,
        relative: bool,
        f: &Frame,
    ) -> Step {
        if frames == 0 {
            if buttons == 0 || buttons & MAPPER_ZR != 0 && !f.grounded() {
                return Step::Release("invalid_manual_hold_scope");
            }
        } else if let Some(end) = self.hold_ended(f, frames) {
            return end;
        }
        if relative {
            stick[0] *= self.direction;
        }
        Step::Output(shield_grab_input(
            f.ctx.status,
            controls(stick, buttons),
            [0.; 2],
        ))
    }
    fn step_face(&mut self, d: Direction, f: &Frame) -> Step {
        // Opponent/ledge references remain absolute targets. A
        // crossing while native recovery prevents turning must not
        // leave Face waiting for the opponent's old direction.
        if matches!(
            d,
            Direction::Opponent
                | Direction::AwayFromOpponent
                | Direction::Ledge
                | Direction::SavedLedge
                | Direction::AwayFromLedge
        ) {
            self.direction = f.direction(d);
        }
        if facing_satisfied(f.ctx, self.direction) {
            Step::Advance(f.now)
        } else if f.elapsed >= f32::from(self.sequence.spatial_timeout) {
            Step::End("face_timeout")
        } else {
            Step::Output(controls([self.direction * 0.3, 0.0], 0))
        }
    }
    /// Shared stop conditions of ground locomotion toward `self.direction`.
    fn walk_blocked(&self, f: &Frame) -> bool {
        f.elapsed >= f32::from(self.sequence.spatial_timeout)
            || !f.grounded()
            || !f.safe_ground(self.direction > 0.0)
    }
    fn step_walk(&self, units: f32, strength: f32, min_gap: Option<f32>, f: &Frame) -> Step {
        let dir = self.direction;
        let distance = (f.ctx.position[0] - self.origin) * dir;
        let gap = f.enemy_dx() * dir;
        let speed = (f.ctx.velocity[0] * dir).max(0.0);
        let stopping = if min_gap.is_some() {
            speed + stopping_distance(f.ctx, speed)
        } else {
            0.0
        };
        if dir == 0.0
            || distance + stopping >= units
            || min_gap.is_some_and(|min| gap <= min + stopping)
        {
            Step::Advance(f.now)
        } else if self.walk_blocked(f) {
            Step::End("unsafe_ground_or_air")
        } else {
            Step::Output(controls([dir * strength, 0.0], 0))
        }
    }
    fn step_ledge_position(
        &self,
        strength: f32,
        tolerance: f32,
        brake: f32,
        frames: u16,
        f: &Frame,
    ) -> Step {
        let ctx = f.ctx;
        let dx = self.origin - ctx.position[0];
        if !dx.is_finite() || !ctx.velocity[0].is_finite() {
            return Step::End("ledge_position_unavailable");
        }
        if ledge_arrived(ctx, self.origin, tolerance, brake) {
            return Step::Advance(f.now);
        }
        let dir = dx.signum();
        if !f.grounded() || !f.safe_ground(dir > 0.) || f.elapsed >= f32::from(frames) {
            return Step::End("ledge_position_timeout_or_unsafe_ground");
        }
        let speed = (ctx.velocity[0] * dir).max(0.);
        let stopping = stopping_distance(ctx, speed);
        let input = if brake > 0.
            && (dx.abs() <= tolerance || speed > 0. && dx.abs() <= stopping + speed + tolerance)
        {
            0.
        } else if brake > 0. && dx.abs() < brake {
            (strength * dx.abs() / brake).clamp(0.3, strength)
        } else {
            strength
        };
        Step::Output(controls([dir * input, 0.], 0))
    }
    fn step_range(&self, range: RangeCheck, f: &Frame) -> Step {
        let max = match range {
            RangeCheck::Ignore => f32::MAX,
            RangeCheck::Verify(value) => value,
            RangeCheck::FighterGrabRange | RangeCheck::FighterGrabRangeInstant => {
                if !f.grab_reach.is_finite() || f.grab_reach <= 0.0 {
                    return Step::End("range_check_failed");
                }
                f.grab_reach
            }
        };
        // Immediate checks require this observation's target. Waiting
        // ledge checks retain their existing last-position behavior.
        if range == RangeCheck::FighterGrabRangeInstant && f.fresh.is_none() {
            return Step::End("range_check_failed");
        }
        if f.enemy_dx().abs() <= max {
            Step::Advance(f.now)
        } else if range == RangeCheck::FighterGrabRangeInstant
            || f.elapsed >= f32::from(self.sequence.spatial_timeout)
        {
            Step::End("range_check_failed")
        } else if shielding(f.ctx.status) {
            Step::Output(guard_hold())
        } else {
            Step::Output(neutral())
        }
    }
}

/// Bit `i` is set when `pred` holds for action `i`.
fn step_mask(actions: &[Action], pred: impl Fn(&Action) -> bool) -> u32 {
    actions
        .iter()
        .enumerate()
        .filter(|(_, action)| pred(action))
        .fold(0, |mask, (index, _)| mask | 1 << index)
}
/// Whether any marked step is at or after the cursor.
fn steps_remain(mask: u32, cursor: u8) -> bool {
    mask.checked_shr(u32::from(cursor))
        .is_some_and(|rest| rest != 0)
}
/// Steps that read the opponent or ground observations.
fn is_spatial(action: &Action) -> bool {
    use InputReference::Opponent as Reference;
    matches!(
        action,
        Action::OpponentInput { .. }
            | Action::Face(_)
            | Action::Walk { .. }
            | Action::Run { .. }
            | Action::Chase { .. }
            | Action::Approach { .. }
            | Action::Drift { .. }
            | Action::Range(_)
            | Action::IfFacingOpponent { .. }
            | Action::LedgePosition { .. }
            | Action::Controller {
                reference: Reference,
                ..
            }
            | Action::Controller {
                cstick_reference: Reference,
                ..
            }
    ) || matches!(action, Action::Check { condition, .. } if condition.needs_enemy())
}
fn moves_on_ground(action: &Action) -> bool {
    matches!(
        action,
        Action::Chase { .. }
            | Action::Walk { .. }
            | Action::Run { .. }
            | Action::LedgePosition { .. }
            | Action::Approach {
                airborne: false,
                ..
            }
    )
}
fn needs_saved_ledge(action: Action) -> bool {
    let saved = |d| matches!(d, Direction::SavedLedge | Direction::AwayFromLedge);
    match action {
        Action::LedgePosition { .. } => true,
        Action::Face(d)
        | Action::Walk { direction: d, .. }
        | Action::Run { direction: d, .. }
        | Action::Drift { direction: d, .. } => saved(d),
        _ => false,
    }
}
/// Steps that wait on live native facts and are re-evaluated every poll.
fn is_predicate(action: Action) -> bool {
    is_waiting_predicate(action)
        || matches!(
            action,
            Action::Walk { .. }
                | Action::Run { .. }
                | Action::Chase { .. }
                | Action::Approach { .. }
                | Action::Drift { .. }
        )
}
/// Predicates with no scheduled input boundary a observation gap could miss.
fn is_waiting_predicate(action: Action) -> bool {
    matches!(
        action,
        Action::Face(_)
            | Action::Range(_)
            | Action::Check { .. }
            | Action::ShieldUntilBlock(_)
            | Action::LedgePosition { .. }
    )
}
/// A spatial action whose goal already holds performs no input at all.
fn satisfied_on_entry(action: Action, f: &Frame) -> bool {
    match action {
        Action::Face(d) => facing_satisfied(f.ctx, f.direction(d)),
        Action::LedgePosition {
            offset,
            tolerance,
            brake,
            ..
        } => ledge_goal(f.ctx, f.anchor, f.inward, offset)
            .is_some_and(|goal| ledge_arrived(f.ctx, goal, tolerance, brake)),
        Action::Chase { gap, .. } | Action::Approach { gap, .. } => f.enemy_dx().abs() <= gap,
        _ => false,
    }
}
fn facing_satisfied(ctx: GameplayContext, desired: f32) -> bool {
    desired == 0. || ctx.facing * desired > 0.5
}

/// Literal controller content of an input action.
#[derive(Clone, Copy)]
struct Literal {
    buttons: u32,
    stick: [f32; 2],
    cstick: [f32; 2],
    /// Frames (or tap pulse width) the content is held.
    hold: u16,
}
fn literal(action: Action) -> Option<Literal> {
    let (buttons, stick, cstick, hold) = match action {
        Action::Input {
            buttons,
            stick,
            frames,
            ..
        }
        | Action::OpponentInput {
            buttons,
            stick,
            frames,
        } => (buttons, stick, [0.; 2], u16::from(frames)),
        Action::Tap {
            buttons,
            stick,
            hold,
            ..
        } => (buttons, stick, [0.; 2], u16::from(hold)),
        Action::Controller {
            buttons,
            stick,
            cstick,
            frames,
            ..
        } => (buttons, stick, cstick, u16::from(frames)),
        Action::CStick { stick, frames } => (0, [0.; 2], stick, u16::from(frames)),
        _ => return None,
    };
    Some(Literal {
        buttons,
        stick,
        cstick,
        hold,
    })
}

// Compile flat branch destinations once. Executing a condition is one bounded
// lookup; no per-frame scan, allocation or synthetic input frame is needed.
fn facing_jumps(actions: &[Action]) -> [u8; MAX_ACTIONS] {
    let mut jumps = [u8::MAX; MAX_ACTIONS];
    let mut open = None;
    let mut other = None;
    for (i, a) in actions.iter().enumerate() {
        match a {
            Action::IfFacingLedge { .. } | Action::IfFacingOpponent { .. } => {
                open = Some(i);
                other = None;
            }
            Action::ElseFacing => {
                if let Some(start) = open {
                    jumps[start] = (i + 1) as u8;
                    other = Some(i);
                }
            }
            Action::EndFacing => {
                if let Some(start) = open.take() {
                    if let Some(other) = other.take() {
                        jumps[other] = (i + 1) as u8;
                    } else {
                        jumps[start] = (i + 1) as u8;
                    }
                }
            }
            _ => {}
        }
    }
    jumps
}
/// Distance a grounded fighter coasts to rest while braking from `speed`.
fn stopping_distance(ctx: GameplayContext, speed: f32) -> f32 {
    ctx.ground_brake
        .filter(|b| b.is_finite() && *b > 0.)
        .map_or(0., |b| speed * speed / (2. * b))
}
fn ledge_arrived(ctx: GameplayContext, goal: f32, tolerance: f32, brake: f32) -> bool {
    if (ctx.position[0] - goal).abs() > tolerance || !ctx.velocity[0].is_finite() {
        return false;
    }
    brake == 0.
        || ctx.velocity[0] == 0.
        || ctx
            .ground_brake
            .filter(|b| b.is_finite() && *b > 0.)
            .is_some_and(|b| {
                (ctx.position[0] + ctx.velocity[0] * ctx.velocity[0].abs() / (2. * b) - goal).abs()
                    <= tolerance
            })
}
fn ledge_goal(
    ctx: GameplayContext,
    anchor: Option<[f32; 2]>,
    inward: f32,
    offset: f32,
) -> Option<f32> {
    let anchor = anchor.filter(|p| finite2(*p))?;
    if !ctx.position[0].is_finite() || !inward.is_finite() || !offset.is_finite() {
        return None;
    }
    if offset == 0. {
        return Some(anchor[0]);
    }
    let side = if inward.abs() >= 0.5 {
        inward.signum()
    } else {
        let dx = ctx.position[0] - anchor[0];
        if dx.abs() <= 0.001 {
            return None;
        }
        dx.signum()
    };
    let goal = anchor[0] + side * offset;
    goal.is_finite().then_some(goal)
}
fn shield_grab_input(status: i32, out: ControllerOutput, right: [f32; 2]) -> ControllerOutput {
    if out.requested & MAPPER_R != 0 {
        crate::out_of_shield::direct_shield_input(status, out, right)
    } else {
        out
    }
}
fn shielding(status: i32) -> bool {
    matches!(status, GUARD_ON | GUARD | GUARD_DAMAGE)
}
fn condition_met(
    condition: Condition,
    ctx: GameplayContext,
    enemy: Option<Opponent>,
    value: f32,
) -> bool {
    match condition {
        Condition::SelfGrounded => return ctx.situation == situation::GROUND,
        Condition::SelfAirborne => return ctx.situation == situation::AIR,
        _ => {}
    }
    let Some(enemy) = enemy else {
        return false;
    };
    let dx = enemy.position[0] - ctx.position[0];
    match condition {
        Condition::EnemyGrounded => enemy.situation == situation::GROUND,
        Condition::EnemyAirborne => enemy.situation == situation::AIR,
        Condition::EnemyShielding => shielding(enemy.status),
        Condition::EnemyNotShielding => !shielding(enemy.status),
        Condition::EnemyInFront => dx * ctx.facing > 0.001,
        Condition::EnemyBehind => dx * ctx.facing < -0.001,
        Condition::EnemyWithin => value.is_finite() && value >= 0.0 && dx.abs() <= value,
        Condition::EnemyOutside => value.is_finite() && value >= 0.0 && dx.abs() >= value,
        _ => false,
    }
}
// Native statuses: guard-on, guard, guard-off, guard-damage. No new queries.
fn shield_handoff_needed(action: Action, status: i32, ground: i32) -> bool {
    if ground != situation::GROUND || !(GUARD_ON..=GUARD_DAMAGE).contains(&status) {
        return false;
    }
    let Some(Literal {
        buttons,
        stick,
        cstick,
        ..
    }) = literal(action)
    else {
        // Checks/waits and explicit shield actions keep their semantics.
        return matches!(
            action,
            Action::Face(_)
                | Action::Walk { .. }
                | Action::Run { .. }
                | Action::LedgePosition { .. }
                | Action::Drift { .. }
                | Action::Chase { .. }
                | Action::Approach { .. }
        );
    };
    if buttons & SEQUENCE_GUARD_BUTTONS != 0 {
        return false;
    }
    if buttons == 0 && stick == [0.; 2] && cstick == [0.; 2] {
        return false;
    }
    if matches!(status, GUARD_OFF | GUARD_DAMAGE) {
        return true;
    }
    // Direct OOS grab, jump, Up-B and up-smash inputs do not pay shield drop.
    let direct = buttons & MAPPER_R != 0 && stick[1] >= 0.
        || buttons & SEQUENCE_JUMP_BUTTONS != 0 && stick[1] >= 0.
        || buttons & MAPPER_A != 0 && stick[1] >= 0.95
        || buttons & MAPPER_B != 0 && stick[1] > 0.5
        || cstick[1] > 0.5;
    !direct
}
fn cstick_buttons(stick: [f32; 2]) -> u32 {
    (if stick[0] > 0. {
        MAPPER_RIGHT_STICK_RIGHT
    } else if stick[0] < 0. {
        MAPPER_RIGHT_STICK_LEFT
    } else {
        0
    }) | (if stick[1] > 0. {
        MAPPER_RIGHT_STICK_UP
    } else if stick[1] < 0. {
        MAPPER_RIGHT_STICK_DOWN
    } else {
        0
    })
}
fn controls(stick: [f32; 2], requested: u32) -> ControllerOutput {
    ControllerOutput {
        consume_trigger: true,
        owned: SEQUENCE_BUTTONS,
        stick: Some(stick),
        requested,
        ..ControllerOutput::default()
    }
}
fn neutral() -> ControllerOutput {
    controls([0.0; 2], 0)
}
fn guard_hold() -> ControllerOutput {
    controls([0.0; 2], MAPPER_ZR)
}
/// Whether the fighter faces `target_x`; unknown when level with it or when
/// the facing is not a real left/right.
fn facing_toward(ctx: GameplayContext, target_x: f32) -> Option<bool> {
    let dot = (target_x - ctx.position[0]) * ctx.facing;
    (dot.is_finite() && ctx.facing.abs() >= 0.5 && dot.abs() > 0.001).then_some(dot > 0.0)
}
fn facing_saved_ledge(ctx: GameplayContext, anchor: Option<[f32; 2]>) -> Option<bool> {
    let anchor = anchor.filter(|p| finite2(*p))?;
    facing_toward(ctx, anchor[0])
}

fn direction(
    d: Direction,
    ctx: GameplayContext,
    enemy: Opponent,
    anchor: Option<[f32; 2]>,
    inward: f32,
) -> f32 {
    let toward = |x: f32| {
        let dx = x - ctx.position[0];
        if dx.abs() < 0.001 {
            0.0
        } else {
            dx.signum()
        }
    };
    match d {
        Direction::Opponent => toward(enemy.position[0]),
        Direction::AwayFromOpponent => -toward(enemy.position[0]),
        Direction::Ledge => toward(anchor.map_or(enemy.position[0], |p| p[0])),
        Direction::Inward => {
            if inward.abs() < 0.5 {
                0.0
            } else {
                inward.signum()
            }
        }
        Direction::Outward => {
            if inward.abs() < 0.5 {
                0.0
            } else {
                -inward.signum()
            }
        }
        Direction::Facing => ctx.facing.signum(),
        Direction::Backward => -ctx.facing.signum(),
        Direction::SavedLedge => anchor.map_or(0.0, |p| toward(p[0])),
        Direction::AwayFromLedge => anchor.map_or(0.0, |p| -toward(p[0])),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/ledge_trap_file_timeline.rs"]
mod tests;
