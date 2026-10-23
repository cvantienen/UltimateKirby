//! Kirby bair responses: backward C-stick -> short-hop jump -> forward stick
//! when front-facing, direct short-hop bair when already back-facing.
//! Timing follows observed native states; no wall-clock waits or status writes.
use super::super::super::{
    LedgeContext, LedgeOutput, LedgeSettings, LedgeTrap, Opponent, CLIFF_CLIMB, CLIFF_JUMP1,
    CLIFF_JUMP3, JUMP_SQUAT_FRAMES,
};
use super::super::one_shot::{Move, MoveKind, OneShot};
use crate::fighter_profiles::kirby::{
    JUMP_BAIR, JUMP_BAIR_DRIFT, NEUTRAL_BAIR_DRIFT, NEUTRAL_BAIR_LEAD,
};
use crate::timing::motion_hash;

pub(super) fn respond(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if !s.responses.jump_getup
        || c.stopped
        || c.situation != 0
        || !c.can_jump
        || e.situation == 0 && !(CLIFF_JUMP1..=CLIFF_JUMP3).contains(&e.status)
    {
        return;
    }
    start(t, c, e, o, JUMP_BAIR_DRIFT);
}

pub(super) fn neutral(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if super::super::super::is_hanging(e.status) {
        t.watch_ledge(s, c, e, o);
        return;
    }
    if !s.responses.neutral_getup
        || !t.climb_seen
        || e.status != CLIFF_CLIMB
        || c.stopped
        || c.situation != 0
        || !c.can_jump
    {
        return;
    }
    o.stick = Some([0.0, 0.0]);
    let Some(opens) = (if e.vulnerable {
        Some(0.0)
    } else {
        e.vulnerable_in
    })
    .filter(|v| v.is_finite() && (0.0..=120.0).contains(v)) else {
        return;
    };
    let setup = u8::from(c.facing * away_from(c, e) < 0.5);
    let startup = f32::from(JUMP_SQUAT_FRAMES + JUMP_BAIR.first + setup) + c.input_delay;
    if opens > startup + f32::from(NEUTRAL_BAIR_LEAD)
        || e.actionable_in.is_some_and(|end| end <= startup)
    {
        return;
    }
    start(t, c, e, o, NEUTRAL_BAIR_DRIFT);
}

fn away_from(c: LedgeContext, e: Opponent) -> f32 {
    let dx = e.position[0] - c.position[0];
    let toward = if dx.abs() > 0.5 {
        dx.signum()
    } else {
        -c.facing.signum()
    };
    -toward
}

fn start(t: &mut LedgeTrap, c: LedgeContext, e: Opponent, o: &mut LedgeOutput, drift: f32) {
    let away = away_from(c, e);
    let recipe = Move {
        kind: MoveKind::Aerial {
            motion: motion_hash("attack_air_b"),
        },
        first: JUMP_BAIR.first,
        last: JUMP_BAIR.last,
        stick: [-1.0, 0.0],
        drift: -drift,
    };
    let action = OneShot::new(recipe, away);
    t.fighter_move = Some(if c.facing * away < 0.5 {
        action.with_attack_cancel()
    } else {
        action.with_turn_before_jump()
    });
    t.continue_fighter_move(c, o);
}
