//! Roll responses: lock an endpoint, approach, brake, then request the saved
//! punish (down smash, grab, or Samus's short-hop zair).
//!
//! Smashes retain their locked endpoint. Kirby/Yoshi walking grabs may extend
//! their approach inward from observed roll travel; they never chase back to ledge.
//! Arrival, attack-window overlap and native down-smash readiness are distinct
//! checks. A requested A pulse still needs observed animation acceptance.
use super::{
    predicted_offset, roll_stopping_distance, AttackWindow, LedgeContext, LedgeOutput,
    LedgeSettings, LedgeTrap, Opponent, Pending, RollIntercept, CLIFF_ESCAPE,
};
use crate::native_input::MAPPER_A;

#[derive(Clone, Copy, PartialEq, Eq)]
enum RollPunish {
    DownSmash,
    FacingDownSmash,
    DownTilt,
    Grab,
    SamusZair,
}

impl LedgeTrap {
    /// Front-only smashes need an actual turn before their timed request.
    pub(super) fn respond_to_roll_facing_smash(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        enemy: Opponent,
        out: &mut LedgeOutput,
    ) {
        self.respond_to_roll_with(settings, ctx, enemy, out, RollPunish::FacingDownSmash);
    }

    /// Body-based alternative for resource-dependent or slow down-smashes.
    pub(super) fn respond_to_roll_down_tilt(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        enemy: Opponent,
        out: &mut LedgeOutput,
    ) {
        self.respond_to_roll_with(settings, ctx, enemy, out, RollPunish::DownTilt);
    }
    /// Move immediately to one common coverage point, then time down smash.
    pub(super) fn respond_to_roll(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        enemy: Opponent,
        out: &mut LedgeOutput,
    ) {
        self.respond_to_roll_with(settings, ctx, enemy, out, RollPunish::DownSmash);
    }

    /// Follow the calculated grab approach and preserve its resulting facing.
    /// Capture hands back in observe_action.
    pub(super) fn respond_to_roll_grab(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        enemy: Opponent,
        out: &mut LedgeOutput,
    ) {
        #[cfg(feature = "diagnostic-logging")]
        {
            self.roll_grab_selected = true;
        }
        self.respond_to_roll_with(settings, ctx, enemy, out, RollPunish::Grab);
    }

    pub(super) fn respond_to_roll_zair(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        enemy: Opponent,
        out: &mut LedgeOutput,
    ) {
        self.respond_to_roll_with(settings, ctx, enemy, out, RollPunish::SamusZair);
    }

