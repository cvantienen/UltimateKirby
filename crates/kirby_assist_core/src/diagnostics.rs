//! Coalesce unchanged diagnostic snapshots before allocating/formatting strings.
//! Native action edges remain immediate; wall-clock samples explain long waits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiagnosticEmission {
    pub periodic: bool,
    pub coalesced_updates: u32,
}

pub struct DiagnosticGate<K> {
    last: Option<K>,
    last_emission_ms: Option<u64>,
    coalesced: u32,
}
impl<K: PartialEq> DiagnosticGate<K> {
    pub const fn new() -> Self {
        Self {
            last: None,
            last_emission_ms: None,
            coalesced: 0,
        }
    }
    pub fn sample(&mut self, key: K, now_ms: u64, periodic: bool) -> Option<DiagnosticEmission> {
        let changed = self.last.as_ref() != Some(&key);
        let sample_due = periodic
            && self
                .last_emission_ms
                .is_none_or(|last| now_ms.saturating_sub(last) >= 500 || now_ms < last);
        if changed || sample_due {
            let emission = DiagnosticEmission {
                periodic: !changed,
                coalesced_updates: self.coalesced,
            };
            self.last = Some(key);
            self.last_emission_ms = Some(now_ms);
            self.coalesced = 0;
            Some(emission)
        } else {
            self.coalesced = self.coalesced.saturating_add(1);
            None
        }
    }
}
impl<K: PartialEq> Default for DiagnosticGate<K> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn waiting_frames_are_coalesced_but_attack_and_hit_events_are_immediate() {
        let mut gate = DiagnosticGate::new();
        assert!(!gate.sample("roll_wait", 0, true).unwrap().periodic);
        for frame in 1..30 {
            assert!(gate.sample("roll_wait", frame * 16, true).is_none());
        }
        let attack = gate.sample("smash_request", 480, true).unwrap();
        assert!(!attack.periodic);
        assert_eq!(attack.coalesced_updates, 29);
        assert!(!gate.sample("hit_confirmed", 496, true).unwrap().periodic);
    }
    #[test]
    fn verbose_waits_emit_at_most_two_periodic_snapshots_per_second() {
        let mut gate = DiagnosticGate::new();
        let emissions = (0..=1000)
            .filter_map(|ms| gate.sample(1, ms, true))
            .collect::<Vec<_>>();
        assert_eq!(emissions.len(), 3); // Initial event, then 500ms and 1000ms.
        assert_eq!(emissions.iter().filter(|event| event.periodic).count(), 2);
        assert_eq!(emissions[1].coalesced_updates, 499);
    }
    #[test]
    fn nonverbose_mode_only_logs_events_and_reset_rearms_first_snapshot() {
        let mut gate = DiagnosticGate::new();
        assert!(gate.sample(1, 0, false).is_some());
        assert!(gate.sample(1, 10_000, false).is_none());
        assert!(gate.sample(2, 10_001, false).is_some());
        gate = DiagnosticGate::new();
        assert!(gate.sample(2, 10_002, false).is_some());
    }
}
