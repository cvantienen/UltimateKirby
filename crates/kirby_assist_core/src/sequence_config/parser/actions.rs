//! Parse action-list tokens into the fixed-size recipe representation.
use super::words_id;
use crate::native_input::{
    MAPPER_A as A, MAPPER_B as B, MAPPER_R as R, MAPPER_X as X, MAPPER_ZR as ZR,
};
use crate::sequence_config::{
    Action, Condition, Direction, InputReference, RangeCheck, Sequence, BUTTON_TOKENS, MAX_ACTIONS,
    MAX_SEQUENCE_FRAME_COUNT,
};

/// Returns the `push` result.
/// Gets the push result.
fn push(sequence: &mut Sequence, action: Action) -> Result<(), String> {
    if usize::from(sequence.len) == MAX_ACTIONS {
        return Err(format!(
            "at most {MAX_ACTIONS} expanded actions are allowed"
        ));
    }
    sequence.actions[usize::from(sequence.len)] = action;
    sequence.len += 1;
    Ok(())
}

/// Parses the actions.
/// Parses the actions.
pub(super) fn parse_actions(value: &str, line: usize) -> Result<Sequence, String> {
    let mut result = Sequence::default();
    for token in value.split(',').map(str::trim) {
        if token.is_empty() {
            return Err(format!("line {line}: empty action"));
        }
        parse_action(token, &mut result).map_err(|e| format!("line {line}: {e}"))?;
    }
    validate_facing_blocks(result.actions()).map_err(|e| format!("line {line}: {e}"))?;
    Ok(result)
}

/// Flat facing blocks are deliberately bounded and cannot nest.
pub fn validate_facing_blocks(actions: &[Action]) -> Result<(), String> {
    let mut open = false;
    let mut otherwise = false;
    for action in actions {
        match action {
            Action::IfFacingLedge { .. } | Action::IfFacingOpponent { .. } => {
                if open {
                    return Err("Facing blocks cannot nest; add end_facing first".into());
                }
                open = true;
                otherwise = false;
            }
            Action::ElseFacing => {
                if !open || otherwise {
                    return Err(
                        "else_facing needs an open facing block and may appear only once".into(),
                    );
                }
                otherwise = true;
            }
            Action::EndFacing => {
                if !open {
                    return Err("end_facing has no matching facing condition".into());
                }
                open = false;
            }
            _ => {}
        }
    }
    if open {
        return Err("Facing block needs end_facing".into());
    }
    Ok(())
}

/// Constructs a parsed input action from the supplied arguments.
/// Builds the input action described by the arguments.
fn input(buttons: u32, stick: [f32; 2], frames: u8, relative: bool) -> Action {
    Action::Input {
        buttons,
        stick,
        frames,
        relative,
    }
}

/// One colon-separated action token after alias normalization. Every accessor
/// reports failures against the author's original text.
struct Args<'a> {
/// Runtime token value used by Args..
    token: &'a str,
/// Runtime parts value used by Args..
    parts: Vec<&'a str>,
}

