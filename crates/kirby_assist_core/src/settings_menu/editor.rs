//! Lazy, in-memory recipe edits. No parsing, I/O, or allocation in execution.
use crate::native_input::*;
use crate::sequence_config::{
    Action, Condition, Direction as D, InputReference as IR, RangeCheck, Sequence, MAX_ACTIONS,
};
use std::collections::BTreeMap;
#[path = "editor/commands.rs"]
mod commands;
#[path = "editor/presentation.rs"]
mod presentation;
use presentation::{field_slot, field_value, fields, recipe_panel, Field};
#[cfg(test)]
use presentation::{same_field, schedule_summary, scheduled_frames};

const fn key(buttons: u32, x: f32, y: f32, frames: u8) -> Action {
    Action::Input {
        buttons,
        stick: [x, y],
        frames,
        relative: true,
    }
}
const PRESETS: &[(&str, Action)] = &[
    ("Empty", Action::Empty),
    ("Short hop", key(MAPPER_X, 0., 0., 1)),
    ("Full hop hold", key(MAPPER_X, 0., 0., 4)),
    ("Neutral A / Nair", key(MAPPER_A, 0., 0., 4)),
    ("Short hop Nair", key(MAPPER_X | MAPPER_A, 0., 0., 4)),
    ("Forward A (Fair / Fsmash)", key(MAPPER_A, 1., 0., 4)),
    ("Bair", key(MAPPER_A, -1., 0., 4)),
    ("Up A (Uair / Usmash)", key(MAPPER_A, 0., 1., 4)),
    ("Down A (Dair / Dsmash)", key(MAPPER_A, 0., -1., 4)),
    ("Short hop Fair", key(MAPPER_X | MAPPER_A, 1., 0., 4)),
    ("Short hop Bair", key(MAPPER_X | MAPPER_A, -1., 0., 4)),
    ("Short hop Uair", key(MAPPER_X | MAPPER_A, 0., 1., 4)),
    ("Short hop Dair", key(MAPPER_X | MAPPER_A, 0., -1., 4)),
    ("Ftilt", key(MAPPER_A, 0.6, 0., 4)),
    ("Utilt", key(MAPPER_A, 0., 0.6, 4)),
    ("Dtilt", key(MAPPER_A, 0., -0.6, 4)),
    ("Neutral B", key(MAPPER_B, 0., 0., 4)),
    ("Side B", key(MAPPER_B, 1., 0., 4)),
    ("Up B", key(MAPPER_B, 0., 1., 4)),
    ("Down B", key(MAPPER_B, 0., -1., 4)),
    ("Shield / air dodge", key(MAPPER_ZR, 0., 0., 4)),
    ("Grab / Zair", key(MAPPER_R, 0., 0., 4)),
    ("Dash", key(0, 1., 0., 4)),
    ("Crouch / fast fall", key(0, 0., -1., 4)),
    ("Roll forward", key(MAPPER_ZR, 1., 0., 4)),
    ("Roll backward", key(MAPPER_ZR, -1., 0., 4)),
    ("Spot dodge", key(MAPPER_ZR, 0., -1., 4)),
    ("Face direction", Action::Face(D::Opponent)),
    (
        "Walk to opponent",
        Action::Approach {
            airborne: false,
            strength: 0.8,
            frames: 30,
            gap: 10.,
        },
    ),
    (
        "Air approach",
        Action::Approach {
            airborne: true,
            strength: 0.5,
            frames: 12,
            gap: 3.,
        },
    ),
    (
        "Walk distance",
        Action::Walk {
            direction: D::Opponent,
            units: 8.,
            strength: 0.5,
            min_gap: None,
        },
    ),
    (
        "Run distance",
        Action::Run {
            direction: D::Opponent,
            units: 8.,
        },
    ),
    (
        "Drift",
        Action::Drift {
            direction: D::Opponent,
            strength: 0.25,
            frames: 20,
        },
    ),
    ("Wait for shield hit", Action::ShieldUntilBlock(90)),
    (
        "Check opponent distance",
        Action::Range(RangeCheck::Verify(10.)),
    ),
    (
        "Check fighter grab reach",
        Action::Range(RangeCheck::FighterGrabRange),
    ),
    (
        "Check fighter grab reach instantly",
        Action::Range(RangeCheck::FighterGrabRangeInstant),
    ),
    (
        "C-stick",
        Action::CStick {
            stick: [1., 0.],
            frames: 4,
        },
    ),
    (
        "Check: I am grounded",
        Action::Check {
            condition: Condition::SelfGrounded,
            value: 0.,
            wait: 0,
        },
    ),
    (
        "Check: I am airborne",
        Action::Check {
            condition: Condition::SelfAirborne,
            value: 0.,
            wait: 0,
        },
    ),
    (
        "Check: enemy grounded",
        Action::Check {
            condition: Condition::EnemyGrounded,
            value: 0.,
            wait: 0,
        },
    ),
    (
        "Check: enemy airborne",
        Action::Check {
            condition: Condition::EnemyAirborne,
            value: 0.,
            wait: 0,
        },
    ),
    (
        "Check: enemy shielding",
        Action::Check {
            condition: Condition::EnemyShielding,
            value: 0.,
            wait: 0,
        },
    ),
    (
        "Check: enemy not shielding",
        Action::Check {
            condition: Condition::EnemyNotShielding,
            value: 0.,
            wait: 0,
        },
    ),
    (
        "Check: enemy in front",
        Action::Check {
            condition: Condition::EnemyInFront,
            value: 0.,
            wait: 0,
        },
    ),
    (
        "Check: enemy behind",
        Action::Check {
            condition: Condition::EnemyBehind,
            value: 0.,
            wait: 0,
        },
    ),
    (
        "Check: enemy X distance within",
        Action::Check {
            condition: Condition::EnemyWithin,
            value: 10.,
            wait: 0,
        },
    ),
    (
        "Check: enemy X distance outside",
        Action::Check {
            condition: Condition::EnemyOutside,
            value: 10.,
            wait: 0,
        },
    ),
    (
        "Repeated taps",
        Action::Tap {
            buttons: MAPPER_A,
            stick: [0., 0.],
            hold: 1,
            gap: 11,
            frames: 120,
        },
    ),
    (
        "Controller input",
        Action::Controller {
            buttons: 0,
            stick: [0.; 2],
            cstick: [0.; 2],
            frames: 4,
            reference: IR::Screen,
            cstick_reference: IR::Screen,
        },
    ),
    ("If facing ledge", Action::IfFacingLedge { away: false }),
    ("If back to ledge", Action::IfFacingLedge { away: true }),
    (
        "If facing opponent",
        Action::IfFacingOpponent { away: false },
    ),
    (
        "If back to opponent",
        Action::IfFacingOpponent { away: true },
    ),
    ("Otherwise (facing block)", Action::ElseFacing),
    (
        "Run to ledge offset",
        Action::LedgePosition {
            offset: 12.,
            strength: 1.,
            tolerance: 1.,
            brake: 6.,
            frames: 180,
        },
    ),
    ("End facing block", Action::EndFacing),
    (
        "Chase only if outside range",
        Action::Chase {
            gap: 30.,
            strength: 1.,
            frames: 14,
        },
    ),
];
const TAP_BUTTONS: &[(u32, [f32; 2], &str)] = &[
    (MAPPER_A, [0., 0.], "A"),
    (MAPPER_B, [0., 0.], "B"),
    (MAPPER_A, [0.6, 0.], "Forward tilt"),
    (MAPPER_A, [0., 0.6], "Up tilt"),
    (MAPPER_A, [0., -0.6], "Down tilt"),
];
const DIRECTIONS: [D; 8] = [
    D::Opponent,
    D::AwayFromOpponent,
    D::Inward,
    D::Outward,
    D::Facing,
    D::Backward,
    D::SavedLedge,
    D::AwayFromLedge,
];

fn preset_kind(action: Action) -> Option<usize> {
    // Keep an edited action in its preset so users can keep editing its fields.
    PRESETS
        .iter()
        .position(|(_, preset)| *preset == action)
        .or_else(|| {
            PRESETS
                .iter()
                .position(|(_, preset)| same_preset_shape(action, *preset))
        })
}

fn same_preset_shape(action: Action, preset: Action) -> bool {
    match (action, preset) {
        (
            Action::Input {
                buttons,
                stick,
                relative,
                ..
            },
            Action::Input {
                buttons: preset_buttons,
                stick: preset_stick,
                relative: preset_relative,
                ..
            },
        ) => buttons == preset_buttons && stick == preset_stick && relative == preset_relative,
        (
            Action::Approach {
                airborne: action_airborne,
                ..
            },
            Action::Approach {
                airborne: preset_airborne,
                ..
            },
        ) => action_airborne == preset_airborne,
        (
            Action::Check {
                condition: action_condition,
                ..
            },
            Action::Check {
                condition: preset_condition,
                ..
            },
        ) => action_condition == preset_condition,
        (
            Action::IfFacingLedge { away: action_away },
            Action::IfFacingLedge { away: preset_away },
        )
        | (
            Action::IfFacingOpponent { away: action_away },
            Action::IfFacingOpponent { away: preset_away },
        ) => action_away == preset_away,
        (Action::LedgePosition { .. }, Action::LedgePosition { .. }) => true,
        (Action::Range(action), Action::Range(preset)) => {
            std::mem::discriminant(&action) == std::mem::discriminant(&preset)
        }
        (Action::CStick { stick: action, .. }, Action::CStick { stick: preset, .. }) => {
            action == preset
        }
        (
            Action::Input { .. }
            | Action::Controller { .. }
            | Action::CStick { .. }
            | Action::Approach { .. },
            _,
        ) => false,
        _ => std::mem::discriminant(&action) == std::mem::discriminant(&preset),
    }
}

