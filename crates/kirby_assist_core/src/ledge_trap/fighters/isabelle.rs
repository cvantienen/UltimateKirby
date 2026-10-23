//! isabelle's compiled menu; only the selected single-move response executes.
use super::super::{LedgeContext, LedgeOutput, LedgeSettings, LedgeTrap, Opponent};
use super::common::{enable_attack, enable_jump, enable_roll, prepare_drop_shield};
use super::one_shot::{Move, MoveKind};
use super::variety;
mod slingshot;
use crate::fighter_profiles::{
    isabelle as data,
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
        startup: 0x2d,
        attack: 0x2f,
    },
    first: data::FORWARD_SMASH.first,
    last: data::FORWARD_SMASH.last,
    stick: [1.0, 0.0],
    drift: 0.0,
};
static NEUTRAL_OPTIONS: [SavedChoice; 5] = [
    BASE.lists[0].options[0],
    BASE.lists[0].options[1],
    BASE.lists[0].options[2],
    SavedChoice::new("party_popper", "Party popper", neutral_fun, enable_neutral),
    SavedChoice::new("slingshot", "Slingshot", slingshot::neutral, enable_neutral),
];
static JUMP_OPTIONS: [SavedChoice; 3] = [
    BASE.lists[1].options[0],
    SavedChoice::new("up_air", "Up air", jump_up_air, enable_jump),
    SavedChoice::new("forward_air", "Slingshot", jump_fair, enable_jump),
];
static ROLL_OPTIONS: [SavedChoice; 3] = [
    BASE.lists[2].options[0],
    SavedChoice::new("grab", "Net grab", roll_grab, enable_roll),
    SavedChoice::new("slingshot", "Slingshot", slingshot::roll, enable_roll),
];
static ATTACK_OPTIONS: [SavedChoice; 4] = [
    BASE.lists[3].options[0],
    BASE.lists[3].options[1],
    SavedChoice::new(
        "shield_up_air",
        "Shield > up air",
        shield_up_air,
        enable_attack,
    ),
    SavedChoice::new(
        "slingshot",
        "Shield > slingshot",
        slingshot::attack,
        enable_attack,
    ),
];
static DROP_OPTIONS: [SavedChoice; 4] = [
    BASE.lists[4].options[0],
    BASE.lists[4].options[1],
    SavedChoice::new(
        "short_hop_dair",
        "Turnip spike (risky)",
        drop_dair,
        prepare_drop_shield,
    ),
    SavedChoice::new("slingshot", "Slingshot", drop_fun, prepare_drop_shield),
];
pub(crate) const MENU: MenuPresets = MenuPresets {
    lists: [
        SavedList::new(&NEUTRAL_OPTIONS),
        SavedList::new(&JUMP_OPTIONS),
        SavedList {
            selected: 0,
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
    slingshot::jump(t, s, c, e, o);
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
    slingshot::drop(t, s, c, e, o);
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
