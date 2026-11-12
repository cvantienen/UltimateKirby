//! File configuration over the existing grab/backup admission and delivery paths.
use super::*;
use crate::{
    gameplay::{GameplayContext, Opponent},
    native_input::{MAPPER_B, MAPPER_R},
    sequence_config::{Action, Direction, Sequence},
    settings::AssistSettings,
};

/// Runtime Smash data carried by `AutomaticConfig`.
#[derive(Clone, Copy, Default)]
pub struct AutomaticConfig<'a> {
/// Runtime grab value used by AutomaticConfig..
    pub grab: Option<&'a Sequence>,
/// Runtime shield grab value used by AutomaticConfig..
    pub shield_grab: Option<&'a Sequence>,
/// Runtime backup value used by AutomaticConfig..
    pub backup: Option<&'a Sequence>,
}
/// Runtime Smash data carried by `AutomaticLimits`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AutomaticLimits {
/// Runtime close value used by AutomaticLimits..
    pub close: f32,
/// Runtime approach value used by AutomaticLimits..
    pub approach: f32,
/// Runtime grab height value used by AutomaticLimits..
    pub grab_height: f32,
/// Runtime shield height value used by AutomaticLimits..
    pub shield_height: f32,
/// Runtime backup value used by AutomaticLimits..
    pub backup: f32,
/// Runtime backup height value used by AutomaticLimits..
    pub backup_height: f32,
}
impl AutomaticLimits {
    /// Returns the `reach` result.
    pub(super) fn reach(self, shielded: bool, startup: Option<f32>) -> simple::Reach {
        simple::Reach {
            range: if shielded { self.close } else { self.approach },
            height: if shielded {
                self.shield_height
            } else {
                self.grab_height
            },
            startup,
            close: self.close,
            approach: !shielded,
            shielded,
        }
    }
}
impl AutomaticConfig<'_> {
    /// Returns the `limits` result.
    pub fn limits(self, close: f32, approach: f32) -> AutomaticLimits {
        let close = self.shield_grab.map_or(close, |s| s.reaction.max);
        AutomaticLimits {
            close,
            approach: self.grab.map_or(approach, |s| s.reaction.max),
            grab_height: self.grab.map_or(24., |s| s.reaction.height),
            shield_height: self.shield_grab.map_or(24., |s| s.reaction.height),
            backup: self.backup.map_or(close, |s| s.reaction.max),
            backup_height: self.backup.map_or(24., |s| s.reaction.height),
        }
    }
}
/// Determines whether the `literal_input` condition holds.
/// Checks whether literal input.
fn literal_input(action: &Action, buttons: u32, stick: [f32; 2], frames: u8) -> bool {
    match action {
        Action::Input {
            buttons: b,
            stick: s,
            frames: f,
            ..
        }
        | Action::OpponentInput {
            buttons: b,
            stick: s,
            frames: f,
        } => *b == buttons && *s == stick && *f == frames,
        Action::Controller {
            buttons: b,
            stick: s,
            cstick,
            frames: f,
            ..
        } => *b == buttons && *s == stick && *f == frames && *cstick == [0.; 2],
        _ => false,
    }
}
/// Determines whether the `native_grab_recipe` condition holds.
/// Checks whether native grab recipe.
fn native_grab_recipe(s: &Sequence) -> bool {
    let grab = match s.actions() {
        [grab] => Some(grab),
        // Native grab admission already owns the short idle dash, range and
        // readiness checks. A chase-first grab must not replace an admitted R
        // with a long run (older edited tapes contain Up-B's drift here).
        [Action::Drift {
            direction: Direction::Opponent,
            ..
        }, grab] => Some(grab),
        _ => None,
    };
    s.start_delay == 0
        && s.timeout >= 24
        && grab.is_some_and(|a| literal_input(a, MAPPER_R, [0.; 2], 4))
}
/// Returns an optional `backup_timing` value.
/// Gets backup timing if it is available.
fn backup_timing(s: &Sequence) -> Option<super::shield_up_b::Timing> {
    if s.start_delay != 0 || s.timeout < 24 {
        return None;
    }
    let frames = match s.actions().first()? {
        Action::Input { frames, .. }
        | Action::OpponentInput { frames, .. }
        | Action::Controller { frames, .. } => *frames,
        _ => return None,
    };
    if frames == 0 || !literal_input(&s.actions()[0], MAPPER_B, [0., 1.], frames) {
        return None;
    }
    if s.actions().len() != 1 {
        return None;
    }
    Some(super::shield_up_b::Timing {
        delivery: frames,
        timeout: s.timeout,
    })
}

