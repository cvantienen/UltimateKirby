//! Lazy, in-memory recipe edits. No parsing, I/O, or allocation in execution.
use super::cycle;
use crate::native_input::*;
use crate::sequence_config::{
    Action, Condition, Direction as D, InputReference as IR, RangeCheck, Sequence, MAX_ACTIONS,
};
use std::collections::BTreeMap;
use std::fmt::Write;
#[path = "editor/commands.rs"]
mod commands;
#[path = "editor/presentation.rs"]
mod presentation;
use presentation::{field_slot, field_value, fields, recipe_panel, Field};
#[cfg(test)]
use presentation::{schedule_summary, scheduled_frames};

/// Technical: Builds the lookup key for the observed state.
///
/// Plain English: Creates the key used to find this state.
const fn key(buttons: u32, x: f32, y: f32, frames: u8) -> Action {
    Action::Input {
        buttons,
        stick: [x, y],
        frames,
        relative: true,
    }
}
/// Technical: Defines the `PRESETS` constant used by this module.
///
/// Plain English: Provides the PRESETS constant value.
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
/// Technical: Defines the `TAP_BUTTONS` constant used by this module.
///
/// Plain English: Provides the TAP BUTTONS constant value.
const TAP_BUTTONS: &[(u32, [f32; 2], &str)] = &[
    (MAPPER_A, [0., 0.], "A"),
    (MAPPER_B, [0., 0.], "B"),
    (MAPPER_A, [0.6, 0.], "Forward tilt"),
    (MAPPER_A, [0., 0.6], "Up tilt"),
    (MAPPER_A, [0., -0.6], "Down tilt"),
];
/// Technical: Defines the `DIRECTIONS` constant used by this module.
///
/// Plain English: Provides the DIRECTIONS constant value.
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

/// Technical: Returns an optional `preset_kind` value.
///
/// Plain English: Gets preset kind if it is available.
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

/// Technical: Determines whether the `same_preset_shape` condition holds.
///
/// Plain English: Checks whether same preset shape.
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

