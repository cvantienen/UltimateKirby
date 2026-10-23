//! Samus-only, compiled recipes. Real native action observations confirm each
//! ordinary input; no runtime scripts, hitbox edits or offstage chasing.
use super::super::{
    AttackWindow, LedgeContext, LedgeDropAction, LedgeOutput, LedgeSettings, LedgeTrap, Opponent,
    OptionPlan, Pending, CLIFF_CLIMB, CLIFF_JUMP1, CLIFF_JUMP3, GUARD, GUARD_DAMAGE, GUARD_ON,
};
use super::common::*;
use crate::fighter_profiles::{
    presets::{MenuPresets, SavedChoice, SavedList},
    samus::{ZAIR_ACTIVE, ZAIR_HIT_LEAD, ZAIR_TAKEOFF_FRAMES},
};
use crate::native_input::{MAPPER_B, MAPPER_ZR};
mod ranged;
mod zair;
pub(crate) use zair::Zair;

const BASE: MenuPresets = super::standard::menu(1, 2, true);
// Permanent backing arrays preserve existing choice indices and defaults.
static NEUTRAL_OPTIONS: [SavedChoice; 7] = [
    BASE.lists[0].options[0],
    BASE.lists[0].options[1],
    BASE.lists[0].options[2],
    SavedChoice::new(
        "zair",
        "Short-hop zair",
        respond_to_neutral_getup_zair,
        enable_neutral_zair,
    ),
    SavedChoice::new(
        "homing_missile",
        "Homing missile",
        ranged::neutral_homing,
        enable_neutral_zair,
    ),
    SavedChoice::new(
        "super_missile",
        "Super missile",
        ranged::neutral_super,
        enable_neutral_zair,
    ),
    SavedChoice::new(
        "forward_smash",
        "Cannon f-smash",
        ranged::neutral_smash,
        enable_neutral_zair,
    ),
];
static JUMP_OPTIONS: [SavedChoice; 6] = [
    BASE.lists[1].options[0],
    SavedChoice::new(
        "zair",
        "Short-hop zair",
        respond_to_jump_getup_zair,
        enable_jump,
    ),
    SavedChoice::new("forward_air", "Flame fair", ranged::jump_fair, enable_jump),
    SavedChoice::new("up_air", "Up air", ranged::jump_up_air, enable_jump),
    SavedChoice::new(
        "homing_missile",
        "Homing missile",
        ranged::jump_homing,
        enable_jump,
    ),
    SavedChoice::new(
        "super_missile",
        "Super missile",
        ranged::jump_super,
        enable_jump,
    ),
];
static ROLL_OPTIONS: [SavedChoice; 5] = [
    BASE.lists[2].options[0],
    SavedChoice::new("grab", "Grab", respond_to_roll_grab, enable_roll),
    SavedChoice::new("zair", "Move > zair", respond_to_roll_zair, enable_roll),
    SavedChoice::new(
        "homing_missile",
        "Homing (stay back)",
        ranged::roll_homing,
        enable_roll,
    ),
    SavedChoice::new(
        "super_missile",
        "Super (stay back)",
        ranged::roll_super,
        enable_roll,
    ),
];
static ATTACK_OPTIONS: [SavedChoice; 6] = [
    BASE.lists[3].options[0],
    BASE.lists[3].options[1],
    BASE.lists[3].options[2],
    SavedChoice::new(
        "shield_zair",
        "Shield > zair",
        respond_to_getup_attack_zair,
        enable_attack,
    ),
    SavedChoice::new(
        "shield_fair",
        "Shield > fair",
        ranged::shield_fair,
        enable_attack,
    ),
    SavedChoice::new(
        "shield_up_air",
        "Shield > up air",
        ranged::shield_up_air,
        enable_attack,
    ),
];
static DROP_OPTIONS: [SavedChoice; 7] = [
    BASE.lists[4].options[0],
    BASE.lists[4].options[1],
    SavedChoice::new(
        "bomb",
        "Immediate down-B",
        respond_to_ledge_drop_bomb,
        prepare_drop_bomb,
    ),
    SavedChoice::new(
        "zair",
        "Short-hop zair",
        respond_to_ledge_drop_zair,
        prepare_drop_shield,
    ),
    SavedChoice::new(
        "homing_missile",
        "Homing missile",
        ranged::drop_homing,
        prepare_drop_shield,
    ),
    SavedChoice::new(
        "super_missile",
        "Super missile",
        ranged::drop_super,
        prepare_drop_shield,
    ),
    SavedChoice::new(
        "forward_air",
        "Flame fair (stay back)",
        ranged::drop_fair,
        prepare_drop_shield,
    ),
];
pub(crate) const MENU: MenuPresets = MenuPresets {
    lists: [
        SavedList {
            selected: 1,
            options: &NEUTRAL_OPTIONS,
        },
        SavedList::new(&JUMP_OPTIONS),
        SavedList {
            selected: 1,
            options: &ROLL_OPTIONS,
        },
        SavedList {
            selected: 2,
            options: &ATTACK_OPTIONS,
        },
        SavedList {
            selected: 2,
            options: &DROP_OPTIONS,
        },
    ],
};