impl Args<'_> {
    /// Returns the number of stored elements.
    fn len(&self) -> usize {
        self.parts.len()
    }

    /// Returns the `invalid` result.
    fn invalid(&self) -> String {
        format!("invalid action {}", self.token)
    }

    /// Returns the `exact` result.
    fn exact(&self, count: usize) -> Result<(), String> {
        if self.len() == count {
            Ok(())
        } else {
            Err(self.invalid())
        }
    }

    /// Returns the `at_most` result.
    fn at_most(&self, count: usize) -> Result<(), String> {
        if self.len() <= count {
            Ok(())
        } else {
            Err(self.invalid())
        }
    }

    /// Returns the floating-point value when the fact contains one.
    fn float(&self, index: usize, min: f32, max: f32) -> Result<f32, String> {
        parse_float(self.parts[index], min, max, self.token)
    }

    /// Returns the `float_or` result.
    fn float_or(&self, index: usize, default: f32, min: f32, max: f32) -> Result<f32, String> {
        if index < self.len() {
            self.float(index, min, max)
        } else {
            Ok(default)
        }
    }

    /// Left- or right-stick pair at `index` and `index + 1`.
    fn stick(&self, index: usize) -> Result<[f32; 2], String> {
        Ok([
            self.float(index, -1.0, 1.0)?,
            self.float(index + 1, -1.0, 1.0)?,
        ])
    }

    /// Returns the `frames_or` result.
    fn frames_or(&self, index: usize, default: u8) -> Result<u8, String> {
        self.parts
            .get(index)
            .map_or(Ok(default), |value| parse_u8(value, self.token))
    }

    /// Input hold length: 1..255 native frames.
    fn duration(&self, index: usize, default: u8) -> Result<u8, String> {
        match self.frames_or(index, default)? {
            0 => Err(format!("{}: input duration must be 1..255", self.token)),
            frames => Ok(frames),
        }
    }

    /// Returns the `buttons` result.
    fn buttons(&self, index: usize) -> Result<u32, String> {
        buttons(self.parts[index])
    }

    /// Returns the `direction` result.
    fn direction(&self, index: usize) -> Result<Direction, String> {
        Direction::from_token(self.parts[index])
            .ok_or_else(|| format!("invalid direction in {}", self.token))
    }

    /// Returns the `reference` result.
    fn reference(&self, index: usize) -> Result<InputReference, String> {
        InputReference::parse(self.parts.get(index).copied().unwrap_or("screen"))
    }
}

/// Canonical single-letter names plus the long aliases accepted in `+` lists.
fn buttons(value: &str) -> Result<u32, String> {
    let mut mask = 0;
    for button in value.split('+') {
        mask |= match button_alias(button) {
            Some(bit) => bit,
            None => BUTTON_TOKENS
                .iter()
                .find(|(_, token)| *token == button)
                .map(|&(bit, _)| bit)
                .ok_or_else(|| format!("unknown button {button}"))?,
        };
    }
    Ok(mask)
}

/// Rewrite the spelled-out shortcuts that are not simple aliases of one token
/// into the canonical `name:arg` layout used by the dispatcher below.
fn normalize(token: &str) -> String {
    let normalized = token
        .split(':')
        .map(|v| {
            if v.trim().starts_with('-') {
                v.trim().to_ascii_lowercase()
            } else {
                words_id(v)
            }
        })
        .collect::<Vec<_>>()
        .join(":");
    if let Some(wait) = normalized.strip_prefix("wait_") {
        format!(
            "wait:{}",
            wait.strip_suffix("_frames")
                .or_else(|| wait.strip_suffix("_frame"))
                .unwrap_or(wait)
        )
    } else if let Some(direction) = normalized.strip_prefix("face_") {
        format!("face:{direction}")
    } else if normalized == "grab_no_range_check" {
        "grab:ignore".into()
    } else if normalized == "grab_range_verified" {
        "grab:fighter_range".into()
    } else {
        normalized
    }
}