/// Technical: Defines the `Recipe` data structure.
///
/// Plain English: Holds the data for Recipe.
#[derive(Clone)]
struct Recipe {
    /// Technical: Stores the `button_recipe` value in `Recipe` as `bool`.
    ///
    /// Plain English: Holds the button recipe value used by Recipe.
    button_recipe: bool,
    /// Technical: Stores the `sequence` value in `Recipe` as `Sequence`.
    ///
    /// Plain English: Holds the sequence value used by Recipe.
    sequence: Sequence,
    /// Technical: Stores the `before` value in `Recipe` as `[u8; MAX_ACTIONS]`.
    ///
    /// Plain English: Stores the before collection for Recipe.
    before: [u8; MAX_ACTIONS],
    /// Technical: Stores the `kinds` value in `Recipe` as `[usize; MAX_ACTIONS]`.
    ///
    /// Plain English: Stores the kinds collection for Recipe.
    kinds: [usize; MAX_ACTIONS],
    /// Technical: Stores the `imported` value in `Recipe` as `Vec<Action>`.
    ///
    /// Plain English: Holds the imported value used by Recipe.
    imported: Vec<Action>,
    /// Technical: Stores the `edited` value in `Recipe` as `u32`.
    ///
    /// Plain English: Holds the edited value used by Recipe.
    edited: u32,
    /// Technical: Stores the `modified` value in `Recipe` as `bool`.
    ///
    /// Plain English: Holds the modified value used by Recipe.
    modified: bool,
}
impl Recipe {
    /// Technical: Constructs a new value with the type’s initial state.
    ///
    /// Plain English: Creates a new value.
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
        let mut recipe = Self {
            button_recipe: false,
            sequence,
            before,
            kinds: [0; MAX_ACTIONS],
            imported: Vec::new(),
            edited: 0,
            modified: false,
        };
        for (i, action) in sequence.actions().iter().enumerate() {
            recipe.kinds[i] = recipe.kind_for(*action);
        }
        recipe
    }
    /// Picker index for an action: its preset, else its (deduplicated) saved-input entry.
    /// Plain English: Picker index for an action: its preset, else its (deduplicated) saved-input entry.
    fn kind_for(&mut self, action: Action) -> usize {
        preset_kind(action).unwrap_or_else(|| {
            let index = self
                .imported
                .iter()
                .position(|saved| *saved == action)
                .unwrap_or_else(|| {
                    self.imported.push(action);
                    self.imported.len() - 1
                });
            PRESETS.len() + index
        })
    }
    /// Technical: Returns the `action_for` result.
    ///
    /// Plain English: Gets the action for result.
    fn action_for(&self, kind: usize) -> Action {
        match PRESETS.get(kind) {
            Some((_, preset)) => *preset,
            None => self.imported[kind - PRESETS.len()],
        }
    }
    /// Technical: Advances the state machine by one step.
    ///
    /// Plain English: Moves the state machine to its next step.
    fn step(&self, slot: usize) -> (Action, u8, usize) {
        (
            self.sequence.actions[slot],
            self.before[slot],
            self.kinds[slot],
        )
    }
    /// Technical: Sets the step.
    ///
    /// Plain English: Sets the step.
    fn set_step(&mut self, slot: usize, (action, delay, kind): (Action, u8, usize)) {
        self.sequence.actions[slot] = action;
        self.before[slot] = delay;
        self.kinds[slot] = kind;
    }
    /// A slot with neither an action nor a delay; the only kind that may be dropped for room.
    /// Plain English: A slot with neither an action nor a delay; the only kind that may be dropped for room.
    fn unused(&self, slot: usize) -> bool {
        self.sequence.actions[slot] == Action::Empty && self.before[slot] == 0
    }
    /// Insert before `at`, shifting later steps down. `reuse_last` lets an unused final
    /// slot make room when all slots are taken. False (and unchanged) when full.
    /// Plain English: Insert before at, shifting later steps down. reuselast lets an unused final slot make room when all slots are taken. False (and unchanged) when full.
    fn insert_step(&mut self, at: usize, step: (Action, u8, usize), reuse_last: bool) -> bool {
        let mut len = usize::from(self.sequence.len);
        if len == MAX_ACTIONS && reuse_last && self.unused(len - 1) {
            len -= 1;
        }
        if len == MAX_ACTIONS {
            return false;
        }
        for i in (at..len).rev() {
            self.set_step(i + 1, self.step(i));
        }
        self.set_step(at, step);
        self.sequence.len = (len + 1) as u8;
        true
    }
    /// Remove a step, shifting later ones up; at least one slot always remains.
    /// Plain English: Remove a step, shifting later ones up; at least one slot always remains.
    fn remove_step(&mut self, slot: usize) {
        let len = usize::from(self.sequence.len);
        for i in slot..len - 1 {
            self.set_step(i, self.step(i + 1));
        }
        self.set_step(len - 1, (Action::Empty, 0, 0));
        self.sequence.len = len.saturating_sub(1).max(1) as u8;
    }
    /// Technical: Returns an optional `compile` value.
    ///
    /// Plain English: Gets compile if it is available.
    fn compile(&self) -> Option<Sequence> {
        let mut result = Sequence {
            len: 0,
            actions: [Action::Empty; MAX_ACTIONS],
            ..self.sequence
        };
        for (i, action) in self.sequence.actions().iter().enumerate() {
            let delay = (self.before[i] > 0).then_some(Action::Wait(self.before[i]));
            let step = (*action != Action::Empty).then_some(*action);
            for a in delay.into_iter().chain(step) {
                if usize::from(result.len) == MAX_ACTIONS {
                    return None;
                }
                result.actions[usize::from(result.len)] = a;
                result.len += 1;
            }
        }
        Some(result)
    }
}
/// Technical: Returns the `action_name` result.
///
/// Plain English: Gets the action name result.
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
/// Technical: Defines the `INPUT_BUTTONS` constant used by this module.
///
/// Plain English: Provides the INPUT BUTTONS constant value.
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
/// Technical: Returns mutable access to the buttons value.
///
/// Plain English: Provides mutable access to buttons.
fn buttons_mut(a: &mut Action) -> Option<&mut u32> {
    match a {
        Action::Input { buttons, .. }
        | Action::OpponentInput { buttons, .. }
        | Action::Controller { buttons, .. } => Some(buttons),
        _ => None,
    }
}
/// Technical: Returns an optional `input_buttons` value.
///
/// Plain English: Gets input buttons if it is available.
fn input_buttons(mut a: Action) -> Option<u32> {
    buttons_mut(&mut a).map(|buttons| *buttons)
}
/// Button names joined by `+`; bits the picker cannot name stay visible as hex.
/// Plain English: Button names joined by +; bits the picker cannot name stay visible as hex.
fn buttons_text(mask: u32, none: &str) -> String {
    let mut text = String::new();
    let mut push = |part: std::fmt::Arguments| {
        if !text.is_empty() {
            text.push('+');
        }
        let _ = text.write_fmt(part);
    };
    let mut known = 0;
    for (bit, name) in INPUT_BUTTONS {
        known |= bit;
        if mask & bit != 0 {
            push(format_args!("{name}"));
        }
    }
    if mask & !known != 0 {
        push(format_args!("0x{:X}", mask & !known));
    }
    if text.is_empty() {
        none.into()
    } else {
        text
    }
}
/// Technical: Returns the `input_reference` result.
///
/// Plain English: Gets the input reference result.
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
/// Technical: Sets the input reference.
///
/// Plain English: Sets the input reference.
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
/// Technical: Returns the `describe_input` result.
///
/// Plain English: Gets the describe input result.
fn describe_input(buttons: u32, stick: [f32; 2], reference: &str, frames: u8) -> String {
    let mut text = format!(
        "{} | X:{:.0}% Y:{:.0}% ({reference}) | ",
        buttons_text(buttons, "Stick only"),
        stick[0] * 100.,
        stick[1] * 100.
    );
    if frames == 0 {
        text.push_str("until manual");
    } else {
        let _ = write!(text, "{frames}f");
    }
    text
}
/// Technical: Returns the `describe_action` result.
///
/// Plain English: Gets the describe action result.
fn describe_action(action: Action) -> String {
    match action {
        Action::Controller{buttons,stick,cstick,reference,cstick_reference,frames}=>format!("{} | Left {:.0}%/{:.0}% {} | Right {:.0}%/{:.0}% {} | {frames}f",buttons_text(buttons, "None (stick only)"),stick[0]*100.,stick[1]*100.,reference.token(),cstick[0]*100.,cstick[1]*100.,cstick_reference.token()),
        Action::LedgePosition{offset,strength,tolerance,brake,frames}=>format!("Run to {offset} units inward from ledge | {:.0}% | tolerance {tolerance} | brake {brake} | max {frames}f",strength*100.),
        Action::IfFacingOpponent{away}=>if away{"If back to opponent (instant)".into()}else{"If facing opponent (instant)".into()},
        Action::ElseFacing=>"Otherwise (instant facing branch)".into(),
        Action::ShieldUntilManual=>"Shield until manual takeover (saved input)".into(),
        Action::Range(RangeCheck::Ignore)=>"Skip range check (saved input)".into(),
        Action::OpponentInput { buttons, stick, frames } => describe_input(buttons, stick, "opponent", frames),
        Action::Input { buttons, stick, relative, frames } => describe_input(buttons, stick, if relative { "facing" } else { "screen" }, frames),
        other => format!("{other:?}"),
    }
}
/// The field under the cursor; the cursor may lag behind a field list that shrank.
/// Plain English: The field under the cursor; the cursor may lag behind a field list that shrank.
fn focus(fields: &[Field], cursor: usize) -> Field {
    fields[cursor.min(fields.len() - 1)]
}
/// Technical: Advances the state machine by one step.
///
/// Plain English: Moves the state machine to its next step.
fn step(v: f32, delta: f32, min: f32, max: f32) -> f32 {
    ((v + delta).clamp(min, max) * 100.).round() / 100.
}
/// Technical: Updates the `byte` state using the supplied inputs.
///
/// Plain English: Updates byte.
fn byte(v: &mut u8, up: bool, min: u8) {
    *v = if up {
        v.saturating_add(1)
    } else {
        v.saturating_sub(1).max(min)
    };
}
/// 16-bit frame counts share one ceiling.
/// Plain English: 16-bit frame counts share one ceiling.
fn wide(v: &mut u16, up: bool, min: u16) {
    *v = if up {
        v.saturating_add(1).min(3600)
    } else {
        v.saturating_sub(1).max(min)
    };
}
/// Technical: Updates the `direction` state using the supplied inputs.
///
/// Plain English: Updates direction.
fn direction(v: &mut D, up: bool, onstage: bool) {
    let mut i = DIRECTIONS.iter().position(|d| d == v).unwrap_or(0);
    for _ in 0..DIRECTIONS.len() {
        i = cycle(i, DIRECTIONS.len(), up);
        if !onstage || !Action::Face(DIRECTIONS[i]).needs_ledge() {
            *v = DIRECTIONS[i];
            break;
        }
    }
}
// Each editable parameter has exactly one accessor; editing, display and
// validation all read the same location so they cannot drift apart.
/// The action's main duration in frames and its lowest allowed value.
/// Plain English: The action's main duration in frames and its lowest allowed value.
fn frames_mut(a: &mut Action) -> Option<(&mut u8, u8)> {
    match a {
        Action::Input { frames, .. }
        | Action::OpponentInput { frames, .. }
        | Action::Controller { frames, .. }
        | Action::CStick { frames, .. }
        | Action::Chase { frames, .. }
        | Action::Approach { frames, .. }
        | Action::Drift { frames, .. }
        | Action::ShieldUntilBlock(frames) => Some((frames, 1)),
        Action::Wait(frames) | Action::Check { wait: frames, .. } => Some((frames, 0)),
        _ => None,
    }
}
/// Technical: Returns the configured frame count.
///
/// Plain English: Gets the number of frames.
fn frames(mut a: Action) -> Option<u8> {
    frames_mut(&mut a).map(|(frames, _)| *frames)
}
/// Technical: Returns mutable access to the stick value.
///
/// Plain English: Provides mutable access to stick.
fn stick_mut(a: &mut Action) -> Option<&mut [f32; 2]> {
    match a {
        Action::Input { stick, .. }
        | Action::OpponentInput { stick, .. }
        | Action::Controller { stick, .. }
        | Action::CStick { stick, .. } => Some(stick),
        _ => None,
    }
}
/// Technical: Returns mutable access to the direction value.
///
/// Plain English: Provides mutable access to direction.
fn direction_mut(a: &mut Action) -> Option<&mut D> {
    match a {
        Action::Face(d)
        | Action::Walk { direction: d, .. }
        | Action::Run { direction: d, .. }
        | Action::Drift { direction: d, .. } => Some(d),
        _ => None,
    }
}
/// A bounded number edited in steps of `scale` per button press.
/// Plain English: A bounded number edited in steps of scale per button press.
struct Scalar<'a> {
    /// Technical: Stores the `value` value in `Scalar` as `&'a mut f32`.
    ///
    /// Plain English: Holds the value value used by Scalar.
    value: &'a mut f32,
    /// Technical: Stores the `min` value in `Scalar` as `f32`.
    ///
    /// Plain English: Holds the min value used by Scalar.
    min: f32,
    /// Technical: Stores the `max` value in `Scalar` as `f32`.
    ///
    /// Plain English: Holds the max value used by Scalar.
    max: f32,
    /// Technical: Stores the `scale` value in `Scalar` as `f32`.
    ///
    /// Plain English: Holds the scale value used by Scalar.
    scale: f32,
}
/// Technical: Returns mutable access to the scalar value.
///
/// Plain English: Provides mutable access to scalar.
fn scalar_mut(a: &mut Action, field: Field) -> Option<Scalar<'_>> {
    let (value, min, max, scale) = match (a, field) {
        (Action::Controller { cstick, .. }, Field::CX(_)) => (&mut cstick[0], -1., 1., 0.01),
        (Action::Controller { cstick, .. }, Field::CY(_)) => (&mut cstick[1], -1., 1., 0.01),
        (a, Field::X(_)) => (&mut stick_mut(a)?[0], -1., 1., 0.01),
        (a, Field::Y(_)) => (&mut stick_mut(a)?[1], -1., 1., 0.01),
        (Action::LedgePosition { strength, .. }, Field::Strength(_)) => (strength, 0.3, 1., 0.01),
        (Action::Chase { strength, .. } | Action::Drift { strength, .. }, Field::Strength(_)) => {
            (strength, 0., 1., 0.01)
        }
        (
            Action::Approach {
                strength, airborne, ..
            },
            Field::Strength(_),
        ) => (strength, 0.1, if *airborne { 1. } else { 0.95 }, 0.01),
        (Action::Walk { strength, .. }, Field::Strength(_)) => (strength, 0.1, 0.95, 0.01),
        (Action::LedgePosition { tolerance, .. }, Field::Tolerance(_)) => (tolerance, 0., 20., 0.1),
        (Action::LedgePosition { brake, .. }, Field::Brake(_)) => (brake, 0., 100., 0.5),
        (Action::LedgePosition { offset, .. }, Field::Distance(_)) => (offset, 0., 200., 1.),
        (
            Action::Walk { units: value, .. }
            | Action::Run { units: value, .. }
            | Action::Check { value, .. },
            Field::Distance(_),
        ) => (value, 0., 100., 1.),
        (Action::Range(RangeCheck::Verify(value)), Field::Distance(_)) => (value, 0., 60., 1.),
        (Action::Chase { gap, .. } | Action::Approach { gap, .. }, Field::Gap(_)) => {
            (gap, 0., 60., 1.)
        }
        _ => return None,
    };
    Some(Scalar {
        value,
        min,
        max,
        scale,
    })
}
/// Technical: Returns an optional `scalar` value.
///
/// Plain English: Gets scalar if it is available.
fn scalar(mut a: Action, field: Field) -> Option<f32> {
    scalar_mut(&mut a, field).map(|s| *s.value)
}
/// Technical: Updates the `cycle_tap_input` state using the supplied inputs.
///
/// Plain English: Updates cycle tap input.
fn cycle_tap_input(action: &mut Action, up: bool) {
    let Action::Tap {
        buttons,
        stick,
        hold,
        ..
    } = action
    else {
        return;
    };
    let current = TAP_BUTTONS
        .iter()
        .position(|(button, axes, _)| button == buttons && axes == stick);
    let index = match current {
        Some(i) => cycle(i, TAP_BUTTONS.len(), up),
        None if up => 0,
        None => TAP_BUTTONS.len() - 1,
    };
    (*buttons, *stick, _) = TAP_BUTTONS[index];
    *hold = 1;
}
/// Technical: Updates the `adjust_action` state using the supplied inputs.
///
/// Plain English: Updates adjust action.
fn adjust_action(action: &mut Action, field: Field, up: bool, onstage: bool) {
    let delta = if up { 1. } else { -1. };
    match (&mut *action, field) {
        (_, Field::TapButtons(_)) => cycle_tap_input(action, up),
        (Action::Tap { gap, .. }, Field::TapGap(_)) => byte(gap, up, 1),
        (Action::Tap { frames, .. }, Field::Duration(_))
        | (Action::LedgePosition { frames, .. }, Field::PositionTimeout(_)) => wide(frames, up, 1),
        (_, Field::Duration(_)) => {
            if let Some((frames, min)) = frames_mut(action) {
                byte(frames, up, min);
            }
        }
        (Action::Walk { min_gap, .. }, Field::Gap(_)) => {
            let old = min_gap.unwrap_or(0.);
            *min_gap = if !up && old == 0. {
                None
            } else {
                Some(step(old, delta, 0., 60.))
            };
        }
        (_, Field::Direction(_)) => {
            if let Some(d) = direction_mut(action) {
                direction(d, up, onstage);
            }
            if let Action::Walk {
                direction,
                min_gap,
                ..
            } = action
            {
                if *direction != D::Opponent {
                    *min_gap = None;
                }
            }
        }
        _ => {
            if let Some(Scalar {
                value,
                min,
                max,
                scale,
            }) = scalar_mut(action, field)
            {
                *value = step(*value, delta * scale, min, max);
            }
        }
    }
}
/// Technical: Defines the `CopySource` data structure.
///
/// Plain English: Holds the data for CopySource.
pub(super) struct CopySource {
    /// Technical: Stores the `slot` value in `CopySource` as `super::PresetSlot`.
    ///
    /// Plain English: Holds the slot value used by CopySource.
    pub slot: super::PresetSlot,
    /// Technical: Stores the `index` value in `CopySource` as `usize`.
    ///
    /// Plain English: Holds the index value used by CopySource.
    pub index: usize,
    /// Technical: Stores the `name` value in `CopySource` as `String`.
    ///
    /// Plain English: Holds the name value used by CopySource.
    pub name: String,
}

