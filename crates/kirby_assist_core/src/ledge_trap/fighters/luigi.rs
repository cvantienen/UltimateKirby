//! Luigi's named ledge responses. These reuse the existing engines
//! with this fighter's move frames. Menu entries are compiled function pointers.
use super::super::{LedgeContext, LedgeOutput, LedgeSettings, LedgeTrap, Opponent};
use super::common::*;
use crate::fighter_profiles::presets::{MenuPresets, SavedChoice, SavedList};

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
        ]),
        SavedList::new(&[SavedChoice::new(
            "nair",
            "Nair",
            respond_to_jump_getup_nair,
            enable_jump,
        )]),
        SavedList::new(&[SavedChoice::new(
            "down_smash",
            "Down smash",
            respond_to_roll_down_smash,
            enable_roll,
        )]),
        SavedList::new(&[
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
        ]),
    ],
};

fn respond_to_neutral_getup_nair(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    trap.watch_ledge(s, ctx, enemy, out);
}
fn respond_to_neutral_getup_down_tilt(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    trap.watch_ledge(s, ctx, enemy, out);
}
fn respond_to_neutral_getup_forward_tilt(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    trap.watch_ledge(s, ctx, enemy, out);
}
fn respond_to_jump_getup_nair(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    trap.respond_to_jump(s, ctx, enemy, out);
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
fn respond_to_getup_attack_nair(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    enemy: Opponent,
    out: &mut LedgeOutput,
) {
    shield_nair(trap, s, ctx, enemy, out);
}
fn respond_to_getup_attack_grab(
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
    respond_to_ledge_drop(trap, s, ctx, out);
}
fn respond_to_ledge_drop_down_tilt_then_shield(
    trap: &mut LedgeTrap,
    s: LedgeSettings,
    ctx: LedgeContext,
    _: Opponent,
    out: &mut LedgeOutput,
) {
    respond_to_ledge_drop(trap, s, ctx, out);
}
