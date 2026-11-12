//! Generic adapter runner for teaching, host tests, and future synchronous tools.
//! The real Switch integration does NOT call this runner: it coordinates game
//! queries and raw mapper input through its own shared State in src/live.rs.
//! `impl GameAdapter` accepts any type implementing the adapter traits.
use crate::{AssistConfig, AssistEngine, FrameDecision, GameAdapter, InputPermissions};

/// Runtime Smash data carried by `AssistRuntime`.
#[derive(Clone, Debug, Default)]
pub struct AssistRuntime {
/// Runtime config value used by AssistRuntime..
    pub config: AssistConfig,
/// Runtime permissions value used by AssistRuntime..
    pub permissions: InputPermissions,
}

impl AssistRuntime {
    /// Call once per simulation frame, using fresh human input rather than the
    /// previous merged output. The adapter owns frame counters and resets them
    /// on scenario re-entry. Missing fighter state releases to fresh human input when available.
    pub fn process_frame(&self, adapter: &mut impl GameAdapter) -> Option<FrameDecision> {
        // `?` unwraps Some(value), or returns None immediately if a read fails.
        // A missing human input read cannot safely reconstruct physical input;
        // adapters retaining output need a watchdog for that particular path.
        let human = adapter.read_human_input()?;
        let Some((context, observation)) = adapter.read_context().zip(adapter.observe_fighter())
        else {
            // Fresh physical input is available: release any previous recipe
            // even when a fighter observation/context disappears during loading.
            adapter.write_merged_input(human);
            return None;
        };
        // &self reads configuration; &mut adapter permits the later output
        // write. The engine returns a value rather than holding that mutable
        // borrow across observation or input application.
        let decision =
            AssistEngine.evaluate(self.config, context, observation, human, self.permissions);
        // Write the fresh human frame on waiting/disabled frames too, so an
        // earlier assist pulse is released instead of remaining latched.
        adapter.write_merged_input(decision.proposed);
        Some(decision)
    }
}

// Compiled only by cargo test on the host; these assertions are not live
// Switch logging or gameplay callbacks. Add regressions here for new behavior.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AutoMash, ButtonSet, ControllerFrame, FighterKind, FighterObservation,
        GameObservationSource, InputBridge, RunContext, Scenario,
    };