#[derive(Clone)]
struct Recipe {
    button_recipe: bool,
    sequence: Sequence,
    before: [u8; MAX_ACTIONS],
    kinds: [usize; MAX_ACTIONS],
    imported: Vec<Action>,
    edited: u32,
    modified: bool,
}
impl Recipe {
    fn new(mut sequence: Sequence) -> Self {
        // Present file Wait steps as delays, preserving their exact frame total.
        let original = sequence;
        sequence.actions.fill(Action::Empty);
        let mut before = [0; MAX_ACTIONS];
        let mut len = 0;
        let mut pending = 0u16;
        for action in original.actions() {
            if let Action::Wait(frames) = action {
                pending += u16::from(*frames);
                continue;
            }
            while pending > 255 {
                before[len] = 255;
                len += 1;
                pending -= 255;
            }
            before[len] = pending as u8;
            sequence.actions[len] = *action;
            len += 1;
            pending = 0;
        }
        while pending > 0 {
            before[len] = pending.min(255) as u8;
            len += 1;
            pending = pending.saturating_sub(255);
        }
        sequence.len = (len as u8).max(6);
        let mut kinds = [0; MAX_ACTIONS];
        let mut imported = Vec::new();
        for (i, a) in sequence.actions().iter().enumerate() {
            kinds[i] = preset_kind(*a).unwrap_or_else(|| {
                let index = imported
                    .iter()
                    .position(|saved| saved == a)
                    .unwrap_or_else(|| {
                        imported.push(*a);
                        imported.len() - 1
                    });
                PRESETS.len() + index
            });
        }
        Self {
            button_recipe: false,
            sequence,
            before,
            kinds,
            imported,
            edited: 0,
            modified: false,
        }
    }
    fn compile(&self) -> Option<Sequence> {
        let mut result = Sequence {
            len: 0,
            ..self.sequence
        };
        for (i, action) in self.sequence.actions().iter().enumerate() {
            for a in [
                if self.before[i] > 0 {
                    Some(Action::Wait(self.before[i]))
                } else {
                    None
                },
                if *action == Action::Empty {
                    None
                } else {
                    Some(*action)
                },
            ]
            .into_iter()
            .flatten()
            {
                if result.len as usize == MAX_ACTIONS {
                    return None;
                }
                result.actions[result.len as usize] = a;
                result.len += 1;
            }
        }
        Some(result)
    }
}
fn action_name(r: &Recipe, i: usize) -> String {
    let action = r.sequence.actions[i];
    if let Some((name, preset)) = PRESETS.get(r.kinds[i]) {
        let same = match (action, *preset) {
            (
                Action::Input {
                    buttons,
                    stick,
                    relative,
                    ..
                },
                Action::Input {
                    buttons: b,
                    stick: s,
                    relative: rel,
                    ..
                },
            ) => buttons == b && stick == s && relative == rel,
            (Action::Controller { .. }, _) => false,
            _ => true,
        };
        if same {
            return (*name).into();
        }
    }
    describe_action(action)
}
const INPUT_BUTTONS: &[(u32, &str)] = &[
    (MAPPER_A, "A"),
    (MAPPER_B, "B"),
    (MAPPER_X, "X"),
    (MAPPER_Y, "Y"),
    (MAPPER_L, "L"),
    (MAPPER_R, "R"),
    (MAPPER_ZL, "ZL"),
    (MAPPER_ZR, "ZR"),
];
fn input_buttons(a: Action) -> Option<u32> {
    match a {
        Action::Input { buttons, .. }
        | Action::OpponentInput { buttons, .. }
        | Action::Controller { buttons, .. } => Some(buttons),
        _ => None,
    }
}
fn buttons_label(mask: u32) -> String {
    let names = INPUT_BUTTONS
        .iter()
        .filter(|(b, _)| mask & b != 0)
        .map(|(_, n)| *n)
        .collect::<Vec<_>>();
    if names.is_empty() {
        "None (stick only)".into()
    } else {
        names.join("+")
    }
}
fn input_reference(a: Action, right: bool) -> IR {
    match a {
        Action::Controller {
            reference,
            cstick_reference,
            ..
        } => {
            if right {
                cstick_reference
            } else {
                reference
            }
        }
        Action::OpponentInput { .. } => IR::Opponent,
        Action::Input { relative: true, .. } => IR::Facing,
        _ => IR::Screen,
    }
}
fn set_input_reference(a: &mut Action, r: IR, right: bool) {
    match a {
        Action::Controller {
            reference,
            cstick_reference,
            ..
        } => {
            if right {
                *cstick_reference = r
            } else {
                *reference = r
            }
        }
        Action::Input {
            buttons,
            stick,
            frames,
            ..
        }
        | Action::OpponentInput {
            buttons,
            stick,
            frames,
        } => {
            *a = if r == IR::Opponent {
                Action::OpponentInput {
                    buttons: *buttons,
                    stick: *stick,
                    frames: if *frames == 0 { 4 } else { *frames },
                }
            } else {
                Action::Input {
                    buttons: *buttons,
                    stick: *stick,
                    frames: *frames,
                    relative: r == IR::Facing,
                }
            };
        }
        _ => {}
    }
}
fn describe_action(action: Action) -> String {
    match action {
        Action::Controller{buttons,stick,cstick,reference,cstick_reference,frames}=>format!("{} | Left {:.0}%/{:.0}% {} | Right {:.0}%/{:.0}% {} | {frames}f",buttons_label(buttons),stick[0]*100.,stick[1]*100.,reference.token(),cstick[0]*100.,cstick[1]*100.,cstick_reference.token()),
        Action::LedgePosition{offset,strength,tolerance,brake,frames}=>format!("Run to {offset} units inward from ledge | {:.0}% | tolerance {tolerance} | brake {brake} | max {frames}f",strength*100.),
        Action::IfFacingOpponent{away}=>if away{"If back to opponent (instant)".into()}else{"If facing opponent (instant)".into()},
        Action::ElseFacing=>"Otherwise (instant facing branch)".into(),
        Action::ShieldUntilManual=>"Shield until manual takeover (saved input)".into(),
        Action::Range(RangeCheck::Ignore)=>"Skip range check (saved input)".into(),
        Action::OpponentInput { buttons, stick, frames } => describe_action(Action::Input { buttons, stick, frames, relative: true }).replace("(facing)", "(opponent)"),
        Action::Input {
            buttons,
            stick,
            relative, frames,
            ..
        } => {
            let mut names = Vec::new();
            let mut known = 0;
            for (mask, name) in [
                (MAPPER_A, "A"),
                (MAPPER_B, "B"),
                (MAPPER_X, "X"),
                (MAPPER_Y, "Y"),
                (MAPPER_L, "L"),
                (MAPPER_R, "R"),
                (MAPPER_ZL, "ZL"),
                (MAPPER_ZR, "ZR"),
            ] {
                known |= mask;
                if buttons & mask != 0 {
                    names.push(name.to_owned());
                }
            }
            if buttons & !known != 0 {
                names.push(format!("0x{:X}", buttons & !known));
            }
            let buttons = if names.is_empty() {
                "Stick only".into()
            } else {
                names.join("+")
            };
            format!(
                "{} | X:{:.0}% Y:{:.0}% ({}) | {}",
                buttons,
                stick[0] * 100.,
                stick[1] * 100.,
                if relative { "facing" } else { "screen" },
                if frames==0 {"until manual".into()}else{format!("{frames}f")}
            )
        }
        other => format!("{other:?}"),
    }
}
fn step(v: f32, delta: f32, min: f32, max: f32) -> f32 {
    ((v + delta).clamp(min, max) * 100.).round() / 100.
}
fn byte(v: &mut u8, up: bool, min: u8) {
    *v = if up {
        v.saturating_add(1)
    } else {
        v.saturating_sub(1).max(min)
    };
}
fn direction(v: &mut D, up: bool, onstage: bool) {
    let mut i = DIRECTIONS.iter().position(|d| d == v).unwrap_or(0);
    for _ in 0..DIRECTIONS.len() {
        i = if up {
            (i + 1) % DIRECTIONS.len()
        } else {
            (i + DIRECTIONS.len() - 1) % DIRECTIONS.len()
        };
        if !onstage || !Action::Face(DIRECTIONS[i]).needs_ledge() {
            *v = DIRECTIONS[i];
            break;
        }
    }
}
fn action_slot(field: Field) -> usize {
    match field {
        Field::TapButtons(i)
        | Field::TapGap(i)
        | Field::Duration(i)
        | Field::X(i)
        | Field::Y(i)
        | Field::Strength(i)
        | Field::Tolerance(i)
        | Field::Brake(i)
        | Field::PositionTimeout(i)
        | Field::Distance(i)
        | Field::Gap(i)
        | Field::Direction(i) => i,
        _ => unreachable!(),
    }
}
fn adjust_action(action: &mut Action, field: Field, up: bool, delta: f32, onstage: bool) {
    match (action, field) {
        (Action::LedgePosition { offset, .. }, Field::Distance(_)) => {
            *offset = step(*offset, delta, 0., 200.)
        }
        (Action::LedgePosition { strength, .. }, Field::Strength(_)) => {
            *strength = step(*strength, delta * 0.01, 0.3, 1.)
        }
        (Action::LedgePosition { tolerance, .. }, Field::Tolerance(_)) => {
            *tolerance = step(*tolerance, delta * 0.1, 0., 20.)
        }
        (Action::LedgePosition { brake, .. }, Field::Brake(_)) => {
            *brake = step(*brake, delta * 0.5, 0., 100.)
        }
        (Action::LedgePosition { frames, .. }, Field::PositionTimeout(_)) => {
            *frames = if up {
                frames.saturating_add(1).min(3600)
            } else {
                frames.saturating_sub(1).max(1)
            }
        }
        (Action::Chase { strength, .. }, Field::Strength(_)) => {
            *strength = step(*strength, delta * 0.01, 0., 1.)
        }
        (
            Action::Tap {
                buttons,
                stick,
                hold,
                ..
            },
            Field::TapButtons(_),
        ) => {
            let index = TAP_BUTTONS
                .iter()
                .position(|(button, axes, _)| button == buttons && axes == stick);
            let index = match index {
                Some(i) => {
                    if up {
                        (i + 1) % TAP_BUTTONS.len()
                    } else {
                        (i + TAP_BUTTONS.len() - 1) % TAP_BUTTONS.len()
                    }
                }
                None => {
                    if up {
                        0
                    } else {
                        TAP_BUTTONS.len() - 1
                    }
                }
            };
            let (button, axes, _) = TAP_BUTTONS[index];
            *buttons = button;
            *stick = axes;
            *hold = 1;
        }
        (Action::Tap { gap, .. }, Field::TapGap(_)) => byte(gap, up, 1),
        (Action::Tap { frames, .. }, Field::Duration(_)) => {
            *frames = if up {
                frames.saturating_add(1).min(3600)
            } else {
                frames.saturating_sub(1).max(1)
            }
        }
        (
            Action::Input { frames, .. }
            | Action::OpponentInput { frames, .. }
            | Action::Controller { frames, .. }
            | Action::CStick { frames, .. }
            | Action::Chase { frames, .. }
            | Action::Approach { frames, .. }
            | Action::Drift { frames, .. },
            Field::Duration(_),
        ) => byte(frames, up, 1),
        (Action::Wait(frames), Field::Duration(_)) => byte(frames, up, 0),
        (Action::ShieldUntilBlock(frames), Field::Duration(_)) => byte(frames, up, 1),
        (Action::Check { wait, .. }, Field::Duration(_)) => byte(wait, up, 0),
        (Action::Check { value, .. }, Field::Distance(_)) => *value = step(*value, delta, 0., 100.),
        (
            Action::Input { stick, .. }
            | Action::OpponentInput { stick, .. }
            | Action::Controller { stick, .. }
            | Action::Tap { stick, .. }
            | Action::CStick { stick, .. },
            Field::X(_),
        ) => stick[0] = step(stick[0], delta * 0.01, -1., 1.),
        (
            Action::Input { stick, .. }
            | Action::OpponentInput { stick, .. }
            | Action::Controller { stick, .. }
            | Action::Tap { stick, .. }
            | Action::CStick { stick, .. },
            Field::Y(_),
        ) => stick[1] = step(stick[1], delta * 0.01, -1., 1.),
        (
            Action::Approach {
                strength, airborne, ..
            },
            Field::Strength(_),
        ) => {
            *strength = step(
                *strength,
                delta * 0.01,
                0.1,
                if *airborne { 1. } else { 0.95 },
            )
        }
        (Action::Walk { strength, .. }, Field::Strength(_)) => {
            *strength = step(*strength, delta * 0.01, 0.1, 0.95)
        }
        (Action::Drift { strength, .. }, Field::Strength(_)) => {
            *strength = step(*strength, delta * 0.01, 0., 1.)
        }
        (Action::Walk { units, .. } | Action::Run { units, .. }, Field::Distance(_)) => {
            *units = step(*units, delta, 0., 100.)
        }
        (Action::Range(RangeCheck::Verify(value)), Field::Distance(_)) => {
            *value = step(*value, delta, 0., 60.)
        }
        (Action::Chase { gap, .. } | Action::Approach { gap, .. }, Field::Gap(_)) => {
            *gap = step(*gap, delta, 0., 60.)
        }
        (Action::Walk { min_gap, .. }, Field::Gap(_)) => {
            let old = min_gap.unwrap_or(0.);
            *min_gap = if !up && old == 0. {
                None
            } else {
                Some(step(old, delta, 0., 60.))
            };
        }
        (
            Action::Walk {
                direction: d,
                min_gap,
                ..
            },
            Field::Direction(_),
        ) => {
            direction(d, up, onstage);
            if *d != D::Opponent {
                *min_gap = None;
            }
        }
        (
            Action::Face(d) | Action::Run { direction: d, .. } | Action::Drift { direction: d, .. },
            Field::Direction(_),
        ) => direction(d, up, onstage),
        _ => {}
    }
}
pub(super) struct CopySource {
    pub slot: super::PresetSlot,
    pub index: usize,
    pub name: String,
}
#[derive(Default)]
pub(super) struct RecipeEditor {
    modal: Option<commands::Modal>,
    return_modal: Option<commands::Modal>,
    step_clipboard: Option<(Action, u8)>,
    manage_requested: bool,
    copy_sources: Vec<CopySource>,
    copy_templates: Vec<String>,
    template_apply: Option<usize>,
    copy_requested: bool,
    copy_apply: Option<(super::PresetSlot, usize)>,
    command_held: u32,
    command_ticks: u16,
    undo: Option<Box<((usize, usize), Recipe)>>,
    recipes: BTreeMap<(usize, usize), Recipe>,
    pub cursor: usize,
    pub limited: bool,
    last_edit: Option<String>,
}
impl RecipeEditor {
    #[cfg(feature = "diagnostic-logging")]
    pub(super) fn retained(&self) -> (usize, usize) {
        let bytes = std::mem::size_of::<Self>()
            + self.undo.as_ref().map_or(0, |undo| {
                std::mem::size_of_val(&**undo)
                    + undo.1.imported.capacity() * std::mem::size_of::<Action>()
            })
            + self.copy_templates.capacity() * std::mem::size_of::<String>()
            + self
                .copy_templates
                .iter()
                .map(String::capacity)
                .sum::<usize>()
            + self.copy_sources.capacity() * std::mem::size_of::<CopySource>()
            + self
                .copy_sources
                .iter()
                .map(|s| s.name.capacity())
                .sum::<usize>()
            + self.last_edit.as_ref().map_or(0, String::capacity)
            + self
                .recipes
                .values()
                .map(|r| {
                    std::mem::size_of::<((usize, usize), Recipe)>()
                        + r.imported.capacity() * std::mem::size_of::<Action>()
                })
                .sum::<usize>();
        (self.recipes.len(), bytes)
    }
    pub fn preview_panel(sequence: Sequence) -> super::RecipePanel {
        recipe_panel(&Recipe::new(sequence), None, false, None)
    }
    pub fn has(&self, key: (usize, usize)) -> bool {
        self.recipes.contains_key(&key)
    }
    pub fn custom() -> Sequence {
        Sequence {
            len: 6,
            ..Sequence::default()
        }
    }
    pub fn ensure(&mut self, key: (usize, usize), source: Sequence) {
        self.close_commands();
        self.recipes.entry(key).or_insert_with(|| {
            let mut r = Recipe::new(source);
            r.button_recipe = key.0 >= 5;
            r
        });
        self.cursor = 0;
        self.limited = false;
        self.last_edit = None;
    }
    pub fn edited(&self, key: (usize, usize)) -> bool {
        self.recipes.get(&key).is_some_and(|r| r.modified)
    }
    pub fn mark_saved(&mut self, key: (usize, usize)) {
        if let Some(r) = self.recipes.get_mut(&key) {
            r.modified = false;
            r.edited = 0;
        }
    }
    pub fn release_tools(&mut self) {
        self.close_commands();
        self.undo = None;
        self.step_clipboard = None;
        self.last_edit = None;
    }
    pub fn retain_recipes(
        &mut self,
        mut retain: impl FnMut((usize, usize), Option<Sequence>, bool) -> bool,
    ) {
        self.recipes
            .retain(|key, recipe| retain(*key, recipe.compile(), recipe.modified));
    }
    pub fn is_empty(&self) -> bool {
        self.recipes.is_empty()
    }
    pub fn discard(&mut self, key: (usize, usize)) {
        self.clear_copy();
        if self.undo.as_ref().is_some_and(|u| u.0 == key) {
            self.undo = None;
        }
        self.modal = None;
        self.return_modal = None;
        self.recipes.remove(&key);
    }
    pub fn while_held(&self, key: (usize, usize)) -> Option<bool> {
        self.recipes.get(&key).map(|r| r.sequence.while_held)
    }
    pub fn compiled(&self, key: (usize, usize)) -> Option<Sequence> {
        self.recipes.get(&key).and_then(Recipe::compile)
    }
    pub fn navigate(&mut self, key: (usize, usize), down: bool) {
        if let Some(r) = self.recipes.get(&key) {
            let n = fields(r).len();
            self.cursor = if down {
                (self.cursor + 1) % n
            } else {
                (self.cursor + n - 1) % n
            };
        }
    }
    pub fn repeatable(&self, key: (usize, usize)) -> bool {
        self.recipes.get(&key).is_some_and(|r| {
            let fs = fields(r);
            !matches!(
                fs[self.cursor.min(fs.len() - 1)],
                Field::Kind(_)
                    | Field::ShieldHold(_)
                    | Field::Direction(_)
                    | Field::TapButtons(_)
                    | Field::WhileHeld
                    | Field::GroundOnly
                    | Field::Buttons(_)
                    | Field::Reference(_)
                    | Field::CReference(_)
                    | Field::Reaction(0 | 1 | 5 | 7)
            )
        })
    }
    pub fn adjust(&mut self, key: (usize, usize), up: bool) -> bool {
        let Some(r) = self.recipes.get_mut(&key) else {
            return false;
        };
        let mut next = r.clone();
        let fs = fields(&next);
        let f = fs[self.cursor.min(fs.len() - 1)];
        let onstage = next.sequence.reaction.active();
        let delta = if up { 1. } else { -1. };
        match f {
            Field::Reaction(0) => {
                let trigger = next.sequence.reaction.trigger;
                let options = trigger.events();
                let i = options.iter().position(|v| *v == trigger).unwrap_or(0);
                next.sequence.reaction.trigger = options[if up {
                    (i + 1) % options.len()
                } else {
                    (i + options.len() - 1) % options.len()
                }];
            }
            Field::Reaction(n) => next.sequence.reaction.adjust(n, up),
            Field::WhileHeld => next.sequence.while_held = !next.sequence.while_held,
            Field::GroundOnly => next.sequence.ground_only = !next.sequence.ground_only,
            Field::Buttons(_) => return false,
            Field::Reference(i) | Field::CReference(i) => {
                let right = matches!(f, Field::CReference(_));
                let old = input_reference(next.sequence.actions[i], right);
                let k = IR::ALL.iter().position(|x| *x == old).unwrap_or(0);
                set_input_reference(
                    &mut next.sequence.actions[i],
                    IR::ALL[(k + if up { 1 } else { 2 }) % 3],
                    right,
                );
            }
            Field::CX(i) | Field::CY(i) => {
                if let Action::Controller { cstick, .. } = &mut next.sequence.actions[i] {
                    let axis = usize::from(matches!(f, Field::CY(_)));
                    cstick[axis] = step(cstick[axis], delta * 0.01, -1., 1.);
                }
            }
            Field::Start => {
                next.sequence.start_delay = if up {
                    next.sequence.start_delay.saturating_add(1).min(3600)
                } else {
                    next.sequence.start_delay.saturating_sub(1)
                }
            }
            Field::Slots => {
                let len = usize::from(next.sequence.len);
                if up && len < MAX_ACTIONS {
                    next.sequence.len += 1;
                } else if !up
                    && len > 1
                    && next.sequence.actions[len - 1] == Action::Empty
                    && next.before[len - 1] == 0
                {
                    next.sequence.len -= 1;
                }
            }
            Field::Before(i) => byte(&mut next.before[i], up, 0),
            Field::Kind(i) => {
                let n = PRESETS.len() + next.imported.len();
                let k = next.kinds[i];
                next.kinds[i] = if k >= n {
                    if up {
                        0
                    } else {
                        n - 1
                    }
                } else if up {
                    (k + 1) % n
                } else {
                    (k + n - 1) % n
                };
                next.sequence.actions[i] = if next.kinds[i] < PRESETS.len() {
                    PRESETS[next.kinds[i]].1
                } else {
                    next.imported[next.kinds[i] - PRESETS.len()]
                };
            }
            Field::ShieldHold(i) => match &mut next.sequence.actions[i] {
                Action::Input { frames, .. } => *frames = if *frames == 0 { 4 } else { 0 },
                Action::ShieldUntilManual => {
                    next.sequence.actions[i] = Action::Input {
                        buttons: MAPPER_ZR,
                        stick: [0.; 2],
                        frames: 4,
                        relative: true,
                    };
                    next.kinds[i] = PRESETS
                        .iter()
                        .position(|(_, a)| *a == next.sequence.actions[i])
                        .unwrap_or(0);
                }
                _ => {}
            },
            other => {
                let i = action_slot(other);
                adjust_action(&mut next.sequence.actions[i], other, up, delta, onstage);
            }
        }
        self.limited = next.compile().is_none();
        if self.limited {
            return false;
        }
        let changed =
            next.sequence != r.sequence || next.before != r.before || next.kinds != r.kinds;
        if changed {
            self.last_edit = Some(format!(
                "Last change: {}{} -> {}",
                field_slot(f).map_or("Start: ".into(), |i| format!("slot {}: ", i + 1)),
                field_value(r, f).trim(),
                field_value(&next, f).trim()
            ));
            next.modified = true;
            if let Some(i) = field_slot(f) {
                next.edited |= 1u32 << i;
            }
            let previous = std::mem::replace(r, next);
            self.undo = Some(Box::new((key, previous)));
            self.cursor = self.cursor.min(fields(r).len() - 1);
        }
        changed
    }
    pub(super) fn reference_visible(&self) -> bool {
        matches!(self.modal, None | Some(commands::Modal::Picker { .. }))
    }
    pub(super) fn reference_label(&self, key: (usize, usize)) -> Option<&'static str> {
        let r = self.recipes.get(&key)?;
        if let Some(commands::Modal::Picker { group, item, .. }) = self.modal {
            let kind = *commands::candidates(r, group).get(item)?;
            return PRESETS.get(kind).map(|(label, _)| *label);
        }
        if self.modal.is_some() {
            return None;
        }
        let fs = fields(r);
        let slot = field_slot(*fs.get(self.cursor.min(fs.len() - 1))?)?;
        let (label, preset) = PRESETS.get(r.kinds[slot])?;
        match (r.sequence.actions[slot], *preset) {
            (
                Action::Input { buttons, stick, .. },
                Action::Input {
                    buttons: b,
                    stick: s,
                    ..
                },
            ) if buttons == b && stick == s => Some(*label),
            (Action::Tap { .. }, Action::Tap { .. }) => Some("Repeated taps"),
            _ => None,
        }
    }
    pub fn panel(&self, key: (usize, usize), editing: bool) -> super::RecipePanel {
        if editing {
            if let Some(panel) = self.command_panel(key) {
                return panel;
            }
        }
        let Some(r) = self.recipes.get(&key) else {
            return super::RecipePanel::default();
        };
        let fs = fields(r);
        let cursor = self.cursor.min(fs.len() - 1);
        let mut panel = recipe_panel(
            r,
            editing.then_some(fs[cursor]),
            self.limited,
            self.last_edit.as_deref(),
        );
        if editing {
            panel.header.push(format!(
                "Field {} / {} | * marks an unsaved slot",
                cursor + 1,
                fs.len()
            ));
        }
        if editing {
            panel
                .footer
                .push("A on move: Categories | ZL+ZR: Tools | ZL/ZR: Jump step".into());
        }
        panel
    }
    #[cfg(test)]
    pub fn lines(&self, key: (usize, usize)) -> Vec<String> {
        self.panel(key, true).into_lines()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        fighter_profiles::{presets::PresetSlot, FighterProfiles},
        ledge_trap::DPAD_DOWN,
        manual_controls::DPAD_RIGHT,
        settings::AssistSettings,
        settings_menu::{MenuSelection, MENU_STICK_CLICK},
    };

    #[test]
    fn edited_actions_keep_their_matching_preset_kind() {
        let forward_tilt = PRESETS
            .iter()
            .position(|(name, _)| *name == "Forward A (Fair / Fsmash)")
            .unwrap();
        let edited_input = Action::Input {
            buttons: MAPPER_A,
            stick: [1., 0.],
            frames: 7,
            relative: true,
        };
        assert_eq!(preset_kind(edited_input), Some(forward_tilt));

        assert!(same_preset_shape(
            Action::Range(RangeCheck::Verify(4.)),
            Action::Range(RangeCheck::Verify(10.))
        ));
        assert!(!same_preset_shape(
            Action::Range(RangeCheck::Verify(4.)),
            Action::Range(RangeCheck::Ignore)
        ));
    }

    fn presets() -> crate::fighter_profiles::presets::MenuPresets {
        let file = crate::sequence_config::Catalog::parse(
            "[kirby.neutral.first]\nstart_delay=0\nactions=short_hop,wait:2,nair",
        )
        .unwrap();
        let mut p = FighterProfiles::default();
        p.add_sequence_choices(&file).unwrap();
        p.profile(6).unwrap().menu_options
    }
    fn tap(
        menu: &mut MenuSelection,
        p: &crate::fighter_profiles::presets::MenuPresets,
        button: u32,
    ) {
        menu.input(0, p);
        menu.input(button, p);
    }
    #[test]
    fn copy_menu_can_template_a_button_into_ledge_without_changing_source_or_name() {
        let file=crate::sequence_config::Catalog::parse("[kirby.neutral.target]\nname=Keep my name\nstart_delay=0\nactions=grab\n[kirby.x.source]\nname=Nice down smash\ndefault=true\nstart_delay=3\ntimeout=700\nspatial_timeout=80\nwhile_held=true\nactions=wait:2,down_smash:4").unwrap();
        let mut profiles = FighterProfiles::default();
        profiles.add_sequence_choices(&file).unwrap();
        let p = profiles.profile(6).unwrap().menu_options;
        let mut m = MenuSelection::new(&p);
        m.show();
        m.input(0, &p);
        tap(&mut m, &p, MENU_STICK_CLICK);
        let before = m.editor.as_ref().unwrap().compiled((0, 0)).unwrap();
        tap(&mut m, &p, MAPPER_A);
        for _ in 0..8 {
            tap(&mut m, &p, crate::ledge_trap::DPAD_DOWN);
        }
        tap(&mut m, &p, MAPPER_A);
        assert!(m.recipe_panel(&p).header.join(" ").contains("COPY RECIPE"));
        let e = m.editor.as_ref().unwrap();
        assert_eq!(e.copy_sources.len(), 1);
        assert_eq!(e.copy_sources[0].name, "Nice down smash");
        for _ in 0..5 {
            tap(&mut m, &p, DPAD_RIGHT);
        }
        tap(&mut m, &p, MAPPER_A);
        let e = m.editor.as_ref().unwrap();
        let target = e.compiled((0, 0)).unwrap();
        let source = *p.lists[5].options[0].sequence;
        assert_eq!(target.actions(), source.actions());
        assert_eq!(target.start_delay, 3);
        assert_eq!(target.timeout, 700);
        assert_eq!(target.spatial_timeout, 80);
        assert!(!target.while_held);
        assert_eq!(
            m.choice(&p, crate::fighter_profiles::presets::PresetSlot::Neutral, 0)
                .unwrap()
                .name,
            "Keep my name"
        );
        assert_eq!(*p.lists[5].options[0].sequence, source);
        assert_ne!(target, before);
        assert!(!m.save_requested);
        assert_eq!(e.copy_sources.capacity(), 0);
    }
    #[test]
    fn editor_tools_consume_controls_cancel_safely_and_do_not_rebuild_while_idle() {
        let p = presets();
        let mut menu = MenuSelection::new(&p);
        menu.show();
        menu.input(0, &p);
        tap(&mut menu, &p, MENU_STICK_CLICK);
        assert!(menu.recipe_editing);
        tap(&mut menu, &p, MAPPER_A);
        assert!(menu
            .recipe_panel(&p)
            .header
            .join(" ")
            .contains("STEP TOOLS"));
        assert!(!menu.recipe_panel(&p).footer.join(" ").contains("B: Manage"));
        let revision = menu.revision;
        for _ in 0..120 {
            menu.input(0, &p);
        }
        assert_eq!(menu.revision, revision);
        tap(&mut menu, &p, MAPPER_Y);
        assert!(
            menu.save_requested,
            "Y saves the current recipe from tools too"
        );
        tap(&mut menu, &p, MAPPER_B);
        assert!(!menu.managing());
        assert!(!menu.recipe_editing);
        tap(&mut menu, &p, MENU_STICK_CLICK);
        let mask = menu.input(MAPPER_ZR, &p);
        assert_ne!(mask & MAPPER_ZR, 0);
        assert!(menu.recipe_panel(&p).header.join(" ").contains("STEP 1 /"));
        menu.input(MAPPER_ZR | MAPPER_MINUS, &p);
        assert!(menu.menu_hidden);
        assert_ne!(menu.input(MAPPER_ZR, &p) & MAPPER_ZR, 0);
        menu.input(0, &p);
        assert_eq!(menu.input(MAPPER_ZR, &p) & MAPPER_ZR, 0);
    }

    #[test]
    fn modified_save_explains_rejection_in_editor_and_tools_without_losing_edits() {
        let p = presets();
        for tools in [false, true] {
            let mut menu = MenuSelection::new(&p);
            menu.show();
            menu.input(0, &p);
            tap(&mut menu, &p, MENU_STICK_CLICK);
            let before = menu
                .recipe_snapshot(&p, crate::fighter_profiles::presets::PresetSlot::Neutral, 0)
                .unwrap();
            if tools {
                tap(&mut menu, &p, MAPPER_A);
            }
            tap(&mut menu, &p, crate::native_input::MAPPER_L | MAPPER_Y);
            assert!(!menu.save_requested);
            assert!(menu
                .save_status
                .as_deref()
                .unwrap()
                .contains("Release other buttons"));
            assert_eq!(
                menu.recipe_snapshot(&p, crate::fighter_profiles::presets::PresetSlot::Neutral, 0),
                Some(before)
            );
            tap(&mut menu, &p, MAPPER_Y);
            assert!(menu.save_requested);
        }
    }
    #[test]
    fn button_activation_is_editable_and_not_exposed_for_ledge_recipes() {
        let mut e = RecipeEditor::default();
        let source = RecipeEditor::custom();
        e.ensure((0, 0), source);
        assert!(!fields(&e.recipes[&(0, 0)])
            .iter()
            .any(|f| matches!(f, Field::WhileHeld)));
        let key = (5, 0);
        e.ensure(key, source);
        e.cursor = fields(&e.recipes[&key])
            .iter()
            .position(|f| matches!(f, Field::WhileHeld))
            .unwrap();
        assert!(!e.repeatable(key));
        assert!(e.adjust(key, true));
        assert!(e.compiled(key).unwrap().while_held);
        assert!(e
            .panel(key, true)
            .body
            .join(" ")
            .contains("Loop while held"));
    }
    #[test]
    fn repeated_tap_picker_has_only_buttons_and_tilts_with_gap_and_total_fields() {
        let source = crate::sequence_config::Catalog::parse(
            "[kirby.x.t]\nstart_delay=0\nactions=tap:a:0:0:1:11:120",
        )
        .unwrap()
        .entries()[0]
            .sequence;
        let mut e = RecipeEditor::default();
        let key = (5, 0);
        e.ensure(key, source);
        assert!(!PRESETS.iter().any(|(name, _)| *name == "Repeat down tilt"));
        let fs = fields(&e.recipes[&key]);
        assert!(!fs.iter().any(|f| matches!(f, Field::X(0) | Field::Y(0))));
        for (button, stick, label) in TAP_BUTTONS.iter().skip(1).chain(TAP_BUTTONS.iter().take(1)) {
            e.cursor = fields(&e.recipes[&key])
                .iter()
                .position(|f| matches!(f, Field::TapButtons(0)))
                .unwrap();
            assert!(e.adjust(key, true));
            assert!(
                matches!(e.compiled(key).unwrap().actions(), [Action::Tap {buttons,stick:axes,hold:1,..}] if buttons==button && axes==stick)
            );
            assert!(field_value(&e.recipes[&key], Field::TapButtons(0)).contains(label));
        }
        for field in [Field::TapGap(0), Field::Duration(0)] {
            e.cursor = fields(&e.recipes[&key])
                .iter()
                .position(|f| same_field(*f, field))
                .unwrap();
            assert!(e.adjust(key, true));
        }
        assert!(matches!(
            e.compiled(key).unwrap().actions(),
            [Action::Tap {
                gap: 12,
                frames: 121,
                ..
            }]
        ));
        assert!(e.panel(key, true).body.join(" ").contains("every 13f"));
    }
    #[test]
    fn condition_fields_roundtrip_and_slot_growth_cannot_erase_actions() {
        let source = crate::sequence_config::Catalog::parse(
            "[yoshi.roll.t]\nstart_delay=0\nactions=check:enemy_within:12:8,hold:a:60\n",
        )
        .unwrap()
        .entries()[0]
            .sequence;
        let key = (0, 0);
        let mut editor = RecipeEditor::default();
        editor.ensure(key, source);
        assert_eq!(editor.compiled(key).unwrap().actions(), source.actions());
        let r = editor.recipes.get(&key).unwrap();
        assert!(action_name(r, 0).contains("distance within"));
        editor.cursor = fields(r)
            .iter()
            .position(|f| matches!(f, Field::Distance(0)))
            .unwrap();
        assert!(editor.adjust(key, true));
        assert!(matches!(
            editor.compiled(key).unwrap().actions()[0],
            Action::Check {
                value: 13.,
                wait: 8,
                ..
            }
        ));
        let r = editor.recipes.get(&key).unwrap();
        editor.cursor = fields(r)
            .iter()
            .position(|f| matches!(f, Field::Slots))
            .unwrap();
        // Filled trailing actions cannot be removed by reducing the count.
        for _ in 0..8 {
            editor.cursor = fields(&editor.recipes[&key]).len() - 1;
            editor.adjust(key, false);
        }
        assert_eq!(editor.recipes[&key].sequence.len, 2);
        for _ in 0..40 {
            let r = &editor.recipes[&key];
            editor.cursor = fields(r).len() - 1;
            editor.adjust(key, true);
        }
        assert_eq!(editor.recipes[&key].sequence.len, MAX_ACTIONS as u8);
        assert_eq!(
            editor.compiled(key).unwrap().actions(),
            &[
                Action::Check {
                    condition: Condition::EnemyWithin,
                    value: 13.,
                    wait: 8
                },
                source.actions()[1],
            ]
        );
    }
    #[test]
    fn schedule_shows_input_frames_with_one_based_recognition_and_preserves_recipe() {
        let source = crate::sequence_config::Catalog::parse(
            "[kirby.neutral.t]\nstart_delay=2\nactions=short_hop:1,wait:3,hold:a:4,wait:2\n",
        )
        .unwrap()
        .entries()[0]
            .sequence;
        let recipe = Recipe::new(source);
        assert_eq!(schedule_summary(&recipe), (12, false));
        let panel = recipe_panel(&recipe, None, false, None);
        let text = panel.header.join("\n") + "\n" + &panel.body.join("\n");
        assert!(text.contains("Input schedule: 12f | done F13"));
        assert!(text.contains("F3"));
        assert!(text.contains("F7-10"));
        assert!(text.contains("F11-12"));
        assert_eq!(recipe.compile().unwrap().start_delay, source.start_delay);
        assert_eq!(recipe.compile().unwrap().actions(), source.actions());
        let mut limited = recipe;
        limited.sequence.timeout = 6;
        let header = recipe_panel(&limited, None, false, None).header.join(" ");
        assert!(header.contains("timeout 6f"));
        assert!(!header.contains("done F"));
    }
    #[test]
    fn schedule_never_claims_exact_frames_after_a_spatial_or_block_check() {
        let source = crate::sequence_config::Catalog::parse(
            "[yoshi.roll.t]\nstart_delay=0\nactions=walk:inward:4:50%,face:opponent,wait:2,grab\n",
        )
        .unwrap()
        .entries()[0]
            .sequence;
        let panel = recipe_panel(&Recipe::new(source), None, false, None);
        assert!(panel.header.join(" ").contains("variable checks"));
        let text = panel.body.join("\n");
        assert!(text.contains("F1 check"));
        assert!(text.contains("after checks"));
        assert!(!panel.header.join(" ").contains("done F"));
    }
    #[test]
    fn imported_input_is_descriptive_and_can_be_reselected_after_cycling() {
        let action = Action::Input {
            buttons: MAPPER_A | MAPPER_ZR,
            stick: [0.35, -0.2],
            frames: 19,
            relative: false,
        };
        let mut source = Sequence::default();
        source.len = 1;
        source.actions[0] = action;
        let mut editor = RecipeEditor::default();
        let key = (0, 0);
        editor.ensure(key, source);
        editor.cursor = 1;
        let name = editor.lines(key).join("\n");
        assert!(name.contains("A+ZR | X:35% Y:-20% (screen)"));
        assert!(editor.adjust(key, true));
        assert_eq!(editor.compiled(key).unwrap().len, 0);
        assert!(editor.adjust(key, false));
        assert_eq!(editor.compiled(key).unwrap().actions(), &[action]);
        // Full forward cycle also keeps the original selectable.
        for _ in 0..PRESETS.len() + 1 {
            editor.adjust(key, true);
        }
        assert_eq!(editor.compiled(key).unwrap().actions(), &[action]);
    }
    #[test]
    fn conditional_range_choices_keep_distinct_names_and_controls() {
        let mut source = Sequence::default();
        source.len = 4;
        source.actions[3] = Action::Range(RangeCheck::FighterGrabRangeInstant);
        source.actions[0] = Action::Range(RangeCheck::FighterGrabRange);
        source.actions[1] = Action::Range(RangeCheck::Verify(17.));
        source.actions[2] = Action::ShieldUntilBlock(60);
        let r = Recipe::new(source);
        assert_eq!(action_name(&r, 0), "Check fighter grab reach");
        assert_eq!(action_name(&r, 1), "Check opponent distance");
        assert_eq!(action_name(&r, 2), "Wait for shield hit");
        assert_eq!(action_name(&r, 3), "Check fighter grab reach instantly");
        let fs = fields(&r);
        assert!(!fs.iter().any(|f| matches!(f, Field::Distance(0))));
        assert!(fs.iter().any(|f| matches!(f, Field::Distance(1))));
        assert!(fs.iter().any(|f| matches!(f, Field::Duration(2))));
        assert_eq!(r.compile().unwrap().actions(), source.actions());
    }
    #[test]
    fn button_hold_duration_changes_only_the_input_hold() {
        let mut source = Sequence::default();
        source.len = 1;
        source.actions[0] = key(MAPPER_B, 0., 0., 4);
        let mut editor = RecipeEditor::default();
        let key = (0, 0);
        editor.ensure(key, source);
        editor.cursor = fields(&editor.recipes[&key])
            .iter()
            .position(|f| matches!(f, Field::Duration(0)))
            .unwrap();
        for _ in 0..6 {
            assert!(editor.adjust(key, true));
        }
        let result = editor.compiled(key).unwrap();
        assert_eq!(result.actions(), &[super::key(MAPPER_B, 0., 0., 10)]);
        assert_eq!(result.start_delay, 0);
        assert!(editor.lines(key).join("\n").contains("Hold input: 10f"));
    }
    #[test]
    fn full_sequence_panel_retains_all_moves_settings_and_exact_focus() {
        let mut sequence = RecipeEditor::custom();
        sequence.len = 32;
        sequence.actions.fill(key(MAPPER_A, 0.5, -0.25, 4));
        let mut editor = RecipeEditor::default();
        let key = (0, 0);
        editor.ensure(key, sequence);
        editor.cursor = fields(&editor.recipes[&key])
            .iter()
            .position(|f| matches!(f, Field::Before(31)))
            .unwrap(); // Exact field, independent of added settings.
        let panel = editor.panel(key, true);
        let body = panel.body.join("\n");
        assert!(body.contains("01  "));
        assert!(body.contains("32  "));
        assert_eq!(body.matches("Hold input: 4f").count(), 32);
        assert_eq!(body.matches("Left X: 50%").count(), 32);
        assert_eq!(body.matches("Left Y: -25%").count(), 32);
        let focused = &panel.body[panel.focus.unwrap()];
        assert!(focused.contains("[Delay: 0f]"));
        assert!(panel.header.iter().any(|s| s.contains("STEP 32 / 32")));
        let preview = editor.panel(key, false);
        assert!(preview.focus.is_none());
        assert!(
            preview.body.len() < panel.body.len(),
            "only the focused step expands its settings"
        );
    }
    #[test]
    fn numbered_slot_overview_keeps_hidden_edits_visible_and_marks_saved_state() {
        let mut e = RecipeEditor::default();
        let key = (0, 0);
        e.ensure(key, RecipeEditor::custom());
        e.cursor = 1;
        assert!(e.adjust(key, true));
        let first = e.lines(key).join("\n");
        assert!(first.contains("STEP 1 / 6"));
        assert!(first.contains("01  Short hop *"));
        e.cursor = fields(&e.recipes[&key])
            .iter()
            .position(|f| matches!(f, Field::Kind(1)))
            .unwrap();
        let next = e.lines(key).join("\n");
        assert!(next.contains("STEP 2 / 6"));
        assert!(next.contains("01  Short hop *"));
        assert!(next.contains("Last change: slot 1"));
        assert!(e.edited(key));
        e.mark_saved(key);
        assert!(!e.edited(key));
        assert!(!e.lines(key).join("\n").contains("01  Short hop *"));
    }
    #[test]
    fn training_reference_tracks_focused_move_and_picker_without_changing_recipe() {
        let p = presets();
        let mut menu = MenuSelection::new(&p);
        menu.show();
        menu.input(0, &p);
        tap(&mut menu, &p, MENU_STICK_CLICK);
        let recipe_key = (
            PresetSlot::Neutral as usize,
            menu.choices[PresetSlot::Neutral as usize],
        );
        let editor = menu.editor.as_mut().unwrap();
        let kind = PRESETS
            .iter()
            .position(|(label, _)| *label == "Dtilt")
            .unwrap();
        let r = editor.recipes.get_mut(&recipe_key).unwrap();
        r.kinds[0] = kind;
        r.sequence.actions[0] = PRESETS[kind].1;
        editor.cursor = fields(r)
            .iter()
            .position(|f| matches!(f, Field::Kind(0)))
            .unwrap();
        let original = editor.compiled(recipe_key).unwrap();
        let hint = menu.recipe_panel(&p).header.join(" ");
        assert!(hint.contains("kirby / Down Tilt"));
        assert!(hint.contains("roster avg"));
        assert_eq!(
            menu.editor.as_ref().unwrap().compiled(recipe_key).unwrap(),
            original
        );
        menu.y_fighter_kind = 5;
        assert!(menu
            .recipe_panel(&p)
            .header
            .join(" ")
            .contains("yoshi / Down Tilt"));
        let editor = menu.editor.as_mut().unwrap();
        let r = editor.recipes.get(&recipe_key).unwrap();
        let fair = PRESETS
            .iter()
            .position(|(label, _)| *label == "Short hop Fair")
            .unwrap();
        let item = commands::candidates(r, 1)
            .iter()
            .position(|i| *i == fair)
            .unwrap();
        editor.modal = Some(commands::Modal::Picker {
            slot: 0,
            group: 1,
            item,
        });
        assert!(menu
            .recipe_panel(&p)
            .header
            .join(" ")
            .contains("yoshi / Forward Air"));
        assert_eq!(
            menu.editor.as_ref().unwrap().compiled(recipe_key).unwrap(),
            original
        );
        menu.lightweight = true;
        let hint = menu.recipe_panel(&p).header.join(" ");
        assert!(!hint.contains("REF "));
        assert!(!hint.contains("roster avg"));
        menu.lightweight = false;
        menu.recipe_editing = false;
        assert!(!menu.recipe_panel(&p).header.join(" ").contains("REF "));
    }

    #[test]
    fn save_wait_consumes_inputs_without_hidden_edits_or_held_save_replay() {
        let p = presets();
        let mut menu = MenuSelection::new(&p);
        menu.show();
        menu.input(0, &p);
        tap(&mut menu, &p, MENU_STICK_CLICK);
        menu.input(MAPPER_Y, &p);
        assert!(menu.save_requested);
        let revision = menu.recipe_revision;
        for _ in 0..10 {
            let buttons = MAPPER_Y | DPAD_RIGHT;
            assert_eq!(menu.wait_for_save(buttons), buttons);
            assert!(!menu.save_requested);
            assert!(!menu.delete_requested);
            assert_eq!(menu.recipe_revision, revision);
        }
        menu.input(MAPPER_Y, &p);
        assert!(
            !menu.save_requested,
            "held Save must not restart when worker exits"
        );
        menu.input(0, &p);
        menu.input(MAPPER_Y, &p);
        assert!(menu.save_requested);
    }

    #[test]
    fn menu_save_is_editor_only_once_per_press_and_cancels_preview() {
        let p = presets();
        let mut menu = MenuSelection::new(&p);
        menu.show();
        menu.input(0, &p);
        menu.input(MAPPER_Y, &p);
        assert!(!menu.save_requested);
        menu.input(0, &p);
        tap(&mut menu, &p, MENU_STICK_CLICK);
        tap(&mut menu, &p, MAPPER_X);
        assert!(menu.preview_slot.is_some());
        let consumed = menu.input(MAPPER_Y, &p);
        assert!(menu.save_requested);
        assert_ne!(consumed & MAPPER_Y, 0);
        assert!(menu.preview_slot.is_none());
        assert!(menu.cancel_pending);
        menu.input(MAPPER_Y, &p);
        assert!(!menu.save_requested);
        menu.input(0, &p);
        menu.input(MAPPER_Y, &p);
        assert!(menu.save_requested);
        menu.reset_inputs();
        assert!(!menu.save_requested);
    }
    #[test]
    fn menu_preview_button_is_one_shot_consumed_and_menu_changes_cancel() {
        let p = presets();
        let mut menu = MenuSelection::new(&p);
        menu.show();
        menu.input(0, &p);
        assert_ne!(menu.input(MAPPER_X, &p) & MAPPER_X, 0);
        assert_eq!(menu.preview_slot, Some(PresetSlot::Neutral));
        let revision = menu.revision;
        for _ in 0..10 {
            menu.input(MAPPER_X, &p);
        }
        assert_eq!(menu.revision, revision);
        menu.input(0, &p);
        tap(&mut menu, &p, MAPPER_X);
        assert_eq!(menu.preview_slot, None);
        tap(&mut menu, &p, MAPPER_X);
        assert!(menu.preview_slot.is_some());
        tap(&mut menu, &p, MENU_STICK_CLICK);
        assert!(menu.recipe_editing);
        assert_eq!(menu.preview_slot, None);
        tap(&mut menu, &p, MAPPER_X);
        assert!(menu.preview_slot.is_some());
        tap(&mut menu, &p, DPAD_DOWN);
        assert_eq!(menu.preview_slot, None);
        tap(&mut menu, &p, MAPPER_X);
        menu.reset_inputs();
        assert_eq!(menu.preview_slot, None);
        menu.input(0, &p);
        menu.input(MAPPER_X, &p);
        let consumed = menu.input(MAPPER_X | crate::native_input::MAPPER_MINUS, &p);
        assert!(menu.menu_hidden && menu.preview_slot.is_none());
        assert_ne!(consumed & MAPPER_X, 0);
        assert_eq!(menu.preview_slot, None);
        assert_ne!(consumed & MAPPER_X, 0);
        let consumed = menu.input(MAPPER_X, &p);
        assert_eq!(
            super::super::observation_buttons(MAPPER_X, consumed) & MAPPER_X,
            0
        );
        menu.input(0, &p);
        assert_ne!(menu.input(MAPPER_X, &p) & MAPPER_X, 0);
    }
    #[test]
    fn recipe_editor_numeric_hold_accelerates_and_release_resets() {
        let p = presets();
        let mut menu = MenuSelection::new(&p);
        menu.show();
        tap(&mut menu, &p, MENU_STICK_CLICK);
        let delay = |m: &MenuSelection| {
            m.settings(&p, AssistSettings::default()).sequence_overrides[0]
                .unwrap()
                .start_delay
        };
        menu.input(DPAD_RIGHT, &p);
        assert_eq!(delay(&menu), 1);
        for _ in 0..23 {
            menu.input(DPAD_RIGHT, &p);
        }
        assert_eq!(delay(&menu), 1);
        menu.input(DPAD_RIGHT, &p);
        assert_eq!(delay(&menu), 2);
        for _ in 0..66 {
            menu.input(DPAD_RIGHT, &p);
        }
        let before = delay(&menu);
        for _ in 0..12 {
            menu.input(DPAD_RIGHT, &p);
        }
        assert_eq!(delay(&menu) - before, 6);
        menu.input(0, &p);
        tap(&mut menu, &p, DPAD_RIGHT);
        assert_eq!(delay(&menu), before + 7);
        tap(&mut menu, &p, DPAD_DOWN); // move picker stays tap-only
        menu.input(DPAD_RIGHT, &p);
        let revision = menu.recipe_revision;
        for _ in 0..120 {
            menu.input(DPAD_RIGHT, &p);
        }
        assert_eq!(menu.recipe_revision, revision);
    }
    #[test]
    fn recipe_editor_file_waits_become_delays_without_losing_frames() {
        let mut source = Sequence::default();
        source.len = 4;
        source.actions[..4].copy_from_slice(&[
            Action::Wait(255),
            Action::Wait(4),
            key(MAPPER_A, 0., 0., 1),
            Action::Wait(2),
        ]);
        let recipe = Recipe::new(source);
        assert!(!PRESETS.iter().any(|(_, a)| matches!(a, Action::Wait(_))));
        assert!(!recipe
            .sequence
            .actions()
            .iter()
            .any(|a| matches!(a, Action::Wait(_))));
        assert_eq!(recipe.compile().unwrap().actions(), source.actions());
        let mut editor = RecipeEditor::default();
        editor.ensure((0, 0), source);
        editor.navigate((0, 0), true);
        let lines = editor.lines((0, 0)).join("\n");
        assert!(!lines.contains("Slot "));
        assert!(lines.contains("Delay:"));
    }
    #[test]
    fn recipe_editor_changes_next_attempt_delay_without_touching_file_or_old_snapshot() {
        let p = presets();
        let mut menu = MenuSelection::new(&p);
        menu.show();
        let old = menu
            .settings(&p, AssistSettings::default())
            .sequence_overrides[0]
            .unwrap();
        tap(&mut menu, &p, MENU_STICK_CLICK);
        assert!(menu.recipe_editing);
        for _ in 0..4 {
            tap(&mut menu, &p, DPAD_RIGHT);
        }
        let next = menu
            .settings(&p, AssistSettings::default())
            .sequence_overrides[0]
            .unwrap();
        assert_eq!(next.start_delay, 4);
        assert_eq!(old.start_delay, 0);
        assert_eq!(p.lists[0].options[0].sequence.start_delay, 0);
        assert_eq!(menu.recipe_revision, 4);
        assert!(!menu.ledge_held(MAPPER_L));
        tap(&mut menu, &p, MENU_STICK_CLICK);
        assert!(!menu.recipe_editing);
        assert!(menu.recipe_modified(PresetSlot::Neutral));
    }
    #[test]
    fn recipe_editor_custom_slots_swap_actions_and_insert_exact_delays() {
        let p = presets();
        let mut menu = MenuSelection::new(&p);
        menu.show();
        menu.choices[0] = p.lists[0].options.len();
        tap(&mut menu, &p, MENU_STICK_CLICK);
        tap(&mut menu, &p, DPAD_DOWN); // slot 1 type
        for _ in 0..1 {
            tap(&mut menu, &p, DPAD_RIGHT);
        } // short hop
        let k = (0, menu.choices[0]);
        {
            let e = menu.editor.as_mut().unwrap();
            e.cursor = fields(&e.recipes[&k])
                .iter()
                .position(|f| matches!(f, Field::Kind(1)))
                .unwrap();
        }
        for _ in 0..3 {
            tap(&mut menu, &p, DPAD_RIGHT);
        } // neutral A
        {
            let e = menu.editor.as_mut().unwrap();
            e.cursor = fields(&e.recipes[&k])
                .iter()
                .position(|f| matches!(f, Field::Before(1)))
                .unwrap();
        }
        for _ in 0..2 {
            tap(&mut menu, &p, DPAD_RIGHT);
        } // 2 frames before A
        let s = menu
            .settings(&p, AssistSettings::default())
            .sequence_overrides[0]
            .unwrap();
        assert_eq!(
            s.actions(),
            &[
                key(MAPPER_X, 0., 0., 1),
                Action::Wait(2),
                key(MAPPER_A, 0., 0., 4)
            ]
        );
    }
    #[test]
    fn recipe_editor_movement_bounds_and_expansion_limit_keep_valid_snapshots() {
        let mut e = RecipeEditor::default();
        let mut source = RecipeEditor::custom();
        source.len = 1;
        source.actions[0] = Action::Approach {
            airborne: false,
            strength: 0.8,
            frames: 30,
            gap: 10.,
        };
        e.ensure((0, 0), source);
        e.cursor = 3;
        e.adjust((0, 0), true);
        e.cursor = 4;
        e.adjust((0, 0), false);
        e.cursor = 5;
        e.adjust((0, 0), false);
        assert_eq!(
            e.compiled((0, 0)).unwrap().actions[0],
            Action::Approach {
                airborne: false,
                strength: 0.81,
                frames: 29,
                gap: 9.
            }
        );
        e.cursor = 3;
        for _ in 0..50 {
            e.adjust((0, 0), true);
        }
        assert!(matches!(
            e.compiled((0, 0)).unwrap().actions[0],
            Action::Approach { strength: 0.95, .. }
        ));
        let mut full = Sequence::default();
        full.len = MAX_ACTIONS as u8;
        full.actions.fill(key(MAPPER_A, 0., 0., 1));
        e.ensure((0, 1), full);
        e.cursor = 2;
        assert!(!e.adjust((0, 1), true));
        assert!(e.limited);
        assert_eq!(e.compiled((0, 1)).unwrap(), full);
    }
    #[test]
    fn menu_edited_a_hold_continues_after_jab_into_the_next_action() {
        use crate::{
            fighter_profiles::presets::PresetSlot,
            gameplay::{GameplayContext, Opponent},
            ledge_trap::LedgeTrap,
            native_input::{ButtonTrace, MapperButtonOverlay},
            settings::AssistSettings,
        };
        let mut source = Sequence::default();
        source.len = 2;
        source.actions[0] = key(MAPPER_A, 0., 0., 4);
        source.actions[1] = key(MAPPER_B, 0., 0., 4);
        let mut editor = RecipeEditor::default();
        let recipe = (PresetSlot::X as usize, 0);
        editor.ensure(recipe, source);
        editor.cursor = fields(&editor.recipes[&recipe])
            .iter()
            .position(|f| matches!(f, Field::Duration(0)))
            .unwrap();
        for _ in 4..60 {
            assert!(editor.adjust(recipe, true));
        }
        let mut settings = AssistSettings::default();
        settings.enabled = true;
        settings.sequence_overrides[PresetSlot::X as usize] = editor.compiled(recipe);
        let mut trap = LedgeTrap::default();
        trap.request_preview(PresetSlot::X);
        let mut overlay = MapperButtonOverlay::default();
        for frame in 0..=64 {
            // A normal jab starts, finishes, then returns to idle. No action
            // readiness or continued physical trigger is needed for the hold.
            let (status, motion, native_frame) = if frame == 0 {
                (0, "wait", 0.)
            } else if frame < 15 {
                (
                    crate::native_states::status::ATTACK,
                    "attack_11",
                    (frame - 1) as f32,
                )
            } else {
                (0, "wait", (frame - 15) as f32)
            };
            let out = trap.sample(
                settings,
                frame,
                GameplayContext {
                    held: true,
                    gameplay: true,
                    facing: 1.,
                    motion_rate: 1.,
                    status,
                    motion_kind: crate::timing::motion_hash(motion),
                    motion_frame: native_frame,
                    ..GameplayContext::default()
                },
                Some(Opponent {
                    id: 2,
                    ..Opponent::default()
                }),
            );
            assert_eq!(trap.requests_attack_hold(), frame < 60);
            let mapped = overlay.apply_raw(ButtonTrace::default(), out.requested, out.owned);
            assert_eq!(
                mapped.held & MAPPER_A,
                if frame < 60 { MAPPER_A } else { 0 },
                "frame {frame}"
            );
            assert_eq!(
                mapped.held & MAPPER_B,
                if (60..64).contains(&frame) {
                    MAPPER_B
                } else {
                    0
                },
                "frame {frame}"
            );
            assert_eq!(
                mapped.pressed & MAPPER_A,
                if frame == 0 { MAPPER_A } else { 0 }
            );
        }
        assert!(trap.manual_after_response());
    }
}

