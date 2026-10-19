//! Switch integration: observe fighter state on the game thread, publish button
//! requests, then overlay raw Pro Controller memory before mapping. No status writes.
//!
//! One update, in order:
//! 1. The mapper captures fresh physical buttons and identifies its controller.
//! 2. For an associated Kirby, resolve its accessor and observe current state.
//! 3. The core chooses a left-stick mash direction or a ZR tech request.
//! 4. Overlay raw held/previous/pressed/released words, call the original mapper,
//!    then restore the physical words so our output cannot feed back as input.
//!
//! Extension points: add game observations in `observe_fighter`, logical tools
//! in the core, and verified button translations in `native_input.rs`. Defining
//! a Scenario alone does not make the live integration detect or output it.
use crate::game_ffi as game;
use kirby_assist_core::ledge_position::{nearest_ground_ledge, LedgePosition};
use kirby_assist_core::ledge_trap::{
    frames_until, grab_transition_term, in_tracking_range, LedgeContext, LedgeOutput, LedgeTrap,
    MotionTiming, Opponent, CLIFF_CATCH_MOVE, CLIFF_JUMP3, DPAD_DOWN, LEDGE_TRIGGER,
};
use kirby_assist_core::ledge_trump::{nearest_floor_edge, LedgeTrump};
use kirby_assist_core::live_input::{
    unique_instruction_offset, LiveConfig as Config, RawAutoControls, ASSIST_BUTTONS, HID_ZR,
};
use kirby_assist_core::manual_controls::{
    suppress_reserved_taunts, turbo_movement_allowed, DpadReservation, ManualOverlay, TurboState,
    DPAD_SIDES,
};
use kirby_assist_core::native_input::{
    controller_index, ButtonTrace, MapperButtonOverlay, MAPPER_MENU, MAPPER_ZR,
};
use kirby_assist_core::tech_prediction::{tech_gate, tech_probes, TechGate};
use skyline::hooks::{getRegionAddress, Region};
use std::{
    ffi::c_void,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex, OnceLock,
    },
    time::{Duration, Instant},
};

// These are Switch SD-card paths, not paths on the Mac used to compile the mod.
// Config is read once at startup; changing it requires restarting Smash.
#[cfg(feature = "diagnostic-logging")]
use std::sync::atomic::AtomicBool;

const CONFIG_PATH: &str = "sd:/ultimate/kirby_assist/config.txt";
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
static UNSUPPORTED_FIGHTER_LOGGED: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "diagnostic-logging")]
static ROUTE_UNAVAILABLE_LOGGED: AtomicBool = AtomicBool::new(false);
static MANAGER_SLOT: AtomicUsize = AtomicUsize::new(0);
static CONFIG: OnceLock<Config> = OnceLock::new();
// STATE.lock() yields a guard. Rust releases the lock when the guard leaves
// its scope (`}`); explicit small scopes below let us copy values, unlock, query
// the game, then lock again to publish results. `if let Ok` skips poisoned locks
// instead of panicking inside a native callback.
static STATE: Mutex<State> = Mutex::new(State::new());
// Wall-clock expiry prevents a request surviving indefinitely when observation
// stops. This is a watchdog duration, not the game's tech input window.
const REQUEST_LIFETIME: Duration = Duration::from_millis(35);
// Pinned Controller prefix: overlay button words, stick floats, and affected
// repeat flags as requested by the active tool. Restore them after mapping.
#[repr(C)]
// Padding fields are intentional: they preserve offsets of later native fields.
// Do not remove padding or reorder fields as a normal Rust refactor would.
struct Controller {
    vtable: usize,
    current_buttons: u32,
    previous_buttons: u32,
    left_stick_x: f32,
    left_stick_y: f32,
    left_trigger: f32,
    left_padding: u32,
    right_stick_x: f32,
    right_stick_y: f32,
    // Rest of the native axes/gyro/repeat prefix remains untouched.
    sticks_gyro_repeat: [u8; 0x60],
    just_down: u32,
    just_release: u32,
    autorepeat: [u32; 3],
    style: u32,
    controller_id: u32,
    colors_and_flags: [u8; 0x17],
    connected: u8,
    trailing_flags: [u8; 8],
    npad: u32,
}
// Compile-time checks catch mistakes in OUR declared layout. They cannot prove
// that an unknown/new game executable uses that same layout.
const _: () = {
    assert!(std::mem::offset_of!(Controller, current_buttons) == 8);
    assert!(std::mem::offset_of!(Controller, previous_buttons) == 0xc);
    assert!(std::mem::offset_of!(Controller, left_stick_x) == 0x10);
    assert!(std::mem::offset_of!(Controller, left_stick_y) == 0x14);
    assert!(std::mem::offset_of!(Controller, right_stick_x) == 0x20);
    assert!(std::mem::offset_of!(Controller, right_stick_y) == 0x24);
    assert!(std::mem::offset_of!(Controller, autorepeat) == 0x90);
    assert!(std::mem::offset_of!(Controller, just_down) == 0x88);
    assert!(std::mem::offset_of!(Controller, just_release) == 0x8c);
    assert!(std::mem::offset_of!(Controller, style) == 0x9c);
    assert!(std::mem::offset_of!(Controller, connected) == 0xbb);
    assert!(std::mem::offset_of!(Controller, npad) == 0xc4);
};

