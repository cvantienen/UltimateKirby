//! mr_game_and_watch's compiled menu and named response functions.
//! Move windows are in fighter_profiles/mr_game_and_watch/timing.rs. No runtime parsing.
use super::super::{LedgeContext, LedgeOutput, LedgeSettings, LedgeTrap, Opponent};
use super::{
    common::*,
    one_shot::{Move, MoveKind},
    standard, variety,
};
use crate::fighter_profiles::{
    mr_game_and_watch as data,
    presets::{MenuPresets, SavedChoice, SavedList},
};
const BASE: MenuPresets = standard::menu(0, 0, true);
const LOW_SMASH: Move = Move {
    kind: MoveKind::Ground {
        startup: 0x30,
        attack: 0x32,
    },
    first: data::LEDGE_DATA.down_smash.first,
    last: data::LEDGE_DATA.down_smash.last,
    stick: [0.0, -1.0],
    drift: 0.0,
};
const NAIR: Move = Move {
    kind: MoveKind::Aerial {
        motion: 0xc3a4e2597,
    },
    first: data::LEDGE_DATA.neutral_air.first,
    last: data::LEDGE_DATA.neutral_air.last,
    stick: [0.0, 0.0],
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
static NEUTRAL_OPTIONS: [SavedChoice; 5] = [
    BASE.lists[0].options[0],
    BASE.lists[0].options[1],
    BASE.lists[0].options[2],
    SavedChoice::new("up_tilt", "Up tilt", neutral_up_tilt, enable_neutral),
    SavedChoice::new(
        "forward_smash",
        "Torch f-smash",
        neutral_smash,
        enable_neutral,
    ),
];
static JUMP_OPTIONS: [SavedChoice; 2] = [
    BASE.lists[1].options[0],
    SavedChoice::new("up_tilt", "Grounded up tilt", jump_up_tilt, enable_jump),
];
static ROLL_OPTIONS: [SavedChoice; 3] = [
    SavedChoice::new("down_smash", "Bury hammers", roll_smash, prepare_roll_smash),
    SavedChoice::new("grab", "Grab", roll_grab, enable_roll),
    SavedChoice::new("down_tilt", "Down tilt", roll_down_tilt, enable_roll),
];
static ATTACK_OPTIONS: [SavedChoice; 3] = [
    BASE.lists[3].options[0],
    BASE.lists[3].options[1],
    BASE.lists[3].options[2],
];
static DROP_OPTIONS: [SavedChoice; 4] = [
    BASE.lists[4].options[0],
    BASE.lists[4].options[1],
    SavedChoice::new(
        "drop_nair",
        "Hop nair (near)",
        drop_nair,
        prepare_drop_shield,
    ),
    SavedChoice::new(
        "down_smash",
        "Bury hammers",
        drop_smash,
        prepare_drop_shield,
    ),
];
pub(crate) const MENU: MenuPresets = MenuPresets {
    lists: [
        SavedList {
            selected: 0,
            options: &NEUTRAL_OPTIONS,
        },
        SavedList {
            selected: 0,
            options: &JUMP_OPTIONS,
        },
        SavedList {
            selected: 0,
            options: &ROLL_OPTIONS,
        },
        SavedList {
            selected: 2,
            options: &ATTACK_OPTIONS,
        },
        SavedList {
            selected: 0,
            options: &DROP_OPTIONS,
        },
    ],
};
fn enable_neutral(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.neutral_getup = true;
    s
}
fn prepare_roll_smash(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.roll_getup = true;
    s.roll_hit_window_frames =
        (data::LEDGE_DATA.down_smash.last - data::LEDGE_DATA.down_smash.first + 1).min(13);
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
fn jump_up_tilt(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::jump(t, s, c, e, o, UP_TILT);
}
fn roll_smash(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    t.respond_to_roll(s, c, e, o);
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
fn roll_down_tilt(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    t.respond_to_roll_down_tilt(s, c, e, o);
}
fn drop_nair(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::drop(t, s, c, e, o, NAIR);
}
fn drop_smash(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::drop(t, s, c, e, o, LOW_SMASH);
}
