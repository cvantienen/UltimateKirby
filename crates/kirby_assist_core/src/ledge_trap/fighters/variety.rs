//! Selected-only helpers for extra single-move choices. No target scans or I/O.
use super::super::{
    AttackWindow, LedgeContext, LedgeOutput, LedgeSettings, LedgeTrap, Opponent, OptionPlan,
    CLIFF_JUMP1, CLIFF_JUMP3, GUARD, GUARD_DAMAGE, GUARD_ON,
};
use super::one_shot::{Move, MoveKind, OneShot};
use crate::native_input::MAPPER_ZR;

pub(super) fn neutral(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
    recipe: Move,
) {
    if !s.responses.neutral_getup || !t.climb_seen || super::super::is_hanging(e.status) {
        return;
    }
    timed(t, s, c, e, o, recipe, false);
}
pub(super) fn jump(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
    recipe: Move,
) {
    if !s.responses.jump_getup
        || e.situation == 0 && !(CLIFF_JUMP1..=CLIFF_JUMP3).contains(&e.status)
    {
        return;
    }
    timed(t, s, c, e, o, recipe, true);
}
fn timed(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
    recipe: Move,
    jumping: bool,
) {
    #[cfg(feature = "diagnostic-logging")]
    {
        t.diagnostic_move = Some(recipe.calibration());
    }
    if c.situation != 0 || !c.can_act || c.stopped {
        return;
    }
    let aerial = recipe.aerial();
    if aerial && !c.can_jump {
        return;
    }
    if matches!(recipe.kind, MoveKind::Ground { attack: 0x32, .. }) && !c.can_down_smash {
        return;
    }
    let full = if aerial {
        let policy = if jumping {
            s.aerial_execution.jump
        } else {
            s.aerial_execution.neutral
        };
        let Some(full) = super::super::prediction::choose_hop(
            policy,
            crate::fighter_profiles::MoveFrames {
                first: recipe.first,
                last: recipe.last,
            },
            s,
            c,
            e,
        ) else {
            return;
        };
        full
    } else {
        false
    };
    let setup = if full {
        4 + c.input_delay.ceil().clamp(0.0, 12.0) as u8
    } else if aerial {
        3
    } else {
        1
    }; // Squat or neutral smash-flick primer.
    let first = recipe.first + setup;
    let last = recipe.last + setup;
    let window = AttackWindow {
        first,
        last,
        range: s.anti_air_range,
        height: [-6.0, s.anti_air_height],
        front_only: recipe.stick[0] != 0.0,
    };
    // Cap early staging even for long aerial windows; don't assume an arbitrary
    // early hop is still at the opponent's height when protection expires.
    let opens = if e.vulnerable {
        Some(0.0)
    } else {
        e.vulnerable_in
    };
    let early = opens.is_some_and(|v| {
        v.is_finite() && v >= 0.0 && v <= f32::from(first + (last - first).min(8)) + c.input_delay
    });
    let launching = jumping && matches!(e.status, CLIFF_JUMP1 | 0x7c);
    if !(early || launching)
        || window
            .scheduled_contact(c, e, e.actionable_in, s.distance_agnostic)
            .is_none()
    {
        return;
    }
    if !face(t, c, e, recipe, o) {
        return;
    }
    t.start_fighter_move(c, o, recipe, full);
}
pub(super) fn blocked(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
    recipe: Move,
) {
    #[cfg(feature = "diagnostic-logging")]
    {
        t.diagnostic_move = Some(recipe.calibration());
    }
    if !s.responses.attack_getup || c.situation != 0 {
        return;
    }
    o.stick = Some([0.0; 2]);
    o.requested = MAPPER_ZR;
    if matches!(t.plan, OptionPlan::Shield { blocked: true })
        && c.status != GUARD_DAMAGE
        && !c.stopped
        && (c.can_act || matches!(c.status, GUARD_ON | GUARD))
        && c.can_jump
        && (recipe.stick[0] == 0.0 || (e.position[0] - c.position[0]) * c.facing >= -0.5)
    {
        let Some(full) = super::super::prediction::choose_hop(
            s.aerial_execution.shield,
            crate::fighter_profiles::MoveFrames {
                first: recipe.first,
                last: recipe.last,
            },
            s,
            c,
            e,
        ) else {
            return;
        };
        t.start_fighter_move(c, o, recipe, full);
    }
}
pub(super) fn drop(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
    recipe: Move,
) {
    #[cfg(feature = "diagnostic-logging")]
    {
        t.diagnostic_move = Some(recipe.calibration());
    }
    if !s.drop_defense
        || c.situation != 0
        || !c.can_act
        || c.stopped
        || recipe.aerial() && !c.can_jump
    {
        return;
    }
    if matches!(recipe.kind, MoveKind::Ground { attack: 0x32, .. }) && !c.can_down_smash {
        return;
    }
    if !face(t, c, e, recipe, o) {
        return;
    }
    t.start_fighter_move(c, o, recipe, false);
}
fn face(
    t: &mut LedgeTrap,
    c: LedgeContext,
    e: Opponent,
    recipe: Move,
    o: &mut LedgeOutput,
) -> bool {
    if recipe.stick[0] == 0.0 || (e.position[0] - c.position[0]) * c.facing >= -0.5 {
        return true;
    }
    let dx = e.position[0] - c.position[0];
    if c.can_act
        && c.safe_ground[usize::from(dx > 0.0)]
        && t.movement_attempts < super::super::MAX_ATTEMPTS
    {
        t.movement_attempts += 1;
        t.movement_preparing = true;
        o.stick = Some([dx.signum() * 0.3, 0.0]);
    }
    false
}
impl LedgeTrap {
    pub(super) fn start_fighter_move(
        &mut self,
        c: LedgeContext,
        o: &mut LedgeOutput,
        recipe: Move,
        full: bool,
    ) {
        self.fighter_move = Some(OneShot::new(recipe, c.facing).with_hop(full));
        self.continue_fighter_move(c, o);
    }
    pub(in crate::ledge_trap) fn continue_fighter_move(
        &mut self,
        c: LedgeContext,
        o: &mut LedgeOutput,
    ) {
        let Some(action) = self.fighter_move.as_mut() else {
            return;
        };
        #[cfg(feature = "diagnostic-logging")]
        {
            self.diagnostic_move = Some(action.calibration());
        }
        *o = action.sample(c);
        self.movement_preparing = !action.finished();
        if action.finished() {
            self.fighter_move = None;
            self.aerial_handed_back = true; // One move only, including a whiff.
        }
    }
}