/// Technical: Updates the `adjust_reference` state using the supplied inputs.
///
/// Plain English: Updates adjust reference.
fn adjust_reference(recipe: &mut Recipe, index: usize, right: bool, up: bool) {
    let old = input_reference(recipe.sequence.actions[index], right);
    let current = IR::ALL.iter().position(|value| *value == old).unwrap_or(0);
    let next = cycle(current, IR::ALL.len(), up);
    set_input_reference(&mut recipe.sequence.actions[index], IR::ALL[next], right);
}

/// Technical: Updates the `adjust_action_kind` state using the supplied inputs.
///
/// Plain English: Updates adjust action kind.
fn adjust_action_kind(recipe: &mut Recipe, index: usize, up: bool) {
    let count = PRESETS.len() + recipe.imported.len();
    recipe.kinds[index] = cycle(recipe.kinds[index], count, up);
    recipe.sequence.actions[index] = recipe.action_for(recipe.kinds[index]);
}

/// Technical: Toggles the shield hold.
///
/// Plain English: Toggles the shield hold.
fn toggle_shield_hold(recipe: &mut Recipe, index: usize) {
    match &mut recipe.sequence.actions[index] {
        Action::Input { frames, .. } => *frames = if *frames == 0 { 4 } else { 0 },
        Action::ShieldUntilManual => {
            let shield = key(MAPPER_ZR, 0., 0., 4);
            recipe.sequence.actions[index] = shield;
            recipe.kinds[index] = recipe.kind_for(shield);
        }
        _ => {}
    }
}

