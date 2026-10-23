//! Hanging stance, legacy grounded get-up moves and shared ground movement.
//!
//! Neutral get-up's default nair is delegated to aerial.rs. The tilt branches
//! remain for existing configs. Stick values request ordinary game movement;
//! the support probes can refuse an approach, and no position is written here.
use super::{
    finite, ground_attack_ready, is_hanging, predicted_offset, AttackWindow, GetupAttack,
    LedgeContext, LedgeOutput, LedgeSettings, LedgeTrap, Opponent, Pending, MAX_ATTEMPTS,
};
use crate::native_input::MAPPER_A;

impl LedgeTrap {
    /// Position while hanging; prepare the selected grounded normal-getup tilt.
    pub(super) fn watch_ledge(
        &mut self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        enemy: Opponent,
        out: &mut LedgeOutput,
    ) {
        let dx = enemy.position[0] - ctx.position[0];
        if is_hanging(enemy.status) && self.position_at_ledge(settings, ctx, out) {
            self.grounded_tilt_prepared = false;
            return;
        }
        if is_hanging(enemy.status) && ctx.can_act {
            out.stick = Some([0.0, 0.6]);
            self.grounded_tilt_prepared = false;
            return;
        }
        if !settings.responses.neutral_getup {
            return;
        }
        if settings.getup_attack == GetupAttack::NeutralAir {
            self.respond_to_normal_nair(settings, ctx, enemy, out);
            return;
        }
        // Wait in place during a normal get-up. Only a bounded, eligible
        // facing correction may move Kirby before the selected tilt.
        out.stick = Some([0.0, 0.0]);
        if !ctx.can_act
            || !ctx.can_tilt
            || !ground_attack_ready(ctx, settings)
            || enemy.situation != 0
        {
            self.grounded_tilt_prepared = false;
            return;
        }
        // To change normal-getup move selection, start with this match and
        // verify its real stick recipe, active frames, and confirmation status.
        // Config selects between existing moves; range changes cannot add a move.
        let (pending, first, last) = match settings.getup_attack {
            GetupAttack::DownTilt => (
                Pending::DownTilt,
                settings.fighter.down_tilt.first,
                settings.fighter.down_tilt.last,
            ),
            GetupAttack::ForwardTilt => (
                Pending::ForwardTilt,
                settings.fighter.forward_tilt.first,
                settings.fighter.forward_tilt.last,
            ),
            GetupAttack::NeutralAir | GetupAttack::DownSpecial | GetupAttack::NeutralSpecial => {
                unreachable!("handled before grounded tilt selection")
            }
        };
        let forecast = AttackWindow {
            first,
            last,
            range: settings.tilt_range,
            height: [-5.0, 3.0],
            front_only: false,
        };
        let contact_dx = forecast
            .scheduled_contact(ctx, enemy, None, settings.distance_agnostic)
            .map(|contact| predicted_offset(ctx, enemy, contact.in_frames)[0]);
        let aim = contact_dx.unwrap_or(dx);
        let direction = if aim.abs() > 0.5 {
            aim.signum()
        } else {
            ctx.facing.signum()
        };
        let stick = match settings.getup_attack {
            GetupAttack::DownTilt => [0.0, -0.6],
            GetupAttack::ForwardTilt => [direction * 0.6, -0.3],
            GetupAttack::NeutralAir | GetupAttack::DownSpecial | GetupAttack::NeutralSpecial => {
                unreachable!("handled before grounded tilt selection")
            }
        };
        if aim * ctx.facing < -0.5 {
            if self.climb_seen
                && contact_dx.is_some()
                && ctx.safe_ground[usize::from(direction > 0.0)]
                && self.movement_attempts < MAX_ATTEMPTS
            {
                self.movement_attempts += 1;
                self.movement_preparing = true;
                out.stick = Some([direction * 0.3, 0.0]);
            }
            return;
        }
        // The attack is directional even though the player need not aim it.
        // Recheck geometry in Kirby's confirmed native facing before pressing A.
        let window = AttackWindow {
            front_only: true,
            ..forecast
        };
        if settings.getup_attack == GetupAttack::ForwardTilt || settings.manual_spacing {
            self.grounded_tilt_prepared = false;
            // Moderate tilt direction + A together; never pre-hold forward
            // while waiting for protection, reach, or a retry release edge.
            if self.climb_seen
                && window
                    .scheduled_contact(ctx, enemy, None, settings.distance_agnostic)
                    .is_some()
                && self.try_tilt(pending)
            {
                out.stick = Some(stick);
                out.requested = MAPPER_A;
            }
            return;
        }
        if predicted_offset(ctx, enemy, f32::from(first))[0].abs() > settings.tilt_range {
            self.grounded_tilt_prepared = false;
            return;
        }
        out.stick = Some(stick);
        let prepared = self.grounded_tilt_prepared && self.last_output.stick == Some(stick);
        if prepared
            && self.climb_seen
            && window
                .scheduled_contact(ctx, enemy, None, settings.distance_agnostic)
                .is_some()
            && self.try_tilt(pending)
        {
            out.requested = MAPPER_A;
        }
        self.grounded_tilt_prepared = true;
    }

    /// Shared by hanging coverage and post-punish recovery. Returning true
    /// reserves this sample for movement/turning; never add an attack to it.
    pub(super) fn position_at_ledge(
        &self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        out: &mut LedgeOutput,
    ) -> bool {
        if settings.manual_spacing || !settings.auto_position || !ctx.can_act {
            return false;
        }
        let Some(anchor) = self.anchor else {
            return false;
        };
        // The anchor is the stage edge, not the launched opponent's position.
        let error = anchor[0] + self.inward * settings.stand_distance
            - (ctx.position[0] + finite(ctx.velocity[0]) * 2.0);
        if self.inward != 0.0 && error.abs() > 0.75 {
            self.ground_move(settings, ctx, error, out);
            return true;
        }
        let toward = anchor[0] - ctx.position[0];
        if toward * ctx.facing < -0.5 {
            self.ground_move(settings, ctx, toward, out);
            return true;
        }
        false
    }

    pub(super) fn ground_move(
        &self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        error: f32,
        out: &mut LedgeOutput,
    ) {
        if !error.is_finite() || error.abs() < 0.5 {
            return;
        }
        // Bool -> index: false=0 chooses left ground probe; true=1 chooses right.
        // A blocked probe leaves the default neutral stick. Movement never forces
        // position/velocity; the game still applies Kirby's ordinary acceleration.
        let index = usize::from(error > 0.0);
        if ctx.safe_ground[index] {
            let strength = if error.abs() < 3.0 || error * ctx.facing < 0.0 {
                0.3
            } else {
                settings.move_strength
            };
            out.stick = Some([error.signum() * strength, 0.0]);
        }
    }
    pub(super) fn approach(
        &self,
        settings: LedgeSettings,
        ctx: LedgeContext,
        dx: f32,
        distance: f32,
        out: &mut LedgeOutput,
    ) {
        if settings.auto_position && dx.abs() > distance.max(0.0) {
            self.ground_move(settings, ctx, dx - dx.signum() * distance.max(0.0), out);
        }
    }
}
