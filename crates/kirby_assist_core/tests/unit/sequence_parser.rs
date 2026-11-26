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
        (
            "check:bogus",
            "line 1: check:bogus: unknown check condition",
        ),
        (
            "check:enemy_within:5:1:1",
            "line 1: invalid check check:enemy_within:5:1:1",
        ),
        (
            "check:enemy_within:101:1",
            "line 1: number out of range in check:enemy_within:101:1",
        ),
        (
            "tap:a:0:0:1:1:0",
            "line 1: tap:a:0:0:1:1:0: total frames must be 1..3600",
        ),
        ("tap:none:0:0:1:1:60", "line 1: tap requires a button"),
        (
            "press",
            "line 1: use press:BUTTONS[:FRAMES] or hold:BUTTONS:FRAMES",
        ),
        (
            "hold:a:0",
            "line 1: hold:a:0: input duration must be 1..255",
        ),
        ("press:a+q:1", "line 1: unknown button q"),
        (
            "input:a:2:0:1",
            "line 1: number out of range in input:a:2:0:1",
        ),
        ("input:a:x:0:1", "line 1: invalid number in input:a:x:0:1"),
        (
            "hold_until_manual:none:0:0",
            "line 1: hold_until_manual requires a button",
        ),
        (
            "walk:away:5:0.5:2",
            "line 1: MIN_GAP requires walk:opponent",
        ),
        (
            "walk:opponent",
            "line 1: use walk:DIRECTION:UNITS:STRENGTH[:MIN_GAP]",
        ),
        (
            "run_to_ledge:1",
            "line 1: use run_to_ledge:OFFSET:STRENGTH[:TOLERANCE[:BRAKE_DISTANCE[:MAX_FRAMES]]]",
        ),
        (
            "run_to_ledge:1:0.5:1:1:0",
            "line 1: invalid movement timeout in run_to_ledge:1:0.5:1:1:0",
        ),
        (
            "face:sideways",
            "line 1: invalid direction in face:sideways",
        ),
        (
            "walk_to_opponent:1.0:3:10",
            "line 1: number out of range in walk_to_opponent:1.0:3:10",
        ),
        (
            "full_hop",
            "line 1: full_hop requires an explicit hold duration, e.g. full_hop:4",
        ),
        (
            "short_hop_jab",
            "line 1: short_hop_jab: short_hop_ requires an aerial name",
        ),
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

#[test]
fn shortcut_prefixes_and_button_arguments_cannot_be_silently_ignored() {
    for token in [
        "short_hop_dash_attack",
        "shorthop_dashattack:3:2",
        "short_hop_a",
        "turnaround_short_hop_zr:4",
        "a:1:0.5",
        "turnaround_b:2:0.5",
    ] {
        assert!(parse(token).is_err(), "accepted malformed shortcut {token}");
    }
    assert_eq!(
        parse("turnaround_short_hop_fair:3:0.5"),
        Ok(vec![
            Action::Face(Direction::Backward),
            input(X | A, [0.5, 0.0], 3, true),
        ])
    );
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