/// Technical: Updates the `adjust_slots` state using the supplied inputs.
///
/// Plain English: Updates adjust slots.
fn adjust_slots(recipe: &mut Recipe, up: bool) {
    let len = usize::from(recipe.sequence.len);
    if up && len < MAX_ACTIONS {
        recipe.sequence.len += 1;
    } else if !up && len > 1 && recipe.unused(len - 1) {
        recipe.sequence.len -= 1;
    }
}

/// Technical: Determines whether the `adjust_field` condition holds.
///
/// Plain English: Checks whether adjust field.
fn adjust_field(recipe: &mut Recipe, field: Field, up: bool, onstage: bool) -> bool {
    match field {
        Field::Reaction(0) => {
            let trigger = recipe.sequence.reaction.trigger;
            let options = trigger.events();
            let index = options
                .iter()
                .position(|value| *value == trigger)
                .unwrap_or(0);
            recipe.sequence.reaction.trigger = options[cycle(index, options.len(), up)];
        }
        Field::Reaction(index) => recipe.sequence.reaction.adjust(index, up),
        Field::WhileHeld => recipe.sequence.while_held = !recipe.sequence.while_held,
        Field::GroundOnly => recipe.sequence.ground_only = !recipe.sequence.ground_only,
        Field::Buttons(_) => return false,
        Field::Reference(index) => adjust_reference(recipe, index, false, up),
        Field::CReference(index) => adjust_reference(recipe, index, true, up),
        Field::Start => wide(&mut recipe.sequence.start_delay, up, 0),
        Field::Slots => adjust_slots(recipe, up),
        Field::Before(index) => byte(&mut recipe.before[index], up, 0),
        Field::Kind(index) => adjust_action_kind(recipe, index, up),
        Field::ShieldHold(index) => toggle_shield_hold(recipe, index),
        other => {
            if let Some(index) = field_slot(other) {
                adjust_action(&mut recipe.sequence.actions[index], other, up, onstage);
            }
        }
    }
    true
}

