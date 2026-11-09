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
//! Extension points: add game observations in `observation::observe_fighter`,
//! logical tools in the core, and verified button translations in
//! `native_input.rs`. Defining a Scenario alone does not make the live
//! integration detect or output it.

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

// Keep hook/session orchestration here; native layouts and read-only queries
// live in children. `pub(super)` exposes a helper only to this module's family.
// Keep these child declarations together so the integration's file layout is
// visible before its imports and runtime implementation.
mod attack_sources;
mod config;
mod controller;
mod damage;
mod frame;
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
mod mapper;
mod observation;
mod observations;
mod reflect;
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
// Only a freshly revalidated selected-controller route may authorize gameplay,
// UI telemetry or projectile tracking. Native ownership discovery stays read-only.
static LOCAL_PARTICIPANT: AtomicBool = AtomicBool::new(false);
pub(crate) use runtime::feature_activity_allowed;
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
    let Some(mapper::MapperAdmission {
        config,
        master_ready_go,
        session_ticket,
    }) = mapper::admit_mapper_callback(wrapper, index)
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
    if training_test_mode && mapper::enter_training_passthrough(wrapper, index, config) {
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
    let Some(mut preparation) = mapper::prepare_mapper_overlay(
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
    mapper::finish_mapper_overlay(output, index, session_ticket, preparation);
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
        observation::observe_fighter(object, true);
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
