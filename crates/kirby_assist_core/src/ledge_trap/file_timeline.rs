//! Ordered file inputs: one cursor, one native-progress clock, no move lookup.
//! Simple inputs never query action readiness, hitboxes, or acceptance. Only
//! explicitly spatial steps use already-copied opponent/ground observations.
use super::{
    attempt::NativeAttempt, LedgeContext, LedgeOutput, LedgeSettings, Opponent, SEQUENCE_BUTTONS,
};
use crate::sequence_config::{Action, Condition, Direction, RangeCheck, Sequence};

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
    output: LedgeOutput,
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
    facing_jumps: [u8; crate::sequence_config::MAX_ACTIONS],
    #[cfg(feature = "diagnostic-logging")]
    pub(super) end_reason: Option<&'static str>,
}
impl Timeline {
    /// Only automatic tapes reject a pending Grab/Up-B after manual takeoff.
    /// Classify once at acquisition; the per-observation test is a bit mask.
    pub fn guard_automatic_ground_actions(&mut self) {
        self.grounded_automatic_actions = true;
        self.automatic_ground_steps =
            self.sequence
                .actions()
                .iter()
                .enumerate()
                .fold(0, |mask, (index, a)| {
                    let guarded = match a {
                        Action::Input { buttons, stick, .. }
                        | Action::OpponentInput { buttons, stick, .. }
                        | Action::Controller { buttons, stick, .. }
                        | Action::Tap { buttons, stick, .. } => {
                            buttons & crate::native_input::MAPPER_R != 0
                                || buttons & crate::native_input::MAPPER_B != 0 && stick[1] > 0.5
                        }
                        _ => false,
                    };
                    mask | if guarded { 1u32 << index } else { 0 }
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
    pub fn confirm_initial_up_b(&mut self, ctx: LedgeContext, enemy: Opponent) {
        self.clock
            .advance_pause(ctx, Some(enemy), self.sequence.timeout);
        self.cursor = 1;
        self.step_start = self.clock.elapsed();
        self.entered = false;
        self.done = false;
        self.output = LedgeOutput::default();
        self.right_stick = [0.; 2];
    }
    pub fn new(sequence: Sequence, enemy: Opponent) -> Self {
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
            facing_jumps: facing_jumps(sequence.actions()),
            #[cfg(feature = "diagnostic-logging")]
            end_reason: None,
            spatial_steps: sequence.actions().iter().enumerate().fold(
                0,
                |mask, (index, action)| {
                    if matches!(
                        action,
                        Action::OpponentInput { .. }
                            | Action::Face(_)
                            | Action::Walk { .. }
                            | Action::Run { .. }
                            | Action::Chase {..} | Action::Approach { .. }
                            | Action::Drift { .. }
                            | Action::Range(_)
                            | Action::IfFacingOpponent {..}
                            | Action::LedgePosition {..}
                    ) || matches!(action, Action::Controller {reference:crate::sequence_config::InputReference::Opponent,..} | Action::Controller {cstick_reference:crate::sequence_config::InputReference::Opponent,..}) || matches!(action, Action::Check { condition, .. } if condition.needs_enemy()) {
                        mask | (1u32 << index)
                    } else {
                        mask
                    }
                },
            ),
            ground_steps: sequence.actions().iter().enumerate().fold(0, |mask, (index, action)| {
                if matches!(action, Action::Chase {..} | Action::Walk { .. } | Action::Run { .. } | Action::LedgePosition {..} | Action::Approach { airborne: false, .. }) {
                    mask | (1u32 << index)
                } else { mask }
            }),
            entered: false,
            direction: 0.0,
            cstick_direction: 1.0,
            origin: 0.0,
            enemy,
            done: crate::sequence_config::validate_facing_blocks(sequence.actions()).is_err(),
            output: LedgeOutput::default(),
        }
    }
    pub fn needs_ground(&self) -> bool {
        self.cursor < 32 && self.ground_steps >> self.cursor != 0
    }
    pub fn finished(&self) -> bool {
        self.done
    }
    pub fn walking(&self) -> bool {
        !self.done
            && matches!(
                self.sequence.actions().get(usize::from(self.cursor)),
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
            && matches!(
                self.sequence.actions().get(usize::from(self.cursor)),
                Some(Action::Drift { .. })
            )
    }
    /// A literal neutral A hold, while this timeline still requests that input.
    pub fn attack_hold(&self) -> bool {
        !self.done
            && self.entered
            && self.output.requested & crate::native_input::MAPPER_A != 0
            && match self.sequence.actions().get(usize::from(self.cursor)) {
                Some(Action::Input {
                    buttons,
                    stick,
                    frames,
                    ..
                })
                | Some(Action::OpponentInput {
                    buttons,
                    stick,
                    frames,
                }) => *buttons == crate::native_input::MAPPER_A && *stick == [0.; 2] && *frames > 1,
                Some(Action::Controller {
                    buttons,
                    stick,
                    cstick,
                    frames,
                    ..
                }) => {
                    *buttons == crate::native_input::MAPPER_A
                        && *stick == [0.; 2]
                        && *cstick == [0.; 2]
                        && *frames > 1
                }
                Some(Action::Tap {
                    buttons,
                    stick,
                    hold,
                    ..
                }) => *buttons == crate::native_input::MAPPER_A && *stick == [0.; 2] && *hold > 1,
                _ => false,
            }
    }
    pub fn right_stick(&self) -> [f32; 2] {
        self.right_stick
    }
    pub fn inputs_only(&self) -> bool {
        self.cursor >= 32 || self.spatial_steps >> self.cursor == 0
    }
    fn finish(&mut self, _reason: &'static str) -> LedgeOutput {
        #[cfg(feature = "diagnostic-logging")]
        {
            self.end_reason.get_or_insert(_reason);
        }
        #[cfg(feature = "probe-counters")]
        if matches!(_reason, "native_clock_gap" | "clock_invalid_or_timeout") {
            crate::probe_counters::record_clock_abort();
        }
        self.done = true;
        self.right_stick = [0.0; 2];
        self.output = controls([0.0; 2], 0);
        self.output
    }
    fn next(&mut self, next_start: f32) {
        self.cursor += 1;
        self.step_start = next_start;
        self.entered = false;
        self.blocked = false;
    }
    #[inline(always)]
    pub fn sample(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        enemy: Option<Opponent>,
        anchor: Option<[f32; 2]>,
        inward: f32,
    ) -> LedgeOutput {
        if self.grounded_automatic_actions
            && (ctx.situation != 0 || ctx.status == 0xa)
            && !crate::native_states::up_special_status(settings.fighter_kind, ctx.status)
            && self
                .automatic_ground_steps
                .checked_shr(u32::from(self.cursor))
                .unwrap_or(0)
                != 0
        {
            // Cancel before a preceding wait/neutral step can keep masking the
            // manual jump. Authored aerial attacks without Grab/Up-B stay literal.
            self.finish("automatic_ground_action_became_airborne");
            self.output = LedgeOutput::default();
            return self.output;
        }
        let out = self.sample_inner(settings, ctx, enemy, anchor, inward);
        if self.grounded_automatic_actions {
            let grab = out.requested & crate::native_input::MAPPER_R != 0;
            let special = out.requested & crate::native_input::MAPPER_B != 0
                && out.stick.is_some_and(|s| s[1] > 0.5)
                && !crate::native_states::up_special_status(settings.fighter_kind, ctx.status);
            if (grab || special) && (ctx.situation != 0 || ctx.status == 0xa) {
                self.finish("automatic_ground_action_became_airborne");
                self.output = LedgeOutput::default();
                return self.output;
            }
        }

        // Keep guard through direct native shield cancels, rather than release shield
        // in the same packet. Apply this only to the returned output: cached
        // literal controls must not retain an injected guard after a status
        // change/takeoff on a duplicate clock sample. No retry or extra frame.
        if ctx.situation == 0 {
            crate::out_of_shield::direct_shield_input(ctx.status, out, self.right_stick)
        } else {
            out
        }
    }
    fn sample_inner(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        enemy: Option<Opponent>,
        anchor: Option<[f32; 2]>,
        inward: f32,
    ) -> LedgeOutput {
        // Death, ordinary capture and hitstun end this visit. Attack recovery,
        // jump squat, air states, shield and special phases do not gate inputs.
        if crate::action_owner::native_grab_active(ctx.status) {
            self.finish("native_grab_handoff");
            self.output = LedgeOutput::default();
            return self.output;
        }
        if crate::native_states::fighter_sequence_interrupted(settings.fighter_kind, ctx.status) {
            self.finish("scope_end_or_damage");
            // Damage/capture already ended this tape. Return control now,
            // rather than blanking one more packet of buffered human inputs.
            self.output = LedgeOutput::default();
            return self.output;
        }
        if self.done || !ctx.gameplay || !ctx.held {
            return self.finish("scope_end_or_damage");
        }
        // Ordinary physical input is masked throughout the tape, not treated
        // as cancellation. Only explicit until-manual holds hand back on intent.
        if self.sequence.ground_only && ctx.situation != 0 {
            self.finish("ground_only_airborne");
            self.output = LedgeOutput::default();
            return self.output;
        }
        // This explicit hold has no elapsed-frame deadline once entered. It
        // never injects an airborne dodge and fresh physical input ends the tape.
        if self.entered
            && matches!(
                self.sequence.actions().get(self.cursor as usize),
                Some(Action::ShieldUntilManual | Action::Input { frames: 0, .. })
            )
        {
            let shield = matches!(
                self.sequence.actions().get(self.cursor as usize),
                Some(Action::ShieldUntilManual)
            ) || self.output.requested & crate::native_input::MAPPER_ZR != 0;
            if shield && ctx.situation != 0 || ctx.manual_input_epoch != self.manual_epoch {
                self.finish("manual_takeover_or_airborne");
                self.output = LedgeOutput::default(); // Physical input owns this very sample.
                return self.output;
            }
            return self.output;
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
        let mut now = self.clock.elapsed();
        // A gap entirely inside an already-held input needs no new edge: the
        // game-visible button stays down. Never catch up across its release,
        // start delay, or another action; missed taps still abort, never burst.
        let inside_hold = self.entered
            && matches!(
                self.sequence.actions().get(usize::from(self.cursor)),
                Some(Action::Input { buttons, frames, .. } | Action::OpponentInput { buttons, frames, .. } | Action::Controller {buttons,frames,..})
                    if *buttons != 0 && now < self.step_start + f32::from(*frames)
            );
        let inside_continuous = self.continuous_progress
            && self.entered
            && matches!(self.sequence.actions().get(usize::from(self.cursor)),
                Some(Action::Drift { frames, .. } | Action::Wait(frames))
                if now < self.step_start + f32::from(*frames));
        let resume_boundary = self.continuous_progress
            && self.entered
            && !self.clock.expired
            && now - before > 1.25
            && match self.sequence.actions().get(usize::from(self.cursor)) {
                Some(
                    Action::Input { frames, .. }
                    | Action::OpponentInput { frames, .. }
                    | Action::Controller { frames, .. }
                    | Action::Drift { frames, .. }
                    | Action::Wait(frames),
                ) if *frames > 0
                    && before < self.step_start + f32::from(*frames)
                    && now >= self.step_start + f32::from(*frames) =>
                {
                    // The previous hold was already delivered. End it on this
                    // observation, rather than spending/skipping the next tap.
                    self.step_start = now - f32::from(*frames);
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
        let waiting_predicate = self.entered
            && matches!(
                self.sequence.actions().get(usize::from(self.cursor)),
                Some(
                    Action::Face(_)
                        | Action::Range(_)
                        | Action::Check { .. }
                        | Action::ShieldUntilBlock(_)
                        | Action::LedgePosition { .. }
                )
            );
        if self.clock.expired
            || now - before > 1.25
                && !(inside_hold
                    || inside_continuous
                    || resume_boundary
                    || waiting_handoff
                    || waiting_predicate)
        {
            return self.finish(if self.clock.expired {
                "clock_invalid_or_timeout"
            } else {
                "native_clock_gap"
            });
        }
        if ctx.stopped {
            if now >= f32::from(self.sequence.start_delay)
                && matches!(
                    self.sequence.actions().get(usize::from(self.cursor)),
                    Some(Action::ShieldUntilBlock(_))
                )
            {
                self.blocked |= ctx.status == 0x1e;
                self.output = controls([0.0; 2], crate::native_input::MAPPER_ZR);
            }
            return self.output;
        }
        // The first observation is frame zero. Later duplicate polls reuse the
        // output even if the mapper tick or opponent metadata changes.
        let predicate_step = matches!(
            self.sequence.actions().get(usize::from(self.cursor)),
            Some(
                Action::Face(_)
                    | Action::Walk { .. }
                    | Action::Run { .. }
                    | Action::Chase { .. }
                    | Action::Approach { .. }
                    | Action::Drift { .. }
                    | Action::Range(_)
                    | Action::LedgePosition { .. }
                    | Action::ShieldUntilBlock(_)
                    | Action::Check { .. }
            )
        );
        if self.entered && now == before && !predicate_step {
            return self.output;
        }
        self.right_stick = [0.0; 2];
        let fresh_enemy = enemy.filter(|enemy| {
            enemy.id == self.enemy.id && enemy.position.iter().all(|v| v.is_finite())
        });
        if let Some(enemy) = fresh_enemy {
            self.enemy = enemy;
        }
        if now < f32::from(self.sequence.start_delay) {
            self.output = controls([0.0; 2], 0);
            return self.output;
        }
        // At most MAX_ACTIONS transitions, including zero-waits and already
        // satisfied spatial conditions. No unbounded per-frame interpreter loop.
        for _ in 0..=crate::sequence_config::MAX_ACTIONS {
            let Some(&action) = self.sequence.actions().get(usize::from(self.cursor)) else {
                if self.sequence.while_held
                    && self.step_start > 0.0
                    && !self.sequence.actions().is_empty()
                {
                    let duration = self.step_start;
                    self.clock.consume(duration);
                    now -= duration;
                    self.step_start = f32::from(self.sequence.start_delay);
                    self.cursor = 0;
                    self.ledge_facing = facing_saved_ledge(ctx, anchor);
                    self.entered = false;
                    if now < self.step_start {
                        self.output = controls([0.; 2], 0);
                        return self.output;
                    }
                    continue;
                }
                return self.finish("completed");
            };
            if let Action::LedgePosition {
                offset,
                strength,
                tolerance,
                brake,
                frames,
            } = action
            {
                if ctx.situation != 0 {
                    return self.finish("ledge_position_requires_ground");
                }
                if !self.entered
                    && (!(0. ..=200.).contains(&offset)
                        || !(0.3..=1.).contains(&strength)
                        || !(0. ..=20.).contains(&tolerance)
                        || !(0. ..=100.).contains(&brake)
                        || !(1..=3600).contains(&frames))
                {
                    return self.finish("invalid_ledge_position");
                }
            }
            if matches!(
                action,
                Action::LedgePosition { .. }
                    | Action::Face(Direction::SavedLedge | Direction::AwayFromLedge)
                    | Action::Walk {
                        direction: Direction::SavedLedge | Direction::AwayFromLedge,
                        ..
                    }
                    | Action::Run {
                        direction: Direction::SavedLedge | Direction::AwayFromLedge,
                        ..
                    }
                    | Action::Drift {
                        direction: Direction::SavedLedge | Direction::AwayFromLedge,
                        ..
                    }
            ) && anchor.filter(|p| p.iter().all(|v| v.is_finite())).is_none()
            {
                return self.finish("saved_ledge_unavailable");
            }
            if !self.entered
                && matches!(action,Action::Chase{gap,..} if (self.enemy.position[0]-ctx.position[0]).abs()<=gap)
            {
                self.next(now);
                continue; // Near branch must preserve direct Up-B out of shield.
            }
            // Satisfied spatial checks do not perform an input. Resolve them
            // before shield handoff so a no-op cannot pay shield-drop recovery.
            if !self.entered {
                let satisfied = match action {
                    Action::Face(d) => {
                        let desired = direction(d, ctx, self.enemy, anchor, inward);
                        desired == 0. || ctx.facing * desired > 0.5
                    }
                    Action::LedgePosition {
                        offset,
                        tolerance,
                        brake,
                        ..
                    } => ledge_goal(ctx, anchor, inward, offset)
                        .is_some_and(|goal| ledge_arrived(ctx, goal, tolerance, brake)),
                    Action::Approach { gap, .. } => {
                        (self.enemy.position[0] - ctx.position[0]).abs() <= gap
                    }
                    _ => false,
                };
                if satisfied {
                    self.next(now);
                    continue;
                }
            }
            // Drop shield with neutral controls before ordinary attacks/movement.
            // The action's own duration starts only after native guard-off ends.
            // This shared tape path covers ledge/button recipes and all assists.
            if !self.entered && shield_handoff_needed(action, ctx.status, ctx.situation) {
                self.shield_handoff = true;
                self.output = controls([0.; 2], 0);
                return self.output;
            }
            if self.shield_handoff {
                self.shield_handoff = false;
                self.step_start = now;
            }
            if !self.entered {
                self.entered = true;
                self.manual_epoch = ctx.manual_input_epoch;
                self.origin = ctx.position[0];
                if let Action::LedgePosition { offset, .. } = action {
                    let Some(goal) = ledge_goal(ctx, anchor, inward, offset) else {
                        return self.finish("ledge_position_unavailable");
                    };
                    self.origin = goal;
                }

                if let Action::Chase { gap, .. } = action {
                    self.origin = if (self.enemy.position[0] - ctx.position[0]).abs() > gap {
                        1.
                    } else {
                        0.
                    };
                }

                self.direction = match action {
                    Action::Controller {
                        reference,
                        cstick_reference,
                        ..
                    } => {
                        let axis = |r| match r {
                            crate::sequence_config::InputReference::Screen => 1.,
                            crate::sequence_config::InputReference::Facing => ctx.facing.signum(),
                            crate::sequence_config::InputReference::Opponent => {
                                direction(Direction::Opponent, ctx, self.enemy, anchor, inward)
                            }
                        };
                        self.cstick_direction = axis(cstick_reference);
                        axis(reference)
                    }

                    Action::OpponentInput { .. } => {
                        direction(Direction::Opponent, ctx, self.enemy, anchor, inward)
                    }
                    Action::Input { relative: true, .. } | Action::Tap { .. } => {
                        ctx.facing.signum()
                    }
                    // Grounded backward stick can turn the fighter around.
                    // Keep this action's initial direction rather than chasing
                    // that turnaround and alternating left/right each update.
                    Action::Drift {
                        direction: Direction::Backward,
                        ..
                    } => -ctx.facing.signum(),
                    Action::Face(d)
                    | Action::Walk { direction: d, .. }
                    | Action::Run { direction: d, .. } => {
                        direction(d, ctx, self.enemy, anchor, inward)
                    }
                    _ => 0.0,
                };
            }
            let elapsed = now - self.step_start;
            let relative = matches!(
                action,
                Action::Input { relative: true, .. } | Action::OpponentInput { .. }
            );
            match action {
                Action::IfFacingLedge { away } | Action::IfFacingOpponent { away } => {
                    let jump = self.facing_jumps[usize::from(self.cursor)];
                    if jump == u8::MAX {
                        return self.finish("invalid_facing_block");
                    }
                    let toward = if matches!(action, Action::IfFacingOpponent { .. }) {
                        let Some(enemy) = fresh_enemy else {
                            return self.finish("facing_opponent_unavailable");
                        };
                        let dot = (enemy.position[0] - ctx.position[0]) * ctx.facing;
                        if !dot.is_finite()
                            || !ctx.facing.is_finite()
                            || ctx.facing.abs() < 0.5
                            || dot.abs() <= 0.001
                        {
                            return self.finish("facing_opponent_unavailable");
                        }
                        dot > 0.
                    } else {
                        let Some(toward) = self.ledge_facing else {
                            return self.finish("saved_ledge_facing_unavailable");
                        };
                        toward
                    };
                    if toward != away {
                        self.next(now);
                    } else {
                        self.cursor = jump;
                        self.step_start = now;
                        self.entered = false;
                        self.blocked = false;
                    }
                }
                Action::ElseFacing => {
                    let jump = self.facing_jumps[usize::from(self.cursor)];
                    if jump == u8::MAX {
                        return self.finish("invalid_facing_block");
                    }
                    self.cursor = jump;
                    self.step_start = now;
                    self.entered = false;
                    self.blocked = false;
                }
                Action::EndFacing => self.next(now),
                Action::Empty => self.next(now),
                Action::Check {
                    condition,
                    value,
                    wait,
                } => {
                    if condition_met(condition, ctx, fresh_enemy, value) {
                        self.next(now);
                        continue;
                    }
                    if elapsed >= f32::from(wait) {
                        return self.finish("condition_failed_or_timeout");
                    }
                    self.output = controls([0.0; 2], 0);
                    return self.output;
                }
                Action::ShieldUntilManual => {
                    if ctx.situation != 0 {
                        self.finish("shield_requires_ground");
                        self.output = LedgeOutput::default();
                        return self.output;
                    }
                    self.output = controls([0.0; 2], crate::native_input::MAPPER_ZR);
                    return self.output;
                }
                Action::ShieldUntilBlock(frames) => {
                    self.blocked |= ctx.status == 0x1e;
                    if self.blocked && matches!(ctx.status, 0x1b | 0x1c) {
                        self.next(now);
                        continue;
                    }
                    if elapsed >= f32::from(frames) {
                        return self.finish("shield_wait_timeout");
                    }
                    self.output = controls([0.0; 2], crate::native_input::MAPPER_ZR);
                    return self.output;
                }
                Action::Input {
                    buttons,
                    mut stick,
                    frames,
                    ..
                }
                | Action::OpponentInput {
                    buttons,
                    mut stick,
                    frames,
                } => {
                    if matches!(action, Action::OpponentInput { .. })
                        && (fresh_enemy.is_none() || frames == 0)
                    {
                        return self.finish("opponent_input_target_unavailable");
                    }
                    if frames == 0 {
                        if buttons == 0
                            || buttons & crate::native_input::MAPPER_ZR != 0 && ctx.situation != 0
                        {
                            self.finish("invalid_manual_hold_scope");
                            self.output = LedgeOutput::default();
                            return self.output;
                        }
                        if relative {
                            stick[0] *= self.direction;
                        }
                        self.output =
                            shield_grab_input(ctx.status, controls(stick, buttons), [0.; 2]);
                        return self.output;
                    }
                    if elapsed >= f32::from(frames) {
                        self.next(self.step_start + f32::from(frames));
                        continue;
                    }
                    if relative {
                        stick[0] *= self.direction;
                    }
                    self.output = shield_grab_input(ctx.status, controls(stick, buttons), [0.; 2]);
                    return self.output;
                }
                Action::Tap {
                    buttons,
                    mut stick,
                    hold,
                    gap,
                    frames,
                } => {
                    if elapsed >= f32::from(frames) {
                        self.next(self.step_start + f32::from(frames));
                        continue;
                    }
                    let phase = elapsed.floor() as u16 % (u16::from(hold) + u16::from(gap));
                    self.output = if phase < u16::from(hold) {
                        stick[0] *= self.direction;
                        shield_grab_input(ctx.status, controls(stick, buttons), [0.; 2])
                    } else {
                        controls([0.; 2], 0)
                    };
                    return self.output;
                }
                Action::Controller {
                    buttons,
                    mut stick,
                    mut cstick,
                    frames,
                    reference,
                    cstick_reference,
                } => {
                    if matches!(reference, crate::sequence_config::InputReference::Opponent)
                        || matches!(
                            cstick_reference,
                            crate::sequence_config::InputReference::Opponent
                        )
                    {
                        if fresh_enemy.is_none() {
                            return self.finish("controller_target_unavailable");
                        }
                    }
                    if frames == 0 {
                        return self.finish("invalid_controller_duration");
                    }
                    if elapsed >= f32::from(frames) {
                        self.next(self.step_start + f32::from(frames));
                        continue;
                    }
                    stick[0] *= self.direction;
                    cstick[0] *= self.cstick_direction;
                    self.right_stick = cstick;
                    self.output = shield_grab_input(
                        ctx.status,
                        controls(stick, buttons | cstick_buttons(cstick)),
                        cstick,
                    );
                    return self.output;
                }
                Action::CStick { stick, frames } => {
                    if elapsed >= f32::from(frames) {
                        self.next(self.step_start + f32::from(frames));
                        continue;
                    }
                    self.right_stick = stick;
                    let buttons = cstick_buttons(stick);
                    self.output = controls([0.0; 2], buttons);
                    return self.output;
                }
                Action::Wait(frames) => {
                    if elapsed >= f32::from(frames) {
                        self.next(self.step_start + f32::from(frames));
                        continue;
                    }
                    self.output = controls([0.0; 2], 0);
                    return self.output;
                }
                Action::Drift {
                    direction: d,
                    strength,
                    frames,
                } => {
                    if elapsed >= f32::from(frames) {
                        self.next(self.step_start + f32::from(frames));
                        continue;
                    }
                    self.output = controls(
                        [
                            if d == Direction::Backward {
                                self.direction * strength
                            } else {
                                direction(d, ctx, self.enemy, anchor, inward) * strength
                            },
                            0.0,
                        ],
                        0,
                    );
                    return self.output;
                }
                Action::Face(d) => {
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
                        self.direction = direction(d, ctx, self.enemy, anchor, inward);
                    }
                    if self.direction == 0.0 || ctx.facing * self.direction > 0.5 {
                        self.next(now);
                        continue;
                    }
                    if elapsed >= f32::from(self.sequence.spatial_timeout) {
                        return self.finish("face_timeout");
                    }
                    self.output = controls([self.direction * 0.3, 0.0], 0);
                    return self.output;
                }
                Action::Walk {
                    units,
                    strength,
                    min_gap,
                    ..
                } => {
                    let distance = (ctx.position[0] - self.origin) * self.direction;
                    let gap = (self.enemy.position[0] - ctx.position[0]) * self.direction;
                    let speed = (ctx.velocity[0] * self.direction).max(0.0);
                    let braking = ctx
                        .ground_brake
                        .filter(|v| v.is_finite() && *v > 0.0)
                        .map_or(0.0, |b| speed * speed / (2.0 * b));
                    let stopping = if min_gap.is_some() {
                        speed + braking
                    } else {
                        0.0
                    };
                    if self.direction == 0.0
                        || distance + stopping >= units
                        || min_gap.is_some_and(|min| gap <= min + stopping)
                    {
                        self.next(now);
                        continue;
                    }
                    if elapsed >= f32::from(self.sequence.spatial_timeout)
                        || ctx.situation != 0
                        || !ctx.safe_ground[usize::from(self.direction > 0.0)]
                    {
                        return self.finish("unsafe_ground_or_air");
                    }
                    self.output = controls([self.direction * strength, 0.0], 0);
                    return self.output;
                }
                Action::LedgePosition {
                    strength,
                    tolerance,
                    brake,
                    frames,
                    ..
                } => {
                    let dx = self.origin - ctx.position[0];
                    if !dx.is_finite() || !ctx.velocity[0].is_finite() {
                        return self.finish("ledge_position_unavailable");
                    }
                    if ledge_arrived(ctx, self.origin, tolerance, brake) {
                        self.next(now);
                        continue;
                    }
                    let dir = dx.signum();
                    if ctx.situation != 0
                        || !ctx.safe_ground[usize::from(dir > 0.)]
                        || elapsed >= f32::from(frames)
                    {
                        return self.finish("ledge_position_timeout_or_unsafe_ground");
                    }
                    let speed = (ctx.velocity[0] * dir).max(0.);
                    let stopping = ctx
                        .ground_brake
                        .filter(|b| b.is_finite() && *b > 0.)
                        .map_or(0., |b| speed * speed / (2. * b));
                    let input = if brake > 0.
                        && (dx.abs() <= tolerance
                            || speed > 0. && dx.abs() <= stopping + speed + tolerance)
                    {
                        0.
                    } else if brake > 0. && dx.abs() < brake {
                        (strength * dx.abs() / brake).clamp(0.3, strength)
                    } else {
                        strength
                    };
                    self.output = controls([dir * input, 0.], 0);
                    return self.output;
                }
                Action::Run { units, .. } => {
                    if self.direction == 0.0
                        || (ctx.position[0] - self.origin) * self.direction >= units
                    {
                        self.next(now);
                        continue;
                    }
                    if elapsed >= f32::from(self.sequence.spatial_timeout)
                        || ctx.situation != 0
                        || !ctx.safe_ground[usize::from(self.direction > 0.0)]
                    {
                        return self.finish("walk_timeout");
                    }
                    self.output = controls([self.direction, 0.0], 0);
                    return self.output;
                }
                Action::Chase {
                    strength, frames, ..
                } => {
                    if self.origin == 0. {
                        self.next(now);
                        continue;
                    }
                    if elapsed >= f32::from(frames) {
                        self.next(self.step_start + f32::from(frames));
                        continue;
                    }
                    let dx = self.enemy.position[0] - ctx.position[0];
                    if ctx.situation != 0 || !ctx.safe_ground[usize::from(dx > 0.)] {
                        return self.finish("unsafe_chase");
                    }
                    self.output = controls([dx.signum() * strength, 0.], 0);
                    return self.output;
                }
                Action::Approach {
                    airborne,
                    strength,
                    frames,
                    gap,
                } => {
                    let dx = self.enemy.position[0] - ctx.position[0];
                    if dx.abs() <= gap || elapsed >= f32::from(frames) {
                        self.next(now);
                        continue;
                    }
                    // A file may place an air approach immediately after Jump;
                    // wait through squat, spending the explicitly configured time.
                    if airborne && ctx.situation != 2 {
                        self.output = controls([0.0; 2], 0);
                    } else {
                        if !airborne
                            && (ctx.situation != 0 || !ctx.safe_ground[usize::from(dx > 0.0)])
                        {
                            return self.finish("unsafe_approach");
                        }
                        self.output = controls([dx.signum() * strength, 0.0], 0);
                    }
                    return self.output;
                }
                Action::Range(range) => {
                    let max = match range {
                        RangeCheck::Ignore => f32::MAX,
                        RangeCheck::Verify(value) => value,
                        RangeCheck::FighterGrabRange | RangeCheck::FighterGrabRangeInstant => {
                            let reach = settings.fighter.grab.standing_range;
                            if !reach.is_finite() || reach <= 0.0 {
                                return self.finish("range_check_failed");
                            }
                            reach
                        }
                    };
                    // Immediate checks require this observation's target. Waiting
                    // ledge checks retain their existing last-position behavior.
                    if range == RangeCheck::FighterGrabRangeInstant && fresh_enemy.is_none() {
                        return self.finish("range_check_failed");
                    }
                    if (self.enemy.position[0] - ctx.position[0]).abs() <= max {
                        self.next(now);
                        continue;
                    }
                    if range == RangeCheck::FighterGrabRangeInstant
                        || elapsed >= f32::from(self.sequence.spatial_timeout)
                    {
                        return self.finish("range_check_failed");
                    }
                    self.output = controls(
                        [0.0; 2],
                        if matches!(ctx.status, 0x1b | 0x1c | 0x1e) {
                            crate::native_input::MAPPER_ZR
                        } else {
                            0
                        },
                    );
                    return self.output;
                }
            }
        }
        self.finish("transition_limit")
    }
}
// Compile flat branch destinations once. Executing a condition is one bounded
// lookup; no per-frame scan, allocation or synthetic input frame is needed.
fn facing_jumps(actions: &[Action]) -> [u8; crate::sequence_config::MAX_ACTIONS] {
    let mut jumps = [u8::MAX; crate::sequence_config::MAX_ACTIONS];
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
fn ledge_arrived(ctx: LedgeContext, goal: f32, tolerance: f32, brake: f32) -> bool {
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
    ctx: LedgeContext,
    anchor: Option<[f32; 2]>,
    inward: f32,
    offset: f32,
) -> Option<f32> {
    let anchor = anchor.filter(|p| p.iter().all(|v| v.is_finite()))?;
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
fn shield_grab_input(status: i32, out: LedgeOutput, right: [f32; 2]) -> LedgeOutput {
    if out.requested & crate::native_input::MAPPER_R != 0 {
        crate::out_of_shield::direct_shield_input(status, out, right)
    } else {
        out
    }
}
fn condition_met(
    condition: Condition,
    ctx: LedgeContext,
    enemy: Option<Opponent>,
    value: f32,
) -> bool {
    if condition == Condition::SelfGrounded {
        return ctx.situation == 0;
    }
    if condition == Condition::SelfAirborne {
        return ctx.situation == 2;
    }
    let Some(enemy) = enemy else {
        return false;
    };
    let dx = enemy.position[0] - ctx.position[0];
    match condition {
        Condition::EnemyGrounded => enemy.situation == 0,
        Condition::EnemyAirborne => enemy.situation == 2,
        Condition::EnemyShielding => matches!(enemy.status, 0x1b | 0x1c | 0x1e),
        Condition::EnemyNotShielding => !matches!(enemy.status, 0x1b | 0x1c | 0x1e),
        Condition::EnemyInFront => dx * ctx.facing > 0.001,
        Condition::EnemyBehind => dx * ctx.facing < -0.001,
        Condition::EnemyWithin => value.is_finite() && value >= 0.0 && dx.abs() <= value,
        Condition::EnemyOutside => value.is_finite() && value >= 0.0 && dx.abs() >= value,
        _ => false,
    }
}
// Native statuses: guard-on, guard, guard-off, guard-damage. No new queries.
fn shield_handoff_needed(action: Action, status: i32, situation: i32) -> bool {
    use crate::native_input::*;
    if situation != 0 || !matches!(status, 0x1b..=0x1e) {
        return false;
    }
    let (buttons, stick, cstick) = match action {
        Action::Input { buttons, stick, .. }
        | Action::OpponentInput { buttons, stick, .. }
        | Action::Tap { buttons, stick, .. } => (buttons, stick, [0.; 2]),
        Action::Controller {
            buttons,
            stick,
            cstick,
            ..
        } => (buttons, stick, cstick),
        Action::CStick { stick, .. } => (0, [0.; 2], stick),
        Action::Face(_)
        | Action::Walk { .. }
        | Action::Run { .. }
        | Action::LedgePosition { .. }
        | Action::Drift { .. }
        | Action::Chase { .. }
        | Action::Approach { .. } => return true,
        _ => return false, // Checks/waits and explicit shield actions keep their semantics.
    };
    if buttons & SEQUENCE_GUARD_BUTTONS != 0 {
        return false;
    }
    if buttons == 0 && stick == [0.; 2] && cstick == [0.; 2] {
        return false;
    }
    if matches!(status, 0x1d | 0x1e) {
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
    use crate::native_input::*;
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
fn controls(stick: [f32; 2], requested: u32) -> LedgeOutput {
    LedgeOutput {
        consume_trigger: true,
        owned: SEQUENCE_BUTTONS,
        stick: Some(stick),
        requested,
        ..LedgeOutput::default()
    }
}
fn facing_saved_ledge(ctx: LedgeContext, anchor: Option<[f32; 2]>) -> Option<bool> {
    let anchor = anchor.filter(|p| p.iter().all(|v| v.is_finite()))?;
    let dot = (anchor[0] - ctx.position[0]) * ctx.facing;
    if !dot.is_finite() || dot.abs() <= 0.001 {
        None
    } else {
        Some(dot > 0.0)
    }
}

fn direction(
    d: Direction,
    ctx: LedgeContext,
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
mod tests {
    use super::*;
    use crate::ledge_trap::{
        tests::{context, enemy},
        LedgeTrap, CLIFF_CLIMB,
    };
    use crate::native_input::*;
    use crate::sequence_config::Catalog;
    fn sequence(actions: &str) -> Sequence {
        Catalog::parse(&format!(
            "[kirby.neutral.timeline]\nstart_delay=0\nactions={actions}"
        ))
        .unwrap()
        .entries()[0]
            .sequence
    }
    fn ctx(frame: f32) -> LedgeContext {
        LedgeContext {
            motion_kind: crate::timing::motion_hash("wait"),
            motion_frame: frame,
            ..context()
        }
    }
    fn sample(timeline: &mut Timeline, context: LedgeContext) -> LedgeOutput {
        timeline.sample(LedgeSettings::default(), context, None, None, 1.0)
    }
    fn timeline(actions: &str) -> Timeline {
        Timeline::new(sequence(actions), enemy(CLIFF_CLIMB, false))
    }
    fn moving_enemy(frame: f32) -> Opponent {
        Opponent {
            motion: Some(crate::ledge_trap::MotionTiming {
                kind: 99,
                frame,
                rate: 1.0,
                remaining: None,
            }),
            ..enemy(CLIFF_CLIMB, false)
        }
    }
    #[test]
    fn while_held_loops_without_extra_frames_and_rebases_clock_until_released() {
        let mut seq = sequence("tap:a:0:-0.6:1:3:8");
        seq.while_held = true;
        seq.timeout = 10;
        let mut t = Timeline::new(seq, enemy(CLIFF_CLIMB, false));
        for f in 0..1000 {
            let out = sample(&mut t, ctx(f as f32));
            assert_eq!(
                out.requested,
                if f % 4 == 0 { MAPPER_A } else { 0 },
                "frame {f}"
            );
            assert!(!t.finished());
        }
        assert_eq!(
            sample(
                &mut t,
                LedgeContext {
                    held: false,
                    ..ctx(1000.)
                }
            )
            .requested,
            0
        );
        assert!(t.finished());
        let mut seq = sequence("tap:a:0:0:1:1:2");
        seq.start_delay = 2;
        seq.while_held = true;
        let mut t = Timeline::new(seq, enemy(CLIFF_CLIMB, false));
        for f in 0..12 {
            assert_eq!(
                sample(&mut t, ctx(f as f32)).requested,
                if f % 4 == 2 { MAPPER_A } else { 0 },
                "delayed frame {f}"
            );
        }
    }
    #[test]
    fn taps_release_between_pulses_pause_on_hitstop_and_continue_to_next_action() {
        let mut t = timeline("tap:a:0:-0.6:2:3:12,hold:b:1");
        for frame in 0..=12 {
            let out = sample(&mut t, ctx(frame as f32));
            let on = frame < 12 && frame % 5 < 2;
            assert_eq!(
                out.requested,
                if frame == 12 {
                    MAPPER_B
                } else if on {
                    MAPPER_A
                } else {
                    0
                },
                "frame {frame}"
            );
            assert_eq!(out.stick, Some(if on { [0., -0.6] } else { [0.; 2] }));
            for _ in 0..4 {
                assert_eq!(sample(&mut t, ctx(frame as f32)), out);
            }
            assert_eq!(
                sample(
                    &mut t,
                    LedgeContext {
                        stopped: true,
                        ..ctx(frame as f32)
                    }
                ),
                out
            );
        }
    }
    #[test]
    fn taps_hold_mapping_only_during_a_pulse_and_never_catch_up_missed_edges() {
        let mut t = timeline("tap:a:0:0:2:2:8");
        for frame in 0..8 {
            sample(&mut t, ctx(frame as f32));
            assert_eq!(t.attack_hold(), frame % 4 < 2);
        }
        let mut t = timeline("tap:a:0:0:1:1:8");
        sample(&mut t, ctx(0.));
        assert_eq!(sample(&mut t, ctx(3.)).requested, 0);
        assert!(t.finished());
    }
    #[test]
    fn explicit_conditions_require_fresh_enemy_and_continue_without_extra_frames() {
        let target = Opponent {
            status: 0x1c,
            ..moving_enemy(0.)
        };
        let mut t = Timeline::new(sequence("check:enemy_shielding,hold:a:4"), target);
        assert!(!t.inputs_only());
        assert!(!t.needs_ground());
        assert_eq!(
            t.sample(LedgeSettings::default(), ctx(0.), Some(target), None, 1.)
                .requested,
            MAPPER_A
        );
        assert!(t.inputs_only());
        for observation in [
            None,
            Some(Opponent {
                id: target.id + 1,
                ..target
            }),
        ] {
            let mut t = Timeline::new(sequence("check:enemy_shielding,hold:a:4"), target);
            assert_eq!(
                t.sample(LedgeSettings::default(), ctx(0.), observation, None, 1.)
                    .requested,
                0
            );
            assert!(t.finished(), "never use the cached or a different enemy");
        }
        let mut t = timeline("check:enemy_shielding:3,hold:a:4");
        assert_eq!(sample(&mut t, ctx(0.)).requested, 0);
        assert_eq!(sample(&mut t, ctx(1.)).requested, 0);
        // A new observation may satisfy a predicate in the same native frame.
        assert_eq!(
            t.sample(LedgeSettings::default(), ctx(1.), Some(target), None, 1.)
                .requested,
            MAPPER_A
        );
        let mut t = timeline("check:enemy_shielding:2,hold:a:4");
        for frame in 0..=2 {
            assert_eq!(sample(&mut t, ctx(frame as f32)).requested, 0);
        }
        assert!(t.finished());
        let mut t = timeline("check:self_grounded,hold:a:4");
        assert!(t.inputs_only());
        assert_eq!(
            sample(
                &mut t,
                LedgeContext {
                    situation: 0,
                    ..ctx(0.)
                }
            )
            .requested,
            MAPPER_A
        );
    }
    #[test]
    fn condition_flags_distance_and_facing_use_only_copied_facts() {
        let own = LedgeContext {
            position: [0., 0.],
            facing: 1.,
            situation: 0,
            ..ctx(0.)
        };
        let target = Opponent {
            position: [10., 0.],
            situation: 2,
            status: 0x1c,
            ..moving_enemy(0.)
        };
        for (condition, expected) in [
            (Condition::SelfGrounded, true),
            (Condition::SelfAirborne, false),
            (Condition::EnemyGrounded, false),
            (Condition::EnemyAirborne, true),
            (Condition::EnemyShielding, true),
            (Condition::EnemyNotShielding, false),
            (Condition::EnemyInFront, true),
            (Condition::EnemyBehind, false),
            (Condition::EnemyWithin, true),
            (Condition::EnemyOutside, true),
        ] {
            assert_eq!(condition_met(condition, own, Some(target), 10.), expected);
        }
        assert!(condition_met(
            Condition::EnemyBehind,
            LedgeContext { facing: -1., ..own },
            Some(target),
            0.
        ));
        assert!(!condition_met(
            Condition::EnemyShielding,
            own,
            Some(Opponent {
                status: 0x1d,
                ..target
            }),
            0.
        ));
        assert!(!condition_met(
            Condition::EnemyWithin,
            own,
            Some(target),
            f32::NAN
        ));
        assert!(timeline("walk:inward:4:50%").needs_ground());
        assert!(timeline("run:inward:4").needs_ground());
        assert!(!timeline("drift:inward:50%:4").needs_ground());
    }
    #[test]
    fn fighter_range_uses_the_selected_fighter_profile() {
        let profiles = crate::fighter_profiles::FighterProfiles::default();
        for (kind, inside, outside) in [(6, 10.0, 13.0)] {
            let mut settings = LedgeSettings::default();
            settings.fighter = profiles.profile(kind).unwrap().ledge;
            for (distance, expected) in [(inside, MAPPER_R), (outside, 0)] {
                let opponent = Opponent {
                    position: [distance, 0.0],
                    ..enemy(CLIFF_CLIMB, true)
                };
                let mut t =
                    Timeline::new(sequence("range:fighter_range,face:opponent,grab"), opponent);
                assert_eq!(
                    t.sample(settings, ctx(0.0), Some(opponent), None, 0.0)
                        .requested,
                    expected
                );
            }
            let opponent = Opponent {
                position: [inside, 0.0],
                ..enemy(CLIFF_CLIMB, true)
            };
            let mut t = Timeline::new(sequence("range:9,grab"), opponent);
            assert_eq!(
                t.sample(settings, ctx(0.0), Some(opponent), None, 0.0)
                    .requested,
                0
            );
        }
    }
    #[test]
    fn instant_grab_range_passes_now_or_cancels_without_a_late_grab() {
        let profiles = crate::fighter_profiles::FighterProfiles::default();
        for kind in [6] {
            let mut settings = LedgeSettings::default();
            settings.fighter = profiles.profile(kind).unwrap().ledge;
            let reach = settings.fighter.grab.standing_range;
            for actions in [
                "range:fighter_range_instant,grab",
                "grab:fighter_range_instant",
            ] {
                for distance in [reach, reach + 1.] {
                    let opponent = Opponent {
                        position: [distance, 0.],
                        ..enemy(CLIFF_CLIMB, true)
                    };
                    let mut t = Timeline::new(sequence(actions), opponent);
                    let out = t.sample(settings, ctx(0.), Some(opponent), None, 0.);
                    if distance <= reach {
                        assert_eq!(
                            out.requested, MAPPER_R,
                            "in-range grab on first observation"
                        );
                        assert!(!t.finished());
                    } else {
                        assert_eq!(out.requested, 0);
                        assert!(t.finished(), "out of range cancels on first observation");
                        let near = Opponent {
                            position: [1., 0.],
                            ..opponent
                        };
                        assert_eq!(
                            t.sample(settings, ctx(1.), Some(near), None, 0.).requested,
                            0
                        );
                    }
                }
                let opponent = enemy(CLIFF_CLIMB, true);
                for observed in [
                    None,
                    Some(Opponent {
                        id: opponent.id + 1,
                        ..opponent
                    }),
                ] {
                    let mut t = Timeline::new(sequence(actions), opponent);
                    assert_eq!(t.sample(settings, ctx(0.), observed, None, 0.).requested, 0);
                    assert!(t.finished(), "missing current target cancels");
                }
            }
            let far = Opponent {
                position: [reach + 1., 0.],
                ..enemy(CLIFF_CLIMB, true)
            };
            let mut waiting = Timeline::new(sequence("range:fighter_range,grab"), far);
            assert_eq!(
                waiting
                    .sample(settings, ctx(0.), Some(far), None, 0.)
                    .requested,
                0
            );
            assert!(!waiting.finished(), "ledge range wait is preserved");
            let near = Opponent {
                position: [reach, 0.],
                ..far
            };
            assert_eq!(
                waiting
                    .sample(settings, ctx(1.), Some(near), None, 0.)
                    .requested,
                MAPPER_R
            );
        }
    }
    #[test]
    fn instant_grab_range_fails_when_fighter_reference_is_unavailable() {
        let profiles = crate::fighter_profiles::FighterProfiles::default();
        let mut settings = LedgeSettings::default();
        settings.fighter = profiles.profile(5).unwrap().ledge;
        assert_eq!(settings.fighter.grab.standing_range, 0.0);
        for actions in [
            "range:fighter_range,grab",
            "range:fighter_range_instant,grab",
            "grab:fighter_range_instant",
        ] {
            let opponent = Opponent {
                position: [0.0, 0.0],
                ..enemy(CLIFF_CLIMB, true)
            };
            let mut timeline = Timeline::new(sequence(actions), opponent);
            let output = timeline.sample(settings, ctx(0.0), Some(opponent), None, 0.0);
            assert_eq!(output.requested, 0);
            assert!(timeline.finished());
        }
    }
    #[test]
    fn movement_rate_changes_do_not_cancel_forward_or_reverse_walks_or_drift() {
        for facing in [-1.0, 1.0] {
            for direction in [-1.0, 1.0] {
                for strength in [0.2, 0.28, 0.3, 0.5, 0.79, 0.95] {
                    for action in [
                        format!("walk:opponent:8:{strength}"),
                        format!("walk_to_opponent:{strength}:3:0"),
                        format!("drift:opponent:{strength}:3"),
                        "run:opponent:8".to_owned(),
                    ] {
                        let opponent = Opponent {
                            position: [direction * 100.0, 0.0],
                            ..enemy(CLIFF_CLIMB, true)
                        };
                        let mut t = Timeline::new(sequence(&format!("{action},grab")), opponent);
                        let mut animation_frame = 0.0;
                        for (tick, rate) in [0.25, 0.5, 2.0, 1.0].into_iter().enumerate() {
                            if tick > 0 {
                                animation_frame += rate;
                            }
                            let c = LedgeContext {
                                facing,
                                status: if action.starts_with("run:") { 4 } else { 1 },
                                position: [
                                    direction * if tick == 3 { 8.0 } else { tick as f32 },
                                    0.0,
                                ],
                                motion_kind: crate::timing::motion_hash("walk_middle"),
                                motion_frame: animation_frame,
                                motion_rate: rate,
                                ..ctx(tick as f32)
                            };
                            let out =
                                t.sample(LedgeSettings::default(), c, Some(opponent), None, 0.0);
                            if tick < 3 {
                                assert!(
                                    !t.finished(),
                                    "{action}: facing {facing}, direction {direction}, tick {tick}"
                                );
                                let expected = if action.starts_with("run:") {
                                    1.0
                                } else {
                                    strength
                                };
                                assert_eq!(out.stick, Some([direction * expected, 0.0]));
                            } else {
                                assert_eq!(out.requested, MAPPER_R, "{action}");
                            }
                            for _ in 0..5 {
                                assert_eq!(
                                    t.sample(
                                        LedgeSettings::default(),
                                        c,
                                        Some(opponent),
                                        None,
                                        0.0
                                    ),
                                    out
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn locomotion_clock_still_rejects_real_gaps_and_does_not_change_attack_clock() {
        for status in [1, 4, 0x24] {
            let mut t = timeline("drift:opponent:0.5:8,grab");
            sample(
                &mut t,
                LedgeContext {
                    status,
                    motion_rate: 0.25,
                    ..ctx(0.0)
                },
            );
            // Two updates at the new rate, not a single rate-adjusted update.
            let out = sample(
                &mut t,
                LedgeContext {
                    status,
                    motion_rate: 2.0,
                    ..ctx(4.0)
                },
            );
            assert!(t.finished());
            assert_eq!(out.requested, 0);
        }
        // Attack clocks retain the existing missed-frame behavior, even when
        // an animation frame delta happens to match a changed rate.
        let mut t = timeline("wait:8,grab");
        sample(
            &mut t,
            LedgeContext {
                status: 0x24,
                motion_rate: 0.25,
                ..ctx(0.0)
            },
        );
        sample(
            &mut t,
            LedgeContext {
                status: 0x24,
                motion_rate: 2.0,
                ..ctx(2.0)
            },
        );
        assert!(t.finished());
    }
    #[test]
    fn file_static_pose_frame_changes_do_not_fabricate_game_frames() {
        let mut t = timeline("wait:1,up_b");
        let own = LedgeContext {
            motion_rate: 0.0,
            ..ctx(5.0)
        };
        assert_eq!(sample(&mut t, own).requested, 0);
        assert_eq!(
            sample(
                &mut t,
                LedgeContext {
                    motion_frame: 3.0,
                    ..own
                }
            )
            .requested,
            0
        );
        assert!(!t.finished());
    }

    #[test]
    fn file_zero_rate_guard_resumes_from_a_block_without_an_extra_timer_frame() {
        let mut t = timeline("shield_until_block:30,grab");
        let guard = LedgeContext {
            status: 0x1c,
            motion_rate: 0.0,
            ..ctx(0.0)
        };
        assert_eq!(sample(&mut t, guard).requested, MAPPER_ZR);
        assert_eq!(
            sample(
                &mut t,
                LedgeContext {
                    status: 0x1e,
                    stopped: true,
                    ..guard
                }
            )
            .requested,
            MAPPER_ZR
        );
        assert_eq!(sample(&mut t, guard).requested, MAPPER_R | MAPPER_ZR);
        assert_eq!(
            sample(&mut t, guard).requested,
            MAPPER_R | MAPPER_ZR,
            "same input holds until a proven native frame advances"
        );
    }

    #[test]
    fn file_zero_rate_pose_uses_locked_native_actor_clock_without_counting_polls() {
        let mut t = timeline("wait:2,up_b");
        for (frame, expected) in [0, 0, MAPPER_B, 0].into_iter().enumerate() {
            let own = LedgeContext {
                motion_rate: 0.0,
                ..ctx(0.0)
            };
            let e = Some(moving_enemy(frame as f32));
            let out = t.sample(LedgeSettings::default(), own, e, None, 1.0);
            assert_eq!(out.requested, expected, "frame {frame}");
            for _ in 0..30 {
                assert_eq!(t.sample(LedgeSettings::default(), own, e, None, 1.0), out);
            }
        }
    }
    #[test]
    fn file_native_clocks_are_not_added_and_missing_samples_are_not_backfilled() {
        let mut t = timeline("wait:3,up_b");
        for frame in 0..4 {
            let out = t.sample(
                LedgeSettings::default(),
                ctx(frame as f32),
                Some(moving_enemy(frame as f32)),
                None,
                1.0,
            );
            assert_eq!(out.requested, if frame == 3 { MAPPER_B } else { 0 });
        }
        let mut t = timeline("wait:1,up_b");
        let own = LedgeContext {
            motion_rate: 0.0,
            ..ctx(0.0)
        };
        t.sample(
            LedgeSettings::default(),
            own,
            Some(moving_enemy(0.0)),
            None,
            1.0,
        );
        assert_eq!(
            t.sample(LedgeSettings::default(), own, None, None, 1.0)
                .requested,
            0
        );
        assert_eq!(
            t.sample(
                LedgeSettings::default(),
                own,
                Some(moving_enemy(10.0)),
                None,
                1.0
            )
            .requested,
            0
        );
        assert_eq!(
            t.sample(
                LedgeSettings::default(),
                own,
                Some(moving_enemy(11.0)),
                None,
                1.0
            )
            .requested,
            MAPPER_B
        );
    }
    #[test]
    fn file_fractional_motion_rate_roundoff_does_not_add_a_wait_frame() {
        let mut t = timeline("wait:5,up_b,wait:2,short_hop_nair");
        let mut motion = 0.0;
        for frame in 0..=10 {
            let out = sample(
                &mut t,
                LedgeContext {
                    motion_rate: 0.1,
                    motion_frame: motion,
                    ..ctx(0.0)
                },
            );
            assert_eq!(
                out.requested,
                match frame {
                    5 => MAPPER_B,
                    8 => MAPPER_X | MAPPER_A,
                    _ => 0,
                },
                "frame {frame}"
            );
            motion += 0.1;
        }
    }

    #[test]
    fn file_observation_demand_drops_after_the_final_spatial_step() {
        let mut t = timeline("face:opponent,shield_until_block:5,grab");
        assert!(!t.inputs_only());
        assert_eq!(sample(&mut t, ctx(0.0)).requested, MAPPER_ZR);
        assert!(
            t.inputs_only(),
            "block/inputs use own status only, without opponent or ground probes"
        );
        assert_eq!(
            sample(
                &mut t,
                LedgeContext {
                    safe_ground: [false; 2],
                    ..ctx(1.0)
                }
            )
            .requested,
            MAPPER_ZR
        );
    }

    #[test]
    fn file_shield_until_block_waits_through_hitstop_and_stun_then_sends_grab() {
        let mut t = timeline("shield_until_block:5, grab");
        assert_eq!(sample(&mut t, ctx(0.0)).requested, MAPPER_ZR);
        let impact = LedgeContext {
            status: 0x1e,
            stopped: true,
            ..ctx(1.0)
        };
        for _ in 0..20 {
            assert_eq!(sample(&mut t, impact).requested, MAPPER_ZR);
        }
        assert_eq!(
            sample(
                &mut t,
                LedgeContext {
                    stopped: false,
                    ..impact
                }
            )
            .requested,
            MAPPER_ZR
        );
        assert_eq!(
            sample(
                &mut t,
                LedgeContext {
                    status: 0x1c,
                    ..ctx(0.0)
                }
            )
            .requested,
            MAPPER_R | MAPPER_ZR
        );
        let mut t = timeline("shield_until_block:2, grab");
        for n in 0..=2 {
            assert_ne!(sample(&mut t, ctx(n as f32)).requested, MAPPER_R);
        }
        assert!(t.finished());
    }

    #[test]
    fn blocked_getup_attack_cancels_shield_into_up_b_on_the_first_guard_frame() {
        for action in [
            "up_b:1",
            "up_special:4",
            "input:b:0:1:4",
            "facing_input:b:0:1:4",
            "opponent_input:b:0:1:4",
            "controller_input:b:0:1:0:0:4",
            "tap:b:0:1:1:1:4",
        ] {
            let opponent = enemy(CLIFF_CLIMB, false);
            let mut t = Timeline::new(
                sequence(&format!("shield_until_block:30,{action}")),
                opponent,
            );
            let call = |t: &mut Timeline, c| {
                t.sample(LedgeSettings::default(), c, Some(opponent), None, 1.)
            };
            assert_eq!(call(&mut t, ctx(0.)).requested, MAPPER_ZR);
            let hit = LedgeContext {
                status: 0x1e,
                stopped: true,
                ..ctx(1.)
            };
            for _ in 0..8 {
                assert_eq!(call(&mut t, hit).requested, MAPPER_ZR);
            }
            assert_eq!(
                call(
                    &mut t,
                    LedgeContext {
                        stopped: false,
                        ..hit
                    }
                )
                .requested,
                MAPPER_ZR
            );
            let ready = LedgeContext {
                status: 0x1c,
                ..ctx(0.)
            };
            let out = call(&mut t, ready);
            assert_eq!(
                out.requested,
                MAPPER_B | MAPPER_ZR,
                "{action}: keep guard through direct OOS Up-B"
            );
            assert_eq!(out.stick, Some([0., 1.]));
            assert_eq!(
                call(&mut t, ready),
                out,
                "duplicate polls must not restart the input"
            );
            // A native status change with no clock advance must not retain a
            // synthetic shield bit once the special has started or is airborne.
            let started = LedgeContext {
                status: 0x1de,
                situation: 2,
                motion_kind: crate::timing::motion_hash("special_hi"),
                ..ctx(0.)
            };
            let after = call(&mut t, started);
            assert_eq!(
                after.requested & MAPPER_ZR,
                0,
                "{action}: no guard leak after acceptance"
            );
            assert!(
                matches!(after.requested, 0 | MAPPER_B),
                "{action}: retain only the configured pulse/gap"
            );
        }
    }

    #[test]
    fn direct_up_b_guard_retention_is_grounded_status_scoped_and_keeps_literal_shields() {
        for status in [0x1b, 0x1c, 0, 0x1de] {
            let mut t = timeline("up_b:4");
            assert_eq!(
                sample(&mut t, LedgeContext { status, ..ctx(0.) }).requested,
                MAPPER_B
                    | if matches!(status, 0x1b | 0x1c) {
                        MAPPER_ZR
                    } else {
                        0
                    }
            );
        }
        // Other specials still use their existing shield-drop path.
        for actions in ["side_special:1", "down_special:1", "neutral_special:1"] {
            let mut t = timeline(actions);
            assert_eq!(
                sample(
                    &mut t,
                    LedgeContext {
                        status: 0x1c,
                        ..ctx(0.)
                    }
                )
                .requested,
                0
            );
        }
        // Explicitly authored guard is not the adapter's temporary guard bit.
        let mut t = timeline("input:b+zr:0:1:4");
        assert_eq!(
            sample(
                &mut t,
                LedgeContext {
                    status: 0x1de,
                    situation: 2,
                    ..ctx(0.)
                }
            )
            .requested,
            MAPPER_B | MAPPER_ZR
        );
    }

    #[test]
    fn timeline_online_status_transitions_and_idle_wraps_preserve_schedule() {
        let mut t = timeline("up_special:2, wait:1, jab");
        assert_eq!(sample(&mut t, ctx(0.0)).requested, MAPPER_B);
        let startup = LedgeContext {
            status: 0x1de,
            motion_kind: crate::timing::motion_hash("special_hi"),
            can_control: false,
            can_act: false,
            ..ctx(0.0)
        };
        assert_eq!(sample(&mut t, startup).requested, MAPPER_B);
        for _ in 0..50 {
            assert_eq!(sample(&mut t, startup).requested, MAPPER_B);
        }
        assert_eq!(
            sample(
                &mut t,
                LedgeContext {
                    motion_frame: 1.0,
                    ..startup
                }
            )
            .requested,
            0
        );
        assert_eq!(
            sample(
                &mut t,
                LedgeContext {
                    status: 0x1fc,
                    motion_kind: crate::timing::motion_hash("special_hi2"),
                    ..startup
                }
            )
            .requested,
            MAPPER_A
        );
        let mut t = timeline("wait:2, up_special");
        assert_eq!(sample(&mut t, ctx(8.0)).requested, 0);
        assert_eq!(sample(&mut t, ctx(9.0)).requested, 0);
        assert_eq!(sample(&mut t, ctx(0.0)).requested, MAPPER_B);
        assert_eq!(sample(&mut t, ctx(0.0)).requested, MAPPER_B);
    }
    #[test]
    fn timeline_missing_file_recipe_never_dispatches_a_compiled_option() {
        let mut trap = LedgeTrap::default();
        let settings = LedgeSettings {
            enabled: true,
            ..LedgeSettings::default()
        };
        for frame in 0..5 {
            assert_eq!(
                trap.sample(
                    settings,
                    frame,
                    ctx(frame as f32),
                    Some(enemy(CLIFF_CLIMB, false))
                )
                .requested,
                0
            );
        }
        assert!(!trap.file_timeline_active());
    }

    #[test]
    fn timeline_spatial_timeout_cancels_without_starting_a_late_attack() {
        let mut s = sequence("face:backward, up_special");
        s.spatial_timeout = 2;
        let mut t = Timeline::new(s, enemy(CLIFF_CLIMB, false));
        for frame in 0..=2 {
            assert_eq!(sample(&mut t, ctx(frame as f32)).requested, 0);
        }
        assert!(t.finished());
    }
    #[test]
    fn backward_drift_keeps_entry_direction_through_turnaround_and_exact_duration() {
        for facing in [-1.0, 1.0] {
            for situation in [0, 2] {
                let mut t = timeline("drift:backward:0.5:3,grab");
                for frame in 0..3 {
                    let c = LedgeContext {
                        facing: if frame == 0 { facing } else { -facing },
                        situation,
                        ..ctx(frame as f32)
                    };
                    assert_eq!(sample(&mut t, c).stick, Some([-facing * 0.5, 0.]));
                    assert_eq!(sample(&mut t, c).stick, Some([-facing * 0.5, 0.]));
                }
                assert_eq!(
                    sample(
                        &mut t,
                        LedgeContext {
                            facing: -facing,
                            situation,
                            ..ctx(3.)
                        }
                    )
                    .requested,
                    MAPPER_R
                );
            }
        }
    }
    #[test]
    fn backward_drift_recaptures_facing_for_each_step_and_leaves_facing_drift_dynamic() {
        let mut t = timeline("drift:backward:0.25:1,wait:1,drift:backward:0.75:1,grab");
        assert_eq!(
            sample(
                &mut t,
                LedgeContext {
                    facing: 1.,
                    ..ctx(0.)
                }
            )
            .stick,
            Some([-0.25, 0.])
        );
        assert_eq!(
            sample(
                &mut t,
                LedgeContext {
                    facing: -1.,
                    ..ctx(1.)
                }
            )
            .stick,
            Some([0., 0.])
        );
        assert_eq!(
            sample(
                &mut t,
                LedgeContext {
                    facing: -1.,
                    ..ctx(2.)
                }
            )
            .stick,
            Some([0.75, 0.])
        );
        assert_eq!(sample(&mut t, ctx(3.)).requested, MAPPER_R);
        let mut t = timeline("drift:facing:0.5:3");
        assert_eq!(
            sample(
                &mut t,
                LedgeContext {
                    facing: 1.,
                    ..ctx(0.)
                }
            )
            .stick,
            Some([0.5, 0.])
        );
        assert_eq!(
            sample(
                &mut t,
                LedgeContext {
                    facing: -1.,
                    ..ctx(1.)
                }
            )
            .stick,
            Some([-0.5, 0.])
        );
    }
    #[test]
    fn timeline_canonical_move_direction_is_captured_at_entry() {
        let mut t = timeline("fair:2");
        assert_eq!(
            sample(
                &mut t,
                LedgeContext {
                    facing: -1.0,
                    ..ctx(0.0)
                }
            )
            .stick,
            Some([-1.0, 0.0])
        );
        assert_eq!(
            sample(
                &mut t,
                LedgeContext {
                    facing: 1.0,
                    ..ctx(1.0)
                }
            )
            .stick,
            Some([-1.0, 0.0])
        );
    }

    #[test]
    fn timeline_inputs_are_immediate_even_without_readiness_or_ground() {
        let mut t = timeline("up_special:2, release, down_special");
        for frame in 0..=1 {
            let out = sample(
                &mut t,
                LedgeContext {
                    situation: 2,
                    status: 0xe,
                    can_act: false,
                    can_control: false,
                    can_jump: false,
                    can_air_attack: false,
                    safe_ground: [false; 2],
                    ..ctx(frame as f32)
                },
            );
            assert_eq!(out.requested, MAPPER_B);
            assert_eq!(out.stick, Some([0.0, 1.0]));
        }
        assert_eq!(sample(&mut t, ctx(2.0)).requested, 0);
        assert_eq!(sample(&mut t, ctx(3.0)).stick, Some([0.0, -1.0]));
        assert_eq!(sample(&mut t, ctx(4.0)).requested, 0);
        assert!(t.finished());
    }
    #[test]
    fn timeline_hop_wait_up_b_and_multiple_attacks_keep_exact_boundaries() {
        let mut t = timeline("short_hop, wait:2, up_special, wait:1, jab:2");
        let expected = [MAPPER_X, 0, 0, MAPPER_B, 0, MAPPER_A, MAPPER_A, 0];
        for (frame, buttons) in expected.into_iter().enumerate() {
            assert_eq!(
                sample(&mut t, ctx(frame as f32)).requested,
                buttons,
                "frame {frame}"
            );
        }
        assert!(t.finished());
    }
    #[test]
    fn timeline_online_delay_and_repeated_polls_do_not_change_frame_schedule() {
        for delay in [0.0, 2.0, 6.0, 12.0] {
            let mut t = timeline("wait:2, up_special:2, release, jab");
            for (frame, expected) in [0, 0, MAPPER_B, MAPPER_B, 0, MAPPER_A, 0]
                .into_iter()
                .enumerate()
            {
                let c = LedgeContext {
                    input_delay: delay,
                    ..ctx(frame as f32)
                };
                let out = sample(&mut t, c);
                assert_eq!(out.requested, expected, "delay {delay}, frame {frame}");
                for _ in 0..40 {
                    assert_eq!(sample(&mut t, c), out);
                }
            }
        }
    }
    #[test]
    fn timeline_hitstop_and_motion_rates_pause_without_losing_button_holds() {
        let mut t = timeline("up_special:2, jab");
        let c = LedgeContext {
            motion_rate: 0.5,
            ..ctx(0.0)
        };
        let held = sample(&mut t, c);
        for _ in 0..20 {
            assert_eq!(sample(&mut t, LedgeContext { stopped: true, ..c }), held);
        }
        assert_eq!(
            sample(
                &mut t,
                LedgeContext {
                    motion_frame: 0.5,
                    ..c
                }
            ),
            held
        );
        assert_eq!(
            sample(
                &mut t,
                LedgeContext {
                    motion_frame: 1.0,
                    ..c
                }
            )
            .requested,
            MAPPER_A
        );
    }
    #[test]
    fn timeline_clock_gaps_invalid_data_damage_and_release_cancel_without_bursting() {
        for changed in [
            ctx(4.0),
            LedgeContext {
                motion_frame: f32::NAN,
                ..ctx(1.0)
            },
            LedgeContext {
                status: 0x49,
                ..ctx(1.0)
            },
            LedgeContext {
                held: false,
                ..ctx(1.0)
            },
        ] {
            let mut t = timeline("wait:2, up_special, jab");
            sample(&mut t, ctx(0.0));
            assert_eq!(sample(&mut t, changed).requested, 0);
            assert!(t.finished());
        }
    }
    #[test]
    fn timeline_start_delay_and_repeated_buttons_have_explicit_release_frames() {
        let mut s = sequence("jab, release, jab");
        s.start_delay = 2;
        let mut t = Timeline::new(s, enemy(CLIFF_CLIMB, false));
        for (frame, expected) in [0, 0, MAPPER_A, 0, MAPPER_A, 0].into_iter().enumerate() {
            assert_eq!(sample(&mut t, ctx(frame as f32)).requested, expected);
        }
    }
    #[test]
    fn timeline_face_captures_direction_and_continues_on_same_confirmed_update() {
        let mut t = timeline("face:backward, jab");
        assert_eq!(sample(&mut t, ctx(0.0)).stick, Some([-0.3, 0.0]));
        let out = sample(
            &mut t,
            LedgeContext {
                facing: -1.0,
                ..ctx(1.0)
            },
        );
        assert_eq!(out.requested, MAPPER_A, "no extra confirmation delay");
        assert_eq!(out.stick, Some([0.0, 0.0]));
    }
    #[test]
    fn timeline_walk_then_wait_preserves_order_and_stops_at_explicit_gap() {
        let target = Opponent {
            position: [10.0, 0.0],
            ..enemy(CLIFF_CLIMB, false)
        };
        let mut t = Timeline::new(
            sequence("walk:opponent:8:0.8:4, wait:2, up_special"),
            target,
        );
        assert_eq!(sample(&mut t, ctx(0.0)).stick, Some([0.8, 0.0]));
        assert_eq!(
            sample(
                &mut t,
                LedgeContext {
                    position: [6.0, 0.0],
                    ..ctx(1.0)
                }
            )
            .requested,
            0
        );
        assert_eq!(sample(&mut t, ctx(2.0)).requested, 0);
        assert_eq!(sample(&mut t, ctx(3.0)).requested, MAPPER_B);
    }
    #[test]
    fn timeline_cstick_has_real_direction_bits_axes_and_duplicate_reuse() {
        let mut t = timeline("cstick:-1:0:1, wait:1");
        let out = sample(&mut t, ctx(0.0));
        assert_eq!(out.requested, MAPPER_RIGHT_STICK_LEFT);
        assert_eq!(t.right_stick(), [-1.0, 0.0]);
        assert_eq!(sample(&mut t, ctx(0.0)), out);
        assert_eq!(t.right_stick(), [-1.0, 0.0]);
        sample(&mut t, ctx(1.0));
        assert_eq!(t.right_stick(), [0.0; 2]);
    }
    #[test]
    fn timeline_dispatch_survives_takeoff_target_loss_and_hit_confirm_then_hands_back() {
        let mut trap = LedgeTrap::default();
        let mut settings = LedgeSettings {
            enabled: true,
            manual_spacing: true,
            ..LedgeSettings::default()
        };
        settings.sequence_overrides[0] = Some(sequence("short_hop, wait:2, up_special"));
        assert_eq!(
            trap.sample(settings, 0, ctx(0.0), Some(enemy(CLIFF_CLIMB, false)))
                .requested,
            MAPPER_X
        );
        for frame in 1..=2 {
            let out = trap.sample(
                settings,
                100 * frame,
                LedgeContext {
                    situation: 2,
                    can_control: false,
                    can_act: false,
                    hit_confirmed: true,
                    ..ctx(frame as f32)
                },
                None,
            );
            assert_eq!(out.requested, 0);
            assert_eq!(out.owned, SEQUENCE_BUTTONS, "wait still owns controls");
        }
        assert_eq!(
            trap.sample(
                settings,
                999,
                LedgeContext {
                    situation: 2,
                    can_control: false,
                    can_act: false,
                    ..ctx(3.0)
                },
                None
            )
            .requested,
            MAPPER_B
        );
        assert_eq!(trap.sample(settings, 1000, ctx(4.0), None).requested, 0);
        assert!(trap.manual_after_response());
        assert_eq!(
            trap.sample(settings, 1001, ctx(5.0), None),
            LedgeOutput::default()
        );
    }
    #[test]
    fn timeline_groundless_raw_dash_does_not_wait_for_native_dash_or_retry() {
        let mut t = timeline("dash_attack:2:1");
        for frame in 0..=1 {
            let out = sample(
                &mut t,
                LedgeContext {
                    can_act: false,
                    safe_ground: [false; 2],
                    ..ctx(frame as f32)
                },
            );
            assert_eq!(out.requested, 0);
            assert_eq!(out.stick, Some([1.0, 0.0]));
        }
        assert_eq!(sample(&mut t, ctx(2.0)).requested, MAPPER_A);
        assert_eq!(sample(&mut t, ctx(3.0)).requested, 0);
    }
    #[cfg(feature = "diagnostic-logging")]
    #[test]
    fn diagnostic_finish_reason_distinguishes_completion_from_missed_native_updates() {
        let mut done = timeline("wait:1,jab");
        for frame in 0..=2 {
            sample(&mut done, ctx(frame as f32));
        }
        assert_eq!(done.end_reason, Some("completed"));
        let mut gap = timeline("wait:8,jab");
        sample(&mut gap, ctx(0.0));
        sample(&mut gap, ctx(3.0));
        assert_eq!(gap.end_reason, Some("native_clock_gap"));
    }
    #[test]
    fn held_a_survives_internal_observation_gaps_without_retriggering() {
        use crate::native_input::{ButtonTrace, MapperButtonOverlay};
        let mut t = timeline("hold:a:60");
        let mut overlay = MapperButtonOverlay::default();
        let first = sample(&mut t, ctx(0.0));
        let edge = overlay.apply_raw(ButtonTrace::default(), first.requested, first.owned);
        assert_eq!(edge.pressed & MAPPER_A, MAPPER_A);
        for frame in [
            1., 4., 7., 8., 12., 16., 20., 24., 28., 32., 36., 40., 44., 48., 52., 56., 59.,
        ] {
            let out = sample(&mut t, ctx(frame));
            assert!(!t.finished());
            let mapped = overlay.apply_raw(ButtonTrace::default(), out.requested, out.owned);
            assert_eq!(mapped.held & MAPPER_A, MAPPER_A);
            assert_eq!(mapped.pressed & MAPPER_A, 0);
            assert_eq!(mapped.released & MAPPER_A, 0);
        }
        let out = sample(&mut t, ctx(60.));
        let mapped = overlay.apply_raw(ButtonTrace::default(), out.requested, out.owned);
        assert_eq!(mapped.held & MAPPER_A, 0);
        assert_eq!(mapped.released & MAPPER_A, MAPPER_A);
        assert!(t.finished());
    }

    #[test]
    fn hold_gap_never_skips_release_or_delayed_press() {
        for recipe in ["hold:a:3,release,hold:a:8", "wait:2,hold:a:60"] {
            let mut t = timeline(recipe);
            sample(&mut t, ctx(0.));
            assert_eq!(sample(&mut t, ctx(4.)).requested, 0);
            assert!(t.finished());
        }
        let mut t = timeline("hold:a:60");
        sample(&mut t, ctx(0.));
        assert_eq!(sample(&mut t, ctx(9.)).requested, 0);
        assert!(t.finished(), "invalid native progress still aborts");
    }
    #[test]
    fn facing_blocks_pick_only_starting_direction_and_add_no_delay() {
        let recipe="if_facing_ledge, input:a:0:0:1, end_facing, if_back_to_ledge, input:b:0:0:1, end_facing";
        for side in [-1.0, 1.0] {
            for toward in [false, true] {
                let mut t = timeline(recipe);
                let mut own = ctx(0.0);
                own.position = [side * 10.0, 0.0];
                own.facing = if toward { -side } else { side };
                let first = t.sample(LedgeSettings::default(), own, None, Some([0.0; 2]), side);
                assert_eq!(first.requested, if toward { MAPPER_A } else { MAPPER_B });
                own.motion_frame = 1.0;
                own.facing = -own.facing;
                let second = t.sample(LedgeSettings::default(), own, None, Some([0.0; 2]), side);
                assert_eq!(
                    second.requested, 0,
                    "Turning must not enable the other block"
                );
                assert!(t.finished());
            }
        }
    }
    #[test]
    fn skipped_block_waits_do_not_consume_time_and_missing_anchor_stops() {
        let mut t = timeline(
            "if_facing_ledge, wait:100, a:4, end_facing, if_back_to_ledge, b:4, end_facing",
        );
        let mut own = ctx(0.0);
        own.position = [10.0, 0.0];
        own.facing = 1.0;
        assert_eq!(
            t.sample(LedgeSettings::default(), own, None, Some([0.0; 2]), 1.0)
                .requested,
            MAPPER_B
        );
        let mut missing = timeline("if_back_to_ledge, a:4, end_facing");
        assert_eq!(sample(&mut missing, own).requested, 0);
        assert!(missing.finished());
    }
    #[test]
    fn explicit_saved_ledge_directions_do_not_fall_back_to_opponent() {
        for token in [
            "drift:saved_ledge:0.25:4",
            "walk:away_from_ledge:4:0.3",
            "run:saved_ledge:4",
            "face:saved_ledge",
        ] {
            let mut t = timeline(token);
            assert_eq!(sample(&mut t, ctx(0.0)).requested, 0);
            assert!(t.finished(), "{token}");
        }
        let mut own = ctx(0.0);
        own.position = [10.0, 0.0];
        own.facing = 1.0;
        let mut toward = timeline("drift:saved_ledge:0.25:4");
        assert_eq!(
            toward
                .sample(LedgeSettings::default(), own, None, Some([0.0; 2]), 1.0)
                .stick,
            Some([-0.25, 0.0])
        );
        let mut away = timeline("drift:away_from_ledge:0.25:4");
        assert_eq!(
            away.sample(LedgeSettings::default(), own, None, Some([0.0; 2]), 1.0)
                .stick,
            Some([0.25, 0.0])
        );
    }
}
