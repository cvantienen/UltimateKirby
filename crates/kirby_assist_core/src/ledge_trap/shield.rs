//! Get-up-attack response: shield first, then use native readiness to punish.
//!
//! Front-side blocks can request a grab; a rear-side block can start a hop.
//! Successful capture is observed by the parent state machine, which hands
//! control back to the player. This file does not choose pummels or throws.
use super::{
    AttackWindow, LedgeContext, LedgeOutput, LedgeSettings, LedgeTrap, Opponent, OptionPlan,
    Pending, CLIFF_ATTACK, GUARD, GUARD_DAMAGE, GUARD_OFF, GUARD_ON, JUMP_SQUAT, MAX_ATTEMPTS,
};
use crate::native_input::{MAPPER_A, MAPPER_X, MAPPER_ZR};

impl LedgeTrap {
    /// Hold shield for a confirmed block, or finish committed pushback pursuit.
    pub(super) fn respond_to_getup_attack(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        enemy: Opponent,
        out: &mut LedgeOutput,
    ) {
        if !settings.responses.attack_getup {
            return;
        }
        let dx = enemy.position[0] - ctx.position[0];
        let dy = enemy.position[1] - ctx.position[1];
        let blocked = matches!(self.plan, OptionPlan::Shield { blocked: true });
        let may_punish = blocked || !settings.distance_agnostic && self.getup_hitboxes_ended;
        let attack_ended = enemy.status != CLIFF_ATTACK || self.getup_hitboxes_ended;
        let behind = dx * ctx.facing < -0.5;
        if settings.distance_agnostic && blocked && (behind || self.shield_nair) {
            if settings.responses.rear_block_nair {
                self.shield_nair = true;
                self.respond_to_rear_block_nair(settings, ctx, enemy, out);
            } else {
                out.requested = MAPPER_ZR;
            }
            return;
        }
        if ctx.situation != 0 {
            return;
        }
        let in_reach = settings.distance_agnostic
            || dx.abs() <= settings.grab_range && dy.abs() <= 5.0 && dx * ctx.facing >= -0.5;
        // The opponent may be protected right now but hittable when Grab
        // becomes active. Conversely, current proximity alone is not enough
        // if their observed motion carries them out of the grab window.
        let startup = settings.fighter.grab.startup(ctx.status)
            + if ctx.status == GUARD_DAMAGE {
                ctx.shield_stun_frames.unwrap_or(0).clamp(0, 6) as u8
            } else {
                0
            };
        let window = AttackWindow {
            first: startup,
            last: startup + settings.fighter.grab.active_frames - 1,
            range: settings.grab_range,
            height: [-5.0, 5.0],
            front_only: true,
        };
        let punish_deadline = enemy
            .actionable_in
            .or_else(|| enemy.motion.and_then(|m| m.remaining));
        // Latch the departure from shield. A transient in-range sample
        // during GUARD_OFF must not re-raise ZR and restart guarding.
        if may_punish
            && !settings.distance_agnostic
            && (ctx.status == GUARD_OFF
                || attack_ended && enemy.situation == 0 && dx * ctx.facing < -0.5
                || settings.auto_position && attack_ended && enemy.situation == 0 && !in_reach)
        {
            self.shield_pursuing = true;
        }
        if self.shield_pursuing {
            if ctx.can_act && enemy.situation == 0 && attack_ended {
                if in_reach
                    && (settings.distance_agnostic && blocked
                        || window
                            .scheduled_contact(
                                ctx,
                                enemy,
                                punish_deadline,
                                settings.distance_agnostic,
                            )
                            .is_some())
                {
                    if self.try_grab(settings, false) {
                        out.requested = settings.grab_button;
                    }
                } else if dx.abs() <= settings.grab_range && dx * ctx.facing < -0.5 {
                    let direction = dx.signum();
                    if ctx.safe_ground[usize::from(direction > 0.0)]
                        && self.movement_attempts < MAX_ATTEMPTS
                    {
                        self.movement_attempts += 1;
                        self.movement_preparing = true;
                        out.stick = Some([direction * 0.3, 0.0]);
                    }
                } else if !in_reach {
                    if settings.auto_position && dx * ctx.facing < -0.5 {
                        self.ground_move(settings, ctx, dx, out);
                    } else {
                        self.approach(settings, ctx, dx, settings.grab_range - 1.0, out);
                    }
                }
            }
            return;
        }
        if !settings.distance_agnostic
            && settings.manual_spacing
            && (dx.abs() > settings.grab_range || dy.abs() > 5.0)
        {
            return;
        }
        // Keep ZR down through shieldstun, its buffer, and all retry
        // release gaps. Only R pulses; an in-range punish never needs
        // a shield-drop/shield-raise cycle.
        out.requested = MAPPER_ZR;
        // For an in-range shield grab, keep ZR held and pulse the configured
        // Grab button; dropping shield first adds native guard-off delay. Only
        // pursuit after pushback deliberately commits to dropping shield.
        let buffer_ready = ctx.status == GUARD_DAMAGE
            && ctx.shield_stun_frames.is_some_and(|frames| {
                frames >= 0 && frames <= i32::from(settings.grab_buffer_frames)
            });
        let legal_guard =
            matches!(ctx.status, GUARD_ON | GUARD) && (ctx.can_grab || ctx.status == GUARD);
        // After a confirmed block, native shieldstun/readiness controls when
        // Grab can start. Waiting for the attacker's hitboxes to disappear
        // can miss the buffer window even though the hit has already blocked.
        // Without block evidence, still require observed hitboxes to clear.
        if may_punish
            && (blocked || enemy.attack_active != Some(true))
            && in_reach
            && (settings.distance_agnostic && blocked
                || window
                    .scheduled_contact(ctx, enemy, punish_deadline, settings.distance_agnostic)
                    .is_some())
            && (enemy.situation == 0 || settings.distance_agnostic && blocked)
            && (buffer_ready || legal_guard)
            && self.try_grab(settings, buffer_ready)
        {
            out.requested |= settings.grab_button;
        }
    }