/// Runtime Smash data carried by `Adapter`.
    struct Adapter {
/// Runtime observation value used by Adapter..
        observation: Option<FighterObservation>,
/// Runtime human value used by Adapter..
        human: Option<ControllerFrame>,
/// Runtime context value used by Adapter..
        context: Option<RunContext>,
/// Runtime writes value used by Adapter..
        writes: Vec<ControllerFrame>,
    }

    impl GameObservationSource for Adapter {
        /// Observes the fighter.
        fn observe_fighter(&self) -> Option<FighterObservation> {
            self.observation
        }
    }
    impl InputBridge for Adapter {
        /// Reads the human input.
        fn read_human_input(&self) -> Option<ControllerFrame> {
            self.human
        }
        /// Writes the merged input.
        fn write_merged_input(&mut self, frame: ControllerFrame) {
            self.writes.push(frame);
        }
    }
    impl GameAdapter for Adapter {
        /// Reads the context.
        fn read_context(&self) -> Option<RunContext> {
            self.context
        }
    }

    /// Regression test for `runner_releases_pulses_and_handles_disable_exit_and_missing_state`.
    #[test]
    fn runner_releases_pulses_and_handles_disable_exit_and_missing_state() {
        let human = ControllerFrame {
            buttons: ButtonSet::JUMP,
            ..ControllerFrame::default()
        };
        let mut adapter = Adapter {
            observation: Some(FighterObservation::one(Scenario::Clatter, 0)),
            human: Some(human),
            context: Some(RunContext {
                is_locally_human_controlled: true,
                fighter: FighterKind::Kirby,
            }),
            writes: Vec::new(),
        };
        let mut runtime = AssistRuntime::default();
        runtime.config.enabled = true;
        assert!(runtime.config.set_rule(AutoMash::default().rule()));
        runtime.process_frame(&mut adapter).unwrap();
        assert_eq!(
            adapter.writes.last().unwrap().buttons,
            ButtonSet::JUMP.union(ButtonSet::ATTACK)
        );
        adapter.observation = Some(FighterObservation::one(Scenario::Clatter, 1));
        runtime.process_frame(&mut adapter).unwrap();
        assert_eq!(adapter.writes.last(), Some(&human));
        // A new scenario occurrence starts its timing at zero again.
        adapter.observation = Some(FighterObservation::one(Scenario::Clatter, 0));
        runtime.process_frame(&mut adapter).unwrap();
        assert!(adapter
            .writes
            .last()
            .unwrap()
            .buttons
            .contains(ButtonSet::ATTACK));
        runtime.config.enabled = false;
        runtime.process_frame(&mut adapter).unwrap();
        assert_eq!(adapter.writes.last(), Some(&human));
        runtime.config.enabled = true;
        adapter.observation = Some(FighterObservation::EMPTY);
        runtime.process_frame(&mut adapter).unwrap();
        assert_eq!(adapter.writes.last(), Some(&human));

        let count = adapter.writes.len();
        adapter.observation = None;
        assert!(runtime.process_frame(&mut adapter).is_none());
        adapter.observation = Some(FighterObservation::EMPTY);
        adapter.human = None;
        assert!(runtime.process_frame(&mut adapter).is_none());
        adapter.human = Some(human);
        adapter.context = None;
        assert!(runtime.process_frame(&mut adapter).is_none());
        assert_eq!(adapter.writes.len(), count + 2);
        assert_eq!(adapter.writes.last(), Some(&human));
    }
}
pub mod runtime_toggle {
    //! Deliberate master-switch gesture. Match detection is supplied by the adapter.
    use std::time::Duration;
/// `CHORD` is a configuration value used by the Smash runtime.
    pub const CHORD: u64 = (1 << 10) | (1 << 11);
/// `HOLD` is a configuration value used by the Smash runtime.
    const HOLD: Duration = Duration::from_millis(350);
/// `RELEASE` is a configuration value used by the Smash runtime.
    const RELEASE: Duration = Duration::from_millis(150);
/// Runtime Smash data carried by `RuntimeToggle`.
    pub struct RuntimeToggle {
/// Runtime armed value used by RuntimeToggle..
        armed: bool,
/// Runtime held since value used by RuntimeToggle..
        held_since: Option<Duration>,
/// Runtime released since value used by RuntimeToggle..
        released_since: Option<Duration>,
    }
    impl RuntimeToggle {
        /// Constructs a new value with the type’s initial state.
        pub const fn new() -> Self {
            Self {
                armed: true,
                held_since: None,
                released_since: None,
            }
        }
        /// Clears the armed state and resets the hold and release timers.
        pub fn disarm(&mut self) {
            self.armed = false;
            self.held_since = None;
            self.released_since = None;
        }
        /// Updates the toggle state from the current button sample and match state.
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
            self.disarm();
            true
        }
    }
    #[cfg(test)]
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
}
pub mod session {
    //! Fence cached inputs across gameplay/loading transitions, even if object IDs
    //! and mapper counters are reused by the next match. No native pointers here.
    use std::time::{Duration, Instant};

