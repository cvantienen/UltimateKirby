//! Maps recipe actions to editable fields and builds the editor's menu panel.
use super::super::RecipePanel;
use super::*;
use std::borrow::Cow;

/// Defines the available `Field` cases.
///
///
///  Lists the available Field choices.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Field {
    /// Represents the `Reaction` case of `Field`.
    ///
    ///
///  Represents the Reaction option in Field.
    Reaction(
        /// Carries the reaction field index in `Field`.
        ///
        ///
///  Holds the reaction field index for this `Field` choice.
        u8,
    ),
    /// Represents the `WhileHeld` case of `Field`.
    ///
    ///
///  Represents the WhileHeld option in Field.
    WhileHeld,
    /// Represents the `GroundOnly` case of `Field`.
    ///
    ///
///  Represents the GroundOnly option in Field.
    GroundOnly,
    /// Represents the `TapButtons` case of `Field`.
    ///
    ///
///  Represents the TapButtons option in Field.
    TapButtons(
        /// Carries the tap-button field index in `Field`.
        ///
        ///
///  Holds the tap-button field index for this `Field` choice.
        usize,
    ),
    /// Represents the `TapGap` case of `Field`.
    ///
    ///
///  Represents the TapGap option in Field.
    TapGap(
        /// Carries the tap-gap field index in `Field`.
        ///
        ///
///  Holds the tap-gap field index for this `Field` choice.
        usize,
    ),
    /// Represents the `ShieldHold` case of `Field`.
    ///
    ///
///  Represents the ShieldHold option in Field.
    ShieldHold(
        /// Carries the shield-hold field index in `Field`.
        ///
        ///
///  Holds the shield-hold field index for this `Field` choice.
        usize,
    ),
    /// Represents the `Start` case of `Field`.
    ///
    ///
///  Represents the Start option in Field.
    Start,
    /// Represents the `Slots` case of `Field`.
    ///
    ///
///  Represents the Slots option in Field.
    Slots,
    /// Represents the `Kind` case of `Field`.
    ///
    ///
///  Represents the Kind option in Field.
    Kind(
        /// Carries the action-kind field index in `Field`.
        ///
        ///
///  Holds the action-kind field index for this `Field` choice.
        usize,
    ),
    /// Represents the `Before` case of `Field`.
    ///
    ///
///  Represents the Before option in Field.
    Before(
        /// Carries the pre-action delay field index in `Field`.
        ///
        ///
///  Holds the pre-action delay field index for this `Field` choice.
        usize,
    ),
    /// Represents the `Duration` case of `Field`.
    ///
    ///
///  Represents the Duration option in Field.
    Duration(
        /// Carries the action-duration field index in `Field`.
        ///
        ///
///  Holds the action-duration field index for this `Field` choice.
        usize,
    ),
    /// Represents the `Buttons` case of `Field`.
    ///
    ///
///  Represents the Buttons option in Field.
    Buttons(
        /// Carries the button field index in `Field`.
        ///
        ///
///  Holds the button field index for this `Field` choice.
        usize,
    ),
    /// Represents the `Reference` case of `Field`.
    ///
    ///
///  Represents the Reference option in Field.
    Reference(
        /// Carries the reference field index in `Field`.
        ///
        ///
///  Holds the reference field index for this `Field` choice.
        usize,
    ),
    /// Represents the `CReference` case of `Field`.
    ///
    ///
///  Represents the CReference option in Field.
    CReference(
        /// Carries the center-reference field index in `Field`.
        ///
        ///
///  Holds the center-reference field index for this `Field` choice.
        usize,
    ),
    /// Represents the `CX` case of `Field`.
    ///
    ///
///  Represents the CX option in Field.
    CX(
        /// Carries the center X-coordinate field index in `Field`.
        ///
        ///
///  Holds the center X-coordinate field index for this `Field` choice.
        usize,
    ),
    /// Represents the `CY` case of `Field`.
    ///
    ///
///  Represents the CY option in Field.
    CY(
        /// Carries the center Y-coordinate field index in `Field`.
        ///
        ///
///  Holds the center Y-coordinate field index for this `Field` choice.
        usize,
    ),
    /// Represents the `X` case of `Field`.
    ///
    ///
