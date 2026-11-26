mod tests {
    use super::*;
    /// Converts a millisecond offset into a `Duration`.
    fn at(ms: u64) -> Duration {
        Duration::from_millis(ms)
    }
    /// Regression test for `holds_toggle_once_and_partial_or_glitchy_releases_do_not_rearm`.
    #[test]
    fn holds_toggle_once_and_partial_or_glitchy_releases_do_not_rearm() {
        let mut s = RuntimeToggle::new();
        assert!(!s.sample(CHORD, false, at(0)));
        assert!(!s.sample(CHORD, false, at(349)));
        assert!(s.sample(CHORD, false, at(350)));
        assert!(!s.sample(CHORD, false, at(900)));
        s.sample(0, false, at(950));
        assert!(!s.sample(CHORD, false, at(960)));
        s.sample(1 << 10, false, at(1000));
        assert!(!s.sample(CHORD, false, at(1500)));
        s.sample(0, false, at(1600));
        s.sample(0, false, at(1750));
        assert!(!s.sample(CHORD, false, at(1800)));
        assert!(s.sample(CHORD, false, at(2150)));
    }
    /// Checks whether cancel pending holds and held chords cannot carry out of battle.
    #[test]
    fn matches_cancel_pending_holds_and_held_chords_cannot_carry_out_of_battle() {
        let mut s = RuntimeToggle::new();
        s.sample(CHORD, false, at(0));
        assert!(!s.sample(CHORD, true, at(500)));
        assert!(!s.sample(CHORD, false, at(1000)));
        assert!(!s.sample(CHORD, true, at(2000)));
        s.sample(0, false, at(2100));
        s.sample(0, false, at(2250));
        s.sample(CHORD, false, at(2300));
        assert!(s.sample(CHORD, false, at(2650)));
    }
    /// Regression test for `missing_controller_samples_cancel_a_partly_completed_gesture`.
    #[test]
    fn missing_controller_samples_cancel_a_partly_completed_gesture() {
        let mut s = RuntimeToggle::new();
        s.sample(CHORD, false, at(0));
        s.disarm();
        assert!(!s.sample(CHORD, false, at(1000)));
        s.sample(0, false, at(1100));
        s.sample(0, false, at(1250));
        s.sample(CHORD, false, at(1300));
        assert!(s.sample(CHORD, false, at(1650)));
    }
}
