// Run the existing recovery scheduling regressions in isolation from unrelated
// legacy lib-test compilation errors, using the actual production module.
pub use kirby_assist_core::{fighter_profiles, ledge_position, ledge_trap, native_input, timing};
#[path = "../src/two_frame.rs"]
mod two_frame;

#[test]
fn held_l_recovery_cover_is_explicitly_kirby_only_and_uses_down_tilt() {
    assert!(!two_frame::TwoFrame::default().committed_input_pending());
    assert_eq!(
        two_frame::TwoFrame::default().action_name(),
        "two_frame_watch_recovery"
    );
    let base = ledge_trap::LedgeSettings {
        enabled: true,
        fighter: fighter_profiles::kirby::LEDGE_DATA,
        ..ledge_trap::LedgeSettings::default()
    };
    let kirby = two_frame::autopilot_settings(6, base).unwrap();
    assert!(kirby.two_frame && kirby.two_frame_down_tilt);
    assert_eq!(kirby.fighter.down_tilt.first, 4);
    assert!(
        !base.two_frame,
        "normal ledge responses keep their own settings"
    );
    for kind in [0, 5, 11, 20, 127] {
        assert!(two_frame::autopilot_settings(kind, base).is_none());
    }
    assert!(two_frame::autopilot_settings(
        6,
        ledge_trap::LedgeSettings {
            enabled: false,
            ..base
        }
    )
    .is_none());
}