///  Represents the X option in Field.
    X(
        /// Carries the X-coordinate field index in `Field`.
        ///
        ///
///  Holds the X-coordinate field index for this `Field` choice.
        usize,
    ),
    /// Represents the `Y` case of `Field`.
    ///
    ///
///  Represents the Y option in Field.
    Y(
        /// Carries the Y-coordinate field index in `Field`.
        ///
        ///
///  Holds the Y-coordinate field index for this `Field` choice.
        usize,
    ),
    /// Represents the `Strength` case of `Field`.
    ///
    ///
///  Represents the Strength option in Field.
    Strength(
        /// Carries the input-strength field index in `Field`.
        ///
        ///
///  Holds the input-strength field index for this `Field` choice.
        usize,
    ),
    /// Represents the `Tolerance` case of `Field`.
    ///
    ///
///  Represents the Tolerance option in Field.
    Tolerance(
        /// Carries the tolerance field index in `Field`.
        ///
        ///
///  Holds the tolerance field index for this `Field` choice.
        usize,
    ),
    /// Represents the `Brake` case of `Field`.
    ///
    ///
///  Represents the Brake option in Field.
    Brake(
        /// Carries the braking field index in `Field`.
        ///
        ///
///  Holds the braking field index for this `Field` choice.
        usize,
    ),
    /// Represents the `PositionTimeout` case of `Field`.
    ///
    ///
///  Represents the PositionTimeout option in Field.
    PositionTimeout(
        /// Carries the position-timeout field index in `Field`.
        ///
        ///
///  Holds the position-timeout field index for this `Field` choice.
        usize,
    ),
    /// Represents the `Distance` case of `Field`.
    ///
    ///
///  Represents the Distance option in Field.
    Distance(
        /// Carries the distance field index in `Field`.
        ///
        ///
///  Holds the distance field index for this `Field` choice.
        usize,
    ),
    /// Represents the `Gap` case of `Field`.
    ///
    ///
///  Represents the Gap option in Field.
    Gap(
        /// Carries the gap field index in `Field`.
        ///
        ///
///  Holds the gap field index for this `Field` choice.
        usize,
    ),
    /// Represents the `Direction` case of `Field`.
    ///
    ///
