//! Ordered file inputs: one cursor, one native-progress clock, no move lookup.
//! Simple inputs never query action readiness, hitboxes, or acceptance. Only
//! explicitly spatial steps use already-copied opponent/ground observations.
use super::{
    attempt::NativeAttempt, ControllerOutput, GameplayContext, LedgeSettings, Opponent,
    SEQUENCE_BUTTONS,
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
    pub fn confirm_initial_up_b(&mut self, ctx: GameplayContext, enemy: Opponent) {
        self.clock
            .advance_pause(ctx, Some(enemy), self.sequence.timeout);
        self.cursor = 1;
        self.step_start = self.clock.elapsed();
        self.entered = false;
        self.done = false;
        self.output = ControllerOutput::default();
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
            output: ControllerOutput::default(),
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
    fn finish(&mut self, _reason: &'static str) -> ControllerOutput {
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
        ctx: GameplayContext,
        enemy: Option<Opponent>,
        anchor: Option<[f32; 2]>,
        inward: f32,
    ) -> ControllerOutput {
        if self.grounded_automatic_actions
            && (ctx.situation != 0 || ctx.status == crate::native_states::status::JUMP_SQUAT)
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
            self.output = ControllerOutput::default();
            return self.output;
        }
        let out = self.sample_inner(settings, ctx, enemy, anchor, inward);
        if self.grounded_automatic_actions {
            let grab = out.requested & crate::native_input::MAPPER_R != 0;
            let special = out.requested & crate::native_input::MAPPER_B != 0
                && out.stick.is_some_and(|s| s[1] > 0.5)
                && !crate::native_states::up_special_status(settings.fighter_kind, ctx.status);
            if (grab || special)
                && (ctx.situation != 0 || ctx.status == crate::native_states::status::JUMP_SQUAT)
            {
                self.finish("automatic_ground_action_became_airborne");
                self.output = ControllerOutput::default();
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
        ctx: GameplayContext,
        enemy: Option<Opponent>,
        anchor: Option<[f32; 2]>,
        inward: f32,
    ) -> ControllerOutput {
        // Death, ordinary capture and hitstun end this visit. Attack recovery,
        // jump squat, air states, shield and special phases do not gate inputs.
        if crate::action_owner::native_grab_active(ctx.status) {
            self.finish("native_grab_handoff");
            self.output = ControllerOutput::default();
            return self.output;
        }
        if crate::native_states::fighter_sequence_interrupted(settings.fighter_kind, ctx.status) {
            self.finish("scope_end_or_damage");
            // Damage/capture already ended this tape. Return control now,
            // rather than blanking one more packet of buffered human inputs.
            self.output = ControllerOutput::default();
            return self.output;
        }
        if self.done || !ctx.gameplay || !ctx.held {
            return self.finish("scope_end_or_damage");
        }
        // Ordinary physical input is masked throughout the tape, not treated
        // as cancellation. Only explicit until-manual holds hand back on intent.
        if self.sequence.ground_only && ctx.situation != 0 {
            self.finish("ground_only_airborne");
            self.output = ControllerOutput::default();
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
                self.output = ControllerOutput::default(); // Physical input owns this very sample.
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
                self.blocked |= ctx.status == crate::native_states::GUARD_DAMAGE;
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
                        self.output = ControllerOutput::default();
                        return self.output;
                    }
                    self.output = controls([0.0; 2], crate::native_input::MAPPER_ZR);
                    return self.output;
                }
                Action::ShieldUntilBlock(frames) => {
                    self.blocked |= ctx.status == crate::native_states::GUARD_DAMAGE;
                    if self.blocked
                        && matches!(
                            ctx.status,
                            crate::native_states::GUARD_ON | crate::native_states::GUARD
                        )
                    {
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
                            self.output = ControllerOutput::default();
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
                        if matches!(
                            ctx.status,
                            crate::native_states::GUARD_ON
                                | crate::native_states::GUARD
                                | crate::native_states::GUARD_DAMAGE
                        ) {
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
fn shield_grab_input(status: i32, out: ControllerOutput, right: [f32; 2]) -> ControllerOutput {
    if out.requested & crate::native_input::MAPPER_R != 0 {
        crate::out_of_shield::direct_shield_input(status, out, right)
    } else {
        out
    }
}
fn condition_met(
    condition: Condition,
    ctx: GameplayContext,
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
        Condition::EnemyShielding => matches!(
            enemy.status,
            crate::native_states::GUARD_ON
                | crate::native_states::GUARD
                | crate::native_states::GUARD_DAMAGE
        ),
        Condition::EnemyNotShielding => !matches!(
            enemy.status,
            crate::native_states::GUARD_ON
                | crate::native_states::GUARD
                | crate::native_states::GUARD_DAMAGE
        ),
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
    if situation != 0
        || !matches!(
            status,
            crate::native_states::GUARD_ON..=crate::native_states::GUARD_DAMAGE
        )
    {
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
    if matches!(
        status,
        crate::native_states::GUARD_OFF | crate::native_states::GUARD_DAMAGE
    ) {
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
fn controls(stick: [f32; 2], requested: u32) -> ControllerOutput {
    ControllerOutput {
        consume_trigger: true,
        owned: SEQUENCE_BUTTONS,
        stick: Some(stick),
        requested,
        ..ControllerOutput::default()
    }
}
fn facing_saved_ledge(ctx: GameplayContext, anchor: Option<[f32; 2]>) -> Option<bool> {
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
#[path = "file_timeline/tests.rs"]
mod tests;
