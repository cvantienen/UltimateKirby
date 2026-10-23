//! Yoshi's single-move choices. Defaults keep lingering nair and safe drop shield;
//! slower headbutts and offstage chases are explicit alternatives.
use super::super::{LedgeContext, LedgeOutput, LedgeSettings, LedgeTrap, Opponent};
use super::common::{enable_attack, enable_jump, enable_roll, prepare_drop_shield};
use super::one_shot::{Move, MoveKind, OneShot};
use super::variety;

use crate::fighter_profiles::{
    presets::{MenuPresets, SavedChoice, SavedList},
    yoshi as data,
};
const BASE: MenuPresets = super::standard::menu(0, 0, false);
// Jump-getup nair deliberately bypasses the generic hop/timing forecast: pair
// neutral jump+A immediately, then drift only after native nair acceptance so
// horizontal input cannot accidentally select fair/back-air during startup.
const JUMP_NAIR: Move = Move {
    kind: MoveKind::Aerial {
        motion: 0xc3a4e2597,
    },
    first: data::LEDGE_DATA.neutral_air.first,
    last: data::LEDGE_DATA.neutral_air.last,
    stick: [0.0, 0.0],
    drift: data::JUMP_NAIR_DRIFT,
};
const UP_TILT: Move = Move {
    kind: MoveKind::Ground {
        startup: 0x2b,
        attack: 0x2b,
    },
    first: data::UP_TILT.first,
    last: data::UP_TILT.last,
    stick: [0.0, 0.6], // Moderate vertical input selects tilt, not up-smash.
    drift: 0.0,
};
const HEADBUTT: Move = Move {
    kind: MoveKind::Ground {
        startup: 0x2d,
        attack: 0x2f,
    },
    first: data::FORWARD_SMASH.first,
    last: data::FORWARD_SMASH.last,
    stick: [1.0, 0.0],
    drift: 0.0,
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
const FAIR: Move = Move {
    kind: MoveKind::Aerial {
        motion: 0xc3495ada5,
    },
    first: data::FORWARD_AIR.first,
    last: data::FORWARD_AIR.last,
    stick: [1.0, 0.0],
    drift: 0.0,
};
// Drop-only drift is relative to facing the opponent. One chase, with no
// fast-fall, double-jump or recovery autopilot; the player handles recovery.
const DROP_FAIR: Move = Move { drift: 1.0, ..FAIR };
const DROP_DAIR: Move = Move {
    kind: MoveKind::Aerial {
        motion: 0xd40042152,
    },
    first: data::DOWN_AIR_FIRST,
    last: data::DOWN_AIR_LAST,
    stick: [0.6, -1.0],
    drift: 1.0,
};
// Static backing arrays satisfy SavedList's 'static lifetime. Existing indices
// and defaults remain in place so adding options doesn't change a saved choice.
static NEUTRAL_OPTIONS: [SavedChoice; 5] = [
    SavedChoice::new("nair", "Nair", neutral_nair, prepare_neutral_nair),
    BASE.lists[0].options[1],
    BASE.lists[0].options[2],
    SavedChoice::new("up_tilt", "Up tilt", neutral_up_tilt, enable_neutral),
    SavedChoice::new(
        "forward_smash",
        "Headbutt f-smash",
        neutral_headbutt,
        enable_neutral,
    ),
];
static JUMP_OPTIONS: [SavedChoice; 3] = [
    SavedChoice::new("nair", "Nair", jump_nair, enable_jump),
    SavedChoice::new("up_air", "Up air", jump_up_air, enable_jump),
    SavedChoice::new(
        "forward_air",
        "Meteor fair (spacing)",
        jump_fair,
        enable_jump,
    ),
];
static ROLL_OPTIONS: [SavedChoice; 2] = [
    SavedChoice::new("down_smash", "Down smash", roll_down_smash, enable_roll),
    SavedChoice::new("grab", "Tongue grab", roll_grab, enable_roll),
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
        "drop_fair",
        "Meteor chase (risky)",
        drop_fair,
        prepare_drop_shield,
    ),
    SavedChoice::new(
        "short_hop_dair",
        "Dair shred (risky)",
        drop_dair,
        prepare_drop_shield,
    ),
];
pub(crate) const MENU: MenuPresets = MenuPresets {
    lists: [
        SavedList::new(&NEUTRAL_OPTIONS),
        SavedList::new(&JUMP_OPTIONS),
        SavedList::new(&ROLL_OPTIONS),
        SavedList::new(&ATTACK_OPTIONS),
        SavedList::new(&DROP_OPTIONS),
    ],
};
fn prepare_neutral_nair(s: LedgeSettings) -> LedgeSettings {
    let mut s = super::common::prepare_neutral_nair(s);
    // Yoshi's requested lead is independent of the shared config's Kirby-era
    // eight-frame default. Keep the real nair startup/active frames unchanged.
    s.fighter.normal_hit_lead_frames = data::NEUTRAL_NAIR_LEAD;
    s
}
fn neutral_nair(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    let already_committed = t.jump_committed;
    t.watch_ledge(s, c, e, o);
    if !already_committed && t.jump_committed {
        // The shared hop path owns the neutral X+A launch. Drift starts only
        // after it observes the exact native nair, so the attack stays neutral.
        let dx = e.position[0] - c.position[0];
        let direction = if dx.abs() > 0.5 {
            dx.signum()
        } else {
            c.facing.signum()
        };
        t.neutral_nair_drift = direction * data::NEUTRAL_NAIR_DRIFT;
    }
}
fn jump_nair(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if !s.responses.jump_getup
        || c.stopped
        || c.situation != 0
        || !c.can_act && !matches!(c.status, super::super::GUARD_ON | super::super::GUARD)
        || !c.can_jump
        || e.situation == 0
            && !(super::super::CLIFF_JUMP1..=super::super::CLIFF_JUMP3).contains(&e.status)
    {
        return;
    }
    let dx = e.position[0] - c.position[0];
    let direction = if dx.abs() > 0.5 {
        dx.signum()
    } else {
        c.facing.signum()
    };
    // Already launches on first recognition; there is no earlier observed cue.
    // Brief, low-strength nudge only after neutral nair acceptance.
    t.fighter_move = Some(
        OneShot::new(JUMP_NAIR, direction).with_drift_until(super::super::NAIR_NUDGE_END_FRAME),
    );
    t.continue_fighter_move(c, o);
}
fn roll_down_smash(
    t: &mut LedgeTrap,
    mut s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    let distance = s.roll_distance.unwrap_or(s.fighter.common_roll_distance);
    s.roll_distance = Some((distance - data::ROLL_DOWN_SMASH_STANDOFF).max(2.0));
    t.respond_to_roll(s, c, e, o);
}

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
fn neutral_headbutt(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::neutral(t, s, c, e, o, HEADBUTT);
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
fn roll_grab(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    t.respond_to_roll_grab(s, c, e, o);
}
fn drop_fair(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::drop(t, s, c, e, o, DROP_FAIR);
}
fn drop_dair(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::drop(t, s, c, e, o, DROP_DAIR);
}