    fn respond_to_roll_with(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        enemy: Opponent,
        out: &mut LedgeOutput,
        punish: RollPunish,
    ) {
        let grab = punish == RollPunish::Grab;
        let fixed_grab = grab && self.fixed_roll_grab_inset.is_some();
        let inward_walk_grab = grab
            && matches!(
                settings.implementation,
                Some(super::ResponseImplementation::Kirby | super::ResponseImplementation::Yoshi)
            );
        let zair = punish == RollPunish::SamusZair;
        let tilt = punish == RollPunish::DownTilt;
        let face_smash = punish == RollPunish::FacingDownSmash;
        if !settings.responses.roll_getup {
            return;
        }
        out.stick = Some([0.0, 0.0]);
        self.roll_phase = "roll_wait_grounded";
        if ctx.situation != 0 {
            return;
        }
        if grab && self.grab_in_progress {
            self.roll_phase = "roll_native_grab_in_progress";
            return; // An accepted grab owns its animation, not another approach.
        }
        if enemy.status != CLIFF_ESCAPE {
            self.roll_positioned = true;
            self.roll_phase = "roll_punish_window_missed";
            if grab {
                // A missed window must not leave the player frozen at the spot
                // for the rest of this L hold. Do not chase into neutral play.
                out.owned = 0;
                out.stick = None;
            }
            return;
        }
        if self.tilt_animations > 0 {
            self.roll_phase = "roll_punish_complete";
            return;
        }
        if self.roll_intercept.is_none() {
            if fixed_grab && self.anchor.or(enemy.cliff_position).is_none() {
                self.roll_phase = "roll_fixed_grab_wait_ledge_anchor";
                return; // Do not substitute a moving opponent root for a ledge.
            }
            let ledge_x = self
                .anchor
                .or(enemy.cliff_position)
                .map_or(enemy.position[0], |p| p[0]);
            let inward = if self.inward.abs() > 0.5 {
                self.inward.signum()
            } else if (ctx.position[0] - ledge_x).abs() > 0.5 {
                (ctx.position[0] - ledge_x).signum()
            } else if enemy.velocity[0].abs() > 0.05 {
                enemy.velocity[0].signum()
            } else {
                -ctx.facing.signum()
            };
            // Commit immediately to one common coverage point. Do not wait for
            // roll velocity, predict a character's root path or chase its body.
            let endpoint = ledge_x
                + inward
                    * self
                        .fixed_roll_grab_inset
                        .filter(|_| grab)
                        .or(settings.roll_distance)
                        .unwrap_or(settings.fighter.common_roll_distance);
            let travel_limit = settings.follow_range;
            let destination = if fixed_grab {
                endpoint
            } else if grab {
                let dx = endpoint - ctx.position[0];
                let reach = settings.fighter.grab.roll_standoff();
                // Keep an already useful spot. Otherwise approach from the
                // nearest side; never cross the target to reach its ledge side.
                if dx.abs() <= reach + 1.5 {
                    ctx.position[0]
                } else {
                    endpoint - dx.signum() * reach
                }
            } else {
                endpoint
                    - if zair {
                        inward * crate::fighter_profiles::samus::ZAIR_ROLL_STANDOFF
                    } else {
                        0.0
                    }
            };
            // Keep the full calculated approach and the enemy timing endpoint.
            let travel = (destination - ctx.position[0]).clamp(-travel_limit, travel_limit);
            let mut stand_x = if fixed_grab {
                destination
            } else {
                ctx.position[0] + travel
            };
            if inward_walk_grab && !fixed_grab && (stand_x - ctx.position[0]) * inward < 0.0 {
                // Already beyond the approach point: hold here rather than
                // walking back toward the ledge and reversing the grab.
                stand_x = ctx.position[0];
            }
            let dx = stand_x - ctx.position[0];
            self.roll_intercept = Some(RollIntercept {
                endpoint,
                stand_x,
                facing: if inward_walk_grab || dx.abs() <= 0.5 {
                    inward
                } else {
                    dx.signum()
                },
            });
            self.roll_dash_origin = Some(ctx.position[0]);
        }
        let mut spot = self.roll_intercept.unwrap();
        if inward_walk_grab && !fixed_grab {
            // The common 25-unit point is only an initial approach estimate.
            // Extend it when native root travel shows a deeper roll. Never move
            // the aim back toward the ledge as the opponent passes our body.
            // A long tongue startup is not permission to extrapolate the roll's
            // peak speed for 14+ frames and lock a point far beyond its landing.
            let lead =
                f32::from(settings.fighter.grab.startup(ctx.status)).min(6.0) + ctx.input_delay;
            let forecast = ctx.position[0] + predicted_offset(ctx, enemy, lead)[0];
            let origin = self.roll_dash_origin.unwrap_or(ctx.position[0]);
            let forecast = forecast.clamp(
                origin - settings.follow_range,
                origin + settings.follow_range,
            );
            if forecast.is_finite() && (forecast - spot.endpoint) * spot.facing > 0.0 {
                spot.endpoint = forecast;
                spot.stand_x = forecast - spot.facing * settings.fighter.grab.roll_standoff();
                self.roll_intercept = Some(spot);
            }
        }
        let (first, last) = if zair {
            use crate::fighter_profiles::samus::{ZAIR_ACTIVE, ZAIR_HIT_LEAD, ZAIR_TAKEOFF_FRAMES};
            let first = ZAIR_TAKEOFF_FRAMES + ZAIR_ACTIVE.first;
            (first, first + ZAIR_HIT_LEAD)
        } else if grab {
            let first = settings.fighter.grab.startup(ctx.status);
            // Long tongue activity must participate in timing; otherwise a
            // legal catch late in that animation never opens the request gate.
            (
                first,
                first.saturating_add(settings.fighter.grab.active_frames.saturating_sub(1)),
            )
        } else if tilt {
            (
                settings.fighter.down_tilt.first,
                settings.fighter.down_tilt.last,
            )
        } else {
            let frames = settings
                .fighter
                .down_smash_toward(spot.endpoint - ctx.position[0], ctx.facing);
            (
                frames.first,
                (frames.first + settings.roll_hit_window_frames.clamp(1, 13) - 1).min(frames.last),
            )
        };
        let window = AttackWindow {
            first,
            last,
            range: if grab {
                settings.fighter.grab.standing_range
            } else {
                settings.roll_dash_range
            },
            height: [-6.0, 8.0],
            front_only: grab,
        };
        let contact_enemy = if grab { roll_grab_target(enemy) } else { enemy };
        let deadline = enemy
            .actionable_in
            .or_else(|| enemy.motion.and_then(|m| m.remaining));
        let timing_contact = window.timing_contact(ctx, contact_enemy, deadline);
        // A distance to the generic endpoint is not evidence that a grab can
        // reach the opponent. Test reach/facing and timing on the SAME active
        // frame, including the tongue's full active interval.
        let contact = if grab {
            window.contact_before(ctx, contact_enemy, deadline)
        } else {
            timing_contact
        };
        self.movement_preparing = true;
        if grab {
            let can_catch_here = contact.is_some()
                && ctx.can_grab
                && (!inward_walk_grab || ctx.facing * spot.facing > 0.5)
                && ctx.velocity[0] * spot.facing >= -0.15;
            // Kirby's fixed stance must be reached before waiting for a catch.
            // Tracked approaches can still take a legal catch along the way.
            if fixed_grab {
                if !self.position_for_fixed_roll_grab(ctx, spot, out) {
                    return;
                }
            }
            if !fixed_grab
                && !can_catch_here
                && !self.position_for_roll_grab(settings, ctx, spot, out)
            {
                return;
            }
        } else {
            // Non-grab responses retain their existing fixed-point navigation.
            let error = spot.stand_x - ctx.position[0];
            let approach_direction = spot.facing;
            let approach_speed = (ctx.velocity[0] * approach_direction).max(0.0);
            let stopping_distance = roll_stopping_distance(ctx, approach_direction)
                .unwrap_or(approach_speed * (ctx.input_delay.clamp(0.0, 12.0) + 2.0));
            self.roll_positioned |= error * spot.facing <= 1.5;
            if !self.roll_positioned {
                if approach_speed > 0.35 && stopping_distance + 1.5 >= error * approach_direction {
                    self.roll_phase = "roll_brake_before_destination";
                    return;
                }
                self.roll_phase = "roll_close_attack_range";
                let direction = error.signum();
                if ctx.can_act && ctx.safe_ground[usize::from(direction > 0.0)] {
                    if error.abs() <= stopping_distance + 3.0 {
                        out.stick = Some([direction * 0.3, 0.0]);
                    } else {
                        self.prepare_roll_dash(ctx, direction, out);
                    }
                }
                return;
            }
        }
        if grab && matches!(ctx.status, super::GUARD_ON | super::GUARD) {
            // Stay in guard while waiting for a shield grab, instead of dropping
            // it and spending the punish window on native guard-off recovery.
            out.requested = crate::native_input::MAPPER_ZR;
        }
        if contact.is_none() {
            self.roll_phase = if grab && timing_contact.is_some() {
                "roll_wait_actual_grab_reach"
            } else {
                "roll_wait_vulnerability"
            };
            return;
        }
        if self
            .last_output
            .stick
            .is_some_and(|stick| stick[0].abs() > 0.1)
        {
            self.movement_preparing = true;
            self.roll_phase = "roll_release_approach";
            return;
        }
        if grab || zair || tilt || face_smash {
            // Only non-grab options perform a final facing correction.
            let dx = if zair || grab {
                spot.endpoint
            } else {
                enemy.position[0]
            } - ctx.position[0];
            // Grab preserves the facing produced by its approach. Other
            // punish options retain their explicit facing correction.
            if !grab && dx * ctx.facing < -0.5 {
                if ctx.can_act
                    && ctx.safe_ground[usize::from(dx > 0.0)]
                    && self.movement_attempts < super::MAX_ATTEMPTS
                {
                    self.movement_attempts += 1;
                    out.stick = Some([dx.signum() * 0.3, 0.0]);
                }
                self.roll_phase = "roll_face_grab";
                return;
            }
            if zair {
                self.roll_phase = "roll_request_zair";
                self.start_samus_zair(ctx, out);
                return;
            }
            if grab {
                self.roll_phase = if !ctx.can_grab {
                    "roll_wait_native_catch"
                } else if self.attempts >= super::MAX_ATTEMPTS {
                    "roll_grab_attempt_limit"
                } else {
                    "roll_release_grab_retry"
                };
                // Native catch readiness is authoritative. A legal guard
                // grab must not fail merely because locomotion is unavailable.
                if ctx.can_grab && self.try_grab(settings, false) {
                    out.requested = settings.grab_button;
                    if matches!(ctx.status, super::GUARD_ON | super::GUARD) {
                        out.requested |= crate::native_input::MAPPER_ZR;
                    }
                    self.grab_delivery =
                        Some(super::attempt::GrabDelivery::new(ctx, out.requested));
                    self.roll_phase = "roll_request_grab";
                }
                return;
            }
        }
        if tilt {
            self.roll_phase = "roll_wait_tilt_ready";
            if ctx.can_act && ctx.can_down_tilt && self.try_tilt(Pending::DownTilt) {
                out.stick = Some([0.0, -0.6]);
                out.requested = MAPPER_A;
                self.roll_phase = "roll_request_down_tilt";
            }
            return;
        }
        if !ctx.can_down_smash {
            self.movement_preparing = true;
            self.roll_phase = "roll_wait_smash_ready";
            return;
        }
        self.roll_phase = "roll_wait_attack_release";
        if self.last_output.requested & MAPPER_A == 0 {
            self.attempts = self.attempts.saturating_add(1);
            self.pending = Pending::DownSmash;
            out.stick = Some([0.0, -1.0]);
            out.requested = MAPPER_A;
            self.roll_phase = "roll_request_down_smash";
        }
    }

