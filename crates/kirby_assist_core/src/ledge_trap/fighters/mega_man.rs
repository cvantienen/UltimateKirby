//! mega_man's compiled menu and named response functions.
//! Move windows are in fighter_profiles/mega_man/timing.rs. No runtime parsing.
use super::super::{LedgeContext, LedgeOutput, LedgeSettings, LedgeTrap, Opponent};
use super::{
    common::*,
    one_shot::{Move, MoveKind},
    standard, variety,
};
use crate::fighter_profiles::{
    mega_man as data,
    presets::{MenuPresets, SavedChoice, SavedList},
};
const BASE: MenuPresets = standard::menu(0, 0, false);
const FAIR: Move = Move {
    kind: MoveKind::Aerial {
        motion: 0xc3495ada5,
    },
    first: data::FORWARD_AIR.first,
    last: data::FORWARD_AIR.last,
    stick: [1.0, 0.0],
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
static NEUTRAL_OPTIONS: [SavedChoice; 3] = [
    BASE.lists[0].options[1],
    SavedChoice::new("up_tilt", "Up tilt", neutral_up_tilt, enable_neutral),
    SavedChoice::new(
        "area_smash",
        "Arm cannon (travel)",
        neutral_smash,
        enable_neutral,
    ),
];
static JUMP_OPTIONS: [SavedChoice; 2] = [
    SavedChoice::new("forward_air", "Forward air", jump_fair, enable_jump),
    SavedChoice::new("up_tilt", "Grounded up tilt", jump_up_tilt, enable_jump),
];
static ROLL_OPTIONS: [SavedChoice; 3] = [
    SavedChoice::new("down_smash", "Down smash", roll_smash, prepare_roll_smash),
    SavedChoice::new("grab", "Grab", roll_grab, enable_roll),
    SavedChoice::new("down_tilt", "Down tilt", roll_down_tilt, enable_roll),
];
static ATTACK_OPTIONS: [SavedChoice; 2] = [
    SavedChoice {
        name: "Grab / rear shield",
        prepare: prepare_safe_grab,
        ..BASE.lists[3].options[1]
    },
    SavedChoice::new("shield_fair", "Shield > fair", shield_fair, enable_attack),
];
static DROP_OPTIONS: [SavedChoice; 3] = [
    BASE.lists[4].options[0],
    BASE.lists[4].options[1],
    SavedChoice::new(
        "forward_air",
        "Hop fair (near)",
        drop_fair,
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
            selected: 1,
            options: &ROLL_OPTIONS,
        },
        SavedList {
            selected: 1,
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
fn prepare_safe_grab(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.attack_getup = true;
    s.responses.rear_block_nair = false; // Pellets are not the shared nair state.
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
    if s.responses.neutral_getup && t.climb_seen && e.status == crate::ledge_trap::CLIFF_CLIMB {
        // Projectile/resource attack: start early; don't pretend startup equals arrival.
        variety::drop(
            t,
            LedgeSettings {
                drop_defense: true,
                ..s
            },
            c,
            e,
            o,
            SMASH,
        );
    }
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
fn shield_fair(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::blocked(t, s, c, e, o, FAIR);
}
fn drop_fair(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::drop(t, s, c, e, o, FAIR);
}
