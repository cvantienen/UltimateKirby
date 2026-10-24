//! Ledge-jump and neutral-get-up aerial responses, plus shared hop completion.
//!
//! Short hops pair neutral X+A; full hops hold X and request A after takeoff.
//! Once a hop is committed,
//! completion follows our own native state even if opponent tracking changes.
//! Input requests, accepted animation and confirmed hits remain separate events.
use super::{
    finite, ground_attack_ready, jump_contact, predicted_offset, AttackWindow, LedgeContext,
    LedgeOutput, LedgeSettings, LedgeTrap, Opponent, Pending, ATTACK_AIR, CLIFF_JUMP1, CLIFF_JUMP3,
    GUARD, GUARD_DAMAGE, GUARD_ON, JUMP_SQUAT, JUMP_SQUAT_FRAMES, LEDGE_BUTTONS,
    NEUTRAL_AIR_MOTION,
};
use crate::native_input::{MAPPER_A, MAPPER_X, MAPPER_ZR};

impl LedgeTrap {
    /// Request the native short-hop aerial macro with neutral jump+A together.
    /// A neutral stick keeps the recipe independent of Kirby's facing.
    pub(super) fn respond_to_jump(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        enemy: Opponent,
        out: &mut LedgeOutput,
    ) {
        if !settings.responses.jump_getup {
            return;
        }
        self.movement_preparing = false;
        out.stick = Some([0.0, 0.0]);
        // Do not turn an unsuccessful ledge-jump intercept into a landing chase.
        if enemy.situation == 0 && !(CLIFF_JUMP1..=CLIFF_JUMP3).contains(&enemy.status) {
            self.jump_committed = false;
            self.jump_air_seen = false;
            return;
        }
        if self.jump_air_seen {
            return; // One actual hop per visit, including a whiff.
        }
        if ctx.situation != 0
            || !ctx.can_act
            || ctx.facing.abs() < 0.5
            || !settings.distance_agnostic
                && settings.manual_spacing
                && enemy.attack_active == Some(true)
        {
            return;
        }
        let Some(full_hop) = super::prediction::choose_hop(
            settings.aerial_execution.jump,
            settings.fighter.neutral_air,
            settings,
            ctx,
            enemy,
        ) else {
            return;
        };
        let extra = if full_hop {
            1 + ctx.input_delay.ceil().clamp(0.0, 12.0) as u8
        } else {
            0
        };
        let profiled = settings.aerial_execution.jump
            == crate::fighter_profiles::execution::Hop::Full
            || settings.aerial_execution.jump == crate::fighter_profiles::execution::Hop::Adaptive
                && ctx.air_physics.is_some()
                && ctx.short_hop_speed.is_some()
                && ctx.full_hop_speed.is_some();
        let window = AttackWindow {
            first: JUMP_SQUAT_FRAMES + settings.fighter.neutral_air.first + extra,
            last: JUMP_SQUAT_FRAMES + settings.fighter.neutral_air.last + extra,
            range: settings.anti_air_range,
            height: [-6.0, settings.anti_air_height],
            front_only: false,
        };
        // Kirby's jump-nair can be queued two frames earlier without changing
        // its actual startup. Already-immediate launch recognition stays immediate.
        let timing_enemy = if settings.implementation == Some(super::ResponseImplementation::Kirby)
        {
            Opponent {
                vulnerable_in: enemy.vulnerable_in.map(|left| (left - 2.0).max(0.0)),
                ..enemy
            }
        } else {
            enemy
        };
        let predicted = predicted_offset(ctx, enemy, f32::from(window.first));
        // Stage the short-hop attack during the launch animation instead of
        // extrapolating its transient root speed for 35 frames.
        let prepare_launch = !profiled
            && (settings.distance_agnostic || ctx.air_physics.is_some())
            && matches!(enemy.status, CLIFF_JUMP1 | 0x7c)
            && enemy
                .actionable_in
                .is_none_or(|left| left > f32::from(window.first))
            && (settings.distance_agnostic
                || (enemy.position[0] - ctx.position[0]).abs() <= settings.anti_air_range
                    && (enemy.position[1] - ctx.position[1]).abs() <= settings.anti_air_height);
        let jump_timed = if profiled {
            true // Explicit full-hop policy or an adaptive plan resolved above.
        } else if settings.distance_agnostic {
            window
                .timing_contact(ctx, timing_enemy, enemy.actionable_in)
                .is_some()
        } else {
            jump_contact(window, ctx, timing_enemy, true, Some(ctx.position[1]))
        };
        if !prepare_launch && !jump_timed {
            if predicted[0].abs() > settings.anti_air_range {
                self.approach(
                    settings,
                    ctx,
                    predicted[0],
                    settings.anti_air_range - 1.0,
                    out,
                );
            }
            return;
        }
        if !ground_attack_ready(ctx, settings)
            || !settings.manual_spacing && finite(ctx.velocity[0]).abs() > 0.12
        {
            return;
        }
        if !ctx.can_jump {
            return;
        }
        if self.last_output.requested & (MAPPER_X | MAPPER_A) != 0
            || !settings.manual_spacing
                && self
                    .last_output
                    .stick
                    .is_some_and(|stick| stick[0].abs() > 0.1)
        {
            return;
        }
        self.jump_attempts = self.jump_attempts.saturating_add(1);
        self.jump_committed = true;
        self.jump_full = full_hop;
        self.jump_floor_y = Some(ctx.position[1]);
        self.pending = Pending::NeutralAir;
        self.attempts = self.attempts.saturating_add(1);
        // Built-in short-hop attack: one neutral X+A sample goes through the
        // mapper/online queue together, avoiding a second observation round trip.
        out.requested = MAPPER_X | if full_hop { 0 } else { MAPPER_A };
    }