    /// Kirby runs to his fixed stance, brakes, then waits for native grab timing.
    /// The rolling opponent cannot move this destination or trigger an early grab.
    fn position_for_fixed_roll_grab(
        &mut self,
        ctx: LedgeContext,
        spot: RollIntercept,
        out: &mut LedgeOutput,
    ) -> bool {
        let error = spot.stand_x - ctx.position[0];
        self.roll_positioned = error.abs() <= 1.0;
        out.owned |= crate::manual_controls::LEFT_STICK_DIRECTIONS;
        if !self.roll_positioned {
            self.roll_phase = "roll_fixed_grab_run_to_spot";
            let direction = error.signum();
            let coast =
                roll_stopping_distance(ctx, direction).unwrap_or(ctx.velocity[0].abs() * 2.0);
            if ctx.velocity[0] * direction < -0.15
                || ctx.velocity[0] * direction > 0.2 && coast >= error.abs() - 1.0
            {
                return false;
            }
            if (ctx.can_act || ctx.can_control && matches!(ctx.status, 0x3 | 0x4))
                && ctx.safe_ground[usize::from(direction > 0.0)]
            {
                if error.abs() < 3.0 {
                    // Fine correction stays below the dash threshold.
                    out.stick = Some([direction * 0.3, 0.0]);
                } else {
                    // Let the mapper deliver the full-stick dash flick here.
                    out.owned &= !crate::manual_controls::LEFT_STICK_DIRECTIONS;
                    self.prepare_roll_dash(ctx, direction, out);
                }
            }
            return false;
        }
        if ctx.facing * spot.facing < 0.5 {
            self.roll_phase = "roll_fixed_grab_face_center";
            if ctx.can_act {
                out.stick = Some([spot.facing * 0.3, 0.0]);
            }
            return false;
        }
        self.roll_phase = "roll_fixed_grab_wait_timing";
        true
    }

