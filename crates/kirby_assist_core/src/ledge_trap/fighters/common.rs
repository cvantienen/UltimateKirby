//! Shared recipes, used only when a fighter explicitly registers a wrapper.
//! No fighter lookup or allocation happens here; all inputs are copied values.
use super::super::{
    ground_attack_ready, GetupAttack, LedgeContext, LedgeDropAction, LedgeOutput, LedgeSettings,
    LedgeTrap, Opponent, OptionPlan, DOWN_TILT, GUARD, GUARD_DAMAGE, GUARD_ON,
};
use crate::fighter_profiles::presets::{MenuPresets, PresetSlot};
use crate::native_input::{MAPPER_A, MAPPER_ZR};

/// Optional approach during an existing neutral-nair wait. Call the attack
/// handler FIRST: position must never become a prerequisite for launching.
pub(super) fn walk_before_neutral_nair(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
    stick: f32,
    inset: f32,
) {
    let launch_in = f32::from(super::super::JUMP_SQUAT_FRAMES + s.fighter.neutral_air.first)
        + ctx.input_delay
        + f32::from(s.fighter.normal_hit_lead_frames);
    if enemy.status == super::super::CLIFF_CLIMB
        && trap.climb_seen
        && !enemy.vulnerable
        && enemy
            .vulnerable_in
            .is_some_and(|left| left.is_finite() && left <= 120.0 && left > launch_in)
    {
        walk_toward_ledge(trap, ctx, out, stick, inset);
    }
}

/// Shared movement only; each opt-in response decides whether it is still
/// waiting. Never issue buttons or modify an attack's timing/commit state.
pub(super) fn walk_toward_ledge(
    trap: &mut LedgeTrap,
    ctx: LedgeContext,
    out: &mut LedgeOutput,
    stick: f32,
    inset: f32,
) {
    if trap.jump_committed
        || ctx.stopped
        || ctx.situation != 0
        || !ctx.can_act
        || !ctx.can_control
        || !matches!(ctx.status, 0..=2)
        || out.requested != 0
    {
        return;
    }
    let Some(anchor) = trap.anchor.filter(|p| p.iter().all(|v| v.is_finite())) else {
        return;
    };
    if !ctx
        .position
        .iter()
        .chain(ctx.velocity.iter())
        .all(|v| v.is_finite())
        || (ctx.position[1] - anchor[1]).abs() > 6.0
        || !trap.inward.is_finite()
        || trap.inward.abs() < 0.5
    {
        return;
    }
    let inward = trap.inward.signum();
    let toward_ledge = -inward;
    let gap = (ctx.position[0] - anchor[0]) * inward - inset;
    let coast = super::super::prediction::roll_stopping_distance(ctx, toward_ledge)
        .unwrap_or(ctx.velocity[0].abs() * 2.0);
    // Keep ownership while braking so physical input cannot push past the stop.
    trap.movement_preparing = true;
    // Explicit ownership lets the native overlay suppress horizontal flick
    // aliases while preserving this fast analog walk request.
    out.owned |= crate::manual_controls::LEFT_STICK_DIRECTIONS;
    out.stick = Some([0.0, 0.0]);
    if gap > coast + 0.5 && ctx.safe_ground[usize::from(toward_ledge > 0.0)] {
        out.stick = Some([toward_ledge * stick, 0.0]);
    }
}

pub(super) fn dispatch(
    menu: &MenuPresets,
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    let slot = match trap.plan {
        OptionPlan::Watch => PresetSlot::Neutral,
        OptionPlan::NeutralAir => PresetSlot::Jump,
        OptionPlan::Roll => PresetSlot::Roll,
        OptionPlan::Shield { .. } => PresetSlot::Attack,
        OptionPlan::DropShield => PresetSlot::Drop,
    };
    let list = menu.list(slot);
    let index = s.response_choices[slot as usize].unwrap_or(list.selected);
    if let Some(choice) = list.options.get(index) {
        (choice.respond)(trap, s, ctx, enemy, out);
    }
}

