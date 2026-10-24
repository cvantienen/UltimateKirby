//! Pac-Man: lingering ghosts, long grab, and paired short-hop aerials.
use super::super::{LedgeContext, LedgeOutput, LedgeSettings, LedgeTrap, Opponent};
use super::common::{enable_jump, enable_roll, prepare_drop_shield};
use super::one_shot::{Move, MoveKind, OneShot};
use super::variety;
use crate::fighter_profiles::{
    pacman as data,
    presets::{MenuPresets, SavedChoice, SavedList},
};
const BASE: MenuPresets = super::standard::menu(0, 0, true);
const GHOST: Move = Move {
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
const LOW_GHOSTS: Move = Move {
    kind: MoveKind::Ground {
        startup: 0x30,
        attack: 0x32,
    },
    first: data::LEDGE_DATA.down_smash.first,
    last: data::LEDGE_DATA.down_smash.last,
    stick: [0.0, -1.0],
    drift: 0.0,
};
const FRUIT_THROW: Move = Move {
    // Pinned skyline-smash: SPECIAL_N, Pac-Man N_HOLD and N_SHOOT. Release B
    // immediately on entry, so a stored-fruit tap cannot become a held recharge.
    kind: MoveKind::Special {
        statuses: &[0x1dc, 0x1e1, 0x1e2],
    },
    first: data::FRUIT_THROW_STARTUP,
    last: data::FRUIT_THROW_STARTUP,
    stick: [0.0, 0.0],
    drift: 0.0,
};
static NEUTRAL_OPTIONS: [SavedChoice; 5] = [
    BASE.lists[0].options[0],
    BASE.lists[0].options[1],
    BASE.lists[0].options[2],
    SavedChoice::new("ghost", "Ghost f-smash", neutral_ghost, enable_neutral),
    SavedChoice::new(
        "fruit_throw",
        "Fruit (stored)",
        neutral_fruit,
        enable_neutral,
    ),
];
static JUMP_OPTIONS: [SavedChoice; 4] = [
    BASE.lists[1].options[0],
    SavedChoice::new("up_air", "Up air", jump_up_air, enable_jump),
    SavedChoice::new("forward_air", "Forward air", jump_fair, enable_jump),
    SavedChoice::new("fruit_throw", "Fruit (stored)", jump_fruit, enable_jump),
];
static ROLL_OPTIONS: [SavedChoice; 2] = [
    SavedChoice {
        name: "Lingering ghosts",
        prepare: lingering_roll,
        ..BASE.lists[2].options[0]
    },
    SavedChoice::new("grab", "Long grab", roll_grab, enable_roll),
];
static ATTACK_OPTIONS: [SavedChoice; 3] = [
    BASE.lists[3].options[0],
    BASE.lists[3].options[1],
    SavedChoice {
        name: "Shield > trampoline",
        ..BASE.lists[3].options[2]
    },
];
static DROP_OPTIONS: [SavedChoice; 4] = [
    BASE.lists[4].options[0],
    BASE.lists[4].options[1],
    SavedChoice::new(
        "forward_air",
        "Short-hop bite",
        drop_fair,
        prepare_drop_shield,
    ),
    SavedChoice::new(
        "ghosts",
        "Low ghosts (spacing)",
        drop_ghosts,
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
fn neutral_fruit(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if s.responses.neutral_getup && t.climb_seen && e.status == super::super::CLIFF_CLIMB {
        throw_stored_fruit(t, c, e, o);
    }
}
fn jump_fruit(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if s.responses.jump_getup
        && (super::super::CLIFF_JUMP1..=super::super::CLIFF_JUMP3).contains(&e.status)
    {
        throw_stored_fruit(t, c, e, o);
    }
}
fn throw_stored_fruit(t: &mut LedgeTrap, c: LedgeContext, e: Opponent, o: &mut LedgeOutput) {
    if c.situation != 0 || !c.can_act || !c.can_control || c.stopped {
        return;
    }
    // Preconditions: manually prepared trampoline/spacing and charge-cancelled
    // stored fruit. No item/article scan, auto-charge or trampoline color edits.
    let dx = e.position[0] - c.position[0];
    if dx.abs() > 0.5 && dx * c.facing < -0.5 {
        t.movement_preparing = true;
        o.stick = Some([dx.signum() * 0.6, 0.0]);
        o.requested = 0;
        return; // Confirm facing before neutral B; never select side-B by drift.
    }
    // Recognition, not an invulnerability/travel countdown, starts one tap.
    t.fighter_move = Some(OneShot::new(FRUIT_THROW, c.facing).with_immediate_special());
    t.continue_fighter_move(c, o);
}
fn lingering_roll(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.roll_getup = true;
    // Existing roll scheduler permits at most 13 active frames; 15-27 lies
    // wholly inside Pac-Man's continuous 15-28 ghost interval.
    s.roll_hit_window_frames = 13;
    s
}
fn neutral_ghost(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::neutral(t, s, c, e, o, GHOST);
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
fn drop_fair(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::drop(t, s, c, e, o, FAIR);
}
fn drop_ghosts(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::drop(t, s, c, e, o, LOW_GHOSTS);
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
