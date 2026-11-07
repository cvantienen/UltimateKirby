//! Maps recipe actions to editable fields and builds the editor's menu panel.
use super::super::RecipePanel;
use super::*;

#[derive(Clone, Copy)]
pub(super) enum Field {
    Reaction(u8),
    WhileHeld,
    GroundOnly,
    TapButtons(usize),
    TapGap(usize),
    ShieldHold(usize),
    Start,
    Slots,
    Kind(usize),
    Before(usize),
    Duration(usize),
    Buttons(usize),
    Reference(usize),
    CReference(usize),
    CX(usize),
    CY(usize),
    X(usize),
    Y(usize),
    Strength(usize),
    Tolerance(usize),
    Brake(usize),
    PositionTimeout(usize),
    Distance(usize),
    Gap(usize),
    Direction(usize),
}
pub(super) fn fields(r: &Recipe) -> Vec<Field> {
    let mut out = vec![Field::Start];
    for (i, a) in r.sequence.actions().iter().enumerate() {
        out.push(Field::Kind(i));
        if *a != Action::Empty || r.before[i] > 0 {
            out.push(Field::Before(i));
        }
        match a {
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
            Action::Input { .. } | Action::OpponentInput { .. } | Action::CStick { .. } => {
                if !matches!(a, Action::CStick { .. }) {
                    out.extend([Field::Buttons(i), Field::Reference(i)]);
                }
                if matches!(*a, Action::Input { buttons, .. } if buttons != 0) {
                    out.push(Field::ShieldHold(i));
                }
                if !matches!(*a, Action::Input { frames: 0, .. }) {
                    out.push(Field::Duration(i));
                }
                out.extend([Field::X(i), Field::Y(i)])
            }
            Action::Tap { .. } => {
                out.extend([Field::TapButtons(i), Field::TapGap(i), Field::Duration(i)])
            }
            Action::ShieldUntilManual => out.push(Field::ShieldHold(i)),
            Action::Wait(_) | Action::ShieldUntilBlock(_) => out.push(Field::Duration(i)),
            Action::Face(_) => out.push(Field::Direction(i)),
            Action::Chase { .. } | Action::Approach { .. } => {
                out.extend([Field::Strength(i), Field::Duration(i), Field::Gap(i)])
            }
            Action::Walk { direction, .. } => {
                out.extend([Field::Direction(i), Field::Strength(i), Field::Distance(i)]);
                if *direction == D::Opponent {
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
    if r.button_recipe {
        out.push(Field::WhileHeld);
    }
    if r.sequence.reaction.active() {
        if r.sequence.reaction.trigger.simple().is_some() {
            out.extend([3, 4].map(Field::Reaction));
        } else {
            out.extend((0..8).map(Field::Reaction));
        }
    }
    if r.sequence.reaction.trigger.simple().is_none()
        && (r.sequence.reaction.active() || r.sequence.ground_only)
    {
        out.push(Field::GroundOnly);
    }
    out.push(Field::Slots);
    out
}
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
pub(super) fn field_value(r: &Recipe, f: Field) -> String {
    match f {
        Field::Buttons(i) => format!(
            "    Buttons: {} (A: edit)",
            buttons_label(input_buttons(r.sequence.actions[i]).unwrap_or(0))
        ),
        Field::Reference(i) | Field::CReference(i) => format!(
            "    {} X reference: {}",
            if matches!(f, Field::CReference(_)) {
                "Right stick"
            } else {
                "Left stick"
            },
            input_reference(r.sequence.actions[i], matches!(f, Field::CReference(_))).token()
        ),
        Field::CX(i) | Field::CY(i) => {
            let axis = usize::from(matches!(f, Field::CY(_)));
            let stick = match r.sequence.actions[i] {
                Action::Controller { cstick, .. } => cstick,
                _ => [0.; 2],
            };
            format!(
                "    Right stick {}: {:.0}%",
                if axis == 0 { "X" } else { "Y" },
                stick[axis] * 100.
            )
        }
        Field::Reaction(n) => r.sequence.reaction.field(n),
        Field::GroundOnly => format!(
            "Stop if airborne: {}",
            if r.sequence.ground_only { "ON" } else { "OFF" }
        ),
        Field::WhileHeld => format!(
            "Button activation: {}",
            if r.sequence.while_held {
                "Loop while held"
            } else {
                "Run once"
            }
        ),
        Field::ShieldHold(i) => format!(
            "    Hold input: {}",
            if matches!(
                r.sequence.actions[i],
                Action::ShieldUntilManual | Action::Input { frames: 0, .. }
            ) {
                "Until manual takeover"
            } else {
                "For frames"
            }
        ),
        Field::TapButtons(i) => match r.sequence.actions[i] {
            Action::Tap { buttons, stick, .. } => format!(
                "    Tap input: {}",
                TAP_BUTTONS
                    .iter()
                    .find(|(mask, axes, _)| *mask == buttons && *axes == stick)
                    .map_or("Saved input (not in picker)", |(_, _, name)| *name)
            ),
            _ => String::new(),
        },
        Field::TapGap(i) => match r.sequence.actions[i] {
            Action::Tap { hold, gap, .. } => format!(
                "    Delay between taps: {gap} frames (every {}f)",
                u16::from(hold) + u16::from(gap)
            ),
            _ => String::new(),
        },
        Field::Duration(i) if matches!(r.sequence.actions[i], Action::Tap { .. }) => {
            match r.sequence.actions[i] {
                Action::Tap { frames, .. } => format!("    Repeat for: {frames} frames"),
                _ => unreachable!(),
            }
        }
        Field::Start => format!("Sequence start delay: {} frames", r.sequence.start_delay),
        Field::Slots => format!("Action slots: {} / {MAX_ACTIONS}", r.sequence.len),
        Field::Kind(i) => action_name(r, i),
        Field::Before(i) => {
            format!("    Delay before: {} frames", r.before[i])
        }
        Field::Duration(i) => format!(
            "    {}: {} frames",
            match r.sequence.actions[i] {
                Action::Wait(_) => "pause length",
                Action::Input { .. }
                | Action::OpponentInput { .. }
                | Action::Controller { .. }
                | Action::CStick { .. } => "Hold input for",
                Action::ShieldUntilBlock(_) => "maximum wait",
                Action::Check { .. } => "Check wait limit (0 = require now)",
                _ => "movement duration",
            },
            match r.sequence.actions[i] {
                Action::Input { frames, .. }
                | Action::OpponentInput { frames, .. }
                | Action::Controller { frames, .. }
                | Action::CStick { frames, .. }
                | Action::Chase { frames, .. }
                | Action::Approach { frames, .. }
                | Action::Drift { frames, .. }
                | Action::Wait(frames)
                | Action::ShieldUntilBlock(frames) => frames,
                Action::Check { wait, .. } => wait,
                _ => 0,
            }
        ),
        Field::X(i) | Field::Y(i) => {
            let axis = usize::from(matches!(f, Field::Y(_)));
            let a = match r.sequence.actions[i] {
                Action::Input { stick, .. }
                | Action::OpponentInput { stick, .. }
                | Action::Controller { stick, .. }
                | Action::Tap { stick, .. }
                | Action::CStick { stick, .. } => stick,
                _ => [0.; 2],
            };
            format!(
                "    Stick {}: {:.0}%",
                if axis == 0 { "X" } else { "Y" },
                a[axis] * 100.
            )
        }
        Field::Tolerance(i) => format!(
            "    Arrival tolerance: {:.1} units",
            match r.sequence.actions[i] {
                Action::LedgePosition { tolerance, .. } => tolerance,
                _ => 0.,
            }
        ),
        Field::Brake(i) => format!(
            "    Braking distance: {:.1} units (0 = off)",
            match r.sequence.actions[i] {
                Action::LedgePosition { brake, .. } => brake,
                _ => 0.,
            }
        ),
        Field::PositionTimeout(i) => format!(
            "    Movement timeout: {} frames",
            match r.sequence.actions[i] {
                Action::LedgePosition { frames, .. } => frames,
                _ => 0,
            }
        ),
        Field::Strength(i) => format!(
            "    Stick strength: {:.0}%",
            match r.sequence.actions[i] {
                Action::Chase { strength, .. }
                | Action::Approach { strength, .. }
                | Action::Walk { strength, .. }
                | Action::LedgePosition { strength, .. }
                | Action::Drift { strength, .. } => strength * 100.,
                _ => 0.,
            }
        ),
        Field::Distance(i) => format!(
            "    {}: {:.1}",
            if matches!(r.sequence.actions[i], Action::LedgePosition { .. }) {
                "Inward distance from ledge"
            } else {
                "Distance / range"
            },
            match r.sequence.actions[i] {
                Action::Walk { units, .. } | Action::Run { units, .. } => units,
                Action::LedgePosition { offset, .. } => offset,
                Action::Range(RangeCheck::Verify(v)) => v,
                Action::Check { value, .. } => value,
                _ => 0.,
            }
        ),
        Field::Gap(i) => format!(
            "    {}: {}",
            if matches!(r.sequence.actions[i], Action::Chase { .. }) {
                "Skip chase within"
            } else {
                "Stop gap"
            },
            match r.sequence.actions[i] {
                Action::Chase { gap, .. } | Action::Approach { gap, .. } => Some(gap),
                Action::Walk { min_gap, .. } => min_gap,
                _ => None,
            }
            .map_or_else(|| "Off".into(), |v| format!("{v:.0} units"))
        ),
        Field::Direction(i) => format!(
            "    Direction: {}",
            match r.sequence.actions[i] {
                Action::Face(d)
                | Action::Walk { direction: d, .. }
                | Action::Run { direction: d, .. }
                | Action::Drift { direction: d, .. } => d,
                _ => D::Opponent,
            }
            .label()
        ),
    }
}
pub(super) fn same_field(a: Field, b: Field) -> bool {
    std::mem::discriminant(&a) == std::mem::discriminant(&b) && field_slot(a) == field_slot(b)
}
fn short_setting(r: &Recipe, f: Field) -> String {
    if let Field::CX(i) | Field::CY(i) = f {
        let axis = usize::from(matches!(f, Field::CY(_)));
        let stick = match r.sequence.actions[i] {
            Action::Controller { cstick, .. } => cstick,
            _ => [0.; 2],
        };
        return format!(
            "Right {}: {:.0}%",
            if axis == 0 { "X" } else { "Y" },
            stick[axis] * 100.
        );
    }
    if let Field::X(i) | Field::Y(i) = f {
        let axis = usize::from(matches!(f, Field::Y(_)));
        let (stick, side) = match r.sequence.actions[i] {
            Action::Input { stick, .. }
            | Action::OpponentInput { stick, .. }
            | Action::Controller { stick, .. }
            | Action::Tap { stick, .. } => (stick, "Left"),
            Action::CStick { stick, .. } => (stick, "Right"),
            _ => ([0.0; 2], "Left"),
        };
        return format!(
            "{side} {}: {:.0}%",
            if axis == 0 { "X" } else { "Y" },
            stick[axis] * 100.0
        );
    }
    let value = field_value(r, f).trim().to_owned();
    value
        .replace("Delay before: ", "Delay: ")
        .replace("Hold input for: ", "Hold input: ")
        .replace("movement duration: ", "Duration: ")
        .replace("maximum wait: ", "Max wait: ")
        .replace("pause length: ", "Pause: ")
        .replace("Stick strength: ", "Stick: ")
        .replace(
            "Distance / range: ",
            if matches!(f,Field::Distance(i) if matches!(r.sequence.actions[i],Action::Range(_))) {
                "Range: "
            } else {
                "Distance: "
            },
        )
        .replace("AwayFromOpponent", "Away from opponent")
        .replace(" frames", "f")
}
// Nominal input time only. Conditional actions can finish early or wait on
// observations; never present those as exact attack startup/recovery timings.
pub(super) fn scheduled_frames(action: Action) -> Option<u32> {
    match action {
        Action::Tap { frames, .. } => Some(u32::from(frames)),
        Action::Empty
        | Action::Range(crate::sequence_config::RangeCheck::Ignore)
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
pub(super) fn schedule_summary(r: &Recipe) -> (u32, bool) {
    r.sequence.actions().iter().enumerate().fold(
        (u32::from(r.sequence.start_delay), false),
        |(frames, variable), (i, action)| {
            (
                frames + u32::from(r.before[i]) + scheduled_frames(*action).unwrap_or(0),
                variable || scheduled_frames(*action).is_none(),
            )
        },
    )
}
fn frame_label(start: u32, duration: Option<u32>, after_check: bool) -> String {
    let first = start + 1;
    if after_check {
        format!("F{first}+ after checks")
    } else {
        match duration {
            None => format!("F{first} check"),
            Some(n) if n > 1 => format!("F{first}-{}", start + n),
            _ => format!("F{first}"),
        }
    }
}

pub(super) fn recipe_panel(
    r: &Recipe,
    selected: Option<Field>,
    limited: bool,
    last: Option<&str>,
) -> RecipePanel {
    let mut panel = RecipePanel::default();
    let fs = fields(r);
    if let Some(field) = selected {
        panel.header.push(field_slot(field).map_or_else(
            || "SEQUENCE | Start and action count".into(),
            |i| {
                format!(
                    "STEP {} / {} | {}",
                    i + 1,
                    r.sequence.len,
                    action_name(r, i)
                )
            },
        ));
    } else {
        panel
            .header
            .push(format!("{} action slots | Full sequence", r.sequence.len));
    }
    let branched = r.sequence.actions().iter().any(|a| {
        matches!(
            a,
            Action::IfFacingLedge { .. } | Action::IfFacingOpponent { .. }
        )
    });
    let (scheduled, variable) = schedule_summary(r);
    panel.header.push(if branched {
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
    });
    panel
        .footer
        .push("F1 = recognition update; move startup/recovery excluded".into());
    let mut frame_cursor = u32::from(r.sequence.start_delay);
    let mut after_check = false;
    let start = format!("Start delay: {} frames", r.sequence.start_delay);
    let focus_start = selected.is_some_and(|f| matches!(f, Field::Start));
    panel.body.push(if focus_start {
        format!("> [{start}]")
    } else {
        start
    });
    if focus_start {
        panel.focus = Some(0);
    }
    for i in 0..usize::from(r.sequence.len) {
        let name = if r.sequence.actions[i] == Action::Empty {
            if r.before[i] > 0 {
                "Delay only".to_owned()
            } else {
                "Empty (unused)".to_owned()
            }
        } else {
            action_name(r, i)
        };
        let before_start = frame_cursor;
        frame_cursor += u32::from(r.before[i]);
        let action = r.sequence.actions[i];
        let duration = scheduled_frames(action);
        let timing = if branched {
            duration.map_or_else(
                || " | Conditional/spatial".into(),
                |frames| format!(" | {frames}f when reached"),
            )
        } else if action == Action::Empty {
            if r.before[i] == 0 {
                String::new()
            } else {
                format!(
                    " | {}",
                    frame_label(before_start, Some(u32::from(r.before[i])), after_check)
                )
            }
        } else {
            format!(" | {}", frame_label(frame_cursor, duration, after_check))
        };
        frame_cursor += duration.unwrap_or(0);
        after_check |= duration.is_none();
        let title = format!(
            "{:02}  {}{}{}",
            i + 1,
            name,
            if r.edited & (1 << i) != 0 { " *" } else { "" },
            timing
        );
        let kind_selected = selected.is_some_and(|f| same_field(f, Field::Kind(i)));
        if kind_selected {
            panel.focus = Some(panel.body.len());
        }
        panel.body.push(if kind_selected {
            format!("> {title}")
        } else {
            title
        });
        let expanded = selected.and_then(field_slot) == Some(i);
        let mut settings = Vec::new();
        for f in fs
            .iter()
            .copied()
            .filter(|f| field_slot(*f) == Some(i) && !matches!(f, Field::Kind(_)))
        {
            let focused = selected.is_some_and(|s| same_field(s, f));
            let text = short_setting(r, f);
            if expanded {
                if focused {
                    panel.focus = Some(panel.body.len());
                }
                panel.body.push(format!(
                    "{}{}",
                    if focused { ">   " } else { "    " },
                    if focused { format!("[{text}]") } else { text }
                ));
            } else {
                settings.push(text);
            }
        }
        if !settings.is_empty() {
            panel.body.push(format!("    {}", settings.join(" | ")));
        }
    }
    if r.button_recipe {
        let text = field_value(r, Field::WhileHeld);
        if selected.is_some_and(|f| matches!(f, Field::WhileHeld)) {
            panel.focus = Some(panel.body.len());
            panel.body.push(format!("> [{text}]"));
        } else {
            panel.body.push(text);
        }
    }
    for n in 0..if r.sequence.reaction.active() { 8 } else { 0 } {
        if r.sequence.reaction.trigger.simple().is_some() && ![0, 3, 4].contains(&n) {
            continue;
        }
        let text = field_value(r, Field::Reaction(n));
        if selected.is_some_and(|f| matches!(f,Field::Reaction(i) if i==n)) {
            panel.focus = Some(panel.body.len());
            panel.body.push(format!("> [{text}]"));
        } else {
            panel.body.push(text);
        }
    }
    if r.sequence.reaction.trigger.simple().is_none()
        && (r.sequence.reaction.active() || r.sequence.ground_only)
    {
        let focus = selected.is_some_and(|f| matches!(f, Field::GroundOnly));
        if focus {
            panel.focus = Some(panel.body.len());
        }
        let text = field_value(r, Field::GroundOnly);
        panel
            .body
            .push(if focus { format!("> [{text}]") } else { text });
    }
    let slots = field_value(r, Field::Slots);
    if selected.is_some_and(|f| matches!(f, Field::Slots)) {
        panel.focus = Some(panel.body.len());
        panel.body.push(format!("> [{slots}]"));
    } else {
        panel.body.push(slots);
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