// Pinned MappedInputs result (common/input.rs). The post-map filter clears
// reserved taunts which may otherwise survive native repeat/history handling.
#[repr(C)]
struct MappedInputs {
    buttons: u32,
    left_x: i8,
    left_y: i8,
    right_x: i8,
    right_y: i8,
}
const _: () = {
    assert!(std::mem::size_of::<MappedInputs>() == 8);
    assert!(std::mem::offset_of!(MappedInputs, buttons) == 0);
};

// Copy the four physical button words before changing any of them.
unsafe fn trace(controller: *const Controller) -> ButtonTrace {
    ButtonTrace {
        held: (*controller).current_buttons,
        previous: (*controller).previous_buttons,
        pressed: (*controller).just_down,
        released: (*controller).just_release,
    }
}
// Write a complete, consistent button history; changing only held would leave
// new-press/release queries inconsistent with what the mapper sees.
unsafe fn write_trace(controller: *mut Controller, buttons: ButtonTrace) {
    (*controller).current_buttons = buttons.held;
    (*controller).previous_buttons = buttons.previous;
    (*controller).just_down = buttons.pressed;
    (*controller).just_release = buttons.released;
}

// Missing config uses Rust defaults; malformed config is an error, not a reason
// to silently enable a tool with settings the user did not intend.
fn load_config() -> Result<Config, String> {
    match std::fs::read_to_string(CONFIG_PATH) {
        Ok(contents) => Config::parse(&contents),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(error) => Err(error.to_string()),
    }
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
    expected_grab_status: Option<i32>,
    hit_confirmed: bool,
    enemy_frame: i32,
    requested: u32,
    action: &'static str,
}

// One session for the configured controller, rather than one global session per
// fighter. `cancel` clears assistance without changing the game's native state.
struct State {
    // Routing: npad from config selects the controller; mapping_index is the
    // game's input slot for it. Neither number is a fighter's battle-object ID.
    mapping_index: Option<i32>,
    // Our mapper-call counter, NOT a verified simulation-frame or HID clock.
    raw_tick: i64,
    raw_seen: Option<Instant>,
    raw_buttons: u32,
    raw_connected: bool,
    // Option permits lazy construction and a clean reset by assigning None.
    policy: Option<RawAutoControls>,
    overlay: Option<MapperButtonOverlay>,
    // Separate state for side-D-pad consumption and temporary left-stick control.
    turbo: Option<TurboState>,
    dpad_reservation: Option<DpadReservation>,
    manual_overlay: Option<ManualOverlay>,
    // Observation-derived eligibility, not just whether a trigger is held.
    // Turbo/ledge must also respect grounded movement and native busy states.
    turbo_ready: bool,
    ledge: Option<LedgeTrap>,
    ledge_active: bool,
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
    // Repeated game queries within one observed mapper update must not mash
    // faster, consume cooldown faster, or issue an extra tech press.
    evaluated_tick: Option<i64>,
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
    last_ledge_diagnostic: Option<LedgeDiagnostic>,
    #[cfg(feature = "diagnostic-logging")]
    pending_ledge_log: Option<String>,
}
impl State {
    // `const fn` allows the global Mutex to be initialized before startup.
    // Options begin as None; get_or_insert_with in the callbacks constructs
    // each helper only when first needed. No game pointer is created here.
    const fn new() -> Self {
        Self {
            mapping_index: None,
            raw_tick: 0,
            raw_seen: None,
            raw_buttons: 0,
            raw_connected: false,
            policy: None,
            overlay: None,
            turbo: None,
            dpad_reservation: None,
            manual_overlay: None,
            turbo_ready: false,
            ledge: None,
            ledge_active: false,
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
            evaluated_tick: None,
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
            last_ledge_diagnostic: None,
            #[cfg(feature = "diagnostic-logging")]
            pending_ledge_log: None,
        }
    }
    // Use for actual cancellation/reassignment. Hitstop deliberately does NOT
    // call this: resetting there would forget cooldown and input history.
    fn end_session(&mut self) {
        self.cancel();
        self.associated_object = None;
        self.mapping_index = None;
        self.raw_connected = false;
        self.raw_seen = None;
        self.raw_buttons = 0;
    }

    fn cancel(&mut self) {
        self.requested = 0;
        self.mash_stick = None;
        self.owned = 0;
        self.request_seen = None;
        self.observed_object = None;
        self.evaluated_tick = None;
        self.policy = None;
        self.overlay = None;
        self.turbo = None;
        self.dpad_reservation = None;
        self.manual_overlay = None;
        self.turbo_ready = false;
        self.ledge = None;
        self.ledge_active = false;
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
            self.last_ledge_diagnostic = None;
            self.pending_ledge_log = None;
        }
    }
}

// Gate BEFORE dereferencing a controller wrapper, enumerating fighters or
// following a fighter's ControlModule chain. Intro/connection/menu callbacks
// also invoke the mapper, but do not establish valid gameplay module lifetimes.
unsafe fn gameplay_ready() -> bool {
    if !game::is_ready_go() {
        return false;
    }
    let slot = MANAGER_SLOT.load(Ordering::Relaxed);
    if slot == 0 {
        return false;
    }
    let manager = (slot as *const *mut game::Object).read();
    !manager.is_null() && !game::is_result(manager)
}