/// Parses the action.
/// Parses the action.
fn parse_action(token: &str, sequence: &mut Sequence) -> Result<(), String> {
    let normalized = normalize(token);
    let args = Args {
        token,
        parts: normalized.split(':').collect(),
    };
    let name = args.parts[0];
    let action = match name {
        "wait" | "release" => wait(&args)?,
        "check" | "require" => check(&args)?,
        "tap" => tap(&args)?,
        "press" | "hold" => press(&args)?,
        "controller_input" => controller_input(&args)?,
        "opponent_input" => opponent_input(&args)?,
        "input" | "facing_input" => {
            args.exact(5)?;
            input(
                args.buttons(1)?,
                args.stick(2)?,
                args.duration(4, 1)?,
                name == "facing_input",
            )
        }
        "cstick" => {
            args.exact(4)?;
            Action::CStick {
                stick: args.stick(1)?,
                frames: args.duration(3, 1)?,
            }
        }
        "stick" => {
            args.exact(4)?;
            input(0, args.stick(1)?, args.duration(3, 1)?, false)
        }
        "hold_until_manual" | "facing_hold_until_manual" => hold_until_manual(&args)?,
        "chase_if_far" => {
            args.exact(4)?;
            Action::Chase {
                gap: args.float(1, 0., 100.)?,
                strength: args.float(2, 0., 1.)?,
                frames: args.duration(3, 14)?,
            }
        }
        "shield_until_manual" => marker(&args, Action::ShieldUntilManual)?,
        "shield_until_block" => {
            args.exact(2)?;
            Action::ShieldUntilBlock(args.duration(1, 90)?)
        }
        "if_facing_ledge" | "if_back_to_ledge" => marker(
            &args,
            Action::IfFacingLedge {
                away: name == "if_back_to_ledge",
            },
        )?,
        "if_facing_opponent" | "if_back_to_opponent" => marker(
            &args,
            Action::IfFacingOpponent {
                away: name == "if_back_to_opponent",
            },
        )?,
        "else_facing" => marker(&args, Action::ElseFacing)?,
        "end_facing" => marker(&args, Action::EndFacing)?,
        "face" => {
            args.exact(2)?;
            Action::Face(args.direction(1)?)
        }
        "walk" => walk(&args)?,
        "run_to_ledge" => run_to_ledge(&args)?,
        "run" => {
            args.exact(3)?;
            Action::Run {
                direction: args.direction(1)?,
                units: args.float(2, 0.0, 100.0)?,
            }
        }
        "walk_to_opponent" | "drift_to_opponent" => approach(&args)?,
        "drift" | "drift_for" => {
            args.exact(4)?;
            Action::Drift {
                direction: args.direction(1)?,
                strength: args.float(2, 0.0, 1.0)?,
                frames: args.duration(3, 1)?,
            }
        }
        "range" => {
            args.exact(2)?;
            Action::Range(range_check(args.parts[1], token)?)
        }
        "grab" => return grab(&args, sequence),
        _ => return named_move(&args, sequence),
    };
    push(sequence, action)
}

/// Zero-argument markers such as `end_facing`.
fn marker(args: &Args, action: Action) -> Result<Action, String> {
    args.exact(1)?;
    Ok(action)
}

/// Returns the `wait` result.
/// Gets the wait result.
fn wait(args: &Args) -> Result<Action, String> {
    args.at_most(2)?;
    Ok(Action::Wait(args.frames_or(1, 1)?))
}

/// Returns the `check` result.
/// Gets the check result.
fn check(args: &Args) -> Result<Action, String> {
    let condition = args
        .parts
        .get(1)
        .and_then(|v| Condition::from_token(v))
        .ok_or_else(|| format!("{}: unknown check condition", args.token))?;
    let wait_index = if condition.has_distance() { 3 } else { 2 };
    if args.len() < wait_index || args.len() > wait_index + 1 {
        return Err(format!("invalid check {}", args.token));
    }
    let value = if condition.has_distance() {
        args.float(2, 0.0, 100.0)?
    } else {
        0.0
    };
    Ok(Action::Check {
        condition,
        value,
        wait: args.frames_or(wait_index, 0)?,
    })
}

/// Returns the `tap` result.
/// Gets the tap result.
fn tap(args: &Args) -> Result<Action, String> {
    args.exact(7)?;
    let frames = parse_frames(args.parts[6])
        .ok_or_else(|| format!("{}: total frames must be 1..3600", args.token))?;
    let buttons = args.buttons(1)?;
    if buttons == 0 {
        return Err("tap requires a button".into());
    }
    Ok(Action::Tap {
        buttons,
        stick: args.stick(2)?,
        hold: args.duration(4, 1)?,
        gap: args.duration(5, 1)?,
        frames,
    })
}

/// Returns the `press` result.
/// Gets the press result.
fn press(args: &Args) -> Result<Action, String> {
    if args.len() != 2 && args.len() != 3 {
        return Err("use press:BUTTONS[:FRAMES] or hold:BUTTONS:FRAMES".into());
    }
    Ok(input(args.buttons(1)?, [0.0; 2], args.duration(2, 1)?, false))
}

