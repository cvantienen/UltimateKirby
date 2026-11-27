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

/// Builds the lookup key for the observed state.
/// Creates the key used to find this state.
const fn key(buttons: u32, x: f32, y: f32, frames: u8) -> Action {
    Action::Input {
        buttons,
        stick: [x, y],
        frames,
        relative: true,
    }
}
/// `PRESETS` is a configuration value used by the Smash runtime.
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
/// `TAP_BUTTONS` is a configuration value used by the Smash runtime.
const TAP_BUTTONS: &[(u32, [f32; 2], &str)] = &[
    (MAPPER_A, [0., 0.], "A"),
    (MAPPER_B, [0., 0.], "B"),
    (MAPPER_A, [0.6, 0.], "Forward tilt"),
    (MAPPER_A, [0., 0.6], "Up tilt"),
    (MAPPER_A, [0., -0.6], "Down tilt"),
];
/// `DIRECTIONS` is a configuration value used by the Smash runtime.
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

/// Returns an optional `preset_kind` value.
/// Gets preset kind if it is available.
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

/// Determines whether the `same_preset_shape` condition holds.
/// Checks whether same preset shape.
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

/// Runtime Smash data carried by `Recipe`.
#[derive(Clone)]
struct Recipe {
    /// Runtime button recipe value used by Recipe..
    button_recipe: bool,
    /// Runtime sequence value used by Recipe..
    sequence: Sequence,
    /// Runtime before value used by Recipe..
    before: [u8; MAX_ACTIONS],
    /// Runtime kinds value used by Recipe..
    kinds: [usize; MAX_ACTIONS],
    /// Runtime imported value used by Recipe..
    imported: Vec<Action>,
    /// Runtime edited value used by Recipe..
    edited: u32,
    /// Runtime modified value used by Recipe..
    modified: bool,
}
impl Recipe {
    /// Constructs a new value with the type’s initial state.
    fn new(mut sequence: Sequence) -> Self {
        // Folding waits across a branch marker can move a branch-local pause
        // into the other branch. Keep explicit waits if the recipe branches or
        // if expanding its delays would exceed the fixed action capacity.
        let original = sequence;
        let mut pending = 0usize;
        let mut expanded = 0usize;
        for action in original.actions() {
            match action {
                Action::Wait(frames) => pending += usize::from(*frames),
                _ => {
                    expanded += pending.div_ceil(255) + 1;
                    pending = 0;
                }
            }
        }
        expanded += pending.div_ceil(255);
        if expanded > MAX_ACTIONS || original.actions().iter().any(|a| matches!(a,
            Action::IfFacingLedge { .. } | Action::IfFacingOpponent { .. }
                | Action::ElseFacing | Action::EndFacing
        )) {
            sequence.len = sequence.len.max(6);
            let mut recipe = Self {
                button_recipe: false,
                sequence,
                before: [0; MAX_ACTIONS],
                kinds: [0; MAX_ACTIONS],
                imported: Vec::new(),
                edited: 0,
                modified: false,
            };
            for (i, action) in original.actions().iter().enumerate() {
                recipe.kinds[i] = recipe.kind_for(*action);
            }
            return recipe;
        }
        sequence.actions.fill(Action::Empty);
        let mut before = [0; MAX_ACTIONS];
        let mut len = 0;
        let mut pending = 0usize;
        for action in original.actions() {
            if let Action::Wait(frames) = action {
                pending += usize::from(*frames);
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
    /// Returns the `action_for` result.
    fn action_for(&self, kind: usize) -> Action {
        match PRESETS.get(kind) {
            Some((_, preset)) => *preset,
            None => self.imported[kind - PRESETS.len()],
        }
    }
    /// Advances the state machine by one step.
    fn step(&self, slot: usize) -> (Action, u8, usize) {
        (
            self.sequence.actions[slot],
            self.before[slot],
            self.kinds[slot],
        )
    }
    /// Sets the step.
    fn set_step(&mut self, slot: usize, (action, delay, kind): (Action, u8, usize)) {
        self.sequence.actions[slot] = action;
        self.before[slot] = delay;
        self.kinds[slot] = kind;
    }
    /// A slot with neither an action nor a delay; the only kind that may be dropped for room.
    fn unused(&self, slot: usize) -> bool {
        self.sequence.actions[slot] == Action::Empty && self.before[slot] == 0
    }
    /// Insert before `at`, shifting later steps down. `reuse_last` lets an unused final
    /// slot make room when all slots are taken. False (and unchanged) when full.
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
    fn remove_step(&mut self, slot: usize) {
        let len = usize::from(self.sequence.len);
        for i in slot..len - 1 {
            self.set_step(i, self.step(i + 1));
        }
        self.set_step(len - 1, (Action::Empty, 0, 0));
        self.sequence.len = len.saturating_sub(1).max(1) as u8;
    }
    /// Returns an optional `compile` value.
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
/// Returns the `action_name` result.
/// Gets the action name result.
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
/// `INPUT_BUTTONS` is a configuration value used by the Smash runtime.
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
/// Returns mutable access to the buttons value.
/// Provides mutable access to buttons.
fn buttons_mut(a: &mut Action) -> Option<&mut u32> {
    match a {
        Action::Input { buttons, .. }
        | Action::OpponentInput { buttons, .. }
        | Action::Controller { buttons, .. } => Some(buttons),
        _ => None,
    }
}
/// Returns an optional `input_buttons` value.
/// Gets input buttons if it is available.
fn input_buttons(mut a: Action) -> Option<u32> {
    buttons_mut(&mut a).map(|buttons| *buttons)
}
/// Button names joined by `+`; bits the picker cannot name stay visible as hex.
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
/// Returns the `input_reference` result.
/// Gets the input reference result.
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
/// Sets the input reference.
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
/// Returns the `describe_input` result.
/// Gets the describe input result.
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
/// Returns the `describe_action` result.
/// Gets the describe action result.
fn describe_action(action: Action) -> String {
    match action {
        Action::Controller {
            buttons,
            stick,
            cstick,
            reference,
            cstick_reference,
            frames,
        } => format!(
            "{} | Left {:.0}%/{:.0}% {} | Right {:.0}%/{:.0}% {} | {frames}f",
            buttons_text(buttons, "None (stick only)"),
            stick[0] * 100.,
            stick[1] * 100.,
            reference.token(),
            cstick[0] * 100.,
            cstick[1] * 100.,
            cstick_reference.token()
        ),
        Action::LedgePosition {
            offset,
            strength,
            tolerance,
            brake,
            frames,
        } => format!(
            "Run to {offset} units inward from ledge | {:.0}% | tolerance {tolerance} | brake {brake} | max {frames}f",
            strength * 100.
        ),
        Action::IfFacingOpponent { away } => {
            if away {
                "If back to opponent (instant)".into()
            } else {
                "If facing opponent (instant)".into()
            }
        }
        Action::ElseFacing => "Otherwise (instant facing branch)".into(),
        Action::ShieldUntilManual => "Shield until manual takeover (saved input)".into(),
        Action::Range(RangeCheck::Ignore) => "Skip range check (saved input)".into(),
        Action::OpponentInput {
            buttons,
            stick,
            frames,
        } => describe_input(buttons, stick, "opponent", frames),
        Action::Input {
            buttons,
            stick,
            relative,
            frames,
        } => describe_input(buttons, stick, if relative { "facing" } else { "screen" }, frames),
        other => format!("{other:?}"),
    }
}
/// The field under the cursor; the cursor may lag behind a field list that shrank.
fn focus(fields: &[Field], cursor: usize) -> Field {
    fields[cursor.min(fields.len() - 1)]
}
/// Advances the state machine by one step.
/// Moves the state machine to its next step.
fn step(v: f32, delta: f32, min: f32, max: f32) -> f32 {
    ((v + delta).clamp(min, max) * 100.).round() / 100.
}
/// Updates the `byte` state using the supplied inputs.
/// Updates byte.
fn byte(v: &mut u8, up: bool, min: u8) {
    *v = if up {
        v.saturating_add(1)
    } else {
        v.saturating_sub(1).max(min)
    };
}
/// 16-bit frame counts share one ceiling.
fn wide(v: &mut u16, up: bool, min: u16) {
    *v = if up {
        v.saturating_add(1).min(3600)
    } else {
        v.saturating_sub(1).max(min)
    };
}
/// Updates the `direction` state using the supplied inputs.
/// Updates direction.
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
/// Returns the configured frame count.
/// Gets the number of frames.
fn frames(mut a: Action) -> Option<u8> {
    frames_mut(&mut a).map(|(frames, _)| *frames)
}
/// Returns mutable access to the stick value.
/// Provides mutable access to stick.
fn stick_mut(a: &mut Action) -> Option<&mut [f32; 2]> {
    match a {
        Action::Input { stick, .. }
        | Action::OpponentInput { stick, .. }
        | Action::Controller { stick, .. }
        | Action::CStick { stick, .. } => Some(stick),
        _ => None,
    }
}
/// Returns mutable access to the direction value.
/// Provides mutable access to direction.
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
struct Scalar<'a> {
    /// Runtime value value used by Scalar..
    value: &'a mut f32,
    /// Runtime min value used by Scalar..
    min: f32,
    /// Runtime max value used by Scalar..
    max: f32,
    /// Runtime scale value used by Scalar..
    scale: f32,
}
/// Returns mutable access to the scalar value.
/// Provides mutable access to scalar.
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
/// Returns an optional `scalar` value.
/// Gets scalar if it is available.
fn scalar(mut a: Action, field: Field) -> Option<f32> {
    scalar_mut(&mut a, field).map(|s| *s.value)
}
/// Updates the `cycle_tap_input` state using the supplied inputs.
/// Updates cycle tap input.
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
/// Updates the `adjust_action` state using the supplied inputs.
/// Updates adjust action.
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
                direction, min_gap, ..
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
/// Runtime Smash data carried by `CopySource`.
pub(super) struct CopySource {
    /// Runtime slot value used by CopySource..
    pub slot: super::PresetSlot,
    /// Runtime index value used by CopySource..
    pub index: usize,
    /// player-facing move/fighter name.
    pub name: String,
}

/// Updates the `adjust_reference` state using the supplied inputs.
/// Updates adjust reference.
fn adjust_reference(recipe: &mut Recipe, index: usize, right: bool, up: bool) {
    let old = input_reference(recipe.sequence.actions[index], right);
    let current = IR::ALL.iter().position(|value| *value == old).unwrap_or(0);
    let next = cycle(current, IR::ALL.len(), up);
    set_input_reference(&mut recipe.sequence.actions[index], IR::ALL[next], right);
}

/// Updates the `adjust_action_kind` state using the supplied inputs.
/// Updates adjust action kind.
fn adjust_action_kind(recipe: &mut Recipe, index: usize, up: bool) {
    let count = PRESETS.len() + recipe.imported.len();
    let mut kind = recipe.kinds[index];
    for _ in 0..count {
        kind = cycle(kind, count, up);
        let action = recipe.action_for(kind);
        if !recipe.sequence.reaction.active() || !action.needs_ledge() {
            recipe.kinds[index] = kind;
            recipe.sequence.actions[index] = action;
            break;
        }
    }
}

/// Toggles the shield hold.
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

/// Updates the `adjust_slots` state using the supplied inputs.
/// Updates adjust slots.
fn adjust_slots(recipe: &mut Recipe, up: bool) {
    let len = usize::from(recipe.sequence.len);
    if up && len < MAX_ACTIONS {
        recipe.sequence.len += 1;
    } else if !up && len > 1 && recipe.unused(len - 1) {
        recipe.sequence.len -= 1;
    }
}

/// Determines whether the `adjust_field` condition holds.
/// Checks whether adjust field.
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

/// Runtime Smash data carried by `RecipeEditor`.
#[derive(Default)]
pub(super) struct RecipeEditor {
    /// Runtime modal value used by RecipeEditor..
    modal: Option<commands::Modal>,
    /// Runtime return modal value used by RecipeEditor..
    return_modal: Option<commands::Modal>,
    /// Runtime step clipboard value used by RecipeEditor..
    step_clipboard: Option<(Action, u8)>,
    /// Runtime manage requested value used by RecipeEditor..
    manage_requested: bool,
    /// Runtime copy sources value used by RecipeEditor..
    copy_sources: Vec<CopySource>,
    /// Runtime copy templates value used by RecipeEditor..
    copy_templates: Vec<String>,
    /// Runtime template apply value used by RecipeEditor..
    template_apply: Option<usize>,
    /// Runtime copy requested value used by RecipeEditor..
    copy_requested: bool,
    /// Runtime copy apply value used by RecipeEditor..
    copy_apply: Option<(super::PresetSlot, usize)>,
    /// D-pad auto-repeat state for the open command tool.
    command_hold: super::HoldRepeat,
    /// Runtime undo value used by RecipeEditor..
    undo: Option<Box<((usize, usize), Recipe)>>,
    /// Runtime recipes value used by RecipeEditor..
    recipes: BTreeMap<(usize, usize), Recipe>,
    /// Runtime cursor value used by RecipeEditor..
    pub cursor: usize,
    /// Runtime limited value used by RecipeEditor..
    pub limited: bool,
    /// Runtime last edit value used by RecipeEditor..
    last_edit: Option<String>,
}
impl RecipeEditor {
    /// Returns the `retained` result.
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
    /// Returns the `preview_panel` result.
    pub fn preview_panel(sequence: Sequence) -> super::RecipePanel {
        let recipe = Recipe::new(sequence);
        recipe_panel(&recipe, &fields(&recipe), None, false, None)
    }
    /// Determines whether the `has` condition holds.
    pub fn has(&self, key: (usize, usize)) -> bool {
        self.recipes.contains_key(&key)
    }
    /// Returns the `custom` result.
    pub fn custom() -> Sequence {
        Sequence {
            len: 6,
            ..Sequence::default()
        }
    }
    /// Updates the `ensure` state using the supplied inputs.
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
    /// Determines whether the `edited` condition holds.
    pub fn edited(&self, key: (usize, usize)) -> bool {
        self.recipes.get(&key).is_some_and(|r| r.modified)
    }
    pub fn has_unsaved_edits(&self) -> bool {
        self.recipes.values().any(|r| r.modified)
    }
    /// Marks the saved.
    pub fn mark_saved(&mut self, key: (usize, usize)) {
        if let Some(r) = self.recipes.get_mut(&key) {
            r.modified = false;
            r.edited = 0;
        }
    }
    /// Updates the `release_tools` state using the supplied inputs.
    pub fn release_tools(&mut self) {
        self.close_commands();
        self.undo = None;
        self.step_clipboard = None;
        self.last_edit = None;
    }
    /// Retains the recipes.
    pub fn retain_recipes(
        &mut self,
        mut retain: impl FnMut((usize, usize), Option<Sequence>, bool) -> bool,
    ) {
        self.recipes
            .retain(|key, recipe| retain(*key, recipe.compile(), recipe.modified));
    }
    /// Checks whether empty.
    pub fn is_empty(&self) -> bool {
        self.recipes.is_empty()
    }
    /// Updates the `discard` state using the supplied inputs.
    pub fn discard(&mut self, key: (usize, usize)) {
        self.clear_copy();
        if self.undo.as_ref().is_some_and(|u| u.0 == key) {
            self.undo = None;
        }
        self.modal = None;
        self.return_modal = None;
        self.recipes.remove(&key);
    }
    /// Returns an optional `while_held` value.
    pub fn while_held(&self, key: (usize, usize)) -> Option<bool> {
        self.recipes.get(&key).map(|r| r.sequence.while_held)
    }
    /// Returns an optional `compiled` value.
    pub fn compiled(&self, key: (usize, usize)) -> Option<Sequence> {
        self.recipes.get(&key).and_then(Recipe::compile)
    }
    /// Updates the `navigate` state using the supplied inputs.
    pub fn navigate(&mut self, key: (usize, usize), down: bool) {
        if let Some(r) = self.recipes.get(&key) {
            self.cursor = cycle(self.cursor, fields(r).len(), down);
        }
    }
    /// Determines whether the `repeatable` condition holds.
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
    /// Determines whether the `adjust` condition holds.
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
    /// Determines whether the `reference_visible` condition holds.
    pub(super) fn reference_visible(&self) -> bool {
        matches!(self.modal, None | Some(commands::Modal::Picker { .. }))
    }
    /// Returns an optional `reference_label` value.
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
    /// Returns the `panel` result.
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
    /// Returns the `lines` result.
    #[cfg(test)]
    pub fn lines(&self, key: (usize, usize)) -> Vec<String> {
        self.panel(key, true).into_lines()
    }
}

#[cfg(test)]
#[path = "../../tests/unit/settings_menu_editor.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/unit/settings_menu_editor_reaction_editor.rs"]
mod reaction_editor_tests;

#[cfg(test)]
#[path = "../../tests/unit/settings_menu_editor_ledge_position_branch_editor.rs"]
mod ledge_position_branch_editor_tests;
