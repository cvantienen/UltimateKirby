//! A bounded, native-observed low-percent route. No status writes or global
//! frame script: grab -> forward throw -> full forward air. Dair is optional.
//! Continuous observed non-actionability is evidence, not proof of a true combo
//! between unobserved simulation frames or against every DI/hurtbox.
use crate::ledge_trap::{LedgeContext, LedgeOutput, Opponent, LEDGE_BUTTONS};
use crate::native_input::{MAPPER_A, MAPPER_X};
use crate::timing::{ActiveWindow, RecoveryWindow};

pub const DAIR_MOTION: u64 = 0x0d40042152;
pub const FAIR_MOTION: u64 = 0x0c3495ada5;
pub const FORWARD_THROW_MOTION: u64 = 0x0749f709a8;
const ATTACK_AIR: i32 = 0x33;
const THROW: i32 = 0x40;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ComboStarter {
    #[default]
    Grab,
    DownAir,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComboSettings {
    pub enabled: bool,
    pub starter: ComboStarter,
    pub max_start_percent: f32,
    pub grab_range: f32,
    pub fair_range: f32,
}
impl Default for ComboSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            starter: ComboStarter::Grab,
            max_start_percent: 0.5,
            grab_range: 6.0,
            fair_range: 12.0,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComboTarget {
    pub fighter: Opponent,
    pub kind: i32,
    pub percent: f32,
    pub actionable: bool,
    pub stopped: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComboContext {
    pub own: LedgeContext,
    pub own_id: u32,
    pub motion_rate: f32,
    pub can_jump: bool,
    pub can_attack_air: bool,
    pub capture_verified: bool,
    pub throw_target: Option<u32>,
    pub target: Option<ComboTarget>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Phase {
    #[default]
    Idle,
    Prepare,
    DairHop,
    Dair,
    DairLanding,
    Grab,
    ThrowSelect,
    ThrowStart,
    ThrowRecovery,
    FairHop,
    Fair,
    Complete,
    Cancelled,
}
#[derive(Default)]
pub struct ComboAttempt {
    phase: Phase,
    reason: &'static str,
    held: bool,
    target: Option<u32>,
    settings: Option<ComboSettings>,
    previous: Option<ComboContext>,
    output: LedgeOutput,
    age: f32,
    seen_grab: bool,
    seen_dair: bool,
    dair_hit: bool,
    fast_fallen: bool,
    fair_hits: [bool; 3],
    previous_percent: f32,
    throw_percent: f32,
}
impl ComboAttempt {
    pub fn target(&self) -> Option<u32> {
        self.target
    }
    pub fn active(&self) -> bool {
        !matches!(self.phase, Phase::Idle | Phase::Complete | Phase::Cancelled)
    }
    pub fn action_name(&self) -> &'static str {
        match self.phase {
            Phase::Idle => "combo_idle",
            Phase::Prepare => "combo_prepare",
            Phase::DairHop => "combo_dair_takeoff",
            Phase::Dair => "combo_dair",
            Phase::DairLanding => "combo_dair_landing",
            Phase::Grab => "combo_grab",
            Phase::ThrowSelect => "combo_capture_confirmed",
            Phase::ThrowStart => "combo_forward_throw_request",
            Phase::ThrowRecovery => "combo_throw_recovery",
            Phase::FairHop => "combo_fair_takeoff",
            Phase::Fair => "combo_forward_air",
            Phase::Complete => "combo_complete",
            Phase::Cancelled => self.reason,
        }
    }
    fn cancel(&mut self, reason: &'static str) -> LedgeOutput {
        self.phase = Phase::Cancelled;
        self.reason = reason;
        self.output = LedgeOutput::default();
        self.output
    }
    fn phase(&mut self, phase: Phase) {
        self.phase = phase;
        self.age = 0.0;
    }

    /// `held` must be supplied by a future explicit trigger; no live key is bound. One attempt per press; release
    /// always cancels, including between duplicate mapper callbacks.
    pub fn sample(
        &mut self,
        settings: ComboSettings,
        held: bool,
        grab_button: u32,
        ctx: ComboContext,
    ) -> LedgeOutput {
        if !settings.enabled || !held {
            *self = Self::default();
            return LedgeOutput::default();
        }
        let pressed = !self.held;
        self.held = true;
        if !ctx.own.gameplay {
            return self.cancel("combo_interrupted");
        }
        if matches!(self.phase, Phase::Complete | Phase::Cancelled) {
            return LedgeOutput::default();
        }
        if !valid(settings, ctx) {
            return self.cancel("combo_invalid_observation");
        }
        if (0x41..=0x5b).contains(&ctx.own.status)
            || matches!(ctx.own.status, 0xb5 | 0xb6 | 0x1d6 | 0x1d9..=0x1db)
        {
            return self.cancel("combo_interrupted");
        }
        let Some(target) = ctx.target else {
            return self.cancel("combo_target_lost");
        };
        if self.target.is_some_and(|id| id != target.fighter.id) {
            return self.cancel("combo_target_changed");
        }
        if self.previous.is_some_and(|old| old.own_id != ctx.own_id) {
            return self.cancel("combo_fighter_changed");
        }
        if self.settings.is_some_and(|old| old != settings) {
            return self.cancel("combo_settings_changed");
        }
        if self.phase == Phase::Idle {
            if !pressed {
                return LedgeOutput::default();
            }
            if ctx.own.situation != 0
                || !ctx.own.can_act
                || ctx.own.stopped
                || target.stopped
                || target.percent > settings.max_start_percent
                || !target.fighter.vulnerable
                || target.fighter.situation != 0
                || !near(ctx, settings.grab_range, [-5.0, 5.0])
            {
                return self.cancel("combo_start_not_ready");
            }
            self.target = Some(target.fighter.id);
            self.settings = Some(settings);
            self.previous_percent = target.percent;
            self.phase(Phase::Prepare);
        }
        if self.previous == Some(ctx) {
            return self.output;
        }
        if let Some(previous) = self.previous {
            let elapsed = native_elapsed(previous, ctx);
            if elapsed > 8.0 {
                return self.cancel("combo_observation_gap");
            }
            if !ctx.own.stopped && !target.stopped {
                self.age += elapsed;
            }
        }
        self.previous = Some(ctx);
        if self.age > 100.0 {
            return self.cancel("combo_phase_timeout");
        }
        let damaged = target.percent > self.previous_percent + 0.001;
        self.previous_percent = target.percent;
        if self.phase == Phase::Dair
            && ctx.own.status == ATTACK_AIR
            && ctx.own.motion_kind == DAIR_MOTION
        {
            self.seen_dair = true;
            self.dair_hit |= damaged && ctx.own.hit_confirmed && in_damage(target.fighter.status);
        }
        if self.phase == Phase::Fair
            && ctx.own.status == ATTACK_AIR
            && ctx.own.motion_kind == FAIR_MOTION
            && damaged
            && ctx.own.hit_confirmed
            && in_damage(target.fighter.status)
        {
            let frame = ctx.own.motion_frame;
            for (i, (first, last)) in [(10.0, 12.0), (17.0, 19.0), (25.0, 28.0)]
                .into_iter()
                .enumerate()
            {
                if (first..last).contains(&frame) {
                    self.fair_hits[i] = true;
                }
            }
        }
        if self.fair_hits.iter().all(|hit| *hit) {
            self.phase(Phase::Complete);
            self.output = LedgeOutput::default();
            return self.output;
        }
        // Once a link begins, an observed actionable target invalidates the
        // continuous combo attempt. Do not chase or continue an escaped route.
        let needs_stun = self.dair_hit
            || matches!(
                self.phase,
                Phase::ThrowRecovery | Phase::FairHop | Phase::Fair
            );
        if needs_stun
            && target.actionable
            && !target.stopped
            && !ctx.own.stopped
            && !ctx.capture_verified
        {
            return self.cancel("combo_target_actionable_gap");
        }
        let mut out = LedgeOutput {
            owned: LEDGE_BUTTONS,
            stick: Some([0.0, 0.0]),
            ..LedgeOutput::default()
        };
        if ctx.own.stopped || target.stopped {
            self.output = out;
            return out;
        }
        match self.phase {
            Phase::Prepare => {
                let dx = target.fighter.position[0] - ctx.own.position[0];
                if ctx.own.status != 0 || !ctx.own.can_act || ctx.own.velocity[0].abs() > 0.12 {
                    self.output = out;
                    return out;
                }
                if dx * ctx.own.facing < -0.5 {
                    if !ctx.own.safe_ground[usize::from(dx > 0.0)] {
                        return self.cancel("combo_unsupported_turn");
                    }
                    out.stick = Some([dx.signum() * 0.3, 0.0]);
                } else if self.output.stick.is_some_and(|s| s[0].abs() > 0.1) {
                    // Release the turn before a fresh directional recipe.
                } else if settings.starter == ComboStarter::DownAir {
                    if !ctx.can_jump {
                        return self.cancel("combo_jump_not_ready");
                    }
                    out.requested = MAPPER_X;
                    self.phase(Phase::DairHop);
                } else {
                    if !ctx.own.can_grab {
                        return self.cancel("combo_grab_not_ready");
                    }
                    if !near_front(ctx, settings.grab_range, [-5.0, 5.0]) {
                        return self.cancel("combo_grab_out_of_reach");
                    }
                    out.requested = grab_button;
                    self.phase(Phase::Grab);
                }
            }
            Phase::DairHop => {
                if ctx.own.situation == 2 && ctx.can_attack_air {
                    out.stick = Some([0.0, -0.6]);
                    out.requested = MAPPER_A;
                    self.phase(Phase::Dair);
                } else if self.age > 8.0 {
                    return self.cancel("combo_takeoff_rejected");
                }
            }
            Phase::Dair => {
                if ctx.own.status == ATTACK_AIR {
                    if ctx.own.motion_kind != DAIR_MOTION {
                        return self.cancel("combo_wrong_aerial");
                    }
                    if self.dair_hit && !self.fast_fallen && ctx.own.velocity[1] < -0.1 {
                        out.stick = Some([0.0, -1.0]);
                        self.fast_fallen = true;
                    }
                } else if ctx.own.situation == 0 && self.seen_dair {
                    if !self.dair_hit {
                        return self.cancel("combo_dair_whiff");
                    }
                    self.phase(Phase::DairLanding);
                } else if self.age > 8.0 && !self.seen_dair {
                    return self.cancel("combo_dair_rejected");
                }
            }
            Phase::DairLanding => {
                if ctx.own.situation == 0 && ctx.own.can_act && ctx.own.can_grab {
                    if !near_front(ctx, settings.grab_range, [-5.0, 5.0]) {
                        return self.cancel("combo_grab_out_of_reach");
                    }
                    out.requested = grab_button;
                    self.phase(Phase::Grab);
                }
            }
            Phase::Grab => {
                self.seen_grab |= matches!(ctx.own.status, 0x37 | 0x39 | 0x3b);
                if ctx.capture_verified && matches!(ctx.own.status, 0x38 | 0x3a | 0x3c) {
                    self.phase(Phase::ThrowSelect);
                } else if self.seen_grab && ctx.own.can_act {
                    return self.cancel("combo_grab_whiff");
                } else if self.age > 15.0 && !self.seen_grab {
                    return self.cancel("combo_grab_rejected");
                }
            }
            Phase::ThrowSelect => {
                if !ctx.capture_verified {
                    return self.cancel("combo_capture_lost");
                }
                if ctx.own.status == 0x3c {
                    out.stick = Some([ctx.own.facing.signum(), 0.0]);
                    self.throw_percent = target.percent;
                    self.phase(Phase::ThrowStart);
                }
            }
            Phase::ThrowStart => {
                if ctx.own.status == THROW {
                    if ctx.own.motion_kind != FORWARD_THROW_MOTION
                        || ctx.throw_target != self.target
                    {
                        return self.cancel("combo_wrong_throw");
                    }
                    self.phase(Phase::ThrowRecovery);
                } else if self.age > 8.0 {
                    return self.cancel("combo_throw_rejected");
                }
            }
            Phase::ThrowRecovery => {
                let released = target.percent > self.throw_percent + 0.001 && !ctx.capture_verified;
                if released && ctx.own.situation == 2 && ctx.can_attack_air {
                    if !fair_reach(settings, ctx, 0) {
                        return self.cancel("combo_fair_out_of_reach");
                    }
                    out.stick = Some([ctx.own.facing.signum() * 0.6, 0.0]);
                    out.requested = MAPPER_A;
                    self.phase(Phase::Fair);
                } else if released && ctx.own.situation == 0 && ctx.can_jump && ctx.own.can_act {
                    if !fair_reach(settings, ctx, 3) {
                        return self.cancel("combo_fair_out_of_reach");
                    }
                    out.requested = MAPPER_X;
                    self.phase(Phase::FairHop);
                }
            }
            Phase::FairHop => {
                if ctx.own.situation == 2 && ctx.can_attack_air {
                    if !fair_reach(settings, ctx, 0) {
                        return self.cancel("combo_fair_out_of_reach");
                    }
                    out.stick = Some([ctx.own.facing.signum() * 0.6, 0.0]);
                    out.requested = MAPPER_A;
                    self.phase(Phase::Fair);
                } else if self.age > 8.0 {
                    return self.cancel("combo_takeoff_rejected");
                }
            }
            Phase::Fair => {
                if ctx.own.status == ATTACK_AIR {
                    if ctx.own.motion_kind != FAIR_MOTION {
                        return self.cancel("combo_wrong_aerial");
                    }
                    if ctx.own.motion_frame >= 28.0 {
                        return self.cancel("combo_fair_hits_unconfirmed");
                    }
                } else if self.age > 8.0 {
                    return self.cancel("combo_fair_rejected_or_landed");
                }
            }
            _ => {}
        }
        self.output = out;
        out
    }
}
fn in_damage(status: i32) -> bool {
    (0x47..=0x4f).contains(&status)
}
fn near(ctx: ComboContext, range: f32, height: [f32; 2]) -> bool {
    ctx.target.is_some_and(|target| {
        let dx = target.fighter.position[0] - ctx.own.position[0];
        let dy = target.fighter.position[1] - ctx.own.position[1];
        dx.abs() <= range && (height[0]..=height[1]).contains(&dy)
    })
}
fn near_front(ctx: ComboContext, range: f32, height: [f32; 2]) -> bool {
    near(ctx, range, height)
        && ctx
            .target
            .is_some_and(|t| (t.fighter.position[0] - ctx.own.position[0]) * ctx.own.facing >= -0.5)
}
fn fair_reach(settings: ComboSettings, ctx: ComboContext, delay: u16) -> bool {
    let Some(target) = ctx.target else {
        return false;
    };
    ActiveWindow {
        first: 10 + delay,
        last: 11 + delay,
    }
    .first_contact(
        0.0,
        RecoveryWindow {
            vulnerable_in: if target.fighter.vulnerable {
                Some(0.0)
            } else {
                target.fighter.vulnerable_in
            },
            // No guessed hitstun deadline: actionability is observed continuously.
            actionable_in: None,
        },
        |lead| {
            let dx = target.fighter.position[0] - ctx.own.position[0]
                + (target.fighter.velocity[0] - ctx.own.velocity[0]) * lead;
            let dy = target.fighter.position[1] - ctx.own.position[1]
                + (target.fighter.velocity[1] - ctx.own.velocity[1]) * lead;
            dx.abs() <= settings.fair_range
                && dx * ctx.own.facing >= -0.5
                && (-6.0..=16.0).contains(&dy)
        },
    )
    .is_some()
}
fn valid(settings: ComboSettings, ctx: ComboContext) -> bool {
    settings.max_start_percent.is_finite()
        && (0.0..=50.0).contains(&settings.max_start_percent)
        && [settings.grab_range, settings.fair_range]
            .iter()
            .all(|r| r.is_finite() && (1.0..=20.0).contains(r))
        && ctx.motion_rate.is_finite()
        && ctx.motion_rate > 0.0
        && ctx.own.motion_kind != 0
        && ctx.own.motion_frame.is_finite()
        && ctx.own.motion_frame >= 0.0
        && ctx.own.facing.is_finite()
        && ctx.own.facing.abs() >= 0.5
        && ctx
            .own
            .position
            .iter()
            .chain(ctx.own.velocity.iter())
            .all(|v| v.is_finite())
        && ctx.target.is_none_or(|t| {
            t.percent.is_finite()
                && t.percent >= 0.0
                && t.fighter
                    .position
                    .iter()
                    .chain(t.fighter.velocity.iter())
                    .all(|v| v.is_finite())
        })
}
fn native_elapsed(previous: ComboContext, current: ComboContext) -> f32 {
    let own = if previous.own.motion_kind == current.own.motion_kind
        && current.own.motion_frame >= previous.own.motion_frame
    {
        (current.own.motion_frame - previous.own.motion_frame) / current.motion_rate
    } else {
        1.0
    };
    let enemy = previous
        .target
        .zip(current.target)
        .and_then(|(p, c)| p.fighter.motion.zip(c.fighter.motion))
        .map_or(0.0, |(p, c)| {
            if p.kind == c.kind && c.frame >= p.frame && c.rate.is_finite() && c.rate > 0.0 {
                (c.frame - p.frame) / c.rate
            } else {
                1.0
            }
        });
    own.max(enemy)
}

#[cfg(test)]
mod tests;