/// One automatic sample's facts, shared by the steps below.
#[derive(Clone, Copy)]
struct Sample<'a> {
/// Runtime allowed value used by Sample..
    allowed: bool,
/// Runtime c value used by Sample..
    c: Context,
/// Runtime context value used by Sample..
    context: GameplayContext,
/// Runtime targets value used by Sample..
    targets: [Option<crate::roll_read::Target>; 8],
/// Runtime follow value used by Sample..
    follow: Option<Opponent>,
/// Runtime shielded value used by Sample..
    shielded: bool,
/// Runtime limits value used by Sample..
    limits: AutomaticLimits,
/// Runtime startup value used by Sample..
    startup: Option<f32>,
/// Runtime settings value used by Sample..
    settings: AssistSettings,
/// Runtime config value used by Sample..
    config: AutomaticConfig<'a>,
}
impl Sample<'_> {
    /// Returns an optional `backup_timing` value.
    fn backup_timing(&self) -> Option<super::shield_up_b::Timing> {
        self.config.backup.and_then(backup_timing)
    }
}

impl OutOfShield {
    /// Samples the automatic configured.
    pub fn sample_automatic_configured(
        &mut self,
        allowed: bool,
        c: Context,
        context: GameplayContext,
        targets: [Option<crate::roll_read::Target>; 8],
        follow: Option<Opponent>,
        shielded: bool,
        close: f32,
        approach: f32,
        startup: Option<f32>,
        settings: AssistSettings,
        config: AutomaticConfig<'_>,
    ) -> ControllerOutput {
        self.bind_actor(c);
        if config.backup.is_none()
            && (self.shield_up_b.active() || self.configured_response && self.configured_backup)
        {
            self.shield_up_b.cancel();
            self.response.suspend();
            self.configured_response = self.response.grab_delivery_pending();
            self.configured_backup = false;
            return ControllerOutput::default();
        }
        let f = Sample {
            allowed,
            c,
            context,
            targets,
            follow,
            shielded: shielded
                || matches!(
                    c.status,
                    crate::native_states::GUARD_ON..=crate::native_states::GUARD_DAMAGE
                ),
            limits: config.limits(close, approach),
            startup,
            settings,
            config,
        };
        self.revalidate_grab_wait(&f);
        if self.configured_settling {
            return self.settle_grab(&f);
        }
        if self.configured_response {
            return self.continue_response(&f);
        }
        let mut fallback_ended = false;
        if self.shield_up_b.active() {
            self.simple.cancel();
            let output = self.run_backup(&f, f.allowed, f.c);
            if output.owned != 0 || self.shield_up_b.active() {
                return output;
            }
            fallback_ended = true;
        }
        if let Some(output) = self.sample_grab(&f, fallback_ended) {
            return output;
        }
        self.sample_backup(&f)
    }

    /// Samples the configured grab.
    fn sample_configured_grab(&mut self, f: &Sample) -> ControllerOutput {
        self.simple.sample(
            f.allowed,
            f.c,
            f.context,
            f.targets,
            f.follow,
            f.limits.reach(f.shielded, f.startup),
        )
    }

    /// Runs the backup.
    fn run_backup(&mut self, f: &Sample, allowed: bool, c: Context) -> ControllerOutput {
        let timing = f
            .backup_timing()
            .unwrap_or_else(super::shield_up_b::Timing::legacy);
        self.shield_up_b.sample(
            allowed,
            c,
            f.context,
            f.targets,
            f.follow,
            f.limits.backup,
            f.limits.backup_height,
            timing,
        )
    }

    /// A configured response waiting for its Grab to be mapped must still match
    /// the current shield/target; otherwise no R was delivered and the stale
    /// unshielded custom response yields to a fresh evaluation.
    fn revalidate_grab_wait(&mut self, f: &Sample) {
        if !(self.configured_response
            && !self.configured_backup
            && self.configured_grab_wait
            && !self.grab_delivery_pending())
        {
            return;
        }
        let valid = f.allowed
            && self.configured_shielded == f.shielded
            && f.c.grounded
            && f.context.situation == crate::native_states::situation::GROUND
            && super::ground_grab_state(f.c.status)
            && super::locked_enemy(self.response.target(), f.follow, &f.targets).is_some_and(|e| {
                simple::target_valid(f.c, f.context, e, f.limits.reach(f.shielded, f.startup))
            });
        if !valid {
            self.cancel_pending();
        }
    }

    /// The configured tape is over; a Grab already delivered keeps settling.
    fn end_response(&mut self) {
        self.configured_response = false;
        self.configured_settling = self.simple.delivery_pending();
    }