#[cfg(test)]
mod reaction_editor_tests {
    use super::*;
    use crate::{fighter_profiles::presets::PresetSlot, settings_menu::MenuSelection};
    #[test]
    fn assist_editor_tunes_roll_conditions_without_moving_rule_to_manual_choices() {
        let source = crate::sequence_config::Catalog::parse(
            "[yoshi.roll.test]\nstart_delay=2\nreaction=roll:any:0:20:18:any:1\nactions=face:opponent,grab\n",
        )
        .unwrap()
        .entries()[0]
            .sequence;
        let key = (2, 0);
        let mut e = RecipeEditor::default();
        e.ensure(key, source);
        e.cursor = fields(e.recipes.get(&key).unwrap())
            .iter()
            .position(|f| matches!(f, Field::Reaction(0)))
            .unwrap();
        assert!(e.adjust(key, true));
        assert_eq!(
            e.compiled(key).unwrap().reaction.trigger,
            crate::reaction_rules::Trigger::Crossed
        );
        e.cursor = fields(e.recipes.get(&key).unwrap())
            .iter()
            .position(|f| matches!(f, Field::Reaction(1)))
            .unwrap();
        e.adjust(key, true);
        let compiled = e.compiled(key).unwrap();
        assert_eq!(
            compiled.reaction.state,
            crate::reaction_rules::SelfState::Grounded
        );
        assert_eq!(compiled.actions(), source.actions());
        assert_eq!(compiled.start_delay, 2);
        assert!(e
            .panel(key, true)
            .into_lines()
            .iter()
            .any(|l| l.contains("Auto reaction: roll")));
        e.cursor = fields(e.recipes.get(&key).unwrap())
            .iter()
            .position(|f| matches!(f, Field::Reaction(0)))
            .unwrap();
        e.adjust(key, false);
        assert_eq!(
            e.compiled(key).unwrap().reaction.trigger,
            crate::reaction_rules::Trigger::Roll
        );
    }
    #[test]
    fn facing_block_editor_preserves_marker_names_and_roundtrip() {
        let c=crate::sequence_config::Catalog::parse("[yoshi.neutral.test]\nstart_delay=0\nactions=if_facing_ledge, a:4, end_facing, if_back_to_ledge, b:4, end_facing").unwrap();
        let sequence = c.entries()[0].sequence;
        let r = Recipe::new(sequence);
        assert_eq!(action_name(&r, 0), "If facing ledge");
        assert_eq!(action_name(&r, 3), "If back to ledge");
        assert_eq!(action_name(&r, 5), "End facing block");
        assert_eq!(r.compile().unwrap(), sequence);
        assert!(recipe_panel(&r, None, false, None)
            .header
            .join(" ")
            .contains("Conditional timing"));
        let mut d = D::Backward;
        direction(&mut d, true, false);
        assert_eq!(d, D::SavedLedge);
        direction(&mut d, true, false);
        assert_eq!(d, D::AwayFromLedge);
        direction(&mut d, true, false);
        assert_eq!(d, D::Opponent);
    }

