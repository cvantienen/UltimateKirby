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
const DOWN_SMASH: Move = Move {
    kind: MoveKind::Ground {
        startup: 0x30,
        attack: 0x32,
    },
    first: data::LEDGE_DATA.down_smash.first,
    last: data::LEDGE_DATA.down_smash.last,
    stick: [0.0, -1.0],
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
// Static backing arrays satisfy SavedList's 'static lifetime. Neutral defaults
// to nair; its reduced catalog contains only the currently requested choices.
static NEUTRAL_OPTIONS: [SavedChoice; 5] = [
    SavedChoice::new("nair", "Nair", neutral_nair, prepare_neutral_nair),
    BASE.lists[0].options[1],
    SavedChoice::new(
        "forward_smash",
        "Headbutt f-smash",
        neutral_headbutt,
        enable_neutral,
    ),
    SavedChoice::new(
        "down_smash",
        "Down smash",
        neutral_down_smash,
        enable_neutral,
    ),
    SavedChoice::new("down_air", "Dair", neutral_dair, enable_neutral),
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
        SavedList {
            selected: 1,
            options: &ROLL_OPTIONS,
        },
        SavedList {
            selected: 1,
            options: &ATTACK_OPTIONS,
        },
        SavedList::new(&DROP_OPTIONS),
    ],
};
fn prepare_neutral_nair(s: LedgeSettings) -> LedgeSettings {
    let mut s = super::common::prepare_neutral_nair(s);
    s.aerial_execution.neutral = crate::fighter_profiles::execution::Hop::Short;
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
    if !s.responses.neutral_getup || !t.climb_seen || e.status != super::super::CLIFF_CLIMB {
        return;
    }
    launch_nair(t, c, e, o);
}

fn neutral_down_smash(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::neutral(t, s, c, e, o, DOWN_SMASH);
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
        || !c.can_control
        || e.situation == 0
            && !(super::super::CLIFF_JUMP1..=super::super::CLIFF_JUMP3).contains(&e.status)
    {
        return;
    }
    launch_nair(t, c, e, o);
}

// Both get-up options use exactly the same native short-hop nair sequence.
fn launch_nair(t: &mut LedgeTrap, c: LedgeContext, e: Opponent, o: &mut LedgeOutput) {
    if c.stopped || c.situation != 0 || !c.can_control {
        return;
    }
    // Keep the same ledge-facing stance as Neutral nair, even once the enemy's
    // airborne root has moved inward or across Yoshi.
    let dx = t.anchor.map_or(e.position[0], |anchor| anchor[0]) - c.position[0];
    let direction = if dx.abs() > 0.5 {
        dx.signum()
    } else if t.inward.is_finite() && t.inward.abs() >= 0.5 {
        -t.inward.signum()
    } else {
        c.facing.signum()
    };
    t.movement_preparing = true;
    o.owned = super::super::LEDGE_BUTTONS;
    o.stick = Some([0.0, 0.0]);
    if c.facing * direction < 0.5 {
        if c.can_act || matches!(c.status, 0x7 | 0x8) {
            o.stick = Some([direction * 0.6, 0.0]);
        }
        return; // Native facing confirmation must precede Jump+Attack.
    }
    if !c.can_jump
        || !c.can_act
            && !matches!(
                c.status,
                0x7 | 0x8 | super::super::GUARD_ON | super::super::GUARD
            )
    {
        return;
    }
    // Launch immediately once facing/readiness is confirmed; drift begins only
    // after native nair acceptance, so the aerial request remains neutral.
    t.fighter_move =
        Some(OneShot::new(JUMP_NAIR, direction).with_drift_until(data::NAIR_DRIFT_END_FRAME));
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
fn neutral_headbutt(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    variety::neutral(t, s, c, e, o, HEADBUTT);
}
/// Immediate short-hop dair: turn toward the remembered ledge, launch with X,
/// then release Jump and buffer down+A during native squat. No timing gate.
fn neutral_dair(
    t: &mut LedgeTrap,
    s: LedgeSettings,
    c: LedgeContext,
    e: Opponent,
    o: &mut LedgeOutput,
) {
    if !s.responses.neutral_getup
        || !t.climb_seen
        || e.status != super::super::CLIFF_CLIMB
        || c.stopped
        || c.situation != 0
        || !c.can_control
    {
        return;
    }
    let dx = t.anchor.map_or(e.position[0], |anchor| anchor[0]) - c.position[0];
    let direction = if dx.abs() > 0.5 {
        dx.signum()
    } else if t.inward.is_finite() && t.inward.abs() >= 0.5 {
        -t.inward.signum()
    } else {
        c.facing.signum()
    };
    let recipe = Move {
        stick: [0.0, -1.0],
        drift: data::NEUTRAL_NAIR_DRIFT,
        ..DROP_DAIR
    };
    t.fighter_move = Some(
        OneShot::new(recipe, direction)
            .with_turn_before_jump()
            .with_squat_buffer()
            .with_drift_until(data::DAIR_DRIFT_END_FRAME),
    );
    t.continue_fighter_move(c, o);
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
    t.fixed_roll_grab_inset = None; // Restore the shared tracked walking approach.
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

#[cfg(test)]
mod neutral_nair_tests {
    use super::*;
    use crate::ledge_trap::tests::{context, enemy};
    use crate::native_input::{MAPPER_A, MAPPER_X};
    use crate::timing::motion_hash;

    #[test]
    fn both_nairs_use_identical_neutral_jump_attack_even_with_unknown_protection() {
        for opens in [None, Some(30.0)] {
            let ctx = LedgeContext {
                motion_kind: motion_hash("wait"),
                ..context()
            };
            let mut neutral = LedgeTrap {
                climb_seen: true,
                ..LedgeTrap::default()
            };
            let mut jump = LedgeTrap::default();
            let mut settings = LedgeSettings::default();
            settings.responses.neutral_getup = true;
            settings.responses.jump_getup = true;
            let target = Opponent {
                vulnerable_in: opens,
                ..enemy(super::super::super::CLIFF_CLIMB, false)
            };
            let mut normal = LedgeOutput::default();
            neutral_nair(&mut neutral, settings, ctx, target, &mut normal);
            let mut jumping = LedgeOutput::default();
            jump_nair(
                &mut jump,
                settings,
                ctx,
                Opponent {
                    status: super::super::super::CLIFF_JUMP1,
                    ..target
                },
                &mut jumping,
            );
            assert_eq!(normal, jumping);
            assert_eq!(normal.requested, MAPPER_X | MAPPER_A);
            assert_eq!(normal.stick, Some([0.0; 2]));
            for c in [
                LedgeContext {
                    status: 0xa,
                    motion_kind: motion_hash("jump_squat"),
                    can_act: false,
                    ..ctx
                },
                LedgeContext {
                    status: 0xb,
                    situation: 2,
                    motion_kind: motion_hash("jump_f"),
                    can_act: false,
                    ..ctx
                },
                LedgeContext {
                    status: 0x36,
                    situation: 2,
                    motion_kind: motion_hash("attack_air_n"),
                    motion_frame: 1.0,
                    can_act: false,
                    ..ctx
                },
            ] {
                neutral.continue_fighter_move(c, &mut normal);
                jump.continue_fighter_move(c, &mut jumping);
                assert_eq!(normal, jumping);
                if c.status != 0x36 {
                    assert_eq!(normal.requested, MAPPER_A);
                }
            }
        }
    }
}