/// Returns the `controller_input` result.
/// Gets the controller input result.
fn controller_input(args: &Args) -> Result<Action, String> {
    if !(7..=9).contains(&args.len()) {
        return Err(
            "use controller_input:BUTTONS:LX:LY:RX:RY:FRAMES[:LEFT_REF[:RIGHT_REF]]".into(),
        );
    }
    Ok(Action::Controller {
        buttons: args.buttons(1)?,
        stick: args.stick(2)?,
        cstick: args.stick(4)?,
        frames: args.duration(6, 4)?,
        reference: args.reference(7)?,
        cstick_reference: args.reference(8)?,
    })
}

/// Returns the `opponent_input` result.
/// Gets the opponent input result.
fn opponent_input(args: &Args) -> Result<Action, String> {
    args.exact(5)?;
    Ok(Action::OpponentInput {
        buttons: args.buttons(1)?,
        stick: args.stick(2)?,
        frames: args.duration(4, 1)?,
    })
}

/// Returns the `hold_until_manual` result.
/// Gets the hold until manual result.
fn hold_until_manual(args: &Args) -> Result<Action, String> {
    args.exact(4)?;
    let held = args.buttons(1)?;
    if held == 0 {
        return Err("hold_until_manual requires a button".into());
    }
    Ok(input(
        held,
        args.stick(2)?,
        0,
        args.parts[0] == "facing_hold_until_manual",
    ))
}

/// Returns the `walk` result.
/// Gets the walk result.
fn walk(args: &Args) -> Result<Action, String> {
    if args.len() != 4 && args.len() != 5 {
        return Err("use walk:DIRECTION:UNITS:STRENGTH[:MIN_GAP]".into());
    }
    let direction = args.direction(1)?;
    let min_gap = if args.len() == 5 {
        if direction != Direction::Opponent {
            return Err("MIN_GAP requires walk:opponent".into());
        }
        Some(args.float(4, 0.0, 60.0)?)
    } else {
        None
    };
    Ok(Action::Walk {
        direction,
        units: args.float(2, 0.0, 100.0)?,
        strength: args.float(3, 0.1, 0.95)?,
        min_gap,
    })
}

/// Runs the to ledge.
/// Runs the to ledge.
fn run_to_ledge(args: &Args) -> Result<Action, String> {
    if !(3..=6).contains(&args.len()) {
        return Err(
            "use run_to_ledge:OFFSET:STRENGTH[:TOLERANCE[:BRAKE_DISTANCE[:MAX_FRAMES]]]".into(),
        );
    }
    Ok(Action::LedgePosition {
        offset: args.float(1, 0., 200.)?,
        strength: args.float(2, 0.3, 1.)?,
        tolerance: args.float_or(3, 1., 0., 20.)?,
        brake: args.float_or(4, 6., 0., 100.)?,
        frames: match args.parts.get(5) {
            Some(value) => parse_frames(value)
                .ok_or_else(|| format!("invalid movement timeout in {}", args.token))?,
            None => 180,
        },
    })
}

/// Returns the `approach` result.
/// Gets the approach result.
fn approach(args: &Args) -> Result<Action, String> {
    args.exact(4)?;
    let airborne = args.parts[0] == "drift_to_opponent";
    Ok(Action::Approach {
        airborne,
        strength: args.float(1, 0.1, if airborne { 1.0 } else { 0.95 })?,
        frames: args.duration(2, 1)?,
        gap: args.float(3, 0.0, 60.0)?,
    })
}

/// Returns the `range_check` result.
/// Gets the range check result.
fn range_check(value: &str, token: &str) -> Result<RangeCheck, String> {
    Ok(match value {
        "ignore" => RangeCheck::Ignore,
        "fighter_range" => RangeCheck::FighterGrabRange,
        "fighter_range_instant" => RangeCheck::FighterGrabRangeInstant,
        value => RangeCheck::Verify(parse_float(value, 0.0, 60.0, token)?),
    })
}