    #[test]
    fn shield_hold_selector_round_trips_without_losing_neighbor_actions() {
        let source = crate::sequence_config::Catalog::parse(
            "[bowser.drop.test]\nstart_delay=0\nactions=shield:20,jab:4",
        )
        .unwrap()
        .entries()[0]
            .sequence;
        let key = (4, 0);
        let mut e = RecipeEditor::default();
        e.ensure(key, source);
        e.cursor = fields(&e.recipes[&key])
            .iter()
            .position(|f| matches!(f, Field::ShieldHold(0)))
            .unwrap();
        assert!(e.adjust(key, true));
        let compiled = e.compiled(key).unwrap();
        assert!(matches!(
            compiled.actions[0],
            Action::Input {
                buttons: MAPPER_ZR,
                frames: 0,
                ..
            }
        ));
        assert_eq!(compiled.actions[1], source.actions[1]);
        assert!(e.adjust(key, false));
        assert!(matches!(
            e.compiled(key).unwrap().actions[0],
            Action::Input {
                buttons: MAPPER_ZR,
                frames: 4,
                ..
            }
        ));
    }

    #[test]
    fn imported_opponent_input_keeps_target_axes_and_duration_editable() {
        let source = crate::sequence_config::Catalog::parse(
            "[bowser.dpad_right.test]\nstart_delay=0\nactions=opponent_input:b:-1:0:1\n",
        )
        .unwrap()
        .entries()[0]
            .sequence;
        let key = (0, 0);
        let mut editor = RecipeEditor::default();
        editor.ensure(key, source);
        assert!(editor.lines(key).join(" ").contains("(opponent)"));
        editor.cursor = fields(&editor.recipes[&key])
            .iter()
            .position(|f| matches!(f, Field::Duration(0)))
            .unwrap();
        assert!(editor.adjust(key, true));
        assert!(matches!(
            editor.compiled(key).unwrap().actions[0],
            Action::OpponentInput { frames: 2, .. }
        ));
        editor.cursor = fields(&editor.recipes[&key])
            .iter()
            .position(|f| matches!(f, Field::X(0)))
            .unwrap();
        assert!(editor.adjust(key, true));
        assert!(
            matches!(editor.compiled(key).unwrap().actions[0],Action::OpponentInput{stick:[x,0.],..} if x > -1.)
        );
    }

