//! Execute the production teardown and route validator with local native stubs.
//! No Switch calls, hooks, SD/network access or synthetic controller injection.
#![allow(dead_code, unexpected_cfgs)]
use kirby_assist_core::session::MatchSession;
use std::time::{Duration, Instant};
const SESSION_OBSERVATION_GAP: Duration = Duration::from_millis(250);
use std::sync::{
    atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicUsize, Ordering},
    Mutex,
};

static RUNTIME_ENABLED: AtomicBool = AtomicBool::new(true);
static LOCAL_PARTICIPANT: AtomicBool = AtomicBool::new(false);
static MASTER_IN_MATCH: AtomicBool = AtomicBool::new(false);
static TRAINING_PASSTHROUGH: AtomicBool = AtomicBool::new(false);
static TRAINING_MODAL_HANDOFF: AtomicBool = AtomicBool::new(false);
static SERIAL: Mutex<()> = Mutex::new(());
static CLEANUPS: AtomicUsize = AtomicUsize::new(0);
static MENU_ENDS: AtomicUsize = AtomicUsize::new(0);
static TOAST: AtomicBool = AtomicBool::new(false);
struct Menu;
impl Menu {
    fn display_battle(&self, active: bool) {
        assert!(!active);
    }
    fn end_match(&mut self) {
        MENU_ENDS.fetch_add(1, Ordering::Relaxed);
    }
}
struct State {
    session: MatchSession,
    observation_sequence: u64,
    settings_menu: Option<Menu>,
    associated_object: Option<u32>,
    menu_battle_active: bool,
    mapping_index: Option<i32>,
    human_routes: [Option<(u32, Instant)>; 8],
    requested: u32,
    owned: u32,
    pending_track: Option<u32>,
}
impl State {
    fn cancel(&mut self) {
        self.requested = 0;
        self.owned = 0;
        self.pending_track = None;
    }
    const fn new() -> Self {
        Self {
            session: MatchSession::new(),
            observation_sequence: 0,
            settings_menu: None,
            associated_object: None,
            menu_battle_active: false,
            mapping_index: None,
            human_routes: [None; 8],
            requested: 0,
            owned: 0,
            pending_track: None,
        }
    }
}
static STATE: Mutex<State> = Mutex::new(State::new());
mod attack_sources {
    pub fn clear() {
        super::CLEANUPS.fetch_add(1, super::Ordering::Relaxed);
    }
}
mod settings_menu {
    pub fn show_master_status(on: bool) {
        super::TOAST.store(on, super::Ordering::Relaxed);
    }
}
#[cfg(feature = "diagnostic-logging")]
mod performance {
    pub fn set_enabled(_: bool) {}
    pub fn clear() {}
}
#[cfg(feature = "diagnostic-logging")]
mod calibration {
    pub fn clear() {}
}
#[cfg(feature = "diagnostic-logging")]
mod logging {
    pub fn clear_snapshot() {}
}
#[path = "../../../src/live/runtime.rs"]
mod runtime;

fn seed() -> kirby_assist_core::session::SessionTicket {
    RUNTIME_ENABLED.store(true, Ordering::Release);
    LOCAL_PARTICIPANT.store(true, Ordering::Release);
    MASTER_IN_MATCH.store(false, Ordering::Release);
    TRAINING_PASSTHROUGH.store(true, Ordering::Relaxed);
    TRAINING_MODAL_HANDOFF.store(true, Ordering::Relaxed);
    CLEANUPS.store(0, Ordering::Relaxed);
    MENU_ENDS.store(0, Ordering::Relaxed);
    let mut state = STATE.lock().unwrap();
    *state = State::new();
    state.session.update(true);
    state.observation_sequence = 90;
    state.settings_menu = Some(Menu);
    state.associated_object = Some(7);
    state.menu_battle_active = true;
    state.mapping_index = Some(0);
    state.human_routes[0] = Some((7, Instant::now()));
    state.requested = u32::MAX;
    state.owned = u32::MAX;
    state.pending_track = Some(7);
    state.session.ticket().unwrap()
}

