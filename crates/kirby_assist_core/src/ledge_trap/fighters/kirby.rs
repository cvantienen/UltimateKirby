//! Kirby's compiled response functions and their menu registrations. A new
//! variation is a named function plus a table entry, not a config action string.
//! Shared timing/input helpers keep the existing mechanics unchanged.
use super::super::{
    GetupAttack, LedgeContext, LedgeDropAction, LedgeOutput, LedgeSettings, LedgeTrap, Opponent,
    OptionPlan,
};
use crate::fighter_profiles::{
    kirby::{NAIR_HIT_LEAD, STONE_HIT_LEAD},
    presets::{MenuPresets, PresetSlot, SavedChoice, SavedList},
};
mod drop_dair;
mod jump_bair;
mod variants;
pub(crate) use drop_dair::DropDair;

pub(crate) const MENU: MenuPresets = MenuPresets {
    lists: [
        SavedList::new(&[
            SavedChoice::new(
                "nair",
                "Nair",
                respond_to_neutral_getup_nair,
                prepare_neutral_nair,
            ),
            SavedChoice::new(
                "down_tilt",
                "Down tilt",
                respond_to_neutral_getup_down_tilt,
                prepare_neutral_down_tilt,
            ),
            SavedChoice::new(
                "forward_tilt",
                "Forward tilt",
                respond_to_neutral_getup_forward_tilt,
                prepare_neutral_forward_tilt,
            ),
            SavedChoice::new(
                "stone",
                "Down-B / Stone",
                respond_to_neutral_getup_stone,
                prepare_neutral_stone,
            ),
            SavedChoice::new(
                "inhale",
                "Neutral-B / Inhale",
                respond_to_neutral_getup_inhale,
                prepare_neutral_inhale,
            ),
            SavedChoice::new("bair", "Bair", jump_bair::neutral, prepare_neutral_nair),
        ]),
        SavedList::new(&[
            SavedChoice::new("nair", "Nair", respond_to_jump_getup_nair, enable_jump),
            SavedChoice::new("bair", "Bair", jump_bair::respond, enable_jump),
        ]),
        SavedList::new(&[
            SavedChoice::new(
                "down_smash",
                "Down smash",
                respond_to_roll_down_smash,
                enable_roll,
            ),
            SavedChoice::new("grab", "Grab", respond_to_roll_grab, enable_roll),
        ]),
        SavedList::new(&[
            SavedChoice::new(
                "shield_grab",
                "Grab / rear nair",
                respond_to_getup_attack_grab_rear_nair,
                prepare_grab_rear_nair,
            ),
            SavedChoice::new(
                "shield_rear",
                "Grab / rear shield",
                respond_to_getup_attack_grab_rear_shield,
                prepare_grab_rear_shield,
            ),
        ]),
        SavedList::new(&[
            SavedChoice::new(
                "shield",
                "Shield",
                respond_to_ledge_drop_shield,
                prepare_drop_shield,
            ),
            SavedChoice::new(
                "down_tilt_shield",
                "Down tilt > shield",
                respond_to_ledge_drop_down_tilt_then_shield,
                prepare_drop_down_tilt,
            ),
            SavedChoice::new(
                "short_hop_dair",
                "Short-hop dair",
                respond_to_ledge_drop_short_hop_dair,
                prepare_drop_dair,
            ),
        ]),
    ],
};

pub(super) fn respond(
    trap: &mut LedgeTrap,
    settings: LedgeSettings,
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
    if let Some(index) = settings.response_choices[slot as usize] {
        // An invalid index produces no action. Never fall through to a different
        // response if a future caller supplies a bad menu selection.
        if let Some(choice) = MENU.list(slot).options.get(index) {
            (choice.respond)(trap, settings, ctx, enemy, out);
        }
        return;
    }
    // Compatibility for direct core callers and text-only policy fixtures.
    // Live battle selections always resolve a compiled function index above.
    match trap.plan {
        OptionPlan::Watch => respond_to_neutral_getup_configured(trap, settings, ctx, enemy, out),
        OptionPlan::NeutralAir => trap.respond_to_jump(settings, ctx, enemy, out),
        OptionPlan::Roll => trap.respond_to_roll(settings, ctx, enemy, out),
        OptionPlan::Shield { .. } => trap.respond_to_getup_attack(settings, ctx, enemy, out),
        OptionPlan::DropShield if settings.drop_action == LedgeDropAction::ShortHopDair => {
            respond_to_ledge_drop_short_hop_dair(trap, settings, ctx, enemy, out);
        }
        OptionPlan::DropShield => super::common::respond_to_ledge_drop(trap, settings, ctx, out),
    }
}