pub(super) fn shield_nair(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    if !s.responses.attack_getup {
        return;
    }
    if matches!(trap.plan, OptionPlan::Shield { blocked: true }) {
        // The existing committed-hop continuation retains neutral A through
        // jump squat and hitstop, even when the opponent leaves tracking range.
        trap.shield_nair = true;
        trap.respond_to_rear_block_nair(s, ctx, enemy, out);
    } else if ctx.situation == 0 {
        out.stick = Some([0.0, 0.0]);
        out.requested = MAPPER_ZR;
    }
}

pub(super) fn prepare_neutral_nair(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.neutral_getup = true;
    s.getup_attack = GetupAttack::NeutralAir;
    // A numeric lead must fit this fighter's actual continuous active interval.
    // In particular Falcon's 7-8 and 13-15 hits are not one lingering hitbox.
    s.fighter.normal_hit_lead_frames = s
        .response_adjustments
        .nair_hit_lead
        .unwrap_or(s.fighter.normal_hit_lead_frames)
        .min(s.fighter.neutral_air.last - s.fighter.neutral_air.first);
    s
}
pub(super) fn prepare_neutral_down_tilt(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.neutral_getup = true;
    s.getup_attack = GetupAttack::DownTilt;
    s
}
pub(super) fn prepare_neutral_forward_tilt(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.neutral_getup = true;
    s.getup_attack = GetupAttack::ForwardTilt;
    s
}
pub(super) fn enable_jump(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.jump_getup = true;
    s
}
pub(super) fn enable_roll(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.roll_getup = true;
    s
}
pub(super) fn enable_attack(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.attack_getup = true;
    s.responses.rear_block_nair = true;
    s
}
pub(super) fn prepare_drop_shield(mut s: LedgeSettings) -> LedgeSettings {
    s.drop_defense = true;
    s.drop_action = LedgeDropAction::Shield;
    s
}
pub(super) fn prepare_drop_down_tilt(mut s: LedgeSettings) -> LedgeSettings {
    s.drop_defense = true;
    s.drop_action = LedgeDropAction::DownTiltShield;
    s
}
// Moved intact from Kirby's variants: all three fighters share the same
// one-shot down-tilt acceptance, timeout and shield recovery mechanics.
pub(super) fn respond_to_ledge_drop(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    out: &mut LedgeOutput,
) {
    if !s.drop_defense {
        return;
    }
    if s.drop_action == LedgeDropAction::Shield {
        shield(ctx, out);
        return;
    }
    if s.drop_action != LedgeDropAction::DownTiltShield {
        return; // Fighter-specific aerial recipes have their own compiled handler.
    }
    // Actual down-tilt startup completes the attack step even on whiff. A hit
    // must still be followed by shield, so don't use the normal punish-hit flag.
    trap.movement_preparing = ctx.situation == 0;
    let fresh =
        trap.variant_attempt
            .advance(ctx, ctx.status == DOWN_TILT, s.drop_request_timeout_frames);
    if trap.variant_attempt.accepted || trap.variant_attempt.expired {
        if ctx.can_act || matches!(ctx.status, GUARD_ON | GUARD | GUARD_DAMAGE) {
            shield(ctx, out);
        }
        return;
    }
    if !fresh {
        *out = trap.last_output;
        return;
    }
    if ctx.situation != 0 || !ctx.can_control {
        return;
    }
    trap.movement_preparing = true; // Own the recipe's neutral release sample.
    out.stick = Some([0.0, 0.0]);
    if ctx.can_act
        && ctx.can_down_tilt
        && ground_attack_ready(ctx, s)
        && trap
            .variant_attempt
            .issue(ctx, trap.last_output.requested, MAPPER_A)
    {
        out.stick = Some([0.0, -0.6]);
        out.requested = MAPPER_A;
    }
    // Exhausted/late requests fall back to shield, without a landing chase.
    if trap.variant_attempt.expired && ctx.can_act {
        shield(ctx, out);
    }
}
fn shield(ctx: LedgeContext, out: &mut LedgeOutput) {
    if ctx.situation == 0 {
        out.stick = Some([0.0, 0.0]);
        out.requested = MAPPER_ZR;
    }
}