    pub(super) fn position_for_roll_grab(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        spot: RollIntercept,
        out: &mut LedgeOutput,
    ) -> bool {
        if matches!(
            settings.implementation,
            Some(super::ResponseImplementation::Kirby | super::ResponseImplementation::Yoshi)
        ) {
            return self.position_for_walk_roll_grab(settings, ctx, spot, out);
        }
        let reach = settings.fighter.grab.roll_usable_reach();
        let dx = spot.endpoint - ctx.position[0];
        let speed = ctx.velocity[0];
        let coast = roll_stopping_distance(ctx, speed.signum())
            .unwrap_or(speed.abs() * (ctx.input_delay.clamp(0.0, 12.0) + 2.0));
        let stop_x = ctx.position[0] + speed.signum() * coast;
        self.roll_positioned = dx.abs() <= reach;
        if self.roll_positioned {
            // A moving grab is fine if it stays in range on the same side.
            // Otherwise settle before facing, instead of turning mid-sprint.
            if speed.abs() > 0.2
                && ((stop_x - spot.endpoint).abs() > reach || dx * (spot.endpoint - stop_x) < 0.0)
            {
                self.roll_grab_braking = true;
                self.roll_phase = "roll_settle_grab";
                return false;
            }
            return true;
        }
        // Correct only into the nearest usable edge, with an interior margin.
        // Do not keep chasing the original exact standing coordinate.
        let origin = self.roll_dash_origin.unwrap_or(ctx.position[0]);
        let destination = (spot.endpoint - dx.signum() * reach * 0.8).clamp(
            origin - settings.follow_range,
            origin + settings.follow_range,
        );
        let error = destination - ctx.position[0];
        if error.abs() <= 1.5 {
            self.roll_phase = "roll_grab_destination_unreachable";
            return false;
        }
        let direction = error.signum();
        let toward_speed = speed * direction;
        if toward_speed < -0.2 {
            // Do not counter-steer a running fighter into turn-dash. First let
            // the queued approach/momentum finish, then walk back into range.
            self.roll_grab_braking = true;
            self.roll_phase = "roll_settle_grab";
            return false;
        }
        if toward_speed > 0.35 && coast + 1.5 >= error.abs() {
            self.roll_grab_braking = true;
            self.roll_phase = "roll_brake_before_destination";
            return false;
        }
        self.roll_phase = "roll_close_attack_range";
        if ctx.can_act && ctx.safe_ground[usize::from(direction > 0.0)] {
            if self.roll_grab_braking || direction * spot.facing < 0.0 || error.abs() <= coast + 3.0
            {
                out.stick = Some([direction * 0.3, 0.0]);
            } else {
                self.prepare_roll_dash(ctx, direction, out);
            }
        }
        false
    }