    /// Finish any committed nair hop using only our native action state.
    pub(super) fn continue_committed_nair(&mut self, ctx: LedgeContext, out: &mut LedgeOutput) {
        let supported = if ctx.situation == 0 {
            matches!(ctx.status, 0..=JUMP_SQUAT | 0x11..=0x18 | GUARD_ON | GUARD | GUARD_DAMAGE)
        } else {
            ctx.situation == 2 && self.owns_jump_state(ctx.status)
        };
        if !supported || !ctx.can_control && !ctx.stopped {
            self.jump_committed = false;
            self.jump_air_seen = true;
            if self.pending == Pending::NeutralAir {
                self.pending = Pending::None;
            }
            return;
        }
        out.owned = LEDGE_BUTTONS;
        out.stick = Some([0.0, 0.0]);
        if self.jump_air_seen && ctx.situation == 0 {
            self.jump_committed = false;
            if self.pending == Pending::NeutralAir {
                self.pending = Pending::None;
            }
            return;
        }
        // Keep competing physical inputs out during stop too. Preserve a queued
        // A hold, but never start an attack or repeat X during frozen progress.
        if ctx.stopped {
            if matches!(ctx.status, GUARD_ON | GUARD | GUARD_DAMAGE) {
                out.requested = MAPPER_ZR;
            } else if self.jump_full
                && self.last_output.requested & MAPPER_X != 0
                && ctx.situation == 0
            {
                out.requested = MAPPER_X;
            } else if self.pending == Pending::NeutralAir
                && (ctx.status == JUMP_SQUAT
                    || ctx.situation == 2 && matches!(ctx.status, 0xb | 0xe | 0xf))
            {
                out.requested = if self.jump_full && ctx.status == JUMP_SQUAT {
                    MAPPER_X
                } else {
                    MAPPER_A
                };
            }
            return;
        }
        if ctx.status == JUMP_SQUAT {
            if self.pending == Pending::NeutralAir {
                out.requested = if self.jump_full { MAPPER_X } else { MAPPER_A };
            }
            return; // Release X; keep a paired short-hop attack buffered.
        }
        if ctx.status == GUARD_DAMAGE {
            out.requested = MAPPER_ZR;
            return;
        }
        if ctx.situation == 2 {
            self.jump_air_seen = true;
            if ctx.status == ATTACK_AIR
                && ctx.motion_kind == NEUTRAL_AIR_MOTION
                && self.neutral_nair_drift != 0.0
                && ctx.motion_frame
                    < f32::from(
                        self.neutral_nair_drift_end_frame
                            .unwrap_or(super::NAIR_NUDGE_END_FRAME),
                    )
            {
                let vertical = out.stick.map_or(0.0, |stick| stick[1]);
                out.stick = Some([self.neutral_nair_drift, vertical]);
            }
            if self.tilt_animations == 0 && matches!(ctx.status, 0xb | 0xe | 0xf) {
                if self.pending != Pending::NeutralAir {
                    self.pending = Pending::NeutralAir;
                    self.attempts = self.attempts.saturating_add(1);
                }
                // Native airborne state is enough. Keep neutral A buffered
                // until attack acceptance; no transition-term or retry gate.
                out.requested = MAPPER_A;
            }
            return;
        }
        if self.jump_full && ctx.can_jump && (ctx.can_act || matches!(ctx.status, GUARD_ON | GUARD))
        {
            out.requested = MAPPER_X;
            return;
        }
        // X was a request, not proof of a jump. Retry rejected ground presses
        // with release gaps until native jump squat/takeoff is actually seen.
        if ctx.can_jump
            && (ctx.can_act || matches!(ctx.status, GUARD_ON | GUARD))
            && self.last_output.requested & (MAPPER_X | MAPPER_A) == 0
        {
            self.jump_attempts = self.jump_attempts.saturating_add(1);
            out.requested = MAPPER_X;
            if self.pending == Pending::NeutralAir {
                out.requested |= MAPPER_A;
            }
        }
    }