fn respond_to_neutral_getup_configured(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    // Hanging stance is shared by all normal-getup choices.
    if !super::super::is_hanging(enemy.status) {
        match s.getup_attack {
            GetupAttack::DownSpecial => {
                variants::neutral_stone(trap, s, ctx, enemy, out);
                return;
            }
            GetupAttack::NeutralSpecial => {
                variants::neutral_inhale(trap, s, ctx, enemy, out);
                return;
            }
            _ => {}
        }
    }
    trap.watch_ledge(s, ctx, enemy, out);
}
fn respond_to_neutral_getup_nair(
    trap: &mut LedgeTrap,
    mut s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    s.fighter.normal_hit_lead_frames = s.fighter.normal_hit_lead_frames.saturating_add(2);
    let committed = trap.jump_committed;
    respond_to_neutral_getup_configured(trap, s, ctx, enemy, out);
    if !committed && trap.jump_committed {
        set_nair_nudge(trap, ctx, enemy);
    }
}
fn set_nair_nudge(trap: &mut LedgeTrap, ctx: LedgeContext, enemy: Opponent) {
    let dx = enemy.position[0] - ctx.position[0];
    let direction = if dx.abs() > 0.5 {
        dx.signum()
    } else {
        ctx.facing.signum()
    };
    trap.neutral_nair_drift = direction * 0.25;
}
fn respond_to_neutral_getup_down_tilt(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    respond_to_neutral_getup_configured(trap, s, ctx, enemy, out);
}
fn respond_to_neutral_getup_forward_tilt(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    respond_to_neutral_getup_configured(trap, s, ctx, enemy, out);
}
fn respond_to_neutral_getup_stone(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    respond_to_neutral_getup_configured(trap, s, ctx, enemy, out);
}
fn respond_to_neutral_getup_inhale(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    respond_to_neutral_getup_configured(trap, s, ctx, enemy, out);
}
fn respond_to_roll_grab(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    trap.respond_to_roll_grab(s, ctx, enemy, out);
}
fn prepare_neutral_inhale(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.neutral_getup = true;
    s.getup_attack = GetupAttack::NeutralSpecial;
    s
}
fn respond_to_jump_getup_nair(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    let committed = trap.jump_committed;
    trap.respond_to_jump(s, ctx, enemy, out);
    if !committed && trap.jump_committed {
        set_nair_nudge(trap, ctx, enemy);
    }
}
fn respond_to_roll_down_smash(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    trap.respond_to_roll(s, ctx, enemy, out);
}
fn respond_to_getup_attack_grab_rear_nair(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    trap.respond_to_getup_attack(s, ctx, enemy, out);
}
fn respond_to_getup_attack_grab_rear_shield(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    trap.respond_to_getup_attack(s, ctx, enemy, out);
}
fn respond_to_ledge_drop_shield(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    _: Opponent,
    out: &mut LedgeOutput,
) {
    super::common::respond_to_ledge_drop(trap, s, ctx, out);
}
fn respond_to_ledge_drop_down_tilt_then_shield(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    _: Opponent,
    out: &mut LedgeOutput,
) {
    super::common::respond_to_ledge_drop(trap, s, ctx, out);
}
fn respond_to_ledge_drop_short_hop_dair(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    if !s.drop_defense || ctx.situation != 0 || !ctx.can_act || !ctx.can_jump {
        return;
    }
    // Commit one direction at recognition, never chase their airborne position.
    // Prefer the witnessed ledge's outward direction; facing is the final fallback.
    let direction = if trap.anchor.is_some() && trap.inward.abs() > 0.5 {
        -trap.inward.signum()
    } else if (enemy.position[0] - ctx.position[0]).abs() > 0.5 {
        (enemy.position[0] - ctx.position[0]).signum()
    } else {
        ctx.facing.signum()
    };
    trap.drop_dair = Some(DropDair::new(direction));
    trap.continue_drop_dair(ctx, out);
}
// Preparation only resolves native-readiness metadata and numeric tuning. It
// runs when a selection changes; the response function owns the action sequence.
fn prepare_neutral_nair(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.neutral_getup = true;
    s.getup_attack = GetupAttack::NeutralAir;
    s.fighter.normal_hit_lead_frames = s
        .response_adjustments
        .nair_hit_lead
        .unwrap_or(NAIR_HIT_LEAD);
    s
}
fn prepare_neutral_down_tilt(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.neutral_getup = true;
    s.getup_attack = GetupAttack::DownTilt;
    s
}
fn prepare_neutral_forward_tilt(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.neutral_getup = true;
    s.getup_attack = GetupAttack::ForwardTilt;
    s
}
fn prepare_neutral_stone(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.neutral_getup = true;
    s.getup_attack = GetupAttack::DownSpecial;
    s.fighter.normal_hit_lead_frames = s
        .response_adjustments
        .stone_hit_lead
        .unwrap_or(STONE_HIT_LEAD);
    if let Some(startup) = s.response_adjustments.stone_startup {
        s.fighter.down_special_startup = Some(startup);
    }
    s
}
fn enable_jump(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.jump_getup = true;
    s
}
fn enable_roll(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.roll_getup = true;
    s
}
fn prepare_grab_rear_nair(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.attack_getup = true;
    s.responses.rear_block_nair = true;
    s
}
fn prepare_grab_rear_shield(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.attack_getup = true;
    s.responses.rear_block_nair = false;
    s
}
fn prepare_drop_shield(mut s: LedgeSettings) -> LedgeSettings {
    s.drop_defense = true;
    s.drop_action = LedgeDropAction::Shield;
    s
}
fn prepare_drop_down_tilt(mut s: LedgeSettings) -> LedgeSettings {
    s.drop_defense = true;
    s.drop_action = LedgeDropAction::DownTiltShield;
    s
}
fn prepare_drop_dair(mut s: LedgeSettings) -> LedgeSettings {
    s.drop_defense = true;
    s.drop_action = LedgeDropAction::ShortHopDair;
    s
}