fn end_session() {
    if let Ok(mut state) = STATE.lock() {
        state.end_session();
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
    if !gameplay_ready() {
        end_session();
        original!()(mappings, index, output, wrapper, arg);
        return;
    }
    // Keep the untouched trace for restoration after original!(). The original
    // mapper receives the overlay only during this call; output keeps the mapped
    // result while the physical Controller memory is restored afterwards.
    // Treat the overlay like a short transaction:
    // save physical data -> temporarily replace -> map -> restore.
    // Only the mapper's RESULT survives. Retaining our temporary raw data would
    // make the next callback mistake automation for a real physical hold.
    let mut restore = None;
    let mut reserved_dpad = 0;
    #[cfg(feature = "diagnostic-logging")]
    let mut trump_pressed = false;
    #[cfg(feature = "diagnostic-logging")]
    let mut ledge_edge = None;
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
            // Clearing state is not enough: do not fall through to roster
            // discovery or fighter observation after a disconnect/style change.
            if !controller.is_null()
                && (*controller).npad == config.npad
                && ((*controller).style != 5 || (*controller).connected == 0)
            {
                end_session();
                original!()(mappings, index, output, wrapper, arg);
                return;
            }
            #[cfg(feature = "diagnostic-logging")]
            if !controller.is_null()
                && SEEN_MAPPERS.fetch_or(1 << index, Ordering::Relaxed) & (1 << index) == 0
            {
                messages.push(format!("mapper observed index {index}: npad={}, style={}, connected={}; configured npad={}", (*controller).npad, (*controller).style, (*controller).connected, config.npad));
            }
            if !controller.is_null() && (*controller).npad == config.npad {
                // Read physical input before applying any assist. Reusing last
                // output as "human input" would keep automated buttons latched.
                let physical = trace(controller);
                #[cfg(feature = "diagnostic-logging")]
                {
                    trump_pressed = physical.pressed & DPAD_DOWN != 0;
                }
                let physical_stick = [(*controller).left_stick_x, (*controller).left_stick_y];
                let physical_right_stick =
                    [(*controller).right_stick_x, (*controller).right_stick_y];
                let physical_autorepeat = (*controller).autorepeat[0];
                if let Ok(mut state) = STATE.lock() {
                    if (*controller).style != 5 || (*controller).connected == 0 {
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
                        let ledge_pressed = physical.held & LEDGE_TRIGGER != 0
                            && state.raw_buttons & LEDGE_TRIGGER == 0;
                        if config.controls.enabled && config.ledge.enabled && ledge_pressed {
                            state.ledge_active = !state.ledge_active;
                            state.ledge = None;
                            state.ledge_position = None;
                            state.ledge_seek_tick = None;
                            state.ledge_output = LedgeOutput::default();
                        }
                        state.raw_buttons = physical.held;
                        if physical.held & MAPPER_MENU != 0 {
                            state.cancel();
                        }
                    }
                }
                // Refresh the previously associated fighter BEFORE this mapper
                // consumes input, avoiding v2's extra mapping-update delay.
                // Resolve by ID each time; never retain an accessor pointer.
                let associated = STATE.lock().ok().and_then(|state| {
                    (state.mapping_index == Some(index))
                        .then_some(state.associated_object)
                        .flatten()
                });
                // Find Kirby on the first mapper call too. Waiting for a later
                // command callback caused startup latency and a first-frame taunt.
                let associated = associated.or_else(|| selected_fighter(index));
                let mut manual_scope = false;
                if let Some(id) = associated {
                    if game::is_active(id) {
                        let object = game::module_accessor(id);
                        if !object.is_null() {
                            observe_fighter(object);
                            manual_scope = config.controls.enabled
                                && physical.held & MAPPER_MENU == 0
                                && human_fighter(object)
                                && game::fighter_kind(object) == 6
                                && controller_index(object.cast()) == Some(index);
                        }
                    } else if let Ok(mut state) = STATE.lock() {
                        state.cancel();
                        state.associated_object = None;
                    }
                }
                if let Ok(mut state) = STATE.lock() {
                    if state.mapping_index == Some(index)
                        && (*controller).connected != 0
                        && (*controller).style == 5
                    {
                        let now = Instant::now();
                        let fresh = state
                            .request_seen
                            .is_some_and(|seen| now.duration_since(seen) < REQUEST_LIFETIME);
                        // Expired requests become neutral. The overlay still
                        // knows the previous assisted output, allowing a release.
                        let requested = if fresh { state.requested } else { 0 };
                        let owned = if fresh { state.owned } else { 0 };
                        let mash_stick = if fresh { state.mash_stick } else { None };
                        let mash_takeover = mash_stick.is_some() && config.controls.takeover;
                        // Layer 1: convert HID-named tech/ownership masks to
                        // the native ButtonTrace. Layer 2 below handles native
                        // D-pad/ledge masks and the final left-stick position.
                        let assisted = state
                            .overlay
                            .get_or_insert_with(MapperButtonOverlay::default)
                            .apply(physical, requested, owned);
                        // Read the trigger from PHYSICAL buttons, then consume
                        // its taunt bits. Releasing returns the fresh physical
                        // stick immediately, rather than forcing neutral.
                        let tick = state.raw_tick;
                        let can_move = fresh && state.turbo_ready;
                        let mut turbo_output = state
                            .turbo
                            .get_or_insert_with(TurboState::default)
                            .sample(config.turbo, tick, physical.held, manual_scope, can_move);
                        // Reserve remapped D-pad directions for the entire valid
                        // gameplay scope, independently of the 35ms request TTL.
                        let enabled_dpad = if manual_scope {
                            (if config.turbo.enabled { DPAD_SIDES } else { 0 })
                                | (if config.auto_ledge_trump {
                                    DPAD_DOWN
                                } else {
                                    0
                                })
                        } else {
                            0
                        };
                        reserved_dpad = state
                            .dpad_reservation
                            .get_or_insert_with(DpadReservation::default)
                            .sample(enabled_dpad, physical.held, manual_scope);
                        // Latched L3 mode takes priority over side-D-pad movement.
                        // Stale observations still release automation ownership.
                        let mut ledge = if fresh && state.ledge_active {
                            state.ledge_output
                        } else {
                            LedgeOutput::default()
                        };
                        if config.ledge.enabled && manual_scope && state.ledge_active {
                            turbo_output.stick = ledge.stick;
                        }
                        // A tapped Down owns its run-off sequence until catch;
                        // it outranks L3/turbo but still yields to mash and tech.
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
                        // Mash wins over both D-pad tools while trapped. Its
                        // separate None-on-exit returns the fresh physical stick.
                        if mash_stick.is_some() {
                            turbo_output.stick = mash_stick;
                        }
                        // A held C-stick can otherwise inject an attack while
                        // left-stick-only mashing. Block its digital aliases too.
                        let blocked_stick_buttons = if mash_takeover {
                            kirby_assist_core::manual_controls::RIGHT_STICK_DIRECTIONS
                                | kirby_assist_core::manual_controls::STICK_CLICKS
                        } else {
                            0
                        };
                        // All manual tools share ONE final edge history. When
                        // adding a tool, compose its requested/owned masks here
                        // and decide its stick priority explicitly; do not write
                        // another independent A/R/ZR history after this point.
                        let manual = state
                            .manual_overlay
                            .get_or_insert_with(ManualOverlay::default)
                            .apply_controls(
                                assisted,
                                physical_stick,
                                turbo_output,
                                reserved_dpad
                                    | if config.ledge.enabled && manual_scope {
                                        LEDGE_TRIGGER
                                    } else {
                                        0
                                    },
                                ledge.requested,
                                ledge.owned | blocked_stick_buttons,
                            );
                        let modified = manual.buttons;
                        #[cfg(feature = "diagnostic-logging")]
                        if ledge.requested & modified.pressed != 0 {
                            ledge_edge = Some(ledge.requested);
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
    if let Some(requested) = ledge_edge {
        let mapped = &*output.cast::<MappedInputs>();
        messages.push(format!(
            "ledge mapper: raw_request={requested:#x}, mapped_buttons={:#x}, left=({},{})",
            mapped.buttons, mapped.left_x, mapped.left_y
        ));
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
        log!(&message);
    }
}

// Exported ControlModule symbol, ABI checked against pinned skyline-smash.
// Category 0 is FIGHTER_PAD_COMMAND_CATEGORY1. This observes game state only:
// it returns the game's flags unchanged and never ORs command/action flags.
#[skyline::hook(replace = game_command_flags)]
unsafe fn observe_commands(object: *mut game::Object, category: i32) -> i32 {
    let flags = original!()(object, category);
    if !gameplay_ready() {
        end_session();
        return flags;
    }
    if category == 0 && !object.is_null() {
        observe_fighter(object);
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
unsafe fn observe_fighter(object: *mut game::Object) {
    let Some(config) = CONFIG.get().copied() else {
        return;
    };
    if !config.controls.enabled || !gameplay_ready() {
        end_session();
        return;
    }
    if object.is_null() {
        return;
    }
    // BattleObjectModuleAccessor category is encoded in the object's ID.
    let object_id = object.cast::<u8>().add(8).cast::<u32>().read();
    if object_id >> 28 != 0 || !game::is_active(object_id) {
        return;
    }
    // Validate the current accessor before walking its nested control pointers.
    // A cached ID alone is not proof that this object still owns those modules.
    if game::module_accessor(object_id) != object {
        return;
    }
    // ControlModule +0x118 -> +0x58 -> +8 -> index at +8.
    // A direct ControlModule+8 read does not resolve the controller index.
    let Some(index) = controller_index(object.cast()) else {
        #[cfg(feature = "diagnostic-logging")]
        if !ROUTE_UNAVAILABLE_LOGGED.swap(true, Ordering::Relaxed) {
            log!("fighter controller ownership chain unavailable; waiting for a valid local fighter input index");
        }
        return;
    };
    let (tick, target, find_trump_edge, find_trap_edge) = {
        let Ok(state) = STATE.lock() else {
            return;
        };
        let down = state.raw_buttons & DPAD_DOWN != 0;
        (
            state.raw_tick,
            state.mapping_index,
            config.auto_ledge_trump
                && state
                    .trump
                    .as_ref()
                    .is_none_or(|trump| trump.needs_edge(down))
                && down,
            config.ledge.enabled
                && state.ledge_active
                && state
                    .ledge_seek_tick
                    .is_none_or(|last| state.raw_tick.wrapping_sub(last) >= 30)
                && state
                    .ledge_position
                    .as_ref()
                    .is_none_or(|p| p.spot().is_none()),
        )
    };
    // Exclude CPU actors before reporting associations; multiple CPUs otherwise
    // produce misleading repeated routing messages for the human's controller.
    if !human_fighter(object) {
        if target == Some(index) {
            if let Ok(mut state) = STATE.lock() {
                state.cancel();
                state.associated_object = None;
            }
        }
        return;
    }
    if target != Some(index) {
        #[cfg(feature = "diagnostic-logging")]
        if ROUTE_MISMATCHES.fetch_or(1 << index, Ordering::Relaxed) & (1 << index) == 0 {
            log!(&format!("ignoring other human fighter input index {index}; selected mapper is {target:?}; check npad_id if this is your fighter"));
        }
        return;
    }
    if game::fighter_kind(object) != 6 {
        #[cfg(feature = "diagnostic-logging")]
        if !UNSUPPORTED_FIGHTER_LOGGED.swap(true, Ordering::Relaxed) {
            log!("selected controller is not Kirby; this build's fighter scope is Kirby only (independent of game mode)");
        }
        if let Ok(mut state) = STATE.lock() {
            state.cancel();
            state.associated_object = None;
        }
        return;
    }
    if let Ok(mut state) = STATE.lock() {
        state.associated_object = Some(object_id);
        if state.evaluated_tick == Some(tick) && state.observed_object == Some(object_id) {
            return;
        }
    }
    if !config.controls.enabled || !game::is_active(object_id) {
        if let Ok(mut state) = STATE.lock() {
            state.cancel();
        }
        return;
    }
    // Hitstop freezes native action progress. It differs from cancellation:
    // keep remembered cooldowns/plans while withholding new mash/tech requests.
    let stopped = game::is_stop(object);
    let clatter = !stopped && game::clatter_time(object, 0) > 0.0;
    // Prediction is evaluated separately from the scheduler: finding contact
    // does not mean the cooldown is ready or that the game will accept a tech.
    let assessment = assess_tech(object, config.lookahead, stopped);
    let tech = !clatter && assessment.reason == "collision_predicted";
    if matches!(assessment.status, 0xb5 | 0xb6 | 0x1d6 | 0x1d9..=0x1db) {
        if let Ok(mut state) = STATE.lock() {
            state.cancel();
        }
        return;
    }

    // Read native data outside STATE's mutex to avoid re-entrant deadlocks.
    let ledge_motion_frame = if config.ledge.enabled || config.auto_ledge_trump {
        game::motion_frame(object)
    } else {
        0.0
    };
    let ledge_observation = if config.ledge.enabled || config.auto_ledge_trump {
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
        let context = LedgeContext {
            held: false,
            gameplay: true,
            can_act: false,
            can_control: false,
            stopped,
            status: assessment.status,
            situation: assessment.situation,
            motion_frame: ledge_motion_frame,
            position,
            velocity,
            facing: game::facing(object),
            previous_status: game::previous_status(object, 0),
            can_grab: game::transition_enabled(object, grab_transition_term(assessment.status)),
            shield_stun_frames,
            shield_attacker,
            hit_confirmed: (matches!(assessment.status, 0x2a..=0x2c) || assessment.status == 0x33)
                && game::attack_infliction(object, game::COLLISION_KIND_MASK_HIT),
            safe_ground: if assessment.situation == 0 {
                ground_support(object, position, velocity[0])
            } else {
                [false, false]
            },
        };
        // The trump uses Kirby's current floor geometry, independently of the
        // opponent roster or L3 acquisition range. Query native rays outside STATE.
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
            nearest_ledge_opponent(object, object_id),
            edge,
            spot,
        ))
    } else {
        None
    };
    #[cfg(feature = "diagnostic-logging")]
    let mut scenario_changed = false;
    #[cfg(feature = "diagnostic-logging")]
    let mut acquired = false;
    #[cfg(feature = "diagnostic-logging")]
    let mut first_button_write = false;
    if let Ok(mut state) = STATE.lock() {
        // Native queries above could re-enter a hook. Recheck the route/tick
        // before publishing so an older observation cannot overwrite a newer one.
        if state.mapping_index != Some(index) || state.raw_tick != tick {
            return;
        }
        let now = Instant::now();
        if !state.raw_connected
            || state
                .raw_seen
                .is_none_or(|seen| now.duration_since(seen) >= REQUEST_LIFETIME)
            || state.raw_buttons & MAPPER_MENU != 0
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
                || (0x67..=0x6b).contains(&assessment.status);
            let key = (assessment.status, assessment.situation, assessment.reason);
            if relevant && state.last_tech_diagnostic != Some(key) {
                state.last_tech_diagnostic = Some(key);
                state.pending_tech_log = Some(format!("tech observation: input_update={tick}, status={:#x}, situation={}, reason={}, velocity=({:.2},{:.2}), lookahead={:.1}", assessment.status, assessment.situation, assessment.reason, assessment.velocity[0], assessment.velocity[1], config.lookahead));
            } else if !relevant {
                state.last_tech_diagnostic = None;
            }
        }
        // During hitstop, publish neutral assist without resetting the policy's
        // cooldown/history. Mapping polls are not advancing the damage clock.
        let requested = if stopped {
            0
        } else {
            state
                .policy
                .get_or_insert_with(RawAutoControls::default)
                .update_with_buttons(
                    config.controls,
                    tick,
                    object_id,
                    clatter,
                    tech,
                    previous_buttons,
                )
        };
        #[cfg(feature = "diagnostic-logging")]
        if config.controls.tech_debug {
            let cooldown = state
                .policy
                .as_ref()
                .map_or(0, RawAutoControls::tech_cooldown_remaining);
            if let Some(message) = state.pending_tech_log.as_mut() {
                message.push_str(&format!(", requested={requested:#x}, cooldown={cooldown}"));
            }
        }
        state.turbo_ready = !clatter
            && !tech
            && turbo_movement_allowed(assessment.status, assessment.situation, stopped);
        if let Some((mut context, opponent, edge, spot)) = ledge_observation {
            context.can_act = state.turbo_ready;
            let down = state.raw_buttons & DPAD_DOWN != 0;
            state.trump_output = state.trump.get_or_insert_with(LedgeTrump::default).sample(
                config.auto_ledge_trump,
                tick,
                down,
                LedgeContext {
                    gameplay: !clatter && !tech,
                    ..context
                },
                edge,
            );
            let held = state.ledge_active;
            // This assist stays grounded. Keep ownership through its tilt,
            // shield, and grab animations; mash/tech keep their existing priority.
            // The policy can preserve ZR during native guard hitstop.
            let grounded = assessment.situation == 0
                && (state.turbo_ready
                    || matches!(assessment.status, 0x1b..=0x1e | 0x2a..=0x2c | 0x37..=0x3d));
            let can_control = !stopped && !clatter && !tech && grounded;
            // can_act permits a new grounded action. can_control also covers
            // our ongoing shield/tilt/grab, allowing ownership without repeatedly
            // launching actions during recovery. Do not collapse these flags.
            context.held = held && state.trump_output.owned == 0;
            context.can_act = state.turbo_ready;
            context.can_control = can_control;
            if context.situation != 0 {
                state.ledge_position = None;
                state.ledge_seek_tick = None;
            }
            if held && context.situation == 0 {
                if find_trap_edge && state.turbo_ready && !stopped && !clatter && !tech {
                    state.ledge_seek_tick = Some(tick);
                }
                state
                    .ledge_position
                    .get_or_insert_with(LedgePosition::default)
                    .lock(spot);
                if let Some(spot) = state.ledge_position.as_ref().and_then(LedgePosition::spot) {
                    state
                        .ledge
                        .get_or_insert_with(LedgeTrap::default)
                        .prepare_stance(spot.edge, spot.inward);
                }
            }
            state.ledge_output = state.ledge.get_or_insert_with(LedgeTrap::default).sample(
                config.ledge,
                tick,
                context,
                opponent,
            );
            // Keep observing options during approach. An actionable opponent
            // response takes priority immediately; hanging/no-target uses the
            // locked, damped preparation controller instead of root chasing.
            #[cfg(feature = "diagnostic-logging")]
            let mut positioning_action = None;
            if held && context.situation == 0 && state.trump_output.owned == 0 && !clatter && !tech
            {
                let preparing = state.ledge_output.owned == 0
                    || state.ledge.as_ref().is_some_and(|trap| {
                        matches!(trap.action_name(), "finished" | "return_to_ledge")
                    })
                    || opponent.is_some_and(|e| (CLIFF_CATCH_MOVE..=0x77).contains(&e.status));
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
            if held {
                let key = LedgeDiagnostic {
                    target: opponent.map(|enemy| enemy.id),
                    enemy_status: opponent.map_or(-1, |enemy| enemy.status),
                    vulnerable: opponent.is_some_and(|enemy| enemy.vulnerable),
                    kirby_status: assessment.status,
                    can_act: context.can_act,
                    can_grab: context.can_grab,
                    expected_grab_status: state
                        .ledge
                        .as_ref()
                        .and_then(LedgeTrap::expected_grab_status),
                    hit_confirmed: context.hit_confirmed,
                    enemy_frame: if config.controls.tech_debug {
                        opponent
                            .and_then(|enemy| enemy.motion)
                            .map_or(-1, |motion| motion.frame.floor() as i32)
                    } else {
                        -1
                    },
                    requested: state.ledge_output.requested,
                    action: positioning_action.unwrap_or_else(|| {
                        state.ledge.as_ref().map_or("none", LedgeTrap::action_name)
                    }),
                };
                if state.last_ledge_diagnostic != Some(key) {
                    state.last_ledge_diagnostic = Some(key);
                    state.pending_ledge_log = Some(format!("ledge observation: input_update={tick}, target={:?}, enemy_status={:#x}, vulnerable={}, kirby_status={:#x}, can_act={}, requested={:#x}, action={}, situation={}, motion_frame={:.2}, previous_status={:#x}, can_grab={}, hit_confirmed={}, shield_stun={:?}, safe_ground={:?}, position={:?}, target_position={:?}, target_vulnerable_in={:?}, target_previous_status={:?}, target_motion={:?}, target_observed_velocity={:?}, getup_attack={:?}, expected_grab_status={:?}, locked_ledge={:?}, stance_distance={}, roll_interception={:?}", key.target, key.enemy_status, key.vulnerable, key.kirby_status, key.can_act, key.requested, key.action, assessment.situation, ledge_motion_frame, context.previous_status, context.can_grab, context.hit_confirmed, context.shield_stun_frames, context.safe_ground, context.position, opponent.map(|enemy|enemy.position), opponent.and_then(|enemy|enemy.vulnerable_in), opponent.map(|enemy| enemy.previous_status), opponent.and_then(|enemy| enemy.motion), state.ledge.as_ref().map(LedgeTrap::observed_target_velocity), config.ledge.getup_attack, key.expected_grab_status, state.ledge_position.as_ref().and_then(LedgePosition::spot), config.ledge.stand_distance, state.ledge.as_ref().and_then(LedgeTrap::roll_interception)));
                }
            } else {
                state.last_ledge_diagnostic = None;
                state.pending_ledge_log = None;
            }
        } else {
            state.ledge_output = LedgeOutput::default();
            state.trump_output = LedgeOutput::default();
        }
        state.requested = requested;
        state.mash_stick = if stopped {
            None
        } else {
            state.policy.as_ref().and_then(RawAutoControls::mash_stick)
        };
        // Mash takeover suppresses competing physical action buttons; it never
        // presses them. Tech still owns only ZR, and neutral owns no buttons.
        state.owned = if config.controls.takeover && !stopped {
            if state.mash_stick.is_some() {
                ASSIST_BUTTONS
            } else if tech && config.controls.auto_tech {
                HID_ZR
            } else {
                0
            }
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
        log!("raw Pro Controller mapper button overlay confirmed; game acceptance still depends on its input rules");
    }
    #[cfg(feature = "diagnostic-logging")]
    if acquired {
        log!(&format!(
            "observing human Kirby object {object_id:#x}, input index {index}"
        ));
    }
    #[cfg(feature = "diagnostic-logging")]
    if scenario_changed {
        log!(&format!(
            "scenario: mashable={clatter}, predicted_tech_collision={tech}"
        ));
    }
}

// Sample support along each possible ground move. Probe a short swept
// lookahead, including current momentum, to stop walking across a visible edge.
// This is intentionally conservative on sloped/moving/passable platforms.
unsafe fn ground_support(object: *mut game::Object, position: [f32; 2], vx: f32) -> [bool; 2] {
    let distance = if vx.is_finite() {
        (vx.abs() * 4.0 + 4.0).clamp(4.0, 16.0)
    } else {
        16.0
    };
    [-1.0, 1.0].map(|direction| {
        [1.5, distance].iter().all(|step| {
            let origin = game::Vector2 {
                x: position[0] + direction * step,
                y: position[1] + 2.0,
            };
            let delta = game::Vector2 { x: 0.0, y: -6.0 };
            game::ray_check(object, &origin, &delta, true) != 0
        })
    })
}

// Read the roster from the manager instead of waiting for opponents to query
// command flags. IDs are resolved each call; no native pointer is retained.
// `Option<[Option<u32>; 8]>` has two levels: None means no usable roster;
// Some(array) can still contain empty slots. Callers use flatten() to visit
// only present fighter IDs. IDs must be resolved to fresh accessors each time.
unsafe fn fighter_ids() -> Option<[Option<u32>; 8]> {
    let slot = MANAGER_SLOT.load(Ordering::Relaxed);
    if slot == 0 {
        return None;
    }
    let manager = (slot as *const *mut game::Object).read();
    if manager.is_null() || game::is_result(manager) {
        return None;
    }
    let mut ids = [None; 8];
    for ordinal in 0..game::entry_count(manager).clamp(0, 8) {
        let entry_id = game::entry_id(manager, ordinal);
        if !(0..8).contains(&entry_id) {
            continue;
        }
        let entry = game::fighter_entry(manager, entry_id) as *mut game::Object;
        if entry.is_null() {
            continue;
        }
        let id = game::current_fighter_id(entry) as u32;
        if id >> 28 == 0 && game::is_active(id) {
            ids[ordinal as usize] = Some(id);
        }
    }
    Some(ids)
}

// Startup fallback: discover the selected human Kirby before its first
// command callback. Input index equality alone cannot exclude a CPU fighter.
unsafe fn selected_fighter(index: i32) -> Option<u32> {
    for id in fighter_ids()?.into_iter().flatten() {
        let object = game::module_accessor(id);
        if !object.is_null()
            && game::fighter_kind(object) == 6
            && controller_index(object.cast()) == Some(index)
            && human_fighter(object)
        {
            return Some(id);
        }
    }
    None
}

// Prefer the tracked opponent through its get-up; selection uses the existing
// bounded follow box. All stage preparation queries are ground-only.
unsafe fn nearest_ledge_opponent(object: *mut game::Object, own_id: u32) -> Option<Opponent> {
    let (tracked, locked_spot) = {
        let state = STATE.lock().ok()?;
        (
            state.ledge.as_ref().and_then(LedgeTrap::target),
            state.ledge_position.as_ref().and_then(LedgePosition::spot),
        )
    };
    let position = [game::pos_x(object), game::pos_y(object)];
    let team = game::team(object);
    let mut nearest: Option<(f32, Opponent)> = None;
    for id in fighter_ids()?.into_iter().flatten() {
        if id == own_id {
            continue;
        }
        let enemy = game::module_accessor(id);
        if enemy.is_null() || game::team(enemy) == team {
            continue;
        }
        let status = game::status_kind(enemy);
        let previous_status = game::previous_status(enemy, 0);
        let on_ledge = (CLIFF_CATCH_MOVE..=CLIFF_JUMP3).contains(&status);
        let recent_option = (0x78..=CLIFF_JUMP3).contains(&previous_status)
            || CONFIG.get().is_some_and(|c| c.ledge.drop_defense)
                && game::situation_kind(enemy) == 2
                && (CLIFF_CATCH_MOVE..=0x77).contains(&previous_status);
        if !on_ledge && !recent_option && tracked != Some(id) {
            continue;
        }
        let vulnerable = game::hit_status(enemy, 0) == 0;
        let cliff_position = if status == 0x77 {
            let p = game::cliff_position(enemy);
            (p.x.is_finite()
                && p.y.is_finite()
                && (p.x - game::pos_x(enemy)).abs() <= 12.0
                && (p.y - game::pos_y(enemy)).abs() <= 64.0)
                .then_some([p.x, p.y])
        } else {
            None
        };
        if let (Some(spot), Some(cliff)) = (locked_spot, cliff_position) {
            if (spot.edge[0] - cliff[0]).abs() > 4.0 || (spot.edge[1] - cliff[1]).abs() > 6.0 {
                continue;
            }
        }
        let frame = game::motion_frame(enemy);
        let rate = game::motion_rate(enemy);
        let kind = game::motion_kind(enemy);
        let motion =
            (frame.is_finite() && frame >= 0.0 && kind != 0 && rate.is_finite() && rate > 0.0)
                .then(|| MotionTiming {
                    kind,
                    frame,
                    rate,
                    remaining: frames_until(frame, game::motion_end_frame(enemy), rate),
                });
        // None means unknown, not "vulnerable immediately". Divide animation
        // time by its native rate in frames_until before comparing attack startup.
        let vulnerable_in = if !vulnerable && (0x78..=0x7d).contains(&status) {
            frames_until(frame, game::normal_hit_frame(enemy, kind, false), rate)
        } else {
            None
        };
        let opponent = Opponent {
            id,
            status,
            previous_status,
            situation: game::situation_kind(enemy),
            position: [game::pos_x(enemy), game::pos_y(enemy)],
            vulnerable,
            velocity: [game::speed_x(enemy, -1), game::speed_y(enemy, -1)],
            cliff_position,
            vulnerable_in,
            motion,
            attack_active: (status == 0x78)
                .then(|| (0..8).any(|index| game::attack_active(enemy, index, false))),
        };
        let config = CONFIG.get()?;
        if !in_tracking_range(config.ledge, position, opponent, tracked == Some(id)) {
            continue;
        }
        let dx = opponent.position[0] - position[0];
        let dy = opponent.position[1] - position[1];
        if tracked == Some(id) {
            return Some(opponent);
        }
        // Squared distance ranks candidates without a square root. A tracked
        // target returned above wins, preventing mid-option target switching.
        let distance = dx * dx + dy * dy;
        if nearest.as_ref().is_none_or(|(best, _)| distance < *best) {
            nearest = Some((distance, opponent));
        }
    }
    nearest.map(|(_, opponent)| opponent)
}

// Controller numbers can overlap with CPU defaults. Resolve the actual entry
// and ask the game whether it is CPU-controlled before accepting observations.
unsafe fn human_fighter(object: *mut game::Object) -> bool {
    let slot = MANAGER_SLOT.load(Ordering::Relaxed);
    if slot == 0 {
        return false;
    }
    let manager = (slot as *const *mut game::Object).read();
    if manager.is_null() || game::is_result(manager) {
        return false;
    }
    let id = object.cast::<u8>().add(8).cast::<u32>().read();
    // Resolve an actual entry ID from the object's identity, not the input index.
    let entry = game_entry_id(id);
    if !(0..8).contains(&entry) {
        return false;
    }
    let information = game::fighter_information(manager, entry);
    !information.is_null() && !game::is_cpu(information)
}
extern "C" {
    #[link_name = "\u{1}_ZN3app7fighter20get_fighter_entry_idEj"]
    fn game_entry_id(object_id: u32) -> i32;
}

// Diagnostic snapshot only: the string describes our detector's decision, not
// a complete explanation of all native tech restrictions or proof of success.
struct TechAssessment {
    status: i32,
    situation: i32,
    velocity: [f32; 2],
    reason: &'static str,
}

// Start with native status/eligibility gates, then test swept stage rays. Keep
// the fixed body offsets in tech_prediction.rs separate so they can be tested
// on the host and tuned without scattering geometry through native FFI code.
unsafe fn assess_tech(object: *mut game::Object, horizon: f32, stopped: bool) -> TechAssessment {
    let status = game::status_kind(object);
    let situation = game::situation_kind(object);
    let disabled = (0x49..=0x4f).contains(&status) && game::is_flag(object, 0x21000016);
    let gate = tech_gate(status, situation, disabled, stopped);
    let mut assessment = TechAssessment {
        status,
        situation,
        velocity: [0.0; 2],
        reason: gate.label(),
    };
    if (0x67..=0x6b).contains(&status) {
        assessment.reason = "tech_status_observed";
        return assessment;
    }
    // Read damage velocity for diagnostics even when the native flag blocks tech.
    if (0x47..=0x5b).contains(&status) {
        assessment.velocity = [game::speed_x(object, 1), game::speed_y(object, 1)];
    }
    if gate != TechGate::Predict {
        return assessment;
    }
    let x = game::pos_x(object);
    let y = game::pos_y(object);
    let [vx, vy] = assessment.velocity;
    if ![x, y, vx, vy].iter().all(|value| value.is_finite()) {
        assessment.reason = "invalid_motion";
        return assessment;
    }
    assessment.reason = "no_collision_predicted";
    for probe in tech_probes(x, y, vx, vy, horizon).into_iter().flatten() {
        let origin = game::Vector2 {
            x: probe.origin[0],
            y: probe.origin[1],
        };
        let path = game::Vector2 {
            x: probe.delta[0],
            y: probe.delta[1],
        };
        if game::ray_check(object, &origin, &path, true) != 0 {
            assessment.reason = "collision_predicted";
            break;
        }
    }
    assessment
}

// Startup is deliberately sequential: prepare diagnostics, parse config, locate
// exactly one supported mapper, resolve the manager, THEN install hooks. A failed
// prerequisite exits without installing a partially configured set of hooks.
pub fn install() {
    // Silent builds only READ configuration: no directory/file creation or
    // console output is compiled into this plugin's normal startup path.
    #[cfg(feature = "diagnostic-logging")]
    crate::logging::start();
    let config = match load_config() {
        Ok(config) => config,
        Err(_error) => {
            log!(&format!("NOT INSTALLED: {_error}"));
            return;
        }
    };
    let _ = CONFIG.set(config);
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
        log!(&format!("hooks installed; mapper={offset:#x}; config={config:?}; direct raw mapper output supports configurable mash groups and ZR tech; assistance gated to ready gameplay, bypassing intros/menus/results"));
    }
}