    /// Normal getup: stage the lingering nair before protection expires.
    pub(super) fn respond_to_normal_nair(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        enemy: Opponent,
        out: &mut LedgeOutput,
    ) {
        if !settings.responses.neutral_getup || !self.climb_seen || self.tilt_animations > 0 {
            return;
        }
        if self.jump_committed {
            self.continue_committed_nair(ctx, out);
            return;
        }
        if self.jump_air_seen || ctx.situation != 0 || !ctx.can_act || !ctx.can_jump {
            return;
        }
        let Some(vulnerable_in) = (if enemy.vulnerable {
            Some(0.0)
        } else {
            enemy.vulnerable_in
        })
        .filter(|left| left.is_finite() && (0.0..=120.0).contains(left)) else {
            return;
        };
        let Some(full_hop) = super::prediction::choose_hop(
            settings.aerial_execution.neutral,
            settings.fighter.neutral_air,
            settings,
            ctx,
            enemy,
        ) else {
            return;
        };
        // Short hops pay one delivery leg; full hops add the airborne A leg.
        let first_active = f32::from(JUMP_SQUAT_FRAMES + settings.fighter.neutral_air.first)
            + ctx.input_delay
            + if full_hop { 1.0 + ctx.input_delay } else { 0.0 };
        if vulnerable_in > first_active + f32::from(settings.fighter.normal_hit_lead_frames) {
            return;
        }
        if !settings.distance_agnostic {
            let extra = if full_hop {
                1 + ctx.input_delay.ceil().clamp(0.0, 12.0) as u8
            } else {
                0
            };
            let window = AttackWindow {
                first: JUMP_SQUAT_FRAMES + settings.fighter.neutral_air.first + extra,
                last: JUMP_SQUAT_FRAMES + settings.fighter.neutral_air.last + extra,
                range: settings.anti_air_range,
                height: [-6.0, settings.anti_air_height],
                front_only: false,
            };
            let forecast_ctx = if full_hop {
                LedgeContext {
                    short_hop_speed: ctx.full_hop_speed,
                    ..ctx
                }
            } else {
                ctx
            };
            if !jump_contact(window, forecast_ctx, enemy, true, Some(ctx.position[1])) {
                return;
            }
        }
        if self.last_output.requested & (MAPPER_X | MAPPER_A) != 0 {
            return;
        }
        self.jump_attempts = self.jump_attempts.saturating_add(1);
        self.jump_committed = true;
        self.jump_full = full_hop;
        self.jump_floor_y = Some(ctx.position[1]);
        self.pending = Pending::NeutralAir;
        self.attempts = self.attempts.saturating_add(1);
        out.stick = Some([0.0, 0.0]);
        out.requested = MAPPER_X | if full_hop { 0 } else { MAPPER_A };
    }
}
