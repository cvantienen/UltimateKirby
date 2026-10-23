//! Switch integration: observe fighter state on the game thread, publish button
//! requests, then overlay raw controller memory before mapping. No status writes.
//!
//! Separate callbacks, with an explicit gameplay/loading fence:
//! 1. The mapper records physical input for the selected local controller.
//! 2. A live fighter command callback validates identity/readiness, reads native
//!    observations and publishes copied input requests with a session ticket.
//! 3. The mapper binds the selected physical controller to its actual slot,
//!    refreshes a callback-discovered fighter, maps the overlay, then restores
//!    physical inputs. Discovery never enumerates fighters from the mapper.
//!
//! Extension points: add game observations in `observe_fighter`, logical tools
//! in the core, and verified button translations in `native_input.rs`. Defining
//! a Scenario alone does not make the live integration detect or output it.
// Keep hook/session orchestration here; native layouts and read-only queries
// live in children. pub(super) exposes a helper only to this module's family.
mod config;
mod controller;
mod ledge_menu;
mod observations;
mod tech;

use config::load_config;
use controller::{trace, write_trace, Controller, MappedInputs};
use observations::{
    combat_targets, fighter_ids, ground_support, human_fighter, native_air_physics,
    native_full_hop_speed, native_short_hop_speed, nearest_ledge_opponent,
    nearest_recovery_opponent,
};
use tech::assess_tech;

use crate::game_ffi as game;
use kirby_assist_core::combat_assist::{
    FairContext, FairFollowup, SdiContext, SdiState, TripContext, TripFollowup,
};
#[cfg(feature = "diagnostic-logging")]
use kirby_assist_core::diagnostics::DiagnosticGate;
use kirby_assist_core::fighter_profiles::{kirby, FighterProfiles};
use kirby_assist_core::ledge_position::{nearest_ground_ledge, LedgePosition};
use kirby_assist_core::ledge_trap::{
    grab_input_ready, LedgeContext, LedgeOutput, LedgeSettings, LedgeTrap, CLIFF_CATCH_MOVE,
    DPAD_DOWN,
};
use kirby_assist_core::ledge_trump::{nearest_floor_edge, LedgeTrump};
use kirby_assist_core::live_input::{
    unique_instruction_offset, LiveConfig as Config, RawAutoControls, ASSIST_BUTTONS, HID_ZR,
};
use kirby_assist_core::manual_controls::{
    ledge_walk_stick, suppress_reserved_taunts, turbo_movement_allowed, DpadReservation,
    ManualOverlay, ShortcutContext, Shortcuts, TurboState, DPAD_SIDES, DPAD_UP, SHORT_HOP_TRIGGER,
    TURBO_TRIGGER,
};
use kirby_assist_core::native_input::{
    controller_index, mapper_buttons, selected_controller_mapper, MAPPER_L, MAPPER_MINUS,
    MAPPER_PLUS, MAPPER_Y, MAPPER_ZR,
};
#[cfg(feature = "diagnostic-logging")]
use kirby_assist_core::native_input::{MAPPER_A, MAPPER_X};
use kirby_assist_core::session::{MatchSession, SessionTicket};
use kirby_assist_core::slide::{SlideContext, SlideState};
use kirby_assist_core::tech_input::{TechContext, TechInput, TechOutput};
#[cfg(feature = "diagnostic-logging")]
use kirby_assist_core::tech_prediction::CATCHED_GANON;
use kirby_assist_core::timing::{AnimationClock, AnimationSample};
use kirby_assist_core::two_frame::TwoFrame;
use skyline::hooks::{getRegionAddress, Region};
use std::{
    ffi::c_void,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Mutex, OnceLock,
    },
    time::{Duration, Instant},
};

// The instruction sequence and FIM ABI come from UltimateTrainingModpack
// revision 077a8e43b11d1f61ce57169156fb82e92e784c79, common/offsets.rs.
// Never fall back to a hard-coded address on a different executable.
const MAPPING_NEEDLE: &[u8] = &[
    0xff, 0x03, 0x02, 0xd1, 0xf7, 0x23, 0x00, 0xf9, 0xf6, 0x57, 0x05, 0xa9, 0xf4, 0x4f, 0x06, 0xa9,
    0xfd, 0x7b, 0x07, 0xa9, 0xfd, 0xc3, 0x01, 0x91, 0x3f, 0x04, 0x00, 0x31, 0xe0, 0x77, 0x00, 0x54,
];
// Atomics hold small installation/diagnostic values independently of STATE.
// OnceLock publishes one immutable startup config. The Mutex protects mutable
// controller/policy history shared by callbacks; never call game APIs while
// holding it, since a native call can re-enter another hooked function.
static MAPPING_OFFSET: AtomicUsize = AtomicUsize::new(0);
#[cfg(feature = "diagnostic-logging")]
static SEEN_MAPPERS: AtomicUsize = AtomicUsize::new(0);
#[cfg(feature = "diagnostic-logging")]
static ROUTE_MISMATCHES: AtomicUsize = AtomicUsize::new(0);
#[cfg(feature = "diagnostic-logging")]
static ROUTE_UNAVAILABLE_LOGGED: AtomicBool = AtomicBool::new(false);
static MANAGER_SLOT: AtomicUsize = AtomicUsize::new(0);
static CONFIG: OnceLock<Config> = OnceLock::new();
static PROFILES: OnceLock<FighterProfiles> = OnceLock::new();
// STATE.lock() yields a guard. Rust releases the lock when the guard leaves
// its scope (`}`); explicit small scopes below let us copy values, unlock, query
// the game, then lock again to publish results. `if let Ok` skips poisoned locks
// instead of panicking inside a native callback.
static STATE: Mutex<State> = Mutex::new(State::new());
static OBSERVING_FIGHTER: AtomicBool = AtomicBool::new(false);

// Native queries can re-enter command hooks. Never recursively sample another
// full observation while the mapper's synchronous refresh is in progress.
struct FighterObservationGuard;
impl FighterObservationGuard {
    fn acquire() -> Option<Self> {
        OBSERVING_FIGHTER
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .ok()
            .map(|_| Self)
    }
}
impl Drop for FighterObservationGuard {
    fn drop(&mut self) {
        OBSERVING_FIGHTER.store(false, Ordering::Release);
    }
}
// Wall-clock expiry prevents a request surviving indefinitely when observation
// stops. This is a watchdog duration, not the game's tech input window.
const REQUEST_LIFETIME: Duration = Duration::from_millis(35);
// A longer gap between valid fighter observations marks a loading transition,
// even when the manager singleton stays allocated across online scenes.
const SESSION_OBSERVATION_GAP: Duration = Duration::from_millis(250);
// Online scheduling may separate controller sampling from fighter callbacks.
// This does not extend REQUEST_LIFETIME: attack proposals still expire at 35 ms.
const CONTROLLER_SAMPLE_LIFETIME: Duration = Duration::from_millis(250);
// Resolution copies small typed values; no JSON parsing or file I/O occurs here.
fn ledge_settings(kind: Option<i32>, base: LedgeSettings) -> LedgeSettings {
    PROFILES.get().map_or(
        LedgeSettings {
            enabled: false,
            ..base
        },
        |profiles| profiles.ledge_settings(kind, base),
    )
}

// Diagnostic snapshots are absent from silent builds, including formatting.
#[cfg(feature = "diagnostic-logging")]
#[derive(Clone, Copy, PartialEq, Eq)]
struct LedgeDiagnostic {
    target: Option<u32>,
    enemy_status: i32,
    vulnerable: bool,
    kirby_status: i32,
    can_act: bool,
    can_grab: bool,
    can_dash_attack: bool,
    can_jump: bool,
    can_air_attack: bool,
    hit_confirmed: bool,
    own_motion: u64,
    requested: u32,
    action: &'static str,
}