fn assert_clean(old: kirby_assist_core::session::SessionTicket) {
    let mut state = STATE.lock().unwrap();
    assert!(!state.session.accepts(old));
    state.session.update(true);
    assert!(
        !state.session.accepts(old),
        "old observations cannot rejoin a fresh session"
    );
    assert!(state.observation_sequence > 90);
    assert!(state.settings_menu.is_none() && state.associated_object.is_none());
    assert!(!state.menu_battle_active && state.mapping_index.is_none());
    assert_eq!(state.human_routes, [None; 8]);
    assert_eq!((state.requested, state.owned), (0, 0));
    assert!(state.pending_track.is_none());
}

#[test]
fn master_off_discards_routes_pending_inputs_tracking_and_old_observation_tickets() {
    let _guard = SERIAL.lock().unwrap();
    let old = seed();
    runtime::set_runtime_enabled(false);
    assert!(!RUNTIME_ENABLED.load(Ordering::Acquire));
    assert!(!runtime::feature_activity_allowed());
    assert!(!TOAST.load(Ordering::Relaxed));
    assert!(!TRAINING_PASSTHROUGH.load(Ordering::Relaxed));
    assert!(!TRAINING_MODAL_HANDOFF.load(Ordering::Relaxed));
    assert_clean(old);
    runtime::set_runtime_enabled(true);
    assert!(RUNTIME_ENABLED.load(Ordering::Acquire));
    assert!(
        !runtime::feature_activity_allowed(),
        "ON still needs fresh local participation"
    );
    assert_eq!(MENU_ENDS.load(Ordering::Relaxed), 1);
}

#[test]
fn lost_participation_suspends_everything_once_without_changing_the_master_preference() {
    let _guard = SERIAL.lock().unwrap();
    let old = seed();
    runtime::suspend_participation();
    assert!(RUNTIME_ENABLED.load(Ordering::Acquire));
    assert!(!runtime::feature_activity_allowed());
    assert_clean(old);
    for _ in 0..600 {
        runtime::suspend_participation();
    }
    assert_eq!(
        CLEANUPS.load(Ordering::Relaxed),
        1,
        "idle spectator polls do not repeat cleanup"
    );
    assert_eq!(MENU_ENDS.load(Ordering::Relaxed), 1);
}

#[test]
fn match_lock_rejects_master_toggles_without_resetting_play_or_input_state() {
    let _guard = SERIAL.lock().unwrap();
    let _ = seed();
    MASTER_IN_MATCH.store(true, Ordering::Release);
    runtime::set_runtime_enabled(false);
    assert!(runtime::feature_activity_allowed());
    assert_eq!(STATE.lock().unwrap().requested, u32::MAX);
    assert_eq!(CLEANUPS.load(Ordering::Relaxed), 0);
    MASTER_IN_MATCH.store(false, Ordering::Release);
}

#[test]
fn startup_and_reconnect_require_fresh_matching_ownership_without_allocating_a_menu() {
    let _guard = SERIAL.lock().unwrap();
    RUNTIME_ENABLED.store(true, Ordering::Release);
    *STATE.lock().unwrap() = State::new();
    assert_eq!(runtime::mapper_candidate(0), None);
    assert_eq!(STATE.lock().unwrap().mapping_index, Some(0));
    assert!(STATE.lock().unwrap().settings_menu.is_none());
    // Native callbacks establish identity; no artificial wait or second press.
    {
        let mut state = STATE.lock().unwrap();
        state.associated_object = Some(7);
        state.human_routes[0] = Some((7, Instant::now()));
    }
    assert_eq!(runtime::mapper_candidate(0), Some(7));
    assert_eq!(
        runtime::mapper_candidate(1),
        None,
        "another controller cannot reuse ownership"
    );
    {
        let mut state = STATE.lock().unwrap();
        state.associated_object = Some(8);
        state.human_routes[1] = Some((8, Instant::now() - Duration::from_secs(1)));
    }
    assert_eq!(
        runtime::mapper_candidate(1),
        None,
        "expired callbacks cannot authorize features"
    );
    RUNTIME_ENABLED.store(false, Ordering::Release);
    assert_eq!(runtime::mapper_candidate(2), None);
    assert_eq!(
        STATE.lock().unwrap().mapping_index,
        Some(1),
        "OFF never changes input ownership"
    );
}

