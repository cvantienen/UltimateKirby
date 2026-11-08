#[path = "../../../src/live/fighters/bowser/up_b.rs"]
mod up_b;

use kirby_assist_core::{fighter_kinds, native_states::status};

#[test]
fn committed_bowser_spin_reserves_observation_only_with_receipt_or_native_startup() {
    let bowser = fighter_kinds::BOWSER;
    let kirby = fighter_kinds::KIRBY;
    assert!(up_b::spin_follow(
        bowser,
        true,
        true,
        status::WAIT,
        status::WAIT
    ));
    assert!(up_b::spin_follow(
        bowser,
        true,
        false,
        status::SPECIAL_HI,
        status::WAIT
    ));
    assert!(!up_b::spin_follow(
        bowser,
        true,
        false,
        status::WAIT,
        status::WAIT
    ));
    assert!(!up_b::spin_follow(
        bowser,
        false,
        true,
        status::WAIT,
        status::WAIT
    ));
    assert!(!up_b::spin_follow(
        kirby,
        true,
        true,
        status::SPECIAL_HI,
        status::WAIT
    ));
    assert_eq!(up_b::input_epoch(bowser, 5), Some(5));
    assert_eq!(up_b::input_epoch(kirby, 5), None);
    assert!(up_b::retain_rules(bowser, true));
    assert!(!up_b::retain_rules(kirby, true));
    assert!(!up_b::retain_rules(bowser, false));
    assert!(up_b::reserve_receipt(Some(bowser), true));
    assert!(!up_b::reserve_receipt(Some(kirby), true));
    assert!(!up_b::reserve_receipt(None, true));
    assert!(!up_b::reserve_receipt(Some(bowser), false));
}