// One session for the configured controller, rather than one global session per
// fighter. `cancel` clears assistance without changing the game's native state.
struct State {
    ledge_menu: Option<ledge_menu::LedgeMenu>,
    // UI identity/lifetime survive combat cancellation and respawn. They never
    // authorize an input sequence; fighter_kind retains that separate role.
    menu_fighter_kind: Option<i32>,
    menu_battle_active: bool,
    // Intro permission is tied to a real local ENTRY object and fresh route,
    // then discarded at GO. Retained CSS roster data cannot reopen the menu.
    menu_intro_object: Option<u32>,
    session: MatchSession,
    // Select a physical controller, then follow its actual mapper slot. Arena
    // participant slots must not be confused with local controller identity.
    mapping_index: Option<i32>,
    // Human fighter IDs learned from live command callbacks, indexed by mapper
    // slot. No accessor is retained and the mapper never enumerates the roster.
    human_routes: [Option<(u32, Instant)>; 8],
    // Roster IDs are discovered only by native callbacks; synchronous mapper
    // refreshes revalidate these IDs instead of enumerating a loading roster.
    ledge_roster: Option<([Option<u32>; 8], i64)>,
    // Our mapper-call counter, NOT a verified simulation-frame or HID clock.
    raw_tick: i64,
    raw_seen: Option<Instant>,
    raw_buttons: u32,
    raw_connected: bool,
    // Option permits lazy construction and a clean reset by assigning None.
    policy: Option<RawAutoControls>,
    animation_clock: Option<AnimationClock>,
    tech_input: Option<TechInput>,
    tech_output: TechOutput,
    last_mapped_guard: bool,
    policy_tick: Option<i64>,
    // Separate state for side-D-pad consumption and temporary left-stick control.
    turbo: Option<TurboState>,
    shortcuts: Option<Shortcuts>,
    shortcut_context: Option<ShortcutContext>,
    dpad_reservation: Option<DpadReservation>,
    manual_overlay: Option<ManualOverlay>,
    // Observation-derived eligibility, not just whether a trigger is held.
    // Turbo/ledge must also respect grounded movement and native busy states.
    turbo_ready: bool,
    fair_followup: Option<FairFollowup>,
    fair_output: LedgeOutput,
    trip_followup: Option<TripFollowup>,
    trip_output: LedgeOutput,
    sdi: Option<SdiState>,
    sdi_context: Option<SdiContext>,
    slide: Option<SlideState>,
    slide_output: LedgeOutput,
    ledge_walk_ready: bool,
    ledge: Option<LedgeTrap>,
    two_frame: Option<TwoFrame>,
    ledge_active: bool,
    two_frame_active: bool,
    ledge_seek_tick: Option<i64>,
    ledge_position: Option<LedgePosition>,
    ledge_output: LedgeOutput,
    trump: Option<LedgeTrump>,
    trump_output: LedgeOutput,
    // HID-named request bits are an internal message format; the mapper overlay
    // translates them to Smash's different raw bit positions before writing.
    requested: u64,
    // Cached with the same timestamp as requested; never used after expiry.
    mash_stick: Option<[f32; 2]>,
    // Owned buttons may be cleared for a release; unrelated human buttons stay.
    owned: u64,
    request_seen: Option<Instant>,
    observed_object: Option<u32>,
    // Store a game object ID, never an accessor pointer that could become stale.
    associated_object: Option<u32>,
    // Native kind for the same validated controller/fighter association.
    fighter_kind: Option<i32>,
    // Repeated game queries within one observed mapper update must not mash
    // faster, consume cooldown faster, or issue an extra tech press.
    evaluated_tick: Option<i64>,
    // Native observation order is separate from mapper polling frequency.
    observation_sequence: u64,
    #[cfg(feature = "diagnostic-logging")]
    last_logged_scenario: (bool, bool),
    #[cfg(feature = "diagnostic-logging")]
    mapper_confirmed: bool,
    #[cfg(feature = "diagnostic-logging")]
    buttons_modified: bool,
    #[cfg(feature = "diagnostic-logging")]
    button_mod_logged: bool,
    // Last game-visible buttons, distinct from the restored physical memory.
    // This tells the tech policy whether it must release ZR before re-pressing.
    last_output_buttons: u32,
    #[cfg(feature = "diagnostic-logging")]
    last_tech_diagnostic: Option<(i32, i32, &'static str)>,
    #[cfg(feature = "diagnostic-logging")]
    pending_tech_log: Option<String>,
    #[cfg(feature = "diagnostic-logging")]
    pending_combat_log: Option<String>,
    #[cfg(feature = "diagnostic-logging")]
    last_combat_diagnostic: (bool, bool, u32),
    #[cfg(feature = "diagnostic-logging")]
    ledge_diagnostics: DiagnosticGate<LedgeDiagnostic>,
    #[cfg(feature = "diagnostic-logging")]
    ledge_log_origin: Option<Instant>,
    #[cfg(feature = "diagnostic-logging")]
    pending_ledge_log: Option<String>,
    #[cfg(feature = "diagnostic-logging")]
    grab_trace: crate::grab_diagnostics::GrabTrace,
    #[cfg(feature = "diagnostic-logging")]
    calibration: crate::calibration::Recorder,
}
impl State {
    // `const fn` allows the global Mutex to be initialized before startup.
    // Options begin as None; get_or_insert_with in the callbacks constructs
    // each helper only when first needed. No game pointer is created here.
    const fn new() -> Self {
        Self {
            ledge_menu: None,
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
            raw_connected: false,
            policy: None,
            animation_clock: None,
            tech_input: None,
            tech_output: TechOutput {
                press: false,
                owns_guard: false,
            },
            last_mapped_guard: false,
            policy_tick: None,
            turbo: None,
            shortcuts: None,
            shortcut_context: None,
            dpad_reservation: None,
            manual_overlay: None,
            turbo_ready: false,
            fair_followup: None,
            fair_output: LedgeOutput {
                consume_trigger: false,
                stick: None,
                requested: 0,
                owned: 0,
            },
            trip_followup: None,
            trip_output: LedgeOutput {
                consume_trigger: false,
                stick: None,
                requested: 0,
                owned: 0,
            },
            sdi: None,
            sdi_context: None,
            slide: None,
            slide_output: LedgeOutput {
                consume_trigger: false,
                stick: None,
                requested: 0,
                owned: 0,
            },
            ledge_walk_ready: false,
            ledge: None,
            two_frame: None,
            ledge_active: false,
            two_frame_active: false,
            ledge_seek_tick: None,
            ledge_position: None,
            trump: None,
            trump_output: LedgeOutput {
                consume_trigger: false,
                stick: None,
                requested: 0,
                owned: 0,
            },
            ledge_output: LedgeOutput {
                consume_trigger: false,
                stick: None,
                requested: 0,
                owned: 0,
            },
            requested: 0,
            mash_stick: None,
            owned: 0,
            request_seen: None,
            observed_object: None,
            associated_object: None,
            fighter_kind: None,
            evaluated_tick: None,
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
    fn cancel_manual_tools(&mut self) {
        #[cfg(feature = "diagnostic-logging")]
        self.grab_trace.reset("menu_choice_or_manual_tool_changed");
        #[cfg(feature = "diagnostic-logging")]
        self.calibration.reset("menu choice/manual tool changed");
        self.ledge = None;
        self.two_frame = None;
        self.ledge_output = LedgeOutput::default();
        self.ledge_position = None;
        self.ledge_seek_tick = None;
        self.trump = None;
        self.trump_output = LedgeOutput::default();
        self.fair_followup = None;
        self.fair_output = LedgeOutput::default();
        self.trip_followup = None;
        self.trip_output = LedgeOutput::default();
        self.slide = None;
        self.slide_output = LedgeOutput::default();
        self.turbo = None;
    }
    fn update_match(&mut self, ready: bool) -> Option<SessionTicket> {
        if self.session.update(ready) || !ready {
            #[cfg(feature = "diagnostic-logging")]
            self.grab_trace.reset("match_session_ended");
            #[cfg(feature = "diagnostic-logging")]
            self.calibration.reset("match session ended");
            let session = self.session;
            let mut ledge_menu = self.ledge_menu.take();
            if let Some(ledge_menu) = ledge_menu.as_mut() {
                ledge_menu.invalidate();
            }
            let menu_fighter_kind = self.menu_fighter_kind;
            let menu_battle_active = self.menu_battle_active;
            let menu_intro_object = self.menu_intro_object;
            *self = Self::new();
            self.session = session;
            self.ledge_menu = ledge_menu;
            self.menu_fighter_kind = menu_fighter_kind;
            self.menu_battle_active = menu_battle_active;
            self.menu_intro_object = menu_intro_object;
        }
        self.session.ticket()
    }
    // Use for actual cancellation/reassignment. Hitstop deliberately does NOT
    // call this: resetting there would forget cooldown and input history.
    fn cancel(&mut self) {
        #[cfg(feature = "diagnostic-logging")]
        self.grab_trace.reset("session_or_controller_cancelled");
        #[cfg(feature = "diagnostic-logging")]
        self.calibration.reset("session/controller cancelled");
        if let Some(menu) = self.ledge_menu.as_mut() {
            menu.invalidate();
        }
        self.observation_sequence = self.observation_sequence.wrapping_add(1);
        self.ledge_roster = None;
        self.requested = 0;
        self.mash_stick = None;
        self.owned = 0;
        self.request_seen = None;
        self.observed_object = None;
        self.fighter_kind = None;
        self.evaluated_tick = None;
        self.policy = None;
        self.animation_clock = None;
        self.tech_input = None;
        self.tech_output = TechOutput::default();
        self.last_mapped_guard = false;
        self.policy_tick = None;
        self.turbo = None;
        self.shortcuts = None;
        self.shortcut_context = None;
        self.dpad_reservation = None;
        self.manual_overlay = None;
        self.turbo_ready = false;
        self.fair_followup = None;
        self.fair_output = LedgeOutput::default();
        self.trip_followup = None;
        self.trip_output = LedgeOutput::default();
        self.sdi = None;
        self.sdi_context = None;
        self.slide = None;
        self.slide_output = LedgeOutput::default();
        self.ledge_walk_ready = false;
        self.ledge = None;
        self.two_frame = None;
        self.ledge_active = false;
        self.two_frame_active = false;
        self.ledge_seek_tick = None;
        self.ledge_position = None;
        self.ledge_output = LedgeOutput::default();
        self.trump = None;
        self.trump_output = LedgeOutput::default();
        self.last_output_buttons = 0;
        #[cfg(feature = "diagnostic-logging")]
        {
            self.last_tech_diagnostic = None;
            self.pending_tech_log = None;
            self.pending_combat_log = None;
            self.last_combat_diagnostic = (false, false, 0);
            self.ledge_diagnostics = DiagnosticGate::new();
            self.ledge_log_origin = None;
            self.pending_ledge_log = None;
        }
    }
}

// A macro lets cfg remove the message expression itself in silent builds.
// Unlike an empty logging function, this also removes formatting and argument
// evaluation; changing tech_debug at runtime cannot re-enable these call sites.
#[cfg(feature = "diagnostic-logging")]
macro_rules! log {
    ($message:expr $(,)?) => {
        crate::logging::write($message)
    };
}
#[cfg(not(feature = "diagnostic-logging"))]
macro_rules! log {
    ($message:expr $(,)?) => {{}};
}

// Controller sampling is independent of battle mode and roster readiness.
// Only a matching live human fighter callback can publish an overlay. A gap
// invalidates the previous association without querying scene data here.
unsafe fn live_match_ticket() -> Option<SessionTicket> {
    let mut state = STATE.lock().ok()?;
    if state
        .request_seen
        .is_some_and(|seen| seen.elapsed() >= SESSION_OBSERVATION_GAP)
    {
        state.update_match(false);
    }
    state.update_match(true)
}

// Overlay physical button words before the original mapping function. This
// works whether the SDK uses single-state or bulk HID reads; no HID hook is
// required. Restore the physical structure after mapping to avoid feedback.
#[skyline::hook(offset = MAPPING_OFFSET.load(Ordering::Relaxed))]
unsafe fn map_controls(
    mappings: *mut c_void,
    index: i32,
    output: *mut c_void,
    wrapper: *mut c_void,
    arg: bool,
) {
    let Some(session_ticket) = live_match_ticket() else {
        original!()(mappings, index, output, wrapper, arg);
        return;
    };
    // Keep the untouched trace for restoration after original!(). The original
    // mapper receives the overlay only during this call; output keeps the mapped
    // result while the physical Controller memory is restored afterwards.
    // Treat the overlay like a short transaction:
    // save physical data -> temporarily replace -> map -> restore.
    // Only the mapper's RESULT survives. Retaining our temporary raw data would
    // make the next callback mistake automation for a real physical hold.
    let mut restore = None;
    let mut reserved_dpad = 0;
    let mut tech_applied = false;
    let mut attack_cancel_applied = false;
    #[cfg(feature = "diagnostic-logging")]
    let mut trump_pressed = false;
    #[cfg(feature = "diagnostic-logging")]
    let mut ledge_edge = None;
    #[cfg(feature = "diagnostic-logging")]
    let mut grab_delivery = None;
    #[cfg(feature = "diagnostic-logging")]
    let mut messages = Vec::new();
    if let Some(config) = CONFIG.get() {
        if !wrapper.is_null() && !output.is_null() && (0..8).contains(&index) {
            // The wrapper stores a Controller pointer at +0x10. read() loads
            // that pointer; cast() alone only changes its type. These native
            // offsets belong to the verified layout, not user-tunable settings.
            let controller = wrapper
                .cast::<u8>()
                .add(0x10)
                .cast::<*mut Controller>()
                .read();
            #[cfg(feature = "diagnostic-logging")]
            if !controller.is_null()
                && SEEN_MAPPERS.fetch_or(1 << index, Ordering::Relaxed) & (1 << index) == 0
            {
                messages.push(format!("mapper observed index {index}: npad={}, style={}, connected={}; configured local controller={}", (*controller).npad, (*controller).style, (*controller).connected, config.player_index));
            }
            if !controller.is_null()
                && selected_controller_mapper(config.player_index, (*controller).npad, index)
                    == Some(index)
            {
                // Read physical input before applying any assist. Reusing last
                // output as "human input" would keep automated buttons latched.
                let physical = trace(controller);
                let physical_stick = [(*controller).left_stick_x, (*controller).left_stick_y];
                let physical_right_stick =
                    [(*controller).right_stick_x, (*controller).right_stick_y];
                let physical_autorepeat = (*controller).autorepeat[0];
                // Only the selected controller checks the existing battle
                // manager guard. Menu lifetime is independent of request TTL.
                let menu_ready_go = observations::battle_menu_ready_go();
                if let Ok(mut state) = STATE.lock() {
                    let intro_current = state.menu_intro_object.is_some_and(|intro| {
                        state.human_routes[index as usize].is_some_and(|(id, seen)| {
                            id == intro && seen.elapsed() < SESSION_OBSERVATION_GAP
                        })
                    });
                    let menu_battle_active = menu_ready_go.is_some_and(|go| go || intro_current);
                    if menu_ready_go != Some(false) {
                        state.menu_intro_object = None;
                    }
                    if state.menu_battle_active != menu_battle_active {
                        state.menu_battle_active = menu_battle_active;
                        if menu_battle_active {
                            state
                                .ledge_menu
                                .get_or_insert_with(ledge_menu::LedgeMenu::default)
                                .begin_match();
                        } else {
                            state.menu_fighter_kind = None;
                            if let Some(ledge_menu) = state.ledge_menu.as_mut() {
                                ledge_menu.end_match();
                            }
                        }
                    }
                    if (*controller).connected == 0 {
                        state.cancel();
                        state.associated_object = None;
                        state.mapping_index = None;
                    } else {
                        if state.mapping_index != Some(index) {
                            state.cancel();
                            state.associated_object = None;
                            state.mapping_index = Some(index);
                            #[cfg(feature = "diagnostic-logging")]
                            {
                                state.mapper_confirmed = false;
                                messages.push(format!("selected local controller {} bound to mapper {index} (npad={}, style={})", config.player_index, (*controller).npad, (*controller).style));
                            }
                        }
                        #[cfg(feature = "diagnostic-logging")]
                        if !state.mapper_confirmed {
                            state.mapper_confirmed = true;
                        }
                        // A new mapper call advances our local scheduling tick.
                        // Other hooks can run several times within this tick;
                        // evaluated_tick prevents them consuming extra pulses.
                        let now = Instant::now();
                        state.raw_tick = state.raw_tick.wrapping_add(1);
                        state.raw_seen = Some(now);
                        state.raw_connected = true;
                        state.raw_buttons = physical.held;
                        if physical.held & MAPPER_PLUS != 0 {
                            state.cancel();
                        }
                        {
                            let kind = state.menu_fighter_kind;
                            let base = ledge_settings(kind, config.ledge);
                            let menu_available =
                                state.menu_battle_active && physical.held & MAPPER_PLUS == 0;
                            let gameplay = menu_available && state.fighter_kind.is_some();
                            let changed = state
                                .ledge_menu
                                .get_or_insert_with(ledge_menu::LedgeMenu::default)
                                .input(
                                    menu_available,
                                    gameplay,
                                    kind,
                                    physical.held,
                                    base,
                                    config.controls.auto_mash,
                                );
                            if changed {
                                state.cancel_manual_tools();
                            }
                            let consumed = state.ledge_menu.as_ref().map_or(0, |p| p.consumed);
                            if let Some(ledge_menu) = state.ledge_menu.as_ref() {
                                ledge_menu
                                    .display_battle(state.menu_battle_active && kind.is_some());
                            }
                            // L remains an assist trigger; Up still blocks slide
                            // diagonals. Menu-consumed sides/Down never reach tools.
                            // Preserve Y for the first native-readiness check;
                            // the later overlay still consumes its mapped action.
                            state.raw_buttons = kirby_assist_core::ledge_menu::observation_buttons(
                                physical.held,
                                consumed,
                            );
                            #[cfg(feature = "diagnostic-logging")]
                            {
                                trump_pressed =
                                    physical.pressed & state.raw_buttons & DPAD_DOWN != 0;
                            }
                            if consumed & DPAD_SIDES != 0
                                || physical.held & (DPAD_UP | DPAD_DOWN | MAPPER_MINUS) != 0
                            {
                                // Menu takeover cancels before a native refresh
                                // could replay a cached jump/dodge request.
                                state.slide = None;
                                state.slide_output = LedgeOutput::default();
                            }
                            let two_frame_held =
                                state.ledge_menu.as_ref().is_some_and(|p| p.two_frame_held);
                            let ledge_held = config.controls.enabled
                                && base.enabled
                                && state
                                    .ledge_menu
                                    .as_ref()
                                    .is_some_and(|p| p.ledge_held || p.two_frame_held);
                            if state.ledge_active != ledge_held
                                || state.two_frame_active != two_frame_held
                            {
                                #[cfg(feature = "diagnostic-logging")]
                                state.grab_trace.reset("autopilot_released_or_mode_changed");
                                #[cfg(feature = "diagnostic-logging")]
                                state.calibration.reset("autopilot released/mode changed");
                                state.ledge_active = ledge_held;
                                state.two_frame_active = two_frame_held;
                                state.ledge = None;
                                state.two_frame = None;
                                state.ledge_position = None;
                                state.ledge_seek_tick = None;
                                state.ledge_output = LedgeOutput::default();
                            }
                        }
                    }
                }
                // Restore 1.0.4's immediate pre-map refresh without its roster
                // scan: only resolve a recently observed human fighter ID.
                // Revalidate identity on every refresh; never retain a pointer.
                let candidate = STATE.lock().ok().and_then(|state| {
                    (state.mapping_index == Some(index))
                        .then_some(state.human_routes[index as usize])
                        .flatten()
                        .filter(|(_, seen)| seen.elapsed() < SESSION_OBSERVATION_GAP)
                        .map(|(id, _)| id)
                });
                if let Some(id) = candidate {
                    let object = if game::is_active(id) {
                        game::module_accessor(id)
                    } else {
                        std::ptr::null_mut()
                    };
                    let valid_route = !object.is_null()
                        && object.cast::<u8>().add(8).cast::<u32>().read() == id
                        && controller_index(object.cast()) == Some(index)
                        && human_fighter(object);
                    if valid_route {
                        observe_fighter(object, false);
                    } else if let Ok(mut state) = STATE.lock() {
                        if state.mapping_index == Some(index) {
                            state.cancel();
                            state.associated_object = None;
                            state.human_routes[index as usize] = None;
                        }
                    }
                }
                if let Ok(mut state) = STATE.lock() {
                    if state.session.accepts(session_ticket)
                        && state.mapping_index == Some(index)
                        && (*controller).connected != 0
                    {
                        let now = Instant::now();
                        let fresh = state
                            .request_seen
                            .is_some_and(|seen| now.duration_since(seen) < REQUEST_LIFETIME);
                        let route_current =
                            state.human_routes[index as usize].is_some_and(|(id, seen)| {
                                Some(id) == state.associated_object
                                    && seen.elapsed() < SESSION_OBSERVATION_GAP
                            });
                        let ledge_config = ledge_settings(state.fighter_kind, config.ledge);
                        let ledge_config = state.ledge_menu.as_ref().map_or(ledge_config, |p| {
                            p.settings(state.fighter_kind, ledge_config)
                        });
                        let manual_scope = config.controls.enabled
                            && route_current
                            && state.associated_object.is_some()
                            && state.associated_object == state.observed_object
                            && physical.held & MAPPER_PLUS == 0;
                        // Expired requests become neutral. The overlay still
                        // knows the previous assisted output, allowing a release.
                        let requested = if fresh { state.requested } else { 0 };
                        let auto_mash = state
                            .ledge_menu
                            .as_ref()
                            .map_or(config.controls.auto_mash, |menu| {
                                menu.auto_mash(config.controls.auto_mash)
                            });
                        // Turning mash off must also release cached takeover on
                        // this mapper sample, before another native observation.
                        let owned = if fresh && (auto_mash || state.mash_stick.is_none()) {
                            state.owned
                        } else if fresh && config.controls.takeover && state.tech_output.owns_guard
                        {
                            HID_ZR
                        } else {
                            0
                        };
                        let mash_stick = if fresh && auto_mash {
                            state.mash_stick
                        } else {
                            None
                        };
                        let mash_takeover = mash_stick.is_some() && config.controls.takeover;
                        // Compose all native requests through ONE button history.
                        // Independent overlays can disagree about a release when
                        // tech, mash and ledge ownership switch on the same frame.
                        // Read the trigger from PHYSICAL buttons, then consume
                        // its taunt bits. Releasing returns the fresh physical
                        // stick immediately, rather than forcing neutral.
                        let tick = state.policy_tick.unwrap_or(state.raw_tick);
                        let can_move = fresh && state.policy_tick.is_some() && state.turbo_ready;
                        // L+R3 is exclusively the menu chord; R3 alone teabags.
                        let r3_held = physical.held & TURBO_TRIGGER != 0
                            && physical.held & MAPPER_L == 0
                            && !state
                                .ledge_menu
                                .as_ref()
                                .is_some_and(|menu| menu.consumed & TURBO_TRIGGER != 0);
                        let turbo_facing = state.shortcut_context.map_or(1.0, |ctx| ctx.facing);
                        let mut turbo_output = state
                            .turbo
                            .get_or_insert_with(TurboState::default)
                            .sample_teabag(
                                config.turbo,
                                tick,
                                r3_held,
                                turbo_facing,
                                manual_scope,
                                can_move,
                            );
                        // Reserve remapped D-pad directions for the entire valid
                        // gameplay scope, independently of the 35ms request TTL.
                        let enabled_dpad = if manual_scope {
                            (if config.dpad_slide { DPAD_SIDES } else { 0 })
                                | if config.auto_ledge_trump {
                                    DPAD_DOWN
                                } else {
                                    0
                                }
                        } else {
                            0
                        };
                        reserved_dpad = state
                            .dpad_reservation
                            .get_or_insert_with(DpadReservation::default)
                            .sample(enabled_dpad, physical.held, manual_scope);
                        // L holds ledge trap; Y now runs the B-reverse shortcut.
                        // Changing tools releases the previous policy.
                        // Stale observations still release automation ownership.
                        let mut ledge = if fresh && state.ledge_active {
                            state.ledge_output
                        } else {
                            LedgeOutput::default()
                        };
                        if ledge_config.enabled
                            && manual_scope
                            && state.ledge_active
                            && !state
                                .ledge
                                .as_ref()
                                .is_some_and(LedgeTrap::manual_after_response)
                        {
                            let walking = if fresh
                                && !ledge_config.two_frame
                                && ledge_config.manual_spacing
                                && state.ledge_walk_ready
                            {
                                Some(ledge_walk_stick(physical_stick, ledge_config.walk_strength))
                            } else {
                                None
                            };
                            turbo_output.stick = ledge.stick.or(turbo_output.stick).or(walking);
                        }
                        // A tapped side owns one hop/dodge, a held side loops
                        // after native landing recovery. It outranks stick-click shortcuts.
                        if fresh && state.slide_output.owned != 0 {
                            ledge = state.slide_output;
                            turbo_output.stick = ledge.stick;
                        }
                        // A tapped hidden-panel Down owns its run-off until catch;
                        // it outranks slide/stick-click shortcuts but still yields to mash and tech.
                        if fresh && !mash_takeover && state.trump_output.owned != 0 {
                            ledge = state.trump_output;
                            turbo_output.stick = ledge.stick;
                        }
                        #[cfg(feature = "diagnostic-logging")]
                        if trump_pressed {
                            messages.push(format!(
                                "trump input: enabled={}, scope={}, fresh={}, action={}, stick={:?}, reserved={reserved_dpad:#x}",
                                config.auto_ledge_trump, manual_scope, fresh,
                                state.trump.as_ref().map_or("unobserved", LedgeTrump::action_name),
                                state.trump_output.stick,
                            ));
                        }
                        let trip_takeover =
                            fresh && manual_scope && !mash_takeover && state.trip_output.owned != 0;
                        let fair_takeover = fresh
                            && manual_scope
                            && !mash_takeover
                            && !trip_takeover
                            && state.fair_output.owned != 0;
                        if trip_takeover || fair_takeover {
                            ledge = if trip_takeover {
                                state.trip_output
                            } else {
                                state.fair_output
                            };
                            turbo_output.stick = ledge.stick;
                            reserved_dpad |= 0xf;
                        }
                        let shortcut_context = state.shortcut_context;
                        let shortcuts_allowed = manual_scope
                            && fresh
                            && !mash_takeover
                            && !state.tech_output.owns_guard
                            && !trip_takeover
                            && !fair_takeover
                            && state.slide_output.owned == 0
                            && state.trump_output.owned == 0
                            && physical.held & (0xf | MAPPER_MINUS) == 0
                            && !(physical.held & (MAPPER_L | TURBO_TRIGGER)
                                == (MAPPER_L | TURBO_TRIGGER));
                        let shortcut = state
                            .shortcuts
                            .get_or_insert_with(Shortcuts::default)
                            .sample(physical.held, shortcuts_allowed, shortcut_context);
                        let shortcut_consumed = state
                            .shortcuts
                            .as_ref()
                            .map_or(0, Shortcuts::consumed_buttons);
                        if shortcut.owned != 0 {
                            // These explicit actions outrank ledge preparation,
                            // but always yield to mash, tech, slide and trump.
                            ledge = shortcut;
                            turbo_output.stick = shortcut.stick;
                        }
                        // Menu input is reserved during entry/respawn too.
                        let menu_consumed =
                            if state.menu_battle_active && state.menu_fighter_kind.is_some() {
                                state.ledge_menu.as_ref().map_or(0, |menu| menu.consumed)
                            } else {
                                0
                            };
                        reserved_dpad |= menu_consumed & 0xf;
                        // Mash wins over both D-pad tools while trapped. Its
                        // separate None-on-exit returns the fresh physical stick.
                        if mash_stick.is_some() {
                            turbo_output.stick = mash_stick;
                        }
                        let sdi_context = state.sdi_context;
                        let sdi_enabled = config.auto_sdi && manual_scope && fresh;
                        let sdi_stick = sdi_context.and_then(|context| {
                            state.sdi.get_or_insert_with(SdiState::default).sample(
                                sdi_enabled,
                                context,
                                physical_stick,
                            )
                        });
                        if let Some(stick) = sdi_stick {
                            turbo_output.stick = Some(stick);
                        } else if !sdi_enabled || sdi_context.is_none() {
                            state.sdi = None;
                        }
                        // A held C-stick can otherwise inject an attack while
                        // left-stick-only mashing. Block its digital aliases too.
                        let blocked_stick_buttons = if mash_takeover {
                            kirby_assist_core::manual_controls::RIGHT_STICK_DIRECTIONS
                                | kirby_assist_core::manual_controls::STICK_CLICKS
                        } else {
                            0
                        };
                        let cancel_stick = if fresh
                            && state.ledge_active
                            && ledge.owned != 0
                            && !mash_takeover
                            && !trip_takeover
                            && !fair_takeover
                            && !state.tech_output.owns_guard
                            && state.slide_output.owned == 0
                            && state.trump_output.owned == 0
                            && shortcut.owned == 0
                        {
                            state
                                .ledge
                                .as_ref()
                                .and_then(LedgeTrap::attack_cancel_right_stick)
                        } else {
                            None
                        };
                        // All manual tools share ONE final edge history. When
                        // adding a tool, compose its requested/owned masks here
                        // and decide its stick priority explicitly; do not write
                        // another independent A/R/ZR history after this point.
                        let manual = state
                            .manual_overlay
                            .get_or_insert_with(ManualOverlay::default)
                            .apply_controls(
                                physical,
                                physical_stick,
                                turbo_output,
                                reserved_dpad
                                    | menu_consumed
                                    | if manual_scope {
                                        shortcut_consumed | MAPPER_Y
                                    } else {
                                        0
                                    }
                                    | if config.turbo.enabled && manual_scope {
                                        TURBO_TRIGGER
                                    } else {
                                        0
                                    },
                                ledge.requested | mapper_buttons(requested),
                                ledge.owned
                                    | if cancel_stick.is_some() {
                                        kirby_assist_core::manual_controls::RIGHT_STICK_DIRECTIONS
                                    } else {
                                        0
                                    }
                                    | blocked_stick_buttons
                                    | mapper_buttons(owned)
                                    | if owned & HID_ZR != 0 {
                                        kirby_assist_core::native_input::MAPPER_ZL
                                            | kirby_assist_core::native_input::MAPPER_ZR
                                            | kirby_assist_core::native_input::MAPPER_L
                                            | kirby_assist_core::native_input::MAPPER_R
                                    } else {
                                        0
                                    },
                            );
                        let modified = manual.buttons;
                        #[cfg(feature = "diagnostic-logging")]
                        if state.grab_trace.active() || state.calibration.active() {
                            grab_delivery = Some((
                                state.ledge_output.requested,
                                modified.held,
                                modified.pressed,
                                if !fresh {
                                    "expired_observation"
                                } else if !manual_scope {
                                    "outside_manual_scope"
                                } else if mash_takeover {
                                    "mash_priority"
                                } else if state.tech_output.owns_guard {
                                    "tech_priority"
                                } else if trip_takeover {
                                    "trip_priority"
                                } else if fair_takeover {
                                    "fair_priority"
                                } else if state.trump_output.owned != 0 {
                                    "trump_priority"
                                } else if state.slide_output.owned != 0 {
                                    "slide_priority"
                                } else if shortcut.owned != 0 {
                                    "shortcut_priority"
                                } else {
                                    "ledge"
                                },
                            ));
                        }
                        tech_applied = requested & HID_ZR != 0 && modified.pressed & MAPPER_ZR != 0;
                        #[cfg(feature = "diagnostic-logging")]
                        if ledge.requested & modified.pressed != 0 {
                            ledge_edge = Some(ledge.requested);
                        }
                        #[cfg(feature = "diagnostic-logging")]
                        if let Some(message) = state.pending_combat_log.take() {
                            messages.push(message);
                        }
                        #[cfg(feature = "diagnostic-logging")]
                        if let Some(message) = state.pending_tech_log.take() {
                            messages.push(message);
                        }
                        #[cfg(feature = "diagnostic-logging")]
                        if let Some(message) = state.pending_ledge_log.take() {
                            messages.push(message);
                        }
                        #[cfg(feature = "diagnostic-logging")]
                        if config.controls.tech_debug
                            && state.last_logged_scenario.1
                            && modified.pressed & MAPPER_ZR != 0
                            && requested & HID_ZR != 0
                        {
                            messages.push(format!("tech ZR applied: input_update={}, held={:#x}, previous={:#x}, pressed={:#x}", state.raw_tick, modified.held, modified.previous, modified.pressed));
                            messages.push(format!(
                                "tech request timing: {:?}",
                                state
                                    .tech_input
                                    .as_ref()
                                    .and_then(TechInput::last_observation)
                            ));
                        }
                        state.last_output_buttons = modified.held;
                        #[cfg(feature = "diagnostic-logging")]
                        {
                            state.buttons_modified |= modified != physical;
                        }
                        // Mutation is confined to the verified controller
                        // fields, after policy evaluation. This is software
                        // input before user button mapping, not a physical HID
                        // report transmitted by a Pro Controller.
                        write_trace(controller, modified);
                        (*controller).left_stick_x = manual.stick[0];
                        (*controller).left_stick_y = manual.stick[1];
                        // Neutralize C-stick only for active ledge ownership or
                        // mash takeover. Restore its physical memory after mapping.
                        if ledge.owned != 0 || mash_takeover {
                            (*controller).right_stick_x = 0.0;
                            (*controller).right_stick_y = 0.0;
                        }
                        if let Some(stick) = cancel_stick {
                            attack_cancel_applied = stick[0] != 0.0;
                            (*controller).right_stick_x = stick[0];
                            (*controller).right_stick_y = stick[1];
                        }
                        // Repeat flags can otherwise leak a held side-taunt
                        // even after held/pressed/released words were consumed.
                        (*controller).autorepeat[0] &= !manual.clear_autorepeat;
                        restore = Some((
                            controller,
                            physical,
                            physical_stick,
                            physical_right_stick,
                            physical_autorepeat,
                        ));
                    }
                }
            }
        }
    }
    // original!() is Skyline's trampoline to the function we replaced. Calling
    // map_controls itself here would recurse into our hook instead.
    original!()(mappings, index, output, wrapper, arg);
    if restore.is_some() && !output.is_null() {
        let guard = (*output.cast::<MappedInputs>()).buttons & 0x8 != 0;
        if let Ok(mut state) = STATE.lock() {
            if state.mapping_index == Some(index) && state.session.accepts(session_ticket) {
                if tech_applied && guard && !state.last_mapped_guard {
                    if let Some(tech) = state.tech_input.as_mut() {
                        tech.mapped_press();
                    }
                    #[cfg(feature = "diagnostic-logging")]
                    if CONFIG
                        .get()
                        .is_some_and(|config| config.controls.tech_debug)
                    {
                        messages.push(
                            "tech mapped: fresh GUARD edge confirmed; request cooldown committed"
                                .into(),
                        );
                    }
                }
                if !tech_applied && guard && !state.last_mapped_guard {
                    if let Some(tech) = state.tech_input.as_mut() {
                        tech.mapped_human_guard();
                    }
                }
                state.last_mapped_guard = guard;
                let mapped = &*output.cast::<MappedInputs>();
                #[cfg(feature = "diagnostic-logging")]
                if let Some((proposed, applied, pressed, gate)) = grab_delivery {
                    state.grab_trace.mapped(
                        proposed,
                        applied,
                        pressed,
                        mapped.buttons,
                        [mapped.left_x, mapped.left_y],
                        gate,
                        Instant::now(),
                    );
                    state
                        .calibration
                        .mapped(proposed, applied, mapped.buttons, gate);
                }
                if attack_cancel_applied {
                    if let Some(trap) = state.ledge.as_mut() {
                        trap.mapped_attack_cancel(mapped.buttons, mapped.right_x);
                    }
                }
            }
        }
    }
    if reserved_dpad != 0 {
        let mapped = output.cast::<MappedInputs>();
        (*mapped).buttons = suppress_reserved_taunts((*mapped).buttons, reserved_dpad);
    }
    #[cfg(feature = "diagnostic-logging")]
    if trump_pressed {
        let mapped = &*output.cast::<MappedInputs>();
        messages.push(format!(
            "trump mapped: buttons={:#x}, left=({},{})",
            mapped.buttons, mapped.left_x, mapped.left_y
        ));
    }
    #[cfg(feature = "diagnostic-logging")]
    if let Some(requested) = ledge_edge.filter(|_| grab_delivery.is_none()) {
        let mapped = &*output.cast::<MappedInputs>();
        messages.push(format!(
            "ledge mapper: raw_request={requested:#x}, mapped_buttons={:#x}, left=({},{})",
            mapped.buttons, mapped.left_x, mapped.left_y
        ));
        if requested & (MAPPER_X | MAPPER_A) == (MAPPER_X | MAPPER_A) {
            messages.push(format!(
                "short-hop mapped pair: jump={}, attack={}, neutral_stick={}; native ATTACK_AIR/attack_air_n still confirms acceptance",
                mapped.buttons & (0x4 | 0x40) != 0,
                mapped.buttons & 0x1 != 0,
                mapped.left_x == 0 && mapped.left_y == 0,
            ));
        }
    }
    // Restore every field we changed, including C-stick and repeat flags.
    // A new field added to the overlay must also be captured and restored here.
    // Clearing reserved taunts affects `output`, so restoring raw input does not
    // undo that filter or any other action the original mapper already produced.
    if let Some((controller, physical, stick, right_stick, autorepeat)) = restore {
        write_trace(controller, physical);
        (*controller).left_stick_x = stick[0];
        (*controller).left_stick_y = stick[1];
        (*controller).right_stick_x = right_stick[0];
        (*controller).right_stick_y = right_stick[1];
        (*controller).autorepeat[0] = autorepeat;
    }
    #[cfg(feature = "diagnostic-logging")]
    for message in messages {
        crate::logging::write_owned(message);
    }
}

// Exported ControlModule symbol, ABI checked against pinned skyline-smash.
// Category 0 is FIGHTER_PAD_COMMAND_CATEGORY1. This observes game state only:
// it returns the game's flags unchanged and never ORs command/action flags.
#[skyline::hook(replace = game_command_flags)]
unsafe fn observe_commands(object: *mut game::Object, category: i32) -> i32 {
    let flags = original!()(object, category);
    if category == 0 && !object.is_null() {
        observe_fighter(object, true);
    }
    flags
}
extern "C" {
    #[link_name = "\u{1}_ZN3app8lua_bind40ControlModule__get_command_flag_cat_implEPNS_26BattleObjectModuleAccessorEi"]
    fn game_command_flags(object: *mut game::Object, category: i32) -> i32;
}

// This is an observer, not a command injector: it reads game facts and publishes
// button requests. Extend scenario detection here without forcing status changes.
// This function has three phases: validate routing, read game facts without a
// lock, then update pure Rust policy under the lock. It publishes requests for
// map_controls; it never forces a fighter status or directly starts an attack.
unsafe fn observe_fighter(object: *mut game::Object, discover_roster: bool) {
    // Read this callback in four stages:
    // 1. Validate the match, fighter identity and physical-controller route.
    // 2. Copy native observations without retaining game pointers.
    // 3. Re-check the session, then update portable policies under STATE.
    // 4. Publish requests for map_controls to overlay and acknowledge.
    // The order matters: a loading transition can happen between callbacks.
    let Some(config) = CONFIG.get().copied() else {
        return;
    };
    let Some(session_ticket) = live_match_ticket() else {
        return;
    };
    if object.is_null() {
        return;
    }
    // The callback owns a live accessor. Reject inactive/recycled identities
    // before dereferencing any ControlModule or other fighter module pointers.
    let object_id = object.cast::<u8>().add(8).cast::<u32>().read();
    if object_id >> 28 != 0
        || !game::is_active(object_id)
        || game::module_accessor(object_id) != object
    {
        return;
    }
    let fighter_kind = game::fighter_kind(object_id);
    let config = Config {
        ledge: ledge_settings(Some(fighter_kind), config.ledge),
        ..config
    };
    let config = Config {
        ledge: STATE
            .lock()
            .ok()
            .and_then(|s| {
                s.ledge_menu
                    .as_ref()
                    .map(|p| p.settings(Some(fighter_kind), config.ledge))
            })
            .unwrap_or(config.ledge),
        ..config
    };
    // ControlModule +0x118 -> +0x58 -> +8 -> index at +8.
    // A direct ControlModule+8 read does not resolve the controller index.
    let Some(index) = controller_index(object.cast()) else {
        #[cfg(feature = "diagnostic-logging")]
        if !ROUTE_UNAVAILABLE_LOGGED.swap(true, Ordering::Relaxed) {
            log!("fighter controller ownership chain unavailable; waiting for a valid local fighter input index");
        }
        return;
    };
    let (
        tick,
        target,
        find_trump_edge,
        find_trap_edge,
        observe_ledge,
        observe_slide,
        observe_trump,
        observe_shortcuts,
    ) = {
        let Ok(state) = STATE.lock() else {
            return;
        };
        let trump_held = state.raw_buttons & DPAD_DOWN != 0;
        (
            state.raw_tick,
            state.mapping_index,
            config.auto_ledge_trump
                && state
                    .trump
                    .as_ref()
                    .is_none_or(|trump| trump.needs_edge(trump_held))
                && trump_held,
            config.ledge.enabled
                && (config.ledge.two_frame
                    || !config.ledge.manual_spacing && !config.ledge.distance_agnostic)
                && state.ledge_active
                && state
                    .ledge_seek_tick
                    .is_none_or(|last| state.raw_tick.wrapping_sub(last) >= 30)
                && state
                    .ledge_position
                    .as_ref()
                    .is_none_or(|p| p.spot().is_none()),
            config.ledge.enabled && state.ledge_active,
            config.dpad_slide
                && (state.raw_buttons & DPAD_SIDES != 0
                    || state.slide.as_ref().is_some_and(SlideState::active)),
            config.auto_ledge_trump
                && (trump_held || state.trump.as_ref().is_some_and(LedgeTrump::active)),
            state.raw_buttons & (SHORT_HOP_TRIGGER | MAPPER_Y) != 0
                || state.shortcuts.as_ref().is_some_and(Shortcuts::active),
        )
    };
    // Discovery is independent of which mapper currently holds Player 1.
    // Cache every human route so the next selected controller poll can refresh
    // the correct fighter synchronously, including after an Arena slot change.
    if !human_fighter(object) {
        return;
    }
    if let Ok(mut state) = STATE.lock() {
        if !state.session.accepts(session_ticket) {
            return;
        }
        state.human_routes[index as usize] = Some((object_id, Instant::now()));
    }
    if target != Some(index) {
        #[cfg(feature = "diagnostic-logging")]
        if ROUTE_MISMATCHES.fetch_or(1 << index, Ordering::Relaxed) & (1 << index) == 0 {
            log!(&format!("human fighter input index {index}; selected physical controller's mapper is {target:?}; route cached for mapping"));
        }
        return;
    }
    let Some(_observation_guard) = FighterObservationGuard::acquire() else {
        return;
    };
    let trip_kirby = config.down_tilt_trip_smash && fighter_kind == kirby::FIGHTER_KIND;
    let native_status = game::status_kind(object);
    let damage_stop = config.auto_sdi
        && (0x47..=0x5b).contains(&native_status)
        && game::is_stop(object)
        && game::is_damage_stop(object);
    let trip_watch = trip_kirby
        && (matches!(native_status, 0x2a | 0x2c)
            || STATE.lock().ok().is_some_and(|state| {
                state
                    .trip_followup
                    .as_ref()
                    .is_some_and(TripFollowup::watching)
            }));
    if discover_roster && (observe_ledge || trip_watch || damage_stop) {
        let needs_roster = STATE.lock().ok().is_some_and(|state| {
            state
                .ledge_roster
                .is_none_or(|(_, observed_tick)| observed_tick != tick)
        });
        if needs_roster {
            let roster = fighter_ids();
            if let Ok(mut state) = STATE.lock() {
                if state.session.accepts(session_ticket) && state.mapping_index == Some(index) {
                    state.ledge_roster = roster.map(|ids| (ids, tick));
                }
            }
        }
    }
    let observation_sequence = {
        let Ok(mut state) = STATE.lock() else {
            return;
        };
        if !state.session.accepts(session_ticket) || state.mapping_index != Some(index) {
            return;
        }
        if state
            .associated_object
            .is_some_and(|previous| previous != object_id)
            || state
                .fighter_kind
                .is_some_and(|previous| previous != fighter_kind)
        {
            state.cancel();
        }
        state.associated_object = Some(object_id);
        state.fighter_kind = Some(fighter_kind);
        state.menu_fighter_kind = Some(fighter_kind);
        if native_status == 0x1d9 && !state.menu_battle_active {
            state.menu_intro_object = Some(object_id);
        }
        if state.evaluated_tick == Some(tick) && state.observed_object == Some(object_id) {
            return;
        }
        state.observation_sequence = state.observation_sequence.wrapping_add(1);
        state.observation_sequence
    };
    if !config.controls.enabled || !game::is_active(object_id) {
        if let Ok(mut state) = STATE.lock() {
            state.cancel();
        }
        return;
    }
    // Hitstop freezes native action progress. It differs from cancellation:
    // keep remembered cooldowns/plans while withholding new mash/tech requests.
    let stopped = game::is_stop(object);
    let clatter_remaining = game::clatter_time(object, 0);
    let clatter = !stopped && clatter_remaining.is_finite() && clatter_remaining > 0.0;
    // Prediction is evaluated separately from the scheduler: finding contact
    // does not mean the cooldown is ready or that the game will accept a tech.
    // Learned input-delay compensation has been removed. Native move timing
    // and explicit response configuration are the only timing inputs.
    let input_delay = 0.0;
    let assessment = assess_tech(object, config.lookahead, stopped, input_delay);
    let tech = !clatter && assessment.reason == "collision_predicted";
    if matches!(assessment.status, 0xb5 | 0xb6 | 0x1d6 | 0x1d9..=0x1db) {
        if let Ok(mut state) = STATE.lock() {
            state.cancel();
        }
        return;
    }

    // Read native data outside STATE's mutex to avoid re-entrant deadlocks.
    let ledge_motion_frame = game::motion_frame(object);
    let own_motion_kind = game::motion_kind(object);
    let can_slide_jump = observe_slide
        && (matches!(assessment.status, 0..=0x9 | 0x11..=0x19 | 0x1b..=0x1d | 0x22..=0x23)
            || game::cancel_enabled(object));
    let timing_rate = game::motion_rate(object);
    let can_slide_air_dodge = observe_slide && game::transition_enabled(object, 0x1e00003d);
    // Observation stage: build ordinary Rust values from read-only game calls.
    // Startup/cancel/hit-status queries describe the real game; they do not
    // make an input legal. The core uses these values to decide what to request.
    // Full movement/ledge queries (including ground rays) are demand-driven.
    // Stick-click/Y shortcuts use the existing motion sample, without ground rays.
    let ledge_observation = if observe_ledge || observe_slide || observe_trump {
        let position = [game::pos_x(object), game::pos_y(object)];
        let velocity = [game::speed_x(object, -1), game::speed_y(object, -1)];
        let shield_stun_frames =
            (assessment.status == 0x1e).then(|| game::work_int(object, 0x1100000e));
        let shield_attacker = if assessment.status == 0x1e {
            let id = game::work_int(object, 0x1100000f) as u32;
            (id != object_id && id >> 28 == 0 && game::is_active(id)).then_some(id)
        } else {
            None
        };
        // Copy native facts into a plain value. held/can_act/can_control are
        // filled later from the same selected controller's history. The core
        // receives these values, not the game-owned accessor pointer.
        let can_tilt = game::transition_enabled(
            object,
            match config.ledge.getup_attack {
                kirby_assist_core::ledge_trap::GetupAttack::DownTilt => 0x1e000057,
                kirby_assist_core::ledge_trap::GetupAttack::ForwardTilt => 0x1e000055,
                kirby_assist_core::ledge_trap::GetupAttack::NeutralAir => 0x1e000062,
                kirby_assist_core::ledge_trap::GetupAttack::DownSpecial => 0x1e000066,
                kirby_assist_core::ledge_trap::GetupAttack::NeutralSpecial => 0x1e000063,
            },
        );
        let context = LedgeContext {
            held: false,
            gameplay: true,
            can_act: false,
            can_control: false,
            stopped,
            status: assessment.status,
            situation: assessment.situation,
            motion_frame: ledge_motion_frame,
            motion_kind: own_motion_kind,
            motion_rate: timing_rate,
            position,
            velocity,
            ground_brake: if observe_ledge {
                let brake = game::param_float(object, 0x0c8cc9db76, 0); // ground_brake
                (brake.is_finite() && (0.001..=3.0).contains(&brake)).then_some(brake)
            } else {
                None
            },
            facing: game::facing(object),
            previous_status: game::previous_status(object, 0),
            can_grab: observe_ledge
                && grab_input_ready(assessment.status, |term| {
                    game::transition_enabled(object, term)
                }),
            can_tilt,
            can_down_tilt: if config.ledge.two_frame_down_tilt {
                game::transition_enabled(object, 0x1e000057)
            } else if config.ledge.getup_attack
                == kirby_assist_core::ledge_trap::GetupAttack::DownTilt
            {
                can_tilt
            } else {
                observe_ledge
                    && config.ledge.drop_action
                        == kirby_assist_core::ledge_trap::LedgeDropAction::DownTiltShield
                    && game::transition_enabled(object, 0x1e000057)
            },
            can_down_special: if config.ledge.getup_attack
                == kirby_assist_core::ledge_trap::GetupAttack::DownSpecial
            {
                can_tilt
            } else {
                observe_ledge
                    && config.ledge.drop_action
                        == kirby_assist_core::ledge_trap::LedgeDropAction::Bomb
                    && game::transition_enabled(object, 0x1e000066)
            },
            // CONT_ATTACK_DASH; standing full-stick+A is not a dash attack.
            can_dash_attack: game::transition_enabled(object, 0x1e000054),
            input_delay: 0.0,
            can_down_smash: game::transition_enabled(object, 0x1e00005f),
            can_jump: game::transition_enabled(object, 0x1e00002c),
            can_air_attack: game::transition_enabled(object, 0x1e000062),
            air_physics: observe_ledge.then(|| native_air_physics(object)).flatten(),
            short_hop_speed: if observe_ledge {
                native_short_hop_speed(object)
            } else {
                None
            },
            full_hop_speed: if observe_ledge {
                native_full_hop_speed(object)
            } else {
                None
            },
            shield_stun_frames,
            shield_attacker,
            hit_confirmed: (matches!(assessment.status, 0x29..=0x2c | 0x2f..=0x32)
                || assessment.status == 0x36)
                && game::attack_infliction(object, game::COLLISION_KIND_MASK_HIT),
            safe_ground: if assessment.situation == 0 {
                ground_support(object, position, velocity[0])
            } else {
                [false, false]
            },
        };
        // The trump uses Kirby's current floor geometry, independently of the
        // opponent roster or Y acquisition range. Query native rays outside STATE.
        let edge = if find_trump_edge
            && turbo_movement_allowed(context.status, context.situation, stopped)
            && !clatter
            && !tech
        {
            nearest_floor_edge(position, context.facing, |probe| {
                let origin = game::Vector2 {
                    x: probe[0],
                    y: probe[1] + 2.0,
                };
                let delta = game::Vector2 { x: 0.0, y: -6.0 };
                game::ray_check(object, &origin, &delta, true) != 0
            })
        } else {
            None
        };
        let spot = if find_trap_edge
            && turbo_movement_allowed(context.status, context.situation, stopped)
            && !clatter
            && !tech
        {
            nearest_ground_ledge(
                position,
                context.facing,
                |probe| {
                    let origin = game::Vector2 {
                        x: probe[0],
                        y: probe[1] + 2.0,
                    };
                    let delta = game::Vector2 { x: 0.0, y: -6.0 };
                    game::ray_check(object, &origin, &delta, true) != 0
                },
                config.ledge.stand_distance,
            )
        } else {
            None
        };
        Some((
            context,
            if observe_ledge && !config.ledge.two_frame {
                nearest_ledge_opponent(object, object_id, config.ledge)
            } else {
                None
            },
            edge,
            spot,
            if observe_ledge && config.ledge.two_frame {
                nearest_recovery_opponent(object, object_id, spot)
            } else {
                None
            },
        ))
    } else {
        None
    };
    let (trip_targets, outward) = if trip_watch || damage_stop {
        combat_targets(object, object_id, trip_watch, damage_stop)
    } else {
        ([None; 8], 0.0)
    };
    let trip_context = TripContext {
        object: object_id,
        kirby: trip_kirby,
        status: assessment.status,
        motion: own_motion_kind,
        frame: ledge_motion_frame,
        grounded: assessment.situation == 0,
        stopped,
        facing: game::facing(object),
        targets: trip_targets,
    };
    let fair_hit_window = fighter_kind == kirby::FIGHTER_KIND
        && config.kirby_fair1_smash
        && assessment.status == 0x36
        && own_motion_kind == 0xc3495ada5
        && (10.0..12.0).contains(&ledge_motion_frame);
    let fair_context = FairContext {
        object: object_id,
        kirby: fighter_kind == kirby::FIGHTER_KIND,
        status: assessment.status,
        motion: own_motion_kind,
        frame: ledge_motion_frame,
        rate: timing_rate,
        grounded: assessment.situation == 0,
        stopped,
        facing: trip_context.facing,
        hit_now: fair_hit_window
            && game::attack_infliction_now(object, game::COLLISION_KIND_MASK_HIT),
    };
    let shortcut_context = ShortcutContext {
        status: assessment.status,
        situation: assessment.situation,
        motion: own_motion_kind,
        frame: ledge_motion_frame,
        rate: timing_rate,
        stopped: stopped || clatter || tech,
        facing: trip_context.facing,
        can_short_hop: observe_shortcuts
            && assessment.situation == 0
            && (game::transition_enabled(object, 0x1e00002c)
                || game::transition_enabled(object, 0x1e00002b)), // Jump button / stick terms
        input_delay: 0.0,
        can_neutral_special: observe_shortcuts && game::transition_enabled(object, 0x1e000063), // CONT_SPECIAL_N
    };
    let mut sdi_context = SdiContext {
        object: object_id,
        damage_stop,
        input_delay: 0.0,
        remaining: if damage_stop {
            game::hitstop_remaining(object)
        } else {
            0
        },
        outward,
    };
    #[cfg(feature = "diagnostic-logging")]
    let mut scenario_changed = false;
    #[cfg(feature = "diagnostic-logging")]
    let mut acquired = false;
    #[cfg(feature = "diagnostic-logging")]
    let mut first_button_write = false;
    // Policy/publication stage: the native reads above are finished. Check the
    // session again before updating history, because those reads can re-enter
    // hooks. No file loading or JSON parsing belongs in this per-update path.
    if let Ok(mut state) = STATE.lock() {
        // Native queries may overlap another mapper poll online. Validate the
        // session/route, then schedule against the latest local mapper tick;
        // requiring the old tick to be identical can starve all publication.
        if !state.session.accepts(session_ticket)
            || state.mapping_index != Some(index)
            || state.associated_object != Some(object_id)
            || state.observation_sequence != observation_sequence
        {
            return;
        }
        let tick = state.raw_tick;
        if state.evaluated_tick == Some(tick) && state.observed_object == Some(object_id) {
            return;
        }
        let now = Instant::now();
        if !state.raw_connected
            || state
                .raw_seen
                .is_none_or(|seen| now.duration_since(seen) >= CONTROLLER_SAMPLE_LIFETIME)
            || state.raw_buttons & MAPPER_PLUS != 0
        {
            state.cancel();
            return;
        }
        #[cfg(feature = "diagnostic-logging")]
        {
            first_button_write = state.buttons_modified && !state.button_mod_logged;
            state.button_mod_logged |= first_button_write;
            acquired = state.observed_object != Some(object_id);
        }
        state.fair_output = state
            .fair_followup
            .get_or_insert_with(FairFollowup::default)
            .sample(config.kirby_fair1_smash && !clatter && !tech, fair_context);
        state.trip_output = state
            .trip_followup
            .get_or_insert_with(TripFollowup::default)
            .sample(
                config.down_tilt_trip_smash && !clatter && !tech,
                trip_context,
            );
        #[cfg(feature = "diagnostic-logging")]
        {
            let key = (
                state
                    .trip_followup
                    .as_ref()
                    .is_some_and(TripFollowup::watching),
                sdi_context.damage_stop && (1..=120).contains(&sdi_context.remaining),
                state.trip_output.requested,
            );
            if key != state.last_combat_diagnostic {
                state.last_combat_diagnostic = key;
                state.pending_combat_log = Some(format!(
                    "combat assist: trip_queued={}, SDI_damage_stop={}, trip_request={:#x}, native_hitstop_remaining={}",
                    key.0, key.1, key.2, sdi_context.remaining));
            }
        }
        sdi_context.input_delay = input_delay;
        state.sdi_context = Some(sdi_context);
        state.observed_object = Some(object_id);
        state.evaluated_tick = Some(tick);
        // Tech needs the last GAME-visible ZR to decide whether a release is
        // required. The restored physical trace may differ from that last output.
        let previous_buttons = if state.last_output_buttons & MAPPER_ZR != 0 {
            HID_ZR
        } else {
            0
        };
        #[cfg(feature = "diagnostic-logging")]
        if config.controls.tech_debug {
            let relevant = (0x47..=0x5b).contains(&assessment.status)
                || (0x67..=0x6b).contains(&assessment.status)
                || (CATCHED_GANON..=0xc1).contains(&assessment.status);
            let key = (assessment.status, assessment.situation, assessment.reason);
            if relevant && state.last_tech_diagnostic != Some(key) {
                state.last_tech_diagnostic = Some(key);
                state.pending_tech_log = Some(format!("tech observation: input_update={tick}, status={:#x}, situation={}, reason={}, velocity=({:.2},{:.2}), lookahead={:.1}", assessment.status, assessment.situation, assessment.reason, assessment.velocity[0], assessment.velocity[1], config.lookahead));
                if let Some(message) = state.pending_tech_log.as_mut() {
                    message.push_str(&format!(
                        ", contact_in={:?}, hitstop_remaining={}, input_delay={input_delay:.1}, capture_clock={:?}",
                        assessment.contact_in, assessment.remaining_stop, assessment.capture_clock
                    ));
                }
            } else if !relevant {
                state.last_tech_diagnostic = None;
            }
        }
        // During hitstop, publish neutral assist without resetting the policy's
        // cooldown/history. Mapping polls are not advancing the damage clock.
        let policy_tick = state
            .animation_clock
            .get_or_insert_with(AnimationClock::default)
            .observe(
                AnimationSample {
                    object: object_id,
                    status: assessment.status,
                    kind: own_motion_kind,
                    frame: ledge_motion_frame,
                    auxiliary: if clatter { clatter_remaining } else { 0.0 },
                },
                stopped,
            );
        state.policy_tick = policy_tick;
        if policy_tick.is_none() {
            if let Some(policy) = state.policy.as_mut() {
                policy.pause_inputs();
            }
        }
        let auto_mash = state
            .ledge_menu
            .as_ref()
            .map_or(config.controls.auto_mash, |menu| {
                menu.auto_mash(config.controls.auto_mash)
            });
        let mash_requested = if let Some(policy_tick) = policy_tick {
            state
                .policy
                .get_or_insert_with(RawAutoControls::default)
                .update_with_buttons(
                    kirby_assist_core::live_input::LiveSettings {
                        auto_tech: false,
                        auto_mash,
                        ..config.controls
                    },
                    policy_tick,
                    object_id,
                    clatter,
                    false,
                    previous_buttons,
                )
        } else {
            0
        };
        let previous_guard = state.last_mapped_guard;
        state.tech_output = state
            .tech_input
            .get_or_insert_with(TechInput::default)
            .sample(
                config.controls.enabled && config.controls.auto_tech,
                config.controls.takeover,
                config.controls.tech_cooldown,
                TechContext {
                    object: object_id,
                    status: assessment.status,
                    motion: assessment
                        .capture_clock
                        .map_or(own_motion_kind, |clock| clock.0),
                    frame: assessment
                        .capture_clock
                        .map_or(ledge_motion_frame, |clock| clock.1),
                    rate: assessment
                        .capture_clock
                        .map_or(timing_rate, |clock| clock.2),
                    stopped,
                    remaining_stop: assessment.remaining_stop,
                    contact_in: if !clatter && tech {
                        assessment.contact_in
                    } else {
                        None
                    },
                    input_delay: input_delay,
                    previous_guard,
                },
            );
        let requested = mash_requested | if state.tech_output.press { HID_ZR } else { 0 };
        #[cfg(feature = "diagnostic-logging")]
        if config.controls.tech_debug {
            let cooldown = state
                .tech_input
                .as_ref()
                .map_or(0.0, TechInput::cooldown_remaining);
            if let Some(message) = state.pending_tech_log.as_mut() {
                message.push_str(&format!(", requested={requested:#x}, cooldown={cooldown}"));
            }
        }
        // Cap grounded physical inputs during recovery too, preventing a held
        // full stick from buffering a dash as the attack finishes. Damage/air
        // movement and higher-priority mash/tech keep their physical inputs.
        state.ledge_walk_ready = !clatter
            && !tech
            && assessment.situation == 0
            && (0..=0x40).contains(&assessment.status);
        state.shortcut_context = Some(shortcut_context);
        state.turbo_ready = !clatter
            && !tech
            && turbo_movement_allowed(assessment.status, assessment.situation, stopped);
        if let Some((mut context, opponent, edge, spot, recovery)) = ledge_observation {
            context.input_delay = 0.0;
            context.can_act = state.turbo_ready;
            let slide_buttons = state.raw_buttons;
            let slide_scope = slide_buttons & (DPAD_UP | DPAD_DOWN) == 0
                && !clatter
                && !tech
                && slide_buttons & MAPPER_MINUS == 0
                && state.trump_output.owned == 0;
            state.slide_output = if let Some(slide_tick) = policy_tick {
                state.slide.get_or_insert_with(SlideState::default).sample(
                    config.dpad_slide,
                    slide_tick,
                    slide_buttons,
                    SlideContext {
                        gameplay: slide_scope,
                        stopped,
                        status: context.status,
                        situation: context.situation,
                        motion_frame: context.motion_frame,
                        can_jump: context.can_jump && can_slide_jump,
                        can_air_dodge: can_slide_air_dodge,
                        safe_ground: context.safe_ground,
                        input_delay: 0.0,
                    },
                )
            } else {
                // Freeze progression when native animation time is unavailable;
                // never mix raw mapper ticks into the native slide clock.
                LedgeOutput::default()
            };
            let trump_held = state.raw_buttons & DPAD_DOWN != 0;
            state.trump_output = state.trump.get_or_insert_with(LedgeTrump::default).sample(
                config.auto_ledge_trump,
                tick,
                trump_held,
                LedgeContext {
                    gameplay: !clatter && !tech,
                    ..context
                },
                edge,
            );
            let held = state.ledge_active;
            // Preparation stays grounded. Continue only a policy-started hop and
            // keep ownership through attack/shield/grab animations; mash/tech keep their existing priority.
            // The policy can preserve ZR during native guard hitstop.
            let grounded = assessment.situation == 0
                && (state.turbo_ready
                    || matches!(assessment.status, 0x1b..=0x1e | 0x29..=0x32 | 0x37..=0x3d));
            let owned_jump = state
                .ledge
                .as_ref()
                .is_some_and(|trap| trap.owns_jump_state(assessment.status));
            let can_control = !stopped && !clatter && !tech && (grounded || owned_jump);
            // can_act permits a new grounded action. can_control also covers
            // our ongoing shield/tilt/dash attack/grab, allowing ownership without repeatedly
            // launching actions during recovery. Do not collapse these flags.
            context.held = held && state.trump_output.owned == 0 && state.slide_output.owned == 0;
            context.can_act = state.turbo_ready;
            context.can_control = can_control;
            if context.situation != 0 {
                state.ledge_position = None;
                state.ledge_seek_tick = None;
            }
            if held
                && context.situation == 0
                && (config.ledge.two_frame
                    || !config.ledge.manual_spacing && !config.ledge.distance_agnostic)
            {
                if find_trap_edge && state.turbo_ready && !stopped && !clatter && !tech {
                    state.ledge_seek_tick = Some(tick);
                }
                state
                    .ledge_position
                    .get_or_insert_with(LedgePosition::default)
                    .lock(spot);
                if let Some(spot) = state
                    .ledge_position
                    .as_ref()
                    .and_then(LedgePosition::spot)
                    .filter(|_| !config.ledge.two_frame)
                {
                    state
                        .ledge
                        .get_or_insert_with(LedgeTrap::default)
                        .prepare_stance(spot.edge, spot.inward);
                }
            }
            state.ledge_output = if config.ledge.two_frame {
                let spot = state.ledge_position.as_ref().and_then(LedgePosition::spot);
                state
                    .two_frame
                    .get_or_insert_with(TwoFrame::default)
                    .sample(
                        config.ledge,
                        LedgeContext {
                            gameplay: context.gameplay && grounded && !clatter && !tech,
                            ..context
                        },
                        spot,
                        recovery,
                    )
            } else {
                state.ledge.get_or_insert_with(LedgeTrap::default).sample(
                    config.ledge,
                    tick,
                    context,
                    opponent,
                )
            };
            // Keep observing options during approach. An actionable opponent
            // response takes priority immediately; hanging/no-target uses the
            // locked, damped preparation controller instead of root chasing.
            #[cfg(feature = "diagnostic-logging")]
            let mut positioning_action = None;
            if held
                && !config.ledge.two_frame
                && !state
                    .ledge
                    .as_ref()
                    .is_some_and(LedgeTrap::fighter_aerial_active)
                && !state
                    .ledge
                    .as_ref()
                    .is_some_and(LedgeTrap::manual_after_response)
                && !config.ledge.manual_spacing
                && !config.ledge.distance_agnostic
                && context.situation == 0
                && state.trump_output.owned == 0
                && !clatter
                && !tech
            {
                let preparing = !state
                    .ledge
                    .as_ref()
                    .is_some_and(LedgeTrap::owns_roll_navigation)
                    && (state.ledge_output.owned == 0
                        || state.ledge.as_ref().is_some_and(|trap| {
                            matches!(trap.action_name(), "finished" | "return_to_ledge")
                        })
                        || opponent.is_some_and(|e| (CLIFF_CATCH_MOVE..=0x77).contains(&e.status)));
                if preparing {
                    let movement = state
                        .ledge_position
                        .get_or_insert_with(LedgePosition::default)
                        .sample(tick, context, config.ledge);
                    #[cfg(feature = "diagnostic-logging")]
                    {
                        positioning_action = state
                            .ledge_position
                            .as_ref()
                            .map(LedgePosition::action_name);
                    }
                    if movement.owned != 0 {
                        state.ledge_output = movement;
                    }
                }
            }
            #[cfg(feature = "diagnostic-logging")]
            {
                let metadata = state.ledge.as_ref().and_then(|trap| {
                    let slot = trap.calibration_slot()?;
                    let profile = PROFILES.get()?.profile(state.fighter_kind?)?;
                    let list = profile.menu_options.list(slot);
                    let index =
                        config.ledge.response_choices[slot as usize].unwrap_or(list.selected);
                    let choice = list.options.get(index)?;
                    let mut frame =
                        trap.calibration_frame(config.ledge, context, opponent, choice.id)?;
                    frame.output = state.ledge_output;
                    Some((
                        frame,
                        profile.name,
                        kirby_assist_core::fighter_profiles::spacing::menu_name(choice.name),
                    ))
                });
                if held && !config.ledge.two_frame {
                    if let Some((frame, fighter, choice)) = metadata {
                        state.calibration.observe(
                            Some(frame),
                            (fighter, fighter_kind),
                            choice,
                            now,
                        );
                    } else {
                        state.calibration.observe(None, ("", fighter_kind), "", now);
                    }
                } else {
                    state.calibration.reset("autopilot released/other mode");
                }
            }
            #[cfg(feature = "diagnostic-logging")]
            let grab_snapshot = if held && !config.ledge.two_frame {
                state
                    .ledge
                    .as_ref()
                    .and_then(LedgeTrap::roll_grab_diagnostic)
            } else {
                None
            };
            #[cfg(feature = "diagnostic-logging")]
            {
                let fighter = state.fighter_kind;
                if !held {
                    state.grab_trace.reset("autopilot_released");
                } else {
                    state.grab_trace.observe(grab_snapshot, fighter, now);
                }
            }
            #[cfg(feature = "diagnostic-logging")]
            if held && grab_snapshot.is_none() {
                let key = LedgeDiagnostic {
                    target: opponent.map(|enemy| enemy.id),
                    enemy_status: opponent.map_or(-1, |enemy| enemy.status),
                    vulnerable: opponent.is_some_and(|enemy| enemy.vulnerable),
                    kirby_status: assessment.status,
                    can_act: context.can_act,
                    can_grab: context.can_grab,
                    can_dash_attack: context.can_dash_attack,
                    can_jump: context.can_jump,
                    can_air_attack: context.can_air_attack,
                    hit_confirmed: context.hit_confirmed,
                    own_motion: own_motion_kind,
                    requested: state.ledge_output.requested,
                    action: if config.ledge.two_frame {
                        state
                            .two_frame
                            .as_ref()
                            .map_or("none", TwoFrame::action_name)
                    } else {
                        positioning_action.unwrap_or_else(|| {
                            state.ledge.as_ref().map_or("none", LedgeTrap::action_name)
                        })
                    },
                };
                let origin = *state.ledge_log_origin.get_or_insert(now);
                let elapsed_ms = now
                    .saturating_duration_since(origin)
                    .as_millis()
                    .min(u128::from(u64::MAX)) as u64;
                if let Some(emission) = state.ledge_diagnostics.sample_throttled(
                    key,
                    elapsed_ms,
                    config.controls.tech_debug,
                    125,
                ) {
                    state.pending_ledge_log = Some(format!(
                        concat!(
                            "ledge observation: input_update={tick}, policy_tick={policy_tick:?}, ",
                            "periodic={}, coalesced_updates={}, ",
                            "target={:?}, enemy_status={:#x}, vulnerable={}, ",
                            "kirby_status={:#x}, own_motion={:#x}, motion_frame={:.2}, ",
                            "can_act={}, requested={:#x}, action={}, situation={}, ",
                            "previous_status={:#x}, can_grab={}, can_dash_attack={}, hit_confirmed={}, ",
                            "can_jump={}, can_air_attack={}, air_physics={:?}, short_hop_speed={:?}, ",
                            "shield_stun={:?}, safe_ground={:?}, position={:?}, ",
                            "target_position={:?}, target_vulnerable_in={:?}, ",
                            "target_previous_status={:?}, target_motion={:?}, target_actionable_in={:?}, ",
                            "target_observed_velocity={:?}, getup_attack={:?}, ",
                            "locked_ledge={:?}, stance_distance={}, roll_interception={:?}"
                        ),
                        emission.periodic,
                        emission.coalesced_updates,
                        key.target,
                        key.enemy_status,
                        key.vulnerable,
                        key.kirby_status,
                        own_motion_kind,
                        ledge_motion_frame,
                        key.can_act,
                        key.requested,
                        key.action,
                        assessment.situation,
                        context.previous_status,
                        context.can_grab,
                        context.can_dash_attack,
                        context.hit_confirmed,
                        context.can_jump,
                        context.can_air_attack,
                        context.air_physics,
                        context.short_hop_speed,
                        context.shield_stun_frames,
                        context.safe_ground,
                        context.position,
                        opponent.map(|enemy| enemy.position),
                        opponent.and_then(|enemy| enemy.vulnerable_in),
                        opponent.map(|enemy| enemy.previous_status),
                        opponent.and_then(|enemy| enemy.motion),
                        opponent.and_then(|enemy| enemy.actionable_in),
                        state
                            .ledge
                            .as_ref()
                            .map(LedgeTrap::observed_target_velocity),
                        config.ledge.getup_attack,
                        state.ledge_position.as_ref().and_then(LedgePosition::spot),
                        config.ledge.stand_distance,
                        state.ledge.as_ref().and_then(LedgeTrap::roll_interception),
                        tick = tick,
                        policy_tick = policy_tick,
                    ));
                }
            } else {
                state.ledge_diagnostics = DiagnosticGate::new();
                state.ledge_log_origin = None;
                state.pending_ledge_log = None;
            }
        } else {
            #[cfg(feature = "diagnostic-logging")]
            state.grab_trace.reset("ledge_observation_unavailable");
            #[cfg(feature = "diagnostic-logging")]
            state.calibration.reset("ledge observation unavailable");
            state.two_frame = None;
            state.ledge_output = LedgeOutput::default();
            state.trump_output = LedgeOutput::default();
            state.trump = None; // Clear released Down history without native reads.
            state.slide_output = LedgeOutput::default();
            state.slide = None;
        }
        state.requested = requested;
        state.mash_stick = if policy_tick.is_none() {
            None
        } else {
            state.policy.as_ref().and_then(RawAutoControls::mash_stick)
        };
        // Mash takeover suppresses competing physical action buttons; it never
        // presses them. Tech requests ZR; its takeover release also clears the
        // other shoulder buttons so mapped GUARD can have a fresh edge.
        state.owned = if config.controls.takeover && state.mash_stick.is_some() {
            ASSIST_BUTTONS
        } else if config.controls.takeover && state.tech_output.owns_guard {
            HID_ZR
        } else {
            0
        };
        state.request_seen = Some(now);
        #[cfg(feature = "diagnostic-logging")]
        if state.last_logged_scenario != (clatter, tech) {
            state.last_logged_scenario = (clatter, tech);
            scenario_changed = true;
        }
    }
    #[cfg(feature = "diagnostic-logging")]
    if first_button_write {
        log!("raw controller mapper button overlay confirmed; game acceptance still depends on its input rules");
    }
    #[cfg(feature = "diagnostic-logging")]
    if acquired {
        log!(&format!(
            "observing human fighter object {object_id:#x}, input index {index}"
        ));
    }
    #[cfg(feature = "diagnostic-logging")]
    if scenario_changed {
        log!(&format!(
            "scenario: mashable={clatter}, predicted_tech_collision={tech}"
        ));
    }
}

// Startup is deliberately sequential: prepare diagnostics, parse config, locate
// exactly one supported mapper, resolve the manager, THEN install hooks. A failed
// prerequisite exits without installing a partially configured set of hooks.
pub fn install() {
    // Silent builds only READ configuration: no directory/file creation or
    // console output is compiled into this plugin's normal startup path.
    #[cfg(feature = "diagnostic-logging")]
    crate::logging::start();
    let (config, profiles) = match load_config() {
        Ok(settings) => settings,
        Err(_error) => {
            log!(&format!("NOT INSTALLED: {_error}"));
            return;
        }
    };
    let _ = CONFIG.set(config);
    let _ = PROFILES.set(profiles);
    if !config.controls.enabled {
        log!("disabled in config; no hooks installed");
        return;
    }
    unsafe {
        let start = getRegionAddress(Region::Text) as usize;
        let end = getRegionAddress(Region::Rodata) as usize;
        if start == 0 || end <= start {
            log!("NOT INSTALLED: invalid executable region");
            return;
        }
        // Build a borrowed view of executable bytes for the signature scan.
        // No instructions are changed by this read; Skyline installs the hooks
        // only after the unique match and manager lookup both succeed below.
        let text = std::slice::from_raw_parts(start as *const u8, end - start);
        let Some(offset) = unique_instruction_offset(text, MAPPING_NEEDLE) else {
            log!("NOT INSTALLED: raw input mapper signature missing or ambiguous on this game/mod build");
            return;
        };
        let mut slot = 0usize;
        skyline::nn::ro::LookupSymbol(
            &mut slot,
            c"_ZN3lib9SingletonIN3app14FighterManagerEE9instance_E"
                .as_ptr()
                .cast(),
        );
        if slot == 0 {
            log!("NOT INSTALLED: FighterManager singleton unavailable");
            return;
        }
        MAPPING_OFFSET.store(offset, Ordering::Relaxed);
        MANAGER_SLOT.store(slot, Ordering::Relaxed);
        skyline::install_hooks!(map_controls, observe_commands);
        ledge_menu::install();
        log!(&format!("hooks installed; mapper={offset:#x}; config={config:?}; physical-controller routing; pre-map fighter refresh; callback-only fighter discovery"));
    }
}
