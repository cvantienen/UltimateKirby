//! krool's compiled menu; only the selected single-move response executes.
use super::super::{LedgeContext, LedgeOutput, LedgeSettings, LedgeTrap, Opponent};
use super::common::{enable_attack, enable_jump, enable_roll, prepare_drop_shield};
use super::one_shot::{Move, MoveKind};
use super::variety;
use crate::fighter_profiles::{
    krool as data,
    presets::{MenuPresets, SavedChoice, SavedList},
};
const BASE: MenuPresets = super::standard::menu(0, 0, false);
const UP_AIR: Move = Move {
    kind: MoveKind::Aerial {
        motion: 0xdde67d935,
    },
    first: data::UP_AIR.first,
    last: data::UP_AIR.last,
    stick: [0.0, 1.0],
    drift: 0.0,
};
const FAIR: Move = Move {
    kind: MoveKind::Aerial {
        motion: 0xc3495ada5,
    },
    first: data::FORWARD_AIR.first,
    last: data::FORWARD_AIR.last,
    stick: [1.0, 0.0],
    drift: 0.0,
};
const DAIR: Move = Move {
    kind: MoveKind::Aerial {
        motion: 0xd40042152,
    },
    first: data::DOWN_AIR.first,
    last: data::DOWN_AIR.last,
    stick: [0.6, -1.0],
    drift: 1.0,
};
const FUN: Move = Move {
    kind: MoveKind::Ground {
        startup: 0x30,
        attack: 0x32,
    },
    first: data::LEDGE_DATA.down_smash.first,
    last: data::LEDGE_DATA.down_smash.last,
    stick: [0.0, -1.0],
    drift: 0.0,
};
static NEUTRAL_OPTIONS: [SavedChoice; 4] = [
    BASE.lists[0].options[0],
    BASE.lists[0].options[1],
    BASE.lists[0].options[2],
    SavedChoice::new("belly_flop", "Belly flop", neutral_fun, enable_neutral),
];
static JUMP_OPTIONS: [SavedChoice; 3] = [
    BASE.lists[1].options[0],
    SavedChoice::new("up_air", "Up air", jump_up_air, enable_jump),
    SavedChoice::new("forward_air", "Forward air", jump_fair, enable_jump),
];
static ROLL_OPTIONS: [SavedChoice; 2] = [
    BASE.lists[2].options[0],
    SavedChoice::new("grab", "Grab", roll_grab, enable_roll),
];
static ATTACK_OPTIONS: [SavedChoice; 3] = [
    BASE.lists[3].options[0],
    BASE.lists[3].options[1],
    SavedChoice::new(
        "shield_up_air",
        "Shield > up air",
        shield_up_air,
        enable_attack,
    ),
];
static DROP_OPTIONS: [SavedChoice; 4] = [
    BASE.lists[4].options[0],
    BASE.lists[4].options[1],
    SavedChoice::new(
        "short_hop_dair",
        "Spike (risky)",
        drop_dair,
        prepare_drop_shield,
    ),
    SavedChoice::new(
        "belly_flop",
        "Belly flop (risky)",
        drop_fun,
        prepare_drop_shield,
    ),
];
pub(crate) const MENU: MenuPresets = MenuPresets {
    lists: [
        SavedList::new(&NEUTRAL_OPTIONS),
        SavedList::new(&JUMP_OPTIONS),
        SavedList {
            selected: 1,
            options: &ROLL_OPTIONS,
        },
        SavedList::new(&ATTACK_OPTIONS),
        SavedList::new(&DROP_OPTIONS),
    ],
};
fn enable_neutral(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.neutral_getup = true;
    s
}
fn neutral_fun(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::neutral(t, s, c, e, o, FUN);
}
fn jump_up_air(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::jump(t, s, c, e, o, UP_AIR);
}
fn jump_fair(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::jump(t, s, c, e, o, FAIR);
}
fn shield_up_air(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::blocked(t, s, c, e, o, UP_AIR);
}
fn drop_dair(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::drop(t, s, c, e, o, DAIR);
}
fn drop_fun(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::drop(t, s, c, e, o, FUN);
}
fn roll_grab(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    t.respond_to_roll_grab(s, c, e, o);
}
