use super::*;
/// Regression test for `stable_grouping_and_repeat_saves`.
#[test]
fn stable_grouping_and_repeat_saves() {
    let source = "[universal.settings]\nenabled = true\n[kirby.roll.a]\nname = First\nactions = grab\n[yoshi.neutral.a]\nactions = grab\n[kirby.settings]\nwalk_strength = 0.4\n[kirby.roll.b]\nname = Second\nactions = grab\n";
    let result = organize(source).unwrap();
    assert_eq!(organize(&result).unwrap(), result);
    assert!(result.find("[kirby.settings]").unwrap() < result.find("[kirby.roll.a]").unwrap());
    assert!(result.find("[kirby.roll.a]").unwrap() < result.find("[kirby.roll.b]").unwrap());
    assert!(result.find("[kirby.roll.b]").unwrap() < result.find("[yoshi.neutral.a]").unwrap());
    assert!(result.starts_with("# ---- UNIVERSAL SETTINGS ----"));
}

#[test]
fn organizing_preserves_properties_between_decorative_borders() {
    let source =
        format!("[kirby.neutral.a]\nstart_delay=0\n{BORDER}\nactions=wait:2,jab\n{BORDER}\n");
    let expected = crate::sequence_config::Catalog::parse(&source).unwrap();
    let organized = organize(&source).unwrap();
    assert_eq!(
        crate::sequence_config::Catalog::parse(&organized).unwrap(),
        expected
    );
    assert_eq!(organize(&organized).unwrap(), organized);
}

#[test]
fn oversized_comment_only_input_is_rejected_before_presentation() {
    let source = format!("#{}", "x".repeat(MAX_FILE_BYTES as usize));
    assert!(organize(&source).is_err());
}

/// Regression test for `decorative_headers_never_reject_a_valid_near_limit_source`.
#[test]
fn decorative_headers_never_reject_a_valid_near_limit_source() {
    let mut source = String::from("[universal.settings]\nenabled=true\n");
    let mut count = 0;
    'fighters: for fighter in ["mario", "luigi", "kirby", "yoshi", "bowser"] {
        for slot in [
            "neutral",
            "jump",
            "roll",
            "attack",
            "drop",
            "x",
            "y",
            "dpad_up",
            "dpad_down",
            "dpad_left",
            "dpad_right",
            "l3",
            "r3",
        ] {
            for i in 0..32 {
                source.push_str(&format!("[{fighter}.{slot}.option_{i}]\nname = A saved option\nstart_delay=0\nactions=up_b:1\n"));
                count += 1;
                if count == 2040 {
                    break 'fighters;
                }
            }
        }
    }
    let remaining = MAX_FILE_BYTES as usize - source.len();
    source.push('#');
    source.extend(std::iter::repeat('x').take(remaining - 2));
    source.push('\n');
    assert_eq!(source.len() as u64, MAX_FILE_BYTES);
    crate::sequence_config::persist::validate_output(&source).unwrap();
    assert_eq!(organize(&source).unwrap(), source);
    assert_eq!(for_save(&source, source.clone()).unwrap(), source);
    source.push('x');
    assert!(organize(&source).is_err());
}
