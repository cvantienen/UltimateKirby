//! Fence cached inputs across gameplay/loading transitions, even if object IDs
//! and mapper counters are reused by the next match. No native pointers here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionTicket(u64);

#[derive(Clone, Copy, Debug, Default)]
pub struct MatchSession {
    ready: bool,
    epoch: u64,
}
impl MatchSession {
    pub const fn new() -> Self {
        Self {
            ready: false,
            epoch: 0,
        }
    }
    /// Returns true when cached fighter/controller/overlay state must reset.
    pub fn update(&mut self, ready: bool) -> bool {
        let changed = self.ready != ready;
        if changed {
            self.epoch = self.epoch.wrapping_add(1);
            self.ready = ready;
        }
        changed
    }
    pub fn ticket(self) -> Option<SessionTicket> {
        self.ready.then_some(SessionTicket(self.epoch))
    }
    pub fn accepts(self, ticket: SessionTicket) -> bool {
        self.ticket() == Some(ticket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn warmup_to_loading_to_match_rejects_old_observations_even_with_reused_ids() {
        let mut session = MatchSession::new();
        assert!(session.ticket().is_none());
        assert!(session.update(true));
        let warmup_observation = session.ticket().unwrap();
        assert!(session.accepts(warmup_observation));
        assert!(!session.update(true));
        assert!(session.update(false));
        assert!(!session.accepts(warmup_observation));
        assert!(session.ticket().is_none());
        assert!(!session.update(false));
        assert!(session.update(true));
        let match_observation = session.ticket().unwrap();
        assert!(!session.accepts(warmup_observation));
        assert!(session.accepts(match_observation));
    }
    #[test]
    fn repeated_gameplay_polls_do_not_reset_current_action_history() {
        let mut session = MatchSession::new();
        session.update(true);
        let ticket = session.ticket().unwrap();
        for _ in 0..100 {
            assert!(!session.update(true));
            assert!(session.accepts(ticket));
        }
        session.update(false);
        session.update(true);
        assert!(!session.accepts(ticket));
    }
}