///  Represents the Direction option in Field.
    Direction(
        /// Carries the direction field index in `Field`.
        ///
        ///
///  Holds the direction field index for this `Field` choice.
        usize,
    ),
}
/// Adds the action fields.
///
///
///  Adds the action fields.
fn push_action_fields(out: &mut Vec<Field>, action: Action, i: usize) {
    match action {
        Action::Controller { .. } => out.extend([
            Field::Buttons(i),
            Field::Reference(i),
            Field::CReference(i),
            Field::Duration(i),
            Field::X(i),
            Field::Y(i),
            Field::CX(i),
            Field::CY(i),
        ]),
        Action::Input {
            buttons, frames, ..
        } => {
            out.extend([Field::Buttons(i), Field::Reference(i)]);
            if buttons != 0 {
                out.push(Field::ShieldHold(i));
            }
            if frames != 0 {
                out.push(Field::Duration(i));
            }
            out.extend([Field::X(i), Field::Y(i)]);
        }
        Action::OpponentInput { .. } => out.extend([
            Field::Buttons(i),
            Field::Reference(i),
            Field::Duration(i),
            Field::X(i),
            Field::Y(i),
        ]),
        Action::CStick { .. } => out.extend([Field::Duration(i), Field::X(i), Field::Y(i)]),
        Action::Tap { .. } => out.extend([Field::TapButtons(i), Field::TapGap(i), Field::Duration(i)]),
        Action::ShieldUntilManual => out.push(Field::ShieldHold(i)),
        Action::Wait(_) | Action::ShieldUntilBlock(_) => out.push(Field::Duration(i)),
        Action::Face(_) => out.push(Field::Direction(i)),
        Action::Chase { .. } | Action::Approach { .. } => {
            out.extend([Field::Strength(i), Field::Duration(i), Field::Gap(i)])
        }
        Action::Walk { direction, .. } => {
            out.extend([Field::Direction(i), Field::Strength(i), Field::Distance(i)]);
            if direction == D::Opponent {
                out.push(Field::Gap(i));
            }
        }
        Action::Run { .. } => out.extend([Field::Direction(i), Field::Distance(i)]),
        Action::LedgePosition { .. } => out.extend([
            Field::Distance(i),
            Field::Strength(i),
            Field::Tolerance(i),
            Field::Brake(i),
            Field::PositionTimeout(i),
        ]),
        Action::Drift { .. } => {
            out.extend([Field::Direction(i), Field::Strength(i), Field::Duration(i)])
        }
        Action::Check { condition, .. } => {
            out.push(Field::Duration(i));
            if condition.has_distance() {
                out.push(Field::Distance(i));
            }
        }
        Action::Range(RangeCheck::Verify(_)) => out.push(Field::Distance(i)),
        _ => {}
    }
}
/// Every focusable field in display order: Start, each slot's fields, then the
/// sequence-level rows. `recipe_panel` renders straight from this list.
///
///  Every focusable field in display order: Start, each slot's fields, then the sequence-level rows. recipepanel renders straight from this list.
pub(super) fn fields(r: &Recipe) -> Vec<Field> {
    let mut out = vec![Field::Start];
    for (i, a) in r.sequence.actions().iter().enumerate() {
        out.push(Field::Kind(i));
        if *a != Action::Empty || r.before[i] > 0 {
            out.push(Field::Before(i));
        }
        push_action_fields(&mut out, *a, i);
    }
    let reaction = &r.sequence.reaction;
    if r.button_recipe {
        out.push(Field::WhileHeld);
    }
    if reaction.active() {
        if reaction.trigger.simple().is_some() {
            out.extend([3, 4].map(Field::Reaction));
        } else {
            out.extend((0..8).map(Field::Reaction));
        }
    }
    if reaction.trigger.simple().is_none() && (reaction.active() || r.sequence.ground_only) {
        out.push(Field::GroundOnly);
    }
    out.push(Field::Slots);
    out
}
/// Returns an optional `field_slot` value.
///
///
///  Gets field slot if it is available.
pub(super) fn field_slot(f: Field) -> Option<usize> {
    match f {
        Field::Start | Field::Slots | Field::WhileHeld | Field::GroundOnly | Field::Reaction(_) => {
            None
        }
        Field::Buttons(i)
        | Field::Reference(i)
        | Field::CReference(i)
        | Field::CX(i)
        | Field::CY(i)
        | Field::ShieldHold(i)
        | Field::TapButtons(i)
        | Field::TapGap(i)
        | Field::Kind(i)
        | Field::Before(i)
        | Field::Duration(i)
        | Field::X(i)
        | Field::Y(i)
        | Field::Strength(i)
        | Field::Tolerance(i)
        | Field::Brake(i)
        | Field::PositionTimeout(i)
        | Field::Distance(i)
        | Field::Gap(i)
        | Field::Direction(i) => Some(i),
    }
}
/// Full label and value of a field.
///
///  Full label and value of a field.
pub(super) fn field_value(r: &Recipe, f: Field) -> String {
    field_text(r, f, false)
}
/// Label and value of a field. `short` is the compact form used in the slot
/// list: abbreviated labels and `Nf` instead of `N frames`.
///
///  Label and value of a field. short is the compact form used in the slot list: abbreviated labels and Nf instead of N frames.
fn field_text(r: &Recipe, f: Field, short: bool) -> String {
    let sequence = &r.sequence;
    let action = field_slot(f).map_or(Action::Empty, |i| sequence.actions[i]);
    let unit = if short { "f" } else { " frames" };
    let pick = |full, brief| if short { brief } else { full };
    let percent = |field| scalar(action, field).unwrap_or(0.) * 100.;
    match f {
        Field::Reaction(n) => sequence.reaction.field(n),
        Field::GroundOnly => format!(
            "Stop if airborne: {}",
            if sequence.ground_only { "ON" } else { "OFF" }
        ),
        Field::WhileHeld => format!(
            "Button activation: {}",
            if sequence.while_held {
                "Loop while held"
            } else {
                "Run once"
            }
        ),
        Field::Start => format!("Sequence start delay: {} frames", sequence.start_delay),
        Field::Slots => format!("Action slots: {} / {MAX_ACTIONS}", sequence.len),
        Field::Kind(i) => action_name(r, i),
        Field::Before(i) => format!("{}: {}{unit}", pick("Delay before", "Delay"), r.before[i]),
        Field::Buttons(_) => format!(
            "Buttons: {} (A: edit)",
            buttons_text(input_buttons(action).unwrap_or(0), "None (stick only)")
        ),
        Field::Reference(_) | Field::CReference(_) => {
            let right = matches!(f, Field::CReference(_));
            format!(
                "{} stick X reference: {}",
                if right { "Right" } else { "Left" },
                input_reference(action, right).token()
            )
        }
        Field::CX(_) | Field::CY(_) => format!(
            "{} {}: {:.0}%",
            pick("Right stick", "Right"),
            if matches!(f, Field::CX(_)) { "X" } else { "Y" },
            percent(f)
        ),
        Field::X(_) | Field::Y(_) => {
            let axis = if matches!(f, Field::X(_)) { "X" } else { "Y" };
            let side = match action {
                Action::CStick { .. } => "Right",
                _ => "Left",
            };
            if short {
                format!("{side} {axis}: {:.0}%", percent(f))
            } else {
                format!("Stick {axis}: {:.0}%", percent(f))
            }
        }
        Field::ShieldHold(_) => format!(
            "Hold input: {}",
            if matches!(
                action,
                Action::ShieldUntilManual | Action::Input { frames: 0, .. }
            ) {
                "Until manual takeover"
            } else {
                "For frames"
            }
        ),
        Field::TapButtons(_) => match action {
            Action::Tap { buttons, stick, .. } => format!(
                "Tap input: {}",
                TAP_BUTTONS
                    .iter()
                    .find(|(mask, axes, _)| *mask == buttons && *axes == stick)
                    .map_or("Saved input (not in picker)", |(_, _, name)| *name)
            ),
            _ => String::new(),
        },
        Field::TapGap(_) => match action {
            Action::Tap { hold, gap, .. } => format!(
                "Delay between taps: {gap}{unit} (every {}f)",
                u16::from(hold) + u16::from(gap)
            ),
            _ => String::new(),
        },
        Field::Duration(_) => match action {
            Action::Tap { frames, .. } => format!("Repeat for: {frames}{unit}"),
            _ => {
                let label = match action {
                    Action::Wait(_) => pick("pause length", "Pause"),
                    Action::Input { .. }
                    | Action::OpponentInput { .. }
                    | Action::Controller { .. }
                    | Action::CStick { .. } => pick("Hold input for", "Hold input"),
                    Action::ShieldUntilBlock(_) => pick("maximum wait", "Max wait"),
                    Action::Check { .. } => "Check wait limit (0 = require now)",
                    _ => pick("movement duration", "Duration"),
                };
                format!("{label}: {}{unit}", frames(action).unwrap_or(0))
            }
        },
        Field::Tolerance(_) => format!(
            "Arrival tolerance: {:.1} units",
            scalar(action, f).unwrap_or(0.)
        ),
        Field::Brake(_) => format!(
            "Braking distance: {:.1} units (0 = off)",
            scalar(action, f).unwrap_or(0.)
        ),
        Field::PositionTimeout(_) => format!(
            "Movement timeout: {}{unit}",
            match action {
                Action::LedgePosition { frames, .. } => frames,
                _ => 0,
            }
        ),
        Field::Strength(_) => format!(
            "{}: {:.0}%",
            pick("Stick strength", "Stick"),
            percent(f)
        ),
        Field::Distance(_) => format!(
            "{}: {:.1}",
            match action {
                Action::LedgePosition { .. } => "Inward distance from ledge",
                Action::Range(_) => pick("Distance / range", "Range"),
                _ => pick("Distance / range", "Distance"),
            },
            scalar(action, f).unwrap_or(0.)
        ),
        Field::Gap(_) => {
            let gap = match action {
                Action::Walk { min_gap, .. } => min_gap,
                _ => scalar(action, f),
            };
            format!(
                "{}: {}",
                if matches!(action, Action::Chase { .. }) {
                    "Skip chase within"
                } else {
                    "Stop gap"
                },
                gap.map_or_else(|| "Off".into(), |v| format!("{v:.0} units"))
            )
        }
        Field::Direction(_) => {
            let mut action = action;
            format!(
                "Direction: {}",
                direction_mut(&mut action).map_or(D::Opponent, |d| *d).label()
            )
        }
    }
}
// Nominal input time only. Conditional actions can finish early or wait on
// observations; never present those as exact attack startup/recovery timings.
/// Returns an optional `scheduled_frames` value.
///
///
///  Gets scheduled frames if it is available.
pub(super) fn scheduled_frames(action: Action) -> Option<u32> {
    match action {
        Action::Tap { frames, .. } => Some(u32::from(frames)),
        Action::Empty
        | Action::Range(RangeCheck::Ignore)
        | Action::Check { wait: 0, .. }
        | Action::IfFacingLedge { .. }
        | Action::IfFacingOpponent { .. }
        | Action::ElseFacing
        | Action::EndFacing => Some(0),
        Action::Input { frames, .. }
        | Action::OpponentInput { frames, .. }
        | Action::Controller { frames, .. }
        | Action::CStick { frames, .. }
        | Action::Wait(frames)
        | Action::Drift { frames, .. } => Some(u32::from(frames)),
        _ => None,
    }
}
/// Returns the `schedule_summary` result.
///
///
///  Gets the schedule summary result.
pub(super) fn schedule_summary(r: &Recipe) -> (u32, bool) {
    r.sequence.actions().iter().enumerate().fold(
        (u32::from(r.sequence.start_delay), false),
        |(total, variable), (i, action)| {
            let frames = scheduled_frames(*action);
            (
                total + u32::from(r.before[i]) + frames.unwrap_or(0),
                variable || frames.is_none(),
            )
        },
    )
}
/// ` | F<n>` style timing suffix; `None` duration means a check of unknown length.
///
///   | F<n> style timing suffix; None duration means a check of unknown length.
fn frame_label(start: u32, duration: Option<u32>, after_check: bool) -> String {
    let first = start + 1;
    if after_check {
        format!(" | F{first}+ after checks")
    } else {
        match duration {
            None => format!(" | F{first} check"),
            Some(n) if n > 1 => format!(" | F{first}-{}", start + n),
            _ => format!(" | F{first}"),
        }
    }
}