    /// Request neutral jump+A directly out of shield, without turning first.
    pub(super) fn respond_to_rear_block_nair(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        enemy: Opponent,
        out: &mut LedgeOutput,
    ) {
        out.stick = Some([0.0, 0.0]);
        if ctx.status == JUMP_SQUAT {
            return;
        } // Release X for short hop.
        if ctx.situation != 0 {
            return;
        }
        if self.jump_air_seen {
            // One completed hop per block, even if the aerial missed.
            self.jump_committed = false;
            return;
        }
        if ctx.status == GUARD_DAMAGE {
            out.requested = MAPPER_ZR;
            return;
        }
        let Some(full_hop) = super::prediction::choose_hop(
            settings.aerial_execution.shield,
            settings.fighter.neutral_air,
            settings,
            ctx,
            enemy,
        ) else {
            return;
        };
        let may_jump = ctx.can_jump && (matches!(ctx.status, GUARD_ON | GUARD) || ctx.can_act);
        if may_jump && self.jump_attempts < MAX_ATTEMPTS {
            if self.last_output.requested & (MAPPER_X | MAPPER_A) == 0 {
                self.jump_attempts += 1;
                self.jump_committed = true;
                self.jump_full = full_hop;
                self.jump_floor_y = Some(ctx.position[1]);
                // Queue the aerial with the jump, just like the other nair
                // responses. Pending also makes shared continuation retain A
                // through squat/hitstop and pair any rejected ground retry.
                self.pending = Pending::NeutralAir;
                self.attempts = self.attempts.saturating_add(1);
                out.requested = MAPPER_X | if full_hop { 0 } else { MAPPER_A };
            }
        } else if !self.jump_committed && matches!(ctx.status, GUARD_ON | GUARD) {
            // Hold shield until native jump-cancel readiness, rather than
            // releasing it early and entering the slower guard-off animation.
            out.requested = MAPPER_ZR;
        }
    }
}