    #[test]
    fn assist_ground_only_switch_preserves_other_recipe_fields() {
        let source=crate::sequence_config::Catalog::parse("[bowser.roll.test]\nstart_delay=0\nreaction=roll_vulnerable:grounded:0:24:10:any:1\nactions=up_b:3\n").unwrap().entries()[0].sequence;
        let key = (0, 0);
        let mut editor = RecipeEditor::default();
        editor.ensure(key, source);
        editor.cursor = fields(&editor.recipes[&key])
            .iter()
            .position(|f| matches!(f, Field::GroundOnly))
            .unwrap();

        assert!(editor.adjust(key, true));
        let compiled = editor.compiled(key).unwrap();
        assert!(compiled.ground_only);
        assert_eq!(compiled.actions(), source.actions());
        assert_eq!(compiled.reaction, source.reaction);
        assert!(editor.lines(key).join(" ").contains("Stop if airborne: ON"));
        assert!(editor.adjust(key, false));
        assert!(!editor.compiled(key).unwrap().ground_only);
    }

    #[test]
    fn both_controller_sticks_and_references_are_editable_and_keep_saved_payload() {
        let source = crate::sequence_config::Catalog::parse(
            "[kirby.x.t]\nstart_delay=0\nactions=controller_input:a:0:0:0:0:19\n",
        )
        .unwrap()
        .entries()[0]
            .sequence;
        let key = (5, 0);
        let mut e = RecipeEditor::default();
        e.ensure(key, source);
        for target in [
            Field::Reference(0),
            Field::CReference(0),
            Field::X(0),
            Field::Y(0),
            Field::CX(0),
            Field::CY(0),
        ] {
            e.cursor = fields(&e.recipes[&key])
                .iter()
                .position(|f| same_field(*f, target))
                .unwrap();
            assert!(e.adjust(key, true));
        }
        assert!(matches!(
            e.compiled(key).unwrap().actions[0],
            Action::Controller {
                stick: [0.01, 0.01],
                cstick: [0.01, 0.01],
                frames: 19,
                reference: IR::Facing,
                cstick_reference: IR::Facing,
                ..
            }
        ));
    }

