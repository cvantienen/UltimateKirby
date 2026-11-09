//! File configuration over the existing grab/backup admission and delivery paths.
use super::*;
use crate::{
    gameplay::{GameplayContext, Opponent},
    native_input::{MAPPER_B, MAPPER_R},
    sequence_config::{Action, Direction, Sequence},
    settings::AssistSettings,
};

#[derive(Clone, Copy, Default)]
pub struct AutomaticConfig<'a> {
    pub grab: Option<&'a Sequence>,
    pub shield_grab: Option<&'a Sequence>,
    pub backup: Option<&'a Sequence>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AutomaticLimits {
    pub close: f32,
    pub approach: f32,
    pub grab_height: f32,
    pub shield_height: f32,
    pub backup: f32,
    pub backup_height: f32,
}
impl AutomaticConfig<'_> {
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
impl OutOfShield {
    fn sample_configured_grab(
        &mut self,
        allowed: bool,
        c: Context,
        context: GameplayContext,
        targets: [Option<crate::roll_read::Target>; 8],
        follow: Option<Opponent>,
        shielded: bool,
        limits: AutomaticLimits,
        startup: Option<f32>,
    ) -> ControllerOutput {
        self.simple.sample(
            allowed,
            c,
            context,
            targets,
            follow,
            if shielded {
                limits.close
            } else {
                limits.approach
            },
            if shielded {
                limits.shield_height
            } else {
                limits.grab_height
            },
            startup,
            limits.close,
            !shielded,
            shielded,
        )
    }

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
        let limits = config.limits(close, approach);
        let shielded = shielded
            || matches!(
                c.status,
                crate::native_states::GUARD_ON..=crate::native_states::GUARD_DAMAGE
            );
        let backup_timing = config.backup.and_then(backup_timing);
        let native_timing = backup_timing.unwrap_or_else(super::shield_up_b::Timing::legacy);
        if self.configured_response
            && !self.configured_backup
            && self.configured_grab_wait
            && !self.grab_delivery_pending()
        {
            let enemy = follow
                .filter(|e| Some(e.id) == self.response.target())
                .or_else(|| {
                    targets
                        .iter()
                        .flatten()
                        .find(|t| Some(t.id) == self.response.target())
                        .copied()
                        .map(super::copied_target)
                });
            let valid = allowed
                && self.configured_shielded == shielded
                && c.grounded
                && context.situation == crate::native_states::situation::GROUND
                && super::ground_grab_state(c.status)
                && enemy.is_some_and(|e| {
                    simple::target_valid(
                        c,
                        context,
                        e,
                        if shielded {
                            limits.close
                        } else {
                            limits.approach
                        },
                        if shielded {
                            limits.shield_height
                        } else {
                            limits.grab_height
                        },
                        startup,
                        !shielded,
                        shielded,
                    )
                });
            if !valid {
                // No R was delivered. Re-evaluate the current shield/target now,
                // rather than keep running the old unshielded custom response.
                self.cancel_pending();
            }
        }
        if self.configured_settling {
            let _ = self.response.sample(false, settings, context, follow);
            self.configured_response = self.response.grab_delivery_pending();
            let _ = self.sample_configured_grab(
                allowed, c, context, targets, follow, shielded, limits, startup,
            );
            if !self.simple.delivery_pending() {
                self.configured_settling = false;
                self.simple.cancel();
            }
            return ControllerOutput::default();
        }
        if self.configured_response {
            let delivered = self.simple.delivery_pending();
            if delivered {
                let receipt = self.sample_configured_grab(
                    allowed, c, context, targets, follow, shielded, limits, startup,
                );
                if receipt.owned == 0 {
                    self.response.cancel();
                    self.configured_response = false;
                    self.configured_settling = self.simple.delivery_pending();
                    return ControllerOutput::default();
                }
            }
            let output = self.response.sample(allowed, settings, context, follow);
            if delivered && output.requested & MAPPER_R == 0 {
                // A queued Grab must never become a throw-producing B/Up input.
                self.response.cancel();
                self.configured_response = false;
                self.configured_settling = self.simple.delivery_pending();
                return ControllerOutput::default();
            }
            if !self.simple.delivery_pending() && output.requested & MAPPER_R != 0 {
                self.simple.rebase_custom_delivery();
            }
            if self.response.target().is_none() {
                self.configured_response = false;
                self.configured_settling = self.simple.delivery_pending();
                if !self.simple.delivery_pending() {
                    self.simple.cancel();
                }
            }
            return output;
        }
        let fallback_ended = if self.shield_up_b.active() {
            self.simple.cancel();
            let output = self.shield_up_b.sample(
                allowed,
                c,
                context,
                targets,
                follow,
                limits.backup,
                limits.backup_height,
                native_timing,
            );
            if output.owned != 0 || self.shield_up_b.active() {
                return output;
            }
            true
        } else {
            false
        };
        let delivered = self.simple.delivery_pending();
        let grab = self.sample_configured_grab(
            allowed, c, context, targets, follow, shielded, limits, startup,
        );
        if grab.owned != 0 || fallback_ended || delivered || self.simple.delivery_pending() {
            let sequence = if shielded {
                config.shield_grab
            } else {
                config.grab
            };
            if grab.requested & MAPPER_R != 0 {
                if let Some(s) = sequence.filter(|s| !native_grab_recipe(s)) {
                    return self.commit_configured(
                        *s, allowed, context, targets, follow, settings, false, shielded,
                    );
                }
            }
            return grab;
        }
        let output = self.shield_up_b.sample(
            allowed && shielded && config.backup.is_some(),
            // Reuse only the fallback's target/range admission for a custom
            // ordinary recipe. Its placeholder B is replaced below and never
            // delivered; Up-B availability must gate only an actual Up-B start.
            if config
                .backup
                .is_some_and(|s| !crate::reaction_rules::starts_direct_up_b(s))
            {
                Context {
                    can_up_special: c.shield_ready
                        && (c.can_grab || c.can_jump || c.can_up_special),
                    ..c
                }
            } else {
                c
            },
            context,
            targets,
            follow,
            limits.backup,
            limits.backup_height,
            native_timing,
        );
        if output.requested & MAPPER_B != 0 {
            if let Some(s) = config.backup.filter(|_| backup_timing.is_none()) {
                return self.commit_configured(
                    *s, allowed, context, targets, follow, settings, true, shielded,
                );
            }
        }
        output
    }
    fn commit_configured(
        &mut self,
        sequence: Sequence,
        allowed: bool,
        mut context: GameplayContext,
        targets: [Option<crate::roll_read::Target>; 8],
        follow: Option<Opponent>,
        settings: AssistSettings,
        backup: bool,
        shielded: bool,
    ) -> ControllerOutput {
        let id = if backup {
            self.shield_up_b.target()
        } else {
            self.simple.target()
        };
        let enemy = follow.filter(|e| Some(e.id) == id).or_else(|| {
            targets
                .into_iter()
                .flatten()
                .find(|t| Some(t.id) == id)
                .map(super::copied_target)
        });
        let Some(enemy) = enemy else {
            self.cancel_pending();
            return ControllerOutput::default();
        };
        // Grab readiness already authorized an ordinary configured response;
        // direct Up-B additionally keeps its specific native readiness gate.
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
        self.configured_shielded = shielded;
        self.configured_grab_wait = sequence.actions().iter().any(|a| match a {
            Action::Input { buttons, .. }
            | Action::OpponentInput { buttons, .. }
            | Action::Controller { buttons, .. }
            | Action::Tap { buttons, .. } => buttons & MAPPER_R != 0,
            _ => false,
        });
        let output = self
            .response
            .sample(allowed, settings, context, Some(enemy));
        if output.requested & MAPPER_R != 0 {
            self.simple.rebase_custom_delivery();
        }
        output
    }
}