fn enable_neutral_zair(mut s: LedgeSettings) -> LedgeSettings {
    s.responses.neutral_getup = true;
    s
}
fn prepare_drop_bomb(mut s: LedgeSettings) -> LedgeSettings {
    s.drop_defense = true;
    s.drop_action = LedgeDropAction::Bomb;
    s
}
fn respond_to_roll_grab(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    t.respond_to_roll_grab(s, c, e, o);
}
fn respond_to_roll_zair(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    t.respond_to_roll_zair(s, c, e, o);
}
fn respond_to_neutral_getup_zair(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if super::super::is_hanging(e.status) {
        t.watch_ledge(s, c, e, o);
    } else if s.responses.neutral_getup && t.climb_seen && e.status == CLIFF_CLIMB {
        timed_zair(t, c, e, o);
    }
}
fn respond_to_jump_getup_zair(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if !s.responses.jump_getup
        || e.situation == 0 && !(CLIFF_JUMP1..=CLIFF_JUMP3).contains(&e.status)
    {
        return;
    }
    timed_zair(t, c, e, o);
}
fn timed_zair(t: &mut LedgeTrap, c: LedgeContext, e: Opponent, o: &mut LedgeOutput) {
    let first = ZAIR_TAKEOFF_FRAMES + ZAIR_ACTIVE.first;
    let window = AttackWindow {
        first,
        last: ZAIR_TAKEOFF_FRAMES + ZAIR_ACTIVE.last,
        range: 0.0,
        height: [0.0; 2],
        front_only: true,
    };
    let opens = if e.vulnerable {
        Some(0.0)
    } else {
        e.vulnerable_in
    };
    if !opens.is_some_and(|v| {
        v.is_finite() && v >= 0.0 && v <= c.input_delay + f32::from(first + ZAIR_HIT_LEAD)
    }) || window.timing_contact(c, e, e.actionable_in).is_none()
    {
        return;
    }
    if face_target(t, c, e.position[0] - c.position[0], o) {
        t.start_samus_zair(c, o);
    }
}
fn face_target(t: &mut LedgeTrap, c: LedgeContext, dx: f32, o: &mut LedgeOutput) -> bool {
    if dx * c.facing >= -0.5 {
        return true;
    }
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
fn respond_to_getup_attack_zair(
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
        && !c.stopped
        && (c.can_act || matches!(c.status, GUARD_ON | GUARD))
    {
        // Turning would drop shield into the attack; wrong-facing stays shielded.
        if (e.position[0] - c.position[0]) * c.facing >= -0.5 {
            t.start_samus_zair(c, o);
        }
    }
}
fn respond_to_ledge_drop_zair(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if s.drop_defense && face_target(t, c, e.position[0] - c.position[0], o) {
        t.start_samus_zair(c, o);
    }
}
fn respond_to_ledge_drop_bomb(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    _: Opponent,
    o: &mut LedgeOutput,
) {
    if !s.drop_defense || c.situation != 0 || !c.can_control {
        return;
    }
    let fresh = t
        .variant_attempt
        .advance(c, false, s.drop_request_timeout_frames);
    if t.variant_attempt.expired {
        return;
    }
    t.movement_preparing = true;
    if !fresh {
        *o = t.last_output;
        return;
    }
    o.stick = Some([0.0; 2]);
    // No range, facing, vulnerability or bomb-explosion prediction gate.
    if t.variant_attempt.holding_special(c)
        || c.can_act
            && c.can_down_special
            && t.variant_attempt
                .issue(c, t.last_output.requested, MAPPER_B)
    {
        t.pending = Pending::Bomb;
        o.stick = Some([0.0, -1.0]);
        o.requested = MAPPER_B;
    }
}

impl LedgeTrap {
    pub(in crate::ledge_trap) fn start_samus_zair(
        &mut self,
        ctx: LedgeContext,
        out: &mut LedgeOutput,
    ) {
        if ctx.situation != 0
            || !ctx.can_jump
            || !(ctx.can_act || matches!(ctx.status, GUARD_ON | GUARD))
        {
            return;
        }
        self.samus_zair = Some(Zair::new());
        self.continue_samus_zair(ctx, out);
    }
    pub(in crate::ledge_trap) fn continue_samus_zair(
        &mut self,
        ctx: LedgeContext,
        out: &mut LedgeOutput,
    ) {
        let Some(zair) = self.samus_zair.as_mut() else {
            return;
        };
        *out = zair.sample(ctx);
        if zair.finished() {
            self.samus_zair = None;
            self.aerial_handed_back = true;
        }
    }
}