    #[test]
    fn invalid_draft_never_falls_back_to_the_old_recipe_snapshot() {
        let catalog = crate::sequence_config::Catalog::parse(
            "[kirby.neutral.first]\nstart_delay=0\nactions=b:4\n",
        )
        .unwrap();
        let mut profiles = crate::fighter_profiles::FighterProfiles::default();
        profiles.add_sequence_choices(&catalog).unwrap();
        let p = profiles.profile(6).unwrap().menu_options;
        let mut m = MenuSelection::new(&p);
        let key = (0, 0);
        let mut editor = RecipeEditor::default();
        let mut sequence = RecipeEditor::custom();
        sequence.len = 32;
        sequence.actions.fill(Action::Input {
            buttons: MAPPER_B,
            stick: [0., 1.],
            frames: 4,
            relative: false,
        });
        editor.ensure(key, sequence);
        editor.recipes.get_mut(&key).unwrap().before[0] = 1;
        assert!(editor.compiled(key).is_none());
        m.editor = Some(Box::new(editor));
        assert!(m.recipe_snapshot(&p, PresetSlot::Neutral, 0).is_none());
        assert!(
            m.choice(&p, PresetSlot::Neutral, 0).is_some(),
            "an older saved recipe exists but must not be saved instead"
        );
    }
    #[test]
    fn rapid_axis_edits_across_drafts_compile_and_serialize_the_exact_snapshot() {
        let mut e = RecipeEditor::default();
        let mut source = RecipeEditor::custom();
        source.len = 1;
        source.actions[0] = Action::Controller {
            buttons: MAPPER_B,
            stick: [0.; 2],
            cstick: [0.; 2],
            frames: 255,
            reference: IR::Screen,
            cstick_reference: IR::Screen,
        };
        for round in 0..12 {
            let key = (5, round % 3);
            e.ensure(key, source);
            for target in [Field::X(0), Field::Y(0), Field::CX(0), Field::CY(0)] {
                e.cursor = fields(&e.recipes[&key])
                    .iter()
                    .position(|f| same_field(*f, target))
                    .unwrap();
                for _ in 0..120 {
                    e.adjust(key, round % 2 == 0);
                }
            }
            let snapshot = e.compiled(key).unwrap();
            let text = crate::sequence_config::persist::update(
                "[kirby.x.test]\nstart_delay=0\nactions=b:4\n",
                6,
                PresetSlot::X,
                "test",
                snapshot,
                false,
            )
            .unwrap();
            assert_eq!(
                crate::sequence_config::Catalog::parse(&text)
                    .unwrap()
                    .entries()[0]
                    .sequence,
                snapshot
            );
            assert!(e.recipes.len() <= 3);
        }
    }
}

