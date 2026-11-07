//! Exercise the production Training handoff with host-only state and menu stubs.
use kirby_assist_core::{
    manual_controls::DPAD_UP,
    native_input::{ButtonTrace, ControlLayout, MAPPER_MINUS, MAPPER_PLUS, MAPPER_ZL},
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::Instant,
};

macro_rules! log {
    ($message:expr $(,)?) => {};
}

static TRAINING_MODAL_HANDOFF: AtomicBool = AtomicBool::new(false);
static TRAINING_PASSTHROUGH: AtomicBool = AtomicBool::new(false);
static SERIAL: Mutex<()> = Mutex::new(());

mod settings_menu {
    use super::*;

    #[derive(Default)]
    pub struct SettingsMenu {
        pub observations: Vec<(u32, ControlLayout)>,
        pub displayed: std::cell::Cell<Option<bool>>,
        pub resumed: usize,
        pub opened: usize,
    }

    impl SettingsMenu {
        pub fn observe_passthrough(&mut self, buttons: u32, layout: ControlLayout) {
            self.observations.push((buttons, layout));
        }
        pub fn display_battle(&self, active: bool) {
            self.displayed.set(Some(active));
        }
        pub fn resume_saved_state(&mut self) {
            self.resumed += 1;
        }
        pub fn request_open(&mut self) {
            self.opened += 1;
        }
    }
}

struct State {
    settings_menu: Option<settings_menu::SettingsMenu>,
    menu_battle_active: bool,
    mapping_index: Option<i32>,
    raw_tick: i64,
    raw_buttons: u32,
    raw_seen: Option<Instant>,
    request_seen: Option<Instant>,
    raw_connected: bool,
    cancellations: usize,
}

impl State {
    const fn new() -> Self {
        Self {
            settings_menu: None,
            menu_battle_active: true,
            mapping_index: None,
            raw_tick: 0,
            raw_buttons: 0,
            raw_seen: None,
            request_seen: None,
            raw_connected: false,
            cancellations: 0,
        }
    }
    fn cancel(&mut self) {
        self.cancellations += 1;
    }
}

static STATE: Mutex<State> = Mutex::new(State::new());

#[path = "../../../src/live/training.rs"]
mod training;

fn reset() {
    TRAINING_MODAL_HANDOFF.store(false, Ordering::Relaxed);
    TRAINING_PASSTHROUGH.store(false, Ordering::Relaxed);
    *STATE.lock().unwrap() = State::new();
}

#[test]
fn ordinary_inputs_do_not_start_or_mutate_a_handoff() {
    let _guard = SERIAL.lock().unwrap();
    reset();
    assert!(!training::enter_passthrough(
        ButtonTrace::default(),
        2,
        ControlLayout::Default
    ));
    let state = STATE.lock().unwrap();
    assert!(state.settings_menu.is_none());
    assert_eq!(state.cancellations, 0);
    assert_eq!(state.raw_tick, 0);
    assert!(!TRAINING_PASSTHROUGH.load(Ordering::Relaxed));
}

#[test]
fn momentary_handoff_cancels_once_refreshes_route_and_resumes_saved_menu() {
    let _guard = SERIAL.lock().unwrap();
    reset();
    let held = MAPPER_ZL | DPAD_UP;
    let physical = ButtonTrace {
        held,
        pressed: held,
        ..Default::default()
    };
    for _ in 0..2 {
        assert!(training::enter_passthrough(
            physical,
            2,
            ControlLayout::Default
        ));
    }
    {
        let state = STATE.lock().unwrap();
        let menu = state.settings_menu.as_ref().unwrap();
        assert_eq!(state.cancellations, 1);
        assert_eq!(state.raw_tick, 2);
        assert_eq!(state.mapping_index, Some(2));
        assert_eq!(state.raw_buttons, held);
        assert!(state.raw_connected);
        assert!(state.raw_seen.is_some());
        assert_eq!(state.request_seen, state.raw_seen);
        assert_eq!(menu.observations.len(), 2);
        assert_eq!(menu.observations[0].0, held);
        assert_eq!(menu.displayed.get(), Some(true));
    }
    assert!(!TRAINING_MODAL_HANDOFF.load(Ordering::Relaxed));
    training::finish_passthrough(true);
    training::finish_passthrough(true);
    let state = STATE.lock().unwrap();
    let menu = state.settings_menu.as_ref().unwrap();
    assert_eq!((menu.resumed, menu.opened), (1, 0));
    assert!(!TRAINING_PASSTHROUGH.load(Ordering::Relaxed));
}

#[test]
fn modal_handoff_hides_menu_until_minus_then_reopens_it() {
    let _guard = SERIAL.lock().unwrap();
    reset();
    let plus = ButtonTrace {
        held: MAPPER_PLUS,
        pressed: MAPPER_PLUS,
        ..Default::default()
    };
    assert!(training::enter_passthrough(plus, 0, ControlLayout::Default));
    assert!(TRAINING_MODAL_HANDOFF.load(Ordering::Relaxed));
    assert!(training::enter_passthrough(
        ButtonTrace::default(),
        0,
        ControlLayout::Default
    ));
    let minus = ButtonTrace {
        held: MAPPER_MINUS,
        pressed: MAPPER_MINUS,
        ..Default::default()
    };
    assert!(!training::enter_passthrough(
        minus,
        0,
        ControlLayout::Default
    ));
    assert_eq!(
        STATE
            .lock()
            .unwrap()
            .settings_menu
            .as_ref()
            .unwrap()
            .displayed
            .get(),
        Some(false)
    );
    training::finish_passthrough(true);
    let state = STATE.lock().unwrap();
    let menu = state.settings_menu.as_ref().unwrap();
    assert_eq!((menu.resumed, menu.opened), (0, 1));
    assert!(!TRAINING_MODAL_HANDOFF.load(Ordering::Relaxed));
}

#[test]
fn leaving_training_clears_handoff_without_resuming_or_opening_menu() {
    let _guard = SERIAL.lock().unwrap();
    reset();
    assert!(training::enter_passthrough(
        ButtonTrace {
            held: MAPPER_PLUS,
            ..Default::default()
        },
        0,
        ControlLayout::Default
    ));
    training::finish_passthrough(false);
    let state = STATE.lock().unwrap();
    let menu = state.settings_menu.as_ref().unwrap();
    assert_eq!((menu.resumed, menu.opened), (0, 0));
    assert!(!TRAINING_PASSTHROUGH.load(Ordering::Relaxed));
    assert!(!TRAINING_MODAL_HANDOFF.load(Ordering::Relaxed));
}