    /// Kirby/Yoshi walk inward only, stopping anywhere inside a usable catch
    /// area. Never walk through that area to hit one exact standing coordinate.
    fn position_for_walk_roll_grab(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        spot: RollIntercept,
        out: &mut LedgeOutput,
    ) -> bool {
        let inward = spot.facing;
        if ctx.facing * inward < 0.5 {
            self.roll_phase = "roll_walk_face_center";
            if ctx.can_act && ctx.safe_ground[usize::from(inward > 0.0)] {
                out.stick = Some([inward * 0.3, 0.0]);
            }
            return false;
        }
        let reach = settings.fighter.grab.roll_usable_reach();
        let gap = (spot.endpoint - ctx.position[0]) * inward;
        let speed = (ctx.velocity[0] * inward).max(0.0);
        let coast = roll_stopping_distance(ctx, inward)
            .unwrap_or(speed * (ctx.input_delay.clamp(0.0, 12.0) + 2.0));
        let stopped_gap = gap - coast;
        self.roll_positioned = (0.0..=reach).contains(&gap);
        if self.roll_positioned {
            if speed > 0.2 && !(0.0..=reach).contains(&stopped_gap) {
                self.roll_grab_braking = true;
                self.roll_phase = "roll_walk_brake_inside_grab_area";
                return false;
            }
            self.roll_phase = "roll_walk_grab_area_ready";
            // Walking can start a standing grab without reaching zero speed.
            // Actual can_grab is checked by the request path after release.
            return ctx.velocity[0] * inward >= -0.15
                && matches!(ctx.status, 0..=2 | super::GUARD_ON | super::GUARD);
        }
        if gap < 0.0 {
            self.roll_phase = "roll_walk_past_grab_area";
            return false; // No reversal into an outward-facing whiff.
        }
        // Brake for crossing the opponent, not for entering the outer edge of
        // the usable area. Braking outside that edge caused walk/brake loops.
        if speed > 0.2 && stopped_gap < 0.0 {
            self.roll_grab_braking = true;
            self.roll_phase = "roll_walk_brake";
            return false;
        }
        self.roll_phase = "roll_fast_walk_to_grab";
        if ctx.can_act && ctx.safe_ground[usize::from(inward > 0.0)] {
            // Below the dash flick threshold; never call prepare_roll_dash.
            out.stick = Some([inward * 0.65, 0.0]);
        }
        false
    }