/// `grab[:RANGE]`: an optional range gate followed by one R press.
fn grab(args: &Args, sequence: &mut Sequence) -> Result<(), String> {
    args.at_most(2)?;
    let range = match args.parts.get(1).copied().unwrap_or("ignore") {
        "no_range_check" | "unverified" => "ignore",
        "range_verified" | "verified" => "fighter_range",
        value => value,
    };
    let range = range_check(range, args.token)?;
    if range != RangeCheck::Ignore {
        push(sequence, Action::Range(range))?;
    }
    push(sequence, input(R, [0.0; 2], 1, false))
}

/// Shortcut names (`jab`, `up_b`, `short_hop_nair`, ...) that expand to a
/// facing-relative input. Returns the buttons and unit stick direction.
fn named_input(name: &str) -> Option<(u32, [f32; 2])> {
    Some(match name {
        "short_hop" | "shorthop" | "jump" => (X, [0.0; 2]),
        "full_hop" | "fullhop" => (X, [0.0; 2]),
        "shield" => (ZR, [0.0; 2]),
        "jab" | "attack" => (A, [0.0; 2]),
        "forward_tilt" | "f_tilt" | "ftilt" => (A, [0.6, 0.0]),
        "up_tilt" | "u_tilt" | "utilt" => (A, [0.0, 0.6]),
        "down_tilt" | "d_tilt" | "dtilt" => (A, [0.0, -0.6]),
        "forward_smash" | "f_smash" | "fsmash" => (A, [1.0, 0.0]),
        "up_smash" | "u_smash" | "usmash" => (A, [0.0, 1.0]),
        "down_smash" | "d_smash" | "dsmash" => (A, [0.0, -1.0]),
        "nair" | "neutral_air" | "neutral_aerial" => (A, [0.0; 2]),
        "fair" | "forward_air" | "forward_aerial" => (A, [1.0, 0.0]),
        "bair" | "back_air" | "back_aerial" => (A, [-1.0, 0.0]),
        "uair" | "up_air" | "up_aerial" => (A, [0.0, 1.0]),
        "dair" | "down_air" | "down_aerial" => (A, [0.0, -1.0]),
        "neutral_special" | "neutral_b" | "special" => (B, [0.0; 2]),
        "side_special" | "side_b" => (B, [1.0, 0.0]),
        "up_special" | "up_b" | "final_cutter" => (B, [0.0, 1.0]),
        "down_special" | "down_b" | "stone" => (B, [0.0, -1.0]),
        "dash" => (0, [1.0, 0.0]),
        "crouch" | "fast_fall" => (0, [0.0, -1.0]),
        "roll_forward" => (ZR, [1.0, 0.0]),
        "roll_backward" => (ZR, [-1.0, 0.0]),
        "spot_dodge" => (ZR, [0.0, -1.0]),
        "air_dodge" => (ZR, [0.0; 2]),
        _ => return None,
    })
}

/// Only the documented aerial spellings may take the `short_hop_` prefix.
fn pairs_with_short_hop(name: &str) -> bool {
    matches!(
        name,
        "nair"
            | "fair"
            | "bair"
            | "uair"
            | "dair"
            | "neutral_air"
            | "forward_air"
            | "back_air"
            | "up_air"
            | "down_air"
    )
}

