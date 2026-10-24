//! Stationary missile zoning and Samus-specific melee alternatives. Projectile
//! travel/homing isn't simulated: fire early on recognition, don't promise an
//! arrival frame or chase a rolling opponent. Native startup ends each request.
use super::*;
use crate::fighter_profiles::samus as data;
use crate::ledge_trap::fighters::{
    one_shot::{Move, MoveKind, OneShot},
    variety,
};

// Pinned skyline-smash: generic SPECIAL_S, Samus S1G (homing), S2G (super).
const HOMING: Move = Move {
    kind: MoveKind::Special {
        statuses: &[0x1dd, 0x1eb],
    },
    first: data::HOMING_MISSILE_STARTUP,
    last: data::HOMING_MISSILE_STARTUP,
    stick: [data::HOMING_MISSILE_STICK, 0.0],
    drift: 0.0,
};
const SUPER: Move = Move {
    kind: MoveKind::Special {
        statuses: &[0x1dd, 0x1ed],
    },
    first: data::SUPER_MISSILE_STARTUP,
    last: data::SUPER_MISSILE_STARTUP,
    stick: [1.0, 0.0], // Neutral primer then a full flick selects Super Missile.
    drift: 0.0,
};
const FAIR: Move = Move {
    kind: MoveKind::Aerial {
        motion: 0xc3495ada5,
    },
    first: data::FORWARD_AIR_FIRST.first,
    last: data::FORWARD_AIR_FIRST.last,
    stick: [1.0, 0.0],
    drift: 0.0,
};
const UP_AIR: Move = Move {
    kind: MoveKind::Aerial {
        motion: 0xdde67d935,
    },
    first: data::UP_AIR_FIRST.first,
    last: data::UP_AIR_FIRST.last,
    stick: [0.0, 1.0],
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

fn missile(t: &mut LedgeTrap, c: LedgeContext, e: Opponent, o: &mut LedgeOutput, recipe: Move) {
    if c.situation != 0 || !c.can_act || !c.can_control || c.stopped {
        return;
    }
    if face_target(t, c, e.position[0] - c.position[0], o) {
        // No melee-range/actionable-in veto: these are ranged area denial, not
        // a claim that startup alone fits the opponent's short punish window.
        t.start_fighter_move(c, o, recipe, false);
    }
}

fn homing_missile(t: &mut LedgeTrap, c: LedgeContext, e: Opponent, o: &mut LedgeOutput) {
    if c.situation != 0 || !c.can_act || !c.can_control || c.stopped {
        return;
    }
    let dx = e.position[0] - c.position[0];
    let direction = if dx.abs() > 0.5 {
        dx.signum()
    } else {
        c.facing.signum()
    };
    // Pair a deliberate side input with B on the first actionable recognition
    // sample, even if currently facing away. Do not send neutral B or wait for
    // vulnerability/arrival timing; native side-special startup ends the request.
    t.fighter_move = Some(OneShot::new(HOMING, direction).with_immediate_special());
    t.continue_fighter_move(c, o);
}
pub(super) fn neutral_homing(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if s.responses.neutral_getup && t.climb_seen && e.status == CLIFF_CLIMB {
        homing_missile(t, c, e, o);
    }
}
pub(super) fn neutral_super(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if s.responses.neutral_getup && t.climb_seen && e.status == CLIFF_CLIMB {
        missile(t, c, e, o, SUPER);
    }
}
pub(super) fn jump_homing(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if s.responses.jump_getup
        && (e.situation == 2 || (CLIFF_JUMP1..=CLIFF_JUMP3).contains(&e.status))
    {
        homing_missile(t, c, e, o);
    }
}
pub(super) fn jump_super(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if s.responses.jump_getup
        && (e.situation == 2 || (CLIFF_JUMP1..=CLIFF_JUMP3).contains(&e.status))
    {
        missile(t, c, e, o, SUPER);
    }
}
pub(super) fn roll_homing(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if s.responses.roll_getup {
        homing_missile(t, c, e, o);
    }
}
pub(super) fn roll_super(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if s.responses.roll_getup {
        missile(t, c, e, o, SUPER);
    }
}
pub(super) fn drop_homing(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if s.drop_defense {
        homing_missile(t, c, e, o);
    }
}
pub(super) fn drop_super(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if s.drop_defense {
        missile(t, c, e, o, SUPER);
    }
}
pub(super) fn neutral_smash(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::neutral(t, s, c, e, o, SMASH);
}
pub(super) fn jump_fair(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::jump(t, s, c, e, o, FAIR);
}
pub(super) fn jump_up_air(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::jump(t, s, c, e, o, UP_AIR);
}
pub(super) fn shield_fair(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::blocked(t, s, c, e, o, FAIR);
}
pub(super) fn shield_up_air(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::blocked(t, s, c, e, o, UP_AIR);
}
pub(super) fn drop_fair(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::drop(t, s, c, e, o, FAIR);
}