#[cfg(test)]
mod ledge_position_branch_editor_tests {
    use super::*;
    #[test]
    fn positioning_fields_edit_and_save_without_converting_to_an_imported_action() {
        let c = crate::sequence_config::Catalog::parse(
            "[bowser.attack.test]\nstart_delay=0\nactions=run_to_ledge:12:1:1:6:180\n",
        )
        .unwrap();
        let source = c.entries()[0].sequence;
        let key = (0, 0);
        let mut e = RecipeEditor::default();
        e.ensure(key, source);
        assert!(e.recipes[&key].kinds[0] < PRESETS.len());
        for (field, up) in [
            (Field::Distance(0), true),
            (Field::Strength(0), false),
            (Field::Tolerance(0), true),
            (Field::Brake(0), true),
            (Field::PositionTimeout(0), true),
        ] {
            e.cursor = fields(&e.recipes[&key])
                .iter()
                .position(|f| same_field(*f, field))
                .unwrap();
            assert!(e.adjust(key, up));
        }
        let edited = e.compiled(key).unwrap();
        assert_eq!(
            edited.actions()[0],
            Action::LedgePosition {
                offset: 13.,
                strength: 0.99,
                tolerance: 1.1,
                brake: 6.5,
                frames: 181
            }
        );
        let saved = crate::sequence_config::persist::update(
            "",
            15,
            crate::fighter_profiles::presets::PresetSlot::Attack,
            "test",
            edited,
            true,
        )
        .unwrap();
        assert_eq!(
            crate::sequence_config::Catalog::parse(&saved)
                .unwrap()
                .entries()[0]
                .sequence,
            edited
        );
        let panel = e.panel(key, true).body.join(" ");
        for label in [
            "Inward distance from ledge",
            "Arrival tolerance",
            "Braking distance",
            "Movement timeout",
        ] {
            assert!(panel.contains(label), "{label}");
        }
    }
    #[test]
    fn opponent_branch_controls_are_zero_time_and_keep_single_branch_timing_label() {
        let cat=crate::sequence_config::Catalog::parse("[bowser.attack.test]\nstart_delay=0\nactions=if_facing_opponent,input:a:0:0:1,else_facing,input:b:0:0:1,end_facing\n").unwrap();
        let key = (0, 0);
        let mut e = RecipeEditor::default();
        e.ensure(key, cat.entries()[0].sequence);
        assert!([0, 2, 4]
            .into_iter()
            .all(|i| e.recipes[&key].kinds[i] < PRESETS.len()));
        for a in [
            Action::IfFacingOpponent { away: false },
            Action::ElseFacing,
            Action::EndFacing,
        ] {
            assert_eq!(scheduled_frames(a), Some(0));
        }
        assert!(e
            .panel(key, true)
            .header
            .join(" ")
            .contains("only the matching facing block runs"));
        assert_eq!(
            e.compiled(key).unwrap().actions(),
            cat.entries()[0].sequence.actions()
        );
    }
}
