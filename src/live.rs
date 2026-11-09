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
// live in children. `pub(super)` exposes a helper only to this module's family.
// Keep these child declarations together so the integration's file layout is
// visible before its imports and runtime implementation.
mod config;
mod controller;
mod damage;
mod attack_sources;
mod frame;
#[path = "live/fighters/kirby/followups.rs"]
mod kirby_followups;
mod native_params {
    pub(super) const PARAM_COMMON: u64 = 0x6e5ec7051;
    pub(super) const PARAM_GROUND_BRAKE: u64 = 0x0c8cc9db76;
    pub(super) const PARAM_AIR_ACCEL_Y: u64 = 0xba18057d9;
    pub(super) const PARAM_AIR_SPEED_Y_STABLE: u64 = 0x12ec5626fe;
    pub(super) const PARAM_DAMAGE_FLY_TOP_GRAVITY: u64 = 0x1a58828323;
    pub(super) const PARAM_DAMAGE_FLY_TOP_TERMINAL_SPEED: u64 = 0x1d71e21b0b;
    pub(super) const PARAM_NO_RAPID_FRAME_VALUE: u64 = 0x14459066dd;
    pub(super) const PARAM_PASSIVE_TRIGGER_FRAME: u64 = 0x15ba3cdb02;
    pub(super) const PARAM_GANON_SPECIAL_S_PASSIVE_TRIGGER_FRAME: u64 = 0x2535742c20;
}
mod observations;
#[path = "live/fighters/game_and_watch/reflect.rs"]
mod reflect;
#[path = "live/runtime.rs"]
mod runtime;
mod settings_menu;
mod state;
mod tech;

use config::load_config;
use controller::{
    controller_from_wrapper, selected_controller, trace, write_trace, ControllerRestore,
    MappedInputs,
};
use kirby_assist_core::action_owner::{ActionOwnership, Intent, Owner};
use kirby_assist_core::training_test;
use observations::{combat_targets, fighter_ids, human_fighter, nearest_ledge_opponent};
use runtime::set_runtime_enabled;
use state::State;
use tech::assess_tech;

// In silent builds, this macro removes both the log call and its arguments.
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

mod training;

use crate::game_ffi as game;
use kirby_assist_core::combat_assist::{SdiContext, SdiState};
#[cfg(feature = "diagnostic-logging")]
use kirby_assist_core::diagnostics::DiagnosticGate;
use kirby_assist_core::fighter_profiles::FighterProfiles;
use kirby_assist_core::gameplay::{ControllerOutput, GameplayContext};
use kirby_assist_core::ledge_position::{nearest_ground_ledge, LedgePosition};
use kirby_assist_core::ledge_trap::{LedgeTrap, CLIFF_CATCH_MOVE};
use kirby_assist_core::ledge_trump::{nearest_floor_edge, LedgeTrump};
use kirby_assist_core::live_input::{
    mapper_hook::mapper_offset, LiveConfig as Config, RawAutoControls, HID_ZR,
};
use kirby_assist_core::manual_controls::{
    ledge_walk_stick, suppress_reserved_taunts, turbo_movement_allowed, DashPivotAssist,
    ManualContext, ManualOverlay, TurboOutput, TurboState, DPAD_SIDES, DPAD_UP, TURBO_TRIGGER,
};
use kirby_assist_core::native_input::{
    controller_index, selected_controller_mapper, MAPPER_DPAD_DOWN, MAPPER_L, MAPPER_MINUS,
    MAPPER_PLUS, MAPPER_ZR,
};
#[cfg(feature = "diagnostic-logging")]
use kirby_assist_core::native_input::{MAPPER_A, MAPPER_X, MAPPER_Y};
use kirby_assist_core::session::{MatchSession, SessionTicket};
use kirby_assist_core::settings::AssistSettings;
use kirby_assist_core::tech_input::{TechContext, TechInput, TechOutput};
use kirby_assist_core::timing::{AnimationClock, AnimationSample};
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
// Recognition lives in the host-testable mapper_hook module, including the
// known Skyline entry patches. Resolve once; install after plugin startup.
static MAPPER_INSTALLED: AtomicBool = AtomicBool::new(false);
static TRAINING_MODPACK_PRESENT: AtomicBool = AtomicBool::new(false);
static TRAINING_PASSTHROUGH: AtomicBool = AtomicBool::new(false);
static TRAINING_MODAL_HANDOFF: AtomicBool = AtomicBool::new(false);
// Startup config authorizes the plugin installation. This runtime gate can be
// toggled only outside battle with Plus+Minus and leaves one tiny UI/HID check
// alive so a fully disabled mod can be re-enabled without restarting Smash.
static RUNTIME_ENABLED: AtomicBool = AtomicBool::new(true);
static MASTER_IN_MATCH: AtomicBool = AtomicBool::new(false);
static MASTER_PULSE_REQUESTED: AtomicBool = AtomicBool::new(true);
static MASTER_PRESENCE: Mutex<kirby_assist_core::session::MenuPresence> =
    Mutex::new(kirby_assist_core::session::MenuPresence::new());

fn refresh_master_match_lock(ready_go: Option<bool>) {
    // Paused matches retain GO; intro callbacks supply a short conservative
    // lease before GO. No assist observation/state is required while OFF.
    let presence_is_recent = MASTER_PRESENCE
        .try_lock()
        .map_or(true, |presence| presence.fresh(Instant::now()));
    let match_is_active = ready_go == Some(true) || presence_is_recent;
    MASTER_IN_MATCH.store(match_is_active, Ordering::Release);
}
// Atomics hold small installation/diagnostic values independently of STATE.
// OnceLock publishes one immutable startup config. The Mutex protects mutable
// controller/policy history shared by callbacks; never call game APIs while
// holding it, since a native call can re-enter another hooked function.
static MAPPING_OFFSET: AtomicUsize = AtomicUsize::new(0);
#[cfg(feature = "diagnostic-logging")]
const ATTACK_SLOT_COUNT: i32 = 8;
#[cfg(feature = "diagnostic-logging")]
static SEEN_MAPPERS: AtomicUsize = AtomicUsize::new(0);
#[cfg(feature = "diagnostic-logging")]
static ROUTE_MISMATCHES: AtomicUsize = AtomicUsize::new(0);
#[cfg(feature = "diagnostic-logging")]
static ROUTE_UNAVAILABLE_LOGGED: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "diagnostic-logging")]
static MANUAL_GRAB_COMPARISON_POLLS: AtomicUsize = AtomicUsize::new(8);
#[cfg(feature = "diagnostic-logging")]
static AUTOMATIC_GRAB_COMPARISON_POLLS: AtomicUsize = AtomicUsize::new(8);
static MANAGER_SLOT: AtomicUsize = AtomicUsize::new(0);
static CONFIG: OnceLock<Box<Config>> = OnceLock::new();
static PROFILES: OnceLock<FighterProfiles> = OnceLock::new();
static SEQUENCES: OnceLock<Mutex<kirby_assist_core::sequence_config::SequenceLibrary>> =
    OnceLock::new();
/// Explicit saves replace compact definitions without publishing static menus.
static SEQUENCES_UPDATED: AtomicBool = AtomicBool::new(false);
// STATE.lock() yields a guard. Rust releases the lock when the guard leaves
// its scope (`}`); explicit small scopes below let us copy values, unlock, query
// the game, then lock again to publish results. `if let Ok` skips poisoned locks
// instead of panicking inside a native callback.
static STATE: Mutex<State> = Mutex::new(State::new());
fn diagnostic_config(config: Config) -> Config {
    #[cfg(feature = "diagnostic-logging")]
    let perf = STATE
        .lock()
        .ok()
        .and_then(|state| {
            state
                .settings_menu
                .as_ref()
                .map(|menu| menu.diagnostic_options())
        })
        .unwrap_or_default();
    #[cfg(feature = "diagnostic-logging")]
    crate::performance::set_enabled(perf);
    config
}
static OBSERVING_FIGHTER: AtomicBool = AtomicBool::new(false);

// Only a freshly revalidated selected-controller route may authorize gameplay,
// UI telemetry or projectile tracking. Native ownership discovery stays read-only.
static LOCAL_PARTICIPANT: AtomicBool = AtomicBool::new(false);
pub(crate) use runtime::feature_activity_allowed;

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
// Same-update proposals also have a wall-clock timeout. A timestamp alone
// never authorizes replay into a later selected-controller update.
const REQUEST_LIFETIME: Duration = Duration::from_millis(35);
// A longer gap between valid fighter observations marks a loading transition,
// even when the manager singleton stays allocated across online scenes.
const SESSION_OBSERVATION_GAP: Duration = Duration::from_millis(250);
// Online scheduling may separate controller sampling from fighter callbacks.
// Proposals still require a current evaluation and expire at 35 ms.
const CONTROLLER_SAMPLE_LIFETIME: Duration = Duration::from_millis(250);
/// Resolution copies small typed values; no JSON parsing or file I/O occurs here.
fn settings_for_fighter(kind: Option<i32>, base: AssistSettings) -> AssistSettings {
    PROFILES.get().map_or(
        AssistSettings {
            enabled: false,
            ..base
        },
        |profiles| profiles.settings_for_fighter(kind, base),
    )
}

// Diagnostic snapshots are absent from silent builds, including formatting.
#[cfg(feature = "diagnostic-logging")]
#[derive(Clone, Copy, PartialEq, Eq)]
/// Represents a diagnostic snapshot of the ledge state for the fighter.
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
/// Acquires a live match ticket for the current session,
/// if available. Returns `None` if the ticket cannot be acquired.
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
/// Represents the observation needs for the current state and configuration.
struct ObservationNeeds {
    find_trump_edge: bool,
    find_trap_edge: bool,
    observe_ledge: bool,
    preview_start: bool,
    continue_ledge_response: bool,
    literal_ledge_timeline: bool,
    file_ledge_timeline: bool,
    ground_ledge_timeline: bool,
    observe_trump: bool,
}
/// Captures a snapshot of the observation policy for the current state.
#[derive(Default)]
struct ObservationPolicySnapshot {
    automatic_suspended: bool,
    trip_queued: bool,
    grab_dodge_enabled: bool,
    posture_modes: (bool, bool),
    posture_down: bool,
    roster_missing: bool,
    oos_enabled: bool,
    oos_target: Option<u32>,
    rules_on: bool,
    rule_target: Option<u32>,
    dodge_follow: Option<u32>,
    automatic_sequence_committed: bool,
    up_b_receipt: bool,
    rules_ground: Option<(f32, f32, bool)>,
    simple_starts: [Option<kirby_assist_core::reaction_rules::SimpleStart>; 3],
    dodge_lookahead: f32,
    manual_input_epoch: u64,
    analysis_settings: kirby_assist_core::fighter_tuning::Tuning,
    automatic_limits: [Option<(f32, f32)>; 3],
    dodge_direct_up_b: bool,
    want_up_special: bool,
    ordinary_backup: bool,
    stationary_intent: bool,
    grab_intent: bool,
    self_shielded: bool,
    grab_dash_pending: bool,
}
/// Determines the observation needs based on the current state and configuration.
fn observation_needs(state: &State, config: &Config) -> ObservationNeeds {
    let trump_held = state
        .settings_menu
        .as_ref()
        .is_some_and(|menu| menu.ledge_trump_held());
    let preview_selected = state
        .settings_menu
        .as_ref()
        .is_some_and(|m| m.preview_slot().is_some());
    let file_ledge_timeline = state
        .ledge
        .as_ref()
        .is_some_and(LedgeTrap::file_timeline_active);

    ObservationNeeds {
        find_trump_edge: state
            .trump
            .as_ref()
            .is_none_or(|trump| trump.needs_edge(trump_held))
            && state
                .trump_seek_tick
                .is_none_or(|last| state.raw_tick.wrapping_sub(last) >= 30)
            && trump_held,
        find_trap_edge: config.assist.enabled
            && config.assist.auto_position
            && !config.assist.manual_spacing
            && !config.assist.distance_agnostic
            && state.ledge_active
            && !preview_selected
            && state
                .ledge_seek_tick
                .is_none_or(|last| state.raw_tick.wrapping_sub(last) >= 30)
            && state
                .ledge_position
                .as_ref()
                .is_none_or(|p| p.spot().is_none()),
        observe_ledge: config.assist.enabled && state.ledge_active,
        preview_start: preview_selected && !file_ledge_timeline,
        continue_ledge_response: state
            .ledge
            .as_ref()
            .is_some_and(LedgeTrap::needs_response_observation),
        literal_ledge_timeline: state
            .ledge
            .as_ref()
            .is_some_and(LedgeTrap::file_inputs_only),
        file_ledge_timeline,
        ground_ledge_timeline: state
            .ledge
            .as_ref()
            .is_some_and(LedgeTrap::file_needs_ground),
        observe_trump: trump_held || state.trump.as_ref().is_some_and(LedgeTrump::active),
    }
}
/// Resolves the observation setup for the given fighter and session,
///  returning the configuration, setup sequence, and observation needs if available.
fn resolve_observation_setup(
    index: i32,
    session_ticket: SessionTicket,
    fighter_kind: i32,
    config: Config,
) -> Option<(Config, u64, ObservationNeeds)> {
    let (assist, setup_sequence) = {
        let state = STATE.lock().ok()?;
        if !state.session.accepts(session_ticket) || state.mapping_index != Some(index) {
            return None;
        }
        let assist = state
            .settings_menu
            .as_ref()
            .map(|menu| menu.settings(Some(fighter_kind), &config.assist))
            .unwrap_or_else(|| settings_for_fighter(Some(fighter_kind), config.assist));
        (assist, state.observation_sequence)
    };
    let config = Config { assist, ..config };
    let needs = {
        let state = STATE.lock().ok()?;
        observation_needs(&state, &config)
    };
    Some((config, setup_sequence, needs))
}
/// Determines whether manual automatic input is currently blocked for the given fighter and context.
fn manual_automatic_input_blocked(
    object_id: u32,
    fighter_kind: i32,
    context: GameplayContext,
) -> bool {
    STATE.lock().ok().map_or(true, |mut state| {
        let actions = state
            .settings_menu
            .as_ref()
            .and_then(|menu| menu.for_fighter(fighter_kind))
            .map_or(0, |menu| menu.manual_native_actions());
        state.grab_handoff.observe_native(true, Some(context));
        let manual_allowed = state.action_owner.current_owner().is_none();
        let pending = state.manual_jump_fence.sample(
            object_id,
            fighter_kind,
            manual_allowed && actions & kirby_assist_core::native_input::SEQUENCE_JUMP_BUTTONS != 0,
            context,
        );
        manual_allowed && (pending || actions != 0)
    })
}
/// Captures a snapshot of the observation policy for the given fighter, configuration, and context.
fn snapshot_observation_policy(
    fighter_kind: i32,
    manual_automatic_blocked: bool,
    native_status: i32,
) -> ObservationPolicySnapshot {
    STATE
        .lock()
        .ok()
        .map_or_else(ObservationPolicySnapshot::default, |state| {
            let menu = state
                .settings_menu
                .as_ref()
                .and_then(|menu| menu.for_fighter(fighter_kind));
            let committed_automatic = state.action_owner.automatic_active();
            let automatic_suspended = state.fighter_kind != Some(fighter_kind)
                || !committed_automatic
                    && (manual_automatic_blocked
                        || state.ledge_active
                        || menu.is_some_and(|menu| menu.reserves_automatic_modes()));
            let rules = (!automatic_suspended)
                .then(|| menu.and_then(|menu| menu.rule_config()))
                .flatten();
            let dodge_enabled =
                !automatic_suspended && menu.is_some_and(|menu| menu.grab_dodge_enabled());
            let grabs_enabled =
                !automatic_suspended && menu.is_some_and(|menu| menu.out_of_shield_enabled());
            let event_mask = if rules.is_some() { 0b111 } else { 0 }
                | if grabs_enabled {
                    (1 << 3) | (1 << 4) | (1 << 6)
                } else {
                    0
                }
                | if dodge_enabled { 1 << 5 } else { 0 };
            let cached_rules = (!automatic_suspended)
                .then(|| menu.and_then(|menu| menu.automatic_rules()))
                .flatten();
            let ground_watch =
                cached_rules.and_then(|rules| rules.automatic_ground_watch(event_mask));
            let want_up_special =
                cached_rules.is_some_and(|rules| rules.automatic_needs_up_special(event_mask));
            let ordinary_backup = grabs_enabled
                && cached_rules.is_some_and(|rules| {
                    rules.simple_sequence(6).is_some() && !rules.automatic_needs_up_special(1 << 6)
                });
            let (rule_target, dodge_follow, automatic_sequence_committed, up_b_receipt) =
                if automatic_suspended {
                    (None, None, false, false)
                } else {
                    (
                        state
                            .rule_engine
                            .as_ref()
                            .and_then(|engine| engine.target()),
                        state
                            .spot_dodge
                            .as_ref()
                            .and_then(|dodge| dodge.follow_target()),
                        state
                            .rule_engine
                            .as_ref()
                            .is_some_and(|engine| engine.committed())
                            || state.oos.configured_committed()
                                && !state.oos.grab_delivery_pending()
                            || state.spot_dodge.as_ref().is_some_and(|dodge| {
                                dodge.punish_pending() && !dodge.grab_delivery_pending()
                            }),
                        state.oos.up_b_receipt_pending()
                            || state
                                .rule_engine
                                .as_ref()
                                .is_some_and(|engine| engine.up_b_receipt_pending())
                            || state
                                .spot_dodge
                                .as_ref()
                                .is_some_and(|dodge| dodge.up_b_receipt_pending()),
                    )
                };
            let rules_ground = ground_watch.map(|(distance, height)| {
                (
                    distance,
                    height,
                    state
                        .rule_engine
                        .as_ref()
                        .is_some_and(|engine| engine.needs_ground())
                        || state.oos.needs_ground()
                        || state
                            .spot_dodge
                            .as_ref()
                            .is_some_and(|dodge| dodge.needs_ground()),
                )
            });

            ObservationPolicySnapshot {
                automatic_suspended,
                trip_queued: !automatic_suspended
                    && kirby_followups::watching(&state.trip_followup),
                grab_dodge_enabled: dodge_enabled,
                posture_modes: if !automatic_suspended {
                    menu.map_or((false, false), |menu| menu.posture_modes())
                } else {
                    (false, false)
                },
                posture_down: state.posture_down,
                roster_missing: state.ledge_roster.is_none(),
                oos_enabled: grabs_enabled,
                oos_target: if automatic_suspended {
                    None
                } else {
                    state.oos.target()
                },
                rules_on: rules.is_some(),
                rule_target,
                dodge_follow,
                automatic_sequence_committed,
                up_b_receipt,
                rules_ground,
                simple_starts: rules.map_or([None; 3], |rules| rules.simple_starts()),
                dodge_lookahead: menu.map_or(kirby_assist_core::spot_dodge::LOOKAHEAD, |menu| {
                    menu.dodge_lookahead()
                }),
                manual_input_epoch: state.action_owner.physical_inputs().epoch(),
                analysis_settings: menu
                    .map_or(Default::default(), |menu| menu.fighter_tuning(fighter_kind)),
                automatic_limits: menu.map_or([None; 3], |menu| menu.automatic_limits()),
                dodge_direct_up_b: menu
                    .and_then(|menu| menu.grab_dodge_sequence())
                    .is_none_or(kirby_assist_core::reaction_rules::starts_direct_up_b),
                want_up_special,
                ordinary_backup,
                stationary_intent: state
                    .action_owner
                    .policy_inputs()
                    .stationary_assist(native_status),
                grab_intent: state
                    .action_owner
                    .policy_inputs()
                    .grounded_grab_assist(native_status),
                self_shielded: state.action_owner.policy_inputs().shield_held()
                    || matches!(
                        native_status,
                        kirby_assist_core::native_states::GUARD_ON
                            ..=kirby_assist_core::native_states::GUARD_DAMAGE
                    ),
                grab_dash_pending: state.oos.approach_pending(),
            }
        })
}
/// Refreshes the master state for the specified controller, returning the readiness status if available.
unsafe fn refresh_master_for_controller(
    wrapper: *mut c_void,
    index: i32,
    config: &Config,
) -> Option<Option<bool>> {
    selected_controller(wrapper, index, config.player_index)?;
    let ready = observations::battle_menu_ready_go();
    refresh_master_match_lock(ready);
    Some(ready)
}

/// Represents the admission state for the mapper callback, including configuration, readiness, and session ticket.
struct MapperAdmission {
    config: &'static Config,
    master_ready_go: Option<Option<bool>>,
    session_ticket: SessionTicket,
}
/// Represents the output adjustments for the mapper callback, including stick input, automatic requests, and other control flags.
struct MapperOutputAdjustments {
    left_stick: Option<[f32; 2]>,
    right_stick: Option<[f32; 2]>,
    automatic_request: Option<u32>,
    short_hop: bool,
    attack_hold: bool,
    literal_request: Option<u32>,
    unshielded_auto_grab: bool,
    tech_output: Option<TechOutput>,
}

