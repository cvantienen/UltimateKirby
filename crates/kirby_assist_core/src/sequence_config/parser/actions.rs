//! Parse action-list tokens into the fixed-size recipe representation.
use super::words_id;
use crate::sequence_config::{
    Action, Condition, Direction, InputReference, RangeCheck, Sequence, MAX_ACTIONS,
};

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

fn input(buttons: u32, stick: [f32; 2], frames: u8, relative: bool) -> Action {
    Action::Input {
        buttons,
        stick,
        frames,
        relative,
    }
}

fn buttons(value: &str) -> Result<u32, String> {
    use crate::native_input::*;
    let mut mask = 0;
    for button in value.split('+') {
        mask |= match button {
            "a" | "attack" => MAPPER_A,
            "b" | "special" => MAPPER_B,
            "x" | "jump" => MAPPER_X,
            "y" => MAPPER_Y,
            "l" => MAPPER_L,
            "r" | "grab" => MAPPER_R,
            "zl" => MAPPER_ZL,
            "zr" | "shield" => MAPPER_ZR,
            "none" | "0" => 0,
            _ => return Err(format!("unknown button {button}")),
        };
    }
    Ok(mask)
}

fn parse_action(token: &str, sequence: &mut Sequence) -> Result<(), String> {
    use crate::native_input::{
        MAPPER_A as A, MAPPER_B as B, MAPPER_R as R, MAPPER_X as X, MAPPER_ZR as ZR,
    };
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
    let normalized = if let Some(wait) = normalized.strip_prefix("wait_") {
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
    };
    let parts: Vec<&str> = normalized.split(':').collect();
    let name = parts[0];
    let exact = |count| {
        if parts.len() == count {
            Ok(())
        } else {
            Err(format!("invalid action {token}"))
        }
    };
    let duration = |index: usize, default: u8| -> Result<u8, String> {
        let frames = match parts.get(index) {
            Some(value) => parse_u8(value, token)?,
            None => default,
        };
        if frames == 0 {
            Err(format!("{token}: input duration must be 1..255"))
        } else {
            Ok(frames)
        }
    };
    match name {
        "wait" | "release" => {
            if parts.len() > 2 {
                return Err(format!("invalid action {token}"));
            }
            push(
                sequence,
                Action::Wait(if parts.len() == 2 {
                    parse_u8(parts[1], token)?
                } else {
                    1
                }),
            )
        }
        "check" | "require" => {
            let condition = parts
                .get(1)
                .and_then(|v| Condition::from_token(v))
                .ok_or_else(|| format!("{token}: unknown check condition"))?;
            let wait_index = if condition.has_distance() { 3 } else { 2 };
            if parts.len() < wait_index || parts.len() > wait_index + 1 {
                return Err(format!("invalid check {token}"));
            }
            let value = if condition.has_distance() {
                parse_float(parts[2], 0.0, 100.0, token)?
            } else {
                0.0
            };
            let wait = parts
                .get(wait_index)
                .map_or(Ok(0), |v| parse_u8(v, token))?;
            push(
                sequence,
                Action::Check {
                    condition,
                    value,
                    wait,
                },
            )
        }
        "tap" => {
            exact(7)?;
            let frames = parts[6]
                .parse::<u16>()
                .ok()
                .filter(|v| (1..=3600).contains(v))
                .ok_or_else(|| format!("{token}: total frames must be 1..3600"))?;
            let tap_buttons = buttons(parts[1])?;
            if tap_buttons == 0 {
                return Err("tap requires a button".into());
            }
            push(
                sequence,
                Action::Tap {
                    buttons: tap_buttons,
                    stick: [
                        parse_float(parts[2], -1., 1., token)?,
                        parse_float(parts[3], -1., 1., token)?,
                    ],
                    hold: duration(4, 1)?,
                    gap: duration(5, 1)?,
                    frames,
                },
            )
        }
        "press" | "hold" => {
            if parts.len() != 2 && parts.len() != 3 {
                return Err(format!("use press:BUTTONS[:FRAMES] or hold:BUTTONS:FRAMES"));
            }
            push(
                sequence,
                input(buttons(parts[1])?, [0.0; 2], duration(2, 1)?, false),
            )
        }
        "controller_input" => {
            if !(7..=9).contains(&parts.len()) {
                return Err(
                    "use controller_input:BUTTONS:LX:LY:RX:RY:FRAMES[:LEFT_REF[:RIGHT_REF]]".into(),
                );
            }
            push(
                sequence,
                Action::Controller {
                    buttons: buttons(parts[1])?,
                    stick: [
                        parse_float(parts[2], -1., 1., token)?,
                        parse_float(parts[3], -1., 1., token)?,
                    ],
                    cstick: [
                        parse_float(parts[4], -1., 1., token)?,
                        parse_float(parts[5], -1., 1., token)?,
                    ],
                    frames: duration(6, 4)?,
                    reference: InputReference::parse(parts.get(7).copied().unwrap_or("screen"))?,
                    cstick_reference: InputReference::parse(
                        parts.get(8).copied().unwrap_or("screen"),
                    )?,
                },
            )
        }
        "opponent_input" => {
            exact(5)?;
            push(
                sequence,
                Action::OpponentInput {
                    buttons: buttons(parts[1])?,
                    stick: [
                        parse_float(parts[2], -1.0, 1.0, token)?,
                        parse_float(parts[3], -1.0, 1.0, token)?,
                    ],
                    frames: duration(4, 1)?,
                },
            )
        }
        "input" | "facing_input" => {
            exact(5)?;
            push(
                sequence,
                input(
                    buttons(parts[1])?,
                    [
                        parse_float(parts[2], -1.0, 1.0, token)?,
                        parse_float(parts[3], -1.0, 1.0, token)?,
                    ],
                    duration(4, 1)?,
                    name == "facing_input",
                ),
            )
        }
        "cstick" => {
            exact(4)?;
            push(
                sequence,
                Action::CStick {
                    stick: [
                        parse_float(parts[1], -1.0, 1.0, token)?,
                        parse_float(parts[2], -1.0, 1.0, token)?,
                    ],
                    frames: duration(3, 1)?,
                },
            )
        }
        "stick" => {
            exact(4)?;
            push(
                sequence,
                input(
                    0,
                    [
                        parse_float(parts[1], -1.0, 1.0, token)?,
                        parse_float(parts[2], -1.0, 1.0, token)?,
                    ],
                    duration(3, 1)?,
                    false,
                ),
            )
        }
        "hold_until_manual" | "facing_hold_until_manual" => {
            exact(4)?;
            let held = buttons(parts[1])?;
            if held == 0 {
                return Err("hold_until_manual requires a button".into());
            }
            push(
                sequence,
                input(
                    held,
                    [
                        parse_float(parts[2], -1., 1., token)?,
                        parse_float(parts[3], -1., 1., token)?,
                    ],
                    0,
                    name == "facing_hold_until_manual",
                ),
            )
        }
        "chase_if_far" => {
            exact(4)?;
            push(
                sequence,
                Action::Chase {
                    gap: parse_float(parts[1], 0., 100., token)?,
                    strength: parse_float(parts[2], 0., 1., token)?,
                    frames: duration(3, 14)?,
                },
            )
        }
        "shield_until_manual" => {
            exact(1)?;
            push(sequence, Action::ShieldUntilManual)
        }
        "shield_until_block" => {
            exact(2)?;
            push(sequence, Action::ShieldUntilBlock(duration(1, 90)?))
        }
        "if_facing_ledge" | "if_back_to_ledge" => {
            exact(1)?;
            push(
                sequence,
                Action::IfFacingLedge {
                    away: name == "if_back_to_ledge",
                },
            )
        }
        "if_facing_opponent" | "if_back_to_opponent" => {
            exact(1)?;
            push(
                sequence,
                Action::IfFacingOpponent {
                    away: name == "if_back_to_opponent",
                },
            )
        }
        "else_facing" => {
            exact(1)?;
            push(sequence, Action::ElseFacing)
        }
        "end_facing" => {
            exact(1)?;
            push(sequence, Action::EndFacing)
        }
        "face" => {
            exact(2)?;
            push(sequence, Action::Face(parse_direction(parts[1], token)?))
        }
        "walk" => {
            if parts.len() != 4 && parts.len() != 5 {
                return Err(format!("use walk:DIRECTION:UNITS:STRENGTH[:MIN_GAP]"));
            }
            let direction = parse_direction(parts[1], token)?;
            let min_gap = if parts.len() == 5 {
                if direction != Direction::Opponent {
                    return Err("MIN_GAP requires walk:opponent".into());
                }
                Some(parse_float(parts[4], 0.0, 60.0, token)?)
            } else {
                None
            };
            push(
                sequence,
                Action::Walk {
                    direction,
                    units: parse_float(parts[2], 0.0, 100.0, token)?,
                    strength: parse_float(parts[3], 0.1, 0.95, token)?,
                    min_gap,
                },
            )
        }
        "run_to_ledge" => {
            if !(3..=6).contains(&parts.len()) {
                return Err(
                    "use run_to_ledge:OFFSET:STRENGTH[:TOLERANCE[:BRAKE_DISTANCE[:MAX_FRAMES]]]"
                        .into(),
                );
            }
            push(
                sequence,
                Action::LedgePosition {
                    offset: parse_float(parts[1], 0., 200., token)?,
                    strength: parse_float(parts[2], 0.3, 1., token)?,
                    tolerance: parts
                        .get(3)
                        .map_or(Ok(1.), |v| parse_float(v, 0., 20., token))?,
                    brake: parts
                        .get(4)
                        .map_or(Ok(6.), |v| parse_float(v, 0., 100., token))?,
                    frames: parts.get(5).map_or(Ok(180), |v| {
                        v.parse::<u16>()
                            .ok()
                            .filter(|n| (1..=3600).contains(n))
                            .ok_or_else(|| format!("invalid movement timeout in {token}"))
                    })?,
                },
            )
        }
        "run" => {
            exact(3)?;
            push(
                sequence,
                Action::Run {
                    direction: parse_direction(parts[1], token)?,
                    units: parse_float(parts[2], 0.0, 100.0, token)?,
                },
            )
        }
        "walk_to_opponent" | "drift_to_opponent" => {
            exact(4)?;
            push(
                sequence,
                Action::Approach {
                    airborne: name == "drift_to_opponent",
                    strength: parse_float(
                        parts[1],
                        0.1,
                        if name == "drift_to_opponent" {
                            1.0
                        } else {
                            0.95
                        },
                        token,
                    )?,
                    frames: duration(2, 1)?,
                    gap: parse_float(parts[3], 0.0, 60.0, token)?,
                },
            )
        }
        "drift" | "drift_for" => {
            exact(4)?;
            push(
                sequence,
                Action::Drift {
                    direction: parse_direction(parts[1], token)?,
                    strength: parse_float(parts[2], 0.0, 1.0, token)?,
                    frames: duration(3, 1)?,
                },
            )
        }
        "range" => {
            exact(2)?;
            let range = match parts[1] {
                "ignore" => RangeCheck::Ignore,
                "fighter_range" => RangeCheck::FighterGrabRange,
                "fighter_range_instant" => RangeCheck::FighterGrabRangeInstant,
                value => RangeCheck::Verify(parse_float(value, 0.0, 60.0, token)?),
            };
            push(sequence, Action::Range(range))
        }
        "grab" => {
            if parts.len() > 2 {
                return Err(format!("invalid action {token}"));
            }
            let range = match parts.get(1).copied().unwrap_or("ignore") {
                "ignore" | "no_range_check" | "unverified" => RangeCheck::Ignore,
                "fighter_range" | "range_verified" | "verified" => RangeCheck::FighterGrabRange,
                "fighter_range_instant" => RangeCheck::FighterGrabRangeInstant,
                v => RangeCheck::Verify(parse_float(v, 0.0, 60.0, token)?),
            };
            if range != RangeCheck::Ignore {
                push(sequence, Action::Range(range))?;
            }
            push(sequence, input(R, [0.0; 2], 1, false))
        }
        _ => {
            if parts.len() > 3 {
                return Err(format!("invalid action {token}"));
            }
            let name = if let Some(name) = name
                .strip_prefix("turnaround_")
                .or_else(|| name.strip_prefix("turn_around_"))
            {
                push(sequence, Action::Face(Direction::Backward))?;
                name
            } else {
                name
            };
            let (paired, name) = name
                .strip_prefix("short_hop_")
                .or_else(|| name.strip_prefix("shorthop_"))
                .map_or((false, name), |v| (true, v));
            if name == "dash_attack" || name == "dashattack" {
                push(sequence, input(0, [1.0, 0.0], duration(1, 1)?, true))?;
                return push(sequence, input(A, [1.0, 0.0], duration(2, 1)?, true));
            }
            if matches!(name, "a" | "b" | "x" | "y" | "l" | "r" | "zl" | "zr") {
                return push(
                    sequence,
                    input(buttons(name)?, [0.0; 2], duration(1, 1)?, false),
                );
            }
            let (mask, stick, default) = match name {
                "short_hop" | "shorthop" | "jump" => (X, [0.0; 2], 1),
                "full_hop" | "fullhop" => {
                    if parts.len() < 2 {
                        return Err(
                            "full_hop requires an explicit hold duration, e.g. full_hop:4".into(),
                        );
                    }
                    (X, [0.0; 2], 1)
                }
                "shield" => (ZR, [0.0; 2], 1),
                "jab" | "attack" => (A, [0.0; 2], 1),
                "forward_tilt" | "f_tilt" | "ftilt" => (A, [0.6, 0.0], 1),
                "up_tilt" | "u_tilt" | "utilt" => (A, [0.0, 0.6], 1),
                "down_tilt" | "d_tilt" | "dtilt" => (A, [0.0, -0.6], 1),
                "forward_smash" | "f_smash" | "fsmash" => (A, [1.0, 0.0], 1),
                "up_smash" | "u_smash" | "usmash" => (A, [0.0, 1.0], 1),
                "down_smash" | "d_smash" | "dsmash" => (A, [0.0, -1.0], 1),
                "nair" | "neutral_air" | "neutral_aerial" => (A, [0.0; 2], 1),
                "fair" | "forward_air" | "forward_aerial" => (A, [1.0, 0.0], 1),
                "bair" | "back_air" | "back_aerial" => (A, [-1.0, 0.0], 1),
                "uair" | "up_air" | "up_aerial" => (A, [0.0, 1.0], 1),
                "dair" | "down_air" | "down_aerial" => (A, [0.0, -1.0], 1),
                "neutral_special" | "neutral_b" | "special" => (B, [0.0; 2], 1),
                "side_special" | "side_b" => (B, [1.0, 0.0], 1),
                "up_special" | "up_b" | "final_cutter" => (B, [0.0, 1.0], 1),
                "down_special" | "down_b" | "stone" => (B, [0.0, -1.0], 1),
                "dash" => (0, [1.0, 0.0], 1),
                "crouch" | "fast_fall" => (0, [0.0, -1.0], 1),
                "roll_forward" => (ZR, [1.0, 0.0], 1),
                "roll_backward" => (ZR, [-1.0, 0.0], 1),
                "spot_dodge" => (ZR, [0.0, -1.0], 1),
                "air_dodge" => (ZR, [0.0; 2], 1),
                _ => return Err(format!("unknown action {token}")),
            };
            if paired
                && !matches!(
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
            {
                return Err(format!("{token}: short_hop_ requires an aerial name"));
            }
            let strength = if parts.len() == 3 {
                parse_float(parts[2], 0.0, 1.0, token)?
            } else {
                1.0
            };
            let stick = [stick[0] * strength, stick[1] * strength];
            push(
                sequence,
                input(
                    mask | if paired { X } else { 0 },
                    stick,
                    duration(1, default)?,
                    true,
                ),
            )
        }
    }
}

fn parse_direction(value: &str, token: &str) -> Result<Direction, String> {
    match value {
        "opponent" | "enemy" | "toward" | "towards" | "toward_opponent" | "towards_opponent"
        | "toward_enemy" | "towards_enemy" => Ok(Direction::Opponent),
        "away" | "away_from_opponent" | "away_from_enemy" => Ok(Direction::AwayFromOpponent),
        "ledge" => Ok(Direction::Ledge),
        "saved_ledge" | "toward_saved_ledge" => Ok(Direction::SavedLedge),
        "away_from_ledge" | "away_from_saved_ledge" => Ok(Direction::AwayFromLedge),
        "inward" => Ok(Direction::Inward),
        "outward" => Ok(Direction::Outward),
        "facing" | "forward" => Ok(Direction::Facing),
        "backward" | "back" => Ok(Direction::Backward),
        _ => Err(format!("invalid direction in {token}")),
    }
}

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