/// The catch-all action family: optional `turnaround_` / `short_hop_` prefixes
/// around a shortcut name, `:FRAMES` and `:STRENGTH` suffixes.
fn named_move(args: &Args, sequence: &mut Sequence) -> Result<(), String> {
    args.at_most(3)?;
    let mut name = args.parts[0];
    if let Some(rest) = name
        .strip_prefix("turnaround_")
        .or_else(|| name.strip_prefix("turn_around_"))
    {
        push(sequence, Action::Face(Direction::Backward))?;
        name = rest;
    }
    let (paired, name) = name
        .strip_prefix("short_hop_")
        .or_else(|| name.strip_prefix("shorthop_"))
        .map_or((false, name), |v| (true, v));
    if name == "dash_attack" || name == "dashattack" {
        push(sequence, input(0, [1.0, 0.0], args.duration(1, 1)?, true))?;
        return push(sequence, input(A, [1.0, 0.0], args.duration(2, 1)?, true));
    }
    if let Some(&(bit, _)) = BUTTON_TOKENS.iter().find(|(_, token)| *token == name) {
        return push(sequence, input(bit, [0.0; 2], args.duration(1, 1)?, false));
    }
    let (mask, stick) =
        named_input(name).ok_or_else(|| format!("unknown action {}", args.token))?;
    if matches!(name, "full_hop" | "fullhop") && args.len() < 2 {
        return Err("full_hop requires an explicit hold duration, e.g. full_hop:4".into());
    }
    if paired && !pairs_with_short_hop(name) {
        return Err(format!("{}: short_hop_ requires an aerial name", args.token));
    }
    let strength = args.float_or(2, 1.0, 0.0, 1.0)?;
    push(
        sequence,
        input(
            mask | if paired { X } else { 0 },
            [stick[0] * strength, stick[1] * strength],
            args.duration(1, 1)?,
            true,
        ),
    )
}

/// Long button spellings; the canonical letters come from `BUTTON_TOKENS`.
fn button_alias(name: &str) -> Option<u32> {
    Some(match name {
        "attack" => A,
        "special" => B,
        "jump" => X,
        "grab" => R,
        "shield" => ZR,
        "none" | "0" => 0,
        _ => return None,
    })
}

/// Parses the float.
/// Parses the float.
fn parse_float(value: &str, min: f32, max: f32, token: &str) -> Result<f32, String> {
    let value = value
        .strip_suffix("_units")
        .or_else(|| value.strip_suffix("_unit"))
        .unwrap_or(value);
    let (raw, divisor) = value
        .strip_suffix('%')
        .or_else(|| value.strip_suffix("_percent"))
        .map_or((value, 1.0), |v| (v, 100.0));
    let value: f32 = raw
        .parse()
        .map_err(|_| format!("invalid number in {token}"))?;
    let value = value / divisor;
    if value.is_finite() && (min..=max).contains(&value) {
        Ok(value)
    } else {
        Err(format!("number out of range in {token}"))
    }
}

/// Parses the u 8.
/// Parses the u8.
fn parse_u8(value: &str, token: &str) -> Result<u8, String> {
    let value = value.strip_prefix("until_motion_frame_").unwrap_or(value);
    let value = value
        .strip_suffix("_frames")
        .or_else(|| value.strip_suffix("_frame"))
        .unwrap_or(value);
    value
        .parse()
        .map_err(|_| format!("invalid frame count in {token}"))
}

