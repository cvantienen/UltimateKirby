//! Deliberate master-switch gesture. Match detection is supplied by the adapter.
use std::time::Duration;
pub const CHORD: u64 = (1 << 10) | (1 << 11);
const HOLD: Duration = Duration::from_millis(350);
const RELEASE: Duration = Duration::from_millis(150);
pub struct RuntimeToggle {
    armed: bool,
    held_since: Option<Duration>,
    released_since: Option<Duration>,
}
impl RuntimeToggle {
    pub const fn new() -> Self {
        Self {
            armed: true,
            held_since: None,
            released_since: None,
        }
    }
    pub fn disarm(&mut self) {
        self.armed = false;
        self.held_since = None;
        self.released_since = None;
    }
    pub fn sample(&mut self, buttons: u64, in_match: bool, now: Duration) -> bool {
        let held = buttons & CHORD;
        if in_match {
            self.disarm();
            return false;
        }
        if held == 0 {
            self.held_since = None;
            let since = *self.released_since.get_or_insert(now);
            if now.saturating_sub(since) >= RELEASE {
                self.armed = true;
            }
            return false;
        }
        self.released_since = None;
        if !self.armed || held != CHORD {
            self.held_since = None;
            return false;
        }
        let since = *self.held_since.get_or_insert(now);
        if now.saturating_sub(since) < HOLD {
            return false;
        }
        self.armed = false;
        self.held_since = None;
        true
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn at(ms: u64) -> Duration {
        Duration::from_millis(ms)
    }
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