/// Technical: Defines the `RecipeEditor` data structure.
///
/// Plain English: Holds the data for RecipeEditor.
#[derive(Default)]
pub(super) struct RecipeEditor {
    /// Technical: Stores the `modal` value in `RecipeEditor` as `Option<commands::Modal>`.
    ///
    /// Plain English: Optionally stores the modal for RecipeEditor; it may be absent.
    modal: Option<commands::Modal>,
    /// Technical: Stores the `return_modal` value in `RecipeEditor` as `Option<commands::Modal>`.
    ///
    /// Plain English: Optionally stores the return modal for RecipeEditor; it may be absent.
    return_modal: Option<commands::Modal>,
    /// Technical: Stores the `step_clipboard` value in `RecipeEditor` as `Option<(Action`.
    ///
    /// Plain English: Optionally stores the step clipboard for RecipeEditor; it may be absent.
    step_clipboard: Option<(Action, u8)>,
    /// Technical: Stores the `manage_requested` value in `RecipeEditor` as `bool`.
    ///
    /// Plain English: Holds the manage requested value used by RecipeEditor.
    manage_requested: bool,
    /// Technical: Stores the `copy_sources` value in `RecipeEditor` as `Vec<CopySource>`.
    ///
    /// Plain English: Holds the copy sources value used by RecipeEditor.
    copy_sources: Vec<CopySource>,
    /// Technical: Stores the `copy_templates` value in `RecipeEditor` as `Vec<String>`.
    ///
    /// Plain English: Holds the copy templates value used by RecipeEditor.
    copy_templates: Vec<String>,
    /// Technical: Stores the `template_apply` value in `RecipeEditor` as `Option<usize>`.
    ///
    /// Plain English: Optionally stores the template apply for RecipeEditor; it may be absent.
    template_apply: Option<usize>,
    /// Technical: Stores the `copy_requested` value in `RecipeEditor` as `bool`.
    ///
    /// Plain English: Holds the copy requested value used by RecipeEditor.
    copy_requested: bool,
    /// Technical: Stores the `copy_apply` value in `RecipeEditor` as `Option<(super::PresetSlot`.
    ///
    /// Plain English: Optionally stores the copy apply for RecipeEditor; it may be absent.
    copy_apply: Option<(super::PresetSlot, usize)>,
    /// Technical: Stores the `command_held` value in `RecipeEditor` as `u32`.
    ///
    /// Plain English: Holds the command held value used by RecipeEditor.
    command_held: u32,
    /// Technical: Stores the `command_ticks` value in `RecipeEditor` as `u16`.
    ///
    /// Plain English: Holds the command ticks value used by RecipeEditor.
    command_ticks: u16,
    /// Technical: Stores the `undo` value in `RecipeEditor` as `Option<Box<((usize`.
    ///
    /// Plain English: Optionally stores the undo for RecipeEditor; it may be absent.
    undo: Option<Box<((usize, usize), Recipe)>>,
    /// Technical: Stores the `recipes` value in `RecipeEditor` as `BTreeMap<(usize`.
    ///
    /// Plain English: Holds the recipes value used by RecipeEditor.
    recipes: BTreeMap<(usize, usize), Recipe>,
    /// Technical: Stores the `cursor` value in `RecipeEditor` as `usize`.
    ///
    /// Plain English: Holds the cursor value used by RecipeEditor.
    pub cursor: usize,
    /// Technical: Stores the `limited` value in `RecipeEditor` as `bool`.
    ///
    /// Plain English: Holds the limited value used by RecipeEditor.
    pub limited: bool,
    /// Technical: Stores the `last_edit` value in `RecipeEditor` as `Option<String>`.
    ///
    /// Plain English: Optionally stores the last edit for RecipeEditor; it may be absent.
    last_edit: Option<String>,
}
impl RecipeEditor {
    /// Technical: Returns the `retained` result.
    ///
    /// Plain English: Gets the retained result.
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
    /// Technical: Returns the `preview_panel` result.
    ///
    /// Plain English: Gets the preview panel result.
    pub fn preview_panel(sequence: Sequence) -> super::RecipePanel {
        let recipe = Recipe::new(sequence);
        recipe_panel(&recipe, &fields(&recipe), None, false, None)
    }
    /// Technical: Determines whether the `has` condition holds.
    ///
    /// Plain English: Checks whether has.
    pub fn has(&self, key: (usize, usize)) -> bool {
        self.recipes.contains_key(&key)
    }
    /// Technical: Returns the `custom` result.
    ///
    /// Plain English: Gets the custom result.
    pub fn custom() -> Sequence {
        Sequence {
            len: 6,
            ..Sequence::default()
        }
    }
    /// Technical: Updates the `ensure` state using the supplied inputs.
    ///
    /// Plain English: Updates ensure.
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
    /// Technical: Determines whether the `edited` condition holds.
    ///
    /// Plain English: Checks whether edited.
    pub fn edited(&self, key: (usize, usize)) -> bool {
        self.recipes.get(&key).is_some_and(|r| r.modified)
    }
    /// Technical: Marks the saved.
    ///
    /// Plain English: Marks the saved.
    pub fn mark_saved(&mut self, key: (usize, usize)) {
        if let Some(r) = self.recipes.get_mut(&key) {
            r.modified = false;
            r.edited = 0;
        }
    }
    /// Technical: Updates the `release_tools` state using the supplied inputs.
    ///
    /// Plain English: Updates release tools.
    pub fn release_tools(&mut self) {
        self.close_commands();
        self.undo = None;
        self.step_clipboard = None;
        self.last_edit = None;
    }
    /// Technical: Retains the recipes.
    ///
    /// Plain English: Retains the recipes.
    pub fn retain_recipes(
        &mut self,
        mut retain: impl FnMut((usize, usize), Option<Sequence>, bool) -> bool,
    ) {
        self.recipes
            .retain(|key, recipe| retain(*key, recipe.compile(), recipe.modified));
    }
    /// Technical: Checks whether empty.
    ///
    /// Plain English: Checks whether empty.
    pub fn is_empty(&self) -> bool {
        self.recipes.is_empty()
    }
    /// Technical: Updates the `discard` state using the supplied inputs.
    ///
    /// Plain English: Updates discard.
    pub fn discard(&mut self, key: (usize, usize)) {
        self.clear_copy();
        if self.undo.as_ref().is_some_and(|u| u.0 == key) {
            self.undo = None;
        }
        self.modal = None;
        self.return_modal = None;
        self.recipes.remove(&key);
    }
    /// Technical: Returns an optional `while_held` value.
    ///
    /// Plain English: Gets while held if it is available.
    pub fn while_held(&self, key: (usize, usize)) -> Option<bool> {
        self.recipes.get(&key).map(|r| r.sequence.while_held)
    }
    /// Technical: Returns an optional `compiled` value.
    ///
    /// Plain English: Gets compiled if it is available.
    pub fn compiled(&self, key: (usize, usize)) -> Option<Sequence> {
        self.recipes.get(&key).and_then(Recipe::compile)
    }
    /// Technical: Updates the `navigate` state using the supplied inputs.
    ///
    /// Plain English: Updates navigate.
    pub fn navigate(&mut self, key: (usize, usize), down: bool) {
        if let Some(r) = self.recipes.get(&key) {
            self.cursor = cycle(self.cursor, fields(r).len(), down);
        }
    }
    /// Technical: Determines whether the `repeatable` condition holds.
    ///
    /// Plain English: Checks whether repeatable.
    pub fn repeatable(&self, key: (usize, usize)) -> bool {
        self.recipes.get(&key).is_some_and(|r| {
            !matches!(
                focus(&fields(r), self.cursor),
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
    /// Technical: Determines whether the `adjust` condition holds.
    ///
    /// Plain English: Checks whether adjust.
    pub fn adjust(&mut self, key: (usize, usize), up: bool) -> bool {
        let Some(r) = self.recipes.get_mut(&key) else {
            return false;
        };
        let mut next = r.clone();
        let f = focus(&fields(&next), self.cursor);
        let onstage = next.sequence.reaction.active();
        if !adjust_field(&mut next, f, up, onstage) {
            return false;
        }
        self.limited = next.compile().is_none();
        if self.limited {
            return false;
        }
        let changed =
            next.sequence != r.sequence || next.before != r.before || next.kinds != r.kinds;
        if changed {
            let mut summary = String::from("Last change: ");
            if let Some(slot) = field_slot(f) {
                let _ = write!(summary, "slot {}: ", slot + 1);
            }
            let _ = write!(
                summary,
                "{} -> {}",
                field_value(r, f),
                field_value(&next, f)
            );
            self.last_edit = Some(summary);
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
    /// Technical: Determines whether the `reference_visible` condition holds.
    ///
    /// Plain English: Checks whether reference visible.
    pub(super) fn reference_visible(&self) -> bool {
        matches!(self.modal, None | Some(commands::Modal::Picker { .. }))
    }
    /// Technical: Returns an optional `reference_label` value.
    ///
    /// Plain English: Gets reference label if it is available.
    pub(super) fn reference_label(&self, key: (usize, usize)) -> Option<&'static str> {
        let r = self.recipes.get(&key)?;
        if let Some(commands::Modal::Picker { group, item, .. }) = self.modal {
            let kind = *commands::candidates(r, group).get(item)?;
            return PRESETS.get(kind).map(|(label, _)| *label);
        }
        if self.modal.is_some() {
            return None;
        }
        let slot = field_slot(focus(&fields(r), self.cursor))?;
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
    /// Technical: Returns the `panel` result.
    ///
    /// Plain English: Gets the panel result.
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
            &fs,
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
            panel
                .footer
                .push("A on move: Categories | ZL+ZR: Tools | ZL/ZR: Jump step".into());
        }
        panel
    }
    /// Technical: Returns the `lines` result.
    ///
    /// Plain English: Gets the lines result.
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

    /// Technical: Regression test for `edited_actions_keep_their_matching_preset_kind`.
    ///
    /// Plain English: Checks that edited actions keep their matching preset kind.
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

    /// Technical: Returns the `presets` result.
    ///
    /// Plain English: Gets the presets result.
    fn presets() -> crate::fighter_profiles::presets::MenuPresets {
        let file = crate::sequence_config::Catalog::parse(
            "[kirby.neutral.first]\nstart_delay=0\nactions=short_hop,wait:2,nair",
        )
        .unwrap();
        let mut p = FighterProfiles::default();
        p.add_sequence_choices(&file).unwrap();
        p.profile(6).unwrap().menu_options
    }
    /// Technical: Updates the `tap` state using the supplied inputs.
    ///
    /// Plain English: Updates tap.
    fn tap(
        menu: &mut MenuSelection,
        p: &crate::fighter_profiles::presets::MenuPresets,
        button: u32,
    ) {
        menu.input(0, p);
        menu.input(button, p);
    }
    /// Technical: Copies the menu can template a button into ledge without changing source or name.
    ///
    /// Plain English: Checks that copy menu can template a button into ledge without changing source or name.
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
    /// Technical: Regression test for `editor_tools_consume_controls_cancel_safely_and_do_not_rebuild_while_idle`.
    ///
    /// Plain English: Checks that editor tools consume controls cancel safely and do not rebuild while idle.
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

    /// Technical: Regression test for `modified_save_explains_rejection_in_editor_and_tools_without_losing_edits`.
    ///
    /// Plain English: Checks that modified save explains rejection in editor and tools without losing edits.
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
    /// Technical: Regression test for `button_activation_is_editable_and_not_exposed_for_ledge_recipes`.
    ///
    /// Plain English: Checks that button activation is editable and not exposed for ledge recipes.
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
    /// Technical: Regression test for `repeated_tap_picker_has_only_buttons_and_tilts_with_gap_and_total_fields`.
    ///
    /// Plain English: Checks that repeated tap picker has only buttons and tilts with gap and total fields.
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
                .position(|f| *f == field)
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
    /// Technical: Regression test for `condition_fields_roundtrip_and_slot_growth_cannot_erase_actions`.
    ///
    /// Plain English: Checks that condition fields roundtrip and slot growth cannot erase actions.
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
    /// Technical: Regression test for `schedule_shows_input_frames_with_one_based_recognition_and_preserves_recipe`.
    ///
    /// Plain English: Checks that schedule shows input frames with one based recognition and preserves recipe.
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
        let panel = recipe_panel(&recipe, &fields(&recipe), None, false, None);
        let text = panel.header.join("\n") + "\n" + &panel.body.join("\n");
        assert!(text.contains("Input schedule: 12f | done F13"));
        assert!(text.contains("F3"));
        assert!(text.contains("F7-10"));
        assert!(text.contains("F11-12"));
        assert_eq!(recipe.compile().unwrap().start_delay, source.start_delay);
        assert_eq!(recipe.compile().unwrap().actions(), source.actions());
        let mut limited = recipe;
        limited.sequence.timeout = 6;
        let header = recipe_panel(&limited, &fields(&limited), None, false, None)
            .header
            .join(" ");
        assert!(header.contains("timeout 6f"));
        assert!(!header.contains("done F"));
    }
    /// Technical: Regression test for `schedule_never_claims_exact_frames_after_a_spatial_or_block_check`.
    ///
    /// Plain English: Checks that schedule never claims exact frames after a spatial or block check.
    #[test]
    fn schedule_never_claims_exact_frames_after_a_spatial_or_block_check() {
        let source = crate::sequence_config::Catalog::parse(
            "[yoshi.roll.t]\nstart_delay=0\nactions=walk:inward:4:50%,face:opponent,wait:2,grab\n",
        )
        .unwrap()
        .entries()[0]
            .sequence;
        let recipe = Recipe::new(source);
        let panel = recipe_panel(&recipe, &fields(&recipe), None, false, None);
        assert!(panel.header.join(" ").contains("variable checks"));
        let text = panel.body.join("\n");
        assert!(text.contains("F1 check"));
        assert!(text.contains("after checks"));
        assert!(!panel.header.join(" ").contains("done F"));
    }
    /// Technical: Regression test for `imported_input_is_descriptive_and_can_be_reselected_after_cycling`.
    ///
    /// Plain English: Checks that imported input is descriptive and can be reselected after cycling.
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
    /// Technical: Regression test for `conditional_range_choices_keep_distinct_names_and_controls`.
    ///
    /// Plain English: Checks that conditional range choices keep distinct names and controls.
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
    /// Technical: Regression test for `button_hold_duration_changes_only_the_input_hold`.
    ///
    /// Plain English: Checks that button hold duration changes only the input hold.
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
    /// Technical: Regression test for `full_sequence_panel_retains_all_moves_settings_and_exact_focus`.
    ///
    /// Plain English: Checks that full sequence panel retains all moves settings and exact focus.
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
    /// Technical: Regression test for `numbered_slot_overview_keeps_hidden_edits_visible_and_marks_saved_state`.
    ///
    /// Plain English: Checks that numbered slot overview keeps hidden edits visible and marks saved state.
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
    /// Technical: Regression test for `training_reference_tracks_focused_move_and_picker_without_changing_recipe`.
    ///
    /// Plain English: Checks that training reference tracks focused move and picker without changing recipe.
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

    /// Technical: Saves the wait consumes inputs without hidden edits or held save replay.
    ///
    /// Plain English: Checks that save wait consumes inputs without hidden edits or held save replay.
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

    /// Technical: Regression test for `menu_save_is_editor_only_once_per_press_and_cancels_preview`.
    ///
    /// Plain English: Checks that menu save is editor only once per press and cancels preview.
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
    /// Technical: Regression test for `menu_preview_button_is_one_shot_consumed_and_menu_changes_cancel`.
    ///
    /// Plain English: Checks that menu preview button is one shot consumed and menu changes cancel.
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
        assert_eq!(menu.input(MAPPER_X, &p) & MAPPER_X, 0);
    }
    /// Technical: Regression test for `recipe_editor_numeric_hold_accelerates_and_release_resets`.
    ///
    /// Plain English: Checks that recipe editor numeric hold accelerates and release resets.
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
    /// Technical: Regression test for `recipe_editor_file_waits_become_delays_without_losing_frames`.
    ///
    /// Plain English: Checks that recipe editor file waits become delays without losing frames.
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
    /// Technical: Regression test for `recipe_editor_changes_next_attempt_delay_without_touching_file_or_old_snapshot`.
    ///
    /// Plain English: Checks that recipe editor changes next attempt delay without touching file or old snapshot.
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
    /// Technical: Regression test for `recipe_editor_custom_slots_swap_actions_and_insert_exact_delays`.
    ///
    /// Plain English: Checks that recipe editor custom slots swap actions and insert exact delays.
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
    /// Technical: Regression test for `recipe_editor_movement_bounds_and_expansion_limit_keep_valid_snapshots`.
    ///
    /// Plain English: Checks that recipe editor movement bounds and expansion limit keep valid snapshots.
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
    /// Technical: Regression test for `menu_edited_a_hold_continues_after_jab_into_the_next_action`.
    ///
    /// Plain English: Checks that menu edited a hold continues after jab into the next action.
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
    /// Technical: Regression test for `assist_editor_tunes_roll_conditions_without_moving_rule_to_manual_choices`.
    ///
    /// Plain English: Checks that assist editor tunes roll conditions without moving rule to manual choices.
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
    /// Technical: Regression test for `facing_block_editor_preserves_marker_names_and_roundtrip`.
    ///
    /// Plain English: Checks that facing block editor preserves marker names and roundtrip.
    #[test]
    fn facing_block_editor_preserves_marker_names_and_roundtrip() {
        let c=crate::sequence_config::Catalog::parse("[yoshi.neutral.test]\nstart_delay=0\nactions=if_facing_ledge, a:4, end_facing, if_back_to_ledge, b:4, end_facing").unwrap();
        let sequence = c.entries()[0].sequence;
        let r = Recipe::new(sequence);
        assert_eq!(action_name(&r, 0), "If facing ledge");
        assert_eq!(action_name(&r, 3), "If back to ledge");
        assert_eq!(action_name(&r, 5), "End facing block");
        assert_eq!(r.compile().unwrap(), sequence);
        assert!(recipe_panel(&r, &fields(&r), None, false, None)
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

    /// Technical: Regression test for `shield_hold_selector_round_trips_without_losing_neighbor_actions`.
    ///
    /// Plain English: Checks that shield hold selector round trips without losing neighbor actions.
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

    /// Technical: Regression test for `imported_opponent_input_keeps_target_axes_and_duration_editable`.
    ///
    /// Plain English: Checks that imported opponent input keeps target axes and duration editable.
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

    /// Technical: Regression test for `assist_ground_only_switch_preserves_other_recipe_fields`.
    ///
    /// Plain English: Checks that assist ground only switch preserves other recipe fields.
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

    /// Technical: Regression test for `both_controller_sticks_and_references_are_editable_and_keep_saved_payload`.
    ///
    /// Plain English: Checks that both controller sticks and references are editable and keep saved payload.
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
                .position(|f| *f == target)
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

    /// Technical: Regression test for `invalid_draft_never_falls_back_to_the_old_recipe_snapshot`.
    ///
    /// Plain English: Checks that invalid draft never falls back to the old recipe snapshot.
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
    /// Technical: Regression test for `rapid_axis_edits_across_drafts_compile_and_serialize_the_exact_snapshot`.
    ///
    /// Plain English: Checks that rapid axis edits across drafts compile and serialize the exact snapshot.
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
                    .position(|f| *f == target)
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
    /// Technical: Regression test for `positioning_fields_edit_and_save_without_converting_to_an_imported_action`.
    ///
    /// Plain English: Checks that positioning fields edit and save without converting to an imported action.
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
                .position(|f| *f == field)
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
    /// Technical: Regression test for `opponent_branch_controls_are_zero_time_and_keep_single_branch_timing_label`.
    ///
    /// Plain English: Checks that opponent branch controls are zero time and keep single branch timing label.
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
