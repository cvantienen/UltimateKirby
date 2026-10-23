//! Stationary slingshot coverage. Only the selected handler runs; the native
//! projectile handles flight/collision after emission. Travel is not predicted.
use super::super::one_shot::{Move, MoveKind, OneShot};
use crate::fighter_profiles::isabelle as data;
use crate::ledge_trap::{
    LedgeContext, LedgeOutput, LedgeSettings, LedgeTrap, Opponent, OptionPlan, CLIFF_CLIMB,
    CLIFF_ESCAPE, CLIFF_JUMP1, CLIFF_JUMP3, GUARD, GUARD_DAMAGE, GUARD_ON, JUMP_SQUAT_FRAMES,
};
use crate::native_input::MAPPER_ZR;
use crate::timing::motion_hash;

fn recipe(c: LedgeContext, target_x: f32) -> Move {
    let back = (target_x - c.position[0]) * c.facing < -0.5;
    let frames = if back {
        data::BACK_AIR
    } else {
        data::FORWARD_AIR
    };
    Move {
        kind: MoveKind::Aerial {
            motion: motion_hash(if back { "attack_air_b" } else { "attack_air_f" }),
        },
        first: frames.first,
        last: frames.last,
        stick: [if back { -1.0 } else { 1.0 }, 0.0],
        drift: 0.0,
    }
}
fn ready(c: LedgeContext) -> bool {
    c.situation == 0
        && !c.stopped
        && c.can_jump
        && (c.can_act || matches!(c.status, GUARD_ON | GUARD))
}
fn start(t: &mut LedgeTrap, c: LedgeContext, o: &mut LedgeOutput, m: Move, full: bool) {
    let action = OneShot::new(m, c.facing).with_hop(full);
    // Start back-air from neutral jump, then select backward+A in squat.
    // This preserves facing instead of turning a ground stick flick into fair.
    t.fighter_move = Some(if m.stick[0] < 0.0 {
        action.with_turn_before_jump()
    } else {
        action
    });
    t.continue_fighter_move(c, o);
}
fn timed(t: &mut LedgeTrap, c: LedgeContext, e: Opponent, o: &mut LedgeOutput, target_x: f32) {
    if !ready(c) {
        return;
    }
    o.stick = Some([0.0; 2]);
    let m = recipe(c, target_x);
    let Some(opens) = (if e.vulnerable {
        Some(0.0)
    } else {
        e.vulnerable_in
    })
    .filter(|v| v.is_finite() && (0.0..=120.0).contains(v)) else {
        return;
    };
    let first = f32::from(JUMP_SQUAT_FRAMES + m.first) + c.input_delay;
    if opens > first + f32::from(data::SLINGSHOT_EARLY_FRAMES)
        || e.actionable_in.is_some_and(|end| end <= first)
    {
        return;
    }
    start(t, c, o, m, false);
}
pub(super) fn neutral(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if !s.responses.neutral_getup {
        return;
    }
    if crate::ledge_trap::is_hanging(e.status) {
        t.watch_ledge(s, c, e, o);
        return;
    }
    if t.climb_seen && e.status == CLIFF_CLIMB {
        timed(t, c, e, o, e.position[0]);
    }
}
pub(super) fn jump(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if !s.responses.jump_getup
        || !ready(c)
        || e.situation == 0 && !(CLIFF_JUMP1..=CLIFF_JUMP3).contains(&e.status)
    {
        return;
    }
    let m = recipe(c, e.position[0]);
    let Some(full) = crate::ledge_trap::prediction::choose_hop(
        s.aerial_execution.jump,
        crate::fighter_profiles::MoveFrames {
            first: m.first,
            last: m.last,
        },
        s,
        c,
        e,
    ) else {
        return;
    };
    start(t, c, o, m, full);
}
pub(super) fn roll(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if !s.responses.roll_getup || e.status != CLIFF_ESCAPE {
        return;
    }
    // Aim at the common roll endpoint without chasing it. Use current target
    // position when a ledge anchor/direction has not been established.
    let target_x = t
        .anchor
        .or(e.cliff_position)
        .filter(|_| t.inward.abs() > 0.5)
        .map_or(e.position[0], |anchor| {
            anchor[0]
                + t.inward.signum() * s.roll_distance.unwrap_or(s.fighter.common_roll_distance)
        });
    timed(t, c, e, o, target_x);
}
pub(super) fn attack(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if !s.responses.attack_getup || c.situation != 0 {
        return;
    }
    o.stick = Some([0.0; 2]);
    o.requested = MAPPER_ZR;
    if matches!(t.plan, OptionPlan::Shield { blocked: true })
        && c.status != GUARD_DAMAGE
        && ready(c)
    {
        start(t, c, o, recipe(c, e.position[0]), false);
    }
}
pub(super) fn drop(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if s.drop_defense && ready(c) {
        start(t, c, o, recipe(c, e.position[0]), false);
    }
}
