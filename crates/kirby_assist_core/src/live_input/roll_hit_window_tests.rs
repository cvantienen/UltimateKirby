//! Existing regression cases for `live_input`. Kept outside the runtime code.
use super::LiveConfig;

#[test]
fn roll_hit_window_defaults_to_five_and_accepts_only_active_frame_counts() {
    assert_eq!(LiveConfig::default().ledge.roll_hit_window_frames, 5);
    for value in [1, 5, 13] {
        let config = LiveConfig::parse(&format!("ledge_roll_hit_window_frames = {value}")).unwrap();
        assert_eq!(config.ledge.roll_hit_window_frames, value);
    }
    for value in ["0", "14", "-1", "NaN", "2.5"] {
        assert!(LiveConfig::parse(&format!("ledge_roll_hit_window_frames = {value}")).is_err());
    }
}