/// Represents the feedback delivered to the mapper callback, including manual and automatic actions, grab delivery, and other relevant events.
#[derive(Clone, Copy)]
struct MapperDeliveryFeedback {
    manual_jump: bool,
    reflect: bool,
    reflector_tick: Option<i64>,
    auto_grab: Option<(i64, u32, bool)>,
    grab_handoff: Option<(i64, Option<Owner>, bool, bool)>,
    dodge_punish_grab: Option<(i64, u32)>,
    auto_dodge: Option<(i64, u32)>,
    auto_up_special: Option<(i64, u32, Owner)>,
    tech_applied: bool,
    #[cfg(feature = "diagnostic-logging")]
    grab_delivery: Option<(u32, u32, u32, &'static str)>,
}
/// Admits the mapper callback for the specified controller, returning the admission state if successful.
unsafe fn admit_mapper_callback(wrapper: *mut c_void, index: i32) -> Option<MapperAdmission> {
    let config = CONFIG.get().map(Box::as_ref)?;
    if !config.controls.enabled {
        return None;
    }

    // Keep the read-only battle gate and bounded menu maintenance active in
    // vanilla passthrough, before any mapper state lock.
    let master_ready_go = refresh_master_for_controller(wrapper, index, config);
    settings_menu::poll_maintenance();
    if !RUNTIME_ENABLED.load(Ordering::Acquire) {
        return None;
    }

    // Only the configured physical controller may establish participation.
    let controller = selected_controller(wrapper, index, config.player_index)?;
    if (*controller).connected == 0 {
        runtime::suspend_participation();
        return None;
    }
    let session_ticket = live_match_ticket()?;
    let candidate = runtime::mapper_candidate(index);
    if !candidate.is_some_and(|id| observations::valid_local_route(id, index)) {
        runtime::suspend_participation();
        return None;
    }

    LOCAL_PARTICIPANT.store(true, Ordering::Release);
    Some(MapperAdmission {
        config,
        master_ready_go,
        session_ticket,
    })
}
/// Enters the training passthrough mode for the specified controller and configuration, returning `true` if successful.
unsafe fn enter_training_passthrough(wrapper: *mut c_void, index: i32, config: &Config) -> bool {
    let selected_physical = CONFIG
        .get()
        .and_then(|config| selected_controller(wrapper, index, config.player_index))
        .map(|controller| trace(controller));
    let Some(physical) = selected_physical else {
        return true;
    };
    training::enter_passthrough(physical, index, config.control_layout)
}

/// Applies the specified output adjustments to the mapper output, modifying stick input, automatic requests, and other control flags.
unsafe fn apply_mapper_output_adjustments(
    output: *mut c_void,
    adjustments: MapperOutputAdjustments,
) {
    let mapped = output.cast::<MappedInputs>();
    // Preserve selected analog input after the native mapper's flick filtering.
    if let Some(stick) = adjustments.left_stick {
        (*mapped).left_x = kirby_assist_core::native_input::mapped_stick_axis(stick[0]);
        (*mapped).left_y = kirby_assist_core::native_input::mapped_stick_axis(stick[1]);
    }
    if let Some(right) = adjustments.right_stick {
        (*mapped).buttons =
            kirby_assist_core::native_input::mapped_sequence_cstick((*mapped).buttons, right);
        (*mapped).right_x = kirby_assist_core::native_input::mapped_stick_axis(right[0]);
        (*mapped).right_y = kirby_assist_core::native_input::mapped_stick_axis(right[1]);
    }
    if let Some(requested) = adjustments.automatic_request {
        (*mapped).buttons =
            kirby_assist_core::native_input::mapped_automatic_actions((*mapped).buttons, requested);
    }
    (*mapped).buttons =
        kirby_assist_core::native_input::mapped_short_hop((*mapped).buttons, adjustments.short_hop);
    (*mapped).buttons =
        kirby_assist_core::native_input::mapped_short_hop((*mapped).buttons, adjustments.short_hop);
    // Raw trigger history can lose A even while retaining the stick.
    (*mapped).buttons = kirby_assist_core::native_input::mapped_attack_hold(
        (*mapped).buttons,
        adjustments.attack_hold,
    );
    if let Some(requested) = adjustments.literal_request {
        (*mapped).buttons =
            kirby_assist_core::native_input::mapped_literal_buttons((*mapped).buttons, requested);
        (*mapped).buttons = kirby_assist_core::native_input::mapped_unshielded_auto_grab(
            (*mapped).buttons,
            requested,
            adjustments.unshielded_auto_grab,
        );
    }
    if let Some(tech) = adjustments.tech_output {
        (*mapped).buttons = kirby_assist_core::tech_input::mapped_guard((*mapped).buttons, tech);
    }
}
/// Records a manual jump input for the given state and mapped inputs, updating the manual jump fence if applicable.
fn record_mapped_manual_jump(state: &mut State, mapped: &MappedInputs, manual_jump: bool) {
    if manual_jump
        && mapped.buttons
            & (kirby_assist_core::native_input::MAPPED_JUMP
                | kirby_assist_core::native_input::MAPPED_JUMP_MINI)
            != 0
    {
        if let (Some(object), Some(kind), Some(context)) = (
            state.associated_object,
            state.fighter_kind,
            state.manual_context,
        ) {
            state.manual_jump_fence.sample(
                object,
                kind,
                true,
                GameplayContext {
                    gameplay: true,
                    status: context.status,
                    situation: context.situation,
                    motion_kind: context.motion,
                    motion_frame: context.frame,
                    motion_rate: context.rate,
                    stopped: context.stopped,
                    ..GameplayContext::default()
                },
            );
        }
    }
}
/// Records a mapped reflector input for the given state and mapped inputs, handling reflector delivery and receipt polling.
fn record_mapped_reflector(
    state: &mut State,
    mapped: &MappedInputs,
    reflect: bool,
    reflector_tick: Option<i64>,
) {
    if reflect
        && mapped.buttons & kirby_assist_core::native_input::MAPPED_SPECIAL != 0
        && mapped.left_y < -63
    {
        let tick = state.raw_tick;
        state.reflector.mapped(tick);
    }
    if let Some(tick) = reflector_tick {
        state.reflector.poll_receipt(tick);
        if !state.reflector.active() {
            state.reflect_output = ControllerOutput::default();
            state.action_owner.release(Owner::Reflect);
        }
    }
}
/// Records mapped grab inputs for the given state and mapped inputs, handling automatic grabs and grab handoff events.
fn record_mapped_grabs(
    state: &mut State,
    mapped: &MappedInputs,
    auto_grab: Option<(i64, u32, bool)>,
    grab_handoff: Option<(i64, Option<Owner>, bool, bool)>,
) {
    if let Some((tick, target, grab)) = auto_grab {
        if grab {
            state.oos.mapped_grab(tick, target);
        } else {
            state.oos.mapped_approach(tick, target);
        }
    }
    if let Some((tick, owner, grab, eligible)) = grab_handoff {
        state.grab_handoff.mapped(
            tick,
            owner,
            grab,
            mapped.buttons & kirby_assist_core::native_input::MAPPED_CATCH != 0,
            eligible,
        );
    }
}
/// Records mapped dodge inputs for the given state and mapped inputs, handling guard, punish grabs, and automatic dodges.
fn record_mapped_dodges(
    state: &mut State,
    mapped: &MappedInputs,
    guard: bool,
    punish_grab: Option<(i64, u32)>,
    auto_dodge: Option<(i64, u32)>,
) {
    let delivery_tick = state.raw_tick;
    if let Some(spot) = state.spot_dodge.as_mut() {
        if let Some((tick, target)) = punish_grab {
            spot.mapped_grab(tick, target);
        }
        spot.delivery_tick(delivery_tick);
    }
    if let Some((tick, target)) = auto_dodge {
        if let Some(spot) = state.spot_dodge.as_mut() {
            spot.attempted_dodge(tick, target);
            // Stop pressing only after actual Guard + Down delivery; failed
            // packets retain their bounded retry budget.
            if guard && mapped.left_y < 0 {
                spot.mapped_dodge(tick, target);
            }
        }
    }
}

/// Records mapped up special inputs for the given state and mapped inputs, handling out-of-shield, roll read, and spot dodge scenarios.
fn record_mapped_up_special(
    state: &mut State,
    mapped: &MappedInputs,
    request: Option<(i64, u32, Owner)>,
) {
    let Some((tick, target, owner)) = request else {
        return;
    };
    // Rejected packets spend delivery attempts but cannot arm a delayed
    // receipt. Confirmation requires final Special + Up.
    let delivered =
        kirby_assist_core::native_input::mapped_up_special_delivered(mapped.buttons, mapped.left_y);
    match owner {
        Owner::OutOfShield => {
            state.oos.attempted_up_special(tick, target);
            if delivered {
                state.oos.mapped_up_special(tick, target);
            }
        }
        Owner::RollRead => {
            if let Some(engine) = state.rule_engine.as_mut() {
                engine.attempted_up_special(tick, target);
                if delivered {
                    engine.mapped_up_special(tick, target);
                }
            }
        }
        Owner::OptionRead => {
            if let Some(spot) = state.spot_dodge.as_mut() {
                spot.attempted_up_special(tick, target);
                if delivered {
                    spot.mapped_up_special(tick, target);
                }
            }
        }
        _ => {}
    }
}

/// Records mapped tech inputs for the given state, handling guard and tech application scenarios.
fn record_mapped_tech(
    state: &mut State,
    guard: bool,
    tech_applied: bool,
    #[cfg(feature = "diagnostic-logging")] messages: &mut Vec<String>,
) {
    if tech_applied && guard && !state.last_mapped_guard {
        if let Some(tech) = state.tech_input.as_mut() {
            tech.mapped_press();
        }
        #[cfg(feature = "diagnostic-logging")]
        if CONFIG
            .get()
            .is_some_and(|config| config.controls.tech_debug)
        {
            messages
                .push("tech mapped: fresh GUARD edge confirmed; request cooldown committed".into());
        }
    }
    if !tech_applied && guard && !state.last_mapped_guard {
        if let Some(tech) = state.tech_input.as_mut() {
            tech.mapped_human_guard();
        }
    }
}

#[cfg(feature = "diagnostic-logging")]
fn record_mapped_grab_diagnostics(
    state: &mut State,
    mapped: &MappedInputs,
    grab_delivery: Option<(u32, u32, u32, &'static str)>,
) {
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
}

/// Records mapped grab diagnostics for the given state and mapped inputs, handling grab delivery events.
unsafe fn record_mapped_delivery(
    output: *mut c_void,
    index: i32,
    session_ticket: SessionTicket,
    feedback: MapperDeliveryFeedback,
    #[cfg(feature = "diagnostic-logging")] messages: &mut Vec<String>,
) {
    let mapped = &*output.cast::<MappedInputs>();
    let guard = mapped.buttons
        & (kirby_assist_core::native_input::MAPPED_GUARD
            | kirby_assist_core::native_input::MAPPED_GUARD_HOLD)
        != 0;
    let Ok(mut state) = STATE.lock() else {
        return;
    };
    if state.mapping_index != Some(index) || !state.session.accepts(session_ticket) {
        return;
    }

    record_mapped_manual_jump(&mut state, mapped, feedback.manual_jump);
    record_mapped_reflector(
        &mut state,
        mapped,
        feedback.reflect,
        feedback.reflector_tick,
    );
    record_mapped_grabs(
        &mut state,
        mapped,
        feedback.auto_grab,
        feedback.grab_handoff,
    );
    let delivery_tick = state.raw_tick;
    state.oos.delivery_tick(delivery_tick);
    if let Some(engine) = state.rule_engine.as_mut() {
        engine.delivery_tick(delivery_tick);
    }
    record_mapped_dodges(
        &mut state,
        mapped,
        guard,
        feedback.dodge_punish_grab,
        feedback.auto_dodge,
    );
    record_mapped_up_special(&mut state, mapped, feedback.auto_up_special);
    record_mapped_tech(
        &mut state,
        guard,
        feedback.tech_applied,
        #[cfg(feature = "diagnostic-logging")]
        messages,
    );
    state.last_mapped_guard = guard;
    state.last_mapped_stick = Some([mapped.left_x as f32 / 127., mapped.left_y as f32 / 127.]);
    #[cfg(feature = "diagnostic-logging")]
    record_mapped_grab_diagnostics(&mut state, mapped, feedback.grab_delivery);
}
/// Updates the mapper menu and input state for the specified controller, handling menu readiness, battle activity, and diagnostic logging.
unsafe fn update_mapper_menu_and_input(
    controller: *mut controller::Controller,
    index: i32,
    config: &Config,
    physical: kirby_assist_core::native_input::ButtonTrace,
    physical_stick: [f32; 2],
    physical_right_stick: [f32; 2],
    menu_ready_go: Option<bool>,
    menu_training_mode: bool,
    training_test_mode: bool,
    #[cfg(feature = "diagnostic-logging")] trump_pressed: &mut bool,
    #[cfg(feature = "diagnostic-logging")] messages: &mut Vec<String>,
) {
    if let Ok(mut state) = STATE.lock() {
        let intro_current = state.menu_intro_object.is_some_and(|intro| {
            state.human_routes[index as usize]
                .is_some_and(|(id, seen)| id == intro && seen.elapsed() < SESSION_OBSERVATION_GAP)
        });
        let local_fighter_current = (*controller).connected != 0
            && state.human_routes[index as usize].is_some_and(|(id, seen)| {
                Some(id) == state.associated_object && seen.elapsed() < SESSION_OBSERVATION_GAP
            });
        let menu_battle_active =
            local_fighter_current && menu_ready_go.is_some_and(|go| go || intro_current);
        if menu_ready_go != Some(false) {
            state.menu_intro_object = None;
        }
        if state.menu_battle_active != menu_battle_active {
            state.menu_battle_active = menu_battle_active;
            if menu_battle_active {
                #[cfg(feature = "imgui-menu")]
                state
                    .settings_menu
                    .get_or_insert_with(settings_menu::SettingsMenu::default)
                    .begin_match();
            } else {
                attack_sources::clear();
                state.menu_fighter_kind = None;
                if let Some(settings_menu) = state.settings_menu.as_mut() {
                    settings_menu.end_match();
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
                    messages.push(format!(
                        "selected local controller {} bound to mapper {index} (npad={}, style={})",
                        config.player_index,
                        (*controller).npad,
                        (*controller).style
                    ));
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
            let policy_buttons = if training_test_mode {
                training_test::policy_buttons(physical.held)
            } else {
                physical.held
            };
            let takeover_buttons = config.control_layout.takeover_buttons(policy_buttons);
            let policy_buttons = config.control_layout.policy_buttons(policy_buttons);
            state.raw_buttons = policy_buttons;
            state.action_owner.observe_manual(
                takeover_buttons,
                [
                    physical_stick[0],
                    physical_stick[1],
                    physical_right_stick[0],
                    physical_right_stick[1],
                ],
            );
            state.posture_down = kirby_assist_core::posture::Posture::any_tilt_intent(
                policy_buttons,
                physical_stick,
                physical_right_stick,
            );
            if physical.held & MAPPER_PLUS != 0 {
                state.cancel();
            }
            {
                let kind = state.menu_fighter_kind;
                let menu_available = state.menu_battle_active && physical.held & MAPPER_PLUS == 0;
                let gameplay = menu_available && state.fighter_kind.is_some();
                // Raw button/menu observations still run, but a committed
                // input owner cannot be replaced by a queued button tape.
                let damage_recovery = state
                    .manual_context
                    .is_some_and(|c| kirby_assist_core::ledge_trap::sequence_interrupted(c.status));
                let committed_inputs = state.action_owner.current_owner().is_some()
                    || state.tech_output.press
                    || state.tech_output.owns_guard
                    || state.mash_stick.is_some();
                let bindings_blocked = committed_inputs || damage_recovery;
                let safety_jump = !committed_inputs && damage_recovery;
                let changed = state
                    .settings_menu
                    .get_or_insert_with(settings_menu::SettingsMenu::default)
                    .input(
                        menu_available,
                        gameplay,
                        bindings_blocked,
                        safety_jump,
                        kind,
                        policy_buttons,
                        if training_test_mode {
                            training_test::policy_buttons(physical.held)
                        } else {
                            physical.held
                        },
                        config.control_layout,
                        &config.assist,
                        config.controls.auto_mash,
                        config.auto_spot_dodge,
                        config.controls.auto_tech,
                        menu_training_mode,
                    );
                if changed {
                    state.cancel_manual_tools();
                }
                let consumed = state.settings_menu.as_ref().map_or(0, |p| p.consumed);
                if let Some(settings_menu) = state.settings_menu.as_ref() {
                    settings_menu.display_battle(state.menu_battle_active && kind.is_some());
                }
                // L remains an assist trigger; Up remains available to bindings
                // diagonals. Menu-consumed sides/Down never reach tools.
                // Preserve Y for the first native-readiness check;
                // the later overlay still consumes its mapped action.
                let helper_consumed = consumed
                    & !state
                        .settings_menu
                        .as_ref()
                        .map_or(0, |m| m.builtin_consumed);
                let policy_buttons = state
                    .settings_menu
                    .as_ref()
                    .map_or(policy_buttons, |m| m.native_buttons);
                state.raw_buttons = kirby_assist_core::settings_menu::observation_buttons(
                    policy_buttons,
                    helper_consumed,
                );
                let ledge_held = config.controls.enabled
                    && state
                        .settings_menu
                        .as_ref()
                        .is_some_and(|p| p.sequence_enabled(config.assist.enabled))
                    && state.settings_menu.as_ref().is_some_and(|p| p.ledge_held);
                attack_sources::tracking(
                    config.autopilot_shield && ledge_held
                        || !ledge_held
                            && state
                                .settings_menu
                                .as_ref()
                                .is_some_and(|m| m.reflect_config(kind.unwrap_or(-1)).is_some()),
                );
                let manual_actions = state
                    .settings_menu
                    .as_ref()
                    .map_or(0, |m| m.manual_native_actions());
                let ledge_inputs_reserved = ledge_held
                    || state
                        .settings_menu
                        .as_ref()
                        .is_some_and(|m| m.reserves_automatic_modes());
                let suspend_auto = state.action_owner.current_owner().is_none()
                    && (ledge_inputs_reserved
                        || manual_actions != 0
                        || kirby_assist_core::native_input::automatic_escape(
                            config.control_layout,
                            physical.held,
                            physical.pressed,
                            false,
                            state.manual_context.is_some_and(|c| {
                                c.situation == kirby_assist_core::native_states::situation::GROUND
                            }),
                            0,
                        ));
                state.action_owner.set_automatic_suspended(suspend_auto);
                if suspend_auto {
                    state.suspend_automatic_punishes();
                }
                if ledge_inputs_reserved
                    && state.action_owner.current_owner().is_none()
                    && state.grab_handoff.reserve_ledge_inputs()
                {
                    // A queued Grab must settle before unrelated playback;
                    // only native confirmation/expiry can release its receipt.
                    state.oos.cancel_pending();
                    if let Some(spot) = state.spot_dodge.as_mut() {
                        spot.cancel_response();
                    }
                }
                #[cfg(feature = "diagnostic-logging")]
                {
                    *trump_pressed = physical.pressed
                        & (kirby_assist_core::native_input::MAPPER_DPAD_BUTTONS
                            | kirby_assist_core::native_input::MAPPER_X
                            | MAPPER_Y)
                        != 0
                        && state
                            .settings_menu
                            .as_ref()
                            .is_some_and(|m| m.ledge_trump_held());
                }
                if consumed & DPAD_SIDES != 0
                    || physical.held & (DPAD_UP | MAPPER_DPAD_DOWN | MAPPER_MINUS) != 0
                {
                    // Menu takeover cancels before a native refresh
                    // could replay a cached jump/dodge request.
                }
                if state.ledge_active != ledge_held {
                    if !ledge_held {
                        state.action_owner.release(Owner::Ledge);
                    }
                    #[cfg(feature = "diagnostic-logging")]
                    state.grab_trace.reset("autopilot_released_or_mode_changed");
                    #[cfg(feature = "diagnostic-logging")]
                    state.calibration.reset("autopilot released/mode changed");
                    state.ledge_active = ledge_held;
                    state.ledge_walk_motion = None;
                    state.autopilot_guarding = false;
                    state.guard_hold = None;
                    state.ledge = None;
                    state.ledge_position = None;
                    state.ledge_seek_tick = None;
                    state.ledge_output = ControllerOutput::default();
                }
                if state.ledge_active && state.ledge.is_none() {
                    if let Some(slot) = state.settings_menu.as_ref().and_then(|m| m.preview_slot())
                    {
                        let mut preview = LedgeTrap::default();
                        preview.request_preview(slot);
                        state.ledge = Some(preview);
                    }
                }
            }
        }
    }
}
/// Refreshes the selected fighter for the specified controller index, observing the fighter if valid or clearing the state if not.
unsafe fn refresh_selected_fighter(index: i32) {
    let candidate = STATE.lock().ok().and_then(|state| {
        (state.mapping_index == Some(index))
            .then_some(state.human_routes[index as usize])
            .flatten()
            .filter(|(_, seen)| seen.elapsed() < SESSION_OBSERVATION_GAP)
            .map(|(id, _)| id)
    });
    let Some(id) = candidate else {
        return;
    };
    let object = if game::is_active(id) {
        game::module_accessor(id)
    } else {
        std::ptr::null_mut()
    };
    let valid_route = !object.is_null()
        && game::battle_object_id(object) == id
        && controller_index(object.cast()) == Some(index)
        && human_fighter(object);
    if valid_route {
        observe_fighter(object, false);
    } else if let Ok(mut state) = STATE.lock() {
        if state.mapping_index == Some(index) {
            LOCAL_PARTICIPANT.store(false, Ordering::Release);
            attack_sources::clear();
            state.cancel();
            state.associated_object = None;
            state.human_routes[index as usize] = None;
        }
    }
}

#[cfg(feature = "diagnostic-logging")]
type GrabMappingComparison = (
    bool,
    Option<Owner>,
    u32,
    kirby_assist_core::native_input::ButtonTrace,
    kirby_assist_core::native_input::ButtonTrace,
    [f32; 2],
    [f32; 2],
    [f32; 2],
    [f32; 2],
);
/// Represents the preparation state for the mapper, 
/// including restored controller state, mapped inputs, and diagnostic information.
struct MapperPreparation {
    restore: Option<ControllerRestore>,
    mapped_left_stick: Option<[f32; 2]>,
    mapped_automatic_request: Option<u32>,
    mapped_auto_grab: Option<(i64, u32, bool)>,
    mapped_reflect: bool,
    mapped_reflector: Option<i64>,
    mapped_auto_up_special: Option<(i64, u32, Owner)>,
    mapped_auto_dodge: Option<(i64, u32)>,
    mapped_dodge_punish_grab: Option<(i64, u32)>,
    mapped_attack_hold: bool,
    mapped_literal_request: Option<u32>,
    mapped_unshielded_auto_grab: bool,
    mapped_sequence_right: Option<[f32; 2]>,
    reserved_dpad: u32,
    tech_applied: bool,
    mapped_tech_output: Option<TechOutput>,
    mapped_grab_handoff: Option<(i64, Option<Owner>, bool, bool)>,
    sequence_short_hop: bool,
    mapped_manual_jump: bool,
    #[cfg(feature = "diagnostic-logging")]
    trump_pressed: bool,
    #[cfg(feature = "diagnostic-logging")]
    ledge_edge: Option<u32>,
    #[cfg(feature = "diagnostic-logging")]
    grab_delivery: Option<(u32, u32, u32, &'static str)>,
    #[cfg(feature = "diagnostic-logging")]
    grab_mapping_comparison: Option<GrabMappingComparison>,
    #[cfg(feature = "diagnostic-logging")]
    messages: Vec<String>,
    #[cfg(feature = "diagnostic-logging")]
    mode_report: Option<kirby_assist_core::diagnostics::modes::Report>,
}

/// Represents the input flags for the mapper, indicating the current conditions of the controller and game state.
/// # Fields
/// * `fresh` - Indicates if the input is fresh.
/// * `manual_scope` - Indicates if manual scope is active.
/// * `mash_active` - Indicates if mash input is active.
/// * `damage_active` - Indicates if damage input is active.
/// * `turbo_active` - Indicates if turbo input is active.
/// * `pivot_active` - Indicates if pivot input is active.
/// * `posture_active` - Indicates if posture input is active.
struct MapperIntentInputs {
    fresh: bool,
    manual_scope: bool,
    mash_active: bool,
    damage_active: bool,
    turbo_active: bool,
    pivot_active: bool,
    posture_active: bool,
}
/// Computes the action intents for the mapper based on the current state and input flags. Returns an array of `Intent` for each possible action.
/// # Arguments
/// * `state` - The current state of the mapper.
/// * `inputs` - The input flags indicating the current conditions.
/// 
/// # Returns
/// An array of `Intent` for each possible action.
fn mapper_action_intents(state: &mut State, inputs: MapperIntentInputs) -> [Intent; 13] {
    let ledge_priority = state.action_owner.ledge_priority()
        && state.ledge_active
        && inputs.fresh
        && inputs.manual_scope
        && state
            .ledge
            .as_ref()
            .is_some_and(|trap| trap.needs_response_observation() || state.ledge_output.owned != 0);
    state.action_owner.set_ledge_priority(ledge_priority);
    let ledge_committed = state
        .ledge
        .as_ref()
        .is_some_and(LedgeTrap::committed_input_pending)
        || state.ledge_output.owned != 0 && !state.autopilot_guarding;
    let intents = [
        Intent {
            owner: Owner::Reflect,
            present: state.reflect_output.owned != 0 || state.reflector.active(),
            committed: true,
        },
        Intent {
            owner: Owner::OutOfShield,
            // Preserve the automation reservation while R may be
            // queued; a neutral output still leaves movement manual.
            present: state.oos_output.owned != 0
                || state.oos.grab_delivery_pending()
                || state.oos.awaiting_up_b(),
            committed: true,
        },
        Intent {
            owner: Owner::Safety,
            present: state.tech_output.press
                || state.tech_output.owns_guard
                || inputs.mash_active
                || inputs.damage_active,
            committed: true,
        },
        Intent {
            owner: Owner::Trip,
            present: state.trip_output.owned != 0,
            committed: true,
        },
        Intent {
            owner: Owner::Fair,
            present: state.fair_output.owned != 0,
            committed: true,
        },
        Intent {
            owner: Owner::Trump,
            present: state.trump_output.owned != 0,
            committed: true,
        },
        Intent {
            owner: Owner::Ledge,
            present: state.ledge_active
                && (state.action_owner.current_owner() == Some(Owner::Ledge)
                    || !state.autopilot_guarding
                // An idle always-on watch must not swallow
                // Turbo movement. Acquired ledge visits/tapes keep priority.
                && (!inputs.turbo_active
                    || ledge_priority || ledge_committed)
                && (state.oos_output.owned == 0 || ledge_priority || ledge_committed))
                && (state.ledge_output.owned != 0
                    || state.ledge_output.stick.is_some()
                    || state
                        .ledge
                        .as_ref()
                        .is_none_or(|trap| !trap.manual_after_response())),
            committed: ledge_committed,
        },
        Intent::sequence_receipt(
            Owner::RollRead,
            state.roll_read_output.owned,
            state
                .rule_engine
                .as_ref()
                .is_some_and(|e| e.awaiting_up_b()),
        ),
        Intent {
            owner: Owner::GrabDodge,
            present: state.spot_output.owned != 0
                && state
                    .spot_dodge
                    .as_ref()
                    .is_some_and(|s| s.target().is_some()),
            committed: true,
        },
        Intent {
            owner: Owner::OptionRead,
            present: (state.spot_output.owned != 0
                || state
                    .spot_dodge
                    .as_ref()
                    .is_some_and(|s| s.punish_pending()))
                && state
                    .spot_dodge
                    .as_ref()
                    .is_none_or(|s| s.target().is_none()),
            committed: true,
        },
        Intent {
            owner: Owner::Defense,
            present: state.autopilot_guarding && state.ledge_output.owned != 0,
            committed: false,
        },
        Intent {
            owner: Owner::Turbo,
            present: inputs.turbo_active || inputs.pivot_active,
            committed: false,
        },
        Intent {
            owner: Owner::Posture,
            present: inputs.posture_active,
            committed: false,
        },
    ];
    intents
}
/// Selects the appropriate controller output based on the winning owner, posture, and current situation, applying grab handoff and out-of-shield adjustments as needed.
fn selected_owner_output(
    state: &mut State,
    winner: Option<Owner>,
    posture: ControllerOutput,
    situation: i32,
) -> ControllerOutput {
    let mut selected = match winner {
        Some(Owner::Reflect) => state.reflect_output,
        Some(Owner::Trip) => state.trip_output,
        Some(Owner::Fair) => state.fair_output,
        Some(Owner::Trump) => state.trump_output,
        Some(Owner::OutOfShield) => state.oos_output,
        Some(Owner::Posture) => posture,
        Some(Owner::Ledge) => state.ledge_output,
        Some(Owner::RollRead) => state.roll_read_output,
        Some(Owner::GrabDodge | Owner::OptionRead) => state.spot_output,
        Some(Owner::Defense) => state.ledge_output,
        _ => ControllerOutput::default(),
    };
    if winner == Some(Owner::OutOfShield) {
        let movement = state.action_owner.policy_inputs().stick();
        selected = state.oos.with_grab_movement(selected, movement);
    }
    if situation == kirby_assist_core::native_states::situation::GROUND {
        selected = kirby_assist_core::grab_handoff::GrabHandoff::grab_packet(selected);
    }
    selected
}

/// Determines the appropriate right stick input based on the winning owner, selected controller output, and current situation.
fn selected_owner_right_stick(
    state: &State,
    winner: Option<Owner>,
    selected: ControllerOutput,
    situation: i32,
) -> Option<[f32; 2]> {
    if situation == kirby_assist_core::native_states::situation::GROUND
        && selected.requested & kirby_assist_core::native_input::MAPPER_R != 0
    {
        return Some([0.; 2]);
    }
    match winner {
        Some(Owner::Ledge) => state
            .ledge
            .as_ref()
            .and_then(LedgeTrap::requested_right_stick),
        Some(Owner::RollRead) => state
            .rule_engine
            .as_ref()
            .and_then(|rule| rule.right_stick()),
        Some(Owner::OutOfShield) => state.oos.right_stick(),
        Some(Owner::OptionRead) => state
            .spot_dodge
            .as_ref()
            .and_then(|dodge| dodge.right_stick()),
        _ => None,
    }
}

/// Represents a sampled posture, including configured C-stick inputs, controller output, and tilt aiming status.
struct PostureSample {
    configured_c_stick: Option<([f32; 2], [f32; 2])>,
    output: ControllerOutput,
    tilt_aiming: bool,
}

fn sample_mapper_posture(
    state: &mut State,
    manual_context: Option<ManualContext>,
    manual_scope: bool,
    fresh: bool,
    policy_buttons: u32,
    physical: kirby_assist_core::native_input::ButtonTrace,
    physical_stick: [f32; 2],
    physical_right_stick: [f32; 2],
) -> PostureSample {
    let (idle_crouch, tilt_turn) = state
        .settings_menu
        .as_ref()
        .map_or((false, false), |m| m.posture_modes());
    let tilt_settings = state.fighter_kind.map_or(Default::default(), |kind| {
        state
            .settings_menu
            .as_ref()
            .map_or(Default::default(), |menu| {
                menu.fighter_tuning(kind).tilt_turn(kind)
            })
    });
    let posture_target = state.posture_target;
    let posture_floor_safe = state.posture_floor_safe;
    let configured_c_stick = manual_context.and_then(|context| {
        (manual_scope && fresh && tilt_turn)
            .then(|| {
                kirby_assist_core::posture::Posture::configured_c_stick(
                    tilt_settings,
                    policy_buttons,
                    physical_stick,
                    physical_right_stick,
                    posture_target,
                    context,
                )
            })
            .flatten()
    });
    let posture = manual_context.map_or_else(ControllerOutput::default, |context| {
        state
            .posture
            .get_or_insert_with(Default::default)
            .sample_configured(
                manual_scope && fresh,
                idle_crouch
                    && posture_floor_safe
                    // Yield immediately on action edges and
                    // the release poll; do not add Down to
                    // an input returning from a grab/shield.
                    && kirby_assist_core::posture::Posture::idle_buttons_clear(physical),
                tilt_turn,
                tilt_settings,
                policy_buttons,
                physical_stick,
                physical_right_stick,
                posture_target,
                context,
            )
    });
    // Turnaround tilt and idle crouch stay passive at the
    // lowest priority, including physical tilt buffer gaps.
    let tilt_aiming = tilt_turn
        && posture.stick.is_some()
        && (configured_c_stick.is_some()
            || kirby_assist_core::posture::Posture::down_tilt_intent(
                policy_buttons,
                physical_stick,
                physical_right_stick,
            ));
    PostureSample {
        configured_c_stick,
        output: posture,
        tilt_aiming,
    }
}
/// Represents a sampled damage stick state, including DI, SDI, and damage stick inputs.
struct DamageStickSample {
    di_stick: Option<[f32; 2]>,
    sdi_stick: Option<[f32; 2]>,
    damage_stick: Option<[f32; 2]>,
}
/// Handles manual ledge walking, returning the appropriate stick input if conditions are met.
fn manual_ledge_walk(
    state: &State,
    settings: &AssistSettings,
    manual_scope: bool,
    fresh: bool,
    policy_buttons: u32,
    physical_stick: [f32; 2],
    physical_right_stick: [f32; 2],
    ledge: ControllerOutput,
    turbo: TurboOutput,
) -> Option<[f32; 2]> {
    if !settings.enabled
        || !manual_scope
        || policy_buttons & MAPPER_L == 0
        || !state.ledge_active
        || state
            .settings_menu
            .as_ref()
            .is_some_and(|menu| menu.preview_slot().is_some())
    {
        return None;
    }

    let walking = if fresh && settings.manual_spacing && state.ledge_walk_ready {
        let stick = ledge_walk_stick(physical_stick, settings.walk_strength);
        let spot = state.ledge_visits.walking_ledge();
        let manual_attack = policy_buttons
            & (kirby_assist_core::native_input::MAPPER_A
                | kirby_assist_core::native_input::MAPPER_B
                | settings.grab_button)
            != 0
            || physical_right_stick.iter().any(|value| value.abs() > 0.2);
        Some(match (state.ledge_walk_motion, spot) {
            (Some((position, velocity)), Some(spot)) if !manual_attack => {
                kirby_assist_core::manual_controls::stop_walk_at_ledge(
                    stick,
                    position,
                    velocity,
                    spot,
                    settings.stand_distance,
                )
            }
            _ => stick,
        })
    } else {
        None
    };
    if ledge.stick.is_none() && turbo.stick.is_none() {
        walking
    } else {
        None
    }
}
/// Samples the damage stick inputs for the given state and configuration, handling DI, SDI, and damage trajectory considerations.
fn sample_damage_sticks(
    state: &mut State,
    config: &Config,
    manual_scope: bool,
    fresh: bool,
    mash_stick: Option<[f32; 2]>,
    physical_stick: [f32; 2],
    #[cfg(feature = "diagnostic-logging")] messages: &mut Vec<String>,
) -> DamageStickSample {
    let sdi_context = state.sdi_context;
    let sdi_enabled = config.auto_sdi && manual_scope && fresh;
    let di_context = state.di_context;
    let damage_trajectory = state.damage_trajectory;
    let di_stick = di_context.and_then(|context| {
        state
            .di
            .get_or_insert_with(Default::default)
            .sample_exclusive_with_trajectory(
                kirby_assist_core::auto_di::Settings {
                    enabled: config.auto_di.enabled && manual_scope && fresh,
                    ..config.auto_di
                },
                context,
                damage_trajectory,
            )
    });
    if !config.auto_di.enabled || di_context.is_none() || !manual_scope || !fresh {
        state.di = None;
    }
    #[cfg(feature = "diagnostic-logging")]
    {
        let decision = di_stick.and_then(|_| state.di.as_ref().and_then(|di| di.decision()));
        if state.last_di_diagnostic != decision {
            if let Some(d) = decision {
                messages.push(format!("AUTO DI: {}; stick=({:.2},{:.2}); reserved launch={:?}. Final victim-hitstop input only; a heuristic, not guaranteed survival or combo escape.", d.reason.label(), d.stick[0], d.stick[1], di_context.map(|c| c.launch)));
            } else if state.last_di_diagnostic.is_some() {
                messages.push("AUTO DI released: hitstop ended or DI became unavailable.".into());
            }
            state.last_di_diagnostic = decision;
        }
    }
    // A tech deadline can precede DI; ordinary tech/release
    // yields to launch DI. Losing modes cannot emit inputs
    // or consume a flick as though it had been delivered.
    let tech_owns = state.tech_output.press || state.tech_output.owns_guard;
    let sdi_stick = if di_stick.is_none() && !tech_owns && mash_stick.is_none() {
        let previous = state.last_mapped_stick.unwrap_or(physical_stick);
        sdi_context.and_then(|context| {
            state
                .sdi
                .get_or_insert_with(SdiState::default)
                .sample_exclusive_with_previous(
                    sdi_enabled,
                    context,
                    damage_trajectory.map(|t| (t.position, t.blast)),
                    previous,
                )
        })
    } else {
        state.sdi = None;
        None
    };
    let damage_stick = di_stick.or(sdi_stick);
    if !sdi_enabled || sdi_context.is_none() {
        state.sdi = None;
    }
    DamageStickSample {
        di_stick,
        sdi_stick,
        damage_stick,
    }
}

/// Prepares the mapper overlay for the given controller index and session ticket. Returns a `MapperPreparation` if successful.
/// 
/// # Safety
/// This function is unsafe because it dereferences raw pointers for the output and wrapper. The caller must ensure that these pointers are valid and correctly aligned.
unsafe fn prepare_mapper_overlay(
    index: i32,
    output: *mut c_void,
    wrapper: *mut c_void,
    master_ready_go: Option<Option<bool>>,
    training_mode: Option<bool>,
    training_test_mode: bool,
    session_ticket: SessionTicket,
) -> Option<MapperPreparation> {
    let mut restore = None;
    let mut mapped_left_stick = None;
    let mut mapped_automatic_request = None;
    let mut mapped_auto_grab = None;
    let mut mapped_reflect = false;
    let mut mapped_reflector = None;
    let mut mapped_auto_up_special = None;
    let mut mapped_auto_dodge = None;
    let mut mapped_dodge_punish_grab = None;
    let mut mapped_attack_hold = false;
    let mut mapped_literal_request = None;
    let mut mapped_unshielded_auto_grab = false;
    let mut mapped_sequence_right = None;
    let mut reserved_dpad = 0;
    let mut tech_applied = false;
    let mut mapped_tech_output = None;
    let mut mapped_grab_handoff = None;
    let mut sequence_short_hop = false;
    let mut mapped_manual_jump = false;
    #[cfg(feature = "diagnostic-logging")]
    let mut trump_pressed = false;
    #[cfg(feature = "diagnostic-logging")]
    let mut ledge_edge = None;
    #[cfg(feature = "diagnostic-logging")]
    let mut grab_delivery = None;
    #[cfg(feature = "diagnostic-logging")]
    let mut grab_mapping_comparison = None;
    #[cfg(feature = "diagnostic-logging")]
    let mut messages = Vec::new();
    #[cfg(feature = "diagnostic-logging")]
    let mut mode_report = None;
    if let Some(config) = CONFIG.get().map(|config| **config) {
        let config = diagnostic_config(config);
        // A startup-disabled mod is a true passthrough: do not bind routes,
        // update menus, refresh fighters, run policies or touch controller data.
        if !config.controls.enabled {
            return None;
        }
        if !wrapper.is_null()
            && !output.is_null()
            && (0..kirby_assist_core::native_input::MAPPER_SLOT_COUNT).contains(&index)
        {
            // The wrapper stores a Controller pointer at this byte offset. read() loads
            // that pointer; cast() alone only changes its type. These native
            // offsets belong to the verified layout, not user-tunable settings.
            let controller = controller_from_wrapper(wrapper);
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
                // Let the classic monitor chord pass before D-pad Down can
                // start a trump. This applies to standalone builds as well as
                // Training; OS overlays remain responsible for their own focus.
                if kirby_assist_core::settings_menu::monitor_chord(
                    config.control_layout.policy_buttons(physical.held),
                ) {
                    if let Ok(mut state) = STATE.lock() {
                        state.cancel();
                        state
                            .settings_menu
                            .get_or_insert_with(Default::default)
                            .observe_passthrough(physical.held, config.control_layout);
                    }
                    return None;
                }
                let physical_stick = [(*controller).left_stick_x, (*controller).left_stick_y];
                let physical_right_stick =
                    [(*controller).right_stick_x, (*controller).right_stick_y];
                let physical_autorepeat = (*controller).autorepeat[0];
                let physical_triggers = [(*controller).left_trigger, (*controller).right_trigger];
                // Only the selected controller checks the existing battle
                // manager guard. Menu lifetime is independent of request TTL.
                let menu_ready_go =
                    master_ready_go.unwrap_or_else(|| observations::battle_menu_ready_go());
                // Without Training Modpack, query only the selected controller;
                // with it, reuse the earlier shortcut-policy scene read.
                let menu_training_mode = training_mode.unwrap_or_else(|| game::is_training_mode());
                update_mapper_menu_and_input(
                    controller,
                    index,
                    &config,
                    physical,
                    physical_stick,
                    physical_right_stick,
                    menu_ready_go,
                    menu_training_mode,
                    training_test_mode,
                    #[cfg(feature = "diagnostic-logging")]
                    &mut trump_pressed,
                    #[cfg(feature = "diagnostic-logging")]
                    &mut messages,
                );
                // Restore 1.0.4's immediate pre-map refresh without its roster
                // scan: only resolve a recently observed human fighter ID.
                // Revalidate identity on every refresh; never retain a pointer.
                refresh_selected_fighter(index);
                if let Ok(mut state) = STATE.lock() {
                    if state.session.accepts(session_ticket)
                        && state.mapping_index == Some(index)
                        && (*controller).connected != 0
                    {
                        // Recreate the canonical policy view in this lock scope.
                        // The earlier copy configures observation/menu state, but
                        // is intentionally scoped to that mapper-state update.
                        let policy_buttons = if training_test_mode {
                            training_test::policy_buttons(physical.held)
                        } else {
                            physical.held
                        };
                        let policy_buttons = config.control_layout.policy_buttons(policy_buttons);
                        let policy_buttons = state
                            .settings_menu
                            .as_ref()
                            .map_or(policy_buttons, |m| m.native_buttons);
                        let now = Instant::now();
                        let fresh = kirby_assist_core::observation::input_update_current(
                            state.evaluated_tick,
                            state.raw_tick,
                        ) && state
                            .request_seen
                            .is_some_and(|seen| now.duration_since(seen) < REQUEST_LIFETIME);
                        let ledge_fresh = fresh;
                        let route_current =
                            state.human_routes[index as usize].is_some_and(|(id, seen)| {
                                Some(id) == state.associated_object
                                    && seen.elapsed() < SESSION_OBSERVATION_GAP
                            });
                        let ledge_config = state.settings_menu.as_ref().map_or_else(
                            || settings_for_fighter(state.fighter_kind, config.assist),
                            |p| p.settings(state.fighter_kind, &config.assist),
                        );
                        let manual_scope = config.controls.enabled
                            && route_current
                            && state.associated_object.is_some()
                            && state.associated_object == state.observed_object
                            && physical.held & MAPPER_PLUS == 0;
                        #[cfg(feature = "diagnostic-logging")]
                        {
                            let expired = manual_scope
                                && state.menu_battle_active
                                && !fresh
                                && state.request_seen.is_some();
                            if expired && !state.last_input_expired {
                                crate::performance::health::event(
                                    crate::performance::health::Event::InputExpired,
                                );
                            }
                            state.last_input_expired = expired;
                        }
                        // Expired requests become neutral. The overlay still
                        // knows the previous assisted output, allowing a release.
                        let native_grab = state.manual_context.is_some_and(|c| {
                            kirby_assist_core::action_owner::native_grab_active(c.status)
                        });
                        let auto_mash = state
                            .settings_menu
                            .as_ref()
                            .map_or(config.controls.auto_mash, |menu| {
                                menu.auto_mash(config.controls.auto_mash)
                            });
                        let mash_stick = if fresh && auto_mash {
                            state.mash_stick
                        } else {
                            None
                        };

                        // Compose all native requests through ONE button history.
                        // Independent overlays can disagree about a release when
                        // tech, mash and ledge ownership switch on the same frame.
                        // Read the trigger from PHYSICAL buttons, then consume
                        // its taunt bits. Releasing returns the fresh physical
                        // stick immediately, rather than forcing neutral.
                        let tick = state.policy_tick.unwrap_or(state.raw_tick);
                        let can_move = fresh && state.policy_tick.is_some() && state.turbo_ready;
                        // The selected button binding alone activates Turbo t-bag.
                        // L3 is not remapped to the L ledge-assist role.
                        let turbo_held = policy_buttons & TURBO_TRIGGER != 0;
                        let turbo_facing = state.manual_context.map_or(1.0, |ctx| ctx.facing);
                        let turbo_input_tick = state.raw_tick;
                        let mut turbo_output = state
                            .turbo
                            .get_or_insert_with(TurboState::default)
                            .sample_teabag(
                                config.turbo,
                                // Generate the next local input packet now;
                                // never wait for delayed crouch acknowledgment.
                                turbo_input_tick,
                                turbo_held,
                                turbo_facing,
                                manual_scope,
                                can_move,
                            );
                        // Bound directions are consumed by the menu adapter. Normal
                        // directions pass through; no hard-coded B-reverse/trump.
                        // L holds ledge trap; held Y runs its saved mode.
                        // Changing tools releases the previous policy.
                        // Stale observations still release automation ownership.
                        let turbo_stick = turbo_output.stick;
                        let pivot_context = state.manual_context.filter(|_| {
                            fresh
                                && manual_scope
                                && state.policy_tick.is_some()
                                && turbo_stick.is_none()
                                && physical.held
                                    & !kirby_assist_core::manual_controls::LEFT_STICK_DIRECTIONS
                                    == 0
                                && physical_right_stick
                                    .iter()
                                    .all(|v| v.is_finite() && v.abs() <= 0.1)
                        });
                        if pivot_context.is_none() {
                            state.dash_pivot = None;
                        }
                        let pivot_stick = pivot_context.map_or(None, |context| {
                            state
                                .dash_pivot
                                .get_or_insert_with(Default::default)
                                .sample(
                                    tick,
                                    context.status,
                                    context.situation,
                                    context.stopped,
                                    physical_stick,
                                )
                                .stick
                        });
                        let mut ledge = if ledge_fresh && state.ledge_active {
                            state.ledge_output
                        } else {
                            ControllerOutput::default()
                        };
                        let manual_walk_stick = manual_ledge_walk(
                            &state,
                            &ledge_config,
                            manual_scope,
                            fresh,
                            policy_buttons,
                            physical_stick,
                            physical_right_stick,
                            ledge,
                            turbo_output,
                        );
                        #[cfg(feature = "diagnostic-logging")]
                        if trump_pressed {
                            messages.push(format!(
                                "trump input: scope={}, fresh={}, action={}, stick={:?}, reserved={reserved_dpad:#x}",
                                manual_scope, fresh,
                                state.trump.as_ref().map_or("unobserved", LedgeTrump::action_name),
                                state.trump_output.stick,
                            ));
                        }
                        let manual_context = state.manual_context;
                        let PostureSample {
                            configured_c_stick,
                            output: posture,
                            tilt_aiming,
                        } = sample_mapper_posture(
                            &mut state,
                            manual_context,
                            manual_scope,
                            fresh,
                            policy_buttons,
                            physical,
                            physical_stick,
                            physical_right_stick,
                        );
                        // Menu input is reserved during entry/respawn too.
                        let menu_consumed =
                            if state.menu_battle_active && state.menu_fighter_kind.is_some() {
                                state.settings_menu.as_ref().map_or(0, |menu| menu.consumed)
                            } else {
                                0
                            };
                        reserved_dpad |=
                            menu_consumed & kirby_assist_core::native_input::MAPPER_DPAD_BUTTONS;
                        let DamageStickSample {
                            di_stick,
                            sdi_stick,
                            damage_stick,
                        } = sample_damage_sticks(
                            &mut state,
                            &config,
                            manual_scope,
                            fresh,
                            mash_stick,
                            physical_stick,
                            #[cfg(feature = "diagnostic-logging")]
                            &mut messages,
                        );
                        // Single final authority for sticks/buttons. Preparation
                        // keeps ledge/roll timing ahead of every optional tool.
                        // A detected ledge visit reserves its response against
                        // general reads. Before acquisition they run normally.
                        let mut intents = mapper_action_intents(
                            &mut state,
                            MapperIntentInputs {
                                fresh,
                                manual_scope,
                                mash_active: mash_stick.is_some(),
                                damage_active: damage_stick.is_some(),
                                turbo_active: turbo_stick.is_some(),
                                pivot_active: pivot_stick.is_some(),
                                posture_active: posture.stick.is_some(),
                            },
                        );
                        let handoff_status = manual_context.map_or(-1, |c| c.status);
                        let handoff_situation = manual_context.map_or(
                            kirby_assist_core::native_states::situation::UNAVAILABLE,
                            |c| c.situation,
                        );
                        let handoff_context =
                            manual_context.filter(|_| fresh).map(|c| GameplayContext {
                                gameplay: manual_scope,
                                status: c.status,
                                situation: c.situation,
                                motion_kind: c.motion,
                                motion_frame: c.frame,
                                motion_rate: c.rate,
                                stopped: c.stopped,
                                ..GameplayContext::default()
                            });
                        state
                            .grab_handoff
                            .observe_native(manual_scope, handoff_context);
                        state.grab_handoff.restrict(&mut intents);
                        let mut winner = state
                            .action_owner
                            .select(fresh && manual_scope && !native_grab, &intents);
                        sequence_short_hop = winner == Some(Owner::Ledge)
                            && state.ledge_output.native_short_hop
                            || winner == Some(Owner::OutOfShield)
                                && state.oos_output.native_short_hop
                            || winner == Some(Owner::OptionRead)
                                && state.spot_output.native_short_hop;
                        let mut selected =
                            selected_owner_output(&mut state, winner, posture, handoff_situation);
                        let selected_right =
                            selected_owner_right_stick(&state, winner, selected, handoff_situation);
                        if !state.grab_handoff.permits(winner, selected, selected_right) {
                            // Stop the old script, not just this packet; otherwise its
                            // delayed B/Up step could resume after receipt expiry.
                            if let Some(owner) = winner {
                                state.action_owner.release(owner);
                            }
                            winner = None;
                            selected = ControllerOutput::default();
                        }
                        if winner == Some(Owner::Safety) {
                            (selected, mapped_tech_output) =
                                kirby_assist_core::action_owner::safety_output_with_deadline(
                                    state.tech_output,
                                    mash_stick,
                                    di_stick,
                                    sdi_stick,
                                    state.tech_urgent,
                                );
                        }
                        selected = kirby_assist_core::action_owner::exclusive_output(
                            winner,
                            state.action_owner.current_owner() == winner && winner.is_some(),
                            selected,
                        );
                        let tech_takeover =
                            winner == Some(Owner::Safety) && mapped_tech_output.is_some();
                        let mash_takeover = winner == Some(Owner::Safety) && mash_stick.is_some();
                        mapped_grab_handoff = Some((
                            state.raw_tick,
                            winner,
                            selected.requested & kirby_assist_core::native_input::MAPPER_R != 0,
                            fresh
                                && manual_scope
                                && !native_grab
                                && handoff_situation
                                    == kirby_assist_core::native_states::situation::GROUND
                                && !kirby_assist_core::ledge_trap::sequence_interrupted(
                                    handoff_status,
                                ),
                        ));
                        ledge = selected;
                        turbo_output.stick = match winner {
                            Some(Owner::Safety) => selected.stick,
                            Some(Owner::Turbo) => turbo_stick.or(pivot_stick),
                            // An armed ledge wait reserves priority without an
                            // attack stick. Keep its shared physical walk cap;
                            // owned releases and active recipes must never fall through.
                            Some(Owner::Ledge)
                                if selected.owned == 0 && selected.requested == 0 =>
                            {
                                selected.stick.or(manual_walk_stick)
                            }
                            None => manual_walk_stick,
                            _ => selected.stick,
                        };
                        state.cancel_non_owner(winner);
                        #[cfg(feature = "diagnostic-logging")]
                        {
                            use kirby_assist_core::diagnostics::modes::{bit, Snapshot};
                            let status = manual_context.map_or(-1, |c| c.status);
                            // Log only a change into a real assist override, not
                            // every poll or native animation stop. No game reads.
                            let blocker = if manual_scope
                                && fresh
                                && physical_stick[0].abs() >= 0.8
                                && physical_stick[1].abs() < 0.3
                                && turbo_output.stick.is_some_and(|s| {
                                    s[0].abs() <= 0.2 || s[0] * physical_stick[0] < 0.0
                                }) {
                                winner
                            } else {
                                None
                            };
                            if blocker != state.last_movement_blocker {
                                state.last_movement_blocker = blocker;
                                if let Some(owner) = blocker {
                                    messages.push(format!("MOVEMENT OVERRIDE: owner={}, native_status={status:#x}, physical_stick={physical_stick:?}, proposed_stick={:?}, requested={:#x}, ledge_active={}",
                                        kirby_assist_core::diagnostics::modes::name(owner),turbo_output.stick,ledge.requested,state.ledge_active));
                                }
                            }
                            let snapshot = Snapshot {
                                winner,
                                present: intents
                                    .iter()
                                    .filter(|i| i.present)
                                    .fold(0, |mask, i| mask | bit(i.owner)),
                                committed: intents
                                    .iter()
                                    .any(|i| Some(i.owner) == winner && i.committed),
                                requested: ledge.requested,
                                stick: turbo_output.stick.map(|stick| {
                                    stick.map(|v| {
                                        if v > 0.08 {
                                            1
                                        } else if v < -0.08 {
                                            -1
                                        } else {
                                            0
                                        }
                                    })
                                }),
                                status,
                                motion: manual_context.map_or(0, |c| c.motion),
                                native_frame: manual_context
                                    .map_or(0, |c| c.frame.clamp(0.0, 65535.0) as u16),
                                stopped: manual_context.is_some_and(|c| c.stopped),
                                can_act: state.turbo_ready,
                                hit: state.mode_hit,
                                valid: fresh && manual_scope,
                                safety: (if state.tech_output.press || state.tech_output.owns_guard
                                {
                                    kirby_assist_core::diagnostics::modes::SAFETY_TECH_GUARD
                                } else {
                                    0
                                }) | (if mash_stick.is_some() {
                                    kirby_assist_core::diagnostics::modes::SAFETY_MASH
                                } else {
                                    0
                                }) | (if sdi_stick.is_some() && di_stick.is_none() {
                                    kirby_assist_core::diagnostics::modes::SAFETY_SDI
                                } else {
                                    0
                                }) | (if di_stick.is_some() {
                                    kirby_assist_core::diagnostics::modes::SAFETY_DI
                                } else {
                                    0
                                }),
                            };
                            // Reuse the mapper's timestamp; do not read another clock per poll.
                            let origin = *state.mode_log_origin.get_or_insert(now);
                            let ms = now
                                .saturating_duration_since(origin)
                                .as_millis()
                                .min(u128::from(u64::MAX))
                                as u64;
                            mode_report = state.mode_diagnostics.sample(snapshot, ms);
                        }

                        // A held C-stick can otherwise inject an attack while
                        // left-stick-only mashing. Block its digital aliases too.
                        let blocked_stick_buttons = if mash_takeover {
                            kirby_assist_core::manual_controls::RIGHT_STICK_DIRECTIONS
                                | kirby_assist_core::manual_controls::STICK_CLICKS
                        } else {
                            0
                        };
                        let cancel_stick = if handoff_situation
                            == kirby_assist_core::native_states::situation::GROUND
                            && selected.requested & kirby_assist_core::native_input::MAPPER_R != 0
                        {
                            Some([0.; 2])
                        } else if winner == Some(Owner::RollRead) && fresh && selected.owned != 0 {
                            state.rule_engine.as_ref().and_then(|r| r.right_stick())
                        } else if winner == Some(Owner::OutOfShield)
                            && fresh
                            && state.oos_output.owned != 0
                        {
                            state.oos.right_stick()
                        } else if winner == Some(Owner::OptionRead) && fresh && selected.owned != 0
                        {
                            state.spot_dodge.as_ref().and_then(|s| s.right_stick())
                        } else if winner == Some(Owner::Ledge) && fresh {
                            state
                                .ledge
                                .as_ref()
                                .and_then(LedgeTrap::requested_right_stick)
                        } else {
                            None
                        };
                        // All manual tools share ONE final edge history. When
                        // adding a tool, compose its requested/owned masks here
                        // and decide its stick priority explicitly; do not write
                        // another independent A/R/ZR history after this point.
                        let sequence_walking = match winner {
                            Some(Owner::Ledge) => state
                                .ledge
                                .as_ref()
                                .and_then(LedgeTrap::requested_walk_stick)
                                .is_some(),
                            Some(Owner::RollRead) => {
                                state.rule_engine.as_ref().is_some_and(|r| r.walking())
                            }
                            // A moving grab retains raw horizontal input; it is not a file Walk.
                            Some(Owner::OutOfShield) => false,
                            Some(Owner::OptionRead) => {
                                state.spot_dodge.as_ref().is_some_and(|s| s.walking())
                            }
                            _ => false,
                        };
                        let exclusive_sequence = ledge.owned
                            & kirby_assist_core::native_input::GAMEPLAY_BUTTONS
                            == kirby_assist_core::native_input::GAMEPLAY_BUTTONS;
                        if exclusive_sequence {
                            reserved_dpad |= kirby_assist_core::native_input::MAPPER_DPAD_BUTTONS;
                        }
                        let canonical_consumed =
                            kirby_assist_core::native_input::sequence_flick_consumption(
                                ledge.owned,
                                sequence_walking,
                            ) | reserved_dpad
                                | menu_consumed
                                | if !exclusive_sequence
                                    && ledge.owned
                                        & kirby_assist_core::manual_controls::LEFT_STICK_DIRECTIONS
                                        != 0
                                    && !mash_takeover
                                    && !matches!(winner, Some(Owner::Trip | Owner::Fair))
                                    && !(matches!(winner, Some(Owner::Posture)) && tilt_aiming)
                                {
                                    (1 << 21) | (1 << 23)
                                } else {
                                    0
                                }
                                | if manual_walk_stick.is_some()
                                    && turbo_output.stick == manual_walk_stick
                                    && ledge.owned == 0
                                    && !mash_takeover
                                {
                                    (1 << 21) | (1 << 23)
                                } else {
                                    0
                                };
                        let canonical_requested =
                            kirby_assist_core::native_input::selected_requests(
                                ledge.owned,
                                ledge.requested,
                                0,
                            );
                        // Deliver the native jump pulse through the same raw
                        // history as every other simulated input. The final
                        // mapped flag supplements this path; it does not replace it.
                        if exclusive_sequence {
                            mapped_automatic_request = Some(canonical_requested);
                        }
                        mapped_manual_jump = winner == Some(Owner::Ledge)
                            && canonical_requested
                                & kirby_assist_core::native_input::SEQUENCE_JUMP_BUTTONS
                                != 0;
                        let canonical_owned = ledge.owned
                            | if tech_takeover {
                                state.tech_output.owned_buttons()
                            } else {
                                0
                            }
                            | if mapped_tech_output.is_some_and(|t| t.press) {
                                MAPPER_ZR
                            } else {
                                0
                            }
                            | if cancel_stick.is_some() {
                                kirby_assist_core::manual_controls::RIGHT_STICK_DIRECTIONS
                            } else {
                                0
                            }
                            | blocked_stick_buttons;
                        // Keep source consumption above; only the final taunt
                        // filter yields to an explicitly owned sequence D-pad.
                        reserved_dpad =
                            kirby_assist_core::manual_controls::sequence_taunt_reservation(
                                reserved_dpad,
                                ledge.requested,
                                ledge.owned,
                            );
                        let approach_pulse =
                            winner == Some(Owner::OutOfShield) && state.oos.approach_pending();
                        if mapped_tech_output.is_some_and(|t| t.press) && !state.last_mapped_guard {
                            state
                                .manual_overlay
                                .get_or_insert_with(ManualOverlay::default)
                                .rearm_tech_guard(
                                    config.control_layout.physical_buttons(MAPPER_ZR),
                                    physical,
                                );
                        }
                        let manual = state
                            .manual_overlay
                            .get_or_insert_with(ManualOverlay::default)
                            .apply_reaction_controls(
                                physical,
                                physical_stick,
                                turbo_output,
                                config.control_layout.physical_buttons(canonical_consumed),
                                config.control_layout.physical_buttons(canonical_requested),
                                config
                                    .control_layout
                                    .physical_owned_buttons(canonical_owned),
                                winner == Some(Owner::RollRead) || approach_pulse,
                            );
                        if fresh && manual_scope && sequence_walking {
                            mapped_left_stick = ledge.stick;
                        }
                        // Built-in analog outputs must survive the native
                        // mapper's flick/history filtering just like file Walk.
                        if fresh
                            && manual_scope
                            && (winner == Some(Owner::Turbo) && turbo_stick.is_some()
                                || (winner == Some(Owner::Ledge) || winner.is_none())
                                    && selected.owned == 0
                                    && manual_walk_stick.is_some()
                                    && turbo_output.stick == manual_walk_stick)
                        {
                            mapped_left_stick = turbo_output.stick;
                        }
                        // Preserve the selected literal A packet, including directional
                        // smashes and one-frame presses. Plan metadata used to retain
                        // only neutral multi-frame holds, losing A while the stick survived.
                        mapped_attack_hold = fresh
                            && manual_scope
                            && matches!(
                                winner,
                                Some(
                                    Owner::Ledge
                                        | Owner::RollRead
                                        | Owner::OutOfShield
                                        | Owner::OptionRead
                                )
                            )
                            && selected.owned & kirby_assist_core::native_input::MAPPER_A != 0
                            && selected.requested & kirby_assist_core::native_input::MAPPER_A != 0
                            && manual.buttons.held
                                & config
                                    .control_layout
                                    .physical_buttons(kirby_assist_core::native_input::MAPPER_A)
                                != 0;
                        // Forward all exclusive sequence axes identically. Drift
                        // and Run must not depend on the native mapper retaining
                        // an analog value after physical flick history was masked.
                        if let Some(stick) =
                            kirby_assist_core::native_input::exclusive_sequence_stick(
                                ledge.owned,
                                ledge.stick,
                            )
                        {
                            mapped_left_stick = Some(stick);
                        }
                        if exclusive_sequence {
                            mapped_sequence_right = Some(cancel_stick.unwrap_or([0.; 2]));
                        }
                        if fresh && manual_scope && exclusive_sequence && winner.is_some() {
                            mapped_literal_request = Some(
                                [
                                    kirby_assist_core::native_input::MAPPER_A,
                                    kirby_assist_core::native_input::MAPPER_B,
                                    kirby_assist_core::native_input::MAPPER_R,
                                    kirby_assist_core::native_input::MAPPER_ZR,
                                ]
                                .into_iter()
                                .fold(0, |mask, button| {
                                    mask | if canonical_requested & selected.owned & button != 0
                                        && manual.buttons.held
                                            & config.control_layout.physical_buttons(button)
                                            != 0
                                    {
                                        button
                                    } else {
                                        0
                                    }
                                }),
                            );
                            mapped_unshielded_auto_grab = winner == Some(Owner::OutOfShield)
                                && state.oos.unshielded_grab_started();
                        }
                        let posture_side = if winner == Some(Owner::Posture) {
                            configured_c_stick
                                .and_then(|(_, right)| (right[1] == 0.0).then_some(right[0]))
                        } else {
                            None
                        };
                        let modified = if winner.is_none() || winner == Some(Owner::Posture) {
                            state
                                .posture_side_aliases
                                .apply(manual.buttons, posture_side)
                        } else {
                            state.posture_side_aliases = Default::default();
                            manual.buttons
                        };
                        if winner == Some(Owner::Reflect) {
                            mapped_reflector = Some(state.raw_tick);
                        }
                        mapped_reflect = winner == Some(Owner::Reflect)
                            && selected.requested & kirby_assist_core::native_input::MAPPER_B != 0
                            && modified.held
                                & config
                                    .control_layout
                                    .physical_buttons(kirby_assist_core::native_input::MAPPER_B)
                                != 0;
                        if winner == Some(Owner::OutOfShield)
                            && selected.requested & kirby_assist_core::native_input::MAPPER_R != 0
                            && modified.held
                                & config
                                    .control_layout
                                    .physical_buttons(kirby_assist_core::native_input::MAPPER_R)
                                != 0
                        {
                            mapped_auto_grab = state
                                .oos
                                .target()
                                .map(|target| (state.raw_tick, target, true));
                        } else if approach_pulse
                            && selected.stick.is_some_and(|s| s[0].abs() >= 0.95)
                        {
                            mapped_auto_grab = state
                                .oos
                                .target()
                                .map(|target| (state.raw_tick, target, false));
                        }
                        if winner == Some(Owner::OptionRead)
                            && selected.requested & kirby_assist_core::native_input::MAPPER_R != 0
                            && modified.held
                                & config
                                    .control_layout
                                    .physical_buttons(kirby_assist_core::native_input::MAPPER_R)
                                != 0
                        {
                            mapped_dodge_punish_grab = state
                                .spot_dodge
                                .as_ref()
                                .and_then(|s| s.follow_target())
                                .map(|id| (state.raw_tick, id));
                        }
                        if winner == Some(Owner::GrabDodge)
                            && selected.requested & MAPPER_ZR != 0
                            && selected.stick.is_some_and(|s| s[1] < -0.5)
                        {
                            mapped_auto_dodge = state
                                .spot_dodge
                                .as_ref()
                                .and_then(|s| s.target())
                                .map(|target| (state.raw_tick, target));
                        }
                        if winner == Some(Owner::OutOfShield)
                            && state.oos.shield_up_b_pending()
                            && selected.requested & kirby_assist_core::native_input::MAPPER_B != 0
                            && modified.held
                                & config
                                    .control_layout
                                    .physical_buttons(kirby_assist_core::native_input::MAPPER_B)
                                != 0
                        {
                            mapped_auto_up_special = state
                                .oos
                                .target()
                                .map(|target| (state.raw_tick, target, Owner::OutOfShield));
                        } else if selected.requested & kirby_assist_core::native_input::MAPPER_B
                            != 0
                            && modified.held
                                & config
                                    .control_layout
                                    .physical_buttons(kirby_assist_core::native_input::MAPPER_B)
                                != 0
                        {
                            let target = match winner {
                                Some(Owner::RollRead) => state
                                    .rule_engine
                                    .as_ref()
                                    .filter(|e| e.direct_up_b())
                                    .and_then(|e| e.target()),
                                Some(Owner::OptionRead) => state
                                    .spot_dodge
                                    .as_ref()
                                    .filter(|s| s.direct_up_b())
                                    .and_then(|s| s.follow_target()),
                                _ => None,
                            };
                            mapped_auto_up_special = target
                                .zip(winner)
                                .map(|(target, owner)| (state.raw_tick, target, owner));
                        }
                        #[cfg(feature = "diagnostic-logging")]
                        if state.grab_trace.active() || state.calibration.active() {
                            grab_delivery = Some((
                                state.ledge_output.requested,
                                modified.held,
                                modified.pressed,
                                if !ledge_fresh {
                                    "expired_observation"
                                } else if !manual_scope {
                                    "outside_manual_scope"
                                } else if mash_takeover {
                                    "mash_priority"
                                } else if state.tech_output.owns_guard {
                                    "tech_priority"
                                } else if winner == Some(Owner::Trip) {
                                    "trip_priority"
                                } else if winner == Some(Owner::Fair) {
                                    "fair_priority"
                                } else if state.trump_output.owned != 0 {
                                    "trump_priority"
                                } else {
                                    "ledge"
                                },
                            ));
                        }
                        tech_applied = mapped_tech_output.is_some_and(|t| t.press);
                        #[cfg(feature = "diagnostic-logging")]
                        if ledge.requested & modified.pressed != 0 {
                            ledge_edge = Some(ledge.requested);
                        }
                        #[cfg(feature = "diagnostic-logging")]
                        {
                            let grab_button = config
                                .control_layout
                                .physical_buttons(kirby_assist_core::native_input::MAPPER_R);
                            let automatic = winner == Some(Owner::OutOfShield)
                                && selected.requested & kirby_assist_core::native_input::MAPPER_R
                                    != 0
                                && modified.held & grab_button != 0;
                            let manual_grab = physical.held & grab_button != 0
                                && selected.requested & kirby_assist_core::native_input::MAPPER_R
                                    == 0;
                            if automatic || manual_grab {
                                let (counter, pressed) = if automatic {
                                    (&AUTOMATIC_GRAB_COMPARISON_POLLS, modified.pressed)
                                } else {
                                    (&MANUAL_GRAB_COMPARISON_POLLS, physical.pressed)
                                };
                                if pressed & grab_button != 0 {
                                    counter.store(0, Ordering::Relaxed);
                                }
                                if counter
                                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |count| {
                                        (count < 8).then_some(count + 1)
                                    })
                                    .is_ok()
                                {
                                    grab_mapping_comparison = Some((
                                        automatic,
                                        winner,
                                        selected.requested,
                                        physical,
                                        modified,
                                        physical_stick,
                                        manual.stick,
                                        physical_triggers,
                                        kirby_assist_core::native_input::owned_trigger_axes(
                                            physical_triggers,
                                            manual.clear_autorepeat,
                                        ),
                                    ));
                                }
                            }
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
                            && mapped_tech_output.is_some_and(|t| t.press)
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
                        let triggers = kirby_assist_core::native_input::owned_trigger_axes(
                            physical_triggers,
                            manual.clear_autorepeat,
                        );
                        (*controller).left_trigger = triggers[0];
                        (*controller).right_trigger = triggers[1];
                        // Action-owning sequences suppress competing C-stick
                        // attacks. The facing helper only angles an existing
                        // physical down C-stick; restore both sticks afterward.
                        if kirby_assist_core::move_input::blocks_physical_c_stick(ledge.owned)
                            || mash_takeover
                        {
                            (*controller).right_stick_x = 0.0;
                            (*controller).right_stick_y = 0.0;
                        }
                        if winner == Some(Owner::Posture) {
                            if let Some(stick) =
                                configured_c_stick.map(|(_, right)| right).or_else(|| {
                                    kirby_assist_core::posture::Posture::aimed_c_stick(
                                        policy_buttons,
                                        physical_right_stick,
                                        posture.stick,
                                    )
                                })
                            {
                                (*controller).right_stick_x = stick[0];
                                (*controller).right_stick_y = stick[1];
                            }
                        }
                        if let Some(stick) = cancel_stick {
                            (*controller).right_stick_x = stick[0];
                            (*controller).right_stick_y = stick[1];
                        }
                        // Repeat flags can otherwise leak a held side-taunt
                        // even after held/pressed/released words were consumed.
                        (*controller).autorepeat[0] &= !manual.clear_autorepeat;
                        restore = Some(ControllerRestore {
                            controller,
                            physical,
                            stick: physical_stick,
                            right_stick: physical_right_stick,
                            autorepeat: physical_autorepeat,
                            triggers: physical_triggers,
                        });
                    }
                }
            }
        }
    }
    Some(MapperPreparation {
        restore,
        mapped_left_stick,
        mapped_automatic_request,
        mapped_auto_grab,
        mapped_reflect,
        mapped_reflector,
        mapped_auto_up_special,
        mapped_auto_dodge,
        mapped_dodge_punish_grab,
        mapped_attack_hold,
        mapped_literal_request,
        mapped_unshielded_auto_grab,
        mapped_sequence_right,
        reserved_dpad,
        tech_applied,
        mapped_tech_output,
        mapped_grab_handoff,
        sequence_short_hop,
        mapped_manual_jump,
        #[cfg(feature = "diagnostic-logging")]
        trump_pressed,
        #[cfg(feature = "diagnostic-logging")]
        ledge_edge,
        #[cfg(feature = "diagnostic-logging")]
        grab_delivery,
        #[cfg(feature = "diagnostic-logging")]
        grab_mapping_comparison,
        #[cfg(feature = "diagnostic-logging")]
        messages,
        #[cfg(feature = "diagnostic-logging")]
        mode_report,
    })
}
/// Finalizes the mapper overlay for the given output and preparation state, applying adjustments and recording delivery feedback.
unsafe fn finish_mapper_overlay(
    output: *mut c_void,
    index: i32,
    session_ticket: SessionTicket,
    preparation: MapperPreparation,
) {
    let MapperPreparation {
        restore,
        mapped_left_stick,
        mapped_automatic_request,
        mapped_auto_grab,
        mapped_reflect,
        mapped_reflector,
        mapped_auto_up_special,
        mapped_auto_dodge,
        mapped_dodge_punish_grab,
        mapped_attack_hold,
        mapped_literal_request,
        mapped_unshielded_auto_grab,
        mapped_sequence_right,
        reserved_dpad,
        tech_applied,
        mapped_tech_output,
        mapped_grab_handoff,
        sequence_short_hop,
        mapped_manual_jump,
        #[cfg(feature = "diagnostic-logging")]
        trump_pressed,
        #[cfg(feature = "diagnostic-logging")]
        ledge_edge,
        #[cfg(feature = "diagnostic-logging")]
        grab_delivery,
        #[cfg(feature = "diagnostic-logging")]
        grab_mapping_comparison,
        #[cfg(feature = "diagnostic-logging")]
        mut messages,
        #[cfg(feature = "diagnostic-logging")]
        mode_report,
    } = preparation;
    #[cfg(feature = "diagnostic-logging")]
    let native_grab_mapping =
        if grab_mapping_comparison.is_some() && restore.is_some() && !output.is_null() {
            Some((*output.cast::<MappedInputs>()).buttons)
        } else {
            None
        };
    if restore.is_some() && !output.is_null() {
        apply_mapper_output_adjustments(
            output,
            MapperOutputAdjustments {
                left_stick: mapped_left_stick,
                right_stick: mapped_sequence_right,
                automatic_request: mapped_automatic_request,
                short_hop: sequence_short_hop,
                attack_hold: mapped_attack_hold,
                literal_request: mapped_literal_request,
                unshielded_auto_grab: mapped_unshielded_auto_grab,
                tech_output: mapped_tech_output,
            },
        );
        record_mapped_delivery(
            output,
            index,
            session_ticket,
            MapperDeliveryFeedback {
                manual_jump: mapped_manual_jump,
                reflect: mapped_reflect,
                reflector_tick: mapped_reflector,
                auto_grab: mapped_auto_grab,
                grab_handoff: mapped_grab_handoff,
                dodge_punish_grab: mapped_dodge_punish_grab,
                auto_dodge: mapped_auto_dodge,
                auto_up_special: mapped_auto_up_special,
                tech_applied,
                #[cfg(feature = "diagnostic-logging")]
                grab_delivery,
            },
            #[cfg(feature = "diagnostic-logging")]
            &mut messages,
        );
    }
    if reserved_dpad != 0 {
        let mapped = output.cast::<MappedInputs>();
        (*mapped).buttons = suppress_reserved_taunts((*mapped).buttons, reserved_dpad);
    }
    #[cfg(feature = "diagnostic-logging")]
    if let (
        Some((
            automatic,
            owner,
            requested,
            physical,
            applied,
            physical_stick,
            applied_stick,
            physical_triggers,
            applied_triggers,
        )),
        Some(native),
    ) = (grab_mapping_comparison, native_grab_mapping)
    {
        let mapped = &*output.cast::<MappedInputs>();
        messages.push(format!(
            "grab mapping comparison: source={}, owner={owner:?}, requested={requested:#x}, physical={physical:?}, applied={applied:?}, physical_stick={physical_stick:?}, applied_stick={applied_stick:?}, physical_triggers={physical_triggers:?}, applied_triggers={applied_triggers:?}, native_buttons={native:#x}, final_buttons={:#x}, final_left=({},{})",
            if automatic { "automatic" } else { "manual" },
            mapped.buttons,
            mapped.left_x,
            mapped.left_y,
        ));
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
                "short-hop mapped pair: jump={}, jump_mini={}, attack={}, neutral_stick={}; native ATTACK_AIR/attack_air_n still confirms acceptance",
                mapped.buttons & kirby_assist_core::native_input::MAPPED_JUMP != 0,
                mapped.buttons & kirby_assist_core::native_input::MAPPED_JUMP_MINI != 0,
                mapped.buttons & kirby_assist_core::native_input::MAPPED_ATTACK != 0,
                mapped.left_x == 0 && mapped.left_y == 0,
            ));
        }
    }
    // Restore every field we changed, including C-stick and repeat flags.
    // A new field added to the overlay must also be captured and restored here.
    // Clearing reserved taunts affects `output`, so restoring raw input does not
    // undo that filter or any other action the original mapper already produced.
    if let Some(saved) = restore {
        saved.restore();
    }
    #[cfg(feature = "diagnostic-logging")]
    if let Some(report) = mode_report {
        crate::logging::write_modes(report);
    }
    #[cfg(feature = "diagnostic-logging")]
    for message in messages {
        crate::logging::write_owned(message);
    }
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
    let Some(MapperAdmission {
        config,
        master_ready_go,
        session_ticket,
    }) = admit_mapper_callback(wrapper, index)
    else {
        original!()(mappings, index, output, wrapper, arg);
        return;
    };
    #[cfg(feature = "diagnostic-logging")]
    let _mapping_time =
        crate::performance::health::Trace::new(crate::performance::health::Lane::Mapping);
    // Installed Training Modpack + actual Training Mode, in both builds.
    // Standalone Switches and other battle modes retain the normal path.
    // Copy this native scene fact before acquiring STATE. The menu and
    // Training shortcut policy share it rather than querying inside the lock.
    let training_mode = TRAINING_MODPACK_PRESENT
        .load(Ordering::Relaxed)
        .then(|| game::is_training_mode());
    let training_test_mode =
        training_test::active(training_mode.is_some(), training_mode.unwrap_or(false));
    if training_test_mode && enter_training_passthrough(wrapper, index, config) {
        original!()(mappings, index, output, wrapper, arg);
        return;
    }
    training::finish_passthrough(training_test_mode);
    // Keep the untouched trace for restoration after original!(). The original
    // mapper receives the overlay only during this call; output keeps the mapped
    // result while the physical Controller memory is restored afterwards.
    // Treat the overlay like a short transaction:
    // save physical data -> temporarily replace -> map -> restore.
    // Only the mapper's RESULT survives. Retaining our temporary raw data would
    // make the next callback mistake automation for a real physical hold.
    let Some(mut preparation) = prepare_mapper_overlay(
        index,
        output,
        wrapper,
        master_ready_go,
        training_mode,
        training_test_mode,
        session_ticket,
    ) else {
        original!()(mappings, index, output, wrapper, arg);
        return;
    };
    // original!() is Skyline's trampoline to the function we replaced. Calling
    // map_controls itself here would recurse into our hook instead.
    if !feature_activity_allowed() {
        if let Some(saved) = preparation.restore.take() {
            saved.restore();
        }
        original!()(mappings, index, output, wrapper, arg);
        return;
    }
    original!()(mappings, index, output, wrapper, arg);
    finish_mapper_overlay(output, index, session_ticket, preparation);
}

// Exported ControlModule symbol, ABI checked against pinned skyline-smash.
// Category 0 is FIGHTER_PAD_COMMAND_CATEGORY1. This observes game state only:
// it returns the game's flags unchanged and never ORs command/action flags.
#[skyline::hook(replace = game_command_flags)]
unsafe fn observe_commands(object: *mut game::Object, category: i32) -> i32 {
    let flags = original!()(object, category);
    if category != 0 || object.is_null() {
        return flags;
    }
    observe_master_presence(object);
    // Plugin entrypoints have finished before battle fighter callbacks. Leaving
    // FIM untouched until here lets Training Modpack finish its signature scan.
    // Skyline's trampoline retains its existing hook; do not bypass it.
    let assistance_enabled = RUNTIME_ENABLED.load(Ordering::Acquire)
        && CONFIG.get().is_some_and(|config| config.controls.enabled);
    if assistance_enabled {
        install_mapper_if_ready();
        observe_fighter(object, true);
    }
    flags
}

/// Observes the presence of the master player, updating internal state and timestamps as necessary.
unsafe fn observe_master_presence(object: *mut game::Object) {
    if !MASTER_PULSE_REQUESTED.swap(false, Ordering::Relaxed) {
        return;
    }
    let ready = observations::battle_menu_ready_go();
    if ready == Some(true)
        || ready == Some(false)
            && game::status_kind(object) == kirby_assist_core::native_states::status::ENTRY
    {
        MASTER_IN_MATCH.store(true, Ordering::Release);
        if let Ok(mut presence) = MASTER_PRESENCE.try_lock() {
            presence.observe(Instant::now());
        }
    }
}
/// Installs the control mapper hook if the necessary conditions are met, ensuring it is only installed once.
unsafe fn install_mapper_if_ready() {
    if MAPPING_OFFSET.load(Ordering::Relaxed) != 0 && !MAPPER_INSTALLED.swap(true, Ordering::AcqRel)
    {
        skyline::install_hook!(map_controls);
        // Read-only bounded projectile IDs for ledge shield; no Lua arguments
        // or native attacks are edited. Tracking runs only for enabled ledge
        // shield or a configured selected-fighter projectile reflector.
        attack_sources::install();
        log!("deferred mapper hook installed; existing Skyline mapper chain preserved");
    }
}
// Declares the external C function for retrieving game command flags for a given object and category.
extern "C" {
    #[link_name = "\u{1}_ZN3app8lua_bind40ControlModule__get_command_flag_cat_implEPNS_26BattleObjectModuleAccessorEi"]
    fn game_command_flags(object: *mut game::Object, category: i32) -> i32;
}
/// Represents an observation of a fighter, including its object ID, index, session ticket, fighter kind, and tick timestamp.
struct FighterObservation {
    object_id: u32,
    index: i32,
    session_ticket: SessionTicket,
    fighter_kind: i32,
    tick: i64,
}

/// Represents a publication of an observation, including the session ticket, index, tick timestamp, and setup sequence.
struct ObservationPublication {
    session_ticket: SessionTicket,
    index: i32,
    tick: i64,
    setup_sequence: u64,
}
/// Provides methods for working with observation publications, including checking if they are current.
impl ObservationPublication {
    fn is_current(&self, state: &State) -> bool {
        state.session.accepts(self.session_ticket)
            && state.mapping_index == Some(self.index)
            && kirby_assist_core::observation::observation_inputs_current(
                self.setup_sequence,
                state.observation_sequence,
                self.tick,
                state.raw_tick,
            )
            && RUNTIME_ENABLED.load(Ordering::Acquire)
    }
}
/// Refreshes the observation roster if necessary, based on the requested state, active consumer status, and the current publication.
unsafe fn refresh_observation_roster(
    requested: bool,
    active_consumer: bool,
    publication: &ObservationPublication,
) {
    if !requested || !active_consumer {
        return;
    }
    let needs_roster = STATE.lock().ok().is_some_and(|state| {
        state
            .ledge_roster
            .is_none_or(|(_, observed_tick)| observed_tick != publication.tick)
    });
    if !needs_roster {
        return;
    }
    #[cfg(feature = "diagnostic-logging")]
    crate::performance::health::event(crate::performance::health::Event::RosterScan);
    let roster = fighter_ids();
    if let Ok(mut state) = STATE.lock() {
        if publication.is_current(&state) {
            state.ledge_roster = roster.map(|ids| (ids, publication.tick));
        }
    }
}
/// Begins a full observation of a fighter, updating the state with the associated object, fighter kind, and observation sequence. Returns the new observation sequence if successful.
fn begin_full_observation(
    publication: &ObservationPublication,
    object_id: u32,
    fighter_kind: i32,
    native_status: i32,
    discover_roster: bool,
) -> Option<u64> {
    let mut state = STATE.lock().ok()?;
    if !publication.is_current(&state) {
        return None;
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
    state.ledge_visits.set_owner(object_id, fighter_kind);
    state.associated_object = Some(object_id);
    state.fighter_kind = Some(fighter_kind);
    state.menu_fighter_kind = Some(fighter_kind);
    if native_status == kirby_assist_core::native_states::status::ENTRY && !state.menu_battle_active
    {
        state.menu_intro_object = Some(object_id);
    }
    state
        .observation_setup
        .complete(publication.tick, discover_roster);
    if state.evaluated_tick == Some(publication.tick) && state.observed_object == Some(object_id) {
        return None;
    }
    state.observation_sequence = state.observation_sequence.wrapping_add(1);
    Some(state.observation_sequence)
}
/// Admits a fighter observation, validating the object and controller index, and updating the state with the observation if successful. Returns the fighter observation if applicable.
unsafe fn admit_fighter_observation(
    object: *mut game::Object,
    discover_roster: bool,
) -> Option<FighterObservation> {
    if object.is_null() {
        return None;
    }
    // The callback owns a live accessor. Reject inactive/recycled identities
    // before dereferencing any ControlModule or other fighter module pointers.
    let object_id = game::battle_object_id(object);
    if object_id >> 28 != kirby_assist_core::battle_objects::OBJECT_CATEGORY_FIGHTER
        || !game::is_active(object_id)
        || game::module_accessor(object_id) != object
    {
        return None;
    }
    // ControlModule +0x118 -> +0x58 -> +8 -> index at +8.
    // A direct ControlModule+8 read does not resolve the controller index.
    let Some(index) = controller_index(object.cast()) else {
        #[cfg(feature = "diagnostic-logging")]
        if feature_activity_allowed() && !ROUTE_UNAVAILABLE_LOGGED.swap(true, Ordering::Relaxed) {
            log!("fighter controller ownership chain unavailable; waiting for a valid local fighter input index");
        }
        return None;
    };
    // Nonselected/remote callbacks may share human metadata. They cannot
    // initialize policies, cache opponents or renew the local menu presence.
    if !STATE
        .lock()
        .ok()
        .is_some_and(|state| state.mapping_index == Some(index))
    {
        return None;
    }
    // CPU actors cannot establish participation even if they share a default
    // input index. Arena roster slots are not physical controller identities.
    if !human_fighter(object) {
        return None;
    }
    let session_ticket = live_match_ticket()?;
    let fighter_kind = game::fighter_kind(object_id);
    let (tick, target) = {
        let mut state = STATE.lock().ok()?;
        if !state.session.accepts(session_ticket) {
            return None;
        }
        let now = Instant::now();
        state.human_routes[index as usize] = Some((object_id, now));
        if state.mapping_index == Some(index) {
            settings_menu::observe_local_fighter(now);
        }
        let tick = state.raw_tick;
        // Keep controller discovery fresh, but skip setup already rejected by
        // the publication guard below. A callback must still refresh the roster
        // after a mapper-only observation, so do not skip that discovery pass.
        if state.mapping_index == Some(index)
            && state.evaluated_tick == Some(tick)
            && state.observed_object == Some(object_id)
            && state.associated_object == Some(object_id)
            && state.fighter_kind == Some(fighter_kind)
            && !state.observation_setup.needed(tick, true, discover_roster)
        {
            #[cfg(feature = "diagnostic-logging")]
            crate::performance::health::event(crate::performance::health::Event::ObservationReuse);
            return None;
        }
        (tick, state.mapping_index)
    };
    if target != Some(index) {
        #[cfg(feature = "diagnostic-logging")]
        if ROUTE_MISMATCHES.fetch_or(1 << index, Ordering::Relaxed) & (1 << index) == 0 {
            log!(&format!("human fighter input index {index}; selected physical controller's mapper is {target:?}; route cached for mapping"));
        }
        return None;
    }
    // Menus/recording may still issue fighter callbacks. Do not regenerate
    // proposals or advance recipes while Training owns the selected input.
    if TRAINING_PASSTHROUGH.load(Ordering::Relaxed) && game::is_training_mode() {
        return None;
    }
    Some(FighterObservation {
        object_id,
        index,
        session_ticket,
        fighter_kind,
        tick,
    })
}
/// Represents an observation of a tech situation, including whether the fighter has stopped, clatter status, input delay, auto-tech settings, floor motion history, tech assessment, and timing information.
struct TechObservation {
    stopped: bool,
    clatter_remaining: f32,
    clatter: bool,
    input_delay: f32,
    auto_tech_enabled: bool,
    tech_floor_motion: kirby_assist_core::tech_prediction::FloorMotionHistory,
    assessment: tech::TechAssessment,
    tech: bool,
    native_tech_guard: bool,
    tech_timing: (u16, u16, u16),
}

/// Represents the basic native observation of a fighter, including status, situation, motion details, and input blocking information.
struct NativeObservationBasics {
    status: i32,
    situation: i32,
    stopped: bool,
    motion_frame: f32,
    motion_kind: u64,
    motion_rate: f32,
    manual_automatic_blocked: bool,
    damage_stop: bool,
}
/// Observes the basic native state of a fighter, including status, situation, motion details, and input blocking information. Returns a `NativeObservationBasics` struct with the observed values.
unsafe fn observe_native_basics(
    reads: &frame::FrameReads,
    object: *mut game::Object,
    object_id: u32,
    fighter_kind: i32,
    config: &Config,
) -> NativeObservationBasics {
    let status = reads.status_kind(object);
    let situation = reads.situation_kind(object);
    let stopped = reads.is_stop(object);
    let motion_frame = reads.motion_frame(object);
    let motion_kind = reads.motion_kind(object);
    let motion_rate = reads.motion_rate(object);
    let manual_context = GameplayContext {
        gameplay: true,
        status,
        situation,
        stopped,
        motion_kind,
        motion_frame,
        motion_rate,
        ..GameplayContext::default()
    };
    let manual_automatic_blocked =
        manual_automatic_input_blocked(object_id, fighter_kind, manual_context);
    let damage_stop = (config.auto_sdi || config.auto_di.enabled)
        && (kirby_assist_core::native_states::status::DAMAGE
            ..=kirby_assist_core::native_states::status::DOWN_REFLECT_LR)
            .contains(&status)
        && reads.is_stop(object)
        && game::is_damage_stop(object);
    NativeObservationBasics {
        status,
        situation,
        stopped,
        motion_frame,
        motion_kind,
        motion_rate,
        manual_automatic_blocked,
        damage_stop,
    }
}
/// Observes the tech state of a fighter, including hitstop, clatter, input delay, auto-tech settings, and floor motion history. Returns a `TechObservation` struct with the observed values.
unsafe fn observe_tech_state(
    reads: &frame::FrameReads,
    object: *mut game::Object,
    config: &Config,
    analysis_settings: kirby_assist_core::fighter_tuning::Tuning,
) -> TechObservation {
    // Hitstop freezes native action progress. It differs from cancellation:
    // keep remembered cooldowns/plans while withholding new mash/tech requests.
    let stopped = reads.is_stop(object);
    let clatter_remaining = game::clatter_time(object, 0);
    let clatter = !stopped && clatter_remaining.is_finite() && clatter_remaining > 0.0;
    // Prediction is evaluated separately from the scheduler: finding contact
    // does not mean the cooldown is ready or that the game will accept a tech.
    // Learned input-delay compensation has been removed. Native move timing
    // and explicit response configuration are the only timing inputs.
    let input_delay = 0.0;
    let (auto_tech_enabled, mut tech_floor_motion) =
        STATE.lock().ok().map_or(Default::default(), |state| {
            (
                state
                    .settings_menu
                    .as_ref()
                    .map_or(config.controls.auto_tech, |menu| {
                        menu.auto_tech(config.controls.auto_tech)
                    }),
                state.tech_floor_motion,
            )
        });
    let assessment = assess_tech(
        reads,
        object,
        config.lookahead,
        stopped,
        input_delay,
        auto_tech_enabled,
        &mut tech_floor_motion,
        analysis_settings.tech_body(),
    );
    let tech = !clatter && assessment.prepare_guard && assessment.contact_in.is_some();
    // Read the fighter's consumed button state outside STATE. GuardHold is a
    // second shield alias; a mapped neutral/press can still be in transit.
    let native_tech_guard = auto_tech_enabled
        && (game::button_on(object, game::CONTROL_PAD_BUTTON_GUARD)
            || game::button_on(object, game::CONTROL_PAD_BUTTON_GUARD_HOLD));
    // Current native common.prc values, rather than the legacy configured
    // synthetic forty-frame lockout. Invalid/missing keys retain compatibility.
    // Parameter names and integer types are pinned in skyline-smash params/common.rs.
    let tech_timing = if auto_tech_enabled && (tech || assessment.prepare_guard) {
        self::tech::native_timing(object, config.controls.tech_cooldown)
    } else {
        (
            config.controls.tech_cooldown,
            kirby_assist_core::tech_input::DEFAULT_PASSIVE_TRIGGER_FRAME,
            kirby_assist_core::tech_input::DEFAULT_GANON_PASSIVE_TRIGGER_FRAME,
        )
    };
    TechObservation {
        stopped,
        clatter_remaining,
        clatter,
        input_delay,
        auto_tech_enabled,
        tech_floor_motion,
        assessment,
        tech,
        native_tech_guard,
        tech_timing,
    }
}

// This observer reads game facts and publishes input requests; it never changes
// fighter status. Native reads happen outside STATE and publication revalidates
/// Observes the fighter's state, including native reads and session validation. This function never changes fighter status and ensures that the session is revalidated because native reads can re-enter hooks.
unsafe fn observe_fighter(object: *mut game::Object, discover_roster: bool) {
    let Some(config) = CONFIG.get().map(Box::as_ref) else {
        return;
    };
    if !config.controls.enabled || !RUNTIME_ENABLED.load(Ordering::Acquire) {
        return;
    }
    let Some(FighterObservation {
        object_id,
        index,
        session_ticket,
        fighter_kind,
        tick,
    }) = admit_fighter_observation(object, discover_roster)
    else {
        return;
    };
    let Some(_observation_guard) = FighterObservationGuard::acquire() else {
        return;
    };
    // Only an accepted local observation needs profile/menu resolution and
    // policy setup. Duplicate command hooks still refresh human routing and
    // presence above, including the existing roster-discovery exception.
    // Training settings are identical to gameplay settings; recording/menu
    // passthrough was handled above. No extra scene query or config identity pass.
    let config = diagnostic_config(*config);
    let Some((config, setup_sequence, needs)) =
        resolve_observation_setup(index, session_ticket, fighter_kind, config)
    else {
        return;
    };
    let ObservationNeeds {
        find_trump_edge,
        find_trap_edge,
        observe_ledge,
        preview_start,
        continue_ledge_response,
        literal_ledge_timeline,
        file_ledge_timeline,
        ground_ledge_timeline,
        observe_trump,
    } = needs;

    let reads = frame::FrameReads::default();
    #[cfg(feature = "diagnostic-logging")]
    let _performance = crate::performance::Scope::root();
    #[cfg(feature = "diagnostic-logging")]
    crate::performance::health::event(crate::performance::health::Event::Observation);
    let NativeObservationBasics {
        status: native_status,
        situation: native_auto_ground,
        stopped: native_auto_stopped,
        motion_frame: ledge_motion_frame,
        motion_kind: own_motion_kind,
        motion_rate: timing_rate,
        manual_automatic_blocked,
        damage_stop,
    } = observe_native_basics(&reads, object, object_id, fighter_kind, &config);
    // Copy saved-mode flags once; no native queries happen while holding this
    // lock. All readers in this observation use the same configuration snapshot.
    let ObservationPolicySnapshot {
        automatic_suspended,
        trip_queued,
        grab_dodge_enabled,
        posture_modes,
        posture_down,
        roster_missing,
        oos_enabled,
        oos_target,
        rules_on,
        rule_target,
        dodge_follow,
        automatic_sequence_committed,
        up_b_receipt,
        rules_ground,
        simple_starts,
        dodge_lookahead,
        manual_input_epoch,
        analysis_settings,
        automatic_limits,
        dodge_direct_up_b,
        want_up_special,
        ordinary_backup,
        stationary_intent,
        grab_intent,
        self_shielded,
        grab_dash_pending,
    } = snapshot_observation_policy(
        fighter_kind,
        manual_automatic_blocked,
        native_status,
    );
    let own_capture_link = if !automatic_suspended
        && (rules_on
            || oos_enabled
            || grab_dodge_enabled
            || rule_target.is_some()
            || oos_target.is_some()
            || dodge_follow.is_some())
        && (kirby_assist_core::reaction_rules::automatic_start_status(native_status)
            || automatic_sequence_committed
            || native_status == kirby_assist_core::native_states::ESCAPE && dodge_follow.is_some())
    {
        reads.capture_link(object)
    } else {
        false
    };
    let automatic_scope = kirby_assist_core::reaction_rules::AutomaticObservationScope::new(
        automatic_suspended,
        native_status,
        native_auto_ground,
        native_auto_stopped,
        own_capture_link,
        dodge_follow.is_some(),
        automatic_sequence_committed,
    )
    .with_up_b_receipt(
        automatic_suspended,
        own_capture_link,
        up_b_receipt,
        native_status,
        native_auto_ground,
        dodge_follow.is_some(),
    );
    let trip_kirby = kirby_followups::trip_enabled(
        automatic_suspended,
        config.down_tilt_trip_smash,
        fighter_kind,
    );
    let pressure_on = oos_enabled;
    let (legacy_grab_close, legacy_grab_approach) =
        if oos_enabled && (automatic_limits[0].is_none() || automatic_limits[1].is_none()) {
            let standing = if config.assist.fighter_kind == fighter_kind {
                config.assist.fighter.grab.standing_range
            } else {
                0.
            };
            analysis_settings.auto_grab_ranges(fighter_kind, standing)
        } else {
            (0., 0.)
        };
    let auto_grab_range = automatic_limits[1].map_or(legacy_grab_close, |v| v.0);
    let auto_grab_approach_range = automatic_limits[0].map_or(legacy_grab_approach, |v| v.0);
    let auto_grab_height = automatic_limits[0].map_or(24., |v| v.1);
    let auto_shield_height = automatic_limits[1].map_or(24., |v| v.1);
    let auto_backup_range = automatic_limits[2].map_or(auto_grab_range, |v| v.0);
    let auto_backup_height = automatic_limits[2].map_or(24., |v| v.1);
    let trip_watch = kirby_followups::trip_watch(trip_kirby, native_status, trip_queued);
    let observe_roll_read =
        automatic_scope.acquire && (rules_on || pressure_on) || automatic_scope.follow;
    let defense_observation_enabled = grab_dodge_enabled || oos_enabled;
    let roster_scan_requested = discover_roster
        || preview_start
        || (rules_on || pressure_on) && roster_missing
        || defense_observation_enabled && roster_missing
        || posture_modes.1 && posture_down && roster_missing;
    let roster_has_active_consumer = observe_ledge
        || trip_watch
        || damage_stop
        || observe_roll_read
        || posture_modes.1
            && (posture_down
                || matches!(
                    native_status,
                    kirby_assist_core::native_states::status::ATTACK_LW3
                        | kirby_assist_core::native_states::status::ATTACK_S3
                        | kirby_assist_core::native_states::status::ATTACK_HI3
                ))
        || posture_modes.0
        || defense_observation_enabled && automatic_scope.acquire;
    // A first Down input can need facing before any other assist discovered
    // opponents. Seed once here; ordinary refreshes stay on the existing path.
    let publication = ObservationPublication {
        session_ticket,
        index,
        tick,
        setup_sequence,
    };
    refresh_observation_roster(
        roster_scan_requested,
        roster_has_active_consumer,
        &publication,
    );
    let Some(observation_sequence) = begin_full_observation(
        &publication,
        object_id,
        fighter_kind,
        native_status,
        discover_roster,
    ) else {
        return;
    };
    if !config.controls.enabled || !game::is_active(object_id) {
        if let Ok(mut state) = STATE.lock() {
            state.cancel();
        }
        return;
    }
    #[cfg(feature = "diagnostic-logging")]
    crate::performance::health::event(crate::performance::health::Event::FullObservation);
    let TechObservation {
        stopped,
        clatter_remaining,
        clatter,
        input_delay,
        auto_tech_enabled,
        tech_floor_motion,
        assessment,
        tech,
        native_tech_guard,
        tech_timing,
    } = observe_tech_state(&reads, object, &config, analysis_settings);
    if matches!(
        assessment.status,
        kirby_assist_core::native_states::status::DEAD
            | kirby_assist_core::native_states::status::REBIRTH
            | kirby_assist_core::native_states::status::STANDBY
            | kirby_assist_core::native_states::status::ENTRY
            ..=kirby_assist_core::native_states::status::LOSE
    ) {
        if let Ok(mut state) = STATE.lock() {
            state.cancel();
        }
        return;
    }

    // Read native data outside STATE's mutex to avoid re-entrant deadlocks.
    // One bounded roster pass, only while custom rules are enabled. Follow the
    // locked target through recovery without a second scan or stale position.
    let rule_observation = if automatic_scope.acquire && (rules_on || pressure_on)
        || rule_target.is_some()
        || oos_target.is_some()
        || dodge_follow.is_some()
    {
        let (targets, follow) = if !clatter && !tech && automatic_scope.observe_targets() {
            observations::rule_targets(
                &reads,
                object,
                object_id,
                rule_target.or(oos_target).or(dodge_follow),
                (oos_enabled && automatic_scope.acquire).then_some(if self_shielded {
                    [
                        auto_grab_range.max(auto_backup_range),
                        auto_shield_height.max(auto_backup_height),
                    ]
                } else {
                    [auto_grab_approach_range, auto_grab_height]
                }),
                self_shielded,
                rules_on && automatic_scope.acquire,
                automatic_sequence_committed,
            )
        } else {
            ([None; 8], None)
        };
        let position = [reads.pos_x(object), reads.pos_y(object)];
        let velocity = [reads.speed_x(object), reads.speed_y(object)];
        let rule_needs_ground = rules_ground.is_some_and(|(distance, height, active)| {
            if rule_target.is_some() || oos_target.is_some() {
                active
            } else {
                targets.iter().flatten().any(|t| {
                    (matches!(
                        t.status,
                        kirby_assist_core::native_states::ESCAPE
                            ..=kirby_assist_core::native_states::ESCAPE_AIR_SLIDE
                    ) || t.vulnerable_in == Some(0.))
                        && (t.position[0] - position[0]).abs() <= distance
                        && (t.position[1] - position[1]).abs() <= height
                }) || dodge_follow.is_some()
                    && follow.is_some_and(|e| {
                        (e.position[0] - position[0]).abs() <= distance
                            && (e.position[1] - position[1]).abs() <= height
                    })
            }
        });
        let grab_ground = oos_enabled
            && !self_shielded
            && grab_intent
            && !stopped
            && (grab_dash_pending
                || kirby_assist_core::out_of_shield::ground_grab_state(assessment.status)
                    && targets.iter().flatten().any(|t| {
                        kirby_assist_core::out_of_shield::automatic_grab_target_status(t.status)
                            && t.vulnerable_in == Some(0.)
                            && assessment.status == kirby_assist_core::native_states::status::WAIT
                            && (t.position[0] - position[0]).abs() > auto_grab_range
                    }));
        let needs_ground = assessment.situation
            == kirby_assist_core::native_states::situation::GROUND
            && (rule_needs_ground || grab_ground);
        Some((
            GameplayContext {
                held: true,
                gameplay: true,
                manual_input_epoch,
                stopped,
                status: assessment.status,
                situation: assessment.situation,
                motion_frame: ledge_motion_frame,
                motion_kind: own_motion_kind,
                motion_rate: timing_rate,
                position,
                velocity,
                facing: reads.facing(object),
                previous_status: reads
                    .previous_status(object, game::LATEST_PREVIOUS_STATUS_HISTORY_INDEX),
                // Only admit a new grounded Up-B reaction when the native game
                // can accept it now. Locomotion keeps its momentum until then.
                can_act: want_up_special
                    && !own_capture_link
                    && !clatter
                    && !tech
                    && !stopped
                    && assessment.situation == kirby_assist_core::native_states::situation::GROUND
                    && (automatic_scope.acquire
                        || automatic_scope.follow
                            && assessment.status == kirby_assist_core::native_states::ESCAPE
                            && game::cancel_enabled(object))
                    && game::transition_enabled(
                        object,
                        kirby_assist_core::native_states::transition_terms::CONT_SPECIAL_HI,
                    ), // CONT_SPECIAL_HI
                ground_brake: if assessment.situation
                    == kirby_assist_core::native_states::situation::GROUND
                    && rule_needs_ground
                {
                    reads.ground_brake(object)
                } else {
                    None
                },
                safe_ground: if needs_ground {
                    reads.ground_support(object, position, velocity[0])
                } else {
                    [false; 2]
                },
                ..GameplayContext::default()
            },
            targets,
            follow,
        ))
    } else {
        None
    };
    // Per-event readiness is computed outside STATE, only for an actual nearby
    // candidate. Ordinary fighter recipes never depend on Up-B availability.
    let simple_readiness = rule_observation
        .as_ref()
        .map_or([false; 3], |(context, targets, _)| {
            kirby_assist_core::reaction_rules::simple_readiness(
                simple_starts,
                *context,
                targets,
                || {
                    game::transition_enabled(object, kirby_assist_core::native_states::transition_terms::CONT_CATCH) // CONT_CATCH
                || game::transition_enabled(object, kirby_assist_core::native_states::transition_terms::CONT_JUMP_SQUAT_BUTTON) // CONT_JUMP_SQUAT_BUTTON
                },
            )
        });
    // Query grab/OOS readiness only with a current vulnerable target or active attempt.
    let oos_readiness = if oos_enabled
        && automatic_scope.acquire
        && (grab_intent || oos_target.is_some())
        && assessment.situation == kirby_assist_core::native_states::situation::GROUND
        && kirby_assist_core::out_of_shield::ground_grab_state(assessment.status)
        && (oos_target.is_some()
            || rule_observation.as_ref().is_some_and(|(_, targets, _)| {
                targets
                    .iter()
                    .flatten()
                    .any(|t| t.vulnerable_in == Some(0.))
            })) {
        let ready = !stopped
            && (assessment.status != kirby_assist_core::native_states::GUARD_DAMAGE
                || game::work_int(
                    object,
                    kirby_assist_core::native_states::work_ids::FIGHTER_STATUS_GUARD_DAMAGE_WORK_INT_STIFF_FRAME,
                ) <= 0);
        (
            ready,
            ready
                && kirby_assist_core::out_of_shield::automatic_grab_ready_with_shield(
                    assessment.status,
                    self_shielded,
                    |term| game::transition_enabled(object, term),
                ),
            ready
                && ordinary_backup
                && self_shielded
                && (game::transition_enabled(
                    object,
                    kirby_assist_core::native_states::transition_terms::CONT_JUMP_SQUAT_BUTTON,
                ) || game::transition_enabled(
                    object,
                    kirby_assist_core::native_states::transition_terms::CONT_JUMP_SQUAT,
                )),
            ready && self_shielded && rule_observation.as_ref().is_some_and(|(c, _, _)| c.can_act),
        )
    } else {
        (false, false, false, false)
    };
    // Observation stage: build ordinary Rust values from read-only game calls.
    // Startup/cancel/hit-status queries describe the real game; they do not
    // make an input legal. The core uses these values to decide what to request.
    // Full movement/ledge queries (including ground rays) are demand-driven.
    // Stick-click/Y shortcuts use the existing motion sample, without ground rays.
    let mut guard_threat = false;
    let mut guard_ready = false;
    // Status-gated opponent acquisition comes first. Unrelated enemies stop at
    // their status checks; a ledge grab/getup gets full facts in this same pass.
    let ledge_opponent = if observe_ledge
        && literal_ledge_timeline
        && kirby_assist_core::ledge_trap::needs_opponent_clock(native_status, timing_rate)
    {
        observations::tracked_ledge_clock(&reads)
    } else {
        (observe_ledge && !literal_ledge_timeline)
            .then(|| nearest_ledge_opponent(&reads, object, object_id, config.assist))
            .flatten()
    };
    let response_observation = observe_ledge
        && !file_ledge_timeline
        && (ledge_opponent.is_some() || continue_ledge_response);
    let spatial_timeline = observe_ledge && ground_ledge_timeline;
    let position_observation = observe_ledge
        && !file_ledge_timeline
        && config.assist.auto_position
        && !config.assist.manual_spacing
        && !config.assist.distance_agnostic;
    let ledge_observation = if observe_ledge || observe_trump {
        #[cfg(feature = "diagnostic-logging")]
        let _performance = crate::performance::Scope::new(crate::performance::Part::Ledge);
        let position = [reads.pos_x(object), reads.pos_y(object)];
        let velocity = [reads.speed_x(object), reads.speed_y(object)];
        let guard_armed = observe_ledge
            && !file_ledge_timeline
            && config.autopilot_shield
            && assessment.situation == kirby_assist_core::native_states::situation::GROUND
            && !kirby_assist_core::native_states::fighter_sequence_interrupted(
                fighter_kind,
                assessment.status,
            )
            && !clatter
            && !tech
            && STATE.lock().ok().is_some_and(|s| s.ledge_active);
        if guard_armed {
            // CONT_GUARD_ON, verified in pinned skyline-smash lua_const.rs.
            // Existing guard/shieldstun can retain ZR even during hitstop.
            guard_ready = matches!(
                assessment.status,
                kirby_assist_core::native_states::GUARD_ON
                    | kirby_assist_core::native_states::GUARD
                    | kirby_assist_core::native_states::GUARD_DAMAGE
            ) || (!stopped
                && game::transition_enabled(
                    object,
                    kirby_assist_core::native_states::transition_terms::CONT_GUARD_ON,
                ));
            guard_ready &= !kirby_assist_core::action_owner::native_grab_active(assessment.status)
                && !reads.capture_link(object);
            if guard_ready {
                guard_threat = observations::incoming_fighter_attack(
                    &reads,
                    object,
                    object_id,
                    position,
                    velocity,
                    analysis_settings.shield(),
                );
            }
        }
        // File actions are literal inputs; do not query move startup/readiness.
        let shield_stun_frames = None;
        let shield_attacker = None;
        let context = GameplayContext {
            held: false,
            gameplay: true,
            manual_input_epoch,
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
            ground_brake: if response_observation || spatial_timeline || position_observation {
                reads.ground_brake(object)
            } else {
                None
            },
            facing: reads.facing(object),
            previous_status: reads
                .previous_status(object, game::LATEST_PREVIOUS_STATUS_HISTORY_INDEX),
            can_grab: false,
            can_tilt: false,
            can_down_tilt: false,
            can_down_special: false,
            can_dash_attack: false,
            can_down_smash: false,
            can_jump: false,
            can_air_attack: false,
            input_delay: 0.0,
            air_physics: None,
            short_hop_speed: None,
            full_hop_speed: None,
            shield_stun_frames,
            shield_attacker,
            hit_confirmed: false,
            safe_ground: if assessment.situation
                == kirby_assist_core::native_states::situation::GROUND
                && (response_observation
                    || spatial_timeline
                    || observe_trump
                    || position_observation)
            {
                reads.ground_support(object, position, velocity[0])
            } else {
                [false, false]
            },
        };
        // The trump uses the current fighter's floor geometry, independently of the
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
                config.assist.stand_distance,
            )
        } else {
            None
        };
        Some((context, ledge_opponent, edge, spot))
    } else {
        None
    };
    let (trip_targets, outward) = if trip_watch || damage_stop {
        combat_targets(&reads, object, object_id, trip_watch, damage_stop)
    } else {
        ([None; 8], 0.0)
    };
    let followup_observation = kirby_followups::Observation {
        object: object_id,
        kind: fighter_kind,
        status: assessment.status,
        situation: assessment.situation,
        motion: own_motion_kind,
        frame: ledge_motion_frame,
        rate: timing_rate,
        stopped,
        facing: reads.facing(object),
    };
    let trip_context =
        kirby_followups::trip_context(&followup_observation, trip_kirby, trip_targets);
    let fair_pending = !automatic_suspended
        && STATE
            .lock()
            .ok()
            .is_some_and(|s| kirby_followups::fair_active(&s.fair_followup));
    // Only the still-supported Kirby automatic fair-one follow-up needs hit
    // evidence. Retired manually bound finishers do not widen this window.
    let fair_hit_window = kirby_followups::fair_hit_window(
        automatic_suspended,
        kirby_followups::is_kirby(fighter_kind),
        config.kirby_fair1_smash,
        assessment.status,
        own_motion_kind,
    );
    let fair_context =
        kirby_followups::fair_context(&followup_observation, fair_hit_window, || {
            game::attack_infliction_now(object, game::COLLISION_KIND_MASK_HIT)
        });
    let observe_posture_target = (posture_modes.0
        || posture_modes.1
            && (posture_down
                || matches!(
                    assessment.status,
                    kirby_assist_core::native_states::status::ATTACK_LW3
                        | kirby_assist_core::native_states::status::ATTACK_S3
                        | kirby_assist_core::native_states::status::ATTACK_HI3
                )))
        && kirby_assist_core::posture::Posture::needs_configured_target(
            assessment.status,
            assessment.situation,
            stopped || clatter || tech,
        );
    let posture_opponent = if observe_posture_target {
        observations::posture_opponent(&reads, object, object_id)
    } else {
        None
    };
    let posture_target = posture_opponent
        .map(|offset| offset[0])
        .filter(|x| x.abs() > 0.5)
        .map(f32::signum);
    // A Down input on a passable platform may drop through. Leave those floors
    // free of automatic Down; walking remains capped on these floors.
    let posture_floor_safe = posture_modes.0
        && assessment.situation == kirby_assist_core::native_states::situation::GROUND
        && kirby_assist_core::posture::Posture::opponent_near_with(
            posture_opponent,
            analysis_settings.value(kirby_assist_core::fighter_tuning::CROUCH_RANGE, 45.),
            analysis_settings.value(kirby_assist_core::fighter_tuning::CROUCH_HEIGHT, 20.),
        )
        && !game::passable_ground(object);
    let manual_context = ManualContext {
        status: assessment.status,
        situation: assessment.situation,
        motion: own_motion_kind,
        frame: ledge_motion_frame,
        rate: timing_rate,
        stopped: stopped || clatter || tech,
        facing: trip_context.facing,
    };
    let combo_velocity_y = if fair_hit_window || fair_pending {
        reads.speed_y(object)
    } else {
        0.0
    };
    let dodge_response_ready = if grab_dodge_enabled && !dodge_direct_up_b {
        !own_capture_link
            && !stopped
            && assessment.situation == kirby_assist_core::native_states::situation::GROUND
            && (automatic_scope.acquire
                || automatic_scope.follow
                    && assessment.status == kirby_assist_core::native_states::ESCAPE
                    && game::cancel_enabled(object))
    } else {
        false
    };
    let spot_observation = if grab_dodge_enabled
        && !own_capture_link
        && assessment.situation == kirby_assist_core::native_states::situation::GROUND
        && (automatic_scope.acquire
            || automatic_scope.follow && assessment.status == kirby_assist_core::spot_dodge::ESCAPE)
        && (stationary_intent && kirby_assist_core::spot_dodge::ground_status(assessment.status)
            || assessment.status == kirby_assist_core::spot_dodge::ESCAPE && dodge_follow.is_some())
    {
        // Detect a real grab before querying our dodge transition. Keep the
        // ordinary sample even with no target so pending requests can release.
        let targets = observations::grab_targets(&reads, object, object_id);
        Some((
            kirby_assist_core::spot_dodge::Context {
                object: object_id,
                status: assessment.status,
                situation: assessment.situation,
                motion: own_motion_kind,
                frame: ledge_motion_frame,
                rate: timing_rate,
                stopped,
                can_spot_dodge: targets.iter().any(Option::is_some)
                    && !clatter
                    && !tech
                    && !stopped
                    && (game::transition_enabled(
                        object,
                        kirby_assist_core::native_states::transition_terms::CONT_ESCAPE,
                    ) || kirby_assist_core::spot_dodge::ground_status(assessment.status)
                        && game::transition_enabled(
                            object,
                            kirby_assist_core::native_states::transition_terms::CONT_GUARD_ON,
                        )),
                position: [reads.pos_x(object), reads.pos_y(object)],
                velocity: [reads.speed_x(object), reads.speed_y(object)],
            },
            targets,
        ))
    } else {
        None
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
    // Read reserved launch velocity only for the damage victim during hitstop.
    // Pinned FIGHTER_INSTANCE_WORK_ID_FLOAT_DAMAGE_SPEED_RESERVED_X/Y.
    let di_context = (config.auto_di.enabled
        && damage_stop
        && (kirby_assist_core::native_states::status::DAMAGE_FLY
            ..=kirby_assist_core::native_states::status::DAMAGE_FALL)
            .contains(&native_status))
    .then(|| kirby_assist_core::auto_di::Context {
        object: object_id,
        damage_stop,
        remaining: sdi_context.remaining,
        launch: reads.damage_launch(object),
        outward,
    });
    let cached_geometry = STATE.lock().ok().and_then(|s| s.damage_geometry);
    let (damage_geometry, damage_trajectory) = damage::observe(
        &reads,
        object,
        object_id,
        damage_stop,
        sdi_context.remaining,
        cached_geometry,
    );
    #[cfg(feature = "diagnostic-logging")]
    let mut scenario_changed = false;
    #[cfg(feature = "diagnostic-logging")]
    let mut acquired = false;
    #[cfg(feature = "diagnostic-logging")]
    let mut first_button_write = false;
    let oos_follow = rule_observation.and_then(|(_, _, follow)| follow);
    let oos_context =
        ((oos_enabled && automatic_scope.acquire) || oos_target.is_some()).then(|| {
            kirby_assist_core::out_of_shield::Context {
                object: object_id,
                kind: fighter_kind,
                status: assessment.status,
                motion: own_motion_kind,
                frame: ledge_motion_frame,
                rate: timing_rate,
                stopped,
                grounded: assessment.situation
                    == kirby_assist_core::native_states::situation::GROUND,
                position: [reads.pos_x(object), reads.pos_y(object)],
                facing: reads.facing(object),
                shield_ready: oos_readiness.0,
                can_grab: oos_readiness.1,
                can_jump: oos_readiness.2,
                can_up_special: oos_readiness.3,
                threatened: guard_threat,
            }
        });
    #[cfg(feature = "diagnostic-logging")]
    let neutral_native_activity = if observe_ledge
        && (ledge_opponent.is_some_and(|e| e.status == kirby_assist_core::ledge_trap::CLIFF_CLIMB)
            || STATE.lock().ok().is_some_and(|s| {
                s.calibration.active()
                    && s.ledge.as_ref().and_then(LedgeTrap::calibration_slot)
                        == Some(kirby_assist_core::fighter_profiles::presets::PresetSlot::Neutral)
            })) {
        Some(
            matches!(
                assessment.status,
                kirby_assist_core::native_states::status::ATTACK
                    ..=kirby_assist_core::native_states::status::ATTACK_AIR
            ) && (0..ATTACK_SLOT_COUNT).any(|slot| game::attack_active(object, slot, false)),
        )
    } else {
        None
    };
    let reflect_observation = {
        let (configured, target, allowed) = STATE.lock().ok().map_or((None, None, false), |s| {
            let manual = s.raw_buttons
                & (kirby_assist_core::native_input::MAPPER_A
                    | kirby_assist_core::native_input::MAPPER_B
                    | kirby_assist_core::native_input::MAPPER_R
                    | kirby_assist_core::native_input::SEQUENCE_JUMP_BUTTONS
                    | kirby_assist_core::native_input::SEQUENCE_GUARD_INPUTS)
                != 0;
            (
                s.settings_menu
                    .as_ref()
                    .and_then(|m| m.reflect_config(fighter_kind)),
                s.reflector.target(),
                !automatic_suspended
                    && !clatter
                    && !tech
                    && (!manual || s.action_owner.current_owner() == Some(Owner::Reflect))
                    && s.action_owner.allows(Owner::Reflect),
            )
        });
        reflect::observe(
            &reads,
            object,
            object_id,
            fighter_kind,
            configured,
            target,
            allowed,
        )
    };
    // Policy/publication stage: the native reads above are finished. Check the
    // session again before updating history, because those reads can re-enter
    // hooks. No file loading or JSON parsing belongs in this per-update path.
    if let Ok(mut state) = STATE.lock() {
        // Queries belong to the controller sample captured before observation.
        // Never relabel a late result as a newer sample: buttons/ledge priority
        // and native eligibility may have changed while those reads ran.
        // The mapper also refreshes synchronously before selecting its packet.
        if !state.session.accepts(session_ticket)
            || state.mapping_index != Some(index)
            || state.associated_object != Some(object_id)
            || state.observation_sequence != observation_sequence
            || !kirby_assist_core::observation::input_update_current(Some(tick), state.raw_tick)
            || !RUNTIME_ENABLED.load(Ordering::Acquire)
        {
            #[cfg(feature = "diagnostic-logging")]
            crate::performance::health::event(
                crate::performance::health::Event::PublicationDiscard,
            );
            return;
        }
        if state.evaluated_tick == Some(tick) && state.observed_object == Some(object_id) {
            return;
        }
        state.tech_floor_motion = tech_floor_motion;
        if state.grab_handoff.pending() {
            let clock_context = rule_observation.map_or(
                GameplayContext {
                    gameplay: true,
                    status: native_status,
                    situation: native_auto_ground,
                    stopped: native_auto_stopped,
                    motion_kind: own_motion_kind,
                    motion_frame: ledge_motion_frame,
                    motion_rate: timing_rate,
                    ..Default::default()
                },
                |(c, _, _)| c,
            );
            let clock_enemy =
                rule_observation.and_then(|(_, targets, follow)| {
                    follow.or_else(|| {
                        targets.iter().flatten().next().map(|t| {
                            kirby_assist_core::gameplay::Opponent {
                                id: t.id,
                                status: t.status,
                                motion: Some(kirby_assist_core::gameplay::MotionTiming {
                                    kind: t.motion,
                                    frame: t.frame,
                                    rate: t.rate,
                                    remaining: None,
                                }),
                                ..Default::default()
                            }
                        })
                    })
                });
            state
                .grab_handoff
                .observe_enemy_progress(clock_context, clock_enemy);
        }
        // Reuse facts already read by active analyses. Missing targets are not
        // evidence of a new event; a known native status change is. This adds
        // no roster pass or native queries and prevents duplicate/stale reads.
        if state.spot_dodge.is_some() || state.rule_engine.is_some() || oos_enabled {
            let known_statuses = reads.known_statuses();
            state.oos.observe_statuses(&known_statuses);
            if let Some(spot) = state.spot_dodge.as_mut() {
                spot.observe_statuses(&known_statuses);
            }
            if let Some(engine) = state.rule_engine.as_mut() {
                engine.observe_statuses(&known_statuses);
            }
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
        // The preceding mapper delivered the final release. Drop completed
        // locks before proposal generation, not after this frame's candidates
        // have already been rejected by allows(). This adds no release frame.
        if !state.rule_engine.as_ref().is_some_and(|e| e.committed()) {
            state.action_owner.release(Owner::RollRead);
        }
        if state.oos.target().is_none() {
            state.action_owner.release(Owner::OutOfShield);
        }
        // Establish priority before sampling policies: a previously committed
        // general read must not prevent same-observation ledge acquisition.
        // Keep watches fresh so static guard can acknowledge elapsed native
        // frames, but do not acquire/consume another reaction behind a queued R.
        // Reconcile raw cancellation with current native/manual eligibility.
        // A cancelled unshielded response must not add a frame to a legal OOS grab.
        // Continuation uses the committed snapshot. Once a completed owner is
        // released above, new admission must see the current physical intent.
        let stationary_intent = state
            .action_owner
            .policy_inputs()
            .stationary_assist(assessment.status);
        let grab_intent = state
            .action_owner
            .policy_inputs()
            .grounded_grab_assist(assessment.status);
        let self_shielded = state.action_owner.policy_inputs().shield_held()
            || matches!(
                assessment.status,
                kirby_assist_core::native_states::GUARD_ON
                    ..=kirby_assist_core::native_states::GUARD_DAMAGE
            );
        let admission_suspended = automatic_suspended
            || state.ledge_active
            || state
                .settings_menu
                .as_ref()
                .is_some_and(|m| m.reserves_automatic_modes());
        state
            .action_owner
            .set_automatic_suspended(admission_suspended);
        let receipt_blocked =
            state.grab_handoff.pending() && state.action_owner.current_owner().is_none();
        let automatic_suspended = automatic_suspended || receipt_blocked;
        let ledge_priority = state.ledge_active
            && observe_ledge
            && state
                .ledge
                .as_ref()
                .map_or(ledge_opponent.is_some(), |trap| {
                    trap.reserves_response(ledge_opponent)
                });
        state.action_owner.set_ledge_priority(ledge_priority);
        #[cfg(feature = "diagnostic-logging")]
        {
            first_button_write = state.buttons_modified && !state.button_mod_logged;
            state.button_mod_logged |= first_button_write;
            acquired = state.observed_object != Some(object_id);
        }
        #[cfg(feature = "diagnostic-logging")]
        let combo_performance = crate::performance::Scope::new(crate::performance::Part::Combo);
        let fair_allowed = state.action_owner.allows(Owner::Fair);
        state.fair_output = kirby_followups::fair(
            &mut state.fair_followup,
            kirby_followups::is_kirby(fighter_kind) && config.kirby_fair1_smash,
            !automatic_suspended && !clatter && !tech && fair_allowed,
            fair_context,
            combo_velocity_y,
        );
        // An incoming-grab defense is sampled once before new offensive
        // choices. Existing committed owners remain protected by allows().
        state.spot_output = if let Some((context, targets)) = spot_observation {
            let allowed = !automatic_suspended
                && !clatter
                && !tech
                && !own_capture_link
                && stationary_intent
                && state.action_owner.allows(Owner::GrabDodge);
            let (_previous_target, output, _current_target) = {
                let State {
                    spot_dodge,
                    action_owner,
                    ..
                } = &mut *state;
                let spot = spot_dodge.get_or_insert_with(Default::default);
                let _previous_target = spot.target();
                let dodge = spot.sample_with_shape(
                    allowed,
                    context,
                    targets,
                    dodge_lookahead,
                    analysis_settings
                        .value(kirby_assist_core::fighter_tuning::GRAB_DODGE_PADDING, 2.),
                    analysis_settings
                        .value(kirby_assist_core::fighter_tuning::GRAB_DODGE_HEIGHT, 8.),
                );

                // Native dodge startup or a cancelled warning releases its
                // defense-only lock before the ordinary follow-up is sampled.
                if spot.target().is_none() {
                    if spot.punish_pending() {
                        action_owner.handoff(Owner::GrabDodge, Owner::OptionRead);
                    } else {
                        action_owner.release(Owner::GrabDodge);
                    }
                }
                let output = dodge;
                let target = spot.target();
                (_previous_target, output, target)
            };
            #[cfg(feature = "diagnostic-logging")]
            if output.requested != 0 && _previous_target.is_none() {
                log!(&format!("AUTO SPOT DODGE proposed: ordinary grab {:?} is within the configured timing/reach forecast. Down+Guard requested; protected ledge/safety priority and native readiness still apply. Native avoidance is unverified.", _current_target));
            } else if _previous_target.is_some()
                && context.status == kirby_assist_core::spot_dodge::ESCAPE
            {
                log!("AUTO SPOT DODGE: native spot-dodge startup observed; released input. Grab avoidance remains unverified.");
            } else if _previous_target.is_some() && output.owned == 0 {
                log!("AUTO SPOT DODGE released without native startup evidence: threat/readiness scope changed, delivery expired, or higher priority prevented it. Avoidance is unverified.");
            }
            output
        } else {
            if let Some(spot) = state.spot_dodge.as_mut() {
                spot.cancel_pending();
                if !grab_dodge_enabled || automatic_suspended || clatter || tech {
                    spot.suspend_response();
                }
            }
            ControllerOutput::default()
        };
        // A confirmed own dodge releases Down/Guard before a single Up-B plan.
        // Native-confirmed dodge recovery retains the chained response owner;
        // its neutral wait cannot expose manual inputs or a competing punish.
        let punish_allowed = grab_dodge_enabled
            && !automatic_suspended
            && !own_capture_link
            && !clatter
            && !tech
            && !state.oos.grab_delivery_pending()
            && state.action_owner.allows(Owner::OptionRead);
        if let Some((context, _, follow)) = rule_observation {
            let State {
                spot_dodge,
                settings_menu,
                action_owner,
                spot_output,
                ..
            } = &mut *state;
            if let Some(spot) = spot_dodge.as_mut() {
                let sequence = settings_menu.as_ref().and_then(|m| m.grab_dodge_sequence());
                let context = if dodge_direct_up_b {
                    context
                } else {
                    GameplayContext {
                        can_act: dodge_response_ready,
                        ..context
                    }
                };
                let punish =
                    spot.punish_automatic(punish_allowed, sequence, config.assist, context, follow);
                if punish.owned != 0 {
                    *spot_output = punish;
                }
                if !spot.punish_pending() && punish.owned == 0 {
                    action_owner.release(Owner::OptionRead);
                }
            }
        } else if let Some(spot) = state.spot_dodge.as_mut() {
            spot.suspend_response();
        }
        // An immediately usable Up-B may replace an unaccepted automatic grab.
        // Admission and this handoff share the core selector; cached Grab input
        // is cleared before the winning reaction can reach the mapper.
        let preempt_grab = |state: &mut State| {
            let prioritize_up_b = (state.action_owner.allows(Owner::RollRead)
                || state.oos.approach_active()
                    && state
                        .action_owner
                        .allows_after_release(Owner::RollRead, Owner::OutOfShield))
                && rules_on
                && !automatic_suspended
                && !clatter
                && !tech
                && !file_ledge_timeline
                && !ledge_priority
                && state.spot_output.owned == 0;
            if prioritize_up_b
                && state.oos.automatic_grab_pending()
                && !state.oos.grab_delivery_pending()
            {
                if let Some((context, targets, follow)) = rule_observation {
                    let yielded = {
                        let State {
                            rule_engine,
                            settings_menu,
                            oos,
                            action_owner,
                            ..
                        } = state;
                        if let Some(rules) = settings_menu.as_ref().and_then(|m| m.rule_config()) {
                            let engine = rule_engine.get_or_insert_with(Box::default);
                            engine.preempt_automatic_grab(
                                true,
                                rules,
                                context,
                                &targets,
                                follow,
                                oos,
                                action_owner,
                            )
                        } else {
                            false
                        }
                    };
                    if yielded {
                        state.oos_output = ControllerOutput::default();
                    }
                }
            }
        };
        preempt_grab(&mut state);
        // Sample remaining pending grabs once so accepted/failed attempts
        // release before fallback. New reactions precede NEW grabs.
        let shield_fallback_pending = state.oos.shield_up_b_pending();
        let grab_pending = state.oos.target().is_some();
        let sample_grab = |state: &mut State| {
            let oos_reserved = automatic_suspended
                || state.action_owner.current_owner() != Some(Owner::OutOfShield)
                    && (ledge_priority || file_ledge_timeline);
            let protected_up_b = state.action_owner.current_owner() == Some(Owner::OutOfShield);
            let oos_allowed = oos_enabled
                && !own_capture_link
                && (automatic_scope.acquire
                    || automatic_scope.follow
                    || state.oos.configured_committed())
                && (grab_intent || state.oos.configured_committed() || protected_up_b)
                && !oos_reserved
                && state.spot_output.owned == 0
                && state.roll_read_output.owned == 0
                && !clatter
                && !tech
                && state.action_owner.allows(Owner::OutOfShield)
                && !state.rule_engine.as_ref().is_some_and(|r| r.committed());
            state.oos_output = if let Some(context) = oos_context {
                let (lc, targets) = rule_observation
                    .map(|(c, t, _)| (c, t))
                    .unwrap_or((GameplayContext::default(), [None; 8]));
                let State {
                    oos, settings_menu, ..
                } = state;
                let sequences = settings_menu
                    .as_ref()
                    .map_or(Default::default(), |m| m.automatic_sequences());
                oos.sample_automatic_configured(
                    oos_allowed,
                    context,
                    lc,
                    targets,
                    oos_follow,
                    self_shielded,
                    legacy_grab_close,
                    legacy_grab_approach,
                    Some(f32::from(if self_shielded {
                        config.assist.fighter.grab.shield_startup
                    } else {
                        config.assist.fighter.grab.dash_startup
                    })),
                    config.assist,
                    sequences,
                )
            } else {
                state.oos = Default::default();
                ControllerOutput::default()
            };
            if state.oos.target().is_none() && state.oos_output.owned == 0 {
                state.action_owner.release(Owner::OutOfShield);
            }
        };
        if grab_pending {
            sample_grab(&mut state);
        }
        // A cancelled shield fallback may have selected a fresh front grab.
        // Apply the same roll priority before either new packet is emitted.
        if shield_fallback_pending
            && !state.oos.shield_up_b_pending()
            && state.oos.automatic_grab_pending()
        {
            // The old fallback ended; the replacement is a new proposal and
            // must compete under current admission priority, not inherit a lock.
            state.action_owner.release(Owner::OutOfShield);
            preempt_grab(&mut state);
        }
        if !rules_on {
            if automatic_suspended {
                if let Some(e) = state.rule_engine.as_mut() {
                    e.cancel();
                }
            } else {
                state.rule_engine = None;
            }
        }
        state.roll_read_output = if let Some((context, targets, follow)) = rule_observation {
            let allowed = rules_on
                && !automatic_suspended
                && !own_capture_link
                && !clatter
                && !tech
                && (state.action_owner.current_owner() == Some(Owner::RollRead)
                    || !file_ledge_timeline && !ledge_priority)
                && state.spot_output.owned == 0
                && !state.oos.grab_delivery_pending()
                && state.action_owner.allows(Owner::RollRead);
            let state = &mut *state;
            let rules = rules_on
                .then(|| state.settings_menu.as_ref().and_then(|m| m.rule_config()))
                .flatten();
            kirby_assist_core::reaction_rules::sample_optional_simple(
                &mut state.rule_engine,
                allowed,
                rules,
                config.assist,
                context,
                targets,
                follow,
                simple_readiness,
            )
        } else {
            if let Some(e) = state.rule_engine.as_mut() {
                e.cancel();
            }
            ControllerOutput::default()
        };
        if !grab_pending {
            sample_grab(&mut state);
        }
        #[cfg(feature = "diagnostic-logging")]
        drop(combo_performance);
        let trip_allowed = state.action_owner.allows(Owner::Trip);
        state.trip_output = kirby_followups::trip(
            &mut state.trip_followup,
            trip_kirby,
            !clatter && !tech && trip_allowed,
            trip_context,
        );
        #[cfg(feature = "diagnostic-logging")]
        {
            let key = (
                kirby_followups::watching(&state.trip_followup),
                sdi_context.damage_stop && sdi_context.remaining <= 120,
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
        state.di_context = di_context;
        state.damage_geometry = damage_geometry;
        state.damage_trajectory = damage_trajectory;
        state.observed_object = Some(object_id);
        state.evaluated_tick = Some(tick);
        // Tech needs the last GAME-visible ZR to decide whether a release is
        // required. The restored physical trace may differ from that last output.
        let previous_buttons =
            if state.last_output_buttons & config.control_layout.physical_buttons(MAPPER_ZR) != 0 {
                HID_ZR
            } else {
                0
            };
        #[cfg(feature = "diagnostic-logging")]
        if config.controls.tech_debug {
            let relevant = (kirby_assist_core::native_states::status::DAMAGE
                ..=kirby_assist_core::native_states::status::DOWN_REFLECT_LR)
                .contains(&assessment.status)
                || (kirby_assist_core::native_states::status::PASSIVE
                    ..=kirby_assist_core::native_states::status::PASSIVE_CEIL)
                    .contains(&assessment.status)
                || (kirby_assist_core::native_states::status::CATCHED_GANON
                    ..=kirby_assist_core::native_states::status::CATCHED_AIR_END_GANON)
                    .contains(&assessment.status);
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
            .settings_menu
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
            .sample_with_native_delivery(
                config.controls.enabled && auto_tech_enabled,
                true,
                tech_timing.0,
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
                    prepare_guard: !clatter && assessment.prepare_guard,
                },
                assessment.reaction_remaining,
                tech_timing.1,
                tech_timing.2,
                native_tech_guard,
            );
        state.tech_urgent = state
            .tech_input
            .as_ref()
            .is_some_and(TechInput::urgent_press);
        let _requested = mash_requested | if state.tech_output.press { HID_ZR } else { 0 };
        #[cfg(feature = "diagnostic-logging")]
        if config.controls.tech_debug {
            let cooldown = state
                .tech_input
                .as_ref()
                .map_or(0.0, TechInput::cooldown_remaining);
            if let Some(message) = state.pending_tech_log.as_mut() {
                message.push_str(&format!(", requested={_requested:#x}, cooldown={cooldown}"));
            }
        }
        // Cap grounded physical inputs during recovery too, preventing a held
        // full stick from buffering a dash as the attack finishes. Damage/air
        // movement and higher-priority mash/tech keep their physical inputs.
        state.ledge_walk_ready = !clatter
            && !tech
            && assessment.situation == kirby_assist_core::native_states::situation::GROUND
            && (0..=kirby_assist_core::native_states::THROW).contains(&assessment.status);
        #[cfg(feature = "diagnostic-logging")]
        {
            state.mode_hit = ledge_observation
                .as_ref()
                .map(|(context, _, _, _)| context.hit_confirmed);
        }
        state.manual_context = Some(manual_context);
        state.posture_target = posture_target;
        state.posture_floor_safe = posture_floor_safe;
        state.turbo_ready = !clatter
            && !tech
            && turbo_movement_allowed(assessment.status, assessment.situation, stopped);

        if let Some((mut context, opponent, edge, spot)) = ledge_observation {
            state.ledge_walk_motion = state
                .ledge_walk_ready
                .then_some((context.position, context.velocity));
            let previewing = state
                .settings_menu
                .as_ref()
                .is_some_and(|m| m.preview_slot().is_some());
            context.input_delay = 0.0;
            context.can_act = state.turbo_ready;
            let trump_held = state
                .settings_menu
                .as_ref()
                .is_some_and(|menu| menu.ledge_trump_held());
            if !trump_held {
                state.trump_seek_tick = None;
            } else if find_trump_edge
                && turbo_movement_allowed(context.status, context.situation, stopped)
                && !clatter
                && !tech
            {
                // A failed geometry search must not repeat hundreds of rays
                // on every held-button poll. A new press still searches now.
                state.trump_seek_tick = Some(tick);
            }
            let trump_allowed = state.action_owner.allows(Owner::Trump);
            state.trump_output = state.trump.get_or_insert_with(LedgeTrump::default).sample(
                trump_allowed,
                tick,
                trump_held,
                GameplayContext {
                    gameplay: !clatter && !tech,
                    ..context
                },
                edge,
            );
            let held = state.ledge_active && state.action_owner.allows(Owner::Ledge);
            // Preparation stays grounded. Continue only a policy-started hop and
            // keep ownership through attack/shield/grab animations; mash/tech keep their existing priority.
            // The policy can preserve ZR during native guard hitstop.
            let grounded = assessment.situation
                == kirby_assist_core::native_states::situation::GROUND
                && (state.turbo_ready
                    || matches!(assessment.status, kirby_assist_core::native_states::status::GUARD_ON..=kirby_assist_core::native_states::status::GUARD_DAMAGE | kirby_assist_core::native_states::status::ATTACK_DASH..=kirby_assist_core::native_states::status::ATTACK_LW4 | kirby_assist_core::native_states::status::CATCH..=kirby_assist_core::native_states::status::CATCH_ATTACK));
            let owned_jump = state
                .ledge
                .as_ref()
                .is_some_and(|trap| trap.owns_jump_state(assessment.status));
            let can_control = !stopped && !clatter && !tech && (grounded || owned_jump);
            // can_act permits a new grounded action. can_control also covers
            // our ongoing shield/tilt/dash attack/grab, allowing ownership without repeatedly
            // launching actions during recovery. Do not collapse these flags.
            context.held = held;
            context.can_act = state.turbo_ready;
            context.can_control = can_control;
            if context.situation != kirby_assist_core::native_states::situation::GROUND {
                state.ledge_position = None;
                state.ledge_seek_tick = None;
            }
            if held
                && !previewing
                && context.situation == kirby_assist_core::native_states::situation::GROUND
                && config.assist.auto_position
                && !config.assist.manual_spacing
                && !config.assist.distance_agnostic
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
                    .filter(|_| !config.assist.manual_spacing && !config.assist.distance_agnostic)
                {
                    state
                        .ledge
                        .get_or_insert_with(LedgeTrap::default)
                        .prepare_stance(spot.edge, spot.inward);
                }
            }
            state.ledge_output = {
                let state = &mut *state;
                let before = state.ledge_visits.visit_epoch();
                let output = state
                    .ledge
                    .get_or_insert_with(LedgeTrap::default)
                    .sample_with_visits(
                        &mut state.ledge_visits,
                        config.assist,
                        tick,
                        context,
                        opponent,
                    );
                if state.ledge_visits.visit_epoch() != before {
                    // A new ledge visit invalidates the previous approach shield
                    // tail, even when our stationary guard clock cannot advance.
                    state.guard_hold = None;
                    state.autopilot_guarding = false;
                }
                output
            };
            // Completion hands ownership back before the general reaction pass.
            // Keep the final emitted input protected until its frame is delivered.
            let reserve = state.ledge_active
                && state.ledge.as_ref().is_some_and(|trap| {
                    trap.needs_response_observation() || state.ledge_output.owned != 0
                });
            state.action_owner.set_ledge_priority(reserve);
            if !reserve {
                state.action_owner.release(Owner::Ledge);
            }
            if previewing
                && state
                    .ledge
                    .as_ref()
                    .is_some_and(LedgeTrap::manual_after_response)
            {
                if let Some(menu) = state.settings_menu.as_mut() {
                    menu.finish_preview();
                }
            }
            // Keep observing options during approach. An actionable opponent
            // response takes priority immediately; hanging/no-target uses the
            // locked, damped preparation controller instead of root chasing.
            #[cfg(feature = "diagnostic-logging")]
            let mut positioning_action = None;
            if held
                && !previewing
                && !state
                    .ledge
                    .as_ref()
                    .is_some_and(LedgeTrap::file_timeline_active)
                && !state
                    .ledge
                    .as_ref()
                    .is_some_and(LedgeTrap::manual_after_response)
                && !config.assist.manual_spacing
                && !config.assist.distance_agnostic
                && context.situation == kirby_assist_core::native_states::situation::GROUND
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
                        || opponent.is_some_and(|e| {
                            (CLIFF_CATCH_MOVE
                                ..=kirby_assist_core::native_states::status::CLIFF_WAIT)
                                .contains(&e.status)
                        }));
                if preparing {
                    let movement = state
                        .ledge_position
                        .get_or_insert_with(LedgePosition::default)
                        .sample(tick, context, config.assist);
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
            // Guard all incoming options while watching/after a response.
            // Committed authored timelines keep their literal controls.
            // Never replace a requested punish input, committed move, capture
            // handback, trump, or a higher-priority mash/tech response.
            let passive_plan = state
                .ledge
                .as_ref()
                .is_none_or(|trap| !trap.committed_input_pending());
            let defense_eligible = state.action_owner.allows(Owner::Defense)
                && held
                && config.autopilot_shield
                && passive_plan
                && context.situation == kirby_assist_core::native_states::situation::GROUND
                && !clatter
                && !tech
                && state.ledge_output.requested == 0
                && state.roll_read_output.owned == 0
                && state.trump_output.owned == 0;
            let guard_eligible = defense_eligible && guard_ready;
            let defensive_guard = state
                .guard_hold
                .get_or_insert_with(kirby_assist_core::autopilot_guard::GuardHold::default)
                .sample_with_progress(
                    guard_eligible,
                    guard_threat,
                    kirby_assist_core::autopilot_guard::GuardClock {
                        status: context.status,
                        motion: context.motion_kind,
                        frame: context.motion_frame,
                        rate: context.motion_rate,
                        stopped,
                    },
                    analysis_settings.value(kirby_assist_core::fighter_tuning::SHIELD_HOLD, 4.),
                    if guard_eligible
                        && matches!(
                            context.status,
                            kirby_assist_core::native_states::GUARD_ON
                                | kirby_assist_core::native_states::GUARD
                        )
                    {
                        reads.known_guard_clock(object_id)
                    } else {
                        None
                    },
                );
            state.autopilot_guarding = defensive_guard;
            if defensive_guard {
                state.ledge_output = ControllerOutput {
                    consume_trigger: true,
                    requested: MAPPER_ZR,
                    owned: kirby_assist_core::native_input::ASSIST_BUTTONS,
                    stick: Some([0.0, 0.0]),
                    native_short_hop: false,
                };
            }
            #[cfg(feature = "diagnostic-logging")]
            {
                let state = &mut *state;
                let metadata = state.ledge.as_ref().and_then(|trap| {
                    let slot = trap.calibration_slot()?;
                    let profile = PROFILES.get()?.profile(state.fighter_kind?)?;
                    let (id, name) = state
                        .settings_menu
                        .as_ref()?
                        .selected_choice_labels(fighter_kind, slot)?;
                    let mut frame = trap.calibration_frame(config.assist, context, opponent, id)?;
                    frame.output = state.ledge_output;
                    frame.native_attack_active = neutral_native_activity;
                    Some((
                        frame,
                        profile.name,
                        kirby_assist_core::diagnostics::modes::Label::new(
                            kirby_assist_core::fighter_profiles::spacing::menu_name(name),
                        ),
                    ))
                });
                if held {
                    if let Some((frame, fighter, choice)) = metadata {
                        state.calibration.observe(
                            Some(frame),
                            (fighter, fighter_kind),
                            choice.as_str(),
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
            let grab_snapshot = if held {
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
                    action: if state.autopilot_guarding {
                        "autopilot_defensive_shield"
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
                            "target_observed_velocity={:?}, file_options={:?}, ",
                            "locked_ledge={:?}, stance_distance={}"
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
                        config.assist.sequence_overrides.iter().any(Option::is_some),
                        state.ledge_position.as_ref().and_then(LedgePosition::spot),
                        config.assist.stand_distance,
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
            state.autopilot_guarding = false;
            state.guard_hold = None;
            state.ledge_output = ControllerOutput::default();
            state.trump_output = ControllerOutput::default();
            state.trump = None; // Clear released Down history without native reads.
            state.trump_seek_tick = None;
        }
        state.reflect_output = if let Some((configured, context, threat)) = reflect_observation {
            // Preparation happens after existing modes. Never admit two new
            // punish inputs on the same observation, nor steal their receipt.
            let allowed = (!state.ledge_active
                || state.action_owner.current_owner() == Some(Owner::Reflect))
                && state.action_owner.allows(Owner::Reflect)
                && !clatter
                && !tech
                && state.oos_output.owned == 0
                && !state.oos.grab_delivery_pending()
                && state.roll_read_output.owned == 0
                && state.spot_output.owned == 0
                && state.trip_output.owned == 0
                && state.fair_output.owned == 0
                && state.trump_output.owned == 0;
            state
                .reflector
                .sample(allowed, Some(configured), context, threat)
        } else {
            state.reflector.cancel();
            ControllerOutput::default()
        };
        state.mash_stick = if policy_tick.is_none() {
            None
        } else {
            state.policy.as_ref().and_then(RawAutoControls::mash_stick)
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

// Publish only the existing box pointer on the small native loader stack.
// Expanded button slots must not create config-sized OnceLock move frames.
#[inline(never)]
fn publish_startup_config(config: Box<Config>) {
    let _ = CONFIG.set(config);
}

// Startup is deliberately sequential: prepare diagnostics, parse config, locate
// exactly one supported mapper, resolve the manager, THEN install hooks. A failed
// prerequisite exits without installing a partially configured set of hooks.
pub fn install() {
    // Silent builds only READ configuration: no directory/file creation or
    // console output is compiled into this plugin's normal startup path.
    #[cfg(feature = "diagnostic-logging")]
    crate::logging::start();
    if let Err(_error) = kirby_assist_core::fighter_data::initialize() {
        log!(&format!("NOT INSTALLED: {_error}"));
        return;
    }
    let (config, profiles, sequences) = match load_config() {
        Ok(settings) => settings,
        Err(_error) => {
            log!(&format!("NOT INSTALLED: {_error}"));
            return;
        }
    };
    #[cfg(feature = "diagnostic-logging")]
    log!(&format!("GLOBAL MODES: assistance={}, auto-tech={}, auto-SDI={}, automatic fair-hit follow-up={}, defensive ledge shield={}. Saved fighter choices are listed separately when selected. Detailed snapshots are batched at most 4 times/second; input requests, native move starts and hit evidence are reported separately.", config.controls.enabled, config.controls.auto_tech, config.auto_sdi, config.kirby_fair1_smash, config.autopilot_shield));
    #[cfg(feature = "diagnostic-logging")]
    log!(&format!("AUTO DI CONFIG: enabled={}, mode={:?}, legacy threshold={:.2}. Adaptive uses survival; Escape requires explicit selection. DI owns the final 3 victim-hitstop frames; SDI never falls back there. DI and SDI share exclusive stick ownership through hitstop.", config.auto_di.enabled, config.auto_di.mode, config.auto_di.survival_speed));
    log!(&format!("AUTO SPOT DODGE: enabled={}; ordinary standing/dash/pivot grabs only, bounded per-fighter proximity forecast, one accepted request per grab with optional configured follow-up; incoming-grab defense waits for committed sequences and yields to native safety.", config.auto_spot_dodge));
    publish_startup_config(config);
    let Some(config) = CONFIG.get() else {
        return;
    };
    let _ = PROFILES.set(profiles);
    log!(&format!(
        "SEQUENCE FILE: {} validated override(s) loaded once at startup.",
        sequences.len()
    ));
    let _ = SEQUENCES.set(Mutex::new(sequences));
    if !config.controls.enabled {
        log!("disabled in config; no hooks installed");
        return;
    }
    // Parse static grab reference facts before any gameplay hooks can run.
    if let Err(_error) = kirby_assist_core::fighter_data::prepare_grab_references() {
        log!(&format!("NOT INSTALLED: {_error}"));
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
        let Some(offset) = mapper_offset(text) else {
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
        TRAINING_MODPACK_PRESENT.store(
            std::path::Path::new("sd:/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libtraining_modpack.nro").is_file(),
            Ordering::Relaxed,
        );
        if TRAINING_MODPACK_PRESENT.load(Ordering::Relaxed) {
            log!("Stock Training Modpack detected: Minus toggles Kirby Assist menu; full gameplay tools enabled. Training shortcuts hand off until the next Minus press");
        }
        skyline::install_hook!(observe_commands);
        settings_menu::install();
        log!(&format!("observer installed; mapper={offset:#x} deferred until fighter callback; training_modpack_present={}; config={config:?}; physical-controller routing; pre-map fighter refresh; callback-only fighter discovery", TRAINING_MODPACK_PRESENT.load(Ordering::Relaxed)));
    }
}
