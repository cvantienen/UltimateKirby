//! Kirby specials share timed ordinary B inputs, not hitbox/status writes.
//! Spacing never vetoes these requests; the real move still needs to reach.
use super::super::super::{
    LedgeContext, LedgeOutput, LedgeSettings, LedgeTrap, Opponent, Pending, CLIFF_CLIMB,
};
use crate::fighter_profiles::kirby::{INHALE_HIT_LEAD, INHALE_STARTUP};
use crate::native_input::MAPPER_B;

pub(super) fn neutral_stone(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    let Some(startup) = s.fighter.down_special_startup else {
        return;
    };
    timed_special(
        trap,
        s,
        ctx,
        enemy,
        out,
        SpecialRequest {
            pending: Pending::DownSpecial,
            startup,
            lead: s.fighter.normal_hit_lead_frames,
            stick: [0.0, -1.0],
        },
    );
}

pub(super) fn neutral_inhale(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    timed_special(
        trap,
        s,
        ctx,
        enemy,
        out,
        SpecialRequest {
            pending: Pending::NeutralSpecial,
            startup: INHALE_STARTUP,
            lead: INHALE_HIT_LEAD,
            stick: [0.0, 0.0],
        },
    );
}

struct SpecialRequest {
    pending: Pending,
    startup: u8,
    lead: u8,
    stick: [f32; 2],
}
fn timed_special(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
    request: SpecialRequest,
) {
    let committed = trap.pending == request.pending;
    if !s.responses.neutral_getup || !trap.climb_seen || ctx.situation != 0 || !ctx.can_act {
        if committed {
            trap.variant_attempt.expired = true;
        }
        return;
    }
    if !ctx.input_delay.is_finite() || !(0.0..=120.0).contains(&ctx.input_delay) {
        return;
    }
    if !committed {
        if enemy.status != CLIFF_CLIMB {
            return;
        }
        let Some(vulnerable_in) = (if enemy.vulnerable {
            Some(0.0)
        } else {
            enemy.vulnerable_in
        })
        .filter(|v| v.is_finite() && (0.0..=120.0).contains(v)) else {
            return;
        };
        // Like nair: first active frame + learned delivery delay + early lead.
        // Inhale's lingering catch window gets lead 8. Stone defaults to lead 0
        // so its actual first hit is aimed at protection ending, not before it.
        if vulnerable_in > f32::from(request.startup) + ctx.input_delay + f32::from(request.lead) {
            return;
        }
    }
    // Native grounded actionability is sufficient to request B, including
    // walking/running. Do not add range/facing or narrow idle/transition gates.
    let fresh = trap.variant_attempt.advance(ctx, false, 30);
    if trap.variant_attempt.expired {
        return;
    }
    trap.movement_preparing = true;
    if !fresh {
        *out = trap.last_output;
        return;
    }
    out.stick = Some([0.0, 0.0]);
    if trap.variant_attempt.holding_special(ctx)
        || trap
            .variant_attempt
            .issue(ctx, trap.last_output.requested, MAPPER_B)
    {
        trap.pending = request.pending;
        // Hold the direction and B together for delay+2 advancing frames.
        // Neutral release samples separate bounded retries. Native acceptance
        // hands control back in observe_action before this function runs again.
        out.stick = Some(request.stick);
        out.requested = MAPPER_B;
    }
}