    /// Render-side fence: cached matchmaking/GO state cannot keep UI alive after
    /// local fighter callbacks stop. A mapper call alone never renews this lease.
    #[derive(Default)]
    pub struct MenuPresence {
/// Runtime last fighter value used by MenuPresence..
        last_fighter: Option<Instant>,
    }
    impl MenuPresence {
        /// Constructs a new value with the type’s initial state.
        pub const fn new() -> Self {
            Self { last_fighter: None }
        }
        /// Records the time of the latest fighter observation.
        pub fn observe(&mut self, now: Instant) {
            self.last_fighter = Some(now);
        }
        /// Clears the remembered fighter observation.
        pub fn clear(&mut self) {
            self.last_fighter = None;
        }
        /// Checks whether the last fighter observation is within the freshness window.
        pub fn fresh(&self, now: Instant) -> bool {
            self.last_fighter.is_some_and(|seen| {
                now.checked_duration_since(seen)
                    .is_some_and(|age| age < Duration::from_millis(250))
            })
        }
    }
    /// Represents a session ticket that uniquely identifies a match session.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SessionTicket(
        /// Opaque numeric identity for one match session.
        u64,
    );

    /// Represents the current match session, tracking readiness and epoch for session validation.
    #[derive(Clone, Copy, Debug, Default)]
    pub struct MatchSession {
/// Runtime ready value used by MatchSession..
        ready: bool,
/// Runtime epoch value used by MatchSession..
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
        pub fn update(&mut self, ready: bool) -> bool {
            let changed = self.ready != ready;
            if changed {
                self.epoch = self.epoch.wrapping_add(1);
                self.ready = ready;
            }
            changed
        }
        /// Returns the current session ticket if the match session is ready.
        pub fn ticket(self) -> Option<SessionTicket> {
            self.ready.then_some(SessionTicket(self.epoch))
        }
        /// Checks if the given session ticket is accepted by the current match session.
        /// # Parameters
        /// * `ticket` - The session ticket to check.
        pub fn accepts(self, ticket: SessionTicket) -> bool {
            self.ticket() == Some(ticket)
        }
    }

    #[cfg(test)]
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
}
pub mod training_test {
    //! Training compatibility in both builds: reserve Minus for OUR menu and hand the
    //! complete controller back to Training for its own menus/save/record shortcuts.
    //! No Training-private state, virtual L press or reduced gameplay config.
    use crate::{
        ledge_trap::DPAD_DOWN,
        live_input::LiveConfig,
        manual_controls::DPAD_UP,
        native_input::{MAPPER_B, MAPPER_PLUS, MAPPER_ZL, MAPPER_ZR},
        settings_menu::{menu_toggle_pressed, monitor_chord},
    };

    /// Training compatibility applies only when it is installed and in use.
    pub fn active(installed: bool, training: bool) -> bool {
        installed && training
    }

    /// The overlay chord, Plus, or one of `pad_buttons` held with D-pad up/down.
    fn shortcut(buttons: u32, pad_buttons: u32) -> bool {
        monitor_chord(buttons)
            || buttons & MAPPER_PLUS != 0
            || buttons & pad_buttons != 0 && buttons & (DPAD_UP | DPAD_DOWN) != 0
    }

    /// Whether the whole controller belongs to Training this frame: a Training
    /// shortcut is held, or an earlier handoff (`already`) has not been ended
    /// by a fresh Minus press.
    pub fn passthrough(buttons: u32, pressed: u32, already: bool) -> bool {
        // Persist the handoff across releases: a Training menu/recording can stay
        // open after its shortcut is released. Fresh Minus resumes and toggles ours.
        shortcut(buttons, MAPPER_B | MAPPER_ZL | MAPPER_ZR)
            || (already && !menu_toggle_pressed(buttons, pressed))
    }

    /// Default ZL+Up/Down save/load is momentary. Menus, recording/playback and
    /// OS-overlay shortcuts can stay open after release and require a Minus resume.
    pub fn sticky_shortcut(buttons: u32) -> bool {
        shortcut(buttons, MAPPER_B | MAPPER_ZR)
    }

    /// Minus is the shared menu event in every build, including Training.
    pub fn policy_buttons(physical: u32) -> u32 {
        physical
    }

    /// All gameplay features retain the same startup/menu settings in Training.
    pub fn settings(config: LiveConfig, _test_mode: bool) -> LiveConfig {
        config
    }
}
