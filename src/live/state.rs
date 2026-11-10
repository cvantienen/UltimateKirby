//! Shared callback state and its session-reset/cancellation rules.
//! Technical: Holds mutable policy/session snapshots shared by callbacks and
//! implements their cancellation and reset semantics.
//! Plain English: This is the runtime's shared memory for the current player
//! session and active helpers.
#[cfg(feature = "diagnostic-logging")]
use super::observation::LedgeDiagnostic;
use super::*;
use kirby_assist_core::kirby::{FairFollowup, TripFollowup};

// One session for the configured controller, rather than one global session per
// fighter. `cancel` clears assistance without changing the game's native state.
/// Aggregates menu, session, input, and helper state for callback sharing.
///
///  Stores everything the currently selected player session needs.
pub(super) struct State {
    /// Lazily created menu state for the selected fighter.
    ///
    ///  The current in-game menu and its selection.
    pub(super) settings_menu: Option<settings_menu::SettingsMenu>,
    // UI identity/lifetime survive combat cancellation and respawn. They never
    // authorize an input sequence; fighter_kind retains that separate role.
    /// Fighter kind used for menu identity, independent of input authorization.
    ///
    ///  Character currently represented by the menu.
    pub(super) menu_fighter_kind: Option<i32>,
    /// Whether menu rendering is in an active battle context.
    ///
    ///  Shows whether the match menu is currently active.
    pub(super) menu_battle_active: bool,
    // Intro permission is tied to a real local ENTRY object and fresh route,
    // then discarded at GO. Retained CSS roster data cannot reopen the menu.
    /// Validated object ID that can authorize an intro menu.
    ///
    ///  The entry fighter allowed to open the menu before the match starts.
    pub(super) menu_intro_object: Option<u32>,
    /// Match-session generation and lifecycle state.
    ///
    ///  Tracks which match this assistance state belongs to.
    pub(super) session: MatchSession,
    // Select a physical controller, then follow its actual mapper slot. Arena
    // participant slots must not be confused with local controller identity.
    /// Selected mapper index for the configured physical controller.
    ///
    ///  The controller slot Kirby Assist follows.
    pub(super) mapping_index: Option<i32>,
    // Human fighter IDs learned from live command callbacks, indexed by mapper
    // slot. No accessor is retained and the mapper never enumerates the roster.
    /// Callback-discovered object IDs and observation times, indexed by mapper slot.
    ///
    ///  Recently seen human fighters for each controller.
    pub(super) human_routes: [Option<(u32, Instant)>; 8],
    // Roster IDs are discovered only by native callbacks; synchronous mapper
    // refreshes revalidate these IDs instead of enumerating a loading roster.
    /// Copied bounded roster and source observation tick.
    ///
    ///  Recent fighter IDs used by ledge checks.
    pub(super) ledge_roster: Option<(kirby_assist_core::battle_objects::FightersInGame, i64)>,
    // Our mapper-call counter, NOT a verified simulation-frame or HID clock.
    /// Local mapper-call counter, not a simulation-frame clock.
    ///
    ///  Counts controller polling updates.
    pub(super) raw_tick: i64,
    /// Wall-clock time of the last physical input sample.
    ///
    ///  When the latest controller state was seen.
    pub(super) raw_seen: Option<Instant>,
    /// Last raw held-button bitfield from the physical controller.
    ///
    ///  Which physical buttons were held most recently.
    pub(super) raw_buttons: u32,
    /// Tracks manual jump ownership and startup fencing.
    ///
    ///  Prevents a real jump from being mistaken for an automatic one.
    pub(super) manual_jump_fence: kirby_assist_core::manual_controls::ManualJumpFence,
    /// Tracks native grab-to-follow-up handoff state.
    ///
    ///  Remembers the current grab sequence and its timing.
    pub(super) grab_handoff: kirby_assist_core::grab_handoff::GrabHandoff,
    /// Whether the selected physical controller reported connected.
    ///
    ///  Shows whether the controller is plugged in/available.
    pub(super) raw_connected: bool,
    // Option permits lazy construction and a clean reset by assigning None.
    /// Optional raw-button policy state for the selected controller.
    ///
    ///  Stores automatic button-policy history when needed.
    pub(super) policy: Option<RawAutoControls>,
    /// Native animation-progress clock for deduplicating repeated polls.
    ///
    ///  Tracks animation time so polling does not count as extra frames.
    pub(super) animation_clock: Option<AnimationClock>,
    /// Tech-input edge and acknowledgment scheduler.
    ///
    ///  Tracks shield presses needed for a possible tech.
    pub(super) tech_input: Option<TechInput>,
    /// Motion history used to predict floor-contact transitions.
    ///
    ///  Remembers movement needed to estimate an upcoming landing.
    pub(super) tech_floor_motion: kirby_assist_core::tech_prediction::FloorMotionHistory,
    /// Current tech scheduler output before mapper acknowledgment.
    ///
    ///  The shield request prepared for the next controller mapping.
    pub(super) tech_output: TechOutput,
    /// Marks tech output as urgent for mapper ownership arbitration.
    ///
    ///  Gives a near-immediate tech request higher priority.
    pub(super) tech_urgent: bool,
    /// Last mapped guard state used for edge handling.
    ///
    ///  Whether shield was present in the previous mapped result.
    pub(super) last_mapped_guard: bool,
    /// Last mapped left-stick vector for continuity checks.
    ///
    ///  The stick direction sent on the previous mapping.
    pub(super) last_mapped_stick: Option<[f32; 2]>,
    /// Mapper tick associated with the latest policy evaluation.
    ///
    ///  Which controller poll produced the current policy request.
    pub(super) policy_tick: Option<i64>,
    // Separate state for side-D-pad consumption and temporary left-stick control.
    /// Optional turbo-button state and repeat timing.
    ///
    ///  Tracks turbo input when it is enabled.
    pub(super) turbo: Option<TurboState>,
    /// Optional dash-pivot timing and input state.
    ///
    ///  Tracks a manual dash-to-turn assist.
    pub(super) dash_pivot: Option<DashPivotAssist>,
    /// Copied manual movement context from native observations.
    ///
    ///  The fighter action details used by manual controls.
    pub(super) manual_context: Option<ManualContext>,
    /// Posture helper state for configured crouch/tilt behavior.
    ///
    ///  Tracks posture adjustments for manual movement.
    pub(super) posture: Option<kirby_assist_core::posture::Posture>,
    /// Targeted posture value for the current helper action.
    ///
    ///  The desired posture or stick target.
    pub(super) posture_target: Option<f32>,
    /// Whether the posture helper currently consumes down input.
    ///
    ///  Shows whether down is being used for posture assistance.
    pub(super) posture_down: bool,
    /// Tracks directional aliases caused by tilt-side controls.
    ///
    ///  Remembers which side inputs are temporarily treated alike.
    pub(super) posture_side_aliases: kirby_assist_core::posture::TiltSideAliases,
    /// Whether the current posture route passed floor-safety checks.
    ///
    ///  Shows whether posture input is safe near the platform edge.
    pub(super) posture_floor_safe: bool,
    /// Optional manual button overlay applied during mapping.
    ///
    ///  Stores button changes for the current manual helper.
    pub(super) manual_overlay: Option<ManualOverlay>,
    // Observation-derived eligibility, not just whether a trigger is held.
    // Turbo/ledge must also respect grounded movement and native busy states.
    /// Observation-derived readiness gate for turbo and ledge movement.
    ///
    ///  Shows whether movement helpers may act in the current state.
    pub(super) turbo_ready: bool,
    /// Tracks exclusive action ownership and priority arbitration.
    ///
    ///  Keeps helpers from sending conflicting actions at once.
    pub(super) action_owner: ActionOwnership,
    #[cfg(feature = "diagnostic-logging")]
    /// Last diagnostic owner that blocked movement.
    ///
    ///  Records why movement assistance was held back.
    pub(super) last_movement_blocker: Option<Owner>,
    #[cfg(feature = "diagnostic-logging")]
    /// Whether the most recent diagnostic request expired before mapping.
    ///
    ///  Records if a helper request became too old to use.
    pub(super) last_input_expired: bool,
    /// Kirby forward-air follow-up policy state.
    ///
    ///  Tracks the automatic forward-air follow-up.
    pub(super) fair_followup: Option<FairFollowup>,
    /// Current forward-air follow-up controller proposal.
    ///
    ///  Buttons requested by the forward-air helper.
    pub(super) fair_output: ControllerOutput,
    /// Kirby trip follow-up policy state.
    ///
    ///  Tracks the automatic trip follow-up.
    pub(super) trip_followup: Option<TripFollowup>,
    /// Current trip follow-up controller proposal.
    ///
    ///  Buttons requested by the trip helper.
    pub(super) trip_output: ControllerOutput,
    /// SDI state machine for the active victim hitstop.
    ///
    ///  Tracks directional influence during hit pause.
    pub(super) sdi: Option<SdiState>,
    /// Copied context used by SDI sampling.
    ///
    ///  Fighter and hit details for the current SDI decision.
    pub(super) sdi_context: Option<SdiContext>,
    /// Automatic DI policy state.
    ///
    ///  Tracks directional influence choices during launch.
    pub(super) di: Option<kirby_assist_core::auto_di::AutoDi>,
    /// Copied blast-zone and launch context for DI prediction.
    ///
    ///  The stage and launch facts used by DI.
    pub(super) di_context: Option<kirby_assist_core::auto_di::Context>,
    /// Last calculated damage trajectory for the active hit.
    ///
    ///  Predicted path of the launched fighter.
    pub(super) damage_trajectory: Option<kirby_assist_core::auto_di::Trajectory>,
    /// Cached object ID, hitstop time, and copied blast-zone geometry.
    ///
    ///  Recent stage bounds reused while the same hit is being observed.
    pub(super) damage_geometry: Option<(u32, u64, Option<kirby_assist_core::auto_di::BlastZone>)>,
    /// Native-observation gate for ledge-walk assistance.
    ///
    ///  Shows whether ledge walking is currently allowed.
    pub(super) ledge_walk_ready: bool,
    /// Current and target movement vectors for ledge walking.
    ///
    ///  The planned path for moving along the ledge.
    pub(super) ledge_walk_motion: Option<([f32; 2], [f32; 2])>,
    /// Active ledge-trap response state.
    ///
    ///  Tracks the selected ledge response sequence.
    pub(super) ledge: Option<LedgeTrap>,
    /// Tracks recent ledge visits for bounded response behavior.
    ///
    ///  Remembers recent ledge interactions.
    pub(super) ledge_visits: kirby_assist_core::ledge_trap::LedgeVisits,
    /// Whether a ledge interaction is active in the current observation.
    ///
    ///  Shows whether the fighter is currently at a ledge.
    pub(super) ledge_active: bool,
    /// Mapper tick of the last ledge-edge search.
    ///
    ///  Prevents searching for a ledge spot on every poll.
    pub(super) ledge_seek_tick: Option<i64>,
    /// Mapper tick of the last trump-edge search.
    ///
    ///  Prevents repeating the ledge-trump search too often.
    pub(super) trump_seek_tick: Option<i64>,
    /// Last copied ground-ledge position and approach facts.
    ///
    ///  The recent stage location used by ledge positioning.
    pub(super) ledge_position: Option<LedgePosition>,
    /// Current ledge controller proposal.
    ///
    ///  Buttons requested by the ledge helper.
    pub(super) ledge_output: ControllerOutput,
    /// Out-of-shield response policy state.
    ///
    ///  Tracks possible actions after shielding an attack.
    pub(super) oos: kirby_assist_core::out_of_shield::OutOfShield,
    /// Current out-of-shield controller proposal.
    ///
    ///  Buttons requested by the shield response helper.
    pub(super) oos_output: ControllerOutput,
    /// Whether automatic defense currently owns a guard hold.
    ///
    ///  Shows whether the defense helper is holding shield.
    pub(super) autopilot_guarding: bool,
    /// Persistent guard-hold scheduler for a defended threat.
    ///
    ///  Keeps shield held for the selected defense window.
    pub(super) guard_hold: Option<kirby_assist_core::autopilot_guard::GuardHold>,
    /// Projectile-reflector policy state, including receipts.
    ///
    ///  Tracks an active projectile reflection response.
    pub(super) reflector: kirby_assist_core::projectile_reflect::Reflector,
    /// Current reflector controller proposal.
    ///
    ///  Buttons requested to reflect a projectile.
    pub(super) reflect_output: ControllerOutput,
    /// Optional spot-dodge policy and target state.
    ///
    ///  Tracks a dodge response to an incoming grab.
    pub(super) spot_dodge: Option<kirby_assist_core::spot_dodge::SpotDodge>,
    /// Current spot-dodge controller proposal.
    ///
    ///  Buttons requested by the spot-dodge helper.
    pub(super) spot_output: ControllerOutput,
    /// Optional ledge-trump timing and ownership state.
    ///
    ///  Tracks a possible ledge trump.
    pub(super) trump: Option<LedgeTrump>,
    /// Current ledge-trump controller proposal.
    ///
    ///  Buttons requested by the ledge-trump helper.
    pub(super) trump_output: ControllerOutput,
    // Cached with the same timestamp as requested; never used after expiry.
    /// Temporary requested stick for mash behavior.
    ///
    ///  The stick direction requested by the current mash helper.
    pub(super) mash_stick: Option<[f32; 2]>,
    /// Wall-clock timestamp associated with the latest request.
    ///
    ///  When the current helper request was made.
    pub(super) request_seen: Option<Instant>,
    /// Optional reaction-rule engine for configured matchup responses.
    ///
    ///  Tracks configured rules that react to opponent behavior.
    pub(super) rule_engine: Option<Box<kirby_assist_core::reaction_rules::Engine>>,
    /// Current roll-read controller proposal.
    ///
    ///  Buttons requested by the roll-read helper.
    pub(super) roll_read_output: ControllerOutput,
    /// Most recently observed native object ID.
    ///
    ///  The fighter object seen in the latest callback.
    pub(super) observed_object: Option<u32>,
    // Store a game object ID, never an accessor pointer that could become stale.
    /// Revalidated native object ID associated with the controller.
    ///
    ///  The fighter currently linked to this controller.
    pub(super) associated_object: Option<u32>,
    // Native kind for the same validated controller/fighter association.
    /// Fighter-kind ID for the validated controller association.
    ///
    ///  The character currently linked to this controller.
    pub(super) fighter_kind: Option<i32>,
    // Repeated game queries within one observed mapper update must not mash
    // faster, consume cooldown faster, or issue an extra tech press.
    /// Mapper tick already evaluated by policies.
    ///
    ///  Prevents the same controller poll from advancing helpers twice.
    pub(super) evaluated_tick: Option<i64>,
    /// Snapshot of observation fields required for native reads.
    ///
    ///  Tracks which game facts this session needs to collect.
    pub(super) observation_setup: kirby_assist_core::observation::ObservationSetup,
    // Native observation order is separate from mapper polling frequency.
    /// Monotonic wrapping fence for accepted observation publication.
    ///
    ///  Distinguishes newer observations from stale callbacks.
    pub(super) observation_sequence: u64,
    #[cfg(feature = "diagnostic-logging")]
    /// Last scenario log state used to suppress duplicate reports.
    ///
    ///  Remembers which scenario status was last logged.
    pub(super) last_logged_scenario: (bool, bool),
    #[cfg(feature = "diagnostic-logging")]
    /// Diagnostic confirmation that the mapper route was observed.
    ///
    ///  Records whether the controller mapping was verified.
    pub(super) mapper_confirmed: bool,
    #[cfg(feature = "diagnostic-logging")]
    /// Diagnostic flag indicating an input field was altered.
    ///
    ///  Records whether assistance changed any button data.
    pub(super) buttons_modified: bool,
    #[cfg(feature = "diagnostic-logging")]
    /// Suppresses repeated button-modification diagnostic events.
    ///
    ///  Avoids logging the same input change over and over.
    pub(super) button_mod_logged: bool,
    // Last game-visible buttons, distinct from the restored physical memory.
    // This tells the tech policy whether it must release ZR before re-pressing.
    /// Last mapped button bitfield observed by the game.
    ///
    ///  Remembers the buttons Smash actually received.
    pub(super) last_output_buttons: u32,
    #[cfg(feature = "diagnostic-logging")]
    /// Last tech diagnostic key and reason tuple.
    ///
    ///  Remembers the most recent tech status message.
    pub(super) last_tech_diagnostic: Option<(i32, i32, &'static str)>,
    #[cfg(feature = "diagnostic-logging")]
    /// Deferred tech diagnostic text awaiting logger output.
    ///
    ///  A tech message to write later outside the callback.
    pub(super) pending_tech_log: Option<String>,
    #[cfg(feature = "diagnostic-logging")]
    /// Deferred combat diagnostic text awaiting logger output.
    ///
    ///  A combat message to write later outside the callback.
    pub(super) pending_combat_log: Option<String>,
    #[cfg(feature = "diagnostic-logging")]
    /// Last combat diagnostic transition and target tuple.
    ///
    ///  Tracks recent combat changes to avoid duplicate reports.
    pub(super) last_combat_diagnostic: (bool, bool, u32),
    #[cfg(feature = "diagnostic-logging")]
    /// Rate gate for ledge diagnostic snapshots.
    ///
    ///  Limits repeated ledge troubleshooting reports.
    pub(super) ledge_diagnostics: DiagnosticGate<LedgeDiagnostic>,
    #[cfg(feature = "diagnostic-logging")]
    /// Recorder for diagnostic scenario/mode events.
    ///
    ///  Collects mode changes for diagnostic reports.
    pub(super) mode_diagnostics: kirby_assist_core::diagnostics::modes::Recorder,
    #[cfg(feature = "diagnostic-logging")]
    /// Wall-clock origin for mode diagnostics.
    ///
    ///  When the current mode report started.
    pub(super) mode_log_origin: Option<Instant>,
    #[cfg(feature = "diagnostic-logging")]
    /// Optional hit-confirmation fact associated with the mode trace.
    ///
    ///  Whether the recorded mode action connected.
    pub(super) mode_hit: Option<bool>,
    #[cfg(feature = "diagnostic-logging")]
    /// Last automatic-DI decision emitted for diagnostics.
    ///
    ///  Remembers the latest DI choice for troubleshooting.
    pub(super) last_di_diagnostic: Option<kirby_assist_core::auto_di::Decision>,
    #[cfg(feature = "diagnostic-logging")]
    /// Wall-clock origin for ledge diagnostic timing.
    ///
    ///  When the current ledge report started.
    pub(super) ledge_log_origin: Option<Instant>,
    #[cfg(feature = "diagnostic-logging")]
    /// Deferred ledge diagnostic text awaiting logger output.
    ///
    ///  A ledge message to write later outside the callback.
    pub(super) pending_ledge_log: Option<String>,
    #[cfg(feature = "diagnostic-logging")]
    /// Fixed-storage trace of grab request and mapper evidence.
    ///
    ///  Saves grab troubleshooting events for a later report.
    pub(super) grab_trace: crate::grab_diagnostics::GrabTrace,
    #[cfg(feature = "diagnostic-logging")]
    /// Recorder for diagnostic calibration samples.
    ///
    ///  Collects ledge calibration details when diagnostics are enabled.
    pub(super) calibration: crate::calibration::Recorder,
}
impl State {
    /// Keep edge histories while discarding automatic plans under ledge priority.
    ///
    ///  Stops lower-priority automatic helpers while preserving input history.
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
        self.trip_output = ControllerOutput::default();
        self.fair_output = ControllerOutput::default();
    }
    /// Cancels pending actions except those owned by the selected priority owner.
    ///
    ///  Clears helper requests that should yield to another action.
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
    /// Constructs empty callback state with initialized policy primitives.
    ///
    ///  Starts a session state with no active helper requests.
    pub(super) const fn new() -> Self {
        Self {
            reflector: kirby_assist_core::projectile_reflect::Reflector::new(),
            reflect_output: ControllerOutput::NONE,
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
            posture_side_aliases: kirby_assist_core::posture::TiltSideAliases::new(),
            posture_floor_safe: false,

            manual_overlay: None,
            turbo_ready: false,
            action_owner: ActionOwnership::new(),
            #[cfg(feature = "diagnostic-logging")]
            last_movement_blocker: None,
            #[cfg(feature = "diagnostic-logging")]
            last_input_expired: false,
            fair_followup: None,
            fair_output: ControllerOutput::NONE,
            trip_followup: None,
            trip_output: ControllerOutput::NONE,
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
            trump_output: ControllerOutput::NONE,
            oos: kirby_assist_core::out_of_shield::OutOfShield::new(),
            oos_output: ControllerOutput::NONE,
            autopilot_guarding: false,
            guard_hold: None,
            spot_dodge: None,
            spot_output: ControllerOutput::NONE,
            ledge_output: ControllerOutput::NONE,
            mash_stick: None,
            request_seen: None,
            rule_engine: None,
            roll_read_output: ControllerOutput::NONE,
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
    /// Drop every queued tool proposal and its output. Callers add the resets
    /// that differ: menu changes keep reflector/tech edge history, a full
    /// cancellation does not.
    ///
    ///  Clears pending helper actions so stale input cannot continue.
    fn clear_tool_proposals(&mut self) {
        self.reflect_output = ControllerOutput::default();
        // Native facts/settings already being read are no longer current, so the
        // old observation must not publish after the new state takes effect.
        self.observation_sequence = self.observation_sequence.wrapping_add(1);
        self.mash_stick = None;
        self.request_seen = None;
        self.oos = Default::default();
        self.oos_output = ControllerOutput::default();
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
        self.spot_dodge = None;
        self.spot_output = ControllerOutput::default();
        self.rule_engine = None;
        self.roll_read_output = ControllerOutput::default();
        self.autopilot_guarding = false;
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
        self.turbo = None;
        self.dash_pivot = None;
    }
    // Menu choices replace settings; do not let a previous configuration keep
    // an old punish/movement proposal queued. Navigation alone never calls this.
    /// Clears manual-control overlays and their pending ownership/outputs.
    ///
    ///  Stops the manual helpers from keeping old input requests.
    pub(super) fn cancel_manual_tools(&mut self) {
        self.reflector.cancel();
        self.clear_tool_proposals();
        // Clear the proposal, not tech's acknowledged-edge/cooldown history.
        // The next observation applies the current enabled/tuning settings.
        if let Some(tech) = self.tech_input.as_mut() {
            tech.cancel_request();
        }
        #[cfg(feature = "diagnostic-logging")]
        {
            self.grab_trace.reset("menu_choice_or_manual_tool_changed");
            self.calibration.reset("menu choice/manual tool changed");
        }
    }
    /// Advances match-session state and yields its current publication ticket.
    ///
    ///  Starts, updates, or ends the active match session.
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
    /// Cancels all pending tool proposals and resets transient assistance state.
    ///
    ///  Clears active helper actions while leaving native game state untouched.
    pub(super) fn cancel(&mut self) {
        self.reflector = kirby_assist_core::projectile_reflect::Reflector::new();
        self.clear_tool_proposals();
        #[cfg(feature = "diagnostic-logging")]
        {
            self.grab_trace.reset("session_or_controller_cancelled");
            self.calibration.reset("session/controller cancelled");
        }
        if let Some(menu) = self.settings_menu.as_mut() {
            menu.invalidate();
        }
        self.ledge_roster = None;
        self.observed_object = None;
        self.fighter_kind = None;
        self.evaluated_tick = None;
        self.observation_setup.clear();
        self.policy = None;
        self.animation_clock = None;
        self.tech_input = None;
        self.tech_floor_motion = Default::default();
        self.last_mapped_guard = false;
        self.last_mapped_stick = None;
        self.policy_tick = None;
        self.manual_context = None;
        self.manual_overlay = None;
        self.manual_jump_fence = kirby_assist_core::manual_controls::ManualJumpFence::new();
        self.grab_handoff = Default::default();
        self.turbo_ready = false;
        self.sdi = None;
        self.sdi_context = None;
        self.di = None;
        self.di_context = None;
        self.damage_trajectory = None;
        self.damage_geometry = None;
        self.ledge_walk_ready = false;
        self.ledge_walk_motion = None;
        self.ledge_active = false;
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