    /// Keep a committed grab request alive across native delivery latency.
    /// Acceptance is observed elsewhere; this never throws or pummels.
    pub(super) fn continue_roll_grab_request(
        &mut self,
        ctx: LedgeContext,
        out: &mut LedgeOutput,
    ) -> bool {
        if self.plan != super::OptionPlan::Roll
            || self.pending != Pending::Grab
            || self.grab_in_progress
            || ctx.situation != 0
            || !ctx.can_control
            || ctx.stopped
        {
            self.grab_delivery = None;
            return false;
        }
        let Some(buttons) = self
            .grab_delivery
            .as_mut()
            .and_then(|delivery| delivery.sample(ctx))
        else {
            self.grab_delivery = None;
            return false;
        };
        self.movement_preparing = true;
        out.owned = super::LEDGE_BUTTONS;
        out.stick = Some([0.0, 0.0]);
        out.requested = buttons;
        self.roll_phase = "roll_deliver_grab";
        true
    }

    pub(super) fn prepare_roll_dash(
        &mut self,
        ctx: LedgeContext,
        direction: f32,
        out: &mut LedgeOutput,
    ) {
        if !matches!(ctx.status, 0..=0x9 | 0x11..=0x15) {
            return;
        }
        let running_toward = matches!(ctx.status, 0x3 | 0x4) && ctx.facing * direction > 0.5;
        self.roll_dash_origin.get_or_insert(ctx.position[0]);
        self.movement_preparing = true;
        self.roll_phase = "roll_start_dash";
        if running_toward {
            self.roll_phase = "roll_close_attack_range";
            out.stick = Some([direction, 0.0]);
        } else {
            self.movement_attempts = self.movement_attempts.saturating_add(1);
            if !self
                .last_output
                .stick
                .is_some_and(|stick| stick[0] * direction > 0.9)
            {
                out.stick = Some([direction, 0.0]);
            }
        }
    }
}

/// Prefer a first active frame inside the vulnerable recovery interval, not
/// a last-active-frame overlap at its opening. Aim at the interval midpoint
/// when narrow, or three frames after protection ends when it is wider.
pub(super) fn roll_grab_target(mut enemy: Opponent) -> Opponent {
    let Some(opens) = (if enemy.vulnerable {
        Some(0.0)
    } else {
        enemy.vulnerable_in
    }) else {
        return enemy; // Unknown protection remains unknown; never guess it away.
    };
    let deadline = enemy
        .actionable_in
        .or_else(|| enemy.motion.and_then(|m| m.remaining));
    let inset = deadline.map_or(2.0, |end| ((end - opens) * 0.5).clamp(0.0, 3.0));
    enemy.vulnerable = false;
    enemy.vulnerable_in = Some(opens + inset);
    enemy
}