    /// Returns the `settle_grab` result.
    fn settle_grab(&mut self, f: &Sample) -> ControllerOutput {
        let _ = self
            .response
            .sample(false, f.settings, f.context, f.follow);
        self.configured_response = self.response.grab_delivery_pending();
        let _ = self.sample_configured_grab(f);
        if !self.simple.delivery_pending() {
            self.configured_settling = false;
            self.simple.cancel();
        }
        ControllerOutput::default()
    }

    /// Returns the `continue_response` result.
    fn continue_response(&mut self, f: &Sample) -> ControllerOutput {
        let delivered = self.simple.delivery_pending();
        if delivered {
            let receipt = self.sample_configured_grab(f);
            if receipt.owned == 0 {
                self.response.cancel();
                self.end_response();
                return ControllerOutput::default();
            }
        }
        let output = self
            .response
            .sample(f.allowed, f.settings, f.context, f.follow);
        if delivered && output.requested & MAPPER_R == 0 {
            // A queued Grab must never become a throw-producing B/Up input.
            self.response.cancel();
            self.end_response();
            return ControllerOutput::default();
        }
        if !self.simple.delivery_pending() && output.requested & MAPPER_R != 0 {
            self.simple.rebase_custom_delivery();
        }
        if self.response.target().is_none() {
            self.end_response();
            if !self.simple.delivery_pending() {
                self.simple.cancel();
            }
        }
        output
    }

    /// The ordinary Grab path. `None` when it neither owns inputs nor awaits a
    /// delivered request, so the backup may be considered.
    fn sample_grab(&mut self, f: &Sample, fallback_ended: bool) -> Option<ControllerOutput> {
        let delivered = self.simple.delivery_pending();
        let grab = self.sample_configured_grab(f);
        if grab.owned == 0 && !fallback_ended && !delivered && !self.simple.delivery_pending() {
            return None;
        }
        if grab.requested & MAPPER_R != 0 {
            let sequence = if f.shielded {
                f.config.shield_grab
            } else {
                f.config.grab
            };
            if let Some(s) = sequence.filter(|s| !native_grab_recipe(s)) {
                return Some(self.commit_configured(*s, f, false));
            }
        }
        Some(grab)
    }

    /// Samples the backup.
    fn sample_backup(&mut self, f: &Sample) -> ControllerOutput {
        // Reuse only the fallback's target/range admission for a custom
        // ordinary recipe. Its placeholder B is replaced below and never
        // delivered; Up-B availability must gate only an actual Up-B start.
        let c = if f
            .config
            .backup
            .is_some_and(|s| !crate::reaction_rules::starts_direct_up_b(s))
        {
            Context {
                can_up_special: f.c.shield_ready
                    && (f.c.can_grab || f.c.can_jump || f.c.can_up_special),
                ..f.c
            }
        } else {
            f.c
        };
        let output = self.run_backup(f, f.allowed && f.shielded && f.config.backup.is_some(), c);
        if output.requested & MAPPER_B != 0 {
            if let Some(s) = f.config.backup.filter(|_| f.backup_timing().is_none()) {
                return self.commit_configured(*s, f, true);
            }
        }
        output
    }

    /// Returns the `commit_configured` result.
    fn commit_configured(
        &mut self,
        sequence: Sequence,
        f: &Sample,
        backup: bool,
    ) -> ControllerOutput {
        let id = if backup {
            self.shield_up_b.target()
        } else {
            self.simple.target()
        };
        let Some(enemy) = super::locked_enemy(id, f.follow, &f.targets) else {
            self.cancel_pending();
            return ControllerOutput::default();
        };
        // Grab readiness already authorized an ordinary configured response;
        // direct Up-B additionally keeps its specific native readiness gate.
        let mut context = f.context;
        if !crate::reaction_rules::starts_direct_up_b(&sequence) {
            context.can_act = true;
        }
        if !self.response.start_sequence(sequence, context, enemy) {
            self.cancel_pending();
            return ControllerOutput::default();
        }
        if backup {
            self.shield_up_b.cancel();
        }
        self.configured_response = true;
        self.configured_backup = backup;
        self.configured_shielded = f.shielded;
        self.configured_grab_wait = sequence.actions().iter().any(|a| match a {
            Action::Input { buttons, .. }
            | Action::OpponentInput { buttons, .. }
            | Action::Controller { buttons, .. }
            | Action::Tap { buttons, .. } => buttons & MAPPER_R != 0,
            _ => false,
        });
        let output = self
            .response
            .sample(f.allowed, f.settings, context, Some(enemy));
        if output.requested & MAPPER_R != 0 {
            self.simple.rebase_custom_delivery();
        }
        output
    }
}