mod game {
    pub type Object = u8;
    pub unsafe fn entry_count(_: *mut Object) -> i32 {
        unreachable!("roster enumeration is outside this test")
    }
    pub unsafe fn entry_id(_: *mut Object, _: i32) -> i32 {
        unreachable!("roster enumeration is outside this test")
    }
    pub unsafe fn fighter_kind(_: u32) -> i32 {
        unreachable!("partner enumeration is outside this test")
    }
    pub unsafe fn work_int(_: *mut Object, _: i32) -> i32 {
        unreachable!("partner enumeration is outside this test")
    }
    pub unsafe fn battle_object_id(object: *mut Object) -> u32 {
        object.add(8).cast::<u32>().read()
    }
    pub unsafe fn is_active(id: u32) -> bool {
        id == super::ACTIVE_ID.load(super::Ordering::Relaxed)
    }
    pub unsafe fn module_accessor(_: u32) -> *mut Object {
        super::ACCESSOR.load(super::Ordering::Relaxed)
    }
    pub unsafe fn fighter_entry(_: *mut Object, _: i32) -> u64 {
        super::ACCESSOR.load(super::Ordering::Relaxed) as u64
    }
    pub unsafe fn current_fighter_id(entry: *mut Object) -> u64 {
        u64::from(battle_object_id(entry))
    }
    pub unsafe fn fighter_information(_: *mut Object, entry: i32) -> *mut Object {
        fighter_entry(std::ptr::null_mut(), entry) as *mut Object
    }
    pub unsafe fn is_cpu(_: *mut Object) -> bool {
        !super::HUMAN.load(super::Ordering::Relaxed)
    }
}
static ACTIVE_ID: AtomicU32 = AtomicU32::new(7);
static ACCESSOR: AtomicPtr<u8> = AtomicPtr::new(std::ptr::null_mut());
static HUMAN: AtomicBool = AtomicBool::new(true);
unsafe fn fighter_manager_instance() -> Option<*mut game::Object> {
    Some(std::ptr::NonNull::dangling().as_ptr())
}
unsafe fn active_fighter_manager() -> Option<*mut game::Object> {
    unreachable!("roster enumeration is outside this test")
}
unsafe fn game_entry_id(id: u32) -> i32 {
    if id == 7 {
        0
    } else {
        -1
    }
}
#[path = "../../../src/live/fighter_roster.rs"]
mod local_fighter;

#[test]
fn route_checks_reject_missing_inactive_recycled_cpu_and_other_controller_fighters() {
    let _guard = SERIAL.lock().unwrap();
    let mut accessor = Box::new([0usize; 16]);
    let mut control = Box::new([0usize; 36]);
    let mut first = Box::new([0usize; 12]);
    let mut second = Box::new([0usize; 2]);
    let mut input = Box::new([0usize; 2]);
    accessor[1] = 7;
    accessor[9] = control.as_mut_ptr() as usize;
    control[35] = first.as_mut_ptr() as usize;
    first[11] = second.as_mut_ptr() as usize;
    second[1] = input.as_mut_ptr() as usize;
    input[1] = 0;
    ACTIVE_ID.store(7, Ordering::Relaxed);
    HUMAN.store(true, Ordering::Relaxed);
    ACCESSOR.store(accessor.as_mut_ptr().cast(), Ordering::Relaxed);
    unsafe {
        assert!(local_fighter::valid_local_route(7, 0));
        assert!(!local_fighter::valid_local_route(7, 1));
        assert!(!local_fighter::valid_local_route(7, -1));
        assert!(!local_fighter::valid_local_route(8, 0));
        HUMAN.store(false, Ordering::Relaxed);
        assert!(!local_fighter::valid_local_route(7, 0));
        HUMAN.store(true, Ordering::Relaxed);
        accessor[1] = 8;
        assert!(!local_fighter::valid_local_route(7, 0));
        accessor[1] = 7;
        second[1] = 0;
        assert!(
            !local_fighter::valid_local_route(7, 0),
            "human metadata without a local input chain is insufficient"
        );
        ACCESSOR.store(std::ptr::null_mut(), Ordering::Relaxed);
        assert!(!local_fighter::valid_local_route(7, 0));
    }
}
