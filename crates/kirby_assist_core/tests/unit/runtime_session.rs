mod tests {
    use super::*;
    /// Regression test for `leaving_elite_warmup_expires_menu_without_another_mapper_callback`.
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
    /// Regression test for `live_fighter_observations_keep_menu_available_through_normal_input_pauses`.
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
    /// Regression test for `warmup_to_loading_to_match_rejects_old_observations_even_with_reused_ids`.
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
    /// Regression test for `repeated_gameplay_polls_do_not_reset_current_action_history`.
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
