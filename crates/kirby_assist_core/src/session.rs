//! Fence cached inputs across gameplay/loading transitions, even if object IDs
//! and mapper counters are reused by the next match. No native pointers here.
use std::time::{Duration, Instant};

/// Render-side fence: cached matchmaking/GO state cannot keep UI alive after
/// local fighter callbacks stop. A mapper call alone never renews this lease.
#[derive(Default)]
pub struct MenuPresence {
    last_fighter: Option<Instant>,
}
impl MenuPresence {
    pub const fn new() -> Self {
        Self { last_fighter: None }
    }
    pub fn observe(&mut self, now: Instant) {
        self.last_fighter = Some(now);
    }
    pub fn clear(&mut self) {
        self.last_fighter = None;
    }
    pub fn fresh(&self, now: Instant) -> bool {
        self.last_fighter.is_some_and(|seen| {
            now.checked_duration_since(seen)
                .is_some_and(|age| age < Duration::from_millis(250))
        })
    }
}
/// Represents a session ticket that uniquely identifies a match session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionTicket(u64);

/// Represents the current match session, tracking readiness and epoch for session validation.
#[derive(Clone, Copy, Debug, Default)]
pub struct MatchSession {
    ready: bool,
    epoch: u64,
}
impl MatchSession {
    /// Creates a new match session with default values.
    pub const fn new() -> Self {
        Self {
            ready: false,
            epoch: 0,
        }
    }
    /// Updates the match session's readiness state.
    /// # Parameters
    /// * `ready` - The new readiness state.
    /// # Returns
    /// * `true` if the readiness state changed and cached fighter/controller/overlay state must reset.
    /// * `false` otherwise.
    pub fn update(&mut self, ready: bool) -> bool {
        let changed = self.ready != ready;
        if changed {
            self.epoch = self.epoch.wrapping_add(1);
            self.ready = ready;
        }
        changed
    }
    /// Returns the current session ticket if the match session is ready.
    /// # Returns
    /// * `Some(SessionTicket)` if the match session is ready.
    /// * `None` otherwise.
    pub fn ticket(self) -> Option<SessionTicket> {
        self.ready.then_some(SessionTicket(self.epoch))
    }
    /// Checks if the given session ticket is accepted by the current match session.
    /// # Parameters
    /// * `ticket` - The session ticket to check.
    /// # Returns
    /// * `true` if the ticket matches the current session ticket.
    /// * `false` otherwise.
    pub fn accepts(self, ticket: SessionTicket) -> bool {
        self.ticket() == Some(ticket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn leaving_elite_warmup_expires_menu_without_another_mapper_callback() {
        let now = Instant::now();
        let mut presence = MenuPresence::new();
        assert!(!presence.fresh(now));
        presence.observe(now);
        assert!(presence.fresh(now + Duration::from_millis(249)));
        // Retained GO/roster/view data does not renew fighter presence.
        for _ in 0..20 {
            assert!(!presence.fresh(now + Duration::from_millis(250)));
        }
        assert!(!presence.fresh(now + Duration::from_secs(30)));
        presence.observe(now + Duration::from_secs(31));
        assert!(presence.fresh(now + Duration::from_secs(31)));
        presence.clear();
        assert!(!presence.fresh(now + Duration::from_secs(31)));
    }
    #[test]
    fn live_fighter_observations_keep_menu_available_through_normal_input_pauses() {
        let now = Instant::now();
        let mut presence = MenuPresence::new();
        for tick in 0..600 {
            let at = now + Duration::from_millis(tick * 16);
            presence.observe(at);
            assert!(presence.fresh(at + Duration::from_millis(16)));
        }
    }
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