/// How a panel row looks when plain and when focused.
///
///  How a panel row looks when plain and when focused.
#[derive(Clone, Copy)]
struct Row {
    /// Stores the `plain` value in `Row` as `&'static str`.
    ///
    ///
///  Holds the plain value used by Row.
    plain: &'static str,
    /// Stores the `focused` value in `Row` as `&'static str`.
    ///
    ///
///  Holds the focused value used by Row.
    focused: &'static str,
    /// Stores the `bracket` value in `Row` as `bool`.
    ///
    ///
///  Holds the bracket value used by Row.
    bracket: bool,
}
/// Defines the `FIELD_ROW` constant used by this module.
///
///
///  Provides the FIELD ROW constant value.
const FIELD_ROW: Row = Row {
    plain: "",
    focused: "> ",
    bracket: true,
};
/// Defines the `TITLE_ROW` constant used by this module.
///
///
///  Provides the TITLE ROW constant value.
const TITLE_ROW: Row = Row {
    plain: "",
    focused: "> ",
    bracket: false,
};
/// Defines the `SETTING_ROW` constant used by this module.
///
///
///  Provides the SETTING ROW constant value.
const SETTING_ROW: Row = Row {
    plain: "    ",
    focused: ">   ",
    bracket: true,
};
/// Adds the row.
///
///
///  Adds the row.
fn push_row(panel: &mut RecipePanel, focused: bool, row: Row, text: impl std::fmt::Display) {
    let text = match (focused, row.bracket) {
        (true, true) => format!("{}[{text}]", row.focused),
        (true, false) => format!("{}{text}", row.focused),
        (false, _) => format!("{}{text}", row.plain),
    };
    if focused {
        panel.focus = Some(panel.body.len());
    }
    panel.body.push(text);
}

