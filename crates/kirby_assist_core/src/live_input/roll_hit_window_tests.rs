use super::LiveConfig;
#[test]
fn legacy_roll_window_is_not_a_move_scheduler_anymore() {
    for value in ["0", "1", "13", "99", "retired"] {
        assert!(LiveConfig::parse(&format!("ledge_roll_hit_window_frames={value}")).is_ok());
    }
}
