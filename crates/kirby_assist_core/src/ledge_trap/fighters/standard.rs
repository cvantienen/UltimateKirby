//! Shared compiled response functions for the starting cast. Each registration
//! supplies its own move data/defaults; no Kirby-specific special is inherited.
use super::super::{
    LedgeContext, LedgeOutput, LedgeSettings, LedgeTrap, Opponent, OptionPlan, Pending, GUARD,
    GUARD_DAMAGE, GUARD_ON,
};
use super::common::*;
use crate::fighter_profiles::presets::{MenuPresets, SavedChoice, SavedList};
use crate::native_input::{MAPPER_B, MAPPER_ZR};

// SavedList stores static slices. Define their backing arrays at module scope
// rather than borrowing temporary arrays inside the const menu constructor.
const NEUTRAL_OPTIONS: &[SavedChoice] = &[
    SavedChoice::new(
        "nair",
        "Nair",
        respond_to_neutral_getup,
        prepare_neutral_nair,
    ),
    SavedChoice::new(
        "down_tilt",
        "Down tilt",
        respond_to_neutral_getup,
        prepare_neutral_down_tilt,
    ),
    SavedChoice::new(
        "forward_tilt",
        "Forward tilt",
        respond_to_neutral_getup,
        prepare_neutral_forward_tilt,
    ),
];
const JUMP_OPTIONS: &[SavedChoice] = &[SavedChoice::new(
    "nair",
    "Nair",
    respond_to_jump_getup_nair,
    enable_jump,
)];
const ROLL_OPTIONS: &[SavedChoice] = &[SavedChoice::new(
    "down_smash",
    "Down smash",
    respond_to_roll_down_smash,
    enable_roll,
)];
const ATTACK_OPTIONS: &[SavedChoice] = &[
    SavedChoice::new(
        "shield_nair",
        "Shield > nair",
        respond_to_getup_attack_nair,
        enable_attack,
    ),
    SavedChoice::new(
        "shield_grab",
        "Grab / rear nair",
        respond_to_getup_attack_grab,
        enable_attack,
    ),
];
const ATTACK_UP_B_OPTIONS: &[SavedChoice] = &[
    ATTACK_OPTIONS[0],
    ATTACK_OPTIONS[1],
    SavedChoice::new(
        "shield_up_b",
        "Shield > Up-B",
        respond_to_getup_attack_up_special,
        enable_attack,
    ),
];
const DROP_OPTIONS: &[SavedChoice] = &[
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
];

pub(super) const fn menu(neutral: usize, attack: usize, up_b: bool) -> MenuPresets {
    MenuPresets {
        lists: [
            SavedList {
                selected: neutral,
                options: NEUTRAL_OPTIONS,
            },
            SavedList::new(JUMP_OPTIONS),
            SavedList::new(ROLL_OPTIONS),
            SavedList {
                selected: attack,
                options: if up_b {
                    ATTACK_UP_B_OPTIONS
                } else {
                    ATTACK_OPTIONS
                },
            },
            SavedList::new(DROP_OPTIONS),
        ],
    }
}
fn respond_to_getup_attack_up_special(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    _: Opponent,
    out: &mut LedgeOutput,
) {
    if !s.responses.attack_getup || s.fighter.up_special_startup.is_none() || ctx.situation != 0 {
        return;
    }
    out.stick = Some([0.0; 2]);
    out.requested = MAPPER_ZR;
    // Preserve shield through the actual block/stun; no opponent-distance or
    // speculative animation-end check can launch this before block evidence.
    if !matches!(trap.plan, OptionPlan::Shield { blocked: true })
        || ctx.status == GUARD_DAMAGE
        || ctx.stopped
        || !matches!(ctx.status, GUARD_ON | GUARD) && !ctx.can_act
    {
        return;
    }
    let fresh = trap.variant_attempt.advance(ctx, false, 30);
    trap.movement_preparing = true;
    if !fresh {
        *out = trap.last_output;
        return;
    }
    if trap.variant_attempt.holding_special(ctx)
        || trap
            .variant_attempt
            .issue(ctx, trap.last_output.requested, MAPPER_B)
    {
        trap.pending = Pending::UpSpecial;
        out.requested = MAPPER_ZR | MAPPER_B;
        out.stick = Some([0.0, 1.0]);
    }
}
fn respond_to_neutral_getup(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    t.watch_ledge(s, c, e, o);
}
fn respond_to_jump_getup_nair(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    t.respond_to_jump(s, c, e, o);
}
fn respond_to_roll_down_smash(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    t.respond_to_roll(s, c, e, o);
}
fn respond_to_getup_attack_nair(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    shield_nair(t, s, c, e, o);
}
fn respond_to_getup_attack_grab(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    t.respond_to_getup_attack(s, c, e, o);
}
fn respond_to_ledge_drop_shield(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    _: Opponent,
    o: &mut LedgeOutput,
) {
    respond_to_ledge_drop(t, s, c, o);
}
fn respond_to_ledge_drop_down_tilt_then_shield(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    _: Opponent,
    o: &mut LedgeOutput,
) {
    respond_to_ledge_drop(t, s, c, o);
}