/// Returns the `timing_summary` result.
///
///
///  Gets the timing summary result.
fn timing_summary(r: &Recipe, branched: bool) -> String {
    let (scheduled, variable) = schedule_summary(r);
    if branched {
        "Conditional timing: only the matching facing block runs".into()
    } else if variable {
        format!("Timed parts: {scheduled}f + variable checks")
    } else if scheduled >= u32::from(r.sequence.timeout) {
        format!(
            "Input schedule: {scheduled}f | timeout {}f",
            r.sequence.timeout
        )
    } else {
        format!("Input schedule: {scheduled}f | done F{}", scheduled + 1)
    }
}

/// Renders every field in `fs` (from `fields(r)`); `selected` marks the focused one.
///
///  Renders every field in fs (from fields(r)); selected marks the focused one.
pub(super) fn recipe_panel(
    r: &Recipe,
    fs: &[Field],
    selected: Option<Field>,
    limited: bool,
    last: Option<&str>,
) -> RecipePanel {
    let mut panel = RecipePanel::default();
    panel.header.push(match selected {
        None => format!("{} action slots | Full sequence", r.sequence.len),
        Some(field) => field_slot(field).map_or_else(
            || "SEQUENCE | Start and action count".into(),
            |i| {
                format!(
                    "STEP {} / {} | {}",
                    i + 1,
                    r.sequence.len,
                    action_name(r, i)
                )
            },
        ),
    });
    let branched = r.sequence.actions().iter().any(|a| {
        matches!(
            a,
            Action::IfFacingLedge { .. } | Action::IfFacingOpponent { .. }
        )
    });
    panel.header.push(timing_summary(r, branched));
    panel
        .footer
        .push("F1 = recognition update; move startup/recovery excluded".into());
    let is_selected = |f: Field| selected == Some(f);
    let mut rest = fs.iter().copied().peekable();
    rest.next_if_eq(&Field::Start);
    push_row(
        &mut panel,
        is_selected(Field::Start),
        FIELD_ROW,
        format_args!("Start delay: {} frames", r.sequence.start_delay),
    );
    let mut frame_cursor = u32::from(r.sequence.start_delay);
    let mut after_check = false;
    for i in 0..usize::from(r.sequence.len) {
        let action = r.sequence.actions[i];
        let delay = u32::from(r.before[i]);
        let name: Cow<str> = match (action == Action::Empty, delay > 0) {
            (false, _) => action_name(r, i).into(),
            (true, true) => "Delay only".into(),
            (true, false) => "Empty (unused)".into(),
        };
        let before_start = frame_cursor;
        frame_cursor += delay;
        let duration = scheduled_frames(action);
        let timing = if branched {
            duration.map_or_else(
                || " | Conditional/spatial".into(),
                |frames| format!(" | {frames}f when reached"),
            )
        } else if action != Action::Empty {
            frame_label(frame_cursor, duration, after_check)
        } else if delay > 0 {
            frame_label(before_start, Some(delay), after_check)
        } else {
            String::new()
        };
        frame_cursor += duration.unwrap_or(0);
        after_check |= duration.is_none();
        push_row(
            &mut panel,
            is_selected(Field::Kind(i)),
            TITLE_ROW,
            format_args!(
                "{:02}  {name}{}{timing}",
                i + 1,
                if r.edited & (1 << i) != 0 { " *" } else { "" }
            ),
        );
        let expanded = selected.and_then(field_slot) == Some(i);
        let mut collapsed = String::new();
        while let Some(f) = rest.next_if(|f| field_slot(*f) == Some(i)) {
            if matches!(f, Field::Kind(_)) {
                continue;
            }
            let text = field_text(r, f, true);
            if expanded {
                push_row(&mut panel, is_selected(f), SETTING_ROW, text);
            } else {
                if !collapsed.is_empty() {
                    collapsed.push_str(" | ");
                }
                collapsed.push_str(&text);
            }
        }
        if !collapsed.is_empty() {
            panel.body.push(format!("    {collapsed}"));
        }
    }
    let simple_event = r.sequence.reaction.trigger.simple().is_some();
    for f in rest {
        if simple_event && f == Field::Reaction(3) {
            // The trigger of a simple event is fixed, so it is shown but never focused.
            panel.body.push(field_value(r, Field::Reaction(0)));
        }
        push_row(&mut panel, is_selected(f), FIELD_ROW, field_value(r, f));
    }
    if r.sequence
        .actions()
        .iter()
        .any(|a| matches!(a, Action::Check { .. }))
    {
        panel.footer.push(
            "Check: true continues; false waits up to its limit, then ends the recipe.".into(),
        );
    }
    if let Some(last) = last {
        panel.footer.push(last.into());
    }
    if limited {
        panel
            .footer
            .push("Step limit (32): remove an action or delay before adding more.".into());
    }
    panel
}
