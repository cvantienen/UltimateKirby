//! Little Mac stays grounded by default. Tiny aerials are explicit joke choices.
use super::super::{LedgeContext, LedgeOutput, LedgeSettings, LedgeTrap, Opponent};
use super::common::{enable_jump, enable_roll, prepare_drop_shield};
use super::one_shot::{Move, MoveKind};
use super::variety;
use crate::fighter_profiles::{
    littlemac as data,
    presets::{MenuPresets, SavedChoice, SavedList},
};
const BASE: MenuPresets = super::standard::menu(1, 2, true);
const UP_TILT: Move = Move {
    kind: MoveKind::Ground {
        startup: 0x2b,
        attack: 0x2b,
    },
    first: data::UP_TILT.first,
    last: data::UP_TILT.last,
    stick: [0.0, 0.6],
    drift: 0.0,
};
const SMASH: Move = Move {
    kind: MoveKind::Ground {
        startup: 0x2d,
        attack: 0x2f,
    },
    first: data::FORWARD_SMASH.first,
    last: data::FORWARD_SMASH.last,
    stick: [1.0, 0.0],
    drift: 0.0,
};
const LOW_SMASH: Move = Move {
    first: data::LOW_FORWARD_SMASH.first,
    last: data::LOW_FORWARD_SMASH.last,
    stick: [1.0, -0.5],
    ..SMASH
};
const UP_AIR: Move = Move {
    kind: MoveKind::Aerial {
        motion: 0xdde67d935,
    },
    first: data::UP_AIR.first,
    last: data::UP_AIR.last,
    stick: [0.0, 1.0],
    drift: 0.0,
};
const SWEEP: Move = Move {
    kind: MoveKind::Ground {
        startup: 0x30,
        attack: 0x32,
    },
    first: data::LEDGE_DATA.down_smash.first,
    last: data::LEDGE_DATA.down_smash.last,
    stick: [0.0, -1.0],
    drift: 0.0,
};
const POKE: Move = Move {
    kind: MoveKind::Ground {
        startup: 0x2a,
        attack: 0x2a,
    },
    first: data::LEDGE_DATA.forward_tilt.first,
    last: data::LEDGE_DATA.forward_tilt.last,
    stick: [0.6, -0.3],
    drift: 0.0,
};
static NEUTRAL_OPTIONS: [SavedChoice; 6] = [
    SavedChoice::new("up_tilt", "Up tilt", neutral_up_tilt, enable_neutral),
    BASE.lists[0].options[1],
    BASE.lists[0].options[2],
    SavedChoice::new(
        "forward_smash",
        "Armored f-smash",
        neutral_smash,
        enable_neutral,
    ),
    SavedChoice::new(
        "low_forward_smash",
        "Low f-smash (precise)",
        neutral_low_smash,
        enable_neutral,
    ),
    SavedChoice {
        name: "Tiny nair (joke)",
        ..BASE.lists[0].options[0]
    },
];
static JUMP_OPTIONS: [SavedChoice; 3] = [
    SavedChoice::new("up_tilt", "Grounded up tilt", jump_up_tilt, enable_jump),
    SavedChoice::new("up_air", "Tiny up-air (joke)", jump_up_air, enable_jump),
    SavedChoice {
        name: "Tiny nair (joke)",
        ..BASE.lists[1].options[0]
    },
];
static ROLL_OPTIONS: [SavedChoice; 2] = [
    BASE.lists[2].options[0],
    SavedChoice::new("grab", "Grab", roll_grab, enable_roll),
];
static ATTACK_OPTIONS: [SavedChoice; 3] = [
    SavedChoice {
        name: "Tiny nair (joke)",
        ..BASE.lists[3].options[0]
    },
    BASE.lists[3].options[1],
    BASE.lists[3].options[2],
];
static DROP_OPTIONS: [SavedChoice; 4] = [
    BASE.lists[4].options[0],
    BASE.lists[4].options[1],
    SavedChoice::new(
        "sweep",
        "Low sweep (spacing)",
        drop_sweep,
        prepare_drop_shield,
    ),
    SavedChoice::new("poke", "F-tilt poke", drop_poke, prepare_drop_shield),
];
pub(crate) const MENU: MenuPresets = MenuPresets {
    lists: [
        SavedList::new(&NEUTRAL_OPTIONS),
        SavedList::new(&JUMP_OPTIONS),
        SavedList::new(&ROLL_OPTIONS),
        SavedList {
            selected: 2,
            options: &ATTACK_OPTIONS,
        },
        SavedList::new(&DROP_OPTIONS),
    ],
};
fn enable_neutral(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.neutral_getup = true;
    s
}
fn neutral_up_tilt(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::neutral(t, s, c, e, o, UP_TILT);
}
fn neutral_smash(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::neutral(t, s, c, e, o, SMASH);
}
fn neutral_low_smash(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::neutral(t, s, c, e, o, LOW_SMASH);
}
fn jump_up_tilt(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::jump(t, s, c, e, o, UP_TILT);
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
fn drop_sweep(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::drop(t, s, c, e, o, SWEEP);
}
fn drop_poke(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::drop(t, s, c, e, o, POKE);
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
