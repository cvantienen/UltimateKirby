//! Shared callback state and its session-reset/cancellation rules.
use super::*;
use kirby_assist_core::kirby::{FairFollowup, TripFollowup};

// One session for the configured controller, rather than one global session per
// fighter. `cancel` clears assistance without changing the game's native state.
pub(super) struct State {
    pub(super) settings_menu: Option<settings_menu::SettingsMenu>,
    // UI identity/lifetime survive combat cancellation and respawn. They never
    // authorize an input sequence; fighter_kind retains that separate role.
    pub(super) menu_fighter_kind: Option<i32>,
    pub(super) menu_battle_active: bool,
    // Intro permission is tied to a real local ENTRY object and fresh route,
    // then discarded at GO. Retained CSS roster data cannot reopen the menu.
    pub(super) menu_intro_object: Option<u32>,
    pub(super) session: MatchSession,
    // Select a physical controller, then follow its actual mapper slot. Arena
    // participant slots must not be confused with local controller identity.
    pub(super) mapping_index: Option<i32>,
    // Human fighter IDs learned from live command callbacks, indexed by mapper
    // slot. No accessor is retained and the mapper never enumerates the roster.
    pub(super) human_routes: [Option<(u32, Instant)>; 8],
    // Roster IDs are discovered only by native callbacks; synchronous mapper
    // refreshes revalidate these IDs instead of enumerating a loading roster.
    pub(super) ledge_roster: Option<(kirby_assist_core::battle_objects::FighterRoster, i64)>,
    // Our mapper-call counter, NOT a verified simulation-frame or HID clock.
    pub(super) raw_tick: i64,
    pub(super) raw_seen: Option<Instant>,
    pub(super) raw_buttons: u32,
    pub(super) manual_jump_fence: kirby_assist_core::manual_controls::ManualJumpFence,
    pub(super) grab_handoff: kirby_assist_core::grab_handoff::GrabHandoff,
    pub(super) raw_connected: bool,
    // Option permits lazy construction and a clean reset by assigning None.
    pub(super) policy: Option<RawAutoControls>,
    pub(super) animation_clock: Option<AnimationClock>,
    pub(super) tech_input: Option<TechInput>,
    pub(super) tech_floor_motion: kirby_assist_core::tech_prediction::FloorMotionHistory,
    pub(super) tech_output: TechOutput,
    pub(super) tech_urgent: bool,
    pub(super) last_mapped_guard: bool,
    pub(super) last_mapped_stick: Option<[f32; 2]>,
    pub(super) policy_tick: Option<i64>,
    // Separate state for side-D-pad consumption and temporary left-stick control.
    pub(super) turbo: Option<TurboState>,
    pub(super) dash_pivot: Option<DashPivotAssist>,
    pub(super) manual_context: Option<ManualContext>,
    pub(super) posture: Option<kirby_assist_core::posture::Posture>,
    pub(super) posture_target: Option<f32>,
    pub(super) posture_down: bool,
    pub(super) posture_side_aliases: kirby_assist_core::posture::TiltSideAliases,
    pub(super) posture_floor_safe: bool,
    pub(super) manual_overlay: Option<ManualOverlay>,
    // Observation-derived eligibility, not just whether a trigger is held.
    // Turbo/ledge must also respect grounded movement and native busy states.
    pub(super) turbo_ready: bool,
    pub(super) action_owner: ActionOwnership,
    #[cfg(feature = "diagnostic-logging")]
    pub(super) last_movement_blocker: Option<Owner>,
    #[cfg(feature = "diagnostic-logging")]
    pub(super) last_input_expired: bool,
    pub(super) fair_followup: Option<FairFollowup>,
    pub(super) fair_output: ControllerOutput,
    pub(super) trip_followup: Option<TripFollowup>,
    pub(super) trip_output: ControllerOutput,
    pub(super) jab_lock: Option<kirby_assist_core::jab_lock::JabLock>,
    pub(super) jab_lock_output: ControllerOutput,
    pub(super) sdi: Option<SdiState>,
    pub(super) sdi_context: Option<SdiContext>,
    pub(super) di: Option<kirby_assist_core::auto_di::AutoDi>,
    pub(super) di_context: Option<kirby_assist_core::auto_di::Context>,
    pub(super) damage_trajectory: Option<kirby_assist_core::auto_di::Trajectory>,
    pub(super) damage_geometry: Option<(u32, u64, Option<kirby_assist_core::auto_di::BlastZone>)>,
    pub(super) ledge_walk_ready: bool,
    pub(super) ledge_walk_motion: Option<([f32; 2], [f32; 2])>,
    pub(super) ledge: Option<LedgeTrap>,
    pub(super) ledge_visits: kirby_assist_core::ledge_trap::LedgeVisits,
    pub(super) ledge_active: bool,
    pub(super) ledge_seek_tick: Option<i64>,
    pub(super) trump_seek_tick: Option<i64>,
    pub(super) ledge_position: Option<LedgePosition>,
    pub(super) ledge_output: ControllerOutput,
    pub(super) oos: kirby_assist_core::out_of_shield::OutOfShield,
    pub(super) oos_output: ControllerOutput,
    pub(super) autopilot_guarding: bool,
    pub(super) guard_hold: Option<kirby_assist_core::autopilot_guard::GuardHold>,
    pub(super) duck_defense: Option<kirby_assist_core::auto_duck::DuckDefense>,
    pub(super) duck_active: bool,
    pub(super) duck_output: ControllerOutput,
    pub(super) reflector: kirby_assist_core::projectile_reflect::Reflector,
    pub(super) reflect_output: ControllerOutput,
    pub(super) spot_dodge: Option<kirby_assist_core::spot_dodge::SpotDodge>,
    pub(super) spot_output: ControllerOutput,
    pub(super) trump: Option<LedgeTrump>,
    pub(super) trump_output: ControllerOutput,
    // Cached with the same timestamp as requested; never used after expiry.
    pub(super) mash_stick: Option<[f32; 2]>,
    pub(super) request_seen: Option<Instant>,
    pub(super) rule_engine: Option<Box<kirby_assist_core::reaction_rules::Engine>>,
    pub(super) roll_read_output: ControllerOutput,
    pub(super) observed_object: Option<u32>,
    // Store a game object ID, never an accessor pointer that could become stale.
    pub(super) associated_object: Option<u32>,
    // Native kind for the same validated controller/fighter association.
    pub(super) fighter_kind: Option<i32>,
    // Repeated game queries within one observed mapper update must not mash
    // faster, consume cooldown faster, or issue an extra tech press.
    pub(super) evaluated_tick: Option<i64>,
    pub(super) observation_setup: kirby_assist_core::observation::ObservationSetup,
    // Native observation order is separate from mapper polling frequency.
    pub(super) observation_sequence: u64,
    #[cfg(feature = "diagnostic-logging")]
    pub(super) last_logged_scenario: (bool, bool),
    #[cfg(feature = "diagnostic-logging")]
    pub(super) mapper_confirmed: bool,
    #[cfg(feature = "diagnostic-logging")]
    pub(super) buttons_modified: bool,
    #[cfg(feature = "diagnostic-logging")]
    pub(super) button_mod_logged: bool,
    // Last game-visible buttons, distinct from the restored physical memory.
    // This tells the tech policy whether it must release ZR before re-pressing.
    pub(super) last_output_buttons: u32,
    #[cfg(feature = "diagnostic-logging")]
    pub(super) last_tech_diagnostic: Option<(i32, i32, &'static str)>,
    #[cfg(feature = "diagnostic-logging")]
    pub(super) pending_tech_log: Option<String>,
    #[cfg(feature = "diagnostic-logging")]
    pub(super) pending_combat_log: Option<String>,
    #[cfg(feature = "diagnostic-logging")]
    pub(super) last_combat_diagnostic: (bool, bool, u32),
    #[cfg(feature = "diagnostic-logging")]
    pub(super) ledge_diagnostics: DiagnosticGate<LedgeDiagnostic>,
    #[cfg(feature = "diagnostic-logging")]
    pub(super) mode_diagnostics: kirby_assist_core::diagnostics::modes::Recorder,
    #[cfg(feature = "diagnostic-logging")]
    pub(super) mode_log_origin: Option<Instant>,
    #[cfg(feature = "diagnostic-logging")]
    pub(super) mode_hit: Option<bool>,
    #[cfg(feature = "diagnostic-logging")]
    pub(super) last_di_diagnostic: Option<kirby_assist_core::auto_di::Decision>,
    #[cfg(feature = "diagnostic-logging")]
    pub(super) ledge_log_origin: Option<Instant>,
    #[cfg(feature = "diagnostic-logging")]
    pub(super) pending_ledge_log: Option<String>,
    #[cfg(feature = "diagnostic-logging")]
    pub(super) grab_trace: crate::grab_diagnostics::GrabTrace,
    #[cfg(feature = "diagnostic-logging")]
    pub(super) calibration: crate::calibration::Recorder,
}
impl State {
    /// Keep edge histories while discarding automatic plans under ledge priority.
    pub(super) fn suspend_automatic_punishes(&mut self) {
        self.reflector.cancel();
        self.reflect_output = ControllerOutput::default();
        if let Some(engine) = self.rule_engine.as_mut() {
            engine.cancel();
        }
        self.roll_read_output = ControllerOutput::default();
        self.oos.suspend_automatic();
        self.oos_output = ControllerOutput::default();
        if let Some(spot) = self.spot_dodge.as_mut() {
            spot.cancel_pending();
            spot.suspend_response();
        }
        self.spot_output = ControllerOutput::default();
        if let Some(tool) = self.trip_followup.as_mut() {
            tool.cancel_pending();
        }
        if let Some(tool) = self.fair_followup.as_mut() {
            tool.cancel_pending();
        }
        if let Some(tool) = self.jab_lock.as_mut() {
            tool.cancel_pending();
        }
        self.trip_output = ControllerOutput::default();
        self.fair_output = ControllerOutput::default();
        self.jab_lock_output = ControllerOutput::default();
    }
    pub(super) fn cancel_non_owner(&mut self, keep: Option<Owner>) {
        if keep != Some(Owner::Safety) {
            if let Some(tech) = self.tech_input.as_mut() {
                tech.cancel_request();
            }
            self.tech_output = TechOutput::default();
            self.tech_urgent = false;
        }
        if keep != Some(Owner::Reflect) {
            self.reflector.cancel();
            self.reflect_output = ControllerOutput::default();
        }
        if keep.is_some_and(|owner| owner != Owner::Turbo) {
            self.turbo = None;
            if let Some(tool) = self.dash_pivot.as_mut() {
                tool.cancel_pending();
            }
        }
        if keep != Some(Owner::JabLock) && self.jab_lock_output.owned != 0 {
            if let Some(tool) = self.jab_lock.as_mut() {
                tool.cancel_pending();
            }
            self.jab_lock_output = ControllerOutput::default();
        }
        if keep != Some(Owner::Trip) {
            if self.trip_output.owned != 0 {
                if let Some(tool) = self.trip_followup.as_mut() {
                    tool.cancel_pending();
                }
            }
            self.trip_output = ControllerOutput::default();
        }
        if keep != Some(Owner::Fair) {
            if self.fair_output.owned != 0 {
                if let Some(tool) = self.fair_followup.as_mut() {
                    tool.cancel_pending();
                }
            }
            self.fair_output = ControllerOutput::default();
        }
        if keep != Some(Owner::RollRead) {
            if let Some(rules) = self.rule_engine.as_mut() {
                rules.cancel();
            }
            self.roll_read_output = ControllerOutput::default();
        }
        if !matches!(keep, Some(Owner::Ledge | Owner::Defense)) {
            // A preempted test is cancelled, never rearmed on the next mapper poll.
            if self
                .settings_menu
                .as_ref()
                .is_some_and(|m| m.preview_slot().is_some())
            {
                if let Some(menu) = self.settings_menu.as_mut() {
                    menu.finish_preview();
                }
                self.ledge = None;
            }
            if self.ledge_output.owned != 0
                || self
                    .ledge
                    .as_ref()
                    .is_some_and(LedgeTrap::needs_response_observation)
            {
                #[cfg(feature = "diagnostic-logging")]
                {
                    self.grab_trace.reset("action_owner_preempted");
                    self.calibration
                        .reset("action owner preempted ledge response");
                }
                self.ledge = None;
            }
            self.ledge_output = ControllerOutput::default();
        }
        if keep != Some(Owner::OutOfShield)
            && (self.oos_output.owned != 0 || self.oos.target().is_some())
        {
            self.oos.cancel_pending();
            self.oos_output = ControllerOutput::default();
        }
        if keep != Some(Owner::Defense) {
            self.autopilot_guarding = false;
            self.guard_hold = None;
            self.duck_defense = None;
            self.duck_output = ControllerOutput::default();
        }
        if keep != Some(Owner::OptionRead) {
            if let Some(spot) = self.spot_dodge.as_mut() {
                if keep != Some(Owner::GrabDodge) {
                    spot.cancel_pending();
                }
                if self.spot_output.owned != 0 || spot.punish_pending() {
                    spot.cancel_response();
                }
            }
            if keep != Some(Owner::GrabDodge) {
                self.spot_output = ControllerOutput::default();
            }
        }
        if keep != Some(Owner::Trump) {
            if let Some(tool) = self.trump.as_mut() {
                tool.cancel_pending();
            }
            self.trump_output = ControllerOutput::default();
        }
    }
    // `const fn` allows the global Mutex to be initialized before startup.
    // Options begin as None; get_or_insert_with in the callbacks constructs
    // each helper only when first needed. No game pointer is created here.
    pub(super) const fn new() -> Self {
        Self {
            reflector: kirby_assist_core::projectile_reflect::Reflector::new(),
            reflect_output: ControllerOutput {
                consume_trigger: false,
                requested: 0,
                owned: 0,
                stick: None,
                native_short_hop: false,
            },
            settings_menu: None,
            menu_fighter_kind: None,
            menu_battle_active: false,
            menu_intro_object: None,
            session: MatchSession::new(),
            mapping_index: None,
            human_routes: [None; 8],
            ledge_roster: None,
            raw_tick: 0,
            raw_seen: None,
            raw_buttons: 0,
            manual_jump_fence: kirby_assist_core::manual_controls::ManualJumpFence::new(),
            grab_handoff: kirby_assist_core::grab_handoff::GrabHandoff::new(),
            raw_connected: false,
            policy: None,
            animation_clock: None,
            tech_input: None,
            tech_floor_motion: kirby_assist_core::tech_prediction::FloorMotionHistory::new(),
            tech_output: TechOutput {
                press: false,
                owns_guard: false,
            },
            tech_urgent: false,
            last_mapped_guard: false,
            last_mapped_stick: None,
            policy_tick: None,
            turbo: None,
            dash_pivot: None,
            manual_context: None,
            posture: None,
            posture_target: None,
            posture_down: false,
            posture_side_aliases: Default::default(),
            posture_floor_safe: false,

            manual_overlay: None,
            turbo_ready: false,
            action_owner: ActionOwnership::new(),
            #[cfg(feature = "diagnostic-logging")]
            last_movement_blocker: None,
            #[cfg(feature = "diagnostic-logging")]
            last_input_expired: false,
            fair_followup: None,
            fair_output: ControllerOutput {
                consume_trigger: false,
                stick: None,
                requested: 0,
                owned: 0,
                native_short_hop: false,
            },
            trip_followup: None,
            jab_lock: None,
            jab_lock_output: ControllerOutput {
                consume_trigger: false,
                requested: 0,
                owned: 0,
                stick: None,
                native_short_hop: false,
            },
            trip_output: ControllerOutput {
                consume_trigger: false,
                stick: None,
                requested: 0,
                owned: 0,
                native_short_hop: false,
            },
            sdi: None,
            sdi_context: None,
            di: None,
            di_context: None,
            damage_trajectory: None,
            damage_geometry: None,
            ledge_walk_ready: false,
            ledge_walk_motion: None,
            ledge: None,
            ledge_visits: kirby_assist_core::ledge_trap::LedgeVisits::new(),
            ledge_active: false,
            ledge_seek_tick: None,
            trump_seek_tick: None,
            ledge_position: None,
            trump: None,
            trump_output: ControllerOutput {
                consume_trigger: false,
                stick: None,
                requested: 0,
                owned: 0,
                native_short_hop: false,
            },
            oos: kirby_assist_core::out_of_shield::OutOfShield::new(),
            oos_output: ControllerOutput {
                consume_trigger: false,
                stick: None,
                requested: 0,
                owned: 0,
                native_short_hop: false,
            },
            autopilot_guarding: false,
            guard_hold: None,
            duck_defense: None,
            duck_active: false,
            spot_dodge: None,
            spot_output: ControllerOutput {
                consume_trigger: false,
                stick: None,
                requested: 0,
                owned: 0,
                native_short_hop: false,
            },
            duck_output: ControllerOutput {
                consume_trigger: false,
                stick: None,
                requested: 0,
                owned: 0,
                native_short_hop: false,
            },
            ledge_output: ControllerOutput {
                consume_trigger: false,
                stick: None,
                requested: 0,
                owned: 0,
                native_short_hop: false,
            },
            mash_stick: None,
            request_seen: None,
            rule_engine: None,
            roll_read_output: ControllerOutput {
                consume_trigger: false,
                stick: None,
                requested: 0,
                owned: 0,
                native_short_hop: false,
            },
            observed_object: None,
            associated_object: None,
            fighter_kind: None,
            evaluated_tick: None,
            observation_setup: kirby_assist_core::observation::ObservationSetup::new(),
            observation_sequence: 0,
            #[cfg(feature = "diagnostic-logging")]
            last_logged_scenario: (false, false),
            #[cfg(feature = "diagnostic-logging")]
            mapper_confirmed: false,
            #[cfg(feature = "diagnostic-logging")]
            buttons_modified: false,
            #[cfg(feature = "diagnostic-logging")]
            button_mod_logged: false,
            last_output_buttons: 0,
            #[cfg(feature = "diagnostic-logging")]
            last_tech_diagnostic: None,
            #[cfg(feature = "diagnostic-logging")]
            pending_tech_log: None,
            #[cfg(feature = "diagnostic-logging")]
            pending_combat_log: None,
            #[cfg(feature = "diagnostic-logging")]
            last_combat_diagnostic: (false, false, 0),
            #[cfg(feature = "diagnostic-logging")]
            ledge_diagnostics: DiagnosticGate::new(),
            #[cfg(feature = "diagnostic-logging")]
            mode_diagnostics: kirby_assist_core::diagnostics::modes::Recorder::new(),
            #[cfg(feature = "diagnostic-logging")]
            mode_log_origin: None,
            #[cfg(feature = "diagnostic-logging")]
            mode_hit: None,
            #[cfg(feature = "diagnostic-logging")]
            last_di_diagnostic: None,
            #[cfg(feature = "diagnostic-logging")]
            ledge_log_origin: None,
            #[cfg(feature = "diagnostic-logging")]
            pending_ledge_log: None,
            #[cfg(feature = "diagnostic-logging")]
            grab_trace: crate::grab_diagnostics::GrabTrace::new(),
            #[cfg(feature = "diagnostic-logging")]
            calibration: crate::calibration::Recorder::new(),
        }
    }
    // Menu choices replace settings; do not let a previous configuration keep
    // an old punish/movement proposal queued. Navigation alone never calls this.
    pub(super) fn cancel_manual_tools(&mut self) {
        self.reflector.cancel();
        self.reflect_output = ControllerOutput::default();
        // A menu change invalidates native facts/settings already being read.
        // Do not let the old observation publish after the new choice is active.
        self.observation_sequence = self.observation_sequence.wrapping_add(1);
        self.mash_stick = None;
        self.request_seen = None;
        self.oos = Default::default();
        self.oos_output = ControllerOutput::default();
        // Clear the proposal, not tech's acknowledged-edge/cooldown history.
        // The next observation applies the current enabled/tuning settings.
        if let Some(tech) = self.tech_input.as_mut() {
            tech.cancel_request();
        }
        self.tech_output = TechOutput::default();
        self.tech_urgent = false;
        self.posture = None;
        self.posture_target = None;
        self.posture_down = false;
        self.posture_side_aliases = Default::default();
        self.posture_floor_safe = false;
        self.action_owner = ActionOwnership::new();
        #[cfg(feature = "diagnostic-logging")]
        {
            self.last_movement_blocker = None;
            self.last_input_expired = false;
        }
        self.guard_hold = None;
        self.duck_defense = None;
        self.duck_active = false;
        self.duck_output = ControllerOutput::default();
        self.spot_dodge = None;
        self.spot_output = ControllerOutput::default();
        self.rule_engine = None;
        self.roll_read_output = ControllerOutput::default();

        self.autopilot_guarding = false;
        #[cfg(feature = "diagnostic-logging")]
        self.grab_trace.reset("menu_choice_or_manual_tool_changed");
        #[cfg(feature = "diagnostic-logging")]
        self.calibration.reset("menu choice/manual tool changed");
        self.ledge = None;
        self.ledge_output = ControllerOutput::default();
        self.ledge_position = None;
        self.ledge_seek_tick = None;
        self.trump_seek_tick = None;
        self.trump = None;
        self.trump_output = ControllerOutput::default();
        self.fair_followup = None;
        self.fair_output = ControllerOutput::default();
        self.trip_followup = None;
        self.trip_output = ControllerOutput::default();
        self.jab_lock = None;
        self.jab_lock_output = ControllerOutput::default();
        self.turbo = None;
        self.dash_pivot = None;
    }
    pub(super) fn update_match(&mut self, ready: bool) -> Option<SessionTicket> {
        if self.session.update(ready) || !ready {
            #[cfg(feature = "diagnostic-logging")]
            crate::performance::health::event(crate::performance::health::Event::StateReset);
            #[cfg(feature = "diagnostic-logging")]
            self.grab_trace.reset("match_session_ended");
            #[cfg(feature = "diagnostic-logging")]
            self.calibration.reset("match session ended");
            let session = self.session;
            let mut settings_menu = self.settings_menu.take();
            if let Some(settings_menu) = settings_menu.as_mut() {
                settings_menu.invalidate();
            }
            let menu_fighter_kind = self.menu_fighter_kind;
            let menu_battle_active = self.menu_battle_active;
            let menu_intro_object = self.menu_intro_object;
            *self = Self::new();
            self.session = session;
            self.settings_menu = settings_menu;
            self.menu_fighter_kind = menu_fighter_kind;
            self.menu_battle_active = menu_battle_active;
            self.menu_intro_object = menu_intro_object;
        }
        self.session.ticket()
    }
    // Use for actual cancellation/reassignment. Hitstop deliberately does NOT
    // call this: resetting there would forget cooldown and input history.
    pub(super) fn cancel(&mut self) {
        self.reflector = kirby_assist_core::projectile_reflect::Reflector::new();
        self.reflect_output = ControllerOutput::default();
        self.action_owner = ActionOwnership::new();
        #[cfg(feature = "diagnostic-logging")]
        {
            self.last_movement_blocker = None;
            self.last_input_expired = false;
        }
        self.guard_hold = None;
        self.duck_defense = None;
        self.duck_active = false;
        self.duck_output = ControllerOutput::default();
        self.spot_dodge = None;
        self.spot_output = ControllerOutput::default();
        self.rule_engine = None;
        self.roll_read_output = ControllerOutput::default();

        self.autopilot_guarding = false;
        #[cfg(feature = "diagnostic-logging")]
        self.grab_trace.reset("session_or_controller_cancelled");
        #[cfg(feature = "diagnostic-logging")]
        self.calibration.reset("session/controller cancelled");
        if let Some(menu) = self.settings_menu.as_mut() {
            menu.invalidate();
        }
        self.observation_sequence = self.observation_sequence.wrapping_add(1);
        self.ledge_roster = None;
        self.mash_stick = None;
        self.request_seen = None;
        self.observed_object = None;
        self.fighter_kind = None;
        self.evaluated_tick = None;
        self.observation_setup.clear();
        self.policy = None;
        self.animation_clock = None;
        self.oos = Default::default();
        self.oos_output = ControllerOutput::default();
        self.tech_input = None;
        self.tech_floor_motion = Default::default();
        self.tech_output = TechOutput::default();
        self.tech_urgent = false;
        self.last_mapped_guard = false;
        self.last_mapped_stick = None;
        self.policy_tick = None;
        self.turbo = None;
        self.dash_pivot = None;
        self.manual_context = None;
        self.posture = None;
        self.posture_target = None;
        self.posture_down = false;
        self.posture_side_aliases = Default::default();
        self.posture_floor_safe = false;
        self.manual_overlay = None;
        self.manual_jump_fence = kirby_assist_core::manual_controls::ManualJumpFence::new();
        self.grab_handoff = Default::default();
        self.turbo_ready = false;
        self.fair_followup = None;
        self.fair_output = ControllerOutput::default();
        self.trip_followup = None;
        self.trip_output = ControllerOutput::default();
        self.jab_lock = None;
        self.jab_lock_output = ControllerOutput::default();
        self.sdi = None;
        self.sdi_context = None;
        self.di = None;
        self.di_context = None;
        self.damage_trajectory = None;
        self.damage_geometry = None;
        self.ledge_walk_ready = false;
        self.ledge_walk_motion = None;
        self.ledge = None;
        self.ledge_active = false;
        self.ledge_seek_tick = None;
        self.trump_seek_tick = None;
        self.ledge_position = None;
        self.ledge_output = ControllerOutput::default();
        self.trump = None;
        self.trump_output = ControllerOutput::default();
        self.last_output_buttons = 0;
        #[cfg(feature = "diagnostic-logging")]
        {
            self.last_tech_diagnostic = None;
            self.pending_tech_log = None;
            self.pending_combat_log = None;
            self.last_combat_diagnostic = (false, false, 0);
            self.ledge_diagnostics = DiagnosticGate::new();
            self.mode_diagnostics = kirby_assist_core::diagnostics::modes::Recorder::new();
            self.mode_log_origin = None;
            self.mode_hit = None;
            self.last_di_diagnostic = None;
            self.ledge_log_origin = None;
            self.pending_ledge_log = None;
        }
    }
}

// Expired native observations invalidate the old controller association.