/// Whole-action frame budgets (taps, movement timeouts): 1..=3600.
fn parse_frames(value: &str) -> Option<u16> {
    value
        .parse::<u16>()
        .ok()
        .filter(|frames| (1..=MAX_SEQUENCE_FRAME_COUNT).contains(frames))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parses the supplied source into the module’s validated representation.
    fn parse(text: &str) -> Result<Vec<Action>, String> {
        parse_actions(text, 1).map(|sequence| sequence.actions().to_vec())
    }

    /// Regression test for `shortcuts_expand_to_the_documented_inputs`.
    #[test]
    fn shortcuts_expand_to_the_documented_inputs() {
        let facing = |buttons, stick, frames| input(buttons, stick, frames, true);
        for (text, expected) in [
            ("Short Hop", vec![facing(X, [0.0, 0.0], 1)]),
            ("up_b:3:50%", vec![facing(B, [0.0, 0.5], 3)]),
            ("short_hop_bair:4", vec![facing(X | A, [-1.0, 0.0], 4)]),
            (
                "turnaround_nair",
                vec![Action::Face(Direction::Backward), facing(A, [0.0, 0.0], 1)],
            ),
            (
                "dash_attack:3:2",
                vec![facing(0, [1.0, 0.0], 3), facing(A, [1.0, 0.0], 2)],
            ),
            ("zr:5", vec![input(ZR, [0.0; 2], 5, false)]),
            ("press:attack+jump", vec![input(A | X, [0.0; 2], 1, false)]),
            ("wait_5_frames", vec![Action::Wait(5)]),
            ("release", vec![Action::Wait(1)]),
            ("face_away", vec![Action::Face(Direction::AwayFromOpponent)]),
            ("grab", vec![input(R, [0.0; 2], 1, false)]),
            ("grab_no_range_check", vec![input(R, [0.0; 2], 1, false)]),
            (
                "grab:verified",
                vec![
                    Action::Range(RangeCheck::FighterGrabRange),
                    input(R, [0.0; 2], 1, false),
                ],
            ),
            (
                "grab:4",
                vec![
                    Action::Range(RangeCheck::Verify(4.0)),
                    input(R, [0.0; 2], 1, false),
                ],
            ),
            (
                "range:fighter_range_instant",
                vec![Action::Range(RangeCheck::FighterGrabRangeInstant)],
            ),
        ] {
            assert_eq!(parse(text), Ok(expected), "{text}");
        }
    }

    /// Regression test for `rejected_tokens_report_the_original_wording`.
    #[test]
    fn rejected_tokens_report_the_original_wording() {
        for (text, error) in [
            ("a,,b", "line 1: empty action"),
            ("wait:1:2", "line 1: invalid action wait:1:2"),
            ("wait:x", "line 1: invalid frame count in wait:x"),
            ("check:bogus", "line 1: check:bogus: unknown check condition"),
            ("check:enemy_within:5:1:1", "line 1: invalid check check:enemy_within:5:1:1"),
            ("check:enemy_within:101:1", "line 1: number out of range in check:enemy_within:101:1"),
            ("tap:a:0:0:1:1:0", "line 1: tap:a:0:0:1:1:0: total frames must be 1..3600"),
            ("tap:none:0:0:1:1:60", "line 1: tap requires a button"),
            ("press", "line 1: use press:BUTTONS[:FRAMES] or hold:BUTTONS:FRAMES"),
            ("hold:a:0", "line 1: hold:a:0: input duration must be 1..255"),
            ("press:a+q:1", "line 1: unknown button q"),
            ("input:a:2:0:1", "line 1: number out of range in input:a:2:0:1"),
            ("input:a:x:0:1", "line 1: invalid number in input:a:x:0:1"),
            ("hold_until_manual:none:0:0", "line 1: hold_until_manual requires a button"),
            ("walk:away:5:0.5:2", "line 1: MIN_GAP requires walk:opponent"),
            ("walk:opponent", "line 1: use walk:DIRECTION:UNITS:STRENGTH[:MIN_GAP]"),
            (
                "run_to_ledge:1",
                "line 1: use run_to_ledge:OFFSET:STRENGTH[:TOLERANCE[:BRAKE_DISTANCE[:MAX_FRAMES]]]",
            ),
            ("run_to_ledge:1:0.5:1:1:0", "line 1: invalid movement timeout in run_to_ledge:1:0.5:1:1:0"),
            ("face:sideways", "line 1: invalid direction in face:sideways"),
            ("walk_to_opponent:1.0:3:10", "line 1: number out of range in walk_to_opponent:1.0:3:10"),
            ("full_hop", "line 1: full_hop requires an explicit hold duration, e.g. full_hop:4"),
            ("short_hop_jab", "line 1: short_hop_jab: short_hop_ requires an aerial name"),
            ("a:1:2:3", "line 1: invalid action a:1:2:3"),
            ("bogus", "line 1: unknown action bogus"),
            ("end_facing", "end_facing has no matching facing condition"),
        ] {
            let error = if error.starts_with("line ") {
                error.to_owned()
            } else {
                format!("line 1: {error}")
            };
            assert_eq!(parse(text), Err(error), "{text}");
        }
    }

    /// Regression test for `the_action_budget_is_checked_after_each_expansion`.
    #[test]
    fn the_action_budget_is_checked_after_each_expansion() {
        let full = "wait:1,".repeat(MAX_ACTIONS - 1);
        assert!(parse(&format!("{full}jab")).is_ok());
        assert!(parse(&format!("{full}dash_attack"))
            .unwrap_err()
            .contains("at most 32 expanded actions"));
    }
}
